//! Expectations (`expect:` actions, the `expect` program operation and
//! member) and `describe` over a scene view: what passes, what fails with
//! its tick and observed value, and the frame as data.
#![cfg(not(feature = "publish"))]

use crate::support;

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
    fn seen(&mut self, world: &World, _cover: &[[f32; 4]]) -> Option<Seen> {
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
            joints: Vec::new(),
        };
        Some(Seen {
            pass: (32, 18),
            camera: json!({"projection": "perspective"}),
            entities: vec![
                shown(crate_, [40.0, 10.0, 10.0, 10.0], 9.0, 0.25),
                shown(player, [10.0, 10.0, 8.0, 12.0], 4.0, 0.0),
            ],
            views: Vec::new(),
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

/// A renderer whose frame is one grey, the tick's number: the frame of one
/// tick tells itself apart from every other tick's.
struct TickGrey;

impl og_probe::Capture for TickGrey {
    fn tier(&self) -> &str {
        "vulkan"
    }
    fn target_size(&self) -> (u32, u32) {
        (4, 2)
    }
    fn capture(&mut self, world: &World, width: u32, height: u32) -> Option<og_probe::Frame> {
        let (width, height) = if width == 0 { (4, 2) } else { (width, height) };
        let v = world.time().tick as u8;
        let image = og_probe::frame::Rgba {
            width,
            height,
            pixels: [v, v, v, 255].repeat((width * height) as usize),
        };
        Some(og_probe::Frame {
            width,
            height,
            png: og_probe::frame::encode_png(&image),
        })
    }
}

#[test]
fn a_failed_frame_check_writes_the_frame_it_compared_at_its_own_tick() {
    let mut h = build(1, false);
    h.probe.set_capture(Box::new(TickGrey));
    let reference = og_probe::frame::encode_png(&og_probe::frame::Rgba {
        width: 4,
        height: 2,
        pixels: [200, 200, 200, 255].repeat(8),
    });
    let wanted = reference.clone();
    h.probe
        .set_reader(move |path| (path == "refs/grey.png").then(|| wanted.clone()));
    // The check fails at tick 30; the run goes on to tick 50, whose frame
    // differs, so only the compared one is the evidence.
    let script =
        "0.5:expect:frame:refs/grey.png:rmse:<=1;0.6:expect:region:0,0,1,1:color:#c8c8c8:2";
    h.probe.schedule(&h.world, script).unwrap();
    run(&mut h, 50);
    let results = h.probe.take_expectations();
    assert_eq!(
        results
            .iter()
            .map(|r| (r.status, r.tick))
            .collect::<Vec<_>>(),
        vec![(Status::Fail, 30), (Status::Fail, 36)]
    );
    assert!(
        results[0]
            .reason
            .ends_with("the compared frame is .failure/frame-grey@30.png"),
        "{}",
        results[0].reason
    );
    let out = h.probe.take_outputs();
    let files: Vec<(String, u64)> = out.iter().map(|o| (o.file_name(), o.tick)).collect();
    assert_eq!(
        files,
        vec![
            (".failure/frame-grey@30.png".into(), 30),
            (".failure/frame-grey@30.reference.png".into(), 30),
            (".failure/region@36.png".into(), 36),
        ]
    );
    let grey = |i: usize| {
        let og_probe::Payload::Png { bytes, .. } = &out[i].payload else {
            panic!("{} is no PNG", files[i].0);
        };
        og_probe::frame::decode_png(bytes).unwrap().pixels[0]
    };
    assert_eq!((grey(0), grey(1), grey(2)), (30, 200, 36));
}
