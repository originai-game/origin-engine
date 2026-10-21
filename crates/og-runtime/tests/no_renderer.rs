//! The runtime without a renderer (the platform's `engine` and `probe`
//! sandboxes carry none, docs/design.md §5): the simulation, the probe, the
//! dumps and the world hash are the same as with one, because rendering
//! never feeds back into simulation; a `screenshot` step writes no PNG and
//! logs, without a warning, the frame it did not take; `hello` says the tier
//! is `none` and `capture` answers null.
//!
//! Two engines run one script: one with a renderer that draws nothing and
//! answers every capture with a 2x2 frame, one with none (its play surface
//! set to the renderer's size, as the native host does when it finds no
//! adapter). A floor, boxes dropped from poses the world's Rng chose, and
//! sprites give physics, the Rng and the extract something to do.
//!
//! Ways this could fail: a screenshot without a renderer warns (so a
//! playtest checking "no warnings" fails) or is dropped silently; the run
//! stops or panics at it; the world hash, a dump or the UI's play surface
//! differs between the two (the extract or the capture wrote into the
//! world, or the surface came only from the renderer); the snapshot and
//! restore of a world without a renderer does not replay to the same hash.
//! Broken before this test by the first: "no renderer in this host: nothing
//! to capture" was a warning.

use og_render::{RenderScene, RgbaImage};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_probe_capture, og_engine_probe_commands, og_engine_probe_dump, og_engine_probe_hello,
    og_engine_probe_restore, og_engine_probe_snapshot, og_engine_take_log, og_engine_take_outputs,
    og_game_game, og_host_begin_bundle, og_ui_viewport, og_world_insert, og_world_spawn, set_hooks,
    set_render_backend, set_viewport,
};
use serde_json::{Value, json};
use std::rc::Rc;

struct NoAssets;

impl Hooks for NoAssets {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets in this test"))
    }
}

/// A renderer that draws nothing and captures a 2x2 grey frame.
struct Blank;

impl RenderBackend for Blank {
    fn tier(&self) -> String {
        "test".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, _: &RenderScene) -> Result<(), String> {
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, width: u32, height: u32) -> Option<RgbaImage> {
        let (w, h) = if width == 0 { (2, 2) } else { (width, height) };
        Some(RgbaImage {
            width: w,
            height: h,
            pixels: vec![128; (w * h * 4) as usize],
        })
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

/// A probe member's answer (not a Result).
fn raw(text: String) -> Value {
    serde_json::from_str(&text).expect("JSON")
}

/// Spawns an entity and answers its probe id (`#<index>.<generation>`).
fn spawn(engine: u32, at: [f32; 3], components: &[(&str, Value)]) -> String {
    let desc = json!({ "position": { "x": at[0], "y": at[1], "z": at[2] } });
    let e = value(og_world_spawn(engine, Some(desc.to_string())))
        .as_f64()
        .unwrap();
    for (name, v) in components {
        value(og_world_insert(
            engine,
            e,
            name.to_string(),
            Some(v.to_string()),
        ));
    }
    let bits = e as u64;
    format!("#{}.{}", bits & 0xffff_ffff, bits >> 32)
}

const SCRIPT: &str = "0.1:seed:7;0.2:screenshot;0.3:screenshot:shot.png;0.35:screenshot:small.png:64;\
0.4:dump_world:world.json;0.5:dump_entity:{box}:box.json;0.6:dump_ui:ui.json;0.7:quit";

/// What one run of the script left: its world hash and tick, its outputs
/// by file (dumps as JSON without the systems' wall-clock timings), and its log.
struct Run {
    engine: u32,
    hash: String,
    tick: u64,
    outputs: Vec<(String, String, Value)>,
    log: Vec<Value>,
}

fn run(renderer: bool, script: &str) -> Run {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":3}"#.into()));
    let engine = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    if renderer {
        set_render_backend(engine, Some(Box::new(Blank))).unwrap();
    } else {
        // What og-host-node's og_node_attach_offscreen does when it finds no adapter.
        set_viewport(engine, json!({ "width": 320, "height": 180 })).unwrap();
    }
    spawn(
        engine,
        [0.0, -0.5, 0.0],
        &[
            ("og.RigidBody", json!({ "kind": 1 })),
            (
                "og.Collider",
                json!({ "shape": 0, "size": { "x": 20, "y": 1, "z": 20 } }),
            ),
        ],
    );
    let mut last = String::new();
    for i in 0..4 {
        let x = i as f32 * 0.7 - 1.0;
        last = spawn(
            engine,
            [x, 2.0 + i as f32, 0.2 * i as f32],
            &[
                ("og.RigidBody", json!({})),
                (
                    "og.Collider",
                    json!({ "shape": 0, "size": { "x": 0.5, "y": 0.5, "z": 0.5 } }),
                ),
                ("og.Sprite", json!({ "width": 1, "height": 1 })),
            ],
        );
    }
    let script = script.replace("{box}", &last);
    let report = raw(og_engine_probe_commands(engine, script));
    assert!(report["scheduled"].as_u64().unwrap() > 0, "{report}");
    for _ in 0..120 {
        let answer = value(og_engine_frame(engine, 1000.0 / 60.0));
        if answer["quit"] == json!(true) {
            break;
        }
    }
    let world = raw(og_engine_probe_dump(engine, None));
    let outputs = raw(og_engine_take_outputs(engine))
        .as_array()
        .unwrap()
        .iter()
        .map(|o| {
            let kind = o["kind"].as_str().unwrap().to_string();
            let data = match kind.as_str() {
                "json" => {
                    let mut v: Value = serde_json::from_str(o["data"].as_str().unwrap()).unwrap();
                    if let Some(systems) = v["systems"].as_array_mut() {
                        for s in systems {
                            s.as_object_mut().unwrap().remove("totalMs");
                        }
                    }
                    v
                }
                _ => json!(o["data"].as_str().unwrap().len()),
            };
            (o["file"].as_str().unwrap().to_string(), kind, data)
        })
        .collect();
    let log = raw(og_engine_take_log(engine)).as_array().unwrap().clone();
    Run {
        engine,
        hash: world["hash"].as_str().unwrap().to_string(),
        tick: world["tick"].as_u64().unwrap(),
        outputs,
        log,
    }
}

#[test]
fn without_a_renderer_the_run_is_the_same_and_its_frames_are_unchecked() {
    let with = run(true, SCRIPT);
    let without = run(false, SCRIPT);

    // The same run: tick, world hash, every dump, the play surface.
    assert!(
        with.tick > 30,
        "the script ran to its quit (tick {})",
        with.tick
    );
    assert_eq!(without.tick, with.tick);
    assert_eq!(
        without.hash, with.hash,
        "rendering never feeds back into simulation"
    );
    let dumps = |r: &Run| -> Vec<(String, Value)> {
        r.outputs
            .iter()
            .filter(|(_, kind, _)| kind == "json")
            .map(|(f, _, v)| (f.clone(), v.clone()))
            .collect()
    };
    assert_eq!(
        dumps(&without)
            .iter()
            .map(|(f, _)| f.as_str())
            .collect::<Vec<_>>(),
        ["world.json", "box.json", "ui.json"]
    );
    assert_eq!(
        dumps(&without),
        dumps(&with),
        "dump_world, dump_entity and dump_ui"
    );
    assert_eq!(
        og_ui_viewport(without.engine),
        og_ui_viewport(with.engine),
        "the UI lays out on the same play surface"
    );

    // With a renderer: three PNGs and no log line about frames.
    let pngs = |r: &Run| -> Vec<String> {
        r.outputs
            .iter()
            .filter(|(_, kind, _)| kind == "png")
            .map(|(f, _, _)| f.clone())
            .collect()
    };
    assert_eq!(
        pngs(&with),
        ["screenshot-000012.png", "shot.png", "small.png"]
    );
    // Without: none, one note per screenshot naming its file, and no warning.
    assert!(pngs(&without).is_empty());
    let warnings: Vec<&Value> = without
        .log
        .iter()
        .filter(|l| l["warning"] == json!(true))
        .collect();
    assert!(warnings.is_empty(), "no warnings: {warnings:?}");
    let notes: Vec<&str> = without
        .log
        .iter()
        .filter_map(|l| l["text"].as_str())
        .filter(|t| t.contains(og_probe::FRAME_UNCHECKED))
        .collect();
    assert_eq!(
        notes,
        [
            "0.2s screenshot: no renderer, frame unchecked (screenshot-000012.png)",
            "0.3s screenshot: no renderer, frame unchecked (shot.png)",
            "0.35s screenshot: no renderer, frame unchecked (small.png)",
        ]
    );
    assert!(
        !with
            .log
            .iter()
            .any(|l| l["text"].as_str().unwrap_or("").contains("screenshot")),
        "{:?}",
        with.log
    );

    // The probe says so: hello's tier, capture's null.
    assert_eq!(
        raw(og_engine_probe_hello(without.engine))["tier"],
        json!("none")
    );
    assert_eq!(
        raw(og_engine_probe_hello(with.engine))["tier"],
        json!("test")
    );
    assert_eq!(og_engine_probe_capture(without.engine, None, None), "null");
    let frame = raw(og_engine_probe_capture(with.engine, Some(4), Some(4)));
    assert_eq!(frame["width"], json!(4));
}

#[test]
fn a_world_without_a_renderer_replays_from_its_snapshot() {
    // No quit: the clock keeps going after the script.
    let r = run(false, "0.1:seed:7;0.2:screenshot;0.5:advance:1");
    let snap = raw(og_engine_probe_snapshot(r.engine));
    let id = snap["id"].as_str().expect("a snapshot id").to_string();
    for _ in 0..30 {
        value(og_engine_frame(r.engine, 1000.0 / 60.0));
    }
    let first = raw(og_engine_probe_dump(r.engine, None))["hash"].clone();
    assert_ne!(first, snap["hash"], "the world moved after the snapshot");
    assert!(og_engine_probe_restore(r.engine, id));
    for _ in 0..30 {
        value(og_engine_frame(r.engine, 1000.0 / 60.0));
    }
    let again = raw(og_engine_probe_dump(r.engine, None))["hash"].clone();
    assert_eq!(
        again, first,
        "the same 30 ticks from the snapshot land on the same hash"
    );
}
