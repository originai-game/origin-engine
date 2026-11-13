//! Building a tile: regions, contours, convex polygons and the links
//! between them (steps 1 to 4 of the module's account).

use super::*;

/// What lies across a contour edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Across {
    Wall,
    /// A region of a tile (this one or a neighbour).
    Region(u32, u32),
}

impl NavMesh {
    /// Monotone regions of one tile: a row sweep in which a run of
    /// connected spans takes the region below it when it is the only run
    /// touching that region, so every region is one run per row.
    pub(in crate::nav::mesh) fn assign_regions(&mut self, t: usize) {
        let (x0, z0, x1, z1) = self.tile_rect(t);
        let mut region: IdMap<usize, u32> = IdMap::new();
        let mut next_region = 1u32;
        let mut rows = RegionRows::new();
        for z in z0..z1 {
            struct Sweep {
                nei: u32,
                ns: u32,
                cells: Vec<(i32, usize)>,
            }
            const MIXED: u32 = u32::MAX;
            let mut sweeps: Vec<Sweep> = Vec::new();
            let mut run_of: IdMap<usize, usize> = IdMap::new();
            let mut prev: IdMap<u32, u32> = IdMap::new();
            for x in x0..x1 {
                for s in self.field.column(x, z) {
                    if !self.walkable(s) {
                        continue;
                    }
                    let sp = self.field.spans[s];
                    let area = self.span_area[s];
                    let left = if x > x0 {
                        self.step(sp, x - 1, z)
                            .filter(|&l| self.span_area[l] == area)
                            .and_then(|l| run_of.get(&l).copied())
                    } else {
                        None
                    };
                    let run = match left {
                        Some(r) => r,
                        None => {
                            sweeps.push(Sweep {
                                nei: 0,
                                ns: 0,
                                cells: Vec::new(),
                            });
                            sweeps.len() - 1
                        }
                    };
                    if z > z0
                        && let Some(d) = self.step(sp, x, z - 1)
                        && self.span_area[d] == area
                        && let Some(&nr) = region.get(&d)
                    {
                        let sw = &mut sweeps[run];
                        if sw.nei == 0 || sw.nei == nr {
                            sw.nei = nr;
                            sw.ns += 1;
                            *prev.entry(nr).or_insert(0) += 1;
                        } else {
                            sw.nei = MIXED;
                        }
                    }
                    run_of.insert(s, run);
                    sweeps[run].cells.push((x, s));
                }
            }
            for sw in &mut sweeps {
                let id = if sw.nei != 0
                    && sw.nei != MIXED
                    && prev.get(&sw.nei).copied() == Some(sw.ns)
                {
                    sw.nei
                } else {
                    next_region += 1;
                    next_region - 1
                };
                for &(_, s) in &sw.cells {
                    region.insert(s, id);
                }
                rows.entry(id)
                    .or_default()
                    .push((z, std::mem::take(&mut sw.cells)));
            }
        }
        // Clear the tile's old regions, then set the new ones.
        for z in z0..z1 {
            for x in x0..x1 {
                for s in self.field.column(x, z) {
                    self.span_region[s] = 0;
                }
            }
        }
        for (s, r) in region {
            self.span_region[s] = r;
        }
        self.rows[t] = rows;
    }

    /// Polygonises one tile from its regions.
    pub fn build_tile(&self, t: usize) -> Tile {
        let (x0, z0, x1, z1) = self.tile_rect(t);
        let f = &self.field;
        // 2.–3. Contours, triangles, polygons per region.
        let mut tile = Tile::default();
        let mut vert_index: BTreeMap<(i32, i32, u32), u16> = BTreeMap::new();
        for (&rid, rws) in &self.rows[t] {
            let Some(contour) = self.contour(t, rid, rws, (x0, z0, x1, z1)) else {
                continue;
            };
            let pts: Vec<(i64, i64)> = contour.iter().map(|c| (c.0 as i64, c.1 as i64)).collect();
            let tris = triangulate(&pts);
            let polys = merge(&pts, tris);
            let area = rws
                .first()
                .and_then(|r| r.1.first())
                .map_or(0, |&(_, s)| self.span_area[s]);
            // Vertex heights: the highest floor of this region's cells around the vertex.
            let height = |(ix, iz): (i32, i32)| -> u32 {
                let mut best: Option<u32> = None;
                for (cx, cz) in [(ix - 1, iz - 1), (ix, iz - 1), (ix - 1, iz), (ix, iz)] {
                    if self.tile_of(cx.max(0), cz.max(0)) != t {
                        continue;
                    }
                    for s in f.column(cx, cz) {
                        if self.span_region[s] == rid {
                            best = Some(
                                best.map_or(f.spans[s].floor, |b: u32| b.max(f.spans[s].floor)),
                            );
                        }
                    }
                }
                best.unwrap_or(0)
            };
            for poly in polys {
                let mut verts = Vec::with_capacity(poly.len());
                for &k in &poly {
                    let (ix, iz) = contour[k];
                    let h = height((ix, iz));
                    let key = (ix, iz, h);
                    let idx = *vert_index.entry(key).or_insert_with(|| {
                        tile.verts.push(f.world(ix, iz, h));
                        tile.lattice.push((ix, iz));
                        (tile.verts.len() - 1) as u16
                    });
                    verts.push(idx);
                }
                let n = verts.len();
                tile.polys.push(Poly {
                    verts,
                    links: vec![NONE; n],
                    region: rid,
                    area,
                });
            }
        }
        tile
    }

    /// A region's outline on the lattice, simplified, counter-clockwise.
    fn contour(
        &self,
        t: usize,
        rid: u32,
        rows: &[(i32, Vec<(i32, usize)>)],
        rect: (i32, i32, i32, i32),
    ) -> Option<Vec<(i32, i32)>> {
        let f = &self.field;
        let _ = rect;
        let zmin = rows.first()?.0;
        let zmax = rows.last()?.0;
        let row = |z: i32| -> &Vec<(i32, usize)> { &rows[(z - zmin) as usize].1 };
        let l = |z: i32| row(z).first().map_or(0, |c| c.0);
        let r = |z: i32| row(z).last().map_or(0, |c| c.0 + 1);
        let span_at = |z: i32, x: i32| -> usize {
            row(z)
                .iter()
                .find(|c| c.0 == x)
                .map_or(row(z)[0].1, |c| c.1)
        };
        // What lies across from the inside span `inside` in cell (ox, oz).
        let across = |inside: usize, ox: i32, oz: i32| -> Across {
            match self.step(f.spans[inside], ox, oz) {
                Some(o) => {
                    let g = self.span_region[o];
                    let ot = self.tile_of(ox, oz) as u32;
                    if g == 0 || (ot as usize == t && g == rid) {
                        Across::Wall
                    } else {
                        Across::Region(ot, g)
                    }
                }
                None => Across::Wall,
            }
        };
        // Unit segments: (from vertex, what lies across).
        let mut segs: Vec<((i32, i32), Across)> = Vec::new();
        let (lb, rb) = (l(zmin), r(zmin));
        for x in lb..rb {
            segs.push(((x, zmin), across(span_at(zmin, x), x, zmin - 1)));
        }
        for z in zmin..=zmax {
            let rz = r(z);
            segs.push(((rz, z), across(span_at(z, rz - 1), rz, z)));
            if z < zmax {
                let rn = r(z + 1);
                if rn > rz {
                    for x in rz..rn {
                        segs.push(((x, z + 1), across(span_at(z + 1, x), x, z)));
                    }
                } else if rn < rz {
                    for x in (rn..rz).rev() {
                        segs.push(((x + 1, z + 1), across(span_at(z, x), x, z + 1)));
                    }
                }
            }
        }
        let (lt, rt) = (l(zmax), r(zmax));
        for x in (lt..rt).rev() {
            segs.push(((x + 1, zmax + 1), across(span_at(zmax, x), x, zmax + 1)));
        }
        for z in (zmin..=zmax).rev() {
            let lz = l(z);
            segs.push(((lz, z + 1), across(span_at(z, lz), lz - 1, z)));
            if z > zmin {
                let lp = l(z - 1);
                if lp < lz {
                    for x in (lp..lz).rev() {
                        segs.push(((x + 1, z), across(span_at(z - 1, x), x, z)));
                    }
                } else if lp > lz {
                    for x in lz..lp {
                        segs.push(((x, z), across(span_at(z, x), x, z - 1)));
                    }
                }
            }
        }
        let n = segs.len();
        if n < 3 {
            return None;
        }
        // Keep corners, changes of what lies across, and every tile-border vertex.
        let dir = |i: usize| {
            let a = segs[i].0;
            let b = segs[(i + 1) % n].0;
            (b.0 - a.0, b.1 - a.1)
        };
        let mut keep: Vec<usize> = Vec::new();
        for i in 0..n {
            let p = (i + n - 1) % n;
            let corner = dir(p) != dir(i);
            let change = segs[p].1 != segs[i].1;
            if corner || change {
                keep.push(i);
            }
        }
        if keep.len() < 3 {
            return None;
        }
        let pts: Vec<(i32, i32)> = keep.iter().map(|&i| segs[i].0).collect();
        let tags: Vec<Across> = keep.iter().map(|&i| segs[i].1).collect();
        Some(simplify_walls(&pts, &tags, self.field.params.max_error))
    }

    /// Links every polygon edge to the polygon sharing it.
    pub(in crate::nav::mesh) fn link(&mut self) {
        let climb = self.field.params.agent_climb + self.field.params.cell_height * 2.0;
        // Directed lattice edge → (poly, edge, mid y).
        let mut edges: EdgeMap = BTreeMap::new();
        for (t, tile) in self.tiles.iter().enumerate() {
            for (p, poly) in tile.polys.iter().enumerate() {
                let n = poly.verts.len();
                for e in 0..n {
                    let (a, b) = (poly.verts[e] as usize, poly.verts[(e + 1) % n] as usize);
                    let y = (tile.verts[a].y + tile.verts[b].y) * 0.5;
                    edges
                        .entry((tile.lattice[a], tile.lattice[b]))
                        .or_default()
                        .push((poly_ref(t, p), e, y));
                }
            }
        }
        for t in 0..self.tiles.len() {
            for p in 0..self.tiles[t].polys.len() {
                let n = self.tiles[t].polys[p].verts.len();
                for e in 0..n {
                    let tile = &self.tiles[t];
                    let poly = &tile.polys[p];
                    let (a, b) = (poly.verts[e] as usize, poly.verts[(e + 1) % n] as usize);
                    let y = (tile.verts[a].y + tile.verts[b].y) * 0.5;
                    let mut best: Option<(f32, PolyRef)> = None;
                    if let Some(list) = edges.get(&(tile.lattice[b], tile.lattice[a])) {
                        for &(q, _, qy) in list {
                            let d = (qy - y).abs();
                            if q != poly_ref(t, p)
                                && d <= climb
                                && best.is_none_or(|(bd, br)| d < bd || (d == bd && q < br))
                            {
                                best = Some((d, q));
                            }
                        }
                    }
                    self.tiles[t].polys[p].links[e] = best.map_or(NONE, |b| b.1);
                }
            }
        }
    }
}

/// Douglas–Peucker over each run of wall edges, keeping the outline simple.
fn simplify_walls(pts: &[(i32, i32)], tags: &[Across], max_error: f32) -> Vec<(i32, i32)> {
    let n = pts.len();
    // Anchors: vertices where an edge that is not a wall starts or ends.
    let anchor: Vec<bool> = (0..n)
        .map(|i| tags[i] != Across::Wall || tags[(i + n - 1) % n] != Across::Wall)
        .collect();
    let mut keep = anchor.clone();
    let anchors: Vec<usize> = (0..n).filter(|&i| anchor[i]).collect();
    let chains: Vec<(usize, usize)> = if anchors.is_empty() {
        // All walls: anchor at the lowest-left vertex and the one farthest from it.
        let a = (0..n).min_by_key(|&i| (pts[i].0, pts[i].1)).unwrap_or(0);
        let far = |i: usize| {
            let (dx, dz) = ((pts[i].0 - pts[a].0) as i64, (pts[i].1 - pts[a].1) as i64);
            dx * dx + dz * dz
        };
        let b = (0..n)
            .max_by_key(|&i| (far(i), std::cmp::Reverse(i)))
            .unwrap_or(0);
        keep[a] = true;
        keep[b] = true;
        vec![(a, b), (b, a)]
    } else {
        let k = anchors.len();
        (0..k).map(|i| (anchors[i], anchors[(i + 1) % k])).collect()
    };
    let err2 = (max_error * max_error) as f64;
    for (s, e) in chains {
        // The chain s → e (exclusive of both), walls only.
        let mut idx = Vec::new();
        let mut i = (s + 1) % n;
        while i != e {
            idx.push(i);
            i = (i + 1) % n;
        }
        if idx.is_empty() {
            continue;
        }
        let mut chain_keep = vec![false; idx.len()];
        dp(pts, s, e, &idx, &mut chain_keep, err2);
        let mut trial = keep.clone();
        for (k, &j) in idx.iter().enumerate() {
            trial[j] = chain_keep[k];
        }
        let candidate: Vec<(i32, i32)> = (0..n).filter(|&j| trial[j]).map(|j| pts[j]).collect();
        if candidate.len() >= 3 && simple(&candidate) && area2(&candidate) > 0 {
            keep = trial;
        } else {
            for &j in &idx {
                keep[j] = true;
            }
        }
    }
    (0..n).filter(|&i| keep[i]).map(|i| pts[i]).collect()
}

fn dp(pts: &[(i32, i32)], a: usize, b: usize, idx: &[usize], keep: &mut [bool], err2: f64) {
    if idx.is_empty() {
        return;
    }
    let (ax, az) = (pts[a].0 as f64, pts[a].1 as f64);
    let (bx, bz) = (pts[b].0 as f64, pts[b].1 as f64);
    let (dx, dz) = (bx - ax, bz - az);
    let len2 = dx * dx + dz * dz;
    let mut far = (0.0f64, 0usize);
    for (k, &i) in idx.iter().enumerate() {
        let (px, pz) = (pts[i].0 as f64 - ax, pts[i].1 as f64 - az);
        let d2 = if len2 > 0.0 {
            let t = ((px * dx + pz * dz) / len2).clamp(0.0, 1.0);
            let (qx, qz) = (px - dx * t, pz - dz * t);
            qx * qx + qz * qz
        } else {
            px * px + pz * pz
        };
        if d2 > far.0 {
            far = (d2, k);
        }
    }
    if far.0 > err2 {
        let k = far.1;
        keep[k] = true;
        let m = idx[k];
        let (left, right) = idx.split_at(k);
        let (lk, rk) = keep.split_at_mut(k);
        dp(pts, a, m, left, lk, err2);
        dp(pts, m, b, &right[1..], &mut rk[1..], err2);
    }
}

fn area2(p: &[(i32, i32)]) -> i64 {
    let n = p.len();
    (0..n)
        .map(|i| {
            let (a, b) = (p[i], p[(i + 1) % n]);
            a.0 as i64 * b.1 as i64 - b.0 as i64 * a.1 as i64
        })
        .sum()
}

/// No two non-adjacent edges touch, and no vertex repeats.
fn simple(p: &[(i32, i32)]) -> bool {
    let n = p.len();
    let q: Vec<(i64, i64)> = p.iter().map(|v| (v.0 as i64, v.1 as i64)).collect();
    for i in 0..n {
        for j in i + 1..n {
            if q[i] == q[j] {
                return false;
            }
        }
    }
    for i in 0..n {
        let (a, b) = (q[i], q[(i + 1) % n]);
        for j in i + 1..n {
            if j == i || (j + 1) % n == i || (i + 1) % n == j {
                continue;
            }
            let (c, d) = (q[j], q[(j + 1) % n]);
            if segments_touch(a, b, c, d) {
                return false;
            }
        }
    }
    true
}

fn segments_touch(a: (i64, i64), b: (i64, i64), c: (i64, i64), d: (i64, i64)) -> bool {
    let d1 = cross(c, d, a);
    let d2 = cross(c, d, b);
    let d3 = cross(a, b, c);
    let d4 = cross(a, b, d);
    if ((d1 > 0 && d2 < 0) || (d1 < 0 && d2 > 0)) && ((d3 > 0 && d4 < 0) || (d3 < 0 && d4 > 0)) {
        return true;
    }
    let on = |p: (i64, i64), q: (i64, i64), r: (i64, i64)| {
        r.0 >= p.0.min(q.0) && r.0 <= p.0.max(q.0) && r.1 >= p.1.min(q.1) && r.1 <= p.1.max(q.1)
    };
    (d1 == 0 && on(c, d, a))
        || (d2 == 0 && on(c, d, b))
        || (d3 == 0 && on(a, b, c))
        || (d4 == 0 && on(a, b, d))
}

/// Ear clipping of a simple counter-clockwise polygon: index triples.
fn triangulate(p: &[(i64, i64)]) -> Vec<[usize; 3]> {
    let mut idx: Vec<usize> = (0..p.len()).collect();
    let mut tris = Vec::new();
    let mut guard = p.len() * p.len() + 8;
    while idx.len() > 3 && guard > 0 {
        guard -= 1;
        let n = idx.len();
        let mut clipped = false;
        for k in 0..n {
            let (a, b, c) = (idx[(k + n - 1) % n], idx[k], idx[(k + 1) % n]);
            if cross(p[a], p[b], p[c]) <= 0 {
                continue;
            }
            let blocked = idx.iter().any(|&v| {
                if v == a || v == b || v == c || p[v] == p[a] || p[v] == p[b] || p[v] == p[c] {
                    return false;
                }
                cross(p[a], p[b], p[v]) >= 0
                    && cross(p[b], p[c], p[v]) >= 0
                    && cross(p[c], p[a], p[v]) >= 0
            });
            if blocked {
                continue;
            }
            tris.push([a, b, c]);
            idx.remove(k);
            clipped = true;
            break;
        }
        if !clipped {
            // Degenerate: drop the flattest vertex.
            let n = idx.len();
            let k = (0..n)
                .min_by_key(|&k| {
                    cross(p[idx[(k + n - 1) % n]], p[idx[k]], p[idx[(k + 1) % n]]).abs()
                })
                .unwrap_or(0);
            let (a, b, c) = (idx[(k + n - 1) % n], idx[k], idx[(k + 1) % n]);
            if cross(p[a], p[b], p[c]) > 0 {
                tris.push([a, b, c]);
            }
            idx.remove(k);
        }
    }
    if idx.len() == 3 && cross(p[idx[0]], p[idx[1]], p[idx[2]]) > 0 {
        tris.push([idx[0], idx[1], idx[2]]);
    }
    tris
}

/// Merges triangles into convex polygons (collinear vertices allowed),
/// longest shared edge first, at most [`MAX_VERTS`] vertices each.
fn merge(p: &[(i64, i64)], tris: Vec<[usize; 3]>) -> Vec<Vec<usize>> {
    let mut polys: Vec<Vec<usize>> = tris.into_iter().map(|t| t.to_vec()).collect();
    loop {
        let mut best: Option<(i64, usize, usize, Vec<usize>)> = None;
        for i in 0..polys.len() {
            for j in i + 1..polys.len() {
                if let Some((len, merged)) = try_merge(p, &polys[i], &polys[j])
                    && best.as_ref().is_none_or(|b| len > b.0)
                {
                    best = Some((len, i, j, merged));
                }
            }
        }
        match best {
            Some((_, i, j, merged)) => {
                polys[i] = merged;
                polys.remove(j);
            }
            None => break,
        }
    }
    polys
}

/// Two polygons sharing an edge, merged when the result is convex and small enough.
fn try_merge(p: &[(i64, i64)], a: &[usize], b: &[usize]) -> Option<(i64, Vec<usize>)> {
    let (na, nb) = (a.len(), b.len());
    if na + nb - 2 > MAX_VERTS {
        return None;
    }
    for i in 0..na {
        let (a0, a1) = (a[i], a[(i + 1) % na]);
        for j in 0..nb {
            let (b0, b1) = (b[j], b[(j + 1) % nb]);
            if !(p[a0] == p[b1] && p[a1] == p[b0]) {
                continue;
            }
            let mut merged = Vec::with_capacity(na + nb - 2);
            for k in 0..na - 1 {
                merged.push(a[(i + 1 + k) % na]);
            }
            for k in 0..nb - 1 {
                merged.push(b[(j + 1 + k) % nb]);
            }
            let m = merged.len();
            let convex = (0..m).all(|k| {
                cross(
                    p[merged[(k + m - 1) % m]],
                    p[merged[k]],
                    p[merged[(k + 1) % m]],
                ) >= 0
            });
            if !convex {
                return None;
            }
            let (dx, dz) = (p[a1].0 - p[a0].0, p[a1].1 - p[a0].1);
            return Some((dx * dx + dz * dz, merged));
        }
    }
    None
}
