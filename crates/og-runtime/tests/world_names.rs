//! A name as a system sees it, with no probe call between frames (a plain
//! `og run`, a published game).
//!
//! A name is free again as soon as the despawn of its entity applies: a
//! system's when it returns, so a later system in the same tick, a later
//! tick and a call between ticks may give it again. It failed while the
//! runtime's name table dropped a name only when a probe call brought it in
//! line with the world (og test and og run --json call the probe every
//! frame, so only a game without one saw the name stay taken).

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
    og_system_define_system, og_world_despawn, og_world_find, og_world_get_tick, og_world_spawn,
    set_hooks,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::rc::Rc;

const KILL: u32 = 1;
const MAKE: u32 = 2;

#[derive(Default)]
struct Script {
    /// (tick, system, what world.spawn of "a" answered).
    spawns: Vec<(u32, &'static str, bool)>,
}

struct Game(Rc<RefCell<Script>>);

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

fn find(w: u32, name: &str) -> Option<f64> {
    og_world_find(w, name.into()).map(|b| b as f64)
}

fn spawn_a(w: u32) -> bool {
    og_world_spawn(w, Some(json!({ "name": "a" }).to_string())).is_ok()
}

impl Hooks for Game {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        let w = call.engine;
        let tick = og_world_get_tick(w);
        let mut s = self.0.borrow_mut();
        match (call.kind, call.id) {
            // Kill despawns "a" at ticks 2 and 4, then (2) asks for the
            // name again: its despawn has not applied yet.
            (0, KILL) if tick == 2 || tick == 4 => {
                if let Some(a) = find(w, "a") {
                    og_world_despawn(w, a).unwrap();
                }
                if tick == 2 {
                    s.spawns.push((tick, "Kill", spawn_a(w)));
                }
            }
            // Make, after Kill: "a" again in the same tick (2) and a later
            // one (6).
            (0, MAKE) if tick == 2 || tick == 6 => {
                s.spawns.push((tick, "Make", spawn_a(w)));
            }
            _ => {}
        }
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets"))
    }
}

fn frames(e: u32, n: u32) {
    for _ in 0..n {
        value(og_engine_frame(e, 1000.0 / 60.0));
    }
}

#[test]
fn a_name_is_free_once_its_despawn_applies() {
    let script = Rc::new(RefCell::new(Script::default()));
    set_hooks(Rc::new(Game(script.clone())));
    og_host_begin_bundle();
    let component = |name: &str| json!({ "kind": "component", "name": name });
    let writes = json!({ "writes": [component("og.Transform")] }).to_string();
    let systems = [
        og_system_define_system("Kill".into(), writes.clone(), KILL),
        og_system_define_system("Make".into(), writes, MAKE),
    ];
    let game = og_game_game(Some(json!({ "seed": 1, "systems": systems }).to_string()));
    let w = value(og_engine_create(game, json!({}).to_string()))
        .as_u64()
        .unwrap() as u32;

    // A between-ticks spawn names "a"; the frames below are the only calls.
    assert!(spawn_a(w));
    frames(w, 8);
    assert_eq!(
        script.borrow().spawns,
        [
            // Kill's own despawn applies when Kill returns.
            (2, "Kill", false),
            (2, "Make", true),
            (6, "Make", true),
        ]
    );
    // Between ticks the name is free after its despawn as well.
    og_world_despawn(w, find(w, "a").expect("Make's a of tick 6")).unwrap();
    assert!(find(w, "a").is_none());
    assert!(spawn_a(w), "a between ticks after its despawn");
}
