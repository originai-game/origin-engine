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
    // The model: a node a chunk at its centroid, children hidden at rest,
    // and the interior material after the brick.
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let model = bundle.load_model("props/wall.ogt").unwrap();
    assert_eq!(model.nodes.len(), a.chunks.len());
    for c in &a.chunks {
        let n = &model.nodes[c.node as usize];
        let t = n.translation;
        assert!((t - c.centroid).length() < 1e-5, "{t} {}", c.centroid);
        assert_eq!(n.visible, c.parent == og_destruct::NONE);
    }
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
