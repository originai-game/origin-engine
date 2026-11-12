//! Every engine UI document (`packages/engine`: the components, their groups
//! included, and the screens) built from its `fixture:default` at
//! 1280×720 and compared with `references/vulkan/doc-<path>.png`
//! (og-tests' tolerance; `OG_UI_CAPTURE=1` records). What only drawing
//! shows: how the default style dresses each document's own CSS, such as
//! a button's own box restyling its plate.
//!
//! The documents in [`PLACEHOLDERS`] draw nothing from their fixture alone,
//! so they are skipped by name until a preview gives them their context: a
//! black frame would only record that they draw nothing.

use crate::support;

use og_render::RenderScene;
use support::*;

/// Documents (`doc-<path>`) with no reference yet, and why their
/// `fixture:default` alone draws nothing.
const PLACEHOLDERS: &[(&str, &str)] = &[
    ("doc-components-damage-numbers", WORLD),
    ("doc-components-interaction-prompt", WORLD),
    ("doc-components-overhead-bars", WORLD),
    ("doc-screens-calendar", VIEW_MODEL),
    ("doc-screens-confirm", VIEW_MODEL),
    ("doc-screens-data-list", VIEW_MODEL),
    ("doc-screens-key-values", VIEW_MODEL),
    ("doc-screens-objective-tracker", VIEW_MODEL),
    ("doc-screens-pagination", VIEW_MODEL),
    ("doc-screens-star-rating", VIEW_MODEL),
    ("doc-screens-tree", VIEW_MODEL),
    ("doc-screens-hud-editor", SCREEN),
    ("doc-screens-notifications", SCREEN),
    ("doc-screens-splash", SCREEN),
    ("doc-screens-subtitles", SCREEN),
];
const WORLD: &str = "world-anchored: it draws at entities the preview's world does not have";
const VIEW_MODEL: &str = "a component that reads its data as `$name` from a view model, which a component's props do not set";
const SCREEN: &str = "a screen that draws nothing from its fixture, pushed or built as a component";

#[test]
#[ignore = "requires GPU"]
fn engine_documents_match_their_reference_frames() {
    let (docs, messages) = engine_documents(&["src/ui/components", "src/ui/screens"]);
    let (width, height) = (1280, 720);
    let mut failures = Vec::new();
    let name_of = |path: &str| {
        // `src/ui/components/rpg/party.svelte` is `doc-components-rpg-party`.
        format!(
            "doc-{}",
            path.trim_start_matches("src/ui/")
                .trim_end_matches(".svelte")
                .replace('/', "-")
        )
    };
    for (name, _) in PLACEHOLDERS {
        assert!(
            docs.iter().any(|d| name_of(&d.path) == *name),
            "{name} is a placeholder for a document that no longer exists"
        );
    }
    for doc in &docs {
        let name = name_of(&doc.path);
        if let Some((_, why)) = PLACEHOLDERS.iter().find(|(n, _)| *n == name) {
            eprintln!("{name}: no reference yet: {why}");
            continue;
        }
        let mut scene = RenderScene::new();
        let mut ui = ui_with_font(add_test_font(&mut scene));
        preview(&mut ui, &docs, doc, &messages, width as f32, height as f32);
        let rendered = render_ui(&ui, &mut scene, width, height);
        if let Err(e) = check_reference(&name, &rendered.image, &rendered.info, Against::Recorded) {
            failures.push(e);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
