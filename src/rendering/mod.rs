//! Ray tracing and rendering components.
//!
//! This module contains the core ray tracing engine and rendering pipeline:
//! - `rays`: Ray definition and transformation
//! - `intersections`: Ray-shape intersection calculations and hit detection
//! - `objects`: Object enum wrapping different shape types with materials
//! - `canvas`: 2D image buffer for storing rendered pixels
//! - `camera`: Camera configuration and ray generation for each pixel
//! - `world`: Scene container with objects and lighting for rendering

pub mod rays;
pub mod intersections;
pub mod objects;
pub mod canvas;
pub mod camera;
pub mod world;
