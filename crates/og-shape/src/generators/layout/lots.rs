//! Lots: the ring of lots along a block's street edges, each strip the
//! ground nearest its street within the ring's depth, cut across.

use glam::DVec2;

use super::geom::{EPS, Poly, area2, edge};
use super::{Lot, LotsDesc, Stream, key};

/// The widths a frontage of `len` is cut into: each drawn between the
/// least and the greatest while what is left allows another, the last
/// what remains (outside the range only when no cut can keep it in).
fn widths(len: f64, w: [f64; 2], s: &mut Stream) -> Vec<f64> {
    let mut out = Vec::new();
    let mut left = len;
    while left >= w[0] * 2.0 && left > w[1] {
        let x = s.range(w[0], w[1].min(left - w[0]));
        out.push(x);
        left -= x;
    }
    out.push(left);
    out
}

/// `p` turned so its first edge runs along the front line (through `a`,
/// along `e`, normal `n`), in the front's direction.
fn front_first(p: Poly, a: DVec2, e: DVec2, n: DVec2) -> Option<Vec<DVec2>> {
    let k = p.len();
    let on = |q: DVec2| (q - a).dot(n).abs() < 1e-7;
    let start = (0..k).find(|&i| {
        let (q, r) = (p.points[i], p.points[(i + 1) % k]);
        on(q) && on(r) && (r - q).dot(e) > EPS
    })?;
    Some((0..k).map(|i| p.points[(start + i) % k]).collect())
}

/// The lots of convex block `b` (its edges each naming their street),
/// pushed to `lots`; a note when it has none.
pub(super) fn lots(
    b: usize,
    poly: &Poly,
    d: &LotsDesc,
    seed: u64,
    lots: &mut Vec<Lot>,
) -> Option<String> {
    let pts = &poly.points;
    let n = pts.len();
    // A street edge shorter than one lot fronts none (a stub where a
    // street clips the region's corner); the ring is shared out among
    // the others.
    let fronts: Vec<usize> = (0..n)
        .filter(|&i| poly.streets[i].is_some() && pts[i].distance(pts[(i + 1) % n]) >= d.width[0])
        .collect();
    let Some(&first) = fronts.first() else {
        return Some(format!(
            "block {b} fronts no street for a lot's width: it has no lots"
        ));
    };
    let mut s = Stream::new(key(
        seed,
        "block",
        (pts[first] + pts[(first + 1) % n]) / 2.0,
    ));
    let depth = s.range(d.depth[0], d.depth[1]);
    let before = lots.len();
    for &i in &fronts {
        let (a, end) = (pts[i], pts[(i + 1) % n]);
        let (e, nrm) = edge(pts, i);
        // Within the depth, and nearer this front's line than any other's.
        let mut strip = poly.clip(nrm, depth + a.dot(nrm), None);
        for &k in fronts.iter().filter(|&&k| k != i) {
            let (ek, nk) = edge(pts, k);
            let g = nrm - nk;
            if g.length() > 1e-9 {
                strip = strip.clip(g, a.dot(nrm) - pts[k].dot(nk), None);
            } else if (pts[k] - a).dot(e) > 0.0 {
                // In line with it: the land between them is split at the
                // ends that face each other.
                strip = strip.clip(e, ((end + pts[k]) / 2.0).dot(e), None);
            } else {
                strip = strip.clip(-ek, -((a + pts[(k + 1) % n]) / 2.0).dot(ek), None);
            }
            if strip.len() < 3 {
                break;
            }
        }
        let strip = strip.tidy();
        if strip.len() < 3 || strip.area() <= EPS {
            continue;
        }
        let len = a.distance(end);
        let mut ws = Stream::new(key(seed, "lots", (a + end) / 2.0));
        let cuts: Vec<f64> = widths(len, d.width, &mut ws)
            .into_iter()
            .scan(0.0, |at, w| {
                *at += w;
                Some(*at)
            })
            .collect();
        let mut from = 0.0;
        for (k, &to) in cuts.iter().enumerate() {
            let last = k + 1 == cuts.len();
            // The first and the last lot reach the strip's ends.
            let mut piece = strip.clone();
            if k > 0 {
                piece = piece.clip(-e, -(a.dot(e) + from), None);
            }
            if !last {
                piece = piece.clip(e, a.dot(e) + to, None);
            }
            let piece = piece.tidy();
            let to = if last { len } else { to };
            if piece.len() >= 3
                && area2(&piece.points) > EPS
                && let Some(footprint) = front_first(piece, a, e, nrm)
            {
                lots.push(Lot {
                    block: b,
                    street: poly.streets[i].expect("a front"),
                    footprint,
                    frontage: to - from,
                    depth,
                    seed: key(seed, "lot", a + e * ((from + to) / 2.0)),
                });
            }
            from = to;
        }
    }
    (lots.len() == before).then(|| format!("block {b} has no room for lots {depth:.1} m deep"))
}
