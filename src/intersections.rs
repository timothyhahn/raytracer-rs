use crate::rays::Ray;
use crate::sphere::Sphere;
use crate::tuples::{Point, Vector};

#[derive(Debug, Copy, Clone)]
pub struct Intersection<'a> {
    pub time: f64,
    pub object: &'a Sphere,
}

pub struct Computations {
    pub time: f64,
    pub object: Sphere,
    pub point: Point,
    pub eye_vector: Vector,
    pub normal_vector: Vector,
    pub inside: bool,
}

impl Intersection<'_> {
    pub fn new(time: f64, object: &Sphere) -> Intersection {
        Intersection { time, object }
    }

    pub fn intersections(mut intersections: Vec<Intersection>) -> Vec<Intersection> {
        intersections.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        intersections
    }

    // Assumes intersections are sorted
    pub fn hit(intersections: Vec<Intersection>) -> Option<Intersection> {
        intersections.iter().find(|i| i.time >= 0.0).cloned()
    }

    pub fn prepare_computations(self, ray: Ray) -> Computations {
        let normal_vector = self.object.normal_at(ray.position(self.time));
        let eye_vector = -ray.direction;

        let (inside, normal_vector) = if normal_vector.dot(&eye_vector) < 0.0 {
            (true, -normal_vector)
        } else {
            (false, normal_vector)
        };
        Computations {
            time: self.time,
            object: self.object.clone(),
            point: ray.position(self.time),
            eye_vector,
            normal_vector,
            inside,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::intersections::Intersection;
    use crate::rays::Ray;
    use crate::sphere::Sphere;
    use crate::tuples::{Point, Tuple, Vector};

    #[test]
    fn an_intersection_encapsulates_time_and_object() {
        let sphere = Sphere::new();
        let intersection = Intersection::new(3.5, &sphere);
        assert_eq!(intersection.time, 3.5);
        assert_eq!(*intersection.object, sphere);
    }

    #[test]
    fn aggregating_intersections() {
        let sphere = Sphere::new();
        let intersection1 = Intersection::new(1.0, &sphere);
        let intersection2 = Intersection::new(2.0, &sphere);
        let intersections = Intersection::intersections(vec![intersection1, intersection2]);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].time, 1.0);
        assert_eq!(intersections[1].time, 2.0);
    }

    #[test]
    fn hit_when_all_intersections_have_positive_time() {
        let sphere = Sphere::new();
        let intersection1 = Intersection::new(1.0, &sphere);
        let intersection2 = Intersection::new(2.0, &sphere);
        let intersections = Intersection::intersections(vec![intersection1, intersection2]);
        let hit = Intersection::hit(intersections);
        assert_eq!(hit.unwrap().time, 1.0);
    }

    #[test]
    fn hit_when_some_intersections_have_negative_time() {
        let sphere = Sphere::new();
        let intersection1 = Intersection::new(-1.0, &sphere);
        let intersection2 = Intersection::new(1.0, &sphere);
        let intersections = Intersection::intersections(vec![intersection1, intersection2]);
        let hit = Intersection::hit(intersections);
        assert_eq!(hit.unwrap().time, 1.0);
    }

    #[test]
    fn hit_when_all_intersections_have_negative_time() {
        let sphere = Sphere::new();
        let intersection1 = Intersection::new(-2.0, &sphere);
        let intersection2 = Intersection::new(-1.0, &sphere);
        let intersections = Intersection::intersections(vec![intersection1, intersection2]);
        let hit = Intersection::hit(intersections);
        assert!(hit.is_none());
    }

    #[test]
    fn the_hit_is_always_the_lowest_nonnegative_intersection() {
        let sphere = Sphere::new();
        let intersection1 = Intersection::new(5.0, &sphere);
        let intersection2 = Intersection::new(7.0, &sphere);
        let intersection3 = Intersection::new(-3.0, &sphere);
        let intersection4 = Intersection::new(2.0, &sphere);
        let intersections = Intersection::intersections(vec![
            intersection1,
            intersection2,
            intersection3,
            intersection4,
        ]);
        let hit = Intersection::hit(intersections);
        assert_eq!(hit.unwrap().time, 2.0);
    }

    #[test]
    fn precomputing_state_of_intersection() {
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let shape = Sphere::new();
        let intersection = Intersection::new(4.0, &shape);
        let computations = intersection.prepare_computations(ray);
        assert_eq!(computations.time, intersection.time);
        assert_eq!(computations.object, intersection.object.clone());
        assert_eq!(computations.point, Point::new(0.0, 0.0, -1.0));
        assert_eq!(computations.eye_vector, Vector::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn precompute_hit_when_intersection_occurs_on_outside() {
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let shape = Sphere::new();
        let intersection = Intersection::new(4.0, &shape);
        let computations = intersection.prepare_computations(ray);
        assert!(!computations.inside);
    }

    #[test]
    fn precompute_hit_when_intersection_occurs_on_inside() {
        let ray = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0));
        let shape = Sphere::new();
        let intersection = Intersection::new(1.0, &shape);
        let computations = intersection.prepare_computations(ray);
        assert_eq!(computations.point, Point::new(0.0, 0.0, 1.0));
        assert_eq!(computations.eye_vector, Vector::new(0.0, 0.0, -1.0));
        assert!(computations.inside);
        assert_eq!(computations.normal_vector, Vector::new(0.0, 0.0, -1.0));
    }
}
