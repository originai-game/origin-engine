//! gfx reference frames: each scene rendered headless must match its
//! committed reference for this backend tier (see og_tests::gfx).

use og_tests::gfx;
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx")
}

fn check(scene: &str) -> gfx::GfxReport {
    match gfx::check(scene, &out_dir()) {
        Ok(report) => {
            println!("{report}");
            report
        }
        Err(e) => panic!("{e}"),
    }
}

/// The spike's reference scene (M0 reproduced it byte for byte before the
/// sky pass; `tests/evidence/M0/spike-frame-reproduced.png`).
#[test]
fn gltf_reference_matches_its_reference() {
    check("gltf-reference");
}

/// 20,000 spheres submitted one instance at a time are drawn as two batches
/// (spheres, ground): one instanced draw per batch in the shadow and main
/// passes, plus the sky and post passes.
#[test]
fn spheres_20k_are_batched_automatically() {
    let report = check("spheres-20k");
    assert_eq!(
        report.stats.instances, 20_001,
        "20,000 spheres and the ground"
    );
    assert_eq!(report.stats.batches, 2, "one batch per (mesh, material)");
    assert_eq!(
        report.stats.draw_calls, 6,
        "shadow 2 + main 2 + sky 1 + post 1"
    );
    assert_eq!(report.stats.triangles, 20_000 * 384 + 2);
}
