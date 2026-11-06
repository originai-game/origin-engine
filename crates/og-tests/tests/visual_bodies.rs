//! Visual bodies (og-render's `VisualBody`; og-physics' SIMULATION_VISUAL):
//! cloth and soft volumes the renderer simulates on the GPU from the rest
//! shapes the host hands in, with nothing read back. The courtyard matches
//! its reference for this tier; against the same scene drawn at rest (what
//! a device without compute draws, and what the old renderer drew of a
//! surface it did not simulate), the banners stream in the wind and the
//! sheet has fallen onto the ball while the host's meshes never moved.

use og_render::{RenderScene, RenderTarget, Renderer, RgbaImage};
use og_tests::{gfx, scenes, visual_scenes};
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx")
}

/// The courtyard after its ticks, one frame each.
fn frames(scene: &mut RenderScene, ticks: u32) -> RgbaImage {
    let mut desc = scenes::base_desc();
    desc.antialias = og_render::Antialias::Taa;
    let mut r = Renderer::new_blocking(&desc).expect("renderer");
    for frame in 0..ticks {
        visual_scenes::next_tick(scene, frame);
        r.render(scene, &RenderTarget::Offscreen).expect("render");
    }
    r.read_offscreen().expect("capture")
}

/// Mean absolute difference of two frames over a rectangle (0..255).
fn differs(a: &RgbaImage, b: &RgbaImage, (x0, y0, x1, y1): (u32, u32, u32, u32)) -> f32 {
    let mut sum = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            let i = ((y * a.width + x) * 4) as usize;
            for k in 0..3 {
                sum += (a.pixels[i + k] as f32 - b.pixels[i + k] as f32).abs();
            }
        }
    }
    sum / ((x1 - x0) * (y1 - y0) * 3) as f32
}

/// A visual fluid is simulated on the GPU from its spawn box: after its
/// ticks the block has fallen from the basin's left end and the water has
/// reached the far side, where the still block (what a device without
/// compute draws, and what the old renderer drew of a fluid with no
/// particles: nothing) leaves the basin's floor bare.
#[test]
fn a_visual_fluid_flows_on_the_gpu_from_its_spawn_box() {
    let mut visual = visual_scenes::visual_fluid_scene(true);
    let flowed = frames(&mut visual, visual_scenes::FLUID_TICKS);
    let mut still = visual_scenes::visual_fluid_scene(false);
    let at_rest = frames(&mut still, 1);
    if std::env::var_os("OG_VISUAL_FRAMES").is_some() {
        let dir = out_dir();
        gfx::save_png(&dir.join("visual-fluid.png"), &flowed).unwrap();
        gfx::save_png(&dir.join("visual-fluid-rest.png"), &at_rest).unwrap();
    }
    for (what, rect, least) in [
        ("the block's place", (330, 230, 580, 430), 10.0),
        ("the far side", (700, 300, 880, 470), 5.0),
    ] {
        let d = differs(&flowed, &at_rest, rect);
        assert!(d > least, "{what} drew as at rest: mean difference {d:.2}");
    }
    let d = differs(&flowed, &at_rest, (0, 0, 200, 100));
    assert!(d < 1.0, "the sky changed: {d:.2}");
}

#[test]
fn visual_bodies_move_on_the_gpu_while_the_host_meshes_stay_at_rest() {
    let mut visual = visual_scenes::visual_bodies_scene(true);
    // The renderer reads the scene (`&RenderScene`): its meshes stay the
    // rest shapes the host gave.
    let simulated = frames(&mut visual, visual_scenes::TICKS);
    let mut still = visual_scenes::visual_bodies_scene(false);
    let at_rest = frames(&mut still, visual_scenes::TICKS);
    if std::env::var_os("OG_VISUAL_FRAMES").is_some() {
        let dir = out_dir();
        gfx::save_png(&dir.join("visual-bodies.png"), &simulated).unwrap();
        gfx::save_png(&dir.join("visual-bodies-rest.png"), &at_rest).unwrap();
    }
    // The banners (left), the sheet over the ball (lower left) and the
    // jelly and balloon (right) all left their rest shapes.
    for (what, rect) in [
        ("the banners", (150, 120, 620, 420)),
        ("the sheet", (300, 430, 620, 640)),
        ("the jelly", (780, 330, 900, 440)),
    ] {
        let d = differs(&simulated, &at_rest, rect);
        assert!(d > 4.0, "{what} drew as at rest: mean difference {d:.2}");
    }
}
