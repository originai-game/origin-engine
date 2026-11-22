//! The collapse loop: candidates from the queue until the target, each checked and applied.

use super::*;

pub(crate) fn collapse(
    s: &Solid,
    target: usize,
    d: &Decimate,
    max_len: f64,
) -> Result<(Solid, DecimateReport), DecimateError> {
    if s.positions.iter().any(|p| !p.is_finite()) {
        return Err(DecimateError::NotFinite);
    }
    let n = s.triangles.len();
    let mut report = DecimateReport {
        triangles_before: n,
        triangles: n,
        target,
        error: 0.0,
        folds_refused: 0,
        blocked: Blocked::default(),
        uv_islands: 0,
        uv_charts: 0,
    };
    if n <= target && max_len == 0.0 {
        return Ok((s.clone(), report));
    }
    let nv = s.positions.len();
    let mut m = Mesh {
        pos: s.positions.clone(),
        tris: s.triangles.clone(),
        alive: vec![true; n],
        live: n,
        around: vec![Vec::new(); nv],
        quadric: vec![Quadric::default(); nv],
        kind: vec![Kind::Interior; nv],
        lines: vec![Vec::new(); nv],
        stamp: vec![0; nv],
        into: (0..nv as u32).collect(),
        max_len,
        weights: s.skin.as_ref().map(|k| {
            let mut w = k.weights.clone();
            w.resize(nv, Weights::default());
            w
        }),
        morphs: s.morphs.as_ref().map(|m| {
            let mut m = m.clone();
            m.resize(nv);
            m
        }),
        keep_borders: !d.cross_bone_borders,
        bend: {
            let (lo, hi) = s.positions.iter().fold(
                (DVec3::splat(f64::INFINITY), DVec3::splat(f64::NEG_INFINITY)),
                |(l, h), p| (l.min(*p), h.max(*p)),
            );
            if s.positions.is_empty() {
                0.0
            } else {
                (hi - lo).length() * BEND_FRACTION
            }
        },
        uvs: s.uvs.clone(),
        charts: None,
        colors: s.colors.clone(),
        cross_colors: d.cross_color_borders,
    };
    if s.uvs.is_some() {
        let tol = d.merge_charts;
        if let Some((chart, frame, islands)) = merge_charts(s, tol) {
            report.uv_islands = islands;
            let mut roots: Vec<u32> = chart.clone();
            roots.sort_unstable();
            roots.dedup();
            report.uv_charts = if tol > 0.0 { roots.len() } else { islands };
            if tol > 0.0 {
                m.charts = Some((chart, frame, tol));
            }
        }
    }
    let mut users: BTreeMap<EdgeKey, Vec<u32>> = BTreeMap::new();
    let mut total_area = 0.0;
    for (i, t) in s.triangles.iter().enumerate() {
        let [a, b, c] = t.map(|v| s.positions[v as usize]);
        let cr = (b - a).cross(c - a);
        let area = cr.length() / 2.0;
        total_area += area;
        let nrm = if area > 0.0 {
            cr / (2.0 * area)
        } else {
            DVec3::ZERO
        };
        let q = Quadric::plane(nrm, a, area);
        for &v in t {
            m.around[v as usize].push(i as u32);
            m.quadric[v as usize].add(&q);
        }
        for k in 0..3 {
            users
                .entry(edge_key(t[k], t[(k + 1) % 3]))
                .or_default()
                .push(i as u32);
        }
    }
    let weight = 1000.0 * (total_area / n as f64).max(f64::MIN_POSITIVE);
    // Borders and features: lines, with heavy quadrics.
    let mut pinned = vec![false; nv];
    let mut border = vec![false; nv];
    for (&(a, b), ts) in &users {
        let named = s.edge_tags.contains_key(&(a, b));
        let feature = ts.len() == 2 && {
            let (x, y) = (ts[0] as usize, ts[1] as usize);
            s.surfaces[x] != s.surfaces[y] || s.face_tags[x] != s.face_tags[y] || {
                // A UV seam: either end has different UVs on the two sides,
                // unless the two sides are one merged chart and agree in
                // its frame.
                s.uvs.as_ref().is_some_and(|u| {
                    let at = |t: usize, v: u32| {
                        let k = s.triangles[t].iter().position(|&w| w == v).unwrap_or(0);
                        u[t][k]
                    };
                    let differ = at(x, a) != at(y, a) || at(x, b) != at(y, b);
                    differ
                        && !m.charts.as_ref().is_some_and(|(chart, frame, tol)| {
                            chart[x] == chart[y]
                                && [a, b].iter().all(|&v| {
                                    (frame[x].apply(at(x, v)) - frame[y].apply(at(y, v))).length()
                                        <= *tol
                                })
                        })
                }) || !d.cross_color_borders
                    && s.colors.as_ref().is_some_and(|c| {
                        // A hard colour border, likewise; a merged chart does
                        // not join colours.
                        let at = |t: usize, v: u32| {
                            let k = s.triangles[t].iter().position(|&w| w == v).unwrap_or(0);
                            c[t][k]
                        };
                        at(x, a) != at(y, a) || at(x, b) != at(y, b)
                    })
            }
        };
        let open = ts.len() == 1;
        if ts.len() > 2 {
            pinned[a as usize] = true;
            pinned[b as usize] = true;
        }
        if !(open || feature || named) {
            continue;
        }
        m.lines[a as usize].push(b);
        m.lines[b as usize].push(a);
        if open {
            border[a as usize] = true;
            border[b as usize] = true;
        }
        let (pa, pb) = (s.positions[a as usize], s.positions[b as usize]);
        for &t in ts {
            let [x, y, z] = s.triangles[t as usize].map(|v| s.positions[v as usize]);
            let side = (pb - pa).cross((y - x).cross(z - x));
            let len = side.length();
            if len > 0.0 {
                let q = Quadric::plane(side / len, pa, weight);
                m.quadric[a as usize].add(&q);
                m.quadric[b as usize].add(&q);
            }
        }
    }
    // A vertex whose triangles make more than one fan (two sheets meeting
    // at a point) is pinned: the link condition assumes one.
    for v in 0..nv {
        let tris = &m.around[v];
        if tris.len() < 2 {
            continue;
        }
        // Triangles around v joined across the edges from v they share.
        let mut parent: Vec<usize> = (0..tris.len()).collect();
        fn root(p: &mut [usize], mut i: usize) -> usize {
            while p[i] != i {
                p[i] = p[p[i]];
                i = p[i];
            }
            i
        }
        let mut by_other: Vec<(u32, usize)> = tris
            .iter()
            .enumerate()
            .flat_map(|(k, &t)| {
                s.triangles[t as usize]
                    .into_iter()
                    .filter(move |&o| o as usize != v)
                    .map(move |o| (o, k))
            })
            .collect();
        by_other.sort_unstable();
        for w in by_other.windows(2) {
            if w[0].0 == w[1].0 {
                let (a, b) = (root(&mut parent, w[0].1), root(&mut parent, w[1].1));
                if a != b {
                    parent[a.max(b)] = a.min(b);
                }
            }
        }
        if (0..tris.len()).any(|i| root(&mut parent, i) != 0) {
            pinned[v] = true;
        }
    }
    for v in 0..nv {
        m.kind[v] = match m.lines[v].len() {
            0 => Kind::Interior,
            2 => Kind::Line,
            _ => Kind::Corner,
        };
        if pinned[v] || (d.lock_border && border[v]) {
            m.kind[v] = Kind::Corner;
        }
    }
    let mut heap: BinaryHeap<Entry> = BinaryHeap::new();
    for &(a, b) in users.keys() {
        if let Some(e) = m.plan(a, b) {
            heap.push(e);
        }
    }
    let mut grid = Grid::new(&m);
    // Edges refused for folding: whether they fold depends on triangles
    // that are not their neighbours, so they are tried again once the
    // queue runs dry, until a round takes none.
    let mut folded: Vec<EdgeKey> = Vec::new();
    let mut first = true;
    loop {
        let mut taken = 0usize;
        while m.live > target {
            let Some(e) = heap.pop() else { break };
            if (m.stamp[e.from as usize], m.stamp[e.to as usize]) != e.stamp {
                continue;
            }
            let err = m.error_of(&e);
            if d.max_error > 0.0 && err > d.max_error {
                continue;
            }
            if !m.valid(e.from, e.to, e.pos) {
                continue;
            }
            if m.folds(&grid, e.from, e.to, e.pos) {
                report.folds_refused += 1;
                folded.push(e.edge);
                continue;
            }
            let fan = m.fan(e.from, e.to);
            for &t in &fan {
                grid.remove(&m, t);
            }
            m.collapse(&e);
            for &t in &fan {
                if m.alive[t as usize] {
                    grid.insert(&m, t);
                }
            }
            if 2 * m.live < grid.built {
                grid = Grid::new(&m);
            }
            taken += 1;
            report.error = report.error.max(err);
            let to = e.to;
            for x in m.neighbours(to) {
                if let Some(next) = m.plan(to, x) {
                    heap.push(next);
                }
            }
        }
        if m.live <= target || folded.is_empty() || (!first && taken == 0) {
            break;
        }
        first = false;
        folded.sort_unstable();
        folded.dedup();
        for (a, b) in std::mem::take(&mut folded) {
            if m.joined(a, b)
                && let Some(e) = m.plan(a, b)
            {
                heap.push(e);
            }
        }
        if heap.is_empty() {
            break;
        }
    }
    if m.live > target {
        report.blocked = m.blocked(&grid, d);
    }
    let mut out = Solid {
        positions: m.pos.clone(),
        names: s.names.clone(),
        uvs: s.uvs.as_ref().map(|_| Vec::new()),
        colors: s.colors.as_ref().map(|_| Vec::new()),
        skin: s.skin.as_ref().map(|k| Skin {
            bones: k.bones.clone(),
            weights: m.weights.clone().unwrap_or_default(),
        }),
        morphs: m.morphs.clone(),
        ..Default::default()
    };
    for (i, t) in m.tris.iter().enumerate() {
        if m.alive[i] {
            out.triangles.push(*t);
            out.surfaces.push(s.surfaces[i]);
            out.face_tags.push(s.face_tags[i].clone());
            if let (Some(to), Some(from)) = (&mut out.uvs, &m.uvs) {
                to.push(from[i]);
            }
            if let (Some(to), Some(from)) = (&mut out.colors, &m.colors) {
                to.push(from[i]);
            }
        }
    }
    for (&(a, b), tags) in &s.edge_tags {
        let (a, b) = (m.root(a), m.root(b));
        if a != b {
            let e = out.edge_tags.entry(edge_key(a, b)).or_default();
            e.extend(tags);
            e.sort_unstable();
            e.dedup();
        }
    }
    let mut out = drop_unused(&out);
    out.canonicalize().map_err(|_| DecimateError::NotFinite)?;
    report.triangles = out.triangles.len();
    Ok((out, report))
}
