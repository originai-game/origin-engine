//! Billboards (`og.Sprite.facing`, `SpriteDesc.facing`): the runtime hands
//! the renderer each world sprite's facing at its index
//! (`RenderScene::sprite_facing`), og.Sprite rows and og.render.sprite
//! alike, and a screen sprite never faces.
//!
//! Ways this could fail: the facing is read from the wrong field or
//! variant name; it lands on another sprite's index (an immediate sprite
//! queued after the rows, a blob shadow pushed before its sprite); a
//! frame keeps the last frame's list.

use og_render::{RenderScene, SpriteFacing};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_render_sprite, og_world_insert, og_world_spawn, set_hooks,
    set_render_backend,
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

/// The last frame's world sprites as (x, facing), and its overlay's count.
#[derive(Default)]
struct Seen {
    sprites: Vec<(f32, SpriteFacing)>,
    screen: usize,
}

struct Recorder(Rc<RefCell<Seen>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        let mut seen = self.0.borrow_mut();
        seen.sprites = scene
            .sprites
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let facing = scene.sprite_facing.get(i).copied().unwrap_or_default();
                (s.position.x, facing)
            })
            .collect();
        seen.screen = scene.overlay.as_ref().map_or(0, |o| o.sprites.len());
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

fn spawn(engine: u32, x: f32, sprite: Value) {
    let desc = json!({ "position": { "x": x, "y": 0, "z": 0 } });
    let e = value(og_world_spawn(engine, Some(desc.to_string())))
        .as_f64()
        .unwrap();
    value(og_world_insert(
        engine,
        e,
        "og.Sprite".into(),
        Some(sprite.to_string()),
    ));
}

#[test]
fn every_world_sprite_reaches_the_renderer_with_its_facing() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let engine = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(Seen::default()));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    frame(engine);
    // A plain sprite, an upright billboard with a blob shadow pushed
    // before it, and one facing the camera.
    spawn(engine, 1.0, json!({}));
    spawn(engine, 2.0, json!({ "facing": "cameraY", "shadow": 0.4 }));
    spawn(engine, 3.0, json!({ "facing": "camera" }));
    // Immediate: one facing the camera, a screen sprite asking for it.
    let sprite = |x: f32, extra: Value| {
        let mut v = json!({ "position": { "x": x, "y": 0, "z": 0 }, "facing": "camera" });
        v.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        value(og_render_sprite(engine, Some(v.to_string())));
    };
    sprite(4.0, json!({}));
    sprite(5.0, json!({ "screen": true }));
    frame(engine);
    let mut drawn = seen.borrow().sprites.clone();
    drawn.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert_eq!(
        drawn,
        [
            (1.0, SpriteFacing::None),
            (2.0, SpriteFacing::None),
            (2.0, SpriteFacing::CameraY),
            (3.0, SpriteFacing::Camera),
            (4.0, SpriteFacing::Camera),
        ],
        "the blob shadow (at the sprite's x) stays flat"
    );
    assert_eq!(
        seen.borrow().screen,
        1,
        "the screen sprite is the overlay's"
    );
    // The next frame: the immediate sprite is gone and with it its facing.
    frame(engine);
    let s = seen.borrow();
    assert_eq!(s.sprites.len(), 4);
    assert_eq!(
        s.sprites
            .iter()
            .filter(|(_, f)| *f != SpriteFacing::None)
            .count(),
        2
    );
}
