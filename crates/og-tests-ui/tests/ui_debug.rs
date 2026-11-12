//! The UI debugger's overlay: the frames og-render draws, against `references/<tier>/`.
//! What og-ui lays out and builds for them is checked in
//! crates/og-ui/tests/ui_debug.rs.

use crate::support;
use og_ui::prelude::*;
use og_ui::reactive::{SourceId, wire};
use og_ui::*;
use serde_json::json;
use support::*;

#[path = "../../og-ui/tests/ui_debug/doc.rs"]
mod doc;
use doc::*;

/// `references/<tier>/ui-debug-<width>.png`: a document with the debugger
/// drawn over it, as the inspector's UI view draws it: every element's
/// layout bounds, and the card's margin, border, padding and content boxes.
#[test]
#[ignore = "requires GPU"]
fn the_debugger_overlay_matches_its_reference_frames() {
    use og_render::RenderScene;
    with_test_font(|files| {
        for (name, w, h) in [
            ("ui-debug-1280", 1280u32, 720u32),
            ("ui-debug-390", 390, 844),
        ] {
            let mut scene = RenderScene::new();
            let font = add_font(&mut scene, files);
            let vp = Viewport::new(w as f32, h as f32);
            let mut game = ui_with_font(font.clone());
            game.define_document(UiDocument::parse("ui/dbg.svelte", DOC).unwrap());
            let mut sources = std::collections::BTreeMap::new();
            let title = SourceId::new("test", "title");
            let set = wire::encode(&json!([{"op": "signal", "source": title, "initial": "Hello"}]));
            game.apply_reactive(&set.unwrap());
            sources.insert("title".to_string(), title);
            game.mount_document(
                SourceId::new("test", "dbg"),
                "dbg",
                sources,
                Default::default(),
                &bind::NoEnv,
            )
            .unwrap();
            for _ in 0..2 {
                game.begin(InputFrame::default(), vp);
                game.build_retained_documents(&bind::NoEnv).unwrap();
                game.end();
            }
            let card = game.debug_element("test:dbg/Card").expect("the card");
            let bounds: Vec<Rect> = game
                .elements()
                .iter()
                .map(|e| Rect {
                    x: e.x,
                    y: e.y,
                    width: e.width,
                    height: e.height,
                })
                .collect();
            let mut over = ui_with_font(font);
            over.begin(InputFrame::default(), vp);
            debug_overlay(&mut over, Some(&card), None, &bounds, 1.0);
            over.end();
            let mut overlay = og_render::Overlay::new(vp.width, vp.height, 1.0);
            game.frame().emit(&EmitDesc::default(), &mut overlay);
            over.frame().emit(
                &EmitDesc {
                    layer: 1,
                    ..EmitDesc::default()
                },
                &mut overlay,
            );
            scene.ui.overlay = Some(overlay);
            on_backdrop(&game, &mut scene);
            let frame = render_frame(&scene, &sized(w, h));
            matches_reference(name, &frame.image, &frame.info, Against::OwnTier);
        }
    });
}
