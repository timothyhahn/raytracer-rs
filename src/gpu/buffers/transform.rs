//! GPU-compatible transform definition

use crate::core::matrices::Matrix4;
use bytemuck::{Pod, Zeroable};

/// Transform matrix for GPU (4x4 matrix)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuTransform {
    pub forward: [[f32; 4]; 4],
    pub inverse: [[f32; 4]; 4],
    pub inverse_transpose: [[f32; 4]; 4],
}

impl GpuTransform {
    pub fn from_matrix(matrix: &Matrix4) -> Self {
        let inverse = matrix.inverse().unwrap_or(Matrix4::identity());
        let inverse_transpose = inverse.transpose();

        let mut forward_gpu = [[0.0f32; 4]; 4];
        let mut inverse_gpu = [[0.0f32; 4]; 4];
        let mut inverse_transpose_gpu = [[0.0f32; 4]; 4];

        let fwd_data = matrix.data();
        let inv_data = inverse.data();
        let inv_t_data = inverse_transpose.data();

        for row in 0..4 {
            for col in 0..4 {
                forward_gpu[row][col] = fwd_data[col][row] as f32;
                inverse_gpu[row][col] = inv_data[col][row] as f32;
                inverse_transpose_gpu[row][col] = inv_t_data[col][row] as f32;
            }
        }

        Self {
            forward: forward_gpu,
            inverse: inverse_gpu,
            inverse_transpose: inverse_transpose_gpu,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_transform_size_is_three_matrices() {
        assert_eq!(std::mem::size_of::<GpuTransform>(), 192);
    }

    #[test]
    fn test_gpu_transform_from_identity() {
        let matrix = Matrix4::identity();
        let gpu_transform = GpuTransform::from_matrix(&matrix);

        for i in 0..4 {
            for j in 0..4 {
                let expected = if i == j { 1.0f32 } else { 0.0f32 };
                assert_eq!(gpu_transform.forward[i][j], expected);
                assert_eq!(gpu_transform.inverse[i][j], expected);
                assert_eq!(gpu_transform.inverse_transpose[i][j], expected);
            }
        }
    }

    #[test]
    fn test_gpu_transform_from_translation() {
        let matrix = Matrix4::translate(5.0, -3.0, 2.0);
        let gpu_transform = GpuTransform::from_matrix(&matrix);

        assert_eq!(gpu_transform.forward[3][0], 5.0);
        assert_eq!(gpu_transform.forward[3][1], -3.0);
        assert_eq!(gpu_transform.forward[3][2], 2.0);

        assert_eq!(gpu_transform.inverse[3][0], -5.0);
        assert_eq!(gpu_transform.inverse[3][1], 3.0);
        assert_eq!(gpu_transform.inverse[3][2], -2.0);
    }

    #[test]
    fn test_gpu_transform_from_scaling() {
        let matrix = Matrix4::scale(2.0, 3.0, 4.0);
        let gpu_transform = GpuTransform::from_matrix(&matrix);

        assert_eq!(gpu_transform.forward[0][0], 2.0);
        assert_eq!(gpu_transform.forward[1][1], 3.0);
        assert_eq!(gpu_transform.forward[2][2], 4.0);

        assert_eq!(gpu_transform.inverse[0][0], 0.5);
        assert_eq!(gpu_transform.inverse[1][1], 1.0 / 3.0);
        assert_eq!(gpu_transform.inverse[2][2], 0.25);
    }

    #[test]
    fn test_gpu_transform_stores_inverse_transpose() {
        let matrix = Matrix4::translate(1.0, 2.0, 3.0);
        let gpu_transform = GpuTransform::from_matrix(&matrix);

        let inverse = matrix.inverse().unwrap();
        let inverse_transpose = inverse.transpose();
        let data = inverse_transpose.data();

        for i in 0..4 {
            for j in 0..4 {
                let expected = data[j][i] as f32;
                assert!((gpu_transform.inverse_transpose[i][j] - expected).abs() < 1e-6);
            }
        }
    }
}
