//! The generators on fixed seeds: what each promises, and the outputs every host must reproduce.

use super::*;
use crate::noise::{Fbm, NoiseKind};

#[test]
fn weighted_draws_follow_their_weights_and_skip_nonpositive_ones() {
    let mut rng = Rng::new(7);
    let mut seen = [0u32; 4];
    for _ in 0..40_000 {
        seen[weighted(&mut rng, &[1.0, 0.0, 3.0, -2.0]).unwrap()] += 1;
    }
    assert_eq!((seen[1], seen[3]), (0, 0));
    let share = f64::from(seen[2]) / 40_000.0;
    assert!((share - 0.75).abs() < 0.01, "{share}");
    assert_eq!(weighted(&mut rng, &[0.0, f64::NAN]), None);
    assert_eq!(weighted(&mut rng, &[]), None);
}

#[test]
fn a_permutation_holds_every_index_once_and_varies_with_the_seed() {
    let p = permutation(&mut Rng::new(1), 100);
    let mut s = p.clone();
    s.sort_unstable();
    assert_eq!(s, (0..100).collect::<Vec<_>>());
    assert_ne!(p, permutation(&mut Rng::new(2), 100));
    assert_ne!(p, (0..100).collect::<Vec<_>>());
    assert!(permutation(&mut Rng::new(1), 0).is_empty());
    assert_eq!(permutation(&mut Rng::new(1), 1), [0]);
}

#[test]
fn poisson_points_keep_their_distance_and_fill_the_area() {
    let pts = poisson_disk(3, 40.0, 30.0, 2.0, 30, 10_000).unwrap();
    for (i, a) in pts.iter().enumerate() {
        assert!(a[0] >= 0.0 && a[0] < 40.0 && a[1] >= 0.0 && a[1] < 30.0);
        for b in &pts[i + 1..] {
            let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
            let d = (dx * dx + dy * dy).sqrt();
            assert!(d >= 2.0, "{a:?} {b:?} {d}");
        }
    }
    // A maximal 2-disc set covers 1,200 m² at between 0.866 r²... and
    // well above the 75 a square grid of 4 m would give.
    assert!(pts.len() > 150 && pts.len() < 320, "{}", pts.len());
    assert_eq!(pts, poisson_disk(3, 40.0, 30.0, 2.0, 30, 10_000).unwrap());
    assert_ne!(pts, poisson_disk(4, 40.0, 30.0, 2.0, 30, 10_000).unwrap());
    assert!(poisson_disk(3, 1e6, 1e6, 0.01, 30, 10).is_err());
    assert_eq!(poisson_disk(3, 40.0, 30.0, 2.0, 30, 10).unwrap().len(), 10);
}

#[test]
fn bsp_rooms_fit_their_leaves_do_not_overlap_and_are_all_joined() {
    let d = BspDesc {
        seed: 5,
        width: 64,
        height: 40,
        ..BspDesc::default()
    };
    let out = bsp_rooms(&d).unwrap();
    assert!(out.rooms.len() >= 6, "{}", out.rooms.len());
    for (i, a) in out.rooms.iter().enumerate() {
        assert!(a.width >= 4 && a.height >= 4 && a.width <= 12 && a.height <= 12);
        assert!(a.x + a.width <= 64 && a.y + a.height <= 40);
        for b in &out.rooms[i + 1..] {
            let apart = a.x + a.width <= b.x
                || b.x + b.width <= a.x
                || a.y + a.height <= b.y
                || b.y + b.height <= a.y;
            assert!(apart, "{a:?} {b:?}");
        }
    }
    // Every floor cell is reachable from the first room's.
    let (w, h) = (64usize, 40usize);
    let r0 = out.rooms[0];
    let mut seen = vec![false; w * h];
    let mut todo = vec![(r0.y * 64 + r0.x) as usize];
    seen[todo[0]] = true;
    while let Some(c) = todo.pop() {
        let (x, y) = (c % w, c / w);
        for (nx, ny) in [
            (x + 1, y),
            (x.wrapping_sub(1), y),
            (x, y + 1),
            (x, y.wrapping_sub(1)),
        ] {
            if nx < w && ny < h && out.cells[ny * w + nx] != 0 && !seen[ny * w + nx] {
                seen[ny * w + nx] = true;
                todo.push(ny * w + nx);
            }
        }
    }
    for (i, c) in out.cells.iter().enumerate() {
        assert!(*c == 0 || seen[i], "cell {} {} is cut off", i % w, i / w);
    }
    assert_eq!(out, bsp_rooms(&d).unwrap());
    assert_ne!(out, bsp_rooms(&BspDesc { seed: 6, ..d }).unwrap());
    assert!(bsp_rooms(&BspDesc { width: 3, ..d }).is_err());
}

fn pipes() -> Vec<WfcTile> {
    // Sockets: 0 empty, 1 pipe. A blank, a straight and a corner.
    vec![
        WfcTile {
            weight: 2.0,
            sockets: [0, 0, 0, 0],
            rotations: false,
        },
        WfcTile {
            weight: 1.0,
            sockets: [1, 0, 1, 0],
            rotations: true,
        },
        WfcTile {
            weight: 1.0,
            sockets: [1, 1, 0, 0],
            rotations: true,
        },
    ]
}

#[test]
fn wfc_tiles_fit_their_neighbours_and_hold_fixed_cells() {
    let desc = Wfc {
        seed: 9,
        width: 20,
        height: 12,
        tiles: pipes(),
        fixed: vec![(3, 4, 2, 1)],
        wrap: false,
        attempts: 5,
    };
    let out = wfc(&desc).unwrap();
    let sockets = |c: usize| {
        let t = &desc.tiles[out.tiles[c] as usize];
        let r = out.rotations[c] as usize;
        let s = t.sockets;
        [
            s[(4 - r) % 4],
            s[(5 - r) % 4],
            s[(6 - r) % 4],
            s[(7 - r) % 4],
        ]
    };
    for y in 0..12usize {
        for x in 0..20usize {
            let c = y * 20 + x;
            if x + 1 < 20 {
                assert_eq!(sockets(c)[1], sockets(c + 1)[3], "({x},{y}) right");
            }
            if y + 1 < 12 {
                assert_eq!(sockets(c)[2], sockets(c + 20)[0], "({x},{y}) down");
            }
        }
    }
    assert_eq!((out.tiles[4 * 20 + 3], out.rotations[4 * 20 + 3]), (2, 1));
    assert!(out.tiles.contains(&1), "some straights");
    assert_eq!(out, wfc(&desc).unwrap());
    assert_ne!(
        out.tiles,
        wfc(&Wfc {
            seed: 10,
            ..desc.clone()
        })
        .unwrap()
        .tiles
    );
}

#[test]
fn wfc_refuses_tiles_that_cannot_fit_naming_a_cell() {
    // A tile whose right edge matches no left edge cannot have a right neighbour.
    let desc = Wfc {
        seed: 1,
        width: 3,
        height: 1,
        tiles: vec![WfcTile {
            weight: 1.0,
            sockets: [0, 1, 0, 2],
            rotations: false,
        }],
        fixed: vec![],
        wrap: false,
        attempts: 3,
    };
    let e = wfc(&desc).unwrap_err();
    assert!(
        e.reason().contains("3 tries") && e.reason().contains("cell ("),
        "{}",
        e.reason()
    );
}

/// FNV-1a over the generators' outputs: the numbers a game builds a
/// level from. Any change to a draw order, a hash table or the float
/// arithmetic moves them, and a moved one means every seeded world in
/// every game changes; the web host must give the same (the bun test
/// in packages/engine/tests/modules.test.ts pins the same values).
#[test]
fn every_generator_gives_its_pinned_output_for_a_seed() {
    fn fnv(bytes: impl IntoIterator<Item = u8>) -> u64 {
        bytes.into_iter().fold(0xcbf2_9ce4_8422_2325, |h, b| {
            (h ^ b as u64).wrapping_mul(0x0100_0000_01b3)
        })
    }
    let mut r = Rng::new(42);
    let picks: Vec<u8> = (0..64)
        .map(|_| weighted(&mut r, &[1.0, 0.0, 3.0, 0.5]).unwrap() as u8)
        .collect();
    let perm = permutation(&mut Rng::new(42), 32);
    let f32s = |v: &[f32]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>();
    let grid = |kind, flat| {
        Fbm {
            seed: 7,
            kind,
            octaves: 4,
            frequency: 0.13,
            ..Fbm::default()
        }
        .grid(16, 16, -3.5, 2.25, 0.7, 1.3, flat)
    };
    let poisson = poisson_disk(5, 40.0, 30.0, 3.0, 30, 10_000).unwrap();
    let bsp = bsp_rooms(&BspDesc {
        seed: 3,
        ..BspDesc::default()
    })
    .unwrap();
    let w = wfc(&Wfc {
        seed: 9,
        width: 20,
        height: 12,
        tiles: pipes(),
        fixed: vec![],
        wrap: true,
        attempts: 5,
    })
    .unwrap();
    let got = [
        fnv(picks),
        fnv(perm.iter().flat_map(|x| x.to_le_bytes())),
        fnv(f32s(&grid(NoiseKind::Perlin, true))),
        fnv(f32s(&grid(NoiseKind::Perlin, false))),
        fnv(f32s(&grid(NoiseKind::Simplex, true))),
        fnv(f32s(&grid(NoiseKind::Simplex, false))),
        fnv(poisson.iter().flat_map(|p| f32s(p))),
        fnv(bsp.cells.iter().copied()),
        fnv(w
            .tiles
            .iter()
            .zip(&w.rotations)
            .map(|(t, r)| (t * 4 + r) as u8)),
    ];
    let text: Vec<String> = got.iter().map(|h| format!("{h:016x}")).collect();
    assert_eq!(text, PINNED, "{text:?}");
}

const PINNED: [&str; 9] = [
    "23c772a8d720158b",
    "9fe65e8b3d98afa5",
    "f408080a80a69ae1",
    "c99bab4bf56e2da4",
    "f87052e661c5c7be",
    "7988b7773f1605ed",
    "8c0c32d7c29b84cd",
    "4c25af53c3dbd445",
    "c3d44cb51b6ea25a",
];
