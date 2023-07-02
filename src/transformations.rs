use crate::matrices::Matrix;
use crate::tuples::Tuple;

pub fn view_transform(from: Tuple, to: Tuple, up: Tuple) -> Matrix {
    let forward = (to - from).normalize();
    let up_normalized = up.normalize();
    let left = forward.cross(&up_normalized);
    let true_up = left.cross(&forward);
    let orientation = Matrix::new(vec![
        vec![left.x, left.y, left.z, 0.0],
        vec![true_up.x, true_up.y, true_up.z, 0.0],
        vec![-forward.x, -forward.y, -forward.z, 0.0],
        vec![0.0, 0.0, 0.0, 1.0]
    ]);

    orientation * Matrix::translation(-from.x, -from.y, -from.z)
}

#[cfg(test)]
mod tests {
    use crate::matrices::Matrix;
    use crate::transformations::view_transform;
    use crate::tuples::Tuple;

    #[test]
    fn transformation_matrix_for_default_orientation() {
        let from = Tuple::point(0.0, 0.0, 0.0);
        let to = Tuple::point(0.0, 0.0, -1.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        let transformation = view_transform(from, to, up);
        assert_eq!(transformation, Matrix::identity(4));
    }

    #[test]
    fn transformation_matrix_looking_in_positive_z_direction() {
        let from = Tuple::point(0.0, 0.0, 0.0);
        let to = Tuple::point(0.0, 0.0, 1.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        let transformation = view_transform(from, to, up);
        assert_eq!(transformation, Matrix::scaling(-1.0, 1.0, -1.0));
    }

    #[test]
    fn view_transformation_transforms_the_world() {
        let from = Tuple::point(0.0, 0.0, 8.0);
        let to = Tuple::point(0.0, 0.0, 0.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        let transformation = view_transform(from, to, up);
        assert_eq!(transformation, Matrix::translation(0.0, 0.0, -8.0));
    }

    #[test]
    fn an_arbitrary_view_transformation() {
        let from = Tuple::point(1.0, 3.0, 2.0);
        let to = Tuple::point(4.0, -2.0, 8.0);
        let up = Tuple::vector(1.0, 1.0, 0.0);
        let transformation = view_transform(from, to, up);
        let expected_data = vec![
            vec![-0.50709, 0.50709, 0.67612, -2.36643],
            vec![0.76772, 0.60609, 0.12122, -2.82843],
            vec![-0.35857, 0.59761, -0.71714, 0.00000],
            vec![0.0, 0.0, 0.0, 1.0],
        ];
        let expected_matrix = Matrix::new(expected_data);
        assert_eq!(transformation, expected_matrix);
    }
}