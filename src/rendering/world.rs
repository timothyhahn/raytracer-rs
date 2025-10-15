use crate::core::color::Color;
use crate::core::matrices::Matrix4;
use crate::core::tuples::{Point, Tuple};
use crate::geometry::sphere::Sphere;
use crate::rendering::intersections::{Computations, Intersection};
use crate::rendering::objects::{HasMaterial, Intersectable, Object};
use crate::rendering::rays::Ray;
use crate::scene::lights::PointLight;
use crate::scene::materials::Material;

const DEFAULT_MAX_REFLECTION_DEPTH: u32 = 5;
const DEFAULT_MAX_REFRACTION_DEPTH: u32 = 5;

pub struct World {
    pub objects: Vec<Object>,
    pub light_source: Option<PointLight>,
}

impl World {
    pub fn new() -> World {
        World {
            objects: Vec::new(),
            light_source: None,
        }
    }

    pub fn intersect(&self, ray: Ray) -> Vec<Intersection<'_>> {
        let mut intersections: Vec<Intersection> = Vec::with_capacity(self.objects.len() * 2);
        for object in self.objects.iter() {
            intersections.extend(
                object
                    .intersect(ray)
                    .iter()
                    .map(|&t| Intersection { object, t }),
            );
        }
        // Sort by t value. NaN values (shouldn't happen) are treated as greater than any number
        intersections.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
        intersections
    }

    pub fn shade_hit(&self, comps: Computations) -> Color {
        self.shade_hit_internal(&comps, DEFAULT_MAX_REFLECTION_DEPTH)
    }

    fn shade_hit_internal(&self, comps: &Computations, remaining: u32) -> Color {
        // If there's no light source, return black (no illumination)
        let light = match self.light_source {
            Some(light) => light,
            None => return Color::black(),
        };

        let in_shadow = self.is_shadowed(comps.over_point);
        let surface = comps.object.material().lighting(
            &comps.object,
            light,
            comps.point,
            comps.eye_vector,
            comps.normal_vector,
            in_shadow,
        );

        let reflected_color = self.reflected_color_internal(comps, remaining);
        let refracted_color = self.refracted_color_internal(comps, remaining);
        let material = comps.object.material();
        // If material is reflective and transparent, use Schlick's approximation to calculate reflectance
        if material.reflectivity > 0.0 && material.transparency > 0.0 {
            let reflectance = comps.schlick();
            return surface + reflected_color * reflectance + refracted_color * (1.0 - reflectance);
        }
        // Otherwise, just add the reflected and refracted colors to the surface color
        surface + reflected_color + refracted_color
    }

    pub fn color_at(&self, ray: Ray) -> Color {
        self.color_at_internal(ray, DEFAULT_MAX_REFLECTION_DEPTH)
    }

    fn color_at_internal(&self, ray: Ray, remaining: u32) -> Color {
        let intersections = self.intersect(ray);
        let hit = Intersection::hit(intersections);
        match hit {
            Some(hit) => {
                let comps = hit.prepare_computations(ray);
                self.shade_hit_internal(&comps, remaining)
            }
            None => Color::black(),
        }
    }

    pub fn is_shadowed(&self, point: Point) -> bool {
        // If there's no light source, there's no shadow
        let light = match self.light_source {
            Some(light) => light,
            None => return false,
        };

        // Measure the distance from point to the light source
        let v = light.position - point;
        let distance = v.magnitude();
        let direction = v.normalize();

        // Create a ray from point toward the light source, then intersect the world
        let ray = Ray::new(point, direction);
        let intersections = self.intersect(ray);

        // See if there was a hit and if so, whether t is less than distance.
        let hit = Intersection::hit(intersections);
        match hit {
            Some(hit) => hit.t < distance,
            None => false,
        }
    }

    pub fn reflected_color(&self, comps: Computations) -> Color {
        self.reflected_color_internal(&comps, DEFAULT_MAX_REFLECTION_DEPTH)
    }

    fn reflected_color_internal(&self, comps: &Computations, remaining: u32) -> Color {
        if remaining == 0 {
            return Color::BLACK;
        }

        if comps.object.material().reflectivity == 0.0 {
            return Color::BLACK;
        }

        let reflect_ray = Ray::new(comps.over_point, comps.reflect_vector);
        let color = self.color_at_internal(reflect_ray, remaining - 1);

        color * comps.object.material().reflectivity
    }

    pub fn refracted_color(&self, comps: Computations) -> Color {
        self.refracted_color_internal(&comps, DEFAULT_MAX_REFRACTION_DEPTH)
    }

    fn refracted_color_internal(&self, comps: &Computations, remaining: u32) -> Color {
        if remaining == 0 {
            return Color::BLACK;
        }

        if comps.object.material().transparency == 0.0 {
            return Color::BLACK;
        }

        let n_ratio = comps.n1 / comps.n2;
        let cos_i = comps.eye_vector.dot(&comps.normal_vector);
        let sin2_t = n_ratio.powi(2) * (1.0 - cos_i.powi(2));

        if sin2_t > 1.0 {
            return Color::BLACK;
        }

        let cos_t = (1.0 - sin2_t).sqrt();
        let direction =
            comps.normal_vector * (n_ratio * cos_i - cos_t) - comps.eye_vector * n_ratio;
        let refract_ray = Ray::new(comps.under_point, direction);
        let color = self.color_at_internal(refract_ray, remaining - 1);

        color * comps.object.material().transparency
    }
}

impl Default for World {
    fn default() -> Self {
        let material = Material {
            color: Color::new(0.8, 1.0, 0.6),
            diffuse: 0.7,
            specular: 0.2,
            ..Default::default()
        };
        let sphere1 = Sphere {
            material,
            ..Default::default()
        };
        let sphere2 = Sphere {
            transformation: Matrix4::scale(0.5, 0.5, 0.5),
            ..Default::default()
        };

        let objects: Vec<Object> = Vec::from([Object::Sphere(sphere1), Object::Sphere(sphere2)]);

        World {
            objects,
            light_source: Some(PointLight::new(
                Point::new(-10.0, 10.0, -10.0),
                Color::new(1.0, 1.0, 1.0),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::color::Color;
    use crate::core::floats::float_equal;
    use crate::core::matrices::Matrix4;
    use crate::core::tuples::{Point, Tuple, Vector};
    use crate::geometry::planes::Plane;
    use crate::geometry::sphere::Sphere;
    use crate::rendering::intersections::Intersection;
    use crate::rendering::objects::{HasMaterial, Intersectable, Object, Transformable};
    use crate::rendering::rays::Ray;
    use crate::rendering::world::World;
    use crate::scene::lights::PointLight;
    use crate::scene::materials::Material;
    use crate::scene::patterns::Pattern;

    #[test]
    fn empty_world() {
        let world = World::new();
        assert_eq!(world.objects.len(), 0);
        assert!(world.light_source.is_none());
    }

    #[test]
    fn default_world() {
        let world = World::default();
        assert_eq!(world.objects.len(), 2);
        assert!(world.light_source.is_some());
    }

    #[test]
    fn intersect_world_with_ray() {
        let world = World::default();
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let intersections = world.intersect(ray);
        assert_eq!(intersections.len(), 4);
        assert_eq!(intersections[0].t, 4.0);
        assert_eq!(intersections[1].t, 4.5);
        assert_eq!(intersections[2].t, 5.5);
        assert_eq!(intersections[3].t, 6.0);
    }

    #[test]
    fn shading_intersection() {
        let world = World::default();
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let shape = &world.objects[0];
        let intersection = Intersection {
            object: shape,
            t: shape.intersect(ray)[0],
        };
        let computations = intersection.prepare_computations(ray);
        let color = world.shade_hit(computations);
        assert_eq!(color, Color::new(0.38066, 0.47583, 0.2855));
    }

    #[test]
    fn shading_intersection_from_inside() {
        let world = World {
            light_source: Some(PointLight::new(
                Point::new(0.0, 0.25, 0.0),
                Color::new(1.0, 1.0, 1.0),
            )),
            ..Default::default()
        };
        let ray = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0));
        let shape = &world.objects[1];
        let intersection = Intersection::new(0.5, shape);
        let computations = intersection.prepare_computations(ray);
        let color = world.shade_hit(computations);
        assert_eq!(color, Color::new(0.90498, 0.90498, 0.90498));
    }

    #[test]
    fn color_when_ray_misses() {
        let world = World::default();
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 1.0, 0.0));
        let color = world.color_at(ray);
        assert_eq!(color, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn color_when_ray_hits() {
        let world = World::default();
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let color = world.color_at(ray);
        assert_eq!(color, Color::new(0.38066, 0.47583, 0.2855));
    }

    #[test]
    fn color_with_intersection_behind_ray() {
        let world = World::default();
        let mut objects = world.objects;

        let outer_material = objects[0].clone().material();
        objects[0].set_material(Material {
            ambient: 1.0,
            ..outer_material
        });

        let inner_material = objects[1].clone().material();
        objects[1].set_material(Material {
            ambient: 1.0,
            ..inner_material
        });
        let world = World {
            objects,
            ..Default::default()
        };
        let ray = Ray::new(Point::new(0.0, 0.0, 0.75), Vector::new(0.0, 0.0, -1.0));
        let color = world.color_at(ray);
        assert_eq!(color, inner_material.color);
    }

    #[test]
    fn there_is_no_shadow_when_nothing_is_collinear_with_point_and_light() {
        let world = World::default();
        let point = Point::new(0.0, 10.0, 0.0);
        assert!(!world.is_shadowed(point));
    }

    #[test]
    fn shadow_when_an_object_is_between_point_and_light() {
        let world = World::default();
        let point = Point::new(10.0, -10.0, 10.0);
        assert!(world.is_shadowed(point));
    }

    #[test]
    fn there_is_no_shadow_when_an_object_is_behind_the_light() {
        let world = World::default();
        let point = Point::new(-20.0, 20.0, -20.0);
        assert!(!world.is_shadowed(point));
    }

    #[test]
    fn there_is_no_shadow_when_an_object_is_behind_the_point() {
        let world = World::default();
        let point = Point::new(-2.0, 2.0, -2.0);
        assert!(!world.is_shadowed(point));
    }

    #[test]
    fn shade_hit_is_given_an_intersection_in_shadow() {
        let s1 = Sphere::new();
        let s2 = Sphere::new();
        let mut obj2 = Object::Sphere(s2);
        obj2.set_transform(Matrix4::translate(0.0, 0.0, 10.0));

        let world = World {
            light_source: Some(PointLight::new(Point::new(0.0, 0.0, -10.0), Color::white())),
            objects: vec![Object::Sphere(s1), obj2],
        };

        let ray = Ray::new(Point::new(0.0, 0.0, 5.0), Vector::new(0.0, 0.0, 1.0));
        let intersection = Intersection::new(4.0, &world.objects[1]);
        let computations = intersection.prepare_computations(ray);
        let color = world.shade_hit(computations);
        assert_eq!(color, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn reflected_color_for_nonreflective_material() {
        let world = World::default();
        let ray = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0));
        let shape = Object::plane();
        let intersection = Intersection::new(1.0, &shape);
        let computations = intersection.prepare_computations(ray);
        let color = world.reflected_color(computations);
        assert_eq!(color, Color::BLACK);
    }

    #[test]
    fn reflected_color_for_reflective_material() {
        let mut world = World::default();

        let mut shape = Object::plane();
        shape.set_material(Material {
            reflectivity: 0.5,
            ..Default::default()
        });
        shape.set_transform(Matrix4::translate(0.0, -1.0, 0.0));

        world.objects.push(shape);

        let ray = Ray::new(
            Point::new(0.0, 0.0, -3.0),
            Vector::new(0.0, -2_f64.sqrt() / 2.0, 2_f64.sqrt() / 2.0),
        );

        let intersection = Intersection::new(2_f64.sqrt(), &world.objects[2]);
        let computations = intersection.prepare_computations(ray);
        let color = world.reflected_color(computations);

        let expected = Color::new(0.19033, 0.23792, 0.14275);
        assert!(
            float_equal(color.red, expected.red)
                && float_equal(color.green, expected.green)
                && float_equal(color.blue, expected.blue),
            "Expected color {:?}, got {:?}",
            expected,
            color
        );
    }

    #[test]
    fn shade_hit_with_reflective_material() {
        let mut world = World::default();

        let mut shape = Object::plane();
        shape.set_material(Material {
            reflectivity: 0.5,
            ..Default::default()
        });
        shape.set_transform(Matrix4::translate(0.0, -1.0, 0.0));

        world.objects.push(shape);

        let ray = Ray::new(
            Point::new(0.0, 0.0, -3.0),
            Vector::new(0.0, -2_f64.sqrt() / 2.0, 2_f64.sqrt() / 2.0),
        );

        let intersection = Intersection::new(2_f64.sqrt(), &world.objects[2]);
        let computations = intersection.prepare_computations(ray);
        let color = world.shade_hit(computations);

        let expected = Color::new(0.87676, 0.92434, 0.82917);
        assert!(
            float_equal(color.red, expected.red)
                && float_equal(color.green, expected.green)
                && float_equal(color.blue, expected.blue),
            "Expected color {:?}, got {:?}",
            expected,
            color
        );
    }

    // AKA the infinite recursion test
    #[test]
    fn color_at_with_mutually_reflective_material() {
        let mut world = World::new();
        world.light_source = Some(PointLight::new(Point::new(0.0, 0.0, 0.0), Color::white()));

        let mut lower = Object::plane();
        lower.set_material(Material {
            reflectivity: 1.0,
            ..Default::default()
        });
        lower.set_transform(Matrix4::translate(0.0, -1.0, 0.0));
        world.objects.push(lower);

        let mut upper = Object::plane();
        upper.set_material(Material {
            reflectivity: 1.0,
            ..Default::default()
        });
        upper.set_transform(Matrix4::translate(0.0, 1.0, 0.0));
        world.objects.push(upper);

        let ray = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 1.0, 0.0));

        // Should terminate successfully without stack overflow
        let _color = world.color_at(ray);
        // If we get here, the test passed (no stack overflow)
    }

    #[test]
    fn reflected_color_at_maximum_recursion_depth() {
        let mut world = World::default();

        let mut shape = Object::plane();
        shape.set_material(Material {
            reflectivity: 0.5,
            ..Default::default()
        });
        shape.set_transform(Matrix4::translate(0.0, -1.0, 0.0));
        world.objects.push(shape);

        let ray = Ray::new(
            Point::new(0.0, 0.0, -3.0),
            Vector::new(0.0, -2_f64.sqrt() / 2.0, 2_f64.sqrt() / 2.0),
        );

        let intersection = Intersection::new(2_f64.sqrt(), &world.objects[2]);
        let comps = intersection.prepare_computations(ray);

        // At recursion depth 0, should return black
        let color = world.reflected_color_internal(&comps, 0);
        assert_eq!(color, Color::BLACK)
    }

    #[test]
    fn refracted_color_with_opaque_surface() {
        let world = World::default();

        let shape = world.objects.first().unwrap();
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let intersections = [
            Intersection::new(4.0, shape),
            Intersection::new(6.0, shape),
        ];
        let comps = intersections[0].prepare_computations(ray);
        let color = world.refracted_color(comps);
        assert_eq!(color, Color::BLACK);
    }

    #[test]
    fn refracted_color_at_maximum_recursion_depth() {
        let mut world = World::default();

        let mut shape = Object::plane();
        shape.set_material(Material {
            transparency: 1.0,
            refractive_index: 1.5,
            ..Default::default()
        });
        shape.set_transform(Matrix4::translate(0.0, -1.0, 0.0));
        world.objects.push(shape);

        let ray = Ray::new(
            Point::new(0.0, 0.0, -3.0),
            Vector::new(0.0, -2_f64.sqrt() / 2.0, 2_f64.sqrt() / 2.0),
        );

        let intersection = Intersection::new(2_f64.sqrt(), &world.objects[2]);
        let comps = intersection.prepare_computations(ray);

        let color = world.refracted_color_internal(&comps, 0);
        assert_eq!(color, Color::BLACK)
    }

    #[test]
    fn refracted_color_with_total_internal_reflection() {
        let world = World::default();

        let mut shape = world.objects[0];

        shape.set_material(Material {
            transparency: 1.0,
            refractive_index: 1.5,
            ..Default::default()
        });

        let ray = Ray::new(
            Point::new(0.0, 0.0, -2_f64.sqrt() / 2.0),
            Vector::new(0.0, 1.0, 0.0),
        );

        let intersections = [
            Intersection::new(-2_f64.sqrt() / 2.0, &shape),
            Intersection::new(2_f64.sqrt() / 2.0, &shape),
        ];

        let comps = intersections[1].prepare_computations_for_intersections(ray, &intersections);

        let color = world.refracted_color(comps);
        assert_eq!(color, Color::BLACK)
    }

    #[test]
    fn refracted_color_with_refracted_ray() {
        // Create outer sphere with test pattern and full ambient lighting
        let shape_a = Object::Sphere(Sphere {
            material: Material {
                color: Color::new(0.8, 1.0, 0.6),
                diffuse: 0.7,
                specular: 0.2,
                ambient: 1.0,
                pattern: Some(Pattern::test()),
                ..Default::default()
            },
            ..Default::default()
        });

        // Create inner sphere (scaled) with transparency
        let shape_b = Object::Sphere(Sphere {
            transformation: Matrix4::scale(0.5, 0.5, 0.5),
            material: Material {
                transparency: 1.0,
                refractive_index: 1.5,
                ..Default::default()
            },
        });

        let world = World {
            objects: vec![shape_a, shape_b],
            light_source: Some(PointLight::new(
                Point::new(-10.0, 10.0, -10.0),
                Color::new(1.0, 1.0, 1.0),
            )),
        };

        let ray = Ray::new(Point::new(0.0, 0.0, 0.1), Vector::new(0.0, 1.0, 0.0));

        let intersections = [
            Intersection::new(-0.9899, &world.objects[0]),
            Intersection::new(-0.4899, &world.objects[1]),
            Intersection::new(0.4899, &world.objects[1]),
            Intersection::new(0.9899, &world.objects[0]),
        ];

        let comps = intersections[2].prepare_computations_for_intersections(ray, &intersections);

        let color = world.refracted_color(comps);
        let expected = Color::new(0.0, 0.99888, 0.04722);
        assert!(
            float_equal(color.red, expected.red)
                && float_equal(color.green, expected.green)
                && float_equal(color.blue, expected.blue),
            "Expected color {:?}, got {:?}",
            expected,
            color
        );
    }

    #[test]
    pub fn shade_hit_with_transparent_material() {
        let mut world = World::default();
        let floor = Object::Plane(Plane {
            transformation: Matrix4::translate(0.0, -1.0, 0.0),
            material: Material {
                transparency: 0.5,
                refractive_index: 1.5,
                ..Default::default()
            },
        });
        world.objects.push(floor);

        let ball = Object::Sphere(Sphere {
            transformation: Matrix4::translate(0.0, -3.5, -0.5),
            material: Material {
                color: Color::new(1.0, 0.0, 0.0),
                ambient: 0.5,
                ..Default::default()
            },
        });
        world.objects.push(ball);

        let ray = Ray::new(
            Point::new(0.0, 0.0, -3.0),
            Vector::new(0.0, -2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0),
        );
        let intersections = [Intersection::new(2_f64.sqrt(), &floor)];

        let comps = intersections[0].prepare_computations_for_intersections(ray, &intersections);

        let color = world.shade_hit(comps);
        let expected = Color::new(0.93642, 0.68642, 0.68642);
        assert!(
            float_equal(color.red, expected.red)
                && float_equal(color.green, expected.green)
                && float_equal(color.blue, expected.blue),
            "Expected color {:?}, got {:?}",
            expected,
            color
        );
    }

    #[test]
    pub fn shade_hit_with_reflective_transparent_material() {
        let mut world = World::default();
        let ray = Ray::new(
            Point::new(0.0, 0.0, -3.0),
            Vector::new(0.0, -2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0),
        );

        let floor = Object::Plane(Plane {
            transformation: Matrix4::translate(0.0, -1.0, 0.0),
            material: Material {
                reflectivity: 0.5,
                transparency: 0.5,
                refractive_index: 1.5,
                ..Default::default()
            },
        });
        world.objects.push(floor);

        let ball = Object::Sphere(Sphere {
            transformation: Matrix4::translate(0.0, -3.5, -0.5),
            material: Material {
                color: Color::new(1.0, 0.0, 0.0),
                ambient: 0.5,
                ..Default::default()
            },
        });
        world.objects.push(ball);

        let intersections = [Intersection::new(2_f64.sqrt(), &world.objects[2])];

        let comps = intersections[0].prepare_computations_for_intersections(ray, &intersections);

        let color = world.shade_hit(comps);
        let expected = Color::new(0.93391, 0.69643, 0.69243);
        assert!(
            float_equal(color.red, expected.red)
                && float_equal(color.green, expected.green)
                && float_equal(color.blue, expected.blue),
            "Expected color {:?}, got {:?}",
            expected,
            color
        );
    }
}
