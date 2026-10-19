//! Forward+ clustered lighting (phase 2, owed row 30): the 256-light
//! reference frame, and behaviour read from frames and stats. A cluster
//! that loses a light shows as a frame that changes when only the cluster
//! layout changes (the depth slices follow the near and far planes), or as
//! a step in the light at a tile boundary; the caps keep the most important
//! lights, which is the frame of a scene holding only those.

use glam::{Mat4, Vec3};
use og_render::{
    Background, Camera, Environment, Exposure, FrameStats, Light, LightKind, MAX_CLUSTER_LIGHTS,
    MAX_LIGHTS, Material, Projection, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage,
    shapes,
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

fn luma(img: &RgbaImage, x: u32, y: u32) -> f32 {
    let p = &img.pixels[((y * img.width + x) * 4) as usize..][..3];
    0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
}

/// 256 lights over the night plaza, against its reference; every light is
/// drawn or culled, none dropped, and the clusters hold every reference.
#[test]
fn lights_256_matches_its_reference() {
    let report = match gfx::check("lights-256", &out_dir()) {
        Ok(r) => r,
        Err(e) => panic!("{e}"),
    };
    println!("{report}");
    let s = report.stats;
    assert_eq!(s.lights + s.lights_culled, 256);
    assert!(s.lights > 200, "{} of 256 lights in view", s.lights);
    assert_eq!(
        (s.lights_dropped, s.cluster_lights_dropped),
        (0, 0),
        "nothing over a cap"
    );
    assert!(s.cluster_lights >= s.lights);
}

/// An untextured ground lit by many small lights, seen from above.
fn plain_ground_lights(count: u32) -> RenderScene {
    let mut scene = RenderScene::new();
    let plane = scene.add_mesh(shapes::plane(60.0, 1.0));
    let white = scene.add_material(Material {
        base_color: [0.8, 0.8, 0.8, 1.0],
        roughness: 0.9,
        ..Default::default()
    });
    scene.add_instance(plane, white, Mat4::IDENTITY);
    scene.lights = p2_scenes::many_lights_list(count);
    scene.light.intensity = 0.0;
    scene.light.cast_shadows = false;
    scene.environment = Environment {
        intensity: 0.0,
        background: Background::Color(Vec3::ZERO),
        ..Default::default()
    };
    scene.camera = Camera {
        position: Vec3::new(0.0, 30.0, 12.0),
        target: Vec3::new(0.0, 0.0, -1.0),
        far: 120.0,
        exposure: Exposure::linear(1.5),
        ..Default::default()
    };
    scene
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

/// No step in the light at the tile boundaries (every 80 pixels across and
/// down): across a boundary the light changes as little as between the
/// columns or rows beside it. A light missing from the tile on one side
/// would cut off there.
#[test]
fn tile_boundaries_leave_no_step() {
    let (img, stats) = frame(&scenes::base_desc(), &plain_ground_lights(256));
    assert!(stats.lights > 150, "{} lights", stats.lights);
    let (w, h) = (img.width, img.height);
    let step_x = |x: u32| -> f32 {
        (0..h)
            .map(|y| (luma(&img, x, y) - luma(&img, x - 1, y)).abs())
            .sum::<f32>()
            / h as f32
    };
    let step_y = |y: u32| -> f32 {
        (0..w)
            .map(|x| (luma(&img, x, y) - luma(&img, x, y - 1)).abs())
            .sum::<f32>()
            / w as f32
    };
    let tile_x = w / og_render::CLUSTER_GRID[0];
    let tile_y = h / og_render::CLUSTER_GRID[1];
    for k in 1..og_render::CLUSTER_GRID[0] {
        let x = k * tile_x;
        let at = step_x(x);
        let beside = (step_x(x - 2) + step_x(x + 2) + step_x(x - 3) + step_x(x + 3)) / 4.0;
        assert!(
            at <= beside * 1.5 + 0.35,
            "a step at column {x}: {at:.3} against {beside:.3} beside it"
        );
    }
    for k in 1..og_render::CLUSTER_GRID[1] {
        let y = k * tile_y;
        let at = step_y(y);
        let beside = (step_y(y - 2) + step_y(y + 2) + step_y(y - 3) + step_y(y + 3)) / 4.0;
        assert!(
            at <= beside * 1.5 + 0.35,
            "a step at row {y}: {at:.3} against {beside:.3} beside it"
        );
    }
}

/// Over a cluster's cap the brightest lights stay: 100 lights at one place
/// (each cluster they reach holds all of them) shade exactly as the 64
/// brightest alone, and the dropped references are counted.
#[test]
fn past_the_cluster_cap_the_brightest_lights_stay() {
    let desc = scenes::base_desc();
    let mut scene = plain_ground_lights(0);
    let light = |i: u32| Light {
        position: Vec3::new(0.0, 1.0, 0.0),
        color: Vec3::new(1.0, 0.8 + (i % 3) as f32 * 0.1, 0.6),
        // Distinct intensities, interleaved in scene order.
        intensity: 0.05 + ((i * 37) % 100) as f32 * 0.004,
        range: 8.0,
        ..Default::default()
    };
    scene.lights = (0..100).map(light).collect();
    let (capped, stats) = frame(&desc, &scene);
    assert_eq!(stats.lights, 100);
    assert!(stats.cluster_lights_dropped > 0, "the cap was reached");
    let mut brightest: Vec<u32> = (0..100).collect();
    brightest.sort_by(|&a, &b| light(b).intensity.total_cmp(&light(a).intensity));
    let mut keep: Vec<u32> = brightest[..MAX_CLUSTER_LIGHTS].to_vec();
    keep.sort();
    scene.lights = keep.iter().map(|&i| light(i)).collect();
    let (top, top_stats) = frame(&desc, &scene);
    assert_eq!(top_stats.cluster_lights_dropped, 0);
    let d = max_diff(&capped, &top);
    assert!(
        d <= 1,
        "the capped frame is not the 64 brightest (max diff {d})"
    );
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

/// Lights whose reach misses the view are culled and counted; the frame is
/// the one without them, and a spot light behind the camera casts nothing.
#[test]
fn lights_outside_the_view_are_culled() {
    let desc = scenes::base_desc();
    let mut scene = plain_ground_lights(64);
    let (inside, inside_stats) = frame(&desc, &scene);
    for k in 0..10 {
        scene.lights.push(Light {
            kind: LightKind::Spot {
                direction: Vec3::NEG_Y,
                angle_degrees: 30.0,
                penumbra: 0.2,
            },
            position: Vec3::new(k as f32, 40.0, 60.0),
            intensity: 500.0,
            range: 5.0,
            ..Default::default()
        });
    }
    let (outside, stats) = frame(&desc, &scene);
    assert_eq!(
        (stats.lights, stats.lights_culled),
        (inside_stats.lights, inside_stats.lights_culled + 10)
    );
    assert_eq!(max_diff(&inside, &outside), 0);
}
