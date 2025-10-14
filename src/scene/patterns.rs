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
    Stripe { color_a: Color, color_b: Color },
    // Future patterns can be added here:
    // Gradient { color_a: Color, color_b: Color },
    // Ring { color_a: Color, color_b: Color },
    // Checkers { color_a: Color, color_b: Color },
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

    fn stripe_color_at(point: Point, color_a: &Color, color_b: &Color) -> Color {
        if point.x.floor() % 2.0 == 0.0 {
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
        }
    }

    pub fn color_at_object(&self, object: &Object, world_point: Point) -> Color {
        let object_point = object
            .transformation()
            .inverse()
            .expect("object transform should be invertible")
            * world_point;
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
}
