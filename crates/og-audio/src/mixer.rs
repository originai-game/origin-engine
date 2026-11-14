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
//! - **Voices**: resampling at `pitch × source rate / output rate`, by a
//!   Kaiser-windowed sinc (`resample.rs`; `og.Audio.resampler` sinc, the
//!   default) or by linear interpolation (`linear`, the engine's output
//!   before round 4); a loop wraps at `loop_end` to `loop_start` with the
//!   interpolation reading across the seam, so a sample-continuous loop
//!   region plays without a discontinuity. Gain and pan changes ramp over
//!   `RAMP_FRAMES`; a stop fades over `FADE_FRAMES`.
//! - **Pan** is Web Audio's StereoPannerNode law (equal power for mono,
//!   balance for stereo). Bus gains (master × bus) fold into each voice.
//!   A voice's low-pass (distance, occlusion, `lowpass`) is a one-pole
//!   whose cutoff glides with its gain ramp. A play at a negative position
//!   starts that many source frames later (a `startAt` inside a tick); a
//!   `cue` after it starts it that far into the sound (`startOffset`), at
//!   full gain from its first sample.
//! - **Buses** form a graph: the built-ins (master, music, sfx, ui) and
//!   user buses 4 to 15, each playing into its parent (a built-in or a
//!   user bus of a lower index; music, sfx and ui into master). A bus's
//!   gain multiplies down the chain into each voice's gain; a `bus`
//!   command with a negative position ramps it linearly across its tick
//!   (a snapshot's blend, straight to the sample). With any insert on,
//!   voices sum into their bus; user buses run their inserts from the
//!   highest index down, each summing into its parent; then music, sfx
//!   and ui (ducking keyed by a built-in bus's level), then master. A
//!   bus's inserts: high-pass, low-pass, a three-band equaliser, the
//!   waveshaper, the compressor, the echo, then ducking; its reverb send
//!   feeds the shared reverb, whose wet output joins master.
//! - **Limiter**: stereo-linked peak envelope (instant attack, 100 ms
//!   release) through a tanh soft knee at -1.9 dBFS: |output| < 1.0.

use crate::command::{Command, FLAG_PRESENTATION, Op, TickPacket, fx};
use crate::dsp::{Band, Biquad, Compressor, Delay, Drive, Ducker, Hrtf, OnePole, Reverb};
use crate::listen::Listen;
use crate::resample::{History, Region};
use crate::schema::{BUS_COUNT, MAX_BUSES};
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
    presentation: bool,
    reader: Reader,
    stereo: bool,
    frames: u64,
    pos: f64,
    /// Where in the sound it starts once its delay is over (a `cue`).
    cue: f64,
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
    lp: OnePole,
    /// The HRTF panner, while the voice has a direction (`hrtf` commands);
    /// `None` is the stereo panner.
    hrtf: Option<Box<Hrtf>>,
    /// The windowed-sinc resampler's frames around the playhead (made on
    /// the first frame it resamples).
    hist: Option<History>,
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
    fn matches(&self, command: &Command) -> bool {
        self.id == command.voice && self.presentation == (command.flags & FLAG_PRESENTATION != 0)
    }

    /// `chain`: each bus's gain times its parents' ([`Mixer::chain`]).
    fn retarget(&mut self, chain: &[f32; MAX_BUSES], frames: u32) {
        let g = if self.stopping {
            0.0
        } else {
            self.gain * chain[self.bus]
        };
        self.target = if self.hrtf.is_some() {
            // The HRTF makes both ears; the matrix only carries the gain.
            [g, 0.0, 0.0, g]
        } else {
            pan_matrix(self.pan, self.stereo, g)
        };
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
        self.bus = usize::from(c.bus).min(MAX_BUSES - 1);
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

    /// Moves the playhead to its cue (once), wrapping a loop into its region.
    fn take_cue(&mut self) {
        if self.cue <= 0.0 {
            return;
        }
        self.pos += self.cue;
        self.cue = 0.0;
        self.wrap();
        self.jumped();
    }

    /// The playhead moved other than by playing: the resampler's history
    /// starts again around it.
    fn jumped(&mut self) {
        if let Some(h) = &mut self.hist {
            h.invalidate();
        }
    }

    /// A looping playhead past its loop's end goes back into the region.
    fn wrap(&mut self) {
        if self.looping && self.pos >= self.loop_end as f64 && self.loop_end > self.loop_start {
            let span = (self.loop_end - self.loop_start) as f64;
            self.pos = self.loop_start as f64 + (self.pos - self.loop_start as f64) % span;
        }
    }

    /// Adds this voice into `out` (interleaved stereo), resampling by the
    /// windowed sinc (`sinc`) or linearly; false once it is done.
    fn render_add(&mut self, out: &mut [f32], sinc: bool) -> bool {
        let step = self.base_step * f64::from(self.pitch);
        let frames = out.len() / 2;
        let mut f = 0;
        while f < frames {
            if self.pos < 0.0 {
                if self.pos > -1e-6 {
                    // Rounding of a start time: this is its frame.
                    self.pos = 0.0;
                    self.take_cue();
                } else {
                    // Scheduled later in the tick: silent, ramps held, until it starts.
                    self.pos += step;
                    if self.stopping {
                        self.done = true;
                        return false;
                    }
                    if self.pos >= 0.0 {
                        // The delay is over inside this frame: the sound starts at its cue.
                        self.take_cue();
                        if !self.looping && self.pos >= self.frames as f64 {
                            self.done = true;
                            return false;
                        }
                    }
                    f += 1;
                    continue;
                }
            }
            let (played, sounding) = self.play_add(&mut out[f * 2..], step, sinc);
            if !sounding {
                return false;
            }
            f += played;
        }
        !self.done
    }

    /// Plays up to [`RUN`] frames from the playhead into `out`: their
    /// source positions, their source frames, then the low-pass, the
    /// panner and the gain ramp on each. The frames played, and false
    /// once the voice is done.
    fn play_add(&mut self, out: &mut [f32], step: f64, sinc: bool) -> (usize, bool) {
        let mut limit = (out.len() / 2).min(RUN);
        if self.done {
            // Ended by a cue: this frame, then gone.
            limit = 1;
        } else if self.stopping && self.ramp > 0 {
            // Its fade ends on this frame.
            limit = limit.min(self.ramp as usize);
        }
        let mut at = [(0u64, 0.0f32); RUN];
        let (mut n, mut ended) = (0, false);
        let span = (self.loop_end - self.loop_start) as f64;
        while n < limit {
            // Through i64 (one instruction each way; the playhead is ≥ 0).
            let i = self.pos as i64;
            at[n] = (i as u64, (self.pos - i as f64) as f32);
            n += 1;
            self.pos += step;
            if self.looping {
                let end = self.loop_end as f64;
                if self.pos >= end {
                    self.pos = self.loop_start as f64 + (self.pos - end) % span;
                }
            } else if self.pos >= self.frames as f64 {
                ended = true;
                break;
            }
        }
        let at = &at[..n];
        let mut input = [(0.0f32, 0.0f32); RUN];
        let input = &mut input[..n];
        if sinc {
            let region = Region {
                looping: self.looping,
                start: self.loop_start,
                end: self.loop_end,
            };
            self.hist.get_or_insert_with(History::new).render(
                &mut self.reader,
                region,
                at,
                step,
                input,
            );
        } else {
            for (x, &(i, frac)) in input.iter_mut().zip(at) {
                let (al, ar) = self.reader.frame(i);
                let (bl, br) = self.reader.frame(self.next_index(i));
                *x = (al + (bl - al) * frac, ar + (br - ar) * frac);
            }
        }
        let frames = out.as_chunks_mut::<2>().0.iter_mut().zip(&*input);
        if !self.lp.on() && self.hrtf.is_none() && self.ramp == 0 {
            // Nothing between the source and the matrix, which holds still.
            let m = self.cur;
            for (o, &(l, r)) in frames {
                o[0] += m[0] * l + m[1] * r;
                o[1] += m[2] * l + m[3] * r;
            }
            if ended {
                self.done = true;
            }
            return (n, !self.done);
        }
        for (o, &(il, ir)) in frames {
            let (mut l, mut r) = self.lp.process(il, ir);
            if let Some(h) = &mut self.hrtf {
                (l, r) = h.process(if self.stereo { 0.5 * (l + r) } else { l });
            }
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
        }
        if ended {
            self.done = true;
        }
        (n, !self.done)
    }
}

/// Frames a voice plays in one run of [`Voice::play_add`].
const RUN: usize = 64;

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
        for o in out.as_chunks_mut::<2>().0.iter_mut() {
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
    buses: [f32; MAX_BUSES],
    /// Each bus's parent (master's is itself; a parent is always lower).
    parents: [usize; MAX_BUSES],
    limiter: Limiter,
    stats: MixStats,
    scratch: Vec<f32>,
    chains: [BusChain; MAX_BUSES],
    /// Per-bus scratch (interleaved stereo), then the reverb's input.
    bus_out: [Vec<f32>; MAX_BUSES],
    send: Vec<f32>,
    reverb: Option<Reverb>,
    room: f32,
    damping: f32,
    /// Per-frame peaks of the buses that key a ducker.
    keys: [Vec<f32>; BUS_COUNT],
    listen: Option<Listen>,
    /// Voices resample by the windowed sinc (`og.Audio.resampler` sinc, the
    /// default), else linearly.
    sinc: bool,
}

/// One bus's inserts and the parameters they were built from.
#[derive(Clone, Debug)]
struct BusChain {
    params: [f32; fx::COUNT],
    lp: Biquad,
    hp: Biquad,
    eq: [Biquad; 3],
    drive: Drive,
    comp: Compressor,
    delay: Delay,
    duck: Ducker,
    reverb: f32,
    /// Any insert is on (else the bus passes straight through).
    active: bool,
}

impl Default for BusChain {
    fn default() -> Self {
        let mut params = [0.0; fx::COUNT];
        params[fx::RATIO as usize] = 4.0;
        params[fx::EQ_LOW_FREQ as usize] = 200.0;
        params[fx::EQ_MID_FREQ as usize] = 1000.0;
        params[fx::EQ_MID_Q as usize] = 0.7;
        params[fx::EQ_HIGH_FREQ as usize] = 5000.0;
        params[fx::DELAY_FEEDBACK as usize] = 0.35;
        params[fx::DELAY_MIX as usize] = 0.35;
        params[fx::DISTORTION_MIX as usize] = 1.0;
        BusChain {
            params,
            lp: Biquad::default(),
            hp: Biquad::default(),
            eq: [Biquad::default(); 3],
            drive: Drive::default(),
            comp: Compressor::default(),
            delay: Delay::default(),
            duck: Ducker::default(),
            reverb: 0.0,
            active: false,
        }
    }
}

impl BusChain {
    fn set(&mut self, param: u32, value: f32, rate: f32) {
        let Some(slot) = self.params.get_mut(param as usize) else {
            return;
        };
        *slot = value;
        let p = self.params;
        let at = |k: u32| p[k as usize];
        match param {
            fx::LOWPASS => self.lp.set(at(fx::LOWPASS), false, rate),
            fx::HIGHPASS => self.hp.set(at(fx::HIGHPASS), true, rate),
            fx::REVERB => self.reverb = at(fx::REVERB).clamp(0.0, 1.0),
            fx::EQ_LOW | fx::EQ_LOW_FREQ => self.eq[0].set_band(
                Band::LowShelf,
                at(fx::EQ_LOW_FREQ),
                at(fx::EQ_LOW),
                0.7,
                rate,
            ),
            fx::EQ_MID | fx::EQ_MID_FREQ | fx::EQ_MID_Q => self.eq[1].set_band(
                Band::Peak,
                at(fx::EQ_MID_FREQ),
                at(fx::EQ_MID),
                at(fx::EQ_MID_Q),
                rate,
            ),
            fx::EQ_HIGH | fx::EQ_HIGH_FREQ => self.eq[2].set_band(
                Band::HighShelf,
                at(fx::EQ_HIGH_FREQ),
                at(fx::EQ_HIGH),
                0.7,
                rate,
            ),
            fx::DELAY | fx::DELAY_FEEDBACK | fx::DELAY_MIX => self.delay.set(
                at(fx::DELAY),
                at(fx::DELAY_FEEDBACK),
                at(fx::DELAY_MIX),
                rate,
            ),
            fx::DISTORTION | fx::DISTORTION_MIX => {
                self.drive.set(at(fx::DISTORTION), at(fx::DISTORTION_MIX))
            }
            fx::THRESHOLD | fx::RATIO | fx::ATTACK | fx::RELEASE | fx::MAKEUP => self.comp.set(
                at(fx::THRESHOLD),
                at(fx::RATIO),
                at(fx::ATTACK),
                at(fx::RELEASE),
                at(fx::MAKEUP),
                rate,
            ),
            fx::DUCK_KEY
            | fx::DUCK_DEPTH
            | fx::DUCK_THRESHOLD
            | fx::DUCK_ATTACK
            | fx::DUCK_RELEASE => {
                let key = at(fx::DUCK_KEY) as usize;
                self.duck.set(
                    (1..=BUS_COUNT).contains(&key).then(|| key - 1),
                    at(fx::DUCK_THRESHOLD),
                    at(fx::DUCK_DEPTH),
                    at(fx::DUCK_ATTACK),
                    at(fx::DUCK_RELEASE),
                    rate,
                );
            }
            _ => {}
        }
        self.active = at(fx::LOWPASS) > 0.0
            || at(fx::HIGHPASS) > 0.0
            || at(fx::THRESHOLD) < 0.0
            || at(fx::MAKEUP) != 0.0
            || self.duck.key.is_some()
            || self.eq.iter().any(Biquad::on)
            || self.drive.on()
            || self.delay.on();
    }

    /// The inserts on one frame, but ducking (high-pass, low-pass, the
    /// equaliser, the waveshaper, the compressor, the echo).
    #[inline]
    fn process(&mut self, mut l: f32, mut r: f32) -> (f32, f32) {
        (l, r) = self.hp.process(l, r);
        (l, r) = self.lp.process(l, r);
        for b in &mut self.eq {
            (l, r) = b.process(l, r);
        }
        (l, r) = self.drive.process(l, r);
        (l, r) = self.comp.process(l, r);
        self.delay.process(l, r)
    }
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
            buses: [1.0; MAX_BUSES],
            parents: [0; MAX_BUSES],
            limiter: Limiter::new(desc.sample_rate),
            stats: MixStats::default(),
            scratch: Vec::new(),
            chains: std::array::from_fn(|_| BusChain::default()),
            bus_out: Default::default(),
            send: Vec::new(),
            reverb: None,
            room: 0.6,
            damping: 0.4,
            keys: Default::default(),
            listen: None,
            sinc: true,
        }
    }

    /// Records every rendered frame into `listen` (the probe's `listen`).
    pub fn record_into(&mut self, listen: Listen) {
        self.listen = Some(listen);
    }

    /// A bus's compressor gain reduction and ducking gain now (dB, linear).
    pub fn bus_dynamics(&self, bus: usize) -> (f32, f32) {
        let c = &self.chains[bus.min(MAX_BUSES - 1)];
        (c.comp.reduction, c.duck.current())
    }

    /// Each bus's gain times its parents', down to master: what a voice on
    /// it is scaled by.
    fn chain(&self) -> [f32; MAX_BUSES] {
        let mut c = [0.0; MAX_BUSES];
        c[0] = self.buses[0];
        for b in 1..MAX_BUSES {
            c[b] = self.buses[b] * c[self.parents[b]];
        }
        c
    }

    /// The parent bus of each bus now (the graph the `fx` parent commands built).
    pub fn bus_parents(&self) -> [usize; MAX_BUSES] {
        self.parents
    }

    /// A driven mixer for `link`'s world and bank.
    pub fn for_link(link: &AudioLink, sample_rate: u32, clock: Clock) -> Mixer {
        let mut m = Mixer::new(
            MixerDesc {
                sample_rate,
                tick_rate: link.tick_rate(),
                clock,
                limiter: true,
            },
            link.bank().clone(),
        );
        m.record_into(link.listen().clone());
        m
    }

    fn ramp_frames(&self, seconds: f64, default: u32) -> u32 {
        if seconds > 0.0 {
            ((seconds * f64::from(self.desc.sample_rate)) as u32).max(1)
        } else {
            default
        }
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
                let chain = self.chain();
                for v in &mut self.voices {
                    if !v.presentation && !v.stopping && live.binary_search(&v.id).is_err() {
                        v.stopping = true;
                        v.retarget(&chain, FADE_FRAMES);
                    }
                }
            }
            Event::Command(c) => {
                self.stats.commands += 1;
                let rate = self.desc.sample_rate as f32;
                match c.op {
                    Op::Play => self.play(&c),
                    Op::Stop => {
                        let fade = self.ramp_frames(c.position, FADE_FRAMES);
                        let chain = self.chain();
                        for v in self.voices.iter_mut().filter(|v| v.matches(&c)) {
                            v.stopping = true;
                            v.retarget(&chain, fade);
                        }
                    }
                    Op::Params => {
                        let buses = self.chain();
                        let ramp = self.ramp_frames(c.position, RAMP_FRAMES);
                        for v in self
                            .voices
                            .iter_mut()
                            .filter(|v| v.matches(&c) && !v.stopping)
                        {
                            v.apply(&c);
                            v.lp.set(c.lowpass, rate, ramp);
                            v.retarget(&buses, ramp);
                        }
                    }
                    Op::Bus => {
                        let b = usize::from(c.bus).min(MAX_BUSES - 1);
                        self.buses[b] = c.gain.max(0.0);
                        // Negative: linear across the tick (a snapshot's blend).
                        let ramp = if c.position < 0.0 {
                            let at = |t| self.frame_of_tick(t).unwrap_or(self.frame);
                            (at(c.tick + 1).saturating_sub(at(c.tick)) as u32).max(1)
                        } else {
                            self.ramp_frames(c.position, RAMP_FRAMES)
                        };
                        let chain = self.chain();
                        for v in &mut self.voices {
                            // A longer fade in progress (a crossfade) keeps its length.
                            v.retarget(&chain, ramp.max(v.ramp));
                        }
                    }
                    Op::Fx => {
                        let b = usize::from(c.bus).min(MAX_BUSES - 1);
                        match c.voice {
                            fx::PARENT => {
                                let p = c.gain as usize;
                                self.parents[b] = if b >= BUS_COUNT && p < b { p } else { 0 };
                                let chain = self.chain();
                                for v in &mut self.voices {
                                    v.retarget(&chain, RAMP_FRAMES.max(v.ramp));
                                }
                            }
                            fx::ROOM => self.room = c.gain,
                            fx::DAMPING => self.damping = c.gain,
                            fx::RESAMPLER => {
                                let sinc = c.gain >= 0.5;
                                if sinc != self.sinc {
                                    self.sinc = sinc;
                                    for v in &mut self.voices {
                                        v.hist = None;
                                    }
                                }
                            }
                            k => self.chains[b].set(k, c.gain, rate),
                        }
                        if let Some(r) = &mut self.reverb {
                            r.set(self.room, self.damping);
                        } else if self.chains.iter().any(|ch| ch.reverb > 0.0) {
                            let mut r = Reverb::new(self.desc.sample_rate);
                            r.set(self.room, self.damping);
                            self.reverb = Some(r);
                        }
                    }
                    Op::Cue => {
                        for v in self
                            .voices
                            .iter_mut()
                            .filter(|v| v.matches(&c) && !v.stopping)
                        {
                            let at = c.position.max(0.0);
                            if v.pos < 0.0 {
                                v.cue = at;
                            } else {
                                v.pos += at;
                                v.wrap();
                                v.jumped();
                                if !v.looping && v.pos >= v.frames as f64 {
                                    v.done = true;
                                }
                            }
                        }
                    }
                    Op::Hrtf => {
                        let buses = self.chain();
                        let dir = [c.gain, c.pitch, c.pan];
                        let off = dir.iter().all(|x| *x == 0.0 || !x.is_finite());
                        for v in self
                            .voices
                            .iter_mut()
                            .filter(|v| v.matches(&c) && !v.stopping)
                        {
                            match (&mut v.hrtf, off) {
                                (Some(_), true) => v.hrtf = None,
                                (Some(h), false) => {
                                    h.set(dir, RAMP_FRAMES);
                                    continue;
                                }
                                (None, true) => continue,
                                (None, false) => {
                                    let mut h = Hrtf::new(rate);
                                    h.set(dir, 0);
                                    v.hrtf = Some(Box::new(h));
                                }
                            }
                            // The matrix follows the panner, finishing a
                            // ramp in progress (a resync's fade-in).
                            let ramp = v.ramp;
                            v.retarget(&buses, ramp);
                        }
                    }
                    Op::StopAll => {
                        let buses = self.chain();
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
        let buses = self.chain();
        for v in self
            .voices
            .iter_mut()
            .filter(|v| v.matches(c) && !v.stopping)
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
            presentation: c.flags & FLAG_PRESENTATION != 0,
            stereo: data.channels > 1,
            frames,
            // Negative: it starts that many source frames later.
            pos: c.position,
            cue: 0.0,
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
            lp: OnePole::default(),
            hrtf: None,
            hist: None,
        };
        v.apply(c);
        v.lp.set(c.lowpass, self.desc.sample_rate as f32, 0);
        if v.looping && v.pos >= v.loop_end as f64 {
            let span = (v.loop_end - v.loop_start) as f64;
            v.pos = v.loop_start as f64 + (v.pos - v.loop_start as f64) % span;
        }
        // A voice resumed mid-sound (a resync) fades in; a fresh one starts
        // at full gain so its attack is kept.
        v.retarget(&buses, if c.position > 0.0 { RAMP_FRAMES } else { 0 });
        self.voices.push(v);
        self.stats.voices_peak = self.stats.voices_peak.max(self.voices.len() as u32);
    }

    fn mix(&mut self, out: &mut [f32]) {
        out.fill(0.0);
        let routed = self.reverb.is_some() || self.chains.iter().any(|c| c.active);
        if !routed {
            // No inserts anywhere: every voice straight into the output.
            let sinc = self.sinc;
            self.voices.retain_mut(|v| v.render_add(out, sinc));
        } else {
            self.mix_buses(out);
        }
        if self.desc.limiter {
            self.stats.limited += self.limiter.process(out);
        }
        if let Some(l) = &self.listen {
            l.record(out, self.desc.sample_rate);
        }
    }

    /// Voices into their buses; each bus's inserts, ducking and reverb send;
    /// the sum, the reverb's return and master's inserts into `out`.
    fn mix_buses(&mut self, out: &mut [f32]) {
        let n = out.len();
        let mut buf = std::mem::take(&mut self.bus_out);
        for b in &mut buf {
            b.clear();
            b.resize(n, 0.0);
        }
        let sinc = self.sinc;
        self.voices
            .retain_mut(|v| v.render_add(&mut buf[v.bus], sinc));
        self.send.clear();
        self.send.resize(n, 0.0);
        // User buses, highest first: each into its parent (always lower).
        for b in (BUS_COUNT..MAX_BUSES).rev() {
            let chain = &mut self.chains[b];
            let send = chain.reverb;
            if chain.active {
                for f in buf[b].as_chunks_mut::<2>().0 {
                    (f[0], f[1]) = chain.process(f[0], f[1]);
                }
            }
            if send > 0.0 {
                for (s, x) in self.send.iter_mut().zip(&buf[b]) {
                    *s += x * send;
                }
            }
            let (low, high) = buf.split_at_mut(b);
            for (o, x) in low[self.parents[b]].iter_mut().zip(&high[0]) {
                *o += x;
            }
        }
        // The keys' levels (built-in buses), before their inserts touch them.
        for (k, peaks) in self.keys.iter_mut().enumerate() {
            peaks.clear();
            if self.chains.iter().any(|c| c.duck.key == Some(k)) {
                peaks.extend(
                    buf[k]
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .map(|f| f[0].abs().max(f[1].abs())),
                );
            }
        }
        for b in [1, 2, 3, 0] {
            let chain = &mut self.chains[b];
            let key = chain.duck.key.map(|k| &self.keys[k]);
            let send = chain.reverb;
            for (i, f) in buf[b].as_chunks_mut::<2>().0.iter_mut().enumerate() {
                let (mut l, mut r) = (f[0], f[1]);
                if chain.active {
                    (l, r) = chain.process(l, r);
                    if let Some(k) = key {
                        let g = chain.duck.gain(k.get(i).copied().unwrap_or(0.0));
                        l *= g;
                        r *= g;
                    }
                }
                f[0] = l;
                f[1] = r;
                if send > 0.0 {
                    self.send[i * 2] += l * send;
                    self.send[i * 2 + 1] += r * send;
                }
            }
            if b != 0 {
                let (master, rest) = buf.split_at_mut(1);
                for (o, x) in master[0].iter_mut().zip(&rest[b - 1]) {
                    *o += x;
                }
            }
        }
        out.copy_from_slice(&buf[0]);
        if let Some(r) = &mut self.reverb {
            r.process_add(&self.send, out);
        }
        self.bus_out = buf;
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
