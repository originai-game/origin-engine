//! The HTML controls: the frames og-render draws, against `references/<tier>/`.
//! What og-ui lays out and builds for them is checked in
//! crates/og-ui/tests/html_controls.rs.

use crate::support;
use og_core::glam::Vec2;
use og_ui::reactive::SourceId;
use og_ui::*;
use serde_json::json;
use support::*;

// The frame drives part of the rig the logic tests drive whole.
#[allow(dead_code)]
#[path = "../../og-ui/tests/html_controls/rig.rs"]
mod rig;
use rig::*;

/// `references/vulkan/html-controls-{1280,390}.png`: the form with the
/// select, last, open below it (its groups, the disabled Orange, the tick
/// on Blue), the bound disclosure open, the meter in its poor region and
/// the dialog in place; `html-controls-modal-{1280,390}.png`: the modal
/// over its scrim (the focus moved in by program, so no ring). Ways it could fail: a
/// list drawn under the page or off its trigger, a group heading missing,
/// a closed disclosure's content showing, a meter in the wrong colour, a
/// modal with no backdrop or its panel off centre.
#[test]
#[ignore = "requires GPU"]
fn html_controls_match_their_reference_frames() {
    use og_render::*;
    for modal in [false, true] {
        for (width, height) in [(1280u32, 720u32), (390, 844)] {
            let mut scene = RenderScene::new();
            let font = add_test_font(&mut scene);
            let mut rig = Rig::new(ui_with_font(font), &STORES);
            let vp = Viewport::new(width as f32, height as f32);
            rig.step_at(InputFrame::default(), vp);
            for (name, value) in [
                ("choice", json!("b")),
                ("hp", json!(20)),
                ("more", json!(true)),
                ("note", json!(true)),
            ] {
                rig.apply(
                    json!({"op": "set", "source": SourceId::new("test", name), "value": value}),
                );
            }
            rig.step_at(InputFrame::default(), vp);
            if modal {
                rig.apply(
                    json!({"op": "set", "source": SourceId::new("test", "ask"), "value": true}),
                );
            } else {
                let e = rig.el("Pick/Trigger").unwrap();
                let at = Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0);
                for (pressed, released) in [(true, false), (false, true)] {
                    rig.step_at(
                        InputFrame {
                            pointer: at,
                            pointer_inside: true,
                            down: pressed,
                            pressed,
                            released,
                            ..InputFrame::default()
                        },
                        vp,
                    );
                }
            }
            for _ in 0..40 {
                rig.step_at(InputFrame::default(), vp);
            }
            let mut overlay = Overlay::new(width as f32, height as f32, 1.0);
            rig.ui.frame().emit(&EmitDesc::default(), &mut overlay);
            scene.ui.overlay = Some(overlay);
            let bg = rig.ui.theme().palette.backdrop.linear();
            scene.lighting.environment.background =
                Background::Color(og_core::glam::Vec3::new(bg[0], bg[1], bg[2]));
            scene.look.post.tonemap = Tonemap::None;
            let mut renderer = Renderer::new_blocking(&RenderDesc {
                width,
                height,
                ..Default::default()
            })
            .expect("a headless adapter");
            let stats = renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
            assert_eq!(stats.missing_glyphs, 0);
            let image = renderer.read_offscreen().unwrap();
            let name = format!(
                "html-controls{}-{width}.png",
                if modal { "-modal" } else { "" }
            );
            image::save_buffer(
                out_dir().join(&name),
                &image.pixels,
                width,
                height,
                image::ColorType::Rgba8,
            )
            .unwrap();
            let own = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("references")
                .join(og_tests::compare::RECORDING_TIER)
                .join(&name);
            if std::env::var_os("OG_UI_CAPTURE").is_some() {
                record_reference(&own, width, height, &image.pixels);
            }
            let expected = image::open(&own)
                .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1 records it)", own.display()))
                .to_rgba8();
            let (rmse, _) = compare(&image.pixels, expected.as_raw(), renderer.info(), &own)
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            println!("{name}: RMSE {rmse:.6}");
        }
    }
}
