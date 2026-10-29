//! og.Sky turned off gives the scene back the built-in sky: the
//! environment the renderer is handed (the image-based light and a
//! `Background::Sky` are generated from it) carries no look of the sky
//! that was on. Before the fix the last enabled sky's colours stayed in
//! the scene for good, so a look that named Sky (heavy-ink-shadows' near
//! black) went on lighting every look after it, and look:none.

use og_render::{ProceduralSky, RenderScene, RgbaImage};
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

/// A renderer that keeps the environment sky of the last frame.
struct Recorder(Rc<Cell<Option<ProceduralSky>>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "test".into()
    }
    fn size(&self) -> (u32, u32) {
        (64, 64)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        self.0.set(Some(scene.environment.sky));
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<RgbaImage> {
        None
    }
}

fn answer(text: String) -> Value {
    serde_json::from_str(&text).expect("wire JSON")
}

fn frame(engine: u32, seen: &Cell<Option<ProceduralSky>>) -> ProceduralSky {
    let a = answer(og_engine_frame(engine, 1000.0 / 60.0));
    assert_eq!(a["ok"], json!(true), "{a}");
    seen.get().expect("the renderer was handed a frame")
}

fn write_sky(engine: u32, sky: Value) {
    let a = answer(og_engine_write_resource(
        engine,
        "og.Sky".into(),
        sky.to_string(),
    ));
    assert_eq!(a["ok"], json!(true), "{a}");
}

#[test]
fn a_sky_turned_off_leaves_no_look_in_the_environment() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some("{}".into()));
    let engine = answer(og_engine_create(game, "{}".into()))["value"]
        .as_u64()
        .unwrap() as u32;
    let seen = Rc::new(Cell::new(None));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();

    let before = frame(engine, &seen);
    assert_eq!(before.look, None, "no Sky: the built-in sky");

    let dark = json!({ "r": 0.12, "g": 0.03, "b": 0.025 });
    write_sky(
        engine,
        json!({ "enabled": true, "horizon": dark, "sunColor": { "r": 0.2, "g": 0.1, "b": 0.1 } }),
    );
    let on = frame(engine, &seen);
    let look = on.look.expect("an enabled Sky reaches the environment");
    assert!((look.horizon.x - 0.12).abs() < 1e-4, "{look:?}");

    write_sky(engine, json!({ "enabled": false }));
    let off = frame(engine, &seen);
    assert_eq!(
        off.look, None,
        "the Sky turned off still lights the scene: {off:?}"
    );
    assert_eq!(
        off.sun_color, before.sun_color,
        "the sun's colour is the built-in one again"
    );
    assert_eq!(
        off.sun_direction, before.sun_direction,
        "the sun follows the light as before"
    );
}
