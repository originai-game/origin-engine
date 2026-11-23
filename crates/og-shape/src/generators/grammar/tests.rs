use std::collections::BTreeMap;

use glam::{DVec2, DVec3};

use super::split::pieces;
use super::{Built, Lot, Size, build, parse};

fn grammar(rules: &str) -> super::Grammar {
    parse(&format!("type grammar\nversion 1\n---\n{rules}")).unwrap()
}

fn lot(points: &[[f64; 2]], base: f64, seed: u64) -> Lot {
    Lot {
        footprint: points.iter().map(|p| DVec2::new(p[0], p[1])).collect(),
        base,
        seed,
    }
}

fn rect(w: f64, d: f64) -> Vec<[f64; 2]> {
    vec![[0.0, 0.0], [w, 0.0], [w, d], [0.0, d]]
}

fn run(rules: &str, l: &Lot) -> Built {
    build(&grammar(rules), l, &BTreeMap::new()).unwrap()
}

fn pos(m: &super::Mesh, i: u32) -> DVec3 {
    DVec3::from(m.positions[i as usize].map(f64::from))
}

/// The area of a label's triangles.
fn area(b: &Built, label: &str) -> f64 {
    b.parts
        .iter()
        .filter(|p| p.label == label)
        .flat_map(|p| {
            p.mesh.indices.as_chunks::<3>().0.iter().map(move |t| {
                let [a, b, c] = t.map(|i| pos(&p.mesh, i));
                (b - a).cross(c - a).length() / 2.0
            })
        })
        .sum()
}

fn faces(b: &Built, label: &str) -> usize {
    b.parts
        .iter()
        .filter(|p| p.label == label)
        .map(|p| p.mesh.triangle_count())
        .sum()
}

/// The enclosed volume of every part together.
fn volume(b: &Built) -> f64 {
    b.parts.iter().map(|p| p.mesh.volume()).sum()
}

/// Every directed edge, welded by position, has its reverse: closed and
/// facing one way.
fn closed(b: &Built) -> bool {
    let key = |p: DVec3| (p * 1e4).round().as_ivec3().to_array();
    let mut edges = BTreeMap::new();
    for p in &b.parts {
        for t in p.mesh.indices.as_chunks::<3>().0 {
            let k = t.map(|i| key(pos(&p.mesh, i)));
            for j in 0..3 {
                *edges.entry((k[j], k[(j + 1) % 3])).or_insert(0i32) += 1;
            }
        }
    }
    edges
        .iter()
        .all(|(&(a, b), n)| edges.get(&(b, a)) == Some(n))
}

#[test]
fn repeats_are_as_many_as_fit_and_fill_the_length_exactly() {
    // 2.8 m bays between 0.6 m piers: 8.39 m of room holds two, 8.41 three.
    let parts = [(Size::Abs, 0.6), (Size::Repeat, 2.8), (Size::Abs, 0.6)];
    for (room, bays) in [(8.39, 2), (8.41, 3)] {
        let total = room + 1.2;
        let p = pieces(&parts, total).unwrap();
        assert_eq!(p.iter().filter(|x| x.0 == 1).count(), bays, "room {room}");
        assert_eq!(p[0], (0, 0.0, 0.6));
        assert_eq!(
            p.last().unwrap().2,
            total,
            "the last piece ends at the total exactly"
        );
        for w in p.windows(2) {
            assert_eq!(w[0].2, w[1].1, "pieces meet without a gap");
        }
        let bay = room / bays as f64;
        assert!(
            p.iter()
                .filter(|x| x.0 == 1)
                .all(|x| (x.2 - x.1 - bay).abs() < 1e-12)
        );
    }
    // Too short for even one: still one, squeezed into what is left.
    let p = pieces(&parts, 2.0).unwrap();
    assert_eq!(p.iter().filter(|x| x.0 == 1).count(), 1);
    // Exact parts alone are laid from 0 and cut at the end.
    let p = pieces(&[(Size::Abs, 2.0), (Size::Abs, 2.0)], 3.0).unwrap();
    assert_eq!(p, vec![(0, 0.0, 2.0), (1, 2.0, 3.0)]);
}

#[test]
fn a_split_cuts_the_face_it_is_given_along_the_axis_it_names() {
    // A 9.3 × 5.1 lot, 7.7 m tall: the front is 9.3 wide and the side
    // 5.1, so a split along the wrong axis or on the wrong face shows.
    let rules = "start Lot
rule Lot {
  extrude 7.7 {
    front Facade
    side Side
  }
}
rule Facade {
  split y {
    part 1 a
    fill 1 b
    part 2 c
  }
}
rule Side {
  split x {
    part 1.1 d
    fill 1 e
  }
}
rule a {
  label a
}
rule b {
  label b
}
rule c {
  label c
}
rule d {
  label d
}
rule e {
  label e
}";
    let b = run(rules, &lot(&rect(9.3, 5.1), 0.0, 1));
    let near = |x: f64, y: f64| (x - y).abs() < 1e-4;
    assert!(near(area(&b, "a"), 9.3 * 1.0), "{}", area(&b, "a"));
    assert!(near(area(&b, "b"), 9.3 * 4.7));
    assert!(near(area(&b, "c"), 9.3 * 2.0));
    // Two sides (the back is neither front nor side).
    assert!(near(area(&b, "d"), 2.0 * 1.1 * 7.7), "{}", area(&b, "d"));
    assert!(near(area(&b, "e"), 2.0 * 4.0 * 7.7));
}

#[test]
fn a_gable_house_is_closed_and_holds_its_volume() {
    // 10 × 6, 4 m walls, a 45° gable over the short span: a 3 m ridge,
    // so 240 m³ of walls and 0.5 × 6 × 3 × 10 = 90 under the roof.
    let rules = "start Lot
rule Lot {
  extrude 4 {
    sides wall
    bottom floor
    top Roof
  }
}
rule Roof {
  roof gable 45 {
    slope tiles
    gable wall
  }
}
rule wall {
  label wall
}
rule floor {
  label floor
}
rule tiles {
  label roof
}";
    let b = run(rules, &lot(&rect(10.0, 6.0), 0.0, 3));
    assert!(closed(&b));
    assert!((volume(&b) - 330.0).abs() < 1e-2, "{}", volume(&b));
    // The ridge runs along the long side: each slope is 10 × 3√2.
    assert!((area(&b, "roof") - 2.0 * 10.0 * 18f64.sqrt()).abs() < 1e-3);
    // Turned the other way round, the ridge still runs along the long side.
    let b = run(
        rules,
        &lot(&[[0.0, 0.0], [6.0, 0.0], [6.0, 10.0], [0.0, 10.0]], 0.0, 3),
    );
    assert!((volume(&b) - 330.0).abs() < 1e-2, "{}", volume(&b));
}

#[test]
fn a_hip_roof_follows_the_straight_skeleton() {
    let rules = "start Lot
rule Lot {
  extrude 4 {
    sides wall
    bottom floor
    top Roof
  }
}
rule Roof {
  roof hip 45 {
    slope tiles
  }
}
rule wall {
  label wall
}
rule floor {
  label floor
}
rule tiles {
  label roof
}";
    // 10 × 6 at 45°: a 3 m ridge 4 m long; under it b²(3a − b)/12 = 72.
    let b = run(rules, &lot(&rect(10.0, 6.0), 0.0, 3));
    assert!(closed(&b));
    assert!((volume(&b) - (240.0 + 72.0)).abs() < 1e-2, "{}", volume(&b));
    let top = b
        .parts
        .iter()
        .filter(|p| p.label == "roof")
        .flat_map(|p| &p.mesh.positions);
    let ridge = top.map(|p| p[1]).fold(0.0f32, f32::max);
    assert!((ridge - 7.0).abs() < 1e-4, "{ridge}");
    // An irregular convex pentagon closes too, every slope at 45°.
    let b = run(
        rules,
        &lot(
            &[[0.0, 0.0], [9.0, 0.0], [11.0, 5.0], [5.0, 9.0], [-1.0, 4.0]],
            0.0,
            3,
        ),
    );
    assert!(closed(&b));
    for p in b.parts.iter().filter(|p| p.label == "roof") {
        for n in &p.mesh.normals {
            assert!(
                (n[1] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-4,
                "{n:?}"
            );
        }
    }
    // A concave lot is refused by name.
    let l = lot(
        &[
            [0.0, 0.0],
            [10.0, 0.0],
            [10.0, 4.0],
            [4.0, 4.0],
            [4.0, 9.0],
            [0.0, 9.0],
        ],
        0.0,
        3,
    );
    let e = build(&grammar(rules), &l, &BTreeMap::new()).unwrap_err();
    assert!(e.contains("convex"), "{e}");
}

#[test]
fn texture_runs_on_across_splits_corners_and_reveals() {
    // Walls split into floors and bays, each window recessed: every point
    // reads one texture coordinate whichever part it is on, but for the
    // seam where the perimeter closes at the first corner.
    let rules = "param floor 3
start Lot
rule Lot {
  extrude 9 {
    sides Facade
  }
}
rule Facade {
  split y {
    repeat floor Floor
  }
}
rule Floor {
  split x {
    part 0.5 wall
    repeat 2.5 Bay
    part 0.5 wall
  }
}
rule Bay {
  split x {
    fill 1 wall
    part 1.2 Window
    fill 1 wall
  }
}
rule Window {
  split y {
    part 0.9 wall
    part 1.5 Glass
    fill 1 wall
  }
}
rule Glass {
  recess 0.1
  label glass
}
rule wall {
  label wall
}";
    let base = 2.5;
    // The front runs from the first corner to the second; walls are laid
    // counter-clockwise seen from above, so they start at the second.
    let corner = DVec2::new(14.0, 2.5);
    let points = [[1.0, 2.0], [14.0, 2.5], [13.0, 9.0], [0.0, 8.0]];
    let b = run(rules, &lot(&points, base, 9));
    assert!(faces(&b, "glass") > 0 && faces(&b, "jamb") > 0);
    // Walls first, then each jamb's edge against the wall round it (the
    // inner corners where two jambs meet cannot unfold both ways).
    let mut seen: BTreeMap<[i64; 3], [f32; 2]> = BTreeMap::new();
    for label in ["wall", "jamb"] {
        for p in b.parts.iter().filter(|p| p.label == label) {
            for (q, uv) in p.mesh.positions.iter().zip(&p.mesh.uvs) {
                let q = DVec3::from(q.map(f64::from));
                if label == "wall" {
                    // v is the height above the lot, written down the image.
                    assert!(
                        (f64::from(-uv[1]) - (q.y - base)).abs() < 1e-4,
                        "{q} {uv:?}"
                    );
                }
                if DVec2::new(q.x, q.z).distance(corner) < 1e-3 {
                    continue;
                }
                let k = (q * 1e3).round().as_i64vec3().to_array();
                let was = if label == "wall" {
                    seen.insert(k, *uv)
                } else {
                    seen.get(&k).copied()
                };
                if let Some(was) = was {
                    let d = (was[0] - uv[0]).abs().max((was[1] - uv[1]).abs());
                    assert!(d < 1e-3, "{label} at {q}: {was:?} and {uv:?}");
                }
            }
        }
    }
    let jamb_on_wall = b
        .parts
        .iter()
        .filter(|p| p.label == "jamb")
        .flat_map(|p| &p.mesh.positions)
        .any(|q| {
            let q = DVec3::from(q.map(f64::from));
            seen.contains_key(&(q * 1e3).round().as_i64vec3().to_array())
        });
    assert!(jamb_on_wall, "the jambs meet the wall");
}

#[test]
fn a_recess_turns_its_jambs_into_the_opening() {
    let rules = "start Lot
rule Lot {
  extrude 3 {
    front Glass
  }
}
rule Glass {
  recess 0.2
  label glass
}";
    let b = run(rules, &lot(&rect(4.0, 4.0), 0.0, 1));
    // The front faces -z (out of the lot at z 0) and is pushed in 0.2 m.
    let glass = &b.parts.iter().find(|p| p.label == "glass").unwrap().mesh;
    assert!(glass.positions.iter().all(|p| (p[2] - 0.2).abs() < 1e-6));
    assert!(glass.normals.iter().all(|n| n[2] < -0.999));
    let centre = DVec3::new(2.0, 1.5, 0.1);
    let jamb = &b.parts.iter().find(|p| p.label == "jamb").unwrap().mesh;
    for t in jamb.indices.as_chunks::<3>().0 {
        let [a, b2, c] = t.map(|i| pos(jamb, i));
        let n = (b2 - a).cross(c - a).normalize();
        let mid = (a + b2 + c) / 3.0;
        assert!(n.dot(centre - mid) > 0.0, "a jamb faces out of the opening");
    }
    assert!((area(&b, "jamb") - 0.2 * (2.0 * 4.0 + 2.0 * 3.0)).abs() < 1e-4);
    // The wall's pattern unfolds into the reveal: a jamb reads the face's
    // own heights (0 to 3 m), never past the opening's edge.
    for uv in &jamb.uvs {
        let v = -f64::from(uv[1]);
        assert!((-1e-5..=3.0 + 1e-5).contains(&v), "a jamb reads height {v}");
    }
}

#[test]
fn an_inset_keeps_the_face_whole() {
    let rules = "start Lot
rule Lot {
  extrude 3 {
    front Opening
  }
}
rule Opening {
  inset 0.25 frame glass
}
rule frame {
  label frame
}
rule glass {
  label glass
}";
    let b = run(rules, &lot(&rect(4.0, 4.0), 0.0, 1));
    assert!((area(&b, "glass") - 3.5 * 2.5).abs() < 1e-4);
    assert!((area(&b, "frame") + area(&b, "glass") - 12.0).abs() < 1e-4);
}

#[test]
fn the_seed_decides_the_picks_and_nothing_else_moves_them() {
    let rules = "start Lot
rule Lot {
  extrude 3 {
    sides Pick
  }
}
rule Pick {
  pick {
    option 1 a
    option 1 b
  }
}
rule a {
  label a
}
rule b {
  label b
}";
    let g = grammar(rules);
    let l = |seed| lot(&rect(5.0, 7.0), 0.0, seed);
    let same = build(&g, &l(4), &BTreeMap::new()).unwrap();
    assert_eq!(same, build(&g, &l(4), &BTreeMap::new()).unwrap());
    let mix: std::collections::BTreeSet<(usize, usize)> = (0..16)
        .map(|s| {
            let b = build(&g, &l(s), &BTreeMap::new()).unwrap();
            (faces(&b, "a"), faces(&b, "b"))
        })
        .collect();
    assert!(mix.len() > 2, "seeds choose differently: {mix:?}");
}

#[test]
fn params_and_detail_reach_what_they_name() {
    let rules = "param height 6
start Lot
rule Lot {
  extrude height {
    top Top
  }
}
rule Top {
  detail 2
  label top
}";
    let g = grammar(rules);
    let l = lot(&rect(2.0, 2.0), 1.0, 1);
    let b = build(&g, &l, &BTreeMap::from([("height".to_string(), 10.0)])).unwrap();
    let top = &b.parts[0];
    assert_eq!((top.label.as_str(), top.detail), ("top", 2));
    assert!(top.mesh.positions.iter().all(|p| p[1] == 11.0));
    let e = build(&g, &l, &BTreeMap::from([("hieght".to_string(), 1.0)])).unwrap_err();
    assert!(e.contains("hieght") && e.contains("height"), "{e}");
}

#[test]
fn a_grammar_is_refused_naming_what_is_wrong() {
    let bad = |text: &str, want: &str| {
        let e = parse(&format!("type grammar\nversion 1\n---\n{text}")).unwrap_err();
        assert!(e.contains(want), "{e:?} lacks {want:?}");
    };
    bad(
        "start A\nrule A {\n  split x {\n    part 1 Missing\n  }\n}",
        "`Missing`",
    );
    bad("start A\nrule A {\n  label a\n  label b\n}", "ends in one");
    bad("start A\nrule A {\n  recess 0.1\n}", "ends in one");
    bad(
        "start A\nrule A {\n  extrude tall {\n    top A\n  }\n}",
        "param `tall`",
    );
    bad("start A\nrule A {\n  wobble 1\n}", "not an operation");
    bad("rule A {\n  drop\n}", "start");
    // A rule that reaches itself without ending is stopped.
    let g = grammar("start A\nrule A {\n  pick {\n    option 1 A\n  }\n}");
    let e = build(&g, &lot(&rect(1.0, 1.0), 0.0, 0), &BTreeMap::new()).unwrap_err();
    assert!(e.contains("deep"), "{e}");
}

#[test]
fn inserts_carry_their_scope() {
    let rules = "start Lot
rule Lot {
  extrude 3 {
    front Door
  }
}
rule Door {
  insert \"models/door.shape.json\"
}";
    let b = run(rules, &lot(&rect(2.0, 5.0), 0.5, 1));
    let i = &b.inserts[0];
    assert_eq!(i.asset, "models/door.shape.json");
    assert!((i.size - DVec2::new(2.0, 3.0)).length() < 1e-9);
    // Its corner at the left seen from in front, its x to the right.
    assert!(
        (i.origin - DVec3::new(2.0, 0.5, 0.0)).length() < 1e-9,
        "{}",
        i.origin
    );
    assert!((i.x - DVec3::NEG_X).length() < 1e-9);
    assert!(
        (i.z - DVec3::NEG_Z).length() < 1e-9,
        "out of the lot: {}",
        i.z
    );
}
