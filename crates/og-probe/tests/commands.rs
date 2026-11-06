//! The command channel (design §3.3): grammar, scheduling on simulation
//! time, the core actions, the registry, `help`, and unknown actions that
//! warn and let the script continue. The scripted run's dump is written to
//! `$CARGO_TARGET_TMPDIR/scripted-run-dump-world.json`.
#![cfg(not(feature = "publish"))]

use crate::support;

use og_core::glam::Vec3;
use og_probe::{Input, Payload};
use serde_json::{Value, json};
use support::{Camera, Harness, Health, Transform, build};

/// Run the host loop: one tick per frame until `quit` (or a bound).
fn run(h: &mut Harness, max_ticks: u64) {
    while !h.probe.quit_requested() && h.probe.tick(&h.world) < max_ticks {
        h.probe.step(&mut h.world).unwrap();
    }
}

fn json_outputs(h: &mut Harness) -> Vec<(u64, String, Option<String>, Value)> {
    h.probe
        .take_outputs()
        .into_iter()
        .filter_map(|o| match o.payload {
            Payload::Json(v) => Some((o.tick, o.action, o.path, v)),
            _ => None,
        })
        .collect()
}

#[test]
fn pointer_and_key_actions_go_through_the_input_layer() {
    let mut h = build(1, false);
    let input = |h: &Harness| *h.world.resource::<Input>().unwrap();
    h.probe
        .schedule(&h.world, "0:touch_move_to:Menu/Start;0:touch_start")
        .unwrap();
    h.probe.pump(&mut h.world);
    let i = input(&h);
    assert_eq!(
        (i.pointer.x, i.pointer.y),
        (60.0, 30.0),
        "the centre of Menu/Start, not Dialog/Start"
    );
    assert_eq!((i.buttons, i.buttons_pressed), (1, 1));
    h.probe.step(&mut h.world).unwrap();
    assert_eq!(input(&h).buttons_pressed, 0, "edges last one tick");

    h.probe
        .schedule(&h.world, "0:touch_end;0:touch_move_to:Start")
        .unwrap();
    h.probe.pump(&mut h.world);
    let i = input(&h);
    assert_eq!((i.buttons, i.buttons_released), (0, 1));
    assert_eq!(
        (i.pointer.x, i.pointer.y),
        (260.0, 30.0),
        "a bare id is the topmost element with it"
    );

    h.probe
        .schedule(&h.world, "0:move_to_text:Opt;0:scroll:0,-120")
        .unwrap();
    h.probe.pump(&mut h.world);
    let i = input(&h);
    assert_eq!((i.pointer.x, i.pointer.y, i.wheel.y), (60.0, 80.0, -120.0));
    h.probe
        .schedule(
            &h.world,
            "0:move_to_text:=Start;0:move_to_text:Start@-1;0:touch_move:7,8",
        )
        .unwrap();
    h.probe.pump(&mut h.world);
    assert_eq!((input(&h).pointer.x, input(&h).pointer.y), (7.0, 8.0));

    // A missed target warns and cancels the bare touch_start/touch_end after it.
    h.probe.take_log();
    h.probe
        .schedule(
            &h.world,
            "0:touch_move_to:Nowhere;0:touch_start;0:touch_end;0:touch_start",
        )
        .unwrap();
    h.probe.pump(&mut h.world);
    let log = h.probe.take_log();
    assert!(log[0].text.contains("target not found: 'Nowhere'"));
    assert!(log[1].text.contains("skipped") && log[2].text.contains("skipped"));
    assert_eq!(
        input(&h).buttons,
        1,
        "the touch_start after the cancelled pair pressed"
    );

    // Keys: held keys steer the player; type spreads presses over ticks.
    h.probe.step(&mut h.world).unwrap();
    let x0 = h.player_x();
    h.probe
        .schedule(&h.world, "0:key_down:ArrowRight;0.5:key_up:ArrowRight")
        .unwrap();
    for _ in 0..60 {
        h.probe.step(&mut h.world).unwrap();
    }
    assert!(
        (h.player_x() - x0 - (3.0 * 2.0 * 0.5 + 2.0 * 0.5)).abs() < 1e-3,
        "{}",
        h.player_x() - x0
    );
    h.probe.schedule(&h.world, "0:type:Hi!").unwrap();
    let mut seen = Vec::new();
    for _ in 0..6 {
        h.probe.pump(&mut h.world);
        let i = input(&h);
        for code in ["KeyH", "KeyI", "Digit1"] {
            if i.pressed(code) {
                seen.push(code);
            }
        }
        h.probe.step(&mut h.world).unwrap();
    }
    assert_eq!(seen, ["KeyH", "KeyI", "Digit1"]);
    assert!(!input(&h).down("Digit1"), "every press is released");
    // A chord: the modifiers held with the key (Command apart from Control),
    // released after it.
    h.probe
        .schedule(
            &h.world,
            "0:key_down:Meta+Shift+ArrowLeft;0.05:key_up:Meta+Shift+ArrowLeft",
        )
        .unwrap();
    h.probe.pump(&mut h.world);
    let i = input(&h);
    assert!(i.pressed("ArrowLeft") && i.down("MetaLeft") && i.down("ShiftLeft"));
    assert!(!i.down("ControlLeft"), "Meta is Command, not Control");
    for _ in 0..4 {
        h.probe.step(&mut h.world).unwrap();
    }
    h.probe.pump(&mut h.world);
    let i = input(&h);
    assert!(!i.down("ArrowLeft") && !i.down("MetaLeft") && !i.down("ShiftLeft"));
    h.probe.schedule(&h.world, "0:key_down:Hyper+KeyA").unwrap();
    h.probe.pump(&mut h.world);
    assert!(
        h.probe
            .take_log()
            .iter()
            .any(|l| l.text.contains("'hyper' is not a modifier"))
    );
    h.probe.schedule(&h.world, "0:key_down:Bogus").unwrap();
    h.probe.pump(&mut h.world);
    assert!(
        h.probe
            .take_log()
            .iter()
            .any(|l| l.text.contains("refused key 'Bogus'"))
    );
}

#[test]
fn teleport_set_frame_and_dump_entity_write_through_the_schema() {
    let mut h = build(1, true);
    h.probe
        .schedule(&h.world, "0:teleport:crate:1,2,3;0:set:player.Health.hp=55;0:set:player.Mover.lane=4;0:frame_entity:crate;0:dump_entity:player:p.json;0:screenshot:shot.png:16;0:screenshot")
        .unwrap();
    h.probe.pump(&mut h.world);
    assert!(h.probe.take_log().iter().all(|l| !l.warning));
    let crate_ = h.probe.names().resolve(&h.world, "crate").unwrap();
    assert_eq!(
        h.world.get::<Transform>(crate_).unwrap().position,
        Vec3::new(1.0, 2.0, 3.0)
    );
    let player = h.probe.names().resolve(&h.world, "player").unwrap();
    assert_eq!(h.world.get::<Health>(player).unwrap().hp, 55.0);
    let camera = *h.world.resource::<Camera>().unwrap();
    let forward = camera.quaternion * Vec3::NEG_Z;
    let want = (Vec3::new(1.0, 2.0, 3.0) - camera.position).normalize();
    assert!(
        (forward - want).length() < 1e-5,
        "the camera's -Z points at the crate: {forward} vs {want}"
    );
    let outputs = h.probe.take_outputs();
    let Payload::Json(dump) = &outputs[0].payload else {
        panic!()
    };
    assert_eq!(outputs[0].file_name(), "p.json");
    assert_eq!(
        dump["components"]["Health"],
        json!({"hp": 55.0, "max": 100.0})
    );
    assert_eq!(
        dump["components"]["Mover"]["lane"],
        json!(4),
        "set writes non-tunable fields too"
    );
    let Payload::Png { width, height, .. } = &outputs[1].payload else {
        panic!()
    };
    assert_eq!(
        (*width, *height, outputs[1].file_name().as_str()),
        (16, 9, "shot.png"),
        "N is the long edge"
    );
    let Payload::Png { width, .. } = &outputs[2].payload else {
        panic!()
    };
    assert_eq!(
        (*width, outputs[2].file_name().as_str()),
        (64, "screenshot-000000.png")
    );

    // Refusals warn, name the fix and change nothing.
    h.probe
        .schedule(&h.world, "0:set:player.Health.hp=2000;0:set:player.Health.hp;0:teleport:ghost:1,2,3;0:set:#1.1.Health.hp=abc;0:dump_entity:ghost")
        .unwrap();
    h.probe.pump(&mut h.world);
    let log = h.probe.take_log();
    assert_eq!(log.iter().filter(|l| l.warning).count(), 5, "{log:?}");
    assert!(log[0].text.contains("above the maximum 1000"));
    assert!(log[2].text.contains("no live entity 'ghost'"));
    assert_eq!(h.world.get::<Health>(player).unwrap().hp, 55.0);

    // Without a renderer, a screenshot writes nothing and notes the frame
    // it did not take (named as it would have been), without a warning.
    let mut bare = build(1, false);
    bare.probe
        .schedule(
            &bare.world,
            "0:screenshot;0:screenshot:menu/a.png:64;0:screenshot:../x.png",
        )
        .unwrap();
    bare.probe.pump(&mut bare.world);
    assert!(bare.probe.take_outputs().is_empty());
    let log = bare.probe.take_log();
    assert_eq!(
        log.iter()
            .map(|l| (l.warning, l.text.as_str()))
            .collect::<Vec<_>>(),
        [
            (
                false,
                "0s screenshot: no renderer, frame unchecked (screenshot-000000.png)"
            ),
            (
                false,
                "0s screenshot: no renderer, frame unchecked (menu/a.png)"
            ),
            (
                true,
                "0s screenshot: output path '../x.png' leaves the output directory; write a relative path without '..'"
            ),
        ],
        "a path that leaves the output directory is still refused"
    );
    assert!(log[0].text.contains(og_probe::FRAME_UNCHECKED));
}

#[test]
fn steps_left_ends_a_script_whose_steps_a_restore_moved_past_its_scheduled_end() {
    // brasswick FINDINGS #13: a host ending the run at the tick the last
    // step was scheduled for (`untilTick`) stopped as soon as a restore
    // moved og.Time past it; the steps keep their delays, so the run ends
    // when none is left.
    let mut h = build(1, false);
    let report = h
        .probe
        .schedule(&h.world, "0.1:seed:3;0.5:seed:4;0.6:quit")
        .unwrap();
    let until = report.until_tick;
    assert_eq!(h.probe.steps_left(), 3);
    while h.probe.tick(&h.world) < 10 {
        h.probe.step(&mut h.world).unwrap();
    }
    assert_eq!(h.probe.steps_left(), 2, "the step at 0.1 s ran");
    // A save of tick 500 restored as tick 10 ended: og.Time jumps forward.
    h.probe.shift_schedule(10, 500);
    assert!(500 > until, "past the scheduled end ({until})");
    assert_eq!(h.probe.steps_left(), 2, "nothing ran or was dropped");
    let mut ticks = 0;
    while h.probe.steps_left() > 0 && !h.probe.quit_requested() {
        h.probe.step(&mut h.world).unwrap();
        ticks += 1;
        assert!(ticks < 1000, "the steps never ran");
    }
    assert!(h.probe.quit_requested());
    assert_eq!(h.probe.steps_left(), 0);
}

#[test]
fn a_gate_that_restores_a_peers_snapshot_keeps_the_steps_on_the_shared_ticks() {
    // og.net's lockstep: a joiner takes the authority's snapshot at the
    // start, and from then on both worlds run on one clock. The steps
    // stayed on this process's own ticks (shifted by the jump), so a `quit`
    // or an `expect` landed on different ticks on each peer.
    let mut authority = build(1, false);
    while authority.probe.tick(&authority.world) < 40 {
        authority.probe.step(&mut authority.world).unwrap();
    }
    let snapshot = authority.world.snapshot();
    let mut h = build(1, false);
    // 60 Hz: the dump at tick 30 runs before the jump, the quit is tick 60.
    h.probe
        .schedule(&h.world, "0.5:dump_world;1.0:quit")
        .unwrap();
    let mut restored = false;
    h.probe.set_gate(Box::new(move |world| {
        if !restored && world.time().tick == 10 {
            world.restore(&snapshot).unwrap();
            restored = true;
        }
        true
    }));
    run(&mut h, 1000);
    assert!(h.probe.quit_requested());
    assert_eq!(
        h.probe.tick(&h.world),
        60,
        "the quit ran on tick 60 of the shared clock"
    );
    // The dump at tick 30, which the jump from 10 to 40 passed, ran once,
    // with the first steps due after the jump (tick 40 taken, at 41).
    let dumps = json_outputs(&mut h);
    assert_eq!(dumps.len(), 1);
    assert_eq!(dumps[0].0, 41);
}
