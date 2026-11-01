//! A style dresses widgets' parts (docs/ui.md §12.3), plays its feedback
//! sets (§7.8) and moves every state on its motion tokens.
//!
//! Ways these could fail, each covered below:
//! - the dressing written first wins over a more specific one written
//!   after it, or a less specific one overrides what a more specific one
//!   set (`Button.base` repainting `Button.base[primary][pressed]`);
//! - a game's attribute never reaches the part (`Slot.frame[rarity=epic]`
//!   dresses no cell, or every cell);
//! - a part the style does not dress draws as a bare flat quad (no line,
//!   no elevation) instead of from the tokens;
//! - a hover jumps to its colour in one tick instead of moving on the
//!   style's `ui` spring;
//! - a set of sounds repeats one sound twice in a row, ignores its volume
//!   or pitch range, differs after a restore, or the player's haptics
//!   setting at 0 still plays the pattern.

use crate::support::*;
use og_ui::dress::PartRef;
use og_ui::tokens::{ElementClass, Feedback, Haptic, HapticPreset, SoundEvent};
use og_ui::*;

/// The paper pack (version 2) without its atlas, with `dress` blocks
/// added to its dressings.
fn dressed(dress: &str) -> Ui {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/styles/paper/style.ogt"
    );
    let text = std::fs::read_to_string(path).expect("the paper pack");
    let file = parse_style(&format!("{text}{dress}")).unwrap_or_else(|e| panic!("{e}"));
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_theme(file.theme(None, [0; 6]));
    ui.set_dressings(file.dressings.clone());
    ui
}

fn fills_in(ui: &Ui, rect: Rect) -> Vec<[f32; 4]> {
    ui.frame()
        .items
        .iter()
        .filter_map(|i| i.cmd.filled())
        .filter(|(r, _)| {
            (r.x - rect.x).abs() < 1.0
                && (r.y - rect.y).abs() < 1.0
                && (r.width - rect.width).abs() < 1.0
        })
        .map(|(_, c)| c)
        .collect()
}

fn near(a: [f32; 4], b: Srgba) -> bool {
    let l = b.linear();
    (0..3).all(|k| (a[k] - l[k]).abs() < 0.01)
}

#[test]
fn the_most_specific_dressing_wins_whatever_order_the_style_writes_them() {
    let ui = dressed(concat!(
        "dress \"Button.base[primary][pressed]\" {\n  fill \"#00ff00\"\n}\n",
        "dress \"Button.base\" {\n  fill \"#ff0000\"\n  stroke 3 \"#ffffff\"\n}\n",
        "dress \"Button.base[primary]\" {\n  fill \"#0000ff\"\n}\n",
    ));
    let fill = |sel: &str| {
        let part = PartRef::parse(sel, ElementClass::Button).unwrap();
        let r = ui.resolve_part(&part);
        (format!("{:?}", r.draw.fills), r.draw.strokes.len())
    };
    let (pressed, strokes) = fill("Button.base[primary][pressed]");
    assert!(
        pressed.contains("[0, 255, 0]"),
        "pressed primary: {pressed}"
    );
    assert_eq!(strokes, 1, "and keeps the base's stroke it does not set");
    assert!(fill("Button.base[primary]").0.contains("[0, 0, 255]"));
    assert!(
        fill("Button.base[secondary][pressed]")
            .0
            .contains("[255, 0, 0]")
    );
}

#[test]
fn a_games_attribute_dresses_only_the_cells_that_carry_it() {
    let epic = Srgba::rgb(0xa3, 0x35, 0xee);
    let mut ui = dressed("dress \"Slot.frame[rarity=epic]\" {\n  fill \"#a335ee\"\n}\n");
    let build = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        let range = ui.open_grid(
            "Bag",
            GridDesc {
                count: 2,
                cell_width: 64.0,
                columns: 2,
                ..GridDesc::default()
            },
        );
        for i in range.first..range.last {
            ui.grid_item(i, false);
            if i == 0 {
                ui.part_attr("rarity", "epic");
            }
            ui.close();
        }
        ui.close_list();
        ui.end();
    };
    for _ in 0..3 {
        build(&mut ui);
    }
    let cell = |k: u32| {
        let e = ui
            .frame()
            .element(&format!("Bag/Cell#{k}"))
            .unwrap_or_else(|| panic!("cell {k}"))
            .clone();
        Rect::new(e.x, e.y, e.width, e.height)
    };
    assert!(
        fills_in(&ui, cell(0)).iter().any(|c| near(*c, epic)),
        "the epic cell draws the dressing's fill"
    );
    assert!(
        !fills_in(&ui, cell(1)).iter().any(|c| near(*c, epic)),
        "a plain cell does not"
    );
}

#[test]
fn an_undressed_panel_draws_its_line_and_elevation_from_the_tokens() {
    let mut ui = ui_with_font(test_font_metrics());
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.panel("Card", PanelDesc::default(), |ui| {
        ui.label("T", "Title", LabelDesc::default());
    });
    ui.end();
    let modes: Vec<&og_render::UiMode> = ui
        .frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Shape { shape, .. } => Some(&shape.mode),
            _ => None,
        })
        .collect();
    assert!(
        modes
            .iter()
            .any(|m| matches!(m, og_render::UiMode::Stroke(_))),
        "its line: {modes:?}"
    );
    assert!(
        modes
            .iter()
            .any(|m| matches!(m, og_render::UiMode::Shadow { .. })),
        "its elevation's shadow: {modes:?}"
    );
}

#[test]
fn a_hover_moves_the_buttons_fill_on_the_ui_spring() {
    let mut ui = ui_with_font(test_font_metrics());
    let p = Theme::default().palette;
    let build = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, Viewport::new(800.0, 600.0));
        ui.button(
            "Go",
            "Go",
            ButtonDesc {
                variant: ButtonVariant::Primary,
                ..ButtonDesc::default()
            },
        );
        ui.end();
    };
    for _ in 0..3 {
        build(&mut ui, InputFrame::default());
    }
    let e = ui.frame().element("Go").unwrap().clone();
    let rect = Rect::new(e.x, e.y, e.width, e.height);
    let (x, y) = e.center();
    let fill = |ui: &Ui| fills_in(ui, rect)[0];
    assert!(near(fill(&ui), p.primary), "at rest: primary");
    build(&mut ui, InputFrame::hover(x, y));
    build(&mut ui, InputFrame::hover(x, y));
    let mid = fill(&ui);
    assert!(
        !near(mid, p.primary) && !near(mid, p.primary_hover),
        "two ticks in it is on its way: {mid:?}"
    );
    for _ in 0..90 {
        build(&mut ui, InputFrame::hover(x, y));
    }
    assert!(near(fill(&ui), p.primary_hover), "and arrives");
}

fn theme_with_set() -> Theme {
    let mut t = Theme::default();
    t.tokens.feedback[SoundEvent::Select.index()] = Feedback {
        sounds: [11, 12, 13, 0],
        count: 3,
        volume: [0.5, 0.8],
        pitch: [0.9, 1.2],
        haptic: Haptic::preset(HapticPreset::Tap),
    };
    t
}

fn plays(ui: &mut Ui, n: usize) -> Vec<FeedbackPlay> {
    for _ in 0..n {
        ui.feedback(SoundEvent::Select);
    }
    ui.take_feedback()
}

#[test]
fn a_sound_set_never_repeats_and_plays_within_its_ranges() {
    let mut ui = Ui::default();
    ui.set_theme(theme_with_set());
    let saved = ui.save_state();
    let a = plays(&mut ui, 60);
    assert_eq!(a.len(), 60);
    for w in a.windows(2) {
        assert_ne!(w[0].sound, w[1].sound, "the same sound twice in a row");
    }
    for s in [11, 12, 13] {
        assert!(a.iter().any(|f| f.sound == s), "{s} never plays");
    }
    assert!(a.iter().all(|f| (0.5..=0.8).contains(&f.volume)));
    assert!(a.iter().all(|f| (0.9..=1.2).contains(&f.pitch)));
    let lo = a.iter().map(|f| f.pitch).fold(f32::MAX, f32::min);
    let hi = a.iter().map(|f| f.pitch).fold(f32::MIN, f32::max);
    assert!(
        hi - lo > 0.15,
        "the pitch spreads over its range: {lo}..{hi}"
    );
    // The choices are UI state: a restore replays them.
    ui.restore_state(&saved).unwrap();
    let b = plays(&mut ui, 60);
    let key = |v: &[FeedbackPlay]| -> Vec<(u64, u32)> {
        v.iter().map(|f| (f.sound, f.pitch.to_bits())).collect()
    };
    assert_eq!(key(&a), key(&b));
    // A rising cascade climbs the range from its bottom to its top.
    ui.feedback_rising(SoundEvent::Select, 0.0);
    ui.feedback_rising(SoundEvent::Select, 1.0);
    let rise = ui.take_feedback();
    assert_eq!((rise[0].pitch, rise[1].pitch), (0.9, 1.2));
}

#[test]
fn the_players_haptics_setting_scales_the_pattern_and_zero_turns_it_off() {
    let mut ui = Ui::default();
    ui.set_theme(theme_with_set());
    let full = plays(&mut ui, 1)[0].haptic;
    assert!(full.count > 0);
    ui.set_haptics(0.5);
    let half = plays(&mut ui, 1)[0].haptic;
    assert_eq!(half.count, full.count);
    let (f, h) = (full.pulses()[0], half.pulses()[0]);
    assert!((h.high - f.high * 0.5).abs() < 1e-6 && (h.low - f.low * 0.5).abs() < 1e-6);
    ui.set_haptics(0.0);
    let off = plays(&mut ui, 1);
    assert_eq!(off[0].haptic.count, 0, "off plays no pattern");
    assert!(off[0].sound != 0, "but still its sound");
}

/// tooltip a level (five).
#[test]
fn a_parts_border_draws_under_its_label() {
    let (shot, _) = with_test_font(sample_screen_shot);
    assert!(
        shot.ui.frame().levels() <= 4,
        "{} levels",
        shot.ui.frame().levels()
    );
}

/// which drew them by the `chip` class (Lantern Vale's settings on metal).
#[test]
fn knobs_and_switch_tracks_stay_round_in_a_square_style() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../templates/archetypes/roguelike/assets/styles/lampdive/style.ogt");
    let file = parse_style(&std::fs::read_to_string(dir).unwrap()).unwrap();
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_theme(file.theme(None, [0; 6]));
    ui.set_dressings(file.dressings.clone());
    let (mut v, mut on) = (0.5, true);
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.slider("S", &mut v, SliderDesc::default());
    ui.toggle("T", &mut on, "Reduce motion");
    ui.end();
    // Every fill drawn for the slider's knob and the switch (shorter than
    // the slider's 4 px track): its corner reaches half its short side.
    let fills: Vec<(f32, f32)> = ui
        .frame()
        .items
        .iter()
        .filter_map(|it| match &it.cmd {
            DrawCmd::Shape { shape, .. } if shape.mode == og_render::UiMode::Fill => {
                let og_render::UiGeometry::Rect { radii, .. } = shape.geometry else {
                    return None;
                };
                Some((radii[0], shape.half.x.min(shape.half.y)))
            }
            _ => None,
        })
        .filter(|&(_, half)| half > 4.0)
        .collect();
    assert_eq!(
        fills.len(),
        3,
        "the knob, the track and its knob: {fills:?}"
    );
    for (radius, half) in fills {
        assert!(
            radius >= half,
            "a corner of {radius} on a half side of {half}"
        );
    }
}

/// a background.
#[test]
fn an_anchor_without_a_background_draws_no_frame() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/styles");
    for seed in ["storybook", "kawaii"] {
        let text = std::fs::read_to_string(dir.join(seed).join("style.ogt")).unwrap();
        let file = parse_style(&text).unwrap();
        let mut ui = ui_with_font(test_font_metrics());
        ui.set_theme(file.theme(None, [0; 6]));
        ui.set_dressings(file.dressings.clone());
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.anchor("Score", AnchorDesc::default(), |ui| {
            ui.label("Value", "680", LabelDesc::default());
        });
        ui.end();
        let drawn: Vec<_> = ui
            .frame()
            .items
            .iter()
            .filter(|it| !matches!(it.cmd, DrawCmd::Text { .. } | DrawCmd::StyledText { .. }))
            .map(|it| format!("{:?}", std::mem::discriminant(&it.cmd)))
            .collect();
        assert!(drawn.is_empty(), "{seed}: {drawn:?}");
    }
}
