//! A string field through the probe: `set:` takes its text (bare, or a JSON
//! string literal), refuses one over the capacity with the fix named and
//! changes nothing; `get:` and `dump_entity` read it back as a JSON string
//! and `dump_world`'s hash moves with it.
#![cfg(not(feature = "publish"))]

use crate::support;

use og_core::{Reflect, Str};
use og_probe::Payload;
use serde_json::{Value, json};
use support::build;

/// A name tag on an entity: a 16-byte name and a rank.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Badge {
    name: Str<16>,
    rank: u32,
}

/// The player's settings: an 8-byte name.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
struct Profile {
    name: Str<8>,
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
fn set_and_get_move_a_string_field_as_text() {
    let mut h = build(1, false);
    h.world.register::<Badge>().unwrap();
    h.world.register::<Profile>().unwrap();
    h.world.insert_resource(Profile::default()).unwrap();
    let player = h.world.find("player").unwrap();
    h.world
        .insert(
            player,
            Badge {
                name: Str::new("Ada").unwrap(),
                rank: 2,
            },
        )
        .unwrap();

    h.probe
        .schedule(
            &h.world,
            "0:dump_world:before.json;0:set:player.Badge.name=Ada Lovelace;0:set:Profile.name=小灯;\
             0:get:player.Badge.name:name.json;0:get:Profile:profile.json;\
             0:dump_entity:player:player.json;0:dump_world:world.json",
        )
        .unwrap();
    h.probe.pump(&mut h.world);
    let log = h.probe.take_log();
    assert!(log.iter().all(|l| !l.warning), "{log:?}");
    assert_eq!(
        h.world.get::<Badge>(player).unwrap().name.as_str(),
        "Ada Lovelace"
    );
    assert_eq!(
        h.world.get::<Badge>(player).unwrap().rank,
        2,
        "the other field is kept"
    );
    assert_eq!(h.world.resource::<Profile>().unwrap().name.as_str(), "小灯");
    let notes: Vec<&str> = log.iter().map(|l| l.text.as_str()).collect();
    assert!(
        notes.contains(&"get player.Badge.name = \"Ada Lovelace\""),
        "{notes:?}"
    );

    let out = values(&mut h);
    let file = |name: &str| &out.iter().find(|(n, _)| n == name).unwrap().1;
    assert_eq!(file("name.json")["value"], json!("Ada Lovelace"));
    assert_eq!(file("profile.json")["value"], json!({"name": "小灯"}));
    assert_eq!(
        file("player.json")["components"]["Badge"],
        json!({"name": "Ada Lovelace", "rank": 2})
    );
    assert_ne!(
        file("world.json")["hash"],
        file("before.json")["hash"],
        "the text is world state: dump_world's hash moves with it"
    );

    // A JSON literal sets escapes and the empty string (the command text
    // escapes its own backslashes: `\\\"` is the JSON `\"`).
    h.probe
        .schedule(
            &h.world,
            r#"0:set:player.Badge.name="say \\\"hi\\\"";0:set:Profile.name="""#,
        )
        .unwrap();
    h.probe.pump(&mut h.world);
    let log = h.probe.take_log();
    assert!(log.iter().all(|l| !l.warning), "{log:?}");
    assert_eq!(
        h.world.get::<Badge>(player).unwrap().name.as_str(),
        "say \"hi\""
    );
    assert_eq!(h.world.resource::<Profile>().unwrap().name.as_str(), "");
    let bytes = h
        .world
        .get_bytes(player, h.world.id_by_name("Badge").unwrap())
        .unwrap();
    assert!(
        bytes[8..16].iter().all(|b| *b == 0),
        "zero-padded after the text"
    );
}

#[test]
fn set_refuses_a_string_over_its_capacity_and_names_the_fix() {
    let mut h = build(1, false);
    h.world.register::<Profile>().unwrap();
    h.world
        .insert_resource(Profile {
            name: Str::new("Mei").unwrap(),
        })
        .unwrap();
    // Nine bytes for eight; three characters of three bytes for eight.
    h.probe
        .schedule(
            &h.world,
            "0:set:Profile.name=Alexandra;0:set:Profile.name=小灯笼",
        )
        .unwrap();
    h.probe.pump(&mut h.world);
    let log = h.probe.take_log();
    let warnings: Vec<&str> = log
        .iter()
        .filter(|l| l.warning)
        .map(|l| l.text.as_str())
        .collect();
    assert_eq!(warnings.len(), 2, "{log:?}");
    assert!(
        warnings[0].contains("'Alexandra' is 9 bytes of UTF-8 and the field holds 8")
            && warnings[0].contains("declare the field str(9) or larger"),
        "{warnings:?}"
    );
    assert!(warnings[1].contains("is 9 bytes of UTF-8"), "{warnings:?}");
    assert_eq!(
        h.world.resource::<Profile>().unwrap().name.as_str(),
        "Mei",
        "never truncated: the value is unchanged"
    );
}
