//! GPU acceleration module using wgpu compute shaders
//!
//! This module provides GPU-accelerated ray tracing as an alternative to the CPU-based
//! implementation. It requires the "gpu" feature flag to be enabled.
//!
//! Architecture:
//! - Scene data is flattened from Rust enums to GPU-friendly buffers
//! - Compute shader runs one invocation per pixel
//! - All ray tracing (primary, shadow, reflection) happens on GPU
//! - Results are downloaded back to CPU for image encoding

#[cfg(feature = "gpu")]
pub mod renderer;

#[cfg(feature = "gpu")]
pub mod buffers;

#[cfg(feature = "gpu")]
pub mod pipeline;

#[cfg(feature = "gpu")]
pub use renderer::GpuRenderer;
