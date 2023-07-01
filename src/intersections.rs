use crate::objects::Object;

#[derive(Debug, Copy, Clone)]
pub struct Intersection<'a, T: Object + Clone> {
    pub time: f64,
    pub object: &'a T,
}

impl<T: Object + Clone> Intersection<'_, T> {
    pub fn new(time: f64, object: &T) -> Intersection<T> {
        Intersection { time, object }
    }

    pub fn intersections(mut intersections: Vec<Intersection<T>>) -> Vec<Intersection<T>> {
        intersections.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        intersections
    }

    // Assumes intersections are sorted
    pub fn hit(intersections: Vec<Intersection<T>>) -> Option<Intersection<T>> {
        intersections
            .iter()
            .filter(|i| i.time >= 0.0)
            .next()
            .map(|i| i.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::intersections::Intersection;
    use crate::sphere::Sphere;

    #[test]
    fn test_an_intersection_encapsulates_time_and_object() {
        let sphere = Sphere::new();
        let intersection = Intersection::new(3.5, &sphere);
        assert_eq!(intersection.time, 3.5);
        assert_eq!(*intersection.object, sphere);
    }

    #[test]
    fn test_aggregating_intersections() {
        let sphere = Sphere::new();
        let intersection1 = Intersection::new(1.0, &sphere);
        let intersection2 = Intersection::new(2.0, &sphere);
        let intersections = Intersection::intersections(vec![intersection1, intersection2]);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].time, 1.0);
        assert_eq!(intersections[1].time, 2.0);
    }

    #[test]
    fn test_hit_when_all_intersections_have_positive_time() {
        let sphere = Sphere::new();
        let intersection1 = Intersection::new(1.0, &sphere);
        let intersection2 = Intersection::new(2.0, &sphere);
        let intersections = Intersection::intersections(vec![intersection1, intersection2]);
        let hit = Intersection::hit(intersections);
        assert_eq!(hit.unwrap().time, 1.0);
    }

    #[test]
    fn test_hit_when_some_intersections_have_negative_time() {
        let sphere = Sphere::new();
        let intersection1 = Intersection::new(-1.0, &sphere);
        let intersection2 = Intersection::new(1.0, &sphere);
        let intersections = Intersection::intersections(vec![intersection1, intersection2]);
        let hit = Intersection::hit(intersections);
        assert_eq!(hit.unwrap().time, 1.0);
    }

    #[test]
    fn test_hit_when_all_intersections_have_negative_time() {
        let sphere = Sphere::new();
        let intersection1 = Intersection::new(-2.0, &sphere);
        let intersection2 = Intersection::new(-1.0, &sphere);
        let intersections = Intersection::intersections(vec![intersection1, intersection2]);
        let hit = Intersection::hit(intersections);
        assert!(hit.is_none());
    }

    #[test]
    fn test_the_hit_is_always_the_lowest_nonnegative_intersection() {
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
}
