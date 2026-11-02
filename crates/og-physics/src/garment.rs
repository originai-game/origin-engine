//! Garments on skinned characters (og.Garment): a mesh's points posed by
//! the animation each tick (the targets the host hands in with
//! [`crate::PhysicsSystem::set_garment_pose`]) and simulated from there by
//! position, like og.Cloth: gravity, a gusting wind and the motion's drag,
//! then passes of edge and bending constraints, each point held within its
//! reach of its target (its rig `freedom` × `maxDistance`; 0 within
//! `pinTop` of the top) and no further than `backstop` behind it along the
//! skin's normal, and solids pushed out at `radius`. One-way, as cloth.
//!
//! Determinism: `+ - * /` and `sqrt` on f32 in a fixed order (edges and
//! bends ascending, points in index order), the gust from an integer hash
//! of (point, tick); the state is the points with the edges they were
//! built with, saved in the og.physics block's trailer.

use crate::schema_ext::Garment;
use crate::verlet::Solids;
use og_core::glam::Vec3;
use og_core::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What a garment is built from (an asset, like a ragdoll rig): its points
/// at rest in the model's space (the bind pose; vertices at the same
/// position, split along a seam, welded into one), its triangles over
/// them, and how free each point is, 0 (it stays on the skin) to 1 (it
/// may stray `maxDistance`): the vertex colour's red.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GarmentRig {
    pub rest: Vec<Vec3>,
    pub triangles: Vec<[u32; 3]>,
    pub freedom: Vec<f32>,
}

/// Where the skin puts a garment's points this tick, in the world, one per
/// rig point, with the skin's normals there, and the scale the model is
/// drawn at (its rest lengths grow with it).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GarmentPose {
    pub targets: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub scale: f32,
}

/// An og.Garment's points after the last step and the targets that step
/// held them to, in the world, in rig order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GarmentPoints {
    pub points: Vec<Vec3>,
    pub targets: Vec<Vec3>,
}

/// A garment's points as the last step left them, and what they were
/// built from (a new rig, or a new `pinTop`, rebuilds it on the skin).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct GarmentState {
    /// The rig's key ([`crate::PhysicsSystem::set_garment_rig`]) and the
    /// `pinTop` bits it was built with.
    pub key: (u64, u32),
    pub points: Vec<Vec3>,
    pub previous: Vec<Vec3>,
    /// The targets of the last step (a substep moves its targets from the
    /// previous tick's towards this tick's).
    pub targets: Vec<Vec3>,
    /// Each point's reach as a share of `maxDistance` (0: on the skin).
    reach: Vec<f32>,
    /// The edges (a, b, rest length at scale 1), a < b, ascending.
    edges: Vec<(u32, u32, f32)>,
    /// Across each pair of triangles sharing an edge: their far corners
    /// (a, b, rest length at scale 1), a < b, ascending.
    bends: Vec<(u32, u32, f32)>,
}

impl GarmentRig {
    /// Checks the rig's lengths and indices.
    pub fn check(&self) -> Result<()> {
        let n = self.rest.len();
        if n == 0 || self.freedom.len() != n {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "garment rig: {n} points and {} freedoms (one each, at least one point)",
                    self.freedom.len()
                ),
            ));
        }
        if let Some(t) = self
            .triangles
            .iter()
            .find(|t| t.iter().any(|&i| i as usize >= n))
        {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("garment rig: triangle {t:?} indexes past its {n} points"),
            ));
        }
        if self.rest.iter().any(|p| !p.is_finite()) || self.freedom.iter().any(|f| !f.is_finite()) {
            return Err(Error::new(ErrorKind::Invalid, "garment rig: not finite"));
        }
        Ok(())
    }
}

impl GarmentState {
    /// Built on the skin (every point at its target).
    pub fn new(garment: &Garment, key: u64, rig: &GarmentRig, pose: &GarmentPose) -> GarmentState {
        let rest = &rig.rest;
        let mut edges: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
        for &[a, b, c] in &rig.triangles {
            if a == b || b == c || c == a {
                continue;
            }
            for (p, q, far) in [(a, b, c), (b, c, a), (c, a, b)] {
                edges.entry((p.min(q), p.max(q))).or_default().push(far);
            }
        }
        let length = |a: u32, b: u32| (rest[b as usize] - rest[a as usize]).length();
        let mut bends = Vec::new();
        for far in edges.values() {
            if let [c, d] = far[..]
                && c != d
            {
                let (a, b) = (c.min(d), c.max(d));
                bends.push((a, b, length(a, b)));
            }
        }
        bends.sort_by_key(|&(a, b, _)| (a, b));
        bends.dedup_by_key(|&mut (a, b, _)| (a, b));
        let edges = edges.keys().map(|&(a, b)| (a, b, length(a, b))).collect();
        let top = rest.iter().fold(f32::MIN, |m, p| m.max(p.y));
        let pin = garment.pin_top.max(0.0);
        let reach = rest
            .iter()
            .zip(&rig.freedom)
            .map(|(p, f)| {
                if pin > 0.0 && p.y >= top - pin {
                    0.0
                } else {
                    f.clamp(0.0, 1.0)
                }
            })
            .collect();
        GarmentState {
            key: (key, garment.pin_top.to_bits()),
            points: pose.targets.clone(),
            previous: pose.targets.clone(),
            targets: pose.targets.clone(),
            reach,
            edges,
            bends,
        }
    }

    /// One tick of `substeps` steps towards `pose`: integrate with gravity
    /// and a gusting wind, then `iterations` passes of edges, bends, the
    /// reach and backstop around the targets (moved from the last tick's
    /// to this one's over the substeps) and solids; then friction.
    pub fn step(
        &mut self,
        garment: &Garment,
        pose: &GarmentPose,
        dt: f32,
        tick: u64,
        gravity: Vec3,
        solids: &dyn Solids,
    ) {
        let n = self.points.len();
        let substeps = garment.substeps.clamp(1, 16);
        let ds = dt / substeps as f32;
        let damping = garment.damping.clamp(0.0, 1.0);
        let keep = if substeps == 1 {
            damping
        } else {
            og_core::math::pow(damping, 1.0 / substeps as f32)
        };
        let g = gravity * garment.gravity_scale;
        let wind = garment.wind;
        let windy = wind.length_squared() > 0.0;
        let scale = if pose.scale.is_finite() && pose.scale > 0.0 {
            pose.scale
        } else {
            1.0
        };
        let stiffness = garment.stiffness.clamp(0.0, 1.0);
        let bending = garment.bending.clamp(0.0, 1.0);
        let max = garment.max_distance.max(0.0);
        let backstop = garment.backstop.max(0.0);
        let radius = garment.radius.max(0.0);
        let w: Vec<f32> = self
            .reach
            .iter()
            .map(|&r| if r * max > 0.0 { 1.0 } else { 0.0 })
            .collect();
        let from = std::mem::replace(&mut self.targets, pose.targets.clone());
        let mut target = vec![Vec3::ZERO; n];
        for s in 0..substeps {
            let t = (s + 1) as f32 / substeps as f32;
            for (i, at) in target.iter_mut().enumerate() {
                *at = from[i] + (pose.targets[i] - from[i]) * t;
            }
            for i in 0..n {
                let p = self.points[i];
                if w[i] == 0.0 {
                    self.previous[i] = target[i];
                    self.points[i] = target[i];
                    continue;
                }
                let mut a = g;
                if windy {
                    let gusted =
                        wind * (1.0 + garment.gust * crate::verlet::gust(i as u32, 0, tick));
                    let nrm = pose.normals[i];
                    a += nrm * nrm.dot(gusted) + gusted * 0.15;
                }
                let v = (p - self.previous[i]) * keep;
                self.previous[i] = p;
                self.points[i] = p + v + a * (ds * ds);
            }
            for _ in 0..garment.iterations.clamp(1, 64) {
                if stiffness > 0.0 {
                    for &(a, b, rest) in &self.edges {
                        link(&mut self.points, &w, a, b, rest * scale, stiffness);
                    }
                }
                if bending > 0.0 {
                    for &(a, b, rest) in &self.bends {
                        link(&mut self.points, &w, a, b, rest * scale, bending);
                    }
                }
                for i in 0..n {
                    if w[i] == 0.0 {
                        continue;
                    }
                    let before = self.points[i];
                    let mut p = before;
                    // Never further than the backstop behind the skin.
                    let nrm = pose.normals[i];
                    let behind = (p - target[i]).dot(nrm);
                    if behind < -backstop {
                        p += nrm * (-backstop - behind);
                    }
                    // Within its reach of the skin.
                    let reach = self.reach[i] * max;
                    let off = p - target[i];
                    let len = off.length();
                    if len > reach {
                        p = target[i] + off * (reach / len);
                    }
                    // Solids last: a body backed into a wall keeps its
                    // cape outside it, beyond its reach if need be.
                    if radius > 0.0
                        && let Some((out, _)) = solids.push_out(p, radius)
                    {
                        p = out;
                    }
                    // The skin carries the point (its velocity is its
                    // own): a fast move drags the hem without flinging it.
                    self.previous[i] += p - before;
                    self.points[i] = p;
                }
            }
            let friction = garment.friction.clamp(0.0, 1.0);
            if friction > 0.0 && radius > 0.0 {
                for (i, &wi) in w.iter().enumerate().take(n) {
                    let p = self.points[i];
                    if wi > 0.0
                        && let Some((_, nrm)) = solids.push_out(p, radius * 1.05)
                    {
                        let v = p - self.previous[i];
                        let slide = v - nrm * nrm.dot(v);
                        self.previous[i] += slide * friction;
                    }
                }
            }
        }
    }
}

/// Moves `a` and `b` towards `rest` apart by `k` of the error, their
/// weights `w` (0: held to the skin).
fn link(points: &mut [Vec3], w: &[f32], a: u32, b: u32, rest: f32, k: f32) {
    let (a, b) = (a as usize, b as usize);
    let (wa, wb) = (w[a], w[b]);
    let sum = wa + wb;
    if sum <= 0.0 {
        return;
    }
    let d = points[b] - points[a];
    let len = d.length();
    if len <= 1e-9 {
        return;
    }
    let c = k * (len - rest) / (len * sum);
    points[a] += d * (c * wa);
    points[b] -= d * (c * wb);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Nothing;
    impl Solids for Nothing {
        fn push_out(&self, _: Vec3, _: f32) -> Option<(Vec3, Vec3)> {
            None
        }
    }

    /// A skirt: a ring of `around` points per row, `rows` rows down from
    /// y 1, radius 0.3, its normals outwards.
    fn skirt(around: u32, rows: u32) -> (GarmentRig, GarmentPose) {
        let mut rest = Vec::new();
        let mut normals = Vec::new();
        for r in 0..rows {
            for k in 0..around {
                let a = std::f32::consts::TAU * k as f32 / around as f32;
                let (s, c) = (og_core::math::sin(a), og_core::math::cos(a));
                rest.push(Vec3::new(c * 0.3, 1.0 - r as f32 * 0.1, s * 0.3));
                normals.push(Vec3::new(c, 0.0, s));
            }
        }
        let mut triangles = Vec::new();
        for r in 0..rows - 1 {
            for k in 0..around {
                let a = r * around + k;
                let b = r * around + (k + 1) % around;
                let (c, d) = (a + around, b + around);
                triangles.push([a, c, b]);
                triangles.push([b, c, d]);
            }
        }
        let n = rest.len();
        let rig = GarmentRig {
            freedom: vec![1.0; n],
            rest: rest.clone(),
            triangles,
        };
        let pose = GarmentPose {
            targets: rest,
            normals,
            scale: 1.0,
        };
        (rig, pose)
    }

    fn moved(pose: &GarmentPose, by: Vec3) -> GarmentPose {
        GarmentPose {
            targets: pose.targets.iter().map(|p| *p + by).collect(),
            ..pose.clone()
        }
    }

    /// A cape: `columns` × `rows` points 0.1 apart in the xy plane at
    /// z -0.2, from y 1 down, its normals away from the body (-z).
    fn cape(columns: u32, rows: u32) -> (GarmentRig, GarmentPose) {
        let mut rest = Vec::new();
        for r in 0..rows {
            for c in 0..columns {
                rest.push(Vec3::new(c as f32 * 0.1, 1.0 - r as f32 * 0.1, -0.2));
            }
        }
        let mut triangles = Vec::new();
        for r in 0..rows - 1 {
            for c in 0..columns - 1 {
                let a = r * columns + c;
                let (b, d) = (a + 1, a + columns);
                triangles.push([a, d, b]);
                triangles.push([b, d, d + 1]);
            }
        }
        let n = rest.len();
        let rig = GarmentRig {
            freedom: vec![1.0; n],
            rest: rest.clone(),
            triangles,
        };
        let pose = GarmentPose {
            targets: rest,
            normals: vec![Vec3::NEG_Z; n],
            scale: 1.0,
        };
        (rig, pose)
    }

    /// Held to the skin at the top, free below: walked forward (+z) at
    /// 3 m/s, the collar stays on the skin and the cape's hem trails
    /// behind within its reach (the damping drags it, as air would), then
    /// swings back down onto the skin once the walk stops.
    #[test]
    fn the_hem_trails_behind_the_collar_within_its_reach() {
        let (rig, pose) = cape(7, 8);
        rig.check().unwrap();
        let g = Garment::default();
        let down = Vec3::new(0.0, -9.8, 0.0);
        let mut st = GarmentState::new(&g, 7, &rig, &pose);
        // Across, down and one diagonal a cell, 6 × 8 + 7 × 7 + 6 × 7.
        assert_eq!(st.edges.len(), 6 * 8 + 7 * 7 + 6 * 7);
        assert!(!st.bends.is_empty());
        let mut to = pose.clone();
        for t in 1..=30 {
            to = moved(&pose, Vec3::new(0.0, 0.0, 0.05 * t as f32));
            st.step(&g, &to, 1.0 / 60.0, t, down, &Nothing);
        }
        let hem = 7 * 7 + 3;
        assert_eq!(st.points[3], to.targets[3], "the collar is pinned");
        let lag = to.targets[hem].z - st.points[hem].z;
        assert!(lag > 0.03 && lag <= g.max_distance + 1e-4, "hem lags {lag}");
        for _ in 0..240 {
            st.step(&g, &to, 1.0 / 60.0, 0, down, &Nothing);
        }
        let lag = (to.targets[hem] - st.points[hem]).length();
        assert!(lag < 0.02, "settled {lag} from the skin");
    }

    /// Gravity hangs a free hem down, but never behind the skin: on a body
    /// bent over (the skirt lying along +x), the lower half sags out from
    /// the skin while the upper half, pulled into the body, stays on it.
    #[test]
    fn gravity_hangs_it_but_the_backstop_keeps_it_out_of_the_body() {
        let (rig, mut pose) = skirt(12, 6);
        let turn = og_core::glam::Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2);
        for (p, n) in pose.targets.iter_mut().zip(&mut pose.normals) {
            *p = turn * *p;
            *n = turn * *n;
        }
        let g = Garment {
            pin_top: 0.0,
            ..Garment::default()
        };
        let mut rig = rig;
        for k in 0..12 {
            rig.freedom[k] = 0.0;
        }
        let mut st = GarmentState::new(&g, 7, &rig, &pose);
        for _ in 0..120 {
            st.step(
                &g,
                &pose,
                1.0 / 60.0,
                0,
                Vec3::new(0.0, -9.8, 0.0),
                &Nothing,
            );
        }
        let mut sank = 0.0f32;
        let mut dropped = 0.0f32;
        for i in 12..st.points.len() {
            let off = st.points[i] - pose.targets[i];
            sank = sank.min(off.dot(pose.normals[i]));
            dropped = dropped.max(-off.y);
            assert!(off.length() <= g.max_distance + 1e-4);
        }
        assert!(sank > -1e-4, "a point sank {sank} behind the skin");
        // Stiff edges around the ring let the lower half sag only so far
        // (about 1.6 cm here); a gravity of the wrong sign sags it none.
        assert!(dropped > 0.01, "gravity moved nothing ({dropped})");
    }

    /// The same poses step to the same points (bit for bit), and a state
    /// restored from a copy goes on as the original.
    #[test]
    fn steps_are_deterministic() {
        let (rig, pose) = skirt(16, 8);
        let g = Garment {
            wind: Vec3::new(3.0, 0.0, 1.0),
            ..Garment::default()
        };
        let run = |ticks: u64| {
            let mut st = GarmentState::new(&g, 1, &rig, &pose);
            for t in 0..ticks {
                let p = moved(
                    &pose,
                    Vec3::new(og_core::math::sin(t as f32 * 0.1), 0.0, 0.0),
                );
                st.step(&g, &p, 1.0 / 60.0, t, Vec3::new(0.0, -9.8, 0.0), &Nothing);
            }
            st
        };
        let a = run(90);
        assert_eq!(a, run(90));
        let mut b = run(45);
        let copy: GarmentState = bincode::deserialize(&bincode::serialize(&b).unwrap()).unwrap();
        b = copy;
        for t in 45..90 {
            let p = moved(
                &pose,
                Vec3::new(og_core::math::sin(t as f32 * 0.1), 0.0, 0.0),
            );
            b.step(&g, &p, 1.0 / 60.0, t, Vec3::new(0.0, -9.8, 0.0), &Nothing);
        }
        assert_eq!(a, b);
    }
}
