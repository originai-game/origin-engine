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
use std::cell::RefCell;
use std::collections::BTreeMap;

enum Sink {
    Headless(Box<HeadlessSink>),
    #[cfg(feature = "native-audio")]
    Device(device::DeviceSink),
}

thread_local! {
    static SINKS: RefCell<BTreeMap<u32, Sink>> = const { RefCell::new(BTreeMap::new()) };
}

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

fn ok(value: String) -> String {
    format!(r#"{{"ok":true,"value":{value}}}"#)
}

fn refuse(reason: &str) -> String {
    format!(r#"{{"ok":false,"reason":{}}}"#, json_str(reason))
}

fn no_link(engine: u32) -> String {
    refuse(&format!(
        "engine {engine} has no audio: the runtime did not install og-audio for it (og_audio::install + publish_link at engine creation)"
    ))
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
        json_str(mode),
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
        return no_link(engine);
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
        "device" => match device::DeviceSink::open(&link) {
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
    ok(json_str(&mode))
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
            ok(d.stats_json())
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
        Some(Sink::Device(d)) => ok(d.stats_json()),
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
        return no_link(engine);
    };
    match link.bank().insert_qoa(&id, &qoa) {
        Ok(handle) => ok(handle.to_string()),
        Err(e) => refuse(e.reason()),
    }
}

#[cfg(feature = "native-audio")]
mod device {
    //! The default output device through cpal. The callback owns the
    //! render: it drains the link, pushes the packets into a free-clock
    //! mixer and renders the device's buffer.

    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use og_audio::{AudioLink, Clock, Mixer};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};

    pub struct DeviceSink {
        _stream: cpal::Stream,
        mixer: Arc<Mutex<Mixer>>,
        link: AudioLink,
        sample_rate: u32,
        name: String,
        /// The device's own latency in seconds (f64 bits), measured each callback.
        device_latency: Arc<AtomicU64>,
        /// The free mixer's buffer in front of the device, in frames.
        buffer_frames: u32,
    }

    /// The seconds a callback's samples wait before they play: its playback
    /// instant less its callback instant (0 when the host cannot tell).
    fn callback_latency(info: &cpal::OutputCallbackInfo) -> f64 {
        let t = info.timestamp();
        t.playback
            .duration_since(&t.callback)
            .map_or(0.0, |d| d.as_secs_f64())
    }

    fn fill<T: cpal::SizedSample + cpal::FromSample<f32>>(
        out: &mut [T],
        channels: usize,
        mixer: &Mutex<Mixer>,
        link: &AudioLink,
        scratch: &mut Vec<f32>,
    ) {
        let frames = out.len() / channels.max(1);
        scratch.resize(frames * 2, 0.0);
        if let Ok(mut m) = mixer.lock() {
            m.pump(link);
            m.render(scratch);
        } else {
            scratch.fill(0.0);
        }
        for (f, frame) in out.chunks_exact_mut(channels).enumerate() {
            let (l, r) = (scratch[f * 2], scratch[f * 2 + 1]);
            for (c, s) in frame.iter_mut().enumerate() {
                let v = match (channels, c) {
                    (1, _) => 0.5 * (l + r),
                    (_, 0) => l,
                    (_, 1) => r,
                    _ => 0.0,
                };
                *s = T::from_sample(v);
            }
        }
    }

    impl DeviceSink {
        pub fn open(link: &AudioLink) -> Result<DeviceSink, String> {
            let host = cpal::default_host();
            let device = host
                .default_output_device()
                .ok_or("no default audio output device")?;
            let name = device
                .description()
                .map(|d| d.name().to_string())
                .unwrap_or_else(|_| "default".into());
            let supported = device
                .default_output_config()
                .map_err(|e| format!("output config: {e}"))?;
            let config = supported.config();
            let channels = usize::from(config.channels);
            let sample_rate = config.sample_rate;
            // 40 ms of latency absorbs the engine thread's frame jitter.
            let buffer_frames = sample_rate / 25;
            let mixer = Arc::new(Mutex::new(Mixer::for_link(
                link,
                sample_rate,
                Clock::Free {
                    latency_frames: buffer_frames,
                },
            )));
            let device_latency = Arc::new(AtomicU64::new(0f64.to_bits()));
            link.attach();
            let err = |e: cpal::StreamError| eprintln!("og-audio: output stream error: {e}");
            let (m, l) = (mixer.clone(), link.clone());
            let (lat_f, lat_i) = (device_latency.clone(), device_latency.clone());
            let mut scratch = Vec::new();
            let stream = match supported.sample_format() {
                cpal::SampleFormat::F32 => device.build_output_stream::<f32, _, _>(
                    &config,
                    move |out, info| {
                        lat_f.store(callback_latency(info).to_bits(), Ordering::Relaxed);
                        fill(out, channels, &m, &l, &mut scratch)
                    },
                    err,
                    None,
                ),
                cpal::SampleFormat::I16 => device.build_output_stream::<i16, _, _>(
                    &config,
                    move |out, info| {
                        lat_i.store(callback_latency(info).to_bits(), Ordering::Relaxed);
                        fill(out, channels, &m, &l, &mut scratch)
                    },
                    err,
                    None,
                ),
                other => return Err(format!("unsupported device sample format {other:?}")),
            }
            .map_err(|e| format!("output stream: {e}"))?;
            stream.play().map_err(|e| format!("play: {e}"))?;
            Ok(DeviceSink {
                _stream: stream,
                mixer,
                link: link.clone(),
                sample_rate,
                name,
                device_latency,
                buffer_frames,
            })
        }

        /// Seconds from a sample's tick to the speaker: the mixer's buffer and the device's latency.
        pub fn latency(&self) -> f64 {
            f64::from(self.buffer_frames) / f64::from(self.sample_rate.max(1))
                + f64::from_bits(self.device_latency.load(Ordering::Relaxed))
        }

        pub fn stats_json(&self) -> String {
            let s = self.mixer.lock().map(|m| m.stats()).unwrap_or_default();
            format!(
                r#"{{"mode":"device","device":{},"sampleRate":{},"frames":{},"commands":{},"voices":{},"voicesPeak":{},"late":{},"reanchors":{},"limited":{},"dropped":{},{}}}"#,
                super::json_str(&self.name),
                self.sample_rate,
                s.frames,
                s.commands,
                s.voices,
                s.voices_peak,
                s.late,
                s.reanchors,
                s.limited,
                self.link.dropped(),
                super::decode_members(self.link.bank().decode_stats())
            )
        }
    }

    impl Drop for DeviceSink {
        fn drop(&mut self) {
            self.link.detach();
        }
    }
}
