use crate::core::color::Color;
use crate::core::matrices::Matrix4;
use crate::core::tuples::{Point, Tuple};
use crate::rendering::canvas::Canvas;
use crate::rendering::rays::Ray;
use crate::rendering::world::World;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressDrawTarget, ProgressStyle};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub hsize: u32,
    pub vsize: u32,
    pub field_of_view: f64,
    pub transform: Matrix4,
    pub pixel_size: f64,
    pub half_width: f64,
    pub half_height: f64,
    pub aa_samples: u32, // Anti-aliasing samples per pixel (1 = no AA)
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
            aa_samples: 1, // Default: no anti-aliasing
        }
    }

    pub fn ray_for_pixel(self, px: usize, py: usize) -> Ray {
        self.ray_for_pixel_with_offset(px, py, 0.5, 0.5)
    }

    /// Generate a ray for a pixel with sub-pixel offset (0.0-1.0 range)
    fn ray_for_pixel_with_offset(self, px: usize, py: usize, x_offset: f64, y_offset: f64) -> Ray {
        let x_offset = (px as f64 + x_offset) * self.pixel_size;
        let y_offset = (py as f64 + y_offset) * self.pixel_size;

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

    /// Sample a pixel color with anti-aliasing if enabled
    fn sample_pixel(&self, world: &World, px: usize, py: usize) -> Color {
        let aa_samples = self.aa_samples.max(1);

        if aa_samples <= 1 {
            let ray = self.ray_for_pixel(px, py);
            return world.color_at(ray);
        }

        let samples_per_side = (aa_samples as f64).sqrt().ceil() as u32;
        let seed = (px as u64) * 73856093 + (py as u64) * 19349663;
        let mut rng = StdRng::seed_from_u64(seed);
        let mut color_sum = Color::black();
        let cell_size = 1.0 / samples_per_side as f64;

        for sy in 0..samples_per_side {
            for sx in 0..samples_per_side {
                let x_jitter = sx as f64 * cell_size + rng.gen::<f64>() * cell_size;
                let y_jitter = sy as f64 * cell_size + rng.gen::<f64>() * cell_size;

                let ray = self.ray_for_pixel_with_offset(px, py, x_jitter, y_jitter);
                color_sum = color_sum + world.color_at(ray);
            }
        }

        let total_samples = samples_per_side * samples_per_side;
        color_sum / total_samples as f64
    }

    /// Render the scene using GPU if available, otherwise fall back to CPU
    pub fn render(&self, world: &World) -> Canvas {
        #[cfg(feature = "gpu")]
        {
            use crate::gpu::GpuRenderer;

            if let Some(renderer) = GpuRenderer::try_new() {
                println!("  Using GPU acceleration");
                return renderer.render(self, world);
            }
            println!("  GPU unavailable, falling back to CPU");
        }

        self.render_cpu(world)
    }

    /// CPU-based rendering (original implementation)
    pub fn render_cpu(&self, world: &World) -> Canvas {
        let total_pixels = (self.hsize * self.vsize) as usize;

        let pb = ProgressBar::new(total_pixels as u64);
        pb.set_draw_target(ProgressDrawTarget::stderr_with_hz(10));

        if atty::is(atty::Stream::Stderr) {
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("  Rendering [{bar:40.cyan/blue}] {pos}/{len} pixels ({percent}%) ETA: {eta}")
                    .expect("Invalid progress bar template")
                    .progress_chars("█▓▒░  "),
            );
        } else {
            pb.set_draw_target(ProgressDrawTarget::hidden());
        }

        let pixels: Vec<_> = (0..total_pixels)
            .into_par_iter()
            .progress_with(pb)
            .map(|idx| {
                let x = (idx as u32) % self.hsize;
                let y = (idx as u32) / self.hsize;
                self.sample_pixel(world, x as usize, y as usize)
            })
            .collect();

        Canvas {
            width: self.hsize,
            height: self.vsize,
            pixels,
        }
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
    #[cfg(not(feature = "gpu"))]
    fn rendering_world_with_camera() {
        let world = World::default();
        let mut camera = Camera::new(11, 11, PI / 2.0);
        let from = Point::new(0.0, 0.0, -5.0);
        let to = Point::new(0.0, 0.0, 0.0);
        let up = Vector::new(0.0, 1.0, 0.0);
        camera.transform = view_transform(from, to, up);

        let image = camera.render(&world);
        let pixel = image.pixel_at(5, 5);
        let expected = Color::new(0.38066, 0.47583, 0.2855);
        assert!((pixel.red - expected.red).abs() < 0.01);
        assert!((pixel.green - expected.green).abs() < 0.01);
        assert!((pixel.blue - expected.blue).abs() < 0.01);
    }

    #[test]
    #[cfg(not(feature = "gpu"))]
    fn camera_with_anti_aliasing() {
        let world = World::default();
        let mut camera = Camera::new(11, 11, PI / 2.0);
        camera.aa_samples = 4;
        let from = Point::new(0.0, 0.0, -5.0);
        let to = Point::new(0.0, 0.0, 0.0);
        let up = Vector::new(0.0, 1.0, 0.0);
        camera.transform = view_transform(from, to, up);

        let image = camera.render(&world);
        let pixel = image.pixel_at(5, 5);
        let expected = Color::new(0.36006, 0.45007, 0.27004);
        assert!(
            (pixel.red - expected.red).abs() < 0.0001,
            "red: got {}, expected {}",
            pixel.red,
            expected.red
        );
        assert!(
            (pixel.green - expected.green).abs() < 0.0001,
            "green: got {}, expected {}",
            pixel.green,
            expected.green
        );
        assert!(
            (pixel.blue - expected.blue).abs() < 0.0001,
            "blue: got {}, expected {}",
            pixel.blue,
            expected.blue
        );
    }

    #[test]
    #[cfg(not(feature = "gpu"))]
    fn camera_handles_zero_aa_samples() {
        let world = World::default();
        let mut camera = Camera::new(11, 11, PI / 2.0);
        camera.aa_samples = 0;
        let from = Point::new(0.0, 0.0, -5.0);
        let to = Point::new(0.0, 0.0, 0.0);
        let up = Vector::new(0.0, 1.0, 0.0);
        camera.transform = view_transform(from, to, up);

        let image = camera.render(&world);
        let pixel = image.pixel_at(5, 5);
        let expected = Color::new(0.38066, 0.47583, 0.2855);
        assert!((pixel.red - expected.red).abs() < 0.01);
        assert!((pixel.green - expected.green).abs() < 0.01);
        assert!((pixel.blue - expected.blue).abs() < 0.01);
    }
}
