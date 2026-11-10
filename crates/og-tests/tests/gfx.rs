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

#[path = "gfx/effect_glow.rs"]
mod effect_glow;
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
#[ignore = "requires GPU"]
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
#[ignore = "requires GPU"]
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

/// The cross-platform tolerance (og_tests::compare::Tolerance::CROSS_PLATFORM)
/// is wide enough for another backend's rasterisation, but not for a real
/// regression: each scene as it is passes against its Metal reference under
/// the tolerance a frame from another backend gets, and each small mutation
/// of it fails. On the recording backend (Metal) the tolerance is the tight
/// one, which the mutations fail by more.
#[test]
#[ignore = "requires GPU"]
fn the_cross_platform_tolerance_still_catches_small_regressions() {
    use og_tests::compare::compare;
    use og_tests::gfx::{load_png, reference_path, render_built};

    let judge = |name: &str, g: &mut GfxScene| -> Result<(), String> {
        let reference = load_png(&reference_path("metal", name)).expect("metal reference");
        let image = render_built(g).expect("render").image;
        let tolerance = g.tolerance.widened(g.cross_tolerance);
        let c = compare(
            &image.pixels,
            &reference.pixels,
            reference.width,
            reference.height,
            &tolerance,
        )
        .expect("same size");
        println!(
            "{name}: rmse {:.5}, outliers {:.3}%, bias {:+.5} {:+.5} {:+.5}",
            c.rmse,
            c.outlier_fraction() * 100.0,
            c.bias[0],
            c.bias[1],
            c.bias[2]
        );
        c.check(&tolerance)
    };
    /// A scene, what changed in it, and the change.
    type Mutation = (&'static str, &'static str, fn(&mut GfxScene));
    let cases: [Mutation; 4] = [
        ("gltf-reference", "the sun 15 % dimmer", |g| {
            g.scene.light.intensity *= 0.85
        }),
        ("shadows-soft", "the sun casting no shadow", |g| {
            g.scene.light.cast_shadows = false
        }),
        ("text-msdf", "a paragraph 3 px to the right", |g| {
            g.scene.texts[2].position.x += 0.088
        }),
        ("text-msdf", "the subtitle white, not amber", |g| {
            g.scene.texts[1].color = [1.0, 1.0, 1.0, 1.0]
        }),
    ];
    for (name, what, mutate) in cases {
        if let Err(e) = judge(name, &mut built(name)) {
            panic!("{name} as it is fails the cross-platform tolerance: {e}");
        }
        let mut mutated = built(name);
        mutate(&mut mutated);
        let verdict = judge(name, &mut mutated);
        println!("{name} with {what}: {verdict:?}");
        assert!(
            verdict.is_err(),
            "{name} with {what} passes the cross-platform tolerance"
        );
    }
}
