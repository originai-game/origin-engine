//! The moon disc (phase 2, owed row 28): its reference frame, and its phase,
//! size and horizon read from frames of the night colonnade without bloom
//! (so the disc's own pixels are measured).

use glam::Vec3;
use og_render::{RenderScene, RenderTarget, Renderer, RgbaImage};
use og_tests::{gfx, p2_scenes, scenes};
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx")
}

fn frame(scene: &RenderScene) -> RgbaImage {
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("render");
    renderer.read_offscreen().expect("capture")
}

fn luma(img: &RgbaImage, x: u32, y: u32) -> f32 {
    let p = &img.pixels[((y * img.width + x) * 4) as usize..][..3];
    0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
}

fn moon(phase: f32, size: f32) -> RgbaImage {
    let mut scene = p2_scenes::moon_scene(phase, size);
    scene.post.bloom = None;
    frame(&scene)
}

/// The moon's disc on screen: its centre and radius in pixels, from the
/// colonnade camera (the scene's) and the moon's direction.
fn disc(img: &RgbaImage, size_degrees: f32) -> (u32, u32, u32) {
    let cam = p2_scenes::moon_scene(0.5, size_degrees).camera;
    let og_render::Projection::Perspective { fov_y_degrees } = cam.projection else {
        panic!("a perspective camera");
    };
    let view = glam::camera::rh::view::look_at_mat4(cam.position, cam.target, cam.up);
    let to_pixels = |d: Vec3| {
        let v = view.transform_vector3(d);
        let f = 1.0 / (fov_y_degrees.to_radians() * 0.5).tan();
        let aspect = img.width as f32 / img.height as f32;
        let ndc = glam::Vec2::new(v.x * f / aspect / -v.z, v.y * f / -v.z);
        glam::Vec2::new(
            (ndc.x * 0.5 + 0.5) * img.width as f32,
            (0.5 - ndc.y * 0.5) * img.height as f32,
        )
    };
    let m = p2_scenes::moon_direction();
    let centre = to_pixels(m);
    let side = m.cross(Vec3::Y).normalize();
    let edge = to_pixels(
        (m * size_degrees.to_radians().cos() + side * size_degrees.to_radians().sin()).normalize(),
    );
    let r = (edge - centre).length();
    assert!(r > 10.0, "the disc is {r} pixels across");
    (centre.x as u32, centre.y as u32, r as u32)
}

/// Mean luma over a rectangle.
fn mean(img: &RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> f32 {
    let mut sum = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            sum += luma(img, x, y);
        }
    }
    sum / ((x1 - x0) * (y1 - y0)) as f32
}

#[test]
fn atmosphere_moon_matches_its_reference() {
    match gfx::check("atmosphere-moon", &out_dir()) {
        Ok(r) => println!("{r}"),
        Err(e) => panic!("{e}"),
    }
}

/// The phase lights the disc: full is bright across, new is the night sky
/// with no star showing through, the first quarter lights the right half
/// and the last quarter the left.
#[test]
fn the_phase_lights_the_disc() {
    let full = moon(0.5, 2.4);
    let (cx, cy, radius) = disc(&full, 2.4);
    let inner = radius * 7 / 10;
    let halves = |img: &RgbaImage| {
        (
            mean(
                img,
                cx - inner,
                cy - inner / 2,
                cx - inner / 4,
                cy + inner / 2,
            ),
            mean(
                img,
                cx + inner / 4,
                cy - inner / 2,
                cx + inner,
                cy + inner / 2,
            ),
        )
    };
    let (fl, fr) = halves(&full);
    println!("full: left {fl:.0} right {fr:.0}");
    assert!(fl > 150.0 && fr > 150.0, "the full moon is lit across");

    let new = moon(0.0, 2.4);
    let none = moon(0.5, 0.0);
    let (nl, nr) = halves(&new);
    let (sl, sr) = halves(&none);
    println!("new: left {nl:.0} right {nr:.0}; no moon: {sl:.0} {sr:.0}");
    // Dark but for the earthshine: within a few levels of the sky, far below full.
    assert!(
        nl < sl + 25.0 && nr < sr + 25.0 && nl < 0.25 * fl && nr < 0.25 * fr,
        "the new moon is dark"
    );
    let brightest_in_disc = (cy - inner..=cy + inner)
        .flat_map(|y| (cx - inner..=cx + inner).map(move |x| (x, y)))
        .map(|(x, y)| luma(&new, x, y))
        .fold(0.0f32, f32::max);
    assert!(
        brightest_in_disc < 60.0,
        "no star shows through the new moon ({brightest_in_disc:.0})"
    );

    let (ql, qr) = halves(&moon(0.25, 2.4));
    println!("first quarter: left {ql:.0} right {qr:.0}");
    assert!(qr > ql + 80.0, "the first quarter lights the right half");
    let (ll, lr) = halves(&moon(0.75, 2.4));
    println!("last quarter: left {ll:.0} right {lr:.0}");
    assert!(ll > lr + 80.0, "the last quarter lights the left half");
}

/// The disc's area follows its size squared.
#[test]
fn the_size_sets_the_disc() {
    // Bright pixels within twice the big disc's radius (the stars are few and far apart).
    let lit = |img: &RgbaImage| {
        let (cx, cy, r) = disc(img, 2.4);
        let r = 2 * r;
        (cy - r..cy + r)
            .flat_map(|y| (cx - r..cx + r).map(move |x| (x, y)))
            .filter(|&(x, y)| luma(img, x, y) > 160.0)
            .count() as f32
    };
    let big = lit(&moon(0.5, 2.4));
    let small = lit(&moon(0.5, 1.2));
    println!("lit pixels: {big} at 2.4 degrees, {small} at 1.2");
    assert!(small > 50.0);
    let ratio = big / small;
    assert!((3.2..4.8).contains(&ratio), "area ratio {ratio:.2}");
}

/// Below the horizon the moon draws nothing: the frame is the moonless one.
#[test]
fn the_moon_sets_below_the_horizon() {
    let mut scene = p2_scenes::moon_scene(0.5, 2.4);
    if let Some(look) = scene.environment.sky.look.as_mut() {
        look.moon_direction = (p2_scenes::moon_direction() * Vec3::new(1.0, -1.0, 1.0)).normalize();
    }
    let set = frame(&scene);
    if let Some(look) = scene.environment.sky.look.as_mut() {
        look.moon_size = 0.0;
    }
    let without = frame(&scene);
    let differ: Vec<usize> = (0..set.pixels.len() / 4)
        .filter(|&i| set.pixels[i * 4..i * 4 + 3] != without.pixels[i * 4..i * 4 + 3])
        .collect();
    assert!(
        differ.is_empty(),
        "a set moon draws nothing, but {} pixels differ (the first at {:?})",
        differ.len(),
        differ
            .first()
            .map(|i| (i % set.width as usize, i / set.width as usize))
    );
}
