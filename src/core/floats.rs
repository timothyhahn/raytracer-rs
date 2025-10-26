// Increased to 0.0001 to match GPU and avoid shadow acne artifacts
pub const EPSILON: f64 = 0.0001;

pub fn float_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < EPSILON
}

#[cfg(test)]
mod test {
    #[test]
    fn float_equal() {
        assert!(super::float_equal(0.0, 0.0));
        // With EPSILON = 0.0001, values within 1e-4 are considered equal
        assert!(super::float_equal(0.0001, 0.00011)); // diff = 1e-5 < EPSILON
        assert!(super::float_equal(0.0001, 0.00009)); // diff = 1e-5 < EPSILON
        assert!(!super::float_equal(0.0001, 0.0003)); // diff = 2e-4 > EPSILON
    }
}
