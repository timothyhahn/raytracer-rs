use crate::intersections::Intersection;
use crate::materials::Material;
use crate::matrices::Matrix4;
use crate::rays::Ray;
use crate::tuples::{Point, Tuple, Vector};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Sphere {
    pub transformation: Matrix4,
    pub material: Material,
    pub center: Point,
}

impl Sphere {
    pub fn new() -> Sphere {
        Sphere {
            transformation: Matrix4::identity(),
            material: Material::default(),
            center: Point::new(0.0, 0.0, 0.0),
        }
    }

    // Returns list of time values where the ray intersects the sphere
    pub fn intersect(&self, ray: Ray) -> Vec<f64> {
        let ray = ray.transform(self.transformation.inverse().unwrap());
        let sphere_to_ray = ray.origin - self.center;
        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * ray.direction.dot(&sphere_to_ray);
        let c = sphere_to_ray.dot(&sphere_to_ray) - 1.0;
        let discriminant = b.powf(2.0) - 4.0 * a * c;
        if discriminant < 0.0 {
            return vec![];
        }

        let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
        Intersection::sort_intersections(vec![t1, t2])
    }

    pub fn normal_at(&self, point: Point) -> Vector {
        let object_point = self.transformation.inverse().unwrap() * point;
        let object_normal = object_point - Point::new(0.0, 0.0, 0.0);

        let world_normal = self.transformation.inverse().unwrap().transpose() * object_normal;

        world_normal.normalize()
    }

    pub fn set_transform(&mut self, transformation: Matrix4) {
        self.transformation = transformation;
    }

    pub fn set_material(&mut self, material: Material) {
        self.material = material;
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::color::Color;
    use crate::materials::Material;
    use crate::matrices::Matrix4;
    use crate::objects::{Intersectable, Object};
    use crate::rays::Ray;
    use crate::sphere::Sphere;
    use crate::tuples::{Point, Tuple, Vector};

    #[test]
    fn spheres_should_start_at_the_center() {
        let sphere = Sphere::new();
        assert_eq!(sphere.center, Point::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn spheres_default_transformation() {
        let sphere = Sphere::new();
        assert_eq!(sphere.transformation, Matrix4::identity());
    }

    #[test]
    fn changing_sphere_transformation() {
        let mut sphere = Sphere::new();
        let t = Matrix4::translate(2.0, 3.0, 4.0);
        sphere.set_transform(t.clone());
        assert_eq!(sphere.transformation, t);
    }

    #[test]
    fn ray_intersects_sphere_at_two_points() {
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let sphere = Sphere::new();
        let intersections = sphere.intersect(ray);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0], 4.0);
        assert_eq!(intersections[1], 6.0);
    }

    #[test]
    fn ray_intersects_sphere_at_a_tangent() {
        let ray = Ray::new(Point::new(0.0, 1.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let sphere = Sphere::new();
        let intersections = sphere.intersect(ray);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0], 5.0);
        assert_eq!(intersections[1], 5.0);
    }

    #[test]
    fn ray_misses_sphere() {
        let ray = Ray::new(Point::new(0.0, 2.0, -0.5), Vector::new(0.0, 0.0, 1.0));
        let sphere = Sphere::new();
        let intersections = sphere.intersect(ray);
        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn ray_originates_inside_sphere() {
        let ray = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0));
        let sphere = Sphere::new();
        let intersections = sphere.intersect(ray);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0], -1.0);
        assert_eq!(intersections[1], 1.0);
    }

    #[test]
    fn sphere_behind_ray() {
        let ray = Ray::new(Point::new(0.0, 0.0, 5.0), Vector::new(0.0, 0.0, 1.0));
        let sphere = Sphere::new();
        let intersections = sphere.intersect(ray);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0], -6.0);
        assert_eq!(intersections[1], -4.0);
    }

    #[test]
    fn intersect_sets_the_object_on_the_intersection() {
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let sphere = Object::Sphere(Sphere::new());
        let intersections = sphere.intersect(ray);
        assert_eq!(intersections.len(), 2);
    }

    #[test]
    fn intersecting_scaled_sphere_with_ray() {
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let mut sphere = Object::Sphere(Sphere::new());
        sphere.set_transform(Matrix4::scale(2.0, 2.0, 2.0));
        let intersections = sphere.intersect(ray);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0], 3.0);
        assert_eq!(intersections[1], 7.0);
    }

    #[test]
    fn intersecting_translated_sphere_with_ray() {
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let mut sphere = Sphere::new();
        sphere.set_transform(Matrix4::translate(5.0, 0.0, 0.0));
        let intersections = sphere.intersect(ray);
        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn normal_on_sphere_on_x_axis() {
        let sphere = Sphere::new();
        let normal = sphere.normal_at(Point::new(1.0, 0.0, 0.0));
        assert_eq!(normal, Vector::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn normal_on_sphere_on_y_axis() {
        let sphere = Sphere::new();
        let normal = sphere.normal_at(Point::new(0.0, 1.0, 0.0));
        assert_eq!(normal, Vector::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn normal_on_sphere_on_z_axis() {
        let sphere = Sphere::new();
        let normal = sphere.normal_at(Point::new(0.0, 0.0, 1.0));
        assert_eq!(normal, Vector::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn normal_on_sphere_at_nonaxial_point() {
        let sphere = Sphere::new();
        let normal = sphere.normal_at(Point::new(
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
        let normal = sphere.normal_at(Point::new(
            3.0_f64.sqrt() / 3.0,
            3.0_f64.sqrt() / 3.0,
            3.0_f64.sqrt() / 3.0,
        ));
        assert_eq!(normal, normal.normalize());
    }

    #[test]
    fn compute_normal_on_translated_sphere() {
        let mut sphere = Sphere::new();
        sphere.set_transform(Matrix4::translate(0.0, 1.0, 0.0));
        let normal = sphere.normal_at(Point::new(0.0, 1.70711, -0.70711));
        assert_eq!(normal, Vector::new(0.0, 0.70711, -0.70711));
    }

    #[test]
    fn compute_normal_on_transformed_sphere() {
        let mut sphere = Sphere::new();
        sphere.set_transform(
            Matrix4::scale(1.0, 0.5, 1.0) * Matrix4::rotate_z(std::f64::consts::PI / 5.0),
        );
        let normal = sphere.normal_at(Point::new(0.0, 2.0_f64.sqrt() / 2.0, -2.0_f64.sqrt() / 2.0));
        assert_eq!(normal, Vector::new(0.0, 0.97014, -0.24254));
    }

    #[test]
    fn sphere_has_default_material() {
        let sphere = Sphere::new();
        let material = sphere.material;
        assert_eq!(material, Material::default());
    }

    #[test]
    fn sphere_can_be_assigned_material() {
        let mut sphere = Sphere::new();
        let material = Material::new(Color::new(0.5, 0.5, 1.0), 0.2, 0.8, 0.8, 90.0);
        sphere.set_material(material.clone());
        sphere.material = material;
        assert_eq!(sphere.material, material);
    }
}
