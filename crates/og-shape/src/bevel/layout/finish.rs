//! The rebuilt solid assembled from the build and the rebuilt faces.

use super::*;

pub(crate) fn finish(
    mut b: Build<'_>,
    inserts: &BTreeMap<(usize, u32), Vec<u32>>,
) -> Result<Solid, Fail> {
    let s = b.s;
    let mut welded_inserts = None;
    if !b.poles.is_empty() {
        // The points of each swallowed run are one point now.
        let to: Vec<u32> = (0..b.pos.len() as u32)
            .map(|v| b.poles.get(&v).copied().unwrap_or(v))
            .collect();
        let map = |ids: &[u32]| {
            let mut out: Vec<u32> = ids.iter().map(|&v| to[v as usize]).collect();
            out.dedup();
            out
        };
        for ids in b.repl.values_mut() {
            *ids = map(ids);
        }
        b.new_tris = b
            .new_tris
            .iter()
            .map(|&(t, u)| (t.map(|v| to[v as usize]), u))
            .filter(|(t, _)| t[0] != t[1] && t[1] != t[2] && t[2] != t[0])
            .collect();
        welded_inserts = Some(
            inserts
                .iter()
                .map(|(k, ids)| (*k, map(ids)))
                .collect::<BTreeMap<_, _>>(),
        );
    }
    let inserts = welded_inserts.as_ref().unwrap_or(inserts);
    let weld = !b.poles.is_empty();
    let kind = b.kind.name();
    let seq = |t: usize, x: u32| -> Vec<u32> { b.repl.get(&(t, x)).cloned().unwrap_or(vec![x]) };
    // A rebuilt triangle's outline: each corner's replacement, then the
    // points a wide bevel's strip put on the side after it.
    let poly = |t: usize| -> Vec<u32> {
        s.triangles[t]
            .iter()
            .flat_map(|&x| {
                let mut v = seq(t, x);
                v.extend(inserts.get(&(t, x)).into_iter().flatten());
                v
            })
            .collect()
    };
    let removed = |t: usize| b.wide.is_some_and(|w| w.removed.contains(&t));
    // The classic model keeps each old triangle and moves its corner at a
    // bevelled vertex. Where the faces there are not one plane (a twisted
    // quad's long triangle, a cone's fan at its apex), the new corner
    // leaves the triangle's plane and tilts the whole triangle, far from
    // the bevel: the result would be wrong without crossing itself
    // (`docs/research.md` §7.7). Such bands are cut instead (`wide.rs`).
    let largest = b.edges.values().flat_map(|e| e.off).fold(0.0f64, f64::max);
    let mut tilted: BTreeSet<(EdgeKey, bool)> = BTreeSet::new();
    let mut tilted_at = None;
    for (&(t, v), pts) in &b.repl {
        if removed(t) {
            continue;
        }
        let n = tri_normal(s, t);
        let p0 = s.positions[v as usize];
        // A vertex of the solid taken as the corner (a wide bevel's cut
        // point) is exact geometry; pass 2 lets a second band take the
        // first band's cut at a shared corner (`wide.rs`).
        let new = s.positions.len() as u32;
        if pts
            .iter()
            .all(|&q| q < new || (b.pos[q as usize] - p0).dot(n).abs() <= b.tilt * largest)
        {
            continue;
        }
        tilted_at.get_or_insert(p0);
        for &(lo, hi) in b.edges.keys().filter(|k| k.0 == v || k.1 == v) {
            for side_a in [true, false] {
                let dir = if side_a { (lo, hi) } else { (hi, lo) };
                if b.topo.dir.get(&dir).map(|&x| s.surfaces[x]) == Some(s.surfaces[t]) {
                    tilted.insert(((lo, hi), side_a));
                }
            }
        }
    }
    if let Some(at) = tilted_at {
        return Err(Fail::Tilted(at, tilted));
    }
    // Old faces, rebuilt triangle by triangle; where a triangle does not
    // fit (a sliver whose corners all moved), its whole planar face is
    // rebuilt from its outline instead.
    let n_old = s.triangles.len();
    let mut pieces: Vec<Option<Vec<[u32; 3]>>> = vec![None; n_old];
    let mut failed = Vec::new();
    for (i, t) in s.triangles.iter().enumerate() {
        if removed(i) {
            pieces[i] = Some(Vec::new());
            continue;
        }
        if !t
            .iter()
            .any(|&x| b.repl.contains_key(&(i, x)) || inserts.contains_key(&(i, x)))
        {
            continue;
        }
        let poly = poly(i);
        if (b.wide.is_some() || weld) && wide::collapsed(&poly) {
            // A band triangle whose corners all landed on the cut.
            pieces[i] = Some(Vec::new());
            continue;
        }
        match clip(&poly, &b.pos, tri_normal(s, i)) {
            Some(p) => pieces[i] = Some(p),
            None => failed.push(i),
        }
    }
    let mut absorbed = vec![false; n_old];
    let mut regions: BTreeMap<usize, Vec<[u32; 3]>> = BTreeMap::new();
    for &i in &failed {
        if absorbed[i] {
            continue;
        }
        let t = s.triangles[i];
        let at = t
            .iter()
            .find(|&&x| b.repl.contains_key(&(i, x)))
            .map_or(DVec3::ZERO, |&x| s.positions[x as usize]);
        let mut region = planar_region(s, &b.topo, i);
        region.retain(|&r| !removed(r));
        let polys: Vec<Vec<u32>> = region.iter().map(|&r| poly(r)).collect();
        let nrm = tri_normal(s, i);
        let tris = outline(&polys)
            .and_then(|loops| bridge(loops, &b.pos, nrm))
            .and_then(|ring| clip(&ring, &b.pos, nrm))
            .ok_or(BevelError::TooLarge(at))?;
        for &r in &region {
            absorbed[r] = true;
        }
        regions.insert(region[0], tris);
    }
    let mut tris: Vec<([u32; 3], Option<usize>, usize)> = Vec::new(); // (tri, unit, old)
    for (i, t) in s.triangles.iter().enumerate() {
        if absorbed[i] {
            for &p in regions.get(&i).into_iter().flatten() {
                tris.push((p, None, i));
            }
            continue;
        }
        match &pieces[i] {
            None => tris.push((*t, None, i)),
            Some(ps) => {
                for &p in ps {
                    tris.push((p, None, i));
                }
            }
        }
    }
    for &(t, unit) in &b.new_tris {
        tris.push((t, Some(unit), usize::MAX));
    }
    // Surfaces of new units.
    let mut parent: Vec<usize> = (0..b.units).collect();
    for &(x, y) in &b.joins {
        union(&mut parent, x, y);
    }
    let roots: Vec<usize> = (0..b.units).map(|u| find(&mut parent, u)).collect();
    let base = s.surfaces.iter().max().map_or(0, |m| m + 1);
    let normal = |t: [u32; 3]| {
        let [p, q, r] = t.map(|v| b.pos[v as usize]);
        (q - p).cross(r - p).normalize_or_zero()
    };
    // Edge → triangles.
    let mut by_edge: BTreeMap<EdgeKey, Vec<usize>> = BTreeMap::new();
    for (i, (t, _, _)) in tris.iter().enumerate() {
        for k in 0..3 {
            by_edge
                .entry(edge_key(t[k], t[(k + 1) % 3]))
                .or_default()
                .push(i);
        }
    }
    let cos10 = double::cos(double::to_radians(10.0));
    let cos40 = double::cos(double::to_radians(40.0));
    let mut new_names: BTreeMap<EdgeKey, Vec<String>> = BTreeMap::new();
    for (&e, ts) in &by_edge {
        if ts.len() != 2 {
            continue;
        }
        let (x, y) = (&tris[ts[0]], &tris[ts[1]]);
        let (ux, uy) = (x.1.map(|u| roots[u]), y.1.map(|u| roots[u]));
        if ux.is_none() && uy.is_none() || ux == uy {
            continue;
        }
        let cos = normal(x.0).dot(normal(y.0));
        let mut names = Vec::new();
        match (ux, uy) {
            (Some(_), Some(_)) => {
                let limit = if b.kind == Kind::Chamfer {
                    cos40
                } else {
                    cos10
                };
                if cos < limit {
                    names.push(kind.to_string());
                }
            }
            _ => {
                if cos < cos10 {
                    names.push(kind.to_string());
                    let old = if ux.is_none() { x.2 } else { y.2 };
                    for &tag in s.face_tags[old].iter() {
                        let name = s.names.name(tag);
                        if is_cap(name) {
                            names.push(name.to_string());
                        }
                    }
                }
            }
        }
        if !names.is_empty() {
            new_names.insert(e, names);
        }
    }
    // Old edge names, following their ends.
    for (&(a, c), tags) in &s.edge_tags {
        if b.edges.contains_key(&(a, c)) {
            continue;
        }
        let Some(&t) = b.topo.dir.get(&(a, c)) else {
            continue;
        };
        let ra = *seq(t, a).last().unwrap_or(&a);
        let rc = *seq(t, c).first().unwrap_or(&c);
        let names = new_names.entry(edge_key(ra, rc)).or_default();
        names.extend(tags.iter().map(|&x| s.names.name(x).to_string()));
    }
    // Assemble with only the vertices in use.
    let mut used: Vec<u32> = tris.iter().flat_map(|t| t.0).collect();
    used.sort_unstable();
    used.dedup();
    let mut remap = vec![u32::MAX; b.pos.len()];
    let mut out = Solid::default();
    for &v in &used {
        remap[v as usize] = out.push_vertex(b.pos[v as usize]);
    }
    for (t, unit, old) in &tris {
        let t = t.map(|v| remap[v as usize]);
        match unit {
            None => {
                let names: Vec<&str> = s.face_tags[*old].iter().map(|&x| s.names.name(x)).collect();
                out.push_triangle(t, s.surfaces[*old], &names);
            }
            Some(u) => {
                out.push_triangle(t, base + roots[*u] as u32, &[kind]);
            }
        }
    }
    for ((x, y), names) in new_names {
        let (rx, ry) = (remap[x as usize], remap[y as usize]);
        if rx == u32::MAX || ry == u32::MAX {
            // An old edge inside a band cut across a wall's rows
            // (`rows.rs`): gone with its triangles.
            continue;
        }
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        out.tag_edge(rx, ry, &refs);
    }
    out.canonicalize()
        .map_err(|_| BevelError::NotFinite("result"))?;
    let m = out.measure();
    if !m.closed || m.degenerate_triangles > 0 {
        let at = out.positions.first().copied().unwrap_or(DVec3::ZERO);
        return Err(BevelError::TooLarge(at).into());
    }
    // The backstop (`docs/research.md` §7.7): the input does not cross
    // itself, so two triangles of the result that cross are the bevel's
    // own, and it did not fit. The error names the bevelled vertex nearest
    // to where they cross.
    let first = match &b.checked {
        Some(input) => check::first_crossing(&out, input),
        None => crate::boolean::selfcheck::self_crossings(&out.positions, &out.triangles)
            .first()
            .copied(),
    };
    if let Some((i, j)) = first {
        let c = [i, j]
            .iter()
            .flat_map(|&t| out.triangles[t as usize])
            .fold(DVec3::ZERO, |a, v| a + out.positions[v as usize])
            / 6.0;
        let mut at = (f64::INFINITY, c);
        for v in b.edges.keys().flat_map(|&(x, y)| [x, y]) {
            let p = s.positions[v as usize];
            let d = (p - c).length_squared();
            if d < at.0 {
                at = (d, p);
            }
        }
        return Err(BevelError::TooLarge(at.1).into());
    }
    Ok(out)
}
