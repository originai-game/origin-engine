//! Which reflection probe a draw samples (`docs/design.md` §2.6), per
//! instance.
//!
//! An instance samples the smallest probe box holding the centre of its
//! bounds (none: the frame's environment); a skinned instance's bounds are
//! its pose's, its joints' bind-pose boxes moved by its palette
//! (`skin.rs` `posed_centre`). The choice is made per
//! instance and carried by the batch: the instance sort keys on it
//! (`instances.rs` [`crate::instances::Run::probe`], the skinned batches
//! the same), so a mesh and material shared by two rooms are two batches,
//! each binding its own room's probe frame group. A per-instance index in
//! the instance record would need every probe's cube bound at once (a cube
//! array, which WebGL2 lacks, or a texture slot per probe, which WebGL2's
//! skinned lit pipeline has none left for); a batch split costs a draw only
//! where a batch spans rooms.
//!
//! What is not an instance takes a probe per draw: a terrain level, a
//! scatter chunk, a water surface, the 3D fluids ([`Binder::at`] before
//! each, by [`pick_sphere`]: the box holding the draw's centre, else the
//! smallest box its bounds reach into, since such a draw is often larger
//! than a room). Inside every draw the shader blends the probe with the
//! environment per fragment by the distance to the box's faces
//! (`frame.wgsl` `probe_weight`, 0 outside the box), so the part of a
//! draw outside its probe's box shows the environment.

use crate::probes::ReflectionProbe;
use glam::Vec3;

/// No probe: the frame's environment.
pub(crate) const NONE: u8 = u8::MAX;

/// The probes' boxes (min and max corners), in the frame's active order.
pub(crate) fn boxes(probes: &[ReflectionProbe]) -> Vec<(Vec3, Vec3)> {
    probes.iter().map(ReflectionProbe::bounds).collect()
}

/// The probe sampled at `p`: the smallest of `boxes` holding it (boxes
/// nest: a cupboard inside a room), [`NONE`] outside every one.
pub(crate) fn pick(boxes: &[(Vec3, Vec3)], p: Vec3) -> u8 {
    let mut best = (NONE, f32::MAX);
    for (i, &(lo, hi)) in boxes.iter().enumerate() {
        if p.cmpge(lo).all() && p.cmple(hi).all() {
            let s = hi - lo;
            let volume = s.x * s.y * s.z;
            if volume < best.1 {
                best = (i as u8, volume);
            }
        }
    }
    best.0
}

/// The probe a draw bounded by the sphere (`c`, `r`) samples: [`pick`] at
/// its centre, else the smallest box the sphere reaches into.
pub(crate) fn pick_sphere(boxes: &[(Vec3, Vec3)], c: Vec3, r: f32) -> u8 {
    let at = pick(boxes, c);
    if at != NONE {
        return at;
    }
    let mut best = (NONE, f32::MAX);
    for (i, &(lo, hi)) in boxes.iter().enumerate() {
        if c.clamp(lo, hi).distance_squared(c) <= r * r {
            let s = hi - lo;
            let volume = s.x * s.y * s.z;
            if volume < best.1 {
                best = (i as u8, volume);
            }
        }
    }
    best.0
}

/// Whether a sphere is near a probe's box (min, max): within the box's own
/// half extent of it. A capture draws scattered copies only there.
pub(crate) fn near_box((lo, hi): (Vec3, Vec3)) -> impl Fn(Vec3, f32) -> bool {
    let margin = (hi - lo) * 0.5;
    let (lo, hi) = (lo - margin, hi + margin);
    move |c: Vec3, r: f32| c.clamp(lo, hi).distance_squared(c) <= r * r
}

/// A pass's probe frame groups: probe `i`'s at `groups[i]`, the frame's own
/// for [`NONE`].
#[derive(Clone, Copy)]
pub(crate) struct ProbeGroups<'a> {
    pub boxes: &'a [(Vec3, Vec3)],
    pub groups: &'a [wgpu::BindGroup],
    pub frame: &'a wgpu::BindGroup,
}

/// Binds group 0 as the draws of a pass need it, only when it changes.
pub(crate) struct Binder<'a> {
    groups: Option<ProbeGroups<'a>>,
    bound: u8,
}

impl<'a> Binder<'a> {
    /// A pass whose group 0 is the frame's (`None`: a pass without probes,
    /// whose binds do nothing).
    pub fn new(groups: Option<ProbeGroups<'a>>) -> Binder<'a> {
        Binder {
            groups,
            bound: NONE,
        }
    }

    /// Binds probe `probe`'s group (or the frame's).
    pub fn bind(&mut self, pass: &mut wgpu::RenderPass, probe: u8) {
        let Some(g) = self.groups else { return };
        let probe = if (probe as usize) < g.groups.len() {
            probe
        } else {
            NONE
        };
        if probe != self.bound {
            pass.set_bind_group(0, g.groups.get(probe as usize).unwrap_or(g.frame), &[]);
            self.bound = probe;
        }
    }

    /// Binds the group of the probe a draw bounded by (`c`, `r`) samples
    /// ([`pick_sphere`]).
    pub fn at(&mut self, pass: &mut wgpu::RenderPass, c: Vec3, r: f32) {
        if let Some(g) = self.groups {
            self.bind(pass, pick_sphere(g.boxes, c, r));
        }
    }

    /// Puts the frame's group back for what follows.
    pub fn reset(&mut self, pass: &mut wgpu::RenderPass) {
        self.bind(pass, NONE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A point takes the smallest box holding it; one outside every box
    /// takes none; boxes nest (a cupboard inside a room).
    #[test]
    fn a_point_takes_the_smallest_box_holding_it() {
        let room = ReflectionProbe::new(1, Vec3::ZERO, Vec3::splat(10.0));
        let cupboard = ReflectionProbe::new(2, Vec3::new(3.0, 0.0, 0.0), Vec3::splat(2.0));
        let b = boxes(&[room, cupboard]);
        assert_eq!(pick(&b, Vec3::new(-2.0, 0.0, 0.0)), 0);
        assert_eq!(pick(&b, Vec3::new(3.2, 0.1, 0.0)), 1);
        assert_eq!(pick(&b, Vec3::new(40.0, 0.0, 0.0)), NONE);
        assert_eq!(pick(&[], Vec3::ZERO), NONE);
        // A terrain level centred outside the room but reaching into it
        // takes the room; one that does not reach it, none.
        assert_eq!(pick_sphere(&b, Vec3::new(20.0, 0.0, 0.0), 15.5), 0);
        assert_eq!(pick_sphere(&b, Vec3::new(20.0, 0.0, 0.0), 14.0), NONE);
        // Reaching into both, the smaller wins.
        assert_eq!(pick_sphere(&b, Vec3::new(7.0, 0.0, 0.0), 2.5), 0);
        assert_eq!(pick_sphere(&b, Vec3::new(5.5, 0.0, 0.0), 1.6), 1);
    }
}
