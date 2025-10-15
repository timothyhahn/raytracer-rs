use crate::core::color::Color;
use crate::core::tuples::Point;

#[derive(Clone, Copy)]
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

#[cfg(test)]
mod tests {
    use super::PointLight;
    use crate::core::color::Color;
    use crate::core::tuples::{Point, Tuple};

    #[test]
    fn point_light_has_position_and_intensity() {
        let intensity = Color::new(1.0, 1.0, 1.0);
        let position = Point::new(0.0, 0.0, 0.0);
        let light = PointLight::new(position, intensity);
        assert_eq!(light.position, position);
        assert_eq!(light.intensity, intensity);
    }
}
