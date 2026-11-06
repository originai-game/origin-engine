//! UI documents through the cook and back (docs/ui.md §10.1): a screen
//! (`type screen`) cooks to an `og.UiDocument` blob whose text og-ui parses
//! into the same document, and a document that does not validate against
//! og-ui's reflected props is refused with its line.

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::PathBuf;

const SCREEN: &str = r#"type screen
version 1
---
name "shop"
layer menu
props {
  gold number "Gold."
}
column Shop {
  label Gold {
    text bind gold compact
  }
}
"#;

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-documents")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, text) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    root
}

#[test]
fn a_screen_cooks_to_a_document_blob() {
    let root = project("ok", &[("ui/shop.ogt", SCREEN)]);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let id = "ui/shop.ogt";
    let out = bundle.output(id).unwrap();
    assert_eq!(out.ty, "og.UiDocument");
    let bytes = std::fs::read(root.join("cooked").join(&out.file)).unwrap();
    let text = bundle.document_text(id, &bytes).unwrap();
    let doc = og_ui::UiDocument::parse(id, &text).unwrap();
    assert_eq!(doc.name, "shop");
    assert_eq!(doc.layer.kind, og_ui::doc::LayerKind::Menu);
    assert!(doc.node_at("Gold").is_some());
}

#[test]
fn a_document_off_the_schema_is_refused_with_its_line() {
    let bad = SCREEN.replace("text bind gold compact", "colour token ink");
    let root = project("bad", &[("ui/shop.ogt", &bad)]);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors().join("\n");
    assert!(
        errors.contains("ui/shop.ogt: line 11: 'colour' is not a prop"),
        "{errors}"
    );
}

/// An installed kit (docs/ui.md §17.1) cooks with the game: its documents
/// are the game's assets under `ui-kits/<kit>/`, and its review sheet, a
/// project of its own, is left out.
#[test]
fn an_installed_kit_cooks_its_documents_and_not_its_sheet() {
    let kit = "ui-kits/racing";
    let root = project(
        "kit",
        &[
            (&format!("{kit}/extension.json"), "{}"),
            (&format!("{kit}/results.ogt"), SCREEN),
            (
                &format!("{kit}/sheet/project.ogt"),
                "type project\nversion 1\n---\nname \"Sheet\"\n",
            ),
            (
                &format!("{kit}/sheet/assets/looks/scifi.ogt"),
                "type look\nversion 1\n---\nname \"sheet\"\nui \"scifi\"\n",
            ),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{}", report.errors().join("\n"));
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(
        bundle.output(&format!("{kit}/results.ogt")).unwrap().ty,
        "og.UiDocument"
    );
    assert!(
        bundle
            .output(&format!("{kit}/sheet/assets/looks/scifi.ogt"))
            .is_err()
    );
}
