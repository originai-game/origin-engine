//! The inspector's Save (G3-7): a field edited in its panel, by the
//! pointer and the keyboard as a person edits it, is written back by
//! `save_scene` as that one line of the scene's text, and the scene cooked
//! again loads the world the edit made.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_commands_schedule, og_engine_create, og_engine_frame,
    og_engine_probe_dump, og_engine_take_outputs, og_game_game, og_host_begin_bundle,
    og_load_scene, og_reflect_define_component, og_world_instantiate, set_hooks,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::rc::Rc;

const YARD: &str = "type scene
version 1
---
# The yard, for the inspector's save.
name \"Yard\"
entity \"crate\" {
  Transform {
    position -1 0.5 0 # dropped from here
  }
  Health {
    hp 10
    armor 2
  }
}
entity \"post\" {
  Health {
    hp 7
  }
}
";

fn cook(root: &Path) {
    use og_core::reflect::{FieldSchema, FieldType, Schema};
    let health = Schema {
        name: "Health".into(),
        version: 1,
        doc: String::new(),
        fields: vec![
            FieldSchema::new("hp", FieldType::F32),
            FieldSchema::new("armor", FieldType::F32),
        ],
    };
    let options = og_cook::CookOptions {
        threads: 1,
        schemas: vec![health],
        ..og_cook::CookOptions::default()
    };
    let report = og_cook::cook_project(root, &options).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
}

fn project() -> PathBuf {
    // One directory a call: the tests run beside each other.
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-inspector-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    std::fs::write(root.join("assets/scenes/yard.ogt"), YARD).unwrap();
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

/// A game declaring Health, the yard placed.
fn engine(root: &Path) -> u32 {
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    value(og_runtime::host_impl::answer(og_reflect_define_component(
        "Health".into(),
        json!([
            { "name": "hp", "kind": "f32", "default": 0, "options": {} },
            { "name": "armor", "kind": "f32", "default": 0, "options": {} },
        ])
        .to_string(),
    )));
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let e = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let scene = value(og_runtime::host_impl::answer(og_load_scene(
        e,
        "scenes/yard.ogt".into(),
    )));
    value(og_runtime::host_impl::answer(og_world_instantiate(
        e,
        scene.as_f64().unwrap(),
        None,
    )));
    e
}

fn step(e: u32, n: u32) {
    for _ in 0..n {
        value(og_engine_frame(e, 1000.0 / 60.0));
    }
}

/// The inspector as data (`dump_inspector`, which runs at the next tick).
fn panel(e: u32) -> Value {
    commands(e, "0:dump_inspector", 1);
    let outputs: Value = serde_json::from_str(&og_engine_take_outputs(e)).unwrap();
    let data = outputs
        .as_array()
        .and_then(|a| {
            a.iter()
                .rev()
                .find(|o| o["action"] == json!("dump_inspector"))
        })
        .and_then(|o| o["data"].as_str())
        .unwrap_or_else(|| panic!("no dump_inspector output: {outputs}"));
    serde_json::from_str(data).unwrap()
}

fn commands(e: u32, script: &str, ticks: u32) {
    value(og_runtime::host_impl::answer(og_commands_schedule(
        e,
        script.into(),
    )));
    step(e, ticks);
}

/// The centre of a panel element by its path, in CSS pixels.
fn centre(p: &Value, path: &str) -> String {
    let e = p["elements"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["path"] == json!(path))
        .unwrap_or_else(|| panic!("no element {path}"));
    let at = |k: &str, d: &str| e[k].as_f64().unwrap() + e[d].as_f64().unwrap() / 2.0;
    format!("{},{}", at("x", "width"), at("y", "height"))
}

/// A click at a panel element.
fn click(e: u32, p: &Value, path: &str) {
    let at = centre(p, path);
    commands(
        e,
        &format!("0:touch_move:{at};0.05:touch_start;0.1:touch_end"),
        10,
    );
}

fn entity(e: u32, id: &str) -> Value {
    let d: Value = serde_json::from_str(&og_engine_probe_dump(
        e,
        Some(json!({ "what": "entity", "id": id }).to_string()),
    ))
    .unwrap();
    d["components"].clone()
}

/// A gizmo handle's grab point on the surface, from `dump_inspector`.
fn handle(p: &Value, name: &str) -> (f64, f64) {
    let h = p["gizmo"]["handles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["handle"] == json!(name))
        .unwrap_or_else(|| panic!("no handle {name}: {}", p["gizmo"]));
    (h["x"].as_f64().unwrap(), h["y"].as_f64().unwrap())
}

fn position(e: u32) -> [f64; 3] {
    let p = &entity(e, "crate")["og.Transform"]["position"];
    let p: Vec<f64> = p
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    [p[0], p[1], p[2]]
}

/// A drag by the pointer from `from` to `to` in `steps` moves.
fn drag(e: u32, from: (f64, f64), to: (f64, f64), steps: u32) {
    let mut script = format!("0:touch_move:{},{};0.05:touch_start", from.0, from.1);
    for k in 1..=steps {
        let t = f64::from(k) / f64::from(steps);
        let at = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
        script.push_str(&format!(
            ";{}:touch_move:{},{}",
            0.05 + 0.05 * f64::from(k),
            at.0,
            at.1
        ));
    }
    script.push_str(&format!(";{}:touch_end", 0.1 + 0.05 * f64::from(steps)));
    commands(e, &script, 12 + 3 * steps);
}

/// og.Camera at (2, 3, 8), looking at the crate: no axis seen end on.
fn camera(e: u32) {
    value(og_runtime::og_engine_write_resource(
        e,
        "og.Camera".into(),
        json!({ "position": { "x": 2, "y": 3, "z": 8 } }).to_string(),
    ));
    og_runtime::og_camera_look_at(e, json!({ "x": -1, "y": 0.5, "z": 0 }).to_string());
}

#[test]
fn a_gizmo_drag_moves_along_its_axis_as_one_undo_step_and_saves_one_line() {
    let root = project();
    let e = engine(&root);
    camera(e);
    step(e, 2);
    commands(e, "0:inspect:crate", 3);
    let p = panel(e);
    assert_eq!(p["gizmo"]["mode"], json!("translate"));
    assert_eq!(p["gizmo"]["shown"], json!(true), "{}", p["gizmo"]);
    let start = position(e);
    let (o, x) = (handle(&p, "view"), handle(&p, "x"));
    // Pulled 60 px further along the x arrow as it lies on screen.
    let len = ((x.0 - o.0).powi(2) + (x.1 - o.1).powi(2)).sqrt();
    let dir = ((x.0 - o.0) / len, (x.1 - o.1) / len);
    let to = (x.0 + dir.0 * 60.0, x.1 + dir.1 * 60.0);
    drag(e, x, to, 4);
    let moved = position(e);
    assert!(moved[0] > start[0] + 0.1, "{start:?} -> {moved:?}");
    assert_eq!([moved[1], moved[2]], [start[1], start[2]], "only x moves");
    // The arrow followed the pointer: about 60 px on, and the pointer on
    // its line on screen (its grab point is a share of a length kept
    // constant on screen, so not the very point grabbed).
    let p = panel(e);
    let (o2, x2) = (handle(&p, "view"), handle(&p, "x"));
    let travel = ((x2.0 - x.0).powi(2) + (x2.1 - x.1).powi(2)).sqrt();
    assert!((travel - 60.0).abs() < 6.0, "{travel}");
    let (ax, ay) = (x2.0 - o2.0, x2.1 - o2.1);
    let off = ((to.0 - o2.0) * ay - (to.1 - o2.1) * ax).abs() / (ax * ax + ay * ay).sqrt();
    assert!(off < 1.0, "the pointer is {off} px off the arrow");
    assert_eq!(p["dirty"], json!(["crate.og.Transform.position"]));
    assert_eq!(p["gizmo"]["dragging"], Value::Null);
    // Four moves, one undo step.
    click(e, &p, "Inspector/Tools/Undo");
    assert_eq!(position(e), start, "one undo takes the whole drag back");
    let p = panel(e);
    click(e, &p, "Inspector/Tools/Redo");
    assert_eq!(position(e), moved);
    // With Ctrl held the move snaps to half units.
    let p = panel(e);
    let (o, x) = (handle(&p, "view"), handle(&p, "x"));
    let to = (x.0 + (x.0 - o.0) * 0.37, x.1 + (x.1 - o.1) * 0.37);
    commands(e, "0:key_down:ControlLeft", 2);
    drag(e, x, to, 3);
    commands(e, "0:key_up:ControlLeft", 2);
    let snapped = position(e);
    let d = snapped[0] - moved[0];
    assert!(d > 0.1 && (d / 0.5 - (d / 0.5).round()).abs() < 1e-4, "{d}");
    // Save writes the position's one line.
    let p = panel(e);
    click(e, &p, "Inspector/Bar/Save");
    let saved = std::fs::read_to_string(root.join("assets/scenes/yard.ogt")).unwrap();
    let line = saved
        .lines()
        .find(|l| l.trim_start().starts_with("position"))
        .unwrap();
    let x: f64 = line.split_whitespace().nth(1).unwrap().parse().unwrap();
    assert!((x - snapped[0]).abs() < 1e-4, "{line}");
    assert_eq!(saved.lines().count(), YARD.lines().count(), "{saved}");
}

const BOARD: &str = "type scene
version 1
---
name \"Board\"
resource Score {
  best 1 # the record
}
entity \"post\" {
  Health {
    hp 7
  }
}
";

#[test]
fn a_resource_edited_in_its_tab_saves_as_its_one_line() {
    use og_core::reflect::{FieldSchema, FieldType, Schema};
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-inspector-res-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    std::fs::write(root.join("assets/scenes/board.ogt"), BOARD).unwrap();
    let schema = |name: &str, field: &str| Schema {
        name: name.into(),
        version: 1,
        doc: String::new(),
        fields: vec![FieldSchema::new(field, FieldType::F32)],
    };
    let report = og_cook::cook_project(
        &root,
        &og_cook::CookOptions {
            threads: 1,
            schemas: vec![schema("Health", "hp"), schema("Score", "best")],
            ..og_cook::CookOptions::default()
        },
    )
    .unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    for (name, field, resource) in [("Health", "hp", false), ("Score", "best", true)] {
        let fields =
            json!([{ "name": field, "kind": "f32", "default": 0, "options": {} }]).to_string();
        value(if resource {
            og_runtime::host_impl::answer(og_runtime::og_reflect_define_resource(
                name.into(),
                fields,
            ))
        } else {
            og_runtime::host_impl::answer(og_reflect_define_component(name.into(), fields))
        });
    }
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let e = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let scene = value(og_runtime::host_impl::answer(og_load_scene(
        e,
        "scenes/board.ogt".into(),
    )));
    value(og_runtime::host_impl::answer(og_world_instantiate(
        e,
        scene.as_f64().unwrap(),
        None,
    )));
    step(e, 2);
    commands(e, "0:inspect:on", 3);
    let p = panel(e);
    click(e, &p, "Inspector/View/Strip/Track/Item#1");
    let p = panel(e);
    click(e, &p, "Inspector/Detail/S:Score");
    let p = panel(e);
    click(e, &p, "Inspector/Detail/F:Score.best/P0");
    commands(
        e,
        "0:key_down:End;0.02:key_up:End;0.04:key_down:Backspace;0.06:key_up:Backspace;0.15:type:9;0.3:key_down:Enter;0.33:key_up:Enter",
        25,
    );
    let p = panel(e);
    assert_eq!(p["dirty"], json!(["Score.best"]), "{}", p["status"]);
    click(e, &p, "Inspector/Bar/Save");
    let p = panel(e);
    assert_eq!(p["dirty"], json!([]), "{}", p["status"]);
    let saved = std::fs::read_to_string(root.join("assets/scenes/board.ogt")).unwrap();
    assert_eq!(
        saved,
        BOARD.replacen("  best 1 # the record\n", "  best 9 # the record\n", 1)
    );
}
