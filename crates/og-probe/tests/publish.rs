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
