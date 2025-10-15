//! Core mathematical primitives and data structures.
//!
//! This module provides the fundamental building blocks for the raytracer:
//! - `floats`: Floating-point comparison utilities with epsilon tolerance
//! - `color`: RGB color representation and operations
//! - `tuples`: Points and vectors in 3D space with associated operations
//! - `matrices`: Matrix types (2x2, 3x3, 4x4) and transformation operations

pub mod color;
pub mod floats;
pub mod matrices;
pub mod tuples;
