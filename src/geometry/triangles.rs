use crate::core::floats::EPSILON;
use crate::core::matrices::Matrix4;
use crate::core::tuples::{Point, Tuple, Vector};
use crate::geometry::bounds::Bounds;
use crate::geometry::shapes::Shape;
use crate::rendering::objects::Object;
use crate::rendering::rays::Ray;
use crate::scene::materials::Material;
use std::cell::RefCell;
use std::rc::Weak;

#[derive(Debug, Clone)]
pub struct Triangle {
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
    pub e1: Vector,
    pub e2: Vector,
    pub normal: Vector,
    pub transformation: Matrix4,
    pub world_transformation: Matrix4,
    pub material: Material,
    pub parent: Option<Weak<RefCell<Object>>>,
}

impl PartialEq for Triangle {
    fn eq(&self, other: &Self) -> bool {
        self.p1 == other.p1
            && self.p2 == other.p2
            && self.p3 == other.p3
            && self.transformation == other.transformation
            && self.material == other.material
        // Ignore parent for equality comparison
    }
}

impl Triangle {
    pub fn new(p1: Point, p2: Point, p3: Point) -> Self {
        let e1 = p2 - p1;
        let e2 = p3 - p1;
        let normal = e2.cross(&e1).normalize();

        Self {
            p1,
            p2,
            p3,
            e1,
            e2,
            normal,
            transformation: Matrix4::identity(),
            world_transformation: Matrix4::identity(),
            material: Material::default(),
            parent: None,
        }
    }
}

impl Shape for Triangle {
    fn local_intersect(&self, ray: Ray) -> Vec<f64> {
        // Moller-Trumbore algorithm
        let dir_cross_e2 = ray.direction.cross(&self.e2);
        let det = self.e1.dot(&dir_cross_e2);

        // If determinant is near zero, ray lies in plane of triangle or is parallel
        if det.abs() < EPSILON {
            return vec![];
        }

        let f = 1.0 / det;

        let p1_to_origin = ray.origin - self.p1;
        let u = f * p1_to_origin.dot(&dir_cross_e2);

        if !(0.0..=1.0).contains(&u) {
            return vec![];
        }

        let origin_cross_e1 = p1_to_origin.cross(&self.e1);
        let v = f * ray.direction.dot(&origin_cross_e1);

        // Can't use range contains for the compound condition (u + v) > 1.0
        if v < 0.0 || (u + v) > 1.0 {
            return vec![];
        }

        let t = f * self.e2.dot(&origin_cross_e1);

        vec![t]
    }

    fn local_normal_at(&self, _point: Point) -> Vector {
        // Triangle normals are constant across the surface
        self.normal
    }

    fn bounds(&self) -> Bounds {
        let min_x = self.p1.x.min(self.p2.x).min(self.p3.x);
        let min_y = self.p1.y.min(self.p2.y).min(self.p3.y);
        let min_z = self.p1.z.min(self.p2.z).min(self.p3.z);

        let max_x = self.p1.x.max(self.p2.x).max(self.p3.x);
        let max_y = self.p1.y.max(self.p2.y).max(self.p3.y);
        let max_z = self.p1.z.max(self.p2.z).max(self.p3.z);

        Bounds::new(
            Point::new(min_x, min_y, min_z),
            Point::new(max_x, max_y, max_z),
        )
    }
}

impl Default for Triangle {
    fn default() -> Self {
        Self::new(
            Point::new(0.0, 1.0, 0.0),
            Point::new(-1.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tuples::Tuple;
    use crate::rendering::objects::{Intersectable, Object};

    #[test]
    fn constructing_triangle() {
        let p1 = Point::new(0.0, 1.0, 0.0);
        let p2 = Point::new(-1.0, 0.0, 0.0);
        let p3 = Point::new(1.0, 0.0, 0.0);
        let t = Triangle::new(p1, p2, p3);

        assert_eq!(t.p1, p1);
        assert_eq!(t.p2, p2);
        assert_eq!(t.p3, p3);
        assert_eq!(t.e1, Vector::new(-1.0, -1.0, 0.0));
        assert_eq!(t.e2, Vector::new(1.0, -1.0, 0.0));
        assert_eq!(t.normal, Vector::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn finding_normal_on_triangle() {
        let t = Object::Triangle(Triangle::new(
            Point::new(0.0, 1.0, 0.0),
            Point::new(-1.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0),
        ));

        let n1 = t.normal_at(Point::new(0.0, 0.5, 0.0));
        let n2 = t.normal_at(Point::new(-0.5, 0.75, 0.0));
        let n3 = t.normal_at(Point::new(0.5, 0.25, 0.0));

        // All normals should be the same
        assert_eq!(n1, Vector::new(0.0, 0.0, -1.0));
        assert_eq!(n2, Vector::new(0.0, 0.0, -1.0));
        assert_eq!(n3, Vector::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn intersecting_ray_parallel_to_triangle() {
        let t = Object::Triangle(Triangle::new(
            Point::new(0.0, 1.0, 0.0),
            Point::new(-1.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0),
        ));
        let r = Ray::new(Point::new(0.0, -1.0, -2.0), Vector::new(0.0, 1.0, 0.0));
        let xs = t.intersect(r);

        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn ray_misses_p1_p3_edge() {
        let t = Object::Triangle(Triangle::new(
            Point::new(0.0, 1.0, 0.0),
            Point::new(-1.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0),
        ));
        let r = Ray::new(Point::new(1.0, 1.0, -2.0), Vector::new(0.0, 0.0, 1.0));
        let xs = t.intersect(r);

        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn ray_misses_p1_p2_edge() {
        let t = Object::Triangle(Triangle::new(
            Point::new(0.0, 1.0, 0.0),
            Point::new(-1.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0),
        ));
        let r = Ray::new(Point::new(-1.0, 1.0, -2.0), Vector::new(0.0, 0.0, 1.0));
        let xs = t.intersect(r);

        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn ray_misses_p2_p3_edge() {
        let t = Object::Triangle(Triangle::new(
            Point::new(0.0, 1.0, 0.0),
            Point::new(-1.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0),
        ));
        let r = Ray::new(Point::new(0.0, -1.0, -2.0), Vector::new(0.0, 0.0, 1.0));
        let xs = t.intersect(r);

        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn ray_strikes_triangle() {
        let t = Object::Triangle(Triangle::new(
            Point::new(0.0, 1.0, 0.0),
            Point::new(-1.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0),
        ));
        let r = Ray::new(Point::new(0.0, 0.5, -2.0), Vector::new(0.0, 0.0, 1.0));
        let xs = t.intersect(r);

        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0], 2.0);
    }
}
