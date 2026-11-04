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
