use super::lights::PointLight;
use super::patterns::Pattern;
use crate::core::color::Color;
use crate::core::floats::float_equal;
use crate::core::tuples::{Point, Vector};
use crate::rendering::objects::Object;

#[derive(Debug, Copy, Clone)]
pub struct Material {
    pub color: Color,
    pub ambient: f64,
    pub diffuse: f64,
    pub specular: f64,
    pub shininess: f64,
    pub reflectivity: f64,
    pub transparency: f64,
    pub refractive_index: f64,
    pub pattern: Option<Pattern>,
}

pub struct MaterialBuilder {
    color: Color,
    ambient: f64,
    diffuse: f64,
    specular: f64,
    shininess: f64,
    reflectivity: f64,
    transparency: f64,
    refractive_index: f64,
    pattern: Option<Pattern>,
}

impl MaterialBuilder {
    pub fn new() -> Self {
        MaterialBuilder {
            color: Color::white(),
            ambient: 0.1,
            diffuse: 0.9,
            specular: 0.9,
            shininess: 200.0,
            reflectivity: 0.0,
            transparency: 0.0,
            refractive_index: 1.0,
            pattern: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn ambient(mut self, ambient: f64) -> Self {
        self.ambient = ambient;
        self
    }

    pub fn diffuse(mut self, diffuse: f64) -> Self {
        self.diffuse = diffuse;
        self
    }

    pub fn specular(mut self, specular: f64) -> Self {
        self.specular = specular;
        self
    }

    pub fn shininess(mut self, shininess: f64) -> Self {
        self.shininess = shininess;
        self
    }

    pub fn reflectivity(mut self, reflectivity: f64) -> Self {
        self.reflectivity = reflectivity;
        self
    }

    pub fn transparency(mut self, transparency: f64) -> Self {
        self.transparency = transparency;
        self
    }

    pub fn refractive_index(mut self, refractive_index: f64) -> Self {
        self.refractive_index = refractive_index;
        self
    }

    pub fn pattern(mut self, pattern: Pattern) -> Self {
        self.pattern = Some(pattern);
        self
    }

    pub fn build(self) -> Material {
        if self.ambient < 0.0 || self.diffuse < 0.0 || self.specular < 0.0 || self.shininess < 0.0 {
            panic!("Material values must be positive");
        }
        Material {
            color: self.color,
            ambient: self.ambient,
            diffuse: self.diffuse,
            specular: self.specular,
            shininess: self.shininess,
            reflectivity: self.reflectivity,
            transparency: self.transparency,
            refractive_index: self.refractive_index,
            pattern: self.pattern,
        }
    }
}

impl Default for MaterialBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Material {
    pub fn builder() -> MaterialBuilder {
        MaterialBuilder::new()
    }

    pub fn lighting(
        &self,
        object: &Object,
        light: PointLight,
        point: Point,
        eye_vector: Vector,
        normal_vector: Vector,
        in_shadow: bool,
    ) -> Color {
        // Use pattern if one is set, otherwise use the material's base color.
        let color = if let Some(pattern) = &self.pattern {
            pattern.color_at_object(object, point)
        } else {
            self.color
        };

        // Combine surface color with the light's color/intensity
        let effective_color = color * light.intensity;

        // Find the direction to the light source
        let light_vector = (light.position - point).normalize();

        // Compute the ambient contribution
        let ambient = effective_color * self.ambient;

        // light_dot_normal represents the cosine of the angle between the
        // light vector and the normal vector. A negative number means the
        // light is on the other side of the surface.
        let light_dot_normal = light_vector.dot(&normal_vector);

        let mut diffuse = Color::black();
        let mut specular = Color::black();

        if light_dot_normal >= 0.0 && !in_shadow {
            // Compute diffuse
            diffuse = effective_color * self.diffuse * light_dot_normal;

            // reflect_dot_eye represents the cosine of the angle between the
            // reflection vector and the eye vector. A negative number means the
            // light reflects away from the eye.
            let reflect_vector = (-light_vector).reflect(&normal_vector);
            let reflect_dot_eye = reflect_vector.dot(&eye_vector);
            if reflect_dot_eye > 0.0 {
                // Compute specular
                let factor = reflect_dot_eye.powf(self.shininess);
                specular = light.intensity * self.specular * factor;
            }
        }

        ambient + diffuse + specular
    }
}

impl Default for Material {
    fn default() -> Material {
        Material::builder().build()
    }
}

impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color
            && float_equal(self.shininess, other.shininess)
            && float_equal(self.ambient, other.ambient)
            && float_equal(self.diffuse, other.diffuse)
            && float_equal(self.specular, other.specular)
            && self.pattern == other.pattern
    }
}

#[cfg(test)]
mod tests {
    use super::Material;
    use crate::core::color::Color;
    use crate::core::tuples::{Point, Tuple, Vector};
    use crate::rendering::objects::Object;
    use crate::scene::lights::PointLight;
    use crate::scene::patterns::Pattern;

    #[test]
    fn default_material() {
        let material = Material::default();
        assert_eq!(material.color, Color::white());
        assert_eq!(material.ambient, 0.1);
        assert_eq!(material.diffuse, 0.9);
        assert_eq!(material.specular, 0.9);
        assert_eq!(material.shininess, 200.0);
    }

    #[test]
    fn lighting_with_eye_between_light_and_surface() {
        let material = Material::default();
        let position = Point::new(0.0, 0.0, 0.0);
        let eye_vector = Vector::new(0.0, 0.0, -1.0);
        let normal_vector = Vector::new(0.0, 0.0, -1.0);
        let light = PointLight::new(Point::new(0.0, 0.0, -10.0), Color::white());
        let result = material.lighting(
            &Object::sphere(),
            light,
            position,
            eye_vector,
            normal_vector,
            false,
        );
        assert_eq!(result, Color::new(1.9, 1.9, 1.9));
    }

    #[test]
    fn lighting_with_eye_between_light_and_surface_eye_offset_45_degrees() {
        let material = Material::default();
        let position = Point::new(0.0, 0.0, 0.0);
        let eye_vector = Vector::new(0.0, 2.0_f64.sqrt() / 2.0, -(2.0_f64.sqrt()) / 2.0);
        let normal_vector = Vector::new(0.0, 0.0, -1.0);
        let light = PointLight::new(Point::new(0.0, 0.0, -10.0), Color::white());
        let result = material.lighting(
            &Object::sphere(),
            light,
            position,
            eye_vector,
            normal_vector,
            false,
        );
        assert_eq!(result, Color::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn lighting_with_eye_opposite_surface_light_offset_45_degrees() {
        let material = Material::default();
        let position = Point::new(0.0, 0.0, 0.0);
        let eye_vector = Vector::new(0.0, 0.0, -1.0);
        let normal_vector = Vector::new(0.0, 0.0, -1.0);
        let light = PointLight::new(Point::new(0.0, 10.0, -10.0), Color::white());
        let result = material.lighting(
            &Object::sphere(),
            light,
            position,
            eye_vector,
            normal_vector,
            false,
        );
        assert_eq!(result, Color::new(0.7364, 0.7364, 0.7364));
    }

    #[test]
    fn lighting_with_eye_in_path_of_reflection_vector() {
        let material = Material::default();
        let position = Point::new(0.0, 0.0, 0.0);
        let eye_vector = Vector::new(0.0, -(2.0_f64.sqrt()) / 2.0, -(2.0_f64.sqrt()) / 2.0);
        let normal_vector = Vector::new(0.0, 0.0, -1.0);
        let light = PointLight::new(Point::new(0.0, 10.0, -10.0), Color::white());
        let result = material.lighting(
            &Object::sphere(),
            light,
            position,
            eye_vector,
            normal_vector,
            false,
        );
        assert_eq!(result, Color::new(1.6364, 1.6364, 1.6364));
    }

    #[test]
    fn lighting_with_light_behind_surface() {
        let material = Material::default();
        let position = Point::new(0.0, 0.0, 0.0);
        let eye_vector = Vector::new(0.0, 0.0, -1.0);
        let normal_vector = Vector::new(0.0, 0.0, -1.0);
        let light = PointLight::new(Point::new(0.0, 0.0, 10.0), Color::white());
        let result = material.lighting(
            &Object::sphere(),
            light,
            position,
            eye_vector,
            normal_vector,
            false,
        );
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn lighting_with_the_surface_in_shadow() {
        let material = Material::default();
        let eye_vector = Vector::new(0.0, 0.0, -1.0);
        let normal_vector = Vector::new(0.0, 0.0, -1.0);
        let light = PointLight::new(Point::new(0.0, 0.0, -10.0), Color::white());
        let in_shadow = true;
        let result = material.lighting(
            &Object::sphere(),
            light,
            Point::new(0.0, 0.0, 0.0),
            eye_vector,
            normal_vector,
            in_shadow,
        );
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn lighting_with_pattern_applied() {
        let material = Material {
            pattern: Some(Pattern::stripe(Color::white(), Color::black())),
            ambient: 1.0,
            diffuse: 0.0,
            specular: 0.0,
            ..Default::default()
        };
        let eye_vector = Vector::new(0.0, 0.0, -1.0);
        let normal_vector = Vector::new(0.0, 0.0, -1.0);
        let light = PointLight::new(Point::new(0.0, 0.0, -10.0), Color::white());
        let c1 = material.lighting(
            &Object::sphere(),
            light,
            Point::new(0.9, 0.0, 0.0),
            eye_vector,
            normal_vector,
            false,
        );
        let c2 = material.lighting(
            &Object::sphere(),
            light,
            Point::new(1.1, 0.0, 0.0),
            eye_vector,
            normal_vector,
            false,
        );
        assert_eq!(c1, Color::white());
        assert_eq!(c2, Color::black());
    }

    #[test]
    fn default_material_has_no_pattern() {
        let material = Material::default();
        assert!(material.pattern.is_none());
    }

    #[test]
    fn default_material_has_no_reflectivity() {
        let material = Material::default();
        assert_eq!(material.reflectivity, 0.0);
    }
}
