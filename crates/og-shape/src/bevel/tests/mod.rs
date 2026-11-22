use super::*;
use crate::bevel::layout;
use crate::edit::{Hole, above, hole};
use crate::generators::solids::{BoxDesc, CylinderDesc, Extrude, cuboid, cylinder, extrude};
use crate::math::DVec2;

mod basics;
mod crossing;
mod layouts;
mod wide;

pub(crate) fn block() -> Solid {
    cuboid(&BoxDesc {
        width: 40.0,
        depth: 30.0,
        height: 20.0,
    })
    .unwrap()
}

pub(crate) fn names(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

/// Closed, and every triangle named `kind` pairs into a quad.
pub(crate) fn check(s: &Solid, kind: &str) -> QuadPairs {
    let m = s.measure();
    assert!(m.closed && m.degenerate_triangles == 0, "{m:?}");
    let f = s.faces(&[kind]).unwrap();
    let q = quad_pairs(s, &f);
    assert_eq!(q.paired, q.triangles, "{q:?}");
    q
}

/// What an n-segment arc loses against the true quarter circle, per unit
/// length of a square edge of radius r.
pub(crate) fn deficit(r: f64, n: u32) -> f64 {
    r * r * (trig::PI / 4.0 - n as f64 * trig::sin(trig::PI / (2.0 * n as f64)) / 2.0)
}

pub(crate) fn check_closed(s: &Solid) {
    let m = s.measure();
    assert!(m.closed && m.degenerate_triangles == 0, "{m:?}");
}

/// A part revolve (2 rad about z) of the section given, `(r, z)` corners.
pub(crate) fn part_ring(section: &[(f64, f64)]) -> Solid {
    crate::generators::solids::revolve(
        &crate::profile::Profile::from_contour(crate::profile::Contour::corners(
            section.iter().map(|&(x, y)| DVec2::new(x, y)).collect(),
        )),
        &crate::generators::solids::Revolve {
            angle: 2.0,
            axis: crate::generators::solids::Axis::Z,
            ..Default::default()
        },
    )
    .unwrap()
}

/// Names the real edges whose ends both pass `pick`.
pub(crate) fn tag(s: &Solid, name: &str, pick: impl Fn(crate::math::DVec3) -> bool) -> Solid {
    let mut out = s.clone();
    for (a, b) in s.real_edges() {
        if pick(s.positions[a as usize]) && pick(s.positions[b as usize]) {
            out.tag_edge(a, b, &[name]);
        }
    }
    out
}

// Bevels wider than the segments of their faces (`docs/research.md` §7.6).

/// No two triangles cross.
pub(crate) fn uncrossed(s: &Solid) {
    let c = crate::boolean::selfcheck::self_crossings(&s.positions, &s.triangles);
    assert!(c.is_empty(), "{} crossing pairs", c.len());
}

/// Canonical: canonicalizing again changes nothing.
pub(crate) fn canonical(s: &Solid) {
    let mut again = s.clone();
    again.canonicalize().unwrap();
    assert_eq!(&again, s);
}

/// Every new face carries only its kind's name, and some edge is named
/// with it and with each cap name given.
pub(crate) fn named(s: &Solid, kind: &str, caps: &[&str]) {
    for t in s.faces(&[kind]).unwrap() {
        assert_eq!(s.face_tags[t as usize].len(), 1);
    }
    let edges = s.edges(&[kind]).unwrap();
    for cap in caps {
        let tag = s.names.get(cap).unwrap();
        assert!(
            edges.iter().any(|e| s.edge_tags[e].contains(&tag)),
            "no {kind} edge also named {cap}"
        );
    }
}

/// The distance from `p` to the nearest triangle of `tris`.
pub(crate) fn to_surface(p: crate::math::DVec3, tris: &[[crate::math::DVec3; 3]]) -> f64 {
    tris.iter()
        .map(|&[a, b, c]| {
            let n = (b - a).cross(c - a).normalize_or_zero();
            let h = (p - a).dot(n);
            let q = p - n * h;
            let inside = [(a, b), (b, c), (c, a)]
                .iter()
                .all(|&(x, y)| (y - x).cross(q - x).dot(n) >= -1e-9);
            if inside {
                h.abs()
            } else {
                [(a, b), (b, c), (c, a)]
                    .iter()
                    .map(|&(x, y)| {
                        let t = ((p - x).dot(y - x) / (y - x).length_squared()).clamp(0.0, 1.0);
                        (p - (x + (y - x) * t)).length()
                    })
                    .fold(f64::INFINITY, f64::min)
            }
        })
        .fold(f64::INFINITY, f64::min)
}

// Results that crossed themselves or were otherwise wrong
// (`docs/research.md` §7.7).

/// Every triangle not named `kind` lies in the plane of an input triangle
/// with its names (no old face tilted; a concave bevel's end may still
/// fill a corner outside the old face, in its plane).
pub(crate) fn in_planes(s: &Solid, out: &Solid, kind: &str) {
    let new: std::collections::BTreeSet<u32> = out.faces(&[kind]).unwrap().into_iter().collect();
    for (i, t) in out.triangles.iter().enumerate() {
        if new.contains(&(i as u32)) {
            continue;
        }
        let p = t.map(|v| out.positions[v as usize]);
        let fits = s.triangles.iter().enumerate().any(|(j, u)| {
            let [a, b, c] = u.map(|v| s.positions[v as usize]);
            let n = (b - a).cross(c - a).normalize_or_zero();
            let tags = |x: &Solid, k: usize| -> Vec<String> {
                x.face_tags[k]
                    .iter()
                    .map(|&g| x.names.name(g).to_string())
                    .collect()
            };
            tags(s, j) == tags(out, i) && p.iter().all(|&q| (q - a).dot(n).abs() < 1e-9)
        });
        assert!(fits, "{p:?} off its face's planes");
    }
}
