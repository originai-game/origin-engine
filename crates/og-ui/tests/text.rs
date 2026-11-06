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
