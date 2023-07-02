use crate::floats::float_equal;
use crate::tuples::Tuple;
use std::ops::Mul;

#[derive(Debug, Clone)]
pub struct Matrix {
    size: usize,
    data: Vec<Vec<f64>>,
}

impl Matrix {
    pub fn new(data: Vec<Vec<f64>>) -> Matrix {
        // Check all rows are the same length
        for row in data.iter() {
            if row.len() != data[0].len() {
                panic!("All rows must be the same length");
            }
        }
        let size = data.len();
        Matrix { size, data }
    }

    // Transformation matrices
    // Allowing since we need to index diagonally
    #[allow(clippy::needless_range_loop)]
    pub fn identity(size: usize) -> Matrix {
        let mut data = vec![vec![0.0; size]; size];

        for i in 0..size {
            data[i][i] = 1.0;
        }
        Matrix::new(data)
    }

    pub fn translation(x: f64, y: f64, z: f64) -> Matrix {
        let mut data = Matrix::identity(4).data;
        data[0][3] = x;
        data[1][3] = y;
        data[2][3] = z;
        Matrix::new(data)
    }

    pub fn scaling(x: f64, y: f64, z: f64) -> Matrix {
        let mut data = Matrix::identity(4).data;
        data[0][0] = x;
        data[1][1] = y;
        data[2][2] = z;
        Matrix::new(data)
    }

    pub fn rotation_x(r: f64) -> Matrix {
        let mut data = Matrix::identity(4).data;
        data[1][1] = r.cos();
        data[1][2] = -r.sin();
        data[2][1] = r.sin();
        data[2][2] = r.cos();
        Matrix::new(data)
    }

    pub fn rotation_y(r: f64) -> Matrix {
        let mut data = Matrix::identity(4).data;
        data[0][0] = r.cos();
        data[0][2] = r.sin();
        data[2][0] = -r.sin();
        data[2][2] = r.cos();
        Matrix::new(data)
    }

    pub fn rotation_z(r: f64) -> Matrix {
        let mut data = Matrix::identity(4).data;
        data[0][0] = r.cos();
        data[0][1] = -r.sin();
        data[1][0] = r.sin();
        data[1][1] = r.cos();
        Matrix::new(data)
    }

    pub fn shearing(xy: f64, xz: f64, yx: f64, yz: f64, zx: f64, zy: f64) -> Matrix {
        let mut data = Matrix::identity(4).data;
        data[0][1] = xy;
        data[0][2] = xz;
        data[1][0] = yx;
        data[1][2] = yz;
        data[2][0] = zx;
        data[2][1] = zy;
        Matrix::new(data)
    }

    // Returns a new Matrix since we need the old values when calculating the output
    // Allowing since using a separate variable for row/idx doesn't make things clearer
    #[allow(clippy::needless_range_loop)]
    pub fn transpose(&self) -> Matrix {
        let mut data = vec![vec![0.0; self.size]; self.size];
        for row in 0..self.size {
            for col in 0..self.size {
                data[col][row] = self.data[row][col];
            }
        }
        Matrix::new(data)
    }

    // Allowing since we need the 0th and columnth'ed variable, not each variable
    #[allow(clippy::needless_range_loop)]
    pub fn determinant(&self) -> f64 {
        // 2x2 matrix
        if self.size == 2 {
            return self.data[0][0] * self.data[1][1] - self.data[0][1] * self.data[1][0];
        }

        let mut result = 0.0;
        for column in 0..self.size {
            result += self.data[0][column] * self.cofactor(0, column);
        }
        result
    }

    // Allowing since we need to do some offset math
    #[allow(clippy::needless_range_loop)]
    pub fn submatrix(&self, row: usize, col: usize) -> Matrix {
        let new_size = self.size - 1;
        let mut data = vec![vec![0.0; new_size]; new_size];
        for i in 0..new_size {
            for j in 0..new_size {
                let row_offset = if i >= row { 1 } else { 0 };
                let col_offset = if j >= col { 1 } else { 0 };
                data[i][j] = self.data[i + row_offset][j + col_offset];
            }
        }
        Matrix::new(data)
    }

    pub fn minor(&self, row: usize, col: usize) -> f64 {
        self.submatrix(row, col).determinant()
    }

    pub fn cofactor(&self, row: usize, col: usize) -> f64 {
        let minor = self.minor(row, col);
        if (row + col) % 2 == 0 {
            minor
        } else {
            -minor
        }
    }

    pub fn is_invertible(&self) -> bool {
        !float_equal(self.determinant(), 0.0)
    }

    // Allowing since having separate idx and col/row variables doesn't make things cleaner
    #[allow(clippy::needless_range_loop)]
    pub fn inverse(&self) -> Option<Matrix> {
        if !self.is_invertible() {
            return None;
        }

        let mut data = vec![vec![0.0; self.size]; self.size];

        for row in 0..self.size {
            for col in 0..self.size {
                let c = self.cofactor(row, col);

                // Transpose here by swapping row/col
                data[col][row] = c / self.determinant();
            }
        }
        Some(Matrix::new(data))
    }
}

impl Mul for Matrix {
    type Output = Matrix;

    // Creates a new Matrix, since we need the old values when calculating the output
    // Might be doable in-place?
    // Allowing since having separate idx and col/row variables doesn't make things cleaner
    #[allow(clippy::needless_range_loop)]
    fn mul(self, other: Self) -> Matrix {
        if self.size != other.size {
            panic!(
                "Cannot multiply matrices of size {} and {}",
                self.size, other.size
            );
        }

        let size = self.size;

        let mut data = vec![vec![0.0; size]; size];

        for row in 0..size {
            for col in 0..size {
                let mut sum = 0.0;
                for i in 0..self.size {
                    sum += self.data[row][i] * other.data[i][col];
                }
                data[row][col] = sum;
            }
        }

        Matrix { size, data }
    }
}

impl Mul<Tuple> for Matrix {
    type Output = Tuple;

    fn mul(self, other: Tuple) -> Tuple {
        // Tuples are treated as 4x1 matrices
        if self.size != 4 {
            panic!("Cannot multiply matrix of size {} and tuple", self.size);
        }

        let mut data = vec![0.0; 4];

        for (idx, row) in self.data.iter().enumerate() {
            data[idx] = row[0] * other.x + row[1] * other.y + row[2] * other.z + row[3] * other.w;
        }

        Tuple::new(data[0], data[1], data[2], data[3])
    }
}

impl PartialEq for Matrix {
    fn eq(&self, other: &Self) -> bool {
        for row in 0..self.size {
            for col in 0..self.size {
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
    use crate::matrices::Matrix;
    use crate::tuples::Tuple;
    use std::f64::consts::PI;

    // First since this is the most used type of matrix.
    #[test]
    fn create_4x4_matrix() {
        let data = vec![
            vec![1.0, 2.0, 3.0, 4.0],
            vec![5.5, 6.5, 7.5, 8.5],
            vec![9.0, 10.0, 11.0, 12.0],
            vec![13.5, 14.5, 15.5, 16.5],
        ];
        let matrix = Matrix::new(data);

        assert_eq!(matrix.size, 4);
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
        let data = vec![vec![-3.0, 5.0], vec![1.0, -2.0]];
        let matrix = Matrix::new(data);

        assert_eq!(matrix.size, 2);
        assert_eq!(matrix.data[0][0], -3.0);
        assert_eq!(matrix.data[0][1], 5.0);
        assert_eq!(matrix.data[1][0], 1.0);
        assert_eq!(matrix.data[1][1], -2.0);
    }

    #[test]
    fn create_3x3_matrix() {
        let data = vec![
            vec![-3.0, 5.0, 0.0],
            vec![1.0, -2.0, -7.0],
            vec![0.0, 1.0, 1.0],
        ];
        let matrix = Matrix::new(data);

        assert_eq!(matrix.size, 3);
        assert_eq!(matrix.data[0][0], -3.0);
        assert_eq!(matrix.data[1][1], -2.0);
        assert_eq!(matrix.data[2][2], 1.0);
    }

    #[test]
    fn equality_with_identical_matrices() {
        let matrix_a = Matrix::new(vec![
            vec![1.0, 2.0, 3.0, 4.0],
            vec![5.0, 6.0, 7.0, 8.0],
            vec![9.0, 8.0, 7.0, 6.0],
            vec![5.0, 4.0, 3.0, 2.0],
        ]);

        let matrix_b = Matrix::new(vec![
            vec![1.0, 2.0, 3.0, 4.0],
            vec![5.0, 6.0, 7.0, 8.0],
            vec![9.0, 8.0, 7.0, 6.0],
            vec![5.0, 4.0, 3.0, 2.0],
        ]);

        assert_eq!(matrix_a, matrix_b);
    }

    #[test]
    fn inequality_with_similar_matrices() {
        let matrix_a = Matrix::new(vec![
            vec![1.0, 2.0, 3.0, 4.0],
            vec![5.0, 6.0, 7.0, 8.0],
            vec![9.0, 8.0, 7.0, 6.0],
            vec![5.0, 4.0, 3.0, 2.0],
        ]);

        let matrix_b = Matrix::new(vec![
            vec![1.0, 2.0, 3.0, 4.0],
            vec![5.0, 6.0, 7.0, 8.0],
            vec![9.0, 8.0, 7.0, 6.0],
            vec![5.0, 4.0, 2.0, 2.0],
        ]);

        assert_ne!(matrix_a, matrix_b);
    }

    #[test]
    fn multiply_two_matrices() {
        let matrix_a = Matrix::new(vec![
            vec![1.0, 2.0, 3.0, 4.0],
            vec![5.0, 6.0, 7.0, 8.0],
            vec![9.0, 8.0, 7.0, 6.0],
            vec![5.0, 4.0, 3.0, 2.0],
        ]);

        let matrix_b = Matrix::new(vec![
            vec![-2.0, 1.0, 2.0, 3.0],
            vec![3.0, 2.0, 1.0, -1.0],
            vec![4.0, 3.0, 6.0, 5.0],
            vec![1.0, 2.0, 7.0, 8.0],
        ]);

        let expected = Matrix::new(vec![
            vec![20.0, 22.0, 50.0, 48.0],
            vec![44.0, 54.0, 114.0, 108.0],
            vec![40.0, 58.0, 110.0, 102.0],
            vec![16.0, 26.0, 46.0, 42.0],
        ]);

        assert_eq!(matrix_a * matrix_b, expected);
    }

    #[test]
    fn multiply_matrix_by_tuple() {
        let matrix = Matrix::new(vec![
            vec![1.0, 2.0, 3.0, 4.0],
            vec![2.0, 4.0, 4.0, 2.0],
            vec![8.0, 6.0, 4.0, 1.0],
            vec![0.0, 0.0, 0.0, 1.0],
        ]);
        let tuple = Tuple::new(1.0, 2.0, 3.0, 1.0);

        assert_eq!(matrix * tuple, Tuple::new(18.0, 24.0, 33.0, 1.0));
    }

    #[test]
    fn multiply_by_identity_matrix() {
        let matrix = Matrix::new(vec![
            vec![0.0, 1.0, 2.0, 4.0],
            vec![1.0, 2.0, 4.0, 8.0],
            vec![2.0, 4.0, 8.0, 16.0],
            vec![4.0, 8.0, 16.0, 32.0],
        ]);

        let identity_matrix = Matrix::identity(matrix.size);

        assert_eq!(matrix.clone() * identity_matrix, matrix);
    }

    #[test]
    fn multiplying_identity_matrix_by_tuple() {
        let tuple = Tuple::new(1.0, 2.0, 3.0, 4.0);
        let identity_matrix = Matrix::identity(4);

        assert_eq!(identity_matrix * tuple, tuple);
    }

    #[test]
    fn transposing_a_matrix() {
        let matrix = Matrix::new(vec![
            vec![0.0, 9.0, 3.0, 0.0],
            vec![9.0, 8.0, 0.0, 8.0],
            vec![1.0, 8.0, 5.0, 3.0],
            vec![0.0, 0.0, 5.0, 8.0],
        ]);

        let expected_matrix = Matrix::new(vec![
            vec![0.0, 9.0, 1.0, 0.0],
            vec![9.0, 8.0, 8.0, 0.0],
            vec![3.0, 0.0, 5.0, 5.0],
            vec![0.0, 8.0, 3.0, 8.0],
        ]);

        assert_eq!(matrix.transpose(), expected_matrix);
    }

    #[test]
    fn calculating_determinant_of_2x2_matrix() {
        let matrix = Matrix::new(vec![vec![1.0, 5.0], vec![-3.0, 2.0]]);

        assert_eq!(matrix.determinant(), 17.0);
    }

    #[test]
    fn calculate_the_determinant_of_a_3x3_matrix() {
        let matrix = Matrix::new(vec![
            vec![1.0, 2.0, 6.0],
            vec![-5.0, 8.0, -4.0],
            vec![2.0, 6.0, 4.0],
        ]);

        assert_eq!(matrix.cofactor(0, 0), 56.0);
        assert_eq!(matrix.cofactor(0, 1), 12.0);
        assert_eq!(matrix.cofactor(0, 2), -46.0);
        assert_eq!(matrix.determinant(), -196.0);
    }

    #[test]
    fn calculate_the_determinant_of_a_4x4_matrix() {
        let matrix = Matrix::new(vec![
            vec![-2.0, -8.0, 3.0, 5.0],
            vec![-3.0, 1.0, 7.0, 3.0],
            vec![1.0, 2.0, -9.0, 6.0],
            vec![-6.0, 7.0, 7.0, -9.0],
        ]);

        assert_eq!(matrix.cofactor(0, 0), 690.0);
        assert_eq!(matrix.cofactor(0, 1), 447.0);
        assert_eq!(matrix.cofactor(0, 2), 210.0);
        assert_eq!(matrix.cofactor(0, 3), 51.0);
        assert_eq!(matrix.determinant(), -4071.0);
    }

    #[test]
    fn get_2x2_submatrix_from_3x3_matrix() {
        let matrix = Matrix::new(vec![
            vec![1.0, 5.0, 0.0],
            vec![-3.0, 2.0, 7.0],
            vec![0.0, 6.0, -3.0],
        ]);

        let expected_matrix = Matrix::new(vec![vec![-3.0, 2.0], vec![0.0, 6.0]]);

        assert_eq!(matrix.submatrix(0, 2), expected_matrix);
    }

    #[test]
    fn get_3x3_submatrix_from_4x4_matrix() {
        let matrix = Matrix::new(vec![
            vec![-6.0, 1.0, 1.0, 6.0],
            vec![-8.0, 5.0, 8.0, 6.0],
            vec![-1.0, 0.0, 8.0, 2.0],
            vec![-7.0, 1.0, -1.0, 1.0],
        ]);

        let expected_matrix = Matrix::new(vec![
            vec![-6.0, 1.0, 6.0],
            vec![-8.0, 8.0, 6.0],
            vec![-7.0, -1.0, 1.0],
        ]);

        assert_eq!(matrix.submatrix(2, 1), expected_matrix);
    }

    #[test]
    fn calculate_minor_of_3x3_matrix() {
        let matrix = Matrix::new(vec![
            vec![3.0, 5.0, 0.0],
            vec![2.0, -1.0, -7.0],
            vec![6.0, -1.0, 5.0],
        ]);

        assert_eq!(matrix.minor(1, 0), 25.0);
    }

    #[test]
    fn calculate_cofactor_of_3x3_matrix() {
        let matrix = Matrix::new(vec![
            vec![3.0, 5.0, 0.0],
            vec![2.0, -1.0, -7.0],
            vec![6.0, -1.0, 5.0],
        ]);

        assert_eq!(matrix.minor(0, 0), -12.0);
        assert_eq!(matrix.cofactor(0, 0), -12.0);
        assert_eq!(matrix.minor(1, 0), 25.0);
        assert_eq!(matrix.cofactor(1, 0), -25.0);
    }

    #[test]
    fn invertible_matrix() {
        let matrix = Matrix::new(vec![
            vec![6.0, 4.0, 4.0, 4.0],
            vec![5.0, 5.0, 7.0, 6.0],
            vec![4.0, -9.0, 3.0, -7.0],
            vec![9.0, 1.0, 7.0, -6.0],
        ]);
        assert_eq!(matrix.determinant(), -2120.0);
        assert!(matrix.is_invertible());
    }

    #[test]
    fn noninvertible_matrix() {
        let matrix = Matrix::new(vec![
            vec![-4.0, 2.0, -2.0, -3.0],
            vec![9.0, 6.0, 2.0, 6.0],
            vec![0.0, -5.0, 1.0, -5.0],
            vec![0.0, 0.0, 0.0, 0.0],
        ]);
        assert_eq!(matrix.determinant(), 0.0);
        assert!(!matrix.is_invertible());
    }

    #[test]
    fn invert_matrix() {
        let matrix = Matrix::new(vec![
            vec![-5.0, 2.0, 6.0, -8.0],
            vec![1.0, -5.0, 1.0, 8.0],
            vec![7.0, 7.0, -6.0, -7.0],
            vec![1.0, -3.0, 7.0, 4.0],
        ]);

        let expected_matrix = Matrix::new(vec![
            vec![
                0.21804511278195488,
                0.45112781954887216,
                0.24060150375939848,
                -0.045112781954887216,
            ],
            vec![
                -0.8082706766917294,
                -1.4567669172932332,
                -0.44360902255639095,
                0.5206766917293233,
            ],
            vec![
                -0.07894736842105263,
                -0.2236842105263158,
                -0.05263157894736842,
                0.19736842105263158,
            ],
            vec![
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
        let matrix = Matrix::new(vec![
            vec![8.0, -5.0, 9.0, 2.0],
            vec![7.0, 5.0, 6.0, 1.0],
            vec![-6.0, 0.0, 9.0, 6.0],
            vec![-3.0, 0.0, -9.0, -4.0],
        ]);

        let expected_matrix = Matrix::new(vec![
            vec![
                -0.15384615384615385,
                -0.15384615384615385,
                -0.28205128205128205,
                -0.5384615384615384,
            ],
            vec![
                -0.07692307692307693,
                0.12307692307692308,
                0.02564102564102564,
                0.03076923076923077,
            ],
            vec![
                0.358974358974359,
                0.358974358974359,
                0.4358974358974359,
                0.9230769230769231,
            ],
            vec![
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
        let matrix = Matrix::new(vec![
            vec![9.0, 3.0, 0.0, 9.0],
            vec![-5.0, -2.0, -6.0, -3.0],
            vec![-4.0, 9.0, 6.0, 4.0],
            vec![-7.0, 6.0, 6.0, 2.0],
        ]);

        let expected_matrix = Matrix::new(vec![
            vec![
                -0.040740740740740744,
                -0.07777777777777778,
                0.14444444444444443,
                -0.2222222222222222,
            ],
            vec![
                -0.07777777777777778,
                0.03333333333333333,
                0.36666666666666664,
                -0.3333333333333333,
            ],
            vec![
                -0.029012345679012345,
                -0.14629629629629629,
                -0.10925925925925926,
                0.12962962962962962,
            ],
            vec![
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
        let matrix_a = Matrix::new(vec![
            vec![3.0, -9.0, 7.0, 3.0],
            vec![3.0, -8.0, 2.0, -9.0],
            vec![-4.0, 4.0, 4.0, 1.0],
            vec![-6.0, 5.0, -1.0, 1.0],
        ]);
        let matrix_b = Matrix::new(vec![
            vec![8.0, 2.0, 2.0, 2.0],
            vec![3.0, -1.0, 7.0, 0.0],
            vec![7.0, 0.0, 5.0, 4.0],
            vec![6.0, -2.0, 0.0, 5.0],
        ]);

        let matrix_c = matrix_a.clone() * matrix_b.clone();

        assert_ne!(matrix_a, matrix_b);
        assert_eq!(matrix_a, matrix_c * matrix_b.inverse().unwrap());
    }

    #[test]
    fn multiplying_by_translation_matrix() {
        let transform = Matrix::translation(5.0, -3.0, 2.0);
        let point = Tuple::point(-3.0, 4.0, 5.0);
        let expected_point = Tuple::point(2.0, 1.0, 7.0);

        assert_eq!(transform * point, expected_point);
    }

    #[test]
    fn multiplying_by_inverse_of_a_translation_matrix() {
        let transform = Matrix::translation(5.0, -3.0, 2.0);
        let inv = transform.inverse().unwrap();
        let point = Tuple::point(-3.0, 4.0, 5.0);
        let expected_point = Tuple::point(-8.0, 7.0, 3.0);

        assert_eq!(inv * point, expected_point);
    }

    #[test]
    fn translation_does_not_affect_vectors() {
        let transform = Matrix::translation(5.0, -3.0, 2.0);
        let vector = Tuple::vector(-3.0, 4.0, 5.0);

        assert_eq!(transform * vector, vector);
    }

    #[test]
    fn a_scaling_matrix_applied_to_a_point() {
        let transform = Matrix::scaling(2.0, 3.0, 4.0);
        let point = Tuple::point(-4.0, 6.0, 8.0);
        let expected_point = Tuple::point(-8.0, 18.0, 32.0);

        assert_eq!(transform * point, expected_point);
    }

    #[test]
    fn a_scaling_matrix_applied_to_a_vector() {
        let transform = Matrix::scaling(2.0, 3.0, 4.0);
        let vector = Tuple::vector(-4.0, 6.0, 8.0);
        let expected_vector = Tuple::vector(-8.0, 18.0, 32.0);

        assert_eq!(transform * vector, expected_vector);
    }

    #[test]
    fn multiplying_by_the_inverse_of_a_scaling_matrix() {
        let transform = Matrix::scaling(2.0, 3.0, 4.0);
        let inverse = transform.inverse().unwrap();
        let vector = Tuple::vector(-4.0, 6.0, 8.0);
        let expected_vector = Tuple::vector(-2.0, 2.0, 2.0);

        assert_eq!(inverse * vector, expected_vector);
    }

    #[test]
    fn reflection() {
        let transform = Matrix::scaling(-1.0, 1.0, 1.0);
        let point = Tuple::point(2.0, 3.0, 4.0);
        let expected_point = Tuple::point(-2.0, 3.0, 4.0);

        assert_eq!(transform * point, expected_point);
    }

    #[test]
    fn rotating_around_the_x_axis() {
        let point = Tuple::point(0.0, 1.0, 0.0);
        let half_quarter = Matrix::rotation_x(PI / 4.0);
        let full_quarter = Matrix::rotation_x(PI / 2.0);

        let expected_half_quarter_point =
            Tuple::point(0.0, 2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0);
        let expected_full_quarter_point = Tuple::point(0.0, 0.0, 1.0);

        assert_eq!(half_quarter * point, expected_half_quarter_point);
        assert_eq!(full_quarter * point, expected_full_quarter_point);
    }

    #[test]
    fn the_inverse_of_an_x_rotation_rotates_in_the_opposite_direction() {
        let point = Tuple::point(0.0, 1.0, 0.0);
        let half_quarter = Matrix::rotation_x(PI / 4.0);
        let inv = half_quarter.inverse().unwrap();

        let expected_point = Tuple::point(0.0, 2.0_f64.sqrt() / 2.0, -2.0_f64.sqrt() / 2.0);

        assert_eq!(inv * point, expected_point);
    }

    #[test]
    fn rotating_around_the_y_axis() {
        let point = Tuple::point(0.0, 0.0, 1.0);
        let half_quarter = Matrix::rotation_y(PI / 4.0);
        let full_quarter = Matrix::rotation_y(PI / 2.0);

        let expected_half_quarter_point =
            Tuple::point(2.0_f64.sqrt() / 2.0, 0.0, 2.0_f64.sqrt() / 2.0);
        let expected_full_quarter_point = Tuple::point(1.0, 0.0, 0.0);

        assert_eq!(half_quarter * point, expected_half_quarter_point);
        assert_eq!(full_quarter * point, expected_full_quarter_point);
    }

    #[test]
    fn rotating_around_the_z_axis() {
        let point = Tuple::point(0.0, 1.0, 0.0);
        let half_quarter = Matrix::rotation_z(PI / 4.0);
        let full_quarter = Matrix::rotation_z(PI / 2.0);

        let expected_half_quarter_point =
            Tuple::point(-2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0, 0.0);
        let expected_full_quarter_point = Tuple::point(-1.0, 0.0, 0.0);

        assert_eq!(half_quarter * point, expected_half_quarter_point);
        assert_eq!(full_quarter * point, expected_full_quarter_point);
    }

    #[test]
    fn shearing_moves_x_in_proportion_to_y() {
        let transform = Matrix::shearing(1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let point = Tuple::point(2.0, 3.0, 4.0);
        let expected_point = Tuple::point(5.0, 3.0, 4.0);

        assert_eq!(transform * point, expected_point);
    }

    #[test]
    fn shearing_moves_x_in_proportion_to_z() {
        let transform = Matrix::shearing(0.0, 1.0, 0.0, 0.0, 0.0, 0.0);
        let point = Tuple::point(2.0, 3.0, 4.0);
        let expected_point = Tuple::point(6.0, 3.0, 4.0);

        assert_eq!(transform * point, expected_point);
    }

    #[test]
    fn shearing_moves_y_in_proportion_to_x() {
        let transform = Matrix::shearing(0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
        let point = Tuple::point(2.0, 3.0, 4.0);
        let expected_point = Tuple::point(2.0, 5.0, 4.0);

        assert_eq!(transform * point, expected_point);
    }

    #[test]
    fn shearing_moves_y_in_proportion_to_z() {
        let transform = Matrix::shearing(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let point = Tuple::point(2.0, 3.0, 4.0);
        let expected_point = Tuple::point(2.0, 7.0, 4.0);

        assert_eq!(transform * point, expected_point);
    }

    #[test]
    fn shearing_moves_z_in_proportion_to_x() {
        let transform = Matrix::shearing(0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        let point = Tuple::point(2.0, 3.0, 4.0);
        let expected_point = Tuple::point(2.0, 3.0, 6.0);

        assert_eq!(transform * point, expected_point);
    }

    #[test]
    fn shearing_moves_z_in_proportion_to_y() {
        let transform = Matrix::shearing(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        let point = Tuple::point(2.0, 3.0, 4.0);
        let expected_point = Tuple::point(2.0, 3.0, 7.0);

        assert_eq!(transform * point, expected_point);
    }

    #[test]
    fn individual_transformations_are_applied_in_sequence() {
        let point = Tuple::point(1.0, 0.0, 1.0);
        let a = Matrix::rotation_x(PI / 2.0);
        let b = Matrix::scaling(5.0, 5.0, 5.0);
        let c = Matrix::translation(10.0, 5.0, 7.0);

        // Apply rotation first
        let point2 = a * point;
        assert_eq!(point2, Tuple::point(1.0, -1.0, 0.0));

        // Then apply scaling
        let point3 = b * point2;
        assert_eq!(point3, Tuple::point(5.0, -5.0, 0.0));

        // Finally, apply translation
        let point4 = c * point3;
        assert_eq!(point4, Tuple::point(15.0, 0.0, 7.0));
    }

    #[test]
    fn chained_transformations_must_be_applied_in_reverse_order() {
        let point = Tuple::point(1.0, 0.0, 1.0);
        let a = Matrix::rotation_x(PI / 2.0);
        let b = Matrix::scaling(5.0, 5.0, 5.0);
        let c = Matrix::translation(10.0, 5.0, 7.0);
        let transformation = c * b * a;

        assert_eq!(transformation * point, Tuple::point(15.0, 0.0, 7.0));
    }
}
