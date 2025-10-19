use raytracer::core::color::Color;
use raytracer::core::matrices::Matrix4;
use raytracer::core::tuples::{Point, Tuple, Vector};
use raytracer::examples::fire_projectiles::{tick, Environment, Projectile};
use raytracer::rendering::camera::Camera;
use raytracer::rendering::canvas::Canvas;
use raytracer::rendering::objects::{HasMaterial, Object, Transformable};
use raytracer::rendering::world::World;
use raytracer::scene::lights::PointLight;
use raytracer::scene::materials::Material;
use raytracer::scene::transformations::view_transform;
use raytracer::scenes::loader::SceneFile;
use raytracer::scenes::obj::{obj_to_group, parse_obj_file};
use std::f64::consts::PI;

#[allow(dead_code)]
fn draw_chapter_2_arc() {
    println!("Drawing chapter 2 arc...");
    let canvas_width: u32 = 990;
    let canvas_height: u32 = 550;
    let start = Point::new(0.0, 1.0, 0.0);
    let velocity = Vector::new(1.0, 1.8, 0.0).normalize() * 11.25;
    let mut projectile = Projectile {
        position: start,
        velocity,
    };

    let gravity = Vector::new(0.0, -0.1, 0.0);
    let wind = Vector::new(-0.01, 0.0, 0.0);
    let environment = Environment { gravity, wind };
    let mut canvas = Canvas::new(canvas_width, canvas_height);

    let arc_color = Color::new(1.0, 0.0, 0.0);

    while (projectile.position.y > 0.0) && (projectile.position.x < canvas_width as f64) {
        let x = projectile.position.x.round() as u32;
        let y = projectile.position.y.round() as u32;
        canvas.write_pixel(x, canvas_height - y, &arc_color);
        projectile = tick(&environment, projectile);
    }

    let _ = canvas.to_ppm("outputs/chapter_2_arc.ppm");
    let _ = canvas.to_jpeg("outputs/chapter_2_arc.jpg");
}

#[allow(dead_code)]
fn draw_chapter_4_clock() {
    println!("Drawing chapter 4 clock...");
    let canvas_width: u32 = 500;
    let canvas_height: u32 = 500;
    let clock_width: f64 = 400.0;
    let clock_height: f64 = 400.0;
    let mut canvas = Canvas::new(canvas_width, canvas_height);
    let color = Color::new(1.0, 1.0, 1.0);
    let center = Point::new(0.0, 0.0, 0.0);
    let twelve = Point::new(0.0, 0.0, 1.0);

    for i in 0..12 {
        let r = Matrix4::rotate_y((i as f64) * PI / 6.0);
        let point = r * twelve;

        let point = point + center;
        let x = point.x();
        let z = point.z();

        // Convert points to canvas coordinates
        let x = (x * clock_width / 2.0) + (canvas_width as f64 / 2.0);
        let z = (z * clock_height / 2.0) + (canvas_height as f64 / 2.0);
        canvas.write_pixel(x.round() as u32, z.round() as u32, &color);
    }

    let _ = canvas.to_ppm("outputs/chapter_4_clock.ppm");
    let _ = canvas.to_jpeg("outputs/chapter_4_clock.jpg");
}

#[allow(dead_code)]
fn draw_chapter_5_circle_from_scene() {
    println!("Drawing chapter 5 circle from scene...");
    let scene =
        SceneFile::from_file("scenes/chapter_5.toml").expect("Failed to load chapter 5 scene");
    let camera = scene.build_camera();
    let world = scene.build_world();
    let canvas = camera.render(&world);
    let _ = canvas.to_ppm("outputs/chapter_5_circle.ppm");
    let _ = canvas.to_jpeg("outputs/chapter_5_circle.jpg");
}

#[allow(dead_code)]
fn draw_chapter_6_sphere_from_scene() {
    println!("Drawing chapter 6 sphere from scene...");
    let scene =
        SceneFile::from_file("scenes/chapter_6.toml").expect("Failed to load chapter 6 scene");
    let camera = scene.build_camera();
    let world = scene.build_world();
    let canvas = camera.render(&world);
    let _ = canvas.to_ppm("outputs/chapter_6_sphere.ppm");
    let _ = canvas.to_jpeg("outputs/chapter_6_sphere.jpg");
}

#[allow(dead_code)]
fn draw_chapter_7_and_8_world_from_scene() {
    println!("Drawing chapter 7 and 8 world from scene...");
    let scene = SceneFile::from_file("scenes/chapter_7_and_8.toml")
        .expect("Failed to load chapter 7 and 8 scene");
    let camera = scene.build_camera();
    let world = scene.build_world();
    let canvas = camera.render(&world);
    let _ = canvas.to_ppm("outputs/chapter_7_and_8_world.ppm");
    let _ = canvas.to_jpeg("outputs/chapter_7_and_8_world.jpg");
}

#[allow(dead_code)]
fn draw_chapter_9_plane_scene_from_scene() {
    println!("Drawing chapter 9 plane scene from scene...");
    let scene =
        SceneFile::from_file("scenes/chapter_9.toml").expect("Failed to load chapter 9 scene");
    let camera = scene.build_camera();
    let world = scene.build_world();
    let canvas = camera.render(&world);
    let _ = canvas.to_ppm("outputs/chapter_9_plane_scene.ppm");
    let _ = canvas.to_jpeg("outputs/chapter_9_plane_scene.jpg");
}

fn draw_chapter_10_patterns_from_scene() {
    println!("Drawing chapter 10 pattern showcase from scene...");
    let scene =
        SceneFile::from_file("scenes/chapter_10.toml").expect("Failed to load chapter 10 scene");
    let camera = scene.build_camera();
    let world = scene.build_world();
    let canvas = camera.render(&world);
    let _ = canvas.to_ppm("outputs/chapter_10_patterns.ppm");
    let _ = canvas.to_jpeg("outputs/chapter_10_patterns.jpg");
}

fn draw_chapter_11_reflections_from_scene() {
    println!("Drawing chapter 11 reflection showcase from scene...");
    let scene =
        SceneFile::from_file("scenes/chapter_11.toml").expect("Failed to load chapter 11 scene");
    let camera = scene.build_camera();
    let world = scene.build_world();
    let canvas = camera.render(&world);
    let _ = canvas.to_ppm("outputs/chapter_11_reflections.ppm");
    let _ = canvas.to_jpeg("outputs/chapter_11_reflections.jpg");
}

fn draw_chapter_12_cubes_from_scene() {
    println!("Drawing chapter 12 cube room scene from scene...");
    let scene =
        SceneFile::from_file("scenes/chapter_12.toml").expect("Failed to load chapter 12 scene");
    let camera = scene.build_camera();
    let world = scene.build_world();
    let canvas = camera.render(&world);
    let _ = canvas.to_ppm("outputs/chapter_12_cubes.ppm");
    let _ = canvas.to_jpeg("outputs/chapter_12_cubes.jpg");
}

fn draw_chapter_13_rocket_from_scene() {
    println!("Drawing chapter 13 rocket scene...");
    let scene =
        SceneFile::from_file("scenes/chapter_13.toml").expect("Failed to load chapter 13 scene");
    let camera = scene.build_camera();
    let world = scene.build_world();
    let canvas = camera.render(&world);
    let _ = canvas.to_ppm("outputs/chapter_13_rocket.ppm");
    let _ = canvas.to_jpeg("outputs/chapter_13_rocket.jpg");
}

fn draw_chapter_14_groups_from_scene() {
    println!("Drawing chapter 14 groups and bounding boxes scene...");
    let scene =
        SceneFile::from_file("scenes/chapter_14.toml").expect("Failed to load chapter 14 scene");
    let camera = scene.build_camera();
    let world = scene.build_world();
    let canvas = camera.render(&world);
    let _ = canvas.to_ppm("outputs/chapter_14_groups.ppm");
    let _ = canvas.to_jpeg("outputs/chapter_14_groups.jpg");
}

fn draw_chapter_15_triangles_from_scene() {
    println!("Drawing chapter 15 triangles scene...");
    let scene =
        SceneFile::from_file("scenes/chapter_15.toml").expect("Failed to load chapter 15 scene");
    let camera = scene.build_camera();
    let world = scene.build_world();
    let canvas = camera.render(&world);
    let _ = canvas.to_ppm("outputs/chapter_15_triangles.ppm");
    let _ = canvas.to_jpeg("outputs/chapter_15_triangles.jpg");
}

fn draw_chapter_15_teapot() {
    println!("Drawing chapter 15 teapot from OBJ file...");

    let obj_content = std::fs::read_to_string("teapot.obj").expect("Failed to read teapot.obj");
    let parser = parse_obj_file(&obj_content);
    println!("Loaded {} vertices and {} normals", parser.vertices.len() - 1, parser.normals.len() - 1);

    let teapot_group = obj_to_group(&parser);
    let mut teapot = Object::Group(teapot_group);

    teapot.set_transform(
        Matrix4::rotate_x(-PI / 2.0) // Rotate to stand upright
            * Matrix4::scale(0.1, 0.1, 0.1) // Scale down
            * Matrix4::translate(0.0, 0.0, 0.0),
    );

    let material = Material::builder()
        .color(Color::new(0.8, 0.5, 0.3))
        .ambient(0.1)
        .diffuse(0.7)
        .specular(0.3)
        .shininess(15.0)
        .build();
    teapot.set_material(material);

    // Add a large metallic plane
    let mut plane = Object::plane();
    plane.set_transform(Matrix4::translate(0.0, 0.0, 0.0));
    let plane_material = Material::builder()
        .color(Color::new(0.3, 0.3, 0.3))
        .ambient(0.1)
        .diffuse(0.6)
        .specular(0.9)
        .shininess(300.0)
        .reflectivity(0.5)
        .build();
    plane.set_material(plane_material);

    let light = PointLight::new(Point::new(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));

    let world = World {
        objects: vec![plane, teapot],
        light_source: Some(light),
    };

    let mut camera = Camera::new(800, 600, PI / 3.0);
    camera.transform = view_transform(
        Point::new(0.0, 1.5, -5.0),
        Point::new(0.0, 1.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
    );

    let canvas = camera.render(&world);
    let _ = canvas.to_ppm("outputs/chapter_15_teapot.ppm");
    let _ = canvas.to_jpeg("outputs/chapter_15_teapot.jpg");
}

fn main() {
    draw_chapter_2_arc();
    draw_chapter_4_clock();
    draw_chapter_5_circle_from_scene();
    draw_chapter_6_sphere_from_scene();
    draw_chapter_7_and_8_world_from_scene();
    draw_chapter_9_plane_scene_from_scene();
    draw_chapter_10_patterns_from_scene();
    draw_chapter_11_reflections_from_scene();
    draw_chapter_12_cubes_from_scene();
    draw_chapter_13_rocket_from_scene();
    draw_chapter_14_groups_from_scene();
    draw_chapter_15_triangles_from_scene();
    draw_chapter_15_teapot();
}
