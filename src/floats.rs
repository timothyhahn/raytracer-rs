// Was running into issues with inversion of matrices, so added this to be a bit more lenient
const EPSILON: f64 = 0.00001;

pub fn float_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < EPSILON
}
