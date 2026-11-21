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
        bundle
            .output("@ui/components/hud/stat-bar.svelte")
            .unwrap()
            .ty,
        "og.UiDocument"
    );
    assert!(bundle.output("@ui/components/hud/stat-bar.en.ogt").is_ok());
    assert!(
        bundle
            .output("@ui/components/hud/ability-bar.svelte")
            .is_err(),
        "only what it uses"
    );
    assert_eq!(
        uses(&root, "ui/vitals.svelte"),
        ["@ui/components/hud/stat-bar.svelte"]
    );
}

/// A built-in screen the code starts to name after the last cook joins the
/// bundle on the next one. Ways it could fail: the cook after a change to
/// code alone rewrites only the asset index, so the screen stays out and
/// og.load.document fails at run time with "not in the cooked bundle".
#[test]
fn a_built_in_screen_newly_named_in_code_cooks_in() {
    let root = project("code-names-screen", &[]);
    std::fs::create_dir_all(root.join("assets")).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("project.ogt"),
        "type project\nversion 1\n---\nname \"p\"\n",
    )
    .unwrap();
    let cook = || {
        let r =
            og_cook::cook::cook_project_with_texture_cache(&root, &CookOptions::default(), None)
                .unwrap();
        assert!(r.ok(), "{}", r.errors().join("\n"));
        Bundle::open(&root.join("cooked")).unwrap()
    };
    std::fs::write(root.join("src/main.ts"), "export default 1;\n").unwrap();
    assert!(cook().output("@ui/screens/pause.svelte").is_err());
    std::fs::write(
        root.join("src/main.ts"),
        "og.load.document(world, \"@ui/screens/pause.svelte\");\n",
    )
    .unwrap();
    let bundle = cook();
    assert_eq!(
        bundle.output("@ui/screens/pause.svelte").unwrap().ty,
        "og.UiDocument"
    );
    assert!(bundle.output("@ui/screens/pause.en.ogt").is_ok());
}

/// A game whose code draws a TSX UI (imports `@origingame/engine/ui`)
/// carries the default face and the default pack's shapes (`nine-disc` …),
/// as a game with a `.svelte` document does, and gains them when its code
/// starts to. Ways it could fail: a TSX-only game cooks without the face, so
/// its tree has no body font and draws no text, or without the shapes, so
/// every `nine-<name>` of the default pack draws nothing; the cook after a
/// change to code alone keeps the old bundle.
#[test]
fn code_drawing_a_tsx_ui_carries_the_default_face() {
    let root = project("code-tsx-face", &[]);
    std::fs::create_dir_all(root.join("assets")).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("project.ogt"),
        "type project\nversion 1\n---\nname \"p\"\n",
    )
    .unwrap();
    let cook = || {
        let r =
            og_cook::cook::cook_project_with_texture_cache(&root, &CookOptions::default(), None)
                .unwrap();
        assert!(r.ok(), "{}", r.errors().join("\n"));
        Bundle::open(&root.join("cooked")).unwrap()
    };
    std::fs::write(root.join("src/main.ts"), "export default 1;\n").unwrap();
    let shapes = ["disc", "ring", "tile", "plate-cut", "panel-cut", "chip-cut"]
        .map(|s| format!("@styles/shared/shapes/{s}.svg"));
    let bundle = cook();
    assert!(
        bundle.output(og_ui::DEFAULT_FONT_ASSET).is_err(),
        "no UI, no face"
    );
    assert!(
        shapes.iter().all(|id| bundle.output(id).is_err()),
        "no UI, no shapes"
    );
    std::fs::write(
        root.join("src/main.ts"),
        "import { mount } from \"@origingame/engine/ui\";\nexport default mount;\n",
    )
    .unwrap();
    let bundle = cook();
    assert!(bundle.output(og_ui::DEFAULT_FONT_ASSET).is_ok());
    for id in &shapes {
        assert!(bundle.output(id).is_ok(), "{id}");
    }
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
    std::fs::write(
        root.join("src/main.ts"),
        "import { mount } from \"@origingame/engine/ui\";\nexport default mount;\n",
    )
    .unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors().join("\n");
    assert!(
        errors.contains("@styles/shared/shapes/disc.svg"),
        "{errors}"
    );
}

#[test]
fn manifest_only_edits_update_a_cached_builtin_documents_dependencies() {
    let root = project(
        "manifest-dependencies",
        &[(
            "ui/page.svelte",
            "<meta name='screen' content='page'><Container/>",
        )],
    );
    let package = root.join("engine");
    let ui = package.join("src/ui");
    std::fs::create_dir_all(ui.join("components")).unwrap();
    for (name, text) in [
        (
            "container",
            "<meta name='component' content='Container'><Spinner/>",
        ),
        (
            "spinner",
            "<meta name='component' content='Spinner'><div>Ready</div>",
        ),
    ] {
        std::fs::write(ui.join(format!("components/{name}.svelte")), text).unwrap();
    }
    let manifest = |include_spinner: bool| {
        std::fs::write(
            ui.join("manifest.json"),
            serde_json::json!({
                "Container": {
                    "id": "@ui/components/container.svelte", "screen": false, "control": null,
                    "uses": if include_spinner { vec!["Spinner"] } else { vec![] },
                    "files": ["@ui/components/container.svelte"], "imports": []
                },
                "Spinner": {
                    "id": "@ui/components/spinner.svelte", "screen": false, "control": null,
                    "uses": [], "files": ["@ui/components/spinner.svelte"], "imports": []
                }
            })
            .to_string(),
        )
        .unwrap();
    };
    // Each invocation reads the package as a new user-facing cook does;
    // neither the documents nor the project's cooked cache are replaced.
    let cook = || {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_og-cook"))
            .arg(&root)
            .env("OG_ENGINE_PACKAGE", &package)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        Bundle::open(&root.join("cooked")).unwrap()
    };
    let container = "@ui/components/container.svelte";
    let spinner = "@ui/components/spinner.svelte";
    manifest(false);
    assert!(cook().output(spinner).is_err());
    assert!(uses(&root, container).is_empty());
    manifest(true);
    assert!(
        cook().output(spinner).is_ok(),
        "manifest-only addition must join the cached bundle"
    );
    assert_eq!(uses(&root, container), [spinner]);
    manifest(false);
    assert!(
        cook().output(spinner).is_err(),
        "manifest-only removal must leave the bundle"
    );
    assert!(uses(&root, container).is_empty());
}

#[test]
fn lowercase_library_components_cook_by_their_declared_names() {
    let source = r#"<meta name="component" content="results-card">
<column>
  <ui:star-rating stars={$stars} label="Rating"/>
  <ui:key-values rows={$rows}/>
  <ui:banner title="Mission complete" subtitle="Stage 1"/>
  <ui:table columns={$columns} cells={$cells} label="Standings"/>
</column>"#;
    let root = project("lowercase-library", &[("ui/results.svelte", source)]);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{}", report.errors().join("\n"));
    assert_eq!(
        uses(&root, "ui/results.svelte"),
        [
            "@ui/screens/banner.svelte",
            "@ui/screens/key-values.svelte",
            "@ui/screens/star-rating.svelte",
            "@ui/screens/table.svelte",
        ]
    );
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    for name in ["banner", "key-values", "star-rating", "table"] {
        assert!(bundle.output(&format!("@ui/screens/{name}.svelte")).is_ok());
    }
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
