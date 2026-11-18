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
use support::documents::populated_preview;
use support::*;

/// Documents (`doc-<path>`) with no reference yet, and why their
/// `fixture:default` alone draws nothing.
const PLACEHOLDERS: &[(&str, &str)] = &[
    ("doc-components-hud-damage-numbers", WORLD),
    ("doc-components-world-interaction-prompt", WORLD),
    ("doc-components-world-overhead-bars", WORLD),
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

/// The preview's panel ground belongs below the scene, not a screen
/// element: anchored UI must paint above it and below screen UI.
fn render_document(
    ui: &og_ui::Ui,
    scene: &mut RenderScene,
    width: u32,
    height: u32,
    picture: Option<og_draw::TextureHandle>,
) -> Rendered {
    let mut overlay = og_render::Overlay::new(width as f32, height as f32, 1.0);
    let mut ground = og_draw::UiShape::rect(
        [0.0, 0.0, width as f32, height as f32],
        0.0,
        ui.theme().palette.panel.linear(),
    );
    ground.layer = -og_ui::LAYER_BASE;
    if let Some(texture) = picture {
        ground.paint = og_draw::UiPaint::Image(og_draw::UiImage {
            texture,
            uv: [0.0, 0.0, 1.0, 1.0],
            size: og_core::glam::Vec2::new(960.0, 540.0),
            density: 1.0,
            tint: [1.0; 4],
            fit: og_draw::UiFit::Cover,
            focal: og_core::glam::Vec2::splat(0.5),
            offset: og_core::glam::Vec2::ZERO,
            luminance: false,
            nearest: false,
            orient: true,
        });
    }
    overlay.ui.shapes.push(ground);
    ui.frame().emit(&og_ui::EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    on_backdrop(ui, scene);
    render_frame(scene, &sized(width, height))
}

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
        let picture = populated_preview(
            &mut ui,
            &mut scene,
            &docs,
            doc,
            &messages,
            width as f32,
            height as f32,
        );
        let rendered = render_document(&ui, &mut scene, width, height, picture);
        if let Err(e) = check_reference(&name, &rendered.image, &rendered.info, Against::Recorded) {
            failures.push(e);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
