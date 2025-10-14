// Was running into issues with inversion of matrices, so added this to be a bit more lenient
pub const EPSILON: f64 = 0.00001;

pub fn float_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < EPSILON
}

#[cfg(test)]
mod test {
    #[test]
    fn float_equal() {
        assert!(super::float_equal(0.0, 0.0));
        assert!(super::float_equal(0.00001, 0.000011));
        assert!(super::float_equal(0.00001, 0.000009));
        assert!(!super::float_equal(0.00001, 0.00002));
    }
}
