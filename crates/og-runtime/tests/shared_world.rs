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

/// A UDP path whose source port the test can change, as a NAT that
/// rebinds: the client sends to `front`; what it sends leaves through the
/// current `back` socket, and only what the server sends to that socket
/// reaches the client (the old mapping is gone).
struct Nat {
    front: std::net::SocketAddr,
    back: std::sync::Arc<std::sync::Mutex<std::net::UdpSocket>>,
    rebinds: std::sync::Arc<std::sync::atomic::AtomicU32>,
}

impl Nat {
    fn new(server: std::net::SocketAddr) -> Nat {
        use std::net::UdpSocket;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::{Arc, Mutex};
        use std::time::Duration;
        let front = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr = front.local_addr().unwrap();
        let back = UdpSocket::bind("127.0.0.1:0").unwrap();
        back.connect(server).unwrap();
        let back = Arc::new(Mutex::new(back));
        let rebinds = Arc::new(AtomicU32::new(0));
        let client: Arc<Mutex<Option<std::net::SocketAddr>>> = Arc::default();
        let (b, c, f) = (back.clone(), client.clone(), front.try_clone().unwrap());
        std::thread::spawn(move || {
            let mut buf = vec![0u8; 65536];
            while let Ok((n, from)) = f.recv_from(&mut buf) {
                *c.lock().unwrap() = Some(from);
                let _ = b.lock().unwrap().send(&buf[..n]);
            }
        });
        // Down: from whichever back socket is current; a superseded one's
        // packets are dropped.
        let (b, r) = (back.clone(), rebinds.clone());
        std::thread::spawn(move || {
            let mut buf = vec![0u8; 65536];
            loop {
                let (sock, generation) = {
                    let s = b.lock().unwrap();
                    (s.try_clone().unwrap(), r.load(Ordering::SeqCst))
                };
                sock.set_read_timeout(Some(Duration::from_millis(5)))
                    .unwrap();
                while r.load(Ordering::SeqCst) == generation {
                    if let Ok(n) = sock.recv(&mut buf)
                        && let Some(to) = *client.lock().unwrap()
                    {
                        let _ = front.send_to(&buf[..n], to);
                    }
                }
            }
        });
        Nat {
            front: addr,
            back,
            rebinds,
        }
    }

    /// The client's packets now leave from a new port.
    fn rebind(&self) {
        let fresh = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let mut b = self.back.lock().unwrap();
        fresh.connect(b.peer_addr().unwrap()).unwrap();
        *b = fresh;
        self.rebinds
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

/// A host carrying one engine's ops over a real connection (as og-host-node).
struct Wire {
    engine: u32,
    url: String,
    hash: [u8; 32],
    token: String,
    client: Option<og_shared::client::Client>,
}

impl Wire {
    fn carry(&mut self) {
        let ops: Vec<Value> = serde_json::from_str(&og_shared_take(self.engine)).unwrap();
        for op in ops {
            match op["op"].as_str().unwrap() {
                "connect" => {
                    self.client = Some(og_shared::client::Client::connect_wt(
                        &self.url,
                        vec![self.hash],
                    ))
                }
                "send" => {
                    if let Some(c) = &self.client {
                        c.send(og_core::base64::decode(op["bytes"].as_str().unwrap()).unwrap());
                    }
                }
                "close" => self.client = None,
                _ => {}
            }
        }
    }

    fn pump(&mut self) {
        self.carry();
        let events = self.client.as_ref().map(|c| c.poll()).unwrap_or_default();
        for e in events {
            use og_shared::client::ClientEvent;
            let v = match e {
                ClientEvent::Open => {
                    json!({ "type": "open", "transport": "wt", "token": self.token })
                }
                ClientEvent::Message(b) => {
                    json!({ "type": "message", "bytes": og_core::base64::encode(&b) })
                }
                ClientEvent::Closed(why) => {
                    self.client = None;
                    json!({ "type": "closed", "reason": why })
                }
            };
            assert!(og_shared_deliver(self.engine, v.to_string()));
        }
        self.carry();
    }
}

/// A WebTransport client whose NAT rebinds mid-session: the server's
/// connection ids are zero-length, so it drops the packets from the new
/// address instead of migrating, and the client is never told. Ways this
/// could fail: the client waits for QUIC's idle timeout and comes back
/// after the server's 10 s linger as a new avatar (the others despawn it
/// and spawn another); it reconnects without resuming (the same); the
/// server keeps both (two avatars of one player); the resumed avatar loses
/// where it was or is corrected; the reconnecting client despawns and
/// respawns the avatars it held; a 3 s load hitch, longer than the silence
/// limit, is taken for a lost connection.
#[test]
fn a_client_whose_nat_rebinds_resumes_its_avatar() {
    use og_shared::auth::{Rules, Verifier, sign_hs256};
    use og_shared::server::{KeySource, Server};
    use std::time::{Duration, Instant};
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let cert = og_shared::cert::self_signed(&["localhost".into(), "127.0.0.1".into()], 1).unwrap();
    let (server, wt) = rt.block_on(async {
        let s = Server::start(
            "127.0.0.1:0".parse().unwrap(),
            Config::default(),
            Verifier::hs256(b"s", Rules::audience("g")),
            KeySource::None,
            false,
            None,
        )
        .await
        .unwrap();
        let wt = s
            .serve_webtransport(
                "127.0.0.1:0".parse().unwrap(),
                &cert.cert_pem,
                &cert.key_pem,
            )
            .unwrap();
        (s, wt)
    });
    let nat = Nat::new(wt);
    let exp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 120;
    let token = |p: &str| sign_hs256(b"s", &json!({ "aud": "g", "sub": p, "exp": exp }));
    let (a, b) = (engine(), engine());
    let me_a = visitor(a, 0.0);
    let _me_b = visitor(b, 10.0);
    let mut wires = [
        Wire {
            engine: a,
            url: format!("https://{}/", nat.front),
            hash: cert.sha256,
            token: token("a"),
            client: None,
        },
        Wire {
            engine: b,
            url: format!("https://{wt}/"),
            hash: cert.sha256,
            token: token("b"),
            client: None,
        },
    ];
    // Real time: the server ticks on its own clock.
    let mut ax = 0.0;
    let mut frame = |wires: &mut [Wire; 2], ax: &mut f64| {
        let t = Instant::now();
        *ax += 1.0 / 60.0;
        place(a, me_a, *ax);
        for w in wires.iter_mut() {
            w.pump();
            answer(og_engine_frame(w.engine, FRAME_MS));
        }
        std::thread::sleep(Duration::from_secs_f64(FRAME_MS / 1000.0).saturating_sub(t.elapsed()));
    };
    let until = |wires: &mut [Wire; 2],
                 ax: &mut f64,
                 frame: &mut dyn FnMut(&mut [Wire; 2], &mut f64),
                 secs: f64,
                 done: &dyn Fn() -> bool| {
        let t = Instant::now();
        while !done() {
            assert!(t.elapsed().as_secs_f64() < secs, "not within {secs} s");
            frame(wires, ax);
        }
        t.elapsed()
    };
    until(&mut wires, &mut ax, &mut frame, 10.0, &|| {
        og_shared_status(a).state == "joined" && remotes(b).len() == 1 && remotes(a).len() == 1
    });
    let held = remotes(b)[0];
    let held_by_a = remotes(a)[0];
    assert_eq!(og_shared_remote(b, held).unwrap().player, "a");

    // A load hitch (or a tab in the background): no frame for 3 s, longer
    // than the silence limit. The connection's own thread kept hearing the
    // server, so nothing is lost and nothing reconnects.
    std::thread::sleep(Duration::from_secs(3));
    for _ in 0..60 {
        frame(&mut wires, &mut ax);
        assert_eq!(
            og_shared_status(a).state,
            "joined",
            "a hitch was taken for a lost connection"
        );
        assert_eq!((remotes(a), remotes(b)), (vec![held_by_a], vec![held]));
    }

    nat.rebind();
    let lost = until(&mut wires, &mut ax, &mut frame, 5.0, &|| {
        og_shared_status(a).state != "joined"
    });
    // Back, and b's avatar of a is the same entity all along.
    let mut seen_b = Vec::new();
    let back = until(&mut wires, &mut ax, &mut frame, 8.0, &|| {
        let r = remotes(b);
        assert!(
            r.len() <= 1 && r.iter().all(|e| *e == held),
            "b holds {r:?}"
        );
        og_shared_status(a).state == "joined" && remotes(a).len() == 1
    });
    // Walk on: what a proposes after the resume reaches b, uncorrected.
    for _ in 0..60 {
        frame(&mut wires, &mut ax);
        let r = remotes(b);
        assert!(r == [held], "b holds {r:?}, not the one avatar it held");
        seen_b.push(x_of(b, held));
    }
    eprintln!(
        "nat rebind: noticed after {:.0} ms, resumed {:.0} ms after the rebind",
        lost.as_secs_f64() * 1000.0,
        (lost + back).as_secs_f64() * 1000.0
    );
    assert!(
        lost + back < Duration::from_secs(5),
        "resumed after {:?}",
        lost + back
    );
    let st = og_shared_status(a);
    assert_eq!((st.player.as_str(), st.corrections), ("a", 0));
    // a's own view of b carried on as the same entity across the reconnect.
    assert_eq!(remotes(a), vec![held_by_a], "a respawned b");
    assert!(
        (ax - seen_b.last().unwrap()).abs() < 1.0,
        "b draws a at {} while it is at {ax}",
        seen_b.last().unwrap()
    );
    assert_eq!(og_shared_status(b).in_sight, 1);
    drop(wires);
    drop(server);
    drop(rt);
}
