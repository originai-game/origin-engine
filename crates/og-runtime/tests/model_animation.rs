//! A cooked rigid-node animation must reach the host's submitted instances,
//! not only the simulation pose queried by og.anim.
use og_render::{RenderScene, RgbaImage};
use og_runtime::*;
use serde_json::{Value, json};
use std::{cell::RefCell, rc::Rc};

struct Assets;
impl Hooks for Assets {
    fn dispatch(&self, _: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(e.to_string()),
        }
    }
}

struct Recorder(Rc<RefCell<Vec<og_core::glam::Mat4>>>);
impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        *self.0.borrow_mut() = scene
            .objects
            .instances
            .iter()
            .map(|i| i.transform)
            .collect();
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<RgbaImage> {
        None
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["ok"], true, "{text}");
    v["value"].clone()
}

#[test]
fn cooked_rigid_node_keys_move_the_drawn_model() {
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("rigid-node-animation-{}", std::process::id()));
    std::fs::create_dir_all(root.join("assets")).unwrap();
    std::fs::write(
        root.join("assets/panel.ogt"),
        "type model\nversion 1\n---\nconform off\npivot \"root\" {\nposition 1 3 0\n}\npart \"panel\" {\npivot \"root\"\nboxGeometry {\nwidth 1\nheight 2\ndepth 0.5\n}\n}\n",
    ).unwrap();
    std::fs::write(
        root.join("assets/panel.ogt.ogt"),
        "type import\nversion 1\n---\nanimation {\nclip \"open\" {\nchannel \"panel\" translation {\nkey 0 0 0 0\nkey 0.5 2 1 0\n}\nchannel \"panel\" rotation {\nkey 0 0 0 0 1\nkey 0.5 0 0 0.70710678 0.70710678\n}\n}\n}\n",
    ).unwrap();
    let cooked = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(cooked.ok(), "{:?}", cooked.errors());
    set_hooks(Rc::new(Assets));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({"seed": 1}).to_string()));
    let engine = value(og_engine_create(game, json!({"root": root}).to_string()))
        .as_u64()
        .unwrap() as u32;
    let drawn = Rc::new(RefCell::new(Vec::new()));
    set_render_backend(engine, Some(Box::new(Recorder(drawn.clone())))).unwrap();
    let model = og_load_gltf(engine, "panel.ogt".into(), None).unwrap();
    let animation = og_load_animation(engine, "panel.ogt".into()).unwrap();
    let entity = og_world_spawn(
        engine,
        Some(json!({"position":{"x":5,"y":2,"z":0}}).to_string()),
    )
    .unwrap()
    .as_f64()
    .unwrap();
    for (component, fields) in [
        ("og.Model", json!({"asset": model})),
        ("og.Skeleton", json!({"asset": animation["skeleton"]})),
        (
            "og.AnimationPlayer",
            json!({"clip": animation["clips"][0]["clip"], "loop":false,"fade":0}),
        ),
    ] {
        og_world_insert(engine, entity, component.into(), Some(fields.to_string())).unwrap();
    }
    value(og_engine_advance(engine, 35));
    value(og_engine_frame(engine, 0.0));
    let matrices = drawn.borrow();
    assert_eq!(matrices.len(), 1);
    let m = matrices[0];
    assert!(
        (m.w_axis.x - 8.0).abs() < 0.001 && (m.w_axis.y - 6.0).abs() < 0.001,
        "{m:?}"
    );
    assert!(
        m.x_axis.x.abs() < 0.001 && (m.x_axis.y - 1.0).abs() < 0.001,
        "{m:?}"
    );
    assert!(og_engine_destroy(engine));
    std::fs::remove_dir_all(root).unwrap();
}
