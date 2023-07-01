use std::f64::consts::PI;
use raytracer::canvas::Canvas;
use raytracer::fire_projectiles::{tick, Environment, Projectile};
use raytracer::matrices::Matrix;
use raytracer::tuples::Tuple;

fn draw_chapter_2_arc() {
    let canvas_width: u32 = 990;
    let canvas_height: u32 = 550;
    let start = Tuple::point(0.0, 1.0, 0.0);
    let velocity = Tuple::vector(1.0, 1.8, 0.0).normalize() * 11.25;
    let mut projectile = Projectile {
        position: start,
        velocity,
    };

    let gravity = Tuple::vector(0.0, -0.1, 0.0);
    let wind = Tuple::vector(-0.01, 0.0, 0.0);
    let environment = Environment { gravity, wind };
    let mut canvas = Canvas::new(canvas_width, canvas_height);

    let arc_color = Tuple::color(1.0, 0.0, 0.0);

    while (projectile.position.y > 0.0) && (projectile.position.x < canvas_width as f64) {
        let x = projectile.position.x.round() as u32;
        let y = projectile.position.y.round() as u32;
        canvas.write(x, canvas_height - y, &arc_color);
        projectile = tick(&environment, projectile);
    }

    let _ = canvas.write_to_file("outputs/chapter_2_arc.ppm");
}

// For chapter_4
fn draw_chapter_4_clock() {
    let canvas_width: u32 = 500;
    let canvas_height: u32 = 500;
    let clock_width: f64 = 400.0;
    let clock_height: f64 = 400.0;
    let mut canvas = Canvas::new(canvas_width, canvas_height);
    let color = Tuple::color(1.0, 1.0, 1.0);
    let center = Tuple::point(0.0, 0.0, 0.0);
    let twelve = Tuple::point(0.0, 0.0, 1.0);

    for i in 0..12 {
        let r = Matrix::rotation_y((i as f64) * PI / 6.0);
        let point = r * twelve;

        let point = point + center;
        let x = point.x;
        let z = point.z;

        // Convert points to canvas coordinates
        let x = (x * clock_width / 2.0) + (canvas_width as f64 / 2.0);
        let z = (z * clock_height / 2.0) + (canvas_height as f64 / 2.0);
        canvas.write(x.round() as u32, z.round() as u32, &color);
    }

    let _ = canvas.write_to_file("outputs/chapter_4_clock.ppm");
}


fn main() {
    draw_chapter_2_arc();
    draw_chapter_4_clock();
}
