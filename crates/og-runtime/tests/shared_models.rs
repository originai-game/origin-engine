//! Models loaded from different files share what is identical in them:
//! two copies of one model draw one mesh (uploaded once), yet each keeps
//! materials of its own, so og.render.updateMaterial on one model's material
//! never shows on the other (the renderer batches the copies only while
//! their materials are equal: og-render's `RenderScene::alias_material`).
//!
//! Ways this could fail: the copies are handed the same material handle (an
//! edit of one then recolours both, as PR #8's s24 did), or the mesh is not
//! shared (each copy uploads its own).

use og_render::{RenderScene, StandardMaterial};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_load_gltf, og_load_gltf_info, og_render_update_material,
    og_world_insert, og_world_spawn, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

const MODEL: &str = "type model\nversion 1\n---\nmaterial \"paint\" {\n  color 0.2 0.5 0.8\n}\npart \"block\" {\n  material \"paint\"\n  boxGeometry {\n    width 1\n  }\n}\n";

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

/// Per instance of the last frame: its mesh, its material handle and what
/// that material draws as.
type Seen = Vec<(
    og_render::MeshHandle,
    og_render::MaterialHandle,
    StandardMaterial,
)>;

struct Recorder(Rc<RefCell<Seen>>);

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
            .map(|i| {
                (
                    i.mesh,
                    i.material,
                    *scene.standard_material(i.material).unwrap(),
                )
            })
            .collect();
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

#[test]
fn copies_of_a_model_share_their_mesh_but_not_their_materials() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-shared-models-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for name in ["a", "b"] {
        let p = root.join(format!("assets/models/{name}.ogt"));
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, MODEL).unwrap();
    }
    let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());

    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 1 }).to_string()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(Seen::new()));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    let mut materials = Vec::new();
    for name in ["a", "b"] {
        let asset = value(og_runtime::host_impl::answer(og_load_gltf(
            engine,
            format!("models/{name}.ogt"),
            None,
        )));
        materials.push(
            value(og_runtime::host_impl::answer(og_load_gltf_info(
                engine,
                asset.as_f64().unwrap(),
            )))["materials"][0]
                .clone(),
        );
        let e = value(og_runtime::host_impl::answer(og_world_spawn(
            engine,
            Some("{}".into()),
        )));
        let model = json!({ "asset": asset }).to_string();
        value(og_runtime::host_impl::answer(og_world_insert(
            engine,
            e.as_f64().unwrap(),
            "og.Model".into(),
            Some(model),
        )));
    }
    assert_ne!(
        materials[0], materials[1],
        "each model holds its own material"
    );
    value(og_engine_frame(engine, 1000.0 / 60.0));
    let before = {
        let s = seen.borrow();
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].0, s[1].0, "one mesh for both copies");
        assert_ne!(s[0].1, s[1].1);
        assert_eq!(s[0].2, s[1].2);
        s[0].2.color
    };

    // Recolour model a's material: model b keeps its colour.
    let red = json!({ "color": { "r": 1, "g": 0, "b": 0 } }).to_string();
    value(og_runtime::host_impl::answer(og_render_update_material(
        engine,
        materials[0].as_f64().unwrap(),
        Some(red),
    )));
    value(og_engine_frame(engine, 1000.0 / 60.0));
    let mut colours: Vec<[f32; 3]> = seen.borrow().iter().map(|i| i.2.color.to_array()).collect();
    colours.sort_by(|x, y| x.partial_cmp(y).unwrap());
    let mut want = [[1.0, 0.0, 0.0], before.to_array()];
    want.sort_by(|x, y| x.partial_cmp(y).unwrap());
    assert_eq!(colours, want, "one copy recoloured, the other as it was");
}
