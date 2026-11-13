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
use og_physics_data::fracture::FractureAsset;
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
fn unchanged_fracture_is_reused_but_source_and_dependency_edits_rebuild_it() {
    let text = WALL
        .replace("chunks 20", "chunks 4")
        .replace("children 4", "children 0");
    let root = cook("og-cook-fracture-reuse", &text, 1);
    let options = CookOptions::default();
    let cache = root.join(".og/test-cache");
    let rerun = || {
        let report =
            og_cook::cook::cook_project_with_texture_cache(&root, &options, Some(&cache)).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        report
    };
    let reused = |r: &og_cook::Report| {
        r.assets
            .iter()
            .any(|a| a.notes.iter().any(|n| n.contains("reused prefracture")))
    };
    let original = asset(&root).to_bytes();
    std::fs::remove_file(root.join(".og/cook-index")).unwrap();
    rerun();
    let report = rerun();
    assert!(reused(&report));
    assert!(
        report
            .assets
            .iter()
            .flat_map(|a| &a.outputs)
            .all(|o| !o.cooked)
    );
    assert_eq!(original, asset(&root).to_bytes());

    let old = root.join("assets/props/wall.ogt");
    let renamed = root.join("assets/props/renamed.ogt");
    std::fs::rename(&old, &renamed).unwrap();
    assert!(
        reused(&rerun()),
        "rename must reuse the content-keyed fracture"
    );
    std::fs::rename(&renamed, &old).unwrap();
    rerun();
    assert_eq!(original, asset(&root).to_bytes());

    let text = text.replace("seed 7", "seed 8");
    std::fs::write(root.join("assets/props/wall.ogt"), &text).unwrap();
    assert!(!reused(&rerun()));
    assert_ne!(original, asset(&root).to_bytes());
    assert!(reused(&rerun()));

    let map = root.join("assets/props/interior.png");
    let write_map = |color| {
        image::RgbaImage::from_pixel(2, 2, image::Rgba(color))
            .save(&map)
            .unwrap()
    };
    write_map([255, 0, 0, 255]);
    let text = text.replace("interior {", "interior {\n    map \"interior.png\"");
    std::fs::write(root.join("assets/props/wall.ogt"), &text).unwrap();
    assert!(!reused(&rerun()));
    assert!(reused(&rerun()));
    let manifest = std::fs::read(root.join("cooked/.cook.ogt")).unwrap();
    write_map([0, 255, 0, 255]);
    assert!(!reused(&rerun()));
    assert_ne!(
        manifest,
        std::fs::read(root.join("cooked/.cook.ogt")).unwrap()
    );
    assert!(reused(&rerun()));
    std::fs::remove_file(map).unwrap();
    assert!(!cook_project(&root, &options).unwrap().ok());
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
    // No mortar block, no mortar.
    assert_eq!(a.mortar, og_physics_data::fracture::NO_MATERIAL);
}

/// The extent of a chunk's hull along each axis.
fn extent(c: &og_physics_data::fracture::Chunk) -> glam::Vec3 {
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
