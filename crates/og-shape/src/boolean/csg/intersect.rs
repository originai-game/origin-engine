//! Step 1: where the two meshes cross, as points on triangles and segments between them (exact sides).

use super::*;

/// The attributes at a triangle's corner.
#[derive(Clone, Copy)]
pub(crate) struct Corner {
    pub(crate) normal: DVec3,
    pub(crate) uv: [f64; 2],
    pub(crate) color: [f64; 4],
}

/// A mesh welded by exact position: `tris` hold global point ids.
pub(crate) struct Solid {
    pub(crate) tris: Vec<[u32; 3]>,
    pub(crate) corners: Vec<[Corner; 3]>,
    /// The input triangle each one is (degenerate ones are dropped).
    pub(crate) source: Vec<u32>,
    pub(crate) min: DVec3,
    pub(crate) max: DVec3,
}

/// Two triangles, each `(mesh, triangle)`.
pub(crate) type TriPair = ((u8, u32), (u8, u32));

/// A region's boundary: its edges by sorted ids, each with how many times
/// more it runs low to high than back.
pub(crate) type Boundary = Vec<((u32, u32), i32)>;

/// An edge of mesh `0` (A) or `1` (B), by its sorted point ids.
pub(crate) type EdgeKey = (u8, u32, u32);

pub(crate) fn edge_key(mesh: u8, a: u32, b: u32) -> EdgeKey {
    (mesh, a.min(b), a.max(b))
}

/// What names a crossing point.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Origin {
    /// An edge through a triangle's inside (mesh, triangle index).
    EdgeTri(EdgeKey, u8, u32),
    /// Two edges through each other (A's first).
    EdgeEdge(EdgeKey, EdgeKey),
    /// Three triangles of one mesh through one point (a self-union), in
    /// increasing order.
    Triple(u32, u32, u32),
}

/// Where a point lies in a triangle.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Loc {
    Inside,
    Edge(usize),
    Vertex(usize),
}

#[derive(Default)]
pub(crate) struct Cut {
    pub(crate) points: Vec<u32>,
    pub(crate) segments: Vec<[u32; 2]>,
    /// The triangle (of the other mesh, or of this one in a self-union)
    /// whose crossing made each segment.
    pub(crate) src: Vec<u32>,
}

pub(crate) struct Csg {
    pub(crate) points: Vec<DVec3>,
    /// The edges each point lies on (beyond its own mesh's corners).
    pub(crate) on: Vec<Vec<EdgeKey>>,
    pub(crate) made: BTreeMap<Origin, u32>,
    /// What made each point (None for the operands' own vertices); a point
    /// two crossings rounded to keeps the first.
    pub(crate) origin_of: Vec<Option<Origin>>,
    /// Each made point by its exact position.
    pub(crate) made_at: BTreeMap<[u64; 3], u32>,
    pub(crate) solids: [Solid; 2],
    pub(crate) cuts: [BTreeMap<u32, Cut>; 2],
    pub(crate) crossings: BTreeSet<(u32, u32)>,
    /// Pairs of triangles cut in one plane (`coplanar`), each as
    /// `((mesh, tri), (mesh, tri))`: where one surface lies on another.
    pub(crate) coplanar_pairs: Vec<TriPair>,
    /// Pairs of triangles that meet without crossing: one lies on one side
    /// of the other's plane and touches it (a vertex or an edge on it).
    pub(crate) touch_pairs: Vec<TriPair>,
    /// The second pass, for operands that touch (`docs/research.md` §5.4,
    /// item 5): crossings that round to one position stay two points,
    /// points on an edge are ordered exactly, and a triangle whose rounded
    /// outline spade cannot keep is triangulated in an exact layout.
    pub(crate) robust: bool,
    /// Second pass: pairs of points that one triangle's layout could not
    /// tell apart; they are made one point everywhere ([`merge_points`]).
    pub(crate) merges: std::cell::RefCell<Vec<(u32, u32)>>,
    /// Second pass: each cut triangle's outline, in order.
    pub(crate) chains: std::cell::RefCell<Vec<Vec<u32>>>,
    /// Per mesh, the triangles that meet the other surface: cut by it,
    /// touching it, or lying in one plane with one of its triangles.
    pub(crate) met: [BTreeSet<u32>; 2],
    /// Triangles of each mesh left uncut against a triangle of the other
    /// lying within the snap distance of it (`arrange`'s `flat`): regions
    /// do not join them to the rest.
    pub(crate) flat: [BTreeSet<u32>; 2],
}

pub(crate) fn c3(p: DVec3) -> Coord3D<f64> {
    Coord3D {
        x: p.x,
        y: p.y,
        z: p.z,
    }
}

pub(crate) fn sgn(v: f64) -> i32 {
    if v > 0.0 {
        1
    } else if v < 0.0 {
        -1
    } else {
        0
    }
}

/// The axis a triangle's normal leans along most: dropping it projects the
/// triangle without folding it (and exactly: no arithmetic is done).
pub(crate) fn drop_axis(n: DVec3) -> usize {
    let a = n.abs();
    if a.x >= a.y && a.x >= a.z {
        0
    } else if a.y >= a.z {
        1
    } else {
        2
    }
}

pub(crate) fn project(p: DVec3, axis: usize) -> DVec2 {
    match axis {
        0 => DVec2::new(p.y, p.z),
        1 => DVec2::new(p.z, p.x),
        _ => DVec2::new(p.x, p.y),
    }
}

pub(crate) fn o2(a: DVec2, b: DVec2, c: DVec2) -> i32 {
    sgn(orient2d(
        Coord { x: a.x, y: a.y },
        Coord { x: b.x, y: b.y },
        Coord { x: c.x, y: c.y },
    ))
}

impl Csg {
    pub(crate) fn p(&self, i: u32) -> DVec3 {
        self.points[i as usize]
    }

    /// The triangle's normal, each component the triangle's signed area
    /// in one axis plane by `orient2d`: its sign is exact, so a sliver
    /// whose cross product cancels to zero in f64 still has a normal.
    pub(crate) fn tri_normal(&self, t: [u32; 3]) -> DVec3 {
        let [a, b, c] = t.map(|i| self.p(i));
        let area = |axis: usize| {
            let [a, b, c] = [a, b, c].map(|p| project(p, axis));
            orient2d(
                Coord { x: a.x, y: a.y },
                Coord { x: b.x, y: b.y },
                Coord { x: c.x, y: c.y },
            )
        };
        DVec3::new(area(0), area(1), area(2))
    }

    pub(crate) fn add_point(
        &mut self,
        origin: Origin,
        p: impl FnOnce(&Self) -> DVec3,
        on: &[EdgeKey],
    ) -> u32 {
        if let Some(&id) = self.made.get(&origin) {
            return id;
        }
        let pos = p(self);
        // Two crossings a rounding error apart can round to one position.
        // They are made one point, on the edges of both, so the triangles
        // of both meshes cut at it agree on it.
        let key = pos.to_array().map(|x| (x + 0.0).to_bits());
        if !self.robust
            && let Some(&id) = self.made_at.get(&key)
        {
            for &e in on {
                self.note_on(id, e);
            }
            self.made.insert(origin, id);
            return id;
        }
        let id = self.points.len() as u32;
        self.points.push(pos);
        self.on.push(on.to_vec());
        self.origin_of.push(Some(origin));
        self.made.insert(origin, id);
        self.made_at.insert(key, id);
        id
    }

    pub(crate) fn note_on(&mut self, id: u32, e: EdgeKey) {
        let v = &mut self.on[id as usize];
        if !v.contains(&e) {
            v.push(e);
        }
    }

    /// Where `p` (exactly in the triangle's plane) lies in triangle `t`.
    pub(crate) fn locate(&self, t: [u32; 3], p: DVec3) -> Option<Loc> {
        let axis = drop_axis(self.tri_normal(t));
        let q = project(p, axis);
        let v = t.map(|i| project(self.p(i), axis));
        let o = [o2(v[0], v[1], q), o2(v[1], v[2], q), o2(v[2], v[0], q)];
        if o.iter().any(|&s| s > 0) && o.iter().any(|&s| s < 0) {
            return None;
        }
        let zeros: Vec<usize> = (0..3).filter(|&k| o[k] == 0).collect();
        Some(match zeros.as_slice() {
            [] => Loc::Inside,
            [k] => Loc::Edge(*k),
            [a, b] => Loc::Vertex(if (*a + 1) % 3 == *b { *b } else { *a }),
            _ => return None,
        })
    }

    /// The point where edge `(p, q)` of mesh `m` crosses triangle `tri` of
    /// mesh `other`, whose plane it crosses strictly. None when it misses.
    pub(crate) fn edge_cross(&mut self, m: u8, p: u32, q: u32, other: u8, tri: u32) -> Option<u32> {
        let t = self.solids[other as usize].tris[tri as usize];
        let e = edge_key(m, p, q);
        let (p, q) = (e.1, e.2);
        let (pp, pq) = (self.p(p), self.p(q));
        let s: [i32; 3] = std::array::from_fn(|k| {
            sgn(orient3d(
                c3(pp),
                c3(pq),
                c3(self.p(t[k])),
                c3(self.p(t[(k + 1) % 3])),
            ))
        });
        if s.iter().any(|&x| x > 0) && s.iter().any(|&x| x < 0) {
            return None;
        }
        let zeros: Vec<usize> = (0..3).filter(|&k| s[k] == 0).collect();
        match zeros.as_slice() {
            [] => {
                let id = self.add_point(
                    Origin::EdgeTri(e, other, tri),
                    |c| {
                        let (a, b, cc) = (c.p(t[0]), c.p(t[1]), c.p(t[2]));
                        let n = (b - a).cross(cc - a);
                        let (dp, dq) = (n.dot(pp - a), n.dot(pq - a));
                        let mut f = dp / (dp - dq);
                        // An edge almost in the plane: the rounded distances
                        // can cancel (0/0) or put the point off the edge.
                        // `orient3d` gives them with their exact signs,
                        // opposite (the edge crosses the plane strictly), so
                        // the fraction lies strictly inside the edge
                        // (`docs/research.md` §18).
                        if !(0.0..=1.0).contains(&f) {
                            let (op, oq) = (
                                orient3d(c3(a), c3(b), c3(cc), c3(pp)),
                                orient3d(c3(a), c3(b), c3(cc), c3(pq)),
                            );
                            f = op / (op - oq);
                        }
                        pp + (pq - pp) * f
                    },
                    &[e],
                );
                Some(id)
            }
            [k] => {
                let f = edge_key(other, t[*k], t[(*k + 1) % 3]);
                Some(self.edge_edge(e, f))
            }
            [a, b] => {
                // Through the vertex the two edges share.
                let v = t[if (*a + 1) % 3 == *b { *b } else { *a }];
                self.note_on(v, e);
                Some(v)
            }
            _ => None,
        }
    }

    /// The point where two edges cross (between meshes: A's first).
    pub(crate) fn edge_edge(&mut self, e: EdgeKey, f: EdgeKey) -> u32 {
        let (e, f) = if e <= f { (e, f) } else { (f, e) };
        self.add_point(
            Origin::EdgeEdge(e, f),
            |c| {
                let (p1, q1, p2, q2) = (c.p(e.1), c.p(e.2), c.p(f.1), c.p(f.2));
                let (d1, d2, r) = (q1 - p1, q2 - p2, p1 - p2);
                let (a, ee, ff) = (d1.dot(d1), d2.dot(d2), d2.dot(r));
                let (cc, b) = (d1.dot(r), d1.dot(d2));
                let den = a * ee - b * b;
                let s = if den != 0.0 {
                    ((b * ff - cc * ee) / den).clamp(0.0, 1.0)
                } else {
                    0.5
                };
                p1 + d1 * s
            },
            &[e, f],
        )
    }

    pub(crate) fn cut(&mut self, m: u8, tri: u32) -> &mut Cut {
        self.cuts[m as usize].entry(tri).or_default()
    }

    pub(crate) fn add_points(&mut self, m: u8, tri: u32, ids: &[u32]) {
        let c = self.cut(m, tri);
        for &i in ids {
            if !c.points.contains(&i) {
                c.points.push(i);
            }
        }
    }

    pub(crate) fn add_segment(&mut self, m: u8, tri: u32, a: u32, b: u32, src: u32) {
        if a == b {
            return;
        }
        self.add_points(m, tri, &[a, b]);
        let c = self.cut(m, tri);
        c.segments.push([a, b]);
        c.src.push(src);
        self.crossings.insert((a.min(b), a.max(b)));
    }

    /// Intersects triangle `ia` of mesh `ma` with triangle `ib` of mesh
    /// `mb` (A with B, or two of one mesh's for a self-union).
    pub(crate) fn pair(&mut self, (ma, ia): (u8, u32), (mb, ib): (u8, u32)) {
        let ta = self.solids[ma as usize].tris[ia as usize];
        let tb = self.solids[mb as usize].tris[ib as usize];
        let side = |c: &Self, plane: [u32; 3], v: u32| {
            sgn(orient3d(
                c3(c.p(plane[0])),
                c3(c.p(plane[1])),
                c3(c.p(plane[2])),
                c3(c.p(v)),
            ))
        };
        let oa = ta.map(|v| side(self, tb, v));
        let ob = tb.map(|v| side(self, ta, v));
        let one_side = |o: [i32; 3]| o.iter().all(|&s| s > 0) || o.iter().all(|&s| s < 0);
        if one_side(oa) || one_side(ob) {
            return;
        }
        if oa == [0; 3] {
            self.met[ma as usize].insert(ia);
            self.met[mb as usize].insert(ib);
            self.coplanar((ma, ia), (mb, ib));
            return;
        }
        let touch = |o: [i32; 3]| o.iter().all(|&s| s >= 0) || o.iter().all(|&s| s <= 0);
        if touch(oa) || touch(ob) {
            self.touch_pairs.push(((ma, ia), (mb, ib)));
            self.met[ma as usize].insert(ia);
            self.met[mb as usize].insert(ib);
        }
        let mut found: Vec<u32> = Vec::new();
        // Vertices of one triangle in the other's plane and inside it.
        for (t, o, ot, om) in [(ta, oa, tb, mb), (tb, ob, ta, ma)] {
            for k in 0..3 {
                if o[k] != 0 {
                    continue;
                }
                let v = t[k];
                match self.locate(ot, self.p(v)) {
                    None => {}
                    Some(Loc::Edge(j)) => {
                        self.note_on(v, edge_key(om, ot[j], ot[(j + 1) % 3]));
                        found.push(v);
                    }
                    // At the other's vertex (two meshes' vertices at one
                    // position keep their own ids) or inside it.
                    Some(Loc::Vertex(_) | Loc::Inside) => found.push(v),
                }
            }
        }
        // Edges of one triangle through the other.
        for (m, t, o, om, other) in [(ma, ta, oa, mb, ib), (mb, tb, ob, ma, ia)] {
            for k in 0..3 {
                let (i, j) = (k, (k + 1) % 3);
                if o[i] * o[j] < 0
                    && let Some(id) = self.edge_cross(m, t[i], t[j], om, other)
                {
                    found.push(id);
                }
            }
        }
        let mut uniq: Vec<u32> = Vec::new();
        for id in found {
            if !uniq.contains(&id) {
                uniq.push(id);
            }
        }
        let dir = self.tri_normal(ta).cross(self.tri_normal(tb));
        crate::math::sort::sort_by(&mut uniq, |&a, &b| {
            self.p(a)
                .dot(dir)
                .total_cmp(&self.p(b).dot(dir))
                .then(a.cmp(&b))
        });
        if !uniq.is_empty() {
            self.met[ma as usize].insert(ia);
            self.met[mb as usize].insert(ib);
        }
        self.add_points(ma, ia, &uniq);
        self.add_points(mb, ib, &uniq);
        for w in uniq.windows(2) {
            self.add_segment(ma, ia, w[0], w[1], ib);
            self.add_segment(mb, ib, w[0], w[1], ia);
        }
    }

    /// Two triangles in one plane: each one's edges inside the other are
    /// cut into it, and the points where their edges cross.
    pub(crate) fn coplanar(&mut self, (ma, ia): (u8, u32), (mb, ib): (u8, u32)) {
        self.coplanar_pairs.push(((ma, ia), (mb, ib)));
        let ta = self.solids[ma as usize].tris[ia as usize];
        let tb = self.solids[mb as usize].tris[ib as usize];
        let axis = drop_axis(self.tri_normal(ta));
        let pr = |c: &Self, i: u32| project(c.p(i), axis);
        // Edge crossings, for each triangle (slot 0: `ta`, 1: `tb`).
        let mut on_edge: [Vec<(u32, EdgeKey)>; 2] = [Vec::new(), Vec::new()];
        for i in 0..3 {
            let (a0, a1) = (ta[i], ta[(i + 1) % 3]);
            for j in 0..3 {
                let (b0, b1) = (tb[j], tb[(j + 1) % 3]);
                let s1 = o2(pr(self, a0), pr(self, a1), pr(self, b0));
                let s2 = o2(pr(self, a0), pr(self, a1), pr(self, b1));
                let s3 = o2(pr(self, b0), pr(self, b1), pr(self, a0));
                let s4 = o2(pr(self, b0), pr(self, b1), pr(self, a1));
                if s1 * s2 < 0 && s3 * s4 < 0 {
                    let (e, f) = (edge_key(ma, a0, a1), edge_key(mb, b0, b1));
                    let id = self.edge_edge(e, f);
                    self.add_points(ma, ia, &[id]);
                    self.add_points(mb, ib, &[id]);
                    on_edge[0].push((id, e));
                    on_edge[1].push((id, f));
                }
            }
        }
        // Vertices of each inside the other.
        for (slot, m, t, ot, om, oi) in [(0, ma, ta, tb, mb, ib), (1, mb, tb, ta, ma, ia)] {
            for &v in &t {
                if let Some(loc) = self.locate(ot, self.p(v)) {
                    if let Loc::Edge(j) = loc {
                        self.note_on(v, edge_key(om, ot[j], ot[(j + 1) % 3]));
                    }
                    self.add_points(om, oi, &[v]);
                    // The vertex lies on its own triangle's two edges.
                    for k in 0..3 {
                        if t[k] == v {
                            on_edge[slot].push((v, edge_key(m, t[k], t[(k + 1) % 3])));
                            on_edge[slot].push((v, edge_key(m, t[(k + 2) % 3], t[k])));
                        }
                    }
                }
            }
        }
        // Each edge's pieces inside the other triangle become its cuts.
        for (slot, m, t, ot, om, oi, ti) in
            [(0, ma, ta, tb, mb, ib, ia), (1, mb, tb, ta, ma, ia, ib)]
        {
            for k in 0..3 {
                let e = edge_key(m, t[k], t[(k + 1) % 3]);
                let (p0, p1) = (self.p(e.1), self.p(e.2));
                let mut along: Vec<u32> = on_edge[slot]
                    .iter()
                    .filter(|x| x.1 == e)
                    .map(|x| x.0)
                    .collect();
                // Vertices of the other triangle lying on this edge.
                for &v in &ot {
                    if self.on[v as usize].contains(&e) {
                        along.push(v);
                    }
                }
                crate::math::sort::sort_by(&mut along, |&a, &b| {
                    (self.p(a) - p0)
                        .dot(p1 - p0)
                        .total_cmp(&(self.p(b) - p0).dot(p1 - p0))
                        .then(a.cmp(&b))
                });
                along.dedup();
                for w in along.windows(2) {
                    let mid = (self.p(w[0]) + self.p(w[1])) * 0.5;
                    let q = project(mid, axis);
                    let v = ot.map(|i| project(self.p(i), axis));
                    let o = [o2(v[0], v[1], q), o2(v[1], v[2], q), o2(v[2], v[0], q)];
                    if !(o.iter().any(|&s| s > 0) && o.iter().any(|&s| s < 0)) {
                        self.add_segment(om, oi, w[0], w[1], ti);
                        self.crossings.insert((w[0].min(w[1]), w[0].max(w[1])));
                    }
                }
            }
        }
    }

    /// The point where the planes of three triangles of mesh 0 meet
    /// (Cramer's rule, in a fixed order); None if two are parallel.
    pub(crate) fn three_planes(&self, t: [u32; 3]) -> Option<DVec3> {
        let plane = |i: u32| {
            let [a, b, c] = self.solids[0].tris[i as usize].map(|v| self.p(v));
            let n = (b - a).cross(c - a);
            (n, n.dot(a))
        };
        let [(n1, d1), (n2, d2), (n3, d3)] = t.map(plane);
        let det = n1.dot(n2.cross(n3));
        if det == 0.0 || !det.is_finite() {
            return None;
        }
        let p = (n2.cross(n3) * d1 + n3.cross(n1) * d2 + n1.cross(n2) * d3) / det;
        p.is_finite().then_some(p)
    }

    /// A self-union's triple points. Where two crossing segments inside one
    /// triangle, made by two other triangles, cross each other, the three
    /// triangles meet in a point. It is made once, named by the three, and
    /// each of them splits its two segments there, so all three are cut
    /// alike whichever of them saw the crossing (two meshes never need
    /// this: neither crosses itself).
    pub(crate) fn triples(&mut self) {
        let mut found: BTreeSet<[u32; 3]> = BTreeSet::new();
        for (&t, cut) in &self.cuts[0] {
            let axis = drop_axis(self.tri_normal(self.solids[0].tris[t as usize]));
            let pr = |k: u32| project(self.p(k), axis);
            for i in 0..cut.segments.len() {
                for j in i + 1..cut.segments.len() {
                    let (u, v) = (cut.src[i], cut.src[j]);
                    let ([a, b], [p, q]) = (cut.segments[i], cut.segments[j]);
                    if u == v || a == p || a == q || b == p || b == q {
                        continue;
                    }
                    let (s1, s2) = (o2(pr(a), pr(b), pr(p)), o2(pr(a), pr(b), pr(q)));
                    let (s3, s4) = (o2(pr(p), pr(q), pr(a)), o2(pr(p), pr(q), pr(b)));
                    if s1 * s2 < 0 && s3 * s4 < 0 {
                        let mut k = [t, u, v];
                        k.sort_unstable();
                        found.insert(k);
                    }
                }
            }
        }
        for k in found {
            let Some(pos) = self.three_planes(k) else {
                continue;
            };
            let id = self.add_point(Origin::Triple(k[0], k[1], k[2]), |_| pos, &[]);
            for (x, y, z) in [(k[0], k[1], k[2]), (k[1], k[0], k[2]), (k[2], k[0], k[1])] {
                self.split(x, y, id);
                self.split(x, z, id);
            }
        }
    }

    /// Splits at point `id` the segment of triangle `tri` (mesh 0) made by
    /// triangle `src`: of several, the one `id` lies nearest.
    pub(crate) fn split(&mut self, tri: u32, src: u32, id: u32) {
        let Some(cut) = self.cuts[0].get(&tri) else {
            return;
        };
        let p = self.p(id);
        let detour = |s: [u32; 2]| {
            let (a, b) = (self.p(s[0]), self.p(s[1]));
            (a - p).length() + (p - b).length() - (a - b).length()
        };
        let mut best: Option<(usize, f64)> = None;
        for (i, (&s, &from)) in cut.segments.iter().zip(&cut.src).enumerate() {
            if from != src || s.contains(&id) {
                continue;
            }
            let d = detour(s);
            if best.is_none_or(|(_, bd)| d < bd) {
                best = Some((i, d));
            }
        }
        let Some((i, _)) = best else {
            return;
        };
        let cut = self.cuts[0].get_mut(&tri).expect("cut");
        let [a, b] = cut.segments[i];
        cut.segments[i] = [a, id];
        cut.segments.push([id, b]);
        cut.src.push(src);
        if !cut.points.contains(&id) {
            cut.points.push(id);
        }
        self.crossings.insert((a.min(id), a.max(id)));
        self.crossings.insert((id.min(b), id.max(b)));
    }
}
