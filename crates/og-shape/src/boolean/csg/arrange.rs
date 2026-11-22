//! Steps 1 to 3 together: the arrangement of two meshes (or one with itself), its pieces merged, and the broad phase that pairs triangles.

use super::*;

/// A piece the operation keeps: of mesh 0 (A) or 1 (B), from its welded
/// triangle `tri`, with point ids, turned inside out where `flip`.
pub(crate) struct Kept {
    pub(crate) mesh: u8,
    pub(crate) tri: u32,
    pub(crate) ids: [u32; 3],
    pub(crate) flip: bool,
}

/// A piece of a cut triangle: of mesh `mesh`, from its welded triangle
/// `tri`, with point ids in the triangle's winding.
pub(crate) struct Piece {
    pub(crate) mesh: u8,
    pub(crate) tri: u32,
    pub(crate) ids: [u32; 3],
}

/// The pieces' regions: each piece's root, and each root's largest piece.
pub(crate) struct Regions {
    pub(crate) root: Vec<usize>,
    pub(crate) best: BTreeMap<usize, usize>,
}

impl Csg {
    pub(crate) fn new(points: Vec<DVec3>, solids: [Solid; 2]) -> Csg {
        let on = vec![Vec::new(); points.len()];
        let origin_of = vec![None; points.len()];
        Csg {
            points,
            on,
            made: BTreeMap::new(),
            origin_of,
            made_at: BTreeMap::new(),
            solids,
            cuts: [BTreeMap::new(), BTreeMap::new()],
            crossings: BTreeSet::new(),
            coplanar_pairs: Vec::new(),
            touch_pairs: Vec::new(),
            robust: false,
            merges: std::cell::RefCell::new(Vec::new()),
            chains: std::cell::RefCell::new(Vec::new()),
            met: [BTreeSet::new(), BTreeSet::new()],
            flat: [BTreeSet::new(), BTreeSet::new()],
        }
    }

    /// Step 2: every triangle of meshes `0..meshes`, cut into pieces.
    pub(crate) fn all_pieces(&self, meshes: u8) -> Vec<Piece> {
        let mut pieces = Vec::new();
        for m in 0..meshes {
            for tri in 0..self.solids[m as usize].tris.len() as u32 {
                for ids in self.pieces(m, tri) {
                    pieces.push(Piece { mesh: m, tri, ids });
                }
            }
        }
        pieces
    }
}

/// Steps 1 to 3 and the choice of step 4: which pieces `op` keeps.
pub(crate) fn arrange(
    points: Vec<DVec3>,
    sa: Solid,
    sb: Solid,
    op: CsgOp,
    robust: bool,
    flat: f64,
) -> (Csg, Vec<Kept>) {
    let mut c = Csg::new(points, [sa, sb]);
    c.robust = robust;
    // 1. Every pair of triangles whose boxes overlap.
    let box_a = (c.solids[0].min, c.solids[0].max);
    let box_b = (c.solids[1].min, c.solids[1].max);
    let near = |c: &Csg, m: usize, other: (DVec3, DVec3)| -> Vec<(u32, (DVec3, DVec3))> {
        c.solids[m]
            .tris
            .iter()
            .enumerate()
            .map(|(i, &t)| (i as u32, bounds(&c.points, t)))
            .filter(|(_, bb)| overlap(*bb, other))
            .collect()
    };
    let near_a = near(&c, 0, box_b);
    let near_b = near(&c, 1, box_a);
    for (ia, ib) in candidate_pairs(&near_a, &near_b) {
        if flat > 0.0 && c.near_coplanar(ia, ib, flat) {
            c.flat[0].insert(ia);
            c.flat[1].insert(ib);
            continue;
        }
        c.pair((0, ia), (1, ib));
    }
    // 2. The pieces.
    let mut pieces = c.all_pieces(2);
    if robust {
        pieces = merge_points(&mut c, pieces);
    }
    // 3. Regions, each classified by the other mesh's winding number.
    let regions = c.regions(&pieces);
    let scale = c.scale();
    // (inside the other mesh in front, behind).
    let mut sides: BTreeMap<usize, (bool, bool)> = BTreeMap::new();
    let twins = if robust {
        coincident_regions(&c, &pieces, &regions)
    } else {
        BTreeMap::new()
    };
    let queries = |m: u8| {
        regions
            .best
            .values()
            .filter(|&&b| pieces[b].mesh != m)
            .count()
    };
    let winders = [0u8, 1].map(|m| Winder::new(&c.points, &c.solids[m as usize].tris, queries(m)));
    // Regions none of whose triangles meets the other surface: wholly
    // inside or outside the other solid, decided exactly at a corner.
    let mut apart: BTreeMap<usize, bool> = BTreeMap::new();
    for (pi, p) in pieces.iter().enumerate() {
        let met = c.met[p.mesh as usize].contains(&p.tri);
        let e = apart.entry(regions.root[pi]).or_insert(true);
        *e = *e && !met;
    }
    for (&r, &b) in &regions.best {
        if let Some(&same) = twins.get(&r) {
            // On the other surface: the other solid lies behind a face
            // facing the same way, in front of one facing the other way.
            sides.insert(r, (!same, same));
            continue;
        }
        let p = &pieces[b];
        if apart[&r]
            && let Some(inside) = c.inside_exact(1 - p.mesh, c.points[p.ids[0] as usize])
        {
            sides.insert(r, (inside, inside));
            continue;
        }
        let (front, back) = c.probes(p, scale);
        let other = &winders[1 - p.mesh as usize];
        let inside = |q: DVec3| other.side(q, None, &[0.5]) > 0.5;
        sides.insert(r, (inside(front), inside(back)));
    }
    let mut choice: Vec<(bool, bool)> = Vec::with_capacity(pieces.len());
    for (pi, p) in pieces.iter().enumerate() {
        let (front, back) = sides[&regions.root[pi]];
        let (keep, flip) = if p.mesh == 0 {
            match op {
                CsgOp::Union => (!front, false),
                CsgOp::Intersect => (back, false),
                CsgOp::Subtract => (!back, false),
            }
        } else if front != back {
            // On A's surface: A's piece answers for it.
            (false, false)
        } else {
            match op {
                CsgOp::Union => (!front, false),
                CsgOp::Intersect => (back, false),
                CsgOp::Subtract => (front, true),
            }
        };
        choice.push((keep, flip));
    }
    if robust {
        close_slivers(&c, &pieces, &regions, op, scale, &mut choice);
    }
    let mut kept = Vec::new();
    for (p, &(keep, flip)) in pieces.iter().zip(&choice) {
        if keep {
            kept.push(Kept {
                mesh: p.mesh,
                tri: p.tri,
                ids: p.ids,
                flip,
            });
        }
    }
    (c, kept)
}

/// Makes points that must be one point one: each pair a triangle's layout
/// merged, every set of points at one position (the result is welded by
/// position), and along each cut triangle's outline, the run between two
/// places where one position comes back (a crossing rounded onto a point
/// past others that lie, in exact order, between them: a run a few ulps
/// long). Repeated until nothing changes. The lowest id stands for each
/// set, in the pieces and in the crossings; pieces left with two equal
/// corners are dropped. Each step collapses edges of the arrangement, so
/// a closed surface stays closed (`docs/research.md` §5.4, item 5).
pub(crate) fn merge_points(c: &mut Csg, pieces: Vec<Piece>) -> Vec<Piece> {
    let merges = std::mem::take(&mut *c.merges.borrow_mut());
    let chains = std::mem::take(&mut *c.chains.borrow_mut());
    let mut parent: BTreeMap<u32, u32> = BTreeMap::new();
    fn find(parent: &BTreeMap<u32, u32>, mut i: u32) -> u32 {
        while let Some(&p) = parent.get(&i) {
            if p == i {
                break;
            }
            i = p;
        }
        i
    }
    fn union(parent: &mut BTreeMap<u32, u32>, a: u32, b: u32) -> bool {
        let (ra, rb) = (find(parent, a), find(parent, b));
        if ra == rb {
            return false;
        }
        parent.insert(ra.max(rb), ra.min(rb));
        true
    }
    for (a, b) in merges {
        union(&mut parent, a, b);
    }
    // Every point a piece uses, by position.
    let mut used: BTreeSet<u32> = BTreeSet::new();
    for p in &pieces {
        used.extend(p.ids);
    }
    let key = |c: &Csg, i: u32| c.points[i as usize].to_array().map(|x| (x + 0.0).to_bits());
    let tiny = c.scale() * (1.0 / (1u64 << 30) as f64);
    loop {
        let mut changed = false;
        let mut at: BTreeMap<[u64; 3], u32> = BTreeMap::new();
        for &i in &used {
            let r = find(&parent, i);
            let k = key(c, r);
            match at.get(&k) {
                Some(&j) => changed |= union(&mut parent, j, r),
                None => {
                    at.insert(k, r);
                }
            }
        }
        for chain in &chains {
            let mut seq: Vec<u32> = Vec::new();
            for &i in chain {
                let r = find(&parent, i);
                if seq.last() != Some(&r) {
                    seq.push(r);
                }
            }
            while seq.len() > 1 && seq.first() == seq.last() {
                seq.pop();
            }
            let n = seq.len();
            let mut first: BTreeMap<u32, usize> = BTreeMap::new();
            for (k, &r) in seq.iter().enumerate() {
                if let Some(&j) = first.get(&r) {
                    // The shorter way round from j to k collapses.
                    let run: Vec<u32> = if k - j <= n - (k - j) {
                        seq[j..=k].to_vec()
                    } else {
                        seq[k..].iter().chain(&seq[..=j]).copied().collect()
                    };
                    // Only a run a rounding error long: a longer one is
                    // left, and the result's check refuses what follows.
                    let (mut lo, mut hi) = (DVec3::INFINITY, DVec3::NEG_INFINITY);
                    for &i in &run {
                        lo = lo.min(c.points[i as usize]);
                        hi = hi.max(c.points[i as usize]);
                    }
                    if (hi - lo).length() > tiny {
                        continue;
                    }
                    for w in run.windows(2) {
                        changed |= union(&mut parent, w[0], w[1]);
                    }
                } else {
                    first.insert(r, k);
                }
            }
        }
        if !changed {
            break;
        }
    }
    if parent.is_empty() {
        return pieces;
    }
    let crossings = std::mem::take(&mut c.crossings);
    c.crossings = crossings
        .into_iter()
        .map(|(a, b)| {
            let (a, b) = (find(&parent, a), find(&parent, b));
            (a.min(b), a.max(b))
        })
        .filter(|(a, b)| a != b)
        .collect();
    pieces
        .into_iter()
        .filter_map(|p| {
            let ids = p.ids.map(|i| find(&parent, i));
            (ids[0] != ids[1] && ids[1] != ids[2] && ids[0] != ids[2]).then_some(Piece { ids, ..p })
        })
        .collect()
}

/// The self-union's steps: one mesh's arrangement with itself, keeping the
/// pieces between winding number below 1 and at least 1.
pub(crate) fn arrange_self(points: Vec<DVec3>, s: Solid) -> (Csg, Vec<Kept>) {
    let empty = Solid {
        tris: Vec::new(),
        corners: Vec::new(),
        source: Vec::new(),
        min: DVec3::INFINITY,
        max: DVec3::NEG_INFINITY,
    };
    let mut c = Csg::new(points, [s, empty]);
    // 1. Pairs whose boxes overlap: those sharing no vertex, and those
    // sharing one or an edge that cross improperly (a shared corner or
    // edge alone is not a crossing).
    let tris = c.solids[0].tris.clone();
    for (i, j) in selfcheck::box_pairs(&c.points, &tris) {
        let (ti, tj) = (tris[i as usize], tris[j as usize]);
        if ti.iter().any(|v| tj.contains(v))
            && !selfcheck::crossing(ti, ti.map(|v| c.p(v)), tj, tj.map(|v| c.p(v)))
        {
            continue;
        }
        c.pair((0, i), (0, j));
    }
    c.triples();
    // 2. The pieces.
    let pieces = c.all_pieces(1);
    // 3. Regions, each classified by the mesh's own winding number.
    let regions = c.regions(&pieces);
    let scale = c.scale();
    // Where triangles lie on one another, the lowest-numbered one answers.
    let mut partners: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for &((_, a), (_, b)) in &c.coplanar_pairs {
        partners.entry(a).or_default().push(b);
        partners.entry(b).or_default().push(a);
    }
    // The winding number just in front of a piece and just behind it differ
    // by one, its own surface. At the piece's centre, the winding number of
    // every other triangle is halfway between: it needs no step off the
    // surface, so a sliver is classified as surely as a large piece. Where
    // another face lies on the piece, that does not hold; points a small
    // step in front and behind decide there.
    let mut sides: BTreeMap<usize, (bool, bool)> = BTreeMap::new();
    let winder = Winder::new(&c.points, &tris, regions.best.len());
    for (&r, &b) in &regions.best {
        let p = &pieces[b];
        let side = if partners.contains_key(&p.tri) {
            let (front, back) = c.probes(p, scale);
            let inside = |q: DVec3| winder.side(q, None, &[0.5]) > 0.5;
            (inside(front), inside(back))
        } else {
            let [a, bb, cc] = p.ids.map(|i| c.p(i));
            let centre = (a + bb + cc) / 3.0;
            let rest = winder.side(centre, Some(p.tri as usize), &[1.0, 0.0]);
            (rest - 0.5 > 0.5, rest + 0.5 > 0.5)
        };
        sides.insert(r, side);
    }
    let covered_by_lower = |p: &Piece| {
        let Some(list) = partners.get(&p.tri) else {
            return false;
        };
        let axis = drop_axis(c.tri_normal(tris[p.tri as usize]));
        let [a, b, cc] = p.ids.map(|i| c.p(i));
        let q = project((a + b + cc) / 3.0, axis);
        list.iter().any(|&u| {
            u < p.tri && {
                let poly: Vec<DVec2> = tris[u as usize]
                    .iter()
                    .map(|&v| project(c.p(v), axis))
                    .collect();
                strictly_inside(&poly, q)
            }
        })
    };
    let mut kept = Vec::new();
    for (pi, p) in pieces.iter().enumerate() {
        let (front, back) = sides[&regions.root[pi]];
        if front == back || covered_by_lower(p) {
            continue;
        }
        kept.push(Kept {
            mesh: 0,
            tri: p.tri,
            ids: p.ids,
            flip: front,
        });
    }
    (c, kept)
}

/// Every pair of triangles whose boxes overlap, in the order the
/// all-pairs loop visited them (A's index, then B's), so crossing points
/// are made in the same order and the output is unchanged. Found by a
/// sweep along x over the boxes sorted by their low x.
pub(crate) fn candidate_pairs(
    a: &[(u32, (DVec3, DVec3))],
    b: &[(u32, (DVec3, DVec3))],
) -> Vec<(u32, u32)> {
    let by_min_x = |v: &[(u32, (DVec3, DVec3))]| {
        let mut v = v.to_vec();
        v.sort_by(|x, y| x.1.0.x.total_cmp(&y.1.0.x).then(x.0.cmp(&y.0)));
        v
    };
    let (sa, sb) = (by_min_x(a), by_min_x(b));
    let (mut ia, mut ib) = (0, 0);
    let mut active_a: Vec<(u32, (DVec3, DVec3))> = Vec::new();
    let mut active_b: Vec<(u32, (DVec3, DVec3))> = Vec::new();
    let mut pairs = Vec::new();
    while ia < sa.len() || ib < sb.len() {
        let take_a = ib >= sb.len() || (ia < sa.len() && sa[ia].1.0.x <= sb[ib].1.0.x);
        if take_a {
            let cur = sa[ia];
            ia += 1;
            active_b.retain(|o| o.1.1.x >= cur.1.0.x);
            pairs.extend(
                active_b
                    .iter()
                    .filter(|o| overlap(cur.1, o.1))
                    .map(|o| (cur.0, o.0)),
            );
            active_a.push(cur);
        } else {
            let cur = sb[ib];
            ib += 1;
            active_a.retain(|o| o.1.1.x >= cur.1.0.x);
            pairs.extend(
                active_a
                    .iter()
                    .filter(|o| overlap(o.1, cur.1))
                    .map(|o| (o.0, cur.0)),
            );
            active_b.push(cur);
        }
    }
    pairs.sort_unstable();
    pairs.dedup();
    pairs
}
