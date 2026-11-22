use super::*;

fn closed(s: &Solid) -> f64 {
    let m = s.measure();
    assert!(m.closed, "open: {m:?}");
    assert_eq!(m.degenerate_triangles, 0);
    m.volume
}

#[test]
fn a_box_has_its_volume_and_names() {
    let s = cuboid(&BoxDesc {
        width: 4.0,
        depth: 3.0,
        height: 2.0,
    })
    .unwrap();
    assert!((closed(&s) - 24.0).abs() < 1e-12);
    assert_eq!(s.faces(&["top"]).unwrap().len(), 2);
    assert_eq!(s.faces(&["side"]).unwrap().len(), 8);
    assert_eq!(s.faces(&["front"]).unwrap().len(), 2);
    assert_eq!(s.edges(&["top"]).unwrap().len(), 4);
    assert_eq!(s.edges(&["side"]).unwrap().len(), 4);
    // Every real edge is named, and only real edges are.
    assert_eq!(s.real_edges().len(), 12);
    assert_eq!(s.edge_tags.len(), 12);
}

#[test]
fn a_tube_names_its_bore_and_is_closed() {
    let s = tube(10.0, 6.0, 5.0).unwrap();
    let v = closed(&s);
    let want = double::PI * (100.0 - 36.0) * 5.0;
    assert!((v - want).abs() / want < 2e-2, "{v}");
    assert!(!s.faces(&["inner"]).unwrap().is_empty());
    // A round wall has no vertical corner edges.
    assert!(s.edges(&["side"]).is_err());
}

#[test]
fn twisted_and_tapered_extrusions_are_closed() {
    let p = rect(&Rect {
        width: 4.0,
        height: 2.0,
        radius: 0.0,
    })
    .unwrap();
    let twisted = extrude(
        &p,
        &Extrude {
            height: 10.0,
            twist: double::PI / 2.0,
            taper: 0.5,
            both_sides: true,
        },
    )
    .unwrap();
    closed(&twisted);
    assert_eq!(twisted.measure().bounds_min.z, -5.0);
    let s = extrude(
        &p,
        &Extrude {
            height: 10.0,
            taper: 0.5,
            ..Default::default()
        },
    )
    .unwrap();
    closed(&s);
    let m = s.measure();
    // A frustum's volume with a scaled square cross-section: h·A·(1 + k + k²)/3.
    let want = 10.0 * 8.0 * (1.0 + 0.5 + 0.25) / 3.0;
    assert!((m.volume - want).abs() / want < 1e-9, "{}", m.volume);
}

#[test]
fn revolved_solids_are_closed() {
    let v = closed(&sphere(5.0).unwrap());
    assert!((v - 4.0 / 3.0 * double::PI * 125.0).abs() / v < 3e-2, "{v}");
    closed(&torus(5.0, 1.0).unwrap());
    closed(&dome(3.0).unwrap());
    closed(&cone(2.0, 4.0).unwrap());
    closed(
        &cylinder(&CylinderDesc {
            radius: 2.0,
            radius_top: 3.0,
            height: 1.0,
            ..Default::default()
        })
        .unwrap(),
    );
    closed(&wedge(1.0, 2.0, 3.0).unwrap());
    let quarter = rect(&Rect {
        width: 2.0,
        height: 2.0,
        radius: 0.0,
    })
    .unwrap()
    .translate(DVec2::new(3.0, 0.0));
    let part = revolve(
        &quarter,
        &Revolve {
            angle: double::PI / 2.0,
            ..Default::default()
        },
    )
    .unwrap();
    closed(&part);
    assert!(!part.faces(&["start"]).unwrap().is_empty());
    assert!(!part.edges(&["end"]).unwrap().is_empty());
}

#[test]
fn round_solids_turn_about_z_and_rest_on_it() {
    let m = sphere_desc(&SphereDesc {
        radius: 5.0,
        axis: Axis::Z,
        ..Default::default()
    })
    .unwrap()
    .measure();
    assert!(m.closed);
    assert_eq!((m.bounds_min.z, m.bounds_max.z), (0.0, 10.0));
    // Same solid as about y, turned (summed in another order).
    let v = sphere(5.0).unwrap().measure().volume;
    assert!((m.volume - v).abs() < 1e-12 * v);
    let t = torus_desc(&TorusDesc {
        radius: 8.0,
        tube_radius: 2.0,
        axis: Axis::Z,
        ..Default::default()
    })
    .unwrap()
    .measure();
    assert!(t.closed);
    assert_eq!((t.bounds_min.z, t.bounds_max.z), (0.0, 4.0));
    let d = dome_desc(&DomeDesc {
        radius: 3.0,
        axis: Axis::Z,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(d.measure().bounds_min.z, 0.0);
    // The flat bottom faces -z.
    for i in d.faces(&["bottom"]).unwrap() {
        let [a, b, c] = d.triangles[i as usize].map(|v| d.positions[v as usize]);
        assert!((b - a).cross(c - a).z < 0.0);
    }
    // revolve about z: profile y is height.
    let ring = rect(&Rect {
        width: 2.0,
        height: 4.0,
        radius: 0.0,
    })
    .unwrap()
    .translate(DVec2::new(5.0, 2.0));
    let r = revolve(
        &ring,
        &Revolve {
            axis: Axis::Z,
            ..Default::default()
        },
    )
    .unwrap()
    .measure();
    assert_eq!((r.bounds_min.z, r.bounds_max.z), (0.0, 4.0));
}

#[test]
fn a_spherical_cap_has_its_volume() {
    // V = π h² (3R − h) / 3, R = (a² + h²) / 2h; for a = 4, h = 2, R = 5.
    for (a, h) in [(4.0, 2.0), (3.0, 3.0), (3.0, 5.0)] {
        let s = dome_desc(&DomeDesc {
            radius: a,
            height: h,
            axis: Axis::Z,
            ..Default::default()
        })
        .unwrap();
        let r = (a * a + h * h) / (2.0 * h);
        let want = double::PI * h * h * (3.0 * r - h) / 3.0;
        let v = closed(&s);
        // The chord rule's deficit: 24 a turn around, 3 to 6 segments
        // along a shallow arc.
        assert!((v - want).abs() / want < 4e-2, "{a} {h}: {v} {want}");
        let m = s.measure();
        assert!((m.bounds_max.z - h).abs() < 1e-12);
        assert!(!s.faces(&["bottom"]).unwrap().is_empty());
    }
}

#[test]
fn bad_input_is_an_error_value() {
    let p = rect(&Rect {
        width: 1.0,
        height: 1.0,
        radius: 0.0,
    })
    .unwrap();
    assert_eq!(
        extrude(&p, &Extrude::default()),
        Err(GenError::NotPositive("height"))
    );
    assert_eq!(revolve(&p, &Revolve::default()), Err(GenError::CrossesAxis));
}

#[test]
fn a_part_torus_is_its_share_of_the_ring() {
    let d = TorusDesc {
        radius: 10.0,
        tube_radius: 2.0,
        segments: 32,
        tube_segments: 16,
        ..Default::default()
    };
    let whole = torus_desc(&d).unwrap().measure();
    let quarter = torus_desc(&TorusDesc {
        angle: double::PI / 2.0,
        ..d
    })
    .unwrap()
    .measure();
    assert!(quarter.closed);
    // 8 of the ring's 32 steps, exactly the same sections.
    assert!((quarter.volume * 4.0 - whole.volume).abs() < 1e-9 * whole.volume);
    let m = torus_desc(&TorusDesc { segments: 2, ..d });
    assert!(matches!(m, Err(GenError::Invalid(_))));
}

#[test]
fn textured_generators_keep_their_geometry_and_read_unmirrored() {
    let bar = rect(&Rect {
        width: 10.0,
        height: 4.0,
        radius: 1.0,
    })
    .unwrap();
    let cases = [
        (
            Generator::Box(BoxDesc {
                width: 4.0,
                depth: 3.0,
                height: 2.0,
            }),
            cuboid(&BoxDesc {
                width: 4.0,
                depth: 3.0,
                height: 2.0,
            }),
        ),
        (
            Generator::Extrude(
                &bar,
                Extrude {
                    height: 6.0,
                    twist: 0.5,
                    ..Default::default()
                },
            ),
            extrude(
                &bar,
                &Extrude {
                    height: 6.0,
                    twist: 0.5,
                    ..Default::default()
                },
            ),
        ),
        (
            Generator::Cylinder(CylinderDesc {
                radius: 5.0,
                height: 8.0,
                ..Default::default()
            }),
            cylinder(&CylinderDesc {
                radius: 5.0,
                height: 8.0,
                ..Default::default()
            }),
        ),
        (
            Generator::Sphere(SphereDesc {
                radius: 6.0,
                axis: Axis::Z,
                ..Default::default()
            }),
            sphere_desc(&SphereDesc {
                radius: 6.0,
                axis: Axis::Z,
                ..Default::default()
            }),
        ),
        (
            Generator::Torus(TorusDesc {
                radius: 10.0,
                tube_radius: 2.0,
                angle: 2.0,
                ..Default::default()
            }),
            torus_desc(&TorusDesc {
                radius: 10.0,
                tube_radius: 2.0,
                angle: 2.0,
                ..Default::default()
            }),
        ),
        (
            Generator::Dome(DomeDesc {
                radius: 5.0,
                ..Default::default()
            }),
            dome_desc(&DomeDesc {
                radius: 5.0,
                ..Default::default()
            }),
        ),
    ];
    for (g, plain) in cases {
        let t = textured(&g).unwrap();
        let plain = plain.unwrap();
        assert!(plain.uvs.is_none());
        let mut bare = t.clone();
        let uvs = bare.uvs.take().unwrap();
        assert_eq!(bare, plain, "{g:?}");
        // Every face with area reads counter-clockwise, as seen from
        // outside.
        for (i, u) in uvs.iter().enumerate() {
            let [a, b, c] = t.triangles[i].map(|v| t.positions[v as usize]);
            if (b - a).cross(c - a).length() > 1e-9 {
                assert_eq!(
                    orient2d(u[0], u[1], u[2]),
                    core::cmp::Ordering::Greater,
                    "{g:?} triangle {i}"
                );
            }
        }
    }
    // The box's top is its plan in mm; a cylinder's wall runs its
    // circumference round and its height up.
    let b = textured(&Generator::Box(BoxDesc {
        width: 4.0,
        depth: 3.0,
        height: 2.0,
    }))
    .unwrap();
    let uvs = b.uvs.as_ref().unwrap();
    for i in b.faces(&["top"]).unwrap() {
        for (k, &v) in b.triangles[i as usize].iter().enumerate() {
            assert_eq!(uvs[i as usize][k], b.positions[v as usize].truncate());
        }
    }
    let c = textured(&Generator::Cylinder(CylinderDesc {
        radius: 5.0,
        height: 8.0,
        ..Default::default()
    }))
    .unwrap();
    let side = c.faces(&["side"]).unwrap();
    let u = c.uvs.as_ref().unwrap();
    let (lo, hi) = side.iter().flat_map(|&i| u[i as usize]).fold(
        (DVec2::splat(f64::INFINITY), DVec2::splat(f64::NEG_INFINITY)),
        |(l, h), q| (l.min(q), h.max(q)),
    );
    let round = hi.x - lo.x;
    assert!((round - double::TAU * 5.0).abs() < 0.2, "{round}");
    assert_eq!(hi.y - lo.y, 8.0);
}
