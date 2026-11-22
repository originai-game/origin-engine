//! A flying game's origin can stay near the player, with clouds below it.
//! The public resource write must accept a negative base and hand that
//! same world height to the renderer, rather than reject or clamp it.

use og_render::{RenderScene, RgbaImage};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_write_resource, og_game_game, og_host_begin_bundle, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::Cell;
use std::rc::Rc;

struct NoAssets;

impl Hooks for NoAssets {
    fn dispatch(&self, _: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets in this test"))
    }
}

struct Recorder(Rc<Cell<Option<f32>>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "test".into()
    }
    fn size(&self) -> (u32, u32) {
        (64, 64)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        self.0.set(scene.atmosphere.clouds.map(|c| c.base));
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<RgbaImage> {
        None
    }
}

fn answer(text: String) -> Value {
    let value: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(value["ok"], json!(true), "{text}");
    value
}

#[test]
fn a_cloud_base_can_be_below_the_game_origin() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some("{}".into()));
    let engine = answer(og_engine_create(game, "{}".into()))["value"]
        .as_u64()
        .unwrap() as u32;
    let seen = Rc::new(Cell::new(None));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    for base in [-2500.0, 1500.0] {
        answer(og_engine_write_resource(
            engine,
            "og.Clouds".into(),
            json!({ "enabled": true, "base": base }).to_string(),
        ));
        answer(og_engine_frame(engine, 1000.0 / 60.0));
        assert_eq!(seen.get(), Some(base));
    }
}
