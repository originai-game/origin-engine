use super::*;
use crate::generators::solids::{BoxDesc, cuboid};

fn block() -> Solid {
    // 10 × 20 × 40 mm, low corner at (1, 2, 3).
    let mut s = cuboid(&BoxDesc {
        width: 10.0,
        depth: 20.0,
        height: 40.0,
    })
    .unwrap();
    let lo = s.positions.iter().fold(DVec3::INFINITY, |a, &p| a.min(p));
    s.map_positions(|p| p - lo + DVec3::new(1.0, 2.0, 3.0), false);
    s
}

#[test]
fn the_24_turns_are_proper_and_send_up_and_front_home() {
    let mut seen = Vec::new();
    for up in Axis::ALL {
        for front in Axis::ALL {
            let Ok(t) = AxisTurn::new(Some(up), Some(front)) else {
                assert_eq!(up.index(), front.index());
                continue;
            };
            assert_eq!(t.image(up), Axis::PosZ);
            assert_eq!(t.image(front), Axis::NegY);
            let [x, y, z] = [DVec3::X, DVec3::Y, DVec3::Z].map(|e| t.apply(e));
            assert_eq!(x.dot(y.cross(z)), 1.0, "{up} {front}");
            seen.push(t);
        }
    }
    seen.sort_by_key(|t| (t.from, t.neg));
    seen.dedup();
    assert_eq!(seen.len(), 24);
    // One axis alone: the smallest turn.
    let t = AxisTurn::new(Some(Axis::PosY), None).unwrap();
    assert_eq!(t.image(Axis::PosX), Axis::PosX);
    let t = AxisTurn::new(Some(Axis::PosZ), None).unwrap();
    assert_eq!(t, AxisTurn::IDENTITY);
}

#[test]
fn base_pivot_and_height_stand_a_lying_block_up() {
    // Lying with its up along +y: stand it up, 100 mm tall, on its base.
    let (out, r) = place(
        &[block()],
        &Place {
            up: Some(Axis::PosY),
            size: Size::Height(100.0),
            pivot: Pivot::Base,
            ..Default::default()
        },
    )
    .unwrap();
    let (lo, hi) = bounds(&out, |p| p).unwrap();
    assert_eq!(hi.z - lo.z, 100.0);
    assert_eq!(lo.z, 0.0);
    assert_eq!(lo.x + hi.x, 0.0);
    assert_eq!(lo.y + hi.y, 0.0);
    assert_eq!(r.scale, 5.0);
    assert_eq!(r.after.pivot_from_base, DVec3::ZERO);
    // The block's 20 mm (y) became its height, 40 mm (z) its depth.
    assert_eq!(r.before.size, DVec3::new(10.0, 20.0, 40.0));
    assert_eq!(r.after.size, DVec3::new(50.0, 200.0, 100.0));
    assert!(r.before.mass_of_volume);
    assert_eq!(r.after.mass, DVec3::splat(0.5));
}

#[test]
fn quarter_turns_are_exact_and_other_angles_turn() {
    let s = block();
    let (a, _) = place(
        std::slice::from_ref(&s),
        &Place {
            turn: 450.0,
            ..Default::default()
        },
    )
    .unwrap();
    for (p, q) in s.positions.iter().zip(&a[0].positions) {
        assert_eq!(*q, DVec3::new(-p.y, p.x, p.z));
    }
    let (b, _) = place(
        &[s],
        &Place {
            turn: 30.0,
            ..Default::default()
        },
    )
    .unwrap();
    let p = b[0].positions[0];
    assert!(p.is_finite());
}

fn skinned_block() -> Solid {
    let mut s = block();
    let rest = Frame {
        origin: DVec3::new(5.0, 10.0, 20.0),
        x: DVec3::X,
        y: DVec3::Y,
        z: DVec3::Z,
    };
    s.skin = Some(Skin {
        bones: vec![Bone::new("root", None, rest)],
        weights: vec![crate::mesh::Weights::single(0); s.positions.len()],
    });
    s
}

#[test]
fn a_skin_turns_scales_and_moves_with_its_mesh() {
    let s = skinned_block();
    let (out, _) = place(
        std::slice::from_ref(&s),
        &Place {
            up: Some(Axis::NegX),
            front: Some(Axis::PosZ),
            size: Size::Scale(2.0),
            pivot: Pivot::Base,
            ..Default::default()
        },
    )
    .unwrap();
    let skin = out[0].skin.as_ref().unwrap();
    skin.check(out[0].positions.len()).unwrap();
    let b = &skin.bones[0];
    // rest · IBM is still the identity, and every vertex in bone
    // coordinates is where it was, scaled.
    let id = mat_mul(&crate::mesh::skin::frame_matrix(&b.rest), &b.inverse_bind);
    for (k, v) in id.iter().enumerate() {
        let want = if k % 5 == 0 { 1.0 } else { 0.0 };
        assert!((v - want).abs() < 1e-12, "{id:?}");
    }
    let local = |m: &Mat4, p: DVec3| {
        DVec3::new(
            m[0] * p.x + m[4] * p.y + m[8] * p.z + m[12],
            m[1] * p.x + m[5] * p.y + m[9] * p.z + m[13],
            m[2] * p.x + m[6] * p.y + m[10] * p.z + m[14],
        )
    };
    let old = &s.skin.as_ref().unwrap().bones[0].inverse_bind;
    for (p, q) in s.positions.iter().zip(&out[0].positions) {
        let d = local(&b.inverse_bind, *q) - local(old, *p) * 2.0;
        assert!(d.length() < 1e-9);
    }
}

#[test]
fn facing_and_mass_are_measured() {
    let m = axis_measures(&[block()]);
    // Faces 20×40 (±x), 10×40 (±y), 10×20 (±z) of 2800 mm².
    let f = m.facing;
    assert!((f[0] - 800.0 / 2800.0).abs() < 1e-12 && f[0] == f[1]);
    assert!((f[4] - 200.0 / 2800.0).abs() < 1e-12);
    assert_eq!((m.longest, m.shortest), (2, 0));
    assert_eq!(m.pivot_from_base, -DVec3::new(6.0, 12.0, 3.0));
    // Open (its top dropped): the centre of area, lower down.
    let mut open = block();
    let top = (0..open.triangles.len())
        .filter(|&i| {
            open.triangles[i]
                .iter()
                .all(|&v| open.positions[v as usize].z == 43.0)
        })
        .collect::<Vec<_>>();
    for i in top.into_iter().rev() {
        open.triangles.remove(i);
        open.surfaces.remove(i);
        open.face_tags.remove(i);
    }
    let m = axis_measures(&[open]);
    assert!(!m.mass_of_volume);
    assert!(m.mass.z < 0.5);
    assert_eq!(m.facing[4], 0.0);
}

#[test]
fn import_reads_the_files_own_axes() {
    use crate::tools::{Import, import, no_files};
    let obj = "v 1 2 3\nv 4 5 6\nv 7 8 10\nf 1 2 3\n";
    let read = |d: Import| {
        import(obj.as_bytes(), &d, &no_files).map(|m| m.parts[0].solid.positions.clone())
    };
    let z_up = read(Import {
        z_up: true,
        ..Default::default()
    })
    .unwrap();
    let named = read(Import {
        up: Some(Axis::PosZ),
        front: Some(Axis::NegY),
        ..Default::default()
    });
    assert_eq!(named.unwrap(), z_up);
    // Up +y is the default; front +z follows.
    let y_up = read(Import {
        up: Some(Axis::PosY),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(y_up, read(Import::default()).unwrap());
    // A frame neither reads: up along -x, facing +y.
    let other = read(Import {
        up: Some(Axis::NegX),
        front: Some(Axis::PosY),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(other[0], DVec3::new(-3000.0, -2000.0, -1000.0));
    assert!(
        read(Import {
            z_up: true,
            up: Some(Axis::PosZ),
            ..Default::default()
        })
        .is_err()
    );
    // glTF: a skinned file read Z up keeps a skin that checks, its
    // mesh and bones turned from the Y-up reading.
    let glb = crate::tools::glb(&crate::tools::Scene {
        name: "legs".into(),
        parts: vec![crate::tools::Part::new("legs", &skinned_block())],
    })
    .unwrap();
    let plain = import(&glb, &Import::default(), &no_files).unwrap();
    let z_up = Import {
        up: Some(Axis::PosZ),
        ..Default::default()
    };
    let turned = import(&glb, &z_up, &no_files).unwrap();
    let (a, b) = (&plain.parts[0].solid, &turned.parts[0].solid);
    b.skin.as_ref().unwrap().check(b.positions.len()).unwrap();
    let c = AxisTurn::IDENTITY.then(&AxisTurn::new(Some(Axis::PosY), None).unwrap().inverse());
    for (p, q) in a.positions.iter().zip(&b.positions) {
        assert_eq!(c.apply(*p), *q);
    }
    let (ra, rb) = (
        &a.skin.as_ref().unwrap().bones[0].rest,
        &b.skin.as_ref().unwrap().bones[0].rest,
    );
    assert_eq!(c.apply(ra.origin), rb.origin);
    assert_eq!(c.apply(ra.z), rb.z);
}

#[test]
fn errors_are_values() {
    let s = [block()];
    let bad = |d: Place| place(&s, &d).unwrap_err();
    assert!(matches!(
        bad(Place {
            up: Some(Axis::PosZ),
            front: Some(Axis::NegZ),
            ..Default::default()
        }),
        PlaceError::Axes(_)
    ));
    assert!(matches!(
        bad(Place {
            size: Size::Height(-1.0),
            ..Default::default()
        }),
        PlaceError::Size(_)
    ));
    assert_eq!(
        place(&[Solid::default()], &Place::default()).unwrap_err(),
        PlaceError::Empty
    );
}
