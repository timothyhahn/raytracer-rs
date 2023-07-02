use crate::floats;
use std::ops::{Add, Div, Mul, Neg, Sub};

// Tuple struct, as well as operators. Equality is done using the EPSILON trick
#[derive(Debug, Clone, Copy)]
pub struct Tuple {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Tuple {
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Tuple {
        Tuple { x, y, z, w }
    }

    pub fn point(x: f64, y: f64, z: f64) -> Tuple {
        Tuple::new(x, y, z, 1.0)
    }

    pub fn vector(x: f64, y: f64, z: f64) -> Tuple {
        Tuple::new(x, y, z, 0.0)
    }

    pub fn color(x: f64, y: f64, z: f64) -> Color {
        Color::from(Tuple::new(x, y, z, 0.0))
    }

    pub fn is_point(&self) -> bool {
        self.w == 1.0
    }

    pub fn is_vector(&self) -> bool {
        self.w == 0.0
    }

    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2) + self.w.powi(2)).sqrt()
    }

    pub fn normalize(&self) -> Tuple {
        let magnitude = self.magnitude();
        Tuple::new(
            self.x / magnitude,
            self.y / magnitude,
            self.z / magnitude,
            self.w / magnitude,
        )
    }

    pub fn dot(&self, other: &Tuple) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    pub fn cross(&self, other: &Tuple) -> Tuple {
        Tuple::vector(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn reflect(&self, normal: &Tuple) -> Tuple {
        *self - *normal * 2.0 * self.dot(normal)
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
        floats::float_equal(self.x, other.x)
            && floats::float_equal(self.y, other.y)
            && floats::float_equal(self.z, other.z)
            && floats::float_equal(self.w, other.w)
    }
}

// Colors are essentially tuples with a different name and we don't care about the w value.
// For the most part, the impls here defer to the Tuple via conversion.
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

const COLOR_W: f64 = 0.0;

impl Color {
    pub fn new(red: f64, green: f64, blue: f64) -> Color {
        Color { red, green, blue }
    }

    pub fn black() -> Color {
        Color::new(0.0, 0.0, 0.0)
    }

    pub fn white() -> Color {
        Color::new(1.0, 1.0, 1.0)
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let self_tuple = Tuple::from(self);
        let other_tuple = Tuple::from(other);
        let result = self_tuple + other_tuple;
        Color::from(result)
    }
}

impl Sub for Color {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let self_tuple = Tuple::from(self);
        let other_tuple = Tuple::from(other);
        let result = self_tuple - other_tuple;
        Color::from(result)
    }
}

impl Mul<f64> for Color {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self::Output {
        let self_tuple = Tuple::from(self);
        let result = self_tuple * scalar;
        Color::from(result)
    }
}

// Hadamard product
// Does not defer to tuple impl, since the tuple impl can't do this.
impl Mul<Color> for Color {
    type Output = Self;
    fn mul(self, other: Color) -> Self::Output {
        Color::new(
            self.red * other.red,
            self.green * other.green,
            self.blue * other.blue,
        )
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        // This should be a relatively fast op?
        let self_tuple = Tuple::from(*self);
        let other_tuple = Tuple::from(*other);
        self_tuple == other_tuple
    }
}

// Conversions between tuples and Colors
impl From<Tuple> for Color {
    fn from(tuple: Tuple) -> Self {
        Color {
            red: tuple.x,
            green: tuple.y,
            blue: tuple.z,
        }
    }
}

impl From<Color> for Tuple {
    fn from(color: Color) -> Self {
        Tuple::new(color.red, color.green, color.blue, COLOR_W)
    }
}

#[cfg(test)]
mod tests {
    use crate::tuples::{Color, Tuple};

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
        assert_eq!(
            vector.normalize(),
            Tuple::vector(
                1.0 / 14.0_f64.sqrt(),
                2.0 / 14.0_f64.sqrt(),
                3.0 / 14.0_f64.sqrt(),
            )
        );
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

    #[test]
    fn colors_are_red_green_blue_tuples() {
        let color = Tuple::color(-0.5, 0.4, 1.7);
        assert_eq!(color.red, -0.5);
        assert_eq!(color.green, 0.4);
        assert_eq!(color.blue, 1.7);
    }

    #[test]
    fn colors_can_be_converted_from_tuples() {
        let tuple = Tuple::new(0.9, 0.6, 0.75, 1.0);
        let color = Color::from(tuple);
        assert_eq!(color.red, 0.9);
        assert_eq!(color.green, 0.6);
        assert_eq!(color.blue, 0.75);
    }

    #[test]
    fn colors_can_be_added() {
        let color1 = Color::new(0.9, 0.6, 0.75);
        let color2 = Color::new(0.7, 0.1, 0.25);
        let result = color1 + color2;
        assert_eq!(result, Color::new(1.6, 0.7, 1.0));
    }

    #[test]
    fn colors_can_be_subtracted() {
        let color1 = Color::new(0.9, 0.6, 0.75);
        let color2 = Color::new(0.7, 0.1, 0.25);
        let result = color1 - color2;
        assert_eq!(result, Color::new(0.2, 0.5, 0.5));
    }

    #[test]
    fn colors_can_be_multiplied_by_a_scalar() {
        let color = Color::new(0.2, 0.3, 0.4);
        let result = color * 2.0;
        assert_eq!(result, Color::new(0.4, 0.6, 0.8));
    }

    #[test]
    fn colors_can_be_multiplied_by_other_colors() {
        let color1 = Color::new(1.0, 0.2, 0.4);
        let color2 = Color::new(0.9, 1.0, 0.1);
        let result = color1 * color2;
        assert_eq!(result, Color::new(0.9, 0.2, 0.04));
    }

    #[test]
    fn reflecting_a_vector_approaching_at_45_degrees() {
        let vector = Tuple::vector(1.0, -1.0, 0.0);
        let normal = Tuple::vector(0.0, 1.0, 0.0);
        let reflect = vector.reflect(&normal);
        assert_eq!(reflect, Tuple::vector(1.0, 1.0, 0.0));
    }

    #[test]
    fn reflecting_vector_off_slanted_surface() {
        let vector = Tuple::vector(0.0, -1.0, 0.0);
        let normal = Tuple::vector(2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0, 0.0);
        let reflect = vector.reflect(&normal);
        assert_eq!(reflect, Tuple::vector(1.0, 0.0, 0.0));
    }
}
