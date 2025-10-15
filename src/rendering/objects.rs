use crate::core::matrices::Matrix4;
use crate::core::tuples::{Point, Vector};
use crate::geometry::cones::Cone;
use crate::geometry::cubes::Cube;
use crate::geometry::cylinders::Cylinder;
use crate::geometry::planes::Plane;
use crate::geometry::shapes::Shape;
use crate::geometry::sphere::Sphere;
use crate::rendering::intersections::Intersection;
use crate::rendering::rays::Ray;
use crate::scene::materials::Material;

/// Trait for objects that can be intersected by rays and have surface normals.
pub trait Intersectable {
    fn intersect(&self, r: Ray) -> Vec<f64>;
    fn intersect_with_object(&self, r: Ray) -> Vec<Intersection<'_>>;
    fn normal_at(&self, p: Point) -> Vector;
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Object {
    Sphere(Sphere),
    Plane(Plane),
    Cube(Cube),
    Cylinder(Cylinder),
    Cone(Cone),
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
}

impl Intersectable for Object {
    /// Abstract intersect method that handles transformation and delegates
    /// to the shape's local_intersect method.
    fn intersect(&self, ray: Ray) -> Vec<f64> {
        // Transform the ray to object space
        let local_ray = ray.transform(
            self.transformation()
                .inverse()
                .expect("shape transformation should be invertible"),
        );

        // Delegate to the shape's local_intersect
        match self {
            Object::Sphere(s) => s.local_intersect(local_ray),
            Object::Plane(p) => p.local_intersect(local_ray),
            Object::Cube(c) => c.local_intersect(local_ray),
            Object::Cylinder(cy) => cy.local_intersect(local_ray),
            Object::Cone(co) => co.local_intersect(local_ray),
        }
    }

    fn intersect_with_object(&self, r: Ray) -> Vec<Intersection<'_>> {
        self.intersect(r)
            .iter()
            .map(|t| Intersection::new(*t, self))
            .collect()
    }

    /// Abstract normal_at method that handles transformation and delegates
    /// to the shape's local_normal_at method.
    fn normal_at(&self, point: Point) -> Vector {
        let inverse_transform = self
            .transformation()
            .inverse()
            .expect("shape transformation should be invertible");

        // Transform point to object space
        let local_point = inverse_transform * point;

        // Get the normal in object space
        let local_normal = match self {
            Object::Sphere(s) => s.local_normal_at(local_point),
            Object::Plane(p) => p.local_normal_at(local_point),
            Object::Cube(c) => c.local_normal_at(local_point),
            Object::Cylinder(cy) => cy.local_normal_at(local_point),
            Object::Cone(co) => co.local_normal_at(local_point),
        };

        // Transform normal to world space
        let world_normal = inverse_transform.transpose() * local_normal;

        // Normalize the result
        world_normal.normalize()
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
        }
    }

    fn set_material(&mut self, material: Material) {
        match self {
            Object::Sphere(s) => s.material = material,
            Object::Plane(p) => p.material = material,
            Object::Cube(c) => c.material = material,
            Object::Cylinder(cy) => cy.material = material,
            Object::Cone(co) => co.material = material,
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
        }
    }

    fn set_transform(&mut self, transformation: Matrix4) {
        match self {
            Object::Sphere(s) => s.transformation = transformation,
            Object::Plane(p) => p.transformation = transformation,
            Object::Cube(c) => c.transformation = transformation,
            Object::Cylinder(cy) => cy.transformation = transformation,
            Object::Cone(co) => co.transformation = transformation,
        }
    }
}
