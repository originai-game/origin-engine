//! Poses and the operations the system composes them with: sampling a clip,
//! blending, additive layering, root-motion extraction and clip markers.
//!
//! **Visibility.** A pose carries a visible flag per node (a visibility
//! channel's, else the skeleton's rest visibility). It is a flag, not a
//! quantity, so blends choose rather than mix: a blend of two poses takes
//! the flag of the heavier (`lerp_to` by `t`: the other pose's from
//! `t ≥ 0.5`, per node by `t · mask` for a masked layer), a blend of many
//! ([`Blend`]) the flag of the heaviest input, the later on a tie; an
//! additive layer sets a node's flag to its clip's where the clip's frame
//! differs from its reference frame (the layer toggles it) and the layer's
//! weight there is at least 0.5. A cross-fade between two mesh-swap clips
//! therefore swaps at its middle instead of drawing both frames at once.

use crate::asset::{ChannelPath, ClipAsset, NONE, SkeletonAsset};
use crate::schema::{BLENDSHAPES, ROOT_MOTION_FULL};
use crate::xform::{Trs, nlerp, twist_y};
use og_core::glam::{Quat, Vec3};

/// Every node's local transform and visible flag, plus the blendshape
/// weights of one mesh.
#[derive(Clone, Debug, PartialEq)]
pub struct Pose {
    pub nodes: Vec<Trs>,
    pub weights: [f32; BLENDSHAPES],
    /// Per node: drawn unless false (with its ancestors',
    /// [`SkeletonAsset::drawn`]).
    pub visible: Vec<bool>,
}

impl Pose {
    pub fn rest(skeleton: &SkeletonAsset) -> Pose {
        Pose {
            nodes: skeleton.rest_pose(),
            weights: [0.0; BLENDSHAPES],
            visible: skeleton.rest_visibility(),
        }
    }

    /// Moves `self` towards `other` by `t`; the visible flags are
    /// `other`'s from `t ≥ 0.5`.
    pub fn lerp_to(&mut self, other: &Pose, t: f32) {
        if t <= 0.0 {
            return;
        }
        if t >= 1.0 {
            self.clone_from(other);
            return;
        }
        self.lerp_values(other, t);
        if t >= 0.5 {
            self.visible.clone_from(&other.visible);
        }
    }

    /// [`Pose::lerp_to`] without the visible flags.
    fn lerp_values(&mut self, other: &Pose, t: f32) {
        for (a, b) in self.nodes.iter_mut().zip(&other.nodes) {
            *a = a.lerp(b, t);
        }
        for (a, b) in self.weights.iter_mut().zip(&other.weights) {
            *a += (*b - *a) * t;
        }
    }

    /// [`Pose::lerp_to`] with node `i` moved by `t · mask[i]` (blendshape
    /// weights by `t`), taking `other`'s flag where that is at least 0.5.
    pub fn lerp_to_masked(&mut self, other: &Pose, t: f32, mask: &[f32]) {
        if t <= 0.0 {
            return;
        }
        for (i, ((a, b), &m)) in self
            .nodes
            .iter_mut()
            .zip(&other.nodes)
            .zip(mask)
            .enumerate()
        {
            let k = t * m;
            if k >= 1.0 {
                *a = *b;
            } else if k > 0.0 {
                *a = a.lerp(b, k);
            }
            if k >= 0.5
                && let (Some(v), Some(&o)) = (self.visible.get_mut(i), other.visible.get(i))
            {
                *v = o;
            }
        }
        let t = t.min(1.0);
        for (a, b) in self.weights.iter_mut().zip(&other.weights) {
            *a += (*b - *a) * t;
        }
    }

    /// [`Pose::add`] with node `i`'s difference scaled by `w · mask[i]`
    /// (blendshape weights by `w`).
    pub fn add_masked(&mut self, pose: &Pose, reference: &Pose, w: f32, mask: &[f32]) {
        if w <= 0.0 {
            return;
        }
        for (i, (((base, p), r), &m)) in self
            .nodes
            .iter_mut()
            .zip(&pose.nodes)
            .zip(&reference.nodes)
            .zip(mask)
            .enumerate()
        {
            if m > 0.0 {
                add_node(base, p, r, w * m);
                if w * m >= 0.5 {
                    toggle(&mut self.visible, pose, reference, i);
                }
            }
        }
        self.add_weights(pose, reference, w);
    }

    /// Adds `pose`'s difference from `reference`, scaled by `w`.
    pub fn add(&mut self, pose: &Pose, reference: &Pose, w: f32) {
        if w <= 0.0 {
            return;
        }
        for ((base, p), r) in self.nodes.iter_mut().zip(&pose.nodes).zip(&reference.nodes) {
            add_node(base, p, r, w);
        }
        if w >= 0.5 {
            for i in 0..self.visible.len() {
                toggle(&mut self.visible, pose, reference, i);
            }
        }
        self.add_weights(pose, reference, w);
    }

    fn add_weights(&mut self, pose: &Pose, reference: &Pose, w: f32) {
        for ((b, p), r) in self
            .weights
            .iter_mut()
            .zip(&pose.weights)
            .zip(&reference.weights)
        {
            *b += (*p - *r) * w;
        }
    }
}

/// An additive layer's visibility: node `i` takes `pose`'s flag where it
/// differs from `reference`'s.
fn toggle(visible: &mut [bool], pose: &Pose, reference: &Pose, i: usize) {
    if let (Some(v), Some(&p), Some(&r)) = (
        visible.get_mut(i),
        pose.visible.get(i),
        reference.visible.get(i),
    ) && p != r
    {
        *v = p;
    }
}

/// Adds `p`'s difference from `r` to `base`, scaled by `w`.
fn add_node(base: &mut Trs, p: &Trs, r: &Trs, w: f32) {
    base.translation += (p.translation - r.translation) * w;
    let dr = r.rotation.conjugate() * p.rotation;
    base.rotation = crate::xform::unit(base.rotation * nlerp(Quat::IDENTITY, dr, w));
    let ds = Vec3::new(
        ratio(p.scale.x, r.scale.x),
        ratio(p.scale.y, r.scale.y),
        ratio(p.scale.z, r.scale.z),
    );
    base.scale *= Vec3::ONE + (ds - Vec3::ONE) * w;
}

fn ratio(a: f32, b: f32) -> f32 {
    if b == 0.0 { 1.0 } else { a / b }
}

/// Accumulates weighted poses (and root deltas) into their normalised blend,
/// in submission order; the visible flags are the heaviest pose's (the
/// later of equal weights).
pub struct Blend {
    pub pose: Option<Pose>,
    pub delta: Trs,
    total: f32,
    /// The weight of the pose the flags came from.
    heaviest: f32,
}

impl Blend {
    pub fn new() -> Blend {
        Blend {
            pose: None,
            delta: Trs::IDENTITY,
            total: 0.0,
            heaviest: 0.0,
        }
    }

    pub fn push(&mut self, pose: Pose, delta: Trs, w: f32) {
        if w <= 0.0 {
            return;
        }
        let t = w / (self.total + w);
        match &mut self.pose {
            None => {
                self.pose = Some(pose);
                self.delta = delta;
                self.heaviest = w;
            }
            Some(p) => {
                if t >= 1.0 {
                    p.clone_from(&pose);
                } else {
                    p.lerp_values(&pose, t);
                }
                if w >= self.heaviest {
                    p.visible.clone_from(&pose.visible);
                    self.heaviest = w;
                }
                self.delta = self.delta.lerp(&delta, t);
            }
        }
        self.total += w;
    }
}

/// How a skeleton's root motion is extracted.
#[derive(Clone, Debug)]
pub struct RootMotion {
    pub mode: u32,
    pub node: u32,
    /// The root's ancestors and itself, top first.
    pub chain: Vec<u32>,
    /// The planar part of the root's rest transform in model space.
    pub reference: Trs,
}

impl RootMotion {
    pub fn new(skeleton: &SkeletonAsset, node: u32, mode: u32) -> RootMotion {
        let mut chain = vec![node];
        let mut n = node;
        while skeleton.nodes[n as usize].parent != NONE {
            n = skeleton.nodes[n as usize].parent;
            chain.push(n);
        }
        chain.reverse();
        let mut rm = RootMotion {
            mode,
            node,
            chain,
            reference: Trs::IDENTITY,
        };
        let rest = skeleton.rest_pose();
        let m = rm.model_of(&rest);
        rm.reference = rm.planar(&m);
        rm
    }

    /// The root's model-space transform in a full pose.
    fn model_of(&self, pose: &[Trs]) -> Trs {
        let mut m = Trs::IDENTITY;
        for &n in &self.chain {
            m = m.mul(&pose[n as usize]);
        }
        m
    }

    /// The part of a model-space transform root motion moves the entity by.
    fn planar(&self, m: &Trs) -> Trs {
        if self.mode == ROOT_MOTION_FULL {
            Trs::new(m.translation, m.rotation, Vec3::ONE)
        } else {
            Trs::new(
                Vec3::new(m.translation.x, 0.0, m.translation.z),
                twist_y(m.rotation),
                Vec3::ONE,
            )
        }
    }

    /// The root's model transform when `clip` is sampled at `t` (only the
    /// chain's channels are read).
    fn sample_model(
        &self,
        skeleton: &SkeletonAsset,
        clip: &ClipAsset,
        binding: &[u32],
        t: f32,
    ) -> Trs {
        let mut local: Vec<Trs> = self
            .chain
            .iter()
            .map(|&n| skeleton.nodes[n as usize].rest)
            .collect();
        let p = clip.frame_pos(t);
        for (ch, &node) in clip.channels.iter().zip(binding) {
            if let Some(k) = self.chain.iter().position(|&c| c == node) {
                match ch.path {
                    ChannelPath::Translation => local[k].translation = ClipAsset::vec3(ch, p),
                    ChannelPath::Rotation => local[k].rotation = ClipAsset::quat(ch, p),
                    ChannelPath::Scale => local[k].scale = ClipAsset::vec3(ch, p),
                    ChannelPath::Weights | ChannelPath::Visibility => {}
                }
            }
        }
        let mut m = Trs::IDENTITY;
        for l in &local {
            m = m.mul(l);
        }
        m
    }

    /// The entity-frame motion from clip time `a` to `b` (no wrap).
    fn delta(
        &self,
        skeleton: &SkeletonAsset,
        clip: &ClipAsset,
        binding: &[u32],
        a: f32,
        b: f32,
    ) -> Trs {
        let pa = self.planar(&self.sample_model(skeleton, clip, binding, a));
        let pb = self.planar(&self.sample_model(skeleton, clip, binding, b));
        let r = &self.reference;
        let d = r.mul(&pa.inverse()).mul(&pb).mul(&r.inverse());
        Trs::new(d.translation, d.rotation, Vec3::ONE)
    }

    /// Replaces the root's travel in `pose` by the reference's, so the
    /// skeleton stays where the entity is.
    fn pin(&self, pose: &mut [Trs]) {
        let m = self.model_of(pose);
        let p = self.planar(&m);
        let pinned = self.reference.mul(&p.inverse()).mul(&m);
        let mut parent = Trs::IDENTITY;
        for &n in &self.chain[..self.chain.len() - 1] {
            parent = parent.mul(&pose[n as usize]);
        }
        pose[self.node as usize] = parent.inverse().mul(&pinned);
    }
}

/// One playing clip this tick: where its time went and how much it counts.
pub struct Leaf<'a> {
    pub handle: u64,
    pub clip: &'a ClipAsset,
    pub binding: &'a [u32],
    pub t0: f32,
    pub t1: f32,
    /// Times the playback wrapped (signed: negative when playing backwards).
    pub wraps: i32,
    pub weight: f32,
}

/// A marker a leaf passed.
#[derive(Clone, Debug, PartialEq)]
pub struct Fired {
    pub clip: u64,
    pub name: String,
    pub time: f32,
    pub weight: f32,
}

impl Leaf<'_> {
    /// The clip-time spans the playback covered, in order.
    fn spans(&self) -> Vec<(f32, f32)> {
        let len = self.clip.duration;
        let w = self.wraps;
        if w == 0 {
            return vec![(self.t0, self.t1)];
        }
        // Full cycles in between count once (a tick never spans more in practice).
        if w > 0 {
            let mut s = vec![(self.t0, len)];
            if w > 1 {
                s.push((0.0, len));
            }
            s.push((0.0, self.t1));
            s
        } else {
            let mut s = vec![(self.t0, 0.0)];
            if w < -1 {
                s.push((len, 0.0));
            }
            s.push((len, self.t1));
            s
        }
    }

    /// The pose at `t1` over `base` (the rest pose), its travel pinned when
    /// root motion is on.
    pub fn pose(&self, skeleton: &SkeletonAsset, root: Option<&RootMotion>, mesh: u32) -> Pose {
        self.pose_at(skeleton, root, mesh, self.t1)
    }

    /// The pose at clip time `t` (a stepped player's held time, [`stepped`])
    /// over the rest pose, pinned as [`Leaf::pose`].
    pub fn pose_at(
        &self,
        skeleton: &SkeletonAsset,
        root: Option<&RootMotion>,
        mesh: u32,
        t: f32,
    ) -> Pose {
        let mut pose = Pose::rest(skeleton);
        sample(skeleton, self.clip, self.binding, t, mesh, &mut pose);
        if let Some(r) = root {
            r.pin(&mut pose.nodes);
        }
        pose
    }

    /// The pose at the clip's first frame (the additive reference).
    pub fn reference(&self, skeleton: &SkeletonAsset, mesh: u32) -> Pose {
        let mut pose = Pose::rest(skeleton);
        sample(skeleton, self.clip, self.binding, 0.0, mesh, &mut pose);
        pose
    }

    /// The root's travel over this tick, in the entity's frame.
    pub fn delta(&self, skeleton: &SkeletonAsset, root: Option<&RootMotion>) -> Trs {
        let Some(r) = root else {
            return Trs::IDENTITY;
        };
        let mut d = Trs::IDENTITY;
        for (a, b) in self.spans() {
            d = d.mul(&r.delta(skeleton, self.clip, self.binding, a, b));
        }
        d
    }

    /// The markers passed this tick: forward spans fire `a <= e < b` (and a
    /// marker at the very end when the span reaches it), backward spans
    /// `b < e <= a` (and one at 0 when the span reaches it).
    pub fn events(&self, out: &mut Vec<Fired>) {
        if self.clip.events.is_empty() || self.weight <= 0.0 {
            return;
        }
        let len = self.clip.duration;
        for (a, b) in self.spans() {
            for e in &self.clip.events {
                let t = e.time;
                let hit = if a < b {
                    a <= t && (t < b || (t == b && b >= len))
                } else if a > b {
                    b < t && t <= a || (t == b && b <= 0.0)
                } else {
                    false
                };
                if hit {
                    out.push(Fired {
                        clip: self.handle,
                        name: e.name.clone(),
                        time: t,
                        weight: self.weight,
                    });
                }
            }
        }
    }
}

/// The clip time a stepped player samples (`AnimationPlayer.stepFps`):
/// `t` floored to whole steps of 1/`fps`, so a pose holds for the step
/// whatever the tick rate; `fps` 0 (or less) is `t`. A non-looping clip
/// that reached its end (`t` at `duration`) shows its last frame.
pub fn stepped(t: f32, fps: f32, duration: f32, looping: bool) -> f32 {
    if fps <= 0.0 || (!looping && t >= duration) {
        return t;
    }
    (t * fps).floor() / fps
}

/// Writes `clip` at time `t` into `pose` (channels the binding leaves
/// unbound are skipped; weight channels count when they target a node that
/// draws mesh `mesh`; visibility channels set the node's flag).
pub fn sample(
    skeleton: &SkeletonAsset,
    clip: &ClipAsset,
    binding: &[u32],
    t: f32,
    mesh: u32,
    pose: &mut Pose,
) {
    let p = clip.frame_pos(t);
    for (ch, &node) in clip.channels.iter().zip(binding) {
        if node == NONE {
            continue;
        }
        let n = &mut pose.nodes[node as usize];
        match ch.path {
            ChannelPath::Translation => n.translation = ClipAsset::vec3(ch, p),
            ChannelPath::Rotation => n.rotation = ClipAsset::quat(ch, p),
            ChannelPath::Scale => n.scale = ClipAsset::vec3(ch, p),
            ChannelPath::Visibility => pose.visible[node as usize] = ClipAsset::visible(ch, p),
            ChannelPath::Weights => {
                if mesh != NONE && skeleton.nodes[node as usize].mesh == mesh {
                    for k in 0..(ch.lanes as usize).min(BLENDSHAPES) {
                        pose.weights[k] = ClipAsset::weight(ch, p, k);
                    }
                }
            }
        }
    }
}
