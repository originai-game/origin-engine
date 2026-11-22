//! The mesh decimation works on: half-edges, attributes per corner, the queue of candidate collapses and why one is refused.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Interior,
    Line,
    Corner,
}

/// A queued collapse: cheapest first, then by edge, then by direction.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Entry {
    pub(crate) cost: f64,
    /// The part of `cost` that is the skin's (weights changed).
    pub(crate) penalty: f64,
    pub(crate) edge: EdgeKey,
    /// The vertex removed.
    pub(crate) from: u32,
    pub(crate) to: u32,
    pub(crate) pos: DVec3,
    pub(crate) stamp: (u32, u32),
}

impl PartialEq for Entry {
    fn eq(&self, o: &Self) -> bool {
        self.cmp(o) == Ordering::Equal
    }
}

impl Eq for Entry {}

impl PartialOrd for Entry {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl Ord for Entry {
    // Reversed, so the heap pops the cheapest.
    fn cmp(&self, o: &Self) -> Ordering {
        o.cost
            .total_cmp(&self.cost)
            .then(o.edge.cmp(&self.edge))
            .then(o.from.cmp(&self.from))
    }
}

pub(crate) struct Mesh {
    pub(crate) pos: Vec<DVec3>,
    pub(crate) tris: Vec<[u32; 3]>,
    pub(crate) alive: Vec<bool>,
    pub(crate) live: usize,
    /// Per vertex, the live triangles around it.
    pub(crate) around: Vec<Vec<u32>>,
    pub(crate) quadric: Vec<Quadric>,
    pub(crate) kind: Vec<Kind>,
    /// Per vertex, the other ends of its border and feature edges.
    pub(crate) lines: Vec<Vec<u32>>,
    pub(crate) stamp: Vec<u32>,
    /// The vertex each removed one went into.
    pub(crate) into: Vec<u32>,
    /// Only edges shorter than this collapse (0: any).
    pub(crate) max_len: f64,
    /// Per vertex, its bone weights, on a skinned solid.
    pub(crate) weights: Option<Vec<Weights>>,
    /// Per vertex, its morph offsets, on a solid with morph targets.
    pub(crate) morphs: Option<crate::mesh::Morphs>,
    /// Collapses that change weights cost extra.
    pub(crate) keep_borders: bool,
    /// The bend scale, mm.
    pub(crate) bend: f64,
    /// Per triangle, its corners' UVs, on a solid with UVs.
    pub(crate) uvs: Option<Vec<[DVec2; 3]>>,
    /// Per triangle, its chart and the move from its UVs to its chart's
    /// frame, when charts are merged; and the tolerance.
    pub(crate) charts: Option<(Vec<u32>, Vec<Rigid>, f64)>,
    /// Per triangle, its corners' colours, on a solid with colours.
    pub(crate) colors: Option<Vec<CornerColors>>,
    /// Colours are matched apart from UVs, so a colour border does not
    /// hold a collapse (`Decimate::cross_color_borders`).
    pub(crate) cross_colors: bool,
}

/// What a corner carries besides its vertex: its UV and its colour
/// (zero where the solid has none). Corners match when both agree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Attr {
    pub(crate) uv: DVec2,
    pub(crate) color: [f64; 4],
}

impl Attr {
    /// The mean of two, the same bits whichever comes first.
    pub(crate) fn mean(a: Attr, b: Attr) -> Attr {
        Attr {
            uv: (a.uv + b.uv) * 0.5,
            color: core::array::from_fn(|k| (a.color[k] + b.color[k]) * 0.5),
        }
    }
}

/// A corner's new attributes: triangle, corner, attributes.
pub(crate) type UvMove = (u32, usize, Attr);

/// Why a collapse is refused, besides a fold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Refusal {
    Link,
    Turn,
    Uv,
}

impl Mesh {
    pub(crate) fn neighbours(&self, v: u32) -> Vec<u32> {
        let mut n: Vec<u32> = self.around[v as usize]
            .iter()
            .flat_map(|&t| self.tris[t as usize])
            .filter(|&x| x != v)
            .collect();
        n.sort_unstable();
        n.dedup();
        n
    }

    pub(crate) fn normal(&self, t: [u32; 3], moved: u32, to: DVec3) -> DVec3 {
        let p = t.map(|v| if v == moved { to } else { self.pos[v as usize] });
        (p[1] - p[0]).cross(p[2] - p[0])
    }

    /// The collapse `from → to` at `p`, if it keeps the mesh sound.
    pub(crate) fn valid(&self, from: u32, to: u32, p: DVec3) -> bool {
        self.refusal(from, to, p).is_none()
    }

    /// Why the collapse `from → to` at `p` would break the mesh (not
    /// counting folds), if it would.
    pub(crate) fn refusal(&self, from: u32, to: u32, p: DVec3) -> Option<Refusal> {
        // Link condition: the shared neighbours are exactly the third
        // corners of the edge's triangles.
        let b = self.neighbours(to);
        let shared: Vec<u32> = self
            .neighbours(from)
            .into_iter()
            .filter(|x| b.binary_search(x).is_ok())
            .collect();
        let mut opposite: Vec<u32> = self.around[from as usize]
            .iter()
            .map(|&t| self.tris[t as usize])
            .filter(|t| t.contains(&to))
            .flat_map(|t| t.into_iter().filter(|&x| x != from && x != to))
            .collect();
        opposite.sort_unstable();
        opposite.dedup();
        if shared != opposite || opposite.is_empty() {
            return Some(Refusal::Link);
        }
        if self.attr_moves(from, to, p).is_none() {
            return Some(Refusal::Uv);
        }
        // Nothing turns over or loses its area.
        for v in [from, to] {
            for &t in &self.around[v as usize] {
                let tri = self.tris[t as usize];
                if tri.contains(&from) && tri.contains(&to) {
                    continue;
                }
                let before = self.normal(tri, u32::MAX, DVec3::ZERO);
                let after = self.normal(tri, v, p);
                if after == DVec3::ZERO || after.dot(before) <= 0.0 {
                    return Some(Refusal::Turn);
                }
            }
        }
        None
    }

    /// The triangle's corners and positions after the collapse
    /// `from → to` at `p` (unchanged when it has neither).
    pub(crate) fn moved(&self, t: u32, from: u32, to: u32, p: DVec3) -> ([u32; 3], [DVec3; 3]) {
        let ids = self.tris[t as usize].map(|v| if v == from { to } else { v });
        let pos = ids.map(|v| if v == to { p } else { self.pos[v as usize] });
        (ids, pos)
    }

    /// The live triangles around `from` or `to`, sorted.
    pub(crate) fn fan(&self, from: u32, to: u32) -> Vec<u32> {
        let mut f: Vec<u32> = self.around[from as usize]
            .iter()
            .chain(&self.around[to as usize])
            .copied()
            .collect();
        f.sort_unstable();
        f.dedup();
        f
    }

    /// Crossing pairs of `tris` (ids and positions) with the live
    /// triangles near them outside `fan`, and among themselves: the pairs,
    /// as (index in `tris`, partner or `u32::MAX` for one of `tris`).
    /// Stops at the first when `first`.
    pub(crate) fn crossings(
        &self,
        grid: &Grid,
        tris: &[([u32; 3], [DVec3; 3])],
        fan: &[u32],
        first: bool,
    ) -> Vec<(usize, u32)> {
        let mut out = Vec::new();
        let boxes: Vec<(DVec3, DVec3)> = tris.iter().map(|t| bounds(t.1)).collect();
        let (lo, hi) = boxes
            .iter()
            .fold((DVec3::INFINITY, DVec3::NEG_INFINITY), |(l, h), b| {
                (l.min(b.0), h.max(b.1))
            });
        // One query for the whole fan; each candidate's box against each
        // triangle's.
        let mut near: Vec<Near> = Vec::new();
        let mut ids: Vec<u32> = Vec::new();
        grid.near(lo, hi, &mut ids);
        for c in ids {
            if !self.alive[c as usize] || fan.binary_search(&c).is_ok() {
                continue;
            }
            let ib = self.tris[c as usize];
            let b = ib.map(|v| self.pos[v as usize]);
            let bb = bounds(b);
            if lo.cmple(bb.1).all() && bb.0.cmple(hi).all() && !degenerate(b) {
                near.push((c, ib, b, bb));
            }
        }
        for (i, &(ia, a)) in tris.iter().enumerate() {
            let (lo, hi) = boxes[i];
            for &(c, ib, b, (blo, bhi)) in &near {
                if lo.cmple(bhi).all() && blo.cmple(hi).all() && crossing(ia, a, ib, b) {
                    out.push((i, c));
                    if first {
                        return out;
                    }
                }
            }
            for &(ib, b) in &tris[i + 1..] {
                let (blo, bhi) = bounds(b);
                if lo.cmple(bhi).all() && blo.cmple(hi).all() && crossing(ia, a, ib, b) {
                    out.push((i, u32::MAX));
                    if first {
                        return out;
                    }
                }
            }
        }
        out
    }

    /// The collapse `from → to` at `p` would make the surface pass through
    /// itself: a rewritten triangle would be flat (exactly), or cross a
    /// triangle it did not cross before, or the fan would cross more
    /// pairs than it does now.
    pub(crate) fn folds(&self, grid: &Grid, from: u32, to: u32, p: DVec3) -> bool {
        let fan = self.fan(from, to);
        let after: Vec<([u32; 3], [DVec3; 3])> = fan
            .iter()
            .filter(|&&t| {
                let tri = self.tris[t as usize];
                !(tri.contains(&from) && tri.contains(&to))
            })
            .map(|&t| self.moved(t, from, to, p))
            .collect();
        if after.iter().any(|t| degenerate(t.1)) {
            return true;
        }
        if self.crossings(grid, &after, &fan, true).is_empty() {
            return false;
        }
        // The fan crosses something after: allowed only if it already did,
        // with the same partners and no more pairs.
        let before: Vec<([u32; 3], [DVec3; 3])> = fan
            .iter()
            .map(|&t| {
                let ids = self.tris[t as usize];
                (ids, ids.map(|v| self.pos[v as usize]))
            })
            .filter(|t| !degenerate(t.1))
            .collect();
        let was = self.crossings(grid, &before, &fan, false);
        let now = self.crossings(grid, &after, &fan, false);
        if now.len() > was.len() {
            return true;
        }
        let mut partners: Vec<u32> = was.iter().map(|x| x.1).collect();
        partners.sort_unstable();
        now.iter()
            .any(|x| x.1 != u32::MAX && partners.binary_search(&x.1).is_err())
    }

    /// A corner's attributes.
    pub(crate) fn attr(&self, t: u32, k: usize) -> Attr {
        Attr {
            uv: self.uvs.as_ref().map_or(DVec2::ZERO, |u| u[t as usize][k]),
            color: self
                .colors
                .as_ref()
                .map_or([0.0; 4], |c| crate::mesh::wide(c[t as usize][k])),
        }
    }

    /// The corners' new UVs and colours for the collapse `from → to` at
    /// `p`: empty on a solid with neither; `None` if they cannot be
    /// matched across a seam, or a UV triangle would turn over or
    /// flatten.
    pub(crate) fn attr_moves(&self, from: u32, to: u32, p: DVec3) -> Option<Vec<UvMove>> {
        if self.uvs.is_none() && self.colors.is_none() {
            return Some(Vec::new());
        }
        let corner = |t: u32, v: u32| self.tris[t as usize].iter().position(|&x| x == v);
        // The edge's own triangles pair each side's attributes at the two
        // ends.
        let mut pairs: Vec<(Attr, Attr)> = Vec::new();
        for &t in &self.around[from as usize] {
            if let (Some(kf), Some(kt)) = (corner(t, from), corner(t, to)) {
                pairs.push((self.attr(t, kf), self.attr(t, kt)));
            }
        }
        // Crossing colour borders, corners match by UV alone, and each
        // takes the other end's colour on the edge (their mean where the
        // edge's triangles differ there).
        let key = |a: Attr| {
            if self.cross_colors {
                Attr {
                    color: [0.0; 4],
                    ..a
                }
            } else {
                a
            }
        };
        let end_color = |from_end: bool| -> [f64; 4] {
            let mut cs: Vec<[f64; 4]> = pairs
                .iter()
                .map(|&(a, b)| if from_end { b.color } else { a.color })
                .collect();
            cs.sort_by(|a, b| {
                (0..4)
                    .map(|k| a[k].total_cmp(&b[k]))
                    .find(|o| o.is_ne())
                    .unwrap_or(Ordering::Equal)
            });
            cs.dedup();
            let n = cs.len().max(1) as f64;
            core::array::from_fn(|k| cs.iter().map(|c| c[k]).sum::<f64>() / n)
        };
        let midpoint = p != self.pos[to as usize];
        let mut moves: Vec<UvMove> = Vec::new();
        // One end's attributes mapped to the other's, if exactly one
        // pairing has them; `Err` if two disagree.
        let other = |u: Attr, from_end: bool| -> Result<Option<Attr>, ()> {
            let mut found: Option<Attr> = None;
            for &(a, b) in &pairs {
                let (mine, theirs) = if from_end { (a, b) } else { (b, a) };
                if key(mine) == key(u) {
                    if found.is_some_and(|f| key(f) != key(theirs)) {
                        return Err(());
                    }
                    found = Some(theirs);
                }
            }
            Ok(found)
        };
        // A corner across a merged seam: through the charts' frames, from
        // an edge triangle of the same chart whose corner at `v` is the
        // same point of the chart (within the tolerance). The corner's
        // new UV in the chart's frame: the other end's (or the midpoint),
        // moved back into the corner's own UVs.
        // The colour does not go through the move: the corner's must be
        // the edge triangle's at `v`, and it takes that triangle's at `w`
        // (or their mean), as on its own side of a seam.
        let across = |t: u32, u: Attr, v: u32, w: u32| -> Option<Attr> {
            let (chart, frame, tol) = self.charts.as_ref()?;
            let uvs = self.uvs.as_ref()?;
            let e = self.around[from as usize]
                .iter()
                .copied()
                .find(|&e| corner(e, to).is_some() && chart[e as usize] == chart[t as usize])?;
            let (fe, ft) = (frame[e as usize], frame[t as usize]);
            let (kv, kw) = (corner(e, v)?, corner(e, w)?);
            let uv = uvs[e as usize];
            let (mine, theirs) = (uv[kv], uv[kw]);
            if (fe.apply(mine) - ft.apply(u.uv)).length() > *tol {
                return None;
            }
            let (cm, ct) = (self.attr(e, kv), self.attr(e, kw));
            if !self.cross_colors && cm.color != u.color {
                return None;
            }
            let (goal, color) = if midpoint {
                (
                    (fe.apply(mine) + fe.apply(theirs)) * 0.5,
                    Attr::mean(cm, ct).color,
                )
            } else {
                (fe.apply(theirs), ct.color)
            };
            Some(Attr {
                uv: ft.unapply(goal),
                color,
            })
        };
        for (v, w, from_end) in [(from, to, true), (to, from, false)] {
            if !from_end && !midpoint {
                break;
            }
            for &t in &self.around[v as usize] {
                if corner(t, w).is_some() {
                    continue;
                }
                let k = corner(t, v)?;
                let u = self.attr(t, k);
                let new = match other(u, from_end).ok()? {
                    Some(o) => match (midpoint, from_end) {
                        (false, _) => o,
                        (true, true) => Attr::mean(u, o),
                        (true, false) => Attr::mean(o, u),
                    },
                    None => across(t, u, v, w)?,
                };
                let new = if self.cross_colors {
                    let o = end_color(from_end);
                    Attr {
                        color: if midpoint {
                            core::array::from_fn(|ch| (u.color[ch] + o[ch]) * 0.5)
                        } else {
                            o
                        },
                        ..new
                    }
                } else {
                    new
                };
                moves.push((t, k, new));
            }
        }
        if let Some(uvs) = &self.uvs {
            for &(t, k, new) in &moves {
                let before = uvs[t as usize];
                let mut after = before;
                after[k] = new.uv;
                let (b, a) = (
                    orient2d(before[0], before[1], before[2]),
                    orient2d(after[0], after[1], after[2]),
                );
                if b.is_ne() && a != b {
                    return None;
                }
            }
        }
        Some(moves)
    }

    /// The best allowed collapse of edge `a b`, if any.
    pub(crate) fn plan(&self, a: u32, b: u32) -> Option<Entry> {
        if self.max_len > 0.0
            && (self.pos[a as usize] - self.pos[b as usize]).length() >= self.max_len
        {
            return None;
        }
        let edge = edge_key(a, b);
        let on_line = self.lines[a as usize].contains(&b);
        let mut best: Option<Entry> = None;
        for (from, to) in [(a, b), (b, a)] {
            let (kf, kt) = (self.kind[from as usize], self.kind[to as usize]);
            let allowed = match kf {
                Kind::Interior => true,
                Kind::Line => on_line && kt != Kind::Interior,
                Kind::Corner => false,
            };
            if !allowed {
                continue;
            }
            let mut q = self.quadric[from as usize];
            q.add(&self.quadric[to as usize]);
            let (pf, pt) = (self.pos[from as usize], self.pos[to as usize]);
            let mut options = vec![pt];
            if kf == Kind::Interior && kt == Kind::Interior {
                options.push((pf + pt) * 0.5);
            }
            // On a skinned solid: what changing the weights costs, as if the
            // surface moved by the weight change times the bend scale.
            let penalty = match (&self.weights, self.keep_borders) {
                (Some(w), true) => {
                    // Removing `from` changes how weights blend over its
                    // ring: the largest difference from it to a neighbour.
                    let wf = &w[from as usize];
                    let step = self
                        .neighbours(from)
                        .iter()
                        .map(|&n| weight_step(wf, &w[n as usize]))
                        .fold(0.0, f64::max);
                    let d = step * self.bend;
                    q.w * d * d
                }
                _ => 0.0,
            };
            for p in options {
                let e = Entry {
                    cost: q.eval(p) + penalty,
                    penalty,
                    edge,
                    from,
                    to,
                    pos: p,
                    stamp: (self.stamp[from as usize], self.stamp[to as usize]),
                };
                if best.is_none_or(|b| e.cmp(&b) == Ordering::Greater) {
                    best = Some(e);
                }
            }
        }
        best
    }

    pub(crate) fn collapse(&mut self, e: &Entry) {
        let (from, to) = (e.from, e.to);
        let moves = self.attr_moves(from, to, e.pos).unwrap_or_default();
        for (t, k, new) in moves {
            if let Some(uvs) = &mut self.uvs {
                uvs[t as usize][k] = new.uv;
            }
            if let Some(c) = &mut self.colors {
                c[t as usize][k] = crate::mesh::stored(new.color);
            }
        }
        let around = std::mem::take(&mut self.around[from as usize]);
        for &t in &around {
            let tri = self.tris[t as usize];
            if tri.contains(&to) {
                self.alive[t as usize] = false;
                self.live -= 1;
                for v in tri {
                    if v != from {
                        self.around[v as usize].retain(|&x| x != t);
                    }
                }
            } else {
                for v in &mut self.tris[t as usize] {
                    if *v == from {
                        *v = to;
                    }
                }
                self.around[to as usize].push(t);
            }
        }
        self.around[to as usize].sort_unstable();
        if let Some(w) = &mut self.weights {
            // The kept vertex's weights by where it lands between the two:
            // its own at its own position, half and half at the midpoint.
            let (wt, wf) = (w[to as usize], w[from as usize]);
            if e.pos != self.pos[to as usize] {
                let mut items = [(wt, 0.5), (wf, 0.5)];
                items.sort_by(|a, b| a.0.cmp_total(&b.0));
                w[to as usize] = Weights::blend(&items);
            }
        }
        if let Some(m) = &mut self.morphs
            && e.pos != self.pos[to as usize]
        {
            // Likewise the kept vertex's morph offsets.
            m.blend(to as usize, &[(to as usize, 0.5), (from as usize, 0.5)]);
        }
        self.pos[to as usize] = e.pos;
        let q = self.quadric[from as usize];
        self.quadric[to as usize].add(&q);
        // Lines through `from` now run through `to`.
        for x in std::mem::take(&mut self.lines[from as usize]) {
            self.lines[x as usize].retain(|&y| y != from);
            if x != to {
                if !self.lines[x as usize].contains(&to) {
                    self.lines[x as usize].push(to);
                }
                if !self.lines[to as usize].contains(&x) {
                    self.lines[to as usize].push(x);
                }
            }
        }
        self.into[from as usize] = to;
        self.stamp[from as usize] += 1;
        self.stamp[to as usize] += 1;
    }

    /// The collapse's error, mm, without the skin's penalty.
    pub(crate) fn error_of(&self, e: &Entry) -> f64 {
        let mut q = self.quadric[e.from as usize];
        q.add(&self.quadric[e.to as usize]);
        if q.w > 0.0 {
            ((e.cost - e.penalty).max(0.0) / q.w).sqrt()
        } else {
            0.0
        }
    }

    /// `a` and `b` are both still there and joined by an edge.
    pub(crate) fn joined(&self, a: u32, b: u32) -> bool {
        self.into[a as usize] == a
            && self.into[b as usize] == b
            && self.around[a as usize]
                .iter()
                .any(|&t| self.tris[t as usize].contains(&b))
    }

    /// Every live edge under the first rule that refuses its cheapest
    /// collapse.
    pub(crate) fn blocked(&self, grid: &Grid, d: &Decimate) -> Blocked {
        let mut edges: Vec<EdgeKey> = Vec::new();
        for (t, tri) in self.tris.iter().enumerate() {
            if self.alive[t] {
                for k in 0..3 {
                    edges.push(edge_key(tri[k], tri[(k + 1) % 3]));
                }
            }
        }
        edges.sort_unstable();
        edges.dedup();
        let mut b = Blocked::default();
        for (x, y) in edges {
            let Some(e) = self.plan(x, y) else {
                b.fixed += 1;
                continue;
            };
            if d.max_error > 0.0 && self.error_of(&e) > d.max_error {
                b.error += 1;
                continue;
            }
            match self.refusal(e.from, e.to, e.pos) {
                Some(Refusal::Link) => b.link += 1,
                Some(Refusal::Turn) => b.turn += 1,
                Some(Refusal::Uv) => b.uv += 1,
                None if self.folds(grid, e.from, e.to, e.pos) => b.fold += 1,
                None => b.not_retried += 1,
            }
        }
        b
    }

    pub(crate) fn root(&self, mut v: u32) -> u32 {
        while self.into[v as usize] != v {
            v = self.into[v as usize];
        }
        v
    }
}
