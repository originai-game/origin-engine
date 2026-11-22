use super::*;

fn cube(offset: DVec3, size: f64) -> Solid {
    let mut s = Solid::default();
    for i in 0..8 {
        let p = DVec3::new((i & 1) as f64, ((i >> 1) & 1) as f64, ((i >> 2) & 1) as f64);
        s.push_vertex(offset + p * size);
    }
    let quads = [
        [0, 2, 3, 1],
        [4, 5, 7, 6],
        [0, 1, 5, 4],
        [2, 6, 7, 3],
        [0, 4, 6, 2],
        [1, 3, 7, 5],
    ];
    for (k, q) in quads.iter().enumerate() {
        s.push_triangle([q[0], q[1], q[2]], k as u32, &["cube"]);
        s.push_triangle([q[0], q[2], q[3]], k as u32, &["cube"]);
    }
    s
}

#[test]
fn repairs_a_broken_cube() {
    let mut s = cube(DVec3::ZERO, 10.0);
    assert!(s.measure().closed);
    // Split one corner into two copies 1e-4 apart, turn two faces
    // over, drop one triangle, add a floater and a sliver.
    let extra = s.push_vertex(DVec3::new(10.0, 0.0, 10.0 + 1e-4));
    s.triangles[11][2] = extra;
    s.triangles[0].swap(1, 2);
    s.triangles[5].swap(1, 2);
    s.triangles.remove(7);
    s.surfaces.remove(7);
    s.face_tags.remove(7);
    s.append(&cube(DVec3::splat(30.0), 0.5));
    let a = s.push_vertex(DVec3::ZERO);
    let b = s.push_vertex(DVec3::new(5.0, 0.0, 0.0));
    let c = s.push_vertex(DVec3::new(10.0, 0.0, 0.0));
    s.push_triangle([a, b, c], 0, &["sliver"]);
    assert!(!s.measure().closed);
    let d = Repair {
        weld: 0.01,
        floaters: Floaters {
            size: 2.0,
            ..Default::default()
        },
        max_hole_edges: 8,
        check_intersections: true,
        ..Default::default()
    };
    let (out, r) = repair(&s, &d).unwrap();
    let m = out.measure();
    assert!(m.closed, "{r:?} {m:?}");
    assert!((m.volume - 1000.0).abs() < 1e-9);
    assert_eq!((r.floaters, r.floater_triangles), (1, 12));
    assert_eq!((r.holes_filled, r.fill_triangles), (1, 1));
    assert_eq!(r.degenerate, 1);
    assert_eq!(r.self_intersections, Some(0));
    assert_eq!(out.faces(&["fill"]).unwrap().len(), 1);
}

#[test]
fn joins_parts_and_mirrors_keep_faces_out() {
    let part = |name: &str, t: Affine| ImportedPart {
        name: name.into(),
        solid: cube(DVec3::ZERO, 1.0),
        transform: t,
        ..Default::default()
    };
    let mirror = Affine {
        x: -DVec3::X,
        t: DVec3::new(-2.0, 0.0, 0.0),
        ..Affine::IDENTITY
    };
    let s = join(&[part("a", Affine::IDENTITY), part("b", mirror)]).unwrap();
    let m = s.measure();
    assert!(m.closed && m.components == 2);
    assert!((m.volume - 2.0).abs() < 1e-12);
}

/// Edges whose two triangles give either end different UVs.
fn uv_seams(s: &Solid) -> usize {
    let uvs = s.uvs.as_ref().unwrap();
    let mut sides: BTreeMap<(u32, u32), Vec<[[u64; 2]; 2]>> = BTreeMap::new();
    for (t, tri) in s.triangles.iter().enumerate() {
        for k in 0..3 {
            let (a, b) = (tri[k], tri[(k + 1) % 3]);
            let (ua, ub) = (uvs[t][k], uvs[t][(k + 1) % 3]);
            let bits = |u: DVec2| [u.x.to_bits(), u.y.to_bits()];
            let side = if a < b {
                [bits(ua), bits(ub)]
            } else {
                [bits(ub), bits(ua)]
            };
            sides.entry(edge_key(a, b)).or_default().push(side);
        }
    }
    sides
        .values()
        .filter(|v| v.len() == 2 && v[0] != v[1])
        .count()
}

#[test]
fn fuse_keeps_the_files_uvs_on_the_vertices_it_keeps() {
    // Two textured balls apart: the union cuts nothing, so every
    // corner is an input corner and the seams are the generator's.
    let ball = crate::generators::solids::textured(&crate::generators::solids::Generator::Sphere(
        crate::generators::solids::SphereDesc {
            radius: 7.3,
            ..Default::default()
        },
    ))
    .unwrap();
    let mut two = ball.clone();
    let mut other = ball.clone();
    for p in &mut other.positions {
        *p += DVec3::new(31.7, 0.3, 0.0);
    }
    two.append(&other);
    two.canonicalize().unwrap();
    let d = Repair {
        fuse: true,
        ..Default::default()
    };
    let (out, r) = repair(&two, &d).unwrap();
    assert_eq!(r.fused, Some(Ok(2)));
    assert_eq!(uv_seams(&out), uv_seams(&two));
}

#[test]
fn fuse_untangles_a_shell_folded_through_itself() {
    // A sphere with one vertex pulled through the far side
    // (`docs/research.md` §9.6): closed, one piece, crossing itself.
    let mut ball = crate::generators::solids::sphere(10.0).unwrap();
    let dir = DVec3::new(-0.51, -0.38, 0.77);
    let v = (0..ball.positions.len())
        .max_by(|&a, &b| {
            ball.positions[a]
                .dot(dir)
                .total_cmp(&ball.positions[b].dot(dir))
        })
        .unwrap();
    ball.positions[v] = -ball.positions[v] * 1.5;
    let d = Repair {
        fuse: true,
        check_intersections: true,
        ..Default::default()
    };
    let (out, r) = repair(&ball, &d).unwrap();
    assert_eq!(r.fused, Some(Ok(1)));
    assert_eq!(r.self_intersections, Some(0));
    let m = out.measure();
    assert!(m.closed && m.components == 1, "{m:?}");
    // Without fuse it is left, and reported.
    let (_, r) = repair(
        &ball,
        &Repair {
            check_intersections: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(r.self_intersections.unwrap() > 0);
}

#[test]
fn fuse_drops_the_pockets_an_untangling_leaves() {
    // A ball whose vertices are moved by up to 1.5 per axis
    // (splitmix64): it folds through itself, and its self-union is a
    // main shell and three small pockets the surface folded round.
    let mut ball = crate::generators::solids::sphere(10.0).unwrap();
    let mut state = 5u64;
    let mut next = || {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        (z >> 11) as f64 / (1u64 << 52) as f64 - 1.0
    };
    for p in &mut ball.positions {
        *p += DVec3::new(next(), next(), next()) * 1.5;
    }
    let d = Repair {
        fuse: true,
        floaters: Floaters {
            fraction: 0.05,
            ..Default::default()
        },
        check_intersections: true,
        ..Default::default()
    };
    let (out, r) = repair(&ball, &d).unwrap();
    assert_eq!(r.fused, Some(Ok(1)));
    assert_eq!((r.pockets, r.pockets_dropped), (3, 3), "{r:?}");
    assert_eq!(r.self_intersections, Some(0));
    assert_eq!(pieces(&out).1, 1);
    assert!(out.measure().closed);
}

#[test]
fn two_cubes_on_one_edge_come_apart_closed() {
    // Two cubes touching along one edge: four faces on it.
    let mut s = cube(DVec3::ZERO, 10.0);
    s.append(&cube(DVec3::new(10.0, 10.0, 0.0), 10.0));
    s.canonicalize().unwrap();
    assert_eq!(s.measure().non_manifold_edges, 1);
    let (out, r) = repair(
        &s,
        &Repair {
            non_manifold: true,
            check_intersections: true,
            ..Default::default()
        },
    )
    .unwrap();
    let nm = r.non_manifold.unwrap();
    assert_eq!(
        (nm.edges, nm.split, nm.left, nm.unpaired),
        (1, 1, 0, 0),
        "{nm:?}"
    );
    assert_eq!(nm.vertices, 2);
    let m = out.measure();
    assert!(m.closed, "{m:?}");
    assert_eq!(pieces(&out).1, 2);
    assert_eq!(r.self_intersections, Some(0));
    // Each copy moved into its own cube, 1 % of its shortest edge.
    assert!((nm.moved - 0.1).abs() < 1e-12, "{nm:?}");
}

#[test]
fn a_fill_across_a_uv_seam_covers_no_texture() {
    let ball = crate::generators::solids::textured(&crate::generators::solids::Generator::Sphere(
        crate::generators::solids::SphereDesc {
            radius: 7.3,
            ..Default::default()
        },
    ))
    .unwrap();
    // A vertex on the seam (two UVs), away from the poles: its fan
    // removed leaves a hole whose rim crosses the seam.
    let uvs = ball.uvs.as_ref().unwrap();
    let mut at: BTreeMap<u32, Vec<[u64; 2]>> = BTreeMap::new();
    for (t, tri) in ball.triangles.iter().enumerate() {
        for k in 0..3 {
            let u = uvs[t][k];
            at.entry(tri[k])
                .or_default()
                .push([u.x.to_bits(), u.y.to_bits()]);
        }
    }
    let v = *at
        .iter()
        .find(|(v, u)| {
            let mut u = (*u).clone();
            u.sort_unstable();
            u.dedup();
            u.len() == 2 && ball.positions[**v as usize].z.abs() < 3.0
        })
        .unwrap()
        .0;
    let mut open = ball.clone();
    let keep: Vec<usize> = (0..open.triangles.len())
        .filter(|&t| !open.triangles[t].contains(&v))
        .collect();
    let remap: Vec<u32> = (0..open.positions.len() as u32).collect();
    open = drop_unused(&rebuild(&open, &keep, &remap, open.positions.clone()));
    let (out, h) = close_holes(&open, 64).unwrap();
    assert_eq!((h.filled, h.uv_seam_fills), (1, 1), "{h:?}");
    let fill = out.faces(&["fill"]).unwrap();
    assert!(!fill.is_empty());
    let uvs = out.uvs.as_ref().unwrap();
    for t in fill {
        let u = uvs[t as usize];
        assert!(u[0] == u[1] && u[1] == u[2], "{u:?}");
    }
    assert!(out.measure().closed);
}

#[test]
fn a_hole_over_the_limit_is_filled_when_flat_and_clear() {
    // A cube without one face: a square hole of four edges.
    let mut s = cube(DVec3::ZERO, 10.0);
    for _ in 0..2 {
        s.triangles.pop();
        s.surfaces.pop();
        s.face_tags.pop();
    }
    let (out, h) = close_holes(&s, 3).unwrap();
    assert_eq!((h.filled, h.large, h.left), (1, 1, 0), "{h:?}");
    assert!(out.measure().closed);
    // Within the limit it is an ordinary fill.
    let (_, h) = close_holes(&s, 4).unwrap();
    assert_eq!((h.filled, h.large), (1, 0));
}

/// A closed tetrahedron standing on z = `base` at (5, 5), its tip at
/// z = `tip`.
fn spike(base: f64, tip: f64) -> Solid {
    let mut s = Solid::default();
    for p in [
        DVec3::new(4.5, 4.7, base),
        DVec3::new(5.5, 4.7, base),
        DVec3::new(5.0, 5.6, base),
        DVec3::new(5.0, 5.0, tip),
    ] {
        s.push_vertex(p);
    }
    for t in [[0, 2, 1], [0, 1, 3], [1, 2, 3], [2, 0, 3]] {
        s.push_triangle(t, 9, &["spike"]);
    }
    s
}

/// A ball with its cap cut away closes round: rings of new vertices
/// that leave the rim along the ball, to within a hundredth of the
/// whole ball's volume (a flat fill is a tenth short). A hole in a
/// flat face stays a flat fill.
#[test]
fn a_large_hole_in_a_curved_surface_closes_round() {
    let ball = crate::generators::solids::sphere_desc(&crate::generators::solids::SphereDesc {
        radius: 50.0,
        segments: 48,
        ..Default::default()
    })
    .unwrap();
    let whole = ball.measure().volume;
    let mut open = ball.clone();
    // Without what stands above 0.6 of its radius (the poles are on y).
    for i in (0..open.triangles.len()).rev() {
        let top = open.triangles[i]
            .iter()
            .map(|&v| open.positions[v as usize].y)
            .sum::<f64>()
            / 3.0;
        if top > 30.0 {
            open.triangles.remove(i);
            open.surfaces.remove(i);
            open.face_tags.remove(i);
        }
    }
    let (out, h) = close_holes(&open, 8).unwrap();
    assert_eq!((h.filled, h.large, h.round, h.left), (1, 1, 1, 0), "{h:?}");
    let m = out.measure();
    assert!(m.closed);
    assert!(
        (m.volume - whole).abs() < 0.01 * whole,
        "{} {whole}",
        m.volume
    );
    // Closed flat it would be short by the cap above the cut (of
    // height 20: π h² (3 r − h) / 3), a tenth of the ball.
    let cap = core::f64::consts::PI * 20.0 * 20.0 * (150.0 - 20.0) / 3.0;
    assert!(whole - cap < 0.91 * whole);

    // A hole in a flat face: no tilt at its rim, a flat fill.
    let mut lid = cube(DVec3::ZERO, 10.0);
    for k in [3, 2] {
        lid.triangles.remove(k);
        lid.surfaces.remove(k);
        lid.face_tags.remove(k);
    }
    let (_, h) = close_holes(&lid, 3).unwrap();
    assert_eq!((h.filled, h.large, h.round), (1, 1, 0), "{h:?}");
}

/// A large hole with something in its way: a spike that just pokes
/// through the flat fill is cleared by a fan from a lifted centre;
/// one that stands far through is closed only when asked, by a fill
/// that crosses it, and counted.
#[test]
fn a_large_hole_is_fanned_round_what_pokes_through_it() {
    let mut open = cube(DVec3::ZERO, 10.0);
    // Without its top (the second quad).
    for k in [3, 2] {
        open.triangles.remove(k);
        open.surfaces.remove(k);
        open.face_tags.remove(k);
    }
    let with = |spike: &Solid| {
        let mut s = open.clone();
        s.append(spike);
        s
    };
    let low = with(&spike(9.0, 10.2));
    let (out, h) = close_holes(&low, 3).unwrap();
    assert_eq!(
        (h.filled, h.large, h.crossing, h.left),
        (1, 1, 0, 0),
        "{h:?}"
    );
    assert_eq!(out.triangles.len(), low.triangles.len() + 4);
    // Whether the fill crosses the rest of the mesh.
    let fill_crosses = |s: &Solid| -> bool {
        let fill = s.faces(&["fill"]).unwrap();
        let rest = Solid {
            positions: s.positions.clone(),
            triangles: (0..s.triangles.len() as u32)
                .filter(|t| !fill.contains(t))
                .map(|t| s.triangles[t as usize])
                .collect(),
            ..Default::default()
        };
        let tris: Vec<[u32; 3]> = fill.iter().map(|&t| s.triangles[t as usize]).collect();
        crosses(&rest, &tris, &[])
    };
    assert!(!fill_crosses(&out));
    let tall = with(&spike(9.0, 15.0));
    let (_, h) = close_holes(&tall, 3).unwrap();
    assert_eq!((h.filled, h.left), (0, 1), "{h:?}");
    let (out, h) = close_holes_through(&tall, 3, true).unwrap();
    assert_eq!(
        (h.filled, h.large, h.crossing, h.left),
        (1, 1, 1, 0),
        "{h:?}"
    );
    assert!(fill_crosses(&out));
}

/// Two columns that touch along one edge, three edges long: each of
/// the three has four faces, and the two in the middle of the chain
/// have such an edge at both ends. All three split, and the columns
/// come apart closed.
#[test]
fn a_chain_of_non_manifold_edges_splits_along_its_length() {
    let mut s = Solid::default();
    for (ox, oy) in [(0.0, 0.0), (1.0, 1.0)] {
        for z in 0..3 {
            let c = cube(DVec3::new(ox, oy, f64::from(z)), 1.0);
            let base = s.positions.len() as u32;
            for p in &c.positions {
                s.push_vertex(*p);
            }
            for (i, t) in c.triangles.iter().enumerate() {
                // The faces between the cubes of a column go.
                let bottom = i < 2 && z > 0;
                let top = (2..4).contains(&i) && z < 2;
                if !bottom && !top {
                    s.push_triangle(t.map(|v| v + base), c.surfaces[i], &["column"]);
                }
            }
        }
    }
    s.canonicalize().unwrap();
    let (out, nm) = split_non_manifold(&s).unwrap();
    assert_eq!(
        (nm.edges, nm.split, nm.left, nm.unpaired),
        (3, 3, 0, 0),
        "{nm:?}"
    );
    let m = crate::tools::measure(&out, &crate::tools::Measure::default());
    assert!(m.closed && m.non_manifold_edges == 0, "{m:?}");
    assert_eq!(m.pieces.len(), 2);
}

/// A skin may hold only part of the skeleton: Mixamo's soldier has a
/// body skinned to 49 bones and a visor skinned to the neck and head
/// alone, whose neck has no parent in its own skin. The two are one
/// skeleton; bones whose parents are named differently are not.
#[test]
fn a_skin_of_part_of_the_skeleton_joins_the_whole_one() {
    use crate::math::Frame;
    use crate::mesh::{Bone, Weights};
    let at = |z: f64| Frame {
        origin: DVec3::new(0.0, 0.0, z),
        ..Frame::IDENTITY
    };
    let skinned = |bones: Vec<Bone>, offset: f64| {
        let mut s = cube(DVec3::new(offset, 0.0, 0.0), 1.0);
        let last = bones.len() as u32 - 1;
        s.skin = Some(Skin {
            bones,
            weights: vec![Weights::single(last); s.positions.len()],
        });
        s
    };
    let body = skinned(
        vec![
            Bone::new("spine", None, at(0.0)),
            Bone::new("neck", Some(0), at(5.0)),
        ],
        0.0,
    );
    let visor = skinned(vec![Bone::new("neck", None, at(5.0))], 3.0);
    for parts in [
        [("body", &body), ("visor", &visor)],
        [("visor", &visor), ("body", &body)],
    ] {
        let (bones, maps, _) = merge_skeletons(&parts).unwrap().unwrap();
        let names: Vec<&str> = bones.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(names, ["spine", "neck"]);
        assert_eq!(bones[1].parent, Some(0));
        // Each part's bones, by its own order, in the merged list.
        let visor_at = usize::from(parts[0].0 == "body");
        assert_eq!(maps[visor_at], [1]);
        assert_eq!(maps[1 - visor_at], [0, 1]);
    }
    // A neck hung on another bone is another skeleton.
    let other = skinned(
        vec![
            Bone::new("hips", None, at(0.0)),
            Bone::new("neck", Some(0), at(5.0)),
        ],
        6.0,
    );
    let e = merge_skeletons(&[("body", &body), ("other", &other)]).unwrap_err();
    assert!(e.to_string().contains("neck"), "{e}");
}

/// Two meshes skinned to one skeleton, each bound in its own space
/// (their inverse bind matrices differ by one matrix for every bone),
/// are one skeleton: the second's vertices go to the first's space.
/// Bones that differ by different matrices are a conflict.
#[test]
fn parts_bound_in_different_spaces_join_on_one_skeleton() {
    use crate::math::Frame;
    use crate::mesh::{Bone, Weights};
    let frame = |x: f64, z: f64| Frame {
        origin: DVec3::new(x, 0.0, z),
        ..Frame::IDENTITY
    };
    let part = |shift: f64, twist: f64| {
        // The same two bones where they stand, and the same cube in
        // the world, its mesh written `shift` along x away: the
        // inverse binds take it back.
        let mut s = cube(DVec3::new(-shift, 0.0, 0.0), 4.0);
        let mut bones = vec![
            Bone::new("hip", None, frame(0.0, 0.0)),
            Bone::new("knee", Some(0), frame(0.0, 5.0)),
        ];
        for (k, b) in bones.iter_mut().enumerate() {
            b.inverse_bind[12] += shift + if k == 1 { twist } else { 0.0 };
        }
        s.skin = Some(Skin {
            bones,
            weights: vec![Weights::single(1); s.positions.len()],
        });
        s
    };
    let (a, b) = (part(0.0, 0.0), part(10.0, 0.0));
    let (bones, maps, fixes) = merge_skeletons(&[("a", &a), ("b", &b)]).unwrap().unwrap();
    assert_eq!(bones.len(), 2);
    assert_eq!(maps, [vec![0, 1], vec![0, 1]]);
    assert_eq!(fixes[0], None);
    let f = fixes[1].expect("a matrix for the second part");
    assert!((f[12] - 10.0).abs() < 1e-9 && f[13] == 0.0 && (f[0] - 1.0).abs() < 1e-12);
    // Joined, the two cubes stand on each other: 8 vertices, not 16.
    let named = |name: &str, solid: &Solid| ImportedPart {
        clips: Vec::new(),
        name: name.into(),
        solid: solid.clone(),
        transform: Affine::IDENTITY,
        material: Default::default(),
        defaults: Vec::new(),
    };
    let joined = join(&[named("a", &a), named("b", &b)]).unwrap();
    assert_eq!(joined.positions.len(), 8);
    joined.skin.as_ref().unwrap().check(8).unwrap();
    // One bone off by another matrix than the other: not one space.
    let e = merge_skeletons(&[("a", &a), ("c", &part(10.0, 3.0))]).unwrap_err();
    assert!(e.to_string().contains("knee"), "{e}");
}
