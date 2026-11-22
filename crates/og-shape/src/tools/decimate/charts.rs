//! UV islands that move as one under decimation: rigid charts merged so their borders are kept.

use super::*;

/// A rigid move of the UV plane: `p ↦ R p + t`, `R` the rotation with
/// cosine `c` and sine `s`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rigid {
    /// The rotation's cosine.
    pub c: f64,
    /// The rotation's sine.
    pub s: f64,
    /// The translation.
    pub t: DVec2,
}

impl Rigid {
    /// The identity.
    pub const IDENTITY: Rigid = Rigid {
        c: 1.0,
        s: 0.0,
        t: DVec2::ZERO,
    };

    pub(crate) fn rotate(&self, p: DVec2) -> DVec2 {
        DVec2::new(self.c * p.x - self.s * p.y, self.s * p.x + self.c * p.y)
    }

    /// The image of `p`.
    pub fn apply(&self, p: DVec2) -> DVec2 {
        self.rotate(p) + self.t
    }

    /// The point whose image is `q`.
    pub fn unapply(&self, q: DVec2) -> DVec2 {
        let d = q - self.t;
        DVec2::new(self.c * d.x + self.s * d.y, -self.s * d.x + self.c * d.y)
    }

    /// `self ∘ o`: `o` first.
    pub(crate) fn after(&self, o: &Rigid) -> Rigid {
        Rigid {
            c: self.c * o.c - self.s * o.s,
            s: self.s * o.c + self.c * o.s,
            t: self.rotate(o.t) + self.t,
        }
    }

    pub(crate) fn inverse(&self) -> Rigid {
        let r = Rigid {
            c: self.c,
            s: -self.s,
            t: DVec2::ZERO,
        };
        Rigid {
            t: -r.rotate(self.t),
            ..r
        }
    }

    /// The move taking `a0 → a1` and `b0 → b1`, if the two segments have
    /// the same length within `tol` (and are not points).
    pub(crate) fn between(a0: DVec2, b0: DVec2, a1: DVec2, b1: DVec2, tol: f64) -> Option<Rigid> {
        let (u, v) = (b0 - a0, b1 - a1);
        let (lu, lv) = (u.length(), v.length());
        if !(lu > 0.0 && lv > 0.0) || (lu - lv).abs() > tol {
            return None;
        }
        let r = Rigid {
            c: u.dot(v) / (lu * lv),
            s: u.perp_dot(v) / (lu * lv),
            t: DVec2::ZERO,
        };
        Some(Rigid {
            t: a1 - r.rotate(a0),
            ..r
        })
    }

    pub(crate) fn near(&self, o: &Rigid, tol: f64) -> bool {
        (self.c - o.c).abs() <= tol && (self.s - o.s).abs() <= tol && (self.t - o.t).length() <= tol
    }
}

/// The UV charts of `s` for the seam rule (`Decimate::merge_charts`):
/// per triangle, its chart and the rigid move from its UVs to its chart's
/// frame; and the number of islands. Islands are triangles joined across
/// edges whose two sides give both ends the same UVs. Two islands join
/// when every seam edge between them gives one rigid move from one's UVs
/// to the other's (a rotation and a translation, within `tol`: the edge's
/// two lengths agree, and the moves agree), and the move puts the two
/// triangles at each such edge on opposite sides of it (no mirror, no
/// overlap there). Pairs of islands are taken in order, each chart framed
/// by its lowest island; the result depends only on the input.
pub fn merge_charts(s: &Solid, tol: f64) -> Option<(Vec<u32>, Vec<Rigid>, usize)> {
    let uvs = s.uvs.as_ref()?;
    let n = s.triangles.len();
    let at = |t: usize, v: u32| {
        let k = s.triangles[t].iter().position(|&w| w == v).unwrap_or(0);
        uvs[t][k]
    };
    let mut users: BTreeMap<EdgeKey, Vec<u32>> = BTreeMap::new();
    for (i, t) in s.triangles.iter().enumerate() {
        for k in 0..3 {
            users
                .entry(edge_key(t[k], t[(k + 1) % 3]))
                .or_default()
                .push(i as u32);
        }
    }
    fn root(p: &mut [u32], mut i: u32) -> u32 {
        while p[i as usize] != i {
            p[i as usize] = p[p[i as usize] as usize];
            i = p[i as usize];
        }
        i
    }
    let mut parent: Vec<u32> = (0..n as u32).collect();
    let mut seams: Vec<(u32, u32, usize, usize)> = Vec::new();
    for (&(a, b), ts) in &users {
        if ts.len() != 2 {
            continue;
        }
        let (x, y) = (ts[0] as usize, ts[1] as usize);
        if at(x, a) == at(y, a) && at(x, b) == at(y, b) {
            let (rx, ry) = (root(&mut parent, x as u32), root(&mut parent, y as u32));
            if rx != ry {
                parent[rx.max(ry) as usize] = rx.min(ry);
            }
        } else {
            seams.push((a, b, x, y));
        }
    }
    let island: Vec<u32> = (0..n as u32).map(|t| root(&mut parent, t)).collect();
    let islands = (0..n).filter(|&t| island[t] == t as u32).count();
    // Per pair of islands (lower, higher): the move from the lower's UVs
    // to the higher's that each seam between them gives, if any.
    let mut between: BTreeMap<(u32, u32), Vec<Option<Rigid>>> = BTreeMap::new();
    for &(a, b, x, y) in &seams {
        let (ix, iy) = (island[x], island[y]);
        if ix == iy {
            continue;
        }
        let (x, y) = if ix < iy { (x, y) } else { (y, x) };
        let m = Rigid::between(at(x, a), at(x, b), at(y, a), at(y, b), tol).filter(|m| {
            // The two triangles on opposite sides of the edge after the
            // move.
            let third = |t: usize| {
                let tri = s.triangles[t];
                let v = tri[(0..3).find(|&k| tri[k] != a && tri[k] != b).unwrap_or(0)];
                at(t, v)
            };
            let (pa, pb) = (at(y, a), at(y, b));
            let sx = orient2d(pa, pb, m.apply(third(x)));
            let sy = orient2d(pa, pb, third(y));
            sx.is_ne() && sy.is_ne() && sx != sy
        });
        between.entry((ix.min(iy), ix.max(iy))).or_default().push(m);
    }
    // Charts: islands joined, each with its move into the chart's frame.
    let mut chart: Vec<u32> = (0..n as u32).collect();
    let mut frame: Vec<Rigid> = vec![Rigid::IDENTITY; n];
    let mut members: BTreeMap<u32, Vec<u32>> = (0..n as u32)
        .filter(|&t| island[t as usize] == t)
        .map(|t| (t, vec![t]))
        .collect();
    for (&(i, j), moves) in &between {
        let Some(Some(m)) = moves.first() else {
            continue;
        };
        if !moves.iter().all(|x| x.is_some_and(|x| x.near(m, tol))) {
            continue;
        }
        let (ci, cj) = (chart[i as usize], chart[j as usize]);
        if ci == cj {
            continue;
        }
        // Reframe the smaller chart into the larger's frame (ties: j's
        // into i's).
        let (keep, gone, to_keep) = if members[&cj].len() <= members[&ci].len() {
            // j-raw → chart i: frame[i] ∘ m⁻¹; then chart j → j-raw.
            let f = frame[i as usize]
                .after(&m.inverse())
                .after(&frame[j as usize].inverse());
            (ci, cj, f)
        } else {
            let f = frame[j as usize]
                .after(m)
                .after(&frame[i as usize].inverse());
            (cj, ci, f)
        };
        let moved = members.remove(&gone).unwrap_or_default();
        for &k in &moved {
            frame[k as usize] = to_keep.after(&frame[k as usize]);
            chart[k as usize] = keep;
        }
        members.entry(keep).or_default().extend(moved);
    }
    let tri_chart = (0..n).map(|t| chart[island[t] as usize]).collect();
    let tri_frame = (0..n).map(|t| frame[island[t] as usize]).collect();
    Some((tri_chart, tri_frame, islands))
}
