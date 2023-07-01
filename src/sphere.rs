use crate::intersections::Intersection;
use crate::matrices::Matrix;
use crate::objects::Object;
use crate::rays::Ray;
use crate::tuples::Tuple;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct Sphere {
    pub id: Uuid,
    pub transformation: Matrix,
    center: Tuple,
}

impl Object for Sphere {}

impl Sphere {
    pub fn new() -> Sphere {
        Sphere {
            id: Uuid::new_v4(),
            transformation: Matrix::identity(4),
            center: Tuple::point(0.0, 0.0, 0.0),
        }
    }

    // Returns list of time values where the ray intersects the sphere
    pub fn intersect(&self, ray: Ray) -> Vec<Intersection<Sphere>> {
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
        Intersection::intersections(vec![
            Intersection::new(t1, self),
            Intersection::new(t2, self),
        ])
    }

    pub fn set_transform(&mut self, transformation: Matrix) {
        self.transformation = transformation;
    }
}

#[cfg(test)]
mod tests {
    use crate::matrices::Matrix;
    use crate::rays::Ray;
    use crate::sphere::Sphere;
    use crate::tuples::Tuple;

    #[test]
    fn test_spheres_should_start_at_the_center() {
        let sphere = Sphere::new();
        assert_eq!(sphere.center, Tuple::point(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_spheres_should_have_unique_identifiers() {
        let sphere1 = Sphere::new();
        let sphere2 = Sphere::new();
        assert_ne!(sphere1.id, sphere2.id);
    }

    #[test]
    fn test_spheres_default_transformation() {
        let sphere = Sphere::new();
        assert_eq!(sphere.transformation, Matrix::identity(4));
    }

    #[test]
    fn test_changing_sphere_transformation() {
        let mut sphere = Sphere::new();
        let t = Matrix::translation(2.0, 3.0, 4.0);
        sphere.set_transform(t.clone());
        assert_eq!(sphere.transformation, t);
    }

    #[test]
    fn test_ray_intersects_sphere_at_two_points() {
        let ray = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let sphere = Sphere::new();
        let intersections = sphere.intersect(ray);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].time, 4.0);
        assert_eq!(intersections[1].time, 6.0);
    }

    #[test]
    fn test_ray_intersects_sphere_at_a_tangent() {
        let ray = Ray::new(Tuple::point(0.0, 1.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let sphere = Sphere::new();
        let intersections = sphere.intersect(ray);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].time, 5.0);
        assert_eq!(intersections[1].time, 5.0);
    }

    #[test]
    fn test_ray_misses_sphere() {
        let ray = Ray::new(Tuple::point(0.0, 2.0, -0.5), Tuple::vector(0.0, 0.0, 1.0));
        let sphere = Sphere::new();
        let intersections = sphere.intersect(ray);
        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn test_ray_originates_inside_sphere() {
        let ray = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let sphere = Sphere::new();
        let intersections = sphere.intersect(ray);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].time, -1.0);
        assert_eq!(intersections[1].time, 1.0);
    }

    #[test]
    fn test_sphere_behind_ray() {
        let ray = Ray::new(Tuple::point(0.0, 0.0, 5.0), Tuple::vector(0.0, 0.0, 1.0));
        let sphere = Sphere::new();
        let intersections = sphere.intersect(ray);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].time, -6.0);
        assert_eq!(intersections[1].time, -4.0);
    }

    #[test]
    fn test_intersect_sets_the_object_on_the_intersection() {
        let ray = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let sphere = Sphere::new();
        let intersections = sphere.intersect(ray);
        assert_eq!(intersections.len(), 2);
        assert_eq!(*intersections[0].object, sphere);
        assert_eq!(*intersections[1].object, sphere);
    }

    #[test]
    fn test_intersecting_scaled_sphere_with_ray() {
        let ray = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut sphere = Sphere::new();
        sphere.set_transform(Matrix::scaling(2.0, 2.0, 2.0));
        let intersections = sphere.intersect(ray);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].time, 3.0);
        assert_eq!(intersections[1].time, 7.0);
    }

    #[test]
    fn test_intersecting_translated_sphere_with_ray() {
        let ray = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut sphere = Sphere::new();
        sphere.set_transform(Matrix::translation(5.0, 0.0, 0.0));
        let intersections = sphere.intersect(ray);
        assert_eq!(intersections.len(), 0);
    }
}
