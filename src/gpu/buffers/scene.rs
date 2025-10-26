//! GPU scene buffer management

use super::camera::GpuCamera;
use super::light::GpuLight;
use super::material::GpuMaterial;
use super::shapes::*;
use super::transform::GpuTransform;
use crate::acceleration::bvh::{BVHTriangle, AABB, BVH};
use crate::core::matrices::Matrix4;
use crate::core::tuples::{Point, Tuple};
use crate::rendering::camera::Camera;
use crate::rendering::objects::{HasMaterial, Object};
use crate::rendering::world::World;
use crate::scene::materials::Material;
use std::collections::HashMap;

/// Scene data collection for GPU upload
#[derive(Debug)]
pub struct GpuSceneBuffers {
    pub camera: GpuCamera,
    pub spheres: Vec<GpuSphere>,
    pub planes: Vec<GpuPlane>,
    pub cubes: Vec<GpuCube>,
    pub cylinders: Vec<GpuCylinder>,
    pub cones: Vec<GpuCone>,
    pub triangles: Vec<GpuTriangle>,
    pub materials: Vec<GpuMaterial>,
    pub transforms: Vec<GpuTransform>,
    pub light: Option<GpuLight>,
    pub bvh_nodes: Vec<crate::acceleration::bvh::GpuBVHNode>,
    pub bvh_triangle_indices: Vec<u32>,
    material_cache: HashMap<Vec<u8>, u32>,
    transform_cache: HashMap<Vec<u8>, u32>,
}

impl GpuSceneBuffers {
    pub fn new(camera: GpuCamera) -> Self {
        Self {
            camera,
            spheres: Vec::new(),
            planes: Vec::new(),
            cubes: Vec::new(),
            cylinders: Vec::new(),
            cones: Vec::new(),
            triangles: Vec::new(),
            materials: Vec::new(),
            transforms: Vec::new(),
            light: None,
            bvh_nodes: Vec::new(),
            bvh_triangle_indices: Vec::new(),
            material_cache: HashMap::new(),
            transform_cache: HashMap::new(),
        }
    }

    pub fn from_scene(camera: &Camera, world: &World) -> Self {
        Self::from_scene_with_depth(camera, world, 5, 0)
    }

    pub fn from_scene_with_depth(
        camera: &Camera,
        world: &World,
        max_depth: u32,
        random_seed: u32,
    ) -> Self {
        let mut gpu_camera = GpuCamera::from_camera_with_depth(camera, max_depth, random_seed);
        let mut buffers = Self::new(gpu_camera);

        if let Some(light) = &world.light_source {
            buffers.light = Some(GpuLight::from_light(light));
        }

        for object in &world.objects {
            buffers.add_object(object);
        }

        gpu_camera.sphere_count = buffers.spheres.len() as u32;
        gpu_camera.plane_count = buffers.planes.len() as u32;
        gpu_camera.cube_count = buffers.cubes.len() as u32;
        gpu_camera.cylinder_count = buffers.cylinders.len() as u32;
        gpu_camera.cone_count = buffers.cones.len() as u32;
        gpu_camera.triangle_count = buffers.triangles.len() as u32;

        if !buffers.triangles.is_empty() {
            buffers.build_bvh();
        }

        buffers.camera = gpu_camera;
        buffers
    }

    fn add_object(&mut self, object: &Object) {
        self.add_object_with_material(object, None);
    }

    fn build_bvh(&mut self) {
        let mut bvh_triangles = Vec::new();

        for (idx, tri) in self.triangles.iter().enumerate() {
            let p1_obj = Point::new(tri.p1[0] as f64, tri.p1[1] as f64, tri.p1[2] as f64);
            let p2_obj = Point::new(
                (tri.p1[0] + tri.e1[0]) as f64,
                (tri.p1[1] + tri.e1[1]) as f64,
                (tri.p1[2] + tri.e1[2]) as f64,
            );
            let p3_obj = Point::new(
                (tri.p1[0] + tri.e2[0]) as f64,
                (tri.p1[1] + tri.e2[1]) as f64,
                (tri.p1[2] + tri.e2[2]) as f64,
            );

            let transform = &self.transforms[tri.transform_idx as usize];

            let p1_world = Self::transform_point(&transform.forward, p1_obj);
            let p2_world = Self::transform_point(&transform.forward, p2_obj);
            let p3_world = Self::transform_point(&transform.forward, p3_obj);

            let mut bounds = AABB::empty();
            bounds.expand_by_point(p1_world);
            bounds.expand_by_point(p2_world);
            bounds.expand_by_point(p3_world);
            let centroid = bounds.centroid();

            bvh_triangles.push(BVHTriangle {
                index: idx,
                bounds,
                centroid,
            });
        }

        let bvh = BVH::build(bvh_triangles);
        self.bvh_nodes = bvh.flat_nodes;
        self.bvh_triangle_indices = bvh.triangle_indices;
    }

    fn transform_point(matrix: &[[f32; 4]; 4], point: Point) -> Point {
        let x = matrix[0][0] as f64 * point.x()
            + matrix[1][0] as f64 * point.y()
            + matrix[2][0] as f64 * point.z()
            + matrix[3][0] as f64;
        let y = matrix[0][1] as f64 * point.x()
            + matrix[1][1] as f64 * point.y()
            + matrix[2][1] as f64 * point.z()
            + matrix[3][1] as f64;
        let z = matrix[0][2] as f64 * point.x()
            + matrix[1][2] as f64 * point.y()
            + matrix[2][2] as f64 * point.z()
            + matrix[3][2] as f64;

        Point::new(x, y, z)
    }

    fn get_or_insert_material(&mut self, material: &Material, pattern_transform_idx: u32) -> u32 {
        let gpu_material = GpuMaterial::from_material(material, pattern_transform_idx);
        let key = bytemuck::bytes_of(&gpu_material).to_vec();

        if let Some(&idx) = self.material_cache.get(&key) {
            idx
        } else {
            let idx = self.materials.len() as u32;
            self.materials.push(gpu_material);
            self.material_cache.insert(key, idx);
            idx
        }
    }

    fn get_or_insert_transform(&mut self, matrix: &Matrix4) -> u32 {
        let gpu_transform = GpuTransform::from_matrix(matrix);
        let key = bytemuck::bytes_of(&gpu_transform).to_vec();

        if let Some(&idx) = self.transform_cache.get(&key) {
            idx
        } else {
            let idx = self.transforms.len() as u32;
            self.transforms.push(gpu_transform);
            self.transform_cache.insert(key, idx);
            idx
        }
    }

    fn add_object_with_material(&mut self, object: &Object, override_material: Option<Material>) {
        let world_transform = object.world_transformation();
        let transform_idx = self.get_or_insert_transform(&world_transform);
        let material = override_material.unwrap_or_else(|| object.material());
        let pattern_transform_idx = if let Some(ref pattern) = material.pattern {
            self.get_or_insert_transform(&pattern.transform)
        } else {
            0
        };

        let material_idx = self.get_or_insert_material(&material, pattern_transform_idx);

        match object {
            Object::Sphere(_sphere) => {
                self.spheres.push(GpuSphere {
                    center: [0.0, 0.0, 0.0],
                    _center_padding: 0.0,
                    radius: 1.0,
                    material_idx,
                    transform_idx,
                    _padding: 0,
                });
            }
            Object::Plane(_plane) => {
                self.planes.push(GpuPlane {
                    normal: [0.0, 1.0, 0.0],
                    distance: 0.0,
                    material_idx,
                    transform_idx,
                    _padding: [0, 0],
                });
            }
            Object::Cube(_cube) => {
                self.cubes.push(GpuCube {
                    material_idx,
                    transform_idx,
                    _padding: [0, 0],
                });
            }
            Object::Cylinder(cylinder) => {
                self.cylinders.push(GpuCylinder {
                    minimum: cylinder.minimum as f32,
                    maximum: cylinder.maximum as f32,
                    closed: if cylinder.closed { 1 } else { 0 },
                    material_idx,
                    transform_idx,
                    _pre_padding: [0, 0, 0],
                    _padding: [0, 0, 0],
                    _final_padding: 0,
                });
            }
            Object::Cone(cone) => {
                self.cones.push(GpuCone {
                    minimum: cone.minimum as f32,
                    maximum: cone.maximum as f32,
                    closed: if cone.closed { 1 } else { 0 },
                    material_idx,
                    transform_idx,
                    _pre_padding: [0, 0, 0],
                    _padding: [0, 0, 0],
                    _final_padding: 0,
                });
            }
            Object::Group(group) => {
                let group_material = override_material.or_else(|| {
                    let mat = object.material();
                    let is_non_default = mat.transparency > 0.0
                        || mat.reflectivity > 0.0
                        || mat.color.red != 1.0
                        || mat.color.green != 1.0
                        || mat.color.blue != 1.0
                        || mat.pattern.is_some();

                    if is_non_default {
                        Some(mat)
                    } else {
                        None
                    }
                });
                for child in group.children() {
                    self.add_object_with_material(child, group_material);
                }
            }
            Object::Triangle(triangle) => {
                self.triangles.push(GpuTriangle {
                    p1: [
                        triangle.p1.x() as f32,
                        triangle.p1.y() as f32,
                        triangle.p1.z() as f32,
                    ],
                    _p1_padding: 0.0,
                    p2: [
                        triangle.p2.x() as f32,
                        triangle.p2.y() as f32,
                        triangle.p2.z() as f32,
                    ],
                    _p2_padding: 0.0,
                    p3: [
                        triangle.p3.x() as f32,
                        triangle.p3.y() as f32,
                        triangle.p3.z() as f32,
                    ],
                    _p3_padding: 0.0,
                    e1: [
                        triangle.e1.x() as f32,
                        triangle.e1.y() as f32,
                        triangle.e1.z() as f32,
                    ],
                    _e1_padding: 0.0,
                    e2: [
                        triangle.e2.x() as f32,
                        triangle.e2.y() as f32,
                        triangle.e2.z() as f32,
                    ],
                    _e2_padding: 0.0,
                    n1: [0.0, 0.0, 0.0],
                    _n1_padding: 0.0,
                    n2: [0.0, 0.0, 0.0],
                    _n2_padding: 0.0,
                    n3: [0.0, 0.0, 0.0],
                    _n3_padding: 0.0,
                    material_idx,
                    transform_idx,
                    is_smooth: 0,
                    _padding: 0,
                });
            }
            Object::SmoothTriangle(smooth_triangle) => {
                self.triangles.push(GpuTriangle {
                    p1: [
                        smooth_triangle.p1.x() as f32,
                        smooth_triangle.p1.y() as f32,
                        smooth_triangle.p1.z() as f32,
                    ],
                    _p1_padding: 0.0,
                    p2: [
                        smooth_triangle.p2.x() as f32,
                        smooth_triangle.p2.y() as f32,
                        smooth_triangle.p2.z() as f32,
                    ],
                    _p2_padding: 0.0,
                    p3: [
                        smooth_triangle.p3.x() as f32,
                        smooth_triangle.p3.y() as f32,
                        smooth_triangle.p3.z() as f32,
                    ],
                    _p3_padding: 0.0,
                    e1: [
                        smooth_triangle.e1.x() as f32,
                        smooth_triangle.e1.y() as f32,
                        smooth_triangle.e1.z() as f32,
                    ],
                    _e1_padding: 0.0,
                    e2: [
                        smooth_triangle.e2.x() as f32,
                        smooth_triangle.e2.y() as f32,
                        smooth_triangle.e2.z() as f32,
                    ],
                    _e2_padding: 0.0,
                    n1: [
                        smooth_triangle.n1.x() as f32,
                        smooth_triangle.n1.y() as f32,
                        smooth_triangle.n1.z() as f32,
                    ],
                    _n1_padding: 0.0,
                    n2: [
                        smooth_triangle.n2.x() as f32,
                        smooth_triangle.n2.y() as f32,
                        smooth_triangle.n2.z() as f32,
                    ],
                    _n2_padding: 0.0,
                    n3: [
                        smooth_triangle.n3.x() as f32,
                        smooth_triangle.n3.y() as f32,
                        smooth_triangle.n3.z() as f32,
                    ],
                    _n3_padding: 0.0,
                    material_idx,
                    transform_idx,
                    is_smooth: 1,
                    _padding: 0,
                });
            }
        }
    }
}
