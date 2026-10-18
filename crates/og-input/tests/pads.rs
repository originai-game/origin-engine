//! Gamepads and several pointers at once, as a game system sees them over a
//! scripted sequence (the failure modes written down first: an edge that
//! lasts more or less than one tick; a press and release between two ticks
//! that leaves no edge; a disconnect that leaves buttons held; a second
//! touch that moves og.Input's pointer or raises events without its id; a
//! click on a touch that never releases; an id or pad out of range accepted;
//! pad state that a snapshot does not carry). Writes
//! `$CARGO_TARGET_TMPDIR/input-pads-transcript.json`.

use og_ecs::{SystemDesc, World, WorldDesc};
use og_input::{
    Gamepad, GamepadAxis, GamepadState, InjectedInput, Input, InputLayer, InputSink, PointerAction,
    PointerButton, PointerDown, PointerUp, Pointers,
};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

fn touch(action: PointerAction, id: u32, x: f32, y: f32) -> InjectedInput {
    InjectedInput::Pointer {
        action,
        x,
        y,
        button: if action == PointerAction::Move {
            PointerButton::None
        } else {
            PointerButton::Left
        },
        id,
    }
}

fn pad(pad: u32, buttons: &[&str], lx: f32, rt: f32) -> InjectedInput {
    InjectedInput::Gamepad(GamepadState {
        pad,
        connected: true,
        buttons: buttons
            .iter()
            .map(|b| 1 << og_input::gamepad_button(b).expect("a standard button"))
            .sum(),
        axes: [lx, 0.0, 0.0, 0.0],
        triggers: [0.0, rt],
    })
}

fn world() -> (World, InputLayer, Arc<Mutex<Vec<Value>>>) {
    let mut w = World::new(&WorldDesc::default()).unwrap();
    let mut layer = InputLayer::new();
    layer.attach(&mut w).unwrap();
    let log = Arc::new(Mutex::new(Vec::new()));
    let out = log.clone();
    w.add_system(
        SystemDesc::new("observe")
            .read::<Input>()
            .read::<Pointers>()
            .read::<Gamepad>()
            .read::<PointerDown>()
            .read::<PointerUp>(),
        move |ctx| {
            let i = ctx.resource::<Input>()?;
            let p = ctx.resource::<Pointers>()?;
            let g = ctx.resource::<Gamepad>()?;
            let (held, pressed, released) = g.masks(0);
            let downs: Vec<u32> = ctx.events::<PointerDown>()?.iter().map(|e| e.id).collect();
            let ups: Vec<u32> = ctx.events::<PointerUp>()?.iter().map(|e| e.id).collect();
            out.lock().unwrap().push(json!({
                "pointer": [i.pointer.x, i.pointer.y],
                "primaryHeld": i.button_down(PointerButton::Left),
                "held": p.held, "pressed": p.pressed, "released": p.released,
                "p1": [p.position(1).x, p.position(1).y], "start1": [p.start(1).x, p.start(1).y],
                "downs": downs, "ups": ups,
                "padConnected": g.is_connected(0),
                "padHeld": held, "padPressed": pressed, "padReleased": released,
                "lx": g.axis(GamepadAxis::LeftX, 0), "rt": g.axis(GamepadAxis::RightTrigger, 0),
            }));
            Ok(())
        },
    )
    .unwrap();
    (w, layer, log)
}

/// Events injected before each tick.
fn script() -> Vec<Vec<InjectedInput>> {
    use PointerAction::*;
    vec![
        // 0: the primary touch goes down on the stick; pad 0 connects holding A.
        vec![touch(Down, 0, 100.0, 600.0), pad(0, &["a"], 0.0, 0.0)],
        // 1: a second touch presses the button while the first drags.
        vec![
            touch(Move, 0, 130.0, 590.0),
            touch(Down, 1, 1100.0, 620.0),
            pad(0, &["a"], 0.8, 0.5),
        ],
        // 2: both held; the second slides.
        vec![touch(Move, 1, 1110.0, 625.0)],
        // 3: the second lifts; A released and B tapped within one interval.
        vec![
            touch(Up, 1, 1110.0, 625.0),
            pad(0, &["b"], 0.0, 0.0),
            pad(0, &[], 0.0, 0.0),
        ],
        // 4: a click on touch 3 (held through this tick, released at its end).
        vec![touch(Click, 3, 50.0, 50.0)],
        // 5: an id past the five pointers and pad 4 are refused; the pad disconnects.
        vec![
            touch(Down, 5, 1.0, 1.0),
            InjectedInput::Gamepad(GamepadState {
                pad: 4,
                ..Default::default()
            }),
            InjectedInput::Gamepad(GamepadState {
                pad: 0,
                connected: false,
                buttons: 1,
                ..Default::default()
            }),
        ],
        // 6: the primary lifts.
        vec![touch(Up, 0, 130.0, 590.0)],
    ]
}

fn run(w: &mut World, layer: &mut InputLayer, steps: &[Vec<InjectedInput>]) -> Vec<bool> {
    let mut accepted = Vec::new();
    for events in steps {
        for e in events {
            accepted.push(layer.inject(w, e));
        }
        w.step().unwrap();
        layer.end_tick(w);
    }
    accepted
}

#[test]
fn pads_and_touches_over_a_scripted_sequence() {
    let (mut w, mut layer, log) = world();
    let accepted = run(&mut w, &mut layer, &script());
    assert_eq!(
        accepted.iter().filter(|a| !**a).count(),
        2,
        "pointer id 5 and pad 4 are refused, nothing else: {accepted:?}"
    );
    let seen = log.lock().unwrap().clone();
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    std::fs::write(
        dir.join("input-pads-transcript.json"),
        serde_json::to_string_pretty(&seen).unwrap(),
    )
    .unwrap();
    let t = |i: usize, k: &str| seen[i][k].clone();
    let abit = 1u32;
    // Tick 0: the primary touch is og.Input's pointer and pointer 0.
    assert_eq!(t(0, "pointer"), json!([100.0, 600.0]));
    assert_eq!((t(0, "held"), t(0, "pressed")), (json!(1), json!(1)));
    assert_eq!(t(0, "padPressed"), json!(abit));
    assert_eq!(
        t(0, "downs"),
        json!([0]),
        "pointer 0 raises its PointerDown once, through og.Input"
    );
    // Tick 1: two pointers held; the second's press edge, start and event with its id;
    // og.Input followed the primary only.
    assert_eq!(t(1, "pointer"), json!([130.0, 590.0]));
    assert_eq!((t(1, "held"), t(1, "pressed")), (json!(0b11), json!(0b10)));
    assert_eq!(t(1, "start1"), json!([1100.0, 620.0]));
    assert_eq!(t(1, "downs"), json!([1]));
    assert_eq!(
        (t(1, "padPressed"), t(1, "padHeld")),
        (json!(0), json!(abit)),
        "A's edge lasted one tick"
    );
    assert_eq!((t(1, "lx"), t(1, "rt")), (json!(0.8f32), json!(0.5)));
    // Tick 2: still both; the second moved, its start did not.
    assert_eq!((t(2, "held"), t(2, "pressed")), (json!(0b11), json!(0)));
    assert_eq!(
        (t(2, "p1"), t(2, "start1")),
        (json!([1110.0, 625.0]), json!([1100.0, 620.0]))
    );
    // Tick 3: the second lifted (edge, event); A released, B pressed and released.
    assert_eq!((t(3, "held"), t(3, "released")), (json!(1), json!(0b10)));
    assert_eq!(t(3, "ups"), json!([1]));
    assert_eq!(t(3, "padHeld"), json!(0));
    assert_eq!(
        t(3, "padPressed"),
        json!(0b10),
        "B tapped between two ticks still has its press edge"
    );
    assert_eq!(t(3, "padReleased"), json!(0b11));
    // Tick 4: the click holds touch 3 for this tick; tick 5 sees it released with its PointerUp.
    assert_eq!(
        (t(4, "held"), t(4, "pressed")),
        (json!(0b1001), json!(0b1000))
    );
    assert_eq!((t(5, "held"), t(5, "released")), (json!(1), json!(0b1000)));
    assert_eq!(t(5, "ups"), json!([3]));
    assert_eq!(t(5, "padConnected"), json!(false));
    assert_eq!(
        t(5, "padHeld"),
        json!(0),
        "a disconnected pad holds nothing"
    );
    assert_eq!(t(5, "lx"), json!(0.0));
    // Tick 6: the primary lifted.
    assert_eq!(
        (t(6, "held"), t(6, "released"), t(6, "primaryHeld")),
        (json!(0), json!(1), json!(false))
    );
}

#[test]
fn pad_and_touch_state_is_world_state_and_replays() {
    let steps = script();
    let (mut a, mut la, log_a) = world();
    run(&mut a, &mut la, &steps[..2]);
    // Two touches and a pad with A held and a stick deflected.
    let snap = a.snapshot();
    run(&mut a, &mut la, &steps[2..]);
    let (mut b, mut lb, log_b) = world();
    b.restore(&snap).unwrap();
    run(&mut b, &mut lb, &steps[2..]);
    assert_eq!(b.world_hash(), a.world_hash());
    assert_eq!(log_b.lock().unwrap()[..], log_a.lock().unwrap()[2..]);
}
