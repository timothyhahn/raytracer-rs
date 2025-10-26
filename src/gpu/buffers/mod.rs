//! GPU-compatible buffer definitions for scene data

mod camera;
mod light;
mod material;
mod scene;
mod shapes;
mod transform;

pub use camera::GpuCamera;
pub use light::GpuLight;
pub use material::GpuMaterial;
pub use scene::GpuSceneBuffers;
pub use shapes::*;
pub use transform::GpuTransform;
