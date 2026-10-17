//! The mixer: consumes tick packets and renders interleaved stereo f32 PCM,
//! outside the tick. Nothing here is read by the simulation.
//!
//! - **Time**: the first packet anchors its tick to the mixer's current
//!   frame (plus the latency in `Clock::Free`); tick `t` then starts at
//!   `anchor_frame + (t - anchor_tick) * sample_rate / tick_rate`, so a
//!   command lands on the sample its tick starts at. A driven mixer (the web
//!   Worker, the null and capture sinks) is rendered exactly up to the
//!   world's tick, so commands are never late and the PCM is a function of
//!   the command stream. A free mixer (cpal) is pulled by the device: a
//!   command more than `MAX_LATE` late or `MAX_AHEAD` early re-anchors.
//! - **Voices**: linear-interpolated resampling at `pitch × source rate /
//!   output rate`; a loop wraps at `loop_end` to `loop_start` with the
//!   interpolation reading across the seam, so a sample-continuous loop
//!   region plays without a discontinuity. Gain and pan changes ramp over
//!   `RAMP_FRAMES`; a stop fades over `FADE_FRAMES`.
//! - **Pan** is Web Audio's StereoPannerNode law (equal power for mono,
//!   balance for stereo). Bus gains (master × bus) fold into each voice.
//! - **Limiter**: stereo-linked peak envelope (instant attack, 100 ms
//!   release) through a tanh soft knee at -1.9 dBFS: |output| < 1.0.

use crate::command::{Command, Op, TickPacket};
use crate::schema::BUS_COUNT;
use crate::sound::{Reader, SoundBank};
use crate::system::AudioLink;
use std::collections::VecDeque;

/// Frames a gain or pan change ramps over (5.3 ms at 48 kHz).
pub const RAMP_FRAMES: u32 = 256;
/// Frames a stopped voice fades out over.
pub const FADE_FRAMES: u32 = 256;
/// The limiter's knee: peaks above it are compressed towards 1.0.
pub const LIMITER_KNEE: f32 = 0.8;

/// How the mixer's clock follows the ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clock {
    /// Rendered up to the world's tick by its host (web Worker, null, capture).
    Driven,
    /// Pulled by an audio device; commands land `latency_frames` after they arrive.
    Free { latency_frames: u32 },
}

/// A mixer's configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MixerDesc {
    pub sample_rate: u32,
    pub tick_rate: u32,
    pub clock: Clock,
    /// The output soft limiter (on by default; off only to measure it).
    pub limiter: bool,
}

impl Default for MixerDesc {
    fn default() -> Self {
        MixerDesc {
            sample_rate: 48_000,
            tick_rate: 60,
            clock: Clock::Driven,
            limiter: true,
        }
    }
}

/// What the mixer has done.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MixStats {
    /// Frames rendered (including skipped ones).
    pub frames: u64,
    /// Commands applied.
    pub commands: u64,
    /// Commands that arrived after their time and were applied at once.
    pub late: u64,
    /// Times the clock was re-anchored.
    pub reanchors: u64,
    /// Voices sounding now, and at most.
    pub voices: u32,
    pub voices_peak: u32,
    /// Frames the limiter reduced.
    pub limited: u64,
}

enum Event {
    Command(Command),
    /// Drop voices not in the set (their entities are gone).
    Live(Vec<u32>),
}

struct Timed {
    at: u64,
    event: Event,
}

struct Voice {
    id: u32,
    reader: Reader,
    stereo: bool,
    frames: u64,
    pos: f64,
    /// Source frames per output frame at pitch 1.
    base_step: f64,
    pitch: f32,
    looping: bool,
    loop_start: u64,
    loop_end: u64,
    gain: f32,
    pan: f32,
    bus: usize,
    /// Output matrix (ll, rl, lr, rr): L = ll·inL + rl·inR, R = lr·inL + rr·inR.
    cur: [f32; 4],
    inc: [f32; 4],
    target: [f32; 4],
    ramp: u32,
    stopping: bool,
    done: bool,
}

/// Web Audio's StereoPannerNode gains for `pan`, as a matrix.
fn pan_matrix(pan: f32, stereo: bool, gain: f32) -> [f32; 4] {
    use std::f32::consts::FRAC_PI_2;
    let pan = pan.clamp(-1.0, 1.0);
    if !stereo {
        let x = (pan + 1.0) * 0.5;
        [
            gain * (x * FRAC_PI_2).cos(),
            0.0,
            gain * (x * FRAC_PI_2).sin(),
            0.0,
        ]
    } else if pan <= 0.0 {
        let x = pan + 1.0;
        [
            gain,
            gain * (x * FRAC_PI_2).cos(),
            0.0,
            gain * (x * FRAC_PI_2).sin(),
        ]
    } else {
        let x = pan;
        [
            gain * (x * FRAC_PI_2).cos(),
            0.0,
            gain * (x * FRAC_PI_2).sin(),
            gain,
        ]
    }
}

impl Voice {
    fn bus_gain(&self, buses: &[f32; BUS_COUNT]) -> f32 {
        if self.bus == 0 {
            buses[0]
        } else {
            buses[0] * buses[self.bus]
        }
    }

    fn retarget(&mut self, buses: &[f32; BUS_COUNT], frames: u32) {
        let g = if self.stopping {
            0.0
        } else {
            self.gain * self.bus_gain(buses)
        };
        self.target = pan_matrix(self.pan, self.stereo, g);
        if frames == 0 {
            self.cur = self.target;
            self.ramp = 0;
        } else {
            self.ramp = frames;
            for k in 0..4 {
                self.inc[k] = (self.target[k] - self.cur[k]) / frames as f32;
            }
        }
    }

    fn apply(&mut self, c: &Command) {
        self.gain = c.gain.max(0.0);
        self.pan = c.pan;
        self.pitch = crate::system::clamp_pitch(c.pitch);
        self.bus = usize::from(c.bus).min(BUS_COUNT - 1);
        self.looping = c.looping();
        let end = if c.loop_end == 0 {
            self.frames
        } else {
            u64::from(c.loop_end).min(self.frames)
        };
        self.loop_end = end.max(1);
        self.loop_start = u64::from(c.loop_start).min(self.loop_end - 1);
    }

    /// The frame after `i`, across the loop seam.
    #[inline]
    fn next_index(&self, i: u64) -> u64 {
        if self.looping && i + 1 >= self.loop_end {
            self.loop_start + (i + 1 - self.loop_end)
        } else {
            i + 1
        }
    }

    /// Adds this voice into `out` (interleaved stereo); false once it is done.
    fn render_add(&mut self, out: &mut [f32]) -> bool {
        let step = self.base_step * f64::from(self.pitch);
        let span = (self.loop_end - self.loop_start) as f64;
        for o in out.chunks_exact_mut(2) {
            let i = self.pos as u64;
            let frac = (self.pos - i as f64) as f32;
            let (al, ar) = self.reader.frame(i);
            let (bl, br) = self.reader.frame(self.next_index(i));
            let l = al + (bl - al) * frac;
            let r = ar + (br - ar) * frac;
            if self.ramp > 0 {
                for k in 0..4 {
                    self.cur[k] += self.inc[k];
                }
                self.ramp -= 1;
                if self.ramp == 0 {
                    self.cur = self.target;
                    if self.stopping {
                        self.done = true;
                    }
                }
            }
            let m = self.cur;
            o[0] += m[0] * l + m[1] * r;
            o[1] += m[2] * l + m[3] * r;
            if self.done {
                return false;
            }
            self.pos += step;
            if self.looping {
                let end = self.loop_end as f64;
                if self.pos >= end {
                    self.pos = self.loop_start as f64 + (self.pos - end) % span;
                }
            } else if self.pos >= self.frames as f64 {
                self.done = true;
                return false;
            }
        }
        !self.done
    }
}

struct Limiter {
    env: f32,
    release: f32,
}

impl Limiter {
    fn new(rate: u32) -> Limiter {
        Limiter {
            env: 0.0,
            release: (-1.0 / (0.1 * rate.max(1) as f32)).exp(),
        }
    }

    /// Returns the frames it reduced.
    fn process(&mut self, out: &mut [f32]) -> u64 {
        let mut limited = 0;
        let span = 1.0 - LIMITER_KNEE;
        for o in out.chunks_exact_mut(2) {
            let (mut l, mut r) = (o[0], o[1]);
            if !l.is_finite() {
                l = 0.0;
            }
            if !r.is_finite() {
                r = 0.0;
            }
            let peak = l.abs().max(r.abs());
            self.env = if peak > self.env {
                peak
            } else {
                peak + (self.env - peak) * self.release
            };
            if self.env > LIMITER_KNEE {
                let y = LIMITER_KNEE + span * ((self.env - LIMITER_KNEE) / span).tanh();
                let g = y / self.env;
                l *= g;
                r *= g;
                limited += 1;
            }
            o[0] = l.clamp(-1.0, 1.0);
            o[1] = r.clamp(-1.0, 1.0);
        }
        limited
    }
}

/// The audio mixer of one engine.
pub struct Mixer {
    desc: MixerDesc,
    bank: SoundBank,
    frame: u64,
    anchor: Option<(u64, u64)>,
    pending: VecDeque<Timed>,
    voices: Vec<Voice>,
    buses: [f32; BUS_COUNT],
    limiter: Limiter,
    stats: MixStats,
    scratch: Vec<f32>,
}

/// A command this late re-anchors a free clock (100 ms).
const MAX_LATE_MS: u64 = 100;
/// A command this early re-anchors a free clock (500 ms).
const MAX_AHEAD_MS: u64 = 500;

impl Mixer {
    pub fn new(desc: MixerDesc, bank: SoundBank) -> Mixer {
        let desc = MixerDesc {
            sample_rate: desc.sample_rate.max(1),
            tick_rate: desc.tick_rate.max(1),
            ..desc
        };
        Mixer {
            desc,
            bank,
            frame: 0,
            anchor: None,
            pending: VecDeque::new(),
            voices: Vec::new(),
            buses: [1.0; BUS_COUNT],
            limiter: Limiter::new(desc.sample_rate),
            stats: MixStats::default(),
            scratch: Vec::new(),
        }
    }

    /// A driven mixer for `link`'s world and bank.
    pub fn for_link(link: &AudioLink, sample_rate: u32, clock: Clock) -> Mixer {
        Mixer::new(
            MixerDesc {
                sample_rate,
                tick_rate: link.tick_rate(),
                clock,
                limiter: true,
            },
            link.bank().clone(),
        )
    }

    pub fn desc(&self) -> MixerDesc {
        self.desc
    }

    /// Frames rendered so far.
    pub fn frame(&self) -> u64 {
        self.frame
    }

    pub fn stats(&self) -> MixStats {
        MixStats {
            voices: self.voices.len() as u32,
            ..self.stats
        }
    }

    /// The frame tick `tick` starts at; `None` before the first packet.
    pub fn frame_of_tick(&self, tick: u64) -> Option<u64> {
        let (t0, f0) = self.anchor?;
        let dt = tick.saturating_sub(t0) as u128;
        Some(f0 + (dt * u128::from(self.desc.sample_rate) / u128::from(self.desc.tick_rate)) as u64)
    }

    fn ms(&self, ms: u64) -> u64 {
        ms * u64::from(self.desc.sample_rate) / 1000
    }

    fn insert(&mut self, at: u64, event: Event) {
        let i = self.pending.partition_point(|t| t.at <= at);
        self.pending.insert(i, Timed { at, event });
    }

    /// Queues a packet's commands at their tick's frame.
    pub fn push(&mut self, packet: TickPacket) {
        let latency = match self.desc.clock {
            Clock::Driven => 0,
            Clock::Free { latency_frames } => u64::from(latency_frames),
        };
        if self.anchor.is_none() {
            self.anchor = Some((packet.tick, self.frame + latency));
        }
        let mut at = self.frame_of_tick(packet.tick).unwrap_or(self.frame);
        if let Clock::Free { .. } = self.desc.clock {
            let late = self.frame.saturating_sub(at) > self.ms(MAX_LATE_MS);
            let early = at.saturating_sub(self.frame) > self.ms(MAX_AHEAD_MS) + latency;
            if late || early || packet.tick < self.anchor.map_or(0, |a| a.0) {
                self.anchor = Some((packet.tick, self.frame + latency));
                self.stats.reanchors += 1;
                at = self.frame + latency;
            }
        }
        if at < self.frame {
            self.stats.late += packet.commands.len() as u64;
            at = self.frame;
        }
        let end = self.frame_of_tick(packet.tick + 1).unwrap_or(at).max(at);
        for c in packet.commands {
            self.insert(at, Event::Command(c));
        }
        if let Some(live) = packet.live {
            // At the tick's end, so a one-shot that ended in it plays its tail.
            self.insert(end, Event::Live(live));
        }
    }

    /// Moves every packet the link queued into the mixer.
    pub fn pump(&mut self, link: &AudioLink) {
        for p in link.drain() {
            self.push(p);
        }
    }

    fn apply(&mut self, event: Event) {
        match event {
            Event::Live(live) => {
                for v in &mut self.voices {
                    if !v.stopping && live.binary_search(&v.id).is_err() {
                        v.stopping = true;
                        v.retarget(&self.buses, FADE_FRAMES);
                    }
                }
            }
            Event::Command(c) => {
                self.stats.commands += 1;
                match c.op {
                    Op::Play => self.play(&c),
                    Op::Stop => {
                        for v in self.voices.iter_mut().filter(|v| v.id == c.voice) {
                            v.stopping = true;
                            v.retarget(&self.buses, FADE_FRAMES);
                        }
                    }
                    Op::Params => {
                        let buses = self.buses;
                        for v in self
                            .voices
                            .iter_mut()
                            .filter(|v| v.id == c.voice && !v.stopping)
                        {
                            v.apply(&c);
                            v.retarget(&buses, RAMP_FRAMES);
                        }
                    }
                    Op::Bus => {
                        let b = usize::from(c.bus).min(BUS_COUNT - 1);
                        self.buses[b] = c.gain.max(0.0);
                        let buses = self.buses;
                        for v in &mut self.voices {
                            v.retarget(&buses, RAMP_FRAMES);
                        }
                    }
                    Op::StopAll => {
                        let buses = self.buses;
                        for v in &mut self.voices {
                            if !v.stopping {
                                v.stopping = true;
                                v.retarget(&buses, FADE_FRAMES);
                            }
                        }
                    }
                }
            }
        }
    }

    fn play(&mut self, c: &Command) {
        let Some(data) = self.bank.get(c.sound) else {
            return;
        };
        // A play for a voice id that is still sounding replaces it.
        let buses = self.buses;
        for v in self
            .voices
            .iter_mut()
            .filter(|v| v.id == c.voice && !v.stopping)
        {
            v.stopping = true;
            v.retarget(&buses, FADE_FRAMES);
        }
        let frames = data.frames;
        if frames == 0 {
            return;
        }
        let mut v = Voice {
            id: c.voice,
            stereo: data.channels > 1,
            frames,
            pos: c.position.max(0.0),
            base_step: f64::from(data.sample_rate) / f64::from(self.desc.sample_rate),
            reader: Reader::new(data),
            pitch: 1.0,
            looping: false,
            loop_start: 0,
            loop_end: frames,
            gain: 1.0,
            pan: 0.0,
            bus: 0,
            cur: [0.0; 4],
            inc: [0.0; 4],
            target: [0.0; 4],
            ramp: 0,
            stopping: false,
            done: false,
        };
        v.apply(c);
        if v.looping && v.pos >= v.loop_end as f64 {
            let span = (v.loop_end - v.loop_start) as f64;
            v.pos = v.loop_start as f64 + (v.pos - v.loop_start as f64) % span;
        }
        // A voice resumed mid-sound (a resync) fades in; a fresh one starts
        // at full gain so its attack is kept.
        v.retarget(&self.buses, if c.position > 0.0 { RAMP_FRAMES } else { 0 });
        self.voices.push(v);
        self.stats.voices_peak = self.stats.voices_peak.max(self.voices.len() as u32);
    }

    fn mix(&mut self, out: &mut [f32]) {
        out.fill(0.0);
        self.voices.retain_mut(|v| v.render_add(out));
        if self.desc.limiter {
            self.stats.limited += self.limiter.process(out);
        }
    }

    /// Renders `out.len() / 2` stereo frames, applying commands at their frames.
    pub fn render(&mut self, out: &mut [f32]) {
        let n = out.len() / 2;
        let mut done = 0;
        while done < n {
            while self.pending.front().is_some_and(|t| t.at <= self.frame) {
                let t = self.pending.pop_front().expect("front exists");
                self.apply(t.event);
            }
            let until = self.pending.front().map_or(u64::MAX, |t| t.at);
            let seg = ((until - self.frame) as usize).min(n - done);
            self.mix(&mut out[done * 2..(done + seg) * 2]);
            done += seg;
            self.frame += seg as u64;
            self.stats.frames += seg as u64;
        }
    }

    /// Frames between the mixer's clock and the start of `tick` (0 when
    /// the mixer is ahead or has no anchor yet).
    pub fn frames_until_tick(&self, tick: u64) -> u64 {
        self.frame_of_tick(tick)
            .map_or(0, |f| f.saturating_sub(self.frame))
    }

    /// Renders up to the start of `tick` (a driven mixer: the world's next
    /// tick), appending to `out`, at most `max_frames` frames.
    pub fn render_until_tick(&mut self, tick: u64, max_frames: usize, out: &mut Vec<f32>) -> usize {
        let n = (self.frames_until_tick(tick) as usize).min(max_frames);
        let start = out.len();
        out.resize(start + n * 2, 0.0);
        self.render(&mut out[start..]);
        n
    }

    /// Renders `frames` and drops them (catching up a clock that fell behind).
    pub fn skip(&mut self, frames: usize) {
        let mut scratch = std::mem::take(&mut self.scratch);
        let mut left = frames;
        while left > 0 {
            let n = left.min(1024);
            scratch.resize(n * 2, 0.0);
            self.render(&mut scratch[..n * 2]);
            left -= n;
        }
        self.scratch = scratch;
    }

    /// Voices sounding now.
    pub fn voice_count(&self) -> usize {
        self.voices.len()
    }
}
