//! og.ui's zero values through the engine (the `conventions` guide): a
//! field left out takes its default, and a 0 written out means what the
//! same field means in the rest of the API. `UiActionDesc.holdMs` is 500
//! left out and never holds at 0, as `ActionDesc.holdMs`; `PlayDesc.speed`
//! is 1 left out and holds the play's time at 0, as og.Timeline's speed.
//!
//! Ways it could fail: the defaults are not applied (a hold with no holdMs
//! never fires, a play with no speed stands still), a 0 is still read as
//! "not set" (the hold fires after 500 ms, the play runs at 1), or a 0 is
//! taken as no wait at all (the hold fires on the press).

use og_runtime::{
    Dispatch, Fetch, Hooks, og_commands_schedule, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_system_define_system, og_ui_action_fired, og_ui_action_held,
    og_ui_define_action, og_ui_define_timeline, og_ui_play_info, og_ui_play_timeline, set_hooks,
    shared_ui,
};
use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const BUILD: u32 = 1;

/// The hold actions on one key, and the speeds the plays are started at.
const ACTIONS: [&str; 3] = ["charge", "quick", "never"];
const SPEEDS: [Option<f32>; 3] = [None, Some(2.0), Some(0.0)];

#[derive(Default)]
struct Seen {
    engine: Cell<u32>,
    /// The element the plays run on; empty until it is laid out.
    path: RefCell<String>,
    plays: RefCell<Vec<u32>>,
    /// Per tick: each action's (fired, held).
    actions: RefCell<Vec<[(bool, bool); 3]>>,
    /// Per tick: each play's time in ms, once they are started.
    times: RefCell<Vec<[f32; 3]>>,
}

struct Host(Rc<Seen>);

impl Hooks for Host {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind != 0 || call.id != BUILD {
            return Ok(String::new());
        }
        let s = &self.0;
        if s.actions.borrow().is_empty() {
            for (name, hold) in [("charge", None), ("quick", Some(100)), ("never", Some(0))] {
                let mut desc = json!({ "name": name, "keys": ["KeyC"], "trigger": "hold" });
                if let Some(ms) = hold {
                    desc["holdMs"] = json!(ms);
                }
                og_ui_define_action(s.engine.get(), Some(desc.to_string())).expect("defines");
            }
            let fade = json!({ "tracks": [{ "prop": "opacity", "keys": [
                { "atMs": 0, "value": 0 }, { "atMs": 1000, "value": 1 },
            ] }] });
            og_ui_define_timeline("fade".into(), Some(fade.to_string()));
        }
        s.actions
            .borrow_mut()
            .push(ACTIONS.map(|a| (og_ui_action_fired(a.into()), og_ui_action_held(a.into()))));
        let path = s.path.borrow().clone();
        if !path.is_empty() && s.plays.borrow().is_empty() {
            *s.plays.borrow_mut() = SPEEDS
                .iter()
                .map(|speed| {
                    let desc = speed.map(|v| json!({ "speed": v }).to_string());
                    og_ui_play_timeline(path.clone(), "fade".into(), desc)
                })
                .collect();
        } else if let &[a, b, c] = s.plays.borrow().as_slice() {
            let at = |id| og_ui_play_info(id).map_or(f32::NAN, |p| p.at_ms);
            s.times.borrow_mut().push([at(a), at(b), at(c)]);
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

#[test]
fn a_fields_default_is_its_left_out_value_and_zero_is_zero() {
    let seen = Rc::new(Seen::default());
    set_hooks(Rc::new(Host(seen.clone())));
    og_host_begin_bundle();
    let system = og_system_define_system(
        "Build".into(),
        json!({ "reads": [], "writes": [] }).to_string(),
        BUILD,
    );
    let game = og_game_game(Some(json!({ "seed": 1, "systems": [system] }).to_string()));
    let e = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    seen.engine.set(e);
    {
        let ui = shared_ui(e).expect("the engine's UI");
        let mut ui = ui.lock();
        let text =
            r#"<meta name="component" content="box"><div id="Root"><p id="Label">Ready</p></div>"#;
        ui.define_document(og_ui::UiDocument::parse("ui/box.svelte", text).expect("parses"));
        ui.mount_document(
            og_ui::reactive::SourceId::new("test", "box"),
            "box",
            Default::default(),
            Default::default(),
            &og_ui::bind::NoEnv,
        )
        .expect("mounts");
    }
    let step = || value(og_engine_frame(e, 1000.0 / 60.0));
    step();
    step();
    {
        let ui = shared_ui(e).expect("the engine's UI");
        let ui = ui.lock();
        *seen.path.borrow_mut() = ui
            .probe_elements()
            .into_iter()
            .find(|el| el.path.ends_with("Label"))
            .expect("the label is laid out")
            .path;
    }

    // The plays, 30 ticks at 60 Hz: half a second.
    for _ in 0..31 {
        step();
    }
    let times = seen.times.borrow().clone();
    let [left_out, double, zero] = *times.last().expect("the plays started");
    assert!(
        (left_out - 500.0).abs() < 20.0,
        "a speed left out plays at 1: {left_out} ms after half a second"
    );
    assert!(
        (double - 2.0 * left_out).abs() < 1.0,
        "a speed of 2 plays twice as fast: {double} ms against {left_out}"
    );
    assert!(
        times.iter().all(|t| t[2] == 0.0),
        "a speed of 0 holds the play's time: {zero} ms"
    );

    // The key goes down and stays: 100 ms is 6 ticks, 500 ms is 30.
    let from = seen.actions.borrow().len();
    value(og_runtime::host_impl::answer(og_commands_schedule(
        e,
        "0:key_down:KeyC".into(),
    )));
    for _ in 0..60 {
        step();
    }
    let actions = seen.actions.borrow()[from..].to_vec();
    let fired =
        |k: usize| -> Vec<usize> { (0..actions.len()).filter(|&t| actions[t][k].0).collect() };
    let (charge, quick, never) = (fired(0), fired(1), fired(2));
    assert_eq!(quick.len(), 1, "a 100 ms hold fires once: {quick:?}");
    assert_eq!(
        charge.len(),
        1,
        "a hold with no holdMs fires once: {charge:?}"
    );
    assert_eq!(
        charge[0] - quick[0],
        24,
        "a holdMs left out is 500 ms, 24 ticks after a 100 ms hold: {charge:?} {quick:?}"
    );
    assert!(never.is_empty(), "a holdMs of 0 never fires: {never:?}");
    let last = actions.last().unwrap();
    assert!(
        last[0].1 && last[1].1,
        "the reached holds are held: {last:?}"
    );
    assert!(!last[2].1, "a holdMs of 0 is never held: {last:?}");
}
