//! GPU-compatible shape definitions

use bytemuck::{Pod, Zeroable};

/// Sphere geometry for GPU
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuSphere {
    pub center: [f32; 3],
    pub _center_padding: f32,
    pub radius: f32,
    pub material_idx: u32,
    pub transform_idx: u32,
    pub _padding: u32,
}

/// Plane geometry for GPU
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuPlane {
    pub normal: [f32; 3],
    pub distance: f32,
    pub material_idx: u32,
    pub transform_idx: u32,
    pub _padding: [u32; 2],
}

/// Cube geometry for GPU (unit cube from -1 to 1 in all axes)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuCube {
    pub material_idx: u32,
    pub transform_idx: u32,
    pub _padding: [u32; 2],
}

/// Cylinder geometry for GPU (unit cylinder, radius 1 in X-Z plane, extends along Y)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuCylinder {
    pub minimum: f32,
    pub maximum: f32,
    pub closed: u32,
    pub material_idx: u32,
    pub transform_idx: u32,
    pub _pre_padding: [u32; 3],
    pub _padding: [u32; 3],
    pub _final_padding: u32,
}

/// Cone for GPU (unit cone in object space, radius = |y|)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuCone {
    pub minimum: f32,
    pub maximum: f32,
    pub closed: u32,
    pub material_idx: u32,
    pub transform_idx: u32,
    pub _pre_padding: [u32; 3],
    pub _padding: [u32; 3],
    pub _final_padding: u32,
}

/// Unified triangle for GPU (handles both regular and smooth triangles)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuTriangle {
    pub p1: [f32; 3],
    pub _p1_padding: f32,
    pub p2: [f32; 3],
    pub _p2_padding: f32,
    pub p3: [f32; 3],
    pub _p3_padding: f32,
    pub e1: [f32; 3],
    pub _e1_padding: f32,
    pub e2: [f32; 3],
    pub _e2_padding: f32,
    pub n1: [f32; 3],
    pub _n1_padding: f32,
    pub n2: [f32; 3],
    pub _n2_padding: f32,
    pub n3: [f32; 3],
    pub _n3_padding: f32,
    pub material_idx: u32,
    pub transform_idx: u32,
    pub is_smooth: u32,
    pub _padding: u32,
}
