//! The sound a timeline plays, heard (docs/design.md §2.7): its audio
//! and music tracks' voices rendered by og-audio's capture sink (what
//! `og run --audio out.wav` writes) are a click at the timeline's 0 s,
//! silence, a tone from its audio key at 1 s for its 0.25 s, silence, and
//! the music from its key at 2 s rising over its 0.5 s fade-in, steady,
//! falling over its 0.5 s fade-out to its `end` at 3.5 s, and silence
//! after. The capture is written to `$OG_EVIDENCE_DIR/timeline-sound.wav`.
//!
//! Ways this could fail: a voice spawned but never started (the music's
//! voice written with `looping`, not og.AudioSource's `loop`, refused the
//! frame: 61c31247; with that line put back this test fails), a key fired
//! a tick late or twice, the gain written once and never followed, a voice
//! left playing past `end`.
//!
//! A file of its own: og-audio's link registry is per process and engine
//! ids are per thread, so a capture sink beside other tests' engines in one
//! process could attach to another test's link.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
    og_load_timeline, og_timeline_play, og_world_insert, og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::rc::Rc;

struct Disk;

impl Hooks for Disk {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
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

const SOUND_TIMELINE: &str = r#"type timeline
version 1
---
name "S"
duration 5
track audio {
  key 0 {
    sound "audio/click.wav"
  }
  key 1 {
    sound "audio/tone.wav"
  }
}
track music {
  key 2 {
    sound "audio/hum.wav"
    fadeIn 0.5
    fadeOut 0.5
    end 3.5
    loop true
  }
}
"#;

/// A mono sine of `hz` at amplitude 0.5 for `seconds`, as a 48 kHz WAV.
// Test reference values: libm checks the result, never the tick.
#[allow(clippy::disallowed_methods)]
fn sine_wav(hz: f32, seconds: f32) -> Vec<u8> {
    let n = (48_000.0 * seconds) as usize;
    let pcm: Vec<f32> = (0..n)
        .map(|i| 0.5 * (std::f32::consts::TAU * hz * i as f32 / 48_000.0).sin())
        .collect();
    og_audio::wav(&pcm, 1, 48_000)
}

#[test]
fn the_timelines_sound_is_heard_where_its_keys_put_it() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-timeline-sound-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let audio = root.join("assets/audio");
    std::fs::create_dir_all(&audio).unwrap();
    std::fs::write(audio.join("click.wav"), sine_wav(880.0, 0.05)).unwrap();
    std::fs::write(audio.join("tone.wav"), sine_wav(440.0, 0.25)).unwrap();
    std::fs::write(audio.join("hum.wav"), sine_wav(220.0, 1.0)).unwrap();
    let t = root.join("assets/timelines/t.ogt");
    std::fs::create_dir_all(t.parent().unwrap()).unwrap();
    std::fs::write(&t, SOUND_TIMELINE).unwrap();
    let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let asset = value(og_load_timeline(engine, "timelines/t.ogt".into()));
    let timeline = value(og_world_spawn(engine, Some("{}".into())))
        .as_f64()
        .unwrap();
    value(og_world_insert(
        engine,
        timeline,
        "og.Timeline".into(),
        Some(json!({ "asset": asset }).to_string()),
    ));
    let link = og_audio::link(engine).expect("the engine's audio link");
    let mut sink = og_audio::HeadlessSink::new(
        &link,
        og_audio::HeadlessMode::Capture {
            sample_rate: 48_000,
        },
    );
    sink.pump();
    value(og_timeline_play(engine, timeline));
    for _ in 0..330 {
        value(og_engine_frame(engine, 1000.0 / 60.0));
        sink.pump();
    }
    // Stereo at 48 kHz. The click at the timeline's 0 s is the clock: the
    // first sound rendered, every other time is read from its onset.
    let pcm = sink.pcm();
    let start = pcm
        .chunks(2)
        .position(|f| f[0].abs() > 0.05)
        .expect("the click at 0 s is heard");
    let span = |a: f32, b: f32| {
        let at = |s: f32| (start + (s * 48_000.0) as usize) * 2;
        &pcm[at(a)..at(b).min(pcm.len())]
    };
    let peak = |a: f32, b: f32| span(a, b).iter().fold(0.0f32, |m, x| m.max(x.abs()));
    let rms = |a: f32, b: f32| {
        let s = span(a, b);
        (s.iter().map(|x| x * x).sum::<f32>() / s.len() as f32).sqrt()
    };
    assert!(
        pcm.len() >= (start + 5 * 48_000) * 2,
        "{} samples",
        pcm.len()
    );
    // After the click, silent until the audio key at 1 s; then the tone,
    // from its key (within a tick) for its 0.25 s.
    assert!(peak(0.1, 0.98) < 1e-3, "before 1 s: {}", peak(0.1, 0.98));
    let onset = span(0.9, 1.3)
        .chunks(2)
        .position(|f| f[0].abs() > 0.05)
        .expect("the tone is heard") as f32
        / 48_000.0
        + 0.9;
    assert!(
        (onset - 1.0).abs() < 1.5 / 60.0,
        "the tone starts at {onset} s"
    );
    assert!(rms(1.05, 1.2) > 0.1, "the tone: rms {}", rms(1.05, 1.2));
    assert!(
        peak(1.3, 1.98) < 1e-3,
        "after the tone: {}",
        peak(1.3, 1.98)
    );
    // The music: rising over its fade-in (2 to 2.5 s), steady, falling
    // over its fade-out to its end at 3.5 s, then silent.
    let (rising, later, full, falling) =
        (rms(2.05, 2.15), rms(2.3, 2.4), rms(2.6, 2.9), rms(3.3, 3.4));
    assert!(
        0.0 < rising && rising < later && later < full,
        "fade-in: {rising} {later} {full}"
    );
    assert!(full > 0.2, "the music at full volume: rms {full}");
    assert!(
        falling < 0.5 * full && falling > 0.0,
        "fade-out: {falling} of {full}"
    );
    assert!(peak(3.6, 4.9) < 1e-3, "after its end: {}", peak(3.6, 4.9));
    let evidence = std::env::var_os("OG_EVIDENCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("evidence"));
    std::fs::create_dir_all(&evidence).unwrap();
    std::fs::write(
        evidence.join("timeline-sound.wav"),
        og_audio::wav(&pcm[start * 2..], 2, 48_000),
    )
    .unwrap();
}
