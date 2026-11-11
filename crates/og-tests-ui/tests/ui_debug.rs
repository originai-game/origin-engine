//! The UI debugger's overlay: the frames og-render draws, against `references/<tier>/`.
//! What og-ui lays out and builds for them is checked in
//! crates/og-ui/tests/ui_debug.rs.

use crate::support;
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
    use og_render::{Background, RenderDesc, RenderScene, RenderTarget, Renderer, Tonemap};
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
            let bg = game.theme().palette.backdrop.linear();
            scene.lighting.environment.background =
                Background::Color(og_core::glam::Vec3::new(bg[0], bg[1], bg[2]));
            scene.look.post.tonemap = Tonemap::None;
            let desc = RenderDesc {
                width: w,
                height: h,
                ..Default::default()
            };
            let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
            renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
            let image = renderer.read_offscreen().unwrap();
            let tier = renderer.info().backend_tier();
            let actual = out_dir().join(format!("{name}-{tier}.png"));
            image::save_buffer(
                &actual,
                &image.pixels,
                image.width,
                image.height,
                image::ColorType::Rgba8,
            )
            .unwrap();
            let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("references");
            let own = dir.join(&tier).join(format!("{name}.png"));
            if std::env::var_os("OG_UI_CAPTURE").is_some() {
                record_reference(&own, image.width, image.height, &image.pixels);
            }
            // A tier without its own frame is judged against the recording
            // tier's under the cross-platform tolerance (`compare`).
            let reference = if own.exists() {
                own
            } else {
                dir.join(og_tests::compare::RECORDING_TIER)
                    .join(format!("{name}.png"))
            };
            let expected = image::open(&reference)
                .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1)", reference.display()))
                .to_rgba8();
            match compare(
                &image.pixels,
                expected.as_raw(),
                renderer.info(),
                &reference,
            ) {
                Ok((rmse, _)) => println!(
                    "{name} {tier}: RMSE {rmse:.5} against {}",
                    reference.display()
                ),
                Err(e) => panic!(
                    "{} differs from {}: {e}",
                    actual.display(),
                    reference.display()
                ),
            }
        }
    });
}
