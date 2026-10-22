//! Paths: a spline through (or controlled by) points, measured by arc
//! length so `t` 0..1 moves along it at an even speed whatever the spacing
//! of its points.
//!
//! - `catmullRom`: through every point, as three.js's `CatmullRomCurve3`
//!   with `curveType "catmullrom"`: a cardinal spline whose `tension` (0.5
//!   is Catmull-Rom) scales the tangents; an open path's ends are
//!   extrapolated as three.js does.
//! - `bezier`: cubic Bezier segments, `p0 c0 c1 p1 c2 c3 p2 ...` (3n + 1
//!   points open; 3n closed, the last segment returning to `p0`), as
//!   three.js's `CubicBezierCurve3` chained in a `CurvePath`.
//! - `linear`: straight segments, as three.js's `LineCurve3` in a `CurvePath`.
//!
//! The arc length is tabulated at [`SAMPLES`] chords per segment when the
//! path is made; [`Path::sample`] finds the segment and its parameter by a
//! binary search of that table. Only `+ − × ÷` and `sqrt`, so a path gives
//! the same bits on every host.

use og_core::glam::Vec3;

/// Chords per segment in the arc-length table.
pub const SAMPLES: usize = 32;

/// How the points make the curve.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PathKind {
    #[default]
    CatmullRom,
    Bezier,
    Linear,
}

impl PathKind {
    /// The IDL name (`catmullRom`, `bezier`, `linear`).
    pub fn name(self) -> &'static str {
        match self {
            PathKind::CatmullRom => "catmullRom",
            PathKind::Bezier => "bezier",
            PathKind::Linear => "linear",
        }
    }

    pub fn from_name(name: &str) -> Option<PathKind> {
        Some(match name {
            "catmullRom" => PathKind::CatmullRom,
            "bezier" => PathKind::Bezier,
            "linear" => PathKind::Linear,
            _ => return None,
        })
    }

    /// The code (the IDL enum's variant index).
    pub fn code(self) -> u32 {
        self as u32
    }

    pub fn from_code(code: u32) -> Option<PathKind> {
        [PathKind::CatmullRom, PathKind::Bezier, PathKind::Linear]
            .get(code as usize)
            .copied()
    }
}

/// What makes a path.
#[derive(Clone, Debug, PartialEq)]
pub struct PathDesc {
    pub points: Vec<Vec3>,
    /// The last point joins the first.
    pub closed: bool,
    pub kind: PathKind,
    /// Catmull-Rom's tangent scale (three.js's `tension`, 0.5).
    pub tension: f32,
}

impl Default for PathDesc {
    fn default() -> Self {
        PathDesc {
            points: Vec::new(),
            closed: false,
            kind: PathKind::CatmullRom,
            tension: 0.5,
        }
    }
}

/// A point of a path.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathSample {
    pub position: Vec3,
    /// The unit direction of travel.
    pub tangent: Vec3,
    /// A unit up perpendicular to the tangent: world +Y made perpendicular
    /// (world +Z where the path runs straight up or down).
    pub up: Vec3,
}

/// The path's point nearest another point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nearest {
    /// Its arc-length parameter, 0..1.
    pub t: f32,
    pub position: Vec3,
    pub distance: f32,
}

/// A path with its arc-length table.
#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    desc: PathDesc,
    segments: usize,
    /// Cumulative length at every table sample: `segments * SAMPLES + 1`.
    cum: Vec<f32>,
}

impl Path {
    /// Checks the points against the kind and tabulates the arc length.
    pub fn new(desc: PathDesc) -> Result<Path, String> {
        let n = desc.points.len();
        if desc.points.iter().any(|p| !p.is_finite()) {
            return Err("a path's points must be finite".into());
        }
        if !desc.tension.is_finite() {
            return Err("a path's tension must be finite".into());
        }
        let segments = match desc.kind {
            PathKind::CatmullRom | PathKind::Linear => {
                if n < 2 {
                    return Err(format!(
                        "a {} path needs at least 2 points (it has {n})",
                        desc.kind.name()
                    ));
                }
                if desc.closed { n } else { n - 1 }
            }
            PathKind::Bezier => {
                if desc.closed {
                    if n < 3 || !n.is_multiple_of(3) {
                        return Err(format!(
                            "a closed bezier path has 3 points per segment (p0 c0 c1, p1 c2 c3, ...; it has {n})"
                        ));
                    }
                    n / 3
                } else {
                    if n < 4 || !(n - 1).is_multiple_of(3) {
                        return Err(format!(
                            "an open bezier path has 3n + 1 points (p0 c0 c1 p1 c2 c3 p2 ...; it has {n})"
                        ));
                    }
                    (n - 1) / 3
                }
            }
        };
        let mut path = Path {
            desc,
            segments,
            cum: Vec::with_capacity(segments * SAMPLES + 1),
        };
        path.cum.push(0.0);
        let mut total = 0.0f32;
        for i in 0..segments {
            let mut prev = path.point(i, 0.0);
            for k in 1..=SAMPLES {
                let p = path.point(i, k as f32 / SAMPLES as f32);
                total += p.distance(prev);
                path.cum.push(total);
                prev = p;
            }
        }
        Ok(path)
    }

    pub fn desc(&self) -> &PathDesc {
        &self.desc
    }

    /// Its length along the curve (the chords of the table).
    pub fn length(&self) -> f32 {
        *self.cum.last().unwrap_or(&0.0)
    }

    pub fn segments(&self) -> usize {
        self.segments
    }

    fn at(&self, k: isize) -> Vec3 {
        let p = &self.desc.points;
        let n = p.len() as isize;
        p[k.rem_euclid(n) as usize]
    }

    /// The Catmull-Rom control points of segment `i` (three.js's ends).
    fn cardinal(&self, i: usize) -> [Vec3; 4] {
        let p = &self.desc.points;
        let n = p.len();
        let i = i as isize;
        if self.desc.closed {
            return [self.at(i - 1), self.at(i), self.at(i + 1), self.at(i + 2)];
        }
        let p0 = if i > 0 {
            self.at(i - 1)
        } else {
            p[0] * 2.0 - p[1]
        };
        let p3 = if (i + 2) < n as isize {
            self.at(i + 2)
        } else {
            p[n - 1] * 2.0 - p[n - 2]
        };
        [p0, self.at(i), self.at(i + 1), p3]
    }

    /// The cubic's coefficients `c0 + c1 s + c2 s² + c3 s³` of segment `i`.
    fn cubic(&self, i: usize) -> [Vec3; 4] {
        match self.desc.kind {
            PathKind::CatmullRom => {
                let [p0, p1, p2, p3] = self.cardinal(i);
                let k = self.desc.tension;
                let (t0, t1) = ((p2 - p0) * k, (p3 - p1) * k);
                [
                    p1,
                    t0,
                    p1 * -3.0 + p2 * 3.0 - t0 * 2.0 - t1,
                    p1 * 2.0 - p2 * 2.0 + t0 + t1,
                ]
            }
            PathKind::Bezier => {
                let b = 3 * i as isize;
                let (a, c0, c1, d) = (self.at(b), self.at(b + 1), self.at(b + 2), self.at(b + 3));
                [
                    a,
                    (c0 - a) * 3.0,
                    (a - c0 * 2.0 + c1) * 3.0,
                    d - a + (c0 - c1) * 3.0,
                ]
            }
            PathKind::Linear => {
                let (a, b) = (self.at(i as isize), self.at(i as isize + 1));
                [a, b - a, Vec3::ZERO, Vec3::ZERO]
            }
        }
    }

    /// Segment `i`'s point at its own parameter `s` (0..1, not arc length).
    pub fn point(&self, i: usize, s: f32) -> Vec3 {
        let [c0, c1, c2, c3] = self.cubic(i);
        c0 + (c1 + (c2 + c3 * s) * s) * s
    }

    /// Segment `i`'s derivative at `s`.
    fn derivative(&self, i: usize, s: f32) -> Vec3 {
        let [_, c1, c2, c3] = self.cubic(i);
        c1 + (c2 * 2.0 + c3 * (3.0 * s)) * s
    }

    /// The segment and its parameter at `distance` along the path.
    fn locate(&self, distance: f32) -> (usize, f32) {
        let cum = &self.cum;
        let last = cum.len() - 1;
        if distance <= 0.0 || self.length() <= 0.0 {
            return (0, 0.0);
        }
        if distance >= cum[last] {
            return (self.segments - 1, 1.0);
        }
        // The last sample at or before the distance.
        let (mut lo, mut hi) = (0usize, last);
        while hi - lo > 1 {
            let mid = (lo + hi) / 2;
            if cum[mid] <= distance {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let span = cum[hi] - cum[lo];
        let f = if span > 0.0 {
            (distance - cum[lo]) / span
        } else {
            0.0
        };
        let seg = (lo / SAMPLES).min(self.segments - 1);
        let s = ((lo - seg * SAMPLES) as f32 + f) / SAMPLES as f32;
        (seg, s.clamp(0.0, 1.0))
    }

    /// The arc length at segment `i`'s parameter `s`.
    fn distance_at(&self, i: usize, s: f32) -> f32 {
        let g = s * SAMPLES as f32;
        let k = (g as usize).min(SAMPLES - 1);
        let j = i * SAMPLES + k;
        self.cum[j] + (self.cum[j + 1] - self.cum[j]) * (g - k as f32)
    }

    fn sample_at(&self, i: usize, s: f32) -> PathSample {
        let position = self.point(i, s);
        let mut tangent = self.derivative(i, s).normalize_or_zero();
        if tangent == Vec3::ZERO {
            // A Bezier control on its end point: the chord to a neighbour.
            let (a, b) = if s < 0.5 {
                (position, self.point(i, s + 1.0 / SAMPLES as f32))
            } else {
                (self.point(i, s - 1.0 / SAMPLES as f32), position)
            };
            tangent = (b - a).normalize_or(Vec3::Z);
        }
        PathSample {
            position,
            tangent,
            up: up_of(tangent),
        }
    }

    /// The point at arc-length parameter `t` (0 the start, 1 the end; a
    /// closed path wraps, an open one clamps).
    pub fn sample(&self, t: f32) -> PathSample {
        let t = self.wrap(t);
        let (i, s) = self.locate(t * self.length());
        self.sample_at(i, s)
    }

    /// The point at `distance` along the path (wrapped or clamped as `sample`).
    pub fn sample_distance(&self, distance: f32) -> PathSample {
        let len = self.length();
        if len <= 0.0 {
            return self.sample_at(0, 0.0);
        }
        self.sample(distance / len)
    }

    fn wrap(&self, t: f32) -> f32 {
        if !t.is_finite() {
            return 0.0;
        }
        if self.desc.closed {
            let w = t - t.floor();
            // 1 is the start again; keep it the end so a lap's last tick is at the join.
            if w == 0.0 && t > 0.0 { 1.0 } else { w }
        } else {
            t.clamp(0.0, 1.0)
        }
    }

    /// The point of the path nearest `target`: the table's nearest sample,
    /// then a bisection along its segment.
    pub fn nearest(&self, target: Vec3) -> Nearest {
        let mut best = (f32::MAX, 0usize, 0.0f32);
        for i in 0..self.segments {
            for k in 0..=SAMPLES {
                let s = k as f32 / SAMPLES as f32;
                let d = self.point(i, s).distance_squared(target);
                if d < best.0 {
                    best = (d, i, s);
                }
            }
        }
        let (_, i, mut s) = best;
        let mut step = 1.0 / SAMPLES as f32;
        let d = |s: f32| self.point(i, s).distance_squared(target);
        for _ in 0..16 {
            let (a, b) = ((s - step).max(0.0), (s + step).min(1.0));
            let (da, db, ds) = (d(a), d(b), d(s));
            if da < ds && da <= db {
                s = a;
            } else if db < ds {
                s = b;
            }
            step *= 0.5;
        }
        let position = self.point(i, s);
        let len = self.length();
        Nearest {
            t: if len > 0.0 {
                (self.distance_at(i, s) / len).clamp(0.0, 1.0)
            } else {
                0.0
            },
            position,
            distance: position.distance(target),
        }
    }
}

/// World +Y made perpendicular to `tangent` (world +Z when they are parallel).
pub fn up_of(tangent: Vec3) -> Vec3 {
    let up = Vec3::Y - tangent * tangent.dot(Vec3::Y);
    if up.length_squared() > 1e-6 {
        up.normalize()
    } else {
        (Vec3::Z - tangent * tangent.dot(Vec3::Z)).normalize_or(Vec3::Z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(kind: PathKind, closed: bool, points: &[[f32; 3]]) -> Path {
        Path::new(PathDesc {
            points: points.iter().map(|p| Vec3::from(*p)).collect(),
            closed,
            kind,
            tension: 0.5,
        })
        .unwrap()
    }

    #[test]
    fn a_linear_path_is_measured_by_arc_length_not_by_points() {
        // Two segments, 1 m and 3 m: t 0.5 is 2 m along, inside the second.
        let p = path(
            PathKind::Linear,
            false,
            &[[0., 0., 0.], [1., 0., 0.], [1., 3., 0.]],
        );
        assert!((p.length() - 4.0).abs() < 1e-5);
        let s = p.sample(0.5);
        assert!(
            s.position.distance(Vec3::new(1.0, 1.0, 0.0)) < 1e-5,
            "{s:?}"
        );
        assert!(s.tangent.distance(Vec3::Y) < 1e-5);
        // The up of a vertical run is +Z, never NaN.
        assert!(s.up.distance(Vec3::Z) < 1e-5, "{s:?}");
        assert_eq!(p.sample(2.0).position, Vec3::new(1.0, 3.0, 0.0));
        assert_eq!(p.sample(-1.0).position, Vec3::ZERO);
    }

    #[test]
    fn catmull_rom_passes_through_its_points_at_their_distances() {
        let pts = [[0., 0., 0.], [4., 0., 2.], [8., 1., 0.], [12., 0., -2.]];
        let p = path(PathKind::CatmullRom, false, &pts);
        for (i, q) in pts.iter().enumerate() {
            assert!(
                p.point(i.min(2), if i == 3 { 1.0 } else { 0.0 })
                    .distance(Vec3::from(*q))
                    < 1e-5
            );
        }
        // Even speed: equal steps of t cover equal lengths (within the chords' error).
        let mut prev = p.sample(0.0).position;
        let steps = 50;
        let mut lens = Vec::new();
        for k in 1..=steps {
            let q = p.sample(k as f32 / steps as f32).position;
            lens.push(q.distance(prev));
            prev = q;
        }
        let mean = p.length() / steps as f32;
        for l in lens {
            assert!((l - mean).abs() < mean * 0.02, "{l} vs {mean}");
        }
    }

    #[test]
    fn bezier_segments_end_on_their_anchors_and_closed_paths_wrap() {
        let p = path(
            PathKind::Bezier,
            false,
            &[[0., 0., 0.], [0., 2., 0.], [4., 2., 0.], [4., 0., 0.]],
        );
        assert_eq!(p.sample(0.0).position, Vec3::ZERO);
        assert!(p.sample(1.0).position.distance(Vec3::new(4.0, 0.0, 0.0)) < 1e-5);
        // Symmetric: half way along is the top, heading +X.
        let mid = p.sample(0.5);
        assert!(
            (mid.position.x - 2.0).abs() < 1e-3 && (mid.position.y - 1.5).abs() < 1e-3,
            "{mid:?}"
        );
        assert!(mid.tangent.distance(Vec3::X) < 1e-3);
        assert!(
            Path::new(PathDesc {
                points: vec![Vec3::ZERO; 5],
                kind: PathKind::Bezier,
                ..PathDesc::default()
            })
            .is_err()
        );
        let ring = path(
            PathKind::Linear,
            true,
            &[[0., 0., 0.], [2., 0., 0.], [2., 0., 2.], [0., 0., 2.]],
        );
        assert!((ring.length() - 8.0).abs() < 1e-5);
        assert!(
            ring.sample(1.25)
                .position
                .distance(ring.sample(0.25).position)
                < 1e-5
        );
        assert!(
            ring.sample(0.875)
                .position
                .distance(Vec3::new(0.0, 0.0, 1.0))
                < 1e-5
        );
    }

    #[test]
    fn nearest_answers_the_point_and_its_parameter() {
        let p = path(PathKind::Linear, false, &[[0., 0., 0.], [10., 0., 0.]]);
        let n = p.nearest(Vec3::new(3.3, 2.0, 0.0));
        assert!((n.t - 0.33).abs() < 1e-4, "{n:?}");
        assert!((n.distance - 2.0).abs() < 1e-4);
        let c = path(
            PathKind::CatmullRom,
            false,
            &[[0., 0., 0.], [5., 0., 5.], [10., 0., 0.]],
        );
        let at = c.sample(0.7);
        let n = c.nearest(at.position + at.up * 0.5);
        assert!((n.t - 0.7).abs() < 2e-3, "{n:?}");
    }
}
