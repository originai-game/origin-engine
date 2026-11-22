//! Charts by planarity: triangles grouped, each chart laid flat by its best axes, overlaps found.

use super::*;

pub(crate) fn unit(v: DVec3) -> DVec3 {
    let l = v.length();
    if l > 0.0 { v / l } else { DVec3::ZERO }
}

/// A chart's projection plane: two axes perpendicular to `n`.
pub(crate) fn basis(n: DVec3) -> (DVec3, DVec3) {
    let a = n.abs();
    let helper = if a.x <= a.y && a.x <= a.z {
        DVec3::X
    } else if a.y <= a.z {
        DVec3::Y
    } else {
        DVec3::Z
    };
    let u = unit(n.cross(helper));
    (u, n.cross(u))
}

pub(crate) type Cell = (i64, i64);

pub(crate) fn cells(t: &[DVec2; 3], size: f64) -> impl Iterator<Item = Cell> {
    let lo = t[0].min(t[1]).min(t[2]);
    let hi = t[0].max(t[1]).max(t[2]);
    let (x0, y0) = ((lo.x / size).floor() as i64, (lo.y / size).floor() as i64);
    let (x1, y1) = ((hi.x / size).floor() as i64, (hi.y / size).floor() as i64);
    (x0..=x1).flat_map(move |x| (y0..=y1).map(move |y| (x, y)))
}

pub(crate) fn charts(s: &Solid, d: &Unwrap) -> Result<Uvs, UnwrapError> {
    let (resolution, padding) = check(s, d)?;
    let (mut charts, mut chart_of, flat) = form(s, d);
    let n = s.triangles.len();
    // Each chart turned to its smallest box, laid wide, from the origin;
    // packed; then checked exactly in its final coordinates. Rounding in
    // the turn and the scale can push two triangles that only touched
    // into a sliver of overlap: such a triangle leaves for a chart of its
    // own, and the charts are packed again.
    let projected = flat.clone();
    let mut local = vec![[DVec2::ZERO; 3]; n];
    let mut sizes: Vec<DVec2> = charts
        .iter()
        .map(|m| lay(m, &projected, &mut local))
        .collect();
    let pad = padding / f64::from(resolution);
    let mut corners = vec![[DVec2::ZERO; 3]; n];
    let mut scale = 0.0;
    for round in 0..16 {
        let (at, sc) = pack_all(&sizes, pad).ok_or(UnwrapError::TooManyCharts(charts.len()))?;
        scale = sc;
        for (c, members) in charts.iter().enumerate() {
            let at = at[c] + DVec2::splat(pad / 2.0);
            for &i in members {
                corners[i as usize] = local[i as usize].map(|q| at + q * scale);
            }
        }
        let mut leaving: Vec<u32> = Vec::new();
        for members in &charts {
            for (i, j) in overlapping(members, &corners) {
                // The later leaves; after three rounds, both do, so this
                // ends (a chart of one triangle overlaps nothing).
                leaving.push(i.max(j));
                if round >= 3 {
                    leaving.push(i.min(j));
                }
            }
        }
        if leaving.is_empty() {
            break;
        }
        if round == 15 {
            return Err(UnwrapError::BadDesc(
                "charts still overlap after rounding; this is a bug".into(),
            ));
        }
        leaving.sort_unstable();
        leaving.dedup();
        for members in &mut charts {
            members.retain(|i| leaving.binary_search(i).is_err());
        }
        for &i in &leaving {
            charts.push(vec![i]);
        }
        charts.retain(|m| !m.is_empty());
        for (c, members) in charts.iter().enumerate() {
            for &i in members {
                chart_of[i as usize] = c as u32;
            }
        }
        sizes = charts
            .iter()
            .map(|m| lay(m, &projected, &mut local))
            .collect();
    }
    Ok(Uvs {
        corners,
        charts: chart_of,
        chart_count: charts.len(),
        scale,
    })
}

/// Turns a chart to its smallest box, laid wide, from the origin; writes
/// its triangles' coordinates into `local` and returns the box's size.
pub(crate) fn lay(members: &[u32], projected: &[[DVec2; 3]], local: &mut [[DVec2; 3]]) -> DVec2 {
    let pts: Vec<DVec2> = members
        .iter()
        .flat_map(|&i| projected[i as usize])
        .collect();
    let (e, p) = best_axes(&pts);
    let mut q: Vec<DVec2> = pts.iter().map(|x| DVec2::new(x.dot(e), x.dot(p))).collect();
    let (mut lo, mut hi) = (q[0], q[0]);
    for x in &q {
        lo = lo.min(*x);
        hi = hi.max(*x);
    }
    let size = hi - lo;
    let wide = size.x >= size.y;
    for x in &mut q {
        *x -= lo;
        if !wide {
            // A quarter turn: (x, y) ↦ (h − y, x), winding kept.
            *x = DVec2::new(size.y - x.y, x.x);
        }
    }
    for (k, &i) in members.iter().enumerate() {
        local[i as usize] = [q[k * 3], q[k * 3 + 1], q[k * 3 + 2]];
    }
    if wide {
        size
    } else {
        DVec2::new(size.y, size.x)
    }
}

/// Pairs of triangles of one chart that overlap, by a sweep along u with
/// exact tests.
pub(crate) fn overlapping(members: &[u32], uv: &[[DVec2; 3]]) -> Vec<(u32, u32)> {
    let bounds = |i: u32| {
        let t = uv[i as usize];
        (t[0].min(t[1]).min(t[2]), t[0].max(t[1]).max(t[2]))
    };
    let mut order: Vec<u32> = members.to_vec();
    order.sort_by(|&a, &b| bounds(a).0.x.total_cmp(&bounds(b).0.x).then(a.cmp(&b)));
    let mut active: Vec<u32> = Vec::new();
    let mut out = Vec::new();
    for &i in &order {
        let (lo, hi) = bounds(i);
        active.retain(|&j| bounds(j).1.x > lo.x);
        for &j in &active {
            let (lo2, hi2) = bounds(j);
            if lo2.y < hi.y && hi2.y > lo.y && tris_overlap_2d(uv[i as usize], uv[j as usize]) {
                out.push((i, j));
            }
        }
        active.push(i);
    }
    out
}

/// The axes (unit `e` and its left normal) along which the points' box is
/// smallest: one hull edge's direction, by rotating calipers.
pub(crate) fn best_axes(pts: &[DVec2]) -> (DVec2, DVec2) {
    let hull = hull(pts);
    let mut best = (f64::INFINITY, DVec2::X);
    for k in 0..hull.len() {
        let d = hull[(k + 1) % hull.len()] - hull[k];
        let len = d.length();
        if len == 0.0 {
            continue;
        }
        let e = d / len;
        let p = DVec2::new(-e.y, e.x);
        let (mut a0, mut a1, mut b0, mut b1) = (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        );
        for q in &hull {
            let (a, b) = (q.dot(e), q.dot(p));
            a0 = a0.min(a);
            a1 = a1.max(a);
            b0 = b0.min(b);
            b1 = b1.max(b);
        }
        let area = (a1 - a0) * (b1 - b0);
        if super::exceeds(best.0, area) {
            best = (area, e);
        }
    }
    let e = best.1;
    (e, DVec2::new(-e.y, e.x))
}

/// The convex hull, counter-clockwise, by the monotone chain with exact
/// turns.
pub(crate) fn hull(pts: &[DVec2]) -> Vec<DVec2> {
    let mut p: Vec<DVec2> = pts.to_vec();
    p.sort_by(|a, b| crate::math::order::cmp_dvec2(*a, *b));
    p.dedup();
    if p.len() < 3 {
        return p;
    }
    let mut lower: Vec<DVec2> = Vec::new();
    for &q in &p {
        while lower.len() >= 2
            && orient2d(lower[lower.len() - 2], lower[lower.len() - 1], q) != Ordering::Greater
        {
            lower.pop();
        }
        lower.push(q);
    }
    let mut upper: Vec<DVec2> = Vec::new();
    for &q in p.iter().rev() {
        while upper.len() >= 2
            && orient2d(upper[upper.len() - 2], upper[upper.len() - 1], q) != Ordering::Greater
        {
            upper.pop();
        }
        upper.push(q);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}
