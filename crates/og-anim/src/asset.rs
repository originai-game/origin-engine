//! Animation assets: skeletons (a glTF's node hierarchy with its skins) and
//! clips (channels resampled at a fixed rate). The cook writes them
//! (`og-cook`, the `<model>#anim` blob); the loaders build these types; the
//! host registers them with [`crate::AnimationSystem`] under a handle.

use crate::xform::{nlerp, unit, Trs};
use og_core::glam::{Mat4, Quat, Vec3};
use og_core::{Error, ErrorKind, Result};

/// "None" in a `u32` index.
pub const NONE: u32 = u32::MAX;

/// One node of a skeleton, in topological order (a parent precedes its children).
#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonNode {
    pub name: String,
    /// The parent's index, or [`NONE`] for a root.
    pub parent: u32,
    /// The rest (bind-time) local transform, glTF's node TRS.
    pub rest: Trs,
    /// The glTF mesh this node draws, or [`NONE`].
    pub mesh: u32,
}

/// A glTF skin: the joints a skinned mesh's vertices name, and their
/// inverse bind matrices.
#[derive(Clone, Debug, PartialEq)]
pub struct Skin {
    pub name: String,
    /// Node indices, in the order vertices' `JOINTS_0` index them.
    pub joints: Vec<u32>,
    /// One per joint: model space to the joint's bind space.
    pub inverse_bind: Vec<Mat4>,
}

/// A rig: every node of a glTF (joints and the nodes between them) and its skins.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkeletonAsset {
    pub nodes: Vec<SkeletonNode>,
    pub skins: Vec<Skin>,
}

impl SkeletonAsset {
    /// Checks the hierarchy (parents precede children) and the skins.
    pub fn validate(&self) -> Result<()> {
        for (i, n) in self.nodes.iter().enumerate() {
            if n.parent != NONE && n.parent as usize >= i {
                return Err(invalid(format!(
                    "skeleton node {i} '{}': its parent {} does not precede it",
                    n.name, n.parent
                )));
            }
        }
        for (s, skin) in self.skins.iter().enumerate() {
            if skin.joints.len() != skin.inverse_bind.len() {
                return Err(invalid(format!(
                    "skin {s}: {} joints but {} inverse bind matrices",
                    skin.joints.len(),
                    skin.inverse_bind.len()
                )));
            }
            if let Some(j) = skin
                .joints
                .iter()
                .find(|&&j| j as usize >= self.nodes.len())
            {
                return Err(invalid(format!(
                    "skin {s}: joint node {j} is past the {} nodes",
                    self.nodes.len()
                )));
            }
        }
        Ok(())
    }

    /// The rest pose: every node's rest transform.
    pub fn rest_pose(&self) -> Vec<Trs> {
        self.nodes.iter().map(|n| n.rest).collect()
    }

    /// The index of the node called `name`.
    pub fn node(&self, name: &str) -> Option<u32> {
        self.nodes
            .iter()
            .position(|n| n.name == name)
            .map(|i| i as u32)
    }

    /// The node of joint `joint` of skin `skin`; without skins, joint `j` is node `j`.
    pub fn joint_node(&self, skin: u32, joint: u32) -> Option<u32> {
        match self.skins.get(skin as usize) {
            Some(s) => s.joints.get(joint as usize).copied(),
            None if self.skins.is_empty() && (joint as usize) < self.nodes.len() => Some(joint),
            None => None,
        }
    }

    /// Model-space transforms of a pose (uniform-scale composition).
    pub fn model_space(&self, pose: &[Trs]) -> Vec<Trs> {
        let mut out: Vec<Trs> = Vec::with_capacity(pose.len());
        for (i, n) in self.nodes.iter().enumerate() {
            let local = pose[i];
            let m = if n.parent == NONE {
                local
            } else {
                out[n.parent as usize].mul(&local)
            };
            out.push(m);
        }
        out
    }

    /// Model-space matrices of a pose (exact glTF composition).
    pub fn model_matrices(&self, pose: &[Trs]) -> Vec<Mat4> {
        let mut out: Vec<Mat4> = Vec::with_capacity(pose.len());
        for (i, n) in self.nodes.iter().enumerate() {
            let local = pose[i].to_mat4();
            let m = if n.parent == NONE {
                local
            } else {
                out[n.parent as usize] * local
            };
            out.push(m);
        }
        out
    }

    /// The joint palette of skin `skin` for a pose: per joint, its model
    /// matrix times its inverse bind matrix (glTF's joint matrix, in the
    /// skeleton's model space).
    pub fn palette(&self, skin: u32, pose: &[Trs], out: &mut Vec<Mat4>) {
        out.clear();
        let Some(skin) = self.skins.get(skin as usize) else {
            return;
        };
        let model = self.model_matrices(pose);
        out.extend(
            skin.joints
                .iter()
                .zip(&skin.inverse_bind)
                .map(|(&j, ibm)| model[j as usize] * *ibm),
        );
    }
}

/// What a channel animates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChannelPath {
    Translation,
    Rotation,
    Scale,
    /// Morph-target weights of the node's mesh (one lane per target).
    Weights,
}

impl ChannelPath {
    pub fn code(self) -> u32 {
        match self {
            ChannelPath::Translation => 0,
            ChannelPath::Rotation => 1,
            ChannelPath::Scale => 2,
            ChannelPath::Weights => 3,
        }
    }

    pub fn from_code(code: u32) -> Option<ChannelPath> {
        Some(match code {
            0 => ChannelPath::Translation,
            1 => ChannelPath::Rotation,
            2 => ChannelPath::Scale,
            3 => ChannelPath::Weights,
            _ => return None,
        })
    }
}

/// One animated property, sampled at the clip's rate.
#[derive(Clone, Debug, PartialEq)]
pub struct Channel {
    /// The target node's index in the source glTF (cooked topological order).
    pub node: u32,
    /// The target node's name: clips bind to a skeleton by name, then by index.
    pub node_name: String,
    pub path: ChannelPath,
    /// The source sampler was STEP: hold each sample until the next one.
    pub step: bool,
    /// Floats per sample (3 translation/scale, 4 rotation xyzw, n weights).
    pub lanes: u32,
    /// `frames × lanes` floats.
    pub samples: Vec<f32>,
}

/// A named marker at a time of a clip.
#[derive(Clone, Debug, PartialEq)]
pub struct ClipEvent {
    /// At most 64 bytes of UTF-8 (it travels in `og.AnimationEvent.name`).
    pub name: String,
    /// Seconds from the clip's start, within `[0, duration]`.
    pub time: f32,
}

/// A clip: channels resampled at `rate` samples per second, from 0 to
/// `duration` (the last sample is at `duration` exactly).
#[derive(Clone, Debug, PartialEq)]
pub struct ClipAsset {
    pub name: String,
    /// Seconds.
    pub duration: f32,
    /// The resampling rate (samples per second).
    pub rate: f32,
    /// The source's own key rate (1 / its smallest key interval), as authored.
    pub source_rate: f32,
    /// Samples per channel: `ceil(duration × rate) + 1`, at least 1.
    pub frames: u32,
    pub channels: Vec<Channel>,
    /// Sorted by time.
    pub events: Vec<ClipEvent>,
}

/// The sample pair and blend factor for a time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FramePos {
    pub i0: usize,
    pub i1: usize,
    pub frac: f32,
}

impl ClipAsset {
    /// The frame count `rate` and `duration` imply.
    pub fn frames_for(duration: f32, rate: f32) -> u32 {
        let n = (duration as f64 * rate as f64 - 1e-4).ceil().max(0.0) as u32;
        n + 1
    }

    pub fn validate(&self) -> Result<()> {
        let what = |r: String| invalid(format!("clip '{}': {r}", self.name));
        if !(self.rate > 0.0 && self.rate.is_finite()) {
            return Err(what(format!("rate {} is not positive", self.rate)));
        }
        if !(self.duration >= 0.0 && self.duration.is_finite()) {
            return Err(what(format!("duration {} is not finite", self.duration)));
        }
        if self.frames != ClipAsset::frames_for(self.duration, self.rate) {
            return Err(what(format!(
                "{} frames for {} s at {} Hz (expected {})",
                self.frames,
                self.duration,
                self.rate,
                ClipAsset::frames_for(self.duration, self.rate)
            )));
        }
        for (c, ch) in self.channels.iter().enumerate() {
            let lanes = match ch.path {
                ChannelPath::Translation | ChannelPath::Scale => 3,
                ChannelPath::Rotation => 4,
                ChannelPath::Weights => ch.lanes.max(1),
            };
            if ch.lanes != lanes {
                return Err(what(format!(
                    "channel {c} has {} lanes, a {:?} channel has {lanes}",
                    ch.lanes, ch.path
                )));
            }
            if ch.samples.len() != (self.frames * ch.lanes) as usize {
                return Err(what(format!(
                    "channel {c} has {} floats, expected {} frames × {} lanes",
                    ch.samples.len(),
                    self.frames,
                    ch.lanes
                )));
            }
        }
        for e in &self.events {
            if e.name.is_empty() || e.name.len() > 64 || e.name.contains('\0') {
                return Err(what(format!(
                    "event '{}' must be 1..=64 bytes without NUL",
                    e.name
                )));
            }
            if !(0.0..=self.duration).contains(&e.time) {
                return Err(what(format!(
                    "event '{}' at {} s is outside 0..={}",
                    e.name, e.time, self.duration
                )));
            }
        }
        if self.events.windows(2).any(|w| w[0].time > w[1].time) {
            return Err(what("events are not sorted by time".into()));
        }
        Ok(())
    }

    /// The samples around `t` (clamped to `[0, duration]`). Times within a
    /// thousandth of a frame of a sample land on it, so a time computed as
    /// `i / rate` reads sample `i` exactly.
    pub fn frame_pos(&self, t: f32) -> FramePos {
        let last = self.frames.saturating_sub(1) as usize;
        let t = t.clamp(0.0, self.duration);
        let x = t * self.rate;
        let i = ((x + 1e-3).floor().max(0.0) as usize).min(last);
        if i >= last {
            return FramePos {
                i0: last,
                i1: last,
                frac: 0.0,
            };
        }
        let t0 = i as f32 / self.rate;
        let t1 = ((i + 1) as f32 / self.rate).min(self.duration);
        let span = t1 - t0;
        let frac = if span > 0.0 {
            ((t - t0) / span).clamp(0.0, 1.0)
        } else {
            0.0
        };
        FramePos {
            i0: i,
            i1: i + 1,
            frac,
        }
    }

    fn lane(ch: &Channel, i: usize, k: usize) -> f32 {
        ch.samples[i * ch.lanes as usize + k]
    }

    /// A translation or scale channel at `p`.
    pub fn vec3(ch: &Channel, p: FramePos) -> Vec3 {
        let a = Vec3::new(
            Self::lane(ch, p.i0, 0),
            Self::lane(ch, p.i0, 1),
            Self::lane(ch, p.i0, 2),
        );
        if ch.step || p.frac == 0.0 {
            return a;
        }
        let b = Vec3::new(
            Self::lane(ch, p.i1, 0),
            Self::lane(ch, p.i1, 1),
            Self::lane(ch, p.i1, 2),
        );
        a + (b - a) * p.frac
    }

    /// A rotation channel at `p` (normalised lerp between samples).
    pub fn quat(ch: &Channel, p: FramePos) -> Quat {
        let q = |i| {
            Quat::from_xyzw(
                Self::lane(ch, i, 0),
                Self::lane(ch, i, 1),
                Self::lane(ch, i, 2),
                Self::lane(ch, i, 3),
            )
        };
        let a = q(p.i0);
        if ch.step || p.frac == 0.0 {
            return unit(a);
        }
        nlerp(a, q(p.i1), p.frac)
    }

    /// Lane `k` of a weights channel at `p`.
    pub fn weight(ch: &Channel, p: FramePos, k: usize) -> f32 {
        let a = Self::lane(ch, p.i0, k);
        if ch.step || p.frac == 0.0 {
            return a;
        }
        a + (Self::lane(ch, p.i1, k) - a) * p.frac
    }
}

fn invalid(reason: String) -> Error {
    Error::new(ErrorKind::Invalid, reason)
}

/// Which skeleton node each channel of a clip drives ([`NONE`]: unbound).
/// By name first, then by index when the names agree or are empty.
pub fn bind(skeleton: &SkeletonAsset, clip: &ClipAsset) -> Vec<u32> {
    clip.channels
        .iter()
        .map(|ch| {
            let by_index = skeleton.nodes.get(ch.node as usize);
            if by_index.is_some_and(|n| n.name == ch.node_name) {
                return ch.node;
            }
            if !ch.node_name.is_empty() {
                if let Some(i) = skeleton.node(&ch.node_name) {
                    return i;
                }
                return NONE;
            }
            if by_index.is_some() {
                ch.node
            } else {
                NONE
            }
        })
        .collect()
}
