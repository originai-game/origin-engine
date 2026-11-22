//! Step: open boundaries closed, flat or as a dome, without crossing the surface.

use super::*;

/// A fill triangle: corners, surface, names, UVs.
pub(crate) type Fill = ([u32; 3], u32, crate::mesh::Tags, [DVec2; 3], CornerColors);

/// What [`close_holes`] did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Holes {
    /// Holes closed.
    pub filled: usize,
    /// Triangles added to close them.
    pub triangles: usize,
    /// Of the holes closed, those with more than `max_edges` edges (up to
    /// [`LARGE_HOLE_EDGES`]): their least-area fill turns no triangle
    /// against the fill's mean normal and crosses nothing.
    pub large: usize,
    /// Of the large holes closed, those whose fill crosses the mesh
    /// (only with [`close_holes_through`]'s `through`).
    pub crossing: usize,
    /// Of the large holes closed, those closed round ([`dome`]): rings
    /// of new vertices that leave the rim along the surface.
    pub round: usize,
    /// Holes left open: larger than `max_edges` and not flat or clear
    /// enough to fill, larger than [`LARGE_HOLE_EDGES`], or with a rim
    /// that does not close (it runs into a non-manifold edge).
    pub left: usize,
    /// Filled holes whose rim crosses a UV seam (a rim vertex with
    /// different UVs on its two rim triangles). Their fill takes one UV
    /// at every corner (its first rim vertex's), so it covers no area of
    /// the texture: one colour, the rim's there, and nothing overlaps.
    pub uv_seam_fills: usize,
}

/// The largest hole, in rim edges, filled when it has more edges than
/// asked (and is flat and clear enough): the fill costs O(n³).
pub const LARGE_HOLE_EDGES: usize = 512;

/// Closes every hole whose rim has at most `max_edges` edges, after
/// [`orient`]. A rim is traced along boundary edges; a rim that passes a
/// vertex twice (pinched) is cut there into simple loops, each closed on
/// its own. The fill is the triangulation of
/// the rim with the least total area (dynamic programming over the rim,
/// O(n³)), wound to match the faces around it; a triangle whose new edge
/// already exists in the mesh is not allowed, so the result stays
/// manifold; a rim with no such fill is fanned from a new vertex at its
/// centre instead. A hole with more edges than `max_edges` (up to
/// [`LARGE_HOLE_EDGES`]) is closed too when its least-area fill is flat
/// enough and clear: no fill triangle faces against the fill's
/// area-weighted mean normal, and none crosses the mesh or another fill
/// (`crate::boolean::selfcheck::crossing`, exact). Returns the solid and
/// what was done ([`Holes`]).
///
/// UVs: each rim vertex takes the UV its corner has in the rim triangle
/// of the rim edge leaving it; a centre vertex takes their mean. A rim
/// vertex whose two rim triangles give it different UVs sits on a seam,
/// and a fan through it would stretch across the texture's atlas: such
/// a fill takes its first rim vertex's UV at every corner instead.
pub fn close_holes(s: &Solid, max_edges: usize) -> Result<(Solid, Holes), RepairError> {
    close_holes_through(s, max_edges, false)
}

/// [`close_holes`]; with `through`, a large hole no clear fill closes
/// is closed all the same by its least-area fill, which then crosses
/// the mesh ([`Holes::crossing`] counts those): for a mesh with faces
/// behind its holes, to be untangled by `fuse` afterwards.
pub fn close_holes_through(
    s: &Solid,
    max_edges: usize,
    through: bool,
) -> Result<(Solid, Holes), RepairError> {
    let mut s = s.clone();
    s.canonicalize()?;
    let mut directed: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    let mut undirected: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for (i, t) in s.triangles.iter().enumerate() {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            directed.insert((a, b), i as u32);
            *undirected.entry(edge_key(a, b)).or_default() += 1;
        }
    }
    // Rim edges run against their boundary triangle: b → a for a → b.
    let mut out_edges: BTreeMap<u32, Vec<(u32, u32)>> = BTreeMap::new();
    for (&(a, b), &tri) in &directed {
        if undirected[&edge_key(a, b)] == 1 {
            out_edges.entry(b).or_default().push((a, tri));
        }
    }
    let mut used: BTreeMap<(u32, u32), ()> = BTreeMap::new();
    let mut h = Holes::default();
    let starts: Vec<(u32, u32)> = out_edges
        .iter()
        .flat_map(|(&v, es)| es.iter().map(move |&(w, _)| (v, w)))
        .collect();
    // Trace every rim; a rim that passes a vertex twice (pinched) is cut
    // there into simple loops, each closed on its own.
    let mut loops: Vec<(Vec<u32>, Vec<u32>)> = Vec::new();
    for (v0, w0) in starts {
        if used.contains_key(&(v0, w0)) {
            continue;
        }
        let mut path: Vec<u32> = vec![v0];
        let mut path_tris: Vec<u32> = Vec::new();
        let (mut v, mut w) = (v0, w0);
        loop {
            used.insert((v, w), ());
            let tri = out_edges[&v].iter().find(|e| e.0 == w).map_or(0, |e| e.1);
            path_tris.push(tri);
            if let Some(k) = path.iter().position(|&x| x == w) {
                // A loop closes at w: cut it off.
                loops.push((path.split_off(k), path_tris.split_off(k)));
                path.push(w);
            } else {
                path.push(w);
            }
            let Some(next) = out_edges.get(&w).and_then(|e| {
                e.iter()
                    .find(|(x, _)| !used.contains_key(&(w, *x)))
                    .map(|&(x, _)| x)
            }) else {
                break;
            };
            v = w;
            w = next;
        }
        // Whatever did not close (a rim through a non-manifold edge).
        if path.len() > 1 {
            h.left += 1;
        }
    }
    let mut new_tris: Vec<Fill> = Vec::new();
    for (rim, rim_tris) in loops {
        if rim.len() < 3 || rim.len() > max_edges.max(LARGE_HOLE_EDGES) {
            h.left += 1;
            continue;
        }
        let large = rim.len() > max_edges;
        // A large hole: its least-area fill where that is flat and
        // clear; else a fan from its centre lifted a little along the
        // fill's mean normal, either way, where one is clear; else,
        // if asked, the least-area fill though it crosses.
        let mut large_fill = None;
        let mut fan_from = None;
        let mut round: Option<Round> = None;
        if large {
            let least = fill(&s, &rim, &undirected);
            let mean: DVec3 = least
                .iter()
                .flatten()
                .map(|t| {
                    let [a, b, c] = t.map(|v| s.positions[v as usize]);
                    (b - a).cross(c - a)
                })
                .sum();
            let along = mean.try_normalize().unwrap_or(DVec3::ZERO);
            // A round fill that leaves the rim along the surface, where
            // it is clear: first where the surface is tilted at the rim
            // (sine 0.1, about 6°: a hole in a curved surface), and,
            // where no flat fill will do, whatever the tilt.
            let try_round = |least_tilt: f64, new_tris: &[Fill]| -> Option<Round> {
                let base = s.positions.len() as u32;
                let (points, tris) = dome(&s, &rim, &rim_tris, along, base, least_tilt)?;
                let mut tried = Solid {
                    positions: s.positions.clone(),
                    triangles: s.triangles.clone(),
                    ..Default::default()
                };
                tried.positions.extend(points.iter().map(|p| p.0));
                (!crosses(&tried, &tris, new_tris)).then_some((points, tris))
            };
            round = try_round(0.1, &new_tris);
            match &least {
                _ if round.is_some() => {}
                Some(tris) if flat(&s, tris) && !crosses(&s, tris, &new_tris) => {
                    large_fill = least.clone();
                }
                _ => {
                    let centre = rim.iter().map(|&v| s.positions[v as usize]).sum::<DVec3>()
                        / rim.len() as f64;
                    let radius = rim
                        .iter()
                        .map(|&v| (s.positions[v as usize] - centre).length())
                        .fold(0.0, f64::max);
                    round = try_round(0.0, &new_tris);
                    for lift in [0.05, 0.15, -0.05, -0.15, 0.0, 0.3] {
                        if round.is_some() {
                            break;
                        }
                        let at = centre + along * (lift * radius);
                        let mut tried = Solid {
                            positions: s.positions.clone(),
                            triangles: s.triangles.clone(),
                            ..Default::default()
                        };
                        let ci = tried.positions.len() as u32;
                        tried.positions.push(at);
                        let fan: Vec<[u32; 3]> = (0..rim.len())
                            .map(|k| [rim[k], rim[(k + 1) % rim.len()], ci])
                            .collect();
                        if !crosses(&tried, &fan, &new_tris) {
                            fan_from = Some(at);
                            break;
                        }
                    }
                    if fan_from.is_none() && round.is_none() && through {
                        large_fill = least.clone();
                        h.crossing += usize::from(least.is_some());
                    }
                }
            }
            if large_fill.is_none() && fan_from.is_none() && round.is_none() {
                h.left += 1;
                continue;
            }
        }
        // Each rim vertex's UV in its rim triangles: leaving (j) and
        // arriving (j − 1).
        let uv_in = |t: u32, v: u32| -> DVec2 {
            let tri = s.triangles[t as usize];
            let k = tri.iter().position(|&x| x == v).unwrap_or(0);
            s.uvs.as_ref().map_or(DVec2::ZERO, |u| u[t as usize][k])
        };
        let n = rim.len();
        let rim_uv: Vec<DVec2> = (0..n).map(|j| uv_in(rim_tris[j], rim[j])).collect();
        let seam = s.uvs.is_some()
            && (0..n).any(|j| uv_in(rim_tris[(j + n - 1) % n], rim[j]) != rim_uv[j]);
        let centre_uv = rim_uv.iter().copied().sum::<DVec2>() / n as f64;
        // Colours likewise: each rim vertex its colour in the rim
        // triangle leaving it, a fan's centre their mean.
        let rim_color: Vec<[f64; 4]> = (0..n)
            .map(|j| {
                let t = rim_tris[j] as usize;
                let k = s.triangles[t]
                    .iter()
                    .position(|&x| x == rim[j])
                    .unwrap_or(0);
                crate::mesh::wide(s.colors.as_ref().map_or(WHITE, |c| c[t][k]))
            })
            .collect();
        let centre_color: [f64; 4] =
            core::array::from_fn(|ch| rim_color.iter().map(|c| c[ch]).sum::<f64>() / n as f64);
        // A round fill's points: each takes its rim vertex's weights,
        // targets, UV and colour, blended towards the rim's mean as it
        // nears the centre.
        let mut spoke: BTreeMap<u32, (usize, f64)> = BTreeMap::new();
        let round_tris = round.map(|(points, tris)| {
            for (p, j, u) in points {
                let v = s.push_vertex(p);
                spoke.insert(v, (j, u));
                let share = |i: usize| u / n as f64 + if i == j { 1.0 - u } else { 0.0 };
                if let Some(k) = &mut s.skin {
                    let items: Vec<(Weights, f64)> = rim
                        .iter()
                        .enumerate()
                        .map(|(i, &r)| (k.weights[r as usize], share(i)))
                        .collect();
                    k.weights[v as usize] = Weights::blend(&items);
                }
                if let Some(m) = &mut s.morphs {
                    let items: Vec<(usize, f64)> = rim
                        .iter()
                        .enumerate()
                        .map(|(i, &r)| (r as usize, share(i)))
                        .collect();
                    m.blend(v as usize, &items);
                }
            }
            h.round += 1;
            tris
        });
        let tris = round_tris
            .or(large_fill)
            .or_else(|| {
                if fan_from.is_some() {
                    None
                } else {
                    fill(&s, &rim, &undirected)
                }
            })
            .unwrap_or_else(|| {
                // Every chord already exists (or a large hole took a
                // fan): fan from a new vertex at the rim's centre,
                // whose edges are all new.
                let c = fan_from.unwrap_or_else(|| {
                    rim.iter().map(|&v| s.positions[v as usize]).sum::<DVec3>() / rim.len() as f64
                });
                let ci = s.push_vertex(c);
                if let Some(k) = &mut s.skin {
                    // The centre takes the blend of its rim's weights.
                    let items: Vec<(Weights, f64)> = rim
                        .iter()
                        .map(|&v| (k.weights[v as usize], 1.0 / rim.len() as f64))
                        .collect();
                    k.weights[ci as usize] = Weights::blend(&items);
                }
                if let Some(m) = &mut s.morphs {
                    let items: Vec<(usize, f64)> = rim
                        .iter()
                        .map(|&v| (v as usize, 1.0 / rim.len() as f64))
                        .collect();
                    m.blend(ci as usize, &items);
                }
                (0..rim.len())
                    .map(|k| [rim[k], rim[(k + 1) % rim.len()], ci])
                    .collect()
            });
        let mut tags: Vec<Tag> = vec![s.names.intern("fill")];
        for &t in &rim_tris {
            tags.extend(s.face_tags[t as usize].iter());
        }
        tags.sort_unstable();
        tags.dedup();
        // One list for all of the fill's triangles.
        let tags: crate::mesh::Tags = tags.into();
        let surface = s.surfaces[rim_tris[0] as usize];
        let uv_of = |v: u32| {
            if seam {
                return rim_uv[0];
            }
            if let Some(&(j, u)) = spoke.get(&v) {
                let from = rim_uv.get(j).copied().unwrap_or(centre_uv);
                return from * (1.0 - u) + centre_uv * u;
            }
            rim.iter()
                .position(|&x| x == v)
                .map_or(centre_uv, |j| rim_uv[j])
        };
        let color_of = |v: u32| {
            if let Some(&(j, u)) = spoke.get(&v) {
                let from = rim_color.get(j).copied().unwrap_or(centre_color);
                return core::array::from_fn(|ch| from[ch] * (1.0 - u) + centre_color[ch] * u);
            }
            rim.iter()
                .position(|&x| x == v)
                .map_or(centre_color, |j| rim_color[j])
        };
        for t in &tris {
            for k in 0..3 {
                *undirected
                    .entry(edge_key(t[k], t[(k + 1) % 3]))
                    .or_default() += 1;
            }
        }
        h.triangles += tris.len();
        h.filled += 1;
        h.large += usize::from(large);
        h.uv_seam_fills += usize::from(seam);
        new_tris.extend(tris.into_iter().map(|t| {
            (
                t,
                surface,
                tags.clone(),
                t.map(uv_of),
                t.map(|v| crate::mesh::stored(color_of(v))),
            )
        }));
    }
    for (t, surface, tags, uv, color) in new_tris {
        s.triangles.push(t);
        s.surfaces.push(surface);
        s.face_tags.push(tags);
        if let Some(u) = &mut s.uvs {
            u.push(uv);
        }
        if let Some(c) = &mut s.colors {
            c.push(color);
        }
    }
    s.canonicalize()?;
    Ok((s, h))
}

/// A round fill: its new points (each with its rim vertex's place in
/// the rim and how far along its curve it is) and its triangles.
pub(crate) type Round = (Vec<(DVec3, usize, f64)>, Vec<[u32; 3]>);

/// A round fill for a rim: rings of vertices between the rim and a
/// centre, each on a curve that leaves its rim vertex along the surface
/// there (in the plane of the triangle at the rim) and arrives at the
/// centre flat. The centre stands off the rim's centre by what a
/// spherical cap of the rim's mean tilt would (`along` is the way the
/// fill faces): 0 for a hole in a flat face, the cap's height for one
/// cut from a ball. Gives the new points (each with its rim vertex's
/// place in the rim and how far along its curve it is; the centre
/// last, with no rim vertex) and the triangles, rim vertices by their
/// own numbers and new points from `base` on.
///
/// `None` for a rim under 8 edges, or where the surface leaves the rim
/// by less than `least_tilt` (the sine of its mean tilt): a flat fill
/// does as well there.
pub(crate) fn dome(
    s: &Solid,
    rim: &[u32],
    rim_tris: &[u32],
    along: DVec3,
    base: u32,
    least_tilt: f64,
) -> Option<Round> {
    let n = rim.len();
    if n < 8 || along == DVec3::ZERO {
        return None;
    }
    let at = |j: usize| s.positions[rim[j % n] as usize];
    let centre = (0..n).map(at).sum::<DVec3>() / n as f64;
    let radius = (0..n).map(|j| (at(j) - centre).length()).sum::<f64>() / n as f64;
    // Into the hole along the surface, at each rim edge: across the
    // edge, in its triangle's plane.
    let inward: Vec<DVec3> = (0..n)
        .map(|j| {
            let [a, b, c] = s.triangles[rim_tris[j] as usize].map(|v| s.positions[v as usize]);
            let normal = (b - a).cross(c - a);
            normal
                .cross(at(j + 1) - at(j))
                .try_normalize()
                .unwrap_or(DVec3::ZERO)
        })
        .collect();
    let at_vertex: Vec<DVec3> = (0..n)
        .map(|j| {
            (inward[j] + inward[(j + n - 1) % n])
                .try_normalize()
                .unwrap_or(DVec3::ZERO)
        })
        .collect();
    // The mean tilt out of the rim's plane, held to 60°; a cap of that
    // tilt stands radius · tan(tilt / 2) off its rim.
    let sin = (at_vertex.iter().map(|d| d.dot(along)).sum::<f64>() / n as f64).clamp(-0.866, 0.866);
    if sin.abs() < least_tilt {
        return None;
    }
    let height = radius * sin / (1.0 + (1.0 - sin * sin).sqrt());
    let top = centre + along * height;
    let rings = (n / 6).clamp(2, 16);
    let mut points: Vec<(DVec3, usize, f64)> = Vec::with_capacity((rings - 1) * n + 1);
    for k in 1..rings {
        let u = k as f64 / rings as f64;
        for (j, leaving) in at_vertex.iter().enumerate() {
            let p = at(j);
            let reach = (top - p).length();
            let out = p - centre;
            let flat = (out - along * out.dot(along))
                .try_normalize()
                .unwrap_or(DVec3::ZERO);
            // A cubic from the rim vertex, leaving along the surface,
            // to the top, arriving in its plane.
            let (p1, p2) = (p + *leaving * (reach / 3.0), top + flat * (reach / 3.0));
            let v = 1.0 - u;
            let q = p * (v * v * v)
                + p1 * (3.0 * v * v * u)
                + p2 * (3.0 * v * u * u)
                + top * (u * u * u);
            points.push((q, j, u));
        }
    }
    points.push((top, usize::MAX, 1.0));
    let ring = |k: usize, j: usize| -> u32 {
        if k == 0 {
            rim[j % n]
        } else {
            base + ((k - 1) * n + j % n) as u32
        }
    };
    let mut tris = Vec::with_capacity(2 * (rings - 1) * n + n);
    for k in 1..rings {
        for j in 0..n {
            let (a, b) = (ring(k - 1, j), ring(k - 1, j + 1));
            let (c, d) = (ring(k, j + 1), ring(k, j));
            tris.push([a, b, c]);
            tris.push([a, c, d]);
        }
    }
    let last = base + ((rings - 1) * n) as u32;
    for j in 0..n {
        tris.push([ring(rings - 1, j), ring(rings - 1, j + 1), last]);
    }
    Some((points, tris))
}

/// No triangle of the fill faces against its area-weighted mean normal.
pub(crate) fn flat(s: &Solid, tris: &[[u32; 3]]) -> bool {
    let normal = |t: &[u32; 3]| {
        let [a, b, c] = t.map(|v| s.positions[v as usize]);
        (b - a).cross(c - a)
    };
    let mean: DVec3 = tris.iter().map(normal).sum();
    tris.iter().all(|t| normal(t).dot(mean) > 0.0)
}

/// A triangle of the fill crosses the mesh, another fill triangle, or a
/// fill already made (exactly; corners shared by index are joins).
pub(crate) fn crosses(s: &Solid, tris: &[[u32; 3]], made: &[Fill]) -> bool {
    let at = |t: [u32; 3]| t.map(|v| s.positions[v as usize]);
    let others: Vec<[u32; 3]> = s
        .triangles
        .iter()
        .copied()
        .chain(made.iter().map(|m| m.0))
        .chain(tris.iter().copied())
        .collect();
    let boxes: Vec<(DVec3, DVec3)> = others
        .iter()
        .map(|&t| {
            let p = at(t);
            (p[0].min(p[1]).min(p[2]), p[0].max(p[1]).max(p[2]))
        })
        .collect();
    let first = others.len() - tris.len();
    for (k, &t) in tris.iter().enumerate() {
        let p = at(t);
        if degenerate(p) {
            return true;
        }
        let (lo, hi) = boxes[first + k];
        for (j, &o) in others.iter().enumerate() {
            if j == first + k {
                continue;
            }
            let (olo, ohi) = boxes[j];
            if !(lo.cmple(ohi).all() && olo.cmple(hi).all()) {
                continue;
            }
            let q = at(o);
            if !degenerate(q) && crate::boolean::selfcheck::crossing(t, p, o, q) {
                return true;
            }
        }
    }
    false
}

/// The least-area triangulation of a rim, avoiding edges the mesh already
/// has (other than the rim's own).
pub(crate) fn fill(
    s: &Solid,
    rim: &[u32],
    edges: &BTreeMap<(u32, u32), u32>,
) -> Option<Vec<[u32; 3]>> {
    let n = rim.len();
    let p = |i: usize| s.positions[rim[i] as usize];
    let chord_ok = |i: usize, j: usize| {
        j == i + 1 || (i == 0 && j == n - 1) || {
            let (a, b) = (rim[i], rim[j]);
            a != b && !edges.contains_key(&edge_key(a, b))
        }
    };
    // cost[i][j]: least area of the polygon rim[i..=j] closed by the
    // chord j → i; choice[i][j]: its apex.
    let mut cost = vec![f64::INFINITY; n * n];
    let mut choice = vec![usize::MAX; n * n];
    for i in 0..n - 1 {
        cost[i * n + i + 1] = 0.0;
    }
    for len in 2..n {
        for i in 0..n - len {
            let j = i + len;
            if !chord_ok(i, j) {
                continue;
            }
            for m in i + 1..j {
                let (a, b) = (cost[i * n + m], cost[m * n + j]);
                if !(a.is_finite() && b.is_finite()) {
                    continue;
                }
                let area = (p(m) - p(i)).cross(p(j) - p(i)).length();
                let c = a + b + area;
                if c < cost[i * n + j] {
                    cost[i * n + j] = c;
                    choice[i * n + j] = m;
                }
            }
        }
    }
    if !cost[n - 1].is_finite() {
        return None;
    }
    let mut out = Vec::with_capacity(n - 2);
    let mut stack = vec![(0usize, n - 1)];
    while let Some((i, j)) = stack.pop() {
        if j <= i + 1 {
            continue;
        }
        let m = choice[i * n + j];
        out.push([rim[i], rim[m], rim[j]]);
        stack.push((i, m));
        stack.push((m, j));
    }
    Some(out)
}
