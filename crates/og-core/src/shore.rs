//! The seabed under a sea (W1): how deep the water is and how far a point
//! lies from the waterline, sampled on a grid over a rectangle of the
//! surface's xz, so the waves of [`crate::ocean::Ocean`] feel the depth:
//! each is shoaled by `tanh(k d)` and the whole sea capped at the depth's
//! breaking height (`H_s ≤ 0.78 d`, the surf zone's saturation, McCowan
//! 1894), and shore trains ([`ShoreTrains`]) arrive parallel to the beach,
//! their phase the distance to the waterline (after Crest's shoreline
//! waves and the Assassin's Creed IV distance-field shore waves).
//!
//! The distance is Felzenszwalb and Huttenlocher's exact Euclidean distance
//! transform of sampled functions (2012), run twice (to the nearest dry and
//! to the nearest wet sample) and signed: positive out to sea, negative on
//! land, zero half a cell between a wet and a dry sample. It is integer
//! sweeps over f64 squared distances and a square root, so the same seabed
//! gives the same field bit for bit on every host: a [`Shore`] is derived
//! state of the world (the terrain and the water it is made from are world
//! state), and the tick floats bodies on the waves it shapes.

use crate::glam::{Vec2, Vec4};

/// A sea's seabed over a grid. See the module documentation.
#[derive(Clone, Debug, PartialEq)]
pub struct Shore {
    /// The xz (the surface's frame) of sample (0, 0), and the grid's cell (m).
    pub origin: Vec2,
    pub cell: f32,
    pub columns: u32,
    pub rows: u32,
    /// Per sample, row-major from -z: the water's depth under the rest level
    /// (m; ≤ 0 is dry), the signed distance to the waterline (m; positive
    /// out to sea) and that distance's gradient (x, z; unit away from the
    /// shore where the field is one shore's, shorter where two meet).
    pub texels: Vec<Vec4>,
}

/// The seabed at a point ([`Shore::sample`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShorePoint {
    /// The water's depth under the rest level (m; ≤ 0: dry).
    pub depth: f32,
    /// The signed distance to the waterline (m; positive out to sea).
    pub distance: f32,
    /// The direction the shore trains travel (unit, xz: towards the shore).
    pub inshore: Vec2,
    /// How much the field is one shore's (the gradient's length, smoothed:
    /// 0 where two shores' distances meet, where trains would cross).
    pub coherence: f32,
    /// How far inside the grid (1 from eight cells in; 0 at its edge, where
    /// the seabed is unknown and the sea is the open sea).
    pub inside: f32,
}

/// The squared distance of a sample with no target: far past any grid's,
/// and small enough that a grid's squared indices add to it exactly.
const FAR: f64 = 1e12;

/// Felzenszwalb–Huttenlocher's lower envelope of parabolas over `f` (the
/// squared distance of each sample along one line), in place.
fn edt_1d(f: &mut [f64], v: &mut [usize], z: &mut [f64], d: &mut [f64]) {
    let n = f.len();
    if n == 0 {
        return;
    }
    let mut k = 0usize;
    v[0] = 0;
    z[0] = -f64::INFINITY;
    z[1] = f64::INFINITY;
    for q in 1..n {
        let qf = q as f64;
        // The parabolas the new one hides are dropped (z[0] is -∞, so the
        // first is never).
        let s = loop {
            let pf = v[k] as f64;
            let s = ((f[q] + qf * qf) - (f[v[k]] + pf * pf)) / (2.0 * qf - 2.0 * pf);
            if s <= z[k] && k > 0 {
                k -= 1;
            } else {
                break s;
            }
        };
        k += 1;
        v[k] = q;
        z[k] = s;
        z[k + 1] = f64::INFINITY;
    }
    k = 0;
    for (q, out) in d.iter_mut().enumerate().take(n) {
        let qf = q as f64;
        while z[k + 1] < qf {
            k += 1;
        }
        let p = v[k] as f64;
        *out = (qf - p) * (qf - p) + f[v[k]];
    }
    f.copy_from_slice(&d[..n]);
}

/// The squared distance (in cells) of every sample to the nearest one
/// `target` holds, over a `columns` × `rows` grid.
fn edt(target: &[bool], columns: usize, rows: usize) -> Vec<f64> {
    let mut g: Vec<f64> = target.iter().map(|&t| if t { 0.0 } else { FAR }).collect();
    let n = columns.max(rows);
    let (mut f, mut v, mut z, mut d) = (
        vec![0.0; n],
        vec![0usize; n],
        vec![0.0; n + 1],
        vec![0.0; n],
    );
    for r in 0..rows {
        let line = &mut f[..columns];
        line.copy_from_slice(&g[r * columns..(r + 1) * columns]);
        edt_1d(line, &mut v, &mut z, &mut d);
        g[r * columns..(r + 1) * columns].copy_from_slice(line);
    }
    for c in 0..columns {
        for r in 0..rows {
            f[r] = g[r * columns + c];
        }
        edt_1d(&mut f[..rows], &mut v, &mut z, &mut d);
        for r in 0..rows {
            g[r * columns + c] = f[r];
        }
    }
    g
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl Shore {
    /// The seabed of `depth` (row-major from -z, `columns` × `rows`
    /// samples `cell` metres apart, sample (0, 0) at `origin`): its
    /// distances and their gradient. `None` for an empty or mis-sized grid.
    pub fn new(origin: Vec2, cell: f32, columns: u32, rows: u32, depth: &[f32]) -> Option<Shore> {
        let (c, r) = (columns as usize, rows as usize);
        if c < 2 || r < 2 || depth.len() != c * r || !(cell.is_finite() && cell > 0.0) {
            return None;
        }
        let wet: Vec<bool> = depth.iter().map(|&d| d > 0.0).collect();
        let dry: Vec<bool> = wet.iter().map(|w| !w).collect();
        let to_dry = edt(&dry, c, r);
        let to_wet = edt(&wet, c, r);
        // Half a cell between a wet and a dry sample is the waterline.
        let cap = (c.max(r) as f64) * 2.0;
        let distance: Vec<f32> = (0..c * r)
            .map(|i| {
                let s = if wet[i] {
                    to_dry[i].sqrt().min(cap) - 0.5
                } else {
                    -(to_wet[i].sqrt().min(cap) - 0.5)
                };
                (s * f64::from(cell)) as f32
            })
            .collect();
        let at = |x: usize, z: usize| distance[z * c + x];
        let texels = (0..r)
            .flat_map(|z| (0..c).map(move |x| (x, z)))
            .map(|(x, z)| {
                let (x0, x1) = (x.saturating_sub(1), (x + 1).min(c - 1));
                let (z0, z1) = (z.saturating_sub(1), (z + 1).min(r - 1));
                let gx = (at(x1, z) - at(x0, z)) / ((x1 - x0) as f32 * cell);
                let gz = (at(x, z1) - at(x, z0)) / ((z1 - z0) as f32 * cell);
                let d = depth[z * c + x];
                Vec4::new(if d.is_finite() { d } else { 0.0 }, at(x, z), gx, gz)
            })
            .collect();
        Some(Shore {
            origin,
            cell,
            columns,
            rows,
            texels,
        })
    }

    /// The seabed at `p` (the surface's frame xz), bilinear between the
    /// samples (water.wgsl `shore_at` reads the same texels the same way);
    /// `None` outside the grid.
    pub fn sample(&self, p: Vec2) -> Option<ShorePoint> {
        let (c, r) = (self.columns as usize, self.rows as usize);
        let g = (p - self.origin) / self.cell;
        let (hx, hz) = ((c - 1) as f32, (r - 1) as f32);
        if !(g.x >= 0.0 && g.y >= 0.0 && g.x <= hx && g.y <= hz) {
            return None;
        }
        let i = Vec2::new(g.x.floor().min(hx - 1.0), g.y.floor().min(hz - 1.0));
        let t = g - i;
        let (x, z) = (i.x as usize, i.y as usize);
        let at = |x: usize, z: usize| self.texels[z * c + x];
        let lo = at(x, z) + (at(x + 1, z) - at(x, z)) * t.x;
        let hi = at(x, z + 1) + (at(x + 1, z + 1) - at(x, z + 1)) * t.x;
        let v = lo + (hi - lo) * t.y;
        let grad = Vec2::new(v.z, v.w);
        let len = grad.length();
        let edge = g.x.min(g.y).min(hx - g.x).min(hz - g.y);
        Some(ShorePoint {
            depth: v.x,
            distance: v.y,
            inshore: if len > 1e-6 { -grad / len } else { Vec2::ZERO },
            coherence: smoothstep(0.5, 0.9, len),
            inside: smoothstep(0.0, 8.0, edge),
        })
    }
}

/// The waves that roll in on a shore ([`crate::ocean::Ocean::surface_near`]):
/// one Gerstner train whose crests are the lines of equal distance to the
/// waterline, travelling inshore, rising where the water is shallower than
/// half its length and leaning over as it nears its breaking depth.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShoreTrains {
    /// Its wavenumber (rad/m) and angular frequency (rad/s, the deep
    /// water's: its crests keep their spacing up the beach).
    pub k: f32,
    pub omega: f32,
    /// Its height amplitude out where it starts (m; 0: no trains).
    pub amplitude: f32,
    /// Its phase at time zero (rad).
    pub phase: f32,
}

impl ShoreTrains {
    /// The trains of a sea of significant height `hs` (m), `strength` times
    /// a quarter of it as their amplitude (0: none), `length` metres from
    /// crest to crest (0: 40 m).
    pub fn new(hs: f32, strength: f32, length: f32) -> ShoreTrains {
        let length = if length.is_finite() && length > 0.5 {
            length
        } else {
            40.0
        };
        let k = std::f32::consts::TAU / length;
        let strength = if strength.is_finite() {
            strength.max(0.0)
        } else {
            0.0
        };
        ShoreTrains {
            k,
            omega: (crate::ocean::GRAVITY * k).sqrt(),
            amplitude: 0.25 * hs.max(0.0) * strength,
            phase: 0.0,
        }
    }

    /// How much of the trains a point of `depth` holds: none deeper than
    /// half their length, all from a quarter of it inshore.
    pub fn rise(&self, depth: f32) -> f32 {
        let half = std::f32::consts::PI / self.k;
        1.0 - smoothstep(0.5 * half, half, depth)
    }

    /// How far they lean at `depth` (Gerstner's Q, before the open sea's
    /// share is taken out): rising to 0.9 as the water shallows to a
    /// twentieth of their length.
    pub fn steepness(&self, depth: f32) -> f32 {
        let half = std::f32::consts::PI / self.k;
        0.9 * (1.0 - smoothstep(0.1 * half, half, depth))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The distance is the true Euclidean one, signed, with the waterline
    /// half a cell between wet and dry: a disc of land in a sea.
    #[test]
    fn the_distance_is_euclidean_and_signed_at_the_waterline() {
        let n = 65u32;
        let centre = Vec2::splat(32.0);
        let depth: Vec<f32> = (0..n * n)
            .map(|i| {
                let p = Vec2::new((i % n) as f32, (i / n) as f32);
                // Land within 10 cells of the centre.
                if p.distance(centre) <= 10.0 {
                    -1.0
                } else {
                    5.0
                }
            })
            .collect();
        let s = Shore::new(Vec2::ZERO, 2.0, n, n, &depth).expect("a shore");
        // A wet sample k cells from the nearest dry one is k - 0.5 from the
        // waterline (times the 2 m cell).
        let far = s.sample(Vec2::new(64.0 * 2.0, 32.0 * 2.0)).unwrap();
        assert!(
            (far.distance - (64.0 - 42.0 - 0.5) * 2.0).abs() < 1e-3,
            "{far:?}"
        );
        // Diagonally: Euclidean, not city-block.
        let diag = s.sample(Vec2::new(52.0 * 2.0, 52.0 * 2.0)).unwrap();
        let want = (Vec2::splat(52.0).distance(centre) - 10.0) * 2.0;
        assert!(
            (diag.distance - want).abs() < 2.0,
            "{} vs {want}",
            diag.distance
        );
        // Trains travel inshore: towards the centre.
        let towards = (centre * 2.0 - Vec2::new(104.0, 104.0)).normalize();
        assert!(diag.inshore.dot(towards) > 0.98, "{:?}", diag.inshore);
        assert!(diag.coherence > 0.99);
        // Land is negative; the centre the most.
        let land = s.sample(centre * 2.0).unwrap();
        assert!(land.distance < -15.0 && land.depth < 0.0, "{land:?}");
        // The same seabed, the same field.
        assert_eq!(s, Shore::new(Vec2::ZERO, 2.0, n, n, &depth).unwrap());
        // Outside the grid there is none; near its edge it fades.
        assert!(s.sample(Vec2::new(-1.0, 3.0)).is_none());
        assert_eq!(s.sample(Vec2::new(0.0, 64.0)).unwrap().inside, 0.0);
        assert_eq!(s.sample(Vec2::splat(64.0)).unwrap().inside, 1.0);
    }

    /// The exact transform agrees with brute force on a scatter of land.
    #[test]
    fn the_transform_matches_brute_force() {
        let (c, r) = (23usize, 17usize);
        let mut target = vec![false; c * r];
        for (i, t) in target.iter_mut().enumerate() {
            *t = (i * 7919) % 31 == 0;
        }
        let got = edt(&target, c, r);
        for (i, &got) in got.iter().enumerate() {
            let (x, z) = ((i % c) as f64, (i / c) as f64);
            let want = (0..c * r)
                .filter(|&j| target[j])
                .map(|j| {
                    let (a, b) = ((j % c) as f64, (j / c) as f64);
                    (a - x) * (a - x) + (b - z) * (b - z)
                })
                .fold(f64::MAX, f64::min);
            assert_eq!(got, want, "at {i}");
        }
    }
}
