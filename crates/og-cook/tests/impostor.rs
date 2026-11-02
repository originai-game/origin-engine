//! `model { impostor true }`: the cook bakes a model's octahedral impostor
//! into two atlases and names them, with the sphere they cover and the
//! error that makes it the model's last level, in the model's blob.
//!
//! Ways this could fail: the setting is ignored (no row, no atlases) or
//! its sizes are; the row names atlases the bundle does not hold; the
//! sphere is not the one og-runtime's levels measure (the bounds'); a
//! size past the limits, or one given without `impostor true`, is cooked
//! silently.

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::PathBuf;

fn project(name: &str, settings: &str) -> (PathBuf, bool) {
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
    std::fs::write(
        models.join("dropship.glb.ogt"),
        format!("type import\nversion 1\n---\nmodel {{\n{settings}\n}}\n"),
    )
    .unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    (root, report.ok())
}

#[test]
fn impostor_true_bakes_two_atlases_the_model_names() {
    let (root, ok) = project(
        "og-cook-impostor",
        "  impostor true\n  impostorGrid 4\n  impostorSize 64",
    );
    assert!(ok);
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let model = bundle.load_model("models/dropship.glb").unwrap();
    let i = model.impostor.clone().expect("an impostor row");
    assert_eq!(i.albedo, "models/dropship.glb#impostor");
    assert_eq!(i.normal, "models/dropship.glb#impostor-normal");
    assert_eq!((i.grid, i.frame), (4, 64));
    let center = (model.min + model.max) * 0.5;
    let radius = ((model.max - model.min) * 0.5).length();
    assert!((i.center - center).length() < 1e-5 && (i.radius - radius).abs() < 1e-5);
    assert!((i.error - 2.0 * radius / 64.0).abs() < 1e-6);
    for id in [&i.albedo, &i.normal] {
        let out = bundle
            .output(id)
            .unwrap_or_else(|e| panic!("{id} not cooked: {e}"));
        assert!(root.join("cooked").join(&out.file).exists());
    }
    // Without the setting: no row, no atlases.
    let (plain, ok) = project("og-cook-impostor-off", "  raycast true");
    assert!(ok);
    let bundle = Bundle::open(&plain.join("cooked")).unwrap();
    assert!(
        bundle
            .load_model("models/dropship.glb")
            .unwrap()
            .impostor
            .is_none()
    );
    assert!(bundle.output("models/dropship.glb#impostor").is_err());
}

#[test]
fn impostor_sizes_out_of_range_are_refused() {
    for (k, bad) in [
        "  impostorGrid 8",
        "  impostor true\n  impostorGrid 1",
        "  impostor true\n  impostorSize 1024",
        "  impostor true\n  impostorGrid 32\n  impostorSize 256",
        "  impostor sometimes",
    ]
    .into_iter()
    .enumerate()
    {
        let (_, ok) = project(&format!("og-cook-impostor-bad-{k}"), bad);
        assert!(!ok, "{bad:?} was cooked");
    }
}
