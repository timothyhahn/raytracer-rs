//! GPU-compatible light definition

use crate::scene::lights::Light;
use bytemuck::{Pod, Zeroable};

/// Light source for GPU
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct GpuLight {
    pub light_type: u32,
    pub _light_type_padding: [u32; 3],
    pub position: [f32; 3],
    pub _position_padding: f32,
    pub intensity: [f32; 3],
    pub _intensity_padding: f32,
    pub uvec: [f32; 3],
    pub _uvec_padding: f32,
    pub vvec: [f32; 3],
    pub _vvec_padding: f32,
    pub usteps: u32,
    pub vsteps: u32,
    pub _padding: [u32; 2],
}

impl GpuLight {
    pub fn from_light(light: &Light) -> Self {
        match light {
            Light::Point(point_light) => Self {
                light_type: 0,
                _light_type_padding: [0, 0, 0],
                position: [
                    point_light.position.x as f32,
                    point_light.position.y as f32,
                    point_light.position.z as f32,
                ],
                _position_padding: 0.0,
                intensity: [
                    point_light.intensity.red as f32,
                    point_light.intensity.green as f32,
                    point_light.intensity.blue as f32,
                ],
                _intensity_padding: 0.0,
                uvec: [0.0, 0.0, 0.0],
                _uvec_padding: 0.0,
                vvec: [0.0, 0.0, 0.0],
                _vvec_padding: 0.0,
                usteps: 1,
                vsteps: 1,
                _padding: [0, 0],
            },
            Light::Area(area_light) => Self {
                light_type: 1,
                _light_type_padding: [0, 0, 0],
                position: [
                    area_light.corner.x as f32,
                    area_light.corner.y as f32,
                    area_light.corner.z as f32,
                ],
                _position_padding: 0.0,
                intensity: [
                    area_light.intensity.red as f32,
                    area_light.intensity.green as f32,
                    area_light.intensity.blue as f32,
                ],
                _intensity_padding: 0.0,
                uvec: [
                    area_light.uvec.x as f32,
                    area_light.uvec.y as f32,
                    area_light.uvec.z as f32,
                ],
                _uvec_padding: 0.0,
                vvec: [
                    area_light.vvec.x as f32,
                    area_light.vvec.y as f32,
                    area_light.vvec.z as f32,
                ],
                _vvec_padding: 0.0,
                usteps: area_light.usteps,
                vsteps: area_light.vsteps,
                _padding: [0, 0],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::color::Color;
    use crate::core::tuples::{Point, Tuple, Vector};
    use crate::scene::lights::{AreaLight, PointLight};

    #[test]
    fn test_gpu_light_size_matches_alignment() {
        assert_eq!(std::mem::size_of::<GpuLight>(), 96);
    }

    #[test]
    fn test_gpu_light_from_point_light() {
        let point_light = PointLight::new(Point::new(1.0, 2.0, 3.0), Color::new(0.9, 0.8, 0.7));
        let light = Light::Point(point_light);
        let gpu_light = GpuLight::from_light(&light);

        assert_eq!(gpu_light.light_type, 0);
        assert_eq!(gpu_light.position, [1.0, 2.0, 3.0]);
        assert_eq!(gpu_light.intensity, [0.9, 0.8, 0.7]);
        assert_eq!(gpu_light.usteps, 1);
        assert_eq!(gpu_light.vsteps, 1);
    }

    #[test]
    fn test_gpu_light_from_area_light() {
        let area_light = AreaLight::new(
            Point::new(0.0, 0.0, 0.0),
            Vector::new(2.0, 0.0, 0.0),
            Vector::new(0.0, 0.0, 1.0),
            4,
            2,
            Color::new(1.0, 1.0, 1.0),
        );
        let light = Light::Area(area_light);
        let gpu_light = GpuLight::from_light(&light);

        assert_eq!(gpu_light.light_type, 1);
        assert_eq!(gpu_light.position, [0.0, 0.0, 0.0]);
        assert_eq!(gpu_light.intensity, [1.0, 1.0, 1.0]);
        assert_eq!(gpu_light.uvec, [2.0, 0.0, 0.0]);
        assert_eq!(gpu_light.vvec, [0.0, 0.0, 1.0]);
        assert_eq!(gpu_light.usteps, 4);
        assert_eq!(gpu_light.vsteps, 2);
    }

    #[test]
    fn test_gpu_light_default() {
        let gpu_light = GpuLight::default();

        assert_eq!(gpu_light.light_type, 0);
        assert_eq!(gpu_light.position, [0.0, 0.0, 0.0]);
        assert_eq!(gpu_light.intensity, [0.0, 0.0, 0.0]);
    }
}
