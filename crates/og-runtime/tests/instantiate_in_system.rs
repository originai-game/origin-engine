//! world.instantiate inside a system (Star Focus ENGINE-ISSUES #7): a game
//! reads its input in systems, so a scene loaded in answer to a button is
//! instantiated from one. The root is spawned as world.spawn spawns one
//! there, and the scene is placed under it when the system returns.
//!
//! The game has two systems over the yard of `prefabs.rs` (two instances
//! of a prefab, so og.PrefabLink is registered while the tick runs): Load
//! instantiates the yard at tick 2, twice; Look, after it, reads the world
//! and saves it.
//!
//! Ways this could fail: the call is refused, as it was ("world.instantiate
//! inside a system needs og-ecs SystemContext packed-byte access"); the
//! scene is placed a tick late, or at the tick's end, so the system after
//! Load does not find it and a save that system takes holds a root with
//! nothing under it, which a restore never fills; the second instance in
//! the same system is not refused at the call (the first one's entities
//! are not in the world yet) and fails when it is placed; the names reach
//! the world only at the next probe call, so the mid-tick save has none;
//! a system that fails after the call leaves the scene's names held, so
//! the scene can never be instantiated again.

use super::prefabs::{YARD, hp, project};
use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_probe_dump, og_game_game,
    og_host_begin_bundle, og_load_scene, og_reflect_define_component, og_system_define_system,
    og_world_find, og_world_get, og_world_get_tick, og_world_instantiate, og_world_restore,
    og_world_snapshot, set_hooks,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::rc::Rc;

const LOAD: u32 = 1;
const LOOK: u32 = 2;
const AT: u32 = 2;

#[derive(Default)]
struct Script {
    /// The engine that instantiates, and its scene.
    loader: u32,
    scene: f64,
    /// What the two world.instantiate calls answered.
    answers: Vec<Value>,
    /// What Look found at the tick of the load: "crate1/lid", and its hp.
    looked: Option<(Value, Value)>,
    saved: Option<String>,
    /// Load fails after its first call.
    fail: bool,
}

struct Game(Rc<RefCell<Script>>);

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

impl Hooks for Game {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        let w = call.engine;
        let mut s = self.0.borrow_mut();
        if call.kind != 0 || w != s.loader || og_world_get_tick(w) != AT {
            return Ok(String::new());
        }
        match call.id {
            LOAD => {
                for _ in 0..2 {
                    let answer = og_runtime::host_impl::answer(og_world_instantiate(
                        w,
                        s.scene,
                        Some(json!({ "name": "yard" }).to_string()),
                    ));
                    s.answers.push(serde_json::from_str(&answer).unwrap());
                    if s.fail {
                        return Err("Load failed".into());
                    }
                }
            }
            LOOK => {
                let lid: Value =
                    serde_json::from_str(&og_world_find(w, "crate1/lid".into())).unwrap();
                let health = lid.as_f64().map_or(Value::Null, |lid| {
                    value(og_runtime::host_impl::answer(og_world_get(
                        w,
                        lid,
                        "Health".into(),
                    )))
                });
                s.looked = Some((lid, health));
                let save = value(og_runtime::host_impl::answer(og_world_snapshot(w, None)));
                s.saved = Some(save.as_str().unwrap().to_string());
            }
            _ => {}
        }
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
}

fn frames(e: u32, n: u32) {
    for _ in 0..n {
        value(og_engine_frame(e, 1000.0 / 60.0));
    }
}

fn hash(e: u32) -> Value {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(e, None)).unwrap();
    dump["hash"].clone()
}

#[test]
fn a_system_instantiates_a_scene_and_the_next_system_finds_it() {
    let root = project("in-system", YARD);
    let script = Rc::new(RefCell::new(Script::default()));
    set_hooks(Rc::new(Game(script.clone())));
    og_host_begin_bundle();
    value(og_reflect_define_component(
        "Health".into(),
        json!([{ "name": "hp", "kind": "f32", "default": 0, "options": {} }]).to_string(),
    ));
    let component = |name: &str| json!({ "kind": "component", "name": name });
    let systems = [
        og_system_define_system(
            "Load".into(),
            json!({ "writes": [component("og.Transform")] }).to_string(),
            LOAD,
        ),
        og_system_define_system(
            "Look".into(),
            json!({ "reads": [component("Health")], "writes": [] }).to_string(),
            LOOK,
        ),
    ];
    let game = og_game_game(Some(json!({ "seed": 1, "systems": systems }).to_string()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = || {
        value(og_engine_create(game, options.clone()))
            .as_u64()
            .unwrap() as u32
    };

    let a = engine();
    let scene = value(og_runtime::host_impl::answer(og_load_scene(
        a,
        "scenes/yard.ogt".into(),
    )));
    script.borrow_mut().loader = a;
    script.borrow_mut().scene = scene.as_f64().unwrap();
    frames(a, AT + 1);

    // The first call answered its root; the second, whose names the first
    // holds, was refused at the call.
    let s = script.borrow();
    assert_eq!(s.answers.len(), 2);
    assert_eq!(s.answers[0]["ok"], json!(true), "{:?}", s.answers);
    let yard = s.answers[0]["value"].as_f64().unwrap();
    assert_eq!(og_world_find(a, "yard".into()).parse(), Ok(yard));
    assert_eq!(s.answers[1]["ok"], json!(false), "{:?}", s.answers);
    let reason = s.answers[1]["reason"].as_str().unwrap();
    assert!(reason.contains("already exists"), "{reason}");
    // Look, the system after Load in the same tick, found the scene.
    let (lid, health) = s.looked.clone().expect("Look ran at the tick");
    assert!(lid.is_number(), "crate1/lid at the tick of the load: {lid}");
    assert_eq!(health["hp"], json!(40.0));
    // Under the root the call answered.
    let crate1: f64 = og_world_find(a, "crate1".into()).parse().unwrap();
    let parent = value(og_runtime::host_impl::answer(og_world_get(
        a,
        crate1,
        "og.Parent".into(),
    )));
    assert_eq!(parent["entity"].as_f64(), Some(yard));
    assert_eq!(hp(a, "crate2/lid"), 10.0);

    // The save Look took in that tick holds the scene: restored where
    // nothing was loaded, the world completes the tick and goes on as `a`.
    let saved = s.saved.clone().expect("Look saved");
    drop(s);
    let b = engine();
    value(og_runtime::host_impl::answer(og_world_restore(
        b, saved, None,
    )));
    assert_eq!(hp(b, "crate1/lid"), 40.0);
    frames(a, 10);
    while og_world_get_tick(b) < og_world_get_tick(a) {
        frames(b, 1);
    }
    assert_eq!(og_world_get_tick(b), og_world_get_tick(a));
    assert_eq!(hash(b), hash(a), "tick for tick as the world that saved");

    // A system that fails after the call: its commands are discarded with
    // the root, and the names it held are free again.
    let c = engine();
    let scene = value(og_runtime::host_impl::answer(og_load_scene(
        c,
        "scenes/yard.ogt".into(),
    )));
    {
        let mut s = script.borrow_mut();
        *s = Script {
            loader: c,
            scene: scene.as_f64().unwrap(),
            fail: true,
            ..Script::default()
        };
    }
    frames(c, AT);
    let failed: Value = serde_json::from_str(&og_engine_frame(c, 1000.0 / 60.0)).unwrap();
    assert_eq!(failed["ok"], json!(false), "{failed}");
    assert!(
        failed["reason"].as_str().unwrap().contains("Load failed"),
        "{failed}"
    );
    assert_eq!(script.borrow().answers[0]["ok"], json!(true));
    assert_eq!(og_world_find(c, "yard".into()), "null");
    script.borrow_mut().loader = u32::MAX;
    let again = og_runtime::host_impl::answer(og_world_instantiate(
        c,
        scene.as_f64().unwrap(),
        Some(json!({ "name": "yard" }).to_string()),
    ));
    value(again);
    assert_eq!(hp(c, "crate1/lid"), 40.0);
}
