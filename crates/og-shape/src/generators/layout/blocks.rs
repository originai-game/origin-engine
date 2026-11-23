//! Blocks: a region cut by a grid and by named streets, or the strips
//! along both sides of a street.

use glam::DVec2;
use og_core::math::double;

use super::geom::{EPS, Poly, middle};
use super::{GridDesc, MAX_CELLS, Stream, Street, key};

/// How far a block stands from a street's centre line: half its roadway
/// and its pavement.
fn half(s: &Street) -> f64 {
    s.width / 2.0 + s.sidewalk
}

fn kept(p: Poly, out: &mut Vec<Poly>) {
    let p = p.tidy();
    if p.len() >= 3 && p.area() > EPS {
        out.push(p);
    }
}

/// The region cut by the grid's streets, which are added to `streets`.
pub(super) fn grid(
    region: &Poly,
    g: &GridDesc,
    seed: u64,
    streets: &mut Vec<Street>,
) -> Result<Vec<Poly>, String> {
    let (s, c) = double::sin_cos(double::to_radians(g.turn));
    let axes = [DVec2::new(c, s), DVec2::new(-s, c)];
    let size = [g.size.x, g.size.y];
    let span = |a: DVec2| {
        region
            .points
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), p| {
                (lo.min(p.dot(a)), hi.max(p.dot(a)))
            })
    };
    let spans = [span(axes[0]), span(axes[1])];
    let cells: Vec<f64> = (0..2)
        .map(|k| ((spans[k].1 - spans[k].0) / size[k]).ceil() + 3.0)
        .collect();
    if cells[0] * cells[1] > MAX_CELLS as f64 {
        return Err(format!(
            "a grid of {} by {} m over the region makes more than {MAX_CELLS} cells",
            size[0], size[1]
        ));
    }
    let h = g.width / 2.0 + g.sidewalk;
    // Each axis's street lines: where they cross it, and the street.
    let mut lines: [Vec<(f64, usize)>; 2] = [Vec::new(), Vec::new()];
    for k in 0..2 {
        let what = if k == 0 { "grid x" } else { "grid z" };
        let first = (spans[k].0 / size[k]).floor() as i64 - 1;
        let last = (spans[k].1 / size[k]).ceil() as i64 + 1;
        for i in first..=last {
            let jitter =
                Stream::new(key(seed, what, DVec2::new(i as f64, 0.0))).range(-g.jitter, g.jitter);
            let at = (i as f64 + jitter) * size[k];
            if at <= spans[k].0 || at >= spans[k].1 {
                continue;
            }
            let (a, b, across) = (axes[k], axes[1 - k], spans[1 - k]);
            streets.push(Street {
                name: String::new(),
                points: vec![a * at + b * across.0, a * at + b * across.1],
                width: g.width,
                sidewalk: g.sidewalk,
            });
            lines[k].push((at, streets.len() - 1));
        }
    }
    // The cells between neighbouring lines (the region's own edges past
    // the first and the last).
    let between = |k: usize| {
        let mut b: Vec<Option<(f64, usize)>> = vec![None];
        b.extend(lines[k].iter().copied().map(Some));
        b.push(None);
        b.windows(2).map(|w| (w[0], w[1])).collect::<Vec<_>>()
    };
    let mut out = Vec::new();
    for (lx, hx) in between(0) {
        for &(lz, hz) in &between(1) {
            let mut p = region.clone();
            for (a, lo, hi) in [(axes[0], lx, hx), (axes[1], lz, hz)] {
                if let Some((at, id)) = lo {
                    p = p.clip(-a, -(at + h), Some(id));
                }
                if let Some((at, id)) = hi {
                    p = p.clip(a, at - h, Some(id));
                }
            }
            kept(p, &mut out);
        }
    }
    Ok(out)
}

/// A strip `depth` deep on each side of every segment of street `id`,
/// mitred at its bends, its front on the street.
pub(super) fn along(s: &Street, depth: f64, id: usize) -> Vec<Poly> {
    let (pts, h) = (&s.points, half(s));
    let n = pts.len();
    let dir = |i: usize| (pts[i + 1] - pts[i]).normalize();
    // The offset of a vertex for a unit distance from each segment there.
    let mitre = |i: usize| {
        if i == 0 {
            return dir(0).perp();
        }
        if i == n - 1 {
            return dir(n - 2).perp();
        }
        let (a, b) = (dir(i - 1).perp(), dir(i).perp());
        let m = (a + b).normalize_or_zero();
        if m.dot(a) < 0.1 { a } else { m / m.dot(a) }
    };
    let mut out = Vec::new();
    for i in 0..n - 1 {
        let (a, b, ma, mb) = (pts[i], pts[i + 1], mitre(i), mitre(i + 1));
        let left = vec![
            a + ma * h,
            b + mb * h,
            b + mb * (h + depth),
            a + ma * (h + depth),
        ];
        let right = vec![
            b - mb * h,
            a - ma * h,
            a - ma * (h + depth),
            b - mb * (h + depth),
        ];
        for points in [left, right] {
            let mut p = Poly::new(points, None);
            p.streets[0] = Some(id);
            kept(p, &mut out);
        }
    }
    out
}

/// The blocks with street `id`'s corridor taken out of each one a
/// segment of it runs across; a block a segment ends inside is kept whole,
/// with a note.
pub(super) fn cut(polys: Vec<Poly>, s: &Street, id: usize, notes: &mut Vec<String>) -> Vec<Poly> {
    let h = half(s);
    let mut polys = polys;
    for w in s.points.windows(2) {
        let (a, b) = (w[0], w[1]);
        let (e, len) = ((b - a).normalize(), a.distance(b));
        let (n, an) = (e.perp(), a.dot(e.perp()));
        let mut next = Vec::new();
        for p in polys {
            let range = |f: &dyn Fn(DVec2) -> f64| {
                p.points
                    .iter()
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), q| {
                        (lo.min(f(*q)), hi.max(f(*q)))
                    })
            };
            let (lo, hi) = range(&|q| (q - a).dot(n));
            let (first, last) = range(&|q| (q - a).dot(e));
            if lo >= h || hi <= -h || first >= len + h || last <= -h {
                next.push(p);
                continue;
            }
            if first < -h || last > len + h {
                let m = middle(&p.points);
                notes.push(format!(
                    "street `{}` ends inside the block near ({:.1}, {:.1}); the block is kept whole",
                    s.name, m.x, m.y
                ));
                next.push(p);
                continue;
            }
            kept(p.clip(-n, -(an + h), Some(id)), &mut next);
            kept(p.clip(n, an - h, Some(id)), &mut next);
        }
        polys = next;
    }
    polys
}
