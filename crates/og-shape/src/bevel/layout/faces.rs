//! Faces rebuilt round the bevel: planar regions, their outlines bridged and clipped into triangles.

use super::*;

pub(crate) fn find(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}

pub(crate) fn union(parent: &mut [usize], a: usize, b: usize) {
    let (ra, rb) = (find(parent, a), find(parent, b));
    if ra != rb {
        parent[ra.max(rb)] = ra.min(rb);
    }
}

/// The plane projection that keeps a triangle facing `n` counter-clockwise.
pub(crate) fn projector(n: DVec3) -> impl Fn(DVec3) -> DVec2 {
    let a = n.abs();
    move |p: DVec3| {
        if a.z >= a.x && a.z >= a.y {
            if n.z >= 0.0 {
                DVec2::new(p.x, p.y)
            } else {
                DVec2::new(p.y, p.x)
            }
        } else if a.x >= a.y {
            if n.x >= 0.0 {
                DVec2::new(p.y, p.z)
            } else {
                DVec2::new(p.z, p.y)
            }
        } else if n.y >= 0.0 {
            DVec2::new(p.z, p.x)
        } else {
            DVec2::new(p.x, p.z)
        }
    }
}

/// Ear-clips a polygon (vertex ids) facing `n`; every triangle must turn
/// counter-clockwise by the exact predicate, or the bevel does not fit.
///
/// The first ear found is cut each time. Where that leaves a triangle of
/// zero area in floating point (three points of a straight chamfer
/// profile, counter-clockwise only by the exact predicate), the polygon
/// is clipped again cutting the best-shaped ear each time
/// (`docs/research.md` §7.5); results without such a triangle are the
/// first pass's.
pub(crate) fn clip(poly: &[u32], pos: &[DVec3], n: DVec3) -> Option<Vec<[u32; 3]>> {
    let first = clip_with(poly, pos, n, false)?;
    let flat = |t: &[u32; 3]| flat(t, pos);
    if !first.iter().any(flat) {
        return Some(first);
    }
    clip_with(poly, pos, n, true).filter(|ts| !ts.iter().any(flat))
}

/// Zero area as [`Solid::measure`] will find it: the canonical solid
/// starts each triangle at its first vertex in position order, and the
/// rounding of the area depends on the corner it starts from. Two equal
/// positions (welded into one) count as zero area too.
pub(crate) fn flat(t: &[u32; 3], pos: &[DVec3]) -> bool {
    use crate::math::order::cmp_dvec3;
    let p = t.map(|v| pos[v as usize] + DVec3::ZERO);
    let mut r = 0;
    for k in 1..3 {
        if cmp_dvec3(p[k], p[r]).is_lt() {
            r = k;
        }
    }
    let (a, b, c) = (p[r], p[(r + 1) % 3], p[(r + 2) % 3]);
    cmp_dvec3(a, b).is_eq()
        || cmp_dvec3(b, c).is_eq()
        || cmp_dvec3(c, a).is_eq()
        || (b - a).cross(c - a).length() == 0.0
}

pub(crate) fn clip_with(
    poly: &[u32],
    pos: &[DVec3],
    n: DVec3,
    best: bool,
) -> Option<Vec<[u32; 3]>> {
    use core::cmp::Ordering::Greater;
    let proj = projector(n);
    let mut ring: Vec<u32> = poly.to_vec();
    ring.dedup();
    while ring.len() > 1 && ring.first() == ring.last() {
        ring.pop();
    }
    let p2 = |i: u32| proj(pos[i as usize]);
    let flat = |t: &[u32; 3]| flat(t, pos);
    let mut out = Vec::new();
    while ring.len() > 3 {
        let len = ring.len();
        let is_ear = |i: usize| {
            let (a, b, c) = (ring[(i + len - 1) % len], ring[i], ring[(i + 1) % len]);
            let (pa, pb, pc) = (p2(a), p2(b), p2(c));
            if predicates::orient2d(pa, pb, pc) != Greater {
                return false;
            }
            ring.iter().all(|&q| {
                if q == a || q == b || q == c {
                    return true;
                }
                let pq = p2(q);
                !(predicates::orient2d(pa, pb, pq) != core::cmp::Ordering::Less
                    && predicates::orient2d(pb, pc, pq) != core::cmp::Ordering::Less
                    && predicates::orient2d(pc, pa, pq) != core::cmp::Ordering::Less)
            })
        };
        let ear = if best {
            // Never a flat ear while another fits; first an ear beside a
            // flat vertex, which then gets a new neighbour (otherwise the
            // flat vertex and its neighbours end as the last triangle);
            // then the most nearly equilateral (twice its area over the
            // sum of its squared sides); then the first by index.
            let at = |i: usize| [ring[(i + len - 1) % len], ring[i], ring[(i + 1) % len]];
            let shape = |i: usize| {
                let [a, b, c] = at(i).map(|v| pos[v as usize]);
                let sides =
                    (b - a).length_squared() + (c - b).length_squared() + (a - c).length_squared();
                (b - a).cross(c - a).length() / sides
            };
            let mut found: Option<((bool, bool, f64), usize)> = None;
            for i in (0..len).filter(|&i| is_ear(i)) {
                let beside = flat(&at((i + len - 1) % len)) || flat(&at((i + 1) % len));
                let q = (!flat(&at(i)), beside, shape(i));
                let better = found.is_none_or(|(f, _)| {
                    (q.0, q.1)
                        .cmp(&(f.0, f.1))
                        .then(q.2.total_cmp(&f.2))
                        .is_gt()
                });
                if better {
                    found = Some((q, i));
                }
            }
            found.map(|(_, i)| i)
        } else {
            (0..len).find(|&i| is_ear(i))
        }?;
        let (a, b, c) = (
            ring[(ear + len - 1) % len],
            ring[ear],
            ring[(ear + 1) % len],
        );
        out.push([a, b, c]);
        ring.remove(ear);
    }
    if ring.len() < 3 {
        return None;
    }
    let t = [ring[0], ring[1], ring[2]];
    if predicates::orient2d(p2(t[0]), p2(t[1]), p2(t[2])) != Greater {
        return None;
    }
    out.push(t);
    Some(out)
}

/// The triangles of `t`'s face that lie in its plane, reached across
/// shared edges, in index order.
pub(crate) fn planar_region(s: &Solid, topo: &Topology, t: usize) -> Vec<usize> {
    let n = tri_normal(s, t);
    let mut seen = BTreeSet::from([t]);
    let mut stack = vec![t];
    while let Some(x) = stack.pop() {
        let tri = s.triangles[x];
        for k in 0..3 {
            let Some(&y) = topo.dir.get(&(tri[(k + 1) % 3], tri[k])) else {
                continue;
            };
            if !seen.contains(&y)
                && s.surfaces[y] == s.surfaces[t]
                && tri_normal(s, y).dot(n) > PLANAR_DOT
            {
                seen.insert(y);
                stack.push(y);
            }
        }
    }
    seen.into_iter().collect()
}

/// The outline loops of a set of polygons sharing edges, or `None` when
/// a vertex is passed twice.
pub(crate) fn outline(polys: &[Vec<u32>]) -> Option<Vec<Vec<u32>>> {
    let mut edges: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for poly in polys {
        let mut ring = poly.clone();
        ring.dedup();
        while ring.len() > 1 && ring.first() == ring.last() {
            ring.pop();
        }
        for k in 0..ring.len() {
            *edges
                .entry((ring[k], ring[(k + 1) % ring.len()]))
                .or_default() += 1;
        }
    }
    let mut next: BTreeMap<u32, u32> = BTreeMap::new();
    for (&(a, c), &count) in &edges {
        let back = edges.get(&(c, a)).copied().unwrap_or(0);
        if count > back && (count - back > 1 || next.insert(a, c).is_some()) {
            return None;
        }
    }
    let mut loops = Vec::new();
    let mut seen = BTreeSet::new();
    for &start in next.keys() {
        if seen.contains(&start) {
            continue;
        }
        let mut ring = vec![start];
        seen.insert(start);
        let mut cur = next[&start];
        while cur != start {
            if !seen.insert(cur) {
                return None;
            }
            ring.push(cur);
            cur = *next.get(&cur)?;
        }
        loops.push(ring);
    }
    Some(loops)
}

/// One ring from an outer loop and its holes, each hole joined to the
/// outer ring by a bridge to the nearest vertex it sees (exact
/// predicates), as in ear-cut hole elimination.
pub(crate) fn bridge(loops: Vec<Vec<u32>>, pos: &[DVec3], n: DVec3) -> Option<Vec<u32>> {
    use core::cmp::Ordering::{Greater, Less};
    let proj = projector(n);
    let p2 = |i: u32| proj(pos[i as usize]);
    let area = |r: &[u32]| {
        (0..r.len())
            .map(|k| p2(r[k]).perp_dot(p2(r[(k + 1) % r.len()])))
            .sum::<f64>()
    };
    let (outer, mut holes): (Vec<Vec<u32>>, Vec<Vec<u32>>) =
        loops.into_iter().partition(|r| area(r) > 0.0);
    let [mut ring] = <[Vec<u32>; 1]>::try_from(outer).ok()?;
    // Rightmost hole first.
    let right = |r: &[u32]| {
        let mut best = 0;
        for k in 1..r.len() {
            let (a, b) = (p2(r[k]), p2(r[best]));
            if a.x > b.x || (a.x == b.x && a.y < b.y) {
                best = k;
            }
        }
        best
    };
    holes.sort_by(|a, b| {
        let (pa, pb) = (p2(a[right(a)]), p2(b[right(b)]));
        pb.x.total_cmp(&pa.x).then(pa.y.total_cmp(&pb.y))
    });
    for (hi, hole) in holes.iter().enumerate() {
        let h = right(hole);
        let ph = p2(hole[h]);
        let mut cands: Vec<(f64, usize)> = ring
            .iter()
            .enumerate()
            .map(|(k, &o)| ((p2(o) - ph).length_squared(), k))
            .collect();
        cands.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let blocked = |po: crate::math::DVec2, r: &[u32]| {
            (0..r.len()).any(|k| {
                let (pa, pb) = (p2(r[k]), p2(r[(k + 1) % r.len()]));
                if pa == ph || pb == ph || pa == po || pb == po {
                    return false;
                }
                let o1 = predicates::orient2d(ph, po, pa);
                let o2 = predicates::orient2d(ph, po, pb);
                let o3 = predicates::orient2d(pa, pb, ph);
                let o4 = predicates::orient2d(pa, pb, po);
                ((o1 == Less && o2 == Greater) || (o1 == Greater && o2 == Less))
                    && ((o3 == Less && o4 == Greater) || (o3 == Greater && o4 == Less))
            })
        };
        // The bridge must leave the hole into the region, which lies to
        // the left of every outline edge, the holes' included.
        let len = hole.len();
        let (hp, hn) = (p2(hole[(h + len - 1) % len]), p2(hole[(h + 1) % len]));
        let k = cands.iter().map(|&(_, k)| k).find(|&k| {
            let po = p2(ring[k]);
            if po == ph {
                return false;
            }
            let (l1, l2) = (
                predicates::orient2d(hp, ph, po) == Greater,
                predicates::orient2d(ph, hn, po) == Greater,
            );
            let into = if predicates::orient2d(hp, ph, hn) == Greater {
                l1 && l2
            } else {
                l1 || l2
            };
            into && !blocked(po, &ring) && holes.iter().skip(hi).all(|r| !blocked(po, r))
        })?;
        let mut merged = ring[..=k].to_vec();
        merged.extend((0..=len).map(|j| hole[(h + j) % len]));
        merged.extend_from_slice(&ring[k..]);
        ring = merged;
    }
    Some(ring)
}

pub(crate) const CAPS: [&str; 4] = ["top", "bottom", "start", "end"];

pub(crate) fn is_cap(name: &str) -> bool {
    CAPS.contains(&name.rsplit('.').next().unwrap_or(name))
}
