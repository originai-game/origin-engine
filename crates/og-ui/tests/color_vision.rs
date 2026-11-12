//! Colour-blind modes (docs/ui.md §3.2): the player's colour vision filter
//! over the whole frame, the `-og-color-vision` media feature, and the
//! colour tokens' cues drawn with `<og-icon cue>`.
//!
//! Ways these could fail, each covered below:
//! - the media feature parsed but never re-evaluated when the setting
//!   changes in a retained mount (its dirty axis missing), or the boolean
//!   form matching typical vision → `the_media_feature_follows_the_setting`
//! - `cue` reading the game's atlas order instead of the style's icon
//!   names, or a non-icon cue drawn as a blank icon without a warning →
//!   `a_cue_draws_the_tokens_icon_from_the_styles_names`
//! - the GPU pass reading the wrong half-plane, skipping the sRGB decode,
//!   filtering the scene but not the overlay (or the reverse), or leaving
//!   the frame untouched → `the_frame_is_the_cpu_filter_of_the_unfiltered_frame`

use crate::support;
use og_ui::prelude::*;

use og_draw::ColorVision;
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
<row><og-icon id="Danger" cue="danger"/><og-icon id="Warn" cue="warn"/><og-icon id="Own" icon="star" cue="danger"/></row>"#;
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

#[test]
fn icon_layers_keep_independent_colors() {
    for rejected in [
        "<icon icon=\"base\" />",
        "<og-icon secondary-tint=\"#fff\" />",
        "<og-icon /><style>og-icon { -og-secondary-tint: #fff; }</style>",
        "<og-icon /><style>og-icon { -og-cue: danger; }</style>",
    ] {
        let source = format!("<meta name=\"component\" content=\"rejected\">{rejected}");
        assert!(
            UiDocument::parse("ui/rejected.svelte", &source).is_err(),
            "{rejected}"
        );
    }
    let source = r##"<meta name="component" content="layers">
<og-icon id="Mark" icon="base" secondary="accent" />
<style>
og-icon::part(image) { color: #c0392b; }
og-icon::part(secondary) { color: #2980b9; }
</style>"##;
    for mounted in [false, true] {
        let mut handles = Handles::default();
        let texture = handles.texture();
        let mut ui = ui_with_font(test_font_metrics());
        ui.set_icons(IconAtlas {
            texture: Some(texture),
            columns: 2,
            rows: 1,
            names: vec!["base".into(), "accent".into()],
            ..Default::default()
        });
        ui.define_document(UiDocument::parse("ui/layers.svelte", source).unwrap());
        if mounted {
            ui.mount_document(
                SourceId::new("test", "layers"),
                "layers",
                BTreeMap::new(),
                BTreeMap::new(),
                &bind::NoEnv,
            )
            .unwrap();
        }
        ui.begin(InputFrame::default(), Viewport::new(160.0, 100.0));
        if mounted {
            ui.build_retained_documents(&bind::NoEnv).unwrap();
        } else {
            ui.component("T", "layers", &mut json!({}), &bind::NoEnv);
        }
        ui.end();
        let images: Vec<_> = ui
            .frame()
            .items
            .iter()
            .filter_map(|item| match &item.cmd {
                DrawCmd::Image { color, uv, .. } => Some((*color, *uv)),
                _ => None,
            })
            .collect();
        assert_eq!(
            images,
            vec![
                (Srgba::rgb(192, 57, 43).linear(), [0.0, 0.0, 0.5, 1.0]),
                (Srgba::rgb(41, 128, 185).linear(), [0.5, 0.0, 1.0, 1.0]),
            ],
            "mounted={mounted}: each atlas layer keeps its own ink"
        );
    }
}
