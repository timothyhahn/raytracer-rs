//! GPU-compatible camera definition

use crate::core::matrices::Matrix4;
use crate::rendering::camera::Camera;
use bytemuck::{Pod, Zeroable};

/// Camera parameters for GPU
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuCamera {
    pub transform_inverse: [[f32; 4]; 4],
    pub pixel_size: f32,
    pub half_width: f32,
    pub half_height: f32,
    pub aa_samples: u32,
    pub chunk_samples: u32,
    pub sample_offset: u32,
    pub samples_per_axis: u32,
    pub width: u32,
    pub height: u32,
    pub sphere_count: u32,
    pub plane_count: u32,
    pub cube_count: u32,
    pub cylinder_count: u32,
    pub cone_count: u32,
    pub triangle_count: u32,
    pub max_depth: u32,
    pub random_seed: u32,
    pub _padding: [u32; 7],
}

impl GpuCamera {
    pub fn from_camera(camera: &Camera) -> Self {
        Self::from_camera_with_depth(camera, 5, 0)
    }

    pub fn from_camera_with_depth(camera: &Camera, max_depth: u32, random_seed: u32) -> Self {
        let inv_transform = camera.transform.inverse().unwrap_or(Matrix4::identity());
        let mut transform_inverse = [[0.0f32; 4]; 4];

        let matrix_data = inv_transform.data();
        for (row, row_data) in transform_inverse.iter_mut().enumerate() {
            for (col, cell) in row_data.iter_mut().enumerate() {
                *cell = matrix_data[col][row] as f32;
            }
        }

        let requested_samples = camera.aa_samples.max(1);
        let samples_per_axis = ((requested_samples as f64).sqrt().ceil() as u32).max(1);
        let total_samples = samples_per_axis * samples_per_axis;

        Self {
            transform_inverse,
            pixel_size: camera.pixel_size as f32,
            half_width: camera.half_width as f32,
            half_height: camera.half_height as f32,
            aa_samples: total_samples,
            chunk_samples: total_samples,
            sample_offset: 0,
            samples_per_axis,
            width: camera.hsize,
            height: camera.vsize,
            sphere_count: 0,
            plane_count: 0,
            cube_count: 0,
            cylinder_count: 0,
            cone_count: 0,
            triangle_count: 0,
            max_depth,
            random_seed,
            _padding: [0, 0, 0, 0, 0, 0, 0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_camera_size_matches_std140() {
        assert_eq!(std::mem::size_of::<GpuCamera>(), 160);
    }

    #[test]
    fn test_gpu_camera_from_camera_basic() {
        let camera = Camera::new(800, 600, std::f64::consts::PI / 2.0);
        let gpu_camera = GpuCamera::from_camera(&camera);

        assert_eq!(gpu_camera.width, 800);
        assert_eq!(gpu_camera.height, 600);
        assert_eq!(gpu_camera.aa_samples, 1);
        assert_eq!(gpu_camera.max_depth, 5);
    }

    #[test]
    fn test_gpu_camera_calculates_samples_per_axis() {
        let mut camera = Camera::new(100, 100, std::f64::consts::PI / 2.0);
        camera.aa_samples = 16;

        let gpu_camera = GpuCamera::from_camera(&camera);

        assert_eq!(gpu_camera.aa_samples, 16);
        assert_eq!(gpu_camera.samples_per_axis, 4);
    }

    #[test]
    fn test_gpu_camera_rounds_up_samples() {
        let mut camera = Camera::new(100, 100, std::f64::consts::PI / 2.0);
        camera.aa_samples = 10;

        let gpu_camera = GpuCamera::from_camera(&camera);

        assert_eq!(gpu_camera.samples_per_axis, 4);
        assert_eq!(gpu_camera.aa_samples, 16);
    }

    #[test]
    fn test_gpu_camera_with_depth_and_seed() {
        let camera = Camera::new(100, 100, std::f64::consts::PI / 2.0);
        let gpu_camera = GpuCamera::from_camera_with_depth(&camera, 10, 42);

        assert_eq!(gpu_camera.max_depth, 10);
        assert_eq!(gpu_camera.random_seed, 42);
    }

    #[test]
    fn test_gpu_camera_handles_zero_aa_samples() {
        let mut camera = Camera::new(100, 100, std::f64::consts::PI / 2.0);
        camera.aa_samples = 0;

        let gpu_camera = GpuCamera::from_camera(&camera);

        assert_eq!(gpu_camera.aa_samples, 1);
        assert_eq!(gpu_camera.samples_per_axis, 1);
    }
}
