//! Shared worlds (og.shared) end to end through the host ABI: two engines
//! join one og-shared shard, carried by this test the way a host carries
//! them (og_shared_take's ops to the server, its messages back through
//! og_shared_deliver), and a third engine replays one session from its
//! og.platform.journal alone.
//!
//! Ways this could fail: a remote avatar is never spawned, is drawn at
//! another place than its visitor's (a wrong cell size, the playout clock
//! stuck behind), or stays after its visitor left; a refused move is not
//! put back where the server holds it, or is put back without what the
//! avatar moved since; the journal misses an event, keeps it at another
//! tick, or applies it as it arrives instead of at a tick's start, so the
//! replay's world hash differs from the session's.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
    og_net_hash, og_platform_journal, og_shared_deliver, og_shared_join, og_shared_remote,
    og_shared_status, og_shared_take, og_world_alive, og_world_get, og_world_insert,
    og_world_spawn, set_hooks,
};
use og_shared::config::Config;
use og_shared::protocol::ClientMsg;
use og_shared::shard::Claims;
use og_shared::shard::Shard;
use serde_json::{Value, json};
use std::collections::BTreeMap;
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

fn answer(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v
}

fn engine() -> u32 {
    let game = og_game_game(Some("{}".into()));
    answer(og_engine_create(game, "{}".into()))["value"]
        .as_u64()
        .unwrap() as u32
}

fn place(engine: u32, id: f64, x: f64) {
    answer(og_runtime::host_impl::answer(og_world_insert(
        engine,
        id,
        "og.Transform".into(),
        Some(json!({ "position": { "x": x, "y": 0, "z": 0 } }).to_string()),
    )));
}

/// A visitor: its avatar entity, and the session joined.
fn visitor(engine: u32, x: f64) -> f64 {
    let id = answer(og_runtime::host_impl::answer(og_world_spawn(
        engine,
        Some(json!({ "position": { "x": x, "y": 0, "z": 0 } }).to_string()),
    )))["value"]
        .as_f64()
        .unwrap();
    answer(og_runtime::host_impl::answer(og_world_insert(
        engine,
        id,
        "og.SharedAvatar".into(),
        Some("{}".into()),
    )));
    assert!(og_shared_deliver(
        engine,
        json!({ "type": "host", "host": "test" }).to_string()
    ));
    answer(og_runtime::host_impl::answer(og_shared_join(
        engine,
        Some(json!({ "world": "main" }).to_string()),
    )));
    id
}

fn x_of(engine: u32, id: f64) -> f64 {
    answer(og_runtime::host_impl::answer(og_world_get(
        engine,
        id,
        "og.Transform".into(),
    )))["value"]["position"]["x"]
        .as_f64()
        .unwrap()
}

/// The og.RemoteAvatar entities an engine holds and has spawned (entity
/// bits are `generation << 32 | index`; this world's are small).
fn remotes(engine: u32) -> Vec<f64> {
    (0..4u64)
        .flat_map(|g| (0..4096u64).map(move |i| (g << 32 | i) as f64))
        .filter(|e| og_shared_remote(engine, *e).is_some() && og_world_alive(engine, *e))
        .collect()
}

/// The host of every engine: their ops to the shard, its messages back.
struct Host {
    shard: Shard,
    /// Engine to (player, session once it said hello).
    links: BTreeMap<u32, (String, Option<u64>)>,
}

impl Host {
    fn pump(&mut self) {
        for (&engine, (player, sid)) in self.links.iter_mut() {
            let ops: Vec<Value> = serde_json::from_str(&og_shared_take(engine)).unwrap();
            for op in ops {
                match op["op"].as_str().unwrap() {
                    "connect" => {
                        let open = json!({ "type": "open", "transport": "ws", "token": "secret" });
                        assert!(og_shared_deliver(engine, open.to_string()));
                    }
                    "send" => {
                        let b = og_core::base64::decode(op["bytes"].as_str().unwrap()).unwrap();
                        match sid {
                            None => {
                                let Ok(ClientMsg::Hello(h)) = ClientMsg::decode(&b, |_| None)
                                else {
                                    panic!("the first message is hello");
                                };
                                assert_eq!(h.token, "secret");
                                let claims = Claims {
                                    player: player.clone(),
                                    name: player.to_uppercase(),
                                    ..Claims::default()
                                };
                                *sid = Some(self.shard.hello(&h, claims).unwrap());
                            }
                            Some(s) => self.shard.receive(*s, &b).unwrap(),
                        }
                    }
                    "close" => {}
                    other => panic!("an unknown op {other}"),
                }
            }
        }
    }

    fn deliver(&mut self) {
        for (&engine, (_, sid)) in &self.links {
            let Some(s) = sid else { continue };
            for m in self.shard.take_out(*s) {
                let ev = json!({ "type": "message", "bytes": og_core::base64::encode(&m) });
                assert!(og_shared_deliver(engine, ev.to_string()));
            }
        }
    }
}

const FRAME_MS: f64 = 1000.0 / 60.0;

#[test]
fn two_visitors_see_each_other_and_a_journal_replays_the_session() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let (a, b) = (engine(), engine());
    let me_a = visitor(a, 0.0);
    let _me_b = visitor(b, 10.0);
    let mut host = Host {
        shard: Shard::new(Config::default(), 7),
        links: BTreeMap::from([(a, ("a".into(), None)), (b, ("b".into(), None))]),
    };
    let mut ax = 0.0;
    // The shard ticks at 20 Hz: every third 60 Hz frame. `a` walks 2 m/s
    // for two seconds, then stands.
    let run = |host: &mut Host, frames: u32, walk: bool, ax: &mut f64| {
        for f in 0..frames {
            if walk {
                *ax += 2.0 / 60.0;
                place(a, me_a, *ax);
            }
            answer(og_engine_frame(a, FRAME_MS));
            answer(og_engine_frame(b, FRAME_MS));
            host.pump();
            if f % 3 == 2 {
                host.shard.step();
                host.deliver();
            }
        }
    };
    run(&mut host, 120, true, &mut ax);
    // `b` holds `a`, drawn a little behind (the 100 ms delay and the trip).
    let seen = remotes(b);
    assert_eq!(seen.len(), 1, "b holds one remote avatar");
    let who = og_shared_remote(b, seen[0]).expect("an og.RemoteAvatar is known");
    assert_eq!((who.player.as_str(), who.name.as_str()), ("a", "A"));
    let lag = ax - x_of(b, seen[0]);
    assert!(
        (0.1..0.8).contains(&lag),
        "a is drawn {lag} m behind where it is"
    );
    assert_eq!(remotes(a).len(), 1, "a holds b");
    assert!((x_of(a, remotes(a)[0]) - 10.0).abs() < 0.01);
    // Standing, `a` is drawn exactly where it stands.
    run(&mut host, 30, false, &mut ax);
    assert!(
        (x_of(b, seen[0]) - ax).abs() < 0.005,
        "{} vs {ax}",
        x_of(b, seen[0])
    );

    // A jump of 50 m with no teleport is refused: the server puts `a` back
    // where it holds it, and keeps what `a` moved since.
    let before = ax;
    place(a, me_a, before + 50.0);
    run(&mut host, 15, false, &mut ax);
    let st = og_shared_status(a);
    assert!(st.corrections >= 1, "the server corrected a");
    assert!(
        (x_of(a, me_a) - before).abs() < 0.05,
        "a is back at {} (from {before})",
        x_of(a, me_a)
    );
    assert!(
        (x_of(b, seen[0]) - before).abs() < 0.05,
        "b never drew the jump"
    );
    assert_eq!(og_shared_status(b).state, "joined");

    // The replay: a fresh engine given b's journal back at its ticks, with
    // the same calls as b's game, no server, and the same world hash.
    let journal = og_platform_journal(b);
    let entries = journal.as_array().unwrap().clone();
    assert!(entries.iter().all(|e| e["kind"] == "shared"));
    assert!(
        entries.iter().all(|e| e["value"].get("token").is_none()),
        "the journal keeps no token"
    );
    let c = engine();
    let ticks = 120 + 30 + 15;
    let mut by_tick: BTreeMap<u64, Vec<Value>> = BTreeMap::new();
    for e in entries {
        by_tick
            .entry(e["tick"].as_u64().unwrap())
            .or_default()
            .push(e["value"].clone());
    }
    // The host event and the join are handed before the first frame.
    let _me_c = {
        let id = answer(og_runtime::host_impl::answer(og_world_spawn(
            c,
            Some(json!({ "position": { "x": 10.0, "y": 0, "z": 0 } }).to_string()),
        )))["value"]
            .as_f64()
            .unwrap();
        answer(og_runtime::host_impl::answer(og_world_insert(
            c,
            id,
            "og.SharedAvatar".into(),
            Some("{}".into()),
        )));
        for v in by_tick.remove(&0).unwrap_or_default() {
            assert!(og_shared_deliver(c, v.to_string()));
        }
        answer(og_runtime::host_impl::answer(og_shared_join(
            c,
            Some(json!({ "world": "main" }).to_string()),
        )));
        id
    };
    let mut tick = 0u64;
    let handed = |c: u32, tick: u64, by_tick: &mut BTreeMap<u64, Vec<Value>>| {
        for v in by_tick.remove(&tick).unwrap_or_default() {
            assert!(og_shared_deliver(c, v.to_string()));
        }
    };
    handed(c, tick, &mut by_tick);
    for _ in 0..ticks {
        answer(og_engine_frame(c, FRAME_MS));
        tick += 1;
        handed(c, tick, &mut by_tick);
        let _ = og_shared_take(c);
    }
    assert!(
        by_tick.is_empty(),
        "every entry was handed back: {by_tick:?}"
    );
    assert_eq!(
        og_net_hash(c),
        og_net_hash(b),
        "the replay's world is the session's"
    );
    assert_eq!(remotes(c).len(), 1);
}
