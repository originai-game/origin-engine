//! Playing a composed effect: what one instance draws at a tick
//! ([`Vfx::draws`]) and what it cues in the ticks a step runs
//! ([`Vfx::cues`]: its events, shakes, sounds and flipbooks), from the
//! effect, the instance's seed and start tick, the tick and where the
//! instance stands alone. Nothing here reads a world or a renderer, so a
//! replay, a restore or a seek gives the same answers; the runtime turns
//! the draws into the renderer's systems, meshes, trails, beams, lights
//! and decals with the handles it linked at load, and plays the cues.

use super::{Vfx, VfxEventKey, VfxKind};
use glam::{Mat4, Quat, Vec3};
use og_core::hash::splitmix64 as splitmix;
use og_render_data::ParticleBlend;

/// An element's random value, 0 to 1, fixed per instance.
pub fn random(seed: u32, element: usize) -> f32 {
    og_core::hash::unit24_of64(splitmix(u64::from(seed) << 16 | element as u64))
}

/// The renderer id of an instance's element (a system, a trail, a mesh).
pub fn element_id(instance: u64, element: usize) -> u64 {
    splitmix(instance ^ ((element as u64 + 1) << 52)) & !(1 << 63)
}

fn ticks(seconds: f32, rate: f32) -> u64 {
    (f64::from(seconds) * f64::from(rate)).round().max(0.0) as u64
}

fn tinted(c: [f32; 4], t: Vec3) -> [f32; 4] {
    [c[0] * t.x, c[1] * t.y, c[2] * t.z, c[3]]
}

fn euler(deg: Vec3) -> Quat {
    let r = deg * (std::f32::consts::PI / 180.0);
    use og_core::math::quat_from_axis_angle as turn;
    turn(Vec3::X, r.x) * turn(Vec3::Y, r.y) * turn(Vec3::Z, r.z)
}

/// Where one instance of an effect stands and how it differs from the
/// next: what [`Vfx::draws`] needs beside the ticks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VfxInstance {
    /// Its identity (the entity's bits): its elements' ids come from it.
    pub id: u64,
    /// What `random` values and emitters pick.
    pub seed: u32,
    /// Its world matrix, its scale applied.
    pub transform: Mat4,
    /// Its size: widths, reaches and jitters scale with it.
    pub scale: f32,
    /// Multiplies the colour of everything it draws (linear).
    pub tint: Vec3,
}

/// One thing an instance draws at a tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VfxDraw {
    /// The element's index in [`Vfx::elements`].
    pub element: usize,
    /// Its renderer id ([`element_id`]).
    pub id: u64,
    pub kind: VfxDrawKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VfxDrawKind {
    /// A particle system of the element's emitter, emitting from
    /// `start_tick` until `stop_tick` (None: left to its own cycle).
    Emitter {
        transform: Mat4,
        start_tick: u64,
        stop_tick: Option<u64>,
        seed: u64,
    },
    /// An effect mesh: a `mesh` element with its effect material, or a
    /// `flash` (the glow sphere, additive).
    Mesh {
        transform: Mat4,
        /// Its life, 0 to 1.
        life: f32,
        seed: u64,
        color: [f32; 4],
        blend: ParticleBlend,
    },
    /// A trail's head this tick and its style's numbers.
    Trail {
        position: Vec3,
        /// Seconds a point lasts.
        lifetime: f32,
        min_distance: f32,
        /// At its head, at its tail.
        width: [f32; 2],
        color: [f32; 4],
    },
    Beam {
        start: Vec3,
        end: Vec3,
        width: f32,
        color: [f32; 4],
        segments: u32,
        jitter: f32,
        seed: u64,
    },
    /// A point light.
    Light {
        position: Vec3,
        color: Vec3,
        intensity: f32,
        range: f32,
    },
    /// A decal box (it projects along its -z).
    Decal {
        transform: Mat4,
        /// `[u0, v0, u1, v1]` of the decal atlas.
        uv: [f32; 4],
        color: [f32; 4],
    },
}

/// One thing an instance does in the tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VfxCue<'a> {
    /// An `event` or `impact` line's time came.
    Event(&'a VfxEventKey),
    /// A `flipbook` element's clip (or its life) is over: its sprite goes.
    FlipbookEnd {
        element: usize,
    },
    Shake {
        trauma: f32,
    },
    /// A `sound` element starts, at the playback rate the seed picked.
    Sound {
        element: usize,
        playback_rate: f32,
    },
    /// A `flipbook` element starts.
    Flipbook {
        element: usize,
    },
}

impl Vfx {
    /// Its length in ticks, at least one.
    pub fn length_ticks(&self, rate: f32) -> u64 {
        ticks(self.duration, rate).max(1)
    }

    /// The tick an instance's current cycle started (a looping effect starts
    /// over every duration), or None before it starts or after a non-looping
    /// one ends.
    pub fn cycle_start(&self, start: u64, tick: u64, rate: f32) -> Option<u64> {
        if tick < start {
            return None;
        }
        let len = self.length_ticks(rate);
        let since = tick - start;
        if self.looping {
            Some(start + since / len * len)
        } else {
            (since <= len).then_some(start)
        }
    }

    /// Whether a non-looping instance started at `start` is over at `tick`.
    pub fn ended(&self, start: u64, tick: u64, rate: f32) -> bool {
        !self.looping && tick >= start && tick - start >= self.length_ticks(rate)
    }

    /// The first tick a step at `tick` runs for an instance that last ran
    /// through `ran`: the tick after it (the start for one that never ran,
    /// or whose start was written again), at most one cycle back. None
    /// before the start and when `tick` has run.
    pub fn run_from(&self, start: u64, ran: Option<u64>, tick: u64, rate: f32) -> Option<u64> {
        if tick < start {
            return None;
        }
        let from = match ran {
            Some(ran) if ran >= start => {
                if ran >= tick {
                    return None;
                }
                ran + 1
            }
            _ => start,
        };
        Some(from.max(tick.saturating_sub(self.length_ticks(rate))))
    }

    /// What an instance cues on the ticks `from` to `tick`: its events in
    /// time order, then per element in file order its flipbook's end and
    /// its start. `clips` is, per element, the length in seconds of a
    /// flipbook element's first clip (what ends one without a `life`).
    pub fn cues(
        &self,
        seed: u32,
        start: u64,
        from: u64,
        tick: u64,
        rate: f32,
        clips: &[f32],
    ) -> Vec<VfxCue<'_>> {
        let len = self.length_ticks(rate);
        // Whether a time lands on one of the ticks run (its tick in the
        // cycle; a looping effect's end is its next start).
        let due = |at: f32| -> bool {
            let k = ticks(at, rate).min(len);
            let k = if self.looping { k % len } else { k };
            (from..=tick).any(|t| {
                let since = t - start;
                k == if self.looping { since % len } else { since }
            })
        };
        let mut out = Vec::new();
        for ev in &self.events {
            if due(ev.time) {
                out.push(VfxCue::Event(ev));
            }
        }
        for (i, el) in self.elements.iter().enumerate() {
            if let VfxKind::Flipbook { .. } = el.kind {
                let clip = clips.get(i).copied().unwrap_or(0.0);
                let life = if el.life > 0.0 { el.life } else { clip };
                if due(el.at + life) {
                    out.push(VfxCue::FlipbookEnd { element: i });
                }
            }
            if !due(el.at) {
                continue;
            }
            match &el.kind {
                VfxKind::Shake { trauma } => out.push(VfxCue::Shake { trauma: *trauma }),
                VfxKind::Sound { pitch, .. } => out.push(VfxCue::Sound {
                    element: i,
                    playback_rate: pitch[0] + (pitch[1] - pitch[0]) * random(seed, i),
                }),
                VfxKind::Flipbook { .. } => out.push(VfxCue::Flipbook { element: i }),
                _ => {}
            }
        }
        out
    }

    /// What an instance started at `start` draws at `tick`, in file order:
    /// each element inside its window, placed by the instance.
    pub fn draws(&self, at: &VfxInstance, start: u64, tick: u64, rate: f32) -> Vec<VfxDraw> {
        let Some(cycle) = self.cycle_start(start, tick, rate) else {
            return Vec::new();
        };
        let (seed, scale, tint, m) = (at.seed, at.scale, at.tint, at.transform);
        let t = (tick - cycle) as f32 / rate;
        let mut out = Vec::new();
        for (i, el) in self.elements.iter().enumerate() {
            let te = t - el.at;
            if te < 0.0 {
                continue;
            }
            let local = m * el.local_at(te);
            let life = el.life.max(1e-4);
            let x = (te / life).clamp(0.0, 1.0);
            let rnd = random(seed, i);
            let alive = te <= el.life;
            let kind = match &el.kind {
                VfxKind::Emitter { seed: s, .. } => {
                    let begin = cycle + ticks(el.at, rate);
                    VfxDrawKind::Emitter {
                        transform: local,
                        start_tick: begin,
                        stop_tick: (el.life > 0.0).then(|| begin + ticks(el.life, rate).max(1)),
                        seed: u64::from(seed.wrapping_add(*s)),
                    }
                }
                VfxKind::Mesh {
                    scale: size,
                    spin,
                    color,
                    blend,
                    ..
                } if alive => VfxDrawKind::Mesh {
                    transform: local
                        * Mat4::from_quat(euler(*spin * te))
                        * Mat4::from_scale(Vec3::splat(size.eval(x, rnd))),
                    life: x,
                    seed: u64::from(seed) ^ i as u64,
                    color: tinted(*color, tint),
                    blend: *blend,
                },
                VfxKind::Flash { size, color } if alive => VfxDrawKind::Mesh {
                    transform: local * Mat4::from_scale(Vec3::splat(size.eval(x, rnd))),
                    life: x,
                    seed: u64::from(seed) ^ i as u64,
                    color: tinted(*color, tint),
                    blend: ParticleBlend::Additive,
                },
                VfxKind::Trail {
                    width,
                    lifetime,
                    color,
                    orbit,
                    rise,
                    ..
                } if te <= el.life + lifetime => {
                    let tc = te.min(el.life);
                    let a = orbit[1] * tc * std::f32::consts::TAU;
                    let (sa, ca) = og_core::math::sin_cos(a);
                    let p = Vec3::new(orbit[0] * ca, rise * tc, orbit[0] * sa);
                    VfxDrawKind::Trail {
                        position: local.transform_point3(p),
                        lifetime: *lifetime,
                        min_distance: 0.02 * scale,
                        width: [width[0] * scale, width[1] * scale],
                        color: tinted(*color, tint),
                    }
                }
                VfxKind::Beam {
                    to,
                    width,
                    color,
                    segments,
                    jitter,
                    ..
                } if alive => VfxDrawKind::Beam {
                    start: local.transform_point3(Vec3::ZERO),
                    end: local.transform_point3(*to),
                    width: width * scale,
                    color: tinted(*color, tint),
                    segments: *segments,
                    jitter: jitter * scale,
                    seed: u64::from(seed) ^ i as u64,
                },
                VfxKind::Light {
                    color,
                    intensity,
                    distance,
                } if alive => VfxDrawKind::Light {
                    position: local.transform_point3(Vec3::ZERO),
                    color: Vec3::from_array(*color) * tint,
                    intensity: intensity.eval(x, rnd).max(0.0),
                    range: distance * scale,
                },
                VfxKind::Decal {
                    uv,
                    size,
                    fade,
                    color,
                } if alive => {
                    let a = if *fade > 0.0 {
                        ((el.life - te) / fade).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    let c = tinted(*color, tint);
                    // Its unit box projects along its -z: turned down the
                    // element's -y, y the depth.
                    VfxDrawKind::Decal {
                        transform: local
                            * Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                            * Mat4::from_scale(Vec3::new(size.x, size.z, size.y)),
                        uv: *uv,
                        color: [c[0], c[1], c[2], c[3] * a],
                    }
                }
                _ => continue,
            };
            out.push(VfxDraw {
                element: i,
                id: element_id(at.id, i),
                kind,
            });
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vfx(looping: bool) -> Vfx {
        Vfx {
            duration: 1.0,
            looping,
            events: Vec::new(),
            elements: Vec::new(),
        }
    }

    #[test]
    fn a_cycle_starts_at_the_start_and_over_at_each_duration_when_looping() {
        // 60 ticks a second: a 1 s effect started at tick 100.
        assert_eq!(vfx(false).cycle_start(100, 99, 60.0), None);
        assert_eq!(vfx(false).cycle_start(100, 160, 60.0), Some(100));
        assert_eq!(vfx(false).cycle_start(100, 161, 60.0), None);
        assert_eq!(vfx(true).cycle_start(100, 161, 60.0), Some(160));
        assert_eq!(vfx(true).cycle_start(100, 280, 60.0), Some(280));
    }

    #[test]
    fn instances_differ_by_seed_and_elements_by_index() {
        assert_ne!(random(1, 0), random(2, 0));
        assert_ne!(random(1, 0), random(1, 1));
        assert!((0.0..1.0).contains(&random(7, 3)));
        assert_ne!(element_id(5, 0), element_id(5, 1));
        assert_eq!(element_id(5, 0) >> 63, 0);
    }
}
