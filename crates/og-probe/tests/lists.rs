//! A list field through the probe: `set:` takes a whole list (a JSON array
//! or numbers by commas, nothing for `[]`), refuses one over the capacity
//! and changes nothing; `get:` and the dumps read it back as a JSON array,
//! `[]` and `[0]` apart; `patch` refuses it, as a list is never tunable.
#![cfg(not(feature = "publish"))]

mod support;

use og_core::{List, Reflect};
use og_probe::Payload;
use serde_json::{Value, json};
use support::build;

/// A chain of passes on an entity.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Chain {
    passes: List<u64, 4>,
    weight: f32,
}

/// The frame's weights.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
struct Grades {
    weights: List<f32, 3>,
}

fn values(h: &mut support::Harness) -> Vec<(String, Value)> {
    h.probe
        .take_outputs()
        .into_iter()
        .filter_map(|o| match &o.payload {
            Payload::Json(v) => Some((o.file_name(), v.clone())),
            _ => None,
        })
        .collect()
}

#[test]
fn set_get_and_the_dumps_move_a_list_field_as_an_array() {
    let mut h = build(1, false);
    h.world.register::<Chain>().unwrap();
    h.world.register::<Grades>().unwrap();
    h.world.insert_resource(Grades::default()).unwrap();
    let player = h.world.find("player").unwrap();
    h.world
        .insert(
            player,
            Chain {
                passes: List::from_slice(&[9]).unwrap(),
                weight: 0.5,
            },
        )
        .unwrap();

    h.probe
        .schedule(
            &h.world,
            "0:dump_world:before.json;0:set:player.Chain.passes=[3,5];0:set:Grades.weights=0.25,1.5;\
             0:get:player.Chain.passes:passes.json;0:get:Grades:grades.json;\
             0:dump_entity:player:player.json;0:dump_world:world.json",
        )
        .unwrap();
    h.probe.pump(&mut h.world);
    let log = h.probe.take_log();
    assert!(log.iter().all(|l| !l.warning), "{log:?}");
    let chain = h.world.get::<Chain>(player).unwrap();
    assert_eq!(chain.passes.as_slice(), [3, 5]);
    assert_eq!(chain.weight, 0.5, "the other field is kept");
    assert_eq!(
        h.world.resource::<Grades>().unwrap().weights.as_slice(),
        [0.25, 1.5]
    );
    let notes: Vec<&str> = log.iter().map(|l| l.text.as_str()).collect();
    assert!(
        notes.contains(&"get player.Chain.passes = [3,5]"),
        "{notes:?}"
    );
    let out = values(&mut h);
    let file = |name: &str| &out.iter().find(|(n, _)| n == name).unwrap().1;
    assert_eq!(file("passes.json")["value"], json!([3, 5]));
    assert_eq!(
        file("grades.json")["value"],
        json!({"weights": [0.25, 1.5]})
    );
    assert_eq!(
        file("player.json")["components"]["Chain"],
        json!({"passes": [3, 5], "weight": 0.5})
    );
    assert_ne!(file("world.json")["hash"], file("before.json")["hash"]);

    // `[0]`, `0` and `[]`, nothing: one zero, then the empty list, which
    // read apart and hash apart.
    let hash = |h: &mut support::Harness, cmd: &str| {
        h.probe.schedule(&h.world, cmd).unwrap();
        h.probe.pump(&mut h.world);
        let log = h.probe.take_log();
        assert!(log.iter().all(|l| !l.warning), "{log:?}");
        let out = values(h);
        (out[0].1["value"].clone(), out[1].1["hash"].clone())
    };
    let zero = hash(
        &mut h,
        "0:set:player.Chain.passes=[0];0:get:player.Chain.passes:a.json;0:dump_world:aw.json",
    );
    let zero_bare = hash(
        &mut h,
        "0:set:player.Chain.passes=0;0:get:player.Chain.passes:b.json;0:dump_world:bw.json",
    );
    let empty = hash(
        &mut h,
        "0:set:player.Chain.passes=[];0:get:player.Chain.passes:c.json;0:dump_world:cw.json",
    );
    let empty_bare = hash(
        &mut h,
        "0:set:player.Chain.passes=;0:get:player.Chain.passes:d.json;0:dump_world:dw.json",
    );
    assert_eq!(zero.0, json!([0]));
    assert_eq!(zero, zero_bare);
    assert_eq!(empty.0, json!([]));
    assert_eq!(empty, empty_bare);
    assert_ne!(zero.1, empty.1, "[0] and [] are different world states");
}

#[test]
fn set_refuses_a_list_over_its_capacity_and_patch_refuses_any_list() {
    let mut h = build(1, false);
    h.world.register::<Chain>().unwrap();
    let player = h.world.find("player").unwrap();
    let start = Chain {
        passes: List::from_slice(&[1, 2]).unwrap(),
        weight: 1.0,
    };
    h.world.insert(player, start).unwrap();
    h.probe
        .schedule(
            &h.world,
            "0:set:player.Chain.passes=[1,2,3,4,5];0:set:player.Chain.passes=[1.5];\
             0:set:player.Chain.passes=-1;0:set:player.Chain.passes=true",
        )
        .unwrap();
    h.probe.pump(&mut h.world);
    let log = h.probe.take_log();
    let warnings: Vec<&str> = log
        .iter()
        .filter(|l| l.warning)
        .map(|l| l.text.as_str())
        .collect();
    assert_eq!(warnings.len(), 4, "{log:?}");
    assert!(
        warnings[0].contains("5 elements and the field holds 4"),
        "{warnings:?}"
    );
    assert!(warnings[1].contains("element 0"), "{warnings:?}");
    assert_eq!(
        h.world.get::<Chain>(player).unwrap(),
        start,
        "never truncated: the value is unchanged"
    );

    // Never tunable: patch refuses it, bounds and entity leave it out.
    assert!(
        !h.probe
            .patch(&mut h.world, "player", "Chain.passes", &json!([7]))
    );
    assert_eq!(h.world.get::<Chain>(player).unwrap(), start);
    let bounds = h.probe.bounds(&h.world, "player").unwrap();
    assert!(!bounds.keys().any(|k| k.contains("passes")), "{bounds:?}");
}
