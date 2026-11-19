//! The default output device through cpal (feature `device`), the sink every
//! native host opens. The callback owns the render: it drains the link,
//! pushes the packets into a free-clock mixer and renders the device's
//! buffer.

use crate::{AudioLink, Clock, MixStats, Mixer};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
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

    /// The mixer's counters.
    pub fn stats(&self) -> MixStats {
        self.mixer.lock().map(|m| m.stats()).unwrap_or_default()
    }

    /// The device's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// The link it plays.
    pub fn link(&self) -> &AudioLink {
        &self.link
    }
}

impl Drop for DeviceSink {
    fn drop(&mut self) {
        self.link.detach();
    }
}
