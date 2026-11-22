//! Step: edges with more than two faces split into manifold sheets.

use super::*;

/// What [`split_non_manifold`] did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NonManifold {
    /// Edges with more than two faces, found.
    pub edges: usize,
    /// Of those, edges split into manifold sheets.
    pub split: usize,
    /// Faces on a split edge that pair with none (an odd fan, or a fin
    /// sheet): the edge is open on their side now.
    pub unpaired: usize,
    /// Edges left with more than two faces: their faces' directions do
    /// not alternate round the edge (not one oriented surface), or the
    /// sheets are joined round both ends, so no vertex could be split.
    pub left: usize,
    /// Vertices copied, one copy per extra sheet through the vertex.
    pub vertices: usize,
    /// The largest distance a copy was moved into its own fan's cone, mm
    /// (1 % of its shortest edge), so that the sheets stay apart once
    /// written to a file and read back.
    pub moved: f64,
    /// Crossing pairs the moves made (a fin's lone face moved through the
    /// sheets round it), counted as `measure` counts them.
    pub crossings_added: usize,
    /// Pieces dropped as floaters once the sheets came apart (with the
    /// repair's `floaters` setting).
    pub sheets_dropped: usize,
    /// Triangles those pieces had.
    pub sheet_triangles_dropped: usize,
}

/// A triangle by its corners' positions, sorted: the same triangle
/// whatever its vertex numbers.
pub(crate) type TriKey = [[u64; 3]; 3];

pub(crate) fn tri_key(s: &Solid, t: u32) -> TriKey {
    let mut k = s.triangles[t as usize].map(|v| {
        let p = s.positions[v as usize];
        [p.x + 0.0, p.y + 0.0, p.z + 0.0].map(f64::to_bits)
    });
    k.sort_unstable();
    k
}

/// The touching pairs of `s` (as `measure` counts them, on vertices
/// welded where positions are equal), each by its triangles' keys,
/// sorted.
pub(crate) fn crossing_keys(s: &Solid) -> Vec<(TriKey, TriKey)> {
    let (positions, triangles) = crate::tools::measure::weld_exact(s);
    let mut out: Vec<(TriKey, TriKey)> = self_intersections(&positions, &triangles)
        .into_iter()
        .map(|(i, j)| {
            let (a, b) = (tri_key(s, i), tri_key(s, j));
            (a.min(b), a.max(b))
        })
        .collect();
    out.sort_unstable();
    out
}

/// Where the face with third corner `c` lies round the edge `a → b`,
/// against the reference face's third corner `c0`: 0 on its half-plane, 1
/// in the half turn after it, 2 on the half-plane opposite, 3 in the
/// half turn back. Exact.
pub(crate) fn half_turn(a: DVec3, b: DVec3, c0: DVec3, c: DVec3) -> u8 {
    match orient3d(a, b, c0, c) {
        Ordering::Less => 1,
        Ordering::Greater => 3,
        Ordering::Equal => {
            // In the reference plane: the same side of the line a b as c0,
            // in the projection that drops the plane's largest normal axis
            // (exact: a coordinate dropped, nothing computed).
            let n = (b - a).cross(c0 - a).abs();
            let f = |p: DVec3| {
                if n.x >= n.y && n.x >= n.z {
                    DVec2::new(p.y, p.z)
                } else if n.y >= n.z {
                    DVec2::new(p.z, p.x)
                } else {
                    DVec2::new(p.x, p.y)
                }
            };
            if orient2d(f(a), f(b), f(c0)) == orient2d(f(a), f(b), f(c)) {
                0
            } else {
                2
            }
        }
    }
}

/// Splits each edge with more than two faces into manifold sheets. The
/// faces are put in their order round the edge (exactly: `orient3d`), and
/// each face running `b → a` pairs with the next face round, running
/// `a → b`: the two bound one wedge of solid, so they are one sheet. At
/// each end of the edge, the faces round the vertex joined through
/// manifold edges and these pairs fall into fans; every fan but the
/// first gets its own copy of the vertex, moved towards the mean of the
/// fan's other corners (inside the cone its faces span) by 1 % of its
/// shortest edge there: two sheets
/// that touched along an edge are then apart, and stay apart in a file
/// (where equal positions would be welded back). Faces that pair with
/// none are left open along the edge. Run after [`orient`]. Names, UVs
/// and skins are kept (a copy takes its vertex's weights).
pub fn split_non_manifold(s: &Solid) -> Result<(Solid, NonManifold), RepairError> {
    let mut s = s.clone();
    s.canonicalize()?;
    let mut r = NonManifold::default();
    let mut by_edge: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
    for (i, t) in s.triangles.iter().enumerate() {
        for k in 0..3 {
            by_edge
                .entry(edge_key(t[k], t[(k + 1) % 3]))
                .or_default()
                .push(i as u32);
        }
    }
    let wide: Vec<(u32, u32)> = by_edge
        .iter()
        .filter(|e| e.1.len() > 2)
        .map(|e| *e.0)
        .collect();
    r.edges = wide.len();
    if wide.is_empty() {
        return Ok((s, r));
    }
    let mut touched: Vec<u32> = wide.iter().flat_map(|&(a, b)| [a, b]).collect();
    touched.sort_unstable();
    touched.dedup();
    let mut around: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (i, t) in s.triangles.iter().enumerate() {
        for &v in t {
            if touched.binary_search(&v).is_ok() {
                around.entry(v).or_default().push(i as u32);
            }
        }
    }
    // Fans round each vertex: its faces joined across manifold edges from
    // it, and across the pairs given (one sheet through a split edge).
    let fans_of = |v: u32, pairs: &BTreeMap<(u32, u32), Vec<(u32, u32)>>| -> Vec<u32> {
        let tris = &around[&v];
        let mut parent: Vec<u32> = (0..tris.len() as u32).collect();
        let at = |t: u32| tris.binary_search(&t).map_or(0, |i| i as u32);
        let join = |x: u32, y: u32, parent: &mut Vec<u32>| {
            let (x, y) = (find(parent, x), find(parent, y));
            if x != y {
                parent[x.max(y) as usize] = x.min(y);
            }
        };
        let mut others: Vec<u32> = tris
            .iter()
            .flat_map(|&t| s.triangles[t as usize])
            .filter(|&x| x != v)
            .collect();
        others.sort_unstable();
        others.dedup();
        for x in others {
            let e = edge_key(v, x);
            let faces = &by_edge[&e];
            if faces.len() == 2 {
                join(at(faces[0]), at(faces[1]), &mut parent);
            } else if let Some(ps) = pairs.get(&e) {
                for &(f, g) in ps {
                    join(at(f), at(g), &mut parent);
                }
            }
        }
        (0..tris.len() as u32)
            .map(|i| find(&mut parent, i))
            .collect()
    };
    let none = BTreeMap::new();
    let manifold: BTreeMap<u32, Vec<u32>> =
        touched.iter().map(|&v| (v, fans_of(v, &none))).collect();
    let runs = |t: [u32; 3], a: u32, b: u32| (0..3).any(|k| t[k] == a && t[(k + 1) % 3] == b);
    // Per non-manifold edge, the pairs of faces that are one sheet.
    // Edges are taken in turn, those first whose faces the surface
    // already pairs round their ends (through manifold edges and the
    // edges paired so far): along a chain of such edges each then pairs
    // as the one before it did, and the sheets come apart at the
    // vertices between. (Each on its own, an edge in the middle of a
    // chain was paired by its faces' order round it, which could join
    // what its neighbours kept apart: the house's 3 edges of §32.2.)
    let mut pairs: BTreeMap<(u32, u32), Vec<(u32, u32)>> = BTreeMap::new();
    let mut waiting: Vec<(u32, u32)> = wide.clone();
    while !waiting.is_empty() {
        // The fans round the ends of the edges waiting, as paired so far.
        let mut ends: Vec<u32> = waiting.iter().flat_map(|&(a, b)| [a, b]).collect();
        ends.sort_unstable();
        ends.dedup();
        let so_far: BTreeMap<u32, Vec<u32>> = if pairs.is_empty() {
            ends.iter().map(|&v| (v, manifold[&v].clone())).collect()
        } else {
            ends.iter().map(|&v| (v, fans_of(v, &pairs))).collect()
        };
        let given_of = |a: u32, b: u32| -> (Vec<(u32, u32)>, Vec<bool>, bool) {
            let faces = &by_edge[&(a, b)];
            let fan = |v: u32, t: u32| so_far[&v][around[&v].binary_search(&t).unwrap_or(0)];
            let mut sheet: Vec<(u32, u32)> = Vec::new();
            let mut paired = vec![false; faces.len()];
            let mut clash = false;
            for i in 0..faces.len() {
                for j in i + 1..faces.len() {
                    let (f, g) = (faces[i], faces[j]);
                    if fan(a, f) == fan(a, g) || fan(b, f) == fan(b, g) {
                        if paired[i] || paired[j] {
                            clash = true;
                        }
                        paired[i] = true;
                        paired[j] = true;
                        sheet.push((f, g));
                    }
                }
            }
            (sheet, paired, clash)
        };
        // The first edge wholly paired by what is there; else the first.
        let pick = waiting
            .iter()
            .position(|&(a, b)| {
                let (_, paired, clash) = given_of(a, b);
                !clash && paired.iter().all(|&p| p)
            })
            .unwrap_or(0);
        let (a, b) = waiting.remove(pick);
        let faces = &by_edge[&(a, b)];
        let k = faces.len();
        let (given, given_paired, given_clash) = given_of(a, b);
        let mut sheet = given;
        let paired = given_paired;
        if given_clash {
            r.left += 1;
            continue;
        }
        // The rest by their order round the edge (exactly: `orient3d`):
        // a face running b → a pairs with the next one round, running
        // a → b; the two bound one wedge of solid.
        let rest: Vec<u32> = (0..k).filter(|&i| !paired[i]).map(|i| faces[i]).collect();
        if rest.len() >= 2 {
            let (pa, pb) = (s.positions[a as usize], s.positions[b as usize]);
            let third = |t: u32| {
                let tri = s.triangles[t as usize];
                s.positions[tri[(0..3).find(|&q| tri[q] != a && tri[q] != b).unwrap_or(0)] as usize]
            };
            let c0 = third(rest[0]);
            let mut round: Vec<(u8, u32)> = rest
                .iter()
                .map(|&t| (half_turn(pa, pb, c0, third(t)), t))
                .collect();
            round.sort_by(|x, y| {
                x.0.cmp(&y.0).then_with(|| {
                    if x.0 % 2 == 0 {
                        return x.1.cmp(&y.1);
                    }
                    // y after x within the half turn: on x's normal side.
                    match orient3d(pa, pb, third(x.1), third(y.1)) {
                        Ordering::Equal => x.1.cmp(&y.1),
                        o => o,
                    }
                })
            });
            let n = round.len();
            let forward: Vec<bool> = round
                .iter()
                .map(|&(_, t)| runs(s.triangles[t as usize], a, b))
                .collect();
            let mut taken = vec![false; n];
            for i in 0..n {
                let j = (i + 1) % n;
                if i != j && !forward[i] && forward[j] && !taken[i] && !taken[j] {
                    taken[i] = true;
                    taken[j] = true;
                    sheet.push((round[i].1, round[j].1));
                }
            }
            r.unpaired += taken.iter().filter(|&&x| !x).count();
        } else {
            r.unpaired += rest.len();
        }
        pairs.insert((a, b), sheet);
    }
    let split: Vec<(u32, Vec<Vec<u32>>)> = touched
        .iter()
        .map(|&v| {
            let roots = fans_of(v, &pairs);
            let mut fans: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
            for (i, &t) in around[&v].iter().enumerate() {
                fans.entry(roots[i]).or_default().push(t);
            }
            (v, fans.into_values().skip(1).collect())
        })
        .collect();
    let mut renames: Vec<(u32, u32, u32)> = Vec::new();
    // Each copy and the vertex it copies.
    let mut copies: Vec<(u32, u32)> = Vec::new();
    for (v, fans) in split {
        for fan in &fans {
            // Towards the mean of the fan's other corners: inside the cone
            // its own faces span, away from the other sheets through v.
            let p = s.positions[v as usize];
            let mut corners: Vec<u32> = fan
                .iter()
                .flat_map(|&t| s.triangles[t as usize])
                .filter(|&w| w != v)
                .collect();
            corners.sort_unstable();
            corners.dedup();
            let mut mean = DVec3::ZERO;
            let mut shortest = f64::INFINITY;
            for &w in &corners {
                let q = s.positions[w as usize];
                mean += q;
                shortest = shortest.min((q - p).length());
            }
            let dir = mean / corners.len().max(1) as f64 - p;
            let len = dir.length();
            if !(len > 0.0 && shortest.is_finite() && shortest > 0.0) {
                continue;
            }
            let step = 0.01 * shortest;
            let copy = s.push_vertex(p + dir / len * step);
            if let Some(k) = &mut s.skin {
                k.weights[copy as usize] = k.weights[v as usize];
            }
            if let Some(m) = &mut s.morphs {
                m.copy(v as usize, copy as usize);
            }
            copies.push((copy, v));
            for &t in fan {
                renames.push((t, v, copy));
            }
        }
    }
    let before = crossing_keys(&s);
    for (t, v, copy) in renames {
        for w in &mut s.triangles[t as usize] {
            if *w == v {
                *w = copy;
            }
        }
    }
    // A fin's lone face, its corner moved, can pass through the sheets
    // round it: those pairs are counted (the fuse step untangles a closed
    // shell).
    let after = crossing_keys(&s);
    r.crossings_added = after
        .iter()
        .filter(|k| before.binary_search(k).is_err())
        .count();
    for (copy, v) in copies {
        r.vertices += 1;
        r.moved = r
            .moved
            .max((s.positions[copy as usize] - s.positions[v as usize]).length());
    }
    s.canonicalize()?;
    // What is still non-manifold.
    let mut count: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for t in &s.triangles {
        for k in 0..3 {
            *count.entry(edge_key(t[k], t[(k + 1) % 3])).or_default() += 1;
        }
    }
    r.left = count.values().filter(|&&n| n > 2).count();
    r.split = r.edges.saturating_sub(r.left);
    Ok((s, r))
}
