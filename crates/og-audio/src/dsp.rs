//! The mixer's signal processing: filters, dynamics and the reverb. Plain
//! f32 arithmetic with coefficients computed when a parameter changes, so a
//! given command stream renders the same samples every run on a host.

use std::f32::consts::PI;

/// The cutoffs a filter accepts, in Hz.
pub const MIN_CUTOFF: f32 = 20.0;
pub const MAX_CUTOFF: f32 = 20_000.0;

fn clamp_cutoff(hz: f32, rate: f32) -> f32 {
    hz.clamp(MIN_CUTOFF, MAX_CUTOFF.min(rate * 0.45))
}

/// A one-pole low-pass per channel (6 dB/octave): a voice's distance,
/// occlusion and `lowpass` filter. Its coefficient glides over a ramp, so a
/// cutoff change does not click.
#[derive(Clone, Copy, Debug, Default)]
pub struct OnePole {
    a: f32,
    target: f32,
    step: f32,
    ramp: u32,
    zl: f32,
    zr: f32,
    on: bool,
}

/// The one-pole coefficient of a cutoff (1: open).
pub fn one_pole_coefficient(cutoff: f32, rate: f32) -> f32 {
    if cutoff <= 0.0 {
        return 1.0;
    }
    1.0 - (-2.0 * PI * clamp_cutoff(cutoff, rate) / rate).exp()
}

impl OnePole {
    /// Sets the cutoff (0: open), gliding over `ramp` frames.
    pub fn set(&mut self, cutoff: f32, rate: f32, ramp: u32) {
        let a = one_pole_coefficient(cutoff, rate);
        if !self.on {
            if a >= 1.0 {
                return;
            }
            // Opening from bypass: start from the open coefficient.
            self.on = true;
            self.a = 1.0;
        }
        self.target = a;
        if ramp == 0 {
            self.a = a;
            self.ramp = 0;
        } else {
            self.ramp = ramp;
            self.step = (a - self.a) / ramp as f32;
        }
    }

    #[inline]
    pub fn process(&mut self, l: f32, r: f32) -> (f32, f32) {
        if !self.on {
            return (l, r);
        }
        if self.ramp > 0 {
            self.a += self.step;
            self.ramp -= 1;
            if self.ramp == 0 {
                self.a = self.target;
            }
        }
        self.zl += self.a * (l - self.zl);
        self.zr += self.a * (r - self.zr);
        (self.zl, self.zr)
    }
}

/// An RBJ biquad, stereo (12 dB/octave, Q 0.707): a bus's low- and
/// high-pass inserts.
#[derive(Clone, Copy, Debug, Default)]
pub struct Biquad {
    b: [f32; 3],
    a: [f32; 2],
    l: [f32; 4],
    r: [f32; 4],
    on: bool,
}

impl Biquad {
    /// A low-pass (`high` false) or high-pass at `cutoff` Hz; 0 bypasses.
    pub fn set(&mut self, cutoff: f32, high: bool, rate: f32) {
        if cutoff <= 0.0 {
            self.on = false;
            self.l = [0.0; 4];
            self.r = [0.0; 4];
            return;
        }
        let w = 2.0 * PI * clamp_cutoff(cutoff, rate) / rate;
        let (s, c) = w.sin_cos();
        let alpha = s / (2.0 * std::f32::consts::FRAC_1_SQRT_2);
        let a0 = 1.0 + alpha;
        let (b0, b1, b2) = if high {
            ((1.0 + c) / 2.0, -(1.0 + c), (1.0 + c) / 2.0)
        } else {
            ((1.0 - c) / 2.0, 1.0 - c, (1.0 - c) / 2.0)
        };
        self.b = [b0 / a0, b1 / a0, b2 / a0];
        self.a = [(-2.0 * c) / a0, (1.0 - alpha) / a0];
        self.on = true;
    }

    #[inline]
    fn run(b: &[f32; 3], a: &[f32; 2], z: &mut [f32; 4], x: f32) -> f32 {
        // z: x1, x2, y1, y2
        let y = b[0] * x + b[1] * z[0] + b[2] * z[1] - a[0] * z[2] - a[1] * z[3];
        z[1] = z[0];
        z[0] = x;
        z[3] = z[2];
        z[2] = y;
        y
    }

    #[inline]
    pub fn process(&mut self, l: f32, r: f32) -> (f32, f32) {
        if !self.on {
            return (l, r);
        }
        (
            Self::run(&self.b, &self.a, &mut self.l, l),
            Self::run(&self.b, &self.a, &mut self.r, r),
        )
    }
}

/// A smoothing coefficient for a time constant in seconds.
pub fn time_coefficient(seconds: f32, rate: f32) -> f32 {
    if seconds <= 0.0 {
        0.0
    } else {
        (-1.0 / (seconds * rate)).exp()
    }
}

pub fn db_to_gain(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

pub fn gain_to_db(g: f32) -> f32 {
    20.0 * g.max(1e-9).log10()
}

/// A stereo-linked feed-forward compressor: a peak detector with attack and
/// release, a hard knee at `threshold` dB with `ratio`, then `makeup` dB.
#[derive(Clone, Copy, Debug)]
pub struct Compressor {
    pub threshold: f32,
    pub ratio: f32,
    attack: f32,
    release: f32,
    makeup: f32,
    env: f32,
    on: bool,
    /// The gain reduction of the last frame, in dB (≤ 0).
    pub reduction: f32,
}

impl Default for Compressor {
    fn default() -> Self {
        Compressor {
            threshold: 0.0,
            ratio: 4.0,
            attack: 0.0,
            release: 0.0,
            makeup: 1.0,
            env: 0.0,
            on: false,
            reduction: 0.0,
        }
    }
}

impl Compressor {
    /// Threshold 0 dB or above turns it off.
    pub fn set(
        &mut self,
        threshold: f32,
        ratio: f32,
        attack: f32,
        release: f32,
        makeup: f32,
        rate: f32,
    ) {
        self.on = threshold < 0.0;
        self.threshold = threshold;
        self.ratio = ratio.max(1.0);
        self.attack = time_coefficient(attack.max(0.0005), rate);
        self.release = time_coefficient(release.max(0.005), rate);
        self.makeup = db_to_gain(makeup);
    }

    #[inline]
    pub fn process(&mut self, l: f32, r: f32) -> (f32, f32) {
        if !self.on {
            return (l, r);
        }
        let peak = l.abs().max(r.abs());
        let k = if peak > self.env {
            self.attack
        } else {
            self.release
        };
        self.env = peak + (self.env - peak) * k;
        let level = gain_to_db(self.env);
        let over = level - self.threshold;
        let reduction = if over > 0.0 {
            -over * (1.0 - 1.0 / self.ratio)
        } else {
            0.0
        };
        self.reduction = reduction;
        let g = db_to_gain(reduction) * self.makeup;
        (l * g, r * g)
    }
}

/// A side-chain ducker: while the key bus is above `threshold` dB the bus
/// is turned down by `depth` dB, easing in over `attack` and out over
/// `release` seconds.
#[derive(Clone, Copy, Debug, Default)]
pub struct Ducker {
    pub key: Option<usize>,
    threshold: f32,
    depth: f32,
    attack: f32,
    release: f32,
    env: f32,
    gain: f32,
}

impl Ducker {
    pub fn set(
        &mut self,
        key: Option<usize>,
        threshold: f32,
        depth: f32,
        attack: f32,
        release: f32,
        rate: f32,
    ) {
        self.key = key;
        self.threshold = db_to_gain(threshold);
        self.depth = db_to_gain(-depth.abs());
        self.attack = time_coefficient(attack.max(0.001), rate);
        self.release = time_coefficient(release.max(0.005), rate);
        if self.gain == 0.0 {
            self.gain = 1.0;
        }
    }

    /// The gain for this frame given the key's peak.
    #[inline]
    pub fn gain(&mut self, key_peak: f32) -> f32 {
        // A 10 ms peak follower on the key.
        self.env = if key_peak > self.env {
            key_peak
        } else {
            self.env * 0.9995
        };
        let target = if self.env > self.threshold {
            self.depth
        } else {
            1.0
        };
        let k = if target < self.gain {
            self.attack
        } else {
            self.release
        };
        self.gain = target + (self.gain - target) * k;
        self.gain
    }

    pub fn current(&self) -> f32 {
        if self.gain == 0.0 { 1.0 } else { self.gain }
    }
}

const COMBS: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const ALLPASSES: [usize; 4] = [556, 441, 341, 225];
const SPREAD: usize = 23;

#[derive(Clone, Debug)]
struct Comb {
    buf: Vec<f32>,
    at: usize,
    store: f32,
}

impl Comb {
    #[inline]
    fn process(&mut self, x: f32, feedback: f32, damp: f32) -> f32 {
        let y = self.buf[self.at];
        self.store = y * (1.0 - damp) + self.store * damp;
        self.buf[self.at] = x + self.store * feedback;
        self.at += 1;
        if self.at == self.buf.len() {
            self.at = 0;
        }
        y
    }
}

#[derive(Clone, Debug)]
struct Allpass {
    buf: Vec<f32>,
    at: usize,
}

impl Allpass {
    #[inline]
    fn process(&mut self, x: f32) -> f32 {
        let b = self.buf[self.at];
        let y = -x + b;
        self.buf[self.at] = x + b * 0.5;
        self.at += 1;
        if self.at == self.buf.len() {
            self.at = 0;
        }
        y
    }
}

/// A Schroeder–Moorer reverb (the Freeverb tunings, scaled to the rate):
/// eight damped combs and four all-passes per channel, the right channel's
/// lines 23 samples longer. The buses' reverb sends feed it; its wet output
/// returns into master.
#[derive(Clone, Debug)]
pub struct Reverb {
    combs: [Vec<Comb>; 2],
    allpasses: [Vec<Allpass>; 2],
    feedback: f32,
    damp: f32,
    /// Frames of silence fed since the last input (the tail is skipped once it has died).
    quiet: u32,
    tail: u32,
}

impl Reverb {
    pub fn new(rate: u32) -> Reverb {
        let scale = rate as f32 / 44_100.0;
        let len = |n: usize, ch: usize| (((n + ch * SPREAD) as f32 * scale) as usize).max(1);
        let combs = [0, 1].map(|ch| {
            COMBS
                .iter()
                .map(|&n| Comb {
                    buf: vec![0.0; len(n, ch)],
                    at: 0,
                    store: 0.0,
                })
                .collect()
        });
        let allpasses = [0, 1].map(|ch| {
            ALLPASSES
                .iter()
                .map(|&n| Allpass {
                    buf: vec![0.0; len(n, ch)],
                    at: 0,
                })
                .collect()
        });
        let mut r = Reverb {
            combs,
            allpasses,
            feedback: 0.0,
            damp: 0.0,
            quiet: u32::MAX,
            tail: rate * 8,
        };
        r.set(0.6, 0.4);
        r
    }

    /// Room size and damping, 0..1 each.
    pub fn set(&mut self, room: f32, damping: f32) {
        self.feedback = 0.7 + 0.28 * room.clamp(0.0, 1.0);
        self.damp = 0.4 * damping.clamp(0.0, 1.0);
    }

    /// Adds the wet signal of `input` (interleaved stereo) into `out`.
    pub fn process_add(&mut self, input: &[f32], out: &mut [f32]) {
        for (i, o) in input
            .as_chunks::<2>()
            .0
            .iter()
            .zip(out.as_chunks_mut::<2>().0.iter_mut())
        {
            if i[0] != 0.0 || i[1] != 0.0 {
                self.quiet = 0;
            } else if self.quiet >= self.tail {
                continue;
            } else {
                self.quiet += 1;
            }
            let x = (i[0] + i[1]) * 0.015;
            for ((combs, allpasses), o) in self
                .combs
                .iter_mut()
                .zip(&mut self.allpasses)
                .zip(o.iter_mut())
            {
                let mut acc = 0.0;
                for c in combs {
                    acc += c.process(x, self.feedback, self.damp);
                }
                for a in allpasses {
                    acc = a.process(acc);
                }
                *o += acc;
            }
        }
    }
}
