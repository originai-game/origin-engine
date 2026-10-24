//! The 2D profile's runtime (`--features profile2d`: the 2D web modules)
//! leaves the terrain out with the rest of the 3D world: every og.terrain
//! call, on a terrain the full build loads, answers the refusal naming
//! RenderCapabilities.world3d, and so does a SHAPE_HEIGHTFIELD collider.
//!
//! Ways this could fail: a call still loads, samples or edits (the module
//! is still built); a call answers some other error (a missing file, a
//! dead handle) that does not say why; the collider is refused as a
//! missing registration instead of the profile, or not at all.
#![cfg(feature = "profile2d")]

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_take_log, og_game_game,
    og_host_begin_bundle, og_terrain_bounds, og_terrain_edit, og_terrain_layer, og_terrain_load,
    og_terrain_sample, og_terrain_set_layer, og_world_insert, og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::rc::Rc;

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

/// A cooked 9 × 9 terrain with a collider, as a 3D game ships it.
fn project() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-profile2d-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let write = |path: &str, bytes: &[u8]| {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    };
    let png = og_runtime::png(&og_render::RgbaImage {
        width: 9,
        height: 9,
        pixels: (0..81)
            .flat_map(|i| [i as u8, i as u8, i as u8, 255])
            .collect(),
    });
    write("terrain/hill.png", &png);
    write(
        "terrain/hill.ogt",
        b"type terrain\nversion 1\n---\nheightmap \"hill.png\"\nheight 0 4\nsize 16 16\n",
    );
    let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    root
}

fn wire(text: &str) -> Value {
    serde_json::from_str(text).expect("wire JSON")
}

/// The answer is the refusal that names the profile's missing 3D world.
fn refused(what: &str, text: String) {
    let v = wire(&text);
    assert_eq!(v["ok"], json!(false), "{what}: {text}");
    let reason = v["reason"].as_str().unwrap_or_default();
    assert!(
        reason.contains("terrain is not built into this module")
            && reason.contains("RenderCapabilities.world3d is false"),
        "{what}: {text}"
    );
}

#[test]
fn the_terrain_and_its_height_fields_answer_the_profile() {
    let root = project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = wire(&og_engine_create(game, options))["value"]
        .as_u64()
        .unwrap() as u32;
    wire(&og_engine_frame(engine, 1000.0 / 60.0));

    refused("load", og_terrain_load(engine, "terrain/hill.ogt".into()));
    // The calls on a terrain's data: refused for the profile, not as a
    // handle that is not alive.
    let data = 1.0;
    let at = json!({ "data": data, "x": 1, "z": 2 }).to_string();
    refused("sample", og_terrain_sample(engine, Some(at)));
    refused("bounds", og_terrain_bounds(engine, data, None, None, None));
    let brush = json!({ "x": 0, "z": 0, "radius": 2, "strength": 1 }).to_string();
    refused("edit", og_terrain_edit(engine, data, Some(brush)));
    refused("layer", og_terrain_layer(engine, data, 0));
    refused(
        "setLayer",
        og_terrain_set_layer(engine, data, 0, Some("{}".into())),
    );

    // A SHAPE_HEIGHTFIELD collider naming a terrain's geometry.
    let e = wire(&og_world_spawn(engine, None))["value"]
        .as_f64()
        .unwrap();
    let collider = json!({ "shape": 6, "mesh": (0x7eu64 << 32) as f64 });
    let inserted = og_world_insert(engine, e, "og.Collider".into(), Some(collider.to_string()));
    assert_eq!(wire(&inserted)["ok"], json!(true), "{inserted}");
    let frame = og_engine_frame(engine, 1000.0 / 60.0);
    let log = og_engine_take_log(engine);
    let said = format!("{frame}\n{log}");
    assert!(
        said.contains("og.Collider.shape 6 (SHAPE_HEIGHTFIELD, a terrain's height field)")
            && said.contains("RenderCapabilities.world3d is false"),
        "{said}"
    );
    let _ = std::fs::remove_dir_all(&root);
}
