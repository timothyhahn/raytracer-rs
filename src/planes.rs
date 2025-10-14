use crate::floats::EPSILON;
use crate::materials::Material;
use crate::matrices::Matrix4;
use crate::rays::Ray;
use crate::shapes::Shape;
use crate::tuples::{Point, Tuple, Vector};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Plane {
    pub transformation: Matrix4,
    pub material: Material,
}

impl Plane {
    pub fn new() -> Self {
        Self {
            transformation: Matrix4::identity(),
            material: Material::default(),
        }
    }
}

// Handles intersections for Planes.
// Assumes missing for the two following scenarios:
// - Ray origin is on the plane (coplanar)
// - Ray direction is parallel to the plane
// and calculates this for the following two scenarios:
// - Ray is above the plane
// - Ray is below the plane
impl Shape for Plane {
    fn local_intersect(&self, ray: Ray) -> Vec<f64> {
        if ray.direction.y.abs() < EPSILON {
            // Ray is parallel to the plane (or close enough), no intersection
            vec![]
        } else {
            // Ray is above or below the plane, t = -origin_y / direction_y
            // This only works if plane is in xz space, with normal pointing up in y direction
            // (This assumption is hard defined in our implementation)
            vec![-ray.origin.y / ray.direction.y]
        }
    }

    fn local_normal_at(&self, _point: Point) -> Vector {
        Vector::new(0.0, 1.0, 0.0)
    }
}

impl Default for Plane {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::{Intersectable, Object};
    use crate::tuples::{Point, Tuple, Vector};

    #[test]
    fn plane_normal_constant_everywhere() {
        let plane = Object::Plane(Plane::new());
        let normal1 = plane.normal_at(Point::new(0.0, 0.0, 0.0));
        assert_eq!(normal1, Vector::new(0.0, 1.0, 0.0));
        let normal2 = plane.normal_at(Point::new(10.0, 0.0, 10.0));
        assert_eq!(normal2, Vector::new(0.0, 1.0, 0.0));
        let normal3 = plane.normal_at(Point::new(-5.0, 0.0, 150.0));
        assert_eq!(normal3, Vector::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn intersect_ray_parallel_to_plane() {
        let plane = Object::Plane(Plane::new());
        let ray = Ray::new(Point::new(0.0, 10.0, 0.0), Vector::new(0.0, 0.0, 1.0));
        let xs = plane.intersect(ray);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn intersect_ray_coplanar_to_plane() {
        let plane = Object::Plane(Plane::new());
        let ray = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0));
        let xs = plane.intersect(ray);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn intersect_plane_from_above() {
        let plane = Object::Plane(Plane::new());
        let ray = Ray::new(Point::new(0.0, 1.0, 0.0), Vector::new(0.0, -1.0, 0.0));
        let xs = plane.intersect(ray);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0], 1.0);
    }

    #[test]
    fn intersect_plane_from_below() {
        let plane = Object::Plane(Plane::new());
        let ray = Ray::new(Point::new(0.0, -1.0, 0.0), Vector::new(0.0, 1.0, 0.0));
        let xs = plane.intersect(ray);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0], 1.0);
    }
}
