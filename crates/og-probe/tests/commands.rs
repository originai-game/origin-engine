//! The command channel (design §3.3): grammar, scheduling on simulation
//! time, the core actions, the registry, `help`, and unknown actions that
//! warn and let the script continue. The scripted run's dump is written to
//! `$CARGO_TARGET_TMPDIR/scripted-run-dump-world.json`.
#![cfg(not(feature = "publish"))]

mod support;

use og_core::glam::Vec3;
use og_probe::command::{CORE_ACTIONS, parse};
use og_probe::{CommandDesc, Input, Payload};
use serde_json::{Value, json};
use support::{Camera, Harness, Health, ReadbackCapture, Stats, Transform, build};

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
fn a_scripted_run_seeds_advances_dumps_and_quits() {
    let script = "0.5:seed:7;1.0:advance:30;1.5:dump_world;2.0:quit";
    let mut h = build(1, false);
    let report = h.probe.schedule(&h.world, script).unwrap();
    assert_eq!(
        report.to_json(),
        json!({"scheduled": 4, "untilTick": 120, "warnings": []})
    );
    run(&mut h, 10_000);
    assert!(h.probe.quit_requested());
    assert_eq!(
        h.probe.tick(&h.world),
        120,
        "quit runs at the start of tick 120 and no tick follows"
    );
    assert_eq!(h.probe.pending(), 0);
    assert!(h.probe.take_log().iter().all(|l| !l.warning));
    let dumps = json_outputs(&mut h);
    assert_eq!(dumps.len(), 1);
    let (tick, action, path, dump) = &dumps[0];
    assert_eq!((*tick, action.as_str(), path), (90, "dump_world", &None));
    assert_eq!(dump["kind"], json!("world"));
    assert_eq!(dump["tick"], json!(90));
    assert_eq!(dump["entities"], json!(5));
    assert_eq!(dump["systems"].as_array().unwrap().len(), 3);
    assert_eq!(h.world.resource::<support::Stats>().unwrap().draws, 120);

    // Replaying the same script on the same world gives the same dump.
    let mut replay = build(1, false);
    replay.probe.schedule(&replay.world, script).unwrap();
    run(&mut replay, 10_000);
    let (_, _, _, replayed) = json_outputs(&mut replay).remove(0);
    assert_eq!(replayed, *dump, "a scripted run is deterministic");
    assert_eq!(replay.world.world_hash(), h.world.world_hash());

    // From another starting seed, `seed:7` makes the Rng stream the same from tick 30 on.
    let rng = |h: &Harness| *h.world.resource::<og_core::Rng>().unwrap();
    let mut other = build(99, false);
    other.probe.schedule(&other.world, script).unwrap();
    run(&mut other, 10_000);
    assert_eq!(rng(&other), rng(&h));
    let mut unseeded = build(99, false);
    unseeded
        .probe
        .schedule(&unseeded.world, "1.0:advance:30;1.5:dump_world;2.0:quit")
        .unwrap();
    run(&mut unseeded, 10_000);
    assert_ne!(rng(&unseeded), rng(&h), "without seed:7 the stream differs");

    let path =
        std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("scripted-run-dump-world.json");
    let file = json!({"script": script, "world": "crates/og-probe/tests/support/mod.rs (seed 1)", "dump": dump});
    std::fs::write(&path, serde_json::to_string_pretty(&file).unwrap() + "\n").unwrap();
    println!("wrote {}", path.display());
}

#[test]
fn help_lists_every_action_core_and_registered() {
    let mut h = build(1, false);
    h.probe
        .register_command(
            "spawn_wave",
            CommandDesc {
                usage: "<count>".into(),
                doc: "Spawn a wave of rocks.".into(),
            },
            |world, args| {
                let n: u32 = args
                    .first()
                    .and_then(|a| a.parse().ok())
                    .ok_or("spawn_wave:<count>")?;
                for _ in 0..n {
                    world
                        .spawn((Transform::at(0.0, 9.0, 0.0),))
                        .map_err(|e| e.reason().to_owned())?;
                }
                Ok(())
            },
        )
        .unwrap();
    let help = h.probe.help();
    let names: Vec<&str> = help.iter().map(|i| i.name.as_str()).collect();
    let mut expected: Vec<&str> = CORE_ACTIONS.iter().map(|(n, _, _)| *n).collect();
    expected.push("spawn_wave");
    assert_eq!(names, expected);
    for core in [
        "screenshot",
        "touch_move_to",
        "touch_move",
        "touch_start",
        "touch_end",
        "scroll",
        "key_down",
        "key_up",
        "type",
        "move_to_text",
        "teleport",
        "frame_entity",
        "dump_world",
        "dump_entity",
        "dump_ui",
        "set",
        "seed",
        "pause",
        "resume",
        "advance",
        "quit",
        "help",
    ] {
        assert!(names.contains(&core), "{core}");
    }
    assert!(
        help.iter()
            .all(|i| !i.doc.is_empty() && i.usage.starts_with(&i.name))
    );
    let wave = help.last().unwrap();
    assert_eq!(
        (wave.usage.as_str(), wave.core),
        ("spawn_wave:<count>", false)
    );

    h.probe
        .schedule(&h.world, "0:help;0:spawn_wave:4;0.1:quit")
        .unwrap();
    run(&mut h, 100);
    assert_eq!(h.world.entity_count(), 9, "the registered action ran");
    let outputs = h.probe.take_outputs();
    let Payload::Text(listing) = &outputs[0].payload else {
        panic!("help writes text")
    };
    for info in &help {
        assert!(listing.contains(&info.usage), "{} in help", info.usage);
    }
    assert!(listing.contains("spawn_wave:<count> (registered)"));
    assert_eq!(outputs[0].file_name(), "help-000000.txt");

    // Registration refusals.
    let noop = |_: &mut og_ecs::World, _: &[String]| Ok(());
    assert!(
        h.probe
            .register_command("screenshot", CommandDesc::default(), noop)
            .is_err(),
        "core names are taken"
    );
    assert!(
        h.probe
            .register_command("spawn_wave", CommandDesc::default(), noop)
            .is_err(),
        "registered names are taken"
    );
    assert!(
        h.probe
            .register_command("SpawnWave", CommandDesc::default(), noop)
            .is_err(),
        "names are snake_case"
    );
    assert!(
        h.probe
            .register_command("spawn:wave", CommandDesc::default(), noop)
            .is_err()
    );
}

#[test]
fn an_unknown_action_warns_and_the_script_continues() {
    let mut h = build(1, false);
    let report = h
        .probe
        .schedule(
            &h.world,
            "0.1:frobnicate:3;0.2:advance:x;0.3:seed:5;0.4:dump_world:out/w.json;0.5:quit",
        )
        .unwrap();
    assert_eq!(report.scheduled, 5);
    assert_eq!(report.warnings.len(), 1);
    assert!(
        report.warnings[0].contains("unknown action 'frobnicate'"),
        "{:?}",
        report.warnings
    );
    run(&mut h, 1000);
    assert!(
        h.probe.quit_requested(),
        "the steps after the unknown action ran"
    );
    assert_eq!(h.world.resource::<og_core::Rng>().unwrap().seed(), 5);
    let log = h.probe.take_log();
    let warnings: Vec<&str> = log
        .iter()
        .filter(|l| l.warning)
        .map(|l| l.text.as_str())
        .collect();
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert!(warnings[0].contains("unknown action 'frobnicate'") && warnings[0].contains("help"));
    assert!(warnings[1].contains("advance") && warnings[1].contains("whole number"));
    assert_eq!(log[0].tick, 6);
    let dumps = json_outputs(&mut h);
    assert_eq!(
        dumps[0].2.as_deref(),
        Some("out/w.json"),
        "the path argument is the output's path"
    );
}

#[test]
fn the_grammar_is_strict_about_times_and_names() {
    let steps =
        parse(" 0.5:seed:7 ;\n1:type:a\\:b\\;c: d ;; 1:set:player.Mover.speed=3\n").unwrap();
    assert_eq!(steps.len(), 3);
    assert_eq!(
        (steps[0].at, steps[0].action.as_str(), steps[0].args.clone()),
        (0.5, "seed", vec!["7".to_owned()])
    );
    assert_eq!(
        steps[1].args,
        vec!["a:b;c".to_owned(), " d ".to_owned()],
        "escapes, and the spaces of typed text kept to the `;` (lantern-vale FINDINGS #21: `type:` lost a trailing space)"
    );
    // Other actions' arguments end where their text does; typed text keeps
    // its spaces, but not trailing white space that holds a line break
    // (a script's layout).
    let typed = parse("1:dump_ui:a.json  ;2:commit:小灯 \n  ;3:compose:x ").unwrap();
    assert_eq!(typed[0].args, vec!["a.json".to_owned()]);
    assert_eq!(typed[1].args, vec!["小灯".to_owned()]);
    assert_eq!(typed[2].args, vec!["x ".to_owned()]);
    assert_eq!(
        (steps[2].action.as_str(), steps[2].args.clone()),
        ("set", vec!["player.Mover.speed=3".to_owned()])
    );
    assert!(parse("").unwrap().is_empty());
    for (bad, step, says) in [
        ("x:seed", 1, "not a time"),
        ("1:seed;0.5:quit", 2, "times ascend"),
        ("-1:quit", 1, "≥ 0"),
        ("inf:quit", 1, "finite"),
        ("1", 1, "no action"),
        ("1:", 1, "not an action name"),
        ("1:Quit", 1, "not an action name"),
        ("1:quit;2:dump-world", 2, "not an action name"),
    ] {
        let err = parse(bad).unwrap_err();
        assert_eq!(err.step, step, "{bad}: {err}");
        assert!(err.reason.contains(says), "{bad}: {err}");
    }
}

#[test]
fn steps_run_on_simulation_time_even_inside_an_advance() {
    let mut h = build(1, false);
    // At 0.5 s (tick 30) advance 60 ticks; the dump at 1.0 s (tick 60) runs during it.
    h.probe
        .schedule(&h.world, "0.5:advance:60;1.0:dump_world;1.1:pause")
        .unwrap();
    for _ in 0..30 {
        h.probe.step(&mut h.world).unwrap();
    }
    h.probe.pump(&mut h.world);
    assert_eq!(h.probe.tick(&h.world), 90);
    assert_eq!(json_outputs(&mut h)[0].0, 60);
    assert!(h.probe.paused());
    // A held clock takes no frame ticks; due steps still run.
    assert_eq!(h.probe.frame(&mut h.world, 4).unwrap(), 0);
    h.probe.schedule(&h.world, "0:resume").unwrap();
    assert_eq!(h.probe.frame(&mut h.world, 4).unwrap(), 4);
    assert_eq!(h.probe.tick(&h.world), 94);
    // The protocol's commands member schedules the same way.
    let r = h.result("commands", json!({"script": "0:advance:6"}));
    assert_eq!(r["untilTick"], json!(94));
    h.probe.pump(&mut h.world);
    assert_eq!(h.probe.tick(&h.world), 100);
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

    // Without a renderer, screenshot warns.
    let mut bare = build(1, false);
    bare.probe.schedule(&bare.world, "0:screenshot").unwrap();
    bare.probe.pump(&mut bare.world);
    assert!(bare.probe.take_log()[0].text.contains("no renderer"));
}

#[test]
fn a_screenshot_whose_frame_lands_later_holds_the_clock_and_the_steps_after_it() {
    let mut h = build(1, false);
    h.probe.set_capture(Box::new(ReadbackCapture::default()));
    h.probe
        .schedule(&h.world, "0:screenshot:a.png;0:set:player.Health.hp=55")
        .unwrap();
    let player = h.probe.names().resolve(&h.world, "player").unwrap();

    // The frame is asked for: no tick is taken and the step after it waits.
    assert_eq!(h.probe.frame(&mut h.world, 3).unwrap(), 0);
    assert!(h.probe.take_outputs().is_empty());
    assert_eq!(h.world.get::<Health>(player).unwrap().hp, 100.0);

    // It lands: written at the tick it was asked at, then the held step
    // runs and the clock goes on.
    assert_eq!(h.probe.frame(&mut h.world, 3).unwrap(), 3);
    let outputs = h.probe.take_outputs();
    assert_eq!(outputs.len(), 1);
    assert_eq!(
        (outputs[0].tick, outputs[0].file_name().as_str()),
        (0, "a.png")
    );
    let Payload::Png { bytes, .. } = &outputs[0].payload else {
        panic!()
    };
    assert!(String::from_utf8_lossy(bytes).starts_with("64x36 tick=0 "));
    assert_eq!(h.world.get::<Health>(player).unwrap().hp, 55.0);
    assert!(h.probe.take_log().iter().all(|l| !l.warning));
}

#[test]
fn set_and_get_reach_resources_entities_components_and_fields() {
    let mut h = build(1, false);
    h.probe
        .schedule(
            &h.world,
            "0:set:Stats.draws=7;0:set:og.Camera.position=1,2,3;0:set:player.Health.hp=40;\
             0:get:Stats.draws;0:get:og.Camera:camera.json;0:get:player.Health.hp;0:get:player.Health;0:get:player",
        )
        .unwrap();
    h.probe.pump(&mut h.world);
    let log = h.probe.take_log();
    assert!(log.iter().all(|l| !l.warning), "{log:?}");
    assert_eq!(h.world.resource::<Stats>().unwrap().draws, 7);
    assert_eq!(
        h.world.resource::<Camera>().unwrap().position,
        Vec3::new(1.0, 2.0, 3.0),
        "a resource name with a dot"
    );
    let notes: Vec<&str> = log.iter().map(|l| l.text.as_str()).collect();
    assert!(notes.contains(&"get Stats.draws = 7"), "{notes:?}");
    assert!(notes.contains(&"get player.Health.hp = 40.0"), "{notes:?}");
    let outputs: Vec<(String, Value)> = h
        .probe
        .take_outputs()
        .into_iter()
        .filter_map(|o| match &o.payload {
            Payload::Json(v) => Some((o.file_name(), v["value"].clone())),
            _ => None,
        })
        .collect();
    let values: Vec<(&str, &Value)> = outputs.iter().map(|(n, v)| (n.as_str(), v)).collect();
    assert_eq!(values[0], ("get-000000.json", &json!(7)));
    assert_eq!(values[1].0, "camera.json");
    assert_eq!(values[1].1["position"], json!([1.0, 2.0, 3.0]));
    assert_eq!(values[2].1, &json!(40.0));
    assert_eq!(values[3].1, &json!({"hp": 40.0, "max": 100.0}));
    assert_eq!(values[4].1["components"]["Health"]["hp"], json!(40.0));

    // Refusals name the fix and change nothing.
    h.probe
        .schedule(
            &h.world,
            "0:set:Stats.nope=1;0:get:Stats.nope;0:get:ghost;0:get:player.Nope;0:set:Stats.draws=x",
        )
        .unwrap();
    h.probe.pump(&mut h.world);
    let log = h.probe.take_log();
    assert_eq!(log.iter().filter(|l| l.warning).count(), 5, "{log:?}");
    assert!(
        log[0].text.contains("'Stats' has no field 'nope'"),
        "{log:?}"
    );
    assert!(
        log[2].text.contains("no resource or live entity 'ghost'"),
        "{log:?}"
    );
    assert_eq!(h.world.resource::<Stats>().unwrap().draws, 7);
}

#[test]
fn get_answers_og_time_whole_and_unnamed_gets_on_one_tick_keep_their_files() {
    let mut h = build(1, false);
    h.probe
        .schedule(
            &h.world,
            "0:get:og.Time;0:get:og.Time.scaledDt;0:get:og.Time.rate;0:get:Stats.draws:named.json;0:get:og.TimeScale",
        )
        .unwrap();
    h.probe.pump(&mut h.world);
    let log = h.probe.take_log();
    assert!(log.iter().all(|l| !l.warning), "{log:?}");
    let outputs: Vec<(String, Value)> = h
        .probe
        .take_outputs()
        .into_iter()
        .filter_map(|o| match &o.payload {
            Payload::Json(v) => Some((o.file_name(), v["value"].clone())),
            _ => None,
        })
        .collect();
    let names: Vec<&str> = outputs.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        [
            "get-000000.json",
            "get-000000-2.json",
            "get-000000-3.json",
            "named.json",
            "get-000000-4.json"
        ]
    );
    // og.Time as game code reads it: the clock, its rate and the time scale.
    let time = &outputs[0].1;
    for key in [
        "tick",
        "dt",
        "elapsed",
        "rate",
        "scale",
        "hitStop",
        "scaledDt",
        "scaledElapsed",
    ] {
        assert!(time.get(key).is_some(), "og.Time has no {key}: {time}");
    }
    assert_eq!(time["scale"], json!(1.0));
    assert_eq!(outputs[1].1, time["dt"]);
    assert_eq!(outputs[2].1, json!(60));
    // og.TimeScale, where the world keeps scale and hit stop, answers too.
    assert_eq!(outputs[4].1["scale"], json!(1.0));

    // A write outside a field's range is refused, naming the range and the fix.
    h.probe
        .schedule(&h.world, "0:set:player.Health.hp=2000")
        .unwrap();
    h.probe.pump(&mut h.world);
    let log = h.probe.take_log();
    assert!(
        log.iter().any(|l| l.warning
            && l.text.contains("above the maximum 1000")
            && l.text.contains("FieldOptions")),
        "{log:?}"
    );
}

#[test]
fn output_paths_stay_inside_the_output_directory() {
    let mut h = build(1, true);
    h.probe
        .schedule(&h.world, "0:dump_world:../escape.json;0:dump_world:/tmp/abs.json;0:dump_entity:player:a/../../b.json;0:screenshot:../src/main.ts;0:dump_world:shots/w.json")
        .unwrap();
    h.probe.pump(&mut h.world);
    let log = h.probe.take_log();
    let refused: Vec<_> = log.iter().filter(|l| l.warning).collect();
    assert_eq!(refused.len(), 4, "{log:?}");
    assert!(
        refused
            .iter()
            .all(|l| l.text.contains("leaves the output directory")),
        "{log:?}"
    );
    let outputs = h.probe.take_outputs();
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].file_name(), "shots/w.json");
}

#[derive(Clone, Copy, Debug, PartialEq, og_core::Reflect)]
struct Mix {
    #[reflect(variants = ["sfx", "music", "ui"])]
    bus: u32,
}

#[test]
fn get_answers_an_enum_by_its_variant_name_and_set_takes_the_name_or_the_index() {
    let mut h = build(1, false);
    h.world.register::<Mix>().unwrap();
    let player = h.world.find("player").unwrap();
    h.world.insert(player, Mix { bus: 1 }).unwrap();
    h.probe
        .schedule(
            &h.world,
            "0:get:player.Mix.bus:a.json;0:get:player.Mix:b.json;0:dump_entity:player:c.json;\
             0:set:player.Mix.bus=ui;0:get:player.Mix.bus:d.json;0:set:player.Mix.bus=0;0:get:player.Mix.bus:e.json;\
             0:set:player.Mix.bus=radio",
        )
        .unwrap();
    h.probe.pump(&mut h.world);
    let log = h.probe.take_log();
    let warnings: Vec<&str> = log
        .iter()
        .filter(|l| l.warning)
        .map(|l| l.text.as_str())
        .collect();
    assert_eq!(warnings.len(), 1, "{log:?}");
    assert!(
        warnings[0].contains("\"radio\" is not one of sfx, music, ui"),
        "{warnings:?}"
    );
    let notes: Vec<&str> = log.iter().map(|l| l.text.as_str()).collect();
    assert!(
        notes.contains(&"get player.Mix.bus = \"music\""),
        "{notes:?}"
    );
    let values: Vec<Value> = json_outputs(&mut h)
        .into_iter()
        .map(|(_, action, _, v)| {
            if action == "get" {
                v["value"].clone()
            } else {
                v
            }
        })
        .collect();
    assert_eq!(
        values[0],
        json!("music"),
        "as game code and the types read it"
    );
    assert_eq!(values[1], json!({ "bus": "music" }));
    assert_eq!(values[2]["components"]["Mix"]["bus"], json!("music"));
    assert_eq!(values[3], json!("ui"), "set by the name");
    assert_eq!(values[4], json!("sfx"), "set by the index");
    assert_eq!(
        h.world.get::<Mix>(player).unwrap().bus,
        0,
        "the refused write changed nothing"
    );
}

#[test]
fn a_second_step_naming_a_written_file_goes_beside_it_with_a_warning_naming_both() {
    let mut h = build(1, false);
    h.probe
        .schedule(
            &h.world,
            "0:get:Stats:x.json;0.5:dump_entity:player:x.json;1:dump_world:x.json;1:dump_world;1:dump_world",
        )
        .unwrap();
    run(&mut h, 61);
    let log = h.probe.take_log();
    let warnings: Vec<&str> = log
        .iter()
        .filter(|l| l.warning)
        .map(|l| l.text.as_str())
        .collect();
    assert_eq!(
        warnings.len(),
        2,
        "two clashes named, the unnamed dumps are not: {log:?}"
    );
    assert!(
        warnings[0].contains("0.5s dump_entity: 'x.json' was already written by the 0s get step, so this one is 'x-2.json'"),
        "{warnings:?}"
    );
    assert!(
        warnings[1].contains("1s dump_world") && warnings[1].contains("'x-3.json'"),
        "{warnings:?}"
    );
    let names: Vec<String> = json_outputs(&mut h)
        .into_iter()
        .map(|(tick, action, path, _)| path.unwrap_or_else(|| format!("{action}-{tick:06}.json")))
        .collect();
    assert_eq!(
        names,
        [
            "x.json",
            "x-2.json",
            "x-3.json",
            "dump_world-000060.json",
            "dump_world-000060-2.json"
        ]
    );
    // A new script starts its own names.
    h.probe.schedule(&h.world, "0:dump_world:x.json").unwrap();
    h.probe.pump(&mut h.world);
    assert!(h.probe.take_log().iter().all(|l| !l.warning));
    assert_eq!(json_outputs(&mut h)[0].2.as_deref(), Some("x.json"));
}

#[test]
fn a_dot_name_is_kept_for_the_harness() {
    let mut h = build(1, false);
    h.probe
        .schedule(
            &h.world,
            "0:dump_world:.store/platform.json;0:dump_world:a/.hidden.json;0:dump_world:./ok.json",
        )
        .unwrap();
    h.probe.pump(&mut h.world);
    let log = h.probe.take_log();
    let refused: Vec<_> = log.iter().filter(|l| l.warning).collect();
    assert_eq!(refused.len(), 2, "{log:?}");
    assert!(
        refused
            .iter()
            .all(|l| l.text.contains("starting with '.'")
                && l.text.contains("without the leading dot")),
        "{log:?}"
    );
    let outputs = h.probe.take_outputs();
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].file_name(), "./ok.json");
}
