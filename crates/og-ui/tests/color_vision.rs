//! Colour-blind modes (docs/ui.md §3.2): the player's colour vision filter
//! over the whole frame, the `-og-color-vision` media feature, and the
//! colour tokens' cues drawn with `-og-cue`.
//!
//! Ways these could fail, each covered below:
//! - the media feature parsed but never re-evaluated when the setting
//!   changes in a retained mount (its dirty axis missing), or the boolean
//!   form matching typical vision → `the_media_feature_follows_the_setting`
//! - `-og-cue` reading the game's atlas order instead of the style's icon
//!   names, or a non-icon cue drawn as a blank icon without a warning →
//!   `a_cue_draws_the_tokens_icon_from_the_styles_names`
//! - the GPU pass reading the wrong half-plane, skipping the sRGB decode,
//!   filtering the scene but not the overlay (or the reverse), or leaving
//!   the frame untouched → `the_frame_is_the_cpu_filter_of_the_unfiltered_frame`

use crate::support;

use og_render::ColorVision;
use og_ui::reactive::SourceId;
use og_ui::*;
use serde_json::json;
use std::collections::BTreeMap;
use support::*;

const MEDIA: &str = r#"<meta name="component" content="cv">
<column id="Root"><panel id="Any"/><panel id="Tri"/><panel id="Typical"/></column>
<style>
panel { width: 10px; height: 10px; }
@media (-og-color-vision) { #Any { width: 20px; } }
@media (-og-color-vision: tritanopia) { #Tri { width: 30px; } }
@media (-og-color-vision: none) { #Typical { width: 40px; } }
</style>"#;

fn widths(ui: &Ui) -> [f32; 3] {
    let els = ui.probe_elements();
    ["Any", "Tri", "Typical"].map(|id| els.iter().find(|e| e.id == id).unwrap().width)
}

#[test]
fn the_media_feature_follows_the_setting() {
    for mounted in [false, true] {
        let mut ui = ui_with_font(test_font_metrics());
        ui.define_document(UiDocument::parse("ui/cv.svelte", MEDIA).unwrap());
        if mounted {
            ui.mount_document(
                SourceId::new("test", "cv"),
                "cv",
                BTreeMap::new(),
                &bind::NoEnv,
            )
            .unwrap();
        }
        for (vision, expected) in [
            (ColorVision::Typical, [10.0, 10.0, 40.0]),
            (ColorVision::Deuteranopia, [20.0, 10.0, 10.0]),
            (ColorVision::Tritanopia, [20.0, 30.0, 10.0]),
            (ColorVision::Typical, [10.0, 10.0, 40.0]),
        ] {
            let mut theme = *ui.theme();
            theme.unit_pin = 1.0;
            theme.color_vision = vision;
            ui.set_theme(theme);
            for _ in 0..2 {
                ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
                if mounted {
                    ui.build_retained_documents(&bind::NoEnv).unwrap();
                } else {
                    ui.component("T", "cv", &mut json!({}), &bind::NoEnv);
                }
                ui.end();
            }
            assert_eq!(widths(&ui), expected, "{vision:?}, mounted {mounted}");
        }
    }
    let err = UiDocument::parse(
        "ui/bad.svelte",
        "<meta name='component' content='bad'/><panel/>\n<style>\n@media (-og-color-vision: achromatopsia) { panel { width: 1px; } }\n</style>",
    )
    .unwrap_err()
    .join("\n");
    assert!(
        err.contains("-og-color-vision value achromatopsia"),
        "{err}"
    );
}

#[test]
fn a_cue_draws_the_tokens_icon_from_the_styles_names() {
    use og_ui::tokens::{Cue, Pattern, Status};
    let doc = r#"<meta name="component" content="cues">
<row><icon id="Danger"/><icon id="Warn"/><icon id="Own" icon="star"/></row>
<style>
#Danger { -og-cue: danger; }
#Warn { -og-cue: warn; }
#Own { -og-cue: danger; }
</style>"#;
    let mut ui = ui_with_font(test_font_metrics());
    let mut theme = *ui.theme();
    theme.tokens.colors.status[Status::Danger.index()] = Cue::Icon(1);
    theme.tokens.colors.status[Status::Warn.index()] = Cue::Pattern(Pattern::Stripes);
    ui.set_theme(theme);
    // The game's atlas lists the names in another order than the style.
    ui.set_icons(IconAtlas {
        columns: 4,
        rows: 1,
        names: ["alert", "check", "star", "info"]
            .map(String::from)
            .to_vec(),
        ..IconAtlas::default()
    });
    ui.set_cue_icons(["check", "alert"].map(String::from).to_vec());
    ui.define_document(UiDocument::parse("ui/cues.svelte", doc).unwrap());
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.component("T", "cues", &mut json!({}), &bind::NoEnv);
    ui.end();
    let els = ui.probe_elements();
    let text = |id: &str| els.iter().find(|e| e.id == id).unwrap().text.clone();
    assert_eq!(text("Danger"), "alert");
    assert_eq!(text("Own"), "star", "an icon attribute wins over the cue");
    let warnings = ui.frame().warnings.join("\n");
    assert!(warnings.contains("warn's cue is Pattern"), "{warnings}");
}

/// The filter on the GPU is the CPU's ([`ColorFilter::apply`]) on every
/// pixel, the overlay's and the scene's alike: one frame rendered with no
/// filter, then with each mode, compared pixel for pixel.
#[test]
#[ignore = "requires GPU"]
fn the_frame_is_the_cpu_filter_of_the_unfiltered_frame() {
    use og_render::*;
    let doc = r#"<meta name="component" content="chips">
<row id="Root"><panel id="R"/><panel id="G"/><panel id="B"/><panel id="Y"/><panel id="M"/></row>
<style>
#Root { padding: 24px; gap: 12px; flex-wrap: wrap; }
panel { width: 120px; height: 120px; }
#R { background: #d03028; } #G { background: #2fa84f; } #B { background: #2f6fe0; }
#Y { background: #e8c83a; } #M { background: #a03ad0; }
</style>"#;
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let mut scene = RenderScene::new();
        let font = add_test_font(&mut scene);
        let mut ui = ui_with_font(font);
        ui.define_document(UiDocument::parse("ui/chips.svelte", doc).unwrap());
        for _ in 0..2 {
            ui.begin(
                InputFrame::default(),
                Viewport::new(width as f32, height as f32),
            );
            ui.component("T", "chips", &mut json!({}), &bind::NoEnv);
            ui.end();
        }
        let mut overlay = Overlay::new(width as f32, height as f32, 1.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        scene.overlay = Some(overlay);
        // A scene colour under the UI: the filter covers it too.
        scene.environment.background = Background::Color(og_core::glam::Vec3::new(0.6, 0.15, 0.05));
        scene.post.tonemap = Tonemap::None;
        let mut renderer = Renderer::new_blocking(&RenderDesc {
            width,
            height,
            ..Default::default()
        })
        .expect("a headless adapter");
        let mut frame = |scene: &RenderScene| {
            renderer.render(scene, &RenderTarget::Offscreen).unwrap();
            renderer.read_offscreen().unwrap().pixels
        };
        let plain = frame(&scene);
        let decode = |v: u8| {
            let c = f32::from(v) / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let encode = |v: f32| {
            let c = v.clamp(0.0, 1.0);
            let e = if c <= 0.003_130_8 {
                c * 12.92
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            };
            (e * 255.0).round() as u8
        };
        for vision in [
            ColorVision::Protanopia,
            ColorVision::Deuteranopia,
            ColorVision::Tritanopia,
        ] {
            for filter in [ColorVisionFilter::Correct, ColorVisionFilter::Simulate] {
                let f = ColorFilter::new(vision, filter, 1.0).unwrap();
                scene.color_filter = Some(f);
                let got = frame(&scene);
                let mut worst = 0u8;
                let mut moved = 0usize;
                for (p, g) in plain.chunks(4).zip(got.chunks(4)) {
                    let o = f.apply([decode(p[0]), decode(p[1]), decode(p[2])]);
                    for k in 0..3 {
                        worst = worst.max(encode(o[k]).abs_diff(g[k]));
                    }
                    moved += usize::from(p[..3] != g[..3]);
                }
                assert!(
                    worst <= 2,
                    "{vision:?} {filter:?}: a pixel {worst}/255 off the CPU filter"
                );
                assert!(
                    moved > plain.len() / 8,
                    "{vision:?} {filter:?}: only {moved} pixels changed"
                );
                if filter == ColorVisionFilter::Correct {
                    image::save_buffer(
                        out_dir().join(format!("color-vision-{}-{width}.png", vision.name())),
                        &got,
                        width,
                        height,
                        image::ColorType::Rgba8,
                    )
                    .unwrap();
                }
            }
        }
        scene.color_filter = None;
        assert_eq!(frame(&scene), plain, "no filter leaves the frame as it was");
    }
}
