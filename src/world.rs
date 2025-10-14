use crate::color::Color;
use crate::intersections::{Computations, Intersection};
use crate::lights::PointLight;
use crate::materials::Material;
use crate::matrices::Matrix4;
use crate::objects::{HasMaterial, Intersectable, Object};
use crate::rays::Ray;
use crate::sphere::Sphere;
use crate::tuples::{Point, Tuple};

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
        intersections.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        intersections
    }

    pub fn shade_hit(&self, comps: Computations) -> Color {
        let in_shadow = self.is_shadowed(comps.over_point);
        comps.object.material().lighting(
            self.light_source.unwrap(),
            comps.point,
            comps.eye_vector,
            comps.normal_vector,
            in_shadow,
        )
    }

    pub fn color_at(&self, ray: Ray) -> Color {
        let intersections = self.intersect(ray);
        let hit = Intersection::hit(intersections);
        match hit {
            Some(hit) => {
                let comps = hit.prepare_computations(ray);
                self.shade_hit(comps)
            }
            None => Color::black(),
        }
    }

    pub fn is_shadowed(&self, point: Point) -> bool {
        // Measure the distance from point to the light source
        let v = self.light_source.unwrap().position - point;
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
    use crate::color::Color;
    use crate::intersections::Intersection;
    use crate::lights::PointLight;
    use crate::materials::Material;
    use crate::matrices::Matrix4;
    use crate::objects::{HasMaterial, Intersectable, Object, Transformable};
    use crate::rays::Ray;
    use crate::sphere::Sphere;
    use crate::tuples::{Point, Tuple, Vector};
    use crate::world::World;

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
}
