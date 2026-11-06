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
use og_render::{Antialias, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage};
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
    let scene = scenes::thin_geometry();
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
#[test]
#[ignore = "requires GPU"]
fn a_moving_object_takes_its_history_along_its_motion_vectors() {
    let mut scene = scenes::thin_geometry();
    let sphere = scene.instances.len() - 1;
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
        scene.instances[sphere].transform = at(f);
        scene.previous_transforms.clear();
        let plain = draw(&mut without, &scene, 1);
        scene.previous_transforms = scene.instances.iter().map(|i| i.transform).collect();
        scene.previous_transforms[sphere] = at(f.saturating_sub(1));
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
