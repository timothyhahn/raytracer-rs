struct Camera {
    transform_inverse: mat4x4<f32>,
    pixel_size: f32,
    half_width: f32,
    half_height: f32,
    aa_samples: u32,
    chunk_samples: u32,
    sample_offset: u32,
    samples_per_axis: u32,
    width: u32,
    height: u32,
    sphere_count: u32,
    plane_count: u32,
    cube_count: u32,
    cylinder_count: u32,
    cone_count: u32,
    triangle_count: u32,
    max_depth: u32,
    random_seed: u32,
    _padding0: u32,
    _padding1: u32,
    _padding2: u32,
    _padding3: u32,
    _padding4: u32,
    _padding5: u32,
    _padding6: u32,
}

struct Material {
    color: vec3<f32>,
    _color_padding: f32,
    ambient: f32,
    diffuse: f32,
    specular: f32,
    shininess: f32,
    reflectivity: f32,
    transparency: f32,
    refractive_index: f32,
    pattern_type: u32,
    pattern_color_a: vec3<f32>,
    _pattern_a_padding: f32,
    pattern_color_b: vec3<f32>,
    _pattern_b_padding: f32,
    pattern_transform_idx: u32,
    _padding: array<u32, 11>,
}

struct Sphere {
    center: vec3<f32>,
    _center_padding: f32,
    radius: f32,
    material_idx: u32,
    transform_idx: u32,
    _padding: u32,
}

struct Plane {
    normal: vec3<f32>,
    distance: f32,
    material_idx: u32,
    transform_idx: u32,
    _padding: vec2<u32>,
}

struct Cube {
    material_idx: u32,
    transform_idx: u32,
    _padding: vec2<u32>,
}

struct Cylinder {
    minimum: f32,
    maximum: f32,
    closed: u32,
    material_idx: u32,
    transform_idx: u32,
    _padding: vec3<u32>,
}

struct Cone {
    minimum: f32,
    maximum: f32,
    closed: u32,
    material_idx: u32,
    transform_idx: u32,
    _padding: vec3<u32>,
}

struct Triangle {
    p1: vec3<f32>,
    _p1_padding: f32,
    p2: vec3<f32>,
    _p2_padding: f32,
    p3: vec3<f32>,
    _p3_padding: f32,
    e1: vec3<f32>,
    _e1_padding: f32,
    e2: vec3<f32>,
    _e2_padding: f32,
    n1: vec3<f32>,
    _n1_padding: f32,
    n2: vec3<f32>,
    _n2_padding: f32,
    n3: vec3<f32>,
    _n3_padding: f32,
    material_idx: u32,
    transform_idx: u32,
    is_smooth: u32,
    _padding: u32,
}

struct Transform {
    forward: mat4x4<f32>,
    inverse: mat4x4<f32>,
    inverse_transpose: mat4x4<f32>,
}

struct Light {
    @align(16) light_type: u32,
    @align(16) position: vec3<f32>,
    @align(16) intensity: vec3<f32>,
    @align(16) uvec: vec3<f32>,
    @align(16) vvec: vec3<f32>,
    @align(16) usteps: u32,
    vsteps: u32,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct HitInfo {
    hit: bool,
    t: f32,                    // Distance in object space (for internal use)
    distance: f32,             // Distance in world space (for comparison)
    point: vec3<f32>,          // Hit point in world space
    object_point: vec3<f32>,   // Hit point in object space (for pattern evaluation)
    normal: vec3<f32>,
    material_idx: u32,
}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<storage, read> spheres: array<Sphere>;
@group(0) @binding(3) var<storage, read> materials: array<Material>;
@group(0) @binding(4) var<uniform> light: Light;
@group(0) @binding(5) var<storage, read> planes: array<Plane>;
@group(0) @binding(6) var<storage, read> cubes: array<Cube>;
@group(0) @binding(7) var<storage, read> cylinders: array<Cylinder>;
@group(0) @binding(8) var<storage, read> cones: array<Cone>;
@group(0) @binding(9) var<storage, read> transforms: array<Transform>;
@group(0) @binding(10) var<storage, read> triangles: array<Triangle>;
@group(0) @binding(11) var<storage, read> bvh_nodes: array<BVHNode>;
@group(0) @binding(12) var<storage, read> bvh_triangle_indices: array<u32>;

struct BVHNode {
    min: vec3<f32>,
    _min_padding: f32,
    max: vec3<f32>,
    _max_padding: f32,
    left_or_first: u32,
    right_or_count: u32,
    is_leaf: u32,
    _padding: u32,
}

fn transform_ray(ray: Ray, transform: Transform) -> Ray {
    let origin_transformed = transform.inverse * vec4<f32>(ray.origin, 1.0);
    let direction_transformed = transform.inverse * vec4<f32>(ray.direction, 0.0);

    return Ray(origin_transformed.xyz, normalize(direction_transformed.xyz));
}

fn transform_normal(normal: vec3<f32>, transform: Transform) -> vec3<f32> {
    let normal_transformed = transform.inverse_transpose * vec4<f32>(normal, 0.0);
    return normalize(normal_transformed.xyz);
}

fn init_hit(material_idx: u32) -> HitInfo {
    var hit: HitInfo;
    hit.hit = false;
    hit.material_idx = material_idx;
    return hit;
}

fn finalize_hit(hit_in: HitInfo, t: f32, local_ray: Ray, ray: Ray, transform: Transform, local_normal: vec3<f32>) -> HitInfo {
    var hit = hit_in;
    hit.hit = true;
    hit.t = t;
    let local_point = local_ray.origin + local_ray.direction * t;
    hit.object_point = local_point;
    let world_point = transform.forward * vec4<f32>(local_point, 1.0);
    hit.point = world_point.xyz;
    hit.distance = length(hit.point - ray.origin);
    hit.normal = transform_normal(local_normal, transform);
    return hit;
}

fn is_valid_transform(transform: Transform) -> bool {
    let EPSILON = 0.0001;
    let matrix_sum = abs(transform.inverse[0][0]) + abs(transform.inverse[0][1]) + abs(transform.inverse[0][2]) +
                     abs(transform.inverse[1][0]) + abs(transform.inverse[1][1]) + abs(transform.inverse[1][2]) +
                     abs(transform.inverse[2][0]) + abs(transform.inverse[2][1]) + abs(transform.inverse[2][2]);
    return matrix_sum >= EPSILON;
}

fn check_cap(ray: Ray, t: f32, radius: f32) -> bool {
    let x = ray.origin.x + t * ray.direction.x;
    let z = ray.origin.z + t * ray.direction.z;
    return (x * x + z * z) <= (radius * radius);
}

fn intersect_aabb(ray: Ray, box_min: vec3<f32>, box_max: vec3<f32>) -> bool {
    var t_min = (box_min - ray.origin) / ray.direction;
    var t_max = (box_max - ray.origin) / ray.direction;

    let tx1 = min(t_min.x, t_max.x);
    let tx2 = max(t_min.x, t_max.x);
    let ty1 = min(t_min.y, t_max.y);
    let ty2 = max(t_min.y, t_max.y);
    let tz1 = min(t_min.z, t_max.z);
    let tz2 = max(t_min.z, t_max.z);

    let t_near = max(max(tx1, ty1), tz1);
    let t_far = min(min(tx2, ty2), tz2);

    return t_near <= t_far && t_far >= 0.0001;
}

fn traverse_bvh(ray: Ray) -> HitInfo {
    var closest_hit: HitInfo;
    closest_hit.hit = false;
    closest_hit.distance = 1000000.0;
    closest_hit.material_idx = 0u;

    if (arrayLength(&bvh_nodes) == 0u) {
        return closest_hit;
    }

    var stack: array<u32, 64>;
    var stack_ptr = 0u;
    stack[0] = 0u;
    stack_ptr = 1u;

    while (stack_ptr > 0u) {
        stack_ptr -= 1u;
        let node_idx = stack[stack_ptr];
        let node = bvh_nodes[node_idx];

        if (!intersect_aabb(ray, node.min, node.max)) {
            continue;
        }

        if (node.is_leaf == 1u) {
            let first_tri = node.left_or_first;
            let tri_count = node.right_or_count;

            for (var i = 0u; i < tri_count; i++) {
                let tri_idx = bvh_triangle_indices[first_tri + i];
                let hit = intersect_triangle(ray, triangles[tri_idx]);
                if (hit.hit && hit.distance < closest_hit.distance) {
                    closest_hit = hit;
                }
            }
        } else {
            let left_idx = node.left_or_first;
            let right_idx = node.right_or_count;

            if (stack_ptr < 64u) {
                stack[stack_ptr] = left_idx;
                stack_ptr += 1u;
            }
            if (stack_ptr < 64u) {
                stack[stack_ptr] = right_idx;
                stack_ptr += 1u;
            }
        }
    }

    return closest_hit;
}

fn traverse_bvh_shadow(ray: Ray, max_distance: f32) -> bool {
    if (arrayLength(&bvh_nodes) == 0u) {
        return false;
    }

    var stack: array<u32, 64>;
    var stack_ptr = 0u;
    stack[0] = 0u;
    stack_ptr = 1u;

    while (stack_ptr > 0u) {
        stack_ptr -= 1u;
        let node_idx = stack[stack_ptr];
        let node = bvh_nodes[node_idx];

        if (!intersect_aabb(ray, node.min, node.max)) {
            continue;
        }

        if (node.is_leaf == 1u) {
            let first_tri = node.left_or_first;
            let tri_count = node.right_or_count;

            for (var i = 0u; i < tri_count; i++) {
                let tri_idx = bvh_triangle_indices[first_tri + i];
                let hit = intersect_triangle(ray, triangles[tri_idx]);
                if (hit.hit && hit.distance < max_distance) {
                    return true;
                }
            }
        } else {
            let left_idx = node.left_or_first;
            let right_idx = node.right_or_count;

            if (stack_ptr < 64u) {
                stack[stack_ptr] = left_idx;
                stack_ptr += 1u;
            }
            if (stack_ptr < 64u) {
                stack[stack_ptr] = right_idx;
                stack_ptr += 1u;
            }
        }
    }

    return false;
}

// Offset by 0.5 for pixel center
fn ray_for_pixel(px: u32, py: u32) -> Ray {
    let x_offset = (f32(px) + 0.5) * camera.pixel_size;
    let y_offset = (f32(py) + 0.5) * camera.pixel_size;

    let world_x = camera.half_width - x_offset;
    let world_y = camera.half_height - y_offset;

    let pixel_world = camera.transform_inverse * vec4<f32>(world_x, world_y, -1.0, 1.0);
    let origin_world = camera.transform_inverse * vec4<f32>(0.0, 0.0, 0.0, 1.0);

    let direction = normalize(pixel_world.xyz - origin_world.xyz);

    return Ray(origin_world.xyz, direction);
}

fn intersect_sphere(ray: Ray, sphere: Sphere) -> HitInfo {
    var hit = init_hit(sphere.material_idx);

    if (sphere.radius <= 0.0) {
        return hit;
    }

    let transform = transforms[sphere.transform_idx];
    let local_ray = transform_ray(ray, transform);
    let sphere_to_ray = local_ray.origin - sphere.center;

    let a = dot(local_ray.direction, local_ray.direction);
    let b = 2.0 * dot(local_ray.direction, sphere_to_ray);
    let c = dot(sphere_to_ray, sphere_to_ray) - sphere.radius * sphere.radius;
    let discriminant = b * b - 4.0 * a * c;

    if (discriminant < 0.0) {
        return hit;
    }

    let sqrt_disc = sqrt(discriminant);
    let t1 = (-b - sqrt_disc) / (2.0 * a);
    let t2 = (-b + sqrt_disc) / (2.0 * a);

    var t = t1;
    if (t < 0.001) {
        t = t2;
    }

    if (t < 0.001) {
        return hit;
    }

    let local_point = local_ray.origin + local_ray.direction * t;
    let local_normal = normalize(local_point - sphere.center);
    return finalize_hit(hit, t, local_ray, ray, transform, local_normal);
}

fn intersect_plane(ray: Ray, plane: Plane) -> HitInfo {
    var hit = init_hit(plane.material_idx);
    let EPSILON = 0.0001;

    if (length(plane.normal) < EPSILON) {
        return hit;
    }

    let transform = transforms[plane.transform_idx];
    let local_ray = transform_ray(ray, transform);
    let local_normal = vec3<f32>(0.0, 1.0, 0.0);

    let denom = dot(local_ray.direction, local_normal);
    if (abs(denom) < EPSILON) {
        return hit;
    }

    let t = -local_ray.origin.y / local_ray.direction.y;
    if (t < 0.001) {
        return hit;
    }

    return finalize_hit(hit, t, local_ray, ray, transform, local_normal);
}

// Helper for cube intersection - check one axis
fn check_axis(origin: f32, direction: f32) -> vec2<f32> {
    let EPSILON = 0.0001;
    let tmin_numerator = -1.0 - origin;
    let tmax_numerator = 1.0 - origin;

    var tmin: f32;
    var tmax: f32;

    if (abs(direction) >= EPSILON) {
        tmin = tmin_numerator / direction;
        tmax = tmax_numerator / direction;
    } else {
        // Direction is near zero - ray is parallel to slab
        tmin = tmin_numerator * 1e30;  // Simulate infinity
        tmax = tmax_numerator * 1e30;
    }

    // Ensure tmin < tmax
    if (tmin > tmax) {
        return vec2<f32>(tmax, tmin);
    }

    return vec2<f32>(tmin, tmax);
}

fn intersect_cube(ray: Ray, cube: Cube) -> HitInfo {
    var hit = init_hit(cube.material_idx);
    let EPSILON = 0.0001;

    let transform = transforms[cube.transform_idx];
    let local_ray = transform_ray(ray, transform);

    let x_range = check_axis(local_ray.origin.x, local_ray.direction.x);
    var tmin = x_range.x;
    var tmax = x_range.y;

    let y_range = check_axis(local_ray.origin.y, local_ray.direction.y);
    tmin = max(tmin, y_range.x);
    tmax = min(tmax, y_range.y);

    if (tmin > tmax) {
        return hit;
    }

    let z_range = check_axis(local_ray.origin.z, local_ray.direction.z);
    tmin = max(tmin, z_range.x);
    tmax = min(tmax, z_range.y);

    if (tmin > tmax) {
        return hit;
    }

    var t = tmin;
    if (t < 0.001) {
        t = tmax;
    }

    if (t < 0.001) {
        return hit;
    }

    let local_point = local_ray.origin + local_ray.direction * t;
    let maxc = max(abs(local_point.x), max(abs(local_point.y), abs(local_point.z)));

    var local_normal: vec3<f32>;
    if (abs(maxc - abs(local_point.x)) < EPSILON) {
        local_normal = vec3<f32>(sign(local_point.x), 0.0, 0.0);
    } else if (abs(maxc - abs(local_point.y)) < EPSILON) {
        local_normal = vec3<f32>(0.0, sign(local_point.y), 0.0);
    } else {
        local_normal = vec3<f32>(0.0, 0.0, sign(local_point.z));
    }

    return finalize_hit(hit, t, local_ray, ray, transform, local_normal);
}

fn intersect_cylinder(ray: Ray, cylinder: Cylinder) -> HitInfo {
    var hit = init_hit(cylinder.material_idx);
    let EPSILON = 0.0001;

    let transform = transforms[cylinder.transform_idx];
    if (!is_valid_transform(transform)) {
        return hit;
    }

    if (abs(cylinder.minimum) < EPSILON && abs(cylinder.maximum) < EPSILON) {
        return hit;
    }

    let local_ray = transform_ray(ray, transform);

    // Calculate intersection with cylindrical surface
    let a = local_ray.direction.x * local_ray.direction.x + local_ray.direction.z * local_ray.direction.z;

    // Special case: ray is parallel to Y axis
    if (abs(a) < EPSILON) {
        // Only check caps if cylinder is closed
        if (cylinder.closed == 0u) {
            return hit;
        }
        // Will handle caps below
    }

    var t_min = 999999.0;
    var t_max = -999999.0;
    var found_surface_hit = false;

    // Find intersections with cylindrical surface if not parallel
    if (abs(a) >= EPSILON) {
        let b = 2.0 * local_ray.origin.x * local_ray.direction.x + 2.0 * local_ray.origin.z * local_ray.direction.z;
        let c = local_ray.origin.x * local_ray.origin.x + local_ray.origin.z * local_ray.origin.z - 1.0;

        let discriminant = b * b - 4.0 * a * c;

        if (discriminant >= 0.0) {
            let sqrt_disc = sqrt(discriminant);
            var t0 = (-b - sqrt_disc) / (2.0 * a);
            var t1 = (-b + sqrt_disc) / (2.0 * a);

            if (t0 > t1) {
                let temp = t0;
                t0 = t1;
                t1 = temp;
            }

            // Check if y values are within bounds
            let y0 = local_ray.origin.y + t0 * local_ray.direction.y;
            let y1 = local_ray.origin.y + t1 * local_ray.direction.y;

            if (cylinder.minimum < y0 && y0 < cylinder.maximum) {
                if (t0 > 0.001 && t0 < t_min) {
                    t_min = t0;
                    found_surface_hit = true;
                }
            }

            if (cylinder.minimum < y1 && y1 < cylinder.maximum) {
                if (t1 > 0.001) {
                    if (!found_surface_hit || t1 < t_min) {
                        t_min = t1;
                        found_surface_hit = true;
                    }
                }
            }
        }
    }

    // Check caps if cylinder is closed
    if (cylinder.closed != 0u && abs(local_ray.direction.y) >= EPSILON) {
        let t_lower = (cylinder.minimum - local_ray.origin.y) / local_ray.direction.y;
        if (check_cap(local_ray, t_lower, 1.0) && t_lower > 0.001) {
            if (!found_surface_hit || t_lower < t_min) {
                t_min = t_lower;
                found_surface_hit = true;
            }
        }

        let t_upper = (cylinder.maximum - local_ray.origin.y) / local_ray.direction.y;
        if (check_cap(local_ray, t_upper, 1.0) && t_upper > 0.001) {
            if (!found_surface_hit || t_upper < t_min) {
                t_min = t_upper;
                found_surface_hit = true;
            }
        }
    }

    if (!found_surface_hit) {
        return hit;
    }

    let local_point = local_ray.origin + local_ray.direction * t_min;
    let dist_from_axis = local_point.x * local_point.x + local_point.z * local_point.z;

    var local_normal: vec3<f32>;
    if (dist_from_axis < 1.0 && local_point.y >= cylinder.maximum - EPSILON) {
        local_normal = vec3<f32>(0.0, 1.0, 0.0);
    } else if (dist_from_axis < 1.0 && local_point.y <= cylinder.minimum + EPSILON) {
        local_normal = vec3<f32>(0.0, -1.0, 0.0);
    } else {
        local_normal = normalize(vec3<f32>(local_point.x, 0.0, local_point.z));
    }

    return finalize_hit(hit, t_min, local_ray, ray, transform, local_normal);
}

fn intersect_cone(ray: Ray, cone: Cone) -> HitInfo {
    var hit = init_hit(cone.material_idx);
    let EPSILON = 0.0001;

    let transform = transforms[cone.transform_idx];
    if (!is_valid_transform(transform)) {
        return hit;
    }

    if (abs(cone.minimum) < EPSILON && abs(cone.maximum) < EPSILON) {
        return hit;
    }

    let local_ray = transform_ray(ray, transform);

    // Calculate intersection with conical surface
    // For cone: x² + z² = y² -> x² - y² + z² = 0
    let a = local_ray.direction.x * local_ray.direction.x - local_ray.direction.y * local_ray.direction.y + local_ray.direction.z * local_ray.direction.z;
    let b = 2.0 * local_ray.origin.x * local_ray.direction.x - 2.0 * local_ray.origin.y * local_ray.direction.y + 2.0 * local_ray.origin.z * local_ray.direction.z;
    let c = local_ray.origin.x * local_ray.origin.x - local_ray.origin.y * local_ray.origin.y + local_ray.origin.z * local_ray.origin.z;

    var t_min = 999999.0;
    var t_max = -999999.0;
    var found_surface_hit = false;

    // Special case: a ≈ 0 (ray parallel to cone surface in some way)
    if (abs(a) < EPSILON) {
        if (abs(b) >= EPSILON) {
            // Linear case: only one intersection
            let t = -c / (2.0 * b);
            let y = local_ray.origin.y + t * local_ray.direction.y;
            if (t > 0.001 && cone.minimum < y && y < cone.maximum) {
                t_min = t;
                found_surface_hit = true;
            }
        }
        // If both a ≈ 0 and b ≈ 0, only check caps below
    } else {
        // Quadratic case
        let discriminant = b * b - 4.0 * a * c;

        if (discriminant >= 0.0) {
            let sqrt_disc = sqrt(discriminant);
            var t0 = (-b - sqrt_disc) / (2.0 * a);
            var t1 = (-b + sqrt_disc) / (2.0 * a);

            if (t0 > t1) {
                let temp = t0;
                t0 = t1;
                t1 = temp;
            }

            // Check if y values are within bounds
            let y0 = local_ray.origin.y + t0 * local_ray.direction.y;
            let y1 = local_ray.origin.y + t1 * local_ray.direction.y;

            if (cone.minimum < y0 && y0 < cone.maximum) {
                if (t0 > 0.001 && t0 < t_min) {
                    t_min = t0;
                    found_surface_hit = true;
                }
            }

            if (cone.minimum < y1 && y1 < cone.maximum) {
                if (t1 > 0.001) {
                    if (!found_surface_hit || t1 < t_min) {
                        t_min = t1;
                        found_surface_hit = true;
                    }
                }
            }
        }
    }

    // Check caps if cone is closed
    if (cone.closed != 0u && abs(local_ray.direction.y) >= EPSILON) {
        let t_lower = (cone.minimum - local_ray.origin.y) / local_ray.direction.y;
        if (check_cap(local_ray, t_lower, abs(cone.minimum)) && t_lower > 0.001) {
            if (!found_surface_hit || t_lower < t_min) {
                t_min = t_lower;
                found_surface_hit = true;
            }
        }

        let t_upper = (cone.maximum - local_ray.origin.y) / local_ray.direction.y;
        if (check_cap(local_ray, t_upper, abs(cone.maximum)) && t_upper > 0.001) {
            if (!found_surface_hit || t_upper < t_min) {
                t_min = t_upper;
                found_surface_hit = true;
            }
        }
    }

    if (!found_surface_hit) {
        return hit;
    }

    let local_point = local_ray.origin + local_ray.direction * t_min;
    let dist_from_axis = local_point.x * local_point.x + local_point.z * local_point.z;

    var local_normal: vec3<f32>;
    if (dist_from_axis < 1.0 && local_point.y >= cone.maximum - EPSILON) {
        local_normal = vec3<f32>(0.0, 1.0, 0.0);
    } else if (dist_from_axis < 1.0 && local_point.y <= cone.minimum + EPSILON) {
        local_normal = vec3<f32>(0.0, -1.0, 0.0);
    } else {
        var y_normal = sqrt(dist_from_axis);
        if (local_point.y > 0.0) {
            y_normal = -y_normal;
        }
        local_normal = normalize(vec3<f32>(local_point.x, y_normal, local_point.z));
    }

    return finalize_hit(hit, t_min, local_ray, ray, transform, local_normal);
}

// Möller-Trumbore ray-triangle intersection
// Returns (hit, t, u, v) where u,v are barycentric coordinates
fn moller_trumbore(p1: vec3<f32>, e1: vec3<f32>, e2: vec3<f32>, ray: Ray) -> vec4<f32> {
    // Two different epsilons for different purposes:
    let DET_EPSILON = 0.000001;  // For determinant - detects tiny triangles (Stanford Bunny)
    let T_EPSILON = 0.001;        // For t-distance - prevents self-intersection (lighting)

    let dir_cross_e2 = cross(ray.direction, e2);
    let det = dot(e1, dir_cross_e2);

    // If determinant is near zero, ray lies in plane of triangle or is parallel
    if (abs(det) < DET_EPSILON) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0); // No hit
    }

    let f = 1.0 / det;
    let p1_to_origin = ray.origin - p1;
    let u = f * dot(p1_to_origin, dir_cross_e2);

    if (u < 0.0 || u > 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0); // No hit
    }

    let origin_cross_e1 = cross(p1_to_origin, e1);
    let v = f * dot(ray.direction, origin_cross_e1);

    if (v < 0.0 || (u + v) > 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0); // No hit
    }

    let t = f * dot(e2, origin_cross_e1);

    if (t > T_EPSILON) {  // Use larger epsilon to prevent self-intersection
        return vec4<f32>(1.0, t, u, v); // Hit with t, u, v
    }

    return vec4<f32>(0.0, 0.0, 0.0, 0.0); // No hit
}

// Handles both regular and smooth triangles
fn intersect_triangle(ray: Ray, triangle: Triangle) -> HitInfo {
    var hit = init_hit(triangle.material_idx);

    let transform = transforms[triangle.transform_idx];
    if (!is_valid_transform(transform)) {
        return hit;
    }

    let local_ray = transform_ray(ray, transform);
    let result = moller_trumbore(triangle.p1, triangle.e1, triangle.e2, local_ray);

    if (result.x == 0.0) {
        return hit;
    }

    let t = result.y;
    let u = result.z;
    let v = result.w;

    var local_normal: vec3<f32>;
    if (triangle.is_smooth == 1u) {
        let interpolated = triangle.n2 * u + triangle.n3 * v + triangle.n1 * (1.0 - u - v);
        if (length(interpolated) < 0.0001) {
            local_normal = normalize(cross(triangle.e1, triangle.e2));
        } else {
            local_normal = normalize(interpolated);
        }
    } else {
        local_normal = normalize(cross(triangle.e1, triangle.e2));
    }

    return finalize_hit(hit, t, local_ray, ray, transform, local_normal);
}

// Find closest intersection with scene
fn intersect_scene(ray: Ray) -> HitInfo {
    var closest_hit: HitInfo;
    closest_hit.hit = false;
    closest_hit.distance = 999999.0;

    // Test all spheres (use actual count from camera uniform)
    for (var i = 0u; i < camera.sphere_count; i++) {
        let hit = intersect_sphere(ray, spheres[i]);
        if (hit.hit && hit.distance < closest_hit.distance) {
            closest_hit = hit;
        }
    }

    // Test all planes
    for (var i = 0u; i < camera.plane_count; i++) {
        let hit = intersect_plane(ray, planes[i]);
        if (hit.hit && hit.distance < closest_hit.distance) {
            closest_hit = hit;
        }
    }

    // Test all cubes
    for (var i = 0u; i < camera.cube_count; i++) {
        let hit = intersect_cube(ray, cubes[i]);
        if (hit.hit && hit.distance < closest_hit.distance) {
            closest_hit = hit;
        }
    }

    // Test all cylinders
    for (var i = 0u; i < camera.cylinder_count; i++) {
        let hit = intersect_cylinder(ray, cylinders[i]);
        if (hit.hit && hit.distance < closest_hit.distance) {
            closest_hit = hit;
        }
    }

    // Test all cones
    for (var i = 0u; i < camera.cone_count; i++) {
        let hit = intersect_cone(ray, cones[i]);
        if (hit.hit && hit.distance < closest_hit.distance) {
            closest_hit = hit;
        }
    }

    // Test all triangles using BVH acceleration
    let bvh_hit = traverse_bvh(ray);
    if (bvh_hit.hit && bvh_hit.distance < closest_hit.distance) {
        closest_hit = bvh_hit;
    }

    return closest_hit;
}

// Check if a point is in shadow
// IMPORTANT: Must offset along normal to prevent self-intersection
fn is_shadowed(point: vec3<f32>, normal: vec3<f32>, light_pos: vec3<f32>) -> bool {
    let direction = light_pos - point;
    let distance = length(direction);
    // Shadow epsilon - was originally 0.01 which gave some wall visibility
    // Smaller values cause more self-shadowing on transformed geometry
    let SHADOW_EPSILON = 0.01;
    let over_point = point + normal * SHADOW_EPSILON;
    let shadow_ray = Ray(over_point, normalize(direction));

    // Check if anything blocks the light (use actual counts from camera uniform)
    // NOTE: Must use hit.distance (world-space) not hit.t (object-space) for comparison
    for (var i = 0u; i < camera.sphere_count; i++) {
        let hit = intersect_sphere(shadow_ray, spheres[i]);
        if (hit.hit && hit.distance < distance) {
            return true;
        }
    }

    // Check planes for shadows
    for (var i = 0u; i < camera.plane_count; i++) {
        let hit = intersect_plane(shadow_ray, planes[i]);
        if (hit.hit && hit.distance < distance) {
            return true;
        }
    }

    // Check cubes for shadows
    for (var i = 0u; i < camera.cube_count; i++) {
        let hit = intersect_cube(shadow_ray, cubes[i]);
        if (hit.hit && hit.distance < distance) {
            return true;
        }
    }

    // Check cylinders for shadows
    for (var i = 0u; i < camera.cylinder_count; i++) {
        let hit = intersect_cylinder(shadow_ray, cylinders[i]);
        if (hit.hit && hit.distance < distance) {
            return true;
        }
    }

    // Check cones for shadows
    for (var i = 0u; i < camera.cone_count; i++) {
        let hit = intersect_cone(shadow_ray, cones[i]);
        if (hit.hit && hit.distance < distance) {
            return true;
        }
    }

    // Check triangles for shadows using BVH acceleration
    if (arrayLength(&bvh_nodes) > 0u && traverse_bvh_shadow(shadow_ray, distance)) {
        return true;
    }

    return false;
}

// Pattern evaluation functions

// Stripe pattern: alternates between two colors based on x coordinate
fn stripe_at(point: vec3<f32>, color_a: vec3<f32>, color_b: vec3<f32>) -> vec3<f32> {
    // Add epsilon to avoid precision issues at boundaries
    let x = i32(floor(point.x + PATTERN_EPSILON));
    if (x & 1) == 0 {
        return color_a;
    } else {
        return color_b;
    }
}

// Gradient pattern: linear interpolation between two colors based on x coordinate
fn gradient_at(point: vec3<f32>, color_a: vec3<f32>, color_b: vec3<f32>) -> vec3<f32> {
    let distance = color_b - color_a;
    let fraction = clamp(point.x, 0.0, 1.0);  // Match CPU: clamp to [0, 1]
    return color_a + distance * fraction;
}

// Ring pattern: concentric rings based on distance from y-axis
fn ring_at(point: vec3<f32>, color_a: vec3<f32>, color_b: vec3<f32>) -> vec3<f32> {
    let distance_from_y = sqrt(point.x * point.x + point.z * point.z);
    let value = i32(floor(distance_from_y + PATTERN_EPSILON));
    if (value & 1) == 0 {
        return color_a;
    } else {
        return color_b;
    }
}

// Checkers pattern: 3D checkerboard based on floor of x, y, z coordinates
fn checkers_at(point: vec3<f32>, color_a: vec3<f32>, color_b: vec3<f32>) -> vec3<f32> {
    // Add epsilon to avoid precision issues at boundaries (match CPU implementation)
    let x = i32(floor(point.x + PATTERN_EPSILON));
    let y = i32(floor(point.y + PATTERN_EPSILON));
    let z = i32(floor(point.z + PATTERN_EPSILON));

    if ((x + y + z) & 1) == 0 {
        return color_a;
    } else {
        return color_b;
    }
}

fn evaluate_pattern(pattern_type: u32, pattern_point: vec3<f32>, color_a: vec3<f32>, color_b: vec3<f32>) -> vec3<f32> {
    switch (pattern_type) {
        case 1u: {
            return stripe_at(pattern_point, color_a, color_b);
        }
        case 2u: {
            return gradient_at(pattern_point, color_a, color_b);
        }
        case 3u: {
            return ring_at(pattern_point, color_a, color_b);
        }
        case 4u: {
            return checkers_at(pattern_point, color_a, color_b);
        }
        default: {
            return color_a;
        }
    }
}

fn get_surface_color(hit: HitInfo) -> vec3<f32> {
    let material = materials[hit.material_idx];

    if (material.pattern_type == 0u) {
        return material.color;
    }

    let pattern_transform = transforms[material.pattern_transform_idx];
    let pattern_point = (pattern_transform.inverse * vec4<f32>(hit.object_point, 1.0)).xyz;
    return evaluate_pattern(material.pattern_type, pattern_point, material.pattern_color_a, material.pattern_color_b);
}

// Simple hash function for pseudo-random number generation
// Based on PCG (Permuted Congruential Generator)
fn hash(seed: u32) -> u32 {
    var state = seed * 747796405u + 2891336453u;
    var word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

// Generate pseudo-random float in [0, 1) from seed
fn random_float(seed: u32) -> f32 {
    return f32(hash(seed)) / 4294967296.0; // 2^32
}

// Calculate a point on an area light surface
// u, v are sample indices in range [0, usteps), [0, vsteps)
// pixel_x, pixel_y used for seeding the RNG for jittering
fn point_on_light(u: u32, v: u32, pixel_x: u32, pixel_y: u32) -> vec3<f32> {
    // Calculate offset as center of each cell
    var u_offset = (f32(u) + 0.5) / f32(light.usteps);
    var v_offset = (f32(v) + 0.5) / f32(light.vsteps);

    // Add jitter for soft shadow quality
    let jitter_amount = 1.0 / (f32(light.usteps) * 2.0);

    // Create unique seed from pixel coordinates and sample indices
    // Incorporate random_seed to vary jitter per iteration (for progressive rendering)
    let seed_u = pixel_x + pixel_y * 1000u + u * 100000u + v * 10000000u + camera.random_seed * 100000000u;
    let seed_v = seed_u + 123456789u;

    let jitter_u = random_float(seed_u) * jitter_amount - jitter_amount * 0.5;
    let jitter_v = random_float(seed_v) * jitter_amount - jitter_amount * 0.5;

    u_offset += jitter_u;
    v_offset += jitter_v;

    return light.position + light.uvec * u_offset + light.vvec * v_offset;
}

// Phong lighting calculation with pattern support and area light sampling
fn phong_lighting(
    hit: HitInfo,
    eye_vector: vec3<f32>,
    pixel_x: u32,
    pixel_y: u32
) -> vec3<f32> {
    let material = materials[hit.material_idx];
    let surface_color = get_surface_color(hit);

    // Flip normal if pointing away from eye (like CPU prepare_computations does)
    // This makes surfaces double-sided for lighting
    var normal = hit.normal;
    if (dot(normal, eye_vector) < 0.0) {
        normal = -normal;
    }

    // TEMP: Visualize normals as colors for debugging
    let debug_normals = false;  // Disabled - normals look correct
    if (debug_normals) {
        return (normal + vec3<f32>(1.0)) * 0.5;  // Map -1..1 to 0..1
    }

    // Sample the area light (or treat as point light if usteps/vsteps = 1)
    let sample_count = light.usteps * light.vsteps;
    var color_sum = vec3<f32>(0.0, 0.0, 0.0);

    for (var v = 0u; v < light.vsteps; v++) {
        for (var u = 0u; u < light.usteps; u++) {
            // Get the position of this sample on the light (with jitter)
            let light_sample_pos = point_on_light(u, v, pixel_x, pixel_y);

            // Check if this sample is shadowed
            // IMPORTANT: Use flipped normal for shadow ray offset, not geometric normal!
            // The flipped normal ensures we offset away from surface for both front/back faces
            let in_shadow = is_shadowed(hit.point, normal, light_sample_pos);

            // Sample intensity (divide total intensity by number of samples, like CPU does)
            let sample_intensity = light.intensity / f32(sample_count);

            // Ambient contribution (ALWAYS added, even in shadow - multiply by sample intensity)
            // CPU: effective_color = color * light.intensity, then ambient = effective_color * material.ambient
            let effective_color = surface_color * sample_intensity;
            var sample_color = effective_color * material.ambient;

            // Light direction and distance for this sample
            let light_dir = normalize(light_sample_pos - hit.point);
            let light_dot_normal = dot(light_dir, normal);  // Use flipped normal

            // Diffuse and specular ONLY if not in shadow and light hits surface
            if (!in_shadow && light_dot_normal > 0.0) {
                // Diffuse contribution
                let diffuse = surface_color * material.diffuse * light_dot_normal;
                sample_color += diffuse * sample_intensity;

                // Specular contribution
                let reflect_dir = reflect(-light_dir, normal);  // Use flipped normal
                let reflect_dot_eye = dot(reflect_dir, eye_vector);

                if (reflect_dot_eye > 0.0) {
                    let specular_factor = pow(reflect_dot_eye, material.shininess);
                    // CPU: specular = light.intensity * material.specular * factor
                    let specular = sample_intensity * material.specular * specular_factor;
                    sample_color += specular;
                }
            }

            // Add this sample's contribution (ambient always, diffuse/specular only if lit)
            color_sum += sample_color;
        }
    }

    return color_sum;
}

// Trace a ray and return the color
// Reflect a vector around a normal
fn reflect(incident: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    return incident - normal * 2.0 * dot(incident, normal);
}

// Refract a vector through a surface using Snell's law
// Returns vec4 where:
//   - xyz is the refracted direction
//   - w is 1.0 if refraction occurred, 0.0 if total internal reflection
fn refract(incident: vec3<f32>, normal: vec3<f32>, n1: f32, n2: f32) -> vec4<f32> {
    let n_ratio = n1 / n2;
    let cos_i = dot(incident, normal);
    let sin2_t = n_ratio * n_ratio * (1.0 - cos_i * cos_i);

    // Total internal reflection
    if (sin2_t > 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let cos_t = sqrt(1.0 - sin2_t);
    let direction = normal * (n_ratio * cos_i - cos_t) - incident * n_ratio;

    return vec4<f32>(direction, 1.0);
}

// Schlick approximation for Fresnel reflectance
fn schlick(cosine: f32, n1: f32, n2: f32) -> f32 {
    var cos = cosine;

    // When going from denser to less dense medium (e.g., glass to air)
    if (n1 > n2) {
        let n = n1 / n2;
        let sin_2t = n * n * (1.0 - cos * cos);

        // Total internal reflection
        if (sin_2t > 1.0) {
            return 1.0;
        }

        // Recalculate cosine with refracted angle
        cos = sqrt(1.0 - sin_2t);
    }

    let r0 = ((n1 - n2) / (n1 + n2)) * ((n1 - n2) / (n1 + n2));
    return r0 + (1.0 - r0) * pow(1.0 - cos, 5.0);
}

// Ray job for iterative tracing
struct RayJob {
    ray: Ray,
    depth: u32,
    weight: f32,
    inside_object: bool,  // Track if we're inside a transparent object
}

// Maximum queue size for iterative ray tracing
// For depth d with binary tree (reflection + refraction), need 2^(d+1) - 1 slots
// depth 5: 2^6 - 1 = 63 jobs
// depth 4: 2^5 - 1 = 31 jobs
// depth 3: 2^4 - 1 = 15 jobs
const MAX_QUEUE_SIZE: u32 = 64u;  // Support depth 5 safely
// Pattern epsilon to avoid precision issues at boundaries
const PATTERN_EPSILON: f32 = 0.000001;
// Early termination threshold: skip rays contributing less than 1/255 to final color
const MIN_RAY_WEIGHT: f32 = 0.004;

fn trace_ray(primary_ray: Ray, pixel_x: u32, pixel_y: u32) -> vec3<f32> {
    // Iterative ray tracing with a queue to avoid recursion
    var ray_queue: array<RayJob, MAX_QUEUE_SIZE>;
    var queue_size = 1u;

    // Initialize with primary ray
    ray_queue[0] = RayJob(primary_ray, 0u, 1.0, false);

    var final_color = vec3<f32>(0.0);

    // Process rays iteratively
    while (queue_size > 0u) {
        // Pop from queue
        queue_size -= 1u;
        let job = ray_queue[queue_size];

        // Early termination: skip rays with negligible contribution
        if (job.weight < MIN_RAY_WEIGHT) {
            continue;
        }

        // Trace this ray
        let hit = intersect_scene(job.ray);

        if (!hit.hit) {
            // Background color (black) - contributes nothing
            continue;
        }

        // Calculate eye vector (direction from hit point to camera)
        let eye_vector = normalize(-job.ray.direction);

        // Calculate surface color with lighting (area light sampling happens inside phong_lighting)
        let surface_color = phong_lighting(
            hit,
            eye_vector,
            pixel_x,
            pixel_y
        );

        // Add surface contribution
        final_color += surface_color * job.weight;

        // Only spawn reflection/refraction rays if we haven't hit max depth
        if (job.depth < camera.max_depth) {
            let EPSILON = 0.0001;
            let material = materials[hit.material_idx];  // Read material for reflectivity/transparency

            // Determine if we need Schlick blending (both reflective AND transparent)
            let use_schlick = material.reflectivity > 0.0 && material.transparency > 0.0;

            // Handle reflections (only if NOT using Schlick - Schlick case handled below)
            if (material.reflectivity > 0.0 && !use_schlick) {
                let reflect_dir = reflect(job.ray.direction, hit.normal);
                let reflect_origin = hit.point + hit.normal * EPSILON;  // Offset to avoid self-intersection
                let reflect_ray = Ray(reflect_origin, normalize(reflect_dir));

                // Add reflection ray to queue
                if (queue_size < MAX_QUEUE_SIZE) {
                    ray_queue[queue_size] = RayJob(
                        reflect_ray,
                        job.depth + 1u,
                        job.weight * material.reflectivity,
                        job.inside_object
                    );
                    queue_size += 1u;
                }
            }

            // Handle transparency and refraction
            if (material.transparency > 0.0) {
                // Flip normal to face the eye (like CPU prepare_computations)
                // This simplifies inside/outside logic
                let eye_vector = -job.ray.direction;
                var normal_facing_eye = hit.normal;
                var inside = false;
                if (dot(hit.normal, eye_vector) < 0.0) {
                    normal_facing_eye = -hit.normal;
                    inside = true;
                }

                // Calculate over_point and under_point like CPU
                let over_point = hit.point + normal_facing_eye * EPSILON;
                let under_point = hit.point - normal_facing_eye * EPSILON;

                // Determine refractive indices
                var n1 = 1.0;  // Air
                var n2 = material.refractive_index;
                if (inside) {
                    n1 = material.refractive_index;
                    n2 = 1.0;
                }

                // Try to refract using the eye vector (like CPU does)
                // CPU uses eye_vector = -ray.direction, not ray.direction!
                let refract_result = refract(eye_vector, normal_facing_eye, n1, n2);

                // Calculate Fresnel reflectance using Schlick approximation
                // Use eye_vector (not ray.direction) to match CPU
                let cos_theta = abs(dot(eye_vector, normal_facing_eye));
                let reflectance = schlick(cos_theta, n1, n2);

                if (refract_result.w > 0.0) {
                    // Refraction succeeded
                    let refract_dir = refract_result.xyz;
                    let refract_ray = Ray(under_point, normalize(refract_dir));  // Use under_point

                    // Weight by transparency and Fresnel (1 - reflectance for transmission)
                    let refract_weight = job.weight * material.transparency * (1.0 - reflectance);

                    if (queue_size < MAX_QUEUE_SIZE && refract_weight > 0.01) {
                        ray_queue[queue_size] = RayJob(
                            refract_ray,
                            job.depth + 1u,
                            refract_weight,
                            false  // Refracted ray inherits the current inside state
                        );
                        queue_size += 1u;
                    }

                    // Add reflected ray with proper weighting
                    if (reflectance > 0.01) {
                        let reflect_dir = reflect(job.ray.direction, normal_facing_eye);  // Use facing normal
                        let reflect_ray = Ray(over_point, normalize(reflect_dir));  // Use over_point

                        // If using Schlick (both reflective and transparent), blend with reflectivity
                        // Otherwise just use transparency * reflectance for Fresnel
                        var reflect_weight: f32;
                        if (use_schlick) {
                            // CPU: reflected_color * reflectance where reflected_color includes reflectivity
                            reflect_weight = job.weight * material.reflectivity * reflectance;
                        } else {
                            // Pure glass - just Fresnel reflection from transparency
                            reflect_weight = job.weight * material.transparency * reflectance;
                        }

                        if (queue_size < MAX_QUEUE_SIZE && reflect_weight > 0.01) {
                            ray_queue[queue_size] = RayJob(
                                reflect_ray,
                                job.depth + 1u,
                                reflect_weight,
                                job.inside_object
                            );
                            queue_size += 1u;
                        }
                    }
                } else {
                    // Total internal reflection - only reflect (Schlick reflectance = 1.0)
                    let reflect_dir = reflect(job.ray.direction, normal_facing_eye);
                    let reflect_ray = Ray(over_point, normalize(reflect_dir));

                    // For Schlick blending, use reflectivity (since reflectance = 1.0)
                    // For pure glass, use transparency
                    var reflect_weight: f32;
                    if (use_schlick) {
                        reflect_weight = job.weight * material.reflectivity;
                    } else {
                        reflect_weight = job.weight * material.transparency;
                    }

                    if (queue_size < MAX_QUEUE_SIZE) {
                        ray_queue[queue_size] = RayJob(
                            reflect_ray,
                            job.depth + 1u,
                            reflect_weight,
                            job.inside_object
                        );
                        queue_size += 1u;
                    }
                }
            }
        }
    }

    return final_color;
}

// Generate a jittered ray for anti-aliasing
// sample_idx is in range [0, aa_samples)
fn ray_for_pixel_aa(pixel_x: u32, pixel_y: u32, sample_idx: u32) -> Ray {
    let total_samples = max(1u, camera.aa_samples);
    let samples_per_axis = max(1u, camera.samples_per_axis);
    let global_sample_idx = (camera.sample_offset + sample_idx) % total_samples;

    // For 1 sample with random_seed=0, use pixel center (traditional rendering)
    // For 1 sample with random_seed>0, use jittered sample (progressive rendering)
    if (total_samples == 1u && camera.random_seed == 0u) {
        return ray_for_pixel(pixel_x, pixel_y);
    }

    // Progressive rendering with 1 sample: jitter within the full pixel
    if (total_samples == 1u) {
        let seed_u = pixel_x + pixel_y * 10000u
            + global_sample_idx * 100000u
            + camera.random_seed * 1000000u;
        let seed_v = seed_u + 987654321u;

        // Random offset within pixel [-0.5, 0.5]
        let offset_x = random_float(seed_u) - 0.5;
        let offset_y = random_float(seed_v) - 0.5;

        // Calculate world coordinates with random offset
        let world_x = (f32(pixel_x) + 0.5 + offset_x) * camera.pixel_size;
        let world_y = (f32(pixel_y) + 0.5 + offset_y) * camera.pixel_size;

        let pixel_offset_x = camera.half_width - world_x;
        let pixel_offset_y = camera.half_height - world_y;

        // Transform by camera inverse
        let origin = camera.transform_inverse * vec4<f32>(0.0, 0.0, 0.0, 1.0);
        let pixel = camera.transform_inverse * vec4<f32>(pixel_offset_x, pixel_offset_y, -1.0, 1.0);

        let direction = normalize(pixel.xyz - origin.xyz);

        return Ray(origin.xyz, direction);
    }

    // Stratified sampling: divide pixel into grid
    // For N samples, use ceil(sqrt(N)) x ceil(sqrt(N)) grid
    let grid_u = global_sample_idx % samples_per_axis;
    let grid_v = global_sample_idx / samples_per_axis;

    // Jitter within the grid cell
    let cell_size = 1.0 / f32(samples_per_axis);
    // Incorporate random_seed to vary jitter per iteration (for progressive rendering)
    let seed_u = pixel_x + pixel_y * 10000u
        + global_sample_idx * 100000u
        + camera.random_seed * 1000000u;
    let seed_v = seed_u + 987654321u;

    let jitter_u = random_float(seed_u) * cell_size;
    let jitter_v = random_float(seed_v) * cell_size;

    // Calculate offset from pixel center [-0.5, 0.5]
    let offset_x = (f32(grid_u) * cell_size + jitter_u) - 0.5;
    let offset_y = (f32(grid_v) * cell_size + jitter_v) - 0.5;

    // Calculate world coordinates with subpixel offset
    let world_x = (f32(pixel_x) + 0.5 + offset_x) * camera.pixel_size;
    let world_y = (f32(pixel_y) + 0.5 + offset_y) * camera.pixel_size;

    let pixel_offset_x = camera.half_width - world_x;
    let pixel_offset_y = camera.half_height - world_y;

    // Transform by camera inverse
    let origin = camera.transform_inverse * vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let pixel = camera.transform_inverse * vec4<f32>(pixel_offset_x, pixel_offset_y, -1.0, 1.0);

    let direction = normalize(pixel.xyz - origin.xyz);

    return Ray(origin.xyz, direction);
}

// Workgroup size optimization notes:
// - 8×8 (64 threads): Good balance, works on all GPUs
// - 16×16 (256 threads): Better for high-end GPUs, more parallelism
// - 32×32 (1024 threads): Maximum on most hardware, may reduce occupancy
// Current configuration is optimized for compatibility and balanced performance
const WORKGROUP_SIZE: u32 = 8u;

// Entry point: one invocation per pixel
@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_x = global_id.x;
    let pixel_y = global_id.y;

    // Bounds check
    if (pixel_x >= camera.width || pixel_y >= camera.height) {
        return;
    }


    // Anti-aliasing: trace multiple rays per pixel
    let samples_this_dispatch = select(camera.chunk_samples, camera.aa_samples, camera.chunk_samples == 0u);
    var color_sum = vec3<f32>(0.0);

    for (var sample_idx = 0u; sample_idx < samples_this_dispatch; sample_idx++) {
        // Generate jittered ray for this sample
        let ray = ray_for_pixel_aa(pixel_x, pixel_y, sample_idx);

        // Trace ray and accumulate color
        color_sum += trace_ray(ray, pixel_x, pixel_y);
    }

    // Average the samples
    let color = color_sum / f32(samples_this_dispatch);

    // Clamp color to [0, 1] range
    let clamped_color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));

    // Write to output texture
    textureStore(output_texture, vec2<i32>(i32(pixel_x), i32(pixel_y)), vec4<f32>(clamped_color, 1.0));
}
