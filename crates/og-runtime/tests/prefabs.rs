//! Prefabs at run time (`docs/design.md` §2.5, G2-1): a scene placing a
//! prefab is cooked (og-cook, with the game's component's schema), loaded
//! and instantiated into a live world.
//!
//! Ways this could fail, and the test that catches each:
//! - a save of an instanced world does not restore in a session that
//!   never placed a prefab (og.PrefabLink is registered lazily), or the
//!   restored world does not go on tick for tick → `a_save_of_an_instanced_world…`;
//! - a world without prefabs registers og.PrefabLink (its type table and
//!   hash move), or a re-cook that changes nothing a prefab spawns moves
//!   the hash of a world that places it → `hashes_are_pinned…`;
//! - save_scene writes a placed entity's every field (or none), revert
//!   leaves the override or the live value, apply leaves the override in
//!   the scene or writes the prefab at the wrong node →
//!   `apply_back_and_revert_round_trip_in_the_text`.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_commands_schedule, og_engine_create, og_engine_frame,
    og_engine_probe_describe, og_engine_probe_dump, og_game_game, og_host_begin_bundle,
    og_load_scene, og_reflect_define_component, og_world_find, og_world_get, og_world_instantiate,
    og_world_prefab_of, og_world_restore, og_world_snapshot, set_hooks,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub(crate) const CRATE: &str = "type prefab
version 1
---
entity \"crate\" {
  Health {
    hp 10
  }
  entity \"lid\" {
    Health {
      hp 10
    }
  }
}
";

pub(crate) const YARD: &str = "type scene
version 1
---
name \"Yard\"
entity \"crate1\" {
  prefab \"prefabs/crate.ogt\"
  override \"lid\" {
    Health {
      hp 40
    }
  }
}
entity \"crate2\" {
  prefab \"prefabs/crate.ogt\"
}
";

/// The same entities as YARD, written out by hand.
const BY_HAND: &str = "type scene
version 1
---
name \"Yard\"
entity \"crate1\" {
  Health {
    hp 10
  }
  entity \"crate1/lid\" {
    Health {
      hp 40
    }
  }
}
entity \"crate2\" {
  Health {
    hp 10
  }
  entity \"crate2/lid\" {
    Health {
      hp 10
    }
  }
}
";

fn cook(root: &Path) {
    use og_core::reflect::{FieldSchema, FieldType, Schema};
    let health = Schema {
        name: "Health".into(),
        version: 1,
        doc: String::new(),
        fields: vec![FieldSchema::new("hp", FieldType::F32)],
    };
    let options = og_cook::CookOptions {
        threads: 1,
        schemas: vec![health],
        ..og_cook::CookOptions::default()
    };
    let report = og_cook::cook_project(root, &options).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
}

/// A project with `scene` at scenes/yard.ogt and the crate prefab, cooked.
pub(crate) fn project(name: &str, scene: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-prefabs-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    std::fs::create_dir_all(root.join("assets/prefabs")).unwrap();
    // Only where the scene places it: a project without prefabs is one
    // the engine before them cooked too.
    if scene.contains("prefab ") {
        std::fs::write(root.join("assets/prefabs/crate.ogt"), CRATE).unwrap();
    }
    std::fs::write(root.join("assets/scenes/yard.ogt"), scene).unwrap();
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

/// A game declaring Health, its engine on `root`.
fn engine(root: &Path) -> u32 {
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    value(og_reflect_define_component(
        "Health".into(),
        json!([{ "name": "hp", "kind": "f32", "default": 0, "options": {} }]).to_string(),
    ));
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    value(og_engine_create(game, options)).as_u64().unwrap() as u32
}

/// An engine with the yard instantiated.
pub(crate) fn placed(root: &Path) -> u32 {
    let e = engine(root);
    let scene = value(og_load_scene(e, "scenes/yard.ogt".into()));
    value(og_world_instantiate(e, scene.as_f64().unwrap(), None));
    e
}

fn hash(e: u32) -> Value {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(e, None)).unwrap();
    dump["hash"].clone()
}

fn step(e: u32, n: u32) {
    for _ in 0..n {
        value(og_engine_frame(e, 1000.0 / 60.0));
    }
}

fn find(e: u32, name: &str) -> f64 {
    serde_json::from_str::<Value>(&og_world_find(e, name.into()))
        .unwrap()
        .as_f64()
        .unwrap_or_else(|| panic!("no entity '{name}'"))
}

pub(crate) fn hp(e: u32, name: &str) -> f64 {
    value(og_world_get(e, find(e, name), "Health".into()))["hp"]
        .as_f64()
        .unwrap()
}

fn link(e: u32, name: &str) -> Value {
    serde_json::from_str(&og_world_prefab_of(e, find(e, name))).unwrap()
}

#[test]
fn a_save_of_an_instanced_world_restores_where_no_prefab_was_placed_and_goes_on_as_it() {
    let root = project("save", YARD);
    let a = placed(&root);
    assert_eq!(hp(a, "crate1/lid"), 40.0);
    assert_eq!(hp(a, "crate2/lid"), 10.0);
    let lid = link(a, "crate1/lid");
    assert_eq!(lid["node"], json!(1));
    assert_eq!(lid["root"].as_f64(), Some(find(a, "crate1")));
    assert_eq!(link(a, "crate1")["root"].as_f64(), Some(find(a, "crate1")));
    step(a, 5);
    let save = value(og_world_snapshot(a, None))
        .as_str()
        .unwrap()
        .to_string();
    let saved = hash(a);

    // A session that has loaded nothing and never placed a prefab.
    let b = engine(&root);
    assert_eq!(
        og_world_prefab_of(b, 1.0),
        "null",
        "no og.PrefabLink before"
    );
    value(og_world_restore(b, save, None));
    assert_eq!(hash(b), saved);
    assert_eq!(link(b, "crate1/lid"), lid, "the link came back with it");
    assert_eq!(hp(b, "crate1/lid"), 40.0);
    step(a, 10);
    step(b, 10);
    assert_eq!(hash(b), hash(a), "tick for tick as the original");
}

#[test]
fn hashes_are_pinned_without_prefabs_and_stable_across_a_re_cook_with_them() {
    // The same entities written by hand: the hash main 67a07d57 gave this world
    // (before prefabs), so a game without prefabs is unchanged.
    let plain = project("plain", BY_HAND);
    let p = placed(&plain);
    step(p, 3);
    assert_eq!(hash(p), json!(BY_HAND_HASH));

    let root = project("recook", YARD);
    let a = placed(&root);
    step(a, 3);
    let first = hash(a);
    // Placed from a prefab, the same values hash otherwise: the links are
    // world state (accepted, the owner, 2026-11-02).
    assert_ne!(first, hash(p));
    // The prefab's text changes but not what it spawns: a comment, the
    // lid's block written another way. The scene re-cooks; the hash stays.
    std::fs::write(
        root.join("assets/prefabs/crate.ogt"),
        CRATE.replace(
            "  entity \"lid\" {",
            "  # the lid opens upwards\n  entity \"lid\" {",
        ),
    )
    .unwrap();
    cook(&root);
    let b = placed(&root);
    step(b, 3);
    assert_eq!(hash(b), first);
}

/// [`hashes_are_pinned_without_prefabs_and_stable_across_a_re_cook_with_them`]'s
/// hand-written world's hash on main 67a07d57 (G2-2 and G2-5), before
/// prefabs; a change on main that moves every world's hash moves it too
/// (G2-5's og.AudioMix resource and audio records moved it from
/// 0ccd86471f57151d; G2-3's sleep from e5f1f831d77c7bc0: og.RigidBody and
/// og.Physics at version 2 with their new fields, og.BodySlept and
/// og.BodyWoke registered, all in the type table every world hashes; then
/// G2-3's breaking joints and contact hooks from 88fa067dd437beb0: og.Joint
/// at version 2, og.JointBroken and og.ContactHook registered; then from
/// 8321f6acb96a66be when those four types became lazy, registered only
/// once a game names them, so only the version-2 layouts remain; then
/// from 8c463e75e426e15b for og.Vehicle's version-2 layout, the
/// drivetrain; then G2-7's rollback from fd13375581ba26c1: og.Lockstep's
/// `mode` and `maxPrediction` fields change its layout in the type table;
/// then G2-7's stall timeout from 93d4a11b9687d3b6: its `stallTimeout` and
/// `rejoin` fields; the seat events are lazy, so they do not; then U6d's
/// UI state version 13 from 13d5d679aa3698ae: Hit.owner; then G3-2's lazy
/// og.VideoPlayer from a99071b12db700c7: a world without footage no longer
/// has the type, so its later changes leave this hash alone; then R11's
/// lazy og.Lighting2D, og.Light2D and og.Occluder2D from fd4dddba26ef754a,
/// for the same reason; then ui-gaps' UI state version 14 from
/// 1069aa19a26d8691; then version 15 from e83b148c0f38efb5; then R9's
/// og.Blendshapes version 2 (`influences`) from 46cc8d6517681608, then
/// R6's og.TerrainScatter impostor fields from 5136c2039d7ed9ec and
/// og.Model.sway from c9cc80ca8f3b2c20, then R5's lazy og.Water from
/// e1f4faa4b95c9f64: a world without water no longer has the type, so the
/// sea's fields leave this hash alone, then og.Environment's blurriness,
/// grounded dome and blend fields from f075e6bd81233636, then T5's lazy
/// og.Terrain from 6d8422aaf5e31ccc, gone from a world without terrain,
/// then area 7's Stylise and MaterialStyle fields from 0ac8584de3c39543 to
/// b06093a615447620, then area 4's lazy og.TerrainScatter and og.Wind).
const BY_HAND_HASH: &str = "f21f6a4b0639cc74";

#[test]
fn apply_back_and_revert_round_trip_in_the_text() {
    let root = project("apply", YARD);
    let scene = |r: &Path| std::fs::read_to_string(r.join("assets/scenes/yard.ogt")).unwrap();
    let prefab = |r: &Path| std::fs::read_to_string(r.join("assets/prefabs/crate.ogt")).unwrap();
    let e = placed(&root);
    // A tuned descendant: save_scene writes it as an override of its node.
    value(og_commands_schedule(
        e,
        "0:set:crate2/lid.Health.hp=25;0:save_scene".into(),
    ));
    step(e, 2);
    assert_eq!(
        scene(&root),
        YARD.replace(
            "  prefab \"prefabs/crate.ogt\"\n}\n",
            "  prefab \"prefabs/crate.ogt\"\n  override \"lid\" {\n    Health {\n      hp 25\n    }\n  }\n}\n"
        )
    );
    // dump_entity says where it came from and what its instance overrides.
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(
        e,
        Some(json!({ "what": "entity", "id": "crate2/lid" }).to_string()),
    ))
    .unwrap();
    assert_eq!(
        dump["prefab"]["prefab"],
        json!("prefabs/crate.ogt"),
        "{dump}"
    );
    assert_eq!(dump["prefab"]["node"], json!("lid"));
    assert_eq!(dump["prefab"]["overrides"], json!(["Health.hp"]));
    assert_eq!(dump["prefab"]["root"], json!("crate2"));
    // describe lists the instances by prefab.
    let described: Value = serde_json::from_str(&og_engine_probe_describe(e)).unwrap();
    assert_eq!(
        described["prefabs"],
        json!([{ "prefab": "prefabs/crate.ogt", "instances": ["crate1", "crate2"] }]),
        "{described}"
    );
    // Reverted: the scene's text is as it was, the live value the prefab's.
    value(og_commands_schedule(e, "0:revert_prefab:crate2/lid".into()));
    step(e, 2);
    assert_eq!(scene(&root), YARD);
    assert_eq!(hp(e, "crate2/lid"), 10.0);
    // Applied: the prefab's lid takes 40, the instance's override goes.
    value(og_commands_schedule(
        e,
        "0:apply_prefab:crate1/lid:Health.hp".into(),
    ));
    step(e, 2);
    assert_eq!(
        prefab(&root),
        CRATE.replace(
            "    Health {\n      hp 10\n    }\n  }",
            "    Health {\n      hp 40\n    }\n  }"
        )
    );
    assert_eq!(
        scene(&root),
        YARD.replace(
            "  override \"lid\" {\n    Health {\n      hp 40\n    }\n  }\n",
            ""
        )
    );
    // Re-cooked: the first crate as before, the other takes the new value.
    cook(&root);
    let again = placed(&root);
    assert_eq!(hp(again, "crate1/lid"), 40.0);
    assert_eq!(hp(again, "crate2/lid"), 40.0);
    assert_eq!(hp(again, "crate1"), 10.0);
}
