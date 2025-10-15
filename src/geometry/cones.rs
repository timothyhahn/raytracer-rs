use crate::{
    core::{
        floats::{float_equal, EPSILON},
        matrices::Matrix4,
        tuples::{Point, Tuple, Vector},
    },
    geometry::shapes::Shape,
    rendering::rays::Ray,
    scene::materials::Material,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Cone {
    pub transformation: Matrix4,
    pub material: Material,
    pub minimum: f64,
    pub maximum: f64,
    pub closed: bool,
}

impl Cone {
    pub fn new() -> Self {
        Self {
            transformation: Matrix4::identity(),
            material: Material::default(),
            minimum: f64::NEG_INFINITY,
            maximum: f64::INFINITY,
            closed: false,
        }
    }
}

impl Default for Cone {
    fn default() -> Self {
        Self::new()
    }
}

impl Shape for Cone {
    fn local_intersect(&self, ray: Ray) -> Vec<f64> {
        let a = ray.direction.x.powi(2) - ray.direction.y.powi(2) + ray.direction.z.powi(2);
        let b = 2.0 * ray.origin.x * ray.direction.x - 2.0 * ray.origin.y * ray.direction.y
            + 2.0 * ray.origin.z * ray.direction.z;
        let c = ray.origin.x.powi(2) - ray.origin.y.powi(2) + ray.origin.z.powi(2);

        let mut xs = Vec::new();

        if float_equal(a, 0.0) {
            if float_equal(b, 0.0) {
                self.intersect_caps(ray, &mut xs);
                return xs;
            }
            let t = -c / (2.0 * b);
            let y = ray.origin.y + t * ray.direction.y;
            if self.minimum < y && y < self.maximum {
                xs.push(t);
            }
            self.intersect_caps(ray, &mut xs);
            return xs;
        }

        let discriminant = b.powi(2) - 4.0 * a * c;
        if discriminant < 0.0 {
            self.intersect_caps(ray, &mut xs);
            return xs;
        }

        let sqrt_disc = discriminant.sqrt();
        let denom = 2.0 * a;
        let mut t0 = (-b - sqrt_disc) / denom;
        let mut t1 = (-b + sqrt_disc) / denom;

        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }

        let y0 = ray.origin.y + t0 * ray.direction.y;
        if self.minimum < y0 && y0 < self.maximum {
            xs.push(t0);
        }

        let y1 = ray.origin.y + t1 * ray.direction.y;
        if self.minimum < y1 && y1 < self.maximum {
            xs.push(t1);
        }

        self.intersect_caps(ray, &mut xs);

        xs
    }

    fn local_normal_at(&self, point: Point) -> Vector {
        let dist = point.x.powi(2) + point.z.powi(2);

        if dist < 1.0 && point.y >= self.maximum - EPSILON {
            return Vector::new(0.0, 1.0, 0.0);
        }

        if dist < 1.0 && point.y <= self.minimum + EPSILON {
            return Vector::new(0.0, -1.0, 0.0);
        }

        let mut y = (point.x.powi(2) + point.z.powi(2)).sqrt();
        if point.y > 0.0 {
            y = -y;
        }

        Vector::new(point.x, y, point.z)
    }
}

impl Cone {
    fn intersect_caps(&self, ray: Ray, intersections: &mut Vec<f64>) {
        if !self.closed || float_equal(ray.direction.y, 0.0) {
            return;
        }

        let t = (self.minimum - ray.origin.y) / ray.direction.y;
        if self.check_cap(ray, t, self.minimum) {
            intersections.push(t);
        }

        let t = (self.maximum - ray.origin.y) / ray.direction.y;
        if self.check_cap(ray, t, self.maximum) {
            intersections.push(t);
        }
    }

    fn check_cap(&self, ray: Ray, t: f64, y: f64) -> bool {
        let x = ray.origin.x + t * ray.direction.x;
        let z = ray.origin.z + t * ray.direction.z;
        let radius = y.abs();
        (x.powi(2) + z.powi(2)) <= radius.powi(2)
    }
}

#[cfg(test)]
mod tests {
    use super::Cone;
    use crate::core::floats::float_equal;
    use crate::{
        core::tuples::{Point, Tuple, Vector},
        geometry::shapes::Shape,
        rendering::rays::Ray,
    };

    #[test]
    fn intersecting_a_cone_with_a_ray() {
        struct TestCase<'a> {
            label: &'a str,
            origin: Point,
            direction: Vector,
            t0: f64,
            t1: f64,
        }

        let cone = Cone::new();
        let test_cases = [
            TestCase {
                label: "straight on",
                origin: Point::new(0.0, 0.0, -5.0),
                direction: Vector::new(0.0, 0.0, 1.0).normalize(),
                t0: 5.0,
                t1: 5.0,
            },
            TestCase {
                label: "diagonal",
                origin: Point::new(0.0, 0.0, -5.0),
                direction: Vector::new(1.0, 1.0, 1.0).normalize(),
                t0: 8.66025,
                t1: 8.66025,
            },
            TestCase {
                label: "offset diagonal",
                origin: Point::new(1.0, 1.0, -5.0),
                direction: Vector::new(-0.5, -1.0, 1.0).normalize(),
                t0: 4.55006,
                t1: 49.44994,
            },
        ];

        for test_case in test_cases {
            let ray = Ray::new(test_case.origin, test_case.direction);
            let xs = cone.local_intersect(ray);
            assert_eq!(
                xs.len(),
                2,
                "Expected two intersections for {}",
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
    fn intersecting_cone_with_parallel_ray() {
        let cone = Cone::new();
        let direction = Vector::new(0.0, 1.0, 1.0).normalize();
        let ray = Ray::new(Point::new(0.0, 0.0, -1.0), direction);
        let xs = cone.local_intersect(ray);
        assert_eq!(xs.len(), 1);
        assert!(float_equal(xs[0], 0.35355));
    }

    #[test]
    fn intersecting_cone_end_caps() {
        struct TestCase<'a> {
            label: &'a str,
            origin: Point,
            direction: Vector,
            count: usize,
        }

        let mut cone = Cone::new();
        cone.minimum = -0.5;
        cone.maximum = 0.5;
        cone.closed = true;

        let test_cases = [
            TestCase {
                label: "miss caps",
                origin: Point::new(0.0, 0.0, -5.0),
                direction: Vector::new(0.0, 1.0, 0.0).normalize(),
                count: 0,
            },
            TestCase {
                label: "through both caps",
                origin: Point::new(0.0, 0.0, -0.25),
                direction: Vector::new(0.0, 1.0, 1.0).normalize(),
                count: 2,
            },
            TestCase {
                label: "hit both caps vertically",
                origin: Point::new(0.0, 0.0, -0.25),
                direction: Vector::new(0.0, 1.0, 0.0).normalize(),
                count: 4,
            },
        ];

        for test_case in test_cases {
            let ray = Ray::new(test_case.origin, test_case.direction);
            let xs = cone.local_intersect(ray);
            assert_eq!(
                xs.len(),
                test_case.count,
                "Unexpected hit count for {}",
                test_case.label
            );
        }
    }

    #[test]
    fn computing_normal_vector_on_cone() {
        struct TestCase<'a> {
            label: &'a str,
            point: Point,
            expected: Vector,
        }

        let cone = Cone::new();
        let sqrt2 = 2.0_f64.sqrt();

        let test_cases = [
            TestCase {
                label: "apex",
                point: Point::new(0.0, 0.0, 0.0),
                expected: Vector::new(0.0, 0.0, 0.0),
            },
            TestCase {
                label: "positive quadrant",
                point: Point::new(1.0, 1.0, 1.0),
                expected: Vector::new(1.0, -sqrt2, 1.0),
            },
            TestCase {
                label: "negative quadrant",
                point: Point::new(-1.0, -1.0, 0.0),
                expected: Vector::new(-1.0, 1.0, 0.0),
            },
        ];

        for test_case in test_cases {
            let normal = cone.local_normal_at(test_case.point);
            assert_eq!(
                normal, test_case.expected,
                "Unexpected normal for {}",
                test_case.label
            );
        }
    }
}
