//! The convex hull of points in space: [`convex_hull`].
//!
//! Incremental, with every side decided by `orient3d` (exact), so the
//! hull is the same on every host and no point is ever left outside it.
//! The points are taken in order of position; a point is added when it
//! lies strictly outside a face, and then every face it sees is replaced
//! by a fan from it to the rim of what it sees. A point in the plane of a
//! face is not outside it: it joins the hull only through a neighbouring
//! face it is strictly outside of, as a triangle in that same plane.
//!
//! The result is a closed solid of triangles. Triangles in one plane that
//! share an edge are one surface, so a box's hull has six surfaces and
//! twelve real edges. Every face is named `hull`; every real edge too.
//!
//! Cost: each point is tested against every face of the hull so far, so
//! it grows with points × hull faces. A hundred thousand points of a
//! scanned surface take a few seconds; a model's parts, milliseconds.

use core::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::math::DVec3;
use crate::math::predicates::{orient2d, orient3d};
use crate::mesh::Solid;

use crate::generators::solids::GenError;

/// The convex hull of `points`, as a closed solid. Refuses points that
/// are not finite, and points that all lie in one plane (a hull with no
/// volume), saying which.
pub fn convex_hull(points: &[DVec3]) -> Result<Solid, GenError> {
    if points.iter().any(|p| !p.is_finite()) {
        return Err(GenError::NotFinite("hull points"));
    }
    // In order of position, each once (−0 and 0 are one point).
    let mut pts: Vec<DVec3> = points
        .iter()
        .map(|p| DVec3::new(p.x + 0.0, p.y + 0.0, p.z + 0.0))
        .collect();
    pts.sort_by(cmp_point);
    pts.dedup();
    let mut faces = faces_of(&pts)?;
    let mut surface = surfaces(&pts, &faces)?;
    // A point on an edge or inside a face of the hull is on its surface
    // but is no corner of it: corners meet three planes or more. Build
    // the hull again from its corners alone, so it has no vertex it does
    // not need.
    let mut planes: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
    for (t, f) in faces.iter().enumerate() {
        for &v in f {
            planes.entry(v).or_default().insert(surface[t]);
        }
    }
    if planes.values().any(|s| s.len() < 3) {
        pts = planes
            .iter()
            .filter(|(_, s)| s.len() >= 3)
            .map(|(&v, _)| pts[v])
            .collect();
        faces = faces_of(&pts)?;
        surface = surfaces(&pts, &faces)?;
    }

    // Only the hull's own vertices, in order of position.
    let used: BTreeSet<usize> = faces.iter().flatten().copied().collect();
    let index: BTreeMap<usize, u32> = used
        .iter()
        .enumerate()
        .map(|(k, &i)| (i, k as u32))
        .collect();
    let mut s = Solid::default();
    for &i in &used {
        s.push_vertex(pts[i]);
    }
    let mut ids: BTreeMap<usize, u32> = BTreeMap::new();
    for (t, f) in faces.iter().enumerate() {
        let next = ids.len() as u32;
        let id = *ids.entry(surface[t]).or_insert(next);
        s.push_triangle(f.map(|i| index[&i]), id, &["hull"]);
    }
    for (u, v) in s.real_edges() {
        s.tag_edge(u, v, &["hull"]);
    }
    s.canonicalize()
        .map_err(|_| GenError::NotFinite("hull points"))?;
    Ok(s)
}

/// The hull's triangles over `pts` (in order of position, each once),
/// counter-clockwise seen from outside.
fn faces_of(pts: &[DVec3]) -> Result<Vec<[usize; 3]>, GenError> {
    let flat = || GenError::Invalid("a hull needs points that are not all in one plane".into());
    if pts.len() < 4 {
        return Err(flat());
    }
    // The first tetrahedron: the first point, the next that differs, the
    // next off their line, the next off their plane.
    let a = 0;
    let b = 1;
    let c = (2..pts.len())
        .find(|&i| !collinear(pts[a], pts[b], pts[i]))
        .ok_or_else(flat)?;
    let d = (2..pts.len())
        .find(|&i| i != c && orient3d(pts[a], pts[b], pts[c], pts[i]) != Ordering::Equal)
        .ok_or_else(flat)?;
    // A face is counter-clockwise seen from outside: the rest of the hull
    // is below it, which `orient3d` calls `Greater`.
    let (a, b, c) = if orient3d(pts[a], pts[b], pts[c], pts[d]) == Ordering::Greater {
        (a, b, c)
    } else {
        (a, c, b)
    };
    let mut faces: Vec<[usize; 3]> = vec![[a, b, c], [b, a, d], [c, b, d], [a, c, d]];
    for p in 0..pts.len() {
        if [a, b, c, d].contains(&p) {
            continue;
        }
        let sees: Vec<bool> = faces
            .iter()
            .map(|f| orient3d(pts[f[0]], pts[f[1]], pts[f[2]], pts[p]) == Ordering::Less)
            .collect();
        if !sees.iter().any(|&s| s) {
            continue;
        }
        // The rim: directed edges of seen faces whose twin is not seen.
        let seen: BTreeSet<(usize, usize)> = faces
            .iter()
            .zip(&sees)
            .filter(|(_, s)| **s)
            .flat_map(|(f, _)| [(f[0], f[1]), (f[1], f[2]), (f[2], f[0])])
            .collect();
        let mut kept: Vec<[usize; 3]> = faces
            .iter()
            .zip(&sees)
            .filter(|(_, s)| !**s)
            .map(|(f, _)| *f)
            .collect();
        kept.extend(
            seen.iter()
                .filter(|(u, v)| !seen.contains(&(*v, *u)))
                .map(|&(u, v)| [u, v, p]),
        );
        faces = kept;
    }
    Ok(faces)
}

/// Per triangle, its surface: triangles in one plane across an edge share
/// one (the least triangle index among them).
fn surfaces(pts: &[DVec3], faces: &[[usize; 3]]) -> Result<Vec<usize>, GenError> {
    let twin: BTreeMap<(usize, usize), usize> = faces
        .iter()
        .enumerate()
        .flat_map(|(t, f)| [(f[0], f[1]), (f[1], f[2]), (f[2], f[0])].map(|e| (e, t)))
        .collect();
    let mut surface: Vec<usize> = (0..faces.len()).collect();
    fn root(s: &mut [usize], mut i: usize) -> usize {
        while s[i] != i {
            s[i] = s[s[i]];
            i = s[i];
        }
        i
    }
    for (t, f) in faces.iter().enumerate() {
        for k in 0..3 {
            let (u, v, w) = (f[k], f[(k + 1) % 3], f[(k + 2) % 3]);
            let Some(&o) = twin.get(&(v, u)) else {
                return Err(GenError::Invalid("the hull did not close".into()));
            };
            let far = faces[o].iter().copied().find(|&x| x != u && x != v);
            let coplanar =
                far.is_some_and(|x| orient3d(pts[u], pts[v], pts[w], pts[x]) == Ordering::Equal);
            if coplanar {
                let (ra, rb) = (root(&mut surface, t), root(&mut surface, o));
                surface[ra.max(rb)] = ra.min(rb);
            }
        }
    }
    Ok((0..faces.len()).map(|t| root(&mut surface, t)).collect())
}

fn cmp_point(a: &DVec3, b: &DVec3) -> Ordering {
    a.x.total_cmp(&b.x)
        .then(a.y.total_cmp(&b.y))
        .then(a.z.total_cmp(&b.z))
}

/// Whether three points lie on one line: they do when each of their
/// three axis projections is collinear.
fn collinear(a: DVec3, b: DVec3, c: DVec3) -> bool {
    use crate::math::DVec2;
    let flat = |f: fn(DVec3) -> DVec2| orient2d(f(a), f(b), f(c)) == Ordering::Equal;
    flat(|p| DVec2::new(p.x, p.y))
        && flat(|p| DVec2::new(p.y, p.z))
        && flat(|p| DVec2::new(p.x, p.z))
}
