//! A navmesh asset's agent profiles (G2-4) through og.nav.load: the first
//! agent block without a profile, a named one by its name (each its own
//! mesh, the doorway open only to the small one), and an unknown profile
//! refused naming the ones the asset has.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_game_game, og_host_begin_bundle, og_nav_find_path,
    og_nav_load, set_hooks,
};
use serde_json::{Value, json};
use std::rc::Rc;

/// A 20 × 10 m floor split by a wall with a 1 m doorway, and two agents.
const DOOR: &str = "type navmesh
version 1
---
agent {
  radius 0.8
}
agent small {
  radius 0.2
  height 0.6
}
cellSize 0.1
tileSize 32
bounds {
  min -1 -1 -1
  max 21 3 11
}
box {
  center 10 -0.1 5
  size 20 0.2 10
}
box {
  center 10 1 2.25
  size 1 2 4.5
}
box {
  center 10 1 7.75
  size 1 2 4.5
}
";

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

fn wire(text: String) -> Result<Value, String> {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    if v["ok"] == json!(true) {
        Ok(v["value"].clone())
    } else {
        Err(v["reason"].as_str().unwrap_or_default().to_string())
    }
}

#[test]
fn og_nav_load_takes_an_agent_profile_and_names_the_ones_an_asset_has() {
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-nav-profiles-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/nav")).unwrap();
    std::fs::write(root.join("assets/nav/door.ogt"), DOOR).unwrap();
    let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());

    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let e = wire(og_engine_create(game, options))
        .unwrap()
        .as_u64()
        .unwrap() as u32;

    let through = |mesh: &Value| {
        let desc = json!({ "nav": mesh, "from": { "x": 3, "y": 0, "z": 5 }, "to": { "x": 17, "y": 0, "z": 5 } });
        wire(og_nav_find_path(e, Some(desc.to_string())))
            .is_ok_and(|p| p["complete"] == json!(true))
    };
    let big = wire(og_nav_load(e, "nav/door.ogt".into(), None)).unwrap();
    assert_eq!(big["profile"], json!(""));
    assert!(
        !through(&big["mesh"]),
        "the doorway is too narrow for 0.8 m"
    );
    let small = wire(og_nav_load(e, "nav/door.ogt".into(), Some("small".into()))).unwrap();
    assert_eq!(small["profile"], json!("small"));
    assert_ne!(small["mesh"], big["mesh"]);
    assert!(through(&small["mesh"]));
    let err = wire(og_nav_load(e, "nav/door.ogt".into(), Some("ogre".into()))).unwrap_err();
    assert!(
        err.contains("no agent profile 'ogre': (unnamed), small"),
        "{err}"
    );
}
