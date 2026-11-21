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
        "a pack rule only the old core reads leaves documents cached"
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
    // A pack's tokens are what the new core's compile reads (its styles
    // are compiled against each look at cook time): editing them cooks
    // the documents again.
    let before = cook();
    std::fs::write(
        root.join("assets/styles/test/tokens.css"),
        ":root { --space-sm: 3px; }",
    )
    .unwrap();
    assert_ne!(before.0, cook().0, "a pack's tokens recook the documents");
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

/// A game whose bundle reaches the engine's TSX UI (its import graph,
/// `CookOptions::code`) carries the default face, the default pack's
/// shapes (`nine-disc` …) and the string tables of the components it
/// imports, and only those. Ways it could fail: a TSX-only game cooks
/// without the face, so its tree has no body font and draws no text, or
/// without the shapes, so every `nine-<name>` of the default pack draws
/// nothing; a component's table left out shows its raw keys; a table the
/// bundle does not hold ships anyway; another graph keeps the old bundle.
#[test]
fn code_reaching_the_tsx_ui_carries_the_default_face_and_its_tables() {
    let root = project("code-tsx-face", &[]);
    std::fs::create_dir_all(root.join("assets")).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("project.ogt"),
        "type project\nversion 1\n---\nname \"p\"\n",
    )
    .unwrap();
    std::fs::write(root.join("src/main.ts"), "export default 1;\n").unwrap();
    let cook = |code: og_cook::cook::CodeGraph| {
        let options = CookOptions {
            code,
            ..CookOptions::default()
        };
        let r = og_cook::cook::cook_project_with_texture_cache(&root, &options, None).unwrap();
        assert!(r.ok(), "{}", r.errors().join("\n"));
        Bundle::open(&root.join("cooked")).unwrap()
    };
    let shapes = ["disc", "ring", "tile", "plate-cut", "panel-cut", "chip-cut"]
        .map(|s| format!("@styles/shared/shapes/{s}.svg"));
    let bundle = cook(Default::default());
    assert!(
        bundle.output(og_ui::DEFAULT_FONT_ASSET).is_err(),
        "no UI, no face"
    );
    assert!(
        shapes.iter().all(|id| bundle.output(id).is_err()),
        "no UI, no shapes"
    );
    let ui = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/engine/src/ui");
    let (bar, ring) = (
        "@ui/components/hud/bar.en.ogt",
        "@ui/components/hud/progress-ring.en.ogt",
    );
    let bundle = cook(og_cook::cook::CodeGraph {
        tsx: true,
        tables: [(bar.to_string(), ui.join("components/hud/bar.en.ogt"))].into(),
        ..Default::default()
    });
    assert!(bundle.output(og_ui::DEFAULT_FONT_ASSET).is_ok());
    for id in &shapes {
        assert!(bundle.output(id).is_ok(), "{id}");
    }
    assert!(bundle.output(bar).is_ok(), "the reached table");
    assert!(
        bundle.output(ring).is_err(),
        "a table the bundle does not hold"
    );
}

/// An engine whose styles directory predates the default pack's shapes
/// fails the cook of a TSX UI naming the missing image, instead of cooking
/// without it so every `nine-<name>` draws nothing at run time. The styles
/// table is process-wide, so the cook runs in a child of this test.
#[test]
fn styles_missing_a_default_image_fail_the_cook() {
    if std::env::var_os("OG_TEST_STALE_STYLES").is_none() {
        let styles = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook-stale-styles");
        let _ = std::fs::remove_dir_all(&styles);
        std::fs::create_dir_all(styles.join("shared")).unwrap();
        std::fs::write(styles.join("shared/LICENSES.md"), "").unwrap();
        let out = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "documents::styles_missing_a_default_image_fail_the_cook",
                "--nocapture",
            ])
            .env("OG_TEST_STALE_STYLES", "1")
            .env("OG_STYLES_DIR", &styles)
            .output()
            .unwrap();
        let log = String::from_utf8_lossy(&out.stdout);
        assert!(
            out.status.success() && log.contains("1 passed"),
            "{log}\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        return;
    }
    let root = project("stale-styles", &[]);
    std::fs::create_dir_all(root.join("assets")).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("project.ogt"),
        "type project\nversion 1\n---\nname \"p\"\n",
    )
    .unwrap();
    let options = CookOptions {
        code: og_cook::cook::CodeGraph {
            tsx: true,
            ..Default::default()
        },
        ..CookOptions::default()
    };
    let report = cook_project(&root, &options).unwrap();
    let errors = report.errors().join("\n");
    assert!(
        errors.contains("@styles/shared/shapes/disc.svg"),
        "{errors}"
    );
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

/// The built-in packs cook as `og.UiPack` blobs: every face, icon and sound
/// they name is an asset of the bundle, and their themes read whole.
#[test]
fn the_built_in_style_packs_cook_to_pack_blobs() {
    let looks: Vec<(String, String)> = ["cartoon", "fantasy", "scifi"]
        .map(|p| {
            (
                format!("looks/{p}.ogt"),
                format!("type look\nversion 1\n---\nui \"@styles/{p}\"\n"),
            )
        })
        .into();
    let files: Vec<(&str, &str)> = looks
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let root = project("built-in-packs", &files);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    for p in ["cartoon", "fantasy", "scifi"] {
        let id = format!("@styles/{p}/pack.json");
        assert_eq!(bundle.output(&id).unwrap().ty, "og.UiPack", "{p}");
        let text = bundle.pack_text(&id, &bundle.read(&id).unwrap()).unwrap();
        let layer = og_ui_next::pack::Layer::parse(&text).unwrap();
        assert!(!layer.fonts.is_empty() || p != "cartoon");
        for a in layer.assets() {
            assert!(bundle.output(a).is_ok(), "{p}: {a} is not in the bundle");
        }
    }
}

/// A pack naming a class its theme has no utility for, or an asset that
/// is not there, is refused with what and where.
#[test]
fn a_pack_with_an_unknown_class_or_a_missing_asset_is_refused() {
    for (name, pack, wants) in [
        (
            "pack-class",
            r#"{ "variants": { "Panel.frame": "rounded-card bg-surfcae" } }"#,
            "variants.Panel.frame: `bg-surfcae`",
        ),
        (
            "pack-asset",
            r#"{ "icons": { "star": "art/star.svg" } }"#,
            "styles/mine/art/star.svg",
        ),
        (
            "pack-form",
            r#"{ "theme": { "colours": {} } }"#,
            "theme.colours",
        ),
        (
            "pack-nine-name",
            r#"{ "variants": { "Panel.frame": "p-2 nine-frame" } }"#,
            "`nine-frame` names no image",
        ),
        (
            "pack-nine-asset",
            r#"{ "images": { "frame": { "src": "art/frame.png", "slice": 12 } }, "variants": { "Panel.frame": "nine-frame" } }"#,
            "styles/mine/art/frame.png",
        ),
    ] {
        let root = project(name, &[("styles/mine/pack.json", pack)]);
        let report = cook_project(&root, &CookOptions::default()).unwrap();
        let errors = format!("{:?}", report.errors());
        assert!(!report.ok() && errors.contains(wants), "{name}: {errors}");
    }
}

/// A variant naming the pack's nine-slice art cooks, its image resolved
/// beside pack.json, and a built-in face the pack names (no CSS naming it)
/// joins the bundle.
#[test]
fn a_variant_names_nine_slice_art() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48"><rect width="48" height="48" rx="8" fill="#336699"/></svg>"##;
    let pack = r#"{ "images": { "frame": { "src": "art/frame.svg", "slice": 12 } }, "variants": { "Panel.frame": "p-4 nine-frame" },
        "fonts": [{ "family": "Mine", "src": "@styles/shared/PTMono-Regular.ttf" }] }"#;
    let root = project(
        "pack-nine",
        &[
            ("styles/mine/pack.json", pack),
            ("styles/mine/art/frame.svg", svg),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let id = "styles/mine/pack.json";
    let layer =
        og_ui_next::pack::Layer::parse(&bundle.pack_text(id, &bundle.read(id).unwrap()).unwrap())
            .unwrap();
    assert_eq!(layer.images[1]["frame"].src, "styles/mine/art/frame.svg");
    assert!(bundle.output("styles/mine/art/frame.svg").is_ok());
    assert!(bundle.output("@styles/shared/PTMono-Regular.ttf").is_ok());
}
