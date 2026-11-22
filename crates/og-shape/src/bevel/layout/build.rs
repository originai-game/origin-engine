//! The build: offsets, boundary points, profiles and strips placed for each bevelled edge and vertex.

use super::*;

pub(crate) struct Build<'a> {
    pub(crate) s: &'a Solid,
    pub(crate) topo: Topology,
    pub(crate) edges: BTreeMap<EdgeKey, EdgeInfo>,
    pub(crate) pos: Vec<DVec3>,
    /// (triangle, vertex) → what replaces the vertex in that triangle,
    /// from the side of the corner after it to the side before it.
    pub(crate) repl: BTreeMap<(usize, u32), Vec<u32>>,
    /// (vertex, other end) → the edge end's chain, after → before.
    pub(crate) chains: BTreeMap<(u32, u32), Vec<u32>>,
    /// New triangles and the unit (strip or patch) each belongs to.
    pub(crate) new_tris: Vec<([u32; 3], usize)>,
    pub(crate) units: usize,
    /// Units to make one surface.
    pub(crate) joins: Vec<(usize, usize)>,
    /// Pairs of edges whose strips meet smoothly.
    pub(crate) smooth: Vec<(EdgeKey, EdgeKey)>,
    /// Patch unit per vertex, with the edges around it.
    pub(crate) patches: Vec<(usize, Vec<EdgeKey>)>,
    pub(crate) samples: BTreeMap<u32, Vec<(f64, f64)>>,
    pub(crate) n: u32,
    pub(crate) exponent: f64,
    pub(crate) kind: Kind,
    pub(crate) miters: Miters,
    /// The cut bands of a wide bevel (`wide.rs`); `None` for every other.
    pub(crate) wide: Option<&'a wide::Wide>,
    /// How far a replaced corner may leave its triangle's plane, as a
    /// fraction of the largest offset.
    pub(crate) tilt: f64,
    /// Whether a fold (two triangles of one surface not in one plane)
    /// counts as a real edge (`docs/research.md` §7.8).
    pub(crate) folds: bool,
    /// Per point of a run of edges the offset line swallows on a face
    /// (`swallowed_runs`, `docs/research.md` §37), the run's one point.
    pub(crate) poles: BTreeMap<u32, u32>,
    /// The checked input's triangles, for the final check (`check.rs`).
    pub(crate) checked: Option<std::sync::Arc<Vec<[DVec3; 3]>>>,
}

impl Build<'_> {
    /// Whether spoke `i` of the fan is a real edge: the surface changes
    /// across it, or, with `folds`, its two triangles are not one plane.
    pub(crate) fn real(&self, f: &Fan, i: usize) -> bool {
        let k = f.tris.len();
        let (x, y) = (f.tris[(i + k - 1) % k], f.tris[i]);
        self.s.surfaces[x] != self.s.surfaces[y]
            || self.folds && tri_normal(self.s, x).dot(tri_normal(self.s, y)) <= PLANAR_DOT
    }

    /// Where a wide bevel's band was cut: the vertex the gap point is, when
    /// the point slides from `v` along spoke `i` and that spoke ends on the
    /// cut of the bevelled edge `key` (`docs/research.md` §7.6).
    pub(crate) fn snap(&self, f: &Fan, i: usize, key: EdgeKey) -> Option<u32> {
        let w = self.wide?;
        let end = f.spokes[i % f.spokes.len()];
        w.cuts.get(&key).filter(|c| c.contains(&end)).map(|_| end)
    }

    /// A wide bevel's profiles between the end chains of `key` (`cu` at
    /// its low end, `cw` at its high end, both from face `a` to face `b`):
    /// one at each cut point between, its other end on the straight side
    /// between the end chains' points there. Also the points to put on
    /// that side's face triangle, after the corner they follow.
    #[allow(clippy::type_complexity)]
    pub(crate) fn stations(
        &mut self,
        key: EdgeKey,
        st: &(bool, Vec<u32>),
        cu: &[u32],
        cw: &[u32],
    ) -> Result<(Vec<Vec<u32>>, ((usize, u32), Vec<u32>)), BevelError> {
        let (lo, hi) = key;
        let (pl, ph) = (self.s.positions[lo as usize], self.s.positions[hi as usize]);
        let &(side_a, ref mids) = st;
        let (u_other, w_other) = if side_a {
            (cu[cu.len() - 1], cw[cw.len() - 1])
        } else {
            (cu[0], cw[0])
        };
        let (bu, bw) = (self.pos[u_other as usize], self.pos[w_other as usize]);
        let mut profiles = Vec::new();
        let mut others = Vec::new();
        let mut last = 0.0;
        let along = self.wide.and_then(|w| w.along.get(&key));
        for (j, &c) in mids.iter().enumerate() {
            let t = match along {
                Some(a) => a[j],
                None => (self.pos[c as usize] - pl).dot(ph - pl) / (ph - pl).length_squared(),
            };
            if !(t > last && t < 1.0) {
                return Err(BevelError::TooLarge(pl));
            }
            last = t;
            let other = self.push(bu + (bw - bu) * t);
            let corner = pl + (ph - pl) * t;
            profiles.push(if side_a {
                self.chain(c, other, corner)
            } else {
                self.chain(other, c, corner)
            });
            others.push(other);
        }
        // The other side's face triangle holds the edge the other way
        // round; its corner before the points is `hi` for face `b`.
        let at = if side_a {
            others.reverse();
            (self.topo.dir[&(hi, lo)], hi)
        } else {
            (self.topo.dir[&(lo, hi)], lo)
        };
        Ok((profiles, (at, others)))
    }

    /// The point slid along unbevelled spoke `i`: the cut vertex there, or
    /// a new one at `p`.
    pub(crate) fn slid(&mut self, f: &Fan, i: usize, key: EdgeKey, p: DVec3) -> u32 {
        match self.snap(f, i, key) {
            Some(id) => id,
            None => self.push(p),
        }
    }

    /// Runs of bevelled edges that the offset line on a face swallows,
    /// each shrunk to a point (`docs/research.md` §37).
    ///
    /// On a face, a vertex between two bevelled edges gets the point where
    /// their two offset lines meet. Round a corner no wider than the
    /// offset (a round corner of the fillet's own radius, as a polygon)
    /// those points come out in the opposite order: the edges between
    /// them have no room, the event an inset calls an edge collapse. Here
    /// every vertex of such a run takes one point instead, where the
    /// offset lines of the two edges beside the run meet, and its profile
    /// is drawn again to that point. The strips along the run then end in
    /// it, as a fan, and the faces on the run's other side are untouched.
    ///
    /// A run that ends at an unbevelled edge, goes all the way round, or
    /// whose neighbours' lines do not meet is left as it is (the layout
    /// then fails as before).
    pub(crate) fn swallowed_runs(&mut self) {
        let s = self.s;
        // The one point that replaces `v` in the triangle on the left of
        // the directed edge `x → y`, when it is one.
        let rail = |b: &Self, x: u32, y: u32, v: u32| -> Option<u32> {
            let t = *b.topo.dir.get(&(x, y))?;
            match b.repl.get(&(t, v)).map(Vec::as_slice) {
                Some([one]) => Some(*one),
                _ => None,
            }
        };
        // Per surface: swallowed directed edges, by their start.
        let mut next: BTreeMap<(u32, u32), u32> = BTreeMap::new();
        let mut bevelled: BTreeMap<(u32, u32), u32> = BTreeMap::new();
        for &(lo, hi) in self.edges.keys() {
            for (x, y) in [(lo, hi), (hi, lo)] {
                let Some(&t) = self.topo.dir.get(&(x, y)) else {
                    continue;
                };
                let face = s.surfaces[t];
                bevelled.insert((face, x), y);
                let (Some(rx), Some(ry)) = (rail(self, x, y, x), rail(self, x, y, y)) else {
                    continue;
                };
                let along = s.positions[y as usize] - s.positions[x as usize];
                if (self.pos[ry as usize] - self.pos[rx as usize]).dot(along) <= 0.0 {
                    next.insert((face, x), y);
                }
            }
        }
        // The bevelled edge of `face` that ends at `v`.
        let before = |face: u32, v: u32| {
            bevelled
                .iter()
                .find(|&(&(f, _), &y)| f == face && y == v)
                .map(|(&(_, x), _)| x)
        };
        let ends: BTreeSet<(u32, u32)> = next.iter().map(|(&(f, _), &y)| (f, y)).collect();
        let starts: Vec<(u32, u32)> = next.keys().copied().filter(|k| !ends.contains(k)).collect();
        let mut moves: Vec<(u32, DVec3)> = Vec::new();
        let mut redraw: BTreeSet<u32> = BTreeSet::new();
        for (face, a) in starts {
            let mut run = vec![a];
            let mut at = a;
            while let Some(&y) = next.get(&(face, at)) {
                run.push(y);
                at = y;
            }
            let b = at;
            let (Some(u), Some(&w)) = (before(face, a), bevelled.get(&(face, b))) else {
                continue;
            };
            if next.contains_key(&(face, u)) {
                continue;
            }
            let p = |v: u32| s.positions[v as usize];
            // Only along one smooth wall: every edge from the one before
            // the run to the one after it has the same face across it (a
            // round corner in chords). Two walls meeting at a real corner
            // whose offsets meet have swallowed the face itself, which is
            // too large and stays so.
            let across = |x: u32, y: u32| self.topo.dir.get(&(y, x)).map(|&t| s.surfaces[t]);
            let wall = across(u, a);
            let mut path = vec![u];
            path.extend(&run);
            path.push(w);
            if wall.is_none() || path.windows(2).any(|e| across(e[0], e[1]) != wall) {
                continue;
            }
            // And only round a corner that bulges out of the face (the
            // boundary turns the same way at every vertex of the run, as
            // the face's own outline does): there the offset shrinks the
            // run. Along a hollow (a hole's rim seen from the face round
            // it) the offset lengthens the edges, and a piece swallowed
            // there is a short piece among long ones, the wide bevel's.
            let Some(&t) = self.topo.dir.get(&(a, run[1])) else {
                continue;
            };
            let n = tri_normal(s, t);
            let bulges = path.windows(3).all(|e| {
                let (d0, d1) = (p(e[1]) - p(e[0]), p(e[2]) - p(e[1]));
                // A real turn, not the rounding of points in a line.
                d0.cross(d1).dot(n) > 1e-6 * d0.length() * d1.length()
            });
            if !bulges {
                continue;
            }
            // Pieces shorter than a billionth of the offset are a boolean's
            // leftovers, welded by the wide bevel's first pass instead.
            let largest = self
                .edges
                .values()
                .flat_map(|e| e.off)
                .fold(0.0f64, f64::max);
            let length: f64 = run
                .windows(2)
                .map(|e| (s.positions[e[1] as usize] - s.positions[e[0] as usize]).length())
                .sum();
            if length <= 1e-9 * largest {
                continue;
            }
            // The offset lines of the edges beside the run: each through
            // its end's own point, along the edge.
            let (Some(ra), Some(rb)) = (rail(self, u, a, a), rail(self, b, w, b)) else {
                continue;
            };
            let (p1, d1) = (self.pos[ra as usize], p(a) - p(u));
            let (p2, d2) = (self.pos[rb as usize], p(w) - p(b));
            let (aa, bb, cc) = (d1.dot(d1), d1.dot(d2), d2.dot(d2));
            let det = aa * cc - bb * bb;
            if det <= 1e-12 * aa * cc {
                continue;
            }
            let gap = p1 - p2;
            let k = (bb * d2.dot(gap) - cc * d1.dot(gap)) / det;
            let pole = p1 + d1 * k;
            let mut ids = Vec::new();
            for (i, &v) in run.iter().enumerate() {
                let (x, y) = if i + 1 < run.len() {
                    (v, run[i + 1])
                } else {
                    (run[i - 1], v)
                };
                match rail(self, x, y, v) {
                    Some(id) => ids.push(id),
                    None => {
                        ids.clear();
                        break;
                    }
                }
            }
            if ids.is_empty() {
                continue;
            }
            let first = ids.iter().copied().min().unwrap_or(0);
            for &id in &ids {
                self.poles.insert(id, first);
            }
            moves.extend(ids.into_iter().map(|id| (id, pole)));
            redraw.extend(run);
        }
        for (id, pole) in moves {
            self.pos[id as usize] = pole;
        }
        // The profiles at the run's vertices, drawn again between their
        // ends as they now stand.
        let samples = self.samples(self.n);
        for v in redraw {
            let Some(chain) = self
                .chains
                .iter()
                .find(|(k, _)| k.0 == v)
                .map(|(_, c)| c.clone())
            else {
                continue;
            };
            if chain.len() != samples.len() {
                continue;
            }
            let (a, b) = (chain[0], chain[chain.len() - 1]);
            let pts = profile::chain(
                self.pos[a as usize],
                self.pos[b as usize],
                s.positions[v as usize],
                &samples,
            );
            for (k, &id) in chain.iter().enumerate() {
                if k != 0 && k + 1 != chain.len() {
                    self.pos[id as usize] = pts[k];
                }
            }
        }
    }

    pub(crate) fn push(&mut self, p: DVec3) -> u32 {
        self.pos.push(p);
        self.pos.len() as u32 - 1
    }

    pub(crate) fn samples(&mut self, n: u32) -> Vec<(f64, f64)> {
        let p = self.exponent;
        self.samples
            .entry(n)
            .or_insert_with(|| profile::samples(p, n))
            .clone()
    }

    pub(crate) fn unit(&mut self) -> usize {
        self.units += 1;
        self.units - 1
    }

    /// A chain of `n` segments from the existing points `a` to `b` around
    /// `c`, its inner points new.
    pub(crate) fn chain(&mut self, a: u32, b: u32, c: DVec3) -> Vec<u32> {
        let s = self.samples(self.n);
        let pts = profile::chain(self.pos[a as usize], self.pos[b as usize], c, &s);
        let last = pts.len() - 1;
        pts.iter()
            .enumerate()
            .map(|(k, &p)| match k {
                0 => a,
                k if k == last => b,
                _ => self.push(p),
            })
            .collect()
    }

    /// The boundary point of the wedge between spokes `lo` and `hi` (fan
    /// indices, `hi` may be `k` for spoke 0), each edge's offset line at
    /// `dl` and `dr` (0 for an unbevelled edge, whose line the point then
    /// lies on).
    pub(crate) fn wedge_point(
        &self,
        v: u32,
        f: &Fan,
        lo: usize,
        hi: usize,
        dl: f64,
        dr: f64,
    ) -> Result<DVec3, BevelError> {
        let k = f.spokes.len();
        let p = self.s.positions[v as usize];
        let dir = |i: usize| (self.s.positions[f.spokes[i % k] as usize] - p).normalize_or_zero();
        let (l, r) = (dir(lo), dir(hi));
        let normals: Vec<DVec3> = (lo..hi).map(|i| tri_normal(self.s, f.tris[i])).collect();
        let planar = normals.len() == 1 || normals.iter().all(|n| n.dot(normals[0]) > PLANAR_DOT);
        let out = if planar {
            let nrm = normals
                .iter()
                .fold(DVec3::ZERO, |a, &b| a + b)
                .normalize_or_zero();
            let e2 = nrm.cross(l).normalize_or_zero();
            let (rx, ry) = (r.dot(l), r.dot(e2));
            if ry.abs() < COLLINEAR_SIN {
                if rx > 0.0 {
                    // A corner of zero angle: the offset lines are
                    // parallel on either side of one ray and meet nowhere,
                    // the limit of a corner too sharp for its offsets.
                    return Err(BevelError::TooLarge(p));
                }
                p + e2 * ((dl + dr) / 2.0)
            } else {
                p + l * ((dr + rx * dl) / ry) + e2 * dl
            }
        } else {
            let sin = |a: DVec3, b: DVec3| a.cross(b).length();
            let in_line = sin(l, r) < COLLINEAR_SIN;
            if in_line && l.dot(r) > 0.0 {
                // Zero angle, as on a plane.
                return Err(BevelError::TooLarge(p));
            }
            if (dl == 0.0 || dr == 0.0) && !in_line {
                // Slide along the unbevelled edge.
                let (along, d) = if dl == 0.0 { (l, dr) } else { (r, dl) };
                p + along * (d / sin(l, r))
            } else {
                // Slide along the inner edge most across the two edges,
                // until the point's distances from them sum to the two
                // offsets.
                // Also where an unbevelled edge is in line with the
                // bevelled one, whose offset line it never meets: the
                // point then steps straight across by half the offset, as
                // collinear edges on a plane do (`docs/research.md` §7.5).
                let mut best: Option<(f64, DVec3)> = None;
                for i in lo + 1..hi {
                    let e = dir(i);
                    let w = sin(e, l) + sin(e, r);
                    if best.is_none_or(|(b, _)| w > b) {
                        best = Some((w, e));
                    }
                }
                let (w, e) = best.ok_or(BevelError::Unsupported(p, "a curved face corner"))?;
                if w < COLLINEAR_SIN {
                    return Err(BevelError::TooLarge(p));
                }
                p + e * ((dl + dr) / w)
            }
        };
        if out.is_finite() {
            Ok(out)
        } else {
            Err(BevelError::TooLarge(p))
        }
    }

    pub(crate) fn vertex(&mut self, v: u32) -> Result<(), BevelError> {
        if self.needs_general(v)? {
            return self.general(v);
        }
        let s = self.s;
        let p = s.positions[v as usize];
        let mut f = fan(s, &self.topo, v)?;
        let k = f.spokes.len();
        let bevelled = |f: &Fan, i: usize| {
            let key = edge_key(v, f.spokes[i % k]);
            self.edges.contains_key(&key)
        };
        let r0 = (0..k).find(|&i| bevelled(&f, i)).unwrap_or(0);
        f.tris.rotate_left(r0);
        f.spokes.rotate_left(r0);
        let real: Vec<bool> = (0..k).map(|i| self.real(&f, i)).collect();
        let bidx: Vec<usize> = (0..k).filter(|&i| bevelled(&f, i)).collect();
        let m = bidx.len();
        let key_of = |i: usize| edge_key(v, f.spokes[i % k]);
        let mut gap_point: Vec<u32> = Vec::with_capacity(m);
        let mut gap_plain: Vec<bool> = Vec::with_capacity(m);
        for g in 0..m {
            let (lo, hi) = (bidx[g], if g + 1 < m { bidx[g + 1] } else { k });
            let inner: Vec<usize> = (lo + 1..hi).filter(|&i| real[i]).collect();
            let (el, er) = (key_of(lo), key_of(hi));
            let dl = self.edges[&el].after(el, v);
            let dr = self.edges[&er].before(er, v);
            gap_plain.push(inner.is_empty());
            // A wall cut across its rows has its gap point already
            // (`rows.rs`).
            let wall = self.wide.and_then(|w| {
                f.tris[lo..hi]
                    .iter()
                    .find_map(|&t| w.gaps.get(&(v, s.surfaces[t])))
            });
            if let Some(&id) = wall
                && m >= 2
            {
                gap_point.push(id);
                for &t in &f.tris[lo..hi] {
                    self.repl.insert((t, v), vec![id]);
                }
                continue;
            }
            match (m, inner.len()) {
                (2.., 0) => {
                    let b = self.wedge_point(v, &f, lo, hi, dl, dr)?;
                    let id = self.push(b);
                    gap_point.push(id);
                    for &t in &f.tris[lo..hi] {
                        self.repl.insert((t, v), vec![id]);
                    }
                }
                (2.., 1) => {
                    let u = inner[0];
                    let p1 = self.wedge_point(v, &f, lo, u, dl, 0.0)?;
                    let p2 = self.wedge_point(v, &f, u, hi, 0.0, dr)?;
                    let id = match (self.snap(&f, u, el), self.snap(&f, u, er)) {
                        (Some(a), _) | (None, Some(a)) => a,
                        (None, None) => self.push((p1 + p2) / 2.0),
                    };
                    gap_point.push(id);
                    for &t in &f.tris[lo..hi] {
                        self.repl.insert((t, v), vec![id]);
                    }
                }
                (1, 2) => {
                    // A terminal end: the profile lies on the end face.
                    let (u1, u2) = (inner[0], inner[1]);
                    let a = self.wedge_point(v, &f, lo, u1, dl, 0.0)?;
                    let b = self.wedge_point(v, &f, u2, hi, 0.0, dr)?;
                    let ia = self.slid(&f, u1, el, a);
                    let ib = self.slid(&f, u2, er, b);
                    for &t in &f.tris[lo..u1] {
                        self.repl.insert((t, v), vec![ia]);
                    }
                    for &t in &f.tris[u2..hi] {
                        self.repl.insert((t, v), vec![ib]);
                    }
                    let chain = self.chain(ia, ib, p);
                    // Share the chain among the end face's triangles by
                    // the direction of their spokes.
                    let mut split = vec![0usize];
                    for j in u1 + 1..u2 {
                        let d = (s.positions[f.spokes[j] as usize] - p).normalize_or_zero();
                        let from = *split.last().unwrap_or(&0);
                        let mut best = (f64::NEG_INFINITY, from);
                        for (c, &id) in chain.iter().enumerate().skip(from) {
                            let cos = (self.pos[id as usize] - p).normalize_or_zero().dot(d);
                            if cos > best.0 {
                                best = (cos, c);
                            }
                        }
                        split.push(best.1);
                    }
                    split.push(chain.len() - 1);
                    for (w, i) in (u1..u2).enumerate() {
                        let mut seq = chain[split[w]..=split[w + 1]].to_vec();
                        seq.reverse();
                        self.repl.insert((f.tris[i], v), seq);
                    }
                    self.chains.insert((v, f.spokes[bidx[0]]), chain);
                    return Ok(());
                }
                _ => {
                    return Err(BevelError::Unsupported(
                        p,
                        "more than one unbevelled edge between two bevelled ones, or a bevelled edge that ends among smooth faces",
                    ));
                }
            }
        }
        if m == 2 {
            let chain = self.chain(gap_point[0], gap_point[1], p);
            let (e0, e1) = (f.spokes[bidx[0]], f.spokes[bidx[1]]);
            let mut rev = chain.clone();
            rev.reverse();
            self.chains.insert((v, e0), chain);
            self.chains.insert((v, e1), rev);
            if gap_plain[0] && gap_plain[1] {
                self.smooth.push((edge_key(v, e0), edge_key(v, e1)));
            }
            return Ok(());
        }
        // Three or more: chains B_j → B_{j-1} around a corner patch.
        let b: Vec<DVec3> = gap_point.iter().map(|&i| self.pos[i as usize]).collect();
        let o = if m == 3 {
            p + b.iter().fold(DVec3::ZERO, |a, &q| a + (q - p)) / 2.0
        } else {
            DVec3::ZERO
        };
        let mut ends = Vec::with_capacity(m);
        let mut keys = Vec::with_capacity(m);
        for j in 0..m {
            let (ia, ib) = (gap_point[j], gap_point[(j + m - 1) % m]);
            let (a, bb) = (self.pos[ia as usize], self.pos[ib as usize]);
            let c = if m == 3 {
                a + bb - o
            } else {
                let l = (s.positions[f.spokes[bidx[j]] as usize] - p).normalize_or_zero();
                p + l * (((a + bb) / 2.0 - p).dot(l))
            };
            let chain = self.chain(ia, ib, c);
            self.chains.insert((v, f.spokes[bidx[j]]), chain.clone());
            ends.push((a, bb, c, chain));
            keys.push(edge_key(v, f.spokes[bidx[j]]));
        }
        let unit = self.unit();
        let corner = Corner {
            v: p,
            o,
            ends,
            n: self.n,
            octant: true,
        };
        let exponent = self.exponent;
        let mut samples = |n: u32| profile::samples(exponent, n);
        let fill = patch::fill(&corner, exponent, &mut samples, &mut self.pos)?;
        for t in fill {
            self.new_tris.push((t, unit));
        }
        self.patches.push((unit, keys));
        Ok(())
    }
}
