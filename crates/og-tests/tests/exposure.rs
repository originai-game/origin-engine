//! Temporal exposure adaptation (`Exposure::Auto` with adapt speeds): the
//! exposure walks towards the metered one at its speeds in EV per second of
//! the scene's ticks, so the same ticks give the same frames in two runs and
//! after a renderer state restore, and it settles on the metered exposure.

use og_render::{Exposure, RenderScene, RenderTarget, Renderer, RendererState, RgbaImage};
use og_tests::scenes;

/// The dusk scene lit `brightness` times, adapting at 1 EV/s up, 2 down.
fn scene(brightness: f32) -> RenderScene {
    let mut s = scenes::dim_scene(1.0);
    light(&mut s, brightness);
    s.camera.exposure = Exposure::Auto {
        compensation: 0.0,
        min_ev: -10.0,
        max_ev: 10.0,
        adapt_speed_up: 1.0,
        adapt_speed_down: 2.0,
    };
    s
}

fn light(s: &mut RenderScene, brightness: f32) {
    s.light.intensity = 0.15 * brightness;
    s.environment.intensity = 0.03 * brightness;
    s.environment.background =
        og_render::Background::Color(glam::Vec3::new(0.01, 0.012, 0.02) * brightness);
}

/// The scene is 16x bright before tick 30, then dim (the view darkens by 4 EV).
fn brightness_at(tick: u64) -> f32 {
    if tick < 30 { 16.0 } else { 1.0 }
}

/// Renders ticks `from..=to` (60 per second) and reads back the last.
fn run(r: &mut Renderer, s: &mut RenderScene, from: u64, to: u64) -> RgbaImage {
    for tick in from..=to {
        light(s, brightness_at(tick));
        s.tick = tick;
        s.tick_rate = 60.0;
        r.render(s, &RenderTarget::Offscreen).expect("render");
    }
    r.read_offscreen().expect("capture")
}

fn mean_luma(img: &RgbaImage) -> f32 {
    let px = img.pixels.as_chunks::<4>().0;
    px.iter()
        .map(|p| 0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32)
        .sum::<f32>()
        / px.len() as f32
}

fn renderer() -> Renderer {
    Renderer::new_blocking(&scenes::base_desc()).expect("renderer")
}

#[test]
fn adaptation_converges_identically_and_survives_a_restore() {
    // Instant metering of the dim scene: where the adaptation must land.
    let mut instant = scene(1.0);
    instant.camera.exposure = Exposure::auto();
    let mut r = renderer();
    let settled = mean_luma(&run(&mut r, &mut instant, 30, 30));

    // Two runs of the same ticks draw the same frames.
    let (mut a, mut b) = (renderer(), renderer());
    let (mut sa, mut sb) = (scene(16.0), scene(16.0));
    let bright_a = run(&mut a, &mut sa, 0, 29);
    let bright_b = run(&mut b, &mut sb, 0, 29);
    assert_eq!(
        bright_a.pixels, bright_b.pixels,
        "tick 29 differs between runs"
    );
    let just_a = run(&mut a, &mut sa, 30, 31);
    let just_b = run(&mut b, &mut sb, 30, 31);
    assert_eq!(just_a.pixels, just_b.pixels, "tick 31 differs between runs");
    let saved = a.save_state();
    assert_ne!(saved, RendererState::default(), "a save is named");
    let mid_a = run(&mut a, &mut sa, 32, 120);
    let mid_b = run(&mut b, &mut sb, 32, 120);
    assert_eq!(mid_a.pixels, mid_b.pixels, "tick 120 differs between runs");

    // Just after the view darkened the exposure still suits the bright
    // scene: the frame is far darker than metered; it brightens over time.
    let (just, mid) = (mean_luma(&just_a), mean_luma(&mid_a));
    println!("mean luma: instant {settled:.1}, tick 31 {just:.1}, tick 120 {mid:.1}");
    assert!(
        just < settled * 0.25,
        "tick 31: {just:.1} against {settled:.1}"
    );
    assert!(
        mid > just * 1.8 && mid < settled * 0.9,
        "tick 120: {mid:.1}"
    );

    // 4 EV at 1 EV/s: settled after four seconds, on the metered exposure.
    let late_a = run(&mut a, &mut sa, 121, 330);
    let late_b = run(&mut b, &mut sb, 121, 330);
    assert_eq!(
        late_a.pixels, late_b.pixels,
        "tick 330 differs between runs"
    );
    let late = mean_luma(&late_a);
    assert!(
        (late / settled - 1.0).abs() < 0.01,
        "settled at {late:.2} against the metered {settled:.2}"
    );

    // Restored to tick 31, the same ticks draw the same frames again.
    a.restore_state(saved);
    let again = run(&mut a, &mut sa, 32, 120);
    assert_eq!(
        again.pixels, mid_a.pixels,
        "tick 120 after the restore differs"
    );

    // A state no longer kept starts afresh at the metered exposure.
    a.restore_state(RendererState::default());
    let fresh = mean_luma(&run(&mut a, &mut sa, 40, 40));
    assert!(
        (fresh / settled - 1.0).abs() < 0.01,
        "a fresh start meters at once: {fresh:.2} against {settled:.2}"
    );
}

/// Speeds of 0 are the instant metering of before: the same frame bit for bit.
#[test]
fn zero_speeds_meter_the_frame_at_once() {
    let mut s = scene(1.0);
    s.camera.exposure = Exposure::Auto {
        compensation: 0.0,
        min_ev: -10.0,
        max_ev: 10.0,
        adapt_speed_up: 0.0,
        adapt_speed_down: 0.0,
    };
    let zero = run(&mut renderer(), &mut s, 30, 31);
    let mut t = scene(1.0);
    t.camera.exposure = Exposure::auto();
    let auto = run(&mut renderer(), &mut t, 30, 31);
    assert_eq!(zero.pixels, auto.pixels);
}
