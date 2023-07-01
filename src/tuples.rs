use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug)]
struct Tuple {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

impl Tuple {
    fn is_point(&self) -> bool {
        self.w == 1.0
    }

    fn is_vector(&self) -> bool {
        self.w == 0.0
    }

    fn new(x: f64, y: f64, z: f64, w: f64) -> Tuple {
        Tuple { x, y, z, w }
    }

    fn point(x: f64, y: f64, z: f64) -> Tuple {
        Tuple::new(x, y, z, 1.0)
    }

    fn vector(x: f64, y: f64, z: f64) -> Tuple {
        Tuple::new(x, y, z, 0.0)
    }

    fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2) + self.w.powi(2)).sqrt()
    }

    fn normalize(&self) -> Tuple {
        let magnitude = self.magnitude();
        Tuple::new(
            self.x / magnitude,
            self.y / magnitude,
            self.z / magnitude,
            self.w / magnitude,
        )
    }

    fn dot(&self, other: &Tuple) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    fn cross(&self, other: &Tuple) -> Tuple {
        Tuple::vector(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x
        )
    }
}

impl Add for Tuple {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z,
            self.w + other.w,
        )
    }
}

impl Sub for Tuple {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(
            self.x - other.x,
            self.y - other.y,
            self.z - other.z,
            self.w - other.w,
        )
    }
}

impl Neg for Tuple {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Tuple::vector(0.0, 0.0, 0.0) - self
    }
}

impl Mul<f64> for Tuple {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self::Output {
        Self::new(
            self.x * scalar,
            self.y * scalar,
            self.z * scalar,
            self.w * scalar,
        )
    }
}
impl Div<f64> for Tuple {
    type Output = Self;
    fn div(self, scalar: f64) -> Self::Output {
        Self::new(
            self.x / scalar,
            self.y / scalar,
            self.z / scalar,
            self.w / scalar,
        )
    }
}

impl PartialEq for Tuple {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z && self.w == other.w
    }
}

#[cfg(test)]
mod tests {
    use crate::tuples::Tuple;

    #[test]
    fn a_tuple_with_1_is_a_point() {
        let point = Tuple::new(4.3, -4.2, 3.1, 1.0);
        assert!(point.is_point());
        assert!(!point.is_vector());
    }

    #[test]
    fn tuple_point_creates_a_point() {
        let point = Tuple::point(4.3, -4.2, 3.1);
        assert!(point.is_point());
        assert!(!point.is_vector());
    }

    #[test]
    fn a_tuple_with_0_is_a_vector() {
        let vector = Tuple::new(4.3, -4.2, 3.1, 0.0);
        assert!(!vector.is_point());
        assert!(vector.is_vector());
    }

    #[test]
    fn tuple_vector_creates_a_vector() {
        let vector = Tuple::vector(4.3, -4.2, 3.1);
        assert!(!vector.is_point());
        assert!(vector.is_vector());
    }

    #[test]
    fn two_tuples_can_equal_be_compared() {
        let tuple1 = Tuple::new(4.3, -4.2, 3.1, 1.0);
        let tuple2 = Tuple::new(4.3, -4.2, 3.1, 1.0);
        assert_eq!(tuple1, tuple2);
    }

    #[test]
    fn a_point_can_be_added_to_a_vector() {
        let point = Tuple::point(3.0, -2.0, 5.0);
        let vector = Tuple::vector(-2.0, 3.0, 1.0);
        let result = point + vector;
        assert_eq!(result, Tuple::point(1.0, 1.0, 6.0));
        assert!(result.is_point());
    }

    #[test]
    fn a_point_subtracting_another_point_becomes_a_vector() {
        let point1 = Tuple::point(3.0, 2.0, 1.0);
        let point2 = Tuple::point(5.0, 6.0, 7.0);
        let result = point1 - point2;
        assert_eq!(result, Tuple::vector(-2.0, -4.0, -6.0));
        assert!(result.is_vector());
    }

    #[test]
    fn a_vector_can_be_subtracted_from_a_point() {
        let point = Tuple::point(3.0, 2.0, 1.0);
        let vector = Tuple::vector(5.0, 6.0, 7.0);
        let result = point - vector;
        assert_eq!(result, Tuple::point(-2.0, -4.0, -6.0));
        assert!(result.is_point());
    }

    #[test]
    fn a_vector_can_be_subtracted_from_a_vector() {
        let vector1 = Tuple::vector(3.0, 2.0, 1.0);
        let vector2 = Tuple::vector(5.0, 6.0, 7.0);
        let result = vector1 - vector2;
        assert_eq!(result, Tuple::vector(-2.0, -4.0, -6.0));
        assert!(result.is_vector());
    }

    #[test]
    fn a_tuple_can_be_negated() {
        let tuple = Tuple::new(1.0, -2.0, 3.0, -4.0);
        let result = -tuple;
        assert_eq!(result, Tuple::new(-1.0, 2.0, -3.0, 4.0));
    }

    #[test]
    fn a_tuple_can_be_multiplied_by_a_scalar() {
        let tuple = Tuple::new(1.0, -2.0, 3.0, -4.0);
        let result = tuple * 3.5;
        assert_eq!(result, Tuple::new(3.5, -7.0, 10.5, -14.0));
    }

    #[test]
    fn a_tuple_can_be_multiplied_by_a_fraction() {
        let tuple = Tuple::new(1.0, -2.0, 3.0, -4.0);
        let result = tuple * 0.5;
        assert_eq!(result, Tuple::new(0.5, -1.0, 1.5, -2.0));
    }

    #[test]
    fn a_tuple_can_be_divided_by_a_scalar() {
        let tuple = Tuple::new(1.0, -2.0, 3.0, -4.0);
        let result = tuple / 2.0;
        assert_eq!(result, Tuple::new(0.5, -1.0, 1.5, -2.0));
    }

    #[test]
    fn the_magnitude_of_vector_1_0_0_should_be_1() {
        let vector = Tuple::vector(1.0, 0.0, 0.0);
        assert_eq!(vector.magnitude(), 1.0);
    }

    #[test]
    fn the_magnitude_of_vector_0_1_0_should_be_1() {
        let vector = Tuple::vector(0.0, 1.0, 0.0);
        assert_eq!(vector.magnitude(), 1.0);
    }

    #[test]
    fn the_magnitude_of_vector_0_0_1_should_be_1() {
        let vector = Tuple::vector(0.0, 0.0, 1.0);
        assert_eq!(vector.magnitude(), 1.0);
    }

    #[test]
    fn the_magnitude_of_vector_1_2_3_should_be_root_14() {
        let vector = Tuple::vector(1.0, 2.0, 3.0);
        assert_eq!(vector.magnitude(), 14.0_f64.sqrt());
    }

    #[test]
    fn the_magnitude_of_vector_neg_1_neg_2_neg_3_should_be_root_14() {
        let vector = Tuple::vector(-1.0, -2.0, -3.0);
        assert_eq!(vector.magnitude(), 14.0_f64.sqrt());
    }

    #[test]
    fn normalizing_vector_4_0_0_should_give_1_0_0() {
        let vector = Tuple::vector(4.0, 0.0, 0.0);
        assert_eq!(vector.normalize(), Tuple::vector(1.0, 0.0, 0.0));
    }

    #[test]
    fn normalizing_vector_1_2_3_should_give_1_over_root_14_2_over_root_14_3_over_root_14() {
        let vector = Tuple::vector(1.0, 2.0, 3.0);
        assert_eq!(vector.normalize(), Tuple::vector(1.0 / 14.0_f64.sqrt(), 2.0 / 14.0_f64.sqrt(), 3.0 / 14.0_f64.sqrt()));
    }

    #[test]
    fn dot_product_of_two_vectors() {
        let tuple1 = Tuple::vector(1.0, 2.0, 3.0);
        let tuple2 = Tuple::vector(2.0, 3.0, 4.0);
        assert_eq!(tuple1.dot(&tuple2), 20.0);
    }

    #[test]
    fn cross_product_of_two_vectors() {
        let tuple1 = Tuple::vector(1.0, 2.0, 3.0);
        let tuple2 = Tuple::vector(2.0, 3.0, 4.0);
        assert_eq!(tuple1.cross(&tuple2), Tuple::vector(-1.0, 2.0, -1.0));
        assert_eq!(tuple2.cross(&tuple1), Tuple::vector(1.0, -2.0, 1.0));
    }
}
