//! Step: interpenetrating shells fused into one by a self-union.

use super::*;

/// The fuse step: `crate::boolean::fuse`, made to scale on a large shell that
/// passes through itself. Each shell is untangled on its own
/// (`crate::boolean::self_union`); the shells an untangling leaves besides its
/// largest are pockets the surface folded round ([`RepairReport::pockets`]),
/// and those that are floaters by `floaters` are dropped. Then only the
/// shells that meet another (exact, `selfcheck::tris_meet`) or may lie
/// inside one (its box within the other's) are joined by booleans, group
/// by group; the rest are kept as they are, since their union is
/// themselves. Before, every pocket was joined to the whole by its own
/// boolean: 133 s on a 619 000-triangle chair (`docs/research.md` §32).
pub(crate) fn fuse_shells(
    s: &Solid,
    floaters: &Floaters,
    r: &mut RepairReport,
) -> Result<Solid, String> {
    let comps = crate::boolean::components(s);
    let tangled: Vec<bool> = comps
        .iter()
        .map(|c| crate::boolean::crossing_pairs(c) > 0)
        .collect();
    if !tangled.iter().any(|&t| t) {
        // Nothing to untangle: the booleans' own fuse, as before.
        return crate::boolean::fuse(s).map_err(|e| e.to_string());
    }
    let mut all = Solid::default();
    let mut whole: Option<Solid> = None;
    let alone = comps.len() == 1;
    // Each shell is given up as it is taken, and each copy dropped once
    // the next stands (`docs/research.md` §36).
    for (c, t) in comps.into_iter().zip(tangled) {
        let shells = if t {
            match crate::boolean::self_union(&c) {
                Ok(u) => {
                    drop(c);
                    let parts = crate::boolean::components(&u);
                    if alone && parts.len() == 1 {
                        whole = Some(u);
                    }
                    parts
                }
                Err(_) => vec![c],
            }
        } else {
            vec![c]
        };
        for x in &shells {
            all.append(x);
        }
        r.pockets += shells.len().saturating_sub(1);
    }
    all.canonicalize().map_err(|e| e.to_string())?;
    if let Some(u) = whole {
        // One shell, untangled whole: the self-union is the result.
        return Ok(u);
    }
    let (kept, dropped, gone) = remove_floaters(&all, floaters).map_err(|e| e.to_string())?;
    drop(all);
    let all = kept;
    r.pockets_dropped = dropped;
    r.pocket_triangles_dropped = gone;
    // Which shells must be joined: they meet, or one's box holds the
    // other's.
    let shells = crate::boolean::components(&all);
    if shells.len() <= 1 {
        return Ok(all);
    }
    let mut points: Vec<DVec3> = Vec::new();
    let mut tris: Vec<[u32; 3]> = Vec::new();
    let mut owner: Vec<u32> = Vec::new();
    let mut boxes: Vec<(DVec3, DVec3)> = Vec::new();
    for (k, x) in shells.iter().enumerate() {
        let base = points.len() as u32;
        points.extend(&x.positions);
        let (mut lo, mut hi) = (DVec3::INFINITY, DVec3::NEG_INFINITY);
        for p in &x.positions {
            lo = lo.min(*p);
            hi = hi.max(*p);
        }
        boxes.push((lo, hi));
        for t in &x.triangles {
            tris.push(t.map(|v| v + base));
            owner.push(k as u32);
        }
    }
    let mut parent: Vec<u32> = (0..shells.len() as u32).collect();
    let join = |parent: &mut Vec<u32>, a: u32, b: u32| {
        let (a, b) = (find(parent, a), find(parent, b));
        if a != b {
            parent[a.max(b) as usize] = a.min(b);
        }
    };
    for (i, j) in crate::boolean::selfcheck::box_pairs(&points, &tris) {
        let (a, b) = (owner[i as usize], owner[j as usize]);
        if a != b
            && find(&mut parent, a) != find(&mut parent, b)
            && crate::boolean::selfcheck::tris_meet(
                tris[i as usize].map(|v| points[v as usize]),
                tris[j as usize].map(|v| points[v as usize]),
            )
        {
            join(&mut parent, a, b);
        }
    }
    for a in 0..shells.len() {
        for b in 0..shells.len() {
            let ((alo, ahi), (blo, bhi)) = (boxes[a], boxes[b]);
            if a != b && blo.cmple(alo).all() && ahi.cmple(bhi).all() {
                join(&mut parent, a as u32, b as u32);
            }
        }
    }
    let mut groups: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for k in 0..shells.len() {
        groups
            .entry(find(&mut parent, k as u32))
            .or_default()
            .push(k);
    }
    let mut out = Solid::default();
    for members in groups.values() {
        if members.len() == 1 {
            out.append(&shells[members[0]]);
            continue;
        }
        // The largest shell against the others joined first among
        // themselves: one boolean on the large one, not one per shell.
        let big = *members
            .iter()
            .max_by_key(|&&k| (shells[k].triangles.len(), std::cmp::Reverse(k)))
            .unwrap_or(&members[0]);
        let mut rest = Solid::default();
        for &k in members {
            if k != big {
                rest.append(&shells[k]);
            }
        }
        rest.canonicalize().map_err(|e| e.to_string())?;
        let rest = crate::boolean::fuse(&rest).map_err(|e| e.to_string())?;
        let joined =
            crate::boolean::boolean(&shells[big], &rest, &crate::boolean::Boolean::default())
                .map_err(|e| e.to_string())?;
        out.append(&joined);
    }
    out.canonicalize().map_err(|e| e.to_string())?;
    Ok(out)
}
