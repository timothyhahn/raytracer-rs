use crate::{
    core::{
        floats::EPSILON,
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
pub struct Cube {
    pub transformation: Matrix4,
    pub world_transformation: Matrix4,
    pub material: Material,
    pub parent: Option<Weak<RefCell<Object>>>,
}

impl PartialEq for Cube {
    fn eq(&self, other: &Self) -> bool {
        self.transformation == other.transformation && self.material == other.material
        // Ignore parent for equality comparison
    }
}

impl Cube {
    pub fn new() -> Self {
        Cube {
            transformation: Matrix4::identity(),
            world_transformation: Matrix4::identity(),
            material: Material::default(),
            parent: None,
        }
    }
}

impl Cube {
    fn check_axis(&self, origin: f64, direction: f64) -> (f64, f64) {
        let tmin_numerator = -1.0 - origin;
        let tmax_numerator = 1.0 - origin;

        let (tmin, tmax) = if direction.abs() >= EPSILON {
            (tmin_numerator / direction, tmax_numerator / direction)
        } else {
            (
                tmin_numerator * f64::INFINITY,
                tmax_numerator * f64::INFINITY,
            )
        };

        if tmin > tmax {
            return (tmax, tmin);
        }

        (tmin, tmax)
    }
}

impl Shape for Cube {
    fn local_intersect(&self, ray: Ray) -> Vec<f64> {
        let (mut tmin, mut tmax) = self.check_axis(ray.origin.x, ray.direction.x);

        let (ytmin, ytmax) = self.check_axis(ray.origin.y, ray.direction.y);

        tmin = tmin.max(ytmin);
        tmax = tmax.min(ytmax);

        // Early exit if ray already misses after checking X and Y
        if tmin > tmax {
            return Vec::new();
        }

        let (ztmin, ztmax) = self.check_axis(ray.origin.z, ray.direction.z);
        tmin = tmin.max(ztmin);
        tmax = tmax.min(ztmax);

        // Final check after all three axes
        if tmin > tmax {
            Vec::new()
        } else {
            vec![tmin, tmax]
        }
    }

    fn local_normal_at(&self, point: Point) -> Vector {
        let maxc = point.x.abs().max(point.y.abs()).max(point.z.abs());

        if (maxc - point.x.abs()).abs() < EPSILON {
            Vector::new(point.x, 0.0, 0.0)
        } else if (maxc - point.y.abs()).abs() < EPSILON {
            Vector::new(0.0, point.y, 0.0)
        } else {
            Vector::new(0.0, 0.0, point.z)
        }
    }

    /// Get the bounding box for a unit cube (always -1 to 1 in all dimensions).
    fn bounds(&self) -> Bounds {
        Bounds::new(Point::new(-1.0, -1.0, -1.0), Point::new(1.0, 1.0, 1.0))
    }
}

impl Default for Cube {
    fn default() -> Self {
        Cube::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::tuples::{Point, Tuple, Vector},
        geometry::{cubes::Cube, shapes::Shape},
        rendering::rays::Ray,
    };

    #[test]
    fn ray_intersects_cube() {
        struct TestCase<'a> {
            label: &'a str,
            origin: Point,
            direction: Vector,
            t1: f64,
            t2: f64,
        }
        let cube = Cube::new();
        let test_cases = [
            TestCase {
                label: "+x",
                origin: Point::new(5.0, 0.5, 0.0),
                direction: Vector::new(-1.0, 0.0, 0.0),
                t1: 4.0,
                t2: 6.0,
            },
            TestCase {
                label: "-x",
                origin: Point::new(-5.0, 0.5, 0.0),
                direction: Vector::new(1.0, 0.0, 0.0),
                t1: 4.0,
                t2: 6.0,
            },
            TestCase {
                label: "+y",
                origin: Point::new(0.5, 5.0, 0.0),
                direction: Vector::new(0.0, -1.0, 0.0),
                t1: 4.0,
                t2: 6.0,
            },
            TestCase {
                label: "-y",
                origin: Point::new(0.5, -5.0, 0.0),
                direction: Vector::new(0.0, 1.0, 0.0),
                t1: 4.0,
                t2: 6.0,
            },
            TestCase {
                label: "+z",
                origin: Point::new(0.5, 0.0, 5.0),
                direction: Vector::new(0.0, 0.0, -1.0),
                t1: 4.0,
                t2: 6.0,
            },
            TestCase {
                label: "-z",
                origin: Point::new(0.5, 0.0, -5.0),
                direction: Vector::new(0.0, 0.0, 1.0),
                t1: 4.0,
                t2: 6.0,
            },
            TestCase {
                label: "inside",
                origin: Point::new(0.0, 0.5, 0.0),
                direction: Vector::new(0.0, 0.0, 1.0),
                t1: -1.0,
                t2: 1.0,
            },
        ];

        test_cases.iter().for_each(|test_case| {
            let ray = Ray::new(test_case.origin, test_case.direction);
            let intersections = cube.local_intersect(ray);
            assert_eq!(intersections.len(), 2);
            let t1 = intersections[0];
            let t2 = intersections[1];
            assert_eq!(t1, test_case.t1, "Failed t1 for {}", test_case.label);
            assert_eq!(t2, test_case.t2, "Failed t2 for {}", test_case.label);
        });
    }

    #[test]
    fn ray_misses_a_cube() {
        struct TestCase {
            origin: Point,
            direction: Vector,
        }
        let cube = Cube::new();
        let test_cases = [
            TestCase {
                origin: Point::new(-2.0, 0.0, 0.0),
                direction: Vector::new(0.2673, 0.5345, 0.8018),
            },
            TestCase {
                origin: Point::new(0.0, -2.0, 0.0),
                direction: Vector::new(0.8108, 0.2673, 0.5345),
            },
            TestCase {
                origin: Point::new(0.0, 0.0, -2.0),
                direction: Vector::new(0.5345, 0.8018, 0.2673),
            },
            TestCase {
                origin: Point::new(2.0, 0.0, 2.0),
                direction: Vector::new(0.0, 0.0, -1.0),
            },
            TestCase {
                origin: Point::new(0.0, 2.0, 2.0),
                direction: Vector::new(0.0, -1.0, 0.0),
            },
            TestCase {
                origin: Point::new(2.0, 2.0, 0.0),
                direction: Vector::new(-1.0, 0.0, 0.0),
            },
        ];
        test_cases.iter().for_each(|test_case| {
            let ray = Ray::new(test_case.origin, test_case.direction);
            let intersections = cube.local_intersect(ray);
            assert_eq!(intersections.len(), 0);
        });
    }

    #[test]
    fn normal_on_surface_of_cube() {
        struct TestCase {
            point: Point,
            normal: Vector,
        }
        let cube = Cube::new();
        let test_cases = [
            TestCase {
                point: Point::new(1.0, 0.5, -0.8),
                normal: Vector::new(1.0, 0.0, 0.0),
            },
            TestCase {
                point: Point::new(-1.0, -0.2, 0.9),
                normal: Vector::new(-1.0, 0.0, 0.0),
            },
            TestCase {
                point: Point::new(-0.4, 1.0, -0.1),
                normal: Vector::new(0.0, 1.0, 0.0),
            },
            TestCase {
                point: Point::new(0.3, -1.0, -0.7),
                normal: Vector::new(0.0, -1.0, 0.0),
            },
            TestCase {
                point: Point::new(-0.6, 0.3, 1.0),
                normal: Vector::new(0.0, 0.0, 1.0),
            },
            TestCase {
                point: Point::new(0.4, 0.4, -1.0),
                normal: Vector::new(0.0, 0.0, -1.0),
            },
            TestCase {
                point: Point::new(1.0, 1.0, 1.0),
                normal: Vector::new(1.0, 0.0, 0.0),
            },
            TestCase {
                point: Point::new(-1.0, -1.0, -1.0),
                normal: Vector::new(-1.0, 0.0, 0.0),
            },
        ];

        test_cases.iter().for_each(|test_case| {
            let normal = cube.local_normal_at(test_case.point);
            assert_eq!(normal, test_case.normal);
        });
    }
}
