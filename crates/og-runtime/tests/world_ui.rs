//! og.ui's world-space members through the engine (`docs/ui.md` §14): the
//! UI's end hands og-ui the tick's og.Camera and the pose of the entity an
//! anchor sits on, so the anchor lands where og.camera.worldToScreen puts
//! that entity and follows the camera when it moves; mass elements a system
//! spawns are counted and age out with the ticks.
//!
//! Ways it could fail: the end never sets the view (the anchor is hidden,
//! `inView` false), it reads the camera of the tick before or a stale
//! default (the anchor does not move with the camera), the entity's pose is
//! not handed in (an entity anchor is hidden), or spawned instances are
//! dropped between ticks.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_camera_world_to_screen, og_engine_create, og_engine_frame,
    og_engine_write_resource, og_game_game, og_host_begin_bundle, og_system_define_system,
    og_ui_close, og_ui_define_mass, og_ui_label, og_ui_mass_count, og_ui_open_world_anchor,
    og_ui_spawn_mass, og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const BUILD: u32 = 1;

#[derive(Default)]
struct Seen {
    entity: Cell<f64>,
    spawn: Cell<bool>,
    template: Cell<u32>,
    infos: RefCell<Vec<Value>>,
    counts: RefCell<Vec<u32>>,
}

struct Host(Rc<Seen>);

impl Hooks for Host {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind == 0 && call.id == BUILD {
            let s = &self.0;
            if s.template.get() == u32::MAX {
                s.template.set(og_ui_define_mass(
                    "damage".into(),
                    Some(json!({ "kind": "number", "lifetime": 10 }).to_string()),
                ));
            }
            if s.spawn.replace(false) {
                let list: Vec<Value> = (0..25)
                    .map(|i| json!({ "position": { "x": i, "y": 1, "z": 0 }, "value": i }))
                    .collect();
                og_ui_spawn_mass(s.template.get(), json!(list).to_string());
            }
            s.counts
                .borrow_mut()
                .push(og_ui_mass_count(s.template.get()));
            let desc = json!({
                "target": { "entity": s.entity.get() },
                "pivot": "center",
            });
            let info = og_ui_open_world_anchor("Tag".into(), Some(desc.to_string()));
            og_ui_label("Name".into(), "Brenna".into(), None);
            og_ui_close();
            s.infos
                .borrow_mut()
                .push(serde_json::from_str(&info).unwrap());
        }
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets"))
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

fn camera(e: u32, x: f64) {
    // Looking down -Z (the identity) from (x, 0, 5).
    let cam = json!({ "position": { "x": x, "y": 0, "z": 5 }, "fov": 50.0 });
    value(og_engine_write_resource(
        e,
        "og.Camera".into(),
        cam.to_string(),
    ));
}

#[test]
fn an_entity_anchor_follows_the_ticks_camera_and_mass_elements_age_out() {
    let seen = Rc::new(Seen::default());
    seen.template.set(u32::MAX);
    set_hooks(Rc::new(Host(seen.clone())));
    og_host_begin_bundle();
    let system = og_system_define_system(
        "BuildTags".into(),
        json!({ "reads": [], "writes": [] }).to_string(),
        BUILD,
    );
    let game = og_game_game(Some(json!({ "seed": 1, "systems": [system] }).to_string()));
    let e = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    let at = json!({ "position": { "x": 1.0, "y": 0.5, "z": 0.0 } });
    seen.entity.set(
        value(og_world_spawn(e, Some(at.to_string())))
            .as_f64()
            .unwrap(),
    );
    camera(e, 0.0);
    seen.spawn.set(true);
    let step = || value(og_engine_frame(e, 1000.0 / 60.0));
    step();
    step();
    // Where og.camera.worldToScreen puts the entity's point.
    let screen = |e: u32| -> (f64, f64) {
        let s: Value = serde_json::from_str(&og_camera_world_to_screen(
            e,
            json!({ "x": 1.0, "y": 0.5, "z": 0.0 }).to_string(),
        ))
        .unwrap();
        (s["x"].as_f64().unwrap(), s["y"].as_f64().unwrap())
    };
    let (sx, sy) = screen(e);
    // Built a tick after it was placed: the last info is where the end
    // before put it.
    let info = seen.infos.borrow().last().cloned().unwrap();
    assert_eq!(info["inView"], json!(true), "{info}");
    assert!(
        (info["distance"].as_f64().unwrap() - 26.25f64.sqrt()).abs() < 0.05,
        "{info}"
    );
    assert!(
        (info["x"].as_f64().unwrap() - sx).abs() < 0.5
            && (info["y"].as_f64().unwrap() - sy).abs() < 0.5,
        "{info} against worldToScreen ({sx}, {sy})"
    );
    // The camera moves right: the anchor moves left with the entity's image.
    camera(e, 2.0);
    step();
    step();
    let (mx, _) = screen(e);
    let moved = seen.infos.borrow().last().cloned().unwrap();
    assert!(
        mx < sx - 50.0,
        "the camera did not move the point: {sx} -> {mx}"
    );
    assert!(
        (moved["x"].as_f64().unwrap() - mx).abs() < 0.5,
        "{moved} did not follow the camera to {mx}"
    );
    // 25 numbers spawned on the first tick, alive for their 10 ticks.
    for _ in 0..10 {
        step();
    }
    let counts = seen.counts.borrow().clone();
    assert_eq!(counts[1], 25, "{counts:?}");
    assert_eq!(*counts.last().unwrap(), 0, "{counts:?}");
}
