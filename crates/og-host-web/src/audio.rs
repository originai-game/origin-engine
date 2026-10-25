//! Audio output of the web host (`crates/og-audio/README.md` §Host
//! contract). The mixer runs here, in the engine Worker, driven by the
//! world's tick: after each frame's ticks the Worker calls
//! `og_web_audio_pump`, which renders the PCM those ticks cover and returns
//! it; `packages/engine/src/web/audio.ts` writes it into the
//! SharedArrayBuffer ring the AudioWorklet reads (or posts it to the
//! worklet's port without cross-origin isolation).
//!
//! The sounds' QOA frames are decoded beside the tick, off this Worker
//! (og-audio's `decode.rs`): `web/audio-decoder.ts` takes the bank's jobs
//! after each pump (`og_web_audio_decode_jobs`), decodes them on a decoder
//! Worker and answers each (`og_web_audio_put_decoded`). A frame the mixer
//! reaches before its answer is decoded here, to the same samples.
//!
//! Every member answers wire JSON except `og_web_audio_pump` (a
//! Float32Array of interleaved stereo frames, empty when nothing is due),
//! `og_web_audio_close` (whether a mixer was open),
//! `og_web_audio_decode_jobs` (an Array) and `og_web_audio_put_decoded`
//! (whether the answer was taken).

use og_audio::{Clock, Mixer};
use og_core::IdMap;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

struct WebAudio {
    link: og_audio::AudioLink,
    mixer: Mixer,
    pcm: Vec<f32>,
    /// Frames dropped to catch up a mixer that fell behind the ticks.
    skipped: u64,
}

thread_local! {
    static OUTPUTS: RefCell<IdMap<u32, WebAudio>> = const { RefCell::new(IdMap::new()) };
}

/// The most audio the mixer may lag the world before it skips ahead (250 ms).
const MAX_BACKLOG_MS: u64 = 250;

fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn refuse(reason: &str) -> String {
    format!(r#"{{"ok":false,"reason":{}}}"#, json_str(reason))
}

fn no_link(engine: u32) -> String {
    refuse(&format!(
        "engine {engine} has no audio: the runtime did not install og-audio for it (og_audio::install + publish_link at engine creation)"
    ))
}

/// The output latency the page reports (the AudioContext's `baseLatency`
/// and `outputLatency` and the worklet ring's buffer, in seconds), written to
/// og.Audio.outputLatency before the next tick: host input. A Result.
#[wasm_bindgen]
pub fn og_web_audio_latency(engine: u32, seconds: f64) -> String {
    match og_runtime::set_audio_latency(engine, seconds) {
        Ok(()) => r#"{"ok":true}"#.to_string(),
        Err(e) => refuse(&e),
    }
}

/// Open engine `engine`'s mixer at `sample_rate` (the AudioContext's) and
/// attach it to the engine's command stream; the next tick resyncs it, so
/// what is already playing is heard. Replaces an open mixer. A Result.
#[wasm_bindgen]
pub fn og_web_audio_open(engine: u32, sample_rate: u32) -> String {
    let Some(link) = og_audio::link(engine) else {
        return no_link(engine);
    };
    let rate = sample_rate.clamp(8_000, 192_000);
    let mixer = Mixer::for_link(&link, rate, Clock::Driven);
    link.attach();
    OUTPUTS.with(|o| {
        o.borrow_mut().insert(
            engine,
            WebAudio {
                link,
                mixer,
                pcm: Vec::new(),
                skipped: 0,
            },
        )
    });
    format!(r#"{{"ok":true,"value":{rate}}}"#)
}

/// Render the audio of the ticks run since the last pump, at most
/// `max_frames` frames (the ring's free space): interleaved stereo f32. A
/// mixer more than 250 ms behind the world skips ahead first.
#[wasm_bindgen]
pub fn og_web_audio_pump(engine: u32, max_frames: u32) -> js_sys::Float32Array {
    OUTPUTS.with(|o| {
        let mut o = o.borrow_mut();
        let Some(out) = o.get_mut(&engine) else {
            return js_sys::Float32Array::new_with_length(0);
        };
        out.mixer.pump(&out.link);
        let tick = out.link.next_tick();
        let due = out.mixer.frames_until_tick(tick);
        let backlog = MAX_BACKLOG_MS * u64::from(out.mixer.desc().sample_rate) / 1000;
        if due > backlog {
            let skip = due - backlog;
            out.mixer.skip(skip as usize);
            out.skipped += skip;
        }
        out.pcm.clear();
        out.mixer
            .render_until_tick(tick, max_frames as usize, &mut out.pcm);
        js_sys::Float32Array::from(&out.pcm[..])
    })
}

/// The mixer's counters. A Result.
#[wasm_bindgen]
pub fn og_web_audio_stats(engine: u32) -> String {
    OUTPUTS.with(|o| match o.borrow().get(&engine) {
        Some(out) => {
            let s = out.mixer.stats();
            let d = out.link.bank().decode_stats();
            format!(
                r#"{{"ok":true,"value":{{"sampleRate":{},"frames":{},"commands":{},"voices":{},"voicesPeak":{},"late":{},"limited":{},"skipped":{},"dropped":{},"behind":{},"decodedAhead":{},"decodeMisses":{},"decodeQueued":{},"decodeInFlight":{}}}}}"#,
                out.mixer.desc().sample_rate,
                s.frames,
                s.commands,
                s.voices,
                s.voices_peak,
                s.late,
                s.limited,
                out.skipped,
                out.link.dropped(),
                out.mixer.frames_until_tick(out.link.next_tick()),
                d.ahead,
                d.misses,
                d.queued,
                d.in_flight
            )
        }
        None => refuse(&format!("engine {engine} has no open audio output (og_web_audio_open)")),
    })
}

/// Close the engine's mixer (the link stops queueing). Whether one was open.
#[wasm_bindgen]
pub fn og_web_audio_close(engine: u32) -> bool {
    OUTPUTS
        .with(|o| o.borrow_mut().remove(&engine))
        .is_some_and(|out| {
            out.link.detach();
            true
        })
}

/// Add a sound (a QOA file: a cooked sound's `qoa` section) to the engine's
/// bank. A Result of its handle, the number `AudioSource.sound` takes.
#[wasm_bindgen]
pub fn og_web_audio_insert_qoa(engine: u32, id: String, qoa: &[u8]) -> String {
    let Some(link) = og_audio::link(engine) else {
        return no_link(engine);
    };
    match link.bank().insert_qoa(&id, qoa) {
        Ok(handle) => format!(r#"{{"ok":true,"value":{handle}}}"#),
        Err(e) => refuse(e.reason()),
    }
}

/// Up to `max` of the engine's decode jobs (og-audio `decode.rs`: the
/// frames the decode-ahead and new sounds want), soonest needed first, for
/// the decoder Worker: a flat Array `[key, frame, Uint8Array, ...]`, each
/// Uint8Array a one-frame QOA file. Empty for an engine without audio.
#[wasm_bindgen]
pub fn og_web_audio_decode_jobs(engine: u32, max: u32) -> js_sys::Array {
    let out = js_sys::Array::new();
    let Some(link) = og_audio::link(engine) else {
        return out;
    };
    for job in link.bank().take_decode_jobs(max as usize) {
        out.push(&JsValue::from(job.key));
        out.push(&JsValue::from(job.frame));
        out.push(&js_sys::Uint8Array::from(&job.qoa[..]));
    }
    out
}

/// The decoder Worker's answer to job `key`: the frame's interleaved
/// samples. Whether they were taken (false: a key not in flight, samples
/// of another length, an engine without audio).
#[wasm_bindgen]
pub fn og_web_audio_put_decoded(engine: u32, key: u32, samples: &[i16]) -> bool {
    og_audio::link(engine).is_some_and(|l| l.bank().put_decoded(key, samples))
}
