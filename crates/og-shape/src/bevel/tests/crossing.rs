//! Results that crossed themselves or were otherwise wrong (`docs/research.md` §7.7 to §7.9): refused inputs, tips at both ends, faces cut on their folds, rows shallower than the bevel.

use super::*;

pub(crate) fn moved(s: &Solid, by: crate::math::DVec3) -> Solid {
    let mut out = s.clone();
    out.map_positions(|p| p + by, false);
    out
}

pub(crate) fn op(a: &Solid, b: &Solid, op: crate::boolean::BoolOp) -> Solid {
    crate::boolean::boolean(
        a,
        b,
        &crate::boolean::Boolean {
            op,
            ..Default::default()
        },
    )
    .unwrap()
}

pub(crate) fn plate() -> Solid {
    cuboid(&BoxDesc {
        width: 20.0,
        depth: 20.0,
        height: 10.0,
    })
    .unwrap()
}

pub(crate) fn pin(r: f64, h: f64) -> Solid {
    cylinder(&CylinderDesc {
        radius: r,
        height: h,
        ..Default::default()
    })
    .unwrap()
}

#[test]
fn a_layout_that_crosses_itself_is_never_returned() {
    // A bore's bottom rim: the boolean leaves the plate's bottom edges in
    // pieces, and at 1.5 the classic layout crossed itself near the bore.
    // The final check sends it on to the straight-run retry, which fits.
    let bore = op(
        &plate(),
        &moved(&pin(4.0, 30.0), crate::math::DVec3::new(0.0, 0.0, -10.0)),
        crate::boolean::BoolOp::Cut,
    );
    let v0 = bore.measure().volume;
    let removed = |d: f64| {
        let s = chamfer(
            &bore,
            &Chamfer {
                distance: d,
                edges: names(&["bottom"]),
                ..Default::default()
            },
        )
        .unwrap();
        check_closed(&s);
        uncrossed(&s);
        v0 - s.measure().volume
    };
    // Every edge square, the same corners: the cut grows with d².
    let ratio = removed(1.5) / removed(0.5);
    assert!((ratio - 9.0).abs() < 0.2, "{ratio}");
}

#[test]
fn inputs_that_cross_themselves_or_are_not_manifold() {
    // A post joined onto a plate: the boolean leaves two rim points 1e-15
    // apart and a sliver across the plate's top triangle. The weld of a
    // wide bevel's pass 0 takes it out first.
    let post = op(
        &plate(),
        &moved(&pin(3.0, 8.0), crate::math::DVec3::new(0.0, 0.0, 10.0)),
        crate::boolean::BoolOp::Join,
    );
    let mut corner = post.clone();
    let (a, b) = post
        .real_edges()
        .into_iter()
        .find(|&(a, b)| {
            let (p, q) = (post.positions[a as usize], post.positions[b as usize]);
            p.x == -10.0 && p.y == -10.0 && q.x == -10.0 && q.y == -10.0
        })
        .unwrap();
    corner.tag_edge(a, b, &["pick"]);
    let s = fillet(
        &corner,
        &Fillet {
            radius: 0.5,
            edges: names(&["pick"]),
            segments: 4,
            ..Default::default()
        },
    )
    .unwrap();
    check_closed(&s);
    uncrossed(&s);
    // Two boxes overlapping, not joined: refused, never bevelled through.
    let mut two = plate();
    let other = moved(&plate(), crate::math::DVec3::new(5.0, 5.0, 5.0));
    let base = two.positions.len() as u32;
    for &p in &other.positions {
        two.push_vertex(p);
    }
    for (t, tri) in other.triangles.iter().enumerate() {
        let names: Vec<&str> = other.face_tags[t]
            .iter()
            .map(|&n| other.names.name(n))
            .collect();
        two.push_triangle(tri.map(|v| v + base), other.surfaces[t] + 10, &names);
    }
    let r = fillet(
        &two,
        &Fillet {
            radius: 1.0,
            ..Default::default()
        },
    );
    assert!(matches!(r, Err(BevelError::SelfCrossing(_))), "{r:?}");
    // Two boxes joined along an edge: four faces on one edge.
    let edge = op(
        &plate(),
        &moved(&plate(), crate::math::DVec3::new(20.0, 0.0, 10.0)),
        crate::boolean::BoolOp::Join,
    );
    let r = fillet(
        &edge,
        &Fillet {
            radius: 1.0,
            ..Default::default()
        },
    );
    assert!(
        matches!(r, Err(BevelError::NotManifold(p, q)) if p.x == 10.0 && q.x == 10.0 && p.z == 10.0 && q.z == 10.0),
        "{r:?}"
    );
}

pub(crate) fn twisted() -> Solid {
    let square = crate::profile::Profile::from_contour(crate::profile::Contour::corners(vec![
        DVec2::new(-10.0, -10.0),
        DVec2::new(10.0, -10.0),
        DVec2::new(10.0, 10.0),
        DVec2::new(-10.0, 10.0),
    ]));
    extrude(
        &square,
        &Extrude {
            height: 20.0,
            twist: 1.0,
            ..Default::default()
        },
    )
    .unwrap()
}

#[test]
fn a_twisted_prism_s_top_keeps_its_sides_at_every_radius() {
    // The side rows are twisted quads: moving the top corner down the
    // helical edge tilted the row's long triangle, and r 0.5 added 11 mm³.
    // The bands are cut instead, at every radius (§7.7).
    let t = twisted();
    let v0 = t.measure().volume;
    let sides: Vec<[crate::math::DVec3; 3]> = t
        .faces(&["side"])
        .unwrap()
        .iter()
        .map(|&i| t.triangles[i as usize].map(|v| t.positions[v as usize]))
        .collect();
    for r in [0.5, 1.0, 2.0, 3.0] {
        let f = fillet(
            &t,
            &Fillet {
                radius: r,
                segments: 4,
                edges: names(&["top"]),
                ..Default::default()
            },
        )
        .unwrap();
        let c = chamfer(
            &t,
            &Chamfer {
                distance: r,
                edges: names(&["top"]),
                ..Default::default()
            },
        )
        .unwrap();
        for s in [&f, &c] {
            check_closed(s);
            uncrossed(s);
            for i in s.faces(&["side"]).unwrap() {
                for v in s.triangles[i as usize] {
                    let p = s.positions[v as usize];
                    assert!(to_surface(p, &sides) < 1e-9, "r {r}: {p}");
                }
            }
        }
        let per_r2 = (v0 - f.measure().volume) / (r * r);
        assert!(per_r2 > 15.0 && per_r2 < 19.0, "r {r}: {per_r2}");
        assert!(c.measure().volume < f.measure().volume);
    }
}

/// Removed volume, and every vertex and centroid of a triangle not named
/// `kind` on the input's surface (no old face left its plane).
pub(crate) fn on_input(s: &Solid, out: &Solid, kind: &str) -> f64 {
    let tris: Vec<[crate::math::DVec3; 3]> = s
        .triangles
        .iter()
        .map(|t| t.map(|v| s.positions[v as usize]))
        .collect();
    let new: std::collections::BTreeSet<u32> = out.faces(&[kind]).unwrap().into_iter().collect();
    for (i, t) in out.triangles.iter().enumerate() {
        if new.contains(&(i as u32)) {
            continue;
        }
        let p = t.map(|v| out.positions[v as usize]);
        for q in [p[0], p[1], p[2], (p[0] + p[1] + p[2]) / 3.0] {
            assert!(to_surface(q, &tris) < 1e-9, "{q} off the input");
        }
    }
    s.measure().volume - out.measure().volume
}

#[test]
fn a_lone_piece_of_a_crease_tapers_to_a_tip_at_both_ends() {
    // One piece of a cylinder's rim, shorter than twice the offset: the
    // width peaks at half its length in the middle. The faces meet square,
    // so a chamfer removes two tetrahedra, (L/2)³/3 together.
    let c = pin(5.0, 10.0);
    let (a, b) = c
        .real_edges()
        .into_iter()
        .find(|&(a, b)| c.positions[a as usize].z == 10.0 && c.positions[b as usize].z == 10.0)
        .unwrap();
    let half = (c.positions[a as usize] - c.positions[b as usize]).length() / 2.0;
    assert!(half < 1.5);
    let mut piece = c.clone();
    piece.tag_edge(a, b, &["pick"]);
    let ch = chamfer(
        &piece,
        &Chamfer {
            distance: 1.5,
            edges: names(&["pick"]),
            ..Default::default()
        },
    )
    .unwrap();
    check_closed(&ch);
    uncrossed(&ch);
    let gone = on_input(&piece, &ch, "chamfer");
    assert!((gone - half * half * half / 3.0).abs() < 1e-9, "{gone}");
    let f = fillet(
        &piece,
        &Fillet {
            radius: 0.5,
            edges: names(&["pick"]),
            ..Default::default()
        },
    )
    .unwrap();
    check_closed(&f);
    uncrossed(&f);
    canonical(&f);
    named(&f, "fillet", &["top"]);
    assert!(on_input(&piece, &f, "fillet") > 0.0);
    // A straight edge of a rounded rectangle's top, longer than twice the
    // offset, meets its arcs tangentially: full width between stations
    // one offset in from each tip, d²/2 (L − 2d) + d³/3 removed.
    let rr = extrude(
        &crate::profile::rect(&crate::profile::Rect {
            width: 30.0,
            height: 20.0,
            radius: 4.0,
        })
        .unwrap(),
        &Extrude {
            height: 6.0,
            ..Default::default()
        },
    )
    .unwrap();
    let (a, b) = rr
        .real_edges()
        .into_iter()
        .find(|&(a, b)| {
            let (p, q) = (rr.positions[a as usize], rr.positions[b as usize]);
            p.z == 6.0 && q.z == 6.0 && p.y == q.y && (p.x - q.x).abs() > 20.0
        })
        .unwrap();
    let len = (rr.positions[a as usize] - rr.positions[b as usize]).length();
    let mut straight = rr.clone();
    straight.tag_edge(a, b, &["pick"]);
    let d = 1.5;
    let ch = chamfer(
        &straight,
        &Chamfer {
            distance: d,
            edges: names(&["pick"]),
            ..Default::default()
        },
    )
    .unwrap();
    check_closed(&ch);
    uncrossed(&ch);
    let gone = on_input(&straight, &ch, "chamfer");
    let want = d * d / 2.0 * (len - 2.0 * d) + d * d * d / 3.0;
    assert!((gone - want).abs() < 1e-9, "{gone} {want}");
    // A middle piece of the twisted prism's helical edge, between two
    // twisted rows: the stations' points lie in the planes of the piece's
    // own triangles, so no row tilts.
    let t = twisted();
    let &(a, b) = t
        .real_edges()
        .iter()
        .find(|&&(a, b)| {
            let (p, q) = (t.positions[a as usize], t.positions[b as usize]);
            p.z.max(q.z) >= 10.0 && p.z.min(q.z) < 10.0
        })
        .unwrap();
    let mut one = t.clone();
    one.tag_edge(a, b, &["pick"]);
    for r in [0.5, 1.5] {
        let f = fillet(
            &one,
            &Fillet {
                radius: r,
                edges: names(&["pick"]),
                ..Default::default()
            },
        )
        .unwrap();
        check_closed(&f);
        uncrossed(&f);
        assert!(on_input(&one, &f, "fillet") > 0.0);
    }
    // Where the taper does not fit, the error names the piece and asks for
    // the whole crease: a counterbore's floor rim beside a wall triangle
    // the boolean left as a sliver.
    let wide_plate = cuboid(&BoxDesc {
        width: 30.0,
        depth: 30.0,
        height: 10.0,
    })
    .unwrap();
    let cb = hole(
        &wide_plate,
        &Hole {
            diameter: 5.0,
            counterbore: (9.0, 3.0),
            at: vec![above(-3.0, 2.0)],
            ..Default::default()
        },
    )
    .unwrap();
    let near = |v: u32, x: f64, y: f64| {
        let p = cb.positions[v as usize];
        (p - crate::math::DVec3::new(x, y, 7.0)).length() < 1e-3
    };
    let (a, b) = cb
        .real_edges()
        .into_iter()
        .find(|&(a, b)| {
            near(a, -0.75, -1.8971) && near(b, 0.18198, -1.18198)
                || near(b, -0.75, -1.8971) && near(a, 0.18198, -1.18198)
        })
        .unwrap();
    let mut floor = cb.clone();
    floor.tag_edge(a, b, &["pick"]);
    let e = fillet(
        &floor,
        &Fillet {
            radius: 0.5,
            edges: names(&["pick"]),
            ..Default::default()
        },
    )
    .unwrap_err();
    let BevelError::LonePiece(_, _, n) = e else {
        panic!("{e:?}");
    };
    assert!(n.contains(&"pick".to_string()));
}

#[test]
fn faces_not_in_one_plane_are_cut_on_their_folds() {
    // Where the classic layout would tilt a face whose triangles at a
    // vertex are not one plane, each fold there is cut like a real edge,
    // in each triangle's own plane (`docs/research.md` §7.8).
    let t = twisted();
    let apex = crate::generators::solids::revolve(
        &crate::profile::Profile::from_contour(crate::profile::Contour::corners(vec![
            DVec2::new(0.0, 0.0),
            DVec2::new(8.0, 0.0),
            DVec2::new(8.0, 4.0),
            DVec2::new(0.0, 10.0),
        ])),
        &crate::generators::solids::Revolve {
            angle: 3.0,
            axis: crate::generators::solids::Axis::Z,
            ..Default::default()
        },
    )
    .unwrap();
    // A loft whose walls are twisted quads between sections of different
    // corner radii: run A's toaster housing, `fillet(14, top)`.
    let rr = |w: f64, h: f64, r: f64| {
        crate::profile::rect(&crate::profile::Rect {
            width: w,
            height: h,
            radius: r,
        })
        .unwrap()
    };
    let loft = crate::generators::loft::loft(&crate::generators::loft::Loft {
        sections: vec![
            (rr(155.0, 285.0, 30.0), 0.0),
            (rr(151.0, 281.0, 32.0), 61.25),
            (rr(125.0, 275.0, 24.0), 175.0),
        ],
        ..Default::default()
    })
    .unwrap();
    let cases: [(&Solid, &[&str], f64); 4] = [
        (&t, &[], 1.5),
        (&t, &["side"], 0.5),
        (&apex, &["start"], 0.5),
        (&loft, &["top"], 14.0),
    ];
    for (s, edges, r) in cases {
        let f = fillet(
            s,
            &Fillet {
                radius: r,
                edges: names(edges),
                segments: 4,
                ..Default::default()
            },
        )
        .unwrap();
        let c = chamfer(
            s,
            &Chamfer {
                distance: r,
                edges: names(edges),
                ..Default::default()
            },
        )
        .unwrap();
        for (out, kind) in [(&f, "fillet"), (&c, "chamfer")] {
            check_closed(out);
            uncrossed(out);
            let gone = on_input(s, out, kind);
            assert!(gone > 0.0, "{edges:?} {kind}: {gone}");
        }
        assert!(on_input(s, &f, "fillet") < on_input(s, &c, "chamfer"));
    }
    // Where the cuts do not fit either, the refusal stays: one of the
    // twisted prism's edges at 1.5.
    let e = t.real_edges()[4];
    let mut one = t.clone();
    one.tag_edge(e.0, e.1, &["pick"]);
    let e = fillet(
        &one,
        &Fillet {
            radius: 1.5,
            edges: names(&["pick"]),
            segments: 4,
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(
        matches!(&e, BevelError::Unsupported(_, why) if why.contains("not in one plane")),
        "{e:?}"
    );
}

pub(crate) fn rounded(w: f64, h: f64, r: f64) -> crate::profile::Profile {
    crate::profile::rect(&crate::profile::Rect {
        width: w,
        height: h,
        radius: r,
    })
    .unwrap()
}

/// A loft of rounded rectangles `steps` rows high: two sections, 10 high.
pub(crate) fn rows_loft(top: (f64, f64, f64), steps: u32, height: f64) -> Solid {
    crate::generators::loft::loft(&crate::generators::loft::Loft {
        sections: vec![
            (rounded(20.0, 20.0, 4.0), 0.0),
            (rounded(top.0, top.1, top.2), height),
        ],
        steps,
        curve: crate::generators::loft::LoftCurve::Straight,
        ..Default::default()
    })
    .unwrap()
}

#[test]
fn a_wall_whose_rows_are_shallower_than_the_bevel_is_cut_across_them() {
    // The smallest form of run Z2's toaster (`docs/research.md` §7.9): a
    // loft between rounded rectangles of different corner radii, its
    // rows 2.5 high, its top rim bevelled 3 deep. The rows are twisted
    // quads, so the classic layout would tilt them, and the fold layout's
    // points fall beyond the first row: it was the tilt refusal.
    let twisted = rows_loft((16.0, 16.0, 3.0), 4, 10.0);
    // The same with straight walls (one section twice): `TooLarge`.
    let straight = rows_loft((20.0, 20.0, 4.0), 4, 10.0);
    // ... whose single-row twin bevels classically.
    let one_row = rows_loft((20.0, 20.0, 4.0), 1, 10.0);
    let bevel = |s: &Solid, kind: &str, r: f64| {
        if kind == "fillet" {
            fillet(
                s,
                &Fillet {
                    radius: r,
                    edges: names(&["top"]),
                    segments: 4,
                    ..Default::default()
                },
            )
        } else {
            chamfer(
                s,
                &Chamfer {
                    distance: r,
                    edges: names(&["top"]),
                    ..Default::default()
                },
            )
        }
    };
    for kind in ["fillet", "chamfer"] {
        let mut gone = Vec::new();
        for s in [&twisted, &straight, &one_row] {
            let out = bevel(s, kind, 3.0).unwrap();
            check_closed(&out);
            uncrossed(&out);
            canonical(&out);
            in_planes(s, &out, kind);
            gone.push(on_input(s, &out, kind));
        }
        // Straight walls: the cut rows remove exactly what the single row
        // does (the extra profiles are the strip's own interpolations).
        assert!(
            (gone[1] - gone[2]).abs() < 1e-9 * gone[2],
            "{kind}: {gone:?}"
        );
        // The twisted wall leans in (the top is smaller): a little less.
        assert!(
            gone[0] > 0.7 * gone[2] && gone[0] < gone[2],
            "{kind}: {gone:?}"
        );
    }
    // A wall too short for the bevel: `TooLarge` where it does not fit,
    // not the tilt refusal.
    let short = rows_loft((16.0, 16.0, 3.0), 4, 4.0);
    for kind in ["fillet", "chamfer"] {
        let e = bevel(&short, kind, 5.0).unwrap_err();
        assert!(matches!(e, BevelError::TooLarge(_)), "{kind}: {e:?}");
    }
}

/// A round corner of the fillet's own radius: on the bottom the offset
/// line swallows the corner's chords (a polygon of radius 5 is 4.96 to
/// its sides), and the classic layout is refused. The wide bevel fitted
/// by sliding the bottom's vertices along the rim, which twisted the
/// wall's facets over them by 0.21 (`docs/research.md` §37). The run is
/// shrunk to one point instead: the wall keeps its columns.
#[test]
fn a_fillet_as_wide_as_the_round_corner_it_follows_ends_in_a_point() {
    let plate = extrude(
        &crate::profile::rect(&crate::profile::Rect {
            width: 80.0,
            height: 50.0,
            radius: 5.0,
        })
        .unwrap(),
        &Extrude {
            height: 80.0,
            ..Default::default()
        },
    )
    .unwrap();
    let s = fillet(
        &plate,
        &Fillet {
            radius: 5.0,
            edges: names(&["bottom"]),
            ..Default::default()
        },
    )
    .unwrap();
    check_closed(&s);
    uncrossed(&s);
    // Convex still: no vertex outside any triangle's plane.
    for t in &s.triangles {
        let [a, b, c] = t.map(|v| s.positions[v as usize]);
        let n = (b - a).cross(c - a).normalize();
        let out = s
            .positions
            .iter()
            .map(|&q| n.dot(q - a))
            .fold(0.0, f64::max);
        assert!(out < 1e-9, "{out} at {a:?}");
    }
    // The bottom is the rectangle between the corners' centres, less what
    // each corner's first and last chords still cut off it (a chord of
    // this polygon is 4.96 from the centre, so its offset line passes
    // 0.04 beyond): three points a corner, the chords between gone.
    let low: Vec<DVec3> = s.positions.iter().copied().filter(|p| p.z == 0.0).collect();
    assert_eq!(low.len(), 12, "{low:?}");
    assert!(low.iter().all(|p| {
        let d = DVec2::new(p.x.abs() - 35.0, p.y.abs() - 20.0);
        d.x < 1e-9 && d.y < 1e-9 && d.length() < 0.33
    }));
    // The wall's columns stand where they stood: its rim at z = 5 is on
    // the corner's own vertices.
    let rim = s
        .positions
        .iter()
        .filter(|p| (p.z - 5.0).abs() < 1e-9 && p.x > 35.0 && p.y > 20.0)
        .count();
    let top = s
        .positions
        .iter()
        .filter(|p| p.z == 80.0 && p.x > 35.0 && p.y > 20.0)
        .count();
    assert_eq!(rim, top);
    assert!(s.positions.iter().all(|p| {
        p.z != 5.0 || p.x.abs() < 35.0 || p.y.abs() < 20.0 || {
            let r = DVec2::new(p.x.abs() - 35.0, p.y.abs() - 20.0).length();
            (r - 5.0).abs() < 1e-9
        }
    }));
    // Two fillets whose offsets meet across a face are still too large.
    let e = fillet(
        &block(),
        &Fillet {
            radius: 15.0,
            edges: names(&["top"]),
            ..Default::default()
        },
    );
    assert!(matches!(e, Err(BevelError::TooLarge(_))), "{e:?}");
}
