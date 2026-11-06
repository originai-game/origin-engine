//! gfx reference frames: each scene rendered headless must match its
//! committed reference for this backend tier (see og_tests::gfx), plus
//! behaviour checks read from rendered frames (shadows per cascade, edge
//! coverage, bloom, metering, sprite order) and from the frame stats.

use glam::{Mat4, Vec2, Vec3};
use og_render::{
    Antialias, Background, Camera, DebugView, FrameStats, RenderDesc, RenderError, RenderScene,
    RenderTarget, Renderer, RgbaImage, Sprite, Tonemap,
};
use og_tests::scenes::{self, GfxScene};

#[path = "gfx/overlay_effects.rs"]
mod overlay_effects;
#[path = "gfx/sprites_post.rs"]
mod sprites_post;

/// Renders `frames` frames of `scene` with `desc` and returns the last one.
fn frame(desc: &RenderDesc, scene: &RenderScene, frames: u32) -> (RgbaImage, FrameStats) {
    let mut renderer = Renderer::new_blocking(desc).expect("renderer");
    let mut stats = FrameStats::default();
    for _ in 0..frames.max(1) {
        stats = renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("render");
    }
    (renderer.read_offscreen().expect("capture"), stats)
}

fn built(name: &str) -> GfxScene {
    scenes::build(name).expect("scene")
}

/// The cascade debug view (red, green, blue for cascades 0, 1, 2) for
/// pixel classification, and the scene with and without shadows.
fn cascade_frames(tall: bool) -> (RgbaImage, RgbaImage, RgbaImage) {
    let g = built("cascades-200m");
    let mut scene = scenes::cascades_200m(tall);
    let (lit, _) = frame(&g.desc, &scene, 1);
    scene.light.cast_shadows = false;
    let (unshadowed, _) = frame(&g.desc, &scene, 1);
    scene.light.cast_shadows = true;
    scene.debug = DebugView::Cascades;
    let (debug, _) = frame(&g.desc, &scene, 1);
    (debug, lit, unshadowed)
}

fn cascade_of(p: &[u8]) -> Option<usize> {
    let max = *p[..3].iter().max().expect("rgb");
    if max < 100 {
        return None;
    }
    let c = p[..3].iter().position(|&v| v == max).expect("max");
    p[..3]
        .iter()
        .enumerate()
        .all(|(i, &v)| i == c || v < 60)
        .then_some(c)
}

/// All three cascades are in use, nearer pixels in nearer cascades: the
/// cascade covering the bottom row (the ground at the camera's feet) is 0,
/// and the far ground at the horizon is covered by cascade 2.
#[test]
fn cascades_cover_the_view_near_to_far() {
    let (debug, _, _) = cascade_frames(true);
    let w = debug.width as usize;
    let mut counts = [0usize; 3];
    for p in debug.pixels.as_chunks::<4>().0.iter() {
        if let Some(c) = cascade_of(p) {
            counts[c] += 1;
        }
    }
    println!("pixels per cascade: {counts:?}");
    for (c, n) in counts.iter().enumerate() {
        assert!(*n > 5_000, "cascade {c} covers only {n} pixels");
    }
    let row_cascade = |y: usize| {
        // The cascade most of row `y` is in.
        let mut row = [0usize; 3];
        for x in 0..w {
            if let Some(c) = cascade_of(&debug.pixels[(y * w + x) * 4..]) {
                row[c] += 1;
            }
        }
        (0..3).max_by_key(|&c| row[c]).expect("three")
    };
    assert_eq!(row_cascade(debug.height as usize - 1), 0, "bottom row");
    // Down the middle column, the first pixel below the sky is the farthest.
    let x = w / 2;
    let horizon = (0..debug.height as usize)
        .find(|&y| cascade_of(&debug.pixels[(y * w + x) * 4..]).is_some())
        .expect("ground in view");
    assert_eq!(
        cascade_of(&debug.pixels[(horizon * w + x) * 4..]),
        Some(2),
        "the far ground at the horizon"
    );
}

/// `RenderInfo::msaa_samples` is the MSAA the device gives, the cap of
/// `RenderQuality::msaa_samples` (`RenderCapabilities.maxMsaaSamples`):
/// 4 where `Msaa4` resolved to MSAA (it stays 4 while a quality of 1 draws
/// FXAA, which `antialias` says), 1 where the renderer resolved TAA, FXAA
/// or none, which a quality asking 4 does not change. Before the debt
/// round nothing said so: a preset's 4 was drawn at 1 unseen.
#[test]
fn the_device_msaa_caps_the_qualitys() {
    use og_render::RenderQuality;
    let scene = RenderScene::new();
    let mut msaa = Renderer::new_blocking(&RenderDesc {
        antialias: Antialias::Msaa4,
        ..scenes::base_desc()
    })
    .expect("renderer");
    let resolved = msaa.info().antialias;
    println!(
        "{}: Msaa4 resolved to {resolved:?}, msaa_samples {}",
        msaa.info().backend_tier(),
        msaa.info().msaa_samples
    );
    if resolved == Antialias::Msaa4 {
        assert_eq!(msaa.info().msaa_samples, og_render::MSAA_SAMPLES);
        let mut one = RenderScene::new();
        one.quality = RenderQuality {
            msaa_samples: 1,
            ..RenderQuality::default()
        };
        msaa.render(&one, &RenderTarget::Offscreen).expect("render");
        assert_eq!(msaa.info().antialias, Antialias::Fxaa);
        assert_eq!(msaa.info().msaa_samples, og_render::MSAA_SAMPLES);
    } else {
        assert_eq!(resolved, Antialias::Fxaa);
        assert_eq!(msaa.info().msaa_samples, 1);
    }
    for antialias in [Antialias::Taa, Antialias::Fxaa, Antialias::Off] {
        let mut r = Renderer::new_blocking(&RenderDesc {
            antialias,
            ..scenes::base_desc()
        })
        .expect("renderer");
        assert_eq!(r.info().msaa_samples, 1, "{antialias:?}");
        r.render(&scene, &RenderTarget::Offscreen).expect("render");
        assert_eq!(
            r.info().antialias,
            antialias,
            "a quality of 4 draws one sample"
        );
    }
}
