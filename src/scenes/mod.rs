//! Scene loading and model importing.
//!
//! This module provides functionality to load scenes and 3D models from external files:
//! - `loader`: TOML scene file parser that builds World and Camera objects from configuration
//! - `obj`: Wavefront OBJ file parser for loading 3D models

pub mod loader;
pub mod obj;
