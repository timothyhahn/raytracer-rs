//! GPU-compatible material definition

use crate::scene::materials::Material;
use crate::scene::patterns::PatternKind;
use bytemuck::{Pod, Zeroable};

/// Material properties for GPU
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuMaterial {
    pub color: [f32; 3],
    pub _color_padding: f32,
    pub ambient: f32,
    pub diffuse: f32,
    pub specular: f32,
    pub shininess: f32,
    pub reflectivity: f32,
    pub transparency: f32,
    pub refractive_index: f32,
    pub pattern_type: u32,
    pub pattern_color_a: [f32; 3],
    pub _pattern_a_padding: f32,
    pub pattern_color_b: [f32; 3],
    pub _pattern_b_padding: f32,
    pub pattern_transform_idx: u32,
    pub _padding: [u32; 11],
}

impl GpuMaterial {
    pub fn from_material(material: &Material, pattern_transform_idx: u32) -> Self {
        let (pattern_type, color_a, color_b) = if let Some(ref pattern) = material.pattern {
            match pattern.kind {
                PatternKind::Stripe { color_a, color_b } => (
                    1u32,
                    [
                        color_a.red as f32,
                        color_a.green as f32,
                        color_a.blue as f32,
                    ],
                    [
                        color_b.red as f32,
                        color_b.green as f32,
                        color_b.blue as f32,
                    ],
                ),
                PatternKind::Gradient { color_a, color_b } => (
                    2u32,
                    [
                        color_a.red as f32,
                        color_a.green as f32,
                        color_a.blue as f32,
                    ],
                    [
                        color_b.red as f32,
                        color_b.green as f32,
                        color_b.blue as f32,
                    ],
                ),
                PatternKind::Ring { color_a, color_b } => (
                    3u32,
                    [
                        color_a.red as f32,
                        color_a.green as f32,
                        color_a.blue as f32,
                    ],
                    [
                        color_b.red as f32,
                        color_b.green as f32,
                        color_b.blue as f32,
                    ],
                ),
                PatternKind::Checkers { color_a, color_b } => (
                    4u32,
                    [
                        color_a.red as f32,
                        color_a.green as f32,
                        color_a.blue as f32,
                    ],
                    [
                        color_b.red as f32,
                        color_b.green as f32,
                        color_b.blue as f32,
                    ],
                ),
                #[cfg(test)]
                PatternKind::Test => (0u32, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
            }
        } else {
            (0u32, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0])
        };

        Self {
            color: [
                material.color.red as f32,
                material.color.green as f32,
                material.color.blue as f32,
            ],
            _color_padding: 0.0,
            ambient: material.ambient as f32,
            diffuse: material.diffuse as f32,
            specular: material.specular as f32,
            shininess: material.shininess as f32,
            reflectivity: material.reflectivity as f32,
            transparency: material.transparency as f32,
            refractive_index: material.refractive_index as f32,
            pattern_type,
            pattern_color_a: color_a,
            _pattern_a_padding: 0.0,
            pattern_color_b: color_b,
            _pattern_b_padding: 0.0,
            pattern_transform_idx,
            _padding: [0; 11],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::color::Color;
    use crate::scene::patterns::Pattern;

    #[test]
    fn test_gpu_material_size_matches_std430() {
        assert_eq!(std::mem::size_of::<GpuMaterial>(), 128);
    }

    #[test]
    fn test_gpu_material_from_simple_material() {
        let material = Material {
            color: Color::new(0.8, 0.6, 0.4),
            ambient: 0.1,
            diffuse: 0.9,
            specular: 0.9,
            shininess: 200.0,
            reflectivity: 0.0,
            transparency: 0.0,
            refractive_index: 1.0,
            pattern: None,
        };

        let gpu_material = GpuMaterial::from_material(&material, 0);

        assert_eq!(gpu_material.color[0], 0.8f32);
        assert_eq!(gpu_material.color[1], 0.6f32);
        assert_eq!(gpu_material.color[2], 0.4f32);
        assert_eq!(gpu_material.ambient, 0.1f32);
        assert_eq!(gpu_material.diffuse, 0.9f32);
        assert_eq!(gpu_material.specular, 0.9f32);
        assert_eq!(gpu_material.shininess, 200.0f32);
        assert_eq!(gpu_material.pattern_type, 0);
    }

    #[test]
    fn test_gpu_material_with_stripe_pattern() {
        let mut material = Material::default();
        material.pattern = Some(Pattern::stripe(
            Color::new(1.0, 0.0, 0.0),
            Color::new(0.0, 1.0, 0.0),
        ));

        let gpu_material = GpuMaterial::from_material(&material, 5);

        assert_eq!(gpu_material.pattern_type, 1);
        assert_eq!(gpu_material.pattern_color_a, [1.0, 0.0, 0.0]);
        assert_eq!(gpu_material.pattern_color_b, [0.0, 1.0, 0.0]);
        assert_eq!(gpu_material.pattern_transform_idx, 5);
    }

    #[test]
    fn test_gpu_material_with_gradient_pattern() {
        let mut material = Material::default();
        material.pattern = Some(Pattern::gradient(
            Color::new(1.0, 0.0, 0.0),
            Color::new(0.0, 0.0, 1.0),
        ));

        let gpu_material = GpuMaterial::from_material(&material, 0);

        assert_eq!(gpu_material.pattern_type, 2);
        assert_eq!(gpu_material.pattern_color_a, [1.0, 0.0, 0.0]);
        assert_eq!(gpu_material.pattern_color_b, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_gpu_material_with_ring_pattern() {
        let mut material = Material::default();
        material.pattern = Some(Pattern::ring(
            Color::new(1.0, 1.0, 1.0),
            Color::new(0.0, 0.0, 0.0),
        ));

        let gpu_material = GpuMaterial::from_material(&material, 0);

        assert_eq!(gpu_material.pattern_type, 3);
    }

    #[test]
    fn test_gpu_material_with_checkers_pattern() {
        let mut material = Material::default();
        material.pattern = Some(Pattern::checkers(
            Color::new(1.0, 1.0, 1.0),
            Color::new(0.0, 0.0, 0.0),
        ));

        let gpu_material = GpuMaterial::from_material(&material, 0);

        assert_eq!(gpu_material.pattern_type, 4);
    }

    #[test]
    fn test_gpu_material_reflective() {
        let mut material = Material::default();
        material.reflectivity = 0.5;

        let gpu_material = GpuMaterial::from_material(&material, 0);

        assert_eq!(gpu_material.reflectivity, 0.5f32);
    }

    #[test]
    fn test_gpu_material_transparent() {
        let mut material = Material::default();
        material.transparency = 0.8;
        material.refractive_index = 1.5;

        let gpu_material = GpuMaterial::from_material(&material, 0);

        assert_eq!(gpu_material.transparency, 0.8f32);
        assert_eq!(gpu_material.refractive_index, 1.5f32);
    }
}
