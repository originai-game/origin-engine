//! The order streaming reads reach the host, through its fetch hook: the
//! chunks of a sound cooked in chunks first, the one its playhead needs
//! soonest first, then the loading groups' packs by priority; never more
//! than `MAX_FETCHES` in flight; `dump_stream`'s `fetch` lists what is in
//! flight and what waits, in order.
//!
//! A project with an 8 s mono WAV cooked in 1.07 s chunks and three manual
//! groups of one palette each ("high" priority 2, "mid" 1, "low" 0) is
//! cooked once. The host answers every chunk and pack read `Pending` (as
//! the web host does while it fetches) until the test lets it land, and
//! records the order of first asks and how many were outstanding at once.
//!
//! Ways this could fail: chunks are asked for in bank order rather than by
//! distance to the playhead; packs go before chunks, or by name rather than
//! priority; a fifth read starts while four are outstanding; a landed read
//! is never put in place (the group never becomes resident, the chunk never
//! reaches the bank); the dump does not show the queue. Broken once by
//! queueing the packs before the chunks (the queue led with them), and
//! once by not bounding the reads (a fifth chunk was asked for at once).

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_probe_commands,
    og_engine_take_outputs, og_game_game, og_host_begin_bundle, og_load_sound, og_stream_request,
    og_stream_status, og_world_insert, og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

const RATE: u32 = 24_000;

/// A mono 16-bit WAV of `seconds` of a slow sweep.
fn wav(seconds: f64) -> Vec<u8> {
    let n = (seconds * f64::from(RATE)) as usize;
    let data: Vec<u8> = (0..n)
        .flat_map(|i| {
            let t = i as f64 / f64::from(RATE);
            let v = (0.3 * (std::f64::consts::TAU * (220.0 + 20.0 * t) * t).sin() * 32767.0) as i16;
            v.to_le_bytes()
        })
        .collect();
    let mut out = b"RIFF".to_vec();
    out.extend((36 + data.len() as u32).to_le_bytes());
    out.extend(b"WAVEfmt ");
    out.extend(16u32.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(RATE.to_le_bytes());
    out.extend((RATE * 2).to_le_bytes());
    out.extend(2u16.to_le_bytes());
    out.extend(16u16.to_le_bytes());
    out.extend(b"data");
    out.extend((data.len() as u32).to_le_bytes());
    out.extend(data);
    out
}

const SCENE: &str = r#"type scene
version 1
---
name "Fetch"
stream "low" {
  asset "p/c.hex"
}
stream "high" {
  asset "p/a.hex"
  priority 2
}
stream "mid" {
  asset "p/b.hex"
  priority 1
}
"#;

fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-stream-fetch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, bytes: &[u8]| {
            let p = root.join("assets").join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        };
        write("audio/theme.wav", &wav(8.0));
        write(
            "audio/theme.wav.ogt",
            b"type import\nversion 1\n---\nsound {\n  stream true\n  chunk 1\n}\n",
        );
        for (name, colour) in [("a", "ff0000"), ("b", "00ff00"), ("c", "0000ff")] {
            write(&format!("p/{name}.hex"), format!("{colour}\n").as_bytes());
        }
        write("scenes/fetch.ogt", SCENE.as_bytes());
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

/// What the host was asked: first asks in order, what is outstanding (asked,
/// not landed), the most outstanding at once, and what the test let land.
#[derive(Default)]
struct Gate {
    order: Vec<String>,
    outstanding: BTreeSet<String>,
    peak: usize,
    landed: BTreeSet<String>,
}

/// Reads from disk; a chunk or a pack is `Pending` until the test lands it.
struct Host(Rc<RefCell<Gate>>);

fn gated(path: &str) -> bool {
    path.contains("/stream/") || path.contains(".chunk")
}

/// A short name: `chunk3`, `pack high`.
fn short(path: &str) -> String {
    if let Some(i) = path.find(".chunk") {
        path[i + 1..].trim_end_matches(".ogb").to_string()
    } else {
        let name = path.rsplit('/').next().unwrap().trim_end_matches(".ogb");
        format!("pack {name}")
    }
}

impl Hooks for Host {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        if gated(path) {
            let mut g = self.0.borrow_mut();
            if !g.landed.contains(path) {
                if g.outstanding.insert(path.to_string()) {
                    g.order.push(short(path));
                    g.peak = g.peak.max(g.outstanding.len());
                }
                return Fetch::Pending;
            }
            g.outstanding.remove(path);
            g.landed.remove(path);
        }
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

struct Game {
    engine: u32,
    gate: Rc<RefCell<Gate>>,
}

impl Game {
    fn frame(&self) {
        value(og_engine_frame(self.engine, 1000.0 / 60.0));
    }

    /// Lets the outstanding reads named land.
    fn land(&self, names: &[&str]) {
        let mut g = self.gate.borrow_mut();
        let paths: Vec<String> = g
            .outstanding
            .iter()
            .filter(|p| names.contains(&short(p).as_str()))
            .cloned()
            .collect();
        assert_eq!(
            paths.len(),
            names.len(),
            "{names:?} outstanding: {:?}",
            g.outstanding
        );
        g.landed.extend(paths);
    }

    fn state(&self, group: &str) -> String {
        value(og_stream_status(self.engine, group.into()))["state"]
            .as_str()
            .unwrap()
            .to_string()
    }

    fn dump_stream(&self) -> Value {
        let scheduled: Value = serde_json::from_str(&og_engine_probe_commands(
            self.engine,
            "0:dump_stream".into(),
        ))
        .unwrap();
        assert_eq!(scheduled["scheduled"], json!(1), "{scheduled}");
        self.frame();
        let outputs: Value = serde_json::from_str(&og_engine_take_outputs(self.engine)).unwrap();
        let data = outputs
            .as_array()
            .and_then(|a| a.iter().rev().find(|o| o["action"] == json!("dump_stream")))
            .and_then(|o| o["data"].as_str())
            .unwrap_or_else(|| panic!("no dump_stream output: {outputs}"));
        serde_json::from_str(data).unwrap()
    }
}

#[test]
fn chunks_nearest_the_playhead_go_first_then_packs_by_priority_four_at_a_time() {
    let root = project();
    let gate = Rc::new(RefCell::new(Gate::default()));
    set_hooks(Rc::new(Host(gate.clone())));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let g = Game { engine, gate };
    g.frame();

    // A voice 2 s in: it wants chunks 1 (under it) to 5 (four seconds ahead).
    let sound = value(og_load_sound(engine, "audio/theme.wav".into()));
    let e = value(og_world_spawn(engine, Some("{}".into())))
        .as_f64()
        .unwrap();
    let source = json!({ "sound": sound, "startOffset": 2.0 }).to_string();
    value(og_world_insert(
        engine,
        e,
        "og.AudioSource".into(),
        Some(source),
    ));
    g.frame();
    assert_eq!(
        g.gate.borrow().order,
        ["chunk1", "chunk2", "chunk3", "chunk4"],
        "the nearest chunks first, four at a time"
    );

    // Every group asked for at once: they wait behind the chunks.
    for group in ["low", "high", "mid"] {
        value(og_stream_request(engine, group.into()));
    }
    g.frame();
    assert_eq!(g.gate.borrow().order.len(), 4, "no slot is free");
    let dump = g.dump_stream();
    let fetch = &dump["fetch"];
    assert_eq!(fetch["max"], json!(4));
    let names = |list: &Value| -> Vec<String> {
        list.as_array()
            .unwrap()
            .iter()
            .map(|w| match w["kind"].as_str().unwrap() {
                "chunk" => format!("chunk{}", w["chunk"]),
                _ => format!("pack {}", w["group"].as_str().unwrap()),
            })
            .collect()
    };
    let mut flying = names(&fetch["inFlight"]);
    flying.sort();
    assert_eq!(flying, ["chunk1", "chunk2", "chunk3", "chunk4"]);
    assert_eq!(
        names(&fetch["queued"]),
        ["chunk5", "pack high", "pack mid", "pack low"],
        "{fetch}"
    );
    let seconds: Vec<f64> = fetch["inFlight"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["seconds"].as_f64().unwrap())
        .collect();
    assert!(
        seconds.iter().all(|s| (0.0..4.0).contains(s)),
        "{seconds:?}"
    );
    assert_eq!(g.state("high"), "loading");

    // Two land: the next chunk and the highest group take their slots.
    g.land(&["chunk1", "chunk2"]);
    g.frame();
    assert_eq!(
        g.gate.borrow().order[4..],
        ["chunk5", "pack high"],
        "{:?}",
        g.gate.borrow().order
    );
    // The rest land: the other groups in priority order.
    g.land(&["chunk3", "chunk4", "chunk5", "pack high"]);
    g.frame();
    assert_eq!(g.gate.borrow().order[6..], ["pack mid", "pack low"]);
    g.land(&["pack mid", "pack low"]);
    g.frame();
    g.frame();
    for group in ["high", "mid", "low"] {
        assert_eq!(g.state(group), "resident", "{group}");
    }
    assert!(
        g.gate.borrow().peak <= 4,
        "{} outstanding at once",
        g.gate.borrow().peak
    );
    assert_eq!(g.gate.borrow().peak, 4);
    let fetch = g.dump_stream()["fetch"].clone();
    assert_eq!(fetch["inFlight"], json!([]), "{fetch}");
    assert_eq!(fetch["queued"], json!([]), "{fetch}");
    assert_eq!(fetch["started"], fetch["finished"], "{fetch}");
}
