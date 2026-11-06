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

use og_render::{GpuMemory, QualityPreset, RenderQuality, RenderScene, RgbaImage};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_render_display, og_render_reset_display, og_render_set_display,
    set_hooks, set_render_backend,
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
    cloth_subdivisions: u32,
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
        self.seen.borrow_mut().cloth_subdivisions = scene.cloth_subdivisions;
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
