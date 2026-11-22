//! `place` on files read by their own axes.

use og_shape::math::*;
use og_shape::mesh::*;
use og_shape::generators::solids::{BoxDesc, cuboid};
use og_shape::tools::place::AxisTurn;
use og_shape::tools::*;
#[test]
fn import_reads_the_files_own_axes() {
    use og_shape::tools::{Import, no_files};
    use og_shape_formats::import;
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
    let glb = og_shape_formats::gltf::write::glb(&og_shape::tools::Scene {
        name: "legs".into(),
        parts: vec![og_shape::tools::Part::new("legs", &skinned_block())],
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
        weights: vec![og_shape::mesh::Weights::single(0); s.positions.len()],
    });
    s
}

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
