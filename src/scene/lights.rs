use crate::core::color::Color;
use crate::core::tuples::{Point, Vector};
use rand::Rng;

#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    pub position: Point,
    pub intensity: Color,
}

impl PointLight {
    pub fn new(position: Point, intensity: Color) -> PointLight {
        PointLight {
            position,
            intensity,
        }
    }
}

/// An area light for soft shadows
#[derive(Clone, Copy, Debug)]
pub struct AreaLight {
    pub corner: Point,        // Corner of the light area
    pub uvec: Vector,         // Vector along u axis (full width)
    pub vvec: Vector,         // Vector along v axis (full height)
    pub usteps: u32,          // Number of samples in u direction
    pub vsteps: u32,          // Number of samples in v direction
    pub intensity: Color,
    jitter: bool,             // Whether to jitter sample positions
}

impl AreaLight {
    pub fn new(corner: Point, uvec: Vector, vvec: Vector, usteps: u32, vsteps: u32, intensity: Color) -> Self {
        // Ensure at least 1 sample in each direction to avoid division by zero
        let usteps = usteps.max(1);
        let vsteps = vsteps.max(1);

        Self {
            corner,
            uvec,
            vvec,
            usteps,
            vsteps,
            intensity,
            jitter: true,
        }
    }

    /// Get the total number of samples for this area light
    pub fn sample_count(&self) -> u32 {
        self.usteps * self.vsteps
    }

    /// Sample a point on the area light
    /// u, v are in range [0, usteps), [0, vsteps)
    pub fn point_on_light(&self, u: u32, v: u32) -> Point {
        let mut u_offset = (u as f64 + 0.5) / self.usteps as f64;
        let mut v_offset = (v as f64 + 0.5) / self.vsteps as f64;

        if self.jitter {
            let mut rng = rand::thread_rng();
            let jitter_amount = 1.0 / (self.usteps as f64 * 2.0);
            u_offset += rng.gen::<f64>() * jitter_amount - jitter_amount / 2.0;
            v_offset += rng.gen::<f64>() * jitter_amount - jitter_amount / 2.0;
        }

        self.corner + self.uvec * u_offset + self.vvec * v_offset
    }

    /// Get intensity contribution for a given sample
    pub fn intensity_at(&self, _u: u32, _v: u32) -> Color {
        self.intensity / self.sample_count() as f64
    }
}

/// Light enum to support both point and area lights
#[derive(Clone, Copy, Debug)]
pub enum Light {
    Point(PointLight),
    Area(AreaLight),
}

impl Light {
    pub fn point(position: Point, intensity: Color) -> Self {
        Light::Point(PointLight::new(position, intensity))
    }

    pub fn area(corner: Point, uvec: Vector, vvec: Vector, usteps: u32, vsteps: u32, intensity: Color) -> Self {
        Light::Area(AreaLight::new(corner, uvec, vvec, usteps, vsteps, intensity))
    }

    pub fn intensity(&self) -> Color {
        match self {
            Light::Point(pl) => pl.intensity,
            Light::Area(al) => al.intensity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::color::Color;
    use crate::core::tuples::{Point, Tuple, Vector};

    #[test]
    fn point_light_has_position_and_intensity() {
        let intensity = Color::new(1.0, 1.0, 1.0);
        let position = Point::new(0.0, 0.0, 0.0);
        let light = PointLight::new(position, intensity);
        assert_eq!(light.position, position);
        assert_eq!(light.intensity, intensity);
    }

    #[test]
    fn area_light_with_valid_parameters() {
        let corner = Point::new(0.0, 0.0, 0.0);
        let uvec = Vector::new(2.0, 0.0, 0.0);
        let vvec = Vector::new(0.0, 0.0, 2.0);
        let light = AreaLight::new(corner, uvec, vvec, 4, 4, Color::white());

        assert_eq!(light.usteps, 4);
        assert_eq!(light.vsteps, 4);
        assert_eq!(light.sample_count(), 16);
    }

    #[test]
    fn area_light_clamps_zero_samples() {
        let corner = Point::new(0.0, 0.0, 0.0);
        let uvec = Vector::new(2.0, 0.0, 0.0);
        let vvec = Vector::new(0.0, 0.0, 2.0);
        let light = AreaLight::new(corner, uvec, vvec, 0, 0, Color::white());

        // Should be clamped to at least 1 sample each
        assert_eq!(light.usteps, 1);
        assert_eq!(light.vsteps, 1);
        assert_eq!(light.sample_count(), 1);
    }

    #[test]
    fn area_light_intensity_per_sample() {
        let light = AreaLight::new(
            Point::new(0.0, 0.0, 0.0),
            Vector::new(2.0, 0.0, 0.0),
            Vector::new(0.0, 0.0, 2.0),
            4,
            4,
            Color::new(16.0, 16.0, 16.0),
        );

        // Each sample should get 1/16th of the total intensity
        let sample_intensity = light.intensity_at(0, 0);
        assert_eq!(sample_intensity, Color::new(1.0, 1.0, 1.0));
    }
}
