//! A `.riv` through the cook (docs/ui.md §11.2): each artboard its
//! settings choose cooks to an `og.RiveArtboard` blob, id
//! `<file>.riv#<artboard>`, that og-ui decodes; what the import does not
//! convert fails the cook with the feature named.

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::PathBuf;

fn project(name: &str, files: &[&str]) -> PathBuf {
    let from = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/rive/assets");
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-rive")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/ui")).unwrap();
    for f in files {
        std::fs::copy(from.join(f), root.join("assets/ui").join(f)).unwrap();
    }
    root
}

#[test]
fn a_riv_cooks_the_artboards_its_settings_name() {
    let root = project(
        "ok",
        &[
            "two_artboards.riv",
            "two_artboards.riv.ogt",
            "nested_artboard_opacity.riv",
            "custom_image_name.riv",
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let load = |id: &str| {
        let out = bundle.output(id).unwrap_or_else(|e| panic!("{id}: {e:?}"));
        assert_eq!(out.ty, "og.RiveArtboard");
        let bytes = std::fs::read(root.join("cooked").join(&out.file)).unwrap();
        og_ui::rive::decode(&bundle.rive_artboard(id, &bytes).unwrap()).unwrap()
    };
    // The settings choose One of the two and set its role, name and fit.
    let one = load("ui/two_artboards.riv#One");
    assert_eq!(
        (one.role.as_str(), one.label.as_str()),
        ("button", "@chest.open")
    );
    assert_eq!(one.fit, og_ui::rive::Fit::Cover);
    assert_eq!(one.align, og_core::glam::Vec2::ZERO);
    assert!(bundle.output("ui/two_artboards.riv#Two").is_err());
    // Without settings, every artboard, a decorative image named after it.
    let parent = load("ui/nested_artboard_opacity.riv#Parent Artboard");
    assert_eq!(
        (parent.role.as_str(), parent.label.as_str()),
        ("image", "Parent Artboard")
    );
    assert_eq!(parent.nested.len(), 1);
    assert!(
        bundle
            .output("ui/nested_artboard_opacity.riv#Nested artboard")
            .is_ok()
    );
    // An embedded image is a texture of its own, which the artboard names.
    let image = load("ui/custom_image_name.riv#My Artboard");
    assert_eq!(image.images[0].id, "ui/custom_image_name.riv#image0");
    let tex = bundle
        .output("ui/custom_image_name.riv#image0")
        .expect("the image's texture");
    assert_eq!(tex.ty, "og.Texture");
}

#[test]
fn a_riv_the_import_cannot_convert_fails_naming_what() {
    let root = project("phase2", &["light_switch.riv"]);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors().join("\n");
    assert!(
        errors.contains("light_switch.riv")
            && errors.contains("StateMachineBool")
            && errors.contains("phase 2"),
        "{errors}"
    );
}
