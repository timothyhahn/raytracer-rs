use crate::core::matrices::Matrix4;
use crate::core::tuples::{Point, Tuple};
use crate::rendering::canvas::Canvas;
use crate::rendering::rays::Ray;
use crate::rendering::world::World;

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub hsize: u32,
    pub vsize: u32,
    pub field_of_view: f64,
    pub transform: Matrix4,
    pub pixel_size: f64,
    pub half_width: f64,
    pub half_height: f64,
}

impl Camera {
    pub fn new(hsize: u32, vsize: u32, field_of_view: f64) -> Camera {
        let half_view = (field_of_view / 2.0).tan();
        let aspect = hsize as f64 / vsize as f64;
        let (half_width, half_height) = if aspect > 1.0 {
            (half_view, half_view / aspect)
        } else {
            (half_view * aspect, half_view)
        };

        Camera {
            hsize,
            vsize,
            field_of_view,
            transform: Matrix4::identity(),
            pixel_size: (half_width * 2.0) / hsize as f64,
            half_width,
            half_height,
        }
    }

    pub fn ray_for_pixel(self, px: usize, py: usize) -> Ray {
        let x_offset = (px as f64 + 0.5) * self.pixel_size;
        let y_offset = (py as f64 + 0.5) * self.pixel_size;

        let world_x = self.half_width - x_offset;
        let world_y = self.half_height - y_offset;

        let inverse_transform = self
            .transform
            .inverse()
            .expect("camera transform should be invertible");
        let pixel = inverse_transform * Point::new(world_x, world_y, -1.0);
        let origin = inverse_transform * Point::new(0.0, 0.0, 0.0);
        let direction = (pixel - origin).normalize();
        Ray::new(origin, direction)
    }

    pub fn render(&self, world: World) -> Canvas {
        let mut image = Canvas::new(self.hsize, self.vsize);
        for y in 0..self.vsize {
            for x in 0..self.hsize {
                let ray = self.ray_for_pixel(x as usize, y as usize);
                let color = world.color_at(ray);
                image.write_pixel(x, y, &color);
            }
        }

        image
    }
}

#[cfg(test)]
mod tests {
    use crate::core::color::Color;
    use crate::core::floats::float_equal;
    use crate::core::matrices::Matrix4;
    use crate::core::tuples::{Point, Tuple, Vector};
    use crate::rendering::camera::Camera;
    use crate::rendering::world::World;
    use crate::scene::transformations::view_transform;
    use std::f64::consts::PI;

    #[test]
    fn constructing_camera() {
        let hsize = 160;
        let vsize = 120;
        let field_of_view = PI / 2.0;
        let camera = Camera::new(hsize, vsize, field_of_view);
        assert_eq!(camera.hsize, 160);
        assert_eq!(camera.vsize, 120);
        assert_eq!(camera.field_of_view, PI / 2.0);
        assert_eq!(camera.transform, Matrix4::identity());
    }

    #[test]
    fn pixel_size_for_horizontal_canvas() {
        let camera = Camera::new(200, 125, PI / 2.0);
        assert!(float_equal(camera.pixel_size, 0.01));
    }

    #[test]
    fn pixel_size_for_vertical_canvas() {
        let camera = Camera::new(125, 200, PI / 2.0);
        assert!(float_equal(camera.pixel_size, 0.01));
    }

    #[test]
    fn constructing_ray_through_center_of_canvas() {
        let camera = Camera::new(201, 101, PI / 2.0);
        let ray = camera.ray_for_pixel(100, 50);
        assert_eq!(ray.origin, Point::new(0.0, 0.0, 0.0));
        assert_eq!(ray.direction, Vector::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn constructing_ray_through_corner_of_canvas() {
        let camera = Camera::new(201, 101, PI / 2.0);
        let ray = camera.ray_for_pixel(0, 0);
        assert_eq!(ray.origin, Point::new(0.0, 0.0, 0.0));
        assert_eq!(
            ray.direction,
            Vector::new(0.6651864261194508, 0.3325932130597254, -0.6685123582500481)
        );
    }

    #[test]
    fn constructing_ray_when_camera_transformed() {
        let mut camera = Camera::new(201, 101, PI / 2.0);
        camera.transform = Matrix4::rotate_y(PI / 4.0) * Matrix4::translate(0.0, -2.0, 5.0);
        let ray = camera.ray_for_pixel(100, 50);
        assert_eq!(ray.origin, Point::new(0.0, 2.0, -5.0));
        assert_eq!(
            ray.direction,
            Vector::new(2.0_f64.sqrt() / 2.0, 0.0, -(2.0_f64.sqrt() / 2.0))
        );
    }

    #[test]
    fn rendering_world_with_camera() {
        let world = World::default();
        let mut camera = Camera::new(11, 11, PI / 2.0);
        let from = Point::new(0.0, 0.0, -5.0);
        let to = Point::new(0.0, 0.0, 0.0);
        let up = Vector::new(0.0, 1.0, 0.0);
        camera.transform = view_transform(from, to, up);

        let image = camera.render(world);
        assert_eq!(image.pixel_at(5, 5), Color::new(0.38066, 0.47583, 0.2855));
    }
}
