//! The engine's component documents (`packages/engine/src/ui/components`):
//! each parses, takes its strings and builds from its `fixture:default`
//! props at a desktop and a phone width with no author or frame warning.
//! A document that stops cooking, names a missing string, or builds a
//! tree the frame refuses fails here before a game ejects it.

use crate::support;
use og_ui::*;
use support::*;

#[test]
fn component_documents_build_from_their_fixtures() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    assert!(!docs.is_empty());
    let mut failures = Vec::new();
    for doc in &docs {
        for (width, height) in [(1280.0, 720.0), (390.0, 844.0)] {
            let mut ui = ui_with_font(test_font_metrics());
            ui.set_icons(icons(og_draw::TextureHandle::from_slot(1)));
            preview(&mut ui, &docs, doc, &messages, width, height);
            let frame = ui.frame();
            if !frame.warnings.is_empty() {
                failures.push(format!("{} at {width}: {:?}", doc.path, frame.warnings));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The ground marks drape over the scene (c8e4b7a8c): each document's
/// `<ground>` reaches the renderer as one layer carrying its drape. Fails
/// when a mark is rebuilt as a flat panel or a screen-space box.
#[test]
fn ground_mark_documents_reach_the_renderer_as_draped_grounds() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    for name in ["SelectionRing", "Telegraph", "PathLine", "PlacementGhost"] {
        let doc = docs.iter().find(|d| d.doc.name == name).unwrap();
        let (width, height) = (1280.0, 720.0);
        let mut ui = ui_with_font(test_font_metrics());
        preview(
            &mut ui,
            std::slice::from_ref(doc),
            doc,
            &messages,
            width,
            height,
        );
        let mut overlay = og_draw::Overlay::new(width, height, 1.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        let grounds = overlay
            .ui
            .layers
            .iter()
            .filter(|l| l.ground.is_some())
            .count();
        assert_eq!(grounds, 1, "{name}: {grounds} draped ground layer(s)");
    }
}
