use crate::core::matrices::Matrix4;
use crate::core::tuples::{Point, Vector};
use crate::geometry::cones::Cone;
use crate::geometry::cubes::Cube;
use crate::geometry::cylinders::Cylinder;
use crate::geometry::groups::{propagate_world_transform_to_group_children, Group};
use crate::geometry::planes::Plane;
use crate::geometry::shapes::Shape;
use crate::geometry::sphere::Sphere;
use crate::geometry::triangles::{SmoothTriangle, Triangle};
use crate::rendering::intersections::Intersection;
use crate::rendering::rays::Ray;
use crate::scene::materials::Material;

/// Trait for objects that can be intersected by rays and have surface normals.
pub trait Intersectable {
    fn intersect(&self, r: Ray) -> Vec<f64>;
    fn intersect_with_object(&self, r: Ray) -> Vec<Intersection<'_>>;
    fn normal_at(&self, p: Point) -> Vector;
    fn normal_at_with_hit(&self, p: Point, hit: Option<&Intersection>) -> Vector;
}

/// Trait for objects that have a material defining their appearance.
pub trait HasMaterial {
    fn material(&self) -> Material;
    fn set_material(&mut self, material: Material);
}

/// Trait for objects that can be transformed (translated, rotated, scaled).
pub trait Transformable {
    fn transformation(&self) -> Matrix4;
    fn set_transform(&mut self, transformation: Matrix4);
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Sphere(Sphere),
    Plane(Plane),
    Cube(Cube),
    Cylinder(Cylinder),
    Cone(Cone),
    Triangle(Triangle),
    SmoothTriangle(SmoothTriangle),
    Group(Group),
}

impl Object {
    /// Create a new default sphere at the origin with identity transformation.
    pub fn sphere() -> Self {
        Object::Sphere(Sphere::new())
    }

    /// Create a new default plane with identity transformation.
    pub fn plane() -> Self {
        Object::Plane(Plane::new())
    }

    /// Create a new default cube with identity transformation.
    pub fn cube() -> Self {
        Object::Cube(Cube::new())
    }

    /// Create a new default cylinder with identity transformation.
    pub fn cylinder() -> Self {
        Object::Cylinder(Cylinder::new())
    }

    /// Create a new default cone with identity transformation.
    pub fn cone() -> Self {
        Object::Cone(Cone::new())
    }

    /// Create a new triangle with the given vertices.
    pub fn triangle(p1: Point, p2: Point, p3: Point) -> Self {
        Object::Triangle(Triangle::new(p1, p2, p3))
    }

    /// Create a new empty group with identity transformation.
    pub fn group() -> Self {
        Object::Group(Group::new())
    }

    /// Get the parent of this object, if any.
    pub fn parent(&self) -> Option<std::rc::Weak<std::cell::RefCell<Object>>> {
        match self {
            Object::Sphere(s) => s.parent.clone(),
            Object::Plane(p) => p.parent.clone(),
            Object::Cube(c) => c.parent.clone(),
            Object::Cylinder(cy) => cy.parent.clone(),
            Object::Cone(co) => co.parent.clone(),
            Object::Triangle(t) => t.parent.clone(),
            Object::SmoothTriangle(st) => st.parent.clone(),
            Object::Group(g) => g.parent.clone(),
        }
    }

    /// Convert a point from world space to object space.
    pub fn world_to_object(&self, world_point: Point) -> Point {
        self.world_transformation()
            .inverse()
            .expect("world transformation should be invertible")
            * world_point
    }

    /// Convert a normal vector from object space to world space.
    pub fn normal_to_world(&self, object_normal: Vector) -> Vector {
        use crate::core::tuples::Tuple;

        let inverse = self
            .world_transformation()
            .inverse()
            .expect("world transformation should be invertible");
        let transformed = inverse.transpose() * object_normal;
        let normal = Vector::new(transformed.x, transformed.y, transformed.z);
        normal.normalize()
    }

    /// Set the parent of this object.
    pub fn set_parent(&mut self, parent: std::rc::Weak<std::cell::RefCell<Object>>) {
        match self {
            Object::Sphere(s) => s.parent = Some(parent),
            Object::Plane(p) => p.parent = Some(parent),
            Object::Cube(c) => c.parent = Some(parent),
            Object::Cylinder(cy) => cy.parent = Some(parent),
            Object::Cone(co) => co.parent = Some(parent),
            Object::Triangle(t) => t.parent = Some(parent),
            Object::SmoothTriangle(st) => st.parent = Some(parent),
            Object::Group(g) => g.parent = Some(parent),
        }
    }
}

impl Intersectable for Object {
    fn intersect(&self, ray: Ray) -> Vec<f64> {
        let local_ray = ray.transform(
            self.transformation()
                .inverse()
                .expect("shape transformation should be invertible"),
        );

        match self {
            Object::Sphere(s) => s.local_intersect(local_ray),
            Object::Plane(p) => p.local_intersect(local_ray),
            Object::Cube(c) => c.local_intersect(local_ray),
            Object::Cylinder(cy) => cy.local_intersect(local_ray),
            Object::Cone(co) => co.local_intersect(local_ray),
            Object::Triangle(t) => t.local_intersect(local_ray),
            Object::SmoothTriangle(st) => st.local_intersect(local_ray),
            Object::Group(g) => g.local_intersect(local_ray),
        }
    }

    fn intersect_with_object(&self, r: Ray) -> Vec<Intersection<'_>> {
        match self {
            Object::Group(g) => {
                let local_ray = r.transform(
                    g.transformation
                        .inverse()
                        .expect("group transformation should be invertible"),
                );

                let group_bounds = g.bounds();
                if !group_bounds.intersects(local_ray) {
                    return vec![];
                }

                let mut all_intersections = Vec::new();
                for child in g.children() {
                    let child_intersections = child.intersect_with_object(local_ray);
                    all_intersections.extend(child_intersections);
                }

                all_intersections
                    .sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));

                all_intersections
            }
            Object::Triangle(t) => {
                let local_ray = r.transform(
                    t.transformation
                        .inverse()
                        .expect("triangle transformation should be invertible"),
                );

                t.local_intersect_uv(local_ray)
                    .iter()
                    .map(|(t_val, u, v)| Intersection::new_with_uv(*t_val, self, *u, *v))
                    .collect()
            }
            Object::SmoothTriangle(st) => {
                let local_ray = r.transform(
                    st.transformation
                        .inverse()
                        .expect("smooth triangle transformation should be invertible"),
                );

                st.local_intersect_uv(local_ray)
                    .iter()
                    .map(|(t_val, u, v)| Intersection::new_with_uv(*t_val, self, *u, *v))
                    .collect()
            }
            _ => self
                .intersect(r)
                .iter()
                .map(|t| Intersection::new(*t, self))
                .collect(),
        }
    }

    fn normal_at(&self, world_point: Point) -> Vector {
        self.normal_at_with_hit(world_point, None)
    }

    fn normal_at_with_hit(&self, world_point: Point, hit: Option<&Intersection>) -> Vector {
        let local_point = self.world_to_object(world_point);

        let local_normal = match self {
            Object::Sphere(s) => s.local_normal_at(local_point),
            Object::Plane(p) => p.local_normal_at(local_point),
            Object::Cube(c) => c.local_normal_at(local_point),
            Object::Cylinder(cy) => cy.local_normal_at(local_point),
            Object::Cone(co) => co.local_normal_at(local_point),
            Object::Triangle(t) => t.local_normal_at(local_point),
            Object::SmoothTriangle(st) => {
                // For smooth triangles, interpolate the normal using u/v from the hit
                if let Some(intersection) = hit {
                    st.interpolated_normal(intersection.u, intersection.v)
                } else {
                    st.local_normal_at(local_point)
                }
            }
            Object::Group(g) => g.local_normal_at(local_point),
        };

        self.normal_to_world(local_normal)
    }
}

impl HasMaterial for Object {
    fn material(&self) -> Material {
        match self {
            Object::Sphere(s) => s.material,
            Object::Plane(p) => p.material,
            Object::Cube(c) => c.material,
            Object::Cylinder(cy) => cy.material,
            Object::Cone(co) => co.material,
            Object::Triangle(t) => t.material,
            Object::SmoothTriangle(st) => st.material,
            Object::Group(g) => g.material,
        }
    }

    fn set_material(&mut self, material: Material) {
        match self {
            Object::Sphere(s) => s.material = material,
            Object::Plane(p) => p.material = material,
            Object::Cube(c) => c.material = material,
            Object::Cylinder(cy) => cy.material = material,
            Object::Cone(co) => co.material = material,
            Object::Triangle(t) => t.material = material,
            Object::SmoothTriangle(st) => st.material = material,
            Object::Group(g) => g.material = material,
        }
    }
}

impl Transformable for Object {
    fn transformation(&self) -> Matrix4 {
        match self {
            Object::Sphere(s) => s.transformation,
            Object::Plane(p) => p.transformation,
            Object::Cube(c) => c.transformation,
            Object::Cylinder(cy) => cy.transformation,
            Object::Cone(co) => co.transformation,
            Object::Triangle(t) => t.transformation,
            Object::SmoothTriangle(st) => st.transformation,
            Object::Group(g) => g.transformation,
        }
    }

    fn set_transform(&mut self, transformation: Matrix4) {
        let parent_world_transform = if let Some(parent_weak) = self.parent() {
            if let Some(parent_rc) = parent_weak.upgrade() {
                parent_rc.borrow().world_transformation()
            } else {
                Matrix4::identity()
            }
        } else {
            Matrix4::identity()
        };

        let world_transform = parent_world_transform * transformation;
        self.update_transforms(transformation, world_transform);
    }
}

impl Object {
    /// Get the cached world transformation.
    pub fn world_transformation(&self) -> Matrix4 {
        match self {
            Object::Sphere(s) => s.world_transformation,
            Object::Plane(p) => p.world_transformation,
            Object::Cube(c) => c.world_transformation,
            Object::Cylinder(cy) => cy.world_transformation,
            Object::Cone(co) => co.world_transformation,
            Object::Triangle(t) => t.world_transformation,
            Object::SmoothTriangle(st) => st.world_transformation,
            Object::Group(g) => g.world_transformation,
        }
    }

    /// Set the cached world transformation.
    pub fn set_world_transform(&mut self, world_transformation: Matrix4) {
        let local = self.transformation();
        self.update_transforms(local, world_transformation);
    }

    pub(crate) fn update_transforms(
        &mut self,
        transformation: Matrix4,
        world_transformation: Matrix4,
    ) {
        match self {
            Object::Sphere(s) => {
                s.transformation = transformation;
                s.world_transformation = world_transformation;
            }
            Object::Plane(p) => {
                p.transformation = transformation;
                p.world_transformation = world_transformation;
            }
            Object::Cube(c) => {
                c.transformation = transformation;
                c.world_transformation = world_transformation;
            }
            Object::Cylinder(cy) => {
                cy.transformation = transformation;
                cy.world_transformation = world_transformation;
            }
            Object::Cone(co) => {
                co.transformation = transformation;
                co.world_transformation = world_transformation;
            }
            Object::Triangle(t) => {
                t.transformation = transformation;
                t.world_transformation = world_transformation;
            }
            Object::SmoothTriangle(st) => {
                st.transformation = transformation;
                st.world_transformation = world_transformation;
            }
            Object::Group(g) => {
                g.transformation = transformation;
                g.world_transformation = world_transformation;
                propagate_world_transform_to_group_children(g, world_transformation);
            }
        }
    }
}
