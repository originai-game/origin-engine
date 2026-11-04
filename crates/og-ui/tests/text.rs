//! Text editing and truncation, tick by tick through `InputFrame`s: the
//! field's selection (Shift with the arrows, a drag, Ctrl/Cmd+A, C and X),
//! the multi-line area, an input method's composition, the caret's blink,
//! the typed-text stream, and labels cut with an ellipsis.
//!
//! Ways these could fail, each covered below:
//! - Shift+arrows move the caret without selecting, typing does not
//!   replace the selection, or Ctrl+C copies nothing or the wrong text →
//!   `shift_arrows_select_and_typing_replaces_the_selection`
//! - a drag selects from the wrong end, or a press does not collapse it →
//!   `a_drag_selects_and_a_press_places_the_caret`
//! - Enter submits a text area instead of breaking the line, ↑/↓ move
//!   focus instead of the caret, the area does not wrap →
//!   `a_text_area_breaks_lines_and_moves_between_them`
//! - a composition is committed early, not shown, does not replace the
//!   selection, or survives the commit →
//!   `a_composition_shows_at_the_caret_until_it_commits`
//! - the caret blinks by anything but ticks, or blinks under reduced motion
//!   → `the_caret_blinks_by_the_tick`
//! - keys type again once a typed-text stream exists (doubled letters) →
//!   `a_typed_text_stream_takes_over_from_the_keys`
//! - an ellipsis cut too early, too late, or the probe reads the cut text →
//!   `labels_cut_with_an_ellipsis_where_they_do_not_fit`
//! - the edit state does not survive a save and restore →
//!   `the_edit_state_is_saved_and_restored`

use crate::support;

use og_core::glam::Vec2;
use og_ui::*;
use support::*;

#[test]
fn scene_text_uses_its_scheme_roles_without_recolouring_normal_labels() {
    let mut scene = og_render::RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    for scheme in [Scheme::Dark, Scheme::Light] {
        let mut theme = Theme::new(scheme);
        theme.palette.scene_ink = Srgba::rgb(17, 190, 127);
        theme.palette.scene_shadow = Srgba::rgb(90, 12, 60);
        ui.set_theme(theme);
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.label(
            "Hud",
            "Speed",
            LabelDesc {
                on_scene: true,
                ..LabelDesc::default()
            },
        );
        ui.label("Plain", "Menu", LabelDesc::default());
        ui.end();
        let effects: Vec<_> = ui
            .frame()
            .items
            .iter()
            .filter_map(|i| match &i.cmd {
                DrawCmd::StyledText { text, effects } => Some((text, effects)),
                _ => None,
            })
            .collect();
        assert_eq!(effects.len(), 1);
        assert!(
            matches!(&**effects[0].0, DrawCmd::Text { color, .. } if *color == theme.palette.scene_ink.linear())
        );
        assert_eq!(
            effects[0].1.shadows[0].color,
            theme.palette.scene_shadow.linear()
        );
        assert!(ui.frame().items.iter().any(|i| matches!(&i.cmd, DrawCmd::Text { color, .. } if *color == theme.palette.ink.linear())));
    }
}

struct Rig {
    ui: Ui,
    value: String,
    area: String,
    vp: Viewport,
}

fn build(ui: &mut Ui, value: &mut String, area: &mut String) -> (Response, Response) {
    let mut r = (Response::default(), Response::default());
    ui.column("Form", Element::column(), |ui| {
        r.0 = ui.text_field(
            "Name",
            value,
            &FieldDesc {
                width: Some(Size::Fixed(300.0)),
                ..FieldDesc::default()
            },
        );
        r.1 = ui.text_area(
            "Notes",
            area,
            &TextAreaDesc {
                width: Some(Size::Fixed(200.0)),
                rows: 3,
                ..TextAreaDesc::default()
            },
        );
    });
    r
}

impl Rig {
    fn new(value: &str) -> Rig {
        let mut scene = og_render::RenderScene::new();
        Rig {
            ui: ui_with_font(add_test_font(&mut scene)),
            value: value.into(),
            area: String::new(),
            vp: Viewport::new(800.0, 600.0),
        }
    }

    fn step(&mut self, input: InputFrame) -> (Response, Response) {
        self.ui.begin(input, self.vp);
        let r = build(&mut self.ui, &mut self.value, &mut self.area);
        self.ui.end();
        r
    }

    fn rect(&self, id: &str) -> (f32, f32, f32, f32) {
        let e = self.ui.frame().element(id).expect(id);
        (e.x, e.y, e.width, e.height)
    }

    /// A click at a point (press, then release a tick later).
    fn click(&mut self, x: f32, y: f32, shift: bool) {
        let at = Vec2::new(x, y);
        self.step(InputFrame {
            pointer: at,
            pointer_inside: true,
            down: true,
            pressed: true,
            shift,
            ..InputFrame::default()
        });
        self.step(InputFrame {
            pointer: at,
            pointer_inside: true,
            released: true,
            ..InputFrame::default()
        });
    }

    fn keys(&mut self, keys: &[UiKey], shift: bool, ctrl: bool) {
        self.step(InputFrame {
            keys: keys.to_vec(),
            shift,
            ctrl,
            pointer_inside: true,
            ..InputFrame::default()
        });
    }

    fn typed(&mut self, text: &str) {
        let events: Vec<og_input::TextInput> = og_input::TextInput::split(text, false);
        self.step(InputFrame::default().with_text(&events));
    }

    fn compose(&mut self, text: &str) {
        let events = og_input::TextInput::split(text, true);
        self.step(InputFrame::default().with_text(&events));
    }

    fn focus_field(&mut self) {
        self.step(InputFrame::default());
        let (x, y, w, h) = self.rect("Name");
        self.click(x + w - 2.0, y + h / 2.0, false);
    }

    fn target(&self) -> TextTarget {
        self.ui
            .frame()
            .text_input
            .clone()
            .expect("a focused text control")
    }
}

#[test]
fn shift_arrows_select_and_typing_replaces_the_selection() {
    let mut rig = Rig::new("hello world");
    rig.focus_field();
    // The press at the field's right end put the caret after "world".
    rig.keys(
        &[
            UiKey::Left,
            UiKey::Left,
            UiKey::Left,
            UiKey::Left,
            UiKey::Left,
        ],
        true,
        false,
    );
    assert_eq!(rig.target().selection, "world");
    rig.keys(&[UiKey::KeyC], false, true);
    assert_eq!(rig.ui.take_clipboard().as_deref(), Some("world"));
    assert_eq!(rig.ui.take_clipboard(), None, "taken once");
    rig.typed("there");
    assert_eq!(rig.value, "hello there");
    assert_eq!(rig.target().selection, "", "typing collapsed the selection");
    // Home with Shift selects back to the start; Ctrl+X cuts it.
    rig.keys(&[UiKey::Home], true, false);
    assert_eq!(rig.target().selection, "hello there");
    rig.keys(&[UiKey::KeyX], false, true);
    assert_eq!(rig.ui.take_clipboard().as_deref(), Some("hello there"));
    assert_eq!(rig.value, "");
    // Ctrl+A selects everything; Backspace deletes the selection.
    rig.typed("abc");
    rig.keys(&[UiKey::KeyA], false, true);
    assert_eq!(rig.target().selection, "abc");
    rig.keys(&[UiKey::Backspace], false, false);
    assert_eq!(rig.value, "");
    // Left without Shift collapses a selection to its start.
    rig.typed("xyz");
    rig.keys(&[UiKey::Left, UiKey::Left], true, false);
    rig.keys(&[UiKey::Left], false, false);
    rig.typed("_");
    assert_eq!(rig.value, "x_yz");
}

#[test]
fn a_drag_selects_and_a_press_places_the_caret() {
    let mut rig = Rig::new("drag to select");
    rig.focus_field();
    let (x, y, _, h) = rig.rect("Name");
    let cy = y + h / 2.0;
    // Press near the start of the text, drag to the right, release.
    let start = Vec2::new(x + 14.0, cy);
    rig.step(InputFrame {
        pointer: start,
        pointer_inside: true,
        down: true,
        pressed: true,
        ..InputFrame::default()
    });
    for dx in [20.0, 40.0, 60.0] {
        rig.step(InputFrame {
            pointer: start + Vec2::new(dx, 0.0),
            pointer_inside: true,
            down: true,
            ..InputFrame::default()
        });
    }
    let sel = rig.target().selection;
    assert!(
        "drag to select".starts_with(&sel) && sel.len() >= 5,
        "a drag of 60 px from the start selects the start: {sel:?}"
    );
    rig.step(InputFrame {
        pointer: start + Vec2::new(60.0, 0.0),
        pointer_inside: true,
        released: true,
        ..InputFrame::default()
    });
    assert_eq!(rig.target().selection, sel, "the release keeps it");
    // A press elsewhere in the field collapses it there; Shift+press extends.
    rig.click(x + 14.0, cy, false);
    assert_eq!(rig.target().selection, "");
    rig.click(x + 14.0 + 60.0, cy, true);
    assert_eq!(
        rig.target().selection,
        sel,
        "Shift+press extends to the press"
    );
}

#[test]
fn a_text_area_breaks_lines_and_moves_between_them() {
    let mut rig = Rig::new("");
    rig.step(InputFrame::default());
    let (x, y, w, _) = rig.rect("Notes");
    rig.click(x + w / 2.0, y + 10.0, false);
    assert!(rig.target().multiline);
    rig.typed("first");
    let (_, r) = rig.step(InputFrame {
        keys: vec![UiKey::Enter],
        ..InputFrame::default()
    });
    assert!(!r.submitted, "Enter breaks the line in an area");
    rig.typed("second");
    assert_eq!(rig.area, "first\nsecond");
    // Up moves to the line above (focus stays), Shift+End selects its end.
    rig.keys(&[UiKey::Up], false, false);
    assert!(rig.ui.frame().text_input.is_some(), "↑ belongs to the area");
    rig.keys(&[UiKey::Home], false, false);
    rig.keys(&[UiKey::End], true, false);
    assert_eq!(rig.target().selection, "first");
    rig.keys(&[UiKey::Down], false, false);
    rig.keys(&[UiKey::End], false, false);
    rig.typed("!");
    assert_eq!(rig.area, "first\nsecond!");
    // A long line wraps between words at the area's width (200 px).
    rig.typed(" and then a much longer line of words that cannot fit");
    let probe = rig.ui.frame().element("Notes").expect("area").text.clone();
    assert!(probe.ends_with("cannot fit"), "the probe reads the value");
}

#[test]
fn a_composition_shows_at_the_caret_until_it_commits() {
    let mut rig = Rig::new("ab");
    rig.focus_field();
    rig.keys(&[UiKey::Left], false, false);
    rig.compose("ni");
    assert_eq!(rig.value, "ab", "nothing is committed while composing");
    assert!(rig.target().composing);
    assert_eq!(
        rig.ui.frame().element("Name").expect("field").text,
        "ab",
        "the probe reads the value, not the composition"
    );
    // The composing text is drawn in the field (the shown text has it at the caret).
    let shown: Vec<String> = rig
        .ui
        .frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert!(shown.iter().any(|t| t == "anib"), "{shown:?}");
    rig.compose("你");
    let events = og_input::TextInput::split("你", false);
    rig.step(InputFrame::default().with_text(&events));
    assert_eq!(rig.value, "a你b");
    assert!(!rig.target().composing, "the commit ended the composition");
    // A composition replaces the selection as it starts.
    rig.keys(&[UiKey::KeyA], false, true);
    rig.compose("x");
    assert_eq!(rig.value, "");
    rig.compose("");
    assert!(!rig.target().composing, "an empty composition ends it");
}

#[test]
fn the_caret_blinks_by_the_tick() {
    let caret = |rig: &Rig| {
        rig.ui.frame().items.iter().any(|i| {
            matches!(&i.cmd, DrawCmd::Quad { rect, .. } if rect.width <= 1.01 && rect.height > 10.0)
        })
    };
    let mut rig = Rig::new("blink");
    rig.focus_field();
    // 60 ticks a second: shown for 32 ticks after the last move, then hidden 32.
    let mut seen = Vec::new();
    for _ in 0..70 {
        rig.step(InputFrame::default());
        seen.push(caret(&rig));
    }
    let first_off = seen.iter().position(|s| !s).expect("it hides");
    let back_on = seen[first_off..]
        .iter()
        .position(|s| *s)
        .expect("it shows again")
        + first_off;
    assert!(
        (29..=32).contains(&first_off),
        "hidden after about 32 ticks: {first_off}"
    );
    assert_eq!(back_on - first_off, 32, "for 32 ticks");
    // A move shows it at once.
    rig.keys(&[UiKey::Left], false, false);
    assert!(caret(&rig));
    // Under reduced motion it never hides.
    let mut theme = *rig.ui.theme();
    theme.reduced_motion = true;
    rig.ui.set_theme(theme);
    for _ in 0..80 {
        rig.step(InputFrame::default());
        assert!(caret(&rig));
    }
}

#[test]
fn a_typed_text_stream_takes_over_from_the_keys() {
    let mut rig = Rig::new("");
    rig.focus_field();
    // Before any stream, key presses type (the probe-less fallback).
    rig.step(InputFrame {
        text: "k".into(),
        ..InputFrame::default()
    });
    assert_eq!(rig.value, "k");
    rig.typed("s");
    assert_eq!(rig.value, "ks");
    // From then on a key's text is ignored (the stream brings it), so a key
    // and its typed character arriving in different ticks type once.
    rig.step(InputFrame {
        text: "x".into(),
        ..InputFrame::default()
    });
    assert_eq!(rig.value, "ks");
}

#[test]
fn an_auto_fit_label_shrinks_by_type_steps_then_cuts() {
    // A title at xl in a 160 px column: a short one keeps xl, a longer one
    // takes the largest smaller step it fits at (never one it would also
    // fit at a step up), and one too long even at meta is meta and cut.
    let mut scene = og_render::RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    let ramp = ui.theme().type_ramp;
    let (xl, meta) = (ramp.xl, ramp.meta);
    let texts = [
        "Gold",
        "Hidden treasures!",
        "The hidden treasures of the drowned bell tower",
    ];
    let draw = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.column(
            "Col",
            Element {
                width: Size::Fixed(160.0),
                ..Element::column()
            },
            |ui| {
                for (k, t) in texts.iter().enumerate() {
                    ui.label(
                        ui_idi("T", k as u32),
                        t,
                        LabelDesc {
                            step: Some(xl),
                            nowrap: true,
                            ellipsis: true,
                            fit_min: Some(meta),
                            width: Size::grow(),
                            ..LabelDesc::default()
                        },
                    );
                }
            },
        );
        ui.end();
        ui.frame()
            .items
            .iter()
            .filter_map(|i| match &i.cmd {
                DrawCmd::Text {
                    text, size, bounds, ..
                } => Some((text.clone(), *size, bounds.width)),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    // Twice: the second frame lays out from the first's cache.
    for _ in 0..2 {
        let drawn = draw(&mut ui);
        let at = |k: usize| {
            drawn
                .iter()
                .find(|d| d.0.starts_with(&texts[k][..4]))
                .unwrap_or_else(|| panic!("{k}: {drawn:?}"))
                .clone()
        };
        assert_eq!(at(0).1, xl.size, "a short title keeps its step");
        let mid = at(1);
        assert_eq!(mid.0, texts[1], "it fits whole: {mid:?}");
        assert!(
            mid.1 < xl.size && mid.1 > meta.size,
            "{mid:?} between meta {} and xl {}",
            meta.size,
            xl.size
        );
        assert!(mid.2 <= 160.5, "{mid:?}");
        // Not one step smaller than it needs: at the next ramp size up it overflows.
        let m = ui.fonts().body.metrics.clone();
        let up = [ramp.lg, ramp.xl, ramp.base, ramp.reading]
            .iter()
            .map(|s| s.size)
            .filter(|&s| s > mid.1)
            .fold(f32::MAX, f32::min);
        assert!(
            m.measure_tracked(texts[1], up, 0.0) > 160.0,
            "{up} fits too"
        );
        let cut = at(2);
        assert_eq!(cut.1, meta.size, "{cut:?}");
        assert!(cut.0.ends_with('…') || cut.0.ends_with("..."), "{cut:?}");
    }
}

#[test]
fn labels_cut_with_an_ellipsis_where_they_do_not_fit() {
    let mut scene = og_render::RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    let long = "A label far too long for the narrow column it sits in";
    let step = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.column(
            "Col",
            Element {
                width: Size::Fixed(160.0),
                ..Element::column()
            },
            |ui| {
                ui.label(
                    "One",
                    long,
                    LabelDesc {
                        nowrap: true,
                        ellipsis: true,
                        width: Size::grow(),
                        ..LabelDesc::default()
                    },
                );
                ui.label(
                    "Two",
                    long,
                    LabelDesc {
                        ellipsis: true,
                        max_lines: 2,
                        width: Size::grow(),
                        ..LabelDesc::default()
                    },
                );
                ui.label(
                    "Fits",
                    "Short",
                    LabelDesc {
                        nowrap: true,
                        ellipsis: true,
                        ..LabelDesc::default()
                    },
                );
            },
        );
        ui.end();
    };
    step(&mut ui);
    let texts: Vec<(String, f32, f32)> = ui
        .frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Text { text, bounds, .. } => Some((text.clone(), bounds.width, bounds.y)),
            _ => None,
        })
        .collect();
    // The test font has no "…": its ellipsis is three stops.
    let cut = |t: &str| t.ends_with('…') || t.ends_with("...");
    let one = texts
        .iter()
        .find(|t| t.0.starts_with("A label") && cut(&t.0));
    let one = one.unwrap_or_else(|| panic!("a cut line: {texts:?}"));
    assert!(one.1 <= 160.5, "it fits its 160 px: {one:?}");
    let two: Vec<&(String, f32, f32)> = texts
        .iter()
        .filter(|t| t.2 > one.2 + 1.0 && !t.0.starts_with("Short"))
        .collect();
    assert_eq!(two.len(), 2, "two lines: {two:?}");
    assert!(cut(&two[1].0) && !cut(&two[0].0), "{two:?}");
    assert!(
        texts.iter().any(|t| t.0 == "Short"),
        "a text that fits is not cut"
    );
    assert_eq!(
        ui.frame().element("One").expect("one").text,
        long,
        "the probe reads the whole text"
    );
}

#[test]
fn the_edit_state_is_saved_and_restored() {
    let mut rig = Rig::new("snapshot");
    rig.focus_field();
    rig.keys(&[UiKey::Left, UiKey::Left], true, false);
    rig.compose("zz");
    let saved = rig.ui.save_state();
    let before = rig.target();
    rig.keys(&[UiKey::Home], false, false);
    rig.compose("");
    rig.ui.restore_state(&saved).expect("restores");
    rig.step(InputFrame::default());
    let after = rig.target();
    assert_eq!(after.selection, before.selection);
    assert!(after.composing);
}

#[test]
fn uppercase_is_measured_but_source_and_editable_values_are_preserved() {
    let mut scene = og_render::RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    let mut theme = *ui.theme();
    for steps in &mut theme.tokens.typography.steps {
        for token in steps {
            token.uppercase = true;
        }
    }
    ui.set_theme(theme);
    let mut value = "mixed".to_string();
    for locale in ["en", "tr", "en"] {
        ui.set_locale(locale);
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.label("Source", "straße", LabelDesc::default());
        ui.label("Locale", "izmir", LabelDesc::default());
        ui.rich_text("Rich", "a[b]bc[/b]", RichTextDesc::default());
        ui.text_field("Edit", &mut value, &FieldDesc::default());
        ui.end();
        let drawn: Vec<String> = ui
            .frame()
            .items
            .iter()
            .filter_map(|item| {
                let cmd = match &item.cmd {
                    DrawCmd::StyledText { text, .. } => &**text,
                    cmd => cmd,
                };
                match cmd {
                    DrawCmd::Text { text, .. } => Some(text.clone()),
                    _ => None,
                }
            })
            .collect();
        for expected in [
            "STRASSE",
            if locale == "tr" { "İZMİR" } else { "IZMIR" },
            "A",
            "BC",
            "mixed",
        ] {
            assert!(
                drawn.iter().any(|text| text == expected),
                "{expected}: {drawn:?}"
            );
        }
        assert_eq!(ui.frame().element("Source").unwrap().text, "straße");
        assert_eq!(ui.frame().element("Rich").unwrap().text, "abc");
        assert_eq!(value, "mixed");
        let expected =
            ui.fonts()
                .body
                .metrics
                .measure_tracked("STRASSE", ui.theme().type_ramp.body.size, 0.0);
        assert!((ui.frame().element("Source").unwrap().width - expected).abs() < 1.0);
    }
}

/// A label that asks for uppercase (a component's eyebrow) gets the same
/// locale-aware casing as a role with `case upper`, in a style whose roles
/// leave case alone: String.toUpperCase in the component drew Turkish
/// "izmir" as IZMIR.
#[test]
fn a_label_asking_for_uppercase_is_cased_in_the_uis_locale() {
    let mut scene = og_render::RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    for (locale, upper) in [("tr", "İZMİR"), ("en", "IZMIR")] {
        ui.set_locale(locale);
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        let caps = LabelDesc {
            uppercase: true,
            ..LabelDesc::default()
        };
        ui.label("Caps", "izmir", caps);
        ui.label("Plain", "izmir", LabelDesc::default());
        ui.end();
        let drawn: Vec<String> = ui
            .frame()
            .items
            .iter()
            .filter_map(|item| match &item.cmd {
                DrawCmd::Text { text, .. } => Some(text.clone()),
                DrawCmd::StyledText { text, .. } => match &**text {
                    DrawCmd::Text { text, .. } => Some(text.clone()),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        assert!(drawn.iter().any(|t| t == upper), "{locale}: {drawn:?}");
        assert!(
            drawn.iter().any(|t| t == "izmir"),
            "the plain label is not cased"
        );
        assert_eq!(ui.frame().element("Caps").unwrap().text, "izmir");
    }
}

/// A label mirrored by its transform (scaleX −1: a right arrow drawn as a
/// mirrored left one) keeps the mirror on its text for the overlay, which
/// flips the glyphs. Baked into the pen like an upright scale, the mirror
/// moved the pen to the box's other edge and drew the glyphs unflipped
/// from there: a left chevron pushed against the right edge.
#[test]
fn a_mirrored_label_keeps_its_mirror_for_the_glyphs() {
    let mut scene = og_render::RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    let draw = |ui: &mut Ui, scale_x: f32| {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.label("Arrow", "<", LabelDesc::default());
        ui.set_props(Props {
            transform: Transform {
                scale_x,
                ..Transform::IDENTITY
            },
            ..Props::default()
        });
        ui.end();
    };
    let maps = |ui: &Ui| -> Vec<f32> {
        ui.frame()
            .items
            .iter()
            .filter(|i| {
                let cmd = match &i.cmd {
                    DrawCmd::StyledText { text, .. } => &**text,
                    c => c,
                };
                matches!(cmd, DrawCmd::Text { .. })
            })
            .map(|i| i.transform.a)
            .collect()
    };
    draw(&mut ui, 1.0);
    draw(&mut ui, 1.0);
    let upright = maps(&ui);
    assert!(!upright.is_empty(), "the label draws its text");
    assert!(
        upright.iter().all(|a| *a == 1.0),
        "upright, baked: {upright:?}"
    );
    draw(&mut ui, -1.0);
    draw(&mut ui, -1.0);
    let mirrored = maps(&ui);
    assert!(!mirrored.is_empty(), "the mirrored label draws its text");
    assert!(
        mirrored.iter().all(|a| *a == -1.0),
        "the mirror reaches the overlay with the text: {mirrored:?}"
    );
}

/// A label in a scroll view takes the style's drop across: a figure at the end of a row
/// in a scroll view kept its last font pixel of drop past its box, where the
/// view's edge cut it (pixel's results screen).
#[test]
fn a_label_holds_its_style_drop_inside_its_box() {
    let mut scene = og_render::RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    let mut theme = *ui.theme();
    theme.art.text_shadows[0] = Some(og_ui::style_file::Shadow {
        dx: 2.0,
        dy: 2.0,
        blur: 0.0,
        spread: 0.0,
        color: Srgba::BLACK,
    });
    ui.set_theme(theme);
    for _ in 0..2 {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.scroll(
            "Page",
            ScrollDesc {
                width: Size::Fixed(300.0),
                height: Size::Fixed(200.0),
                ..ScrollDesc::default()
            },
            |ui| ui.label("Value", "48,210", LabelDesc::default()),
        );
        ui.end();
    }
    let e = ui.frame().element("Page/Value").expect("the label");
    let right = e.x + e.width;
    let texts: Vec<Rect> = ui
        .frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Text { bounds, .. } => Some(*bounds),
            _ => None,
        })
        .collect();
    assert_eq!(texts.len(), 2, "the drop and the text: {texts:?}");
    for b in texts {
        assert!(b.x + b.width <= right + 0.01, "{b:?} inside {right}");
    }
}

/// A line's ink is the line's, never the style's drop under it: the drop
/// (and the half-strength copy a blurred drop adds) are pushed first, a
/// step off, in the shadow's colour, and a check that read the first run
/// measured a blue "Lv. 125" over its black drop at 1:1 (foundry's
/// generators). The same words a row below are a line of their own.
#[test]
fn drawn_lines_are_the_lines_not_their_drops() {
    let mut scene = og_render::RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    let mut theme = *ui.theme();
    theme.art.text_shadows[0] = Some(og_ui::style_file::Shadow {
        dx: 0.0,
        dy: 1.0,
        blur: 1.0,
        spread: 0.0,
        color: Srgba::BLACK,
    });
    ui.set_theme(theme);
    let blue = Srgba::rgb(76, 140, 230);
    for _ in 0..2 {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.column("List", Element::column(), |ui| {
            for id in ["A", "B"] {
                ui.label(
                    id,
                    "Lv. 125",
                    LabelDesc {
                        color: Some(blue),
                        ..LabelDesc::default()
                    },
                );
            }
        });
        ui.end();
    }
    let items = &ui.frame().items;
    let all = items
        .iter()
        .filter(|i| matches!(i.cmd, DrawCmd::Text { .. }))
        .count();
    assert_eq!(all, 6, "two lines, each with its drop and the blur's copy");
    let lines = og_ui::drawn_lines(items.iter().map(|i| &i.cmd));
    let inks: Vec<[f32; 4]> = lines
        .iter()
        .map(|c| match c {
            DrawCmd::Text { color, .. } => *color,
            _ => unreachable!(),
        })
        .collect();
    assert_eq!(
        inks,
        [blue.linear(), blue.linear()],
        "the two lines in their own ink"
    );
}
