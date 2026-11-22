//! Operands before the arrangement: weld, snap onto the other operand, bounds, and why an operand is refused.

use super::*;

/// Why [`csg_points`] refused its operands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperandError {
    /// A position was NaN or infinite (operand 0 = A, 1 = B).
    NotFinite(u8),
    /// After exact welding, an edge is used other than once each way:
    /// the operand is open, not oriented, or not manifold.
    NotClosed(u8),
}

pub(crate) fn weld(
    mesh: &Mesh,
    placement: Placement,
    points: &mut Vec<DVec3>,
    at: &mut BTreeMap<[u64; 3], u32>,
) -> Result<Solid, String> {
    let placed = place(mesh, placement.matrix())?;
    if placed.positions.iter().any(|p| !p.is_finite()) {
        return Err("it holds a position that is not finite".into());
    }
    let mut ids = Vec::with_capacity(placed.positions.len());
    let start = points.len() as u32;
    for p in &placed.positions {
        // + 0.0 makes -0 and 0 one point.
        let key = [
            (p.x + 0.0).to_bits(),
            (p.y + 0.0).to_bits(),
            (p.z + 0.0).to_bits(),
        ];
        let id = match at.get(&key) {
            Some(&id) if id >= start => id,
            _ => {
                let id = points.len() as u32;
                points.push(*p);
                at.insert(key, id);
                id
            }
        };
        ids.push(id);
    }
    let mut tris = Vec::new();
    let mut corners = Vec::new();
    let mut source = Vec::new();
    let (mut min, mut max) = (DVec3::INFINITY, DVec3::NEG_INFINITY);
    for (ti, t) in placed.indices.as_chunks::<3>().0.iter().enumerate() {
        let w = [ids[t[0] as usize], ids[t[1] as usize], ids[t[2] as usize]];
        if w[0] == w[1] || w[1] == w[2] || w[0] == w[2] {
            continue;
        }
        tris.push(w);
        source.push(ti as u32);
        corners.push([0, 1, 2].map(|k| {
            let i = t[k] as usize;
            Corner {
                normal: placed.normals[i],
                uv: placed.uvs[i],
                color: placed.colors[i],
            }
        }));
        for &i in &w {
            min = min.min(points[i as usize]);
            max = max.max(points[i as usize]);
        }
    }
    Ok(Solid {
        tris,
        corners,
        source,
        min,
        max,
    })
}

/// Welds an f64 operand (`m`: 0 = A, 1 = B) by exact position into
/// `points`, refusing it unless it is closed and oriented: every directed
/// edge used once and its reverse once, or with `balanced`, as often as
/// its reverse (faces lying on one another, as a self-union takes).
pub(crate) fn weld64(
    m: u8,
    balanced: bool,
    (pos, tris): (&[DVec3], &[[u32; 3]]),
    points: &mut Vec<DVec3>,
    at: &mut BTreeMap<[u64; 3], u32>,
) -> Result<Solid, OperandError> {
    if pos.iter().any(|p| !p.is_finite()) {
        return Err(OperandError::NotFinite(m));
    }
    let start = points.len() as u32;
    let ids: Vec<u32> = pos
        .iter()
        .map(|p| {
            let key = [
                (p.x + 0.0).to_bits(),
                (p.y + 0.0).to_bits(),
                (p.z + 0.0).to_bits(),
            ];
            match at.get(&key) {
                Some(&id) if id >= start => id,
                _ => {
                    let id = points.len() as u32;
                    points.push(*p + DVec3::ZERO);
                    at.insert(key, id);
                    id
                }
            }
        })
        .collect();
    let mut s = Solid {
        tris: Vec::new(),
        corners: Vec::new(),
        source: Vec::new(),
        min: DVec3::INFINITY,
        max: DVec3::NEG_INFINITY,
    };
    let mut directed: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    let flat = Corner {
        normal: DVec3::Y,
        uv: [0.0; 2],
        color: [1.0; 4],
    };
    for (ti, t) in tris.iter().enumerate() {
        let w = t.map(|i| ids[i as usize]);
        if w[0] == w[1] || w[1] == w[2] || w[0] == w[2] {
            continue;
        }
        for k in 0..3 {
            *directed.entry((w[k], w[(k + 1) % 3])).or_default() += 1;
        }
        s.tris.push(w);
        s.source.push(ti as u32);
        s.corners.push([flat; 3]);
        for &i in &w {
            s.min = s.min.min(points[i as usize]);
            s.max = s.max.max(points[i as usize]);
        }
    }
    let closed = !s.tris.is_empty()
        && directed
            .iter()
            .all(|(&(x, y), &n)| (balanced || n == 1) && directed.get(&(y, x)) == Some(&n));
    if !closed {
        return Err(OperandError::NotClosed(m));
    }
    Ok(s)
}

pub(crate) fn bounds(points: &[DVec3], t: [u32; 3]) -> (DVec3, DVec3) {
    let p = t.map(|i| points[i as usize]);
    (p[0].min(p[1]).min(p[2]), p[0].max(p[1]).max(p[2]))
}

pub(crate) fn overlap(a: (DVec3, DVec3), b: (DVec3, DVec3)) -> bool {
    a.0.cmple(b.1).all() && b.0.cmple(a.1).all()
}

/// 2⁻⁴⁰ of the diagonal of the box around both operands' positions: the
/// design's snap distance (`docs/design.md` §6).
pub(crate) fn snap_distance(a: &[DVec3], b: &[DVec3]) -> f64 {
    let (mut lo, mut hi) = (DVec3::INFINITY, DVec3::NEG_INFINITY);
    for &p in a.iter().chain(b) {
        lo = lo.min(p);
        hi = hi.max(p);
    }
    (hi - lo).length() * (1.0 / (1u64 << 40) as f64)
}

/// The tool's positions `b` with each one that lies within 2⁻⁴⁰ of the
/// operands' size of a position of the target `a`, but not on one, moved
/// onto the nearest (the first of equals); None when none moves. The
/// design's snap before a boolean (`docs/design.md` §6), tried only when
/// the result is open or not manifold.
pub(crate) fn snap_onto(a: &[DVec3], b: &[DVec3]) -> Option<Vec<DVec3>> {
    let (mut lo, mut hi) = (DVec3::INFINITY, DVec3::NEG_INFINITY);
    for &p in a.iter().chain(b) {
        lo = lo.min(p);
        hi = hi.max(p);
    }
    let tol = (hi - lo).length() * (1.0 / (1u64 << 40) as f64);
    let tol2 = tol * tol;
    // The target's positions by x, for a window search.
    let mut by_x: Vec<u32> = (0..a.len() as u32).collect();
    og_core::sort::sort_by(&mut by_x, |&i, &j| {
        a[i as usize].x.total_cmp(&a[j as usize].x).then(i.cmp(&j))
    });
    let mut out = b.to_vec();
    let mut moved = false;
    for p in &mut out {
        let start = by_x.partition_point(|&i| a[i as usize].x < p.x - tol);
        let mut best: Option<(f64, u32)> = None;
        for &i in &by_x[start..] {
            let q = a[i as usize];
            if q.x > p.x + tol {
                break;
            }
            let d2 = (q - *p).length_squared();
            if d2 <= tol2 && best.is_none_or(|(bd, bi)| d2 < bd || (d2 == bd && i < bi)) {
                best = Some((d2, i));
            }
        }
        if let Some((d2, i)) = best
            && d2 > 0.0
        {
            *p = a[i as usize];
            moved = true;
        }
    }
    moved.then_some(out)
}
