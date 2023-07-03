use crate::matrices::Matrix4;
use crate::tuples::{Point, Vector};

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vector,
}

impl Ray {
    pub fn new(origin: Point, direction: Vector) -> Ray {
        Ray { origin, direction }
    }

    pub fn position(self, t: f64) -> Point {
        self.origin + self.direction * t
    }

    pub fn transform(self, matrix: Matrix4) -> Ray {
        Ray::new(matrix.clone() * self.origin, matrix * self.direction)
    }
}

#[cfg(test)]
mod tests {
    use crate::matrices::Matrix4;
    use crate::rays::Ray;
    use crate::tuples::{Point, Tuple, Vector};

    #[test]
    fn querying_a_ray() {
        let origin = Point::new(1.0, 2.0, 3.0);
        let direction = Vector::new(4.0, 5.0, 6.0);
        let ray = Ray::new(origin, direction);
        assert_eq!(ray.origin, origin);
        assert_eq!(ray.direction, direction);
    }

    #[test]
    fn compute_point_from_distance() {
        let ray = Ray::new(Point::new(2.0, 3.0, 4.0), Vector::new(1.0, 0.0, 0.0));
        assert_eq!(ray.position(0.0), Point::new(2.0, 3.0, 4.0));
        assert_eq!(ray.position(1.0), Point::new(3.0, 3.0, 4.0));
        assert_eq!(ray.position(-1.0), Point::new(1.0, 3.0, 4.0));
        assert_eq!(ray.position(2.5), Point::new(4.5, 3.0, 4.0));
    }

    #[test]
    fn translating_ray() {
        let ray = Ray::new(Point::new(1.0, 2.0, 3.0), Vector::new(0.0, 1.0, 0.0));
        let translation = Matrix4::translate(3.0, 4.0, 5.0);
        let translated_ray = ray.transform(translation);
        assert_eq!(translated_ray.origin, Point::new(4.0, 6.0, 8.0));
        assert_eq!(translated_ray.direction, Vector::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn scaling_a_ray() {
        let ray = Ray::new(Point::new(1.0, 2.0, 3.0), Vector::new(0.0, 1.0, 0.0));
        let scaling = Matrix4::scale(2.0, 3.0, 4.0);
        let scaled_ray = ray.transform(scaling);
        assert_eq!(scaled_ray.origin, Point::new(2.0, 6.0, 12.0));
        assert_eq!(scaled_ray.direction, Vector::new(0.0, 3.0, 0.0));
    }
}
