//! og.Garment through the runtime (R9 cloth): a posed model's mesh named
//! by og.Garment is simulated by og.physics from the skin's pose each tick
//! and drawn at the simulated points.
//!
//! Ways this could fail: the pose system never runs (no points, or points
//! that never leave the bind pose while the model walks); the drawing
//! keeps the skinned mesh (no deformed vertices, or vertices that do not
//! match the mesh); the garment is drawn through the skinned copy with
//! weights, so the palette moves it twice; two runs of the same inputs
//! differ.

use og_render::RenderScene;
use og_runtime::{
    Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_load_animation, og_load_gltf, og_physics_garment_points,
    og_world_insert, og_world_instantiate, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-garment-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let walker = std::fs::read(repo.join("templates/game/assets/models/walker.glb")).unwrap();
        let p = root.join("assets/models/walker.glb");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, walker).unwrap();
        let options = og_cook::CookOptions {
            threads: 1,
            ..og_cook::CookOptions::default()
        };
        let report = og_cook::cook_project(&root, &options).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

struct Disk;

impl Hooks for Disk {
    fn dispatch(&self, _call: &og_runtime::Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
}

/// The last frame's skinned instances: whether each brings its own
/// vertices, and if so whether they match its mesh's vertex count and
/// its skin weights are all zero.
#[derive(Default)]
struct Seen {
    deformed: Vec<(usize, bool)>,
    plain: usize,
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
        let mut s = self.0.borrow_mut();
        s.deformed.clear();
        s.plain = 0;
        for i in &scene.skinned_instances {
            match i.deformed {
                Some(d) => {
                    let v = &scene.deformed_vertices[d as usize];
                    let n = scene.mesh(i.mesh).map_or(0, |m| m.vertices.len());
                    let weightless = scene
                        .skin(i.mesh)
                        .is_some_and(|s| s.weights.iter().all(|w| *w == [0.0; 4]));
                    s.deformed
                        .push((v.positions.len(), n == v.positions.len() && weightless));
                }
                None => s.plain += 1,
            }
        }
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

/// A walking model with og.Garment on its first mesh; the garment's
/// points after each of `frames` frames, and what the last frame drew.
fn run(frames: usize) -> (Vec<Vec<[f64; 3]>>, Seen) {
    let root = project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(Seen::default()));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    let anim = value(og_load_animation(engine, "models/walker.glb".into()));
    let clip = anim["clips"].as_array().unwrap()[0]["clip"].clone();
    let asset = value(og_load_gltf(engine, "models/walker.glb".into(), None));
    let e = value(og_world_instantiate(engine, asset.as_f64().unwrap(), None))
        .as_f64()
        .unwrap();
    for (name, v) in [
        ("og.Skeleton", json!({ "asset": anim["skeleton"] })),
        ("og.AnimationPlayer", json!({ "clip": clip, "loop": true })),
        ("og.Garment", json!({ "mesh": 0, "maxDistance": 0.1 })),
    ] {
        value(og_world_insert(engine, e, name.into(), Some(v.to_string())));
    }
    let mut out = Vec::new();
    for _ in 0..frames {
        value(og_engine_frame(engine, 1000.0 / 60.0));
        let text = og_physics_garment_points(engine, e);
        let v: Value = serde_json::from_str(&text).unwrap();
        out.push(
            v["value"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|p| ["x", "y", "z"].map(|k| p[k].as_f64().unwrap()))
                        .collect()
                })
                .unwrap_or_default(),
        );
    }
    let seen = std::mem::take(&mut *seen.borrow_mut());
    (out, seen)
}

#[test]
fn a_garment_follows_the_skin_and_is_drawn_at_its_points() {
    let (points, seen) = run(30);
    let last = points.last().unwrap();
    assert!(!last.is_empty(), "the garment has points");
    // The walk moves them.
    let first = points.iter().find(|p| p.len() == last.len()).unwrap();
    let moved = first
        .iter()
        .zip(last)
        .map(|(a, b)| (0..3).map(|k| (a[k] - b[k]).abs()).fold(0.0, f64::max))
        .fold(0.0, f64::max);
    assert!(moved > 1e-3, "the points stayed put: {moved}");
    assert!(
        !seen.deformed.is_empty(),
        "the garment is drawn at its points"
    );
    assert!(
        seen.deformed.iter().all(|&(n, ok)| n > 0 && ok),
        "{:?}",
        seen.deformed
    );
    // The same inputs, the same points.
    let (again, _) = run(30);
    assert_eq!(points, again);
}
