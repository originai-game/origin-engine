//! Protocol 5 conformance through the transport-free JSON entry: every
//! member, every refusal rule, every malformed-call code, as
//! `origin/packages/contracts/src/probe.ts` and `engine/docs/probe-wire.md`
//! fix them. A refusal is `result: false` or `result: null`; `error` is only
//! for a malformed call (-32700, -32600, -32602), an unknown method (-32601)
//! or a fault (-32603). The whole exchange is written to
//! `$CARGO_TARGET_TMPDIR/conformance-transcript.json`.
#![cfg(not(feature = "publish"))]

mod support;

use og_ecs::SystemDesc;
use og_probe::{MEMBERS, Rebuilt, Reloader};
use serde_json::{Value, json};
use support::{Harness, ReadbackCapture, base64_decode, build, bundle};

const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;
const INTERNAL_ERROR: i64 = -32603;

/// Calls and their answers, in order, for the evidence file.
struct Transcript {
    h: Harness,
    log: Vec<Value>,
    checks: usize,
}

impl Transcript {
    fn frame(&mut self, frame: &str) -> Value {
        let answer = self.h.raw(frame);
        let call = serde_json::from_str::<Value>(frame).unwrap_or(Value::String(frame.into()));
        self.log.push(json!({"call": call, "answer": answer}));
        answer
    }

    fn call(&mut self, method: &str, params: Value) -> Value {
        let id = self.log.len() + 1;
        let frame =
            json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string();
        let answer = self.frame(&frame);
        assert_eq!(answer["jsonrpc"], json!("2.0"), "{answer}");
        assert_eq!(
            answer["id"],
            json!(id),
            "the answer carries the call's id: {answer}"
        );
        answer
    }

    /// A result (refusals included); fails on an error frame.
    fn ok(&mut self, method: &str, params: Value) -> Value {
        let answer = self.call(method, params);
        assert!(
            answer.get("error").is_none(),
            "{method}: unexpected error {answer}"
        );
        self.checks += 1;
        answer["result"].clone()
    }

    /// An error frame with exactly this code.
    fn err(&mut self, method: &str, params: Value, code: i64) {
        let answer = self.call(method, params);
        assert!(
            answer.get("result").is_none(),
            "{method}: expected error {code}, got {answer}"
        );
        assert_eq!(answer["error"]["code"], json!(code), "{method}: {answer}");
        assert!(answer["error"]["message"].is_string());
        self.checks += 1;
    }

    fn tick(&mut self) -> u64 {
        self.ok("tick", json!({})).as_u64().unwrap()
    }
}

#[test]
fn every_member_answers_as_protocol_5_says() {
    let mut t = Transcript {
        h: build(7, true),
        log: Vec::new(),
        checks: 0,
    };

    // hello: who is speaking.
    let hello = t.ok("hello", json!({}));
    assert_eq!(hello["protocol"], json!(5));
    assert_eq!(hello["wire"], json!(1));
    assert_eq!(hello["substrate"], json!("origin-engine"));
    assert!(
        hello["engine"]
            .as_str()
            .unwrap()
            .starts_with("origin-engine ")
    );
    assert_eq!(hello["ticksPerSecond"], json!(60));
    assert_eq!(hello["tier"], json!("vulkan"));
    assert!(["dev", "release"].contains(&hello["build"].as_str().unwrap()));
    assert_eq!(hello["members"], json!(MEMBERS));
    assert_eq!(MEMBERS.len(), 22, "13 protocol-5 members + 9 wire members");

    // tick, state: the fixed head, then the game's facts.
    assert_eq!(t.tick(), 0);
    let state = t.ok("state", json!({}));
    assert_eq!(state["tick"], json!(0));
    assert_eq!(state["paused"], json!(false));
    assert_eq!(state["seed"], json!(7));
    for k in ["step", "render", "frameMs"] {
        assert!(state["timings"][k].is_number(), "timings.{k}: {state}");
    }
    assert_eq!(
        state["entities"],
        json!([{"id": "player"}, {"id": "crate"}])
    );
    assert_eq!(state["facts"], json!({"draws": 0}));
    t.h.probe.set_timings(1.5, 4.0);
    assert_eq!(
        t.ok("state", json!({}))["timings"],
        json!({"step": 1.5, "render": 4.0, "frameMs": 5.5})
    );

    // pause / resume: true, reflected in state; advance still steps while held.
    assert_eq!(t.ok("pause", json!({})), json!(true));
    assert_eq!(t.ok("state", json!({}))["paused"], json!(true));
    assert_eq!(t.ok("advance", json!({"steps": 60})), json!(60));
    assert!(t.h.probe.paused(), "advance keeps the pause state");
    assert_eq!(t.ok("resume", json!({})), json!(true));
    assert_eq!(t.ok("state", json!({}))["paused"], json!(false));
    assert_eq!(t.tick(), 60);
    assert!(
        (t.h.player_x() - 2.0).abs() < 1e-4,
        "speed 2 for one second: {}",
        t.h.player_x()
    );
    assert_eq!(t.ok("advance", json!({"steps": 0})), json!(60));

    // advance: malformed steps are errors and move nothing.
    for steps in [json!(-1), json!(3601), json!("5"), json!(1.5), Value::Null] {
        t.err("advance", json!({"steps": steps}), INVALID_PARAMS);
    }
    t.err("advance", json!({}), INVALID_PARAMS);
    assert_eq!(t.tick(), 60, "a malformed call moves nothing");

    // input: through the input layer's own path; false is a refusal.
    let right = json!({"kind": "key", "key": "ArrowRight", "action": "down"});
    assert_eq!(t.ok("input", json!({"event": right})), json!(true));
    assert_eq!(t.ok("advance", json!({"steps": 30})), json!(90));
    assert!(
        (t.h.player_x() - 5.0).abs() < 1e-4,
        "held ArrowRight triples the velocity: {}",
        t.h.player_x()
    );
    let up = json!({"kind": "key", "key": "ArrowRight", "action": "up", "modifiers": {"alt": false, "control": false, "platform": false, "shift": true}});
    assert_eq!(t.ok("input", json!({"event": up})), json!(true));
    for refused in [
        // An unknown kind (gamepad is an engine extension now; its pad 9 is
        // past the four pads and refused by the input layer).
        json!({"kind": "joystick"}),
        json!({"kind": "gamepad", "pad": 9}),
        json!({"kind": "key", "key": "NotAKey", "action": "down"}),
        json!({"kind": "key", "key": "", "action": "down"}),
        json!({"kind": "key", "key": "KeyA", "action": "tap"}),
        json!({"kind": "key", "key": "KeyA", "action": "down", "modifiers": {"alt": true}}),
        json!({"kind": "pointer", "action": "move", "x": "1", "y": 0, "button": "none"}),
        json!({"kind": "pointer", "action": "move", "x": 1, "y": 0, "button": "back"}),
        json!({"kind": "wheel", "x": 0, "y": 0, "deltaX": 0}),
        json!(null),
    ] {
        assert_eq!(
            t.ok("input", json!({"event": refused})),
            json!(false),
            "{refused}"
        );
    }
    let click = json!({"kind": "pointer", "action": "click", "x": 3, "y": 4.5, "button": "left"});
    assert_eq!(t.ok("input", json!({"event": click})), json!(true));
    let wheel = json!({"kind": "wheel", "x": 3, "y": 4.5, "deltaX": 0, "deltaY": -120});
    assert_eq!(t.ok("input", json!({"event": wheel})), json!(true));
    t.err("input", json!({}), INVALID_PARAMS);

    // accepted: what the input layer took, refusals not counted.
    assert_eq!(t.ok("accepted", json!({})), json!(4));

    // pick: a hit, nothing (null), malformed coordinates.
    assert_eq!(
        t.ok("pick", json!({"x": 5, "y": 3})),
        json!({"entity": "player", "world": [5.0, 3.0, 0.0], "source": "player.ts::move"})
    );
    assert_eq!(t.ok("pick", json!({"x": 50, "y": 3})), Value::Null);
    t.err("pick", json!({"x": "a", "y": 3}), INVALID_PARAMS);
    t.err("pick", json!({"x": 1}), INVALID_PARAMS);

    // entity: tunable fields as Component.field; null for an unknown entity.
    assert_eq!(
        t.ok("entity", json!({"id": "player"})),
        json!({"Health.hp": 100.0, "Mover.speed": 2.0})
    );
    assert_eq!(t.ok("entity", json!({"id": "ghost"})), Value::Null);
    let crate_id = {
        let e = t.h.probe.names().resolve(&t.h.world, "crate").unwrap();
        format!("#{}.{}", e.index(), e.generation())
    };
    assert_eq!(
        t.ok("entity", json!({"id": crate_id})),
        json!({"Health.hp": 30.0}),
        "unnamed ids address an entity too"
    );
    assert_eq!(t.ok("entity", json!({"id": "#99.1"})), Value::Null);
    t.err("entity", json!({}), INVALID_PARAMS);
    t.err("entity", json!({"id": 3}), INVALID_PARAMS);

    // bounds: the schema's ranges; null for an unknown entity.
    assert_eq!(
        t.ok("bounds", json!({"id": "player"})),
        json!({
            "Health.hp": {"path": "Health.hp", "label": "Hit points", "min": 0.0, "max": 1000.0},
            "Mover.speed": {"path": "Mover.speed", "label": "Speed", "min": 0.0, "max": 20.0},
        })
    );
    assert_eq!(t.ok("bounds", json!({"id": "ghost"})), Value::Null);
    t.err("bounds", json!({}), INVALID_PARAMS);

    // patch: applies within one frame; every refusal is false.
    assert_eq!(
        t.ok(
            "patch",
            json!({"id": "player", "parameter": "Mover.speed", "value": 4})
        ),
        json!(true)
    );
    assert_eq!(
        t.ok("entity", json!({"id": "player"}))["Mover.speed"],
        json!(4.0)
    );
    let x0 = t.h.player_x();
    t.ok("advance", json!({"steps": 60}));
    assert!(
        (t.h.player_x() - x0 - 4.0).abs() < 1e-3,
        "the next steps see the patch"
    );
    assert_eq!(
        t.ok(
            "patch",
            json!({"id": "player", "parameter": "Mover.speed", "value": 0.1})
        ),
        json!(true)
    );
    assert_eq!(
        t.ok("entity", json!({"id": "player"}))["Mover.speed"],
        json!(0.1),
        "f32 reads back by its shortest form"
    );
    for (id, parameter, value) in [
        ("ghost", "Mover.speed", json!(1)),
        ("player", "Mover.jump", json!(1)),
        ("player", "Jump.speed", json!(1)),
        ("player", "speed", json!(1)),
        ("player", "Mover.lane", json!(2)),
        ("player", "Health.max", json!(2)),
        ("player", "Mover.speed", json!("fast")),
        ("player", "Mover.speed", json!(25)),
        ("player", "Mover.speed", json!(-1)),
        ("player", "Mover.speed", json!([1, 2])),
        ("crate", "Mover.speed", json!(1)),
    ] {
        assert_eq!(
            t.ok(
                "patch",
                json!({"id": id, "parameter": parameter, "value": value})
            ),
            json!(false),
            "patch {id} {parameter} {value}"
        );
    }
    assert_eq!(
        t.ok("entity", json!({"id": "player"}))["Mover.speed"],
        json!(0.1),
        "refusals change nothing"
    );
    t.err(
        "patch",
        json!({"id": "player", "parameter": "Mover.speed"}),
        INVALID_PARAMS,
    );
    t.err("patch", json!({"id": "player", "value": 1}), INVALID_PARAMS);

    // run: one evaluation, stopping at the first refusal.
    let tick = t.tick();
    let result = t.ok("run", json!({"program": [
        {"op": "pause"}, {"op": "advance", "steps": 2}, {"op": "pick", "x": 50, "y": 0},
        {"op": "pick", "x": 1, "y": 0}, {"op": "read"}, {"op": "input", "event": {"kind": "key", "key": "KeyA", "action": "press"}},
        {"op": "entity", "id": "crate"}, {"op": "entity", "id": "ghost"}, {"op": "advance", "steps": 5}]}));
    assert_eq!(result["refused"], json!(7));
    assert_eq!(result["tick"], json!(tick + 2));
    let results = result["results"].as_array().unwrap();
    assert_eq!(results.len(), 7);
    assert_eq!(results[0], json!(true));
    assert_eq!(results[1], json!(tick + 2));
    assert_eq!(
        results[2],
        Value::Null,
        "a null pick is an answer, not a refusal"
    );
    assert_eq!(results[3]["entity"], json!("player"));
    assert_eq!(results[4]["tick"], json!(tick + 2));
    assert_eq!(results[4]["state"]["paused"], json!(true));
    assert_eq!(results[5], json!(true));
    assert_eq!(results[6], json!({"Health.hp": 30.0}));
    t.ok("resume", json!({}));
    for (program, refused) in [
        (json!([{"op": "advance", "steps": 0}]), 0),
        (
            json!([{"op": "resume"}, {"op": "advance", "steps": 3601}]),
            1,
        ),
        (json!([{"op": "teleport"}]), 0),
        (json!([{"op": "input", "event": {"kind": "joystick"}}]), 0),
        (json!([{"op": "pick", "x": 1}]), 0),
        (json!([{"op": "entity"}]), 0),
    ] {
        let r = t.ok("run", json!({"program": program}));
        assert_eq!(r["refused"], json!(refused), "{program}");
    }
    let long: Vec<Value> = (0..257).map(|_| json!({"op": "pause"})).collect();
    let r = t.ok("run", json!({"program": long}));
    assert_eq!(
        (r["results"].clone(), r["refused"].clone()),
        (json!([]), json!(0)),
        "more than 256 operations is refused whole"
    );
    assert!(!t.h.probe.paused(), "a refused program ran nothing");
    let at256: Vec<Value> = (0..256).map(|_| json!({"op": "resume"})).collect();
    assert_eq!(
        t.ok("run", json!({"program": at256}))["refused"],
        Value::Null
    );
    assert_eq!(
        t.ok("run", json!({"program": []})),
        json!({"results": [], "refused": null, "tick": t.h.probe.tick(&t.h.world)})
    );
    t.err("run", json!({"program": {"op": "pause"}}), INVALID_PARAMS);
    t.err("run", json!({}), INVALID_PARAMS);

    // capture: the frame at the current tick; sizes go together.
    let tick = t.tick();
    let frame = t.ok("capture", json!({}));
    assert_eq!(
        (
            frame["tick"].as_u64(),
            frame["width"].as_u64(),
            frame["height"].as_u64()
        ),
        (Some(tick), Some(64), Some(36))
    );
    let png = String::from_utf8(base64_decode(frame["png"].as_str().unwrap())).unwrap();
    assert!(png.starts_with(&format!("64x36 tick={tick} ")), "{png}");
    let frame = t.ok("capture", json!({"width": 32, "height": 18}));
    assert_eq!(
        (frame["width"].as_u64(), frame["height"].as_u64()),
        (Some(32), Some(18))
    );
    t.ok("capture", json!({"width": 0, "height": 0}));
    t.err("capture", json!({"width": 32}), INVALID_PARAMS);
    t.err(
        "capture",
        json!({"width": 5000, "height": 10}),
        INVALID_PARAMS,
    );
    t.err(
        "capture",
        json!({"width": -1, "height": 10}),
        INVALID_PARAMS,
    );

    // snapshot / restore: equal hashes mean equal worlds.
    let snap = t.ok("snapshot", json!({}));
    assert_eq!(snap["id"], json!("s1"));
    assert_eq!(snap["tick"], json!(tick));
    let hash = snap["hash"].as_str().unwrap().to_owned();
    assert_eq!(hash.len(), 16);
    assert_eq!(hash, format!("{:016x}", t.h.world.world_hash()));
    assert!(snap["bytes"].as_u64().unwrap() > 0);
    t.ok("advance", json!({"steps": 10}));
    t.ok(
        "patch",
        json!({"id": "player", "parameter": "Health.hp", "value": 1}),
    );
    assert_ne!(format!("{:016x}", t.h.world.world_hash()), hash);
    assert_eq!(t.ok("restore", json!({"id": "s1"})), json!(true));
    assert_eq!(t.tick(), tick);
    assert_eq!(format!("{:016x}", t.h.world.world_hash()), hash);
    assert_eq!(
        t.ok("entity", json!({"id": "player"}))["Health.hp"],
        json!(100.0)
    );
    assert_eq!(t.ok("restore", json!({"id": "s99"})), json!(false));
    t.err("restore", json!({}), INVALID_PARAMS);

    // dump: world, entity (null when unknown), ui.
    let world = t.ok("dump", json!({"what": "world"}));
    assert_eq!(world["kind"], json!("world"));
    assert_eq!(world["tick"], json!(tick));
    assert_eq!(world["hash"], json!(hash));
    assert_eq!(world["entities"], json!(5));
    let counted: u64 = world["archetypes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["count"].as_u64().unwrap())
        .sum();
    assert_eq!(counted, 5);
    assert!(
        world["archetypes"]
            .as_array()
            .unwrap()
            .iter()
            .any(
                |a| a["components"] == json!(["og.Transform", "Mover", "Velocity", "Health"])
                    && a["count"] == json!(1)
            )
    );
    assert!(
        world["archetypes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["components"] == json!(["og.Transform"]) && a["count"] == json!(3))
    );
    assert_eq!(
        world["systems"][0],
        json!({"name": "steer", "matched": 1, "reads": ["og.Input"], "writes": ["Velocity"], "ms": null, "totalMs": 0.0, "runs": 0})
    );
    assert_eq!(world["systems"][1]["matched"], json!(1));
    assert_eq!(
        world["systems"][2],
        json!({"name": "draw", "matched": 0, "reads": [], "writes": ["og.Rng", "Stats"], "ms": null, "totalMs": 0.0, "runs": 0})
    );
    for r in ["og.Input", "og.Camera", "Stats"] {
        assert!(
            world["resources"].as_array().unwrap().contains(&json!(r)),
            "{r} in {}",
            world["resources"]
        );
    }
    let player = t.ok("dump", json!({"what": "entity", "id": "player"}));
    assert_eq!(player["kind"], json!("entity"));
    assert_eq!(player["id"], json!("player"));
    assert_eq!(player["name"], json!("player"));
    assert_eq!(
        player["components"]["Mover"],
        json!({"speed": 0.1, "lane": 1})
    );
    assert_eq!(
        player["components"]["og.Transform"]["quaternion"],
        json!([0.0, 0.0, 0.0, 1.0])
    );
    let rock = t.ok("dump", json!({"what": "entity", "id": "#2.1"}));
    assert_eq!(
        (rock["id"].clone(), rock["name"].clone()),
        (json!("#2.1"), Value::Null)
    );
    assert_eq!(
        t.ok("dump", json!({"what": "entity", "id": "ghost"})),
        Value::Null
    );
    let ui = t.ok("dump", json!({"what": "ui"}));
    assert_eq!(ui["kind"], json!("ui"));
    assert_eq!(ui["elements"].as_array().unwrap().len(), 5);
    assert_eq!(
        ui["elements"][1],
        json!({"id": "Start", "parent": "Menu", "kind": "", "disabled": false, "clipped": false, "text": "Start game", "role": "", "name": "", "value": "", "states": [], "x": 10.0, "y": 10.0, "width": 100.0, "height": 40.0})
    );
    t.err("dump", json!({"what": "entity"}), INVALID_PARAMS);
    t.err("dump", json!({"what": "scene"}), INVALID_PARAMS);
    t.err("dump", json!({}), INVALID_PARAMS);

    // commands: a report; null for a malformed script.
    let tick = t.tick();
    assert_eq!(
        t.ok(
            "commands",
            json!({"script": "0:advance:10;0.5:seed:3;1:spawn_wave:4"})
        ),
        json!({"scheduled": 3, "untilTick": tick + 60, "warnings": [
            "1s: unknown action 'spawn_wave'; it is skipped unless registered before tick ".to_owned() + &(tick + 60).to_string() + " (`help` lists the actions)"]})
    );
    for bad in [
        "x:advance",
        "1:Advance",
        "2:pause;1:resume",
        "-1:pause",
        "1.0",
        "1:",
    ] {
        assert_eq!(
            t.ok("commands", json!({"script": bad})),
            Value::Null,
            "{bad}"
        );
    }
    t.err("commands", json!({}), INVALID_PARAMS);
    assert_eq!(t.h.probe.pending(), 3);

    // reload without a bundle loader: null, the world keeps running.
    assert_eq!(t.ok("reload", json!({})), Value::Null);

    // listen without audio: null; a malformed length is refused.
    assert_eq!(t.ok("listen", json!({})), Value::Null);
    assert_eq!(t.ok("listen", json!({"seconds": 0.5})), Value::Null);
    t.err("listen", json!({"seconds": 3}), INVALID_PARAMS);

    // memory from a host that gave no figures: null.
    assert_eq!(t.ok("memory", json!({})), Value::Null);

    // Unknown methods, including names the in-page probe has but the wire does not.
    for method in ["teleport", "version", "entities", "record", "Hello", ""] {
        t.err(method, json!({}), METHOD_NOT_FOUND);
    }

    // Malformed frames.
    let code = |v: &Value| v["error"]["code"].as_i64();
    assert_eq!(code(&t.frame("{not json")), Some(PARSE_ERROR));
    assert_eq!(
        code(&t.frame(r#"[{"jsonrpc":"2.0","id":1,"method":"tick"}]"#)),
        Some(INVALID_REQUEST)
    );
    assert_eq!(
        code(&t.frame(r#"{"jsonrpc":"2.0","method":"tick"}"#)),
        Some(INVALID_REQUEST)
    );
    assert_eq!(
        code(&t.frame(r#"{"jsonrpc":"2.0","id":null,"method":"tick"}"#)),
        Some(INVALID_REQUEST)
    );
    assert_eq!(
        code(&t.frame(r#"{"jsonrpc":"1.0","id":1,"method":"tick"}"#)),
        Some(INVALID_REQUEST)
    );
    assert_eq!(
        code(&t.frame(r#"{"jsonrpc":"2.0","id":1,"method":7}"#)),
        Some(INVALID_REQUEST)
    );
    assert_eq!(
        code(&t.frame(r#"{"jsonrpc":"2.0","id":1,"method":"tick","params":[1]}"#)),
        Some(INVALID_PARAMS)
    );
    assert_eq!(code(&t.frame("7")), Some(INVALID_REQUEST));
    let answer = t.frame(r#"{"jsonrpc":"2.0","id":"s","method":"tick"}"#);
    assert_eq!(answer["id"], json!("s"), "string ids are answered as given");
    assert!(answer["result"].is_u64(), "params may be omitted");
    t.checks += 10;

    // Without a renderer: capture and sweep refuse (null), hello says none.
    let mut bare = Transcript {
        h: build(7, false),
        log: Vec::new(),
        checks: 0,
    };
    assert_eq!(bare.ok("hello", json!({}))["tier"], json!("none"));
    assert_eq!(bare.ok("capture", json!({})), Value::Null);
    assert_eq!(
        bare.ok("pick", json!({"x": 1, "y": 1})),
        Value::Null,
        "no renderer, no pick"
    );
    assert_eq!(
        bare.ok(
            "sweep",
            json!({"id": "player", "parameter": "Mover.speed", "values": [1, 2], "steps": 1})
        ),
        Value::Null
    );
    t.checks += bare.checks;
    t.log.extend(bare.log);

    write_evidence(
        "conformance-transcript.json",
        &json!({
            "suite": "og-probe protocol 5 conformance (crates/og-probe/tests/conformance.rs)",
            "checks": t.checks,
            "exchanges": t.log,
        }),
    );
}

#[test]
fn a_sweep_whose_frames_land_later_answers_them_all_on_a_later_call() {
    let mut h = build(7, true);
    let params = json!({"id": "player", "parameter": "Mover.speed", "values": [1, 2, 3], "steps": 30, "width": 32, "height": 18});
    let want = h.result("sweep", params.clone());
    let before = h.world.world_hash();
    h.probe.set_capture(Box::new(ReadbackCapture::default()));
    let mut calls = 0;
    let got = loop {
        calls += 1;
        let answer = h.result("sweep", params.clone());
        assert_eq!(
            h.world.world_hash(),
            before,
            "the world is restored after every call"
        );
        if !answer.is_null() {
            break answer;
        }
        assert!(calls < 10, "the sweep never answered its frames");
    };
    assert_eq!(got, want, "the frames of a host that reads back at once");
    assert_eq!(
        calls, 4,
        "each call takes the frame in flight and asks for the next"
    );
}

#[test]
fn a_sweep_yields_one_frame_per_value_and_restores_the_world() {
    let mut h = build(7, true);
    h.result("advance", json!({"steps": 30}));
    h.result(
        "patch",
        json!({"id": "player", "parameter": "Mover.speed", "value": 3}),
    );
    let before = h.world.world_hash();
    let x0 = h.player_x();
    let values: Vec<f32> = (0..8).map(|i| i as f32 * 1.5).collect();
    let sweep = h.result("sweep", json!({"id": "player", "parameter": "Mover.speed", "values": values, "steps": 30, "width": 32, "height": 18}));
    let frames = sweep["frames"].as_array().unwrap();
    assert_eq!(frames.len(), 8);
    for (frame, speed) in frames.iter().zip(&values) {
        assert_eq!(frame["value"], json!(speed));
        assert_eq!(
            frame["tick"],
            json!(60),
            "each value restarts from the call's tick"
        );
        let png = String::from_utf8(base64_decode(frame["png"].as_str().unwrap())).unwrap();
        let x: f32 = png.rsplit("x=").next().unwrap().parse().unwrap();
        assert!(
            (x - (x0 + speed * 0.5)).abs() < 1e-3,
            "{png}: speed {speed} for half a second from {x0}"
        );
        assert!(png.starts_with("32x18 tick=60 "));
    }
    assert_eq!(
        h.world.world_hash(),
        before,
        "the world is restored after the sweep"
    );
    assert_eq!(h.probe.tick(&h.world), 30);
    assert_eq!(
        h.result("entity", json!({"id": "player"}))["Mover.speed"],
        json!(3.0)
    );

    // Refusals: null, and the world is still as found.
    for params in [
        json!({"id": "ghost", "parameter": "Mover.speed", "values": [1], "steps": 1}),
        json!({"id": "player", "parameter": "Mover.jump", "values": [1], "steps": 1}),
        json!({"id": "player", "parameter": "Mover.lane", "values": [1], "steps": 1}),
        json!({"id": "crate", "parameter": "Mover.speed", "values": [1], "steps": 1}),
        json!({"id": "player", "parameter": "Mover.speed", "values": [1, "fast"], "steps": 10}),
        json!({"id": "player", "parameter": "Mover.speed", "values": [1, 99], "steps": 10}),
    ] {
        assert_eq!(h.result("sweep", params.clone()), Value::Null, "{params}");
        assert_eq!(h.world.world_hash(), before, "{params}");
    }
    for params in [
        json!({"id": "player", "parameter": "Mover.speed", "values": [], "steps": 1}),
        json!({"id": "player", "parameter": "Mover.speed", "values": vec![1; 65], "steps": 1}),
        json!({"id": "player", "parameter": "Mover.speed", "values": [1], "steps": 3601}),
        json!({"id": "player", "parameter": "Mover.speed", "values": [1]}),
        json!({"parameter": "Mover.speed", "values": [1], "steps": 1}),
        json!({"id": "player", "parameter": "Mover.speed", "values": [1], "steps": 1, "width": 10}),
    ] {
        assert_eq!(
            h.call("sweep", params.clone())["error"]["code"],
            json!(INVALID_PARAMS),
            "{params}"
        );
    }
    // A sweep never runs scheduled command steps.
    h.probe.schedule(&h.world, "0.1:quit").unwrap();
    h.result(
        "sweep",
        json!({"id": "player", "parameter": "Mover.speed", "values": [1], "steps": 30}),
    );
    assert!(!h.probe.quit_requested());
    assert_eq!(h.probe.pending(), 1);
}

#[test]
fn a_failing_system_is_a_fault_and_the_probe_still_answers() {
    let mut h = build(7, false);
    h.world
        .add_system(SystemDesc::new("explode"), |ctx| {
            if ctx.time().tick == 3 {
                Err(og_core::Error::new(
                    og_core::ErrorKind::Invalid,
                    "the reactor is critical",
                ))
            } else {
                Ok(())
            }
        })
        .unwrap();
    let r = h.call("advance", json!({"steps": 10}));
    assert_eq!(r["error"]["code"], json!(INTERNAL_ERROR));
    let message = r["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("system 'explode'") && message.contains("the reactor is critical"),
        "{message}"
    );
    assert_eq!(
        h.result("tick", json!({})),
        json!(3),
        "the world stops at the failing tick"
    );
    let r = h.call("run", json!({"program": [{"op": "advance", "steps": 1}]}));
    assert_eq!(
        r["error"]["code"],
        json!(INTERNAL_ERROR),
        "a program's fault errs the whole call"
    );
}

struct SameBundle;
impl Reloader for SameBundle {
    fn rebuild(&mut self, _: &og_ecs::World) -> Result<Rebuilt, String> {
        Ok(Rebuilt {
            world: bundle(0, true, &mut |_| {}),
            ms: 12.5,
        })
    }
}

struct WithoutHealth;
impl Reloader for WithoutHealth {
    fn rebuild(&mut self, _: &og_ecs::World) -> Result<Rebuilt, String> {
        Ok(Rebuilt {
            world: bundle(0, false, &mut |_| {}),
            ms: 3.0,
        })
    }
}

/// A bundle whose Mover gained a field: a layout change.
struct MoverChanged;
impl Reloader for MoverChanged {
    fn rebuild(&mut self, _: &og_ecs::World) -> Result<Rebuilt, String> {
        #[derive(Clone, Copy, og_core::Reflect)]
        #[reflect(name = "Mover")]
        struct MoverV2 {
            speed: f32,
            lane: u32,
            accel: f32,
        }
        let mut world = og_ecs::World::new(&og_ecs::WorldDesc::default()).unwrap();
        world.register::<MoverV2>().unwrap();
        Ok(Rebuilt { world, ms: 1.0 })
    }
}

struct Broken;
impl Reloader for Broken {
    fn rebuild(&mut self, _: &og_ecs::World) -> Result<Rebuilt, String> {
        Err("player.ts:3: Unexpected token".into())
    }
}

#[test]
fn reload_swaps_the_bundle_and_keeps_the_world() {
    let mut h = build(7, false);
    h.result("advance", json!({"steps": 20}));
    let hash = h.world.world_hash();

    h.probe.set_reloader(Box::new(SameBundle));
    let report = h.result("reload", json!({}));
    assert_eq!(
        report,
        json!({"tick": 20, "ms": 12.5, "migrated": [], "dropped": [], "warnings": []})
    );
    assert_eq!(
        h.world.world_hash(),
        hash,
        "same layouts: the same world in the new bundle"
    );
    h.result("advance", json!({"steps": 10}));
    assert_eq!(
        h.result("tick", json!({})),
        json!(30),
        "the new bundle's systems run"
    );

    h.probe.set_reloader(Box::new(Broken));
    let hash = h.world.world_hash();
    assert_eq!(h.result("reload", json!({})), Value::Null);
    assert_eq!(
        h.world.world_hash(),
        hash,
        "a bundle that fails to load leaves the old one running"
    );
    assert!(
        h.probe
            .take_log()
            .iter()
            .any(|l| l.warning && l.text.contains("Unexpected token"))
    );

    h.probe.set_reloader(Box::new(WithoutHealth));
    let report = h.result("reload", json!({}));
    assert_eq!(report["dropped"], json!(["Health"]));
    assert_eq!(
        h.result("entity", json!({"id": "player"})),
        json!({"Mover.speed": 2.0}),
        "Health was dropped with its data"
    );
    assert_eq!(h.result("entity", json!({"id": "crate"})), json!({}));
    assert_eq!(h.result("advance", json!({"steps": 5})), json!(35));

    // A layout change migrates field by field: Mover gained `accel` (its
    // default), kept `speed`; the types this bundle no longer declares go.
    h.probe.set_reloader(Box::new(MoverChanged));
    let report = h.result("reload", json!({}));
    assert_eq!(report["tick"], json!(35));
    assert!(
        report["migrated"]
            .as_array()
            .unwrap()
            .contains(&json!("Mover")),
        "{report}"
    );
    assert!(
        report["migrated"]
            .as_array()
            .unwrap()
            .contains(&json!("Mover.accel")),
        "{report}"
    );
    assert!(
        report["dropped"]
            .as_array()
            .unwrap()
            .contains(&json!("Velocity")),
        "{report}"
    );
    let mover = h.world.id_by_name("Mover").unwrap();
    let player = h.probe.names().resolve(&h.world, "player").unwrap();
    let bytes = h.world.get_bytes(player, mover).unwrap();
    assert_eq!(
        f32::from_le_bytes(bytes[0..4].try_into().unwrap()),
        2.0,
        "speed kept"
    );
    assert_eq!(
        f32::from_le_bytes(bytes[8..12].try_into().unwrap()),
        0.0,
        "accel added at its default"
    );
}

fn write_evidence(name: &str, value: &Value) {
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&path, serde_json::to_string_pretty(value).unwrap() + "\n").unwrap();
    println!("wrote {}", path.display());
}

/// A renderer with state that follows the world's time (as og-render's
/// adapted exposure): every frame drawn moves it on, and the probe's
/// snapshots keep it through `save_state` and bring it back on restore.
struct StatefulCapture {
    adapted: u64,
    saves: Vec<u64>,
}

impl og_probe::Capture for StatefulCapture {
    fn tier(&self) -> &str {
        "vulkan"
    }
    fn target_size(&self) -> (u32, u32) {
        (8, 8)
    }
    fn capture(&mut self, world: &og_ecs::World, w: u32, h: u32) -> Option<og_probe::Frame> {
        self.adapted += 1;
        Some(og_probe::Frame {
            width: w.max(8),
            height: h.max(8),
            png: format!("tick={} adapted={}", world.time().tick, self.adapted).into_bytes(),
        })
    }
    fn save_state(&mut self) -> u64 {
        self.saves.push(self.adapted);
        self.saves.len() as u64
    }
    fn restore_state(&mut self, state: u64) {
        self.adapted = match state {
            0 => 0,
            n => self.saves[n as usize - 1],
        };
    }
}

#[test]
fn snapshots_and_sweeps_carry_the_renderers_state() {
    let mut h = build(7, false);
    h.probe.set_capture(Box::new(StatefulCapture {
        adapted: 0,
        saves: Vec::new(),
    }));
    let png = |v: &Value| String::from_utf8(base64_decode(v["png"].as_str().unwrap())).unwrap();
    h.result("advance", json!({"steps": 10}));
    let first = png(&h.result("capture", json!({})));
    assert_eq!(first, "tick=10 adapted=1");
    let snap = h.result("snapshot", json!({}));
    let id = snap["id"].as_str().unwrap().to_string();
    h.result("advance", json!({"steps": 5}));
    assert_eq!(png(&h.result("capture", json!({}))), "tick=15 adapted=2");
    assert_eq!(h.result("restore", json!({ "id": id })), json!(true));
    h.result("advance", json!({"steps": 5}));
    assert_eq!(
        png(&h.result("capture", json!({}))),
        "tick=15 adapted=2",
        "after the restore the renderer's state is the snapshot's"
    );
    // Every value of a sweep starts from the renderer's state at the call.
    let sweep = h.result(
        "sweep",
        json!({"id": "player", "parameter": "Mover.speed", "values": [1, 2, 3], "steps": 2}),
    );
    for f in sweep["frames"].as_array().unwrap() {
        assert_eq!(png(f), "tick=17 adapted=3");
    }
    assert_eq!(png(&h.result("capture", json!({}))), "tick=15 adapted=3");
}
