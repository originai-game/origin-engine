//! A model's `fracture` block: the cook cuts it into Voronoi chunks, one
//! node each, and writes `<model>#fracture` (`og.Fracture`).
//!
//! Ways this could fail: the chunks do not tile the model (a cell lost, a
//! piece counted twice), so the volumes do not add up; a chunk's node is
//! not where its mesh's centroid is, so a chunk spawned at its node jumps;
//! an anchor is given to a chunk off the floor, or none to the bottom row;
//! the children are drawn at rest (the intact model doubled); the pattern
//! depends on something other than its seed (the cook's thread count).

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use og_destruct::FractureAsset;
use std::path::PathBuf;

const WALL: &str = r#"type model
version 1
---
material "brick" {
  color 0.6 0.3 0.2
  roughness 0.9
}
part "wall" {
  material "brick"
  position 0 1 0
  boxGeometry {
    width 4
    height 2
    depth 0.3
  }
}
fracture {
  chunks 20
  seed 7
  children 4
  interior {
    color 0.5 0.45 0.4
  }
}
"#;

fn cook(name: &str, text: &str, threads: usize) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/props")).unwrap();
    std::fs::write(root.join("assets/props/wall.ogt"), text).unwrap();
    let report = cook_project(
        &root,
        &CookOptions {
            threads,
            ..CookOptions::default()
        },
    )
    .unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    root
}

fn asset(root: &std::path::Path) -> FractureAsset {
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(
        bundle.output("props/wall.ogt#fracture").unwrap().ty,
        "og.Fracture"
    );
    let bytes = bundle.read("props/wall.ogt#fracture").unwrap();
    FractureAsset::from_bytes(&bundle.fracture("props/wall.ogt#fracture", &bytes).unwrap()).unwrap()
}

#[test]
fn a_wall_cooks_into_chunks_that_tile_it_with_bonds_anchors_and_hidden_children() {
    let root = cook("og-cook-fracture-wall", WALL, 0);
    let a = asset(&root);
    // Every seed lands in the box, and a box cell is one piece.
    assert_eq!(a.support, 20);
    let volume: f32 = a.chunks[..20].iter().map(|c| c.volume).sum();
    assert!(
        (volume - 2.4).abs() < 1e-3,
        "the chunks tile the 4 × 2 × 0.3 wall: {volume}"
    );
    // Each chunk's children tile it.
    for c in &a.chunks[..20] {
        assert_eq!(c.children.1, 4);
        let (first, n) = c.children;
        let sum: f32 = a.chunks[first as usize..(first + n) as usize]
            .iter()
            .map(|k| k.volume)
            .sum();
        assert!(
            (sum - c.volume).abs() < c.volume * 1e-3,
            "{sum} {}",
            c.volume
        );
    }
    // A wall is thin: its cells are shards through its depth, so every
    // chunk touches both faces and the graph is planar enough to have
    // between n - 1 and 3n bonds.
    assert!(
        a.bonds.len() >= 19 && a.bonds.len() <= 60,
        "{}",
        a.bonds.len()
    );
    let total_bond: f32 = a.bonds.iter().map(|b| b.area).sum();
    assert!(total_bond > 0.3 * 2.0, "{total_bond}");
    // Anchors: the chunks on the floor, and only those.
    let anchored: Vec<_> = a.chunks[..20]
        .iter()
        .filter(|c| c.anchor_area > 0.0)
        .collect();
    assert!(!anchored.is_empty());
    let floor: f32 = anchored.iter().map(|c| c.anchor_area).sum();
    assert!(
        (floor - 4.0 * 0.3).abs() < 1e-3,
        "the anchors cover the wall's foot: {floor}"
    );
    for c in &anchored {
        assert!(c.anchor_centroid.y.abs() < 1e-4);
    }
    // The model: a node a chunk at its centroid, hidden at rest, and the
    // intact model the one node drawn (no cut face shows through a crack
    // that is not there yet); the interior material after the brick.
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let model = bundle.load_model("props/wall.ogt").unwrap();
    assert_eq!(model.nodes.len(), a.chunks.len() + 1);
    for c in &a.chunks {
        let n = &model.nodes[c.node as usize];
        let t = n.translation;
        assert!((t - c.centroid).length() < 1e-5, "{t} {}", c.centroid);
        assert!(!n.visible);
    }
    let shown: Vec<_> = model.nodes.iter().filter(|n| n.visible).collect();
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].name, "intact");
    assert_eq!(model.materials[a.interior as usize].name, "interior");
}

#[test]
fn the_same_seed_cooks_the_same_chunks_on_any_thread_count() {
    let a = asset(&cook("og-cook-fracture-1", WALL, 1));
    let b = asset(&cook("og-cook-fracture-8", WALL, 8));
    assert_eq!(a, b);
    let c = asset(&cook(
        "og-cook-fracture-seed",
        &WALL.replace("seed 7", "seed 8"),
        0,
    ));
    assert_ne!(a.chunks[0].centroid, c.chunks[0].centroid);
}

/// The extent of a chunk's hull along each axis.
fn extent(c: &og_destruct::Chunk) -> glam::Vec3 {
    let (lo, hi) = c.hull.iter().fold(
        (glam::Vec3::splat(f32::MAX), glam::Vec3::splat(f32::MIN)),
        |(lo, hi), p| (lo.min(*p), hi.max(*p)),
    );
    hi - lo
}

const BRICK: &str = r#"type model
version 1
---
part "wall" {
  position 0 0.5 0
  boxGeometry {
    width 2
    height 1
    depth 0.2
  }
}
fracture {
  style brick
  brick 0.25 0.1
  chunks 8
  seed 3
  children 32
  anchor bottom ends
}
"#;

#[test]
fn brick_breaks_along_its_mortar_into_runs_of_whole_bricks_along_a_course_and_their_bricks() {
    let a = asset(&cook("og-cook-fracture-brick", BRICK, 0));
    let support = &a.chunks[..a.support as usize];
    let volume: f32 = support.iter().map(|c| c.volume).sum();
    assert!((volume - 0.4).abs() < 1e-4, "{volume}");
    // A block is a box of the half-brick grid: its cracks run in the
    // mortar (its volume its bounds').
    let on = |x: f32, step: f32| ((x / step).round() * step - x).abs() < 1e-4;
    for c in support {
        let (lo, hi) = c.hull.iter().fold(
            (glam::Vec3::splat(f32::MAX), glam::Vec3::splat(f32::MIN)),
            |(lo, hi), p| (lo.min(*p + c.centroid), hi.max(*p + c.centroid)),
        );
        for q in [lo, hi] {
            assert!(on(q.x + 1.0, 0.125) && on(q.y, 0.1), "{q} off the grid");
        }
        let e = hi - lo;
        assert!(
            (e.x * e.y * e.z - c.volume).abs() < 1e-5,
            "a box: {e} {}",
            c.volume
        );
        // One course of whole bricks, at most three: its ends on the
        // course's joints (or the wall's), so no joint runs up the wall.
        assert!((e.y - 0.1).abs() < 1e-4, "a course high: {e}");
        assert!(e.x <= 0.75 + 1e-4, "at most three bricks: {e}");
        let course = (c.centroid.y / 0.1).floor() as i32;
        let offset = if course % 2 == 1 { 0.125 } else { 0.0 };
        for x in [lo.x + 1.0, hi.x + 1.0] {
            assert!(
                on(x - offset, 0.25) || x < 1e-4 || x > 2.0 - 1e-4,
                "course {course}: a block ends at {x}, inside a brick"
            );
        }
    }
    // Its children are its bricks: a course high and at most a brick long,
    // laid in a running bond (a brick's ends on the half grid, offset a half
    // brick between courses), tiling it.
    let mut bricks = 0;
    for c in support.iter().filter(|c| c.children.1 > 0) {
        let (first, n) = c.children;
        let kids = &a.chunks[first as usize..(first + n) as usize];
        let sum: f32 = kids.iter().map(|k| k.volume).sum();
        assert!((sum - c.volume).abs() < 1e-5);
        for k in kids {
            let e = extent(k);
            assert!((e.y - 0.1).abs() < 1e-4 && e.x <= 0.25 + 1e-4, "{e}");
            let left = k.centroid.x - e.x * 0.5 + 1.0;
            let course = (k.centroid.y / 0.1).floor() as i32;
            // A whole brick starts on a joint of its course.
            if (e.x - 0.25).abs() < 1e-4 {
                let joint = left - if course % 2 == 1 { 0.125 } else { 0.0 };
                assert!(on(joint, 0.25), "course {course}: a brick from {left}");
            }
            bricks += 1;
        }
    }
    assert!(bricks > 40, "{bricks}");
    // Anchored on its foot and at both ends: 2 × 0.2 + 2 × 1 × 0.2.
    let anchor: f32 = support.iter().map(|c| c.anchor_area).sum();
    assert!((anchor - 0.8).abs() < 1e-4, "{anchor}");
}

const PANE: &str = r#"type model
version 1
---
part "pane" {
  position 0 0.6 0
  boxGeometry {
    width 1.6
    height 1.2
    depth 0.01
  }
}
fracture {
  style glass
  chunks 25
  rings 2
  seed 4
  impact 0.3 0.7 0 0.2
}
"#;

#[test]
fn glass_cracks_radially_and_in_rings_round_its_impact() {
    let a = asset(&cook("og-cook-fracture-glass", PANE, 0));
    let support = &a.chunks[..a.support as usize];
    assert!(
        support.len() >= 18 && support.len() <= 25,
        "{}",
        support.len()
    );
    let hit = glam::Vec3::new(0.3, 0.7, 0.0);
    let near: Vec<f32> = support
        .iter()
        .filter(|c| (c.centroid - hit).length() < 0.15)
        .map(|c| c.volume)
        .collect();
    let far: Vec<f32> = support
        .iter()
        .filter(|c| (c.centroid - hit).length() > 0.5)
        .map(|c| c.volume)
        .collect();
    let mean = |v: &[f32]| v.iter().sum::<f32>() / v.len() as f32;
    assert!(!near.is_empty() && !far.is_empty());
    assert!(
        mean(&near) * 3.0 < mean(&far),
        "small shards at the blow, large ones away: {} {}",
        mean(&near),
        mean(&far)
    );
    // The shards outside the centre are wedges pointing at the impact: each
    // longer along the ray from it than across.
    let mut radial = 0;
    let mut outer = 0;
    for c in support.iter().filter(|c| (c.centroid - hit).length() > 0.4) {
        let ray = (c.centroid - hit).normalize();
        let across = glam::Vec3::new(-ray.y, ray.x, 0.0);
        let span = |d: glam::Vec3| {
            let (lo, hi) = c.hull.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| {
                (lo.min(p.dot(d)), hi.max(p.dot(d)))
            });
            hi - lo
        };
        outer += 1;
        if span(ray) > span(across) {
            radial += 1;
        }
    }
    assert!(outer > 4 && radial * 3 >= outer * 2, "{radial} of {outer}");
}

#[test]
fn wood_splinters_along_its_grain() {
    let text = r#"type model
version 1
---
part "box" {
  boxGeometry {
    width 0.8
    height 0.8
    depth 0.8
  }
}
fracture {
  style wood
  grain x
  chunks 12
  seed 2
  anchor none
}
"#;
    let a = asset(&cook("og-cook-fracture-wood", text, 0));
    let support = &a.chunks[..a.support as usize];
    let (mut along, mut across) = (0.0, 0.0);
    for c in support {
        let e = extent(c);
        along += e.x;
        across += (e.y + e.z) * 0.5;
    }
    assert!(
        along > across * 2.0,
        "splinters run along x: {along} against {across}"
    );
}

#[test]
fn material_maps_and_the_interior_s_cook_into_the_model_and_are_part_of_it() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook-fracture-maps");
    let _ = std::fs::remove_dir_all(&root);
    let png = |name: &str, rgb: [u8; 3]| {
        let mut pixels = Vec::new();
        for _ in 0..16 * 16 {
            pixels.extend([rgb[0], rgb[1], rgb[2], 255]);
        }
        let img = og_render::RgbaImage {
            width: 16,
            height: 16,
            pixels,
        };
        og_tests::gfx::save_png(&root.join("assets/textures").join(name), &img).unwrap();
    };
    png("brick.png", [160, 60, 40]);
    png("brick_n.png", [128, 128, 255]);
    png("cut.png", [150, 120, 100]);
    std::fs::create_dir_all(root.join("assets/props")).unwrap();
    std::fs::write(
        root.join("assets/props/wall.ogt"),
        r#"type model
version 1
---
material "brick" {
  map "../textures/brick.png"
  normalMap "../textures/brick_n.png"
  normalScale 0.5
}
material "glass" {
  transmission 1
  ior 1.5
  roughness 0
}
part "wall" {
  material "brick"
  position 0 0.5 0
  boxGeometry {
    width 1
    height 1
    depth 0.2
  }
}
part "pane" {
  material "glass"
  position 0 1.5 0
  boxGeometry {
    width 1
    height 1
    depth 0.01
  }
}
fracture {
  chunks 4
  interior {
    map "../textures/cut.png"
  }
}
"#,
    )
    .unwrap();
    let cook = || {
        let r = cook_project(&root, &CookOptions::default()).unwrap();
        assert!(r.ok(), "{:?}", r.errors());
        std::fs::read(root.join("cooked/props/wall.ogt.ogb")).unwrap()
    };
    let first = cook();
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let model = bundle.load_model("props/wall.ogt").unwrap();
    let by = |n: &str| model.materials.iter().find(|m| m.name == n).unwrap();
    let (brick, glass, cut) = (by("brick"), by("glass"), by("interior"));
    assert!(brick.base_color_texture.is_some() && brick.normal_texture.is_some());
    assert_eq!(brick.normal_scale, 0.5);
    assert_eq!(glass.physical.transmission, 1.0);
    assert_eq!(glass.physical.ior, 1.5);
    assert!(cut.base_color_texture.is_some());
    assert_ne!(cut.base_color_texture, brick.base_color_texture);
    assert_eq!(model.textures.len(), 3);
    // The images are the model's: no texture asset of their own, and an
    // edit to one cooks the model again.
    assert!(bundle.output("textures/cut.png").is_err());
    png("cut.png", [90, 90, 90]);
    assert_ne!(cook(), first, "the interior's map is part of the model");
}

#[test]
fn every_chunk_hull_holds_a_volume() {
    let root = cook(
        "og-cook-scratch-pillar",
        r#"type model
version 1
---
# A concrete pillar, 0.5 m across and 3 m tall: a blow at its middle takes
# it away in chunky lumps that show their gravel, and what stood on it
# comes down.
material "concrete" {
  color 1 1 1
  roughness 0.85
}
part "shaft" {
  material "concrete"
  position 0 1.5 0
  cylinderGeometry {
    radiusTop 0.25
    radiusBottom 0.25
    height 3
    radialSegments 20
    uvRepeat 1.6 3
  }
}
fracture {
  style concrete
  chunks 48
  seed 5
  children 5
  interiorScale 2.5
  interior {
    color 1 1 1
    roughness 0.95
  }
}
"#,
        0,
    );
    let a = asset(&root);
    // A CSG sliver (triangles in one plane, unwelded to the rest of their
    // cell) once cooked into a chunk of its own: a hull with no volume, a
    // body of no mass, which physics then disabled mid-flight.
    for (i, c) in a.chunks.iter().enumerate() {
        let h: Vec<glam::DVec3> = c.hull.iter().map(|v| v.as_dvec3()).collect();
        let lo = h
            .iter()
            .fold(glam::DVec3::splat(f64::MAX), |m, v| m.min(*v));
        let hi = h
            .iter()
            .fold(glam::DVec3::splat(f64::MIN), |m, v| m.max(*v));
        // A greedy tetrahedron: the far point, the point farthest off that
        // line, the point farthest off that plane.
        let far = |f: &dyn Fn(glam::DVec3) -> f64| {
            h.iter().copied().max_by(|p, q| f(*p).total_cmp(&f(*q)))
        };
        let best = match h.first() {
            Some(&x) => {
                let y = far(&|p| p.distance_squared(x)).unwrap();
                let z = far(&|p| (p - x).cross(y - x).length_squared()).unwrap();
                let n = (y - x).cross(z - x);
                let w = far(&|p| n.dot(p - x).abs()).unwrap();
                n.dot(w - x).abs() / 6.0
            }
            None => 0.0,
        };
        assert!(
            h.len() >= 4 && best > 1e-3 * f64::from(c.volume),
            "chunk {i} (parent {}): a hull of {} points spanning {best:e} m³ (box {:?}) for {:e} m³",
            c.parent,
            h.len(),
            hi - lo,
            c.volume
        );
    }
}
