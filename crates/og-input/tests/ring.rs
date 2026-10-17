//! The input ring as the web shell writes it: `fixtures/ring.bin` is a
//! SharedArrayBuffer written by `packages/engine/src/web/input.ts` itself
//! (`node fixtures/make-ring.mjs`), with its live events straddling the slot
//! wrap, and `fixtures/ring.json` is what the TypeScript `drainRing` reads
//! from it. The Rust decoder must read the same events, and the Rust writer
//! must produce bytes the same decoder reads back.

use og_ecs::{World, WorldDesc};
use og_input::ring::{self, BYTE_LEN, SLOTS};
use og_input::{InjectedInput, Input, InputLayer, InputSink, PointerAction, PointerButton};
use serde_json::Value;

fn fixture() -> (Vec<u8>, Value) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let bin = std::fs::read(dir.join("ring.bin")).unwrap();
    let json =
        serde_json::from_str(&std::fs::read_to_string(dir.join("ring.json")).unwrap()).unwrap();
    (bin, json)
}

#[test]
fn the_decoder_reads_what_the_web_shell_wrote() {
    let (bin, json) = fixture();
    assert_eq!(bin.len(), BYTE_LEN);
    assert_eq!(json["bytes"].as_u64().unwrap() as usize, BYTE_LEN);
    assert_eq!(json["slots"].as_u64().unwrap() as usize, SLOTS);
    let expected: Vec<InjectedInput> = json["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| InjectedInput::from_json(e).unwrap())
        .collect();
    assert_eq!(expected.len(), 8);
    let read = json["read"].as_u64().unwrap() as usize;
    assert!(
        read % SLOTS + expected.len() > SLOTS,
        "the fixture wraps the slot index"
    );

    let drained = ring::drain(&bin);
    assert!(drained.skipped.is_empty(), "{:?}", drained.skipped);
    assert_eq!(drained.events, expected);
    assert_eq!(drained.read as u64, json["write"].as_u64().unwrap());

    let mut owned = bin.clone();
    assert_eq!(ring::drain_mut(&mut owned).events.len(), 8);
    assert!(
        ring::drain(&owned).events.is_empty(),
        "the read index was stored"
    );
}

#[test]
fn the_writer_and_the_decoder_agree_and_a_full_ring_refuses() {
    let mut r = ring::new_ring();
    let events = [
        InjectedInput::Pointer {
            action: PointerAction::Click,
            x: 3.0,
            y: 4.0,
            button: PointerButton::Left,
        },
        InjectedInput::from_json(
            &serde_json::json!({"kind": "key", "key": "Space", "action": "press"}),
        )
        .unwrap(),
        InjectedInput::Wheel {
            x: 1.0,
            y: 1.0,
            delta_x: 0.0,
            delta_y: -40.0,
        },
    ];
    for e in &events {
        assert!(ring::write(&mut r, e));
    }
    assert_eq!(ring::drain_mut(&mut r).events, events);
    for i in 0..SLOTS {
        let e = InjectedInput::Pointer {
            action: PointerAction::Move,
            x: i as f32,
            y: 0.0,
            button: PointerButton::None,
        };
        assert!(ring::write(&mut r, &e), "slot {i}");
    }
    assert!(
        !ring::write(&mut r, &events[0]),
        "a full ring drops the event"
    );
    let drained = ring::drain_mut(&mut r);
    assert_eq!(drained.events.len(), SLOTS);
    assert!(
        ring::write(&mut r, &events[0]),
        "drained, it takes events again"
    );

    // A slot that names no event is skipped with a reason, the rest decode.
    let mut r = ring::new_ring();
    ring::write(&mut r, &events[0]);
    ring::write(&mut r, &events[2]);
    let at = ring::HEADER_WORDS * 4; // slot 0
    r[at..at + 4].copy_from_slice(&9u32.to_le_bytes());
    let drained = ring::drain(&r);
    assert_eq!(drained.events, [events[2].clone()]);
    assert_eq!(drained.skipped, ["event 0: kind 9"]);
}

#[test]
fn the_input_layer_drains_a_ring_into_the_world() {
    let (bin, _) = fixture();
    let mut w = World::new(&WorldDesc::default()).unwrap();
    let mut layer = InputLayer::new();
    layer.attach(&mut w).unwrap();
    let report = layer.drain_ring(&mut w, &bin);
    assert_eq!((report.accepted, report.refused), (8, 0));
    assert_eq!(layer.accepted(), 8);
    let input = *w.resource::<Input>().unwrap();
    assert!(input.down("KeyW") && input.down("ArrowLeft") && !input.down("NumpadSubtract"));
    assert!(input.button_down(PointerButton::Left) && input.button_down(PointerButton::Right));
    assert_eq!(input.wheel.y, 120.0);
    assert_eq!(input.pointer, og_core::glam::Vec2::new(0.1, 0.2));
}
