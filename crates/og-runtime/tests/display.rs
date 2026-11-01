//! `og.render.setDisplay` is presentation only (src/display.rs): a host that
//! presents (the web canvas) draws at its render scale, a native run draws
//! at scale 1 whatever the settings say, automatic quality's preset steps
//! cap the quality handed to the renderer without writing RenderSettings,
//! and the world's hash is the same with or without any of it.
//!
//! Ways this could fail: the settings kept in a world resource (the hash
//! moves, a replay diverges); the scale applied on the native host (the
//! reference frames move); a preset step written into RenderSettings (the
//! probe and a save see automatic quality's choice); the frame cap not
//! answered with the frame, so the Worker never caps; a device made again
//! after a loss handed the lost one's texture encodings (the frame is
//! refused) or textures transcoded again for the old formats.

use og_render::{
    GpuMemory, QualityPreset, RenderQuality, RenderScene, RgbaImage, TextureFormat, TextureFormats,
    TextureHandle, TextureSource,
};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, lose_device, og_engine_create, og_engine_frame,
    og_game_game, og_host_begin_bundle, og_load_texture, og_load_texture_array, og_render_display,
    og_render_reset_display, og_render_set_display, set_hooks, set_render_backend, set_viewport,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::rc::Rc;

struct NoAssets;

impl Hooks for NoAssets {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets in this test"))
    }
}

#[derive(Default)]
struct Seen {
    scales: Vec<f32>,
    quality: Option<RenderQuality>,
    projection: Option<og_render::Projection>,
}

/// A renderer that records the render scales it was set and the quality
/// of the last frame; `presents` as the web canvas does or not.
struct Recorder {
    seen: Rc<RefCell<Seen>>,
    presents: bool,
    size: (u32, u32),
}

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "test".into()
    }
    fn size(&self) -> (u32, u32) {
        self.size
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        self.seen.borrow_mut().quality = Some(scene.quality);
        self.seen.borrow_mut().projection = Some(scene.camera.projection);
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<RgbaImage> {
        None
    }
    fn presents(&self) -> bool {
        self.presents
    }
    fn set_render_scale(&mut self, scale: f32) {
        self.seen.borrow_mut().scales.push(scale);
    }
    fn gpu_memory(&self) -> Option<GpuMemory> {
        Some(GpuMemory {
            textures: 1000,
            meshes: 200,
            render_targets: 30,
            buffers: 4,
        })
    }
}

fn answer(text: String) -> Value {
    serde_json::from_str(&text).expect("wire JSON")
}

fn engine(presents: bool) -> (u32, Rc<RefCell<Seen>>) {
    engine_sized(presents, (200, 100))
}

fn engine_sized(presents: bool, size: (u32, u32)) -> (u32, Rc<RefCell<Seen>>) {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some("{}".into()));
    let engine = answer(og_engine_create(game, "{}".into()))["value"]
        .as_u64()
        .unwrap() as u32;
    let seen = Rc::new(RefCell::new(Seen::default()));
    set_render_backend(
        engine,
        Some(Box::new(Recorder {
            seen: seen.clone(),
            presents,
            size,
        })),
    )
    .unwrap();
    (engine, seen)
}

fn frames(engine: u32, n: usize) -> Value {
    let mut last = Value::Null;
    for _ in 0..n {
        last = answer(og_engine_frame(engine, 1000.0 / 60.0));
        assert_eq!(last["ok"], json!(true), "{last}");
    }
    last
}

/// The world's save bytes (og.world.snapshot): everything a save and a
/// replay carry.
fn snapshot(engine: u32) -> Value {
    let s = answer(og_runtime::og_world_snapshot(engine, None));
    assert_eq!(s["ok"], json!(true), "{s}");
    assert!(s["value"].to_string().len() > 16, "{s}");
    s["value"].clone()
}

#[test]
fn a_presenting_host_draws_at_the_render_scale_and_a_native_run_at_one() {
    let (web, seen) = engine(true);
    let set = |e, s: Value| answer(og_render_set_display(e, Some(s.to_string())));
    assert_eq!(
        set(web, json!({ "renderScale": 0.7, "autoQuality": false }))["ok"],
        json!(true)
    );
    let a = frames(web, 3);
    assert_eq!(seen.borrow().scales.last(), Some(&0.7));
    // A desktop (no coarse pointer) runs at the display's rate.
    assert_eq!(a["maxFps"], json!(0));
    let info = answer(og_render_display(web));
    assert_eq!(info["presenting"], json!(true));
    assert_eq!(info["renderScale"], json!(0.7f32));
    assert_eq!(info["gpu"]["total"], json!(1234.0));
    assert_eq!(info["settings"]["frameCap"], json!("auto"));

    // A phone's auto cap is 60; the battery saver is 30 and caps the scale.
    set_viewport(
        web,
        json!({ "width": 400, "height": 800, "coarsePointer": true }),
    )
    .unwrap();
    assert_eq!(frames(web, 1)["maxFps"], json!(60));
    set(web, json!({ "renderScale": 1.0, "batterySaver": true }));
    assert_eq!(frames(web, 1)["maxFps"], json!(30));
    assert_eq!(seen.borrow().scales.last(), Some(&0.75));
    assert!(set(web, json!({ "frameCap": "fps90" }))["ok"] == json!(false));

    // A native run never has its scale set, whatever the settings.
    let (native, seen) = engine(false);
    set(native, json!({ "renderScale": 0.5, "autoQuality": false }));
    frames(native, 3);
    assert!(
        seen.borrow().scales.is_empty(),
        "{:?}",
        seen.borrow().scales
    );
    let info = answer(og_render_display(native));
    assert_eq!(
        (info["presenting"].clone(), info["renderScale"].clone()),
        (json!(false), json!(1.0))
    );
}

/// The settings are not in the world: two engines of the same game, one
/// with every setting changed between its frames, save the same bytes.
#[test]
fn display_settings_never_reach_the_world() {
    let (a, _) = engine(true);
    let (b, _) = engine(true);
    for i in 0..6 {
        let cap = ["display", "fps30"][i % 2];
        let s = json!({ "renderScale": 0.5 + 0.25 * i as f32, "batterySaver": i % 2 == 0, "frameCap": cap });
        og_render_set_display(b, Some(s.to_string()));
        frames(a, 2);
        frames(b, 2);
    }
    assert_eq!(snapshot(a), snapshot(b));
}

/// The memory report carries the renderer's GPU memory.
#[test]
fn the_memory_report_counts_gpu_memory() {
    let (e, _) = engine(false);
    frames(e, 1);
    let m = og_runtime::memory_report(e);
    assert_eq!(
        m["gpu"],
        json!({ "textures": 1000.0, "meshes": 200.0, "renderTargets": 30.0, "buffers": 4.0, "total": 1234.0 })
    );
    // Field telemetry reads the device losses beside it (none yet).
    assert_eq!(m["deviceLosses"], json!(0));
}

/// The quality a preset step hands the renderer is the game's capped:
/// a game at high quality with autoPresetSteps 2 that automatic quality
/// lowered all the way draws low, its RenderSettings untouched.
#[test]
fn preset_steps_cap_the_quality_drawn_not_the_settings() {
    let (e, seen) = engine(true);
    og_runtime::og_render_apply_quality(e, "high".into());
    og_render_set_display(
        e,
        Some(json!({ "autoPresetSteps": 2, "autoMinScale": 0.5 }).to_string()),
    );
    og_runtime::set_display_interval(e, 1000.0 / 60.0);
    // Frames far over the budget (100 ms intervals, little CPU work,
    // no GPU time) walk the ladder down to its last rung.
    for _ in 0..2000 {
        let a = answer(og_engine_frame(e, 100.0));
        assert_eq!(a["ok"], json!(true));
    }
    let info = answer(og_render_display(e));
    assert_eq!(info["presetSteps"], json!(2), "{info}");
    assert_eq!(info["renderScale"], json!(0.5));
    assert_eq!(
        seen.borrow().quality,
        Some(RenderQuality::preset(QualityPreset::Low))
    );
    let settings = answer(og_runtime::og_world_resource(e, "og.RenderSettings".into()));
    assert_eq!(settings["value"]["quality"], json!("high"), "{settings}");
    assert_eq!(
        settings["value"]["shadowMapScale"],
        json!(2.0),
        "{settings}"
    );
}

// ── a device that comes back taking other texture encodings ─────────────

/// Reads the cooked files from disk; textures transcode natively (the
/// default texture hook).
struct Disk;

impl Hooks for Disk {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
}

/// A project with two colour textures (Basis-coded by the cook) and a
/// texture array of them, cooked once per test process.
fn textured_project() -> &'static std::path::Path {
    static ROOT: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    ROOT.get_or_init(|| {
        let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-device-formats-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let tex = root.join("assets/tex");
        std::fs::create_dir_all(&tex).unwrap();
        for name in ["crystal", "ember"] {
            std::fs::copy(
                repo.join(format!(
                    "templates/archetypes/idle/assets/sprites/mote-{name}.png"
                )),
                tex.join(format!("{name}.png")),
            )
            .unwrap();
        }
        std::fs::write(
            tex.join("layers.ogt"),
            "type texture-array\nversion 1\n---\nlayer \"crystal.png\"\nlayer \"ember.png\"\n",
        )
        .unwrap();
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

fn formats(compressed: bool) -> TextureFormats {
    if compressed {
        TextureFormats {
            color: TextureFormat::Bc7,
            normal: TextureFormat::Bc5,
            data: TextureFormat::Bc7,
            supported: vec![TextureFormat::Rgba8, TextureFormat::Bc7, TextureFormat::Bc5],
        }
    } else {
        TextureFormats {
            color: TextureFormat::Rgba8,
            normal: TextureFormat::Rgba8,
            data: TextureFormat::Rgba8,
            supported: vec![TextureFormat::Rgba8],
        }
    }
}

/// Each texture and texture-array layer of a frame: its encoding and its
/// first level's bytes.
type Drawn = Vec<(String, Vec<u8>)>;

fn drawn(source: &TextureSource) -> (String, Vec<u8>) {
    match source {
        TextureSource::Rgba(t) => ("rgba8".into(), t.rgba.clone()),
        TextureSource::Encoded(u) => (u.desc.format.name().into(), u.levels[0].clone()),
        TextureSource::Target(_) => ("target".into(), Vec::new()),
    }
}

/// A device as og-render's is to the runtime: it takes the encodings of
/// `formats` and refuses a frame holding any other (as the renderer's
/// upload does, `RenderError::Unsupported`); lost, it comes back on
/// `recover` taking `after`'s.
struct Device {
    formats: TextureFormats,
    after: TextureFormats,
    lost: bool,
    losses: u32,
    seen: Rc<RefCell<Drawn>>,
}

impl RenderBackend for Device {
    fn tier(&self) -> String {
        "test".into()
    }
    fn size(&self) -> (u32, u32) {
        (64, 64)
    }
    fn texture_formats(&self) -> Option<TextureFormats> {
        Some(self.formats.clone())
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        if self.lost {
            return Err("lost".into());
        }
        let mut out = Vec::new();
        let textures = (0..scene.texture_count() as u32)
            .filter_map(|i| TextureHandle::from_slot(i + 1))
            .filter_map(|h| {
                scene
                    .texture_upload(h)
                    .map(|u| TextureSource::Encoded(u.clone()))
                    .or_else(|| scene.texture(h).map(|t| TextureSource::Rgba(t.clone())))
            });
        let layers = scene
            .texture_arrays()
            .iter()
            .flat_map(|a| a.layers.iter().cloned());
        for source in textures.chain(layers) {
            if let TextureSource::Encoded(u) = &source
                && !self.formats.supported.contains(&u.desc.format)
            {
                return Err(format!("unsupported: {}", u.desc.format.name()));
            }
            out.push(drawn(&source));
        }
        *self.seen.borrow_mut() = out;
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<RgbaImage> {
        None
    }
    fn recover(&mut self) -> Result<bool, String> {
        if self.lost {
            self.lost = false;
            self.losses += 1;
            self.formats = self.after.clone();
        }
        Ok(false)
    }
    fn device_losses(&self) -> u32 {
        self.losses
    }
    fn lose_device(&mut self) -> bool {
        self.lost = true;
        true
    }
}

/// An engine on a [`Device`] taking `before`'s encodings (then `after`'s),
/// with the project's texture and texture array loaded and drawn once.
fn textured(before: bool, after: bool) -> (u32, Rc<RefCell<Drawn>>) {
    let root = textured_project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = answer(og_engine_create(game, options))["value"]
        .as_u64()
        .unwrap() as u32;
    let seen = Rc::new(RefCell::new(Drawn::new()));
    set_render_backend(
        engine,
        Some(Box::new(Device {
            formats: formats(before),
            after: formats(after),
            lost: false,
            losses: 0,
            seen: seen.clone(),
        })),
    )
    .unwrap();
    for loaded in [
        og_load_texture(engine, "tex/crystal.png".into()),
        og_load_texture_array(engine, "tex/layers.ogt".into()),
    ] {
        assert_eq!(answer(loaded.clone())["ok"], json!(true), "{loaded}");
    }
    frames(engine, 1);
    (engine, seen)
}

#[test]
fn a_device_back_with_other_encodings_gets_its_textures_transcoded_again() {
    let (engine, seen) = textured(true, false);
    let before = seen.borrow().clone();
    let encodings: Vec<&str> = before.iter().map(|(f, _)| f.as_str()).collect();
    // The texture and both layers went to the first device as BC7 (the
    // cook's Basis transcoded for it), beside the engine's own RGBA8 ones.
    assert_eq!(
        encodings.iter().filter(|f| **f == "bc7").count(),
        3,
        "{encodings:?}"
    );

    // Lost, and back taking RGBA8 only: the frame draws (no BC7 handed to
    // the new device) and every cooked texture is transcoded again for it.
    assert!(lose_device(engine));
    frames(engine, 1);
    let after = seen.borrow().clone();
    assert!(after.iter().all(|(f, _)| f == "rgba8"), "{after:?}");
    assert_eq!(answer(og_render_display(engine))["deviceLosses"], json!(1));

    // The frame is the frame of an engine on that device from the start.
    let (_, fresh) = textured(false, false);
    assert!(
        *fresh.borrow() == after,
        "the textures transcoded again differ from those transcoded for the device at once"
    );
}

// ── the player's graphics settings (G1-7) ───────────────────────────────

/// The graphics settings round-trip and apply: what og.render.display
/// answers is the object a game saves, and setting it back on a fresh
/// engine gives the same settings; a call that names a few fields keeps
/// the others (a game's frame cap leaves the player's render scale); the
/// player's preset is drawn in place of RenderSettings' quality (which
/// keeps the game's), on a native run too, and null is the game's again;
/// the 120 and 144 caps reach the frame answer, and so do fullscreen, the
/// power preference and raw mouse input, what the web shell acts on.
///
/// Ways it could fail: a field missing from the saved object (fullscreen
/// or the power preference forgotten on the next load); a partial call
/// that resets the rest (the G1-4 contract, which a settings page writing
/// one switch broke); the preset written into RenderSettings (the world
/// moves, a replay diverges); the page never told to go fullscreen.
#[test]
fn graphics_settings_round_trip_and_apply() {
    let (e, seen) = engine(false);
    let set = |e, s: Value| answer(og_render_set_display(e, Some(s.to_string())));
    let game = answer(og_runtime::og_world_resource(e, "og.RenderSettings".into()));
    let chosen = json!({
        "quality": "low", "frameCap": "fps144", "batterySaver": false, "renderScale": 0.8,
        "autoQuality": false, "autoMinScale": 0.6, "autoPresetSteps": 2,
        "fullscreen": true, "powerPreference": "lowPower",
    });
    assert_eq!(set(e, chosen.clone())["ok"], json!(true));
    let a = frames(e, 1);
    assert_eq!(
        (
            a["maxFps"].clone(),
            a["fullscreen"].clone(),
            a["lowPower"].clone(),
            a["rawMouse"].clone()
        ),
        (json!(144), json!(true), json!(true), json!(true)),
        "{a}"
    );
    assert_eq!(
        seen.borrow().quality,
        Some(RenderQuality::preset(QualityPreset::Low))
    );
    let settings = answer(og_runtime::og_world_resource(e, "og.RenderSettings".into()));
    assert_eq!(settings, game, "RenderSettings keep the game's quality");
    let saved = answer(og_render_display(e))["settings"].clone();
    for (k, v) in chosen.as_object().unwrap() {
        let got = &saved[k];
        let same = match (got.as_f64(), v.as_f64()) {
            (Some(a), Some(b)) => (a - b).abs() < 1e-6,
            _ => got == v,
        };
        assert!(same, "{k}: saved {got}, set {v}");
    }

    // The saved object, through JSON (a settings save), on a fresh engine.
    let text = saved.to_string();
    let (f, seen_f) = engine(false);
    assert_eq!(
        set(f, serde_json::from_str(&text).unwrap())["ok"],
        json!(true)
    );
    assert_eq!(answer(og_render_display(f))["settings"], saved);
    frames(f, 1);
    assert_eq!(
        seen_f.borrow().quality,
        Some(RenderQuality::preset(QualityPreset::Low))
    );

    // A partial call keeps the rest; quality null is the game's own again.
    set(f, json!({ "frameCap": "fps120", "quality": null }));
    let now = answer(og_render_display(f))["settings"].clone();
    assert_eq!(now["frameCap"], json!("fps120"));
    assert_eq!(now["renderScale"], saved["renderScale"]);
    assert_eq!(now["fullscreen"], json!(true));
    assert_eq!(now["quality"], Value::Null);
    frames(f, 1);
    assert_eq!(seen_f.borrow().quality, Some(RenderQuality::default()));
    assert!(set(f, json!({ "quality": "ultra" }))["ok"] == json!(false));
    assert!(set(f, json!({ "powerPreference": "battery" }))["ok"] == json!(false));

    // resetDisplay is the way back: every field at its default, as on an
    // engine nothing set, and the page told to leave fullscreen.
    assert_eq!(answer(og_render_reset_display(f))["ok"], json!(true));
    let reset = answer(og_render_display(f))["settings"].clone();
    let (g, _) = engine(false);
    assert_eq!(reset, answer(og_render_display(g))["settings"]);
    assert_eq!(
        (
            reset["frameCap"].clone(),
            reset["renderScale"].clone(),
            reset["fullscreen"].clone(),
            reset["powerPreference"].clone(),
            reset["quality"].clone()
        ),
        (
            json!("auto"),
            json!(1.0),
            json!(false),
            json!("highPerformance"),
            Value::Null
        )
    );
    assert_eq!(frames(f, 1)["fullscreen"], json!(false));
    assert_eq!(answer(og_render_reset_display(9999))["ok"], json!(false));
}

/// What the browser decides comes back from the page: fullscreen follows
/// it (Escape turns the setting off, so the page does not ask again), the
/// adapter's power preference is the boot's and a different one waits
/// for the next load, vsync is always on where the host presents, and the
/// mouse's raw input is what pointer lock was granted.
#[test]
fn the_page_reports_fullscreen_power_and_raw_mouse() {
    let (e, _) = engine(true);
    og_runtime::set_power_preference(e, false);
    let info = answer(og_render_display(e));
    assert_eq!(
        (
            info["vsync"].clone(),
            info["powerPreference"].clone(),
            info["restartNeeded"].clone()
        ),
        (json!(true), json!("highPerformance"), json!(false)),
        "{info}"
    );
    og_render_set_display(
        e,
        Some(json!({ "fullscreen": true, "powerPreference": "lowPower" }).to_string()),
    );
    assert_eq!(frames(e, 1)["fullscreen"], json!(true));
    let info = answer(og_render_display(e));
    assert_eq!(
        info["restartNeeded"],
        json!(true),
        "low power applies at the next load"
    );
    assert_eq!(
        info["fullscreen"],
        json!(false),
        "not yet: the page has not gone fullscreen"
    );
    og_runtime::set_fullscreen(e, true);
    assert_eq!(answer(og_render_display(e))["fullscreen"], json!(true));
    // The player pressed Escape.
    og_runtime::set_fullscreen(e, false);
    let info = answer(og_render_display(e));
    assert_eq!(
        (
            info["fullscreen"].clone(),
            info["settings"]["fullscreen"].clone()
        ),
        (json!(false), json!(false))
    );
    assert_eq!(
        frames(e, 1)["fullscreen"],
        json!(false),
        "the page is not asked again"
    );
    // Booted on the low-power adapter: no restart for lowPower.
    og_runtime::set_power_preference(e, true);
    assert_eq!(answer(og_render_display(e))["restartNeeded"], json!(false));

    let mouse = |e| answer(og_runtime::og_input_mouse(e));
    assert_eq!(
        mouse(e),
        json!({ "raw": true, "rawActive": false, "rawUnsupported": false })
    );
    assert_eq!(
        answer(og_runtime::og_input_set_mouse(
            e,
            Some(json!({ "raw": false }).to_string())
        ))["ok"],
        json!(true)
    );
    assert_eq!(frames(e, 1)["rawMouse"], json!(false));
    og_runtime::set_mouse_report(e, false, true);
    assert_eq!(
        mouse(e),
        json!({ "raw": false, "rawActive": false, "rawUnsupported": true })
    );

    // A native run presents nothing: no vsync to speak of.
    let (n, _) = engine(false);
    assert_eq!(answer(og_render_display(n))["vsync"], json!(false));
}

/// The settings are presentation only: two engines of one game, one with
/// every new setting changed and reported between frames, save the same
/// bytes.
#[test]
fn graphics_settings_and_page_reports_never_reach_the_world() {
    let (a, _) = engine(true);
    let (b, _) = engine(true);
    for i in 0..4 {
        let q = ["low", "high"][i % 2];
        og_render_set_display(b, Some(json!({ "quality": q, "fullscreen": i % 2 == 0, "frameCap": "fps144", "powerPreference": "lowPower" }).to_string()));
        og_runtime::set_fullscreen(b, i % 2 == 1);
        og_runtime::set_mouse_report(b, i % 2 == 0, false);
        frames(a, 2);
        frames(b, 2);
    }
    assert_eq!(snapshot(a), snapshot(b));
}

/// Hor+: the vertical field of view is og.Camera.fov at every aspect and a
/// wider surface sees more at the sides. On a 32:9 surface the drawn
/// projection's vertical FOV is the camera's, a point at the 32:9 half
/// angle across lands on the right edge (og.camera.worldToScreen, which
/// rays and world-anchored UI share), and that same point is off a 16:9
/// surface; a point at the vertical half angle is on the top edge on both.
///
/// Ways it could fail: Vert- (the horizontal FOV fixed, the 32:9 view
/// cropped top and bottom); the picking projection on another aspect than
/// the drawn one (a click at the edge of an ultrawide picks the wrong
/// thing); the render scale's rounding changing the aspect.
#[test]
// The expected angles, worked out here: no tick reads them.
#[allow(clippy::disallowed_methods)]
fn the_field_of_view_is_hor_plus() {
    let fov = 50.0f32;
    let half = (fov.to_radians() / 2.0).tan();
    // A point 10 m ahead, across by the 32:9 half width, up by the half height.
    let across = 10.0 * half * 32.0 / 9.0;
    let up = 10.0 * half;
    let screen = |e: u32, x: f32, y: f32| {
        answer(og_runtime::og_camera_world_to_screen(
            e,
            json!({ "x": x, "y": y, "z": -10.0 }).to_string(),
        ))
    };
    for (w, h) in [(3840u32, 1080u32), (1920, 1080)] {
        let (e, seen) = engine_sized(true, (w, h));
        og_runtime::set_viewport(e, json!({ "width": w, "height": h })).unwrap();
        frames(e, 1);
        match seen.borrow().projection {
            Some(og_render::Projection::Perspective { fov_y_degrees }) => {
                assert!(
                    (fov_y_degrees - fov).abs() < 1e-3,
                    "{w}x{h}: drawn at {fov_y_degrees}°"
                )
            }
            other => panic!("{other:?}"),
        }
        let top = screen(e, 0.0, up);
        assert!(top["y"].as_f64().unwrap().abs() < 0.5, "{w}x{h}: {top}");
        let edge = screen(e, across, 0.0)["x"].as_f64().unwrap();
        if w == 3840 {
            assert!((edge - 3840.0).abs() < 0.5, "the 32:9 edge: {edge}");
        } else {
            assert!(edge > 1920.0 + 100.0, "off a 16:9 surface: {edge}");
        }
    }
}
