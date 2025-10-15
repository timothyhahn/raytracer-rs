//! Geometric primitives and shape definitions.
//!
//! This module defines the geometric shapes that can be rendered:
//! - `shapes`: The Shape trait defining common behavior for all geometric primitives
//! - `sphere`: Sphere implementation with ray intersection and normal calculation
//! - `planes`: Infinite plane implementation for floors, walls, and other flat surfaces

pub mod planes;
pub mod shapes;
pub mod sphere;
