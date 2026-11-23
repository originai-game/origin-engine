//! Roofs on a face: flat, a gable on a rectangle, and a hip on any convex
//! face by its straight skeleton (Aichholzer, Aurenhammer, Alberts and
//! Gärtner, *A Novel Type of Skeleton for Polygons*, 1995), traced by the
//! wavefront's edge events (Felkel and Obdržálek, *Straight Skeleton
//! Implementation*, 1998): the only events a convex polygon has. Each edge
//! moves inwards at unit speed along its normal; when an edge shrinks to a
//! point its neighbours meet there, and the roof's height at a point is
//! the time the wavefront reached it times the slope.

use glam::{DVec2, DVec3};
use og_core::math::double;

use super::RoofKind;
use super::face::{self, EPS, Face, Uv};

/// The tag of a roof's children's ids.
const TAG: u64 = 4;

/// What a roof's face is.
pub(super) enum Role {
    Slope,
    Gable,
    Eave,
}

/// The roof on `f` (a face facing up or out), `angle` degrees steep with
/// eaves `overhang` beyond its edges; `base` is the lot's height, from
/// which a gable's texture reads its height.
pub(super) fn roof(
    f: &Face,
    kind: RoofKind,
    angle: f64,
    overhang: f64,
    base: f64,
) -> Result<Vec<(Role, Face)>, String> {
    if kind == RoofKind::Flat {
        return Ok(vec![(
            Role::Slope,
            f.with_poly(f.poly.clone(), f.child(TAG, 0)),
        )]);
    }
    if !(angle > 0.0 && angle < 90.0) {
        return Err(format!(
            "a roof's angle is above 0 and under 90 degrees, not {angle}"
        ));
    }
    if !(overhang >= 0.0 && overhang.is_finite()) {
        return Err(format!("a roof's overhang is 0 or more, not {overhang}"));
    }
    let pitch = double::tan(double::to_radians(angle));
    let mut out = Vec::new();
    if kind == RoofKind::Gable && oblong(&f.poly) {
        gable(f, pitch, overhang, base, &mut out)?;
    } else {
        if !face::convex(&f.poly) {
            return Err(
                "a hip roof needs a convex face: split the face into convex parts first".into(),
            );
        }
        hip(f, pitch, overhang, &mut out)?;
    }
    if overhang > 0.0 {
        let slopes: Vec<Face> = out
            .iter()
            .filter(|(r, _)| matches!(r, Role::Slope))
            .map(|(_, s)| s.clone())
            .collect();
        for s in slopes {
            let id = f.child(TAG, out.len() as u64);
            out.push((Role::Eave, s.flipped(id)));
        }
    }
    Ok(out)
}

/// A rectangle whose sides differ (a square takes a hip roof).
fn oblong(poly: &[DVec2]) -> bool {
    poly.len() == 4
        && (0..4).all(|i| {
            face::edge(poly, i)
                .0
                .dot(face::edge(poly, (i + 1) % 4).0)
                .abs()
                < 1e-9
        })
        && (poly[0].distance(poly[1]) - poly[1].distance(poly[2])).abs() > EPS
}

/// A planar face through `points` (counter-clockwise seen from where it
/// faces), its x along `x`; `None` when fewer than three distinct points
/// are left.
fn planar(points: &[DVec3], x: DVec3, uv: Uv, id: u64, detail: u32) -> Option<Face> {
    let mut pts: Vec<DVec3> = Vec::with_capacity(points.len());
    for &p in points {
        if pts.last().is_none_or(|q| q.distance(p) > EPS) {
            pts.push(p);
        }
    }
    while pts.len() > 1 && pts[0].distance(pts[pts.len() - 1]) <= EPS {
        pts.pop();
    }
    if pts.len() < 3 {
        return None;
    }
    // Newell's normal: the sum of the edges' cross products.
    let n = (0..pts.len())
        .map(|i| pts[i].cross(pts[(i + 1) % pts.len()]))
        .fold(DVec3::ZERO, |a, b| a + b)
        .normalize_or_zero();
    if n == DVec3::ZERO {
        return None;
    }
    let x = (x - n * x.dot(n)).normalize();
    let y = n.cross(x);
    let o = pts[0];
    let poly = pts
        .iter()
        .map(|p| DVec2::new((*p - o).dot(x), (*p - o).dot(y)))
        .collect();
    Some(Face::new(o, x, y, poly, uv, id, detail))
}

/// The perimeter's length to each corner, from the first.
fn perimeter(poly: &[DVec2]) -> Vec<f64> {
    let mut u = 0.0;
    (0..poly.len())
        .map(|i| {
            let at = u;
            u += poly[i].distance(poly[(i + 1) % poly.len()]);
            at
        })
        .collect()
}

/// Two slopes along the longer sides, meeting at a level ridge, and a
/// gable over each shorter side at the face's own edge.
fn gable(
    f: &Face,
    pitch: f64,
    ov: f64,
    base: f64,
    out: &mut Vec<(Role, Face)>,
) -> Result<(), String> {
    let o = &f.poly;
    let s = usize::from(o[0].distance(o[1]) > o[1].distance(o[2]));
    let r = if ov > 0.0 {
        face::offset(o, -ov).ok_or("the overhang turns the roof inside out")?
    } else {
        o.clone()
    };
    let lift = |q: DVec2, h: f64| f.world(q) + f.n * h;
    let ridge = o[s].distance(o[s + 1]) / 2.0 * pitch;
    let eave = -ov * pitch;
    let at = |i: usize| r[i % 4];
    let near = lift((at(s) + at(s + 1)) / 2.0, ridge);
    let far = lift((at(s + 2) + at(s + 3)) / 2.0, ridge);
    let u = perimeter(&r);
    for (edge, top) in [(s + 1, [far, near]), (s + 3, [near, far])] {
        let (a, b) = (at(edge), at(edge + 1));
        let pts = [lift(a, eave), lift(b, eave), top[0], top[1]];
        let x = f.along((b - a).normalize());
        let id = f.child(TAG, out.len() as u64);
        if let Some(slope) = planar(&pts, x, Uv::wall(u[edge % 4], 0.0), id, f.detail) {
            out.push((Role::Slope, slope));
        }
    }
    let u = perimeter(o);
    for edge in [s, s + 2] {
        let (a, b) = (o[edge], o[(edge + 1) % 4]);
        let (e, inward) = face::edge(o, edge);
        let uv = if f.n.y > 1.0 - 1e-9 {
            Uv::wall(u[edge], f.world(a).y - base)
        } else {
            f.uv.turned(a, e, -inward)
        };
        let pts = [lift(a, 0.0), lift(b, 0.0), lift((a + b) / 2.0, ridge)];
        let id = f.child(TAG, out.len() as u64);
        if let Some(g) = planar(&pts, f.along(e), uv, id, f.detail) {
            out.push((Role::Gable, g));
        }
    }
    Ok(())
}

/// An edge's line, `n · p = c + t` at time `t`: its inward normal and
/// offset.
type Line = (DVec2, f64);

/// When three moving lines pass through one point.
fn concurrent(l: &[Line], a: usize, b: usize, c: usize) -> Option<f64> {
    let det = |k: [f64; 3]| {
        let (na, nb, nc) = (l[a].0, l[b].0, l[c].0);
        na.x * (nb.y * k[2] - k[1] * nc.y) - na.y * (nb.x * k[2] - k[1] * nc.x)
            + k[0] * (nb.x * nc.y - nb.y * nc.x)
    };
    let d0 = det([-l[a].1, -l[b].1, -l[c].1]);
    let d1 = det([-1.0, -1.0, -1.0]);
    (d1.abs() > 1e-12).then_some(-d0 / d1)
}

/// Where the three lines meet at time `t`, from the two of them furthest
/// from parallel.
fn meet(l: &[Line], ids: [usize; 3], t: f64) -> DVec2 {
    let pairs = [(ids[0], ids[1]), (ids[1], ids[2]), (ids[0], ids[2])];
    let (a, b) = pairs
        .into_iter()
        .fold((pairs[0], 0.0), |best, (a, b)| {
            let d = l[a].0.perp_dot(l[b].0).abs();
            if d > best.1 { ((a, b), d) } else { best }
        })
        .0;
    let (na, nb) = (l[a].0, l[b].0);
    let (ca, cb) = (l[a].1 + t, l[b].1 + t);
    let d = na.perp_dot(nb);
    DVec2::new((ca * nb.y - cb * na.y) / d, (na.x * cb - nb.x * ca) / d)
}

/// A slope rising from every edge of a convex face, over its straight
/// skeleton.
fn hip(f: &Face, pitch: f64, ov: f64, out: &mut Vec<(Role, Face)>) -> Result<(), String> {
    let p = if ov > 0.0 {
        face::offset(&f.poly, -ov).ok_or("the overhang turns the roof inside out")?
    } else {
        f.poly.clone()
    };
    let m = p.len();
    let lines: Vec<Line> = (0..m)
        .map(|i| {
            let n = face::edge(&p, i).1;
            (n, n.dot(p[i]))
        })
        .collect();
    // The skeleton's nodes each edge's face reaches, from its end corner
    // and from its start corner, as the wavefront reached them.
    let mut starts: Vec<Vec<(DVec2, f64)>> = vec![Vec::new(); m];
    let mut ends: Vec<Vec<(DVec2, f64)>> = vec![Vec::new(); m];
    let mut active: Vec<usize> = (0..m).collect();
    let mut time = 0.0;
    while active.len() > 2 {
        let k = active.len();
        if k == 3 {
            let ids = [active[0], active[1], active[2]];
            let t = concurrent(&lines, ids[0], ids[1], ids[2])
                .ok_or("the roof's last edges never meet")?;
            let x = meet(&lines, ids, t);
            for e in ids {
                starts[e].push((x, t));
                ends[e].push((x, t));
            }
            break;
        }
        let mut next: Option<(f64, usize)> = None;
        for i in 0..k {
            let (a, e, b) = (active[(i + k - 1) % k], active[i], active[(i + 1) % k]);
            if let Some(t) = concurrent(&lines, a, e, b)
                && t >= time - 1e-9
                && next.is_none_or(|(best, _)| t < best - 1e-12)
            {
                next = Some((t, i));
            }
        }
        let (t, i) = next.ok_or("the roof's wavefront found no edge to close")?;
        let (a, e, b) = (active[(i + k - 1) % k], active[i], active[(i + 1) % k]);
        let x = meet(&lines, [a, e, b], t);
        starts[e].push((x, t));
        ends[e].push((x, t));
        ends[a].push((x, t));
        starts[b].push((x, t));
        active.remove(i);
        time = t;
    }
    let lift = |q: DVec2, t: f64| f.world(q) + f.n * ((t - ov) * pitch);
    let u = perimeter(&p);
    for i in 0..m {
        let (a, b) = (p[i], p[(i + 1) % m]);
        let mut pts = vec![lift(a, 0.0), lift(b, 0.0)];
        pts.extend(ends[i].iter().map(|&(q, t)| lift(q, t)));
        pts.extend(starts[i].iter().rev().map(|&(q, t)| lift(q, t)));
        let x = f.along((b - a).normalize());
        let id = f.child(TAG, out.len() as u64);
        if let Some(slope) = planar(&pts, x, Uv::wall(u[i], 0.0), id, f.detail) {
            out.push((Role::Slope, slope));
        }
    }
    Ok(())
}
