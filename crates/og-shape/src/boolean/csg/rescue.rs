//! A result that crosses itself by a rounding error, uncrossed and rejoined.

use super::*;

/// `out`, a result of `a op b`, with the triangles that cross by a
/// rounding error repaired ([`uncross`](crate::boolean::uncross)): unchanged, byte
/// for byte, when none crosses; with the number of crossing pairs before
/// and after.
pub fn uncross_output(
    a: &[DVec3],
    b: &[DVec3],
    out: CsgOutput,
) -> (CsgOutput, crate::boolean::uncross::Uncrossed) {
    let none = crate::boolean::uncross::Uncrossed::default();
    if out.unmatched > 0 || out.tris.is_empty() {
        return (out, none);
    }
    let w = weld_ids(&out.points);
    let n = w.iter().map(|&x| x + 1).max().unwrap_or(0) as usize;
    let mut wpoints = vec![DVec3::ZERO; n];
    for (i, &x) in w.iter().enumerate() {
        wpoints[x as usize] = out.points[i];
    }
    let key = |p: DVec3| p.to_array().map(|x| (x + 0.0).to_bits());
    let mut input: BTreeMap<[u64; 3], u8> = BTreeMap::new();
    for &p in a {
        input.insert(key(p), 0);
    }
    for &p in b {
        input.entry(key(p)).or_insert(1);
    }
    let rank: Vec<u8> = wpoints
        .iter()
        .map(|&p| input.get(&key(p)).copied().unwrap_or(2))
        .collect();
    let wtris: Vec<[u32; 3]> = out.tris.iter().map(|t| t.map(|v| w[v as usize])).collect();
    let (mut lo, mut hi) = (DVec3::INFINITY, DVec3::NEG_INFINITY);
    for &p in a.iter().chain(b) {
        lo = lo.min(p);
        hi = hi.max(p);
    }
    let tol = (hi - lo).length() * (1.0 / (1u64 << 40) as f64);
    let (report, kept) = crate::boolean::uncross::uncross(&wpoints, &wtris, &rank, tol);
    let Some(kept) = kept else {
        return (out, report);
    };
    let mut remap: BTreeMap<u32, u32> = BTreeMap::new();
    let mut points = Vec::new();
    let mut tris = Vec::with_capacity(kept.len());
    let mut origins = Vec::with_capacity(kept.len());
    for (i, t) in kept {
        tris.push(t.map(|v| {
            *remap.entry(v).or_insert_with(|| {
                points.push(wpoints[v as usize]);
                points.len() as u32 - 1
            })
        }));
        origins.push(out.origins[i]);
    }
    let ids: Vec<u32> = (0..points.len() as u32).collect();
    let counts = edge_counts(&ids, &tris, &origins);
    (
        CsgOutput {
            points,
            tris,
            origins,
            unmatched: counts.unmatched,
            touching_edges: counts.touching,
            non_manifold: counts.non_manifold,
            ..out
        },
        report,
    )
}

/// A closed result `first` of `a` with `b` left in several shells
/// (faces a rounding error apart that should be one: a union of solids
/// stacked with a gap of a few ulps, a cut leaving a membrane an ulp
/// thick), built again with the tool snapped onto the target
/// ([`snap_onto`]); that result is taken if it is closed, manifold and in
/// fewer shells. Otherwise, and when nothing snaps, `first` as built.
#[allow(clippy::type_complexity)]
pub(crate) fn rejoined(
    a: (&[DVec3], &[[u32; 3]]),
    b: (&[DVec3], &[[u32; 3]]),
    first: CsgOutput,
    pass: &dyn Fn(bool, (&[DVec3], &[[u32; 3]])) -> Result<CsgOutput, OperandError>,
) -> CsgOutput {
    let shells = |o: &CsgOutput| shell_count(&o.points, &o.tris);
    let n = shells(&first);
    if n < 2 {
        return first;
    }
    let Some(snapped) = snap_onto(a.0, b.0) else {
        return first;
    };
    let clean = |o: &CsgOutput| o.unmatched == 0 && o.non_manifold == 0 && o.touching_edges == 0;
    match pass(false, (&snapped, b.1)) {
        Ok(o) if clean(&o) && !o.tris.is_empty() && shells(&o) < n => {
            CsgOutput { snapped: true, ..o }
        }
        _ => first,
    }
}

/// The number of shells (triangles joined by a shared position).
pub(crate) fn shell_count(points: &[DVec3], tris: &[[u32; 3]]) -> usize {
    let w = weld_ids(points);
    let mut parent: Vec<u32> = (0..points.len() as u32).collect();
    fn find(p: &mut [u32], mut i: u32) -> u32 {
        while p[i as usize] != i {
            p[i as usize] = p[p[i as usize] as usize];
            i = p[i as usize];
        }
        i
    }
    for t in tris {
        let t = t.map(|v| w[v as usize]);
        for k in 1..3 {
            let (x, y) = (find(&mut parent, t[0]), find(&mut parent, t[k]));
            if x != y {
                parent[x.max(y) as usize] = x.min(y);
            }
        }
    }
    let mut roots: Vec<u32> = tris
        .iter()
        .map(|t| find(&mut parent, w[t[0] as usize]))
        .collect();
    roots.sort_unstable();
    roots.dedup();
    roots.len()
}
