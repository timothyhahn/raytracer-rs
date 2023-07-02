use raytracer::canvas::Canvas;
use raytracer::fire_projectiles::{tick, Environment, Projectile};
use raytracer::intersections::Intersection;
use raytracer::lights::PointLight;
use raytracer::materials::Material;
use raytracer::matrices::Matrix;
use raytracer::rays::Ray;
use raytracer::sphere::Sphere;
use raytracer::tuples::{Color, Tuple};
use std::f64::consts::PI;
use raytracer::camera::Camera;
use raytracer::transformations::view_transform;
use raytracer::world::World;

#[allow(dead_code)]
fn draw_chapter_2_arc() {
    println!("Drawing chapter 2 arc...");
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
        canvas.write_pixel(x, canvas_height - y, &arc_color);
        projectile = tick(&environment, projectile);
    }

    let _ = canvas.write_to_file("outputs/chapter_2_arc.ppm");
}

#[allow(dead_code)]
fn draw_chapter_4_clock() {
    println!("Drawing chapter 4 clock...");
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
        canvas.write_pixel(x.round() as u32, z.round() as u32, &color);
    }

    let _ = canvas.write_to_file("outputs/chapter_4_clock.ppm");
}

#[allow(dead_code)]
fn draw_chapter_5_circle() {
    println!("Drawing chapter 5 circle...");
    let canvas_width = 500;
    let canvas_height = 500;
    let ray_origin = Tuple::point(0.0, 0.0, -5.0);
    let wall_z = 10.0;
    let wall_size = 7.0;
    let pixel_size = wall_size / (canvas_width as f64);
    let half = wall_size / 2.0;
    let mut canvas = Canvas::new(canvas_width, canvas_height);
    let color = Tuple::color(1.0, 0.0, 0.0);
    let sphere = Sphere::new();
    for y in 0..canvas_height - 1 {
        let world_y = half - pixel_size * (y as f64);
        for x in 0..canvas_width - 1 {
            let world_x = -half + pixel_size * (x as f64);
            let position = Tuple::point(world_x, world_y, wall_z);
            let r = Ray::new(ray_origin, (position - ray_origin).normalize());
            let intersections = sphere.intersect(r);
            if Intersection::hit(intersections).is_some() {
                canvas.write_pixel(x, y, &color);
            }
        }
    }
    let _ = canvas.write_to_file("outputs/chapter_5_circle.ppm");
}

#[allow(dead_code)]
fn draw_chapter_6_sphere() {
    println!("Drawing chapter 6 sphere...");
    let canvas_width = 500;
    let canvas_height = 500;
    let ray_origin = Tuple::point(0.0, 0.0, -5.0);
    let wall_z = 10.0;
    let wall_size = 7.0;
    let pixel_size = wall_size / (canvas_width as f64);
    let half = wall_size / 2.0;
    let mut canvas = Canvas::new(canvas_width, canvas_height);
    let mut sphere = Sphere::new();
    let material = Material {
        color: Tuple::color(1.0, 0.2, 1.0),
        ..Default::default()
    };
    sphere.set_material(material);
    let light_position = Tuple::point(-10.0, 10.0, -10.0);
    let light_color = Color::white();
    let light = PointLight::new(light_position, light_color);

    for y in 0..canvas_height - 1 {
        let world_y = half - pixel_size * (y as f64);
        for x in 0..canvas_width - 1 {
            let world_x = -half + pixel_size * (x as f64);
            let position = Tuple::point(world_x, world_y, wall_z);
            let r = Ray::new(ray_origin, (position - ray_origin).normalize());
            let intersections = sphere.intersect(r);
            if let Some(hit) = Intersection::hit(intersections) {
                let point = r.position(hit.time);
                let normal = hit.object.normal_at(point);
                let eye = -r.direction;
                let color = hit.object.material.lighting(light, point, eye, normal);
                canvas.write_pixel(x, y, &color);
            }
        }
    }
    let _ = canvas.write_to_file("outputs/chapter_6_sphere.ppm");
}

#[allow(dead_code)]
fn draw_chapter_7_world() {
    println!("Drawing chapter 7 world...");
    let floor_material = Material {
        color: Tuple::color(1.0, 0.9, 0.9),
        specular: 0.0,
        ..Default::default()
    };

    let floor = Sphere {
        transformation: Matrix::scaling(10.0, 0.01, 10.0),
        material: floor_material,
        ..Default::default()
    };

    let left_wall = Sphere {
        material: floor_material,
        transformation: Matrix::translation(0.0, 0.0, 5.0) * Matrix::rotation_y(-PI / 4.0) * Matrix::rotation_x(PI / 2.0) * Matrix::scaling(10.0, 0.01, 10.0),
        ..Default::default()
    };

    let right_wall = Sphere {
        material: floor_material,
        transformation: Matrix::translation(0.0, 0.0, 5.0) * Matrix::rotation_y(PI / 4.0) * Matrix::rotation_x(PI / 2.0) * Matrix::scaling(10.0, 0.01, 10.0),
        ..Default::default()
    };

    let middle_material = Material {
        color: Tuple::color(0.1, 1.0, 0.5),
        diffuse: 0.7,
        specular: 0.3,
        ..Default::default()
    };

    let middle = Sphere {
        material: middle_material,
        transformation: Matrix::translation(-0.5, 1.0, 0.5),
        ..Default::default()
    };

    let right_material = Material {
        color: Tuple::color(0.5, 1.0, 0.1),
        diffuse: 0.7,
        specular: 0.3,
        ..Default::default()
    };

    let right = Sphere {
        material: right_material,
        transformation: Matrix::translation(1.5, 0.5, -0.5) * Matrix::scaling(0.5, 0.5, 0.5),
        ..Default::default()
    };

    let left_material = Material {
        color: Tuple::color(1.0, 0.8, 0.1),
        diffuse: 0.7,
        specular: 0.3,
        ..Default::default()
    };

    let left = Sphere {
        material: left_material,
        transformation: Matrix::translation(-1.5, 0.33, -0.75) * Matrix::scaling(0.33, 0.33, 0.33),
        ..Default::default()
    };

    let world = World {
        objects: vec![floor, left_wall, right_wall, middle, right, left],
        light_source: Some(PointLight::new(Tuple::point(-10.0, 10.0, -10.0), Color::white())),
    };

    let mut camera = Camera::new(1000, 500, PI / 3.0);
    camera.transform = view_transform(Tuple::point(0.0, 1.5, -5.0), Tuple::point(0.0, 1.0, 0.0), Tuple::vector(0.0, 1.0, 0.0));

    let canvas = camera.render(world);
    let _ = canvas.write_to_file("outputs/chapter_7_world.ppm");
}

fn main() {
    // Commenting out old chapters now that they're taking a while
    // draw_chapter_2_arc();
    // draw_chapter_4_clock();
    // draw_chapter_5_circle();
    // draw_chapter_6_sphere();
    draw_chapter_7_world();
}
