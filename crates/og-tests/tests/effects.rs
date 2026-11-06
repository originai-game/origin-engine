//! The E1 effects, asserted on frames and counts: particles (determinism of
//! a 600-tick run on both paths, the CPU and GPU paths agreeing, spawn
//! counts, soft particles, depth collision, 2D emitters), trails and beams,
//! 2D lights and soft shadows, glow, sprite flash, decals, the grading stack
//! (LUT, vignette, chromatic aberration, grain, flash), camera shake,
//! ambient occlusion, depth of field, motion blur, god rays, outlines,
//! toon, dissolve, rim, and GPU skinning with blendshapes.

use glam::{Mat4, Vec2, Vec3};
use og_render::{
    AmbientOcclusion, AoMethod, Background, Beam, Burst, Camera, CameraShake, ChromaticAberration,
    ColorAdjust, ColorCurve, ColorGrading, ColorLut, Curve, Decal, DepthOfField, DofQuality,
    EmitterShape, Exposure, FilmGrain, FrameStats, Glow, GodRays, JointPalette, LensDistortion,
    LensFlare, Light2d, Lighting2d, MAX_CLUSTER_DECALS, Material, MorphTargets, MotionBlur,
    Occluder2d, Outline, ParticleCollision, ParticleEmitter, ParticleRender, ParticleSimulation,
    ParticleSystem, Projection, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage,
    ScreenBlur, ScreenBlurKind, ScreenFlash, SkinnedInstance, Sprite, SpriteFx, StandardMaterial,
    Tonemap, Trail, TrailStyle, View, ViewTarget, Vignette, shapes,
};
use og_tests::fx_scenes as fx;
use og_tests::{gfx, scenes};
use std::path::PathBuf;

#[path = "effects/models.rs"]
mod models;
#[path = "effects/particles.rs"]
mod particles;
#[path = "effects/post.rs"]
mod post;
#[path = "effects/quality.rs"]
mod quality;
#[path = "effects/sprites.rs"]
mod sprites;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-effects")
}

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

fn frame(desc: &RenderDesc, scene: &RenderScene) -> (RgbaImage, FrameStats) {
    let mut renderer = Renderer::new_blocking(desc).expect("renderer");
    let stats = renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("render");
    (renderer.read_offscreen().expect("capture"), stats)
}

fn px(img: &RgbaImage, x: u32, y: u32) -> [u8; 3] {
    let i = ((y.min(img.height - 1) * img.width + x.min(img.width - 1)) * 4) as usize;
    [img.pixels[i], img.pixels[i + 1], img.pixels[i + 2]]
}

fn luma(p: [u8; 3]) -> f32 {
    0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
}

/// Mean luma over a rectangle (x0, y0, x1, y1), exclusive ends.
fn region(img: &RgbaImage, r: (u32, u32, u32, u32)) -> f32 {
    let mut sum = 0.0;
    let mut n = 0.0;
    for y in r.1..r.3 {
        for x in r.0..r.2 {
            sum += luma(px(img, x, y));
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
        .filter(|(p, q)| (luma([p[0], p[1], p[2]]) - luma([q[0], q[1], q[2]])).abs() > by)
        .count()
}

/// The pixel a world point projects to.
fn project(cam: &Camera, w: u32, h: u32, p: Vec3) -> (u32, u32) {
    let view = glam::camera::rh::view::look_at_mat4(cam.position, cam.target, cam.up);
    let proj = match cam.projection {
        Projection::Perspective { fov_y_degrees } => glam::camera::rh::proj::directx::perspective(
            fov_y_degrees.to_radians(),
            w as f32 / h as f32,
            cam.near,
            cam.far,
        ),
        Projection::Orthographic { height } => {
            let hh = height * 0.5;
            let hw = hh * w as f32 / h as f32;
            glam::camera::rh::proj::directx::orthographic(-hw, hw, -hh, hh, cam.near, cam.far)
        }
    };
    let c = proj * view * p.extend(1.0);
    let ndc = c.truncate() / c.w;
    (
        ((ndc.x * 0.5 + 0.5) * w as f32) as u32,
        ((0.5 - ndc.y * 0.5) * h as f32) as u32,
    )
}

fn save(name: &str, img: &RgbaImage) {
    let _ = gfx::save_png(&out_dir().join(format!("{name}.png")), img);
}
