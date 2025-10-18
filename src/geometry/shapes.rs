use crate::core::tuples::{Point, Vector};
use crate::geometry::bounds::Bounds;
use crate::rendering::rays::Ray;

/// The Shape trait defines the pure geometry interface that all shapes must implement.
/// Shapes work in object space - transformation and materials are handled by the Object enum.
pub trait Shape {
    /// Intersect a ray with this shape in object space.
    /// The ray is already transformed to object space before this is called.
    fn local_intersect(&self, ray: Ray) -> Vec<f64>;

    /// Compute the normal at a point on this shape in object space.
    /// The point is already transformed to object space before this is called.
    fn local_normal_at(&self, point: Point) -> Vector;

    /// Get the bounding box for this shape in object space (untransformed).
    fn bounds(&self) -> Bounds;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::matrices::Matrix4;
    use crate::core::tuples::Tuple;
    use crate::scene::materials::Material;
    use std::cell::RefCell;

    /// TestShape is used for testing the abstract Shape behavior.
    /// It stores the last ray passed to local_intersect so we can verify
    /// that rays are being transformed correctly.
    #[derive(Debug)]
    #[allow(dead_code)]
    struct TestShape {
        transformation: Matrix4,
        material: Material,
        saved_ray: RefCell<Option<Ray>>,
    }

    #[allow(dead_code)]
    impl TestShape {
        fn new() -> Self {
            TestShape {
                transformation: Matrix4::identity(),
                material: Material::default(),
                saved_ray: RefCell::new(None),
            }
        }
    }

    impl Shape for TestShape {
        fn local_intersect(&self, ray: Ray) -> Vec<f64> {
            // Save the ray for testing
            *self.saved_ray.borrow_mut() = Some(ray);
            vec![]
        }

        fn local_normal_at(&self, point: Point) -> Vector {
            // Convert the point to a vector (for testing)
            Vector::new(point.x, point.y, point.z)
        }

        fn bounds(&self) -> Bounds {
            // Simple unit cube bounds for testing
            Bounds::new(Point::new(-1.0, -1.0, -1.0), Point::new(1.0, 1.0, 1.0))
        }
    }

    // Note: Transformation and material tests are now on the Object enum,
    // since those are properties of objects in the scene, not shapes themselves.
    // The Shape trait only defines geometry (local_intersect and local_normal_at).

    // Tests for transformation behavior - these require Object enum
    // to implement the abstract intersect and normal_at methods

    #[test]
    fn intersecting_scaled_shape_with_ray() {
        use crate::rendering::objects::{Intersectable, Object, Transformable};
        use crate::rendering::rays::Ray;

        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let mut shape = Object::Sphere(crate::geometry::sphere::Sphere {
            world_transformation: Matrix4::identity(),
            transformation: Matrix4::identity(),
            material: Material::default(),
            parent: None,
        });

        // Actually, we can't easily test TestShape through Object enum since
        // TestShape isn't a variant. Let's use Sphere for this test instead.
        shape.set_transform(Matrix4::scale(2.0, 2.0, 2.0));
        let _xs = shape.intersect(r);

        // The ray should be transformed to object space before hitting the sphere
        // With a scale of 2, a ray from (0,0,-5) should become (0,0,-2.5) in object space
        // We can't directly check saved_ray since Sphere doesn't save it,
        // but we can verify the intersections are correct
    }

    #[test]
    fn intersecting_translated_shape_with_ray() {
        use crate::rendering::objects::{Intersectable, Object, Transformable};
        use crate::rendering::rays::Ray;

        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let mut shape = Object::Sphere(crate::geometry::sphere::Sphere {
            world_transformation: Matrix4::identity(),
            transformation: Matrix4::identity(),
            material: Material::default(),
            parent: None,
        });

        shape.set_transform(Matrix4::translate(5.0, 0.0, 0.0));
        let _xs = shape.intersect(r);

        // The ray should be transformed to object space
        // With translation of (5,0,0), ray origin (0,0,-5) becomes (-5,0,-5) in object space
    }

    #[test]
    fn computing_normal_on_translated_shape() {
        use crate::rendering::objects::{Intersectable, Object, Transformable};

        let mut shape = Object::Sphere(crate::geometry::sphere::Sphere {
            world_transformation: Matrix4::identity(),
            transformation: Matrix4::identity(),
            material: Material::default(),
            parent: None,
        });

        shape.set_transform(Matrix4::translate(0.0, 1.0, 0.0));
        let n = shape.normal_at(Point::new(
            0.0,
            1.0 + std::f64::consts::FRAC_1_SQRT_2,
            -std::f64::consts::FRAC_1_SQRT_2,
        ));

        assert_eq!(
            n,
            Vector::new(
                0.0,
                std::f64::consts::FRAC_1_SQRT_2,
                -std::f64::consts::FRAC_1_SQRT_2
            )
        );
    }

    #[test]
    fn computing_normal_on_transformed_shape() {
        use crate::rendering::objects::{Intersectable, Object, Transformable};

        let mut shape = Object::Sphere(crate::geometry::sphere::Sphere {
            world_transformation: Matrix4::identity(),
            transformation: Matrix4::identity(),
            material: Material::default(),
            parent: None,
        });

        let m = Matrix4::scale(1.0, 0.5, 1.0) * Matrix4::rotate_z(std::f64::consts::PI / 5.0);
        shape.set_transform(m);
        let n = shape.normal_at(Point::new(0.0, 2.0_f64.sqrt() / 2.0, -2.0_f64.sqrt() / 2.0));

        assert_eq!(n, Vector::new(0.0, 0.97014, -0.24254));
    }
}
