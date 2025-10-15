use crate::core::color::Color;
use crate::core::matrices::Matrix4;
use crate::core::tuples::{Point, Tuple, Vector};
use crate::geometry::planes::Plane;
use crate::geometry::sphere::Sphere;
use crate::rendering::camera::Camera;
use crate::rendering::objects::{Object, Transformable};
use crate::rendering::world::World;
use crate::scene::lights::PointLight;
use crate::scene::materials::Material;
use crate::scene::patterns::Pattern;
use crate::scene::transformations::view_transform;
use serde::Deserialize;
use std::f64::consts::PI;
use std::fs;

#[derive(Deserialize)]
pub struct SceneFile {
    pub camera: CameraConfig,
    pub light: LightConfig,
    pub objects: Vec<ObjectConfig>,
}

#[derive(Deserialize)]
pub struct CameraConfig {
    pub width: u32,
    pub height: u32,
    pub fov: f64,
    pub from: [f64; 3],
    pub to: [f64; 3],
    pub up: [f64; 3],
}

#[derive(Deserialize)]
pub struct LightConfig {
    pub position: [f64; 3],
    pub intensity: [f64; 3],
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ObjectConfig {
    #[serde(rename = "sphere")]
    Sphere {
        transform: Option<TransformConfig>,
        material: Option<MaterialConfig>,
    },
    #[serde(rename = "plane")]
    Plane {
        transform: Option<TransformConfig>,
        material: Option<MaterialConfig>,
    },
}

#[derive(Deserialize)]
pub struct MaterialConfig {
    #[serde(default = "default_white")]
    pub color: [f64; 3],
    #[serde(default = "default_ambient")]
    pub ambient: f64,
    #[serde(default = "default_diffuse")]
    pub diffuse: f64,
    #[serde(default = "default_specular")]
    pub specular: f64,
    #[serde(default = "default_shininess")]
    pub shininess: f64,
    #[serde(default = "default_reflectivity")]
    pub reflectivity: f64,
    #[serde(default = "default_transparency")]
    pub transparency: f64,
    #[serde(default = "default_refractive_index")]
    pub refractive_index: f64,
    pub pattern: Option<PatternConfig>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum PatternConfig {
    #[serde(rename = "stripe")]
    Stripe {
        color_a: [f64; 3],
        color_b: [f64; 3],
        #[serde(flatten)]
        transform: Option<TransformConfig>,
    },
    #[serde(rename = "gradient")]
    Gradient {
        color_a: [f64; 3],
        color_b: [f64; 3],
        #[serde(flatten)]
        transform: Option<TransformConfig>,
    },
    #[serde(rename = "ring")]
    Ring {
        color_a: [f64; 3],
        color_b: [f64; 3],
        #[serde(flatten)]
        transform: Option<TransformConfig>,
    },
    #[serde(rename = "checkers")]
    Checkers {
        color_a: [f64; 3],
        color_b: [f64; 3],
        #[serde(flatten)]
        transform: Option<TransformConfig>,
    },
}

fn default_white() -> [f64; 3] {
    [1.0, 1.0, 1.0]
}
fn default_ambient() -> f64 {
    0.1
}
fn default_diffuse() -> f64 {
    0.9
}
fn default_specular() -> f64 {
    0.9
}
fn default_shininess() -> f64 {
    200.0
}

fn default_reflectivity() -> f64 {
    0.0
}

fn default_transparency() -> f64 {
    0.0
}

fn default_refractive_index() -> f64 {
    1.0
}

#[derive(Deserialize, Clone)]
pub struct TransformConfig {
    #[serde(default)]
    pub translate: Option<[f64; 3]>,
    #[serde(default)]
    pub scale: Option<[f64; 3]>,
    #[serde(default)]
    pub rotate_x: Option<f64>,
    #[serde(default)]
    pub rotate_y: Option<f64>,
    #[serde(default)]
    pub rotate_z: Option<f64>,
}

impl SceneFile {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let scene: SceneFile = toml::from_str(&contents)?;
        Ok(scene)
    }

    pub fn build_camera(&self) -> Camera {
        let mut camera = Camera::new(
            self.camera.width,
            self.camera.height,
            self.camera.fov * PI / 180.0, // Convert degrees to radians
        );
        camera.transform = view_transform(
            Point::new(
                self.camera.from[0],
                self.camera.from[1],
                self.camera.from[2],
            ),
            Point::new(self.camera.to[0], self.camera.to[1], self.camera.to[2]),
            Vector::new(self.camera.up[0], self.camera.up[1], self.camera.up[2]),
        );
        camera
    }

    pub fn build_world(&self) -> World {
        let light = PointLight::new(
            Point::new(
                self.light.position[0],
                self.light.position[1],
                self.light.position[2],
            ),
            Color::new(
                self.light.intensity[0],
                self.light.intensity[1],
                self.light.intensity[2],
            ),
        );

        let objects: Vec<Object> = self.objects.iter().map(build_object).collect();

        World {
            objects,
            light_source: Some(light),
        }
    }
}

fn build_object(config: &ObjectConfig) -> Object {
    match config {
        ObjectConfig::Sphere {
            transform,
            material,
        } => {
            let transformation = build_transform(transform);
            let mat = build_material(material);
            Object::Sphere(Sphere {
                transformation,
                material: mat,
            })
        }
        ObjectConfig::Plane {
            transform,
            material,
        } => {
            let transformation = build_transform(transform);
            let mat = build_material(material);
            Object::Plane(Plane {
                transformation,
                material: mat,
            })
        }
    }
}

fn build_transform(config: &Option<TransformConfig>) -> Matrix4 {
    let mut matrix = Matrix4::identity();

    if let Some(transform) = config {
        // Apply transformations in reverse order (translate, rotate, scale)
        // because matrix multiplication applies right-to-left
        if let Some(translate) = transform.translate {
            matrix = matrix * Matrix4::translate(translate[0], translate[1], translate[2]);
        }
        if let Some(angle) = transform.rotate_z {
            matrix = matrix * Matrix4::rotate_z(angle * PI / 180.0);
        }
        if let Some(angle) = transform.rotate_y {
            matrix = matrix * Matrix4::rotate_y(angle * PI / 180.0);
        }
        if let Some(angle) = transform.rotate_x {
            matrix = matrix * Matrix4::rotate_x(angle * PI / 180.0);
        }
        if let Some(scale) = transform.scale {
            matrix = matrix * Matrix4::scale(scale[0], scale[1], scale[2]);
        }
    }

    matrix
}

fn build_material(config: &Option<MaterialConfig>) -> Material {
    if let Some(mat_config) = config {
        let mut builder = Material::builder()
            .color(Color::new(
                mat_config.color[0],
                mat_config.color[1],
                mat_config.color[2],
            ))
            .ambient(mat_config.ambient)
            .diffuse(mat_config.diffuse)
            .specular(mat_config.specular)
            .shininess(mat_config.shininess)
            .reflectivity(mat_config.reflectivity)
            .transparency(mat_config.transparency)
            .refractive_index(mat_config.refractive_index);

        if let Some(pattern) = mat_config.pattern.as_ref().map(build_pattern) {
            builder = builder.pattern(pattern);
        }

        builder.build()
    } else {
        Material::default()
    }
}

fn build_pattern(config: &PatternConfig) -> Pattern {
    let mut pattern = match config {
        PatternConfig::Stripe {
            color_a, color_b, ..
        } => Pattern::stripe(
            Color::new(color_a[0], color_a[1], color_a[2]),
            Color::new(color_b[0], color_b[1], color_b[2]),
        ),
        PatternConfig::Gradient {
            color_a, color_b, ..
        } => Pattern::gradient(
            Color::new(color_a[0], color_a[1], color_a[2]),
            Color::new(color_b[0], color_b[1], color_b[2]),
        ),
        PatternConfig::Ring {
            color_a, color_b, ..
        } => Pattern::ring(
            Color::new(color_a[0], color_a[1], color_a[2]),
            Color::new(color_b[0], color_b[1], color_b[2]),
        ),
        PatternConfig::Checkers {
            color_a, color_b, ..
        } => Pattern::checkers(
            Color::new(color_a[0], color_a[1], color_a[2]),
            Color::new(color_b[0], color_b[1], color_b[2]),
        ),
    };

    // Apply pattern transformation
    let transform_config = match config {
        PatternConfig::Stripe { transform, .. } => transform,
        PatternConfig::Gradient { transform, .. } => transform,
        PatternConfig::Ring { transform, .. } => transform,
        PatternConfig::Checkers { transform, .. } => transform,
    };

    if let Some(transform) = transform_config {
        pattern.set_transform(build_transform(&Some(transform.clone())));
    }

    pattern
}
