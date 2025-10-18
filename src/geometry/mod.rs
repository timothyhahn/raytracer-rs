//! Geometric primitives and shape definitions.
//!
//! This module defines the geometric shapes that can be rendered:
//! - `cubes`: Cube implementation with ray intersection and normal calculation
//! - `cylinders`: Cylinder primitives with optional truncation and caps
//! - `cones`: Double-cone primitives with optional truncation and caps
//! - `shapes`: The Shape trait defining common behavior for all geometric primitives
//! - `sphere`: Sphere implementation with ray intersection and normal calculation
//! - `planes`: Infinite plane implementation for floors, walls, and other flat surfaces
//! - `groups`: Group container for hierarchical transformations
//! - `bounds`: Axis-aligned bounding boxes for optimization

pub mod bounds;
pub mod cones;
pub mod cubes;
pub mod cylinders;
pub mod groups;
pub mod planes;
pub mod shapes;
pub mod sphere;
