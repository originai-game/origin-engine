//! Paths: A* over polygons, the funnel through their shared edges, and
//! walking the surface (`move_along`).

use super::*;

impl NavMesh {
    /// The shortest polygon corridor from `start` to `end` (A* over
    /// polygons through edge midpoints), then the funnel's corners.
    /// `extent` is the search box for the end points' polygons.
    pub fn find_path(&self, start: Vec3, end: Vec3, extent: Vec3) -> Option<MeshPath> {
        self.find_path_filtered(start, end, extent, &Filter::default())
    }

    /// [`Self::find_path`] weighing each polygon's length by its area's
    /// cost in `filter`, through no polygon of an excluded area.
    pub fn find_path_filtered(
        &self,
        start: Vec3,
        end: Vec3,
        extent: Vec3,
        filter: &Filter,
    ) -> Option<MeshPath> {
        let (sref, spt) = self.nearest(start, extent)?;
        let (eref, ept) = self.nearest(end, extent)?;
        let mut g: BTreeMap<PolyRef, (f32, PolyRef, Vec3)> = BTreeMap::new();
        let mut closed: BTreeMap<PolyRef, ()> = BTreeMap::new();
        let mut open: BinaryHeap<Reverse<(u32, u32, PolyRef)>> = BinaryHeap::new();
        let key = |x: f32| x.max(0.0).to_bits();
        // The heuristic at the cheapest area's cost, so it never overestimates
        // (1 without a filter: the distance itself).
        let least = filter.least();
        let area_of = |r: PolyRef| self.poly(r).map_or(0, |(_, p)| p.area);
        g.insert(sref, (0.0, NONE, spt));
        open.push(Reverse((key(spt.distance(ept) * least), 0, sref)));
        let mut seq = 0u32;
        let mut expanded = 0u32;
        let mut best_end = (spt.distance(ept), sref);
        // Polygons last reached across a link: (link, reversed).
        let links = !self.link_from.is_empty();
        let mut via: BTreeMap<PolyRef, (u32, bool)> = BTreeMap::new();
        let mut found = false;
        while let Some(Reverse((_, _, cur))) = open.pop() {
            if closed.insert(cur, ()).is_some() {
                continue;
            }
            expanded += 1;
            if cur == eref {
                found = true;
                break;
            }
            let (gc, _, pos) = g[&cur];
            let h = pos.distance(ept);
            if h < best_end.0 {
                best_end = (h, cur);
            }
            let (_, poly) = self.poly(cur).expect("a live polygon");
            let here = filter.cost(poly.area);
            for (e, &nb) in poly.links.iter().enumerate() {
                if nb == NONE || closed.contains_key(&nb) {
                    continue;
                }
                let there = area_of(nb);
                if filter.excludes(there) {
                    continue;
                }
                let (a, b) = self.edge(cur, e);
                let mid = (a + b) * 0.5;
                let ng = gc
                    + pos.distance(mid) * here
                    + if nb == eref {
                        mid.distance(ept) * filter.cost(there)
                    } else {
                        0.0
                    };
                if g.get(&nb).is_none_or(|old| ng < old.0) {
                    g.insert(nb, (ng, cur, mid));
                    if links {
                        via.remove(&nb);
                    }
                    seq += 1;
                    open.push(Reverse((key(ng + mid.distance(ept) * least), seq, nb)));
                }
            }
            if links && let Some(out) = self.link_from.get(&cur) {
                for &(id, reverse) in out {
                    let l = &self.links[&id];
                    let (s, e) = self.link_ends[&id];
                    let nb = if reverse { s } else { e };
                    let there = area_of(nb);
                    if !l.open || nb == cur || closed.contains_key(&nb) || filter.excludes(there) {
                        continue;
                    }
                    let (a, b) = if reverse {
                        (l.end, l.start)
                    } else {
                        (l.start, l.end)
                    };
                    let ng = gc
                        + pos.distance(a) * here
                        + l.path_cost()
                        + if nb == eref {
                            b.distance(ept) * filter.cost(there)
                        } else {
                            0.0
                        };
                    if g.get(&nb).is_none_or(|old| ng < old.0) {
                        g.insert(nb, (ng, cur, b));
                        via.insert(nb, (id, reverse));
                        seq += 1;
                        open.push(Reverse((key(ng + b.distance(ept) * least), seq, nb)));
                    }
                }
            }
        }
        let last = if found { eref } else { best_end.1 };
        let mut polys = vec![last];
        let mut at = last;
        while g[&at].1 != NONE {
            at = g[&at].1;
            polys.push(at);
        }
        polys.reverse();
        let goal = if found {
            ept
        } else {
            let (tile, poly) = self.poly(last).expect("a live polygon");
            Self::closest_on(tile, poly, ept)
        };
        let (points, crossed) = if via.is_empty() {
            (self.funnel(&polys, spt, goal), Vec::new())
        } else {
            // A stretch of surface per link, funnelled on its own.
            let mut points = Vec::new();
            let mut crossed = Vec::new();
            let (mut first, mut from) = (0usize, spt);
            for i in 1..polys.len() {
                let Some(&(id, reverse)) = via.get(&polys[i]) else {
                    continue;
                };
                let l = &self.links[&id];
                let (a, b) = if reverse {
                    (l.end, l.start)
                } else {
                    (l.start, l.end)
                };
                points.extend(self.funnel(&polys[first..i], from, a));
                crossed.push(PathLink {
                    corner: points.len() as u32 - 1,
                    link: id,
                    reverse,
                });
                (first, from) = (i, b);
            }
            points.extend(self.funnel(&polys[first..], from, goal));
            (points, crossed)
        };
        Some(MeshPath {
            points,
            links: crossed,
            polys,
            expanded,
            complete: found,
        })
    }

    /// The portal (left, right) from polygon `a` into its neighbour `b`.
    fn portal(&self, a: PolyRef, b: PolyRef) -> Option<(Vec3, Vec3)> {
        let (_, poly) = self.poly(a)?;
        let n = poly.links.len();
        // The run of edges into `b` (one run: the polygon is convex), from
        // its first edge's start to its last edge's end.
        let first = (0..n)
            .find(|&e| poly.links[e] == b && poly.links[(e + n - 1) % n] != b)
            .or_else(|| poly.links.iter().position(|&l| l == b))?;
        let mut last = first;
        while poly.links[(last + 1) % n] == b && (last + 1) % n != first {
            last = (last + 1) % n;
        }
        let (p, _) = self.edge(a, first);
        let (_, q) = self.edge(a, last);
        // Counter-clockwise polygons: walking out through p → q, q is on the left.
        Some((q, p))
    }

    /// The simple stupid funnel algorithm over the corridor's portals.
    pub fn funnel(&self, polys: &[PolyRef], start: Vec3, end: Vec3) -> Vec<Vec3> {
        let mut portals: Vec<(Vec3, Vec3)> = vec![(start, start)];
        for w in polys.windows(2) {
            if let Some(p) = self.portal(w[0], w[1]) {
                portals.push(p);
            }
        }
        portals.push((end, end));
        let tri = |a: Vec3, b: Vec3, c: Vec3| -> f32 {
            let (ax, az) = (b.x - a.x, b.z - a.z);
            let (bx, bz) = (c.x - a.x, c.z - a.z);
            bx * az - ax * bz
        };
        let same = |a: Vec3, b: Vec3| (a.x - b.x).abs() < 1e-6 && (a.z - b.z).abs() < 1e-6;
        let mut path = vec![start];
        let (mut apex, mut left, mut right) = (start, portals[0].0, portals[0].1);
        let (mut left_i, mut right_i) = (0usize, 0usize);
        let mut i = 1;
        let mut guard = portals.len() * 4 + 8;
        while i < portals.len() {
            guard -= 1;
            if guard == 0 {
                break;
            }
            let (pl, pr) = portals[i];
            if tri(apex, right, pr) <= 0.0 {
                if same(apex, right) || tri(apex, left, pr) >= 0.0 {
                    right = pr;
                    right_i = i;
                } else {
                    apex = left;
                    let apex_i = left_i;
                    if !same(*path.last().unwrap(), apex) {
                        path.push(apex);
                    }
                    right = apex;
                    right_i = apex_i;
                    i = apex_i + 1;
                    continue;
                }
            }
            if tri(apex, left, pl) >= 0.0 {
                if same(apex, left) || tri(apex, right, pl) <= 0.0 {
                    left = pl;
                    left_i = i;
                } else {
                    apex = right;
                    let apex_i = right_i;
                    if !same(*path.last().unwrap(), apex) {
                        path.push(apex);
                    }
                    left = apex;
                    left_i = apex_i;
                    i = apex_i + 1;
                    continue;
                }
            }
            i += 1;
        }
        if !same(*path.last().unwrap(), end) || path.len() == 1 {
            path.push(end);
        }
        path
    }

    /// Moves from `start` (on polygon `from`) towards `end` along the
    /// surface: across linked edges into neighbours, sliding along walls.
    /// The polygon reached and the point, on its surface.
    pub fn move_along(&self, from: PolyRef, start: Vec3, end: Vec3) -> (PolyRef, Vec3) {
        let mut cur = from;
        let mut a = Self::xz(start);
        let mut b = Self::xz(end);
        let mut slid = false;
        for _ in 0..24 {
            let Some((tile, poly)) = self.poly(cur) else {
                break;
            };
            if Self::inside(tile, poly, b) {
                let y = Self::height_at(tile, poly, b);
                return (cur, Vec3::new(b.x, y, b.y));
            }
            // The edge the segment leaves through first.
            let n = poly.verts.len();
            let d = b - a;
            // Ranked by: the hit lies on the edge (collinear edges share a
            // line, only one holds the point), then the smallest t.
            let mut exit: Option<(bool, f32, usize)> = None;
            for e in 0..n {
                let p = Self::xz(tile.verts[poly.verts[e] as usize]);
                let q = Self::xz(tile.verts[poly.verts[(e + 1) % n] as usize]);
                let edge = q - p;
                let side_b = edge.perp_dot(b - p);
                if side_b >= 0.0 {
                    continue;
                }
                let den = edge.perp_dot(d);
                let t = if den.abs() > 1e-12 {
                    -edge.perp_dot(a - p) / den
                } else {
                    0.0
                };
                let t = t.clamp(0.0, 1.0);
                let hit = a + d * t;
                let len2 = edge.length_squared();
                let s = if len2 > 0.0 {
                    (hit - p).dot(edge) / len2
                } else {
                    0.0
                };
                let on = (-1e-4..=1.0 + 1e-4).contains(&s);
                let better = match exit {
                    None => true,
                    Some((bon, bt, _)) => (on && !bon) || (on == bon && t < bt),
                };
                if better {
                    exit = Some((on, t, e));
                }
            }
            let exit = exit.map(|(_, t, e)| (t, e));
            let Some((t, e)) = exit else { break };
            let hit = a + d * t;
            let link = poly.links[e];
            if link != NONE {
                cur = link;
                a = hit;
                continue;
            }
            if slid {
                let y = Self::height_at(tile, poly, hit);
                return (cur, Vec3::new(hit.x, y, hit.y));
            }
            // A wall: keep the motion along it.
            let p = Self::xz(tile.verts[poly.verts[e] as usize]);
            let q = Self::xz(tile.verts[poly.verts[(e + 1) % n] as usize]);
            let along = (q - p).normalize_or_zero();
            let rest = (b - hit).dot(along);
            // Nudge inside by a hair so the next test starts in this polygon.
            let inward = Vec2::new(-along.y, along.x) * 1e-4;
            a = hit + inward;
            b = hit + along * rest + inward;
            slid = true;
        }
        let (tile, poly) = self.poly(cur).expect("a live polygon");
        let c = Self::closest_on(tile, poly, Vec3::new(a.x, start.y, a.y));
        (cur, c)
    }
}
