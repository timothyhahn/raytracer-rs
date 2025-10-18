use crate::core::matrices::Matrix4;
use crate::core::tuples::{Point, Tuple};
use crate::rendering::rays::Ray;

/// Axis-Aligned Bounding Box (AABB) that describes the minimum and maximum extents.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub min: Point,
    pub max: Point,
}

impl Bounds {
    /// Create a new bounding box with the given minimum and maximum extents.
    pub fn new(min: Point, max: Point) -> Self {
        Bounds { min, max }
    }

    /// Create an empty bounding box (with inverted extents).
    /// This is useful as a starting point for combining bounds.
    pub fn empty() -> Self {
        Bounds {
            min: Point::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            max: Point::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        }
    }

    /// Create an infinite bounding box.
    pub fn infinite() -> Self {
        Bounds {
            min: Point::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
            max: Point::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
        }
    }

    /// Add a point to the bounding box, expanding it if necessary.
    pub fn add_point(&mut self, point: Point) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.min.z = self.min.z.min(point.z);

        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
        self.max.z = self.max.z.max(point.z);
    }

    /// Combine two bounding boxes into one that contains both.
    pub fn merge(&self, other: &Bounds) -> Bounds {
        let mut result = *self;
        result.add_point(other.min);
        result.add_point(other.max);
        result
    }

    /// Transform a bounding box by a transformation matrix.
    /// This transforms all 8 corners and creates a new axis-aligned box that contains them all.
    /// Special handling for infinite bounds to avoid NaN from operations like 0 * ∞.
    pub fn transform(&self, matrix: Matrix4) -> Bounds {
        // Check if bounds are infinite - if so, keep them infinite
        // Transforming infinite bounds with rotation/scale can produce NaN
        let has_infinite_min =
            self.min.x.is_infinite() || self.min.y.is_infinite() || self.min.z.is_infinite();
        let has_infinite_max =
            self.max.x.is_infinite() || self.max.y.is_infinite() || self.max.z.is_infinite();

        if has_infinite_min || has_infinite_max {
            // For infinite bounds, return infinite bounds
            // This prevents NaN from operations like rotating a plane
            return Bounds::infinite();
        }

        // Get all 8 corners of the bounding box
        let corners = [
            self.min,
            Point::new(self.min.x, self.min.y, self.max.z),
            Point::new(self.min.x, self.max.y, self.min.z),
            Point::new(self.min.x, self.max.y, self.max.z),
            Point::new(self.max.x, self.min.y, self.min.z),
            Point::new(self.max.x, self.min.y, self.max.z),
            Point::new(self.max.x, self.max.y, self.min.z),
            self.max,
        ];

        // Transform all corners and build a new bounding box
        let mut result = Bounds::empty();
        for corner in &corners {
            result.add_point(matrix * *corner);
        }
        result
    }

    /// Test if a ray intersects this bounding box.
    /// Returns true if the ray intersects the box, false otherwise.
    /// This uses the same algorithm as the cube intersection, but with arbitrary bounds.
    pub fn intersects(&self, ray: Ray) -> bool {
        let (xtmin, xtmax) =
            Self::check_axis(ray.origin.x, ray.direction.x, self.min.x, self.max.x);
        let (ytmin, ytmax) =
            Self::check_axis(ray.origin.y, ray.direction.y, self.min.y, self.max.y);
        let (ztmin, ztmax) =
            Self::check_axis(ray.origin.z, ray.direction.z, self.min.z, self.max.z);

        let tmin = xtmin.max(ytmin).max(ztmin);
        let tmax = xtmax.min(ytmax).min(ztmax);

        tmin <= tmax
    }

    /// Helper function to check a single axis for intersection.
    /// This is the cube's check_axis algorithm adapted for arbitrary bounds.
    fn check_axis(origin: f64, direction: f64, min: f64, max: f64) -> (f64, f64) {
        let tmin_numerator = min - origin;
        let tmax_numerator = max - origin;

        let (tmin, tmax) = if direction.abs() >= 1e-10 {
            (tmin_numerator / direction, tmax_numerator / direction)
        } else {
            (
                tmin_numerator * f64::INFINITY,
                tmax_numerator * f64::INFINITY,
            )
        };

        if tmin > tmax {
            (tmax, tmin)
        } else {
            (tmin, tmax)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tuples::{Tuple, Vector};

    #[test]
    fn creating_empty_bounding_box() {
        let bounds = Bounds::empty();
        assert_eq!(bounds.min.x, f64::INFINITY);
        assert_eq!(bounds.max.x, f64::NEG_INFINITY);
    }

    #[test]
    fn creating_bounding_box_with_volume() {
        let bounds = Bounds::new(Point::new(-1.0, -2.0, -3.0), Point::new(3.0, 2.0, 1.0));
        assert_eq!(bounds.min, Point::new(-1.0, -2.0, -3.0));
        assert_eq!(bounds.max, Point::new(3.0, 2.0, 1.0));
    }

    #[test]
    fn adding_points_to_empty_bounding_box() {
        let mut bounds = Bounds::empty();
        bounds.add_point(Point::new(-5.0, 2.0, 0.0));
        bounds.add_point(Point::new(7.0, 0.0, -3.0));

        assert_eq!(bounds.min, Point::new(-5.0, 0.0, -3.0));
        assert_eq!(bounds.max, Point::new(7.0, 2.0, 0.0));
    }

    #[test]
    fn merging_two_bounding_boxes() {
        let box1 = Bounds::new(Point::new(-5.0, -2.0, 0.0), Point::new(7.0, 4.0, 4.0));
        let box2 = Bounds::new(Point::new(8.0, -7.0, -2.0), Point::new(14.0, 2.0, 8.0));

        let merged = box1.merge(&box2);
        assert_eq!(merged.min, Point::new(-5.0, -7.0, -2.0));
        assert_eq!(merged.max, Point::new(14.0, 4.0, 8.0));
    }

    #[test]
    fn intersecting_ray_with_bounding_box_at_origin() {
        let bounds = Bounds::new(Point::new(-1.0, -1.0, -1.0), Point::new(1.0, 1.0, 1.0));

        // Ray intersects from positive x
        let ray1 = Ray::new(Point::new(5.0, 0.5, 0.0), Vector::new(-1.0, 0.0, 0.0));
        assert!(bounds.intersects(ray1));

        // Ray intersects from negative x
        let ray2 = Ray::new(Point::new(-5.0, 0.5, 0.0), Vector::new(1.0, 0.0, 0.0));
        assert!(bounds.intersects(ray2));

        // Ray misses
        let ray3 = Ray::new(Point::new(0.0, 5.0, 0.0), Vector::new(1.0, 0.0, 0.0));
        assert!(!bounds.intersects(ray3));
    }

    #[test]
    fn intersecting_ray_with_non_cubic_bounding_box() {
        let bounds = Bounds::new(Point::new(5.0, -2.0, 0.0), Point::new(11.0, 4.0, 7.0));

        // Ray intersects from the side
        let ray1 = Ray::new(Point::new(15.0, 1.0, 2.0), Vector::new(-1.0, 0.0, 0.0));
        assert!(bounds.intersects(ray1));

        // Ray also intersects (y=-1 is within [-2,4], z=2 is within [0,7])
        let ray2 = Ray::new(Point::new(15.0, -1.0, 2.0), Vector::new(-1.0, 0.0, 0.0));
        assert!(bounds.intersects(ray2));

        // Ray misses (y=-3 is below the box minimum of -2)
        let ray3 = Ray::new(Point::new(15.0, -3.0, 2.0), Vector::new(-1.0, 0.0, 0.0));
        assert!(!bounds.intersects(ray3));
    }

    #[test]
    fn transforming_bounding_box() {
        let bounds = Bounds::new(Point::new(-1.0, -1.0, -1.0), Point::new(1.0, 1.0, 1.0));

        let matrix = Matrix4::rotate_x(std::f64::consts::PI / 4.0)
            * Matrix4::rotate_y(std::f64::consts::PI / 4.0);

        let transformed = bounds.transform(matrix);

        // The transformed box should contain all 8 transformed corners
        // After rotation, the box should be larger in some dimensions
        assert!(transformed.min.x <= -1.0);
        assert!(transformed.max.x >= 1.0);
    }

    #[test]
    fn transforming_bounding_box_with_translation() {
        let bounds = Bounds::new(Point::new(-1.0, -1.0, -1.0), Point::new(1.0, 1.0, 1.0));

        let matrix = Matrix4::translate(5.0, 3.0, 2.0);
        let transformed = bounds.transform(matrix);

        assert_eq!(transformed.min, Point::new(4.0, 2.0, 1.0));
        assert_eq!(transformed.max, Point::new(6.0, 4.0, 3.0));
    }

    #[test]
    fn transforming_bounding_box_with_scale() {
        let bounds = Bounds::new(Point::new(-1.0, -1.0, -1.0), Point::new(1.0, 1.0, 1.0));

        let matrix = Matrix4::scale(2.0, 3.0, 4.0);
        let transformed = bounds.transform(matrix);

        assert_eq!(transformed.min, Point::new(-2.0, -3.0, -4.0));
        assert_eq!(transformed.max, Point::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn sphere_bounds() {
        use crate::geometry::shapes::Shape;
        use crate::geometry::sphere::Sphere;

        let sphere = Sphere::new();
        let bounds = sphere.bounds();

        assert_eq!(bounds.min, Point::new(-1.0, -1.0, -1.0));
        assert_eq!(bounds.max, Point::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn cube_bounds() {
        use crate::geometry::cubes::Cube;
        use crate::geometry::shapes::Shape;

        let cube = Cube::new();
        let bounds = cube.bounds();

        assert_eq!(bounds.min, Point::new(-1.0, -1.0, -1.0));
        assert_eq!(bounds.max, Point::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn unbounded_cylinder_bounds() {
        use crate::geometry::cylinders::Cylinder;
        use crate::geometry::shapes::Shape;

        let cylinder = Cylinder::new();
        let bounds = cylinder.bounds();

        assert_eq!(bounds.min.x, -1.0);
        assert_eq!(bounds.max.x, 1.0);
        assert_eq!(bounds.min.z, -1.0);
        assert_eq!(bounds.max.z, 1.0);
        assert!(bounds.min.y.is_infinite() && bounds.min.y.is_sign_negative());
        assert!(bounds.max.y.is_infinite() && bounds.max.y.is_sign_positive());
    }

    #[test]
    fn bounded_cylinder_bounds() {
        use crate::geometry::cylinders::Cylinder;
        use crate::geometry::shapes::Shape;

        let mut cylinder = Cylinder::new();
        cylinder.minimum = -5.0;
        cylinder.maximum = 3.0;
        let bounds = cylinder.bounds();

        assert_eq!(bounds.min, Point::new(-1.0, -5.0, -1.0));
        assert_eq!(bounds.max, Point::new(1.0, 3.0, 1.0));
    }

    #[test]
    fn unbounded_cone_bounds() {
        use crate::geometry::cones::Cone;
        use crate::geometry::shapes::Shape;

        let cone = Cone::new();
        let bounds = cone.bounds();

        // Infinite cone has infinite bounds
        assert!(bounds.min.x.is_infinite() && bounds.min.x.is_sign_negative());
        assert!(bounds.max.x.is_infinite() && bounds.max.x.is_sign_positive());
        assert!(bounds.min.y.is_infinite() && bounds.min.y.is_sign_negative());
        assert!(bounds.max.y.is_infinite() && bounds.max.y.is_sign_positive());
        assert!(bounds.min.z.is_infinite() && bounds.min.z.is_sign_negative());
        assert!(bounds.max.z.is_infinite() && bounds.max.z.is_sign_positive());
    }

    #[test]
    fn bounded_cone_bounds() {
        use crate::geometry::cones::Cone;
        use crate::geometry::shapes::Shape;

        let mut cone = Cone::new();
        cone.minimum = -5.0;
        cone.maximum = 3.0;
        let bounds = cone.bounds();

        // Radius at y=5 is 5, radius at y=3 is 3, so limit is 5
        assert_eq!(bounds.min, Point::new(-5.0, -5.0, -5.0));
        assert_eq!(bounds.max, Point::new(5.0, 3.0, 5.0));
    }

    #[test]
    fn plane_bounds() {
        use crate::geometry::planes::Plane;
        use crate::geometry::shapes::Shape;

        let plane = Plane::new();
        let bounds = plane.bounds();

        assert!(bounds.min.x.is_infinite() && bounds.min.x.is_sign_negative());
        assert!(bounds.max.x.is_infinite() && bounds.max.x.is_sign_positive());
        assert_eq!(bounds.min.y, 0.0);
        assert_eq!(bounds.max.y, 0.0);
        assert!(bounds.min.z.is_infinite() && bounds.min.z.is_sign_negative());
        assert!(bounds.max.z.is_infinite() && bounds.max.z.is_sign_positive());
    }
}
