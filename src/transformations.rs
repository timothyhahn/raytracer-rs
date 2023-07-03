use crate::matrices::Matrix4;
use crate::tuples::{Point, Vector};

pub fn view_transform(from: Point, to: Point, up: Vector) -> Matrix4 {
    let forward = (to - from).normalize();
    let up_normalized = up.normalize();
    let left = forward.cross(&up_normalized);
    let true_up = left.cross(&forward);
    let orientation = Matrix4::new([
        [left.x, left.y, left.z, 0.0],
        [true_up.x, true_up.y, true_up.z, 0.0],
        [-forward.x, -forward.y, -forward.z, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    orientation * Matrix4::translate(-from.x, -from.y, -from.z)
}

#[cfg(test)]
mod tests {
    use crate::matrices::Matrix4;
    use crate::transformations::view_transform;
    use crate::tuples::{Point, Tuple, Vector};

    #[test]
    fn transformation_matrix_for_default_orientation() {
        let from = Point::new(0.0, 0.0, 0.0);
        let to = Point::new(0.0, 0.0, -1.0);
        let up = Vector::new(0.0, 1.0, 0.0);
        let transformation = view_transform(from, to, up);
        assert_eq!(transformation, Matrix4::identity());
    }

    #[test]
    fn transformation_matrix_looking_in_positive_z_direction() {
        let from = Point::new(0.0, 0.0, 0.0);
        let to = Point::new(0.0, 0.0, 1.0);
        let up = Vector::new(0.0, 1.0, 0.0);
        let transformation = view_transform(from, to, up);
        assert_eq!(transformation, Matrix4::scale(-1.0, 1.0, -1.0));
    }

    #[test]
    fn view_transformation_transforms_the_world() {
        let from = Point::new(0.0, 0.0, 8.0);
        let to = Point::new(0.0, 0.0, 0.0);
        let up = Vector::new(0.0, 1.0, 0.0);
        let transformation = view_transform(from, to, up);
        assert_eq!(transformation, Matrix4::translate(0.0, 0.0, -8.0));
    }

    #[test]
    fn an_arbitrary_view_transformation() {
        let from = Point::new(1.0, 3.0, 2.0);
        let to = Point::new(4.0, -2.0, 8.0);
        let up = Vector::new(1.0, 1.0, 0.0);
        let transformation = view_transform(from, to, up);
        let expected_data = [
            [-0.50709, 0.50709, 0.67612, -2.36643],
            [0.76772, 0.60609, 0.12122, -2.82843],
            [-0.35857, 0.59761, -0.71714, 0.00000],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let expected_matrix = Matrix4::new(expected_data);
        assert_eq!(transformation, expected_matrix);
    }
}
