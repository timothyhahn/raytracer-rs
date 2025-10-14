//! Scene composition elements.
//!
//! This module contains components for defining the visual properties and layout of a scene:
//! - `lights`: Light sources (currently point lights) for illuminating the scene
//! - `materials`: Surface material properties (color, ambient, diffuse, specular, shininess)
//! - `transformations`: View transformation utilities for camera positioning

pub mod lights;
pub mod materials;
pub mod transformations;
