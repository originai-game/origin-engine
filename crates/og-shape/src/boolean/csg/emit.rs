//! Step 4: the kept pieces written out as a closed mesh, slivers closed and fins cancelled.

use super::*;

/// The kept pieces as points (only those used, in first-use order),
/// triangles and each one's source `(operand, triangle)`, plus the map
/// from arrangement point ids to output ids.
#[allow(clippy::type_complexity)]
pub(crate) fn emit(
    c: &Csg,
    kept: &[Kept],
) -> (
    Vec<DVec3>,
    Vec<[u32; 3]>,
    Vec<(u8, u32)>,
    BTreeMap<u32, u32>,
) {
    let mut remap: BTreeMap<u32, u32> = BTreeMap::new();
    let mut out_points = Vec::new();
    let mut out_tris = Vec::with_capacity(kept.len());
    let mut origins = Vec::with_capacity(kept.len());
    for p in kept {
        let ids = if p.flip {
            [p.ids[0], p.ids[2], p.ids[1]]
        } else {
            p.ids
        };
        let t = ids.map(|id| {
            *remap.entry(id).or_insert_with(|| {
                out_points.push(c.points[id as usize]);
                out_points.len() as u32 - 1
            })
        });
        out_tris.push(t);
        let solid = &c.solids[p.mesh as usize];
        origins.push((p.mesh, solid.source[p.tri as usize]));
    }
    (out_points, out_tris, origins, remap)
}

/// The second pass's last resort for regions no probe can classify: a
/// region whose largest piece has an inradius under 2⁻³⁰ of the operands'
/// size (a sliver between surfaces a rounding error apart) keeps its
/// probe's answer unless the result is open along it. Then, for each
/// cluster of such regions on the open edges (at most 12, joined by
/// edges), the choice of which to keep that leaves the fewest unmatched
/// edges is taken: fewest changes first, then the lowest-numbered.
/// Regions a probe can resolve are never changed, and a kept piece faces
/// as the operation turns it (`docs/research.md` §5.4, item 5).
pub(crate) fn close_slivers(
    c: &Csg,
    pieces: &[Piece],
    regions: &Regions,
    op: CsgOp,
    scale: f64,
    choice: &mut [(bool, bool)],
) {
    let directed = |p: &Piece, flip: bool| {
        let ids = if flip {
            [p.ids[0], p.ids[2], p.ids[1]]
        } else {
            p.ids
        };
        (0..3).map(move |k| (ids[k], ids[(k + 1) % 3]))
    };
    let balance = |choice: &[(bool, bool)]| {
        let mut b: BTreeMap<(u32, u32), i64> = BTreeMap::new();
        for (p, &(keep, flip)) in pieces.iter().zip(choice) {
            if keep {
                for (x, y) in directed(p, flip) {
                    *b.entry((x.min(y), x.max(y))).or_default() += if x < y { 1 } else { -1 };
                }
            }
        }
        b
    };
    let base = balance(choice);
    let open: BTreeSet<(u32, u32)> = base
        .iter()
        .filter(|(_, n)| **n != 0)
        .map(|(&e, _)| e)
        .collect();
    if open.is_empty() {
        return;
    }
    let inradius = |p: &Piece| {
        let [a, b, cc] = p.ids.map(|i| c.points[i as usize]);
        let perimeter = (b - a).length() + (cc - b).length() + (a - cc).length();
        (b - a).cross(cc - a).length() / perimeter.max(1e-300)
    };
    let tiny = scale * (1.0 / (1u64 << 30) as f64);
    // Unreliable regions on an open edge, with their pieces.
    let mut members: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (pi, &r) in regions.root.iter().enumerate() {
        members.entry(r).or_default().push(pi);
    }
    let edges_of = |r: usize| -> BTreeSet<(u32, u32)> {
        members[&r]
            .iter()
            .flat_map(|&pi| directed(&pieces[pi], false).map(|(x, y)| (x.min(y), x.max(y))))
            .collect()
    };
    let unsure: Vec<usize> = regions
        .best
        .iter()
        .filter(|&(&r, &b)| {
            inradius(&pieces[b]) < tiny && edges_of(r).iter().any(|e| open.contains(e))
        })
        .map(|(&r, _)| r)
        .collect();
    // Clusters: regions sharing an edge.
    let mut cluster: BTreeMap<usize, usize> = unsure.iter().map(|&r| (r, r)).collect();
    fn find(c: &mut BTreeMap<usize, usize>, mut r: usize) -> usize {
        while c[&r] != r {
            r = c[&r];
        }
        r
    }
    for (i, &a) in unsure.iter().enumerate() {
        let ea = edges_of(a);
        for &b in &unsure[i + 1..] {
            if edges_of(b).iter().any(|e| ea.contains(e)) {
                let (x, y) = (find(&mut cluster, a), find(&mut cluster, b));
                if x != y {
                    cluster.insert(x.max(y), x.min(y));
                }
            }
        }
    }
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for &r in &unsure {
        let k = find(&mut cluster, r);
        groups.entry(k).or_default().push(r);
    }
    // How a region's pieces face when kept.
    let flip_of = |m: u8| m == 1 && op == CsgOp::Subtract;
    let mut base = base;
    for rs in groups.values() {
        if rs.len() > 12 {
            continue;
        }
        let add = |b: &mut BTreeMap<(u32, u32), i64>, pi: usize, flip: bool, sign: i64| {
            for (x, y) in directed(&pieces[pi], flip) {
                *b.entry((x.min(y), x.max(y))).or_default() += sign * if x < y { 1 } else { -1 };
            }
        };
        // The rest of the result, without the cluster.
        let mut rest = base.clone();
        for &r in rs {
            for &pi in &members[&r] {
                if choice[pi].0 {
                    add(&mut rest, pi, choice[pi].1, -1);
                }
            }
        }
        let touched: BTreeSet<(u32, u32)> = rs.iter().flat_map(|&r| edges_of(r)).collect();
        let now: u32 = rs
            .iter()
            .enumerate()
            .filter(|&(_, &r)| choice[members[&r][0]].0)
            .map(|(i, _)| 1 << i)
            .sum();
        let mut best: Option<(u64, u32, u32)> = None;
        for mask in 0u32..(1 << rs.len()) {
            let mut b = rest.clone();
            for (i, &r) in rs.iter().enumerate() {
                if mask & (1 << i) != 0 {
                    for &pi in &members[&r] {
                        add(&mut b, pi, flip_of(pieces[pi].mesh), 1);
                    }
                }
            }
            let cost: u64 = touched
                .iter()
                .map(|e| b.get(e).map_or(0, |n| n.unsigned_abs()))
                .sum();
            let key = (cost, (mask ^ now).count_ones(), mask);
            if best.is_none_or(|bk| key < bk) {
                best = Some(key);
            }
        }
        let Some((_, _, mask)) = best else {
            continue;
        };
        for (i, &r) in rs.iter().enumerate() {
            let keep = mask & (1 << i) != 0;
            for &pi in &members[&r] {
                choice[pi] = (keep, keep && flip_of(pieces[pi].mesh));
            }
        }
        // The cluster's choice is part of the result the next one sees.
        for &r in rs {
            for &pi in &members[&r] {
                if choice[pi].0 {
                    add(&mut rest, pi, choice[pi].1, 1);
                }
            }
        }
        base = rest;
    }
}

/// Drops pairs of kept triangles on the same three positions facing
/// opposite ways: a piece of each operand where their surfaces lie on one
/// another, both kept because a sliver a rounding error wide was
/// classified on each side. Such a pair bounds no volume, and its edges
/// cancel, so a closed result stays closed; without it, its edges are
/// used four times (`docs/research.md` §5.4, item 5). Pairs are taken in
/// output order.
pub(crate) fn cancel_fins(
    w: &[u32],
    tris: Vec<[u32; 3]>,
    origins: Vec<(u8, u32)>,
) -> (Vec<[u32; 3]>, Vec<(u8, u32)>) {
    // Each triangle by its sorted corners, and whether its winding is the
    // sorted order's (an even permutation).
    let key = |t: [u32; 3]| {
        let t = t.map(|v| w[v as usize]);
        let mut s = t;
        s.sort_unstable();
        let r = (0..3).find(|&k| t[k] == s[0]).unwrap_or(0);
        (s, t[(r + 1) % 3] == s[1])
    };
    let mut by: BTreeMap<[u32; 3], (Vec<usize>, Vec<usize>)> = BTreeMap::new();
    for (i, &t) in tris.iter().enumerate() {
        let (s, even) = key(t);
        if s[0] == s[1] || s[1] == s[2] {
            continue;
        }
        let e = by.entry(s).or_default();
        if even { e.0.push(i) } else { e.1.push(i) }
    }
    let mut drop = vec![false; tris.len()];
    for (pos, neg) in by.values() {
        for (&i, &j) in pos.iter().zip(neg) {
            drop[i] = true;
            drop[j] = true;
        }
    }
    if !drop.contains(&true) {
        return (tris, origins);
    }
    tris.into_iter()
        .zip(origins)
        .zip(drop)
        .filter(|(_, d)| !d)
        .map(|(x, _)| x)
        .unzip()
}

/// Each point's id after welding by exact position (first-seen order).
pub(crate) fn weld_ids(points: &[DVec3]) -> Vec<u32> {
    let mut id: BTreeMap<[u64; 3], u32> = BTreeMap::new();
    points
        .iter()
        .map(|p| {
            let n = id.len() as u32;
            *id.entry(p.to_array().map(|x| (x + 0.0).to_bits()))
                .or_insert(n)
        })
        .collect()
}

/// [`CsgOutput`]'s edge counts.
pub(crate) struct EdgeCounts {
    pub(crate) unmatched: usize,
    pub(crate) touching: usize,
    pub(crate) non_manifold: usize,
}

/// How a result's edges are used, after welding (`w`), zero-area
/// triangles skipped: directed edges that do not balance, edges where the
/// operands touch (two faces of each, one each way), and other edges not
/// used once each way.
pub(crate) fn edge_counts(w: &[u32], tris: &[[u32; 3]], origins: &[(u8, u32)]) -> EdgeCounts {
    // Per edge, low id first: uses by (operand, low to high).
    let mut uses: BTreeMap<(u32, u32), [u32; 4]> = BTreeMap::new();
    for (t, &(m, _)) in tris.iter().zip(origins) {
        let t = t.map(|v| w[v as usize]);
        if t[0] == t[1] || t[1] == t[2] || t[0] == t[2] {
            continue;
        }
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            let slot = 2 * (m.min(1) as usize) + usize::from(a > b);
            uses.entry((a.min(b), a.max(b))).or_default()[slot] += 1;
        }
    }
    let mut c = EdgeCounts {
        unmatched: 0,
        touching: 0,
        non_manifold: 0,
    };
    for u in uses.values() {
        let (fwd, back) = (u[0] + u[2], u[1] + u[3]);
        c.unmatched += fwd.abs_diff(back) as usize;
        if fwd == 1 && back == 1 {
            continue;
        }
        if *u == [1, 1, 1, 1] {
            c.touching += 1;
        } else {
            c.non_manifold += 1;
        }
    }
    c
}
