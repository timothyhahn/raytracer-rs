use crate::matrices::Matrix;
use crate::tuples::Tuple;

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: Tuple,
    pub direction: Tuple,
}

impl Ray {
    pub fn new(origin: Tuple, direction: Tuple) -> Ray {
        Ray { origin, direction }
    }

    pub fn position(self, time: f64) -> Tuple {
        self.origin + self.direction * time
    }

    pub fn transform(self, matrix: Matrix) -> Ray {
        Ray::new(matrix.clone() * self.origin, matrix * self.direction)
    }
}

#[cfg(test)]
mod tests {
    use crate::matrices::Matrix;
    use crate::rays::Ray;
    use crate::tuples::Tuple;

    #[test]
    fn test_querying_a_ray() {
        let origin = Tuple::point(1.0, 2.0, 3.0);
        let direction = Tuple::vector(4.0, 5.0, 6.0);
        let ray = Ray::new(origin, direction);
        assert_eq!(ray.origin, origin);
        assert_eq!(ray.direction, direction);
    }

    #[test]
    fn test_compute_point_from_distance() {
        let ray = Ray::new(Tuple::point(2.0, 3.0, 4.0), Tuple::vector(1.0, 0.0, 0.0));
        assert_eq!(ray.position(0.0), Tuple::point(2.0, 3.0, 4.0));
        assert_eq!(ray.position(1.0), Tuple::point(3.0, 3.0, 4.0));
        assert_eq!(ray.position(-1.0), Tuple::point(1.0, 3.0, 4.0));
        assert_eq!(ray.position(2.5), Tuple::point(4.5, 3.0, 4.0));
    }

    #[test]
    fn test_translating_ray() {
        let ray = Ray::new(Tuple::point(1.0, 2.0, 3.0), Tuple::vector(0.0, 1.0, 0.0));
        let translation = Matrix::translation(3.0, 4.0, 5.0);
        let translated_ray = ray.transform(translation);
        assert_eq!(translated_ray.origin, Tuple::point(4.0, 6.0, 8.0));
        assert_eq!(translated_ray.direction, Tuple::vector(0.0, 1.0, 0.0));
    }

    #[test]
    fn test_scaling_a_ray() {
        let ray = Ray::new(Tuple::point(1.0, 2.0, 3.0), Tuple::vector(0.0, 1.0, 0.0));
        let scaling = Matrix::scaling(2.0, 3.0, 4.0);
        let scaled_ray = ray.transform(scaling);
        assert_eq!(scaled_ray.origin, Tuple::point(2.0, 6.0, 12.0));
        assert_eq!(scaled_ray.direction, Tuple::vector(0.0, 3.0, 0.0));
    }
}
