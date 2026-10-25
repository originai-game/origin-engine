//! PostEffects.aoResolution reaches the renderer: a game that writes
//! `half` hands the renderer `AoResolution::Half` (the cheaper
//! half-resolution ambient occlusion, an `ao-upsample` pass), and a game
//! that leaves it out keeps `Full`, today's frame. The probe's `get` reads
//! the field back as written.
//!
//! Ways this could fail: the IDL field is not in the generated layout (the
//! write is refused or dropped), the runtime never reads it (the renderer
//! always gets the default), or the variants are mapped in the wrong order.

use og_render::{AmbientOcclusion, AoResolution, RenderScene, RgbaImage};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_probe_commands, og_engine_take_outputs, og_engine_write_resource, og_game_game,
    og_host_begin_bundle, set_hooks, set_render_backend,
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

/// A renderer that keeps the ambient occlusion of the last frame it was handed.
struct Recorder(Rc<Cell<Option<Option<AmbientOcclusion>>>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "test".into()
    }
    fn size(&self) -> (u32, u32) {
        (64, 64)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        self.0.set(Some(scene.post.ao));
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<RgbaImage> {
        None
    }
}

fn answer(text: String) -> Value {
    serde_json::from_str(&text).expect("wire JSON")
}

/// Writes `post` to og.PostEffects, runs a frame with the probe's `get` of
/// it; what the probe read and the AO the renderer was handed.
fn frame_with(
    engine: u32,
    seen: &Cell<Option<Option<AmbientOcclusion>>>,
    post: Value,
) -> (Value, Option<AmbientOcclusion>) {
    let a = answer(og_engine_write_resource(
        engine,
        "og.PostEffects".into(),
        post.to_string(),
    ));
    assert_eq!(a["ok"], json!(true), "{a}");
    let report = answer(og_engine_probe_commands(
        engine,
        "0:get:og.PostEffects".into(),
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
    (
        read["value"].clone(),
        seen.get().expect("the renderer was handed a frame"),
    )
}

#[test]
fn ao_resolution_reaches_the_renderer_and_defaults_to_full() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some("{}".into()));
    let engine = answer(og_engine_create(game, "{}".into()))["value"]
        .as_u64()
        .unwrap() as u32;
    let seen = Rc::new(Cell::new(None));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();

    // Left out: full resolution, today's frame.
    let (read, ao) = frame_with(engine, &seen, json!({ "ao": true }));
    assert_eq!(read["aoResolution"], json!("full"), "{read}");
    assert_eq!(ao.expect("ao on").resolution, AoResolution::Full);

    let (read, ao) = frame_with(
        engine,
        &seen,
        json!({ "ao": true, "aoResolution": "half", "aoMethod": "ssao" }),
    );
    assert_eq!(read["aoResolution"], json!("half"), "{read}");
    let ao = ao.expect("ao on");
    assert_eq!(ao.resolution, AoResolution::Half);
    assert_eq!(ao.method, og_render::AoMethod::Ssao, "the fields beside it");

    let (_, ao) = frame_with(engine, &seen, json!({ "aoResolution": "full" }));
    assert_eq!(ao.expect("ao on").resolution, AoResolution::Full);
}
