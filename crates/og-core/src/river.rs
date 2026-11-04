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

use glam::{DVec2, Vec2, Vec3, Vec3Swizzles};

/// A river's centreline, width (m) and mid-stream speed (m/s).
#[derive(Clone, Debug, PartialEq)]
pub struct River {
    /// The centreline's points (frame xz and the water's level y), at
    /// least two, none repeated.
    points: Vec<Vec3>,
    /// The arc length to each point (m, along xz).
    along: Vec<f64>,
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
        if kept.len() < 2 || width.is_nan() || width <= 0.0 {
            return None;
        }
        let mut along = vec![0.0f64];
        for w in kept.windows(2) {
            let d = f64::from(w[0].xz().distance(w[1].xz()));
            along.push(along.last().copied().unwrap_or(0.0) + d);
        }
        Some(River {
            points: kept,
            along,
            width,
            speed: speed.max(0.0),
        })
    }

    /// The centreline's points (frame xz and level y).
    pub fn points(&self) -> &[Vec3] {
        &self.points
    }

    /// The centreline's length (m, along xz).
    pub fn length(&self) -> f64 {
        self.along.last().copied().unwrap_or(0.0)
    }

    /// The river at frame point `p` (xz): None outside it (further from the
    /// centreline than half its width, or past either end).
    pub fn at(&self, p: Vec2) -> Option<RiverPoint> {
        let q = DVec2::new(f64::from(p.x), f64::from(p.y));
        // The nearest point of the centreline: the segment it lies on, how far
        // along it, and its distance (ties to the earlier segment).
        let mut best: Option<(usize, f64, f64)> = None;
        for (i, w) in self.points.windows(2).enumerate() {
            let a = w[0].xz().as_dvec2();
            let b = w[1].xz().as_dvec2();
            let ab = b - a;
            let t = ((q - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
            let d = q.distance(a + ab * t);
            if best.is_none_or(|(_, _, e)| d < e) {
                best = Some((i, t, d));
            }
        }
        let (i, t, d) = best?;
        let half = f64::from(self.width) * 0.5;
        let a = self.points[i];
        let b = self.points[i + 1];
        let tangent = (b.xz() - a.xz()).normalize();
        let off = q - a.xz().as_dvec2();
        let ab = (b.xz() - a.xz()).as_dvec2();
        // Past either end of the river (off the end of its first or last segment).
        let s = off.dot(ab) / ab.length_squared();
        if (i == 0 && s < 0.0) || (i + 2 == self.points.len() && s > 1.0) || d > half {
            return None;
        }
        // Right of the tangent (facing downstream) is +across.
        let side = tangent.x * off.y as f32 - tangent.y * off.x as f32;
        let across = (d / half) as f32 * if side >= 0.0 { 1.0 } else { -1.0 };
        Some(RiverPoint {
            level: a.y + (b.y - a.y) * t as f32,
            velocity: tangent * (self.speed * bank_profile(across)),
            tangent,
            across,
            along: self.along[i] + (self.along[i + 1] - self.along[i]) * t,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bend() -> River {
        // East 20 m, then north (-z) 20 m, falling 1 m along the way.
        River::new(
            &[
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(20.0, 0.5, 0.0),
                Vec3::new(20.0, 0.0, -20.0),
            ],
            6.0,
            2.0,
        )
        .expect("a river")
    }

    /// The current runs downstream along the centreline, at the full speed
    /// mid-stream, slower towards either bank and none past it or past the
    /// river's ends; its level falls along it.
    #[test]
    fn the_current_runs_down_the_centreline_and_slows_at_the_banks() {
        let r = bend();
        let mid = r.at(Vec2::new(10.0, 0.0)).expect("in the river");
        assert!(
            (mid.velocity - Vec2::new(2.0, 0.0)).length() < 1e-5,
            "{mid:?}"
        );
        assert!((mid.level - 0.75).abs() < 1e-5);
        assert!((mid.along - 10.0).abs() < 1e-9);
        // 2.4 m off the centre (0.8 of the half width), to the right facing east (+z).
        let near_bank = r.at(Vec2::new(10.0, 2.4)).expect("in the river");
        assert!((near_bank.across - 0.8).abs() < 1e-5, "{near_bank:?}");
        let k = near_bank.velocity.length() / 2.0;
        assert!((k - 0.36f32.sqrt().sqrt()).abs() < 1e-4, "{k}");
        assert!(r.at(Vec2::new(10.0, -2.4)).expect("in").across < 0.0);
        assert!(r.at(Vec2::new(10.0, 3.1)).is_none(), "past the bank");
        assert!(r.at(Vec2::new(-0.5, 0.0)).is_none(), "above its source");
        assert!(r.at(Vec2::new(20.0, -20.5)).is_none(), "past its mouth");
        // After the bend it runs north.
        let north = r.at(Vec2::new(20.0, -10.0)).expect("in the river");
        assert!(
            (north.velocity - Vec2::new(0.0, -2.0)).length() < 1e-5,
            "{north:?}"
        );
        assert!((north.along - 30.0).abs() < 1e-9);
        assert!((r.length() - 40.0).abs() < 1e-9);
    }

    /// Fewer than two distinct points, or no width, is no river.
    #[test]
    fn a_river_needs_a_course_and_a_width() {
        assert!(River::new(&[Vec3::ZERO, Vec3::ZERO], 4.0, 1.0).is_none());
        assert!(River::new(&[Vec3::ZERO, Vec3::X], 0.0, 1.0).is_none());
        assert!(River::new(&[Vec3::ZERO, Vec3::new(f32::NAN, 0.0, 0.0)], 4.0, 1.0).is_none());
    }
}
