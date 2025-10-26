//! Ray Tracer - Main Entry Point
//!
//! This binary provides the main rendering interface for the ray tracer.
//! It supports three types of scenes:
//!
//! 1. **Early Chapters** (chapter_2_arc, chapter_4_clock)
//!    - Hardcoded scenes that predate the TOML scene file system
//!    - Used for basic demonstrations of projectile physics and transformations
//!
//! 2. **TOML-Based Scenes** (chapter_5 through chapter_15)
//!    - Declarative scene definitions loaded from `scenes/*.toml`
//!    - Support full feature set: materials, patterns, lights, transformations
//!
//! 3. **OBJ Model Rendering** (teapot, bunny, dragon)
//!    - Loads triangulated meshes from `obj/*.obj` files
//!    - Uses standard setup: checkerboard plane, area light, configurable camera
//!
//! ## Rendering Modes
//!
//! - **CPU Mode** (default): Software ray tracing with progress bars
//! - **GPU Mode** (--features gpu): Hardware-accelerated with chunked batching
//!
//! ## Usage
//!
//! ```bash
//! # Render a specific scene
//! cargo run --release -- chapter_9
//!
//! # Render with GPU acceleration
//! cargo run --release --features gpu -- chapter_9
//!
//! # Render all auto-discovered scenes
//! cargo run --release
//! ```

use clap::Parser;
use raytracer::scenes::loader::SceneFile;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about = "Ray Tracer - Render scenes with CPU or GPU", long_about = None)]
struct Args {
    /// Scene name to render (e.g., "chapter_9", "chapter_11", "teapot", "bunny", "dragon")
    /// If not specified, auto-discovers and renders all available scenes
    scene: Option<String>,
}

fn main() {
    let args = Args::parse();

    let using_gpu = cfg!(feature = "gpu");
    let output_dir = if using_gpu { "outputs/gpu" } else { "outputs" };

    println!("Ray Tracer");
    println!("  Mode: {}", if using_gpu { "GPU" } else { "CPU" });
    println!("  Output: {}/", output_dir);
    println!();

    std::fs::create_dir_all(output_dir).expect("Failed to create output directory");

    match args.scene {
        Some(scene_name) => {
            render_scene(&scene_name, output_dir);
        }
        None => {
            let mut scenes = Vec::new();

            // Early chapters predate TOML scene files
            scenes.push("chapter_2_arc".to_string());
            scenes.push("chapter_4_clock".to_string());

            if let Ok(entries) = std::fs::read_dir("scenes") {
                for entry in entries.flatten() {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.ends_with(".toml") {
                            let scene_name = filename.trim_end_matches(".toml");
                            scenes.push(scene_name.to_string());
                        }
                    }
                }
            }

            // Sort for consistent ordering
            scenes.sort();

            println!("Found {} scenes to render", scenes.len());

            for scene_name in &scenes {
                render_scene(scene_name, output_dir);
            }
        }
    }

    println!();
    println!("✓ All renders complete!");
}

/// Render a scene by name, automatically detecting the appropriate rendering method.
///
/// This dispatcher function handles three categories of scenes:
/// - Early chapters (chapter_2_arc, chapter_4_clock): Hardcoded procedural scenes
/// - TOML scenes (chapter_5-15): Loaded from `scenes/*.toml` files
/// - OBJ models (teapot, bunny, dragon): Triangle mesh rendering with standard setup
///
/// The function checks for a corresponding TOML file first, then falls back to
/// special-case handlers for non-TOML scenes.
fn render_scene(scene_name: &str, output_dir: &str) {
    println!("Rendering {}...", scene_name);

    let scene_file = format!("scenes/{}.toml", scene_name);

    // Check if scene file exists
    if !Path::new(&scene_file).exists() {
        // Handle special cases that aren't in scene files
        match scene_name {
            "chapter_2_arc" => {
                draw_chapter_2_arc(output_dir);
                return;
            }
            "chapter_4_clock" => {
                draw_chapter_4_clock(output_dir);
                return;
            }
            "teapot" => {
                render_obj_model("teapot", output_dir);
                return;
            }
            "bunny" => {
                render_obj_model("bunny", output_dir);
                return;
            }
            "dragon" => {
                render_obj_model("dragon", output_dir);
                return;
            }
            _ => {
                eprintln!("  ✗ Scene file not found: {}", scene_file);
                return;
            }
        }
    }

    let scene = match SceneFile::from_file(&scene_file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  ✗ Failed to load scene: {}", e);
            return;
        }
    };

    let camera = scene.build_camera();
    let world = scene.build_world();
    let canvas = camera.render(&world);

    let output_name = match scene_name {
        "chapter_5" => "chapter_5_circle",
        "chapter_6" => "chapter_6_sphere",
        "chapter_7_and_8" => "chapter_7_and_8_world",
        "chapter_9" => "chapter_9_plane_scene",
        "chapter_10" => "chapter_10_patterns",
        "chapter_11" => "chapter_11_reflections",
        "chapter_12" => "chapter_12_cubes",
        "chapter_13" => "chapter_13_rocket",
        "chapter_14" => "chapter_14_groups",
        "chapter_15" => "chapter_15_triangles",
        _ => scene_name,
    };

    let jpg_path = format!("{}/{}.jpg", output_dir, output_name);
    let ppm_path = format!("{}/{}.ppm", output_dir, output_name);

    match canvas.to_jpeg(&jpg_path) {
        Ok(_) => println!("  ✓ Saved to {}", jpg_path),
        Err(e) => eprintln!("  ✗ Failed to save: {}", e),
    }
    match canvas.to_ppm(&ppm_path) {
        Ok(_) => println!("  ✓ Saved to {}", ppm_path),
        Err(e) => eprintln!("  ✗ Failed to save: {}", e),
    }
}

// ============================================================================
// Special Scene Renderers
// ============================================================================
//
// These functions implement hardcoded scenes for early chapters before the
// TOML scene file system was introduced.

/// Render Chapter 2: Projectile Arc
///
/// Demonstrates basic physics simulation by plotting the trajectory of a
/// projectile under gravity. This was the first example to combine tuples,
/// vectors, and canvas rendering.
fn draw_chapter_2_arc(output_dir: &str) {
    use raytracer::core::color::Color;
    use raytracer::core::tuples::{Point, Tuple, Vector};
    use raytracer::examples::fire_projectiles::{tick, Environment, Projectile};
    use raytracer::rendering::canvas::Canvas;

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

    let jpg_path = format!("{}/chapter_2_arc.jpg", output_dir);
    let ppm_path = format!("{}/chapter_2_arc.ppm", output_dir);
    canvas.to_jpeg(&jpg_path).expect("Failed to save JPG");
    canvas.to_ppm(&ppm_path).expect("Failed to save PPM");
    println!("  ✓ Saved to {} and {}", jpg_path, ppm_path);
}

/// Render Chapter 4: Analog Clock Face
///
/// Demonstrates matrix transformations (rotation and translation) by placing
/// 12 dots in a circle to form a clock face. Each dot is positioned by rotating
/// a point at 12 o'clock around the origin.
fn draw_chapter_4_clock(output_dir: &str) {
    use raytracer::core::color::Color;
    use raytracer::core::matrices::Matrix4;
    use raytracer::core::tuples::{Point, Tuple};
    use raytracer::rendering::canvas::Canvas;
    use std::f64::consts::PI;

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
        let point = r * twelve + center;
        let x = point.x();
        let z = point.z();

        let x = (x * clock_width / 2.0) + (canvas_width as f64 / 2.0);
        let z = (z * clock_height / 2.0) + (canvas_height as f64 / 2.0);
        canvas.write_pixel(x.round() as u32, z.round() as u32, &color);
    }

    let jpg_path = format!("{}/chapter_4_clock.jpg", output_dir);
    let ppm_path = format!("{}/chapter_4_clock.ppm", output_dir);
    canvas.to_jpeg(&jpg_path).expect("Failed to save JPG");
    canvas.to_ppm(&ppm_path).expect("Failed to save PPM");
    println!("  ✓ Saved to {} and {}", jpg_path, ppm_path);
}

/// Render an OBJ model with standardized scene setup.
///
/// This function provides a consistent rendering environment for displaying
/// complex triangle meshes loaded from Wavefront OBJ files.
///
/// ## Standard Setup
///
/// - **Model Material**: White with high specularity (glossy appearance)
/// - **Ground Plane**: Grey checkerboard pattern with reflectivity
/// - **Lighting**: 8×8 area light for soft shadows
/// - **Camera**: 1280×720 resolution with configurable position and samples
///
/// ## Supported Models
///
/// - `teapot`: Utah teapot (~6K triangles)
/// - `bunny`: Stanford bunny (~144K triangles)
/// - `dragon`: Stanford dragon (~250K triangles)
///
/// Each model has custom transforms and camera positions optimized for viewing.
/// The grey checkerboard plane (from dragon animation work) provides better
/// visualization of glass/reflective materials compared to solid colors.
fn render_obj_model(model_name: &str, output_dir: &str) {
    use raytracer::core::color::Color;
    use raytracer::core::matrices::Matrix4;
    use raytracer::core::tuples::{Point, Tuple, Vector};
    use raytracer::rendering::camera::Camera;
    use raytracer::rendering::objects::{HasMaterial, Object, Transformable};
    use raytracer::rendering::world::World;
    use raytracer::scene::lights::Light;
    use raytracer::scene::materials::Material;
    use raytracer::scene::patterns::Pattern;
    use raytracer::scene::transformations::view_transform;
    use raytracer::scenes::obj::{obj_to_group, parse_obj_file};
    use std::f64::consts::PI;

    let obj_path = format!("obj/{}.obj", model_name);
    let obj_content = std::fs::read_to_string(&obj_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", obj_path));
    let parser = parse_obj_file(&obj_content);

    println!(
        "  Loaded {} vertices, {} normals",
        parser.vertices.len() - 1,
        parser.normals.len() - 1
    );

    let model_group = obj_to_group(&parser);
    let mut model = Object::Group(model_group);

    // Model-specific transform and camera settings
    let (transform, camera_from, camera_to, aa_samples) = match model_name {
        "teapot" => (
            Matrix4::rotate_x(-PI / 2.0) * Matrix4::scale(0.1, 0.1, 0.1),
            Point::new(0.0, 1.5, -5.0),
            Point::new(0.0, 1.0, 0.0),
            128,
        ),
        "bunny" => (
            Matrix4::translate(0.0, -0.4, 0.0)
                * Matrix4::rotate_y(PI * 0.75 + 70.0 * PI / 180.0)
                * Matrix4::scale(2.5, 2.5, 2.5),
            Point::new(0.0, 2.0, -9.0),
            Point::new(0.0, 1.5, 0.0),
            1024,
        ),
        "dragon" => (
            Matrix4::translate(0.5, 2.5, 3.0) // Centered, higher, pushed back
                * Matrix4::rotate_y(-PI / 6.0 + PI) // Rotate 180° from current
                * Matrix4::scale(0.0465, 0.0465, 0.0465), // Dragon is huge, normalize to ~2 units
            Point::new(0.0, 2.5, -8.5),
            Point::new(0.0, 2.0, 0.0),
            1024, // High sampling for dragon's complex mesh
        ),
        _ => (
            Matrix4::identity(),
            Point::new(0.0, 1.5, -5.0),
            Point::new(0.0, 1.0, 0.0),
            128,
        ),
    };

    model.set_transform(transform);

    let material = Material::builder()
        .color(Color::new(1.0, 1.0, 1.0))
        .ambient(0.1)
        .diffuse(0.7)
        .specular(0.9)
        .shininess(300.0)
        .build();
    model.set_material(material);

    let mut plane = Object::plane();
    let checkerboard =
        Pattern::checkers(Color::new(0.15, 0.15, 0.15), Color::new(0.85, 0.85, 0.85));
    let plane_material = Material::builder()
        .pattern(checkerboard)
        .ambient(0.1)
        .diffuse(0.6)
        .specular(0.4)
        .reflectivity(0.4)
        .build();
    plane.set_material(plane_material);

    let light = Light::area(
        Point::new(-5.0, 10.0, -5.0),
        Vector::new(4.0, 0.0, 0.0),
        Vector::new(0.0, 0.0, 4.0),
        8,
        8,
        Color::new(1.5, 1.5, 1.5),
    );

    let world = World {
        objects: vec![plane, model],
        light_source: Some(light),
    };

    let mut camera = Camera::new(1280, 720, PI / 3.0);
    camera.transform = view_transform(camera_from, camera_to, Vector::new(0.0, 1.0, 0.0));
    camera.aa_samples = aa_samples;

    let canvas = camera.render(&world);

    let jpg_path = format!("{}/{}.jpg", output_dir, model_name);
    let ppm_path = format!("{}/{}.ppm", output_dir, model_name);
    canvas.to_jpeg(&jpg_path).expect("Failed to save JPG");
    canvas.to_ppm(&ppm_path).expect("Failed to save PPM");
    println!("  ✓ Saved to {} and {}", jpg_path, ppm_path);
}
