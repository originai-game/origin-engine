//! The input resource over a scripted sequence, as a game system sees it:
//! events injected between ticks N-1 and N are seen during tick N, edges
//! last exactly that tick, presses and clicks release when it ends, and the
//! input events arrive in injection order. Writes the transcript to
//! `$CARGO_TARGET_TMPDIR/input-edges-transcript.json`.

use og_ecs::{SystemDesc, World, WorldDesc};
use og_input::{
    InjectedInput, Input, InputLayer, InputSink, KeyAction, KeyDown, KeyUp, Modifiers,
    PointerAction, PointerButton, PointerDown, PointerMove, PointerUp, Wheel,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

fn key(code: &str, action: KeyAction) -> InjectedInput {
    InjectedInput::Key {
        key: code.into(),
        action,
        modifiers: Modifiers::default(),
    }
}

fn pointer(action: PointerAction, x: f32, y: f32, button: PointerButton) -> InjectedInput {
    InjectedInput::Pointer {
        action,
        x,
        y,
        button,
        id: 0,
    }
}

fn wheel(x: f32, y: f32, dy: f32) -> InjectedInput {
    InjectedInput::Wheel {
        x,
        y,
        delta_x: 0.0,
        delta_y: dy,
    }
}

/// A world whose one system records, every tick, what a game sees.
fn world() -> (World, InputLayer, Arc<Mutex<Vec<Value>>>) {
    let mut w = World::new(&WorldDesc::default()).unwrap();
    let mut layer = InputLayer::new();
    layer.attach(&mut w).unwrap();
    let log = Arc::new(Mutex::new(Vec::new()));
    let out = log.clone();
    w.add_system(
        SystemDesc::new("observe")
            .read::<Input>()
            .read::<KeyDown>()
            .read::<KeyUp>()
            .read::<PointerDown>()
            .read::<PointerUp>()
            .read::<PointerMove>()
            .read::<Wheel>(),
        move |ctx| {
            let i = ctx.resource::<Input>()?;
            let keys = ["KeyW", "Space", "KeyA", "KeyD"];
            let set = |f: &dyn Fn(&str) -> bool| -> Vec<&str> {
                keys.iter().copied().filter(|k| f(k)).collect()
            };
            let buttons = |f: &dyn Fn(PointerButton) -> bool| -> Vec<&str> {
                [
                    PointerButton::Left,
                    PointerButton::Middle,
                    PointerButton::Right,
                ]
                .into_iter()
                .filter(|b| f(*b))
                .map(PointerButton::name)
                .collect()
            };
            // Events in send order across the six types: each event type's
            // queue keeps its order; the observer merges them by type in the
            // order a game would poll them.
            let mut events: Vec<String> = Vec::new();
            for e in ctx.events::<KeyDown>()? {
                events.push(format!(
                    "KeyDown {}{}",
                    e.key,
                    if e.repeat { " repeat" } else { "" }
                ));
            }
            for e in ctx.events::<KeyUp>()? {
                events.push(format!("KeyUp {}", e.key));
            }
            for e in ctx.events::<PointerMove>()? {
                events.push(format!("PointerMove {},{}", e.x, e.y));
            }
            for e in ctx.events::<PointerDown>()? {
                events.push(format!(
                    "PointerDown {} {},{}",
                    e.button().unwrap().name(),
                    e.x,
                    e.y
                ));
            }
            for e in ctx.events::<PointerUp>()? {
                events.push(format!(
                    "PointerUp {} {},{}",
                    e.button().unwrap().name(),
                    e.x,
                    e.y
                ));
            }
            for e in ctx.events::<Wheel>()? {
                events.push(format!("Wheel {},{}", e.delta_x, e.delta_y));
            }
            out.lock().unwrap().push(json!({
                "tick": ctx.time().tick,
                "down": set(&|k| i.down(k)),
                "pressed": set(&|k| i.pressed(k)),
                "released": set(&|k| i.released(k)),
                "axisAD": i.axis("KeyA", "KeyD"),
                "buttonsDown": buttons(&|b| i.button_down(b)),
                "buttonsPressed": buttons(&|b| i.button_pressed(b)),
                "buttonsReleased": buttons(&|b| i.button_released(b)),
                "pointer": [i.pointer.x, i.pointer.y],
                "pointerInside": i.pointer_inside,
                "wheel": [i.wheel.x, i.wheel.y],
                "events": events,
            }));
            Ok(())
        },
    )
    .unwrap();
    (w, layer, log)
}

/// The script: the events injected before each tick.
fn script() -> Vec<Vec<InjectedInput>> {
    use KeyAction::{Down, Press, Up};
    use PointerAction::{Click, Move};
    vec![
        // 0: a key down and a hover move.
        vec![
            key("KeyW", Down),
            pointer(Move, 10.0, 20.0, PointerButton::None),
        ],
        // 1: nothing: held, no edges, no events.
        vec![],
        // 2: release W, press Space, click left elsewhere, two wheel steps.
        vec![
            key("KeyW", Up),
            key("Space", Press),
            pointer(Click, 30.0, 40.0, PointerButton::Left),
            wheel(30.0, 40.0, 5.0),
            wheel(30.0, 40.0, 7.0),
        ],
        // 3: nothing: the press and the click release now.
        vec![],
        // 4: a tap shorter than a tick, and an unknown code (refused).
        vec![key("KeyA", Down), key("KeyA", Up), key("Hyper", Down)],
        // 5: D held twice (the second is a repeat), A held: axis 0.
        vec![key("KeyD", Down), key("KeyD", Down), key("KeyA", Down)],
        // 6: A up: axis +1.
        vec![key("KeyA", Up)],
    ]
}

fn expected() -> Vec<Value> {
    vec![
        json!({"tick": 0, "down": ["KeyW"], "pressed": ["KeyW"], "released": [], "axisAD": 0.0,
               "buttonsDown": [], "buttonsPressed": [], "buttonsReleased": [],
               "pointer": [10.0, 20.0], "pointerInside": true, "wheel": [0.0, 0.0],
               "events": ["KeyDown KeyW", "PointerMove 10,20"]}),
        json!({"tick": 1, "down": ["KeyW"], "pressed": [], "released": [], "axisAD": 0.0,
               "buttonsDown": [], "buttonsPressed": [], "buttonsReleased": [],
               "pointer": [10.0, 20.0], "pointerInside": true, "wheel": [0.0, 0.0], "events": []}),
        json!({"tick": 2, "down": ["Space"], "pressed": ["Space"], "released": ["KeyW"], "axisAD": 0.0,
               "buttonsDown": ["left"], "buttonsPressed": ["left"], "buttonsReleased": [],
               "pointer": [30.0, 40.0], "pointerInside": true, "wheel": [0.0, 12.0],
               "events": ["KeyDown Space", "KeyUp KeyW", "PointerMove 30,40", "PointerDown left 30,40",
                          "Wheel 0,5", "Wheel 0,7"]}),
        json!({"tick": 3, "down": [], "pressed": [], "released": ["Space"], "axisAD": 0.0,
               "buttonsDown": [], "buttonsPressed": [], "buttonsReleased": ["left"],
               "pointer": [30.0, 40.0], "pointerInside": true, "wheel": [0.0, 0.0],
               "events": ["KeyUp Space", "PointerUp left 30,40"]}),
        json!({"tick": 4, "down": [], "pressed": ["KeyA"], "released": ["KeyA"], "axisAD": 0.0,
               "buttonsDown": [], "buttonsPressed": [], "buttonsReleased": [],
               "pointer": [30.0, 40.0], "pointerInside": true, "wheel": [0.0, 0.0],
               "events": ["KeyDown KeyA", "KeyUp KeyA"]}),
        json!({"tick": 5, "down": ["KeyA", "KeyD"], "pressed": ["KeyA", "KeyD"], "released": [], "axisAD": 0.0,
               "buttonsDown": [], "buttonsPressed": [], "buttonsReleased": [],
               "pointer": [30.0, 40.0], "pointerInside": true, "wheel": [0.0, 0.0],
               "events": ["KeyDown KeyD", "KeyDown KeyD repeat", "KeyDown KeyA"]}),
        json!({"tick": 6, "down": ["KeyD"], "pressed": [], "released": ["KeyA"], "axisAD": 1.0,
               "buttonsDown": [], "buttonsPressed": [], "buttonsReleased": [],
               "pointer": [30.0, 40.0], "pointerInside": true, "wheel": [0.0, 0.0],
               "events": ["KeyUp KeyA"]}),
    ]
}

/// Injects the script's events before each tick, as the probe's `frame`
/// does: inject, step, end the tick. Returns the refusals.
fn run(w: &mut World, layer: &mut InputLayer, steps: &[Vec<InjectedInput>]) -> Vec<String> {
    let mut refused = Vec::new();
    for events in steps {
        for e in events {
            if !layer.inject(w, e) {
                refused.push(e.to_json()["key"].as_str().unwrap_or("?").to_owned());
            }
        }
        w.step().unwrap();
        layer.end_tick(w);
    }
    refused
}

#[test]
fn edges_last_one_tick_over_a_scripted_sequence() {
    let (mut w, mut layer, log) = world();
    let refused = run(&mut w, &mut layer, &script());
    assert_eq!(refused, ["Hyper"], "an unknown code is refused");
    assert_eq!(layer.accepted(), 13, "fourteen events, one refused");
    let log = log.lock().unwrap().clone();
    let transcript = json!({"script": script().iter().map(|s| s.iter().map(InjectedInput::to_json).collect::<Vec<_>>()).collect::<Vec<_>>(), "ticks": log});
    let path =
        std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("input-edges-transcript.json");
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&transcript).unwrap() + "\n",
    )
    .unwrap();
    for (got, want) in log.iter().zip(expected()) {
        assert_eq!(got, &want, "tick {}", want["tick"]);
    }
    assert_eq!(log.len(), expected().len());
}

#[test]
fn injected_input_is_world_state_and_replays_exactly() {
    let steps = script();
    let (mut a, mut la, log_a) = world();
    run(&mut a, &mut la, &steps[..3]);
    // Between tick 2 and 3: the click and the press wait to be released and
    // their events wait to be delivered; all of it is in the snapshot.
    let snap = a.snapshot();
    run(&mut a, &mut la, &steps[3..]);

    let (mut b, mut lb, log_b) = world();
    b.restore(&snap).unwrap();
    run(&mut b, &mut lb, &steps[3..]);
    assert_eq!(b.world_hash(), a.world_hash());
    let seen = log_b.lock().unwrap();
    assert_eq!(
        seen[..],
        log_a.lock().unwrap()[3..],
        "every tick after the restore is seen as in the original run"
    );
    assert_eq!(
        seen[0],
        expected()[3],
        "the restored world sees the releases"
    );

    // And the input changes the hash: the same ticks without input differ.
    let (mut c, mut lc, _) = world();
    run(&mut c, &mut lc, &vec![vec![]; steps.len()]);
    assert_ne!(c.world_hash(), a.world_hash());
}

/// A key event's modifiers are held while it happens: one no key holds is
/// pressed with it for that tick (Command as `MetaLeft`, apart from
/// Control), one already held stays as it is, and an `up` releases none.
#[test]
fn a_key_event_holds_the_modifiers_it_names() {
    let mut i = Input::default();
    let cmd = Modifiers {
        platform: true,
        ..Modifiers::default()
    };
    let events = i
        .apply(&InjectedInput::Key {
            key: "ArrowLeft".into(),
            action: KeyAction::Press,
            modifiers: cmd,
        })
        .unwrap();
    assert!(i.pressed("MetaLeft") && i.down("MetaLeft"));
    assert!(!i.down("ControlLeft"), "Command is not Control");
    assert!(i.pressed("ArrowLeft"));
    let codes: Vec<String> = events
        .iter()
        .filter_map(|e| match e {
            og_input::InputEvent::KeyDown(k) => Some(k.key.as_str().to_owned()),
            _ => None,
        })
        .collect();
    assert_eq!(codes, ["MetaLeft", "ArrowLeft"], "the modifier first");
    i.end_tick();
    assert!(!i.down("MetaLeft") && !i.down("ArrowLeft"), "both released");
    // A held Control stays held through an event naming it, and after an up.
    i.apply(&key("ControlRight", KeyAction::Down)).unwrap();
    i.end_tick();
    let ctrl = Modifiers {
        control: true,
        ..Modifiers::default()
    };
    i.apply(&InjectedInput::Key {
        key: "KeyA".into(),
        action: KeyAction::Press,
        modifiers: ctrl,
    })
    .unwrap();
    assert!(
        !i.down("ControlLeft"),
        "Control was held already (the right key)"
    );
    i.apply(&InjectedInput::Key {
        key: "KeyA".into(),
        action: KeyAction::Up,
        modifiers: ctrl,
    })
    .unwrap();
    i.end_tick();
    assert!(i.down("ControlRight"), "an up leaves the modifiers held");
}
