use crate::objects::{Intersectable, Object};
use crate::rays::Ray;
use crate::tuples::{Point, Vector};

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
    pub inside: bool,
}

impl Intersection<'_> {
    pub fn new(t: f64, object: &Object) -> Intersection {
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
        let normal_vector = self.object.normal_at(ray.position(self.t));
        let eye_vector = -ray.direction;

        let (inside, normal_vector) = if normal_vector.dot(&eye_vector) < 0.0 {
            (true, -normal_vector)
        } else {
            (false, normal_vector)
        };
        Computations {
            time: self.t,
            object: *self.object,
            point: ray.position(self.t),
            eye_vector,
            normal_vector,
            inside,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::intersections::Intersection;
    use crate::objects::Object;
    use crate::rays::Ray;
    use crate::sphere::Sphere;
    use crate::tuples::{Point, Tuple, Vector};

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
}
