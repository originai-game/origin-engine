//! Temporal upscaling (R2): under TAA the render scale's frame is resolved
//! to the output by TAA itself, whose history is the output's size, and
//! TAA follows what moves along the prepass's motion vectors. GPU tests:
//! macrunner runs them on Metal.
//!
//! Ways this could fail: TAA at the scene's size and a bilinear upscale
//! after it (the frame graph still has `upscale`, and the converged frame
//! is no closer to the full-scale one than a plain upscale of one frame);
//! the jitter not walking the scene's samples over every output pixel (the
//! converged frame stays blocky); the passes after TAA sized to the scene
//! (a graph error or a quarter-sized image); a moving object reprojected
//! with the camera's motion alone (it trails a ghost the motion vectors
//! remove).

use glam::{Mat4, Vec3};
use og_render::{Antialias, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage, Upscaler};
use og_tests::scenes;

fn desc(antialias: Antialias) -> RenderDesc {
    RenderDesc {
        width: 320,
        height: 180,
        antialias,
        ..scenes::base_desc()
    }
}

fn draw(renderer: &mut Renderer, scene: &RenderScene, frames: u32) -> RgbaImage {
    for _ in 0..frames.max(1) {
        renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("frame");
    }
    renderer.read_offscreen().expect("capture")
}

/// The mean absolute difference of two frames' bytes.
fn diff(a: &RgbaImage, b: &RgbaImage) -> f64 {
    assert_eq!((a.width, a.height), (b.width, b.height));
    a.pixels
        .iter()
        .zip(&b.pixels)
        .map(|(x, y)| f64::from(x.abs_diff(*y)))
        .sum::<f64>()
        / a.pixels.len() as f64
}

/// At render scale 0.5 under TAA the scene is drawn at a quarter of the
/// pixels, TAA writes the output-sized frame (no upscale pass), and after
/// the jitter's 32 phases the frame is nearer the full-scale TAA frame
/// than one frame at half scale upscaled bilinearly.
#[test]
#[ignore = "requires GPU"]
fn taau_resolves_the_half_scale_scene_to_the_output() {
    let mut scene = scenes::thin_geometry();
    // TAAU itself: the default draws FSR below render scale 1 (fsr.rs).
    scene.frame.quality.upscaler = Upscaler::Taau;
    let mut full = Renderer::new_blocking(&desc(Antialias::Taa)).expect("renderer");
    assert_eq!(
        full.info().antialias,
        Antialias::Taa,
        "GPU lacks TAA support"
    );
    let reference = draw(&mut full, &scene, 32);

    let mut taau = Renderer::new_blocking(&desc(Antialias::Taa)).expect("renderer");
    taau.set_render_scale(0.5);
    let upscaled = draw(&mut taau, &scene, 64);
    assert_eq!(taau.scene_size_drawn(), (160, 90));
    assert_eq!((upscaled.width, upscaled.height), (320, 180));
    let names = taau.frame_graph().expect("a graph").ordered_names();
    assert!(names.contains(&"taa"), "{names:?}");
    assert!(
        !names.contains(&"upscale"),
        "TAAU upscales in TAA: {names:?}"
    );

    let mut plain = Renderer::new_blocking(&desc(Antialias::Off)).expect("renderer");
    plain.set_render_scale(0.5);
    let bilinear = draw(&mut plain, &scene, 1);
    let (d_taau, d_plain) = (diff(&upscaled, &reference), diff(&bilinear, &reference));
    println!("mean byte difference from full scale: TAAU {d_taau:.2}, bilinear {d_plain:.2}");
    assert!(
        d_taau < d_plain,
        "TAAU {d_taau} is no nearer than bilinear {d_plain}"
    );
}

/// A sphere crossing a still camera's view: with its previous transforms
/// the prepass's motion vectors carry its history with it, so after a
/// dozen frames it is nearer the frame a fresh history draws of the same
/// pose than without them, where only the clamp holds back its trail.
/// Under GPU culling too, whose sorted instances' previous transforms are
/// built apart (a still instance's from its current one).
#[test]
#[ignore = "requires GPU"]
fn a_moving_object_takes_its_history_along_its_motion_vectors() {
    moving_object();
}

fn moving_object() {
    let mut scene = scenes::thin_geometry();
    // TAAU itself: the default draws FSR below render scale 1 (fsr.rs).
    scene.frame.quality.upscaler = Upscaler::Taau;
    let sphere = scene.objects.instances.len() - 1;
    let at = |f: u32| {
        Mat4::from_translation(Vec3::new(-2.0 + 0.2 * f as f32, 1.0, 1.0))
            * Mat4::from_scale(Vec3::splat(2.0))
    };
    let mut with = Renderer::new_blocking(&desc(Antialias::Taa)).expect("renderer");
    assert_eq!(
        with.info().antialias,
        Antialias::Taa,
        "GPU lacks TAA support"
    );
    let mut without = Renderer::new_blocking(&desc(Antialias::Taa)).expect("renderer");
    let frames = 12;
    let (mut a, mut b) = (None, None);
    for f in 0..frames {
        scene.objects.instances[sphere].transform = at(f);
        scene.objects.previous_transforms.clear();
        let plain = draw(&mut without, &scene, 1);
        scene.objects.previous_transforms = scene
            .objects
            .instances
            .iter()
            .map(|i| i.transform)
            .collect();
        scene.objects.previous_transforms[sphere] = at(f.saturating_sub(1));
        let moved = draw(&mut with, &scene, 1);
        (a, b) = (Some(moved), Some(plain));
    }
    let mut fresh = Renderer::new_blocking(&desc(Antialias::Taa)).expect("renderer");
    let still = draw(&mut fresh, &scene, 1);
    let (d_with, d_without) = (diff(&a.unwrap(), &still), diff(&b.unwrap(), &still));
    println!(
        "mean byte difference from a fresh frame: with motion {d_with:.2}, without {d_without:.2}"
    );
    assert!(
        d_with < d_without,
        "motion vectors {d_with}, camera only {d_without}"
    );
}

/// A bright object that vanishes over thin bright detail leaves no ghost
/// round the detail (taa.wgsl drops a history far brighter than its
/// neighbourhood): a glowing panel stands
/// behind thin glowing lines, a pixel wide, for 16 frames and is then taken
/// away. In the frame after, the panel's area is within a twentieth of the
/// panel's light of a history that never saw it. A min/max box of the 3x3
/// neighbourhood took its top from the line beside each pixel, so the
/// panel's history stayed at the line's brightness in a band round every
/// line it had stood behind (a bolt gone over rain: weather-lightning's
/// frame after its strike).
#[test]
#[ignore = "requires GPU"]
fn a_vanished_bright_object_leaves_no_ghost_round_thin_detail() {
    use og_render::{Background, Camera, Exposure, Material, shapes};
    let mut scene = RenderScene::new();
    // TAAU itself: the default draws FSR below render scale 1 (fsr.rs).
    scene.frame.quality.upscaler = Upscaler::Taau;
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.lighting.environment.intensity = 0.0;
    scene.lighting.light.intensity = 0.0;
    let glow = |scene: &mut RenderScene, e: f32| {
        scene.add_material(Material {
            base_color: [0.0, 0.0, 0.0, 1.0],
            emissive: [0.8 * e, 0.85 * e, e],
            ..Default::default()
        })
    };
    // Lines 2 cm wide 6 m out: about a pixel of the 180-pixel frame.
    let line = scene.add_mesh(shapes::cuboid(Vec3::new(0.02, 3.0, 0.02)));
    let lit = glow(&mut scene, 1.5);
    for i in 0..9 {
        let x = -2.0 + 0.5 * i as f32;
        scene.add_instance(line, lit, Mat4::from_translation(Vec3::new(x, 1.5, 0.0)));
    }
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 1.5, 6.0),
        target: Vec3::new(0.0, 1.5, 0.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    let mut fresh = Renderer::new_blocking(&desc(Antialias::Taa)).expect("renderer");
    assert_eq!(
        fresh.info().antialias,
        Antialias::Taa,
        "GPU lacks TAA support"
    );
    let without = draw(&mut fresh, &scene, 16);
    let panel = scene.add_mesh(shapes::plane_xy(5.0, 3.0));
    let bright = glow(&mut scene, 20.0);
    scene.add_instance(
        panel,
        bright,
        Mat4::from_translation(Vec3::new(0.0, 1.5, -1.0)),
    );
    let mut taa = Renderer::new_blocking(&desc(Antialias::Taa)).expect("renderer");
    let with = draw(&mut taa, &scene, 16);
    scene.objects.instances.pop();
    let after = draw(&mut taa, &scene, 1);
    // The mean light over the panel's middle, which lies between the lines.
    let luma = |img: &RgbaImage| {
        let mut s = 0.0f64;
        let (x0, x1, y0, y1) = (90u32, 230u32, 50u32, 130u32);
        for y in y0..y1 {
            for x in x0..x1 {
                let p = &img.pixels[((y * img.width + x) * 4) as usize..][..3];
                s += 0.2126 * f64::from(p[0]) + 0.7152 * f64::from(p[1]) + 0.0722 * f64::from(p[2]);
            }
        }
        s / f64::from((x1 - x0) * (y1 - y0))
    };
    let (plain, lit_panel, ghost) = (luma(&without), luma(&with), luma(&after));
    println!(
        "the panel's area: {plain:.2} without it, {lit_panel:.2} with it, {ghost:.2} the frame after"
    );
    assert!(
        lit_panel - plain > 100.0 && plain > 5.0,
        "the scene shows no lines ({plain}) or no panel ({lit_panel})"
    );
    assert!(
        ghost - plain < 0.05 * (lit_panel - plain),
        "the frame after the panel keeps {ghost} where it was, {plain} without it and {lit_panel} with it"
    );
}
