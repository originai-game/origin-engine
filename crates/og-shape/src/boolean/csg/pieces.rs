//! Step 2: each crossed triangle cut into pieces by its segments (constrained Delaunay, exact where the rounded layout fails).

use super::*;

impl Csg {
    /// [`Csg::pieces`] in the first pass: positions as rounded, and spade
    /// keeps what it can (the result's check refuses an open result).
    pub(crate) fn pieces_plain(&self, m: u8, tri: u32) -> Vec<[u32; 3]> {
        let t = self.solids[m as usize].tris[tri as usize];
        let Some(cut) = self.cuts[m as usize].get(&tri) else {
            return vec![t];
        };
        let normal = self.tri_normal(t);
        let axis = drop_axis(normal);
        // The outline: each edge with the points on it, in order.
        let mut chain: Vec<u32> = Vec::new();
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            let e = edge_key(m, a, b);
            let (pa, pb) = (self.p(a), self.p(b));
            let mut along: Vec<u32> = cut
                .points
                .iter()
                .copied()
                .filter(|&i| i != a && i != b && self.on[i as usize].contains(&e))
                .collect();
            crate::math::sort::sort_by(&mut along, |&x, &y| {
                (self.p(x) - pa)
                    .dot(pb - pa)
                    .total_cmp(&(self.p(y) - pa).dot(pb - pa))
                    .then(x.cmp(&y))
            });
            chain.push(a);
            chain.extend(along);
        }
        // A point made inside the triangle (an edge through it) can round to
        // a position on or past its outline, as seen in the projection: past
        // a corner's edge, or past a point rounded off an edge. Its segments
        // would then run along the outline or through an outline point, and
        // spade would merge or drop them, so regions would leak or pieces be
        // lost. It is placed in the triangulation just inside the outline
        // instead; its own position (shared with the other mesh) is
        // unchanged, as are which segments end at it (`docs/research.md`
        // §5.4, follow-up 4).
        let outline: Vec<DVec2> = chain.iter().map(|&i| project(self.p(i), axis)).collect();
        let v = t.map(|i| project(self.p(i), axis));
        let turn = o2(v[0], v[1], v[2]);
        let centre = (v[0] + v[1] + v[2]) / 3.0;
        let at = |i: u32| {
            let q = project(self.p(i), axis);
            // A point of the other mesh's at one of ours keeps its place, so
            // spade merges the two.
            let shared = chain.iter().any(|&j| self.p(j) == self.p(i));
            if turn == 0 || shared || chain.contains(&i) || strictly_inside(&outline, q) {
                return q;
            }
            // The smallest step towards the centre, by powers of two, that
            // puts it strictly inside.
            let mut step = f64::EPSILON;
            while step < 0.5 {
                let r = q + (centre - q) * step;
                if strictly_inside(&outline, r) {
                    return r;
                }
                step *= 2.0;
            }
            centre
        };
        let mut cdt: ConstrainedDelaunayTriangulation<Point2<f64>> =
            ConstrainedDelaunayTriangulation::new();
        let mut id_of: BTreeMap<usize, u32> = BTreeMap::new();
        let mut handle_of: BTreeMap<u32, spade::handles::FixedVertexHandle> = BTreeMap::new();
        for &i in chain.iter().chain(cut.points.iter()) {
            if handle_of.contains_key(&i) {
                continue;
            }
            let q = at(i);
            let Ok(h) = cdt.insert(Point2::new(q.x, q.y)) else {
                continue;
            };
            id_of.entry(h.index()).or_insert(i);
            handle_of.insert(i, h);
        }
        let mut boundary: BTreeSet<(usize, usize)> = BTreeSet::new();
        for k in 0..chain.len() {
            let (a, b) = (chain[k], chain[(k + 1) % chain.len()]);
            let (Some(&ha), Some(&hb)) = (handle_of.get(&a), handle_of.get(&b)) else {
                continue;
            };
            if ha != hb {
                cdt.try_add_constraint(ha, hb);
                boundary.insert((ha.index().min(hb.index()), ha.index().max(hb.index())));
            }
        }
        for s in &cut.segments {
            let (Some(&ha), Some(&hb)) = (handle_of.get(&s[0]), handle_of.get(&s[1])) else {
                continue;
            };
            if ha != hb {
                cdt.try_add_constraint(ha, hb);
            }
        }
        let faces: Vec<[usize; 3]> = cdt
            .inner_faces()
            .map(|f| f.vertices().map(|v| v.fix().index()))
            .collect();
        // Flood from the hull through every edge that is not the outline:
        // what it reaches lies outside the triangle.
        let key = |a: usize, b: usize| (a.min(b), a.max(b));
        let mut by_edge: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
        for (fi, f) in faces.iter().enumerate() {
            for k in 0..3 {
                by_edge
                    .entry(key(f[k], f[(k + 1) % 3]))
                    .or_default()
                    .push(fi);
            }
        }
        let mut outside = vec![false; faces.len()];
        let mut queue: Vec<usize> = Vec::new();
        for (fi, f) in faces.iter().enumerate() {
            for k in 0..3 {
                let e = key(f[k], f[(k + 1) % 3]);
                if by_edge[&e].len() == 1 && !boundary.contains(&e) && !outside[fi] {
                    outside[fi] = true;
                    queue.push(fi);
                }
            }
        }
        while let Some(fi) = queue.pop() {
            let f = faces[fi];
            for k in 0..3 {
                let e = key(f[k], f[(k + 1) % 3]);
                if boundary.contains(&e) {
                    continue;
                }
                for &g in &by_edge[&e] {
                    if !outside[g] {
                        outside[g] = true;
                        queue.push(g);
                    }
                }
            }
        }
        // spade's faces turn counter-clockwise in the projection; the
        // triangle's own winding there decides whether they turn back.
        let flip = turn < 0;
        let mut out = Vec::new();
        for (fi, f) in faces.iter().enumerate() {
            if outside[fi] {
                continue;
            }
            let ids = f.map(|h| id_of[&h]);
            if ids[0] == ids[1] || ids[1] == ids[2] || ids[0] == ids[2] {
                continue;
            }
            out.push(if flip { [ids[0], ids[2], ids[1]] } else { ids });
        }
        out
    }

    /// The points `ids` on edge `e`, in order from its end `from`: sorted
    /// by their rounded positions, and where two of those lie within 2⁻³⁰
    /// of the edge of each other, by their exact places on it
    /// ([`Csg::param`]), so the two triangles that share the edge put them
    /// in one order, and in the order the operands give them.
    pub(crate) fn along_edge(&self, e: EdgeKey, mut ids: Vec<u32>, from: u32) -> Vec<u32> {
        let (p, q) = (self.p(e.1), self.p(e.2));
        let d = q - p;
        let u = |i: u32| (self.p(i) - p).dot(d);
        crate::math::sort::sort_by(&mut ids, |&x, &y| u(x).total_cmp(&u(y)).then(x.cmp(&y)));
        let tol = d.length_squared() * (1.0 / (1u64 << 30) as f64);
        let mut start = 0;
        while start < ids.len() {
            let mut end = start + 1;
            while end < ids.len() && u(ids[end]) - u(ids[end - 1]) <= tol {
                end += 1;
            }
            if end - start > 1 {
                let params: Vec<(u32, Option<exact::Fraction>)> = ids[start..end]
                    .iter()
                    .map(|&i| (i, self.param(i, e)))
                    .collect();
                let cmp = |a: &(u32, Option<exact::Fraction>),
                           b: &(u32, Option<exact::Fraction>)| {
                    match (&a.1, &b.1) {
                        (Some(x), Some(y)) => x.cmp(y),
                        _ => u(a.0).total_cmp(&u(b.0)),
                    }
                    .then(a.0.cmp(&b.0))
                };
                // Insertion sort: it never needs the order to be total.
                let mut run = params;
                for i in 1..run.len() {
                    let mut j = i;
                    while j > 0 && cmp(&run[j - 1], &run[j]) == std::cmp::Ordering::Greater {
                        run.swap(j - 1, j);
                        j -= 1;
                    }
                }
                for (k, (i, _)) in run.into_iter().enumerate() {
                    ids[start + k] = i;
                }
            }
            start = end;
        }
        if from != e.1 {
            ids.reverse();
        }
        ids
    }

    /// Where point `x` lies along edge `e` (from its lower id), exactly:
    /// for an operand's vertex on the edge, a crossing of the edge with a
    /// triangle's plane, or with another edge. None for a point that only
    /// rounded onto the edge, or a self-union's triple point.
    pub(crate) fn param(&self, x: u32, e: EdgeKey) -> Option<exact::Fraction> {
        let arr = |i: u32| self.p(i).to_array();
        let (p, q) = (arr(e.1), arr(e.2));
        match self.origin_of[x as usize] {
            None => {
                let d = self.p(e.2) - self.p(e.1);
                let k = drop_axis(d);
                let v = arr(x);
                exact::Fraction::new(exact::Exp::diff(v[k], p[k]), exact::Exp::diff(q[k], p[k]))
            }
            Some(Origin::EdgeTri(f, om, tri)) if f == e => {
                let t = self.solids[om as usize].tris[tri as usize].map(arr);
                let dp = exact::orient3d(t[0], t[1], t[2], p);
                let dq = exact::orient3d(t[0], t[1], t[2], q);
                let den = dp.sub(&dq);
                exact::Fraction::new(dp, den)
            }
            Some(Origin::EdgeEdge(f, g)) if f == e || g == e => {
                let o = if f == e { g } else { f };
                let n = (self.p(e.2) - self.p(e.1)).cross(self.p(o.2) - self.p(o.1));
                let axis = drop_axis(n);
                let pr = |i: u32| project(self.p(i), axis).to_array();
                let dp = exact::orient2d(pr(o.1), pr(o.2), pr(e.1));
                let dq = exact::orient2d(pr(o.1), pr(o.2), pr(e.2));
                let den = dp.sub(&dq);
                exact::Fraction::new(dp, den)
            }
            _ => None,
        }
    }

    /// Triangle `tri` of mesh `m` cut into pieces (global ids, the
    /// triangle's own winding).
    pub(crate) fn pieces(&self, m: u8, tri: u32) -> Vec<[u32; 3]> {
        if !self.robust {
            return self.pieces_plain(m, tri);
        }
        let t = self.solids[m as usize].tris[tri as usize];
        let Some(cut) = self.cuts[m as usize].get(&tri) else {
            return vec![t];
        };
        let normal = self.tri_normal(t);
        let axis = drop_axis(normal);
        // The outline: each edge with the points on it, in order.
        let mut chain: Vec<u32> = Vec::new();
        let mut sides: Vec<usize> = Vec::new();
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            let e = edge_key(m, a, b);
            let along: Vec<u32> = cut
                .points
                .iter()
                .copied()
                .filter(|&i| i != a && i != b && self.on[i as usize].contains(&e))
                .collect();
            let along = self.along_edge(e, along, a);
            chain.push(a);
            sides.push(3);
            sides.extend(along.iter().map(|_| k));
            chain.extend(along);
        }
        // A point made inside the triangle (an edge through it) can round to
        // a position on or past its outline, as seen in the projection: past
        // a corner's edge, or past a point rounded off an edge. Its segments
        // would then run along the outline or through an outline point, and
        // spade would merge or drop them, so regions would leak or pieces be
        // lost. It is placed in the triangulation just inside the outline
        // instead; its own position (shared with the other mesh) is
        // unchanged, as are which segments end at it (`docs/research.md`
        // §5.4, follow-up 4).
        self.chains.borrow_mut().push(chain.clone());
        let outline: Vec<DVec2> = chain.iter().map(|&i| project(self.p(i), axis)).collect();
        let v = t.map(|i| project(self.p(i), axis));
        let turn = o2(v[0], v[1], v[2]);
        let at = |i: u32| {
            let q = project(self.p(i), axis);
            // A point of the other mesh's at one of ours keeps its place, so
            // spade merges the two.
            let shared = chain.iter().any(|&j| self.p(j) == self.p(i));
            if turn == 0 || shared || chain.contains(&i) {
                return q;
            }
            inward(&outline, (v[0] + v[1] + v[2]) / 3.0, q)
        };
        if let Some(out) = self.triangulate(&chain, cut, at, turn < 0) {
            return out;
        }
        // The rounded outline is not a simple polygon, or a segment was lost
        // or merged into another: points made a rounding error from a corner
        // or from each other (operands that touch, `docs/research.md` §5.4,
        // item 5). The triangle is triangulated again in a layout of its
        // own where the outline is exact: the corners at (0, 0), (1, 0) and
        // (0, 1), and each point on an edge at a distinct multiple of 2⁻⁵²
        // along it, in its order. Points inside are placed by their
        // barycentric coordinates, just inside the outline. Only which
        // points the pieces join comes from the layout; every position is
        // unchanged.
        let layout = self.exact_layout(t, axis, &chain, &sides);
        let corners = [DVec2::ZERO, DVec2::X, DVec2::Y];
        let square: Vec<DVec2> = chain.iter().map(|i| layout[i]).collect();
        let centre = (corners[0] + corners[1] + corners[2]) / 3.0;
        let mut place: BTreeMap<u32, DVec2> = layout.clone();
        // Where each place is taken, and by which position.
        let mut taken: BTreeMap<[u64; 2], u32> = BTreeMap::new();
        for (&i, q) in &layout {
            taken.insert(q.to_array().map(f64::to_bits), i);
        }
        for &i in &cut.points {
            if place.contains_key(&i) {
                continue;
            }
            if let Some(&j) = chain.iter().find(|&&j| self.p(j) == self.p(i)) {
                place.insert(i, layout[&j]);
                continue;
            }
            let q = project(self.p(i), axis);
            let den = (v[1] - v[0]).perp_dot(v[2] - v[0]);
            let b = if den != 0.0 {
                DVec2::new(
                    (q - v[0]).perp_dot(v[2] - v[0]) / den,
                    (v[1] - v[0]).perp_dot(q - v[0]) / den,
                )
            } else {
                centre
            };
            let mut q = inward(&square, centre, b);
            // Two points at different positions that this layout cannot
            // tell apart: the later one moves along one of its segments
            // (towards the point at its other end, as rounded) by the
            // smallest step that frees it, staying inside.
            let busy = |q: DVec2, taken: &BTreeMap<[u64; 2], u32>| {
                taken
                    .get(&q.to_array().map(f64::to_bits))
                    .is_some_and(|&j| self.p(j) != self.p(i))
            };
            if busy(q, &taken) {
                let toward = cut
                    .segments
                    .iter()
                    .find_map(|s| {
                        let k = if s[0] == i {
                            s[1]
                        } else if s[1] == i {
                            s[0]
                        } else {
                            return None;
                        };
                        Some(place.get(&k).copied().unwrap_or(centre))
                    })
                    .unwrap_or(centre);
                let mut step = f64::EPSILON;
                while step < 0.5 {
                    let r = q + (toward - q) * step;
                    if !busy(r, &taken) && strictly_inside(&square, r) {
                        q = r;
                        break;
                    }
                    step *= 2.0;
                }
            }
            taken.entry(q.to_array().map(f64::to_bits)).or_insert(i);
            place.insert(i, q);
        }
        let at = |i: u32| place.get(&i).copied().unwrap_or(centre);
        self.triangulate(&chain, cut, at, false)
            .unwrap_or_else(|| self.triangulate_lossy(&chain, cut, at, false))
    }

    /// Each outline point's place in [`Csg::pieces`]' exact layout: the
    /// corners at (0, 0), (1, 0), (0, 1), and the points of edge `k` at
    /// distinct multiples of 2⁻⁵² along it, in chain order, as near their
    /// own fraction of the edge as that allows.
    pub(crate) fn exact_layout(
        &self,
        t: [u32; 3],
        axis: usize,
        chain: &[u32],
        sides: &[usize],
    ) -> BTreeMap<u32, DVec2> {
        const N: i64 = 1 << 52;
        let mut layout: BTreeMap<u32, DVec2> = BTreeMap::new();
        let corners = [DVec2::ZERO, DVec2::X, DVec2::Y];
        for k in 0..3 {
            layout.insert(t[k], corners[k]);
            let (a, b) = (t[k], t[(k + 1) % 3]);
            let ids: Vec<u32> = chain
                .iter()
                .zip(sides)
                .filter(|&(_, &s)| s == k)
                .map(|(&i, _)| i)
                .collect();
            if ids.is_empty() {
                continue;
            }
            let (pa, pb) = (project(self.p(a), axis), project(self.p(b), axis));
            let d = pb - pa;
            let l2 = d.length_squared();
            let raw: Vec<i64> = ids
                .iter()
                .map(|&i| {
                    let u = if l2 > 0.0 {
                        (project(self.p(i), axis) - pa).dot(d) / l2
                    } else {
                        0.5
                    };
                    (u.clamp(0.0, 1.0) * N as f64) as i64
                })
                .collect();
            // Points at one position share a place, and so does every
            // point between two of them (they are made one point later,
            // `merge_points`); so do points at a corner's position, with
            // the corner.
            let key = |i: u32| self.p(i).to_array().map(|x| (x + 0.0).to_bits());
            let mut group: Vec<usize> = (0..ids.len()).collect();
            let mut first: BTreeMap<[u64; 3], usize> = BTreeMap::new();
            for (k, &i) in ids.iter().enumerate() {
                match first.get(&key(i)) {
                    Some(&j) => {
                        let gj = group[j];
                        for g in &mut group[j..=k] {
                            *g = gj;
                        }
                    }
                    None => {
                        first.insert(key(i), k);
                    }
                }
            }
            let at_a = |i: u32| key(i) == key(a);
            let at_b = |i: u32| key(i) == key(b);
            let mut ks: Vec<i64> = vec![0; ids.len()];
            let mut prev = 0;
            for k in 0..ids.len() {
                ks[k] = if k > 0 && group[k] == group[k - 1] {
                    ks[k - 1]
                } else if group[..=k]
                    .iter()
                    .zip(&ids)
                    .any(|(&g, &i)| g == group[k] && at_a(i))
                {
                    0
                } else {
                    raw[k].max(prev + 1)
                };
                prev = ks[k];
            }
            let mut next = N;
            for k in (0..ids.len()).rev() {
                let same = k + 1 < ids.len() && group[k] == group[k + 1];
                let tail = group[k..]
                    .iter()
                    .zip(&ids[k..])
                    .any(|(&g, &i)| g == group[k] && at_b(i));
                ks[k] = if same {
                    ks[k + 1]
                } else if tail {
                    N
                } else if ks[k] == 0 && at_a(ids[k]) {
                    0
                } else {
                    ks[k].min(next - 1)
                };
                next = ks[k];
            }
            for (&i, &x) in ids.iter().zip(&ks) {
                let (f, g) = (x as f64 / N as f64, (N - x) as f64 / N as f64);
                let q = match k {
                    0 => DVec2::new(f, 0.0),
                    1 => DVec2::new(g, f),
                    _ => DVec2::new(0.0, g),
                };
                layout.insert(i, q);
            }
        }
        layout
    }

    /// The pieces of a cut triangle, from a constrained Delaunay
    /// triangulation of its outline (`chain`) and segments with each point
    /// at `at`; None unless every outline edge and segment came out as one
    /// edge and no two points at different positions merged.
    pub(crate) fn triangulate(
        &self,
        chain: &[u32],
        cut: &Cut,
        at: impl Fn(u32) -> DVec2,
        flip: bool,
    ) -> Option<Vec<[u32; 3]>> {
        self.cdt_pieces(chain, cut, at, flip, true)
    }

    /// [`Csg::triangulate`], keeping what it can when the triangulation is
    /// not faithful (the result's check then refuses an open result).
    pub(crate) fn triangulate_lossy(
        &self,
        chain: &[u32],
        cut: &Cut,
        at: impl Fn(u32) -> DVec2,
        flip: bool,
    ) -> Vec<[u32; 3]> {
        self.cdt_pieces(chain, cut, at, flip, false)
            .unwrap_or_default()
    }

    pub(crate) fn cdt_pieces(
        &self,
        chain: &[u32],
        cut: &Cut,
        at: impl Fn(u32) -> DVec2,
        flip: bool,
        strict: bool,
    ) -> Option<Vec<[u32; 3]>> {
        let mut cdt: ConstrainedDelaunayTriangulation<Point2<f64>> =
            ConstrainedDelaunayTriangulation::new();
        let mut id_of: BTreeMap<usize, u32> = BTreeMap::new();
        let mut handle_of: BTreeMap<u32, spade::handles::FixedVertexHandle> = BTreeMap::new();
        let mut faithful = true;
        for &i in chain.iter().chain(cut.points.iter()) {
            if handle_of.contains_key(&i) {
                continue;
            }
            let q = at(i);
            let Ok(h) = cdt.insert(Point2::new(q.x, q.y)) else {
                faithful = false;
                continue;
            };
            match id_of.get(&h.index()) {
                Some(&j) if self.p(j) != self.p(i) => {
                    if strict {
                        faithful = false;
                    } else if self.robust {
                        self.merges.borrow_mut().push((j, i));
                    }
                }
                Some(&j) => {
                    if self.robust {
                        self.merges.borrow_mut().push((j, i));
                    }
                }
                None => {
                    id_of.insert(h.index(), i);
                }
            }
            handle_of.insert(i, h);
        }
        let mut boundary: BTreeSet<(usize, usize)> = BTreeSet::new();
        for k in 0..chain.len() {
            let (a, b) = (chain[k], chain[(k + 1) % chain.len()]);
            let (Some(&ha), Some(&hb)) = (handle_of.get(&a), handle_of.get(&b)) else {
                continue;
            };
            if ha != hb {
                if cdt.try_add_constraint(ha, hb).len() != 1 {
                    faithful = false;
                }
                boundary.insert((ha.index().min(hb.index()), ha.index().max(hb.index())));
            }
        }
        for s in &cut.segments {
            let (Some(&ha), Some(&hb)) = (handle_of.get(&s[0]), handle_of.get(&s[1])) else {
                continue;
            };
            if ha != hb
                && cdt.try_add_constraint(ha, hb).len() != 1
                && !cdt.exists_constraint(ha, hb)
            {
                faithful = false;
            }
        }
        if strict && !faithful {
            return None;
        }
        let faces: Vec<[usize; 3]> = cdt
            .inner_faces()
            .map(|f| f.vertices().map(|v| v.fix().index()))
            .collect();
        // Flood from the hull through every edge that is not the outline:
        // what it reaches lies outside the triangle.
        let key = |a: usize, b: usize| (a.min(b), a.max(b));
        let mut by_edge: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
        for (fi, f) in faces.iter().enumerate() {
            for k in 0..3 {
                by_edge
                    .entry(key(f[k], f[(k + 1) % 3]))
                    .or_default()
                    .push(fi);
            }
        }
        let mut outside = vec![false; faces.len()];
        let mut queue: Vec<usize> = Vec::new();
        for (fi, f) in faces.iter().enumerate() {
            for k in 0..3 {
                let e = key(f[k], f[(k + 1) % 3]);
                if by_edge[&e].len() == 1 && !boundary.contains(&e) && !outside[fi] {
                    outside[fi] = true;
                    queue.push(fi);
                }
            }
        }
        while let Some(fi) = queue.pop() {
            let f = faces[fi];
            for k in 0..3 {
                let e = key(f[k], f[(k + 1) % 3]);
                if boundary.contains(&e) {
                    continue;
                }
                for &g in &by_edge[&e] {
                    if !outside[g] {
                        outside[g] = true;
                        queue.push(g);
                    }
                }
            }
        }
        // spade's faces turn counter-clockwise in the layout; the
        // triangle's own winding there decides whether they turn back.
        let mut out = Vec::new();
        for (fi, f) in faces.iter().enumerate() {
            if outside[fi] {
                continue;
            }
            let ids = f.map(|h| id_of[&h]);
            if ids[0] == ids[1] || ids[1] == ids[2] || ids[0] == ids[2] {
                continue;
            }
            out.push(if flip { [ids[0], ids[2], ids[1]] } else { ids });
        }
        if strict {
            // Every outline edge borders one piece, and every segment
            // inside two: the rounded outline was a simple polygon and the
            // segments lie inside it.
            let mut uses: BTreeMap<(u32, u32), u32> = BTreeMap::new();
            for t in &out {
                for k in 0..3 {
                    let (a, b) = (t[k], t[(k + 1) % 3]);
                    *uses.entry((a.min(b), a.max(b))).or_default() += 1;
                }
            }
            let id = |a: u32| {
                handle_of
                    .get(&a)
                    .and_then(|h| id_of.get(&h.index()))
                    .copied()
                    .unwrap_or(a)
            };
            let n = |a: u32, b: u32| {
                let (a, b) = (id(a), id(b));
                uses.get(&(a.min(b), a.max(b))).copied().unwrap_or(0)
            };
            let mut outline: BTreeSet<(u32, u32)> = BTreeSet::new();
            for k in 0..chain.len() {
                let (a, b) = (chain[k], chain[(k + 1) % chain.len()]);
                outline.insert((id(a).min(id(b)), id(a).max(id(b))));
                if id(a) != id(b) && n(a, b) != 1 {
                    return None;
                }
            }
            for s in &cut.segments {
                let (a, b) = (s[0], s[1]);
                if id(a) != id(b)
                    && !outline.contains(&(id(a).min(id(b)), id(a).max(id(b))))
                    && n(a, b) != 2
                {
                    return None;
                }
            }
        }
        Some(out)
    }
}
