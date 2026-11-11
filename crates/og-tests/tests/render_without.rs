//! RenderOptions.without (og-render `RenderDesc::without`): a renderer made
//! without optional lit features draws a scene that uses none of them as
//! one made with every feature does, draws a scene that uses one as if it
//! did not, reports what it was made without, and says so once.
//!
//! Ways this could fail, each caught below: a lit module the renderer makes
//! keeps a left-out feature's code (the material shaders', the built-in lit
//! and prepass module's, or the full variant a capture draws with), so the
//! wet scene's surfaces stay wet while its weather still reaches them; a
//! left-out feature still draws through another path (a rect light still
//! lights, a decal still prints, the probes still bounce) or is still
//! prepared unseen (a rect light kept in the clusters, a decal recorded,
//! the probes traced); the list
//! changes what a scene without those features draws; the renderer forgets
//! what it was made without; or the log repeats itself every frame.

use glam::{Mat4, Vec3};
use og_render::{
    FrameStats, GpuMemory, LitFeatures, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage,
    StandardMaterial, shapes,
};
use og_tests::{p2_scenes, scenes};
use std::sync::{Mutex, OnceLock};

const W: u32 = 320;
const H: u32 = 180;

/// Every optional lit feature.
fn every() -> LitFeatures {
    LitFeatures::NAMED
        .iter()
        .fold(LitFeatures::NONE, |a, (f, _)| a | *f)
}

/// A registered scene, its desc at the tests' size and its frame count.
fn built(name: &str) -> (RenderDesc, RenderScene, u32) {
    let g = scenes::build(name).expect(name);
    (
        RenderDesc {
            width: W,
            height: H,
            ..g.desc
        },
        g.scene,
        g.frames.max(1),
    )
}

/// `scene` drawn by a renderer made without `without`: `frames - 1` frames,
/// then a capture (every lit pipeline its full variant, waited for), read
/// back, with the capture's counts and the GPU memory the renderer holds.
fn render(
    desc: &RenderDesc,
    without: LitFeatures,
    scene: &RenderScene,
    frames: u32,
) -> (RgbaImage, FrameStats, GpuMemory) {
    let desc = RenderDesc { without, ..*desc };
    let mut renderer = Renderer::new_blocking(&desc).expect("renderer");
    for _ in 1..frames {
        renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("render");
    }
    let stats = renderer
        .render_capture(scene, &RenderTarget::Offscreen)
        .expect("capture");
    assert_eq!(renderer.info().without, without, "what it was made without");
    let image = renderer.read_offscreen().expect("read");
    (image, stats, renderer.gpu_memory())
}

/// The largest difference of one channel between two frames.
fn max_diff(a: &RgbaImage, b: &RgbaImage) -> u8 {
    assert_eq!((a.width, a.height), (b.width, b.height));
    a.pixels
        .iter()
        .zip(&b.pixels)
        .map(|(x, y)| x.abs_diff(*y))
        .max()
        .unwrap_or(0)
}

/// The log's warnings that name RenderOptions.without.
fn warnings() -> &'static Mutex<Vec<String>> {
    static LOG: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    LOG.get_or_init(|| Mutex::new(Vec::new()))
}

struct Collect;

impl log::Log for Collect {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.level() <= log::Level::Warn
    }
    fn log(&self, r: &log::Record) {
        let text = r.args().to_string();
        if text.contains("RenderOptions.without") {
            warnings().lock().unwrap().push(text);
        }
    }
    fn flush(&self) {}
}

/// The weather scene with a mesh drawn through a material shader beside
/// its built-in surfaces, all of them wet; `wet` false: no weather.
fn weather_scene(wet: bool) -> (RenderDesc, RenderScene, u32) {
    let (desc, mut scene, frames) = built("weather-wet");
    if !wet {
        scene.atmosphere.surface_weather = None;
    }
    let shader = scene.add_shader(p2_scenes::ripple_shader());
    let m = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.7, 0.62, 0.5),
        roughness: 0.6,
        ..Default::default()
    });
    scene.set_material_shader(m, Some(shader));
    let cube = scene.add_mesh(shapes::cuboid(Vec3::new(1.2, 1.2, 1.2)));
    scene.add_instance(cube, m, Mat4::from_translation(Vec3::new(0.2, 0.6, -1.5)));
    (desc, scene, frames)
}

/// What takes a feature out of a scene.
type Strip = fn(&mut RenderScene);

fn no_rect_lights(s: &mut RenderScene) {
    s.lighting
        .lights
        .retain(|l| !matches!(l.kind, og_render::LightKind::Rect { .. }));
}

fn no_decals(s: &mut RenderScene) {
    s.decals.clear();
}

fn no_probes(s: &mut RenderScene) {
    s.look.post.gi_probes = None;
}

fn no_caustics(s: &mut RenderScene) {
    s.water.iter_mut().for_each(|w| w.caustics = 0.0);
}

/// A registered scene that uses each feature, and what takes it out.
fn feature_scenes() -> Vec<(&'static str, LitFeatures, Strip)> {
    vec![
        (
            "rect-lights",
            LitFeatures::AREA_LIGHTS,
            no_rect_lights as Strip,
        ),
        ("decals", LitFeatures::DECALS, no_decals),
        ("gi-room-probes", LitFeatures::GI, no_probes),
        ("water-underwater", LitFeatures::CAUSTICS, no_caustics),
    ]
}

#[test]
#[ignore = "requires GPU"]
fn the_features_left_out_are_in_no_pipeline_and_not_drawn() {
    // One test, so the warnings counted are its own.
    static COLLECT: Collect = Collect;
    let _ = log::set_logger(&COLLECT);
    log::set_max_level(log::LevelFilter::Warn);

    // A scene that uses none of them draws as with every feature, byte for
    // byte: the pipelines differ only in code it never runs.
    let (desc, dry, frames) = weather_scene(false);
    let full = render(&desc, LitFeatures::NONE, &dry, frames).0;
    assert_eq!(
        max_diff(&full, &render(&desc, every(), &dry, frames).0),
        0,
        "a scene that uses none of them changed"
    );
    assert!(warnings().lock().unwrap().is_empty(), "{:?}", warnings());

    // The weather still reaches every surface, but no lit module the
    // renderer made has its code: the wet scene, the material shader's
    // mesh too, draws as the dry one.
    let (_, wet, _) = weather_scene(true);
    assert!(
        max_diff(&render(&desc, LitFeatures::NONE, &wet, frames).0, &full) > 8,
        "the weather must show to be left out"
    );
    let off = LitFeatures::SURFACE_WEATHER;
    let dry_off = render(&desc, off, &dry, frames).0;
    assert_eq!(
        max_diff(&render(&desc, off, &wet, frames).0, &dry_off),
        0,
        "a lit module made without the weather on surfaces drew it"
    );

    // Rect lights, decals, the probes' bounce and caustics, each in its own
    // scene: left out, the scene draws as without them. (The caustics' rows
    // still reach the lit modules, as the weather's do.)
    for (name, feature, strip) in feature_scenes() {
        let (desc, scene, frames) = built(name);
        let (_, mut without, _) = built(name);
        strip(&mut without);
        assert!(
            max_diff(
                &render(&desc, LitFeatures::NONE, &scene, frames).0,
                &render(&desc, LitFeatures::NONE, &without, frames).0
            ) > 8,
            "{name}: the feature must show to be left out"
        );
        let (left_out, counts, memory) = render(&desc, feature, &scene, frames);
        let (stripped, stripped_counts, stripped_memory) = render(&desc, feature, &without, frames);
        assert_eq!(
            max_diff(&left_out, &stripped),
            0,
            "{name}: left out, it still drew"
        );
        // Nor was anything made for it: no rect light kept for the clusters
        // or the shadow maps, no decal recorded, no pass for the probes, and
        // no memory held for any of it.
        let made = |c: &FrameStats| {
            (
                c.lights,
                c.cluster_lights,
                c.shadow_passes,
                c.decals,
                c.graph_passes,
            )
        };
        assert_eq!(
            made(&counts),
            made(&stripped_counts),
            "{name}: left out, it was still prepared (lights, cluster refs, shadow passes, decals, graph passes)"
        );
        assert_eq!(
            memory, stripped_memory,
            "{name}: left out, it still holds memory"
        );
    }

    // Each scene that used a left-out feature said so once, over all its
    // frames and its capture.
    let said = warnings().lock().unwrap().clone();
    for (_, name) in LitFeatures::NAMED {
        assert_eq!(
            said.iter()
                .filter(|w| w.contains(&format!(" {name},")))
                .count(),
            1,
            "{name}: {said:?}"
        );
    }
}
