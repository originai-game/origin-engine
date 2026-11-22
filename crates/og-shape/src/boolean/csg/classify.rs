//! Step 3's decision: which regions are inside the other mesh, by winding numbers and exact probes, and which lie on it.

use super::*;

/// `q`, or if it is not strictly inside `outline`, the smallest step from
/// it towards `centre`, by powers of two, that is.
pub(crate) fn inward(outline: &[DVec2], centre: DVec2, q: DVec2) -> DVec2 {
    if strictly_inside(outline, q) {
        return q;
    }
    let mut step = f64::EPSILON;
    while step < 0.5 {
        let r = q + (centre - q) * step;
        if strictly_inside(outline, r) {
            return r;
        }
        step *= 2.0;
    }
    centre
}

/// Whether `q` lies inside the closed polygon and on none of its edges,
/// decided by exact predicates (a winding number, Sunday's crossing rule).
pub(crate) fn strictly_inside(poly: &[DVec2], q: DVec2) -> bool {
    let mut wn = 0i32;
    for k in 0..poly.len() {
        let (a, b) = (poly[k], poly[(k + 1) % poly.len()]);
        let o = o2(a, b, q);
        if o == 0 && q.cmpge(a.min(b)).all() && q.cmple(a.max(b)).all() {
            return false;
        }
        if a.y <= q.y {
            if b.y > q.y && o > 0 {
                wn += 1;
            }
        } else if b.y <= q.y && o < 0 {
            wn -= 1;
        }
    }
    wn != 0
}

/// The winding number of a closed mesh round `p` (solid angles, Van
/// Oosterom and Strackee): 1 inside, 0 outside.
pub(crate) fn winding(points: &[DVec3], tris: &[[u32; 3]], p: DVec3) -> f64 {
    crate::boolean::winding::winding(points, tris, None, p)
}

/// Which side of thresholds a mesh's winding number lies, for many
/// points: the direct sum, or for a large mesh and many points, a tree
/// with the direct sum's decisions (`docs/research.md` §18).
pub(crate) enum Winder<'a> {
    Direct(&'a [DVec3], &'a [[u32; 3]]),
    Tree(crate::boolean::winding::Tree<'a>),
}

impl<'a> Winder<'a> {
    pub(crate) fn new(points: &'a [DVec3], tris: &'a [[u32; 3]], queries: usize) -> Winder<'a> {
        if tris.len() >= crate::boolean::winding::MIN_TRIS && queries >= 16 {
            Winder::Tree(crate::boolean::winding::Tree::new(points, tris))
        } else {
            Winder::Direct(points, tris)
        }
    }

    /// The winding number round `p`, triangle `skip` left out, on the same
    /// side of each of `thresholds` as the direct sum.
    pub(crate) fn side(&self, p: DVec3, skip: Option<usize>, thresholds: &[f64]) -> f64 {
        match self {
            Winder::Direct(points, tris) => crate::boolean::winding::winding(points, tris, skip, p),
            Winder::Tree(t) => t.side(p, skip, thresholds),
        }
    }
}

impl Csg {
    /// Step 3's regions: pieces of one mesh joined across edges no
    /// crossing runs along.
    pub(crate) fn regions(&self, pieces: &[Piece]) -> Regions {
        let mut parent: Vec<usize> = (0..pieces.len()).collect();
        fn find(parent: &mut [usize], mut i: usize) -> usize {
            while parent[i] != i {
                parent[i] = parent[parent[i]];
                i = parent[i];
            }
            i
        }
        let mut first: BTreeMap<(u8, u32, u32), usize> = BTreeMap::new();
        for (pi, p) in pieces.iter().enumerate() {
            for k in 0..3 {
                let (x, y) = (p.ids[k], p.ids[(k + 1) % 3]);
                let e = (x.min(y), x.max(y));
                if self.crossings.contains(&e) {
                    continue;
                }
                match first.get(&(p.mesh, e.0, e.1)) {
                    Some(&q)
                        if self.flat[p.mesh as usize].contains(&p.tri)
                            != self.flat[p.mesh as usize].contains(&pieces[q].tri) => {}
                    Some(&q) => {
                        let (r1, r2) = (find(&mut parent, pi), find(&mut parent, q));
                        if r1 != r2 {
                            parent[r1.max(r2)] = r1.min(r2);
                        }
                    }
                    None => {
                        first.insert((p.mesh, e.0, e.1), pi);
                    }
                }
            }
        }
        let area = |p: &Piece| {
            let [a, b, cc] = p.ids.map(|i| self.points[i as usize]);
            (b - a).cross(cc - a).length() * 0.5
        };
        // Each region's largest piece (the first of equals).
        let mut best: BTreeMap<usize, usize> = BTreeMap::new();
        let mut root = Vec::with_capacity(pieces.len());
        for pi in 0..pieces.len() {
            let r = find(&mut parent, pi);
            root.push(r);
            match best.get(&r) {
                Some(&q) if area(&pieces[q]) >= area(&pieces[pi]) => {}
                _ => {
                    best.insert(r, pi);
                }
            }
        }
        Regions { root, best }
    }

    /// Points just in front of and just behind a piece, at its centre: a
    /// millionth of `scale`, or half its inradius if that is less.
    pub(crate) fn probes(&self, p: &Piece, scale: f64) -> (DVec3, DVec3) {
        let [a, b, cc] = p.ids.map(|i| self.points[i as usize]);
        let n = (b - a).cross(cc - a);
        let perimeter = (b - a).length() + (cc - b).length() + (a - cc).length();
        let inradius = n.length() / perimeter.max(1e-300);
        let eps = (scale * 1e-6).min(inradius * 0.5);
        let n = normalize(n);
        let centre = (a + b + cc) / 3.0;
        (centre + n * eps, centre - n * eps)
    }

    /// Whether `p`, a point on no triangle of mesh `m`, lies inside it,
    /// exactly: the signed crossings of a segment from `p` to a point
    /// outside both meshes' box, each decided by `orient3d`. A segment
    /// through an edge, a vertex or a plane of the mesh at its far end is
    /// tried along another direction; None if every one is.
    pub(crate) fn inside_exact(&self, m: u8, p: DVec3) -> Option<bool> {
        let s = &self.solids[m as usize];
        let reach = 2.0 * self.scale();
        let o = |a: DVec3, b: DVec3, c: DVec3, d: DVec3| sgn(orient3d(c3(a), c3(b), c3(c), c3(d)));
        // Directions, fixed, none along an axis plane (need not be unit).
        const DIRS: [[f64; 3]; 6] = [
            [0.5773502691896258, 0.5345224838248488, 0.6172133998483676],
            [-0.6123724356957945, 0.4082482904638631, 0.6770032003863302],
            [0.3713906763541037, -0.7427813527082074, 0.5570860145311556],
            [
                -0.4472135954999579,
                -0.5163977794943222,
                -0.7302967433402214,
            ],
            [0.7, -0.3015113445777636, -0.6396021490668313],
            [-0.2672612419124244, 0.8017837257372732, -0.5345224838248488],
        ];
        'dir: for d in DIRS {
            let q = p + DVec3::from(d) * reach;
            let (lo, hi) = (p.min(q), p.max(q));
            let mut w = 0i32;
            for t in &s.tris {
                let [a, b, c] = t.map(|v| self.points[v as usize]);
                if a.max(b).max(c).cmplt(lo).any() || a.min(b).min(c).cmpgt(hi).any() {
                    continue;
                }
                let (sp, sq) = (o(a, b, c, p), o(a, b, c, q));
                if sq == 0 {
                    continue 'dir;
                }
                // `p` in the plane, but on no triangle: the segment leaves
                // the plane at once.
                if sp == 0 || sp == sq {
                    continue;
                }
                let e = [o(p, q, a, b), o(p, q, b, c), o(p, q, c, a)];
                if e.contains(&0) {
                    if e.iter().all(|&x| x >= 0) || e.iter().all(|&x| x <= 0) {
                        continue 'dir;
                    }
                    continue;
                }
                if e[0] == e[1] && e[1] == e[2] {
                    w += if sp > 0 { 1 } else { -1 };
                }
            }
            return Some(w != 0);
        }
        None
    }

    /// Whether the target's triangle `ia` and the tool's `ib` lie within
    /// `tol` of each other's planes, every corner, without being exactly
    /// in one plane: one surface placed by two rounded motions
    /// (`docs/research.md` §27). A candidate only: a result built with
    /// such pairs left uncut is kept or dropped by exact checks
    /// ([`csg_points_repaired`]).
    pub(crate) fn near_coplanar(&self, ia: u32, ib: u32, tol: f64) -> bool {
        let ta = self.solids[0].tris[ia as usize].map(|v| self.p(v));
        let tb = self.solids[1].tris[ib as usize].map(|v| self.p(v));
        let o = |t: [DVec3; 3], q: DVec3| sgn(orient3d(c3(t[0]), c3(t[1]), c3(t[2]), c3(q)));
        if ta.iter().all(|&q| o(tb, q) == 0) {
            return false;
        }
        let within = |t: [DVec3; 3], u: [DVec3; 3]| {
            let n = (t[1] - t[0]).cross(t[2] - t[0]);
            let l = n.length();
            l > 0.0 && u.iter().all(|&q| (n.dot(q - t[0])).abs() <= tol * l)
        };
        within(ta, tb) && within(tb, ta)
    }

    /// The diagonal of both meshes' box.
    pub(crate) fn scale(&self) -> f64 {
        let lo = self.solids[0].min.min(self.solids[1].min);
        let hi = self.solids[0].max.max(self.solids[1].max);
        (hi - lo).length().max(1e-30)
    }
}

/// The second pass's regions that lie on the other mesh's surface,
/// decided symbolically: a region of one mesh whose boundary is the
/// boundary of a region of the other, edge for edge (by point ids, after
/// [`merge_points`]), every triangle of each lying in one plane with a
/// triangle of the other (`coplanar`, exact). Each maps to whether the two
/// face the same way. A sliver a rounding error wide, which no probe can
/// classify, is decided so (`docs/research.md` §5.4, item 5).
pub(crate) fn coincident_regions(
    c: &Csg,
    pieces: &[Piece],
    regions: &Regions,
) -> BTreeMap<usize, bool> {
    let mut partners: BTreeMap<(u8, u32), BTreeSet<(u8, u32)>> = BTreeMap::new();
    for &(a, b) in &c.coplanar_pairs {
        partners.entry(a).or_default().insert(b);
        partners.entry(b).or_default().insert(a);
    }
    // Each region's triangles, but for pieces whose corners are exactly
    // collinear (they bound nothing).
    let mut tris: BTreeMap<usize, BTreeSet<(u8, u32)>> = BTreeMap::new();
    for (pi, p) in pieces.iter().enumerate() {
        let e = tris.entry(regions.root[pi]).or_default();
        if c.tri_normal(p.ids) != DVec3::ZERO {
            e.insert((p.mesh, p.tri));
        }
    }
    let on = |r: usize, o: usize| {
        tris[&r].iter().all(|t| {
            partners
                .get(t)
                .is_some_and(|ps| ps.iter().any(|q| tris[&o].contains(q)))
        })
    };
    // Each region's boundary: directed edges not cancelled within it.
    let mut edges: BTreeMap<usize, BTreeMap<(u32, u32), i32>> = BTreeMap::new();
    for (pi, p) in pieces.iter().enumerate() {
        let e = edges.entry(regions.root[pi]).or_default();
        for k in 0..3 {
            let (a, b) = (p.ids[k], p.ids[(k + 1) % 3]);
            *e.entry((a.min(b), a.max(b))).or_default() += if a < b { 1 } else { -1 };
        }
    }
    let mut by_boundary: BTreeMap<Boundary, Vec<usize>> = BTreeMap::new();
    let mut mesh_of: BTreeMap<usize, u8> = BTreeMap::new();
    for (&r, e) in &edges {
        let b: Vec<((u32, u32), i32)> = e
            .iter()
            .filter(|(_, n)| **n != 0)
            .map(|(&k, &n)| (k, n))
            .collect();
        if b.is_empty() {
            continue;
        }
        mesh_of.insert(r, pieces[regions.best[&r]].mesh);
        by_boundary.entry(b).or_default().push(r);
    }
    let mut twins: BTreeMap<usize, bool> = BTreeMap::new();
    for (b, rs) in &by_boundary {
        let flipped: Vec<((u32, u32), i32)> = b.iter().map(|&(k, n)| (k, -n)).collect();
        for &r in rs {
            let m = mesh_of[&r];
            let twin = |o: usize| mesh_of[&o] != m && on(r, o) && on(o, r);
            if rs.iter().any(|&o| twin(o)) {
                twins.insert(r, true);
            } else if by_boundary
                .get(&flipped)
                .is_some_and(|os| os.iter().any(|&o| twin(o)))
            {
                twins.insert(r, false);
            }
        }
    }
    twins
}
