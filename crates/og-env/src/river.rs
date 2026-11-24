//! W5: a river, the water of an og.Water laid along a path: its centreline
//! (the path sampled evenly, xz and the water's level y), its width and its
//! speed. The same record floats bodies in the tick (og-physics'
//! `WaterBody`: the current carries them) and draws the river (the
//! renderer's ribbon and its flow), so a floater drifts with the water it
//! is drawn on, the same on every run.
//!
//! The current runs along the centreline's tangent, fastest mid-stream and
//! slowing to nothing at the banks: across an open channel the surface's
//! speed falls off as the bed's friction holds the water back, a profile
//! close to a power law of the distance from the bank (Chow, *Open-Channel
//! Hydraulics*, 1959, §8-1), here (1 − a²)^¼ of the speed for `a` the share
//! of the half width from the centre.

use og_core::glam::{Vec2, Vec3, Vec3Swizzles};
use std::collections::BTreeMap;

/// A river's centreline, width (m) and mid-stream speed (m/s).
#[derive(Clone, Debug, PartialEq)]
pub struct River {
    /// The centreline's points (frame xz and the water's level y), at
    /// least two, none repeated.
    points: Vec<Vec3>,
    /// The arc length to each point (m, along xz).
    along: Vec<f64>,
    /// Segment AABBs expanded by half width, in canonical segment order.
    grid: BTreeMap<(i32, i32), Vec<usize>>,
    grid_origin: Vec2,
    cell: f32,
    pub width: f32,
    pub speed: f32,
}

/// The river at a point ([`River::at`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RiverPoint {
    /// The water's level there (m).
    pub level: f32,
    /// The current (m/s, xz).
    pub velocity: Vec2,
    /// The centreline's unit tangent there (xz, downstream).
    pub tangent: Vec2,
    /// How far across the river the point is: -1 at its left bank (facing
    /// downstream), 0 mid-stream, 1 at its right.
    pub across: f32,
    /// How far downstream (m along the centreline from its first point).
    pub along: f64,
}

/// How much of the mid-stream speed the current keeps at `across` (the
/// share of the half width from the centre).
pub fn bank_profile(across: f32) -> f32 {
    // The fourth root as two square roots: exact in IEEE arithmetic, the same on every host.
    (1.0 - across * across).max(0.0).sqrt().sqrt()
}

impl River {
    /// A river along `points` (the centreline in order: frame xz and the
    /// water's level y; repeated points are dropped), `width` across (m)
    /// and `speed` mid-stream (m/s). None with fewer than two points or no
    /// width.
    pub fn new(points: &[Vec3], width: f32, speed: f32) -> Option<River> {
        let mut kept: Vec<Vec3> = Vec::with_capacity(points.len());
        for &p in points {
            if !p.is_finite() {
                return None;
            }
            if kept
                .last()
                .is_none_or(|q: &Vec3| q.xz().distance(p.xz()) > 1e-4)
            {
                kept.push(p);
            }
        }
        if kept.len() < 2 || !width.is_finite() || width <= 0.0 {
            return None;
        }
        let mut along = vec![0.0f64];
        for w in kept.windows(2) {
            let d = f64::from(w[0].xz().distance(w[1].xz()));
            along.push(along.last().copied().unwrap_or(0.0) + d);
        }
        let cell = kept
            .windows(2)
            .map(|w| w[0].xz().distance(w[1].xz()))
            .fold(width, f32::max);
        let grid_origin = kept[0].xz();
        let mut grid: BTreeMap<(i32, i32), Vec<usize>> = BTreeMap::new();
        for (i, w) in kept.windows(2).enumerate() {
            let lo = ((w[0].xz().min(w[1].xz()) - Vec2::splat(width * 0.5) - grid_origin) / cell)
                .floor()
                .as_ivec2();
            let hi = ((w[0].xz().max(w[1].xz()) + Vec2::splat(width * 0.5) - grid_origin) / cell)
                .floor()
                .as_ivec2();
            for z in lo.y..=hi.y {
                for x in lo.x..=hi.x {
                    grid.entry((x, z)).or_default().push(i);
                }
            }
        }
        Some(River {
            points: kept,
            along,
            grid,
            grid_origin,
            cell,
            width,
            speed: speed.max(0.0),
        })
    }

    /// The centreline's points (frame xz and level y).
    pub fn points(&self) -> &[Vec3] {
        &self.points
    }

    /// This river shifted in world space by `delta` (for floating-origin rebase).
    pub fn translated(&self, delta: Vec3) -> River {
        River {
            points: self.points.iter().map(|&p| p + delta).collect(),
            along: self.along.clone(),
            grid: self.grid.clone(),
            grid_origin: self.grid_origin + delta.xz(),
            cell: self.cell,
            width: self.width,
            speed: self.speed,
        }
    }

    /// The centreline's length (m, along xz).
    pub fn length(&self) -> f64 {
        self.along.last().copied().unwrap_or(0.0)
    }

    /// The river at frame point `p` (xz): None outside it (further from the
    /// centreline than half its width, or past either end).
    pub fn at(&self, p: Vec2) -> Option<RiverPoint> {
        // The nearest point of the centreline: the segment it lies on, how far
        // along it, and its distance (ties to the earlier segment).
        let key = ((p - self.grid_origin) / self.cell).floor().as_ivec2();
        let mut best: Option<(usize, f32, f32)> = None;
        for &i in self.grid.get(&(key.x, key.y))? {
            let a = self.points[i].xz();
            let b = self.points[i + 1].xz();
            let ab = b - a;
            let t = ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
            let d = p.distance(a + ab * t);
            if best.is_none_or(|(_, _, e)| d < e) {
                best = Some((i, t, d));
            }
        }
        let (i, t, d) = best?;
        let half = self.width * 0.5;
        let a = self.points[i];
        let b = self.points[i + 1];
        let tangent = (b.xz() - a.xz()).normalize();
        let off = p - a.xz();
        let ab = b.xz() - a.xz();
        // Past either end of the river (off the end of its first or last segment).
        let s = off.dot(ab) / ab.length_squared();
        if (i == 0 && s < 0.0) || (i + 2 == self.points.len() && s > 1.0) || d > half {
            return None;
        }
        // Right of the tangent (facing downstream) is +across.
        let side = tangent.x * off.y - tangent.y * off.x;
        let across = (d / half) * if side >= 0.0 { 1.0 } else { -1.0 };
        Some(RiverPoint {
            level: a.y + (b.y - a.y) * t,
            velocity: tangent * (self.speed * bank_profile(across)),
            tangent,
            across,
            along: self.along[i] + (self.along[i + 1] - self.along[i]) * f64::from(t),
        })
    }
}
