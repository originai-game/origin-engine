//! The light cookies' cap (debt render-light #4): the frame draws at most
//! RenderCapabilities.maxLightCookies layers of cookies (a spot's takes
//! one, a point's six), and a frame whose cookies ask for more warns in the
//! log, once, naming the lights that shine without theirs.
//!
//! Ways this could fail: the cap stays hidden (no capability, no warning:
//! the old runtime dropped the fifth point light's cookie silently); the
//! capability says something the renderer does not cap at; the warning
//! repeats every frame.

use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_engine_take_log,
    og_game_game, og_host_begin_bundle, og_render_capabilities, og_render_render_target,
    og_world_insert, og_world_spawn, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::rc::Rc;

struct NoAssets;

impl Hooks for NoAssets {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets here"))
    }
}

/// A backend that draws nothing: the frame's scene is what is checked.
struct Blank;

impl RenderBackend for Blank {
    fn tier(&self) -> String {
        "full".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, _: &RenderScene) -> Result<(), String> {
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

fn frame(engine: u32) {
    value(og_engine_frame(engine, 1000.0 / 60.0));
}

/// The warnings logged since the last call.
fn warnings(engine: u32) -> Vec<String> {
    let log: Value = serde_json::from_str(&og_engine_take_log(engine)).unwrap();
    log.as_array()
        .unwrap()
        .iter()
        .filter(|l| l["warning"] == json!(true))
        .map(|l| l["text"].as_str().unwrap().to_string())
        .collect()
}

/// A point light with its own cookie (a render target's texture), its id.
fn cookie_point(engine: u32, x: f64) -> f64 {
    let target = value(og_render_render_target(
        engine,
        Some(json!({ "width": 8, "height": 48 }).to_string()),
    ));
    let e = value(og_world_spawn(
        engine,
        Some(json!({ "position": { "x": x, "y": 2, "z": 0 } }).to_string()),
    ))
    .as_f64()
    .unwrap();
    let light = json!({ "intensity": 5, "distance": 6, "cookie": target["texture"] });
    value(og_world_insert(
        engine,
        e,
        "og.PointLight".into(),
        Some(light.to_string()),
    ));
    e
}

#[test]
fn cookies_past_the_cap_are_reported() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let engine = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    set_render_backend(engine, Some(Box::new(Blank))).unwrap();
    let caps: Value = serde_json::from_str(&og_render_capabilities(engine)).unwrap();
    assert_eq!(caps["maxLightCookies"], json!(24), "{caps}");

    // Four point cookies: 24 layers, all drawn.
    for i in 0..4 {
        cookie_point(engine, i as f64);
    }
    frame(engine);
    assert_eq!(warnings(engine), Vec::<String>::new(), "24 layers fit");

    // A fifth: 30 layers asked for; it shines without its cookie, and the
    // log says so once.
    let fifth = cookie_point(engine, 4.0);
    frame(engine);
    let w = warnings(engine);
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(
        w[0].contains("take 30 layers")
            && w[0].contains("maxLightCookies (24")
            && w[0].contains("this light shines without its cookie")
            && w[0].contains(&og_ecs::Entity::from_bits(fifth as u64).to_string()),
        "{}",
        w[0]
    );
    frame(engine);
    assert_eq!(warnings(engine), Vec::<String>::new(), "warned once");
}
