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
        let frame = render_ui(&rig.ui, &mut scene, width, height);
        assert_eq!(frame.stats.missing_glyphs, 0);
        let name = format!("input-types-{width}");
        if let Err(e) = check_reference(&name, &frame.image, &frame.info, Against::Recorded) {
            failures.push(e);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
