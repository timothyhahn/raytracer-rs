use crate::{
    core::{color::Color, matrices::Matrix4, tuples::Point},
    rendering::objects::{Object, Transformable},
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Pattern {
    pub transform: Matrix4,
    pub kind: PatternKind,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PatternKind {
    Stripe {
        color_a: Color,
        color_b: Color,
    },
    Gradient {
        color_a: Color,
        color_b: Color,
    },
    Ring {
        color_a: Color,
        color_b: Color,
    },
    Checkers {
        color_a: Color,
        color_b: Color,
    },
    #[cfg(test)]
    Test, // Test pattern that returns Color(x, y, z) for point (x, y, z)
}

impl Transformable for Pattern {
    fn transformation(&self) -> Matrix4 {
        self.transform
    }

    fn set_transform(&mut self, transformation: Matrix4) {
        self.transform = transformation;
    }
}

impl Pattern {
    pub fn stripe(color_a: Color, color_b: Color) -> Self {
        Self {
            transform: Matrix4::identity(),
            kind: PatternKind::Stripe { color_a, color_b },
        }
    }

    pub fn gradient(color_a: Color, color_b: Color) -> Self {
        Self {
            transform: Matrix4::identity(),
            kind: PatternKind::Gradient { color_a, color_b },
        }
    }

    pub fn ring(color_a: Color, color_b: Color) -> Self {
        Self {
            transform: Matrix4::identity(),
            kind: PatternKind::Ring { color_a, color_b },
        }
    }

    pub fn checkers(color_a: Color, color_b: Color) -> Self {
        Self {
            transform: Matrix4::identity(),
            kind: PatternKind::Checkers { color_a, color_b },
        }
    }

    #[cfg(test)]
    pub fn test() -> Self {
        Self {
            transform: Matrix4::identity(),
            kind: PatternKind::Test,
        }
    }

    fn stripe_color_at(point: Point, color_a: &Color, color_b: &Color) -> Color {
        if point.x.floor() % 2.0 == 0.0 {
            *color_a
        } else {
            *color_b
        }
    }

    fn gradient_color_at(point: Point, color_a: &Color, color_b: &Color) -> Color {
        let distance = *color_b - *color_a;
        let fraction = point.x.clamp(0.0, 1.0);
        *color_a + distance * fraction
    }

    fn ring_color_at(point: Point, color_a: &Color, color_b: &Color) -> Color {
        // floor(sqrt(p^2x + p^2z)) mod 2
        if (point.x.powi(2) + point.z.powi(2)).sqrt().floor() % 2.0 == 0.0 {
            *color_a
        } else {
            *color_b
        }
    }

    fn checkers_color_at(point: Point, color_a: &Color, color_b: &Color) -> Color {
        // Add small epsilon and floor to avoid floating point precision issues at boundaries
        const EPSILON: f64 = 1e-6;
        let x = (point.x + EPSILON).floor() as i32;
        let y = (point.y + EPSILON).floor() as i32;
        let z = (point.z + EPSILON).floor() as i32;

        if (x + y + z) & 1 == 0 {
            *color_a
        } else {
            *color_b
        }
    }

    pub fn color_at(&self, point: Point) -> Color {
        match self.kind {
            PatternKind::Stripe { color_a, color_b } => {
                Self::stripe_color_at(point, &color_a, &color_b)
            }
            PatternKind::Gradient { color_a, color_b } => {
                Self::gradient_color_at(point, &color_a, &color_b)
            }
            PatternKind::Ring { color_a, color_b } => {
                Self::ring_color_at(point, &color_a, &color_b)
            }
            PatternKind::Checkers { color_a, color_b } => {
                Self::checkers_color_at(point, &color_a, &color_b)
            }
            #[cfg(test)]
            PatternKind::Test => Color::new(point.x, point.y, point.z),
        }
    }

    pub fn color_at_object(&self, object: &Object, world_point: Point) -> Color {
        // Use the new world_to_object method which handles parent transformations
        let object_point = object.world_to_object(world_point);
        let pattern_point = self
            .transform
            .inverse()
            .expect("pattern transform should be invertible")
            * object_point;
        self.color_at(pattern_point)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::{
            color::Color,
            matrices::Matrix4,
            tuples::{Point, Tuple},
        },
        rendering::objects::{Object, Transformable},
        scene::patterns::Pattern,
    };

    #[test]
    fn default_pattern_transform() {
        let pattern = Pattern::test();
        assert_eq!(pattern.transform, Matrix4::identity());
    }

    #[test]
    fn set_pattern_transform() {
        let mut pattern = Pattern::test();
        pattern.set_transform(Matrix4::translate(1.0, 2.0, 3.0));
        assert_eq!(pattern.transformation(), Matrix4::translate(1.0, 2.0, 3.0));
    }

    #[test]
    fn pattern_with_object_transform() {
        let mut object = Object::sphere();
        object.set_transform(Matrix4::scale(2.0, 2.0, 2.0));
        let pattern = Pattern::test();
        let color = pattern.color_at_object(&object, Point::new(2.0, 3.0, 4.0));
        assert_eq!(color, Color::new(1.0, 1.5, 2.0));
    }

    #[test]
    fn pattern_with_pattern_transform() {
        let object = Object::sphere();
        let mut pattern = Pattern::test();
        pattern.set_transform(Matrix4::scale(2.0, 2.0, 2.0));
        let color = pattern.color_at_object(&object, Point::new(2.0, 3.0, 4.0));
        assert_eq!(color, Color::new(1.0, 1.5, 2.0));
    }

    #[test]
    fn pattern_with_both_transforms() {
        let mut object = Object::sphere();
        object.set_transform(Matrix4::scale(2.0, 2.0, 2.0));
        let mut pattern = Pattern::test();
        pattern.set_transform(Matrix4::translate(0.5, 1.0, 1.5));
        let color = pattern.color_at_object(&object, Point::new(2.5, 3.0, 3.5));
        assert_eq!(color, Color::new(0.75, 0.5, 0.25));
    }

    #[test]
    fn stripe_pattern_constant_in_y() {
        let pattern = Pattern::stripe(Color::WHITE, Color::BLACK);
        assert_eq!(pattern.color_at(Point::new(0.0, 0.0, 0.0)), Color::WHITE);
        assert_eq!(pattern.color_at(Point::new(0.0, 1.0, 0.0)), Color::WHITE);
        assert_eq!(pattern.color_at(Point::new(0.0, 2.0, 0.0)), Color::WHITE);
    }

    #[test]
    fn stripe_pattern_constant_in_z() {
        let pattern = Pattern::stripe(Color::WHITE, Color::BLACK);
        assert_eq!(pattern.color_at(Point::new(0.0, 0.0, 0.0)), Color::WHITE);
        assert_eq!(pattern.color_at(Point::new(0.0, 0.0, 1.0)), Color::WHITE);
        assert_eq!(pattern.color_at(Point::new(0.0, 0.0, 2.0)), Color::WHITE);
    }

    #[test]
    fn stripe_pattern_alternates_in_x() {
        let pattern = Pattern::stripe(Color::WHITE, Color::BLACK);
        assert_eq!(pattern.color_at(Point::new(0.0, 0.0, 0.0)), Color::WHITE);
        assert_eq!(pattern.color_at(Point::new(0.9, 0.0, 0.0)), Color::WHITE);
        assert_eq!(pattern.color_at(Point::new(1.0, 0.0, 0.0)), Color::BLACK);
        assert_eq!(pattern.color_at(Point::new(-0.1, 0.0, 0.0)), Color::BLACK);
        assert_eq!(pattern.color_at(Point::new(-1.0, 0.0, 0.0)), Color::BLACK);
        assert_eq!(pattern.color_at(Point::new(-1.1, 0.0, 0.0)), Color::WHITE);
    }

    #[test]
    fn stripes_with_object_transformation() {
        let mut object = Object::sphere();
        object.set_transform(Matrix4::scale(2.0, 2.0, 2.0));
        let pattern = Pattern::stripe(Color::WHITE, Color::BLACK);
        assert_eq!(
            pattern.color_at_object(&object, Point::new(1.5, 0.0, 0.0)),
            Color::WHITE
        );
    }

    #[test]
    fn stripes_with_pattern_transformation() {
        let object = Object::sphere();
        let mut pattern = Pattern::stripe(Color::WHITE, Color::BLACK);
        pattern.set_transform(Matrix4::scale(2.0, 2.0, 2.0));
        assert_eq!(
            pattern.color_at_object(&object, Point::new(1.5, 0.0, 0.0)),
            Color::WHITE
        );
    }

    #[test]
    fn stripes_with_object_and_pattern_transformation() {
        let mut object = Object::sphere();
        object.set_transform(Matrix4::scale(2.0, 2.0, 2.0));
        let mut pattern = Pattern::stripe(Color::WHITE, Color::BLACK);
        pattern.set_transform(Matrix4::translate(0.5, 0.0, 0.0));
        assert_eq!(
            pattern.color_at_object(&object, Point::new(2.5, 0.0, 0.0)),
            Color::WHITE
        );
    }

    #[test]
    fn gradient_linear_interpolation_between_colors() {
        let pattern = Pattern::gradient(Color::WHITE, Color::BLACK);
        assert_eq!(pattern.color_at(Point::new(0.0, 0.0, 0.0)), Color::WHITE);
        assert_eq!(
            pattern.color_at(Point::new(0.25, 0.0, 0.0)),
            Color::new(0.75, 0.75, 0.75)
        );
        assert_eq!(
            pattern.color_at(Point::new(0.5, 0.0, 0.0)),
            Color::new(0.5, 0.5, 0.5)
        );
        assert_eq!(
            pattern.color_at(Point::new(0.75, 0.0, 0.0)),
            Color::new(0.25, 0.25, 0.25)
        );
        assert_eq!(pattern.color_at(Point::new(1.0, 0.0, 0.0)), Color::BLACK);
    }

    #[test]
    pub fn ring_should_extend_in_both_x_and_z() {
        let pattern = Pattern::ring(Color::WHITE, Color::BLACK);
        assert_eq!(pattern.color_at(Point::new(0.0, 0.0, 0.0)), Color::WHITE);
        assert_eq!(pattern.color_at(Point::new(1.0, 0.0, 0.0)), Color::BLACK);
        assert_eq!(pattern.color_at(Point::new(0.0, 0.0, 1.0)), Color::BLACK);
        assert_eq!(
            pattern.color_at(Point::new(0.708, 0.0, 0.708)),
            Color::BLACK
        );
    }

    #[test]
    pub fn checkers_should_repeat_in_x() {
        let pattern = Pattern::checkers(Color::WHITE, Color::BLACK);
        assert_eq!(pattern.color_at(Point::new(0.0, 0.0, 0.0)), Color::WHITE);
        assert_eq!(pattern.color_at(Point::new(0.99, 0.0, 0.0)), Color::WHITE);
        assert_eq!(pattern.color_at(Point::new(1.01, 0.0, 0.0)), Color::BLACK);
    }

    #[test]
    pub fn checkers_should_repeat_in_y() {
        let pattern = Pattern::checkers(Color::WHITE, Color::BLACK);
        assert_eq!(pattern.color_at(Point::new(0.0, 0.0, 0.0)), Color::WHITE);
        assert_eq!(pattern.color_at(Point::new(0.0, 0.99, 0.0)), Color::WHITE);
        assert_eq!(pattern.color_at(Point::new(0.0, 1.01, 0.0)), Color::BLACK);
    }

    #[test]
    pub fn checkers_should_repeat_in_z() {
        let pattern = Pattern::checkers(Color::WHITE, Color::BLACK);
        assert_eq!(pattern.color_at(Point::new(0.0, 0.0, 0.0)), Color::WHITE);
        assert_eq!(pattern.color_at(Point::new(0.0, 0.0, 0.99)), Color::WHITE);
        assert_eq!(pattern.color_at(Point::new(0.0, 0.0, 1.01)), Color::BLACK);
    }
}
