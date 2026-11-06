//! Forward+ clustered lighting (phase 2, owed row 30): the 256-light
//! reference frame, and behaviour read from frames and stats. A cluster
//! that loses a light shows as a frame that changes when only the cluster
//! layout changes (the depth slices follow the near and far planes), or as
//! a step in the light at a tile boundary; the caps keep the most important
//! lights, which is the frame of a scene holding only those.

use glam::Vec3;
use og_render::{
    Camera, FrameStats, Light, MAX_LIGHTS, Projection, RenderDesc, RenderScene, RenderTarget,
    Renderer, RgbaImage,
};
use og_tests::compare::{Tolerance, compare};
use og_tests::{gfx, p2_scenes, scenes};
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx")
}

fn frame(desc: &RenderDesc, scene: &RenderScene) -> (RgbaImage, FrameStats) {
    let mut renderer = Renderer::new_blocking(desc).expect("renderer");
    let stats = renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("render");
    (renderer.read_offscreen().expect("capture"), stats)
}

fn max_diff(a: &RgbaImage, b: &RgbaImage) -> u8 {
    compare(
        &a.pixels,
        &b.pixels,
        a.width,
        a.height,
        &Tolerance::default(),
    )
    .expect("same size")
    .max_channel_diff
}

/// Pixels with a channel more than 2 apart.
fn changed(a: &RgbaImage, b: &RgbaImage) -> usize {
    let t = Tolerance {
        channel: 2,
        ..Tolerance::default()
    };
    compare(&a.pixels, &b.pixels, a.width, a.height, &t)
        .expect("same size")
        .outliers
}

/// The depth slices follow the camera's near and far planes, so moving them
/// moves every light into other clusters while the picture stays the same:
/// a light a slice lost would change the cluster's pixels. Only a few
/// pixels where geometry meets the ground may change, with the depth
/// buffer's precision. Perspective and orthographic.
#[test]
fn moving_the_depth_slices_changes_nothing() {
    let desc = scenes::base_desc();
    let mut scene = p2_scenes::many_lights(256);
    let (base, stats) = frame(&desc, &scene);
    assert!(stats.lights > 200);
    for (near, far) in [(0.5, 120.0), (0.05, 2000.0), (2.0, 150.0)] {
        scene.camera.near = near;
        scene.camera.far = far;
        let (other, _) = frame(&desc, &scene);
        let d = changed(&base, &other);
        println!("near {near} far {far}: {d} pixels changed");
        if d > 20 {
            let dir = out_dir().join("lights");
            gfx::save_png(&dir.join("slices-base.png"), &base).expect("save");
            gfx::save_png(&dir.join(format!("slices-{near}-{far}.png")), &other).expect("save");
        }
        assert!(
            d <= 20,
            "near {near}, far {far}: the lights changed ({d} pixels)"
        );
    }
    scene.camera = Camera {
        position: Vec3::new(0.0, 40.0, 0.01),
        target: Vec3::ZERO,
        projection: Projection::Orthographic { height: 50.0 },
        near: 1.0,
        far: 80.0,
        ..scene.camera
    };
    let (ortho, _) = frame(&desc, &scene);
    scene.camera.near = 20.0;
    scene.camera.far = 300.0;
    let (ortho_moved, _) = frame(&desc, &scene);
    let d = changed(&ortho, &ortho_moved);
    println!("orthographic: {d} pixels changed");
    assert!(d <= 20, "orthographic: the lights changed ({d} pixels)");
}

/// Past MAX_LIGHTS visible lights the least important are dropped: 256
/// bright lights and 40 faint far ones draw as the 256 alone.
#[test]
fn past_max_lights_the_faint_far_ones_are_dropped() {
    let desc = scenes::base_desc();
    let mut scene = p2_scenes::many_lights(0);
    scene.lights = p2_scenes::many_lights_list(256)
        .into_iter()
        .map(|mut l| {
            // All in view.
            l.position.z = l.position.z.clamp(-24.0, 18.0);
            l
        })
        .collect();
    let bright = scene.lights.clone();
    let (alone, alone_stats) = frame(&desc, &scene);
    assert_eq!(alone_stats.lights, 256);
    // Faint lights at the far edge, interleaved with the bright ones.
    let faint = Light {
        position: Vec3::new(0.0, 0.5, -26.0),
        color: Vec3::new(0.0, 1.0, 0.0),
        intensity: 0.01,
        range: 3.0,
        ..Default::default()
    };
    scene.lights = Vec::new();
    for (i, l) in bright.iter().enumerate() {
        scene.lights.push(*l);
        if i % 6 == 0 && scene.lights.len() < 296 {
            let mut f = faint;
            f.position.x = -20.0 + (i as f32) * 0.15;
            scene.lights.push(f);
        }
    }
    let faint_count = scene.lights.len() - 256;
    assert!(faint_count >= 40);
    let (mixed, stats) = frame(&desc, &scene);
    assert_eq!(
        (stats.lights, stats.lights_dropped),
        (MAX_LIGHTS as u32, faint_count as u32)
    );
    let d = max_diff(&alone, &mixed);
    assert!(
        d <= 1,
        "the faint lights replaced bright ones (max diff {d})"
    );
}
