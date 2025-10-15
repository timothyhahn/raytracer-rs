use crate::core::floats::{float_equal, EPSILON};
use crate::core::tuples::{Point, Vector};
use crate::rendering::objects::{HasMaterial, Intersectable, Object};
use crate::rendering::rays::Ray;

#[derive(Debug, Copy, Clone)]
pub struct Intersection<'a> {
    pub t: f64,
    pub object: &'a Object,
}

pub struct Computations {
    pub time: f64,
    pub object: Object,
    pub point: Point,
    pub eye_vector: Vector,
    pub normal_vector: Vector,
    pub reflect_vector: Vector,
    pub inside: bool,
    pub over_point: Point,
    pub under_point: Point,
    pub n1: f64,
    pub n2: f64,
}

impl Intersection<'_> {
    pub fn new(t: f64, object: &Object) -> Intersection<'_> {
        Intersection { t, object }
    }

    pub fn sort_intersections(mut intersections: Vec<f64>) -> Vec<f64> {
        intersections.sort_by(|a, b| a.partial_cmp(b).unwrap());
        intersections
    }

    // Assumes intersections are sorted
    pub fn hit(intersections: Vec<Intersection>) -> Option<Intersection> {
        intersections.iter().find(|i| i.t >= 0.0).cloned()
    }

    pub fn prepare_computations(&self, ray: Ray) -> Computations {
        self.prepare_computations_for_intersections(ray, &[*self])
    }

    pub fn prepare_computations_for_intersections(
        &self,
        ray: Ray,
        intersections: &[Intersection],
    ) -> Computations {
        // Basic properties
        let normal_vector = self.object.normal_at(ray.position(self.t));
        let eye_vector = -ray.direction;

        let (inside, normal_vector) = if normal_vector.dot(&eye_vector) < 0.0 {
            (true, -normal_vector)
        } else {
            (false, normal_vector)
        };

        let reflect_vector = ray.direction.reflect(&normal_vector);
        let point = ray.position(self.t);
        let over_point = point + normal_vector * EPSILON;
        let under_point = point - normal_vector * EPSILON;

        // Track which objects we're currently inside
        let mut containers: Vec<&Object> = Vec::new();
        let mut n1 = 1.0;
        let mut n2 = 1.0;

        for intersection in intersections {
            // Check if this intersection is the one we're computing for
            let is_hit = std::ptr::eq(intersection.object, self.object)
                && float_equal(intersection.t, self.t);

            // If this is the hit, n1 is the refractive index of the last container
            if is_hit {
                n1 = containers
                    .last()
                    .map(|obj| obj.material().refractive_index)
                    .unwrap_or(1.0);
            }

            // Update containers: remove if exiting, add if entering
            if let Some(index) = containers
                .iter()
                .position(|obj| std::ptr::eq(*obj, intersection.object))
            {
                // Exiting this object
                containers.remove(index);
            } else {
                // Entering this object
                containers.push(intersection.object);
            }

            // If this is the hit, n2 is the refractive index after updating containers
            if is_hit {
                n2 = containers
                    .last()
                    .map(|obj| obj.material().refractive_index)
                    .unwrap_or(1.0);
                break; // We found our intersection, no need to continue
            }
        }

        Computations {
            time: self.t,
            object: *self.object,
            point,
            eye_vector,
            normal_vector,
            reflect_vector,
            inside,
            over_point,
            under_point,
            n1,
            n2,
        }
    }
}

impl Computations {
    // https://en.wikipedia.org/wiki/Schlick%27s_approximation
    pub fn schlick(&self) -> f64 {
        let mut cos = self.eye_vector.dot(&self.normal_vector);
        if self.n1 > self.n2 {
            let n = self.n1 / self.n2;
            let sin_2t = n.powi(2) * (1.0 - cos.powi(2));
            if sin_2t > 1.0 {
                return 1.0;
            }

            cos = (1.0 - sin_2t).sqrt();
        }
        let r0 = ((self.n1 - self.n2) / (self.n1 + self.n2)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cos).powi(5)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::floats::{float_equal, EPSILON};
    use crate::core::matrices::Matrix4;
    use crate::core::tuples::{Point, Tuple, Vector};
    use crate::geometry::sphere::Sphere;
    use crate::rendering::intersections::Intersection;
    use crate::rendering::objects::{HasMaterial, Object, Transformable};
    use crate::rendering::rays::Ray;

    #[test]
    fn an_intersection_encapsulates_time_and_object() {
        let sphere = Object::Sphere(Sphere::new());
        let intersection = Intersection::new(3.5, &sphere);
        assert_eq!(intersection.t, 3.5);
    }

    #[test]
    fn aggregating_intersections() {
        let sphere = Object::Sphere(Sphere::new());
        let intersection1 = Intersection::new(1.0, &sphere);
        let intersection2 = Intersection::new(2.0, &sphere);
        let intersections =
            Intersection::sort_intersections(vec![intersection1.t, intersection2.t]);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0], 1.0);
        assert_eq!(intersections[1], 2.0);
    }

    #[test]
    fn hit_when_all_intersections_have_positive_time() {
        let sphere = Object::Sphere(Sphere::new());
        let intersections = Intersection::sort_intersections(vec![1.0, 2.0]);
        let hit = Intersection::hit(
            intersections
                .iter()
                .map(|i| Intersection::new(*i, &sphere))
                .collect(),
        );
        assert_eq!(hit.unwrap().t, 1.0);
    }

    #[test]
    fn hit_when_some_intersections_have_negative_time() {
        let sphere = Object::Sphere(Sphere::new());
        let intersections = Intersection::sort_intersections(vec![-1.0, 1.0]);
        let hit = Intersection::hit(
            intersections
                .iter()
                .map(|i| Intersection::new(*i, &sphere))
                .collect(),
        );
        assert_eq!(hit.unwrap().t, 1.0);
    }

    #[test]
    fn hit_when_all_intersections_have_negative_time() {
        let sphere = Object::Sphere(Sphere::new());
        let intersections = Intersection::sort_intersections(vec![-2.0, -1.0]);
        let hit = Intersection::hit(
            intersections
                .iter()
                .map(|i| Intersection::new(*i, &sphere))
                .collect(),
        );
        assert!(hit.is_none());
    }

    #[test]
    fn the_hit_is_always_the_lowest_nonnegative_intersection() {
        let sphere = Object::Sphere(Sphere::new());
        let intersection1 = 5.0;
        let intersection2 = 7.0;
        let intersection3 = -3.;
        let intersection4 = 2.0;
        let intersections = Intersection::sort_intersections(vec![
            intersection1,
            intersection2,
            intersection3,
            intersection4,
        ]);
        let hit = Intersection::hit(
            intersections
                .iter()
                .map(|i| Intersection::new(*i, &sphere))
                .collect(),
        );
        assert_eq!(hit.unwrap().t, 2.0);
    }

    #[test]
    fn precomputing_state_of_intersection() {
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let shape = Object::Sphere(Sphere::new());
        let intersection = Intersection::new(4.0, &shape);
        let computations = intersection.prepare_computations(ray);
        assert_eq!(computations.time, intersection.t);
        assert_eq!(computations.object, shape);
        assert_eq!(computations.point, Point::new(0.0, 0.0, -1.0));
        assert_eq!(computations.eye_vector, Vector::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn precompute_hit_when_intersection_occurs_on_outside() {
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let shape = Object::Sphere(Sphere::new());
        let intersection = Intersection::new(4.0, &shape);
        let computations = intersection.prepare_computations(ray);
        assert!(!computations.inside);
    }

    #[test]
    fn precompute_hit_when_intersection_occurs_on_inside() {
        let ray = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0));
        let shape = Object::Sphere(Sphere::new());
        let intersection = Intersection::new(1.0, &shape);
        let computations = intersection.prepare_computations(ray);
        assert_eq!(computations.point, Point::new(0.0, 0.0, 1.0));
        assert_eq!(computations.eye_vector, Vector::new(0.0, 0.0, -1.0));
        assert!(computations.inside);
        assert_eq!(computations.normal_vector, Vector::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn hit_should_offset_the_point() {
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let sphere = Sphere::new();
        let mut shape = Object::Sphere(sphere);
        shape.set_transform(Matrix4::translate(0.0, 0.0, 1.0));
        let intersection = Intersection::new(5.0, &shape);
        let computations = intersection.prepare_computations(ray);
        assert!(computations.over_point.z < -EPSILON / 2.0);
    }

    #[test]
    fn hit_should_offset_the_point_under() {
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let sphere = Sphere::glass();
        let mut shape = Object::Sphere(sphere);
        shape.set_transform(Matrix4::translate(0.0, 0.0, 1.0));
        let intersection = Intersection::new(5.0, &shape);
        let computations = intersection.prepare_computations(ray);
        assert!(computations.under_point.z > EPSILON / 2.0);
        assert!(computations.point.z < computations.under_point.z);
    }

    #[test]
    fn precomputing_the_reflection_vector() {
        let shape = Object::plane();
        let ray = Ray::new(
            Point::new(0.0, 1.0, -1.0),
            Vector::new(0.0, -2_f64.sqrt() / 2.0, 2_f64.sqrt() / 2.0),
        );
        let intersection = Intersection::new(2_f64.sqrt(), &shape);
        let computations = intersection.prepare_computations(ray);
        assert_eq!(
            computations.reflect_vector,
            Vector::new(0.0, 2_f64.sqrt() / 2.0, 2_f64.sqrt() / 2.0)
        );
    }

    #[test]
    fn finding_n1_and_n2_at_various_intersections() {
        let mut a = Object::Sphere(Sphere::glass());
        a.set_transform(Matrix4::scale(2.0, 2.0, 2.0));
        let mut a_mat = a.material();
        a_mat.refractive_index = 1.5;
        a.set_material(a_mat);

        let mut b = Object::Sphere(Sphere::glass());
        b.set_transform(Matrix4::translate(0.0, 0.0, -0.25));
        let mut b_mat = b.material();
        b_mat.refractive_index = 2.0;
        b.set_material(b_mat);

        let mut c = Object::Sphere(Sphere::glass());
        c.set_transform(Matrix4::translate(0.0, 0.0, 0.25));
        let mut c_mat = c.material();
        c_mat.refractive_index = 2.5;
        c.set_material(c_mat);

        let ray = Ray::new(Point::new(0.0, 0.0, -4.0), Vector::new(0.0, 0.0, 1.0));
        let intersections = vec![
            Intersection::new(2.0, &a),
            Intersection::new(2.75, &b),
            Intersection::new(3.25, &c),
            Intersection::new(4.75, &b),
            Intersection::new(5.25, &c),
            Intersection::new(6.0, &a),
        ];

        let expected_values = vec![
            (1.0, 1.5),
            (1.5, 2.0),
            (2.0, 2.5),
            (2.5, 2.5),
            (2.5, 1.5),
            (1.5, 1.0),
        ];
        intersections
            .iter()
            .zip(expected_values)
            .for_each(|(intersection, (n1, n2))| {
                let computations =
                    intersection.prepare_computations_for_intersections(ray, &intersections);
                assert_eq!(computations.n1, n1);
                assert_eq!(computations.n2, n2);
            });
    }

    #[test]
    pub fn shlick_approximation_under_total_reflection() {
        let shape = Object::Sphere(Sphere::glass());
        let ray = Ray::new(
            Point::new(0.0, 0.0, 2_f64.sqrt() / 2.0),
            Vector::new(0.0, 1.0, 0.0),
        );
        let intersections = [
            Intersection::new(-2_f64.sqrt() / 2.0, &shape),
            Intersection::new(2_f64.sqrt() / 2.0, &shape),
        ];
        let computations =
            intersections[1].prepare_computations_for_intersections(ray, &intersections);
        let reflectance = computations.schlick();
        assert_eq!(reflectance, 1.0);
    }

    #[test]
    pub fn shlick_approximation_with_perpendicular_viewing_angle() {
        let shape = Object::Sphere(Sphere::glass());
        let ray = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 1.0, 0.0));
        let intersections = [
            Intersection::new(-1.0, &shape),
            Intersection::new(1.0, &shape),
        ];
        let computations =
            intersections[1].prepare_computations_for_intersections(ray, &intersections);
        let reflectance = computations.schlick();
        assert!(float_equal(reflectance, 0.04));
    }

    #[test]
    pub fn shlick_approximation_with_small_angle_and_n2_lt_n1() {
        let shape = Object::Sphere(Sphere::glass());
        let ray = Ray::new(Point::new(0.0, 0.99, -2.0), Vector::new(0.0, 0.0, 1.0));
        let intersections = [Intersection::new(1.8589, &shape)];
        let computations =
            intersections[0].prepare_computations_for_intersections(ray, &intersections);
        let reflectance = computations.schlick();
        assert!(float_equal(reflectance, 0.48873));
    }
}
