use crate::core::floats::EPSILON;
use crate::core::matrices::Matrix4;
use crate::core::tuples::{Point, Tuple, Vector};
use crate::geometry::bounds::Bounds;
use crate::geometry::shapes::Shape;
use crate::rendering::objects::Object;
use crate::rendering::rays::Ray;
use crate::scene::materials::Material;
use std::sync::{RwLock, Weak as SyncWeak};

/// Möller-Trumbore ray-triangle intersection algorithm
/// Returns Some((t, u, v)) if intersection found, None otherwise
/// t is the ray parameter, u and v are barycentric coordinates
fn moller_trumbore_intersect(
    p1: Point,
    e1: Vector,
    e2: Vector,
    ray: Ray,
) -> Option<(f64, f64, f64)> {
    let dir_cross_e2 = ray.direction.cross(&e2);
    let det = e1.dot(&dir_cross_e2);

    // If determinant is near zero, ray lies in plane of triangle or is parallel
    if det.abs() < EPSILON {
        return None;
    }

    let f = 1.0 / det;

    let p1_to_origin = ray.origin - p1;
    let u = f * p1_to_origin.dot(&dir_cross_e2);

    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let origin_cross_e1 = p1_to_origin.cross(&e1);
    let v = f * ray.direction.dot(&origin_cross_e1);

    if v < 0.0 || (u + v) > 1.0 {
        return None;
    }

    let t = f * e2.dot(&origin_cross_e1);

    Some((t, u, v))
}

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
    pub parent: Option<SyncWeak<RwLock<Object>>>,
}

#[derive(Debug, Clone)]
pub struct SmoothTriangle {
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
    pub n1: Vector, // Normal at p1
    pub n2: Vector, // Normal at p2
    pub n3: Vector, // Normal at p3
    pub e1: Vector,
    pub e2: Vector,
    pub transformation: Matrix4,
    pub world_transformation: Matrix4,
    pub material: Material,
    pub parent: Option<SyncWeak<RwLock<Object>>>,
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

    /// Compute intersection with u/v coordinates
    /// Returns Vec<(t, u, v)>
    pub fn local_intersect_uv(&self, ray: Ray) -> Vec<(f64, f64, f64)> {
        moller_trumbore_intersect(self.p1, self.e1, self.e2, ray)
            .map(|result| vec![result])
            .unwrap_or_default()
    }
}

impl Shape for Triangle {
    fn local_intersect(&self, ray: Ray) -> Vec<f64> {
        moller_trumbore_intersect(self.p1, self.e1, self.e2, ray)
            .map(|(t, _u, _v)| vec![t])
            .unwrap_or_default()
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

impl PartialEq for SmoothTriangle {
    fn eq(&self, other: &Self) -> bool {
        self.p1 == other.p1
            && self.p2 == other.p2
            && self.p3 == other.p3
            && self.n1 == other.n1
            && self.n2 == other.n2
            && self.n3 == other.n3
            && self.transformation == other.transformation
            && self.material == other.material
        // Ignore parent for equality comparison
    }
}

impl SmoothTriangle {
    pub fn new(p1: Point, p2: Point, p3: Point, n1: Vector, n2: Vector, n3: Vector) -> Self {
        let e1 = p2 - p1;
        let e2 = p3 - p1;

        // Normalize input normals to ensure they're unit vectors
        // OBJ files may contain non-unit normals
        Self {
            p1,
            p2,
            p3,
            n1: n1.normalize(),
            n2: n2.normalize(),
            n3: n3.normalize(),
            e1,
            e2,
            transformation: Matrix4::identity(),
            world_transformation: Matrix4::identity(),
            material: Material::default(),
            parent: None,
        }
    }

    /// Compute intersection with u/v coordinates
    /// Returns Vec<(t, u, v)>
    pub fn local_intersect_uv(&self, ray: Ray) -> Vec<(f64, f64, f64)> {
        moller_trumbore_intersect(self.p1, self.e1, self.e2, ray)
            .map(|result| vec![result])
            .unwrap_or_default()
    }

    /// Interpolate the normal at the given u/v coordinates
    /// Returns a normalized vector
    pub fn interpolated_normal(&self, u: f64, v: f64) -> Vector {
        (self.n2 * u + self.n3 * v + self.n1 * (1.0 - u - v)).normalize()
    }
}

impl Shape for SmoothTriangle {
    fn local_intersect(&self, ray: Ray) -> Vec<f64> {
        moller_trumbore_intersect(self.p1, self.e1, self.e2, ray)
            .map(|(t, _u, _v)| vec![t])
            .unwrap_or_default()
    }

    fn local_normal_at(&self, _point: Point) -> Vector {
        // Return n1 as a fallback when no hit data is available
        // When hit data is present, Object::normal_at_with_hit uses interpolated_normal instead
        self.n1
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

impl Default for SmoothTriangle {
    fn default() -> Self {
        Self::new(
            Point::new(0.0, 1.0, 0.0),
            Point::new(-1.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0),
            Vector::new(0.0, 1.0, 0.0),
            Vector::new(-1.0, 0.0, 0.0),
            Vector::new(1.0, 0.0, 0.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tuples::Tuple;
    use crate::rendering::intersections::Intersection;
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

    #[test]
    fn constructing_smooth_triangle() {
        let p1 = Point::new(0.0, 1.0, 0.0);
        let p2 = Point::new(-1.0, 0.0, 0.0);
        let p3 = Point::new(1.0, 0.0, 0.0);
        let n1 = Vector::new(0.0, 1.0, 0.0);
        let n2 = Vector::new(-1.0, 0.0, 0.0);
        let n3 = Vector::new(1.0, 0.0, 0.0);
        let tri = SmoothTriangle::new(p1, p2, p3, n1, n2, n3);

        assert_eq!(tri.p1, p1);
        assert_eq!(tri.p2, p2);
        assert_eq!(tri.p3, p3);
        assert_eq!(tri.n1, n1);
        assert_eq!(tri.n2, n2);
        assert_eq!(tri.n3, n3);
    }

    #[test]
    fn intersection_with_smooth_triangle_stores_uv() {
        let p1 = Point::new(0.0, 1.0, 0.0);
        let p2 = Point::new(-1.0, 0.0, 0.0);
        let p3 = Point::new(1.0, 0.0, 0.0);
        let n1 = Vector::new(0.0, 1.0, 0.0);
        let n2 = Vector::new(-1.0, 0.0, 0.0);
        let n3 = Vector::new(1.0, 0.0, 0.0);
        let tri = Object::SmoothTriangle(SmoothTriangle::new(p1, p2, p3, n1, n2, n3));

        let r = Ray::new(Point::new(-0.2, 0.3, -2.0), Vector::new(0.0, 0.0, 1.0));
        let xs = tri.intersect_with_object(r);

        assert_eq!(xs.len(), 1);
        assert!((xs[0].u - 0.45).abs() < 0.0001);
        assert!((xs[0].v - 0.25).abs() < 0.0001);
    }

    #[test]
    fn smooth_triangle_uses_uv_to_interpolate_normal() {
        let p1 = Point::new(0.0, 1.0, 0.0);
        let p2 = Point::new(-1.0, 0.0, 0.0);
        let p3 = Point::new(1.0, 0.0, 0.0);
        let n1 = Vector::new(0.0, 1.0, 0.0);
        let n2 = Vector::new(-1.0, 0.0, 0.0);
        let n3 = Vector::new(1.0, 0.0, 0.0);
        let tri = Object::SmoothTriangle(SmoothTriangle::new(p1, p2, p3, n1, n2, n3));

        let i = Intersection::new_with_uv(1.0, &tri, 0.45, 0.25);
        let n = tri.normal_at_with_hit(Point::new(0.0, 0.0, 0.0), Some(&i));

        assert!((n.x - (-0.5547)).abs() < 0.0001);
        assert!((n.y - 0.83205).abs() < 0.0001);
        assert!((n.z - 0.0).abs() < 0.0001);
    }

    #[test]
    fn preparing_normal_on_smooth_triangle() {
        let p1 = Point::new(0.0, 1.0, 0.0);
        let p2 = Point::new(-1.0, 0.0, 0.0);
        let p3 = Point::new(1.0, 0.0, 0.0);
        let n1 = Vector::new(0.0, 1.0, 0.0);
        let n2 = Vector::new(-1.0, 0.0, 0.0);
        let n3 = Vector::new(1.0, 0.0, 0.0);
        let tri = Object::SmoothTriangle(SmoothTriangle::new(p1, p2, p3, n1, n2, n3));

        let i = Intersection::new_with_uv(1.0, &tri, 0.45, 0.25);
        let r = Ray::new(Point::new(-0.2, 0.3, -2.0), Vector::new(0.0, 0.0, 1.0));
        let comps = i.prepare_computations(r);

        assert!((comps.normal_vector.x - (-0.5547)).abs() < 0.0001);
        assert!((comps.normal_vector.y - 0.83205).abs() < 0.0001);
        assert!((comps.normal_vector.z - 0.0).abs() < 0.0001);
    }
}
