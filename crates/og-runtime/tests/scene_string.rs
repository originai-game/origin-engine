//! A string field through the real load path: a scene whose entities carry a
//! game component with a `str(16)` field is cooked (og-cook, with the
//! component's schema, as `og cook` packs the engine's own), then loaded with
//! og.load.scene into a live world and instantiated; world.get answers each
//! entity's text as it was written (FINDINGS row 65: og-cook's
//! `scene_string.rs` covers the cook alone).
//!
//! Ways this could fail: the runtime reads the cooked row with another
//! layout than the live component's (the text shifted, or the u32 after it
//! read from the padding); the zero padding leaks into the text (a 16-byte
//! name with garbage, an empty name that is not ""); multi-byte UTF-8 or a
//! quote is mangled between the blob and the world; a text that fills the
//! capacity exactly loses its last byte.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_game_game, og_host_begin_bundle, og_load_scene,
    og_reflect_define_component, og_world_find, og_world_get, og_world_instantiate, set_hooks,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

/// The entities, their names and levels, as the scene text writes them.
const PEOPLE: &[(&str, &str, u32)] = &[
    ("mei", "Mei", 3),
    ("lantern", "小灯 \"lantern\"", 2),
    ("nobody", "", 1),
    ("full", "abcdefghijklmnop", 0),
];

/// The project, cooked once per test process with Profile's schema.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-scene-string-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
        let mut scene = String::from("type scene\nversion 1\n---\nname \"People\"\n");
        for (entity, name, level) in PEOPLE {
            let quoted = name.replace('\\', "\\\\").replace('"', "\\\"");
            scene.push_str(&format!(
                "entity \"{entity}\" {{\n  Profile {{\n    name \"{quoted}\"\n    level {level}\n  }}\n}}\n"
            ));
        }
        std::fs::write(root.join("assets/scenes/people.ogt"), scene).unwrap();
        use og_core::reflect::{FieldSchema, FieldType, Schema};
        let profile = Schema {
            name: "Profile".into(),
            version: 1,
            doc: String::new(),
            fields: vec![
                FieldSchema::new("name", FieldType::string(16).unwrap()),
                FieldSchema::new("level", FieldType::U32),
            ],
        };
        let options = og_cook::CookOptions {
            threads: 1,
            schemas: vec![profile],
            ..og_cook::CookOptions::default()
        };
        let report = og_cook::cook_project(&root, &options).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
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

#[test]
fn a_cooked_scene_with_a_string_field_loads_into_a_live_world_with_the_text_intact() {
    let root = project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    // The game's component, declared as TypeScript's str(16) and u32() send it.
    let fields = json!([
        { "name": "name", "kind": "str", "default": "", "options": {}, "capacity": 16 },
        { "name": "level", "kind": "u32", "default": 0, "options": {} },
    ]);
    value(og_reflect_define_component(
        "Profile".into(),
        fields.to_string(),
    ));
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;

    let scene = value(og_load_scene(engine, "scenes/people.ogt".into()));
    value(og_world_instantiate(engine, scene.as_f64().unwrap(), None));
    for (entity, name, level) in PEOPLE {
        let found =
            serde_json::from_str::<Value>(&og_world_find(engine, (*entity).into())).unwrap();
        let e = found
            .as_f64()
            .unwrap_or_else(|| panic!("no entity '{entity}' after instantiate: {found}"));
        let got = value(og_world_get(engine, e, "Profile".into()));
        assert_eq!(got, json!({ "name": name, "level": level }), "{entity}");
    }
}
