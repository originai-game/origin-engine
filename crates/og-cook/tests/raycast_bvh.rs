//! A model's raycast BVH is cooked into its blob (depth S1's debt): each
//! primitive's `Bvh` over LOD 0, its bounds on the primitive's position
//! grid, so the runtime loads it with the mesh and the first
//! og.render.raycast builds nothing; `model { raycast false }` leaves it
//! out.
//!
//! Ways this could fail: the cooked tree is over the source positions (or
//! another order of the triangles) rather than the decoded LOD 0, so it
//! answers different faces; its bounds lose a hair in the quantisation and
//! a ray grazing an edge misses; the import setting is ignored.

use glam::Vec3;
use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use og_render::geometry::Bvh;
use std::path::PathBuf;

fn project(name: &str, settings: Option<&str>) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&root);
    let models = root.join("assets/models");
    std::fs::create_dir_all(&models).unwrap();
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::fs::copy(
        repo.join("templates/game/assets/models/dropship.glb"),
        models.join("dropship.glb"),
    )
    .unwrap();
    if let Some(s) = settings {
        std::fs::write(models.join("dropship.glb.ogt"), s).unwrap();
    }
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    root
}

#[test]
fn a_cooked_model_carries_the_bvh_a_build_over_its_mesh_makes() {
    let root = project("og-cook-raycast-bvh", None);
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let model = bundle.load_model("models/dropship.glb").unwrap();
    assert!(model.primitives.len() > 10);
    for (i, p) in model.primitives.iter().enumerate() {
        let mesh = p.mesh(0);
        let builds = Bvh::builds();
        let cooked = p
            .bvh(&mesh)
            .unwrap_or_else(|| panic!("primitive {i} has no cooked BVH"))
            .unwrap();
        assert_eq!(Bvh::builds(), builds, "loading the BVH built one");
        // Bit for bit the tree the runtime would have built at its first ray.
        assert_eq!(cooked, Bvh::build(&mesh), "primitive {i}");
        // And it answers rays as that tree does: a fan through the bounds.
        let (lo, hi) = cooked.bounds().unwrap();
        for k in 0..32 {
            let a = k as f32 * 0.7;
            let target = lo + (hi - lo) * Vec3::new(0.3 + 0.4 * a.sin().abs(), 0.5, 0.6);
            let origin =
                target + Vec3::new(a.cos(), 0.8, a.sin()) * (hi - lo).length().max(1.0) * 2.0;
            let dir = target - origin;
            let got = cooked.raycast(origin, dir, f32::INFINITY, true);
            let want = Bvh::build(&mesh).raycast(origin, dir, f32::INFINITY, true);
            assert_eq!(got, want, "primitive {i} ray {k}");
        }
    }
}

#[test]
fn raycast_false_cooks_no_bvh() {
    let with = project("og-cook-raycast-bvh-on", None);
    let without = project(
        "og-cook-raycast-bvh-off",
        Some("type import\nversion 1\n---\nmodel {\n  raycast false\n}\n"),
    );
    let size = |root: &PathBuf| {
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        std::fs::metadata(
            root.join("cooked")
                .join(&bundle.output("models/dropship.glb").unwrap().file),
        )
        .unwrap()
        .len()
    };
    let bundle = Bundle::open(&without.join("cooked")).unwrap();
    let model = bundle.load_model("models/dropship.glb").unwrap();
    assert!(model.primitives.iter().all(|p| p.bvh_parts.is_none()));
    assert!(
        size(&without) < size(&with),
        "{} vs {}",
        size(&without),
        size(&with)
    );
    // A malformed setting is refused, not cooked silently.
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook-raycast-bvh-bad");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/models")).unwrap();
    std::fs::copy(
        with.join("assets/models/dropship.glb"),
        root.join("assets/models/dropship.glb"),
    )
    .unwrap();
    std::fs::write(
        root.join("assets/models/dropship.glb.ogt"),
        "type import\nversion 1\n---\nmodel {\n  raycast maybe\n}\n",
    )
    .unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(!report.ok());
}
