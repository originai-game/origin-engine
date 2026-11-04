//! A look that switches the stylisation (passes added or removed, the pixel
//! grid, the material style: `Stylise::chain_bits`) resets the TAA history
//! as a camera cut does, so the first frame of the new look holds no ghost
//! of the old one; a change of a setting alone keeps it.
//!
//! Ways this could fail: the renderer blends the new look's first frame
//! with the old look's history (`FrameStats::taa_history` true there, and
//! the frame differs from one drawn after `reset_history`); it resets on a
//! setting that does not switch a pass (a posterize level), losing the
//! antialiasing of every frame of an animated look. Broken once by leaving
//! the reset out: the look's first frame reports a history blend and
//! differs from the cut's frame.
//!
//! Under a transparent surface (order-independent transparency, which
//! writes no depth or motion) TAA trusts its history by the revealage, the
//! reactive mask: a pane that appears keeps far less of the frames before
//! it than the plain history weight would (taa.wgsl, binding 8).
//!
//! A frame with `RenderScene::camera_cut` (render interpolation's
//! og.render.snapCamera, or a camera jump past snapDistance) is a cut too:
//! it blends no history and is the frame `reset_history` draws, byte for
//! byte, and the frame after it blends again. The code before this blended
//! the cut's frame with the previous shot's history.

use glam::Vec3;
use og_render::{
    Antialias, DitherPattern, Quantize, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage,
};
use og_tests::scenes;

fn desc() -> RenderDesc {
    RenderDesc {
        width: 320,
        height: 180,
        antialias: Antialias::Taa,
        ..scenes::base_desc()
    }
}

fn quantize(posterize: u32) -> Quantize {
    Quantize {
        posterize,
        palette: None,
        dither: DitherPattern::Bayer4,
        dither_strength: 1.0,
        dither_scale: 1,
    }
}

/// Draws `scene`, returning the frame and whether TAA blended history.
fn draw(renderer: &mut Renderer, scene: &RenderScene) -> (RgbaImage, bool) {
    let stats = renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("frame");
    (
        renderer.read_offscreen().expect("capture"),
        stats.taa_history,
    )
}

#[test]
fn a_look_that_switches_the_stylisation_resets_the_taa_history() {
    let mut scene = scenes::thin_geometry();
    let mut look = Renderer::new_blocking(&desc()).expect("renderer");
    if look.info().antialias != Antialias::Taa {
        println!("skipped: TAA is not available on this adapter");
        return;
    }
    let mut cut = Renderer::new_blocking(&desc()).expect("renderer");
    // Three frames of the plain look: history from the second on.
    for f in 0..3 {
        let (a, blend_a) = draw(&mut look, &scene);
        let (b, blend_b) = draw(&mut cut, &scene);
        assert_eq!(blend_a, f > 0, "frame {f}");
        assert_eq!(blend_b, f > 0, "frame {f}");
        assert!(a.pixels == b.pixels, "frame {f}: the two renderers agree");
    }
    // The new look: posterized and toon-shaded. `cut` is told it is a cut.
    scene.stylise.quantize = Some(quantize(6));
    scene.stylise.material.toon = true;
    cut.reset_history();
    let (a, blend_a) = draw(&mut look, &scene);
    let (b, blend_b) = draw(&mut cut, &scene);
    assert!(!blend_a, "the new look's first frame blends no history");
    assert!(!blend_b);
    assert!(
        a.pixels == b.pixels,
        "the new look's first frame is the cut's, byte for byte"
    );
    // The next frame blends again, and a setting alone keeps the history.
    assert!(draw(&mut look, &scene).1);
    scene.stylise.quantize = Some(quantize(8));
    assert!(draw(&mut look, &scene).1, "a posterize level is no cut");
    // Back to the plain look: a cut again.
    scene.stylise = Default::default();
    assert!(!draw(&mut look, &scene).1, "the stylisation switched off");
}

#[test]
fn a_camera_cut_resets_the_taa_history() {
    let mut scene = scenes::thin_geometry();
    let mut flagged = Renderer::new_blocking(&desc()).expect("renderer");
    if flagged.info().antialias != Antialias::Taa {
        println!("skipped: TAA is not available on this adapter");
        return;
    }
    let mut reset = Renderer::new_blocking(&desc()).expect("renderer");
    for f in 0..3 {
        assert_eq!(draw(&mut flagged, &scene).1, f > 0, "frame {f}");
        assert_eq!(draw(&mut reset, &scene).1, f > 0, "frame {f}");
    }
    // The next shot: the camera somewhere else. `flagged` is told through
    // the scene, `reset` through the call.
    let shift = Vec3::new(0.6, 0.2, 0.0);
    scene.camera.position += shift;
    scene.camera.target += shift;
    scene.camera_cut = true;
    let (a, blend_a) = draw(&mut flagged, &scene);
    scene.camera_cut = false;
    reset.reset_history();
    let (b, blend_b) = draw(&mut reset, &scene);
    assert!(
        !blend_a,
        "the cut's frame blends no history of the shot before"
    );
    assert!(!blend_b);
    assert!(
        a.pixels == b.pixels,
        "the cut's frame is reset_history's, byte for byte"
    );
    assert!(
        draw(&mut flagged, &scene).1,
        "the frame after the cut blends"
    );
}

#[test]
fn smaa_t2x_keeps_only_two_frames_and_resets_on_cuts() {
    let desc = RenderDesc {
        antialias: Antialias::SmaaT2x,
        ..desc()
    };
    let mut a = Renderer::new_blocking(&desc).expect("renderer");
    let mut b = Renderer::new_blocking(&desc).expect("renderer");
    let mut scene = scenes::thin_geometry();
    let camera = scene.camera;
    let shift = Vec3::new(0.6, 0.2, 0.0);
    draw(&mut a, &scene);
    scene.camera.position += shift;
    scene.camera.target += shift;
    draw(&mut b, &scene);
    scene.camera = camera;
    draw(&mut a, &scene);
    draw(&mut b, &scene);
    let first = draw(&mut a, &scene).0;
    let second = draw(&mut b, &scene).0;
    assert!(
        first.pixels == second.pixels,
        "history older than two frames must not survive"
    );
    // The scene flag and explicit reset agree at the same jitter phase.
    scene.camera.position += shift;
    scene.camera.target += shift;
    scene.camera_cut = true;
    let cut = draw(&mut a, &scene).0;
    scene.camera_cut = false;
    b.reset_history();
    assert!(cut.pixels == draw(&mut b, &scene).0.pixels);
    // Render scale uses the spatial upscale, not a second temporal AA.
    a.set_render_scale(0.5);
    b.set_render_scale(0.5);
    b.reset_history();
    assert!(draw(&mut a, &scene).0.pixels == draw(&mut b, &scene).0.pixels);
    let names = a.frame_graph().unwrap().ordered_names();
    assert!(names.contains(&"smaa-resolve") && names.contains(&"upscale"));
    assert!(!names.contains(&"taa"));
    // A float destination alone is not HDR: history must still match the
    // SDR chain's blend pipeline, including after a render-size change.
    let mut float = Renderer::new_blocking(&RenderDesc {
        output_format: Some(og_render::wgpu::TextureFormat::Rgba16Float),
        ..desc
    })
    .expect("float renderer");
    assert!(!float.info().hdr);
    for scale in [1.0, 0.5] {
        float.set_render_scale(scale);
        float
            .render(&scene, &RenderTarget::Offscreen)
            .expect("float destination with SDR history");
        let graph = float.frame_graph().unwrap();
        assert_eq!(
            graph.physical()[graph.slot("ldr.0").unwrap()].format,
            og_render::wgpu::TextureFormat::Rgba8UnormSrgb
        );
    }
}

/// A static opaque stripe pattern (a triangle wave across x, about 1.5
/// pixels a column, so TAA's neighbourhood box is wide) seen from above
/// orthographically at 10 pixels a unit, and with `pane` a grey
/// half-transparent pane over its middle.
fn striped(pane: bool) -> RenderScene {
    use og_render::{
        Background, Camera, MeshData, Projection, StandardMaterial, Tonemap, Vertex, shapes,
    };
    let mut scene = RenderScene::new();
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.post.tonemap = Tonemap::None;
    scene.light.cast_shadows = false;
    scene.camera = Camera {
        position: Vec3::new(0.0, 10.0, 0.0),
        target: Vec3::ZERO,
        up: Vec3::NEG_Z,
        projection: Projection::Orthographic { height: 18.0 },
        ..Default::default()
    };
    let columns = 270;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for i in 0..=columns {
        let x = -20.25 + 0.15 * i as f32;
        let level = if i % 2 == 0 { 0.1 } else { 0.9 };
        for z in [-10.0, 10.0] {
            vertices.push(Vertex {
                tangent: [0.0; 4],
                position: [x, 0.0, z],
                normal: [0.0, 1.0, 0.0],
                uv: [0.0; 2],
                color: [level, level, level, 1.0],
            });
        }
        if i < columns {
            let k = 2 * i as u32;
            indices.extend([k, k + 1, k + 3, k, k + 3, k + 2]);
        }
    }
    let stripes = scene.add_mesh(MeshData { vertices, indices });
    let white = scene.add_standard_material(StandardMaterial {
        unlit: 1,
        ..Default::default()
    });
    scene.add_instance(stripes, white, glam::Mat4::IDENTITY);
    if pane {
        let plane = scene.add_mesh(shapes::plane(6.0, 1.0));
        let grey = scene.add_standard_material(StandardMaterial {
            color: Vec3::splat(0.5),
            opacity: 0.5,
            transparent: 1,
            unlit: 1,
            ..Default::default()
        });
        scene.add_instance(plane, grey, glam::Mat4::from_translation(Vec3::Y));
    }
    scene
}

/// The mean absolute difference of two frames over the pane's middle (40
/// pixels square at the centre), in 8-bit codes per channel.
fn pane_difference(a: &RgbaImage, b: &RgbaImage) -> f64 {
    let (cx, cy) = (a.width / 2, a.height / 2);
    let mut sum = 0u64;
    let mut n = 0u64;
    for y in cy - 20..cy + 20 {
        for x in cx - 20..cx + 20 {
            let i = ((y * a.width + x) * 4) as usize;
            for c in 0..3 {
                sum += u64::from(a.pixels[i + c].abs_diff(b.pixels[i + c]));
                n += 1;
            }
        }
    }
    sum as f64 / n as f64
}

/// A pane that appears over a converged history: the frame it appears in is
/// compared with a renderer that drew the pane from the start, on the same
/// jitter. What is left of the pane-less history is the plain weight's
/// 1 - 0.1 = 0.9 of the frame before's difference without the mask, and
/// (1 - 0.1) * revealage 0.5 = 0.45 with it, less where the neighbourhood
/// clamp takes more: the mask must leave at most 0.65 of it.
#[test]
fn the_oit_revealage_lowers_the_taa_history_under_a_pane() {
    let mut appears = Renderer::new_blocking(&desc()).expect("renderer");
    if appears.info().antialias != Antialias::Taa {
        println!("skipped: TAA is not available on this adapter");
        return;
    }
    let mut always = Renderer::new_blocking(&desc()).expect("renderer");
    let (bare, with) = (striped(false), striped(true));
    let mut before = 0.0;
    for f in 0..16 {
        let (a, _) = draw(&mut appears, &bare);
        let (b, _) = draw(&mut always, &with);
        before = pane_difference(&a, &b);
        assert!(f == 0 || before > 0.0, "frame {f}: the pane shows");
    }
    let (a, history) = draw(&mut appears, &with);
    let (b, _) = draw(&mut always, &with);
    assert!(history, "the pane's first frame blends the history");
    let after = pane_difference(&a, &b);
    println!("pane difference: {before:.2} codes before it appears, {after:.2} as it does");
    assert!(before > 8.0, "the pane changes the frame: {before:.2}");
    assert!(
        after <= 0.65 * before,
        "the history under the pane kept {:.2} of the difference",
        after / before
    );
}
