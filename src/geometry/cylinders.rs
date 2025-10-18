use crate::{
    core::{
        floats::{float_equal, EPSILON},
        matrices::Matrix4,
        tuples::{Point, Tuple, Vector},
    },
    geometry::{bounds::Bounds, shapes::Shape},
    rendering::{objects::Object, rays::Ray},
    scene::materials::Material,
};
use std::cell::RefCell;
use std::rc::Weak;

#[derive(Debug, Clone)]
pub struct Cylinder {
    pub transformation: Matrix4,
    pub world_transformation: Matrix4,
    pub material: Material,
    pub minimum: f64,
    pub maximum: f64,
    pub closed: bool,
    pub parent: Option<Weak<RefCell<Object>>>,
}

impl PartialEq for Cylinder {
    fn eq(&self, other: &Self) -> bool {
        self.transformation == other.transformation
            && self.material == other.material
            && self.minimum == other.minimum
            && self.maximum == other.maximum
            && self.closed == other.closed
        // Ignore parent for equality comparison
    }
}

impl Cylinder {
    pub fn new() -> Self {
        Self {
            transformation: Matrix4::identity(),
            world_transformation: Matrix4::identity(),
            material: Material::default(),
            minimum: f64::NEG_INFINITY,
            maximum: f64::INFINITY,
            closed: false,
            parent: None,
        }
    }
}

impl Default for Cylinder {
    fn default() -> Self {
        Self::new()
    }
}

impl Shape for Cylinder {
    fn local_intersect(&self, ray: Ray) -> Vec<f64> {
        let a = ray.direction.x.powi(2) + ray.direction.z.powi(2);

        let mut xs = Vec::new();

        if float_equal(a, 0.0) {
            if self.closed {
                self.intersect_caps(ray, &mut xs);
            }
            return xs;
        }

        let b = 2.0 * ray.origin.x * ray.direction.x + 2.0 * ray.origin.z * ray.direction.z;
        let c = ray.origin.x.powi(2) + ray.origin.z.powi(2) - 1.0;

        let discriminant = b.powi(2) - 4.0 * a * c;
        if discriminant < 0.0 {
            return Vec::new();
        }

        let sqrt_disc = discriminant.sqrt();
        let denom = 2.0 * a;
        let mut t0 = (-b - sqrt_disc) / denom;
        let mut t1 = (-b + sqrt_disc) / denom;

        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }

        let y0 = ray.origin.y + t0 * ray.direction.y;
        let y1 = ray.origin.y + t1 * ray.direction.y;

        if self.minimum < y0 && y0 < self.maximum {
            xs.push(t0);
        }
        if self.minimum < y1 && y1 < self.maximum {
            xs.push(t1);
        }

        self.intersect_caps(ray, &mut xs);

        xs
    }

    fn local_normal_at(&self, point: Point) -> Vector {
        let dist = point.x.powi(2) + point.z.powi(2);

        if dist < 1.0 && point.y >= self.maximum - EPSILON {
            Vector::new(0.0, 1.0, 0.0)
        } else if dist < 1.0 && point.y <= self.minimum + EPSILON {
            Vector::new(0.0, -1.0, 0.0)
        } else {
            Vector::new(point.x, 0.0, point.z)
        }
    }

    /// Get the bounding box for this cylinder.
    /// Radius is 1 in x and z, and extends from minimum to maximum in y.
    fn bounds(&self) -> Bounds {
        Bounds::new(
            Point::new(-1.0, self.minimum, -1.0),
            Point::new(1.0, self.maximum, 1.0),
        )
    }
}

impl Cylinder {
    fn intersect_caps(&self, ray: Ray, intersections: &mut Vec<f64>) {
        if !self.closed || float_equal(ray.direction.y, 0.0) {
            return;
        }

        let t = (self.minimum - ray.origin.y) / ray.direction.y;
        if self.check_cap(ray, t) {
            intersections.push(t);
        }

        let t = (self.maximum - ray.origin.y) / ray.direction.y;
        if self.check_cap(ray, t) {
            intersections.push(t);
        }
    }

    fn check_cap(&self, ray: Ray, t: f64) -> bool {
        let x = ray.origin.x + t * ray.direction.x;
        let z = ray.origin.z + t * ray.direction.z;
        (x.powi(2) + z.powi(2)) <= 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::Cylinder;
    use crate::core::floats::float_equal;
    use crate::{
        core::tuples::{Point, Tuple, Vector},
        geometry::shapes::Shape,
        rendering::rays::Ray,
    };

    #[test]
    fn default_bounds() {
        let cylinder = Cylinder::new();
        assert!(cylinder.minimum.is_infinite() && cylinder.minimum.is_sign_negative());
        assert!(cylinder.maximum.is_infinite() && cylinder.maximum.is_sign_positive());
    }

    #[test]
    fn ray_misses_cylinder() {
        struct TestCase<'a> {
            label: &'a str,
            origin: Point,
            direction: Vector,
        }

        let cylinder = Cylinder::new();
        let test_cases = [
            TestCase {
                label: "offset x-axis",
                origin: Point::new(1.0, 0.0, 0.0),
                direction: Vector::new(0.0, 1.0, 0.0).normalize(),
            },
            TestCase {
                label: "origin on axis",
                origin: Point::new(0.0, 0.0, 0.0),
                direction: Vector::new(0.0, 1.0, 0.0).normalize(),
            },
            TestCase {
                label: "diagonal miss",
                origin: Point::new(0.0, 0.0, -5.0),
                direction: Vector::new(1.0, 1.0, 1.0).normalize(),
            },
        ];

        for test_case in test_cases {
            let ray = Ray::new(test_case.origin, test_case.direction);
            let xs = cylinder.local_intersect(ray);
            assert_eq!(
                xs.len(),
                0,
                "Expected no intersections for test case {}",
                test_case.label
            );
        }
    }

    #[test]
    fn ray_strikes_cylinder() {
        struct TestCase<'a> {
            label: &'a str,
            origin: Point,
            direction: Vector,
            t0: f64,
            t1: f64,
        }

        let cylinder = Cylinder::new();
        let test_cases = [
            TestCase {
                label: "grazing hit",
                origin: Point::new(1.0, 0.0, -5.0),
                direction: Vector::new(0.0, 0.0, 1.0).normalize(),
                t0: 5.0,
                t1: 5.0,
            },
            TestCase {
                label: "center hit",
                origin: Point::new(0.0, 0.0, -5.0),
                direction: Vector::new(0.0, 0.0, 1.0).normalize(),
                t0: 4.0,
                t1: 6.0,
            },
            TestCase {
                label: "diagonal hit",
                origin: Point::new(0.5, 0.0, -5.0),
                direction: Vector::new(0.1, 1.0, 1.0).normalize(),
                t0: 6.80798,
                t1: 7.08872,
            },
        ];

        for test_case in test_cases {
            let ray = Ray::new(test_case.origin, test_case.direction);
            let xs = cylinder.local_intersect(ray);
            assert_eq!(
                xs.len(),
                2,
                "Expected two intersections for test case {}",
                test_case.label
            );
            assert!(
                float_equal(xs[0], test_case.t0),
                "Expected t0 ≈ {} got {} for {}",
                test_case.t0,
                xs[0],
                test_case.label
            );
            assert!(
                float_equal(xs[1], test_case.t1),
                "Expected t1 ≈ {} got {} for {}",
                test_case.t1,
                xs[1],
                test_case.label
            );
        }
    }

    #[test]
    fn intersecting_constrained_cylinder() {
        struct TestCase<'a> {
            label: &'a str,
            origin: Point,
            direction: Vector,
            count: usize,
        }

        let mut cylinder = Cylinder::new();
        cylinder.minimum = 1.0;
        cylinder.maximum = 2.0;

        let test_cases = [
            TestCase {
                label: "case 1",
                origin: Point::new(0.0, 1.5, 0.0),
                direction: Vector::new(0.1, 1.0, 0.0).normalize(),
                count: 0,
            },
            TestCase {
                label: "case 2",
                origin: Point::new(0.0, 3.0, -5.0),
                direction: Vector::new(0.0, 0.0, 1.0).normalize(),
                count: 0,
            },
            TestCase {
                label: "case 3",
                origin: Point::new(0.0, 0.0, -5.0),
                direction: Vector::new(0.0, 0.0, 1.0).normalize(),
                count: 0,
            },
            TestCase {
                label: "case 4",
                origin: Point::new(0.0, 2.0, -5.0),
                direction: Vector::new(0.0, 0.0, 1.0).normalize(),
                count: 0,
            },
            TestCase {
                label: "case 5",
                origin: Point::new(0.0, 1.0, -5.0),
                direction: Vector::new(0.0, 0.0, 1.0).normalize(),
                count: 0,
            },
            TestCase {
                label: "case 6",
                origin: Point::new(0.0, 1.5, -2.0),
                direction: Vector::new(0.0, 0.0, 1.0).normalize(),
                count: 2,
            },
        ];

        for test_case in test_cases {
            let ray = Ray::new(test_case.origin, test_case.direction);
            let xs = cylinder.local_intersect(ray);
            assert_eq!(
                xs.len(),
                test_case.count,
                "Unexpected hit count for {}",
                test_case.label
            );
        }
    }

    #[test]
    fn intersecting_closed_cylinder_end_caps() {
        struct TestCase<'a> {
            label: &'a str,
            origin: Point,
            direction: Vector,
            count: usize,
        }

        let mut cylinder = Cylinder::new();
        cylinder.minimum = 1.0;
        cylinder.maximum = 2.0;
        cylinder.closed = true;

        let test_cases = [
            TestCase {
                label: "case 1",
                origin: Point::new(0.0, 3.0, 0.0),
                direction: Vector::new(0.0, -1.0, 0.0).normalize(),
                count: 2,
            },
            TestCase {
                label: "case 2",
                origin: Point::new(0.0, 3.0, -2.0),
                direction: Vector::new(0.0, -1.0, 2.0).normalize(),
                count: 2,
            },
            TestCase {
                label: "case 3",
                origin: Point::new(0.0, 4.0, -2.0),
                direction: Vector::new(0.0, -1.0, 1.0).normalize(),
                count: 2,
            },
            TestCase {
                label: "case 4",
                origin: Point::new(0.0, 0.0, -2.0),
                direction: Vector::new(0.0, 1.0, 2.0).normalize(),
                count: 2,
            },
            TestCase {
                label: "case 5",
                origin: Point::new(0.0, -1.0, -2.0),
                direction: Vector::new(0.0, 1.0, 1.0).normalize(),
                count: 2,
            },
        ];

        for test_case in test_cases {
            let ray = Ray::new(test_case.origin, test_case.direction);
            let xs = cylinder.local_intersect(ray);
            assert_eq!(
                xs.len(),
                test_case.count,
                "Unexpected hit count for {}",
                test_case.label
            );
        }
    }

    #[test]
    fn normal_vector_on_cylinder() {
        struct TestCase<'a> {
            label: &'a str,
            point: Point,
            expected: Vector,
        }

        let cylinder = Cylinder::new();
        let test_cases = [
            TestCase {
                label: "positive x",
                point: Point::new(1.0, 0.0, 0.0),
                expected: Vector::new(1.0, 0.0, 0.0),
            },
            TestCase {
                label: "positive z",
                point: Point::new(0.0, 5.0, -1.0),
                expected: Vector::new(0.0, 0.0, -1.0),
            },
            TestCase {
                label: "negative z",
                point: Point::new(0.0, -2.0, 1.0),
                expected: Vector::new(0.0, 0.0, 1.0),
            },
            TestCase {
                label: "negative x",
                point: Point::new(-1.0, 1.0, 0.0),
                expected: Vector::new(-1.0, 0.0, 0.0),
            },
        ];

        for test_case in test_cases {
            let normal = cylinder.local_normal_at(test_case.point);
            assert_eq!(
                normal, test_case.expected,
                "Unexpected normal for test case {}",
                test_case.label
            );
        }
    }

    #[test]
    fn normal_vector_on_capped_cylinder() {
        struct TestCase<'a> {
            label: &'a str,
            point: Point,
            expected: Vector,
        }

        let mut cylinder = Cylinder::new();
        cylinder.minimum = 1.0;
        cylinder.maximum = 2.0;
        cylinder.closed = true;

        let test_cases = [
            TestCase {
                label: "lower center",
                point: Point::new(0.0, 1.0, 0.0),
                expected: Vector::new(0.0, -1.0, 0.0),
            },
            TestCase {
                label: "lower interior",
                point: Point::new(0.5, 1.0, 0.0),
                expected: Vector::new(0.0, -1.0, 0.0),
            },
            TestCase {
                label: "lower interior z",
                point: Point::new(0.0, 1.0, 0.5),
                expected: Vector::new(0.0, -1.0, 0.0),
            },
            TestCase {
                label: "upper center",
                point: Point::new(0.0, 2.0, 0.0),
                expected: Vector::new(0.0, 1.0, 0.0),
            },
            TestCase {
                label: "upper interior",
                point: Point::new(0.5, 2.0, 0.0),
                expected: Vector::new(0.0, 1.0, 0.0),
            },
            TestCase {
                label: "upper interior z",
                point: Point::new(0.0, 2.0, 0.5),
                expected: Vector::new(0.0, 1.0, 0.0),
            },
        ];

        for test_case in test_cases {
            let normal = cylinder.local_normal_at(test_case.point);
            assert_eq!(
                normal, test_case.expected,
                "Unexpected normal for test case {}",
                test_case.label
            );
        }
    }
}
