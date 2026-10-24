//! A `type model` hot-reloads as a glTF does (`og dev`: the `.ogt` edited
//! and cooked, then the probe's `reload`): the asset keeps its handle and
//! its geometry is the edit's.
//!
//! Ways this could fail: the reload keys models by a `.glb` id or
//! extension and skips a model written as text; the old blob is kept
//! because the edit's output id or hash is compared otherwise.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_probe_reload,
    og_game_game, og_host_begin_bundle, og_load_gltf, og_load_gltf_info, og_render_geometry_info,
    set_hooks,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::rc::Rc;

fn model(width: f32) -> String {
    format!(
        "type model\nversion 1\n---\nmaterial \"paint\" {{\n  color 0.2 0.5 0.8\n}}\npart \"block\" {{\n  material \"paint\"\n  boxGeometry {{\n    width {width}\n  }}\n}}\n"
    )
}

/// Writes `models/block.ogt` and cooks the project (what `og dev` does on an edit).
fn write(root: &Path, text: &str) {
    let p = root.join("assets/models/block.ogt");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
    let report = og_cook::cook_project(root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
}

struct Game;

impl Hooks for Game {
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

/// The loaded model's first geometry's greatest x.
fn max_x(engine: u32, asset: f64) -> f64 {
    let info = value(og_load_gltf_info(engine, asset));
    let g = info["geometries"][0].as_f64().unwrap();
    value(og_render_geometry_info(engine, g))["max"]["x"]
        .as_f64()
        .unwrap()
}

#[test]
fn an_edited_type_model_loads_again_under_its_handle() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-model-reload-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    write(&root, &model(1.0));
    set_hooks(Rc::new(Game));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 1 }).to_string()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let asset = value(og_load_gltf(engine, "models/block.ogt".into(), None))
        .as_f64()
        .unwrap();
    value(og_engine_frame(engine, 1000.0 / 60.0));
    assert!((max_x(engine, asset) - 0.5).abs() < 1e-4);

    write(&root, &model(3.0));
    let answer: Value = serde_json::from_str(&og_engine_probe_reload(engine)).unwrap();
    assert!(answer.is_object(), "the reload swapped the world: {answer}");
    // The same handle answers the same path, and holds the edit.
    let again = value(og_load_gltf(engine, "models/block.ogt".into(), None))
        .as_f64()
        .unwrap();
    assert_eq!(again, asset);
    assert!(
        (max_x(engine, asset) - 1.5).abs() < 1e-4,
        "the edit did not load: {}",
        max_x(engine, asset)
    );
}
