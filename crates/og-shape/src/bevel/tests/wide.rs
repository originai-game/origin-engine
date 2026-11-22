//! Bevels wider than the segments of their faces (`docs/research.md` §7.6).

use super::*;

/// Volume of a 2-radian part revolve of the rectangle r 5–10, z 0–8,
/// every edge rounded by `rho` (a rolling ball: every edge is square, so
/// the result is the eroded body grown back by `rho`; Steiner's formula,
/// the eroded body's area by the midpoint rule).
pub(crate) fn ring_rounded(rho: f64) -> f64 {
    let (a, b, h, ang) = (5.0 + rho, 10.0 - rho, 8.0 - 2.0 * rho, 2.0);
    let al = |r: f64| trig::asin(rho / r);
    let ext = |r: f64| ang - 2.0 * al(r);
    let n = 20_000;
    let step = (b - a) / n as f64;
    let area: f64 = (0..n)
        .map(|i| {
            let r = a + (i as f64 + 0.5) * step;
            r * ext(r) * step
        })
        .sum();
    let side = (b * b - rho * rho).sqrt() - (a * a - rho * rho).sqrt();
    let per = a * ext(a) + b * ext(b) + 2.0 * side;
    let mean = -ext(a) * h / 2.0
        + ext(b) * h / 2.0
        + trig::PI / 2.0 * per
        + h * (trig::PI + al(b) - al(a));
    area * h
        + rho * (2.0 * area + h * per)
        + rho * rho * mean
        + 4.0 * trig::PI / 3.0 * rho * rho * rho
}

#[test]
fn a_bevel_wider_than_the_bore_s_segments() {
    // The part revolve's bore at r 5 has segments 0.91 long; a 1.5
    // fillet on its start and end edges swallows the rims' first
    // vertices, which slide along the rims past the corner's reach.
    let ring = part_ring(&[(5.0, 0.0), (10.0, 0.0), (10.0, 8.0), (5.0, 8.0)]);
    let v0 = ring.measure().volume;
    for (rho, lo, hi) in [(1.0, 1.0, 1.1), (1.5, 1.0, 1.1)] {
        let s = fillet(
            &ring,
            &Fillet {
                radius: rho,
                segments: 4,
                ..Default::default()
            },
        )
        .unwrap();
        check_closed(&s);
        uncrossed(&s);
        canonical(&s);
        named(&s, "fillet", &["start", "end"]);
        // Against the smooth rounded body: the 4-segment profile cuts 9 %
        // more than a circle (as below r 1, where the classic layout fits).
        let removed = v0 - s.measure().volume;
        let want = 600.0 - ring_rounded(rho);
        assert!(
            removed > lo * want && removed < hi * want,
            "{removed} {want}"
        );
        let q = quad_pairs(&s, &s.faces(&["fillet"]).unwrap());
        assert!(q.paired * 10 >= q.triangles * 9, "{q:?}");
    }
    // The same profile's excess where the classic layout fits.
    let s = fillet(
        &ring,
        &Fillet {
            radius: 0.5,
            segments: 4,
            ..Default::default()
        },
    )
    .unwrap();
    let excess = (v0 - s.measure().volume) / (600.0 - ring_rounded(0.5));
    assert!(excess > 1.09 && excess < 1.1, "{excess}");
    let c = chamfer(
        &ring,
        &Chamfer {
            distance: 1.5,
            ..Default::default()
        },
    )
    .unwrap();
    check_closed(&c);
    uncrossed(&c);
    named(&c, "chamfer", &["start", "end"]);
}

/// A 30 × 30 × 10 plate with a 6 mm hole through it: by the hole tool (a
/// boolean, whose rims come in pieces down to 1e-15 long), and extruded
/// from an outline with the same 24-gon hole.
pub(crate) fn plates() -> (Solid, Solid) {
    let plate = cuboid(&BoxDesc {
        width: 30.0,
        depth: 30.0,
        height: 10.0,
    })
    .unwrap();
    let cut = hole(
        &plate,
        &Hole {
            diameter: 6.0,
            at: vec![above(0.0, 0.0)],
            ..Default::default()
        },
    )
    .unwrap();
    let mut ring: Vec<DVec2> = (0..24)
        .map(|k| {
            let (sin, cos) = trig::sin_cos(trig::TAU * k as f64 / 24.0);
            DVec2::new(3.0 * cos, 3.0 * sin)
        })
        .collect();
    ring.reverse();
    let outline = crate::profile::Profile {
        regions: vec![crate::profile::Region {
            outer: crate::profile::Contour::corners(vec![
                DVec2::new(-15.0, -15.0),
                DVec2::new(15.0, -15.0),
                DVec2::new(15.0, 15.0),
                DVec2::new(-15.0, 15.0),
            ]),
            holes: vec![crate::profile::Contour::smooth(ring)],
        }],
    };
    let plain = extrude(
        &outline,
        &Extrude {
            height: 10.0,
            ..Default::default()
        },
    )
    .unwrap();
    let plain = tag(&plain, "hole", |p| p.truncate().length() < 3.0 + 1e-9);
    (cut, plain)
}

#[test]
fn a_hole_s_rims_wider_than_their_pieces() {
    let (cut, plain) = plates();
    // Rims in pieces down to 1e-15 long: needles whose f64 area is zero.
    assert!(cut.triangles.iter().any(|t| {
        let [a, b, c] = t.map(|v| cut.positions[v as usize]);
        (b - a).cross(c - a) == DVec3::ZERO
    }));
    for r in [0.5, 1.5, 2.5] {
        let round = |s: &Solid, edges: &[&str]| {
            fillet(
                s,
                &Fillet {
                    radius: r,
                    segments: 4,
                    edges: names(edges),
                    ..Default::default()
                },
            )
            .unwrap()
        };
        let a = round(&cut, &["hole"]);
        let b = round(&plain, &["hole"]);
        check(&a, "fillet");
        uncrossed(&a);
        canonical(&a);
        named(&a, "fillet", &["top", "bottom"]);
        // The pieces and the points 1e-15 apart are merged on the rims'
        // own lines: the same volume as the plain plate's.
        let (va, vb) = (a.measure().volume, b.measure().volume);
        assert!((va - vb).abs() < 1e-9 * vb, "{va} {vb}");
        // One rim and the plate's edges with it.
        let t = round(&cut, &["top"]);
        check_closed(&t);
        uncrossed(&t);
    }
}

#[test]
fn a_cylinder_s_top_rim_chamfered_over_several_segments() {
    // A 12-gon cylinder, r 3: its wall segments are 1.55 wide. A chamfer
    // 2 deep (three profile segments) fitted before; its volume is the
    // frustum's, P d² / 2 − n tan(π/n) d³ / 3.
    let c = cylinder(&CylinderDesc {
        radius: 3.0,
        height: 10.0,
        segments: 12,
        ..Default::default()
    })
    .unwrap();
    let d = 2.0;
    let s = chamfer(
        &c,
        &Chamfer {
            distance: d,
            segments: 3,
            edges: names(&["top"]),
            ..Default::default()
        },
    )
    .unwrap();
    check(&s, "chamfer");
    let n = 12.0;
    let per = n * 2.0 * 3.0 * trig::sin(trig::PI / n);
    let want = per * d * d / 2.0 - n * trig::tan(trig::PI / n) * d * d * d / 3.0;
    let removed = c.measure().volume - s.measure().volume;
    assert!((removed - want).abs() < 1e-9 * want, "{removed} {want}");
    // A slice of it, 2 radians: the cut faces' vertical edges are wider
    // than a wall segment once their chamfer is 1, and the rims' first
    // vertices slide past it.
    let pie = crate::generators::solids::revolve(
        &crate::profile::Profile::from_contour(crate::profile::Contour::corners(vec![
            DVec2::new(0.0, 0.0),
            DVec2::new(5.0, 0.0),
            DVec2::new(5.0, 8.0),
            DVec2::new(0.0, 8.0),
        ])),
        &crate::generators::solids::Revolve {
            angle: 2.0,
            axis: crate::generators::solids::Axis::Z,
            segments: 24,
        },
    )
    .unwrap();
    for d in [1.0, 1.5] {
        let s = chamfer(
            &pie,
            &Chamfer {
                distance: d,
                segments: 3,
                ..Default::default()
            },
        )
        .unwrap();
        check_closed(&s);
        uncrossed(&s);
        canonical(&s);
        named(&s, "chamfer", &["start", "end"]);
    }
}

#[test]
fn a_twisted_prism_s_top_edges_cut_across_its_rows() {
    // Each side of a square twisted 1 rad is seven rows of twisted quads,
    // 2.86 high; a 5 fillet on the top edges crosses the first row.
    let square = crate::profile::Profile::from_contour(crate::profile::Contour::corners(vec![
        DVec2::new(-10.0, -10.0),
        DVec2::new(10.0, -10.0),
        DVec2::new(10.0, 10.0),
        DVec2::new(-10.0, 10.0),
    ]));
    let twisted = extrude(
        &square,
        &Extrude {
            height: 20.0,
            twist: 1.0,
            ..Default::default()
        },
    )
    .unwrap();
    let v0 = twisted.measure().volume;
    let sides: Vec<[crate::math::DVec3; 3]> = twisted
        .faces(&["side"])
        .unwrap()
        .iter()
        .map(|&t| twisted.triangles[t as usize].map(|v| twisted.positions[v as usize]))
        .collect();
    let mut per_r2 = Vec::new();
    for r in [4.5, 5.0, 5.5] {
        let s = fillet(
            &twisted,
            &Fillet {
                radius: r,
                segments: 4,
                edges: names(&["top"]),
                ..Default::default()
            },
        )
        .unwrap();
        check_closed(&s);
        uncrossed(&s);
        canonical(&s);
        named(&s, "fillet", &["top"]);
        // The sides keep their own triangles' planes, cut where the
        // offset line crosses them.
        for t in s.faces(&["side"]).unwrap() {
            for v in s.triangles[t as usize] {
                let p = s.positions[v as usize];
                assert!(to_surface(p, &sides) < 1e-9, "{p}");
            }
        }
        let q = quad_pairs(&s, &s.faces(&["fillet"]).unwrap());
        eprintln!("r {r} {q:?} removed {}", v0 - s.measure().volume);
        assert!(q.paired * 10 >= q.triangles * 8, "{q:?}");
        per_r2.push((v0 - s.measure().volume) / (r * r));
        // A fillet cuts less than the chamfer of the same offsets.
        let c = chamfer(
            &twisted,
            &Chamfer {
                distance: r,
                edges: names(&["top"]),
                ..Default::default()
            },
        )
        .unwrap();
        check_closed(&c);
        uncrossed(&c);
        assert!(c.measure().volume < s.measure().volume);
    }
    // The cut grows with r² while it crosses the same row.
    for w in per_r2.windows(2) {
        assert!((w[0] - w[1]).abs() < 0.02 * w[0], "{per_r2:?}");
    }
}
