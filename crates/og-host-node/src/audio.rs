//! Audio output of the native host (`crates/og-audio/README.md` §Host
//! contract). An engine's audio link (published by the runtime at engine
//! creation) is consumed by one sink per engine:
//!
//! - `null`: drains the command stream every pump and renders nothing, so a
//!   headless run keeps its queue empty and its stream deterministic;
//! - `capture`: renders the stream with a driven mixer up to the world's
//!   tick into memory; `og_node_audio_wav` returns it as a 16-bit WAV;
//! - `device` (feature `native-audio`): the default output device through
//!   cpal; the device callback drains the link and pulls the mixer itself,
//!   and measures the device's latency (its playback instant less its
//!   callback instant), which every pump reports to og.Audio.outputLatency
//!   with the mixer's own buffer. The null and capture sinks are driven to
//!   the tick: their latency is 0.
//!
//! Every member answers wire JSON (`{ok, value}` / `{ok:false, reason}`)
//! except `og_node_audio_wav` (a Buffer, empty when nothing was captured)
//! and `og_node_audio_close` (whether a sink was open).

// napi registers the exports outside `cfg(test)` only.
#![cfg_attr(test, allow(dead_code))]

use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use og_audio::{HeadlessMode, HeadlessSink, SinkStats};
use og_runtime::host_impl::{json_string, no_audio, refuse};
use std::cell::RefCell;
use std::collections::BTreeMap;

enum Sink {
    Headless(Box<HeadlessSink>),
    #[cfg(feature = "native-audio")]
    Device(og_audio::device::DeviceSink),
}

thread_local! {
    static SINKS: RefCell<BTreeMap<u32, Sink>> = const { RefCell::new(BTreeMap::new()) };
}

fn ok(value: String) -> String {
    format!(r#"{{"ok":true,"value":{value}}}"#)
}

/// The bank's decoding counters as JSON members (og-audio `decode.rs`):
/// frames the decode thread decoded, frames a voice decoded where it read
/// them (none had decoded them first), jobs waiting.
fn decode_members(d: og_audio::DecodeStats) -> String {
    format!(
        r#""decodedAhead":{},"decodeMisses":{},"decodeQueued":{}"#,
        d.ahead, d.misses, d.queued
    )
}

fn stats_json(engine: u32, mode: &str, s: &SinkStats, sample_rate: u32, dropped: u64) -> String {
    format!(
        r#"{{"mode":{},"sampleRate":{},"packets":{},"commands":{},"plays":{},"resyncs":{},"receivedHash":"{:016x}","frames":{},"voices":{},"voicesPeak":{},"late":{},"limited":{},"dropped":{},{}}}"#,
        json_string(mode),
        sample_rate,
        s.packets,
        s.commands,
        s.plays,
        s.resyncs,
        s.received_hash,
        s.mix.frames,
        s.mix.voices,
        s.mix.voices_peak,
        s.mix.late,
        s.mix.limited,
        dropped,
        decode_members(
            og_audio::link(engine)
                .map(|l| l.bank().decode_stats())
                .unwrap_or_default()
        )
    )
}

/// Open engine `engine`'s audio sink: `mode` is `null`, `capture` or
/// `device`; `sample_rate` (capture; default 48000). Replaces an open sink.
/// A Result of the mode actually opened.
#[napi(js_name = "og_node_audio_open")]
pub fn og_node_audio_open(engine: u32, mode: String, sample_rate: Option<u32>) -> String {
    let Some(link) = og_audio::link(engine) else {
        return no_audio(engine);
    };
    og_node_audio_close(engine);
    let sink = match mode.as_str() {
        "null" => Sink::Headless(Box::new(HeadlessSink::new(&link, HeadlessMode::Null))),
        "capture" => Sink::Headless(Box::new(HeadlessSink::new(
            &link,
            HeadlessMode::Capture {
                sample_rate: sample_rate.unwrap_or(48_000).clamp(8_000, 192_000),
            },
        ))),
        #[cfg(feature = "native-audio")]
        "device" => match og_audio::device::DeviceSink::open(&link) {
            Ok(d) => Sink::Device(d),
            Err(e) => return refuse(&e),
        },
        #[cfg(not(feature = "native-audio"))]
        "device" => {
            return refuse(
                "this addon was built without the `native-audio` feature (cpal); use null or capture",
            );
        }
        other => {
            return refuse(&format!(
                "unknown audio mode '{other}': null, capture or device"
            ));
        }
    };
    SINKS.with(|s| s.borrow_mut().insert(engine, sink));
    ok(json_string(&mode))
}

/// Drain the engine's command stream into its sink and (capture) render up
/// to the world's tick. Call after every advance/frame. A Result of the
/// sink's stats.
#[napi(js_name = "og_node_audio_pump")]
pub fn og_node_audio_pump(engine: u32) -> String {
    SINKS.with(|s| match s.borrow_mut().get_mut(&engine) {
        Some(Sink::Headless(h)) => {
            h.pump();
            ok(headless_stats(engine, h))
        }
        #[cfg(feature = "native-audio")]
        Some(Sink::Device(d)) => {
            // The device's latency reaches og.Audio.outputLatency between ticks, as host input.
            let _ = og_runtime::set_audio_latency(engine, d.latency());
            ok(device_json(d))
        }
        None => refuse(&format!(
            "engine {engine} has no open audio sink (og_node_audio_open)"
        )),
    })
}

fn headless_stats(engine: u32, h: &HeadlessSink) -> String {
    let dropped = og_audio::link(engine).map_or(0, |l| l.dropped());
    match h.sample_rate() {
        Some(rate) => stats_json(engine, "capture", &h.stats(), rate, dropped),
        None => stats_json(engine, "null", &h.stats(), 0, dropped),
    }
}

/// The sink's counters (packets, commands, frames, voices, the hash of
/// what it received). A Result.
#[napi(js_name = "og_node_audio_stats")]
pub fn og_node_audio_stats(engine: u32) -> String {
    SINKS.with(|s| match s.borrow().get(&engine) {
        Some(Sink::Headless(h)) => ok(headless_stats(engine, h)),
        #[cfg(feature = "native-audio")]
        Some(Sink::Device(d)) => ok(device_json(d)),
        None => refuse(&format!("engine {engine} has no open audio sink")),
    })
}

/// The captured PCM as a 16-bit stereo WAV (empty unless the sink is `capture`).
#[napi(js_name = "og_node_audio_wav")]
pub fn og_node_audio_wav(engine: u32) -> Buffer {
    SINKS.with(|s| match s.borrow().get(&engine) {
        Some(Sink::Headless(h)) => match h.sample_rate() {
            Some(rate) => og_audio::wav(h.pcm(), 2, rate).into(),
            None => Vec::new().into(),
        },
        _ => Vec::new().into(),
    })
}

/// Close the engine's sink (the link stops queueing). Whether one was open.
#[napi(js_name = "og_node_audio_close")]
pub fn og_node_audio_close(engine: u32) -> bool {
    SINKS.with(|s| s.borrow_mut().remove(&engine)).is_some()
}

/// Add a sound (a QOA file: a cooked sound's `qoa` section) to the engine's
/// bank. A Result of its handle, the number `AudioSource.sound` takes.
#[napi(js_name = "og_node_audio_insert_qoa")]
pub fn og_node_audio_insert_qoa(engine: u32, id: String, qoa: Buffer) -> String {
    let Some(link) = og_audio::link(engine) else {
        return no_audio(engine);
    };
    match link.bank().insert_qoa(&id, &qoa) {
        Ok(handle) => ok(handle.to_string()),
        Err(e) => refuse(e.reason()),
    }
}

/// The device sink's stats as wire JSON.
#[cfg(feature = "native-audio")]
fn device_json(d: &og_audio::device::DeviceSink) -> String {
    let s = d.stats();
    format!(
        r#"{{"mode":"device","device":{},"sampleRate":{},"frames":{},"commands":{},"voices":{},"voicesPeak":{},"late":{},"reanchors":{},"limited":{},"dropped":{},{}}}"#,
        json_string(d.name()),
        d.sample_rate(),
        s.frames,
        s.commands,
        s.voices,
        s.voices_peak,
        s.late,
        s.reanchors,
        s.limited,
        d.link().dropped(),
        decode_members(d.link().bank().decode_stats())
    )
}
