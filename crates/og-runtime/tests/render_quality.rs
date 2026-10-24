//! `og.render.applyQuality` writes a graphics preset's row of
//! RenderSettings' quality fields (the table in api.ogt's doc), records
//! the preset in RenderSettings.quality, and the renderer reads those
//! fields and nothing else: what the probe's `get` reads of RenderSettings
//! is the RenderQuality the renderer is handed.
//!
//! Ways this could fail: the preset is kept somewhere the probe does not
//! see (the renderer draws low while RenderSettings says medium); the
//! fields written are not the table's (a typo, a field left out); a field
//! the game sets after the preset is overridden by the preset; an unknown
//! preset is accepted. Before the profile-quality round there was no
//! applyQuality and nothing read GraphicsQuality.

use og_render::{RenderQuality, RenderScene, RgbaImage};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_probe_commands, og_engine_take_outputs, og_engine_write_resource, og_game_game,
    og_host_begin_bundle, og_render_apply_quality, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::Cell;
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

/// A renderer that keeps the quality of the last frame it was handed.
struct Recorder(Rc<Cell<Option<RenderQuality>>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "test".into()
    }
    fn size(&self) -> (u32, u32) {
        (64, 64)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        self.0.set(Some(scene.quality));
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<RgbaImage> {
        None
    }
}

fn answer(text: String) -> Value {
    serde_json::from_str(&text).expect("wire JSON")
}

/// The quality fields of RenderSettings as the probe's `get` reads them
/// in the next frame, and the quality the renderer was handed in it.
fn observe(engine: u32, seen: &Cell<Option<RenderQuality>>) -> (Value, RenderQuality) {
    let report = answer(og_engine_probe_commands(
        engine,
        "0:get:og.RenderSettings".into(),
    ));
    assert_eq!(report["scheduled"], json!(1), "{report}");
    let a = answer(og_engine_frame(engine, 1000.0 / 60.0));
    assert_eq!(a["ok"], json!(true), "{a}");
    let outputs = answer(og_engine_take_outputs(engine));
    let got = outputs
        .as_array()
        .and_then(|o| o.iter().find(|o| o["kind"] == "json"))
        .unwrap_or_else(|| panic!("no get output: {outputs}"));
    let read: Value = serde_json::from_str(got["data"].as_str().unwrap()).unwrap();
    let settings = &read["value"];
    let pick = [
        "quality",
        "rectShadows",
        "shadowedLights",
        "shadowMapScale",
        "volumetricFog",
        "softParticles",
        "screenSpaceReflections",
        "ambientOcclusion",
        "msaaSamples",
    ];
    let fields = Value::Object(
        pick.iter()
            .map(|k| (k.to_string(), settings[*k].clone()))
            .collect(),
    );
    (fields, seen.get().expect("the renderer was handed a frame"))
}

/// The renderer's quality as the dump's fields would say it.
fn as_fields(quality: &str, q: RenderQuality) -> Value {
    json!({
        "quality": quality,
        "rectShadows": q.rect_shadows,
        "shadowedLights": q.shadowed_lights,
        "shadowMapScale": q.shadow_map_scale,
        "volumetricFog": q.volumetric_fog,
        "softParticles": q.soft_particles,
        "screenSpaceReflections": q.ssr,
        "ambientOcclusion": q.ambient_occlusion,
        "msaaSamples": q.msaa_samples,
    })
}

#[test]
fn apply_quality_writes_the_presets_row_and_the_renderer_reads_it() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some("{}".into()));
    let engine = answer(og_engine_create(game, "{}".into()))["value"]
        .as_u64()
        .unwrap() as u32;
    let seen = Rc::new(Cell::new(None));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();

    // Before any preset: no record, the full frame.
    let (fields, q) = observe(engine, &seen);
    assert_eq!(q, RenderQuality::default());
    assert_eq!(fields, as_fields("none", q));

    // The table of api.ogt's applyQuality doc, written out independently.
    let table = [
        (
            "low",
            json!({ "quality": "low", "rectShadows": false, "shadowedLights": 2, "shadowMapScale": 0.5,
                "volumetricFog": false, "softParticles": false, "screenSpaceReflections": false,
                "ambientOcclusion": false, "msaaSamples": 1 }),
        ),
        (
            "high",
            json!({ "quality": "high", "rectShadows": true, "shadowedLights": 8, "shadowMapScale": 2.0,
                "volumetricFog": true, "softParticles": true, "screenSpaceReflections": true,
                "ambientOcclusion": true, "msaaSamples": 4 }),
        ),
        (
            "medium",
            json!({ "quality": "medium", "rectShadows": true, "shadowedLights": 8, "shadowMapScale": 1.0,
                "volumetricFog": true, "softParticles": true, "screenSpaceReflections": true,
                "ambientOcclusion": true, "msaaSamples": 4 }),
        ),
    ];
    for (preset, row) in table {
        let a = answer(og_render_apply_quality(engine, preset.into()));
        assert_eq!(a["ok"], json!(true), "{preset}: {a}");
        let (fields, q) = observe(engine, &seen);
        assert_eq!(fields, row, "{preset}: the fields written");
        assert_eq!(
            as_fields(preset, q),
            row,
            "{preset}: the renderer reads the fields"
        );
    }

    // A field set after the preset is drawn as set; the record stays.
    answer(og_render_apply_quality(engine, "low".into()));
    let a = answer(og_engine_write_resource(
        engine,
        "og.RenderSettings".into(),
        json!({ "rectShadows": true }).to_string(),
    ));
    assert_eq!(a["ok"], json!(true), "{a}");
    let (fields, q) = observe(engine, &seen);
    assert!(q.rect_shadows && !q.ssr, "{q:?}");
    assert_eq!(fields["quality"], json!("low"));
    assert_eq!(fields["rectShadows"], json!(true));

    let a = answer(og_render_apply_quality(engine, "ultra".into()));
    assert_eq!(a["ok"], json!(false), "{a}");
    assert!(
        a["reason"]
            .as_str()
            .unwrap()
            .contains("not a graphics preset")
    );
}
