//! The play gate's warm-up (og_runtime::warm_tick, also run by
//! og.render.compile): outside a tick it builds the game's UI documents on
//! a copy of the UI, draws that copy once in the UI's place, and has physics
//! build its height fields' chunk shapes, so the first played frame does
//! not pay for them.
//!
//! Two engines run the same world: a screen document, a collision terrain of
//! two chunks and boxes dropped onto it. One is warmed before its first
//! tick, and again between ticks; the other never is.
//!
//! Ways this could fail: the warm-up builds the real UI instead of a copy
//! (its mounts, triggers, stores, control zones or doc events change, so
//! the UI dump or the snapshot's presentation differs), or does not put the
//! UI back after drawing the copy; it moves the physics (a collider inserted out of
//! order, a chunk shape that is not the step's own), so the boxes land
//! elsewhere and the world hash differs; or it touches what a snapshot
//! keeps, so the snapshots differ.
#![cfg(not(feature = "profile2d"))]

use og_render::{RenderScene, RgbaImage};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_engine_probe_dump,
    og_engine_probe_snapshot, og_game_game, og_host_begin_bundle, og_terrain_load, og_world_insert,
    og_world_spawn, set_hooks, set_render_backend, warm_tick,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

/// The project, cooked once per test process.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-warm-tick-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, bytes: &[u8]| {
            let p = root.join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        };
        // 129 x 33 samples: two chunks along x, a slope down towards +x.
        let (w, h) = (129u32, 33u32);
        let rgba: Vec<u8> = (0..h)
            .flat_map(|z| {
                (0..w).flat_map(move |x| {
                    let v = (255 - x * 255 / (w - 1)).min(255) as u8 / 2 + (z % 3) as u8;
                    [v, v, v, 255]
                })
            })
            .collect();
        let png = og_runtime::png(&og_render::RgbaImage {
            width: w,
            height: h,
            pixels: rgba,
        });
        write("assets/terrain/hill.png", &png);
        write(
            "assets/terrain/hill.ogt",
            b"type terrain\nversion 1\n---\nheightmap \"hill.png\"\nheight 0 8\nsize 64 16\n",
        );
        write(
            "assets/ui/hud.svelte",
            br#"<meta name="screen" content="Hud" />
<div id="Hud"><p id="Clock">06:54</p><p>Summer - Dawn</p>
<button id="Use">Use</button></div>
<style>
#Hud { display: flex; flex-direction: column; gap: 4px; padding: 12px; }
#Use { border-radius: 6px; background-color: #335; color: white; }
</style>
"#,
        );
        let project = "type project\nversion 1\n---\nname \"warm tick\"\nid \"warm-tick\"\nentry \"main.ts\"\n";
        write("project.ogt", project.as_bytes());
        write("main.ts", b"");
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

/// A renderer that draws nothing: the warm-up's frame goes through the
/// whole frame path but the GPU.
struct Blank;

impl RenderBackend for Blank {
    fn tier(&self) -> String {
        "test".into()
    }
    fn size(&self) -> (u32, u32) {
        (640, 360)
    }
    fn render(&mut self, _: &RenderScene) -> Result<(), String> {
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<RgbaImage> {
        None
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

fn raw(text: String) -> Value {
    serde_json::from_str(&text).expect("JSON")
}

fn spawn(engine: u32, at: [f32; 3], components: &[(&str, Value)]) {
    let desc = json!({ "position": { "x": at[0], "y": at[1], "z": at[2] } });
    let e = value(og_runtime::host_impl::answer(og_world_spawn(
        engine,
        Some(desc.to_string()),
    )))
    .as_f64()
    .unwrap();
    for (name, v) in components {
        value(og_runtime::host_impl::answer(og_world_insert(
            engine,
            e,
            name.to_string(),
            Some(v.to_string()),
        )));
    }
}

/// What one run left: the world hash after each frame, the UI's dump and
/// the snapshot taken at the end (its id left out).
struct Run {
    hashes: Vec<String>,
    ui: Value,
    snapshot: Value,
}

fn run(warm: bool) -> Run {
    let root = project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":5}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let e = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    set_render_backend(e, Some(Box::new(Blank))).unwrap();
    let info = value(og_runtime::host_impl::answer(og_terrain_load(
        e,
        "terrain/hill.ogt".into(),
    )));
    spawn(
        e,
        [0.0, 0.0, 0.0],
        &[
            ("og.Terrain", json!({ "data": info["data"] })),
            (
                "og.Collider",
                json!({ "shape": "heightfield", "mesh": info["geometry"] }),
            ),
        ],
    );
    for i in 0..6 {
        let x = i as f32 * 9.0 - 24.0;
        spawn(
            e,
            [x, 10.0 + i as f32, 0.5 * i as f32],
            &[
                ("og.RigidBody", json!({})),
                (
                    "og.Collider",
                    json!({ "shape": "box", "size": { "x": 0.6, "y": 0.6, "z": 0.6 } }),
                ),
            ],
        );
    }
    value(og_runtime::host_impl::answer(og_runtime::og_load_document(
        e,
        "ui/hud.svelte".into(),
    )));
    value(og_runtime::host_impl::answer(og_runtime::og_ui_push(
        e,
        "Hud".into(),
        None,
    )));
    if warm {
        warm_tick(e);
    }
    let mut hashes = Vec::new();
    for i in 0..90 {
        value(og_engine_frame(e, 1000.0 / 60.0));
        // Again mid-run, as og.render.compile behind a level's loading screen.
        if warm && i == 30 {
            warm_tick(e);
        }
        let world = raw(og_engine_probe_dump(e, None));
        hashes.push(world["hash"].as_str().unwrap().to_string());
    }
    let ui = raw(og_engine_probe_dump(e, Some(r#"{"what":"ui"}"#.into())));
    let mut snapshot = raw(og_engine_probe_snapshot(e));
    snapshot.as_object_mut().map(|o| o.remove("id"));
    Run {
        hashes,
        ui,
        snapshot,
    }
}

#[test]
fn a_warmed_world_ticks_hashes_and_snapshots_as_one_never_warmed() {
    let cold = run(false);
    let warmed = run(true);
    // The boxes fell onto the terrain: the step had height-field chunks to collide with.
    assert_ne!(cold.hashes[0], cold.hashes[89], "the world moved");
    assert_eq!(warmed.hashes, cold.hashes, "the world hash, frame by frame");
    assert_eq!(warmed.ui, cold.ui, "the UI as a tick left it");
    assert_eq!(warmed.snapshot, cold.snapshot, "the snapshot");
}
