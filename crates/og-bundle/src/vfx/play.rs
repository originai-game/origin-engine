//! Playing a composed effect: what one instance draws at a tick
//! ([`Vfx::draws`]) and what it cues in the ticks a step runs
//! ([`Vfx::cues`]: its events, shakes, sounds and flipbooks), from the
//! effect, the instance's seed and start tick, the tick and where the
//! instance stands alone. Nothing here reads a world or a renderer, so a
//! replay, a restore or a seek gives the same answers; the runtime turns
//! the draws into the renderer's systems, meshes, trails, beams, lights
//! and decals with the handles it linked at load, and plays the cues.

use super::{MAX_PARAMS, Vfx, VfxEventKey, VfxKind};
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

/// `rgb` turned `a` radians about the grey axis: the WGSL library's
/// `hue_rotate`, for the colours drawn without a shader of the effect's
/// own to turn them (trails, beams, lights, decals).
pub fn hue_rotated(rgb: Vec3, a: f32) -> Vec3 {
    if a == 0.0 {
        return rgb;
    }
    let k = Vec3::splat(0.57735);
    let (s, c) = og_core::math::sin_cos(a);
    rgb * c + k.cross(rgb) * s + k * k.dot(rgb) * (1.0 - c)
}

/// `c` tinted, then its hue turned `hue` radians.
fn shaded(c: [f32; 4], t: Vec3, hue: f32) -> [f32; 4] {
    let c = tinted(c, t);
    hue_rotated(Vec3::new(c[0], c[1], c[2]), hue)
        .extend(c[3])
        .to_array()
}

pub(super) fn euler(deg: Vec3) -> Quat {
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
    /// Radians it turns the hue of everything it draws. Its trails',
    /// beams', lights' and decals' colours come turned; its emitters and
    /// meshes take it as theirs and turn their drawn colour.
    pub hue: f32,
    /// Its params' values ([`Vfx::values`]).
    pub params: [f32; MAX_PARAMS],
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
        /// Its share of the particles born (its `amount`), 0 to 1.
        emission: f32,
    },
    /// An effect mesh: a `mesh` element with its effect material, or a
    /// `glow` (the glow sphere, additive).
    Mesh {
        transform: Mat4,
        /// Its life, 0 to 1.
        life: f32,
        seed: u64,
        color: [f32; 4],
        blend: ParticleBlend,
        /// Turned to face the camera (`face camera`): `transform`'s
        /// rotation and scale are in the camera's frame
        /// (`og_render_data::EffectMesh::billboard`).
        billboard: bool,
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
    /// A decal box (it projects along its -z), with its element's image
    /// and normal map.
    Decal {
        transform: Mat4,
        /// `[u0, v0, u1, v1]` of its textures.
        uv: [f32; 4],
        color: [f32; 4],
        emissive: Vec3,
        normal_scale: f32,
        /// Negative: the surface's own.
        roughness: f32,
        metallic: f32,
        wet: bool,
        angle_fade: f32,
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
    /// An instance's params' values: `given` by declaration order, each
    /// past `given`'s length its default (0 past the declared ones).
    pub fn values(&self, given: &[f32]) -> [f32; MAX_PARAMS] {
        std::array::from_fn(|i| match (given.get(i), self.params.get(i)) {
            (Some(v), Some(_)) if v.is_finite() => *v,
            (_, Some(p)) => p.default,
            _ => 0.0,
        })
    }

    /// Whether element `i` plays with these values (its `when` above 0).
    pub fn plays(&self, i: usize, values: &[f32; MAX_PARAMS]) -> bool {
        self.elements[i].when.is_none_or(|p| values[p] > 0.0)
    }

    /// What element `i`'s `stretch` multiplies its length by with these
    /// values (1 without one).
    pub fn stretch(&self, i: usize, values: &[f32; MAX_PARAMS]) -> f32 {
        self.elements[i].stretch.map_or(1.0, |p| values[p].max(0.0))
    }

    /// What element `i`'s `amount` multiplies it by with these values.
    pub fn amount(&self, i: usize, values: &[f32; MAX_PARAMS]) -> f32 {
        self.elements[i].amount.map_or(1.0, |p| values[p].max(0.0))
    }

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

    /// What an instance cues on the ticks `run`: its events in
    /// time order, then per element in file order its flipbook's end and
    /// its start. `clips` is, per element, the length in seconds of a
    /// flipbook element's first clip (what ends one without a `life`).
    /// An element whose `when` is off with `values` starts nothing; a
    /// shake's trauma comes multiplied by its `amount` (a sound's volume
    /// and a flipbook's size are the caller's, [`Vfx::amount`]).
    pub fn cues(
        &self,
        seed: u32,
        values: &[f32; MAX_PARAMS],
        start: u64,
        run: std::ops::RangeInclusive<u64>,
        rate: f32,
        clips: &[f32],
    ) -> Vec<VfxCue<'_>> {
        let len = self.length_ticks(rate);
        // Whether a time lands on one of the ticks run (its tick in the
        // cycle; a looping effect's end is its next start).
        let due = |at: f32| -> bool {
            let k = ticks(at, rate).min(len);
            let k = if self.looping { k % len } else { k };
            run.clone().any(|t| {
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
            if !due(el.at) || !self.plays(i, values) {
                continue;
            }
            match &el.kind {
                VfxKind::Shake { trauma } => out.push(VfxCue::Shake {
                    trauma: (trauma * self.amount(i, values)).min(1.0),
                }),
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
        let (seed, scale, tint, hue, m) = (at.seed, at.scale, at.tint, at.hue, at.transform);
        let t = (tick - cycle) as f32 / rate;
        let mut out = Vec::new();
        for (i, el) in self.elements.iter().enumerate() {
            let te = t - el.at;
            if te < 0.0 || !self.plays(i, &at.params) {
                continue;
            }
            let amount = self.amount(i, &at.params);
            let long = self.stretch(i, &at.params);
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
                        emission: amount.min(1.0),
                    }
                }
                VfxKind::Mesh {
                    scale: size,
                    spin,
                    color,
                    blend,
                    face_camera,
                    ..
                } if alive => {
                    let k = size.eval(x, rnd) * amount;
                    let s = Mat4::from_scale(Vec3::new(k, k * long, k));
                    // Facing the camera: where it is and how large, its
                    // spin in the camera's frame (the renderer's), no
                    // rotation of its own.
                    let placed = if *face_camera {
                        let at = local.w_axis.truncate();
                        let s = (local.x_axis.truncate().length()
                            + local.y_axis.truncate().length()
                            + local.z_axis.truncate().length())
                            / 3.0;
                        Mat4::from_translation(at) * Mat4::from_scale(Vec3::splat(s))
                    } else {
                        local
                    };
                    VfxDrawKind::Mesh {
                        transform: placed * Mat4::from_quat(euler(*spin * te)) * s,
                        life: x,
                        seed: u64::from(seed) ^ i as u64,
                        color: tinted(*color, tint),
                        blend: *blend,
                        billboard: *face_camera,
                    }
                }
                VfxKind::Glow { size, color } if alive => VfxDrawKind::Mesh {
                    transform: local * {
                        let k = size.eval(x, rnd) * amount;
                        Mat4::from_scale(Vec3::new(k, k * long, k))
                    },
                    life: x,
                    seed: u64::from(seed) ^ i as u64,
                    color: tinted(*color, tint),
                    blend: ParticleBlend::Additive,
                    billboard: false,
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
                        width: [width[0] * scale * amount, width[1] * scale * amount],
                        color: shaded(*color, tint, hue),
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
                    width: width * scale * amount,
                    color: shaded(*color, tint, hue),
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
                    color: hue_rotated(Vec3::from_array(*color) * tint, hue),
                    intensity: intensity.eval(x, rnd).max(0.0) * amount,
                    range: distance * scale,
                },
                VfxKind::Decal {
                    uv,
                    size,
                    fade,
                    color,
                    emissive,
                    normal_scale,
                    roughness,
                    metallic,
                    wet,
                    angle_fade,
                    turn,
                    ..
                } if alive => {
                    let a = if *fade > 0.0 {
                        ((el.life - te) / fade).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    let c = shaded(*color, tint, hue);
                    // Its unit box projects along its -z: turned down the
                    // element's -y, y the depth.
                    let turned = (turn[0] + (turn[1] - turn[0]) * rnd).to_radians();
                    VfxDrawKind::Decal {
                        transform: local
                            * Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                            * Mat4::from_rotation_z(turned)
                            * Mat4::from_scale(Vec3::new(size.x * amount, size.z * amount, size.y)),
                        uv: *uv,
                        color: [c[0], c[1], c[2], c[3] * a],
                        emissive: hue_rotated(Vec3::from_array(*emissive) * tint, hue),
                        normal_scale: *normal_scale,
                        roughness: *roughness,
                        metallic: *metallic,
                        wet: *wet,
                        angle_fade: *angle_fade,
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
