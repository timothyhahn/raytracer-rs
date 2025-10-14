use super::floats::float_equal;
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl Color {
    pub const BLACK: Self = Self {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };
    pub const WHITE: Self = Self {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
    };

    pub const fn new(red: f64, green: f64, blue: f64) -> Color {
        Color { red, green, blue }
    }

    pub const fn black() -> Color {
        Self::BLACK
    }

    pub const fn white() -> Color {
        Self::WHITE
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Color::new(
            self.red + other.red,
            self.green + other.green,
            self.blue + other.blue,
        )
    }
}

impl Sub for Color {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Color::new(
            self.red - other.red,
            self.green - other.green,
            self.blue - other.blue,
        )
    }
}

impl Mul<f64> for Color {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self::Output {
        Color::new(self.red * scalar, self.green * scalar, self.blue * scalar)
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
        float_equal(self.red, other.red)
            && float_equal(self.green, other.green)
            && float_equal(self.blue, other.blue)
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn colors_are_red_green_blue_tuples() {
        let color = Color::new(-0.5, 0.4, 1.7);
        assert_eq!(color.red, -0.5);
        assert_eq!(color.green, 0.4);
        assert_eq!(color.blue, 1.7);
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
}
