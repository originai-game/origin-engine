//! R6's wind on placed models: a material running og-render's wind hook
//! leans its mesh downwind by its height (the root stays, the top moves),
//! and the same tick draws the same frame.
//!
//! Ways this could fail: the hook moves nothing (its parameters unread),
//! moves the root too (the whole column slides), moves against the wind,
//! moves the still column (a shared material or pipeline), or depends on
//! something besides the tick.

use super::*;
use og_tests::wind_scenes as ws;

/// A bent tree is not a moving tree. History must contain the previous
/// deformation, and advance when the wind pauses, not just when it changes.
#[test]
#[ignore = "requires GPU"]
fn model_wind_velocity_stops_when_the_wind_stops_changing() {
    let mut scene = ws::wind_scene(true, 90);
    let red = scene.instances[1].material;
    scene.set_shader_param(red, "wind", &[1.0, 0.0, 0.15, 0.0]);
    let blur = og_render::MotionBlur {
        shutter: 1.0,
        samples: 16,
        max_pixels: 24.0,
    };
    scene.look.post.motion_blur = Some(blur);
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let draw = |r: &mut Renderer, s: &RenderScene| {
        r.render(s, &RenderTarget::Offscreen).expect("render");
        r.read_offscreen().expect("capture")
    };
    draw(&mut renderer, &scene);
    let stationary = draw(&mut renderer, &scene);
    scene.look.post.motion_blur = None;
    let crisp = draw(&mut renderer, &scene);
    let different = |a: &RgbaImage, b: &RgbaImage| {
        a.pixels
            .as_chunks::<4>()
            .0
            .iter()
            .zip(b.pixels.as_chunks::<4>().0.iter())
            .filter(|(a, b)| a[..3].iter().zip(&b[..3]).any(|(a, b)| a.abs_diff(*b) > 8))
            .count()
    };
    assert!(
        different(&stationary, &crisp) < 10,
        "steady bend produced motion blur"
    );

    // Change strength without a tick: motion is between drawn frames, not
    // inferred from elapsed tick time or a hard-coded previous tick.
    scene.set_shader_param(red, "wind", &[1.0, 0.0, 0.4, 0.0]);
    scene.look.post.motion_blur = Some(blur);
    let moving = draw(&mut renderer, &scene);
    let paused = draw(&mut renderer, &scene);
    scene.look.post.motion_blur = None;
    let crisp = draw(&mut renderer, &scene);
    assert!(
        different(&moving, &crisp) > 50,
        "wind change lost its motion vectors"
    );
    assert!(
        different(&paused, &crisp) < 10,
        "paused wind retained stale velocity"
    );
}
