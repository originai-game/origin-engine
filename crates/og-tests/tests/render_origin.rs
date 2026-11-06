//! The render origin (`og_render::RenderScene::rebase`, `origin.rs`): a
//! world far from its zero is drawn from a point near the camera, so the
//! GPU's f32 arithmetic works on small numbers.
//!
//! A scene placed 1000 km out and drawn as it is (every position in the
//! world, as before render origins) loses its precision: the view matrix,
//! the vertices and the shadow cascades are rounded to 6 cm there, and the
//! frame breaks up. The same scene moved to a render origin at its place
//! draws the frame of the scene at the world's zero.
//!
//! A move of the render origin while the frame keeps history (TAA,
//! world-space particles, trails) is invisible: the frames match the ones
//! drawn without the move.
//!
//! Ways this could fail: `rebase` misses a kind of position (the lights,
//! the fog's height, the decals, the particle systems: the frame differs
//! from the one at zero), the shadow cascades snap to a texel grid through
//! the render origin instead of the world's zero, or the renderer keeps
//! the history of the old origin (the particles jump 8 km away and vanish,
//! the trails streak, TAA reprojects from the wrong place).

use glam::Vec3;
use og_render::{RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage};
use og_tests::compare::{Comparison, Tolerance, compare};
use og_tests::{atmo_scenes, fx_scenes, scenes, shadow_scenes};

/// A scene's builder, and what moves it to a frame.
type Build = fn() -> RenderScene;
type Step = fn(&mut RenderScene, u32);

const W: u32 = 480;
const H: u32 = 270;

fn renderer() -> Renderer {
    Renderer::new_blocking(&RenderDesc {
        width: W,
        height: H,
        ..scenes::base_desc()
    })
    .expect("renderer")
}

fn draw(r: &mut Renderer, scene: &RenderScene) -> RgbaImage {
    r.render(scene, &RenderTarget::Offscreen).expect("frame");
    r.read_offscreen().expect("capture")
}

fn diff(a: &RgbaImage, b: &RgbaImage) -> Comparison {
    compare(&a.pixels, &b.pixels, W, H, &Tolerance::default()).expect("same size")
}

/// What the same frame may differ by: the GPU's rounding (a scene 8 km out,
/// 1 mm, against one moved to its origin), and the shadow cascades' texels,
/// which lie on the grid through the world's zero, a part of a texel from
/// the grid through the render origin (the sun's shadow edges move by up
/// to a texel, one pixel in 600).
fn same(d: &Comparison) -> Result<(), String> {
    within(d, SHADOWED)
}

/// The rmse `same` allows: the shadow edges' rounding (the crypt measured
/// 0.0116 on metal).
const SHADOWED: f64 = 0.02;
/// The rmse a scene without the sun's shadow edges allows: the GPU's
/// rounding alone (metal measured 0.00046 for the fog, 0.00063 for the
/// decals; drawn 1000 km out in the world's coordinates they break up
/// to 0.0168 and 0.074).
const UNSHADOWED: f64 = 0.005;

fn within(d: &Comparison, rmse: f64) -> Result<(), String> {
    d.check(&Tolerance {
        rmse,
        ..Tolerance::default()
    })
}

/// Moves every position of `scene` by `by` in the world: the scene as a
/// world placed there and drawn without a render origin.
fn place(scene: &mut RenderScene, by: Vec3) {
    scene.rebase(-by);
    scene.origin = Vec3::ZERO;
}

/// 1000 km out along x and 600 km along z: a multiple of the render
/// origin's grid, where f32 positions are 1/16 m apart.
const FAR: Vec3 = Vec3::new(999_424.0, 0.0, 598_016.0);

#[test]
#[ignore = "requires GPU"]
fn a_scene_a_thousand_kilometres_out_draws_as_at_the_world_zero() {
    // Each with the rmse its frame may differ by: the crypt's sun shadows
    // round, the others' frames only by the GPU's arithmetic.
    let builds: [(&str, Build, f64); 3] = [
        ("crypt", shadow_scenes::crypt_scene, SHADOWED),
        ("decals", fx_scenes::decals_scene, UNSHADOWED),
        ("fog", atmo_scenes::fog_scene, UNSHADOWED),
    ];
    for (name, build, rmse) in builds {
        // The scene with its positions on the f32 spacing 1000 km out (each
        // moved there and back: at most 3 cm), so placing it there is exact.
        let on_grid = || {
            let mut s = build();
            place(&mut s, FAR);
            place(&mut s, -FAR);
            s
        };
        let mut r = renderer();
        let home = draw(&mut r, &on_grid());
        // Placed far out and drawn as it is.
        let mut far = on_grid();
        place(&mut far, FAR);
        let mut r = renderer();
        let as_before = draw(&mut r, &far);
        // The same world drawn from a render origin at its place.
        far.rebase(FAR);
        let mut r = renderer();
        let from_origin = draw(&mut r, &far);
        let before = diff(&as_before, &home);
        let now = diff(&from_origin, &home);
        if let Err(e) = within(&now, rmse) {
            panic!("{name}: drawn from the render origin, the frame is the one at zero: {e}");
        }
        assert!(
            within(&before, rmse).is_err(),
            "{name}: drawn in the world's coordinates 1000 km out it breaks up \
             (rmse {:.5}), which is what this test guards",
            before.rmse
        );
    }
}

/// The render origin's grid in og-runtime (`extract::RENDER_GRID`).
const GRID: Vec3 = Vec3::new(8192.0, 0.0, 0.0);

/// The trails scene's wisps and their orbs moved to `frame`, in the
/// coordinates the scene is drawn in (its world sits at GRID).
fn wisps(scene: &mut RenderScene, frame: u32) {
    fx_scenes::move_wisps(scene, frame);
    let by = GRID - scene.origin;
    for i in 0..scene.trails.len() {
        scene.trails[i].position += by;
        scene.instances[1 + i].transform.w_axis += by.extend(0.0);
    }
}

/// Draws `frames` frames of a scene whose world sits at GRID, stepped by
/// `step`; from frame `rebase_at` on (if any) the frame is drawn from a
/// render origin at GRID instead of the world's zero.
fn run(build: Build, step: Step, frames: u32, rebase_at: Option<u32>) -> Vec<RgbaImage> {
    let mut scene = build();
    place(&mut scene, GRID);
    let mut r = renderer();
    let mut out = Vec::new();
    for f in 0..frames {
        if Some(f) == rebase_at {
            scene.rebase(GRID);
        }
        step(&mut scene, f);
        out.push(draw(&mut r, &scene));
    }
    out
}

#[test]
#[ignore = "requires GPU"]
fn moving_the_render_origin_keeps_the_history_in_place() {
    let cases: [(&str, Build, Step); 2] = [
        (
            "particles",
            fx_scenes::particles_scene,
            fx_scenes::tick_update,
        ),
        ("trails", fx_scenes::trails_scene, wisps),
    ];
    for (name, build, step) in cases {
        let still = run(build, step, 24, None);
        let moved = run(build, step, 24, Some(12));
        for f in 12..24 {
            let d = diff(&moved[f as usize], &still[f as usize]);
            if let Err(e) = same(&d) {
                panic!(
                    "{name} frame {f}: the render origin moved at frame 12 and the \
                     frame is the one drawn without the move: {e}"
                );
            }
        }
    }
}
