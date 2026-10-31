//! Mouse-look: the relative movement of pointer 0 summed over a tick in
//! og.Pointers.movement (the failure modes written down first: a delta that
//! lasts past its tick; a further touch's movement counted as the mouse's; a
//! locked mouse's move at a still position that raises no PointerMove or
//! moves og.Input's pointer; a delta lost on the ring or in protocol 5 JSON;
//! the host's lock report not reaching og.PointerLock, or touching the
//! game's `wanted`).

use og_ecs::{SystemDesc, World, WorldDesc};
use og_input::{
    InjectedInput, Input, InputLayer, InputSink, PointerAction, PointerButton, PointerLock,
    PointerMove, Pointers, ring,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

fn mouse(id: u32, x: f32, y: f32, dx: f32, dy: f32) -> InjectedInput {
    InjectedInput::Pointer {
        action: PointerAction::Move,
        x,
        y,
        button: PointerButton::None,
        id,
        movement_x: dx,
        movement_y: dy,
    }
}

#[test]
fn movement_sums_over_the_tick_and_resets() {
    let mut w = World::new(&WorldDesc::default()).unwrap();
    let mut layer = InputLayer::new();
    layer.attach(&mut w).unwrap();
    let log = Arc::new(Mutex::new(Vec::<Value>::new()));
    let out = log.clone();
    w.add_system(
        SystemDesc::new("look")
            .read::<Input>()
            .read::<Pointers>()
            .read::<PointerLock>()
            .read::<PointerMove>(),
        move |ctx| {
            let p = ctx.resource::<Pointers>()?;
            let i = ctx.resource::<Input>()?;
            let l = ctx.resource::<PointerLock>()?;
            let moves: Vec<[f32; 2]> = ctx
                .events::<PointerMove>()?
                .iter()
                .map(|e| [e.movement_x, e.movement_y])
                .collect();
            out.lock().unwrap().push(json!({
                "movement": [p.movement.x, p.movement.y],
                "pointer": [i.pointer.x, i.pointer.y],
                "moves": moves, "locked": l.locked, "wanted": l.wanted,
            }));
            Ok(())
        },
    )
    .unwrap();
    w.resource_mut::<PointerLock>().unwrap().wanted = true;
    let steps: Vec<Vec<InjectedInput>> = vec![
        // 0: the mouse arrives at (100, 50) with no movement.
        vec![mouse(0, 100.0, 50.0, 0.0, 0.0)],
        // 1: locked; two moves at the same position, and a touch's movement that does not count.
        vec![
            InjectedInput::PointerLock { locked: true },
            mouse(0, 100.0, 50.0, 3.0, -1.0),
            mouse(0, 100.0, 50.0, 4.0, 2.0),
            mouse(1, 10.0, 10.0, 50.0, 50.0),
        ],
        // 2: nothing: the movement is gone.
        vec![],
        // 3: Escape released the lock.
        vec![InjectedInput::PointerLock { locked: false }],
    ];
    for events in &steps {
        for e in events {
            assert!(layer.inject(&mut w, e));
        }
        w.step().unwrap();
        layer.end_tick(&mut w);
    }
    let log = log.lock().unwrap();
    assert_eq!(log[0]["movement"], json!([0.0, 0.0]));
    assert_eq!(log[1]["movement"], json!([7.0, 1.0]), "{}", log[1]);
    assert_eq!(
        log[1]["pointer"],
        json!([100.0, 50.0]),
        "the position stands still"
    );
    // A move at a still position with movement still raises PointerMove (the touch's too).
    assert_eq!(
        log[1]["moves"],
        json!([[3.0, -1.0], [4.0, 2.0], [50.0, 50.0]])
    );
    assert_eq!(log[1]["locked"], json!(true));
    assert_eq!(log[2]["movement"], json!([0.0, 0.0]));
    assert_eq!(log[2]["moves"], json!([]));
    assert_eq!(log[3]["locked"], json!(false));
    assert!(
        log.iter().all(|t| t["wanted"] == json!(true)),
        "the host never writes wanted"
    );
}

#[test]
fn movement_crosses_json_and_the_ring() {
    let parsed = InjectedInput::from_json(&json!({
        "kind": "pointer", "action": "move", "x": 1, "y": 2, "button": "none",
        "movementX": 5.5, "movementY": -2
    }))
    .unwrap();
    assert_eq!(parsed, mouse(0, 1.0, 2.0, 5.5, -2.0));
    assert_eq!(
        InjectedInput::from_json(&parsed.to_json()),
        Some(parsed.clone())
    );
    // Omitted is 0, and protocol 5's JSON is unchanged without it.
    let plain = InjectedInput::from_json(
        &json!({"kind": "pointer", "action": "move", "x": 1, "y": 2, "button": "none"}),
    )
    .unwrap();
    assert_eq!(plain, mouse(0, 1.0, 2.0, 0.0, 0.0));
    assert!(plain.to_json().get("movementX").is_none());
    let lock = InjectedInput::from_json(&json!({"kind": "pointerLock", "locked": true})).unwrap();
    assert_eq!(lock, InjectedInput::PointerLock { locked: true });

    let mut r = ring::new_ring();
    for e in [
        &parsed,
        &lock,
        &InjectedInput::PointerLock { locked: false },
    ] {
        assert!(ring::write(&mut r, e));
    }
    let drained = ring::drain(&r);
    assert!(drained.skipped.is_empty(), "{:?}", drained.skipped);
    assert_eq!(
        drained.events,
        vec![parsed, lock, InjectedInput::PointerLock { locked: false }]
    );
}
