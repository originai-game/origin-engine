//! Headless sinks: `null` consumes the command stream and renders nothing;
//! `capture` renders it with a driven mixer into memory (a WAV for
//! `og run`, evidence and tests). Both follow the world's tick, so a
//! headless run's audio is a function of its command stream.

use crate::command::{Op, TickPacket};
use crate::mixer::{Clock, MixStats, Mixer};
use crate::system::AudioLink;
use og_core::hash::StableHasher;

/// What a headless sink does with the stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadlessMode {
    /// Count and hash the packets; no PCM.
    Null,
    /// Render PCM at this rate into memory.
    Capture { sample_rate: u32 },
}

/// Counters a headless sink keeps.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SinkStats {
    pub packets: u64,
    pub commands: u64,
    pub plays: u64,
    pub resyncs: u64,
    /// FNV-1a over every packet the sink received (resyncs included).
    pub received_hash: u64,
    pub mix: MixStats,
}

/// A sink that keeps up with the world without an audio device.
pub struct HeadlessSink {
    link: AudioLink,
    mixer: Option<Mixer>,
    pcm: Vec<f32>,
    stats: SinkStats,
}

impl HeadlessSink {
    /// Attaches to `link` (so the next tick resyncs).
    pub fn new(link: &AudioLink, mode: HeadlessMode) -> HeadlessSink {
        link.attach();
        let mixer = match mode {
            HeadlessMode::Null => None,
            HeadlessMode::Capture { sample_rate } => {
                Some(Mixer::for_link(link, sample_rate, Clock::Driven))
            }
        };
        HeadlessSink {
            link: link.clone(),
            mixer,
            pcm: Vec::new(),
            stats: SinkStats::default(),
        }
    }

    fn note(&mut self, p: &TickPacket) {
        let mut h = StableHasher::new();
        h.write_u64(self.stats.received_hash);
        h.write_u64(p.tick);
        h.write(&crate::command::encode(&p.commands));
        if let Some(live) = &p.live {
            for v in live {
                h.write_u32(*v);
            }
        }
        self.stats.received_hash = h.finish();
        self.stats.packets += 1;
        self.stats.commands += p.commands.len() as u64;
        self.stats.plays += p.commands.iter().filter(|c| c.op == Op::Play).count() as u64;
        self.stats.resyncs += u64::from(p.resync);
    }

    /// Drains the link and (capture) renders up to the world's next tick.
    /// Call after every `World::step` (or after each frame's ticks).
    pub fn pump(&mut self) {
        for p in self.link.drain() {
            self.note(&p);
            if let Some(m) = &mut self.mixer {
                m.push(p);
            }
        }
        if let Some(m) = &mut self.mixer {
            m.render_until_tick(self.link.next_tick(), usize::MAX, &mut self.pcm);
        }
    }

    pub fn stats(&self) -> SinkStats {
        SinkStats {
            mix: self.mixer.as_ref().map(Mixer::stats).unwrap_or_default(),
            ..self.stats
        }
    }

    /// The captured interleaved stereo PCM so far.
    pub fn pcm(&self) -> &[f32] {
        &self.pcm
    }

    /// Takes the captured PCM.
    pub fn take_pcm(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.pcm)
    }

    /// The capture rate, or `None` for a null sink.
    pub fn sample_rate(&self) -> Option<u32> {
        self.mixer.as_ref().map(|m| m.desc().sample_rate)
    }

    pub fn mixer(&mut self) -> Option<&mut Mixer> {
        self.mixer.as_mut()
    }
}

impl Drop for HeadlessSink {
    fn drop(&mut self) {
        self.link.detach();
    }
}

/// A 16-bit PCM WAV of interleaved f32 samples (`channels` per frame).
pub fn wav(pcm: &[f32], channels: u16, sample_rate: u32) -> Vec<u8> {
    let data = (pcm.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&(sample_rate * u32::from(channels) * 2).to_le_bytes());
    out.extend_from_slice(&(channels * 2).to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data.to_le_bytes());
    for s in pcm {
        let v = (s.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}
