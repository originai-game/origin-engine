//! Step: degenerate and duplicate triangles removed.

use super::*;

/// Drops triangles with no area (three corners in a line, decided
/// exactly) and triangles that repeat another's corners in the same
/// winding (the first is kept). Two triangles on the same corners wound
/// opposite ways (a two-sided sheet) are both kept. Returns the solid and
/// the counts `(degenerate, duplicates)`.
pub fn remove_degenerate(s: &Solid) -> Result<(Solid, usize, usize), RepairError> {
    let mut seen: BTreeMap<[u32; 3], ()> = BTreeMap::new();
    let (mut deg, mut dup) = (0, 0);
    let mut keep = Vec::with_capacity(s.triangles.len());
    for (i, t) in s.triangles.iter().enumerate() {
        if degenerate(t.map(|v| s.positions[v as usize])) {
            deg += 1;
            continue;
        }
        let r = (0..3).min_by_key(|&k| t[k]).unwrap_or(0);
        let key = [t[r], t[(r + 1) % 3], t[(r + 2) % 3]];
        if seen.insert(key, ()).is_some() {
            dup += 1;
            continue;
        }
        keep.push(i);
    }
    let mut out = kept(s, &keep);
    out.canonicalize()?;
    Ok((out, deg, dup))
}
