//! A blast's push on the pieces near it.

use super::breaking::Settings;
use super::debris::{Crumb, Debris};
use crate::destruction::Damage;
use glam::Vec3;
use std::collections::BTreeMap;

/// How unevenly a blast pushes the pieces near it: each takes its push
/// times 1 ± SHARE / 2.
pub const SHARE: f32 = 0.6;

/// A blast's turn on a piece: this much of its push's speed over its size
/// (rad/s per m/s per m), enough to open its joints as it goes, not to
/// spin it into what stands.
pub const SPIN: f32 = 0.1;

/// The fastest turn (rad/s) a blast gives a piece.
pub const TURN_MAX: f32 = 12.0;

/// A fixed scramble of `x` (splitmix64's finaliser, public domain): a
/// piece's own share of a blast, the same on every run and host.
pub fn scramble(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// A blast's push on a piece of mass `m` at `at` (its velocity and turn),
/// `None` where it does not reach. A blast's front is not a smooth field
/// near the charge: each piece there takes its own share of the push (up
/// to SHARE either way) and a small turn from the uneven pressure on its
/// face, both fading with the push to none at the pier feet it barely
/// reaches, so the blocks of a wall go their own ways rather than in step
/// and the piers keep their footing. Fixed per piece by `first`, its first
/// chunk, so the same chunk of a model takes the same share in any scene.
pub fn blast_push(
    falloff: &Damage,
    impulse: f32,
    at: Vec3,
    m: f32,
    density: f32,
    first: u64,
) -> Option<(Vec3, Vec3)> {
    let j = falloff.at(at) * face(m / density);
    if m <= 0.0 || j <= 0.0 {
        return None;
    }
    let r = scramble(first);
    let u = |s: u32| ((r >> s) & 0x3ff) as f32 / 1023.0;
    let axis = Vec3::new(u(0) - 0.5, u(10) - 0.5, u(20) - 0.5).normalize_or(Vec3::X);
    let dir = (at - falloff.point).normalize_or(Vec3::Y);
    let near = falloff.at(at) / impulse;
    let dv = dir * (j / m) * (1.0 + near * SHARE * (u(40) - 0.5));
    let size = og_core::math::pow(m / density, 1.0 / 3.0).max(0.02);
    let dw = axis * (near * SPIN * dv.length() / size).min(TURN_MAX);
    Some((dv, dw))
}

/// The face a piece of `volume` m³ shows a blast (m²): a blast's push is
/// its impulse per square metre times that, so a brick is thrown faster
/// than a run of twenty bricks, by the cube root of twenty, not twenty
/// times (a blast pushes on surfaces: its momentum to a piece grows with
/// the piece's face, not as one shove per body however small).
pub fn face(volume: f32) -> f32 {
    og_core::math::pow(volume.max(0.0), 2.0 / 3.0)
}

/// An explosion: its point, inner radius, radius and impulse.
pub type Explosion = (Vec3, f32, f32, f32);

/// The blasts' push on the fine pieces broken off this tick, as on a body
/// (the pieces already on the GPU are the renderer's), and on the grit,
/// which goes with the piece beside its joint.
pub fn push_blasted_pieces(
    settings: &BTreeMap<u64, Settings>,
    explosions: &[Explosion],
    debris: &mut [(Debris, f32)],
    crumbs: &mut [(Crumb, f32)],
) {
    for &(point, inner, radius, impulse) in explosions {
        if impulse <= 0.0 {
            continue;
        }
        let falloff = Damage {
            point,
            min_radius: inner,
            max_radius: radius,
            amount: impulse,
        };
        for (d, m) in debris.iter_mut() {
            let density = settings.get(&d.prop).map_or(1000.0, |s| s.mat.density);
            let at = Vec3::new(d.pose[0], d.pose[1], d.pose[2]);
            if let Some((dv, dw)) =
                blast_push(&falloff, impulse, at, *m, density, u64::from(d.chunk))
            {
                d.velocity = (Vec3::from(d.velocity) + dv).to_array();
                d.angular = (Vec3::from(d.angular) + dw).to_array();
            }
        }
        // Grit goes with the piece beside its joint.
        for (c, m) in crumbs.iter_mut() {
            let density = settings.get(&c.prop).map_or(1000.0, |s| s.mat.density);
            let at = Vec3::from(c.point);
            if let Some((dv, _)) = blast_push(&falloff, impulse, at, *m, density, u64::from(c.bond))
            {
                c.velocity = (Vec3::from(c.velocity) + dv).to_array();
            }
        }
    }
}
