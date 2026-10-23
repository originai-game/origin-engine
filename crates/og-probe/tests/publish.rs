//! A `publish` build keeps only the player-facing members
//! (`cargo test -p og-probe --features publish --test publish`).
#![cfg(feature = "publish")]

mod support;

use og_probe::{BUILD, MEMBERS, PUBLISH_MEMBERS};
use serde_json::json;

#[test]
fn only_player_facing_members_answer() {
    let mut h = support::build(7, true);
    let hello = h.result("hello", json!({}));
    assert_eq!(hello["build"], json!("publish"));
    assert_eq!(BUILD, "publish");
    assert_eq!(
        hello["members"],
        json!([
            "hello", "tick", "state", "pick", "entity", "input", "capture"
        ])
    );
    for member in MEMBERS {
        let r = h.call(member, json!({"id": "player", "x": 1, "y": 1, "event": {"kind": "key", "key": "KeyA", "action": "press"}}));
        if PUBLISH_MEMBERS.contains(member) {
            assert!(r.get("result").is_some(), "{member}: {r}");
        } else {
            assert_eq!(
                r["error"]["code"],
                json!(-32601),
                "{member} is dropped: {r}"
            );
        }
    }
}

/// The command channel of a publish build runs what changes the world (what
/// og.ai's commands and a timeline's event keys reach) and refuses what only
/// observes it for a tool, whose code is not built in; a dump a module
/// registers is not kept.
#[test]
fn the_command_channel_runs_world_actions_and_refuses_the_tools() {
    let mut h = support::build(7, true);
    let ran = std::rc::Rc::new(std::cell::Cell::new(0));
    let seen = ran.clone();
    h.probe
        .register_command("ping", og_probe::CommandDesc::default(), move |_, _| {
            seen.set(seen.get() + 1);
            Ok(())
        })
        .unwrap();
    h.probe
        .register_dump("dump_thing", og_probe::CommandDesc::default(), 0, |_, _| {
            Ok(json!({"kind": "thing"}))
        })
        .unwrap();
    let script = "0:teleport:player:3,4,5;0:ping;0:dump_world;0:describe;0:expect:state:tick>=0;0:get:player;0:screenshot;0:dump_thing";
    h.probe.schedule(&h.world, script).unwrap();
    h.probe.step(&mut h.world).unwrap();
    assert_eq!(ran.get(), 1, "a registered command runs");
    assert!(
        h.player_x() > 2.9,
        "teleport moved the player: {}",
        h.player_x()
    );
    assert!(
        h.probe.take_outputs().is_empty(),
        "no dump, describe, get or frame is written"
    );
    let warnings: Vec<String> = h
        .probe
        .take_log()
        .into_iter()
        .filter(|l| l.warning)
        .map(|l| l.text)
        .collect();
    for action in ["dump_world", "describe", "expect", "get", "screenshot"] {
        assert!(
            warnings
                .iter()
                .any(|w| w.contains(&format!("`{action}` is not built into a publish build"))),
            "{action}: {warnings:?}"
        );
    }
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("unknown action 'dump_thing'")),
        "{warnings:?}"
    );
}
