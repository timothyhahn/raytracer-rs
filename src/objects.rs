use crate::intersections::Intersection;
use crate::materials::Material;
use crate::matrices::Matrix4;
use crate::rays::Ray;
use crate::sphere::Sphere;
use crate::tuples::{Point, Vector};

pub trait Intersectable {
    fn intersect(&self, r: Ray) -> Vec<f64>;
    fn intersect_with_object(&self, r: Ray) -> Vec<Intersection>;
    fn normal_at(&self, p: Point) -> Vector;
    fn material(&self) -> Material;
    fn transformation(&self) -> Matrix4;
    fn set_transform(&mut self, transformation: Matrix4);
    fn set_material(&mut self, material: Material);
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Object {
    Sphere(Sphere),
}

impl Intersectable for Object {
    fn intersect(&self, r: Ray) -> Vec<f64> {
        match *self {
            Object::Sphere(ref s) => s.intersect(r),
        }
    }

    fn intersect_with_object(&self, r: Ray) -> Vec<Intersection> {
        match *self {
            Object::Sphere(ref s) => s
                .intersect(r)
                .iter()
                .map(|t| Intersection::new(*t, self))
                .collect(),
        }
    }

    fn normal_at(&self, p: Point) -> Vector {
        match *self {
            Object::Sphere(ref s) => s.normal_at(p),
        }
    }

    fn material(&self) -> Material {
        match *self {
            Object::Sphere(ref s) => s.material,
        }
    }

    fn transformation(&self) -> Matrix4 {
        match *self {
            Object::Sphere(ref s) => s.transformation,
        }
    }

    fn set_transform(&mut self, transformation: Matrix4) {
        match *self {
            Object::Sphere(ref mut s) => s.set_transform(transformation),
        }
    }

    fn set_material(&mut self, material: Material) {
        match *self {
            Object::Sphere(ref mut s) => s.set_material(material),
        }
    }
}
