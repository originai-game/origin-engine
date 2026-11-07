//! UI documents through the cook and back (docs/ui.md §2.1): a scriptless
//! Svelte screen cooks to an `og.UiDocument` blob whose text og-ui parses
//! into the same document, and a document that does not validate against
//! og-ui's reflected props is refused with its line.

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::PathBuf;

const SCREEN: &str = r#"<meta name="screen" content="shop" data-layer="menu">
<meta name="prop:gold" content="number" data-doc="Gold.">
<column id="Shop">
  <text id="Gold">{$gold}</text>
</column>
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
    let root = project("ok", &[("ui/shop.svelte", SCREEN)]);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let id = "ui/shop.svelte";
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
fn string_table_edits_recook_the_document_and_missing_tables_fail() {
    let source = format!("<meta name='strings' content='./shop.en.ogt'>\n{SCREEN}");
    let english = "type strings\nversion 1\n---\nlocale \"en\"\nstring \"shop.title\" \"Shop\"\n";
    let root = project(
        "strings",
        &[("ui/shop.svelte", &source), ("ui/shop.en.ogt", english)],
    );
    let options = CookOptions::default();
    let report = cook_project(&root, &options).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let before = Bundle::open(&root.join("cooked")).unwrap();
    let output = before.output("ui/shop.svelte").unwrap();
    let bytes = std::fs::read(root.join("cooked").join(&output.file)).unwrap();
    let text = before.document_text("ui/shop.svelte", &bytes).unwrap();
    let doc = og_ui::UiDocument::parse("ui/shop.svelte", &text).unwrap();
    assert_eq!(doc.strings, ["ui/shop.en.ogt"]);
    let stamp = output.stamp;
    let hash = output.hash;

    std::fs::write(
        root.join("assets/ui/shop.en.ogt"),
        english.replace("Shop", "Market"),
    )
    .unwrap();
    let report = cook_project(&root, &options).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let after = Bundle::open(&root.join("cooked")).unwrap();
    let output = after.output("ui/shop.svelte").unwrap();
    assert_ne!(
        output.stamp, stamp,
        "an unchanged document depends on its messages"
    );
    assert_ne!(
        output.hash, hash,
        "hot reload observes the cooked document hash"
    );

    std::fs::remove_file(root.join("assets/ui/shop.en.ogt")).unwrap();
    let report = cook_project(&root, &options).unwrap();
    assert!(!report.ok());
    assert!(
        report
            .errors()
            .iter()
            .any(|e| e.contains("ui/shop.svelte") && e.contains("ui/shop.en.ogt"))
    );
    assert!(
        Bundle::open(&root.join("cooked"))
            .unwrap()
            .output("ui/shop.svelte")
            .is_err()
    );
}

#[test]
fn rejects_legacy_documents_and_invalid_template_expressions() {
    let root = project(
        "legacy",
        &[(
            "ui/shop.ogt",
            "type screen\nversion 2\n---\nname \"shop\"\ncolumn Shop {}\n",
        )],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(!report.ok());
    assert!(report.errors().iter().any(|e| e.contains(".svelte")));
    for (name, body, reason) in [
        (
            "unkeyed",
            "<column>{#each $items as item}<text>{item.name}</text>{/each}</column>",
            "stable key",
        ),
        ("effect", "<text>{$gold = 4}</text>", "side-effect-free"),
        ("element", "<mystery/>", "unknown element"),
    ] {
        let source = format!("<meta name='screen' content='shop'>\n{body}");
        let root = project(name, &[("ui/shop.svelte", &source)]);
        let report = cook_project(&root, &CookOptions::default()).unwrap();
        assert!(!report.ok());
        assert!(
            report
                .errors()
                .iter()
                .any(|e| e.contains("ui/shop.svelte:2:") && e.contains(reason)),
            "{:?}",
            report.errors()
        );
    }
}

#[test]
fn css_tokens_survive_cooking_and_unresolved_names_fail() {
    let source = format!(
        "{SCREEN}<style>#Shop {{ border-radius: var(--radius-lg); background: var(--panel); }}</style>"
    );
    let root = project("tokens", &[("ui/shop.svelte", &source)]);
    let options = CookOptions::default();
    let report = cook_project(&root, &options).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let out = bundle.output("ui/shop.svelte").unwrap();
    let bytes = std::fs::read(root.join("cooked").join(&out.file)).unwrap();
    let text = bundle.document_text("ui/shop.svelte", &bytes).unwrap();
    assert!(
        text.contains("var(--radius-lg)"),
        "cook must not freeze a style's radius"
    );
    for (from, missing) in [
        ("--radius-lg", "--radius-missing"),
        ("--panel", "--missing-colour"),
    ] {
        std::fs::write(
            root.join("assets/ui/shop.svelte"),
            source.replace(from, missing),
        )
        .unwrap();
        let report = cook_project(&root, &options).unwrap();
        assert!(!report.ok());
        assert!(format!("{:?}", report.errors()).contains(missing));
    }
}

/// An installed kit (docs/ui.md §5) cooks with the game: its documents
/// are the game's assets under `ui-kits/<kit>/`, and its review sheet, a
/// project of its own, is left out.
#[test]
fn an_installed_kit_cooks_its_documents_and_not_its_sheet() {
    let kit = "ui-kits/racing";
    let root = project(
        "kit",
        &[
            (&format!("{kit}/extension.json"), "{}"),
            (&format!("{kit}/results.svelte"), SCREEN),
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
        bundle.output(&format!("{kit}/results.svelte")).unwrap().ty,
        "og.UiDocument"
    );
    assert!(
        bundle
            .output(&format!("{kit}/sheet/assets/looks/scifi.ogt"))
            .is_err()
    );
}
