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

/// The mean x of the pixels of `hue` (red or blue) along row `y`.
fn centre(img: &RgbaImage, y: u32, red: bool) -> Option<f32> {
    let xs: Vec<f32> = (0..img.width)
        .filter(|&x| {
            let c = px(img, x, y);
            if red {
                c[0] > c[2] * 2.0 && c[0] > c[1] * 2.0
            } else {
                c[2] > c[0] * 2.0 && c[2] > c[1] * 1.5
            }
        })
        .map(|x| x as f32)
        .collect();
    (!xs.is_empty()).then(|| xs.iter().sum::<f32>() / xs.len() as f32)
}

#[test]
fn the_wind_leans_a_swaying_model_by_its_height() {
    check("wind-models");
    let (calm, _) = render(&ws::wind_scene(false, 90));
    let (windy, _) = render(&ws::wind_scene(true, 90));
    keep("wind-calm", &calm);
    keep("wind-models", &windy);
    let cam = ws::wind_scene(true, 90).camera;
    let row = |at: Vec3, h: f32| project(&cam, at + Vec3::new(0.0, h, 0.0)).1;
    // Near the top the red column is right of where it stands calm (the
    // wind blows +x), near the root it has not moved.
    let (top, root) = (
        row(ws::SWAYING_AT, ws::COLUMN * 0.9),
        row(ws::SWAYING_AT, 0.4),
    );
    let moved = |y| centre(&windy, y, true).unwrap() - centre(&calm, y, true).unwrap();
    assert!(
        moved(top) > 15.0,
        "the top leans downwind: {} px",
        moved(top)
    );
    assert!(
        moved(root).abs() < 2.0,
        "the root stays: {} px",
        moved(root)
    );
    // The blue column, still, is where it was.
    let blue = row(ws::STILL_AT, ws::COLUMN * 0.9);
    let still = centre(&windy, blue, false).unwrap() - centre(&calm, blue, false).unwrap();
    assert!(still.abs() < 0.5, "the still column moved {still} px");
    // The same tick, the same frame; another tick, another lean.
    let (again, _) = render(&ws::wind_scene(true, 90));
    assert_eq!(again.pixels, windy.pixels);
    let (later, _) = render(&ws::wind_scene(true, 130));
    assert_ne!(later.pixels, windy.pixels);
}

/// A bent tree is not a moving tree. History must contain the previous
/// deformation, and advance when the wind pauses, not just when it changes.
#[test]
fn model_wind_velocity_stops_when_the_wind_stops_changing() {
    let mut scene = ws::wind_scene(true, 90);
    let red = scene.instances[1].material;
    scene.set_shader_param(red, "wind", &[1.0, 0.0, 0.15, 0.0]);
    let blur = og_render::MotionBlur {
        shutter: 1.0,
        samples: 16,
        max_pixels: 24.0,
    };
    scene.post.motion_blur = Some(blur);
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let draw = |r: &mut Renderer, s: &RenderScene| {
        r.render(s, &RenderTarget::Offscreen).expect("render");
        r.read_offscreen().expect("capture")
    };
    draw(&mut renderer, &scene);
    let stationary = draw(&mut renderer, &scene);
    scene.post.motion_blur = None;
    let crisp = draw(&mut renderer, &scene);
    let different = |a: &RgbaImage, b: &RgbaImage| {
        a.pixels
            .chunks_exact(4)
            .zip(b.pixels.chunks_exact(4))
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
    scene.post.motion_blur = Some(blur);
    let moving = draw(&mut renderer, &scene);
    let paused = draw(&mut renderer, &scene);
    scene.post.motion_blur = None;
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
