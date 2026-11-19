//! RenderSettings.autoExposureMin and autoExposureMax reach the renderer as
//! auto exposure's range: a game that bounds it hands the renderer those
//! stops, one that leaves them out keeps -10 to 10, and a maximum below the
//! minimum is taken as the minimum (never an empty range).
//!
//! Ways this could fail: the runtime keeps its fixed -10 to 10 (the fields
//! never read), the two are swapped, or an inverted range reaches the
//! renderer as written.

use og_render::{Exposure, RenderScene, RgbaImage};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_write_resource, og_game_game, og_host_begin_bundle, set_hooks, set_render_backend,
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

/// A renderer that keeps the exposure of the last frame it was handed.
struct Recorder(Rc<Cell<Option<Exposure>>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "test".into()
    }
    fn size(&self) -> (u32, u32) {
        (64, 64)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        self.0.set(Some(scene.frame.camera.exposure));
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<RgbaImage> {
        None
    }
}

fn answer(text: String) -> Value {
    serde_json::from_str(&text).expect("wire JSON")
}

#[test]
fn the_auto_exposure_range_reaches_the_renderer() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some("{}".into()));
    let engine = answer(og_engine_create(game, "{}".into()))["value"]
        .as_u64()
        .unwrap() as u32;
    let seen = Rc::new(Cell::new(None));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    let range = |settings: Value| {
        let a = answer(og_engine_write_resource(
            engine,
            "og.RenderSettings".into(),
            settings.to_string(),
        ));
        assert_eq!(a["ok"], json!(true), "{a}");
        let a = answer(og_engine_frame(engine, 1000.0 / 60.0));
        assert_eq!(a["ok"], json!(true), "{a}");
        match seen.get().expect("the renderer was handed a frame") {
            Exposure::Auto { min_ev, max_ev, .. } => (min_ev, max_ev),
            other => panic!("auto exposure on, the renderer got {other:?}"),
        }
    };
    assert_eq!(range(json!({ "autoExposure": true })), (-10.0, 10.0));
    assert_eq!(
        range(json!({ "autoExposureMin": -3.5, "autoExposureMax": 1.25 })),
        (-3.5, 1.25)
    );
    assert_eq!(range(json!({ "autoExposureMax": -6 })), (-3.5, -3.5));
}
