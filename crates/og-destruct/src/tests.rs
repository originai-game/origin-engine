use super::*;

/// `n` unit cubes in a row along `axis` (centres 0.5, 1.5, ...), bonded
/// face to face; `anchored` lists the chunks resting on the world through
/// their -y face.
fn row(n: u32, axis: Vec3, anchored: &[u32]) -> FractureAsset {
    let chunks = (0..n)
        .map(|i| {
            let centroid = axis * (i as f32 + 0.5);
            let a = anchored.contains(&i);
            Chunk {
                centroid,
                volume: 1.0,
                hull: vec![],
                node: i,
                parent: NONE,
                children: (0, 0),
                anchor_area: if a { 1.0 } else { 0.0 },
                anchor_centroid: if a {
                    centroid - Vec3::Y * 0.5
                } else {
                    Vec3::ZERO
                },
            }
        })
        .collect();
    let bonds = (0..n - 1)
        .map(|i| Bond {
            a: i,
            b: i + 1,
            centroid: axis * (i as f32 + 1.0),
            normal: axis,
            area: 1.0,
        })
        .collect();
    FractureAsset {
        chunks,
        bonds,
        support: n,
        ..FractureAsset::default()
    }
}

fn material(strength: f32) -> Material {
    Material {
        toughness: 100.0,
        density: 1000.0,
        limits: StressLimits {
            compression_elastic: strength * 10.0,
            compression_fatal: strength * 20.0,
            tension_elastic: strength,
            tension_fatal: strength * 2.0,
            shear_elastic: strength,
            shear_fatal: strength * 2.0,
        },
    }
}

const G: Vec3 = Vec3::new(0.0, -9.81, 0.0);

/// Runs the stress solver until the graph is at rest; the bonds broken.
fn settle(a: &FractureAsset, inst: &mut Instance, m: &Material) -> u32 {
    let mut broke = 0;
    inst.dirty = true;
    for _ in 0..64 {
        if !inst.dirty {
            break;
        }
        let islands = inst.islands(a);
        broke += inst.stress(a, &islands, G, 1.0, m, 0);
    }
    broke
}

#[test]
fn a_broken_bond_splits_the_islands_and_only_the_anchored_one_stays_held() {
    let a = row(4, Vec3::Y, &[0]);
    let mut inst = Instance::new(&a);
    assert_eq!(inst.islands(&a).len(), 1);
    inst.bonds[1] = 0.0;
    let islands = inst.islands(&a);
    assert_eq!(
        islands,
        vec![
            Island {
                chunks: vec![0, 1],
                anchored: true
            },
            Island {
                chunks: vec![2, 3],
                anchored: false
            },
        ]
    );
}

#[test]
fn a_cantilever_breaks_at_its_root_where_a_bridge_and_a_pillar_of_the_same_material_hold() {
    // 1 m cubes of 1000 kg; a 30 kPa material.
    let m = material(30_000.0);
    // A pillar of five: its root carries 49 kN in compression, under 300 kPa.
    let pillar = row(5, Vec3::Y, &[0]);
    let mut p = Instance::new(&pillar);
    assert_eq!(settle(&pillar, &mut p, &m), 0, "{:?}", p.bonds);
    // A bridge of five held at both ends: each end carries half its weight.
    let bridge = row(5, Vec3::X, &[0, 4]);
    let mut b = Instance::new(&bridge);
    assert_eq!(settle(&bridge, &mut b, &m), 0, "{:?}", b.bonds);
    // The same five held at one end only: the bond at its root bends past
    // its limits, and what hangs from it falls free.
    let arm = row(5, Vec3::X, &[0]);
    let mut c = Instance::new(&arm);
    assert!(settle(&arm, &mut c, &m) > 0);
    assert_eq!(c.bonds[0], 0.0, "the root bond breaks first: {:?}", c.bonds);
    let islands = c.islands(&arm);
    assert!(islands.iter().any(|i| !i.anchored && i.chunks.contains(&4)));
}

#[test]
fn a_stronger_material_holds_the_cantilever() {
    let arm = row(5, Vec3::X, &[0]);
    let mut c = Instance::new(&arm);
    assert_eq!(settle(&arm, &mut c, &material(5_000_000.0)), 0);
    assert!(c.bonds.iter().all(|&h| h == 1.0));
}

#[test]
fn radial_damage_breaks_the_bonds_within_its_reach_and_weakens_those_at_its_edge() {
    let a = row(6, Vec3::X, &[]);
    let mut inst = Instance::new(&a);
    let m = material(1e9);
    // Toughness 100 N·s/m², 1 m² bonds: 100 breaks a bond.
    let d = Damage {
        point: Vec3::new(1.0, 0.0, 0.0),
        min_radius: 1.0,
        max_radius: 3.0,
        amount: 150.0,
    };
    assert!(inst.damage(&a, &d, 1.0, &m, &|_, p| p));
    // Bonds at x = 1, 2 (within 1 m) break; 3 takes 75, 4 nothing.
    assert_eq!(inst.bonds[0], 0.0);
    assert_eq!(inst.bonds[1], 0.0);
    assert!((inst.bonds[2] - 0.25).abs() < 1e-6, "{:?}", inst.bonds);
    assert_eq!(inst.bonds[3], 1.0);
    assert_eq!(inst.bonds[4], 1.0);
    // Scaled up twice (the chunks' places and the bonds' areas, four
    // times as large) the same blast at the same place breaks none.
    let mut big = Instance::new(&a);
    let at = Damage {
        point: d.point * 2.0,
        min_radius: 2.0,
        max_radius: 6.0,
        ..d
    };
    big.damage(&a, &at, 2.0, &m, &|_, p| p * 2.0);
    assert!(big.bonds.iter().all(|&h| h > 0.0), "{:?}", big.bonds);
}

#[test]
fn the_asset_round_trips_and_refuses_a_bad_index() {
    let a = row(3, Vec3::Y, &[0]);
    let bytes = a.to_bytes();
    assert_eq!(FractureAsset::from_bytes(&bytes).unwrap(), a);
    let mut bad = a.clone();
    bad.bonds[0].b = 7;
    assert!(FractureAsset::from_bytes(&bad.to_bytes()).is_err());
}

#[test]
fn cube_root_is_exact_enough() {
    for x in [1e-6f32, 0.001, 0.125, 1.0, 8.0, 27.0, 1e6] {
        let r = cube_root(x);
        assert!((r * r * r - x).abs() <= x * 1e-5, "{x}: {r}");
    }
}
