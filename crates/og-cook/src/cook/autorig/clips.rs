//! The clip library: hand-keyed clips per template, scaled to the rig's
//! proportions (its leg length and height), played as any rig's clips.
//!
//! Each track is keyed by hand at the poses that matter (a walk's
//! contact, down, passing and up), interpolated by a monotone cubic (it
//! never overshoots its keys, so a planted foot never dips below the
//! ground) or given as a curve, and sampled into linear keys at
//! [`RATE`]. Secondary motion follows the primary with a lag: the head
//! and torso counter-rotate against the hips, a forearm trails its upper
//! arm and a hand its forearm, and a cape, a tail or hair swing through
//! their chains, each bone later and wider than the one before.
//!
//! Angles are degrees counter-clockwise added to the setup pose (the art
//! faces right, so a positive turn swings an arm forwards and a negative
//! one leans the torso forwards); moves are rig pixels added to a bone's
//! setup position, given in leg lengths.

use super::template::Leg;
use og_anim::{
    BoneTimeline, Clip2d, Ease, IkKey, IkTimeline, Key, Rig2dAsset, Rotation, SlotTimeline, Turn,
};
use std::f32::consts::TAU;

/// Keys per second each track is sampled at.
pub const RATE: f32 = 30.0;

/// A curve keyed at fractions of a cycle (0..1), through its keys by a
/// monotone cubic (Fritsch and Carlson): no overshoot between keys.
#[derive(Clone)]
pub struct Curve {
    t: Vec<f32>,
    v: Vec<f32>,
    m: Vec<f32>,
    periodic: bool,
}

impl Curve {
    fn new(keys: &[(f32, f32)], periodic: bool) -> Curve {
        let mut keys = keys.to_vec();
        if periodic {
            // The neighbours across the wrap, so the cycle's ends join smoothly.
            let n = keys.len();
            let (a, b) = (keys[n - 2], keys[1]);
            keys.insert(0, (a.0 - 1.0, a.1));
            keys.push((1.0 + b.0, b.1));
        }
        let n = keys.len();
        let t: Vec<f32> = keys.iter().map(|k| k.0).collect();
        let v: Vec<f32> = keys.iter().map(|k| k.1).collect();
        let d: Vec<f32> = (0..n - 1)
            .map(|i| (v[i + 1] - v[i]) / (t[i + 1] - t[i]).max(1e-6))
            .collect();
        let mut m: Vec<f32> = (0..n)
            .map(
                |i| match (i.checked_sub(1).map(|j| d[j]), d.get(i).copied()) {
                    (Some(a), Some(b)) if a * b > 0.0 => (a + b) / 2.0,
                    (None, Some(b)) => b,
                    (Some(a), None) => a,
                    _ => 0.0,
                },
            )
            .collect();
        for i in 0..n - 1 {
            if d[i] == 0.0 {
                m[i] = 0.0;
                m[i + 1] = 0.0;
                continue;
            }
            let (a, b) = (m[i] / d[i], m[i + 1] / d[i]);
            let s = a * a + b * b;
            if s > 9.0 {
                let k = 3.0 / s.sqrt();
                m[i] = k * a * d[i];
                m[i + 1] = k * b * d[i];
            }
        }
        Curve { t, v, m, periodic }
    }

    /// A curve that repeats every cycle; its first and last keys (at 0 and 1) agree.
    pub fn cycle(keys: &[(f32, f32)]) -> Curve {
        Curve::new(keys, true)
    }

    /// A curve that holds its first and last keys beyond them.
    pub fn once(keys: &[(f32, f32)]) -> Curve {
        Curve::new(keys, false)
    }

    /// A one-shot curve through `values` at the fractions `times`.
    pub fn keyed(times: &[f32], values: &[f32]) -> Curve {
        let keys: Vec<(f32, f32)> = times.iter().copied().zip(values.iter().copied()).collect();
        Curve::once(&keys)
    }

    pub fn at(&self, t: f32) -> f32 {
        let t = if self.periodic { t.rem_euclid(1.0) } else { t };
        let n = self.t.len();
        if t <= self.t[0] {
            return self.v[0];
        }
        if t >= self.t[n - 1] {
            return self.v[n - 1];
        }
        let i = self.t.partition_point(|&k| k <= t).clamp(1, n - 1) - 1;
        let h = self.t[i + 1] - self.t[i];
        let s = (t - self.t[i]) / h;
        let (s2, s3) = (s * s, s * s * s);
        (2.0 * s3 - 3.0 * s2 + 1.0) * self.v[i]
            + (s3 - 2.0 * s2 + s) * h * self.m[i]
            + (-2.0 * s3 + 3.0 * s2) * self.v[i + 1]
            + (s3 - s2) * h * self.m[i + 1]
    }
}

/// One clip being built: tracks sampled from functions of the clip's
/// fraction (0..1), for the bones the rig has.
pub struct Clip<'a> {
    rig: &'a Rig2dAsset,
    clip: Clip2d,
}

impl<'a> Clip<'a> {
    pub fn new(rig: &'a Rig2dAsset, name: &str, duration: f32, looping: bool) -> Clip<'a> {
        Clip {
            rig,
            clip: Clip2d {
                name: name.into(),
                duration,
                looping,
                bones: Vec::new(),
                slots: Vec::new(),
                deforms: Vec::new(),
                order: Vec::new(),
                iks: Vec::new(),
                events: Vec::new(),
            },
        }
    }

    fn times(&self) -> Vec<f32> {
        let n = (self.clip.duration * RATE).round().max(1.0) as usize;
        (0..=n).map(|i| i as f32 / n as f32).collect()
    }

    fn track(&mut self, bone: &str) -> Option<&mut BoneTimeline> {
        let b = self.rig.bone(bone)?;
        let i = match self.clip.bones.iter().position(|t| t.bone == b) {
            Some(i) => i,
            None => {
                self.clip.bones.push(BoneTimeline {
                    bone: b,
                    ..Default::default()
                });
                self.clip.bones.len() - 1
            }
        };
        Some(&mut self.clip.bones[i])
    }

    /// Turns `bone` by `f(fraction)` degrees.
    pub fn rot(&mut self, bone: &str, f: impl Fn(f32) -> f32) {
        let d = self.clip.duration;
        let keys: Vec<Key<Rotation>> = self
            .times()
            .into_iter()
            .map(|t| Key {
                time: t * d,
                value: Rotation {
                    degrees: f(t),
                    turn: Turn::Shorter,
                },
                ease: Ease::Linear,
            })
            .collect();
        if let Some(track) = self.track(bone) {
            track.rotate = keys;
        }
    }

    /// Moves `bone` by `f(fraction)` pixels.
    pub fn mov(&mut self, bone: &str, f: impl Fn(f32) -> [f32; 2]) {
        let d = self.clip.duration;
        let keys: Vec<Key<[f32; 2]>> = self
            .times()
            .into_iter()
            .map(|t| Key {
                time: t * d,
                value: f(t),
                ease: Ease::Linear,
            })
            .collect();
        if let Some(track) = self.track(bone) {
            track.translate = keys;
        }
    }

    /// Moves `bone` by `f(fraction)` pixels along the rig's axes (x right,
    /// y up), whichever way its parent points.
    pub fn shift(&mut self, bone: &str, f: impl Fn(f32) -> [f32; 2]) {
        let Some(b) = self.rig.bone(bone) else { return };
        let pose = self.rig.pose(None, 0.0, &[]);
        let inverse = self.rig.bones[b]
            .parent
            .map_or(og_anim::Affine2::IDENTITY, |p| pose.world[p].inverse());
        let zero = inverse.apply_to([0.0, 0.0]);
        self.mov(bone, |t| {
            let q = inverse.apply_to(f(t));
            [q[0] - zero[0], q[1] - zero[1]]
        });
    }

    /// Scales `bone` by `f(fraction)` (x along the bone, y across it).
    pub fn scale(&mut self, bone: &str, f: impl Fn(f32) -> [f32; 2]) {
        let d = self.clip.duration;
        let keys: Vec<Key<[f32; 2]>> = self
            .times()
            .into_iter()
            .map(|t| Key {
                time: t * d,
                value: f(t),
                ease: Ease::Linear,
            })
            .collect();
        if let Some(track) = self.track(bone) {
            track.scale = keys;
        }
    }

    /// Turns a chain (`base`, `base-2`, ...): bone `k` by `f(fraction, k)`.
    pub fn chain(&mut self, base: &str, f: impl Fn(f32, usize) -> f32) {
        for k in 0..8 {
            let name = if k == 0 {
                base.to_string()
            } else {
                format!("{base}-{}", k + 1)
            };
            if self.rig.bone(&name).is_none() {
                break;
            }
            self.rot(&name, |t| f(t, k));
        }
    }

    /// Adds `f(fraction)` (an RGBA offset) to every slot: a flash.
    pub fn flash(&mut self, f: impl Fn(f32) -> [f32; 4]) {
        let d = self.clip.duration;
        let keys: Vec<Key<[f32; 4]>> = self
            .times()
            .into_iter()
            .map(|t| Key {
                time: t * d,
                value: f(t),
                ease: Ease::Linear,
            })
            .collect();
        for slot in 0..self.rig.slots.len() {
            self.clip.slots.push(SlotTimeline {
                slot,
                offset: keys.clone(),
                ..Default::default()
            });
        }
    }

    /// Draws the named slots in front of every other for the whole clip
    /// (a far arm raised in front of the head), in their own order.
    pub fn in_front(&mut self, names: &[String]) {
        let moved: Vec<usize> = (0..self.rig.slots.len())
            .filter(|&i| names.contains(&self.rig.slots[i].name))
            .collect();
        if moved.is_empty() {
            return;
        }
        let mut order: Vec<usize> = (0..self.rig.slots.len())
            .filter(|i| !moved.contains(i))
            .collect();
        order.extend(moved);
        self.clip.order = vec![(0.0, order)];
    }

    /// Bends the leg whose IK is named `ik` the other way for the whole
    /// clip (`counterclockwise`), from a pose where it is straight so it
    /// never snaps.
    pub fn bend(&mut self, ik: &str, counterclockwise: bool) {
        if let Some(i) = self.rig.iks.iter().position(|k| k.name == ik) {
            self.clip.iks.push(IkTimeline {
                ik: i,
                keys: vec![Key {
                    time: 0.0,
                    value: IkKey {
                        weight: 1.0,
                        counterclockwise,
                    },
                    ease: Ease::Linear,
                }],
            });
        }
    }

    pub fn event(&mut self, fraction: f32, name: &str) {
        self.clip
            .events
            .push((fraction * self.clip.duration, name.into()));
    }

    pub fn done(self) -> Clip2d {
        let mut c = self.clip;
        c.events.sort_by(|a, b| a.0.total_cmp(&b.0));
        c
    }
}

/// `amplitude` times a cosine of `cycles` per clip, `lag` (fraction of
/// the clip) late.
pub fn swing(t: f32, amplitude: f32, cycles: f32, lag: f32) -> f32 {
    amplitude * (TAU * cycles * (t - lag)).cos()
}

/// The rig's measures the clips scale by, rig pixels: leg length (hip to
/// ankle), how far its back reaches behind the root, how high its ankles
/// are and how tall it is drawn.
pub struct Measure {
    pub leg: f32,
    pub back: f32,
    pub ankle: f32,
    pub height: f32,
}

/// Where a bone's origin is in the setup pose, rig pixels.
pub fn origin(rig: &Rig2dAsset, bone: &str) -> Option<[f32; 2]> {
    let pose = rig.pose(None, 0.0, &[]);
    rig.bone(bone).map(|b| pose.world[b].apply(0.0, 0.0))
}

/// The box (left, bottom, right, top) of what a slot draws in the setup
/// pose, rig pixels.
pub fn slot_box(rig: &Rig2dAsset, slot: &str) -> Option<[f32; 4]> {
    let pose = rig.pose(None, 0.0, &[]);
    let (vertices, _) = rig.triangles(&pose);
    let want = rig.slots.iter().position(|s| s.name == slot)?;
    let mut at = 0;
    for &s in &pose.order {
        let n = match pose.slots[s].map(|a| &rig.attachments[a].kind) {
            Some(og_anim::AttachmentKind::Region { .. }) => 4,
            Some(og_anim::AttachmentKind::Mesh { vertices, .. }) => vertices.len(),
            _ => 0,
        };
        if s == want {
            let p = vertices[at..at + n]
                .iter()
                .map(|v| v.position.map(|c| c * rig.unit));
            return Some(p.fold([f32::MAX, f32::MAX, f32::MIN, f32::MIN], |b, q| {
                [
                    b[0].min(q[0]),
                    b[1].min(q[1]),
                    b[2].max(q[0]),
                    b[3].max(q[1]),
                ]
            }));
        }
        at += n;
    }
    None
}

pub fn measure(rig: &Rig2dAsset, legs: &[Leg], height: f32) -> Measure {
    let pose = rig.pose(None, 0.0, &[]);
    let at = |name: &str| rig.bone(name).map(|b| pose.world[b].apply(0.0, 0.0));
    let lengths: Vec<f32> = legs
        .iter()
        .filter_map(|l| {
            let (h, a) = (at(l.upper)?, at(l.end)?);
            Some((h[0] - a[0]).hypot(h[1] - a[1]))
        })
        .collect();
    let leg = if lengths.is_empty() {
        height * 0.45
    } else {
        lengths.iter().sum::<f32>() / lengths.len() as f32
    };
    let (vertices, _) = rig.triangles(&pose);
    let back = vertices
        .iter()
        .map(|v| -v.position[0] * rig.unit)
        .fold(0.0f32, f32::max);
    let ankles: Vec<f32> = legs.iter().filter_map(|l| Some(at(l.end)?[1])).collect();
    let ankle = ankles.iter().sum::<f32>() / ankles.len().max(1) as f32;
    Measure {
        leg,
        back,
        ankle,
        height,
    }
}
