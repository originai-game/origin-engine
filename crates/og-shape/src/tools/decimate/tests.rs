use super::*;
use crate::generators::solids::{BoxDesc, cuboid, sphere};

#[test]
fn halves_a_sphere_closed_and_keeps_a_box_square() {
    let s = sphere(20.0).unwrap();
    let n = s.triangles.len();
    let (half, r) = decimate(
        &s,
        &Decimate {
            ratio: 0.5,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(half.triangles.len() <= n / 2 + 1, "{r:?}");
    assert!(half.measure().closed);
    assert!(r.error > 0.0 && r.error < 2.0, "{r:?}");
    let chain = lods(
        &s,
        &[0.5, 0.25, 0.05].map(|ratio| Decimate {
            ratio,
            ..Default::default()
        }),
    )
    .unwrap();
    let counts: Vec<usize> = chain.iter().map(|l| l.0.triangles.len()).collect();
    assert!(counts[0] > counts[1] && counts[1] > counts[2], "{counts:?}");
    assert!(chain.iter().all(|l| l.0.measure().closed));
    assert!(decimate(&s, &Decimate::default()).is_err());
    // A box's faces are separate surfaces: their edges are features
    // and its corners stay, so it keeps its volume.
    let b = cuboid(&BoxDesc {
        width: 10.0,
        depth: 10.0,
        height: 10.0,
    })
    .unwrap();
    let (low, _) = decimate(
        &b,
        &Decimate {
            target: 1,
            ..Default::default()
        },
    )
    .unwrap();
    assert!((low.measure().volume - 1000.0).abs() < 1e-9);
}

#[test]
fn short_edges_collapse_and_the_box_stays_closed() {
    let b = cuboid(&BoxDesc {
        width: 10.0,
        depth: 10.0,
        height: 10.0,
    })
    .unwrap();
    let (s, gone) = collapse_short_edges(&sliver_box(&b), 1e-3).unwrap();
    assert_eq!(gone, 1);
    let m = s.measure();
    assert!(m.closed && m.triangles == b.triangles.len(), "{m:?}");
}

/// Crossing pairs, as `measure` counts them.
fn crossings(s: &Solid) -> usize {
    let (p, t) = crate::tools::measure::weld_exact(s);
    crate::tools::intersect::self_intersections(&p, &t).len()
}

/// A hollow ball, its wall 1 % of the radius: decimated hard, the
/// outer sphere's chords would cut through the inner one.
fn hollow_ball() -> Solid {
    let mut shell = sphere(10.0).unwrap();
    let mut inner = sphere(9.9).unwrap();
    for i in 0..inner.triangles.len() {
        inner.reverse_triangle(i);
    }
    shell.append(&inner);
    shell.canonicalize().unwrap();
    shell
}

#[test]
fn a_thin_shell_does_not_fold_through_itself() {
    let shell = hollow_ball();
    assert_eq!(crossings(&shell), 0);
    let (low, r) = decimate(
        &shell,
        &Decimate {
            target: 8,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(crossings(&low), 0, "{r:?}");
    assert!(low.measure().closed);
    assert!(r.folds_refused > 0, "{r:?}");
    // The target is not reachable without folding: the report says
    // how far it got and why.
    assert!(r.triangles > r.target && r.blocked.fold > 0, "{r:?}");
    assert_eq!(r.triangles, low.triangles.len());
}

#[test]
fn a_shell_crossing_itself_crosses_no_more_after() {
    // The hollow ball with one outer vertex pushed through the inner
    // wall: a few crossings to start with.
    let mut shell = hollow_ball();
    let v = (0..shell.positions.len())
        .max_by(|&a, &b| shell.positions[a].x.total_cmp(&shell.positions[b].x))
        .unwrap();
    shell.positions[v] *= 0.97;
    let before = crossings(&shell);
    assert!(before > 0);
    let (low, r) = decimate(
        &shell,
        &Decimate {
            target: 100,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(crossings(&low) <= before, "{before} {r:?}");
}

#[test]
fn islands_a_rigid_move_apart_are_one_chart() {
    // A flat 6 × 6 grid; its right half's UVs are its left half's
    // turned a quarter and moved: two islands, one rigid move apart
    // along the middle column.
    let mut g = Solid {
        uvs: Some(Vec::new()),
        ..Default::default()
    };
    for y in 0..=6 {
        for x in 0..=6 {
            g.push_vertex(DVec3::new(f64::from(x), f64::from(y), 0.0));
        }
    }
    let uv = |x: u32, y: u32, right: bool| {
        let (x, y) = (f64::from(x) * 0.125, f64::from(y) * 0.125);
        if right {
            DVec2::new(2.0 - y, x)
        } else {
            DVec2::new(x, y)
        }
    };
    for y in 0..6u32 {
        for x in 0..6u32 {
            let right = x >= 3;
            let v = |x: u32, y: u32| y * 7 + x;
            for t in [
                [(x, y), (x + 1, y), (x + 1, y + 1)],
                [(x, y), (x + 1, y + 1), (x, y + 1)],
            ] {
                g.push_triangle_uv(
                    t.map(|(x, y)| v(x, y)),
                    0,
                    &["grid"],
                    t.map(|(x, y)| uv(x, y, right)),
                );
            }
        }
    }
    g.canonicalize().unwrap();
    let run = |merge_charts: f64| {
        decimate(
            &g,
            &Decimate {
                target: 2,
                max_error: 1e-6,
                merge_charts,
                ..Default::default()
            },
        )
        .unwrap()
    };
    let (apart, a) = run(0.0);
    let (one, b) = run(1e-9);
    assert_eq!((a.uv_islands, a.uv_charts), (2, 2));
    assert_eq!((b.uv_islands, b.uv_charts), (2, 1));
    assert!(one.triangles.len() < apart.triangles.len(), "{a:?} {b:?}");
    // Every corner still lies on the texture as the grid put it: its
    // UV is the left half's mapping of its position, or the right
    // half's.
    let uvs = one.uvs.as_ref().unwrap();
    for (t, tri) in one.triangles.iter().enumerate() {
        for k in 0..3 {
            let p = one.positions[tri[k] as usize] * 0.125;
            let u = uvs[t][k];
            let left = (u - DVec2::new(p.x, p.y)).length();
            let right = (u - DVec2::new(2.0 - p.y, p.x)).length();
            assert!(left.min(right) < 1e-12, "{t} {k} {u:?} {p:?}");
        }
    }
}

/// The box with its top's first triangle split at a point 1e-5 mm
/// from one of its corners: three slivers, one short edge.
fn sliver_box(b: &Solid) -> Solid {
    let mut s = b.clone();
    let top = s.faces(&["top"]).unwrap()[0] as usize;
    let t = s.triangles[top];
    let p = s.positions[t[0] as usize] * (1.0 - 1e-6)
        + (s.positions[t[1] as usize] + s.positions[t[2] as usize]) * 5e-7;
    let v = s.push_vertex(p);
    let (surface, tags) = (s.surfaces[top], s.face_tags[top].clone());
    s.triangles[top] = [t[0], t[1], v];
    for tri in [[t[1], t[2], v], [t[2], t[0], v]] {
        s.triangles.push(tri);
        s.surfaces.push(surface);
        s.face_tags.push(tags.clone());
    }
    s.canonicalize().unwrap();
    s
}
