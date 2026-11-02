//! A `publish` build has no inspector (G3-7): its `inspect` and
//! `dump_inspector` are unknown actions, so a game shipped to players
//! cannot open the panel or read the world through it
//! (`cargo test -p og-runtime --features publish --test tests publish::`).
#![cfg(feature = "publish")]

use og_runtime::{
    Dispatch, Fetch, Hooks, og_commands_schedule, og_engine_create, og_engine_frame,
    og_engine_take_log, og_game_game, og_host_begin_bundle, set_hooks,
};
use serde_json::{Value, json};
use std::rc::Rc;

struct NoAssets;

impl Hooks for NoAssets {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets in this test"))
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

#[test]
fn a_publish_build_refuses_inspect() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let e = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    value(og_commands_schedule(
        e,
        "0:inspect;0:inspect:key:F2;0:dump_inspector".into(),
    ));
    for _ in 0..3 {
        value(og_engine_frame(e, 1000.0 / 60.0));
    }
    let log = og_engine_take_log(e);
    for action in ["inspect", "dump_inspector"] {
        assert!(
            log.contains(&format!("unknown action '{action}'")),
            "{action} is refused: {log}"
        );
    }
}
