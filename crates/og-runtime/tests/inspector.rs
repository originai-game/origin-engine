//! The inspector's Save (G3-7): a field edited in its panel, by the
//! pointer and the keyboard as a person edits it, is written back by
//! `save_scene` as that one line of the scene's text, and the scene cooked
//! again loads the world the edit made.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_commands_schedule, og_engine_create, og_engine_frame,
    og_engine_probe_dump, og_engine_take_outputs, og_game_game, og_host_begin_bundle,
    og_load_scene, og_reflect_define_component, og_world_instantiate, set_hooks,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::rc::Rc;

const YARD: &str = "type scene
version 1
---
# The yard, for the inspector's save.
name \"Yard\"
entity \"crate\" {
  Transform {
    position -1 0.5 0 # dropped from here
  }
  Health {
    hp 10
    armor 2
  }
}
entity \"post\" {
  Health {
    hp 7
  }
}
";

fn cook(root: &Path) {
    use og_core::reflect::{FieldSchema, FieldType, Schema};
    let health = Schema {
        name: "Health".into(),
        version: 1,
        doc: String::new(),
        fields: vec![
            FieldSchema::new("hp", FieldType::F32),
            FieldSchema::new("armor", FieldType::F32),
        ],
    };
    let options = og_cook::CookOptions {
        threads: 1,
        schemas: vec![health],
        ..og_cook::CookOptions::default()
    };
    let report = og_cook::cook_project(root, &options).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
}

fn project() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-inspector-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    std::fs::write(root.join("assets/scenes/yard.ogt"), YARD).unwrap();
    cook(&root);
    root
}

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

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

/// A game declaring Health, the yard placed.
fn engine(root: &Path) -> u32 {
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    value(og_reflect_define_component(
        "Health".into(),
        json!([
            { "name": "hp", "kind": "f32", "default": 0, "options": {} },
            { "name": "armor", "kind": "f32", "default": 0, "options": {} },
        ])
        .to_string(),
    ));
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let e = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let scene = value(og_load_scene(e, "scenes/yard.ogt".into()));
    value(og_world_instantiate(e, scene.as_f64().unwrap(), None));
    e
}

fn step(e: u32, n: u32) {
    for _ in 0..n {
        value(og_engine_frame(e, 1000.0 / 60.0));
    }
}

/// The inspector as data (`dump_inspector`, which runs at the next tick).
fn panel(e: u32) -> Value {
    commands(e, "0:dump_inspector", 1);
    let outputs: Value = serde_json::from_str(&og_engine_take_outputs(e)).unwrap();
    let data = outputs
        .as_array()
        .and_then(|a| {
            a.iter()
                .rev()
                .find(|o| o["action"] == json!("dump_inspector"))
        })
        .and_then(|o| o["data"].as_str())
        .unwrap_or_else(|| panic!("no dump_inspector output: {outputs}"));
    serde_json::from_str(data).unwrap()
}

fn commands(e: u32, script: &str, ticks: u32) {
    value(og_commands_schedule(e, script.into()));
    step(e, ticks);
}

/// The centre of a panel element by its path, in CSS pixels.
fn centre(p: &Value, path: &str) -> String {
    let e = p["elements"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["path"] == json!(path))
        .unwrap_or_else(|| panic!("no element {path}"));
    let at = |k: &str, d: &str| e[k].as_f64().unwrap() + e[d].as_f64().unwrap() / 2.0;
    format!("{},{}", at("x", "width"), at("y", "height"))
}

/// A click at a panel element.
fn click(e: u32, p: &Value, path: &str) {
    let at = centre(p, path);
    commands(
        e,
        &format!("0:touch_move:{at};0.05:touch_start;0.1:touch_end"),
        10,
    );
}

fn entity(e: u32, id: &str) -> Value {
    let d: Value = serde_json::from_str(&og_engine_probe_dump(
        e,
        Some(json!({ "what": "entity", "id": id }).to_string()),
    ))
    .unwrap();
    d["components"].clone()
}

#[test]
fn an_edit_in_the_panel_saves_as_its_one_line_and_reloads_the_same() {
    let root = project();
    let e = engine(&root);
    step(e, 2);
    commands(e, "0:inspect:crate", 3);
    let hp = "Inspector/Detail/F:crate.Health.hp/P0";
    click(e, &panel(e), hp);
    // Clear the field's two characters (Ctrl+A is the line's start on a
    // Mac) and type over them; `type` takes two ticks a character.
    commands(
        e,
        "0:key_down:End;0.02:key_up:End;0.04:key_down:Backspace;0.06:key_up:Backspace;0.08:key_down:Backspace;0.1:key_up:Backspace;0.15:type:25;0.35:key_down:Enter;0.38:key_up:Enter",
        30,
    );
    let p = panel(e);
    assert_eq!(p["dirty"], json!(["crate.Health.hp"]), "{}", p["status"]);
    assert_eq!(entity(e, "crate")["Health"]["hp"], json!(25.0));
    click(e, &p, "Inspector/Bar/Save");
    let p = panel(e);
    assert_eq!(p["dirty"], json!([]), "{}", p["status"]);
    let saved = std::fs::read_to_string(root.join("assets/scenes/yard.ogt")).unwrap();
    assert_eq!(
        saved,
        YARD.replacen("    hp 10\n", "    hp 25\n", 1),
        "only the edited field's line changes"
    );
    // Cooked again, the scene loads the world the edit made.
    let edited = [entity(e, "crate"), entity(e, "post")];
    cook(&root);
    let again = engine(&root);
    step(again, 2);
    assert_eq!([entity(again, "crate"), entity(again, "post")], edited);
}
