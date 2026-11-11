//! The input types: the frames og-render draws, against `references/<tier>/`.
//! What og-ui lays out and builds for them is checked in
//! crates/og-ui/tests/input_types.rs.

use crate::support;
use og_ui::reactive::SourceId;
use og_ui::*;
use serde_json::json;
use support::*;

// The frame drives part of the rig the logic tests drive whole.
#[allow(dead_code)]
#[path = "../../og-ui/tests/input_types/rig.rs"]
mod rig;
use rig::*;

/// `references/vulkan/input-types-{1280,390}.png`: the form with a masked
/// password, its reveal eye, an invalid number and a valid email, the
/// password focused. Ways it could fail: the mask drawn as text, the eye
/// missing or over the text, the invalid frame not dressed.
#[test]
#[ignore = "requires GPU"]
fn input_types_match_their_reference_frames() {
    use og_render::*;
    let mut failures = Vec::new();
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let mut scene = RenderScene::new();
        let font = add_test_font(&mut scene);
        let mut ui = ui_with_font(font);
        ui.define_document(UiDocument::parse("ui/form.svelte", FORM).unwrap());
        let mut rig = Rig { ui };
        let mut inputs = std::collections::BTreeMap::new();
        for (name, value) in STORES.into_iter().zip(["", "12.5e", "ada@example.org", ""]) {
            let source = SourceId::new("test", name);
            rig.apply(json!({"op": "signal", "source": source, "initial": value}));
            inputs.insert(name.to_string(), source);
        }
        rig.ui
            .mount_document(
                SourceId::new("test", "form"),
                "form",
                inputs,
                Default::default(),
                &bind::NoEnv,
            )
            .unwrap();
        rig.step(InputFrame::default());
        rig.focus("Pin");
        rig.typed("hunter2");
        for _ in 0..30 {
            rig.step(InputFrame::default());
        }
        let vp = Viewport::new(width as f32, height as f32);
        rig.ui.begin(InputFrame::default(), vp);
        rig.ui.build_retained_documents(&bind::NoEnv).unwrap();
        rig.ui.end();
        assert_eq!(
            rig.el("Age/Increment").width,
            if width == 390 { 44.0 } else { 24.0 }
        );
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
        let name = format!("input-types-{width}.png");
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
        match compare(&image.pixels, expected.as_raw(), renderer.info(), &own) {
            Ok((rmse, _)) => println!("{name}: RMSE {rmse:.6}"),
            Err(error) => failures.push(format!("{name}: {error}")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
