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
fn pack_edits_leave_document_blobs_cached() {
    let root = project(
        "pack-stamp",
        &[
            ("ui/shop.svelte", SCREEN),
            ("styles/test/main.css", "button { opacity: 0.5; }"),
        ],
    );
    let cook = || {
        let report = cook_project(&root, &CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        (
            bundle.output("ui/shop.svelte").unwrap().stamp,
            bundle.output("styles/test").unwrap().stamp,
        )
    };
    let before = cook();
    std::fs::write(
        root.join("assets/styles/test/main.css"),
        "button { opacity: 0.8; }",
    )
    .unwrap();
    let after = cook();
    assert_eq!(
        before.0, after.0,
        "documents are not lowered against a pack at cook time"
    );
    assert_ne!(before.1, after.1, "the changed pack must recook");
    std::fs::write(
        root.join("assets/ui/shop.svelte"),
        SCREEN.replace("Gold", "Coins"),
    )
    .unwrap();
    assert_ne!(
        after.0,
        cook().0,
        "a document's own source still invalidates it"
    );
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
fn css_tokens_cook_symbolically_and_missing_values_fail_when_computed() {
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
        assert!(report.ok(), "{:?}", report.errors());
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let out = bundle.output("ui/shop.svelte").unwrap();
        let bytes = std::fs::read(root.join("cooked").join(&out.file)).unwrap();
        let text = bundle.document_text("ui/shop.svelte", &bytes).unwrap();
        let mut ui = og_ui::Ui::new(og_ui::UiDesc::default());
        ui.define_document(og_ui::UiDocument::parse("ui/shop.svelte", &text).unwrap());
        ui.push_screen("shop", serde_json::json!({"gold": 1}))
            .unwrap();
        ui.begin(
            og_ui::InputFrame::default(),
            og_ui::Viewport::new(640.0, 480.0),
        );
        ui.end();
        assert!(
            ui.frame()
                .warnings
                .iter()
                .any(|error| error.contains(missing) && error.contains("computed-value")),
            "{:?}",
            ui.frame().warnings
        );
    }
}

/// The document ids a cooked document loads with it.
fn uses(root: &std::path::Path, id: &str) -> Vec<String> {
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let bytes = std::fs::read(root.join("cooked").join(&bundle.output(id).unwrap().file)).unwrap();
    bundle.document_uses(id, &bytes).unwrap()
}

const VITALS: &str = r#"<meta name="screen" content="vitals">
<column id="Vitals"><StatBar value={50} max={100} /></column>
"#;

/// A screen that uses a built-in component by name cooks it in: the
/// engine's document joins the bundle under `@ui/` with its string table,
/// and the screen names it to load with it. Ways it could fail: the
/// component left out (the screen mounts nothing in its place), its
/// strings left out (its labels show their keys), or the screen not
/// naming it (it is never defined).
#[test]
fn a_screen_cooks_in_the_built_in_components_it_uses() {
    let root = project("built-in", &[("ui/vitals.svelte", VITALS)]);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{}", report.errors().join("\n"));
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(
        bundle.output("@ui/components/hud/stat-bar.svelte").unwrap().ty,
        "og.UiDocument"
    );
    assert!(bundle.output("@ui/components/hud/stat-bar.en.ogt").is_ok());
    assert!(
        bundle.output("@ui/components/hud/ability-bar.svelte").is_err(),
        "only what it uses"
    );
    assert_eq!(
        uses(&root, "ui/vitals.svelte"),
        ["@ui/components/hud/stat-bar.svelte"]
    );
}

/// A document with a control element cooks in that element's control
/// document (docs/ui.md §2.4), and a game's own control document for the
/// element replaces the built-in one. Ways it could fail: the control left
/// out (the element draws its fallback), every control bundled whatever
/// the game shows, or the built-in one loaded beside the game's.
#[test]
fn a_document_cooks_in_the_controls_its_elements_need() {
    let bar = r#"<meta name="screen" content="loading">
<div id="Loading"><progress id="Done" value={$done}></progress></div>
"#;
    let root = project("control", &[("ui/loading.svelte", bar)]);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{}", report.errors().join("\n"));
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    assert!(bundle.output("@ui/controls/progress.svelte").is_ok());
    assert!(
        bundle.output("@ui/controls/meter.svelte").is_err(),
        "only the controls it shows"
    );
    assert_eq!(
        uses(&root, "ui/loading.svelte"),
        ["@ui/controls/progress.svelte"]
    );
    let mine = r#"<meta name="control" content="progress">
<meta name="prop:value" content="number" data-doc="Done, 0 to 1.">
<div id="Mine"></div>
"#;
    let root = project(
        "control-override",
        &[
            ("ui/loading.svelte", bar),
            ("ui/controls/progress.svelte", mine),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{}", report.errors().join("\n"));
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    assert!(bundle.output("@ui/controls/progress.svelte").is_err());
    assert_eq!(
        uses(&root, "ui/loading.svelte"),
        ["ui/controls/progress.svelte"]
    );
}

/// A game's own document declaring a built-in component's name (what
/// `og ui eject` writes) replaces it: the screen loads the game's, and
/// the engine's is not bundled. Ways it could fail: both bundled, and the
/// built-in one loaded over the game's edit.
#[test]
fn a_game_component_of_a_built_in_name_wins() {
    let mine = r#"<meta name="component" content="StatBar">
<meta name="prop:value" content="number" data-doc="The value.">
<meta name="prop:max" content="number" data-doc="Its maximum.">
<div id="Mine"></div>
"#;
    let root = project(
        "built-in-override",
        &[("ui/vitals.svelte", VITALS), ("ui/my-bar.svelte", mine)],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{}", report.errors().join("\n"));
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    assert!(bundle.output("@ui/components/hud/stat-bar.svelte").is_err());
    assert_eq!(uses(&root, "ui/vitals.svelte"), ["ui/my-bar.svelte"]);
}

/// A document an `<iframe>` embeds loads with its embedder, and one that
/// names no document of the project is refused at cook. Ways it could
/// fail: the frame's document left out of the embedder's loads (the frame
/// mounts nothing at run time), or a typo in `src` cooking clean.
#[test]
fn an_iframes_document_loads_with_its_embedder() {
    let page = r#"<meta name="screen" content="page">
<div><iframe src="news.svelte"></iframe></div>
"#;
    let news = r#"<meta name="screen" content="news">
<div id="News"></div>
"#;
    let root = project(
        "iframe",
        &[("ui/page.svelte", page), ("ui/news.svelte", news)],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{}", report.errors().join("\n"));
    assert_eq!(uses(&root, "ui/page.svelte"), ["ui/news.svelte"]);
    let root = project("iframe-missing", &[("ui/page.svelte", page)]);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(
        report.errors().iter().any(|e| e.contains("ui/news.svelte")),
        "{:?}",
        report.errors()
    );
    // A remote one cooks only from an origin project.ogt's `frames` lists.
    let remote = r#"<meta name="screen" content="page">
<div><iframe src="https://events.example.com/og/a.svelte"></iframe></div>
"#;
    let root = project("iframe-remote", &[("ui/page.svelte", remote)]);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(
        report.errors().iter().any(|e| e.contains("`frames`")),
        "{:?}",
        report.errors()
    );
    std::fs::write(
        root.join("project.ogt"),
        "type project\nversion 1\n---\nname \"p\"\nframes \"https://events.example.com\"\n",
    )
    .unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{}", report.errors().join("\n"));
    assert!(
        uses(&root, "ui/page.svelte").is_empty(),
        "a remote one is fetched, not bundled"
    );
}

/// A document's `@import` is read from beside it (with the imports of the
/// imported sheet) into its blob, and an edit to the stylesheet re-cooks
/// the document. Ways it could fail: the runtime parsing the text without
/// the stylesheet (it refuses the `@import`), or a stale blob after the
/// sheet changed.
#[test]
fn imported_stylesheets_ride_in_the_document_blob() {
    let source = SCREEN.replace(
        "</column>\n",
        "</column>\n<style>@import \"theme/base.css\";</style>\n",
    );
    let root = project(
        "imports",
        &[
            ("ui/shop.svelte", &source),
            (
                "ui/theme/base.css",
                "@import \"tokens.css\";\n#Gold { opacity: 0.5; }",
            ),
            ("ui/theme/tokens.css", "#Shop { --tone: 1; }"),
        ],
    );
    let cooked = |root: &PathBuf| {
        let report = cook_project(root, &CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let out = bundle.output("ui/shop.svelte").unwrap();
        let bytes = std::fs::read(root.join("cooked").join(&out.file)).unwrap();
        let text = bundle.document_text("ui/shop.svelte", &bytes).unwrap();
        let imports = bundle.document_imports("ui/shop.svelte", &bytes).unwrap();
        og_ui::UiDocument::parse_with_imports("ui/shop.svelte", &text, imports.clone()).unwrap();
        imports
    };
    let imports = cooked(&root);
    let paths: Vec<_> = imports.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(paths, ["ui/theme/base.css", "ui/theme/tokens.css"]);
    std::fs::write(
        root.join("assets/ui/theme/tokens.css"),
        "#Shop { --tone: 2; }",
    )
    .unwrap();
    let imports = cooked(&root);
    assert!(imports[1].1.contains("--tone: 2"), "{imports:?}");
}
