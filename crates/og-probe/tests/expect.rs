//! Expectations (`expect:` actions, the `expect` program operation and
//! member) and `describe` over a scene view: what passes, what fails with
//! its tick and observed value, and the frame as data.
#![cfg(not(feature = "publish"))]

mod support;

use og_core::Reflect;
use og_ecs::World;
use og_probe::{Pick, SceneView, Seen, Shown, Status};
use serde_json::{Value, json};
use support::{Harness, build};

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Hit {
    damage: f32,
}

/// A scene view that sees the player near and the crate far, at fixed boxes.
struct FixedView;

impl SceneView for FixedView {
    fn seen(&mut self, world: &World) -> Option<Seen> {
        let player = world.find("player")?;
        let crate_ = world.find("crate")?;
        let shown = |entity, rect: [f32; 4], depth: f32, occlusion: f32| Shown {
            entity,
            rect,
            drawn: rect,
            point: [rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0],
            depth,
            coverage: 0.01,
            occlusion,
            labels: vec!["mesh".into()],
        };
        Some(Seen {
            pass: (32, 18),
            camera: json!({"projection": "perspective"}),
            entities: vec![
                shown(crate_, [40.0, 10.0, 10.0, 10.0], 9.0, 0.25),
                shown(player, [10.0, 10.0, 8.0, 12.0], 4.0, 0.0),
            ],
        })
    }

    fn pick(&mut self, world: &World, x: f32, _y: f32) -> Option<Pick> {
        (x > 30.0).then(|| Pick {
            entity: world.find("crate").expect("a crate"),
            world: vec![5.0, 0.0, 0.0],
            source: None,
        })
    }
}

fn run(h: &mut Harness, ticks: u64) {
    for _ in 0..ticks {
        h.probe.step(&mut h.world).unwrap();
    }
}

#[test]
fn a_script_of_expectations_reports_each_with_its_tick_and_observed_value() {
    let mut h = build(1, false);
    h.world.register::<Hit>().unwrap();
    h.probe.set_scene_view(Box::new(FixedView));
    let script = [
        "0.1:expect:ui:text:Start game",
        "0.2:expect:ui:text:=Quit",
        "0.3:expect:entity:player:within:0,-1,-1,2,1,1",
        "0.3:expect:entity:player:within:-5,-1,-1,0,1,1",
        "0.4:expect:entity:player:within:0,0,30,30",
        "0.4:expect:entity:crate:within:0,0,30,30",
        "0.5:expect:state:draws>=20",
        "0.5:expect:state:draws==3",
        "0.6:expect:event:Hit:2",
        "0.6:expect:event:Hit",
        "0.7:expect:region:0,0,4,4:color:#000000",
    ]
    .join(";");
    h.probe.schedule(&h.world, &script).unwrap();
    // Two hits sent between ticks, one by the host 20 ticks in.
    run(&mut h, 20);
    h.world.send(Hit { damage: 1.0 }).unwrap();
    h.world.send(Hit { damage: 2.0 }).unwrap();
    run(&mut h, 30);
    let results = h.probe.take_expectations();
    let status: Vec<(&str, Status)> = results
        .iter()
        .map(|r| (r.expect.as_str(), r.status))
        .collect();
    assert_eq!(
        status,
        vec![
            ("ui:text:Start game", Status::Pass),
            ("ui:text:=Quit", Status::Fail),
            ("entity:player:within:0,-1,-1,2,1,1", Status::Pass),
            ("entity:player:within:-5,-1,-1,0,1,1", Status::Fail),
            ("entity:player:within:0,0,30,30", Status::Pass),
            ("entity:crate:within:0,0,30,30", Status::Fail),
            ("state:draws>=20", Status::Pass),
            ("state:draws==3", Status::Fail),
            ("event:Hit:2", Status::Pass),
            ("event:Hit", Status::Pass),
            ("region:0,0,4,4:color:#000000", Status::Unchecked),
        ]
    );
    // A failure carries the tick it ran at and what was there.
    let quit = &results[1];
    assert_eq!(quit.tick, 12);
    assert_eq!(quit.observed, json!(["Start game", "Options", "Start"]));
    let draws = &results[7];
    assert_eq!((draws.tick, draws.observed.clone()), (30, json!(30)));
    let away = &results[3];
    // The player moved 2 m/s × 1 for 18 ticks.
    assert_eq!(away.observed, json!([0.6, 0.0, 0.0]));
    let boxed = &results[5];
    assert_eq!(
        boxed.observed,
        json!({"x": 40.0, "y": 10.0, "width": 10.0, "height": 10.0})
    );
    // Failures are warnings in the log, naming the tick; passes are not.
    let log = h.probe.take_log();
    let warned: Vec<&str> = log
        .iter()
        .filter(|l| l.warning)
        .map(|l| l.text.as_str())
        .collect();
    assert_eq!(warned.len(), 4, "{warned:?}");
    assert!(warned[0].starts_with("expect ui:text:=Quit: FAIL at tick 12"));
}

#[test]
fn a_malformed_expectation_is_a_warning_not_a_result() {
    let mut h = build(1, false);
    h.probe
        .schedule(&h.world, "0:expect:state:started;0:expect:what")
        .unwrap();
    run(&mut h, 1);
    assert!(h.probe.take_expectations().is_empty());
    let log = h.probe.take_log();
    assert_eq!(log.iter().filter(|l| l.warning).count(), 2, "{log:?}");
}

#[test]
fn programs_and_the_member_answer_results_and_expectations_drains_them() {
    let mut h = build(1, false);
    let program = json!([
        {"op": "advance", "steps": 10},
        {"op": "expect", "what": "state:draws==10"},
        {"op": "expect", "what": "state:draws==11"},
        {"op": "advance", "steps": 1},
    ]);
    let r = h.result("run", json!({"program": program}));
    assert_eq!(
        r["refused"],
        Value::Null,
        "a failed expectation does not stop the program"
    );
    assert_eq!(r["results"][1]["status"], json!("pass"));
    assert_eq!(r["results"][2]["status"], json!("fail"));
    assert_eq!(r["results"][2]["observed"], json!(10));
    let one = h.result("expect", json!({"what": "ui:text:Options"}));
    assert_eq!(one["status"], json!("pass"));
    let bad = h.call("expect", json!({"what": "nope"}));
    assert_eq!(bad["error"]["code"], json!(-32602));
    let all = h.result("expectations", json!({}));
    assert_eq!(
        (all["passed"].clone(), all["failed"].clone()),
        (json!(2), json!(1))
    );
    let again = h.result("expectations", json!({}));
    assert_eq!(again["results"], json!([]));
}

#[test]
fn describe_orders_entities_nearest_first_with_ui_and_pick_falls_back_to_the_view() {
    let mut h = build(1, false);
    h.probe.set_scene_view(Box::new(FixedView));
    let d = h.result("describe", json!({}));
    assert_eq!(d["kind"], json!("describe"));
    assert_eq!(d["pass"], json!({"width": 32, "height": 18}));
    let ids: Vec<&str> = d["entities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["player", "crate"]);
    assert_eq!(d["entities"][1]["order"], json!(1));
    assert_eq!(d["entities"][1]["occlusion"], json!(0.25));
    assert!(
        d["entities"][0]["archetype"]
            .as_array()
            .unwrap()
            .contains(&json!("Mover"))
    );
    assert_eq!(d["ui"].as_array().unwrap().len(), 5);
    assert_eq!(d["ui"][1]["text"], json!("Start game"));
    // Same world, same answer.
    assert_eq!(h.result("describe", json!({})), d);
    // No capture hook picks here: the scene view answers.
    let hit = h.result("pick", json!({"x": 45, "y": 15}));
    assert_eq!(hit["entity"], json!("crate"));
    assert_eq!(h.result("pick", json!({"x": 5, "y": 15})), Value::Null);
    // describe:<out> writes the same JSON as an output.
    h.probe.schedule(&h.world, "0:describe:frame.json").unwrap();
    run(&mut h, 1);
    let out = h.probe.take_outputs();
    assert_eq!(out[0].file_name(), "frame.json");
}
