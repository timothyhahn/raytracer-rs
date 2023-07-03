use crate::color::Color;
use crate::intersections::{Computations, Intersection};
use crate::lights::PointLight;
use crate::materials::Material;
use crate::matrices::Matrix4;
use crate::objects::{Intersectable, Object};
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

    pub fn intersect(&self, ray: Ray) -> Vec<Intersection> {
        let mut intersections: Vec<Intersection> = Vec::new();
        for object in self.objects.iter() {
            let object_intersections = object.intersect(ray);
            for intersection in object_intersections {
                intersections.push(Intersection {
                    object,
                    t: intersection,
                });
            }
        }
        intersections.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        intersections
    }

    pub fn shade_hit(&self, comps: Computations) -> Color {
        comps.object.material().lighting(
            self.light_source.unwrap(),
            comps.point,
            comps.eye_vector,
            comps.normal_vector,
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
    use crate::objects::Intersectable;
    use crate::rays::Ray;
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
}
