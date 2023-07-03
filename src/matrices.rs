use crate::floats::float_equal;
use crate::tuples::{Point, Tuple, Vector};
use std::ops::Mul;

// Most things rely on Matrix4, everything else is used by Matrix2 for things like cofactors.
#[derive(Debug, Copy, Clone)]
pub struct Matrix4 {
    data: [[f64; 4]; 4],
}

impl Matrix4 {
    pub fn new(data: [[f64; 4]; 4]) -> Matrix4 {
        Matrix4 { data }
    }

    // Transformation matrices
    pub fn identity() -> Matrix4 {
        Matrix4 {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn translate(x: f64, y: f64, z: f64) -> Matrix4 {
        Matrix4 {
            data: [
                [1.0, 0.0, 0.0, x],
                [0.0, 1.0, 0.0, y],
                [0.0, 0.0, 1.0, z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn scale(x: f64, y: f64, z: f64) -> Matrix4 {
        Matrix4 {
            data: [
                [x, 0.0, 0.0, 0.0],
                [0.0, y, 0.0, 0.0],
                [0.0, 0.0, z, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotate_x(r: f64) -> Matrix4 {
        Matrix4 {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, r.cos(), -r.sin(), 0.0],
                [0.0, r.sin(), r.cos(), 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotate_y(r: f64) -> Matrix4 {
        Matrix4 {
            data: [
                [r.cos(), 0.0, r.sin(), 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-r.sin(), 0.0, r.cos(), 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotate_z(r: f64) -> Matrix4 {
        Matrix4 {
            data: [
                [r.cos(), -r.sin(), 0.0, 0.0],
                [r.sin(), r.cos(), 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn shear(xy: f64, xz: f64, yx: f64, yz: f64, zx: f64, zy: f64) -> Matrix4 {
        Matrix4 {
            data: [
                [1.0, xy, xz, 0.0],
                [yx, 1.0, yz, 0.0],
                [zx, zy, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn determinant(&self) -> f64 {
        let mut result = 0.0;
        for column in 0..4 {
            result += self.data[0][column] * self.cofactor(0, column);
        }
        result
    }

    #[allow(clippy::needless_range_loop)]
    pub fn submatrix(&self, row: usize, col: usize) -> Matrix3 {
        let mut data = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                let row_offset = if i >= row { 1 } else { 0 };
                let col_offset = if j >= col { 1 } else { 0 };
                data[i][j] = self.data[i + row_offset][j + col_offset];
            }
        }
        Matrix3::new(data)
    }

    #[allow(clippy::needless_range_loop)]
    pub fn inverse(&self) -> Option<Matrix4> {
        if !self.is_invertible() {
            return None;
        }

        let mut data = [[0.0; 4]; 4];

        for row in 0..4 {
            for col in 0..4 {
                let c = self.cofactor(row, col);

                // Transpose here by swapping row/col
                data[col][row] = c / self.determinant();
            }
        }
        Some(Matrix4::new(data))
    }

    // Returns a new Matrix since we need the old values when calculating the output
    pub fn transpose(&self) -> Matrix4 {
        let mut matrix = Matrix4::default();
        for row in 0..4 {
            for col in 0..4 {
                matrix.data[col][row] = self.data[row][col];
            }
        }
        matrix
    }

    pub fn is_invertible(&self) -> bool {
        !float_equal(self.determinant(), 0.0)
    }

    pub fn cofactor(&self, row: usize, col: usize) -> f64 {
        let minor = self.minor(row, col);
        if (row + col) % 2 == 0 {
            minor
        } else {
            -minor
        }
    }

    pub fn minor(&self, row: usize, col: usize) -> f64 {
        self.submatrix(row, col).determinant()
    }
}

impl Default for Matrix4 {
    fn default() -> Self {
        Matrix4::new([[0.0; 4]; 4])
    }
}

impl Mul for Matrix4 {
    type Output = Matrix4;

    // Creates a new Matrix, since we need the old values when calculating the output
    // Might be doable in-place?
    // Allowing since having separate idx and col/row variables doesn't make things cleaner
    #[allow(clippy::needless_range_loop)]
    fn mul(self, other: Self) -> Matrix4 {
        let mut data = [[0.0; 4]; 4];

        for row in 0..4 {
            for col in 0..4 {
                let mut sum = 0.0;
                for i in 0..4 {
                    sum += self.data[row][i] * other.data[i][col];
                }
                data[row][col] = sum;
            }
        }

        Matrix4 { data }
    }
}

impl Mul<Vector> for Matrix4 {
    type Output = Vector;

    fn mul(self, other: Vector) -> Vector {
        let mut data = [0.0; 4];

        for (idx, row) in self.data.iter().enumerate() {
            data[idx] = row[0] * other.x + row[1] * other.y + row[2] * other.z + row[3] * other.w();
        }

        Vector::new(data[0], data[1], data[2])
    }
}

impl Mul<Point> for Matrix4 {
    type Output = Point;

    fn mul(self, other: Point) -> Point {
        let mut data = [0.0; 4];

        for (idx, row) in self.data.iter().enumerate() {
            data[idx] = row[0] * other.x + row[1] * other.y + row[2] * other.z + row[3] * other.w();
        }

        Point::new(data[0], data[1], data[2])
    }
}

impl PartialEq for Matrix4 {
    fn eq(&self, other: &Self) -> bool {
        for row in 0..4 {
            for col in 0..4 {
                if !float_equal(self.data[row][col], other.data[row][col]) {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Matrix3 {
    data: [[f64; 3]; 3],
}

impl Matrix3 {
    pub fn new(data: [[f64; 3]; 3]) -> Matrix3 {
        Matrix3 { data }
    }

    pub fn cofactor(&self, row: usize, col: usize) -> f64 {
        let minor = self.minor(row, col);
        if (row + col) % 2 == 0 {
            minor
        } else {
            -minor
        }
    }

    pub fn minor(&self, row: usize, col: usize) -> f64 {
        self.submatrix(row, col).determinant()
    }

    pub fn determinant(&self) -> f64 {
        let mut result = 0.0;
        for column in 0..3 {
            result += self.data[0][column] * self.cofactor(0, column);
        }
        result
    }

    #[allow(clippy::needless_range_loop)]
    pub fn submatrix(&self, row: usize, col: usize) -> Matrix2 {
        let mut data = [[0.0; 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                let row_offset = if i >= row { 1 } else { 0 };
                let col_offset = if j >= col { 1 } else { 0 };
                data[i][j] = self.data[i + row_offset][j + col_offset];
            }
        }
        Matrix2::new(data)
    }
}

impl Default for Matrix3 {
    fn default() -> Self {
        Matrix3::new([[0.0; 3]; 3])
    }
}

impl PartialEq for Matrix3 {
    fn eq(&self, other: &Self) -> bool {
        for row in 0..3 {
            for col in 0..3 {
                if !float_equal(self.data[row][col], other.data[row][col]) {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Matrix2 {
    data: [[f64; 2]; 2],
}

impl Matrix2 {
    pub fn new(data: [[f64; 2]; 2]) -> Matrix2 {
        Matrix2 { data }
    }

    pub fn determinant(self) -> f64 {
        self.data[0][0] * self.data[1][1] - self.data[0][1] * self.data[1][0]
    }
}

impl Default for Matrix2 {
    fn default() -> Self {
        Matrix2::new([[0.0; 2]; 2])
    }
}

impl PartialEq for Matrix2 {
    fn eq(&self, other: &Self) -> bool {
        for row in 0..2 {
            for col in 0..2 {
                if !float_equal(self.data[row][col], other.data[row][col]) {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::matrices::{Matrix2, Matrix3, Matrix4};
    use crate::tuples::{Point, Tuple, Vector};
    use std::f64::consts::PI;

    // First since this is the most used type of matrix.
    #[test]
    fn create_4x4_matrix() {
        let data = [
            [1.0, 2.0, 3.0, 4.0],
            [5.5, 6.5, 7.5, 8.5],
            [9.0, 10.0, 11.0, 12.0],
            [13.5, 14.5, 15.5, 16.5],
        ];
        let matrix = Matrix4::new(data);

        assert_eq!(matrix.data[0][0], 1.0);
        assert_eq!(matrix.data[0][3], 4.0);
        assert_eq!(matrix.data[1][0], 5.5);
        assert_eq!(matrix.data[1][2], 7.5);
        assert_eq!(matrix.data[2][2], 11.0);
        assert_eq!(matrix.data[3][0], 13.5);
        assert_eq!(matrix.data[3][2], 15.5);
    }

    #[test]
    fn create_2x2_matrix() {
        let data = [[-3.0, 5.0], [1.0, -2.0]];
        let matrix = Matrix2::new(data);

        assert_eq!(matrix.data[0][0], -3.0);
        assert_eq!(matrix.data[0][1], 5.0);
        assert_eq!(matrix.data[1][0], 1.0);
        assert_eq!(matrix.data[1][1], -2.0);
    }

    #[test]
    fn create_3x3_matrix() {
        let data = [[-3.0, 5.0, 0.0], [1.0, -2.0, -7.0], [0.0, 1.0, 1.0]];
        let matrix = Matrix3::new(data);

        assert_eq!(matrix.data[0][0], -3.0);
        assert_eq!(matrix.data[1][1], -2.0);
        assert_eq!(matrix.data[2][2], 1.0);
    }

    #[test]
    fn equality_with_identical_matrices() {
        let matrix_a = Matrix4::new([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ]);

        let matrix_b = Matrix4::new([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ]);

        assert_eq!(matrix_a, matrix_b);
    }

    #[test]
    fn inequality_with_similar_matrices() {
        let matrix_a = Matrix4::new([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ]);

        let matrix_b = Matrix4::new([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 2.0, 2.0],
        ]);

        assert_ne!(matrix_a, matrix_b);
    }

    #[test]
    fn multiply_two_matrices() {
        let matrix_a = Matrix4::new([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ]);

        let matrix_b = Matrix4::new([
            [-2.0, 1.0, 2.0, 3.0],
            [3.0, 2.0, 1.0, -1.0],
            [4.0, 3.0, 6.0, 5.0],
            [1.0, 2.0, 7.0, 8.0],
        ]);

        let expected = Matrix4::new([
            [20.0, 22.0, 50.0, 48.0],
            [44.0, 54.0, 114.0, 108.0],
            [40.0, 58.0, 110.0, 102.0],
            [16.0, 26.0, 46.0, 42.0],
        ]);

        assert_eq!(matrix_a * matrix_b, expected);
    }

    #[test]
    fn multiply_matrix_by_tuple() {
        let matrix = Matrix4::new([
            [1.0, 2.0, 3.0, 4.0],
            [2.0, 4.0, 4.0, 2.0],
            [8.0, 6.0, 4.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        let tuple = Point::new(1.0, 2.0, 3.0);

        assert_eq!(matrix * tuple, Point::new(18.0, 24.0, 33.0));
    }

    #[test]
    fn multiply_by_identity_matrix() {
        let matrix = Matrix4::new([
            [0.0, 1.0, 2.0, 4.0],
            [1.0, 2.0, 4.0, 8.0],
            [2.0, 4.0, 8.0, 16.0],
            [4.0, 8.0, 16.0, 32.0],
        ]);

        let identity_matrix = Matrix4::identity();

        assert_eq!(matrix * identity_matrix, matrix);
    }

    #[test]
    fn multiplying_identity_matrix_by_tuple() {
        let tuple = Vector::new(1.0, 2.0, 3.0);
        let identity_matrix = Matrix4::identity();

        assert_eq!(identity_matrix * tuple, tuple);
    }

    #[test]
    fn transposing_a_matrix() {
        let matrix = Matrix4::new([
            [0.0, 9.0, 3.0, 0.0],
            [9.0, 8.0, 0.0, 8.0],
            [1.0, 8.0, 5.0, 3.0],
            [0.0, 0.0, 5.0, 8.0],
        ]);

        let expected_matrix = Matrix4::new([
            [0.0, 9.0, 1.0, 0.0],
            [9.0, 8.0, 8.0, 0.0],
            [3.0, 0.0, 5.0, 5.0],
            [0.0, 8.0, 3.0, 8.0],
        ]);

        assert_eq!(matrix.transpose(), expected_matrix);
    }

    #[test]
    fn calculating_determinant_of_2x2_matrix() {
        let matrix = Matrix2::new([[1.0, 5.0], [-3.0, 2.0]]);

        assert_eq!(matrix.determinant(), 17.0);
    }

    #[test]
    fn calculate_the_determinant_of_a_3x3_matrix() {
        let matrix = Matrix3::new([[1.0, 2.0, 6.0], [-5.0, 8.0, -4.0], [2.0, 6.0, 4.0]]);

        assert_eq!(matrix.cofactor(0, 0), 56.0);
        assert_eq!(matrix.cofactor(0, 1), 12.0);
        assert_eq!(matrix.cofactor(0, 2), -46.0);
        assert_eq!(matrix.determinant(), -196.0);
    }

    #[test]
    fn calculate_the_determinant_of_a_4x4_matrix() {
        let matrix = Matrix4::new([
            [-2.0, -8.0, 3.0, 5.0],
            [-3.0, 1.0, 7.0, 3.0],
            [1.0, 2.0, -9.0, 6.0],
            [-6.0, 7.0, 7.0, -9.0],
        ]);

        assert_eq!(matrix.cofactor(0, 0), 690.0);
        assert_eq!(matrix.cofactor(0, 1), 447.0);
        assert_eq!(matrix.cofactor(0, 2), 210.0);
        assert_eq!(matrix.cofactor(0, 3), 51.0);
        assert_eq!(matrix.determinant(), -4071.0);
    }

    #[test]
    fn get_2x2_submatrix_from_3x3_matrix() {
        let matrix = Matrix3::new([[1.0, 5.0, 0.0], [-3.0, 2.0, 7.0], [0.0, 6.0, -3.0]]);

        let expected_matrix = Matrix2::new([[-3.0, 2.0], [0.0, 6.0]]);

        assert_eq!(matrix.submatrix(0, 2), expected_matrix);
    }

    #[test]
    fn get_3x3_submatrix_from_4x4_matrix() {
        let matrix = Matrix4::new([
            [-6.0, 1.0, 1.0, 6.0],
            [-8.0, 5.0, 8.0, 6.0],
            [-1.0, 0.0, 8.0, 2.0],
            [-7.0, 1.0, -1.0, 1.0],
        ]);

        let expected_matrix = Matrix3::new([[-6.0, 1.0, 6.0], [-8.0, 8.0, 6.0], [-7.0, -1.0, 1.0]]);

        assert_eq!(matrix.submatrix(2, 1), expected_matrix);
    }

    #[test]
    fn calculate_minor_of_3x3_matrix() {
        let matrix = Matrix3::new([[3.0, 5.0, 0.0], [2.0, -1.0, -7.0], [6.0, -1.0, 5.0]]);

        assert_eq!(matrix.minor(1, 0), 25.0);
    }

    #[test]
    fn calculate_cofactor_of_3x3_matrix() {
        let matrix = Matrix3::new([[3.0, 5.0, 0.0], [2.0, -1.0, -7.0], [6.0, -1.0, 5.0]]);

        assert_eq!(matrix.minor(0, 0), -12.0);
        assert_eq!(matrix.cofactor(0, 0), -12.0);
        assert_eq!(matrix.minor(1, 0), 25.0);
        assert_eq!(matrix.cofactor(1, 0), -25.0);
    }

    #[test]
    fn invertible_matrix() {
        let matrix = Matrix4::new([
            [6.0, 4.0, 4.0, 4.0],
            [5.0, 5.0, 7.0, 6.0],
            [4.0, -9.0, 3.0, -7.0],
            [9.0, 1.0, 7.0, -6.0],
        ]);
        assert_eq!(matrix.determinant(), -2120.0);
        assert!(matrix.is_invertible());
    }

    #[test]
    fn noninvertible_matrix() {
        let matrix = Matrix4::new([
            [-4.0, 2.0, -2.0, -3.0],
            [9.0, 6.0, 2.0, 6.0],
            [0.0, -5.0, 1.0, -5.0],
            [0.0, 0.0, 0.0, 0.0],
        ]);
        assert_eq!(matrix.determinant(), 0.0);
        assert!(!matrix.is_invertible());
    }

    #[test]
    fn invert_matrix() {
        let matrix = Matrix4::new([
            [-5.0, 2.0, 6.0, -8.0],
            [1.0, -5.0, 1.0, 8.0],
            [7.0, 7.0, -6.0, -7.0],
            [1.0, -3.0, 7.0, 4.0],
        ]);

        let expected_matrix = Matrix4::new([
            [
                0.21804511278195488,
                0.45112781954887216,
                0.24060150375939848,
                -0.045112781954887216,
            ],
            [
                -0.8082706766917294,
                -1.4567669172932332,
                -0.44360902255639095,
                0.5206766917293233,
            ],
            [
                -0.07894736842105263,
                -0.2236842105263158,
                -0.05263157894736842,
                0.19736842105263158,
            ],
            [
                -0.5225563909774437,
                -0.8139097744360902,
                -0.3007518796992481,
                0.30639097744360905,
            ],
        ]);

        assert!(matrix.is_invertible());
        assert_eq!(matrix.determinant(), 532.0);
        assert_eq!(matrix.cofactor(2, 3), -160.0);
        assert_eq!(matrix.cofactor(3, 2), 105.0);

        let inverted_matrix = matrix.inverse();
        assert!(matrix.inverse().is_some());

        let inverted_matrix = inverted_matrix.unwrap();
        assert_eq!(inverted_matrix.data[3][2], -160.0 / 532.0);
        assert_eq!(inverted_matrix.data[2][3], 105.0 / 532.0);

        assert_eq!(inverted_matrix, expected_matrix);
    }

    #[test]
    fn invert_matrix_2() {
        let matrix = Matrix4::new([
            [8.0, -5.0, 9.0, 2.0],
            [7.0, 5.0, 6.0, 1.0],
            [-6.0, 0.0, 9.0, 6.0],
            [-3.0, 0.0, -9.0, -4.0],
        ]);

        let expected_matrix = Matrix4::new([
            [
                -0.15384615384615385,
                -0.15384615384615385,
                -0.28205128205128205,
                -0.5384615384615384,
            ],
            [
                -0.07692307692307693,
                0.12307692307692308,
                0.02564102564102564,
                0.03076923076923077,
            ],
            [
                0.358974358974359,
                0.358974358974359,
                0.4358974358974359,
                0.9230769230769231,
            ],
            [
                -0.6923076923076923,
                -0.6923076923076923,
                -0.7692307692307693,
                -1.9230769230769231,
            ],
        ]);
        assert!(matrix.inverse().is_some());
        assert_eq!(matrix.inverse().unwrap(), expected_matrix);
    }

    #[test]
    fn invert_matrix_3() {
        let matrix = Matrix4::new([
            [9.0, 3.0, 0.0, 9.0],
            [-5.0, -2.0, -6.0, -3.0],
            [-4.0, 9.0, 6.0, 4.0],
            [-7.0, 6.0, 6.0, 2.0],
        ]);

        let expected_matrix = Matrix4::new([
            [
                -0.040740740740740744,
                -0.07777777777777778,
                0.14444444444444443,
                -0.2222222222222222,
            ],
            [
                -0.07777777777777778,
                0.03333333333333333,
                0.36666666666666664,
                -0.3333333333333333,
            ],
            [
                -0.029012345679012345,
                -0.14629629629629629,
                -0.10925925925925926,
                0.12962962962962962,
            ],
            [
                0.17777777777777778,
                0.06666666666666667,
                -0.26666666666666666,
                0.3333333333333333,
            ],
        ]);
        assert!(matrix.inverse().is_some());
        assert_eq!(matrix.inverse().unwrap(), expected_matrix);
    }

    #[test]
    fn multiplying_matrix_by_inverse_returns_original_matrix() {
        let matrix_a = Matrix4::new([
            [3.0, -9.0, 7.0, 3.0],
            [3.0, -8.0, 2.0, -9.0],
            [-4.0, 4.0, 4.0, 1.0],
            [-6.0, 5.0, -1.0, 1.0],
        ]);
        let matrix_b = Matrix4::new([
            [8.0, 2.0, 2.0, 2.0],
            [3.0, -1.0, 7.0, 0.0],
            [7.0, 0.0, 5.0, 4.0],
            [6.0, -2.0, 0.0, 5.0],
        ]);

        let matrix_c = matrix_a.clone() * matrix_b.clone();

        assert_ne!(matrix_a, matrix_b);
        assert_eq!(matrix_a, matrix_c * matrix_b.inverse().unwrap());
    }

    #[test]
    fn multiplying_by_translation_matrix() {
        let transform = Matrix4::translate(5.0, -3.0, 2.0);
        let point = Point::new(-3.0, 4.0, 5.0);
        let expected_point = Point::new(2.0, 1.0, 7.0);

        assert_eq!(transform * point, expected_point);
    }

    #[test]
    fn multiplying_by_inverse_of_a_translation_matrix() {
        let transform = Matrix4::translate(5.0, -3.0, 2.0);
        let inv = transform.inverse().unwrap();
        let point = Point::new(-3.0, 4.0, 5.0);
        let expected_point = Point::new(-8.0, 7.0, 3.0);

        assert_eq!(inv * point, expected_point);
    }

    #[test]
    fn translation_does_not_affect_vectors() {
        let transform = Matrix4::translate(5.0, -3.0, 2.0);
        let vector = Vector::new(-3.0, 4.0, 5.0);

        assert_eq!(transform * vector, vector);
    }

    #[test]
    fn a_scaling_matrix_applied_to_a_point() {
        let transform = Matrix4::scale(2.0, 3.0, 4.0);
        let point = Point::new(-4.0, 6.0, 8.0);
        let expected_point = Point::new(-8.0, 18.0, 32.0);

        assert_eq!(transform * point, expected_point);
    }

    #[test]
    fn a_scaling_matrix_applied_to_a_vector() {
        let transform = Matrix4::scale(2.0, 3.0, 4.0);
        let vector = Vector::new(-4.0, 6.0, 8.0);
        let expected_vector = Vector::new(-8.0, 18.0, 32.0);

        assert_eq!(transform * vector, expected_vector);
    }

    #[test]
    fn multiplying_by_the_inverse_of_a_scaling_matrix() {
        let transform = Matrix4::scale(2.0, 3.0, 4.0);
        let inverse = transform.inverse().unwrap();
        let vector = Vector::new(-4.0, 6.0, 8.0);
        let expected_vector = Vector::new(-2.0, 2.0, 2.0);

        assert_eq!(inverse * vector, expected_vector);
    }

    #[test]
    fn reflection() {
        let transform = Matrix4::scale(-1.0, 1.0, 1.0);
        let point = Point::new(2.0, 3.0, 4.0);
        let expected_point = Point::new(-2.0, 3.0, 4.0);

        assert_eq!(transform * point, expected_point);
    }

    #[test]
    fn rotating_around_the_x_axis() {
        let point = Point::new(0.0, 1.0, 0.0);
        let half_quarter = Matrix4::rotate_x(PI / 4.0);
        let full_quarter = Matrix4::rotate_x(PI / 2.0);

        let expected_half_quarter_point =
            Point::new(0.0, 2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0);
        let expected_full_quarter_point = Point::new(0.0, 0.0, 1.0);

        assert_eq!(half_quarter * point, expected_half_quarter_point);
        assert_eq!(full_quarter * point, expected_full_quarter_point);
    }

    #[test]
    fn the_inverse_of_an_x_rotation_rotates_in_the_opposite_direction() {
        let point = Point::new(0.0, 1.0, 0.0);
        let half_quarter = Matrix4::rotate_x(PI / 4.0);
        let inv = half_quarter.inverse().unwrap();

        let expected_point = Point::new(0.0, 2.0_f64.sqrt() / 2.0, -2.0_f64.sqrt() / 2.0);

        assert_eq!(inv * point, expected_point);
    }

    #[test]
    fn rotating_around_the_y_axis() {
        let point = Point::new(0.0, 0.0, 1.0);
        let half_quarter = Matrix4::rotate_y(PI / 4.0);
        let full_quarter = Matrix4::rotate_y(PI / 2.0);

        let expected_half_quarter_point =
            Point::new(2.0_f64.sqrt() / 2.0, 0.0, 2.0_f64.sqrt() / 2.0);
        let expected_full_quarter_point = Point::new(1.0, 0.0, 0.0);

        assert_eq!(half_quarter * point, expected_half_quarter_point);
        assert_eq!(full_quarter * point, expected_full_quarter_point);
    }

    #[test]
    fn rotating_around_the_z_axis() {
        let point = Point::new(0.0, 1.0, 0.0);
        let half_quarter = Matrix4::rotate_z(PI / 4.0);
        let full_quarter = Matrix4::rotate_z(PI / 2.0);

        let expected_half_quarter_point =
            Point::new(-(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0, 0.0);
        let expected_full_quarter_point = Point::new(-1.0, 0.0, 0.0);

        assert_eq!(half_quarter * point, expected_half_quarter_point);
        assert_eq!(full_quarter * point, expected_full_quarter_point);
    }

    #[test]
    fn shearing_moves_x_in_proportion_to_y() {
        let transform = Matrix4::shear(1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let point = Point::new(2.0, 3.0, 4.0);
        let expected_point = Point::new(5.0, 3.0, 4.0);

        assert_eq!(transform * point, expected_point);
    }

    #[test]
    fn shearing_moves_x_in_proportion_to_z() {
        let transform = Matrix4::shear(0.0, 1.0, 0.0, 0.0, 0.0, 0.0);
        let point = Point::new(2.0, 3.0, 4.0);
        let expected_point = Point::new(6.0, 3.0, 4.0);

        assert_eq!(transform * point, expected_point);
    }

    #[test]
    fn shearing_moves_y_in_proportion_to_x() {
        let transform = Matrix4::shear(0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
        let point = Point::new(2.0, 3.0, 4.0);
        let expected_point = Point::new(2.0, 5.0, 4.0);

        assert_eq!(transform * point, expected_point);
    }

    #[test]
    fn shearing_moves_y_in_proportion_to_z() {
        let transform = Matrix4::shear(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let point = Point::new(2.0, 3.0, 4.0);
        let expected_point = Point::new(2.0, 7.0, 4.0);

        assert_eq!(transform * point, expected_point);
    }

    #[test]
    fn shearing_moves_z_in_proportion_to_x() {
        let transform = Matrix4::shear(0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        let point = Point::new(2.0, 3.0, 4.0);
        let expected_point = Point::new(2.0, 3.0, 6.0);

        assert_eq!(transform * point, expected_point);
    }

    #[test]
    fn shearing_moves_z_in_proportion_to_y() {
        let transform = Matrix4::shear(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        let point = Point::new(2.0, 3.0, 4.0);
        let expected_point = Point::new(2.0, 3.0, 7.0);

        assert_eq!(transform * point, expected_point);
    }

    #[test]
    fn individual_transformations_are_applied_in_sequence() {
        let point = Point::new(1.0, 0.0, 1.0);
        let a = Matrix4::rotate_x(PI / 2.0);
        let b = Matrix4::scale(5.0, 5.0, 5.0);
        let c = Matrix4::translate(10.0, 5.0, 7.0);

        // Apply rotation first
        let point2 = a * point;
        assert_eq!(point2, Point::new(1.0, -1.0, 0.0));

        // Then apply scaling
        let point3 = b * point2;
        assert_eq!(point3, Point::new(5.0, -5.0, 0.0));

        // Finally, apply translation
        let point4 = c * point3;
        assert_eq!(point4, Point::new(15.0, 0.0, 7.0));
    }

    #[test]
    fn chained_transformations_must_be_applied_in_reverse_order() {
        let point = Point::new(1.0, 0.0, 1.0);
        let a = Matrix4::rotate_x(PI / 2.0);
        let b = Matrix4::scale(5.0, 5.0, 5.0);
        let c = Matrix4::translate(10.0, 5.0, 7.0);
        let transformation = c * b * a;

        assert_eq!(transformation * point, Point::new(15.0, 0.0, 7.0));
    }
}
