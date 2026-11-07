//! The default style is the design language's: `src/theme.rs` is exactly
//! what the generator writes from `packages/design/tokens.json` (a hand
//! edit, a stale file or a token changed without regenerating fail here),
//! its glass tints are og-render's presets, and its dressings parse and put
//! glass on floating chrome only.

#[path = "../scripts/theme_codegen.rs"]
mod theme_codegen;

use og_ui::{Selector, Srgba};
use std::path::PathBuf;

fn tokens() -> serde_json::Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(
        &std::fs::read_to_string(root.join("../../packages/design/tokens.json"))
            .expect("tokens.json"),
    )
    .expect("tokens.json parses")
}

#[test]
fn theme_rs_is_generated_from_the_design_tokens() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tokens = std::fs::read_to_string(root.join("../../packages/design/tokens.json"))
        .expect("tokens.json");
    let committed = std::fs::read_to_string(root.join("src/theme.rs")).expect("theme.rs");
    let generated = theme_codegen::generate(&tokens).expect("the tokens generate");
    if committed != generated {
        let line = committed
            .lines()
            .zip(generated.lines())
            .position(|(a, b)| a != b)
            .unwrap_or(committed.lines().count().min(generated.lines().count()));
        panic!(
            "src/theme.rs differs from the generator's output at line {}:\n  committed: {:?}\n  generated: {:?}\nrun crates/og-ui/scripts/gen-theme.sh (never edit theme.rs by hand)",
            line + 1,
            committed.lines().nth(line),
            generated.lines().nth(line)
        );
    }
    // The theme the toolkit starts from is the generated one.
    assert_eq!(og_ui::Theme::dark().palette, og_ui::theme::DARK);
    assert_eq!(og_ui::Theme::light().palette, og_ui::theme::LIGHT);
    assert_eq!(og_ui::Theme::dark().radius, og_ui::theme::RADIUS);
}

/// The tokens' material tints are the ones og-render's glass draws: a
/// tint changed on one side only would hold the language's contrast pairs
/// in the test and not on the screen.
#[test]
fn the_tokens_glass_tints_are_og_renders_presets() {
    use og_render::{UiGlassStyle, UiGlassWeight};
    let t = tokens();
    for (dark, scheme) in [(false, &t["color"]), (true, &t["dark"]["color"])] {
        for (weight, key) in [
            (UiGlassWeight::Thin, "material-thin"),
            (UiGlassWeight::Regular, "material-regular"),
            (UiGlassWeight::Thick, "material-thick"),
        ] {
            let s = scheme[key]
                .as_str()
                .or(t["color"][key].as_str())
                .expect(key);
            let inner = s.trim_start_matches("rgb(").trim_end_matches(')');
            let (rgb, a) = inner.split_once('/').expect("an alpha");
            let c: Vec<u8> = rgb
                .split_whitespace()
                .map(|p| p.parse().expect("a channel"))
                .collect();
            let want = Srgba {
                rgb: [c[0], c[1], c[2]],
                alpha: a.trim().parse().expect("an alpha"),
            }
            .linear();
            let (_, got) = UiGlassStyle {
                weight,
                dark,
                ..UiGlassStyle::default()
            }
            .backdrop();
            for k in 0..4 {
                assert!(
                    (want[k] - got[k]).abs() < 1e-4,
                    "{key} ({}): tokens {want:?}, og-render {got:?}",
                    if dark { "dark" } else { "light" }
                );
            }
        }
    }
}

/// Floating chrome is glass and nothing else is: dialogs, sheets, menus,
/// popovers and tooltips thick, HUD capsules regular; cards and controls
/// carry no backdrop, and controls no border.
#[test]
fn the_default_dressings_put_glass_on_floating_chrome_only() {
    use og_ui::paint::Effect;
    let d = og_ui::default_dressings();
    let glass = |sel: &str| -> Option<og_render::UiGlassWeight> {
        let s = Selector::parse(sel).expect("a selector");
        d.iter().find(|x| x.selector == s).and_then(|x| {
            x.draw.effects.iter().find_map(|e| match e {
                Effect::Backdrop { glass, .. } => glass.weight,
                _ => None,
            })
        })
    };
    use og_render::UiGlassWeight::{Regular, Thick};
    for s in [
        "Panel.frame",
        "Sheet.frame",
        "Menu.frame",
        "Popover.frame",
        "Tooltip.frame",
    ] {
        assert_eq!(glass(s), Some(Thick), "{s}");
    }
    assert_eq!(glass("Hud.frame"), Some(Regular));
    for s in ["Card.frame", "Button.base", "Field.frame", "Slot.frame"] {
        assert_eq!(glass(s), None, "{s}");
    }
    for s in [
        "Button.base",
        "Field.frame",
        "Chip.base",
        "Slot.frame",
        "Toggle.row",
        "Checkbox.row",
        "Radio.option",
        "Radio.card",
    ] {
        let x = d
            .iter()
            .find(|x| x.selector == Selector::parse(s).expect("a selector"))
            .expect(s);
        assert!(
            x.draw.strokes.is_empty() && x.draw.cleared & og_ui::dress::CLEAR_STROKES != 0,
            "{s} is borderless"
        );
    }
    // A row's lit top edge (the tokens' sheen on a button) read as an outline.
    for s in ["Toggle.row", "Checkbox.row", "Radio.option"] {
        let x = d
            .iter()
            .find(|x| x.selector == Selector::parse(s).expect("a selector"))
            .expect(s);
        assert!(
            x.draw.effects.is_empty() && x.draw.cleared & og_ui::dress::CLEAR_EFFECTS != 0,
            "{s} has no lit edge"
        );
    }
}

/// The og-ui unit (docs/ui.md §6.1) zooms the whole UI: a 1080p window
/// lays out on 1280×720 units and draws them at 1.5 CSS px, so a fixed
/// 300 px box a game writes is 450 CSS px wide, body text draws at 21 px
/// and the caption step at 19.5, a pointer at the CSS point lands on the
/// box and the probe reports CSS px. 720p and touch keep CSS px, a pin
/// wins, and the text scale multiplies over the unit.
#[test]
fn the_unit_zooms_the_whole_ui_on_a_mouse_surface() {
    use crate::support::{add_test_font, ui_with_font};
    use og_ui::{ButtonDesc, DrawCmd, InputFrame, Size, Theme, Viewport};
    let frame = |vp: Viewport, configure: &dyn Fn(&mut Theme), input: InputFrame| {
        let mut ui = ui_with_font(add_test_font(&mut og_render::RenderScene::new()));
        let mut t = *ui.theme();
        configure(&mut t);
        ui.set_theme(t);
        let mut hovered = false;
        for _ in 0..2 {
            ui.begin(input.clone(), vp);
            hovered = ui
                .button(
                    "Go",
                    "Go",
                    ButtonDesc {
                        width: Some(Size::Fixed(300.0)),
                        ..ButtonDesc::default()
                    },
                )
                .hovered;
            ui.end();
        }
        let text = ui
            .frame()
            .items
            .iter()
            .find_map(|i| {
                let cmd = match &i.cmd {
                    DrawCmd::StyledText { text, .. } => &**text,
                    c => c,
                };
                match cmd {
                    DrawCmd::Text { size, bounds, .. } => Some((*size, bounds.x)),
                    _ => None,
                }
            })
            .expect("the label is drawn");
        (ui, hovered, text)
    };
    let body = Theme::dark().type_ramp.body.size;
    let hd = Viewport::new(1920.0, 1080.0);
    let (ui, _, (size, _)) = frame(hd, &|_| {}, InputFrame::default());
    let (f, t) = (ui.frame(), *ui.theme());
    assert_eq!(t.unit, 1.5);
    assert_eq!(
        (f.viewport.width, f.viewport.height, f.viewport.dpr),
        (1280.0, 720.0, 1.5)
    );
    assert_eq!(
        t.type_ramp.body.size, body,
        "lengths stay the style's, in units"
    );
    assert_eq!(f.element("Go").expect("the button").width, 300.0);
    assert_eq!(size, body * 1.5, "drawn at 1.5 CSS px a unit");
    assert_eq!(
        t.type_ramp.xs.size * 1.5,
        19.5,
        "xs aliases the 13 px caption, scaled to 1080p"
    );
    let css = ui.probe_elements();
    let go = css
        .iter()
        .find(|e| e.path == "Go")
        .expect("the probe's button");
    assert_eq!(go.width, 450.0, "the probe reports CSS px");
    // A pointer at the button's CSS centre is over it; one past its CSS
    // right edge (inside its width in units) is not.
    let (cx, cy) = go.center();
    assert!(frame(hd, &|_| {}, InputFrame::hover(cx, cy)).1);
    assert!(!frame(hd, &|_| {}, InputFrame::hover(go.x + 460.0, cy)).1);
    // An ultrawide 1080p is still 1.5 (its height), a 16:10 window by its
    // 16:9 height, 720p and below 1, 4K at most 2.
    let unit = |vp: Viewport| frame(vp, &|_| {}, InputFrame::default()).0.theme().unit;
    assert_eq!(unit(Viewport::new(2560.0, 1080.0)), 1.5);
    assert_eq!(unit(Viewport::new(1440.0, 900.0)), 1.125);
    assert_eq!(unit(Viewport::new(1280.0, 720.0)), 1.0);
    assert_eq!(unit(Viewport::new(3840.0, 2160.0)), 2.0);
    // Touch keeps CSS px; a pin wins.
    let touch = Viewport {
        coarse_pointer: true,
        ..hd
    };
    assert_eq!(frame(touch, &|_| {}, InputFrame::default()).2.0, body);
    assert_eq!(
        frame(hd, &|t| t.unit_pin = 1.0, InputFrame::default()).2.0,
        body
    );
    // The text scale multiplies over the unit.
    assert_eq!(
        frame(hd, &|t| t.text_scale = 1.5, InputFrame::default())
            .2
            .0,
        body * 2.25
    );
}

/// A zoomed frame's shapes inside a scroll view are clipped where the view
/// stands on the screen: every clip is the 1× frame's times the unit, from
/// the origin (a clip scaled about its own centre cut the rows' fills).
#[test]
fn a_zoomed_scroll_view_clips_where_it_stands() {
    use crate::support::{add_test_font, ui_with_font};
    use og_ui::{ButtonDesc, EmitDesc, InputFrame, ScrollDesc, Size, Viewport};
    let clips = |vp: Viewport| {
        let mut ui = ui_with_font(add_test_font(&mut og_render::RenderScene::new()));
        for _ in 0..2 {
            ui.begin(InputFrame::default(), vp);
            ui.scroll(
                "S",
                ScrollDesc {
                    width: Size::Fixed(300.0),
                    height: Size::Fixed(200.0),
                    ..ScrollDesc::default()
                },
                |ui| {
                    for i in 0..8 {
                        ui.button(format!("B{i}"), "Row", ButtonDesc::default());
                    }
                },
            );
            ui.end();
        }
        let mut overlay = og_render::Overlay::new(vp.width, vp.height, 0.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        overlay
            .ui
            .shapes
            .iter()
            .map(|s| overlay.clips[s.clip.expect("clipped") as usize])
            .collect::<Vec<_>>()
    };
    let one = clips(Viewport::new(1280.0, 720.0));
    let zoomed = clips(Viewport::new(1920.0, 1080.0));
    assert!(!one.is_empty());
    let want: Vec<[f32; 4]> = one.iter().map(|c| c.map(|v| v * 1.5)).collect();
    assert_eq!(zoomed, want);
}

/// A scrim made stronger stays a scrim: the dark scrim reads CSS 0.42 over
/// mid grey (linear 0.687), the spotlight's 1.6 times it reads CSS 0.67,
/// linear about 0.89, and high contrast's 1.5 times stays see-through.
/// Scaling the linear alpha itself drew both opaque black.
#[test]
fn a_deeper_scrim_reads_deeper_and_stays_translucent() {
    let scrim = og_ui::Theme::dark().palette.scrim;
    assert!((scrim.alpha - 0.687).abs() < 0.01, "{scrim:?}");
    // CSS 0.42 × 1.6 = 0.672 over #808080: 128 × 0.328 = 42 → linear
    // 0.0232 against the grey's 0.2158, so 1 − 0.0232 / 0.2158 = 0.893.
    let spot = scrim.deepen(1.6).alpha;
    assert!((spot - 0.893).abs() < 0.01, "{spot}");
    assert!((scrim.deepen(1.0).alpha - scrim.alpha).abs() < 0.005);
    let high = og_ui::Theme::dark().to_high_contrast().palette.scrim.alpha;
    assert!(high > scrim.alpha && high < 0.95, "{high}");
}
