//! A model's level of detail follows how much of the screen it covers
//! (og_render::lod), for a skinned model as for a plain one.
//!
//! The strider is a skinned glTF whose mesh node sits under an `Armature`
//! scaled 0.01 over vertices in metres (a common exporter layout): glTF
//! ignores a skinned mesh node's own transform, the joints place its
//! vertices (2.15 m tall at rest). The walker is skinned at scale 1, so
//! the node and its bind pose agree. Both are drawn here without a
//! skeleton, the static fallback a posed model takes when its pose is
//! missing.
//!
//! Ways this could fail: the model's bounds, its levels' errors or the
//! drawn part take the node's transform instead of the bind pose (the old
//! code: the strider's errors a hundredth of their size, so it picked its
//! coarsest level while filling the screen, and its static draw was 2 cm
//! tall); or the levels stop coarsening with distance.

use og_core::glam::Vec3;
use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_probe_commands, og_engine_take_outputs, og_engine_write_resource, og_game_game,
    og_host_begin_bundle, og_world_insert, og_world_spawn, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::Cell;
use std::rc::Rc;

/// The view's height in pixels (the renderer's size).
const VIEW: f32 = 108.0;

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

/// A project with the template's strider and walker, cooked once.
fn project() -> &'static std::path::Path {
    static ROOT: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    ROOT.get_or_init(|| {
        let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-model-lod-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        for m in ["strider", "walker"] {
            let to = root.join(format!("assets/models/{m}.glb"));
            std::fs::create_dir_all(to.parent().unwrap()).unwrap();
            let from = repo.join(format!("tests/fixtures/motion/assets/models/{m}.glb"));
            std::fs::copy(from, &to).unwrap();
        }
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
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

/// Keeps the height of everything the last frame drew without a pose.
struct Heights(Rc<Cell<f32>>);

impl RenderBackend for Heights {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (192, VIEW as u32)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for i in &scene.objects.instances {
            let Some((a, b)) = scene.mesh(i.mesh).and_then(|m| m.bounds()) else {
                continue;
            };
            for c in 0..8 {
                let p = Vec3::new(
                    if c & 1 == 0 { a.x } else { b.x },
                    if c & 2 == 0 { a.y } else { b.y },
                    if c & 4 == 0 { a.z } else { b.z },
                );
                let y = i.transform.transform_point3(p).y;
                lo = lo.min(y);
                hi = hi.max(y);
            }
        }
        self.0.set(if hi >= lo { hi - lo } else { 0.0 });
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
    }
}

/// `model`, `height` metres tall, seen covering `cover` of the view's
/// height: the level it drew and how tall its static draw was.
fn seen(model: &str, height: f32, cover: f32) -> (u64, f32) {
    let root = project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let e = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let drawn = Rc::new(Cell::new(0.0));
    set_render_backend(e, Some(Box::new(Heights(drawn.clone())))).unwrap();
    let asset = value(og_runtime::og_load_gltf(
        e,
        format!("models/{model}.glb"),
        None,
    ));
    let id = value(og_world_spawn(e, Some(json!({ "name": "it" }).to_string())))
        .as_f64()
        .unwrap();
    let m = json!({ "asset": asset }).to_string();
    value(og_world_insert(e, id, "og.Model".into(), Some(m)));
    // fov 50°: at distance d the view is 2 d tan 25° tall (tan 25° = 0.466 307 66).
    let d = height / cover / (2.0 * 0.466_307_66);
    let camera = json!({
        "position": { "x": 0, "y": height * 0.5, "z": d },
        "quaternion": { "x": 0, "y": 0, "z": 0, "w": 1 },
        "fov": 50,
    });
    value(og_engine_write_resource(
        e,
        "og.Camera".into(),
        camera.to_string(),
    ));
    for _ in 0..3 {
        let a: Value = serde_json::from_str(&og_engine_frame(e, 1000.0 / 60.0)).unwrap();
        assert_eq!(a["ok"], json!(true), "{a}");
    }
    let r: Value =
        serde_json::from_str(&og_engine_probe_commands(e, "0:dump_entity:it".into())).unwrap();
    assert_eq!(r["scheduled"], json!(1), "{r}");
    let a: Value = serde_json::from_str(&og_engine_frame(e, 1000.0 / 60.0)).unwrap();
    assert_eq!(a["ok"], json!(true), "{a}");
    let outputs: Value = serde_json::from_str(&og_engine_take_outputs(e)).unwrap();
    let dump = outputs
        .as_array()
        .and_then(|o| o.iter().find(|o| o["kind"] == "json"))
        .unwrap_or_else(|| panic!("no dump: {outputs}"));
    let dump: Value = serde_json::from_str(dump["data"].as_str().unwrap()).unwrap();
    let level = dump["render"]["lod"]
        .as_u64()
        .unwrap_or_else(|| panic!("no level: {dump}"));
    (level, drawn.get())
}

#[test]
fn a_model_filling_the_screen_draws_its_full_mesh() {
    for (model, height) in [("strider", 2.15), ("walker", 1.72)] {
        // 80 % of the view's height: the full mesh, at its true size.
        let (level, drawn) = seen(model, height, 0.8);
        assert_eq!(
            level, 0,
            "{model} filling 80 % of the view drew level {level}"
        );
        assert!(
            (drawn - height).abs() < 0.1 * height,
            "{model}: drawn {drawn} m tall, not {height} m"
        );
        // A few pixels tall: a coarser level.
        let (far, _) = seen(model, height, 4.0 / VIEW);
        assert!(far > 0, "{model} 4 px tall still drew level 0");
    }
}
