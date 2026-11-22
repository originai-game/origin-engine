//! Step: vertices within a distance welded into one.

use super::*;

/// How [`weld`] merges vertices. The zero value welds exact duplicates
/// only.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Weld {
    /// Merges vertices closer than this, in mm.
    pub distance: f64,
    /// Merges any vertices, not only those on open edges (Blender's merge
    /// by distance). Off, a weld only closes cracks: a closed surface's
    /// vertices never move, so two close but separate sheets are not
    /// pinched together.
    pub all: bool,
}

/// Welds exactly coincident vertices, then each vertex on an open edge
/// (or each vertex, with `all`) to an earlier one within `distance` mm,
/// and canonicalizes (which drops triangles that lost a corner and
/// vertices no triangle uses). Vertices are visited in position order;
/// each joins the nearest kept vertex within reach, or is kept itself.
/// Kept vertices do not move, so the result depends only on the
/// positions, not on their order. Returns the solid and how many vertices
/// went.
pub fn weld(s: &Solid, d: &Weld) -> Result<(Solid, usize), RepairError> {
    weld_owned(s.clone(), d)
}

/// [`weld`], on a solid it may take apart: no second copy is made.
pub(crate) fn weld_owned(s: Solid, d: &Weld) -> Result<(Solid, usize), RepairError> {
    let distance = d.distance;
    if !(distance >= 0.0 && distance.is_finite()) {
        return Err(RepairError::BadDistance);
    }
    if s.positions.iter().any(|p| p.is_nan()) {
        return Err(RepairError::NaN);
    }
    let before = s.positions.len();
    let mut exact = s;
    exact.canonicalize()?;
    let welded = if distance > 0.0 {
        let s = &exact;
        let mut uses: BTreeMap<(u32, u32), u32> = BTreeMap::new();
        for t in &s.triangles {
            for k in 0..3 {
                *uses.entry(edge_key(t[k], t[(k + 1) % 3])).or_default() += 1;
            }
        }
        let mut open = vec![d.all; s.positions.len()];
        for (&(a, b), &n) in &uses {
            if n == 1 {
                open[a as usize] = true;
                open[b as usize] = true;
            }
        }
        let order = sorted_order(&s.positions);
        let cell = |p: DVec3| {
            let c = (p / distance).floor();
            (c.x as i64, c.y as i64, c.z as i64)
        };
        let mut grid: BTreeMap<(i64, i64, i64), Vec<u32>> = BTreeMap::new();
        let mut remap = vec![0u32; s.positions.len()];
        let mut kept: Vec<DVec3> = Vec::new();
        let d2 = distance * distance;
        for &v in &order {
            let p = s.positions[v as usize];
            if !open[v as usize] {
                kept.push(p);
                remap[v as usize] = kept.len() as u32 - 1;
                continue;
            }
            let (cx, cy, cz) = cell(p);
            let mut best: Option<(f64, u32)> = None;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        let Some(ids) = grid.get(&(cx + dx, cy + dy, cz + dz)) else {
                            continue;
                        };
                        for &k in ids {
                            let e = (kept[k as usize] - p).length_squared();
                            if e <= d2 && best.is_none_or(|(be, bk)| e < be || (e == be && k < bk))
                            {
                                best = Some((e, k));
                            }
                        }
                    }
                }
            }
            remap[v as usize] = match best {
                Some((_, k)) => k,
                None => {
                    kept.push(p);
                    let k = kept.len() as u32 - 1;
                    grid.entry((cx, cy, cz)).or_default().push(k);
                    k
                }
            };
        }
        let all: Vec<usize> = (0..s.triangles.len()).collect();
        Some(rebuild(s, &all, &remap, kept))
    } else {
        None
    };
    // One copy at a time: each is dropped before the next step's stands.
    let out = match welded {
        Some(w) => {
            drop(exact);
            w
        }
        None => exact,
    };
    let used = drop_unused(&out);
    drop(out);
    let mut out = used;
    out.canonicalize()?;
    let removed = before - out.positions.len();
    Ok((out, removed))
}
