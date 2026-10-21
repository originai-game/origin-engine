//! A sound cooked in chunks inside a streaming group (examples/
//! WEB-FINDINGS.md #2): until its group is resident it is silence of its
//! own length and nothing of it is read; once the group's pack lands, its
//! head (the first chunk) takes the stand-in's handle and its other chunks
//! are read ahead of the playheads, the nearest first, as a chunked sound of
//! the first group's are; when the group is let go the sound is silent again
//! and no chunk is read.
//!
//! A project with an 8 s mono WAV cooked in 1 s chunks, in a manual group
//! "music", is cooked once. The host answers chunk and pack reads `Pending`
//! until the test lets them land and records the order of first asks.
//!
//! Ways this could fail: the cook refuses the sound in a group, or packs
//! its chunks (the pack then carries the whole track); the stand-in asks
//! for chunks before its group is resident (the boot would fetch them
//! anyway); the head is not put in place (the playhead never asks for a
//! chunk, the sound stays silent); a released group keeps asking for
//! chunks.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
    og_load_sound, og_stream_release, og_stream_request, og_stream_status, og_world_insert,
    og_world_spawn, set_hooks,
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
name "Music"
stream "music" {
  asset "audio/theme.wav"
}
"#;

fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-stream-chunked-{}", std::process::id()));
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
        write("scenes/music.ogt", SCENE.as_bytes());
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

#[derive(Default)]
struct Gate {
    order: Vec<String>,
    outstanding: BTreeSet<String>,
    landed: BTreeSet<String>,
}

/// Reads from disk; a chunk or a pack is `Pending` until the test lands it.
struct Host(Rc<RefCell<Gate>>);

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
        if path.contains("/stream/") || path.contains(".chunk") {
            let mut g = self.0.borrow_mut();
            if !g.landed.contains(path) {
                if g.outstanding.insert(path.to_string()) {
                    g.order.push(short(path));
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

#[test]
fn a_chunked_sound_in_a_group_reads_no_chunk_until_its_group_is_resident() {
    let root = project();
    // The pack holds the head only; the chunks are files of their own.
    let manifest = std::fs::read_to_string(root.join("cooked/.cook.ogt")).unwrap();
    let chunks = std::fs::read_dir(root.join("cooked/audio"))
        .unwrap()
        .filter(|f| {
            f.as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains(".chunk")
        })
        .count();
    assert_eq!(
        chunks, 7,
        "8 s in 1 s chunks: the head and 7 files\n{manifest}"
    );

    let gate = Rc::new(RefCell::new(Gate::default()));
    set_hooks(Rc::new(Host(gate.clone())));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let frame = || value(og_engine_frame(engine, 1000.0 / 60.0));
    let state = || {
        value(og_stream_status(engine, "music".into()))["state"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let land = |names: &[&str]| {
        let mut g = gate.borrow_mut();
        let paths: Vec<String> = g
            .outstanding
            .iter()
            .filter(|p| names.contains(&short(p).as_str()))
            .cloned()
            .collect();
        assert_eq!(paths.len(), names.len(), "{names:?}: {:?}", g.outstanding);
        g.landed.extend(paths);
    };
    frame();

    // A voice 2 s in on the stand-in: silence of its length, nothing read.
    let sound = value(og_load_sound(engine, "audio/theme.wav".into()));
    let e = value(og_world_spawn(engine, Some("{}".into())))
        .as_f64()
        .unwrap();
    let source = json!({ "sound": sound, "startOffset": 2.0, "loop": true }).to_string();
    value(og_world_insert(
        engine,
        e,
        "og.AudioSource".into(),
        Some(source),
    ));
    for _ in 0..10 {
        frame();
    }
    assert_eq!(state(), "unloaded");
    assert!(gate.borrow().order.is_empty(), "{:?}", gate.borrow().order);

    // The group loads: its pack, then the chunks ahead of the playhead.
    value(og_stream_request(engine, "music".into()));
    frame();
    assert_eq!(gate.borrow().order, ["pack music"]);
    land(&["pack music"]);
    frame();
    frame();
    assert_eq!(state(), "resident");
    let asked = gate.borrow().order[1..].to_vec();
    assert!(!asked.is_empty(), "the playhead asks for its chunks");
    assert!(asked.iter().all(|c| c.starts_with("chunk")), "{asked:?}");
    // The playhead is past 2 s (2.13 s is chunk 2's start): half a second
    // behind it to four ahead, the nearest first.
    assert!(
        ["chunk1", "chunk2"].contains(&asked[0].as_str()),
        "the chunks around the playhead first: {asked:?}"
    );

    // Let go: silent again, and no chunk is asked for any more.
    let landed: Vec<String> = gate.borrow().outstanding.iter().map(|p| short(p)).collect();
    land(&landed.iter().map(String::as_str).collect::<Vec<_>>());
    value(og_stream_release(engine, "music".into()));
    for _ in 0..4 {
        frame();
    }
    assert_eq!(state(), "unloaded");
    let before = gate.borrow().order.len();
    for _ in 0..120 {
        frame();
    }
    assert_eq!(
        gate.borrow().order.len(),
        before,
        "{:?}",
        &gate.borrow().order[before..]
    );
}
