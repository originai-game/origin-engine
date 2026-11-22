//! Step: small pieces apart from the rest removed.

use super::*;

pub(crate) fn find(p: &mut [u32], mut i: u32) -> u32 {
    while p[i as usize] != i {
        p[i as usize] = p[p[i as usize] as usize];
        i = p[i as usize];
    }
    i
}

/// Per triangle, its piece (sharing a vertex), numbered by first
/// triangle; and the number of pieces.
pub(crate) fn pieces(s: &Solid) -> (Vec<u32>, usize) {
    let mut parent: Vec<u32> = (0..s.positions.len() as u32).collect();
    for t in &s.triangles {
        for k in 1..3 {
            let (a, b) = (find(&mut parent, t[0]), find(&mut parent, t[k]));
            if a != b {
                parent[a.max(b) as usize] = a.min(b);
            }
        }
    }
    let mut ids: BTreeMap<u32, u32> = BTreeMap::new();
    let of: Vec<u32> = s
        .triangles
        .iter()
        .map(|t| {
            let r = find(&mut parent, t[0]);
            let n = ids.len() as u32;
            *ids.entry(r).or_insert(n)
        })
        .collect();
    (of, ids.len())
}

/// Drops pieces (connected by shared vertices) smaller than `d` says.
/// The largest piece (by bounding-box diagonal, then triangle count, then
/// order) is always kept. Returns the solid and the counts
/// `(pieces dropped, triangles dropped)`.
pub fn remove_floaters(s: &Solid, d: &Floaters) -> Result<(Solid, usize, usize), RepairError> {
    for v in [d.size, d.fraction] {
        if !(v >= 0.0 && v.is_finite()) {
            return Err(RepairError::BadDistance);
        }
    }
    let (of, n) = pieces(s);
    if n <= 1 {
        let mut out = s.clone();
        out.canonicalize()?;
        return Ok((out, 0, 0));
    }
    let mut lo = vec![DVec3::splat(f64::INFINITY); n];
    let mut hi = vec![DVec3::splat(f64::NEG_INFINITY); n];
    let mut count = vec![0usize; n];
    for (i, t) in s.triangles.iter().enumerate() {
        let c = of[i] as usize;
        count[c] += 1;
        for &v in t {
            lo[c] = lo[c].min(s.positions[v as usize]);
            hi[c] = hi[c].max(s.positions[v as usize]);
        }
    }
    let diag: Vec<f64> = (0..n).map(|c| (hi[c] - lo[c]).length()).collect();
    let largest = (0..n)
        .max_by(|&a, &b| {
            diag[a]
                .total_cmp(&diag[b])
                .then(count[a].cmp(&count[b]))
                .then(b.cmp(&a))
        })
        .unwrap_or(0);
    let limit = d.size.max(d.fraction * diag[largest]);
    let drop: Vec<bool> = (0..n).map(|c| c != largest && diag[c] < limit).collect();
    let keep: Vec<usize> = (0..s.triangles.len())
        .filter(|&i| !drop[of[i] as usize])
        .collect();
    let dropped = drop.iter().filter(|&&x| x).count();
    let tris = s.triangles.len() - keep.len();
    let mut out = kept(s, &keep);
    out.canonicalize()?;
    Ok((out, dropped, tris))
}
