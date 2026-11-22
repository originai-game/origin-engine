//! merge: solids joined into one without a boolean, and the triangle splits that carry UVs through an edit.

use super::*;

/// The UVs of a piece of triangle `t` (UVs `uv`) whose corners are
/// `t`'s corners or the midpoints `mid` gives of its edges.
pub(crate) fn piece_uvs(
    t: [u32; 3],
    uv: &CornerUvs,
    piece: [u32; 3],
    mid: &BTreeMap<(u32, u32), u32>,
) -> CornerUvs {
    piece.map(|v| {
        if let Some(k) = t.iter().position(|&x| x == v) {
            return uv[k];
        }
        (0..3)
            .find(|&k| mid.get(&edge_key(t[k], t[(k + 1) % 3])) == Some(&v))
            .map_or(DVec2::ZERO, |k| (uv[k] + uv[(k + 1) % 3]) / 2.0)
    })
}

/// A triangle with some edges split at the given midpoints, as 1 to 4
/// triangles with the same winding.
pub(crate) fn split_triangle(t: [u32; 3], m: [Option<u32>; 3]) -> Vec<[u32; 3]> {
    let [a, b, c] = t;
    match m {
        [None, None, None] => vec![t],
        [Some(ab), Some(bc), Some(ca)] => vec![[a, ab, ca], [ab, b, bc], [ca, bc, c], [ab, bc, ca]],
        _ => {
            // Rotate so the split edges come first, then fan.
            let k = (0..3)
                .find(|&k| m[k].is_some() && m[(k + 2) % 3].is_none())
                .unwrap_or(0);
            let v = [t[k], t[(k + 1) % 3], t[(k + 2) % 3]];
            let mm = [m[k], m[(k + 1) % 3], m[(k + 2) % 3]];
            match mm {
                [Some(x), None, None] => vec![[v[0], x, v[2]], [x, v[1], v[2]]],
                [Some(x), Some(y), None] => vec![[v[0], x, v[2]], [x, v[1], y], [x, y, v[2]]],
                _ => vec![t],
            }
        }
    }
}

/// One solid holding all the given ones, untouched: no boolean. Shells
/// that touch stay separate; a later boolean fuses them first. Skins
/// follow [`Solid::append`]: kept when every solid has the same skeleton,
/// otherwise reported by [`crate::mesh::Skin::check`]. UVs too: kept, and
/// a solid without them among solids with them gets (0, 0) at every
/// corner (zero-area UV triangles, which `measure` counts); `shape-model`
/// refuses such a merge.
pub fn merge(solids: &[&Solid]) -> Solid {
    let mut out = Solid::default();
    for s in solids {
        out.append(s);
    }
    let _ = out.canonicalize();
    out
}
