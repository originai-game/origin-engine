//! Seams cut into a smooth region until it flattens as one chart.

use super::*;

/// What [`Patch::judge`] found.
#[derive(Default)]
pub(crate) struct Judged {
    pub(crate) laid: Vec<[DVec2; 3]>,
    pub(crate) shrunk: Vec<f64>,
    pub(crate) well: bool,
}

/// The first of `laid` that lies over one before it.
pub(crate) fn first_over(laid: &[[DVec2; 3]], cell: f64) -> Option<usize> {
    let mut grid: BTreeMap<Cell, Vec<u32>> = BTreeMap::new();
    for (k, t) in laid.iter().enumerate() {
        let mut near: Vec<u32> = cells(t, cell)
            .filter_map(|c| grid.get(&c))
            .flatten()
            .copied()
            .collect();
        near.sort_unstable();
        near.dedup();
        if near.iter().any(|&o| tris_overlap_2d(*t, laid[o as usize])) {
            return Some(k);
        }
        for c in cells(t, cell) {
            grid.entry(c).or_default().push(k as u32);
        }
    }
    None
}

/// The most triangles [`seamed`] is given as one region.
pub(crate) const SEAM_REGION: usize = 8192;

/// How many rounds of cuts a region is given.
pub(crate) const SEAM_ROUNDS: usize = 16;

/// What [`seamed`] reads of the solid.
pub(crate) struct Cut<'a> {
    pub(crate) s: &'a Solid,
    pub(crate) by_edge: &'a BTreeMap<(u32, u32), Vec<u32>>,
    pub(crate) normals: &'a [DVec3],
    pub(crate) areas: &'a [f64],
    pub(crate) cos_limit: f64,
    pub(crate) cell: f64,
}

/// What [`seamed`] made of a region.
pub(crate) enum Seamed {
    /// The whole region as one chart: each triangle, in its order.
    Whole(Vec<[DVec2; 3]>),
    /// A part of it as one chart: which of the region's triangles (by
    /// their place in it), and each of those.
    Part(Vec<usize>, Vec<[DVec2; 3]>),
    /// Nothing worth a chart.
    Nothing,
}

/// The triangles `region` (joined across edges of two triangles)
/// flattened as one chart, cut open along seams; or, where cutting
/// stops helping, the largest part of it that did flatten. `spent`
/// counts the triangles solved for.
///
/// 1. **To a disk.** A tree is grown over the triangles from the first,
///    breadth first, across the region's edges that are not sharp. The
///    edges it did not cross, less every one that ends at a vertex with
///    no other (taken away in turn), are the fewest that open the
///    region to a disk: none on a patch, one line between the two rims
///    of a tube, two loops on a ring. A closed surface with no handle
///    is left none, and is cut between two vertices far apart.
/// 2. **Until it flattens.** The disk is flattened ([`Patch::solve`]).
///    Where a triangle folds or its area is past twice or under half
///    its own, the shortest path of edges from the triangle shrunk most
///    to the seam or rim nearest it is cut as well (a sharp edge counts
///    half its length, so seams take creases), and the chart is
///    flattened again. This is the cutting of Gu, Gortler and Hoppe's
///    geometry images (2002), with this chart's own measure of what is
///    shrunk (`docs/research.md` §65).
/// 3. **Or a part.** After sixteen cuts, or six that left no more
///    triangles well than the best before them, the triangles that are well and lie
///    over none before them are gathered across uncut edges, and the
///    largest such part (by area) is the chart, flattened again by
///    itself where that is well too. The rest is the caller's to try
///    again: fingers and ears leave a body this way.
pub(crate) fn seamed(c: &Cut, region: &[u32], spent: &mut usize) -> Seamed {
    let s = c.s;
    let local: BTreeMap<u32, usize> = region.iter().enumerate().map(|(k, &i)| (i, k)).collect();
    // The region's inner edges (its two triangles, whether sharp), and
    // how many rim edges meet at each vertex.
    let mut inner: BTreeMap<(u32, u32), (usize, usize, bool)> = BTreeMap::new();
    let mut rim: BTreeMap<u32, u32> = BTreeMap::new();
    for (k, &i) in region.iter().enumerate() {
        let t = s.triangles[i as usize];
        for j in 0..3 {
            let e = edge_key(t[j], t[(j + 1) % 3]);
            let users = &c.by_edge[&e];
            let other = if users.len() == 2 {
                let o = if users[0] == i { users[1] } else { users[0] };
                local.get(&o).copied().filter(|&o| o != k)
            } else {
                None
            };
            match other {
                Some(o) => {
                    let sharp =
                        c.normals[i as usize].dot(c.normals[region[o] as usize]) < c.cos_limit;
                    inner.insert(e, (k.min(o), k.max(o), sharp));
                }
                None => {
                    *rim.entry(e.0).or_default() += 1;
                    *rim.entry(e.1).or_default() += 1;
                }
            }
        }
    }
    // The tree over the triangles, and the edges it crossed.
    let mut crossed: std::collections::BTreeSet<(u32, u32)> = std::collections::BTreeSet::new();
    let mut reached = vec![false; region.len()];
    reached[0] = true;
    let mut queue = vec![0usize];
    let mut head = 0;
    while head < queue.len() {
        let k = queue[head];
        head += 1;
        let t = s.triangles[region[k] as usize];
        for j in 0..3 {
            let e = edge_key(t[j], t[(j + 1) % 3]);
            let Some(&(a, b, sharp)) = inner.get(&e) else {
                continue;
            };
            let o = if a == k { b } else { a };
            if sharp || reached[o] {
                continue;
            }
            reached[o] = true;
            crossed.insert(e);
            queue.push(o);
        }
    }
    // What it did not cross, pruned of its loose ends.
    let mut cut: std::collections::BTreeSet<(u32, u32)> = inner
        .keys()
        .filter(|e| !crossed.contains(e))
        .copied()
        .collect();
    let mut degree = rim.clone();
    let mut at: BTreeMap<u32, Vec<(u32, u32)>> = BTreeMap::new();
    for e in &cut {
        for v in [e.0, e.1] {
            *degree.entry(v).or_default() += 1;
            at.entry(v).or_default().push(*e);
        }
    }
    let mut loose: Vec<u32> = degree
        .iter()
        .filter(|(_, d)| **d == 1)
        .map(|(v, _)| *v)
        .collect();
    while let Some(v) = loose.pop() {
        if degree[&v] != 1 {
            continue;
        }
        let Some(e) = at
            .get(&v)
            .and_then(|l| l.iter().find(|e| cut.contains(e)))
            .copied()
        else {
            continue;
        };
        cut.remove(&e);
        for w in [e.0, e.1] {
            let d = degree.get_mut(&w).expect("counted above");
            *d -= 1;
            if *d == 1 {
                loose.push(w);
            }
        }
    }
    // The shortest path of uncut inner edges from `from` to a vertex
    // that `ends` it, as edges.
    let length = |e: &(u32, u32), sharp: bool| {
        let l = (s.positions[e.0 as usize] - s.positions[e.1 as usize]).length();
        if sharp { l * 0.5 } else { l }
    };
    let mut around: BTreeMap<u32, Vec<(u32, u32)>> = BTreeMap::new();
    for e in inner.keys() {
        around.entry(e.0).or_default().push(*e);
        around.entry(e.1).or_default().push(*e);
    }
    let path = |from: u32,
                cut: &std::collections::BTreeSet<(u32, u32)>,
                ends: &dyn Fn(u32) -> bool|
     -> Option<Vec<(u32, u32)>> {
        let mut best: BTreeMap<u32, (f64, (u32, u32))> = BTreeMap::new();
        let mut open: std::collections::BTreeSet<(u64, u32)> = std::collections::BTreeSet::new();
        best.insert(from, (0.0, (from, from)));
        open.insert((0, from));
        while let Some((bits, v)) = open.pop_first() {
            let far = f64::from_bits(bits);
            if far > best[&v].0 {
                continue;
            }
            if v != from && ends(v) {
                let mut edges = Vec::new();
                let mut w = v;
                while w != from {
                    let e = best[&w].1;
                    edges.push(e);
                    w = if e.0 == w { e.1 } else { e.0 };
                }
                return Some(edges);
            }
            for e in around.get(&v).into_iter().flatten() {
                if cut.contains(e) {
                    continue;
                }
                let w = if e.0 == v { e.1 } else { e.0 };
                let next = far + length(e, inner[e].2);
                if best.get(&w).is_none_or(|b| next < b.0) {
                    best.insert(w, (next, *e));
                    open.insert((next.to_bits(), w));
                }
            }
        }
        None
    };
    if rim.is_empty() && cut.is_empty() {
        // Closed, no handle: between the vertex furthest from the
        // first triangle's and the one furthest from that.
        let verts: Vec<u32> = around.keys().copied().collect();
        let furthest = |from: u32| {
            let mut best = (0.0, from);
            for &v in &verts {
                let d = (s.positions[v as usize] - s.positions[from as usize]).length_squared();
                if super::exceeds(d, best.0) {
                    best = (d, v);
                }
            }
            best.1
        };
        let a = furthest(s.triangles[region[0] as usize][0]);
        let b = furthest(a);
        match path(a, &cut, &|v| v == b) {
            Some(more) => cut.extend(more),
            None => return Seamed::Nothing,
        }
    }
    // The chart of some of the region's triangles (`inside`), split
    // along `cut`: a corner is one with its neighbour's across every
    // edge that is not cut.
    let split = |inside: &[bool], cut: &std::collections::BTreeSet<(u32, u32)>| {
        let mut parent: Vec<usize> = (0..3 * region.len()).collect();
        fn find(parent: &mut [usize], mut x: usize) -> usize {
            while parent[x] != x {
                parent[x] = parent[parent[x]];
                x = parent[x];
            }
            x
        }
        let corner = |k: usize, v: u32| {
            let t = s.triangles[region[k] as usize];
            3 * k + t.iter().position(|&w| w == v).expect("the edge's vertex")
        };
        for (e, &(a, b, _)) in &inner {
            if cut.contains(e) || !inside[a] || !inside[b] {
                continue;
            }
            for v in [e.0, e.1] {
                let (x, y) = (
                    find(&mut parent, corner(a, v)),
                    find(&mut parent, corner(b, v)),
                );
                parent[x.max(y)] = x.min(y);
            }
        }
        let mut number: BTreeMap<usize, usize> = BTreeMap::new();
        let mut points: Vec<DVec3> = Vec::new();
        let mut members: Vec<u32> = Vec::new();
        let mut tris: Vec<[usize; 3]> = Vec::new();
        for (k, &i) in region.iter().enumerate() {
            if !inside[k] {
                continue;
            }
            let t = s.triangles[i as usize];
            members.push(i);
            tris.push([0, 1, 2].map(|j| {
                let root = find(&mut parent, 3 * k + j);
                *number.entry(root).or_insert_with(|| {
                    points.push(s.positions[t[j] as usize]);
                    points.len() - 1
                })
            }));
        }
        Patch::new(points, tris, &members, c.normals, c.areas)
    };
    let all = vec![true; region.len()];
    let fine = |r: f64| (0.5..=2.0).contains(&r);
    // The flattening with the most triangles well, and its cuts.
    let mut kept: Option<(usize, Judged)> = None;
    let mut kept_cut = cut.clone();
    let mut stale = 0;
    for _ in 0..=SEAM_ROUNDS {
        let patch = split(&all, &cut);
        let Some(x) = patch.solve(spent) else {
            break;
        };
        let judged = patch.judge(&x, c.cell);
        if judged.well {
            return Seamed::Whole(judged.laid);
        }
        if judged.laid.is_empty() {
            break;
        }
        let bad = judged.shrunk.clone();
        let well = bad.iter().filter(|r| fine(**r)).count();
        if kept.as_ref().is_none_or(|k| well > k.0) {
            kept = Some((well, judged));
            kept_cut = cut.clone();
            stale = 0;
        } else {
            stale += 1;
            if stale == 6 {
                break;
            }
        }
        // From the triangle shrunk most (the one furthest, as the flat
        // chart measures, from any seam) that has a vertex off every
        // seam and rim, to the nearest of them.
        let on_edge = |v: u32, cut: &std::collections::BTreeSet<(u32, u32)>| {
            rim.contains_key(&v)
                || around
                    .get(&v)
                    .is_some_and(|l| l.iter().any(|e| cut.contains(e)))
        };
        let mut order: Vec<usize> = (0..bad.len()).collect();
        let top = bad.iter().copied().fold(0.0, |m: f64, x| m.max(x.abs()));
        order.sort_by_key(|&a| (super::level(bad[a], top), a));
        let Some(from) = order.iter().find_map(|&k| {
            s.triangles[region[k] as usize]
                .into_iter()
                .find(|&v| !on_edge(v, &cut))
        }) else {
            break;
        };
        match path(from, &cut, &|v| on_edge(v, &cut)) {
            Some(more) => cut.extend(more),
            None => break,
        }
    }
    let Some((_, judged)) = kept else {
        return Seamed::Nothing;
    };
    let cut = kept_cut;
    // The triangles that are well and over none before them.
    let mut well: Vec<bool> = judged.shrunk.iter().map(|r| fine(*r)).collect();
    let mut grid: BTreeMap<Cell, Vec<u32>> = BTreeMap::new();
    for (k, t) in judged.laid.iter().enumerate() {
        if !well[k] {
            continue;
        }
        let mut near: Vec<u32> = cells(t, c.cell)
            .filter_map(|c| grid.get(&c))
            .flatten()
            .copied()
            .collect();
        near.sort_unstable();
        near.dedup();
        if near
            .iter()
            .any(|&o| tris_overlap_2d(*t, judged.laid[o as usize]))
        {
            well[k] = false;
            continue;
        }
        for cl in cells(t, c.cell) {
            grid.entry(cl).or_default().push(k as u32);
        }
    }
    // Gathered across uncut edges; the largest part.
    let mut part_of = vec![usize::MAX; region.len()];
    let mut best = (0.0, usize::MAX);
    let mut parts = 0;
    for k in 0..region.len() {
        if !well[k] || part_of[k] != usize::MAX {
            continue;
        }
        let mut area = 0.0;
        let mut stack = vec![k];
        part_of[k] = parts;
        while let Some(k) = stack.pop() {
            area += c.areas[region[k] as usize];
            let t = s.triangles[region[k] as usize];
            for j in 0..3 {
                let e = edge_key(t[j], t[(j + 1) % 3]);
                let Some(&(a, b, _)) = inner.get(&e) else {
                    continue;
                };
                let o = if a == k { b } else { a };
                if cut.contains(&e) || !well[o] || part_of[o] != usize::MAX {
                    continue;
                }
                part_of[o] = parts;
                stack.push(o);
            }
        }
        if super::exceeds(area, best.0) {
            best = (area, parts);
        }
        parts += 1;
    }
    let inside: Vec<bool> = part_of.iter().map(|&p| p == best.1).collect();
    let which: Vec<usize> = (0..region.len()).filter(|&k| inside[k]).collect();
    if best.1 == usize::MAX || which.len() < 2 {
        return Seamed::Nothing;
    }
    let patch = split(&inside, &cut);
    if let Some(x) = patch.solve(spent) {
        let again = patch.judge(&x, c.cell);
        if again.well {
            return Seamed::Part(which, again.laid);
        }
    }
    let laid = which.iter().map(|&k| judged.laid[k]).collect();
    Seamed::Part(which, laid)
}

/// Projection directions for the whole solid, and each triangle's (an
/// index into them), chosen as Blender's Smart UV Project chooses its
/// own (`docs/research.md` §49): the largest triangle's normal first;
/// every triangle within half the angle of it gathered and their
/// normals, weighted by area, averaged into a direction; then the
/// triangle whose normal lies furthest from every direction so far
/// starts the next, until none lies further than the angle. A triangle
/// takes the direction nearest its normal.
pub(crate) fn directions(
    normals: &[DVec3],
    areas: &[f64],
    largest_first: &[usize],
    angle: f64,
) -> (Vec<DVec3>, Vec<u32>) {
    let cos_half = trig::cos(trig::to_radians(angle / 2.0));
    let cos_whole = trig::cos(trig::to_radians(angle));
    let faces: Vec<usize> = largest_first
        .iter()
        .copied()
        .filter(|&i| normals[i] != DVec3::ZERO)
        .collect();
    let mut list: Vec<DVec3> = Vec::new();
    let mut taken = vec![false; normals.len()];
    let mut start = faces.first().copied();
    while let Some(first) = start {
        let towards = normals[first];
        let mut sum = DVec3::ZERO;
        for &i in &faces {
            if !taken[i] && (i == first || normals[i].dot(towards) > cos_half) {
                taken[i] = true;
                sum += normals[i] * areas[i];
            }
        }
        let mean = unit(sum);
        if mean != DVec3::ZERO {
            list.push(mean);
        }
        // The triangle left whose normal is furthest from them all.
        let mut furthest: Option<(f64, usize)> = None;
        for &i in &faces {
            if taken[i] {
                continue;
            }
            let nearest = list.iter().map(|d| d.dot(normals[i])).fold(-1.0, f64::max);
            if furthest.is_none_or(|(best, _)| super::exceeds(best, nearest)) {
                furthest = Some((nearest, i));
            }
        }
        start = furthest
            .filter(|(nearest, _)| *nearest < cos_whole)
            .map(|(_, i)| i);
    }
    let of = normals
        .iter()
        .map(|n| {
            let mut best = (f64::NEG_INFINITY, 0u32);
            for (k, d) in list.iter().enumerate() {
                let dot = d.dot(*n);
                if super::exceeds(dot, best.0) {
                    best = (dot, k as u32);
                }
            }
            best.1
        })
        .collect();
    (list, of)
}
