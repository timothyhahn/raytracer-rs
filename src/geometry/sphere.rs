use crate::core::matrices::Matrix4;
use crate::core::tuples::{Point, Tuple, Vector};
use crate::geometry::shapes::Shape;
use crate::rendering::rays::Ray;
use crate::scene::materials::Material;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Sphere {
    pub transformation: Matrix4,
    pub material: Material,
}

impl Sphere {
    pub fn new() -> Sphere {
        Sphere {
            transformation: Matrix4::identity(),
            material: Material::default(),
        }
    }
}

impl Shape for Sphere {
    /// Intersect a ray with a unit sphere centered at the origin in object space.
    fn local_intersect(&self, ray: Ray) -> Vec<f64> {
        // The sphere is centered at the origin in object space
        let sphere_to_ray = ray.origin - Point::new(0.0, 0.0, 0.0);

        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * ray.direction.dot(&sphere_to_ray);
        let c = sphere_to_ray.dot(&sphere_to_ray) - 1.0;

        let discriminant = b.powf(2.0) - 4.0 * a * c;

        if discriminant < 0.0 {
            return vec![];
        }

        let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b + discriminant.sqrt()) / (2.0 * a);

        // Return sorted intersections
        if t1 <= t2 {
            vec![t1, t2]
        } else {
            vec![t2, t1]
        }
    }

    /// Compute the normal at a point on a unit sphere centered at the origin in object space.
    fn local_normal_at(&self, point: Point) -> Vector {
        // For a sphere at the origin, the normal is just the point as a vector
        point - Point::new(0.0, 0.0, 0.0)
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::tuples::{Point, Tuple, Vector};
    use crate::geometry::shapes::Shape;
    use crate::geometry::sphere::Sphere;
    use crate::rendering::rays::Ray;

    #[test]
    fn ray_intersects_sphere_at_two_points() {
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let sphere = Sphere::new();
        let intersections = sphere.local_intersect(ray);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0], 4.0);
        assert_eq!(intersections[1], 6.0);
    }

    #[test]
    fn ray_intersects_sphere_at_a_tangent() {
        let ray = Ray::new(Point::new(0.0, 1.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let sphere = Sphere::new();
        let intersections = sphere.local_intersect(ray);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0], 5.0);
        assert_eq!(intersections[1], 5.0);
    }

    #[test]
    fn ray_misses_sphere() {
        let ray = Ray::new(Point::new(0.0, 2.0, -0.5), Vector::new(0.0, 0.0, 1.0));
        let sphere = Sphere::new();
        let intersections = sphere.local_intersect(ray);
        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn ray_originates_inside_sphere() {
        let ray = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0));
        let sphere = Sphere::new();
        let intersections = sphere.local_intersect(ray);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0], -1.0);
        assert_eq!(intersections[1], 1.0);
    }

    #[test]
    fn sphere_behind_ray() {
        let ray = Ray::new(Point::new(0.0, 0.0, 5.0), Vector::new(0.0, 0.0, 1.0));
        let sphere = Sphere::new();
        let intersections = sphere.local_intersect(ray);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0], -6.0);
        assert_eq!(intersections[1], -4.0);
    }

    #[test]
    fn normal_on_sphere_on_x_axis() {
        let sphere = Sphere::new();
        let normal = sphere.local_normal_at(Point::new(1.0, 0.0, 0.0));
        assert_eq!(normal, Vector::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn normal_on_sphere_on_y_axis() {
        let sphere = Sphere::new();
        let normal = sphere.local_normal_at(Point::new(0.0, 1.0, 0.0));
        assert_eq!(normal, Vector::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn normal_on_sphere_on_z_axis() {
        let sphere = Sphere::new();
        let normal = sphere.local_normal_at(Point::new(0.0, 0.0, 1.0));
        assert_eq!(normal, Vector::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn normal_on_sphere_at_nonaxial_point() {
        let sphere = Sphere::new();
        let normal = sphere.local_normal_at(Point::new(
            3.0_f64.sqrt() / 3.0,
            3.0_f64.sqrt() / 3.0,
            3.0_f64.sqrt() / 3.0,
        ));
        assert_eq!(
            normal,
            Vector::new(
                3.0_f64.sqrt() / 3.0,
                3.0_f64.sqrt() / 3.0,
                3.0_f64.sqrt() / 3.0
            )
        );
    }

    #[test]
    fn normal_is_normalized() {
        let sphere = Sphere::new();
        let normal = sphere.local_normal_at(Point::new(
            3.0_f64.sqrt() / 3.0,
            3.0_f64.sqrt() / 3.0,
            3.0_f64.sqrt() / 3.0,
        ));
        assert_eq!(normal, normal.normalize());
    }
}
