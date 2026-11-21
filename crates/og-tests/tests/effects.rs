//! The E1 effects, asserted on frames and counts: particles (determinism of
//! a 600-tick run on both paths, the CPU and GPU paths agreeing, spawn
//! counts, soft particles, depth collision, 2D emitters), trails and beams,
//! 2D lights and soft shadows, glow, sprite flash, decals, the grading stack
//! (LUT, vignette, chromatic aberration, grain, flash), camera shake,
//! ambient occlusion, depth of field, motion blur, god rays, outlines,
//! toon, dissolve, rim, and GPU skinning with blendshapes.

use glam::{Mat4, Vec2, Vec3};
use og_render::{
    AmbientOcclusion, Background, Camera, Curve, Decal, DepthOfField, DofQuality, EmitterShape,
    FrameStats, GodRays, JointPalette, LensDistortion, Light2d, Lighting2d, MAX_CLUSTER_DECALS,
    Material, MotionBlur, ParticleCollision, ParticleEmitter, ParticleRender, ParticleSimulation,
    ParticleSystem, Projection, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage,
    SkinnedInstance, Sprite, Tonemap, Trail, TrailStyle, shapes,
};
use og_tests::fx_scenes as fx;
use og_tests::harness::{frame, luma, out_dir, project, px_clamped, save};
use og_tests::{gfx, scenes};

#[path = "effects/models.rs"]
mod models;
#[path = "effects/particles.rs"]
mod particles;
#[path = "effects/post.rs"]
mod post;
#[path = "effects/quality.rs"]
mod quality;
#[path = "effects/sight.rs"]
mod sight;
#[path = "effects/sprites.rs"]
mod sprites;

fn small() -> RenderDesc {
    RenderDesc {
        width: 320,
        height: 180,
        ..scenes::base_desc()
    }
}

/// Renders `frames` frames (calling `update(scene, frame)` before each) and
/// returns the last one.
fn run(
    desc: &RenderDesc,
    scene: &mut RenderScene,
    frames: u32,
    update: impl Fn(&mut RenderScene, u32),
) -> (RgbaImage, FrameStats, Renderer) {
    let mut renderer = Renderer::new_blocking(desc).expect("renderer");
    let mut stats = FrameStats::default();
    for f in 0..frames.max(1) {
        update(scene, f);
        stats = renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("render");
    }
    (renderer.read_offscreen().expect("capture"), stats, renderer)
}

/// Mean luma over a rectangle (x0, y0, x1, y1), exclusive ends.
fn region(img: &RgbaImage, r: (u32, u32, u32, u32)) -> f32 {
    let mut sum = 0.0;
    let mut n = 0.0;
    for y in r.1..r.3 {
        for x in r.0..r.2 {
            sum += luma(&px_clamped(img, x, y));
            n += 1.0;
        }
    }
    sum / n
}

/// Pixels whose luma differs by more than `by`.
fn differing(a: &RgbaImage, b: &RgbaImage, by: f32) -> usize {
    a.pixels
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.pixels.as_chunks::<4>().0.iter())
        .filter(|(p, q)| (luma(&[p[0], p[1], p[2]]) - luma(&[q[0], q[1], q[2]])).abs() > by)
        .count()
}
