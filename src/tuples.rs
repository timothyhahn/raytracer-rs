use crate::floats;
use std::ops::{Add, Div, Mul, Neg, Sub};

pub trait Tuple {
    fn new(x: f64, y: f64, z: f64) -> Self;
    fn zero() -> Self;

    fn x(&self) -> f64;
    fn y(&self) -> f64;
    fn z(&self) -> f64;
    fn w(&self) -> f64;
}

#[derive(Debug, Clone, Copy)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector {
    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    pub fn normalize(&self) -> Vector {
        *self / self.magnitude()
    }

    pub fn cross(&self, other: &Vector) -> Vector {
        Vector::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn reflect(&self, normal: &Vector) -> Vector {
        *self - *normal * 2.0 * self.dot(normal)
    }

    pub fn dot(&self, other: &Vector) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl Tuple for Vector {
    fn new(x: f64, y: f64, z: f64) -> Vector {
        Vector { x, y, z }
    }

    fn zero() -> Vector {
        Vector::new(0.0, 0.0, 0.0)
    }

    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }

    fn z(&self) -> f64 {
        self.z
    }

    fn w(&self) -> f64 {
        0.0
    }
}

impl Add for Vector {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z,
        )
    }
}

impl Sub for Vector {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(
            self.x - other.x,
            self.y - other.y,
            self.z - other.z,
        )
    }
}

impl Neg for Vector {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(0.0, 0.0, 0.0) - self
    }
}

impl Mul<f64> for Vector {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self::Output {
        Self::new(
            self.x * scalar,
            self.y * scalar,
            self.z * scalar,
        )
    }
}

impl Div<f64> for Vector {
    type Output = Self;
    fn div(self, scalar: f64) -> Self::Output {
        Self::new(
            self.x / scalar,
            self.y / scalar,
            self.z / scalar,
        )
    }
}

impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        floats::float_equal(self.x, other.x)
            && floats::float_equal(self.y, other.y)
            && floats::float_equal(self.z, other.z)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Tuple for Point {
    fn new(x: f64, y: f64, z: f64) -> Point {
        Point { x, y, z }
    }

    fn zero() -> Point {
        Point::new(0.0, 0.0, 0.0)
    }

    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }

    fn z(&self) -> f64 {
        self.z
    }

    fn w(&self) -> f64 {
        1.0
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z,
        )
    }
}

impl Add<Vector> for Point {
    type Output = Self;

    fn add(self, other: Vector) -> Point {
        Self::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z,
        )
    }
}

impl Sub for Point {
    type Output = Vector;

    fn sub(self, other: Self) -> Vector {
        Vector::new(
            self.x - other.x,
            self.y - other.y,
            self.z - other.z,
        )
    }
}

impl Sub<Vector> for Point {
    type Output = Self;

    fn sub(self, other: Vector) -> Self {
        Self::new(
            self.x - other.x,
            self.y - other.y,
            self.z - other.z,
        )
    }
}

impl Neg for Point {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Point::new(
            -self.x,
            -self.y,
            -self.z,
        )
    }
}

impl Mul<f64> for Point {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self::Output {
        Self::new(
            self.x * scalar,
            self.y * scalar,
            self.z * scalar,
        )
    }
}

impl Div<f64> for Point {
    type Output = Self;
    fn div(self, scalar: f64) -> Self::Output {
        Self::new(
            self.x / scalar,
            self.y / scalar,
            self.z / scalar,
        )
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        floats::float_equal(self.x, other.x)
            && floats::float_equal(self.y, other.y)
            && floats::float_equal(self.z, other.z)
    }
}

#[cfg(test)]
mod tests {
    use crate::floats::float_equal;
    use crate::tuples::{Point, Tuple, Vector};

    #[test]
    fn a_tuple_with_1_is_a_point() {
        let point = Point::new(4.3, -4.2, 3.1);
        assert!(float_equal(point.x(), 4.3));
        assert!(float_equal(point.y(), -4.2));
        assert!(float_equal(point.z(), 3.1));
        assert!(float_equal(point.w(), 1.0));
    }

    #[test]
    fn a_tuple_with_0_is_a_vector() {
        let vector = Vector::new(4.3, -4.2, 3.1);
        assert!(float_equal(vector.x(), 4.3));
        assert!(float_equal(vector.y(), -4.2));
        assert!(float_equal(vector.z(), 3.1));
        assert!(float_equal(vector.w(), 0.0));
    }

    #[test]
    fn two_vectors_can_equal_be_compared() {
        let tuple1 = Vector::new(4.3, -4.2, 3.1);
        let tuple2 = Vector::new(4.3, -4.2, 3.1);
        assert_eq!(tuple1, tuple2);
    }

    #[test]
    fn two_points_can_equal_be_compared() {
        let tuple1 = Point::new(4.3, -4.2, 3.1);
        let tuple2 = Point::new(4.3, -4.2, 3.1);
        assert_eq!(tuple1, tuple2);
    }

    #[test]
    fn a_point_can_be_added_to_a_vector() {
        let point = Point::new(3.0, -2.0, 5.0);
        let vector = Vector::new(-2.0, 3.0, 1.0);
        let result = point + vector;
        assert_eq!(result, Point::new(1.0, 1.0, 6.0));
    }

    #[test]
    fn a_point_subtracting_another_point_becomes_a_vector() {
        let point1 = Point::new(3.0, 2.0, 1.0);
        let point2 = Point::new(5.0, 6.0, 7.0);
        let result = point1 - point2;
        assert_eq!(result, Vector::new(-2.0, -4.0, -6.0));
    }

    #[test]
    fn a_vector_can_be_subtracted_from_a_point() {
        let point = Point::new(3.0, 2.0, 1.0);
        let vector = Vector::new(5.0, 6.0, 7.0);
        let result = point - vector;
        assert_eq!(result, Point::new(-2.0, -4.0, -6.0));
    }

    #[test]
    fn a_vector_can_be_subtracted_from_a_vector() {
        let vector1 = Vector::new(3.0, 2.0, 1.0);
        let vector2 = Vector::new(5.0, 6.0, 7.0);
        let result = vector1 - vector2;
        assert_eq!(result, Vector::new(-2.0, -4.0, -6.0));
    }

    #[test]
    fn a_vector_can_be_negated() {
        let tuple = Vector::new(1.0, -2.0, 3.0);
        let result = -tuple;
        assert_eq!(result, Vector::new(-1.0, 2.0, -3.0));
        assert_eq!(result.w(), 0.0);
    }

    #[test]
    fn a_point_can_be_negated() {
        let tuple = Point::new(1.0, -2.0, 3.0);
        let result = -tuple;
        assert_eq!(result, Point::new(-1.0, 2.0, -3.0));
        assert_eq!(result.w(), 1.0);
    }

    #[test]
    fn a_vector_can_be_multiplied_by_a_scalar() {
        let tuple = Vector::new(1.0, -2.0, 3.0);
        let result = tuple * 3.5;
        assert_eq!(result, Vector::new(3.5, -7.0, 10.5));
        assert_eq!(result.w(), 0.0);
    }

    #[test]
    fn a_point_can_be_multiplied_by_a_scalar() {
        let tuple = Point::new(1.0, -2.0, 3.0);
        let result = tuple * 3.5;
        assert_eq!(result, Point::new(3.5, -7.0, 10.5));
        assert_eq!(result.w(), 1.0);
    }

    #[test]
    fn a_vector_can_be_multiplied_by_a_fraction() {
        let tuple = Vector::new(1.0, -2.0, 3.0);
        let result = tuple * 0.5;
        assert_eq!(result, Vector::new(0.5, -1.0, 1.5));
        assert_eq!(result.w(), 0.0);
    }

    #[test]
    fn a_point_can_be_multiplied_by_a_fraction() {
        let tuple = Point::new(1.0, -2.0, 3.0);
        let result = tuple * 0.5;
        assert_eq!(result, Point::new(0.5, -1.0, 1.5));
        assert_eq!(result.w(), 1.0);
    }

    #[test]
    fn a_vector_can_be_divided_by_a_scalar() {
        let tuple = Vector::new(1.0, -2.0, 3.0);
        let result = tuple / 2.0;
        assert_eq!(result, Vector::new(0.5, -1.0, 1.5));
        assert_eq!(result.w(), 0.0);
    }

    #[test]
    fn a_point_can_be_divided_by_a_scalar() {
        let tuple = Point::new(1.0, -2.0, 3.0);
        let result = tuple / 2.0;
        assert_eq!(result, Point::new(0.5, -1.0, 1.5));
        assert_eq!(result.w(), 1.0);
    }

    #[test]
    fn the_magnitude_of_vector_1_0_0_should_be_1() {
        let vector = Vector::new(1.0, 0.0, 0.0);
        assert_eq!(vector.magnitude(), 1.0);
    }

    #[test]
    fn the_magnitude_of_vector_0_1_0_should_be_1() {
        let vector = Vector::new(0.0, 1.0, 0.0);
        assert_eq!(vector.magnitude(), 1.0);
    }

    #[test]
    fn the_magnitude_of_vector_0_0_1_should_be_1() {
        let vector = Vector::new(0.0, 0.0, 1.0);
        assert_eq!(vector.magnitude(), 1.0);
    }

    #[test]
    fn the_magnitude_of_vector_1_2_3_should_be_root_14() {
        let vector = Vector::new(1.0, 2.0, 3.0);
        assert_eq!(vector.magnitude(), 14.0_f64.sqrt());
    }

    #[test]
    fn the_magnitude_of_vector_neg_1_neg_2_neg_3_should_be_root_14() {
        let vector = Vector::new(-1.0, -2.0, -3.0);
        assert_eq!(vector.magnitude(), 14.0_f64.sqrt());
    }

    #[test]
    fn normalizing_vector_4_0_0_should_give_1_0_0() {
        let vector = Vector::new(4.0, 0.0, 0.0);
        assert_eq!(vector.normalize(), Vector::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn normalizing_vector_1_2_3_should_give_1_over_root_14_2_over_root_14_3_over_root_14() {
        let vector = Vector::new(1.0, 2.0, 3.0);
        assert_eq!(
            vector.normalize(),
            Vector::new(
                1.0 / 14.0_f64.sqrt(),
                2.0 / 14.0_f64.sqrt(),
                3.0 / 14.0_f64.sqrt(),
            )
        );
    }

    #[test]
    fn dot_product_of_two_vectors() {
        let tuple1 = Vector::new(1.0, 2.0, 3.0);
        let tuple2 = Vector::new(2.0, 3.0, 4.0);
        assert_eq!(tuple1.dot(&tuple2), 20.0);
    }

    #[test]
    fn cross_product_of_two_vectors() {
        let tuple1 = Vector::new(1.0, 2.0, 3.0);
        let tuple2 = Vector::new(2.0, 3.0, 4.0);
        assert_eq!(tuple1.cross(&tuple2), Vector::new(-1.0, 2.0, -1.0));
        assert_eq!(tuple2.cross(&tuple1), Vector::new(1.0, -2.0, 1.0));
    }

    #[test]
    fn reflecting_a_vector_approaching_at_45_degrees() {
        let vector = Vector::new(1.0, -1.0, 0.0);
        let normal = Vector::new(0.0, 1.0, 0.0);
        let reflect = vector.reflect(&normal);
        assert_eq!(reflect, Vector::new(1.0, 1.0, 0.0));
    }

    #[test]
    fn reflecting_vector_off_slanted_surface() {
        let vector = Vector::new(0.0, -1.0, 0.0);
        let normal = Vector::new(2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0, 0.0);
        let reflect = vector.reflect(&normal);
        assert_eq!(reflect, Vector::new(1.0, 0.0, 0.0));
    }
}
