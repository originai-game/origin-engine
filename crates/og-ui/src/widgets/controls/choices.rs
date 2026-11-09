//! Choices: the switch, the checkbox, the radio group and the segmented
//! control. Each moves between its states as a §7.3 group (selection on
//! and off, the knob's slide, the tick's pop, the thumb's glide) on the
//! style's motion, and is a row the whole of which takes the press.

use super::*;

/// Which side of a switch or a check its label sits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LabelSide {
    /// Before it, the switch at the row's end (a settings row).
    #[default]
    Start,
    /// After it (a check's default).
    End,
}

impl LabelSide {
    pub fn from_name(s: &str) -> Option<LabelSide> {
        Some(match s {
            "start" => LabelSide::Start,
            "end" => LabelSide::End,
            _ => return None,
        })
    }
}

/// A switch: its label, description and states.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToggleSwitchDesc {
    /// Its label (the whole row takes the press).
    pub label: String,
    pub description: String,
    pub disabled: bool,
    pub side: LabelSide,
    pub size: ControlSize,
    /// The row's width; `None` fits.
    pub width: Option<Size>,
    /// What a screen reader says when on and off; empty for none.
    pub on_label: String,
    pub off_label: String,
}

/// A switch's or a check's state after this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SwitchResult {
    pub response: Response,
    /// Its state, to keep and pass back.
    pub on: bool,
    pub changed: bool,
}

/// A checkbox: its label, description and states.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckFieldDesc {
    pub label: String,
    pub description: String,
    /// Neither: some of what it stands for is on. A press turns it on.
    pub mixed: bool,
    pub disabled: bool,
    /// It must be checked: danger outline and the validity group.
    pub invalid: bool,
    pub side: LabelSide,
    pub width: Option<Size>,
}

impl Default for CheckFieldDesc {
    fn default() -> Self {
        CheckFieldDesc {
            label: String::new(),
            description: String::new(),
            mixed: false,
            disabled: false,
            invalid: false,
            side: LabelSide::End,
            width: None,
        }
    }
}

/// One choice of a radio group.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RadioOption {
    pub label: String,
    pub description: String,
    pub icon: String,
    pub disabled: bool,
}

/// How a radio group draws its choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RadioVariant {
    #[default]
    List,
    Cards,
}

/// A radio group: its choices and layout.
#[derive(Clone, Debug, PartialEq)]
pub struct RadioGroupDesc {
    pub options: Vec<RadioOption>,
    /// The chosen one; -1 is none.
    pub selected: i32,
    /// The group's name, shown above it; empty for none.
    pub label: String,
    pub variant: RadioVariant,
    /// The choices across (they wrap) rather than down.
    pub row: bool,
    pub disabled: bool,
    pub width: Option<Size>,
}

impl Default for RadioGroupDesc {
    fn default() -> Self {
        RadioGroupDesc {
            options: Vec::new(),
            selected: -1,
            label: String::new(),
            variant: RadioVariant::List,
            row: false,
            disabled: false,
            width: None,
        }
    }
}

/// A choice control's state after this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChoiceIndexResult {
    /// The chosen index, to keep and pass back (-1: none).
    pub selected: i32,
    pub changed: bool,
    /// The index with the focus (-1: none).
    pub focused: i32,
}

/// One segment of a segmented control.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SegmentItem {
    pub label: String,
    pub icon: String,
    /// A count badge after its label (`3`, `New`); empty is none.
    pub badge: String,
    pub disabled: bool,
    /// Its accessible name when it has no label.
    pub aria_label: String,
}

/// A segmented control: its segments and layout.
#[derive(Clone, Debug, PartialEq)]
pub struct SegmentedDesc {
    pub items: Vec<SegmentItem>,
    pub selected: i32,
    pub size: ControlSize,
    /// Every segment as wide as the widest.
    pub equal: bool,
    pub width: Option<Size>,
    pub label: String,
    pub disabled: bool,
}

impl Default for SegmentedDesc {
    fn default() -> Self {
        SegmentedDesc {
            items: Vec::new(),
            selected: 0,
            size: ControlSize::Medium,
            equal: true,
            width: None,
            label: String::new(),
            disabled: false,
        }
    }
}

impl Ui {
    /// A label and its description, a column that grows.
    fn labelled(&mut self, label: &str, description: &str, disabled: bool) {
        if label.is_empty() && description.is_empty() {
            return;
        }
        let t = self.theme;
        self.open(
            "Text",
            Element {
                direction: Direction::Column,
                width: Size::grow(),
                gap: t.space.xxs,
                ..Element::default()
            },
        );
        if !label.is_empty() {
            self.text(
                "Label",
                label,
                Text {
                    color: Ink::Role(if disabled { "inkFaint" } else { "ink" }),
                    width: Some(Size::grow()),
                    ..Text::default()
                },
            );
        }
        if !description.is_empty() {
            self.text(
                "Description",
                description,
                Text {
                    step: Some(t.type_ramp.sm),
                    color: Ink::Role(if disabled { "inkFaint" } else { "inkMuted" }),
                    width: Some(Size::grow()),
                    ..Text::default()
                },
            );
        }
        self.close();
    }

    /// A switch: a track and a knob that slides from its start to its end
    /// as it turns on (its selection group, on the style's motion), the
    /// track filling with primary, with a label and a description; the
    /// whole row takes a press, Enter, Space or A. Plays toggleOn and
    /// toggleOff. Parts: `Toggle.row`, `Toggle.track[on|off]`,
    /// `Toggle.knob[on]`. Role switch. Probe: `<id>`, `<id>/Track`,
    /// `<id>/Track/Knob`.
    pub fn toggle_switch(
        &mut self,
        id: impl Into<Id>,
        on: bool,
        desc: &ToggleSwitchDesc,
    ) -> SwitchResult {
        let id = id.into();
        let t = self.theme;
        let th = self.by_text(match desc.size {
            ControlSize::Small => 22.0,
            ControlSize::Large => 32.0,
            ControlSize::Medium => 28.0,
        });
        let tw = (th * 1.72).round();
        let inset = (th * 0.11).round().max(2.0);
        let knob = th - inset * 2.0;
        let travel = (tw - knob - inset * 2.0) * if self.rtl() { -1.0 } else { 1.0 };
        let off_fill = mix(self.role("controlStrong"), self.role("ink"), 0.12);
        let on_fill = self.role("primary");
        let knob_fill = self.role("knob");
        // An undressed knob that all but vanishes into the track it is on (a
        // neutral style's white on white) turns the inverse ink there, so it
        // and its tick in the track's colour still read. A style that dresses
        // the knob chose its look.
        let knob_on = if !self.dress.dresses("Toggle", "knob")
            && knob_fill.srgba().contrast(on_fill.srgba()) < 1.5
        {
            self.role("inkInverse")
        } else {
            knob_fill
        };
        let name = if desc.label.is_empty() {
            id.as_str().to_owned()
        } else {
            desc.label.clone()
        };
        let value = (!desc.on_label.is_empty() || !desc.off_label.is_empty()).then(|| {
            if on {
                desc.on_label.clone()
            } else {
                desc.off_label.clone()
            }
        });
        let still = self.still();
        let ink = self.role("ink");
        let built = self.ctl(
            id,
            Ctl {
                part: "Toggle.row",
                disabled: desc.disabled,
                flags: vec![Some(if on { "on" } else { "off" })],
                width: desc.width,
                padding: Insets::xy(t.space.xs, t.space.xxs),
                gap: t.space.md,
                fill: None,
                wash: Some(Some(ink)),
                wash_by: 0.06,
                lift: 0.0,
                press: 1.0,
                semantics: Some(Semantics {
                    role: Some("switch".into()),
                    name: Some(name),
                    checked: Some(on),
                    value,
                    ..Semantics::default()
                }),
                ..Ctl::default()
            },
            |ui, ix| {
                if desc.side == LabelSide::Start {
                    ui.labelled(&desc.label, &desc.description, desc.disabled);
                }
                let track = format!(
                    "Toggle.track{}",
                    flags(&[
                        Some(if on { "on" } else { "off" }),
                        Some("round"),
                        ix.flag()
                    ])
                );
                ui.open(
                    "Track",
                    Element {
                        width: Size::px(tw),
                        height: Size::px(th),
                        cross: Cross::Center,
                        radius: th / 2.0,
                        background: Some(if on { on_fill } else { off_fill }.srgba()),
                        part: crate::dress::PartRef::parse(&track, ElementClass::Chip).ok(),
                        ..Element::default()
                    },
                );
                let kpart = format!("Toggle.knob{}", flags(&[on.then_some("on"), Some("round")]));
                ui.open(
                    "Knob",
                    Element {
                        width: Size::px(knob),
                        height: Size::px(knob),
                        radius: knob / 2.0,
                        align: Align::Center,
                        cross: Cross::Center,
                        background: Some(if on { knob_on } else { knob_fill }.srgba()),
                        part: crate::dress::PartRef::parse(&kpart, ElementClass::Chip).ok(),
                        ..Element::default()
                    },
                );
                // A tick in the knob while on: the state is never colour alone (XAG 103).
                ui.open(
                    "Mark",
                    Element {
                        width: Size::px(knob),
                        height: Size::px(knob),
                        align: Align::Center,
                        cross: Cross::Center,
                        ..Element::default()
                    },
                );
                ui.drawn(
                    "Tick",
                    "tick",
                    (knob * 0.62).round(),
                    on_fill,
                    Some((knob / 9.0).max(1.5)),
                );
                ui.close();
                let (g, s) = two_state(
                    "selection",
                    on,
                    vec![(Prop::Opacity, 0.0), (Prop::Scale, 0.5)],
                    vec![(Prop::Opacity, 1.0), (Prop::Scale, 1.0)],
                    Vec::new(),
                );
                ui.set_props(Props {
                    states: vec![g],
                    state: vec![s],
                    ..Props::default()
                });
                ui.close();
                // The knob slides along the track and stretches a little while pressed.
                let (g, s) = two_state(
                    "selection",
                    on,
                    [vec![(Prop::X, 0.0)], bg_values(knob_fill)].concat(),
                    [vec![(Prop::X, travel)], bg_values(knob_on)].concat(),
                    Vec::new(),
                );
                ui.over(Props {
                    insets: Edges {
                        start: Some(inset),
                        top: Some(inset),
                        ..Edges::default()
                    },
                    states: vec![g],
                    state: vec![s],
                    transform: Transform {
                        scale_x: if ix == Ix::Pressed && !still {
                            1.12
                        } else {
                            1.0
                        },
                        ..Transform::IDENTITY
                    },
                    motion: Some(Transition::Token(MotionToken::Ui)),
                    ..Props::default()
                });
                ui.close();
                let (g, s) = two_state(
                    "selection",
                    on,
                    bg_values(off_fill),
                    bg_values(on_fill),
                    Vec::new(),
                );
                ui.set_props(Props {
                    states: vec![g],
                    state: vec![s],
                    ..Props::default()
                });
                if desc.side == LabelSide::End {
                    ui.labelled(&desc.label, &desc.description, desc.disabled);
                }
            },
        );
        let mut next = on;
        if built.r.clicked && !desc.disabled {
            next = !on;
            self.feedback(if next {
                SoundEvent::ToggleOn
            } else {
                SoundEvent::ToggleOff
            });
        }
        SwitchResult {
            response: built.r,
            on: next,
            changed: next != on,
        }
    }

    /// A checkbox: a box that fills with primary and draws its tick (a dash
    /// when mixed) with a pop as it turns on (its selection group), with a
    /// label and a description; the whole row takes a press. Plays
    /// toggleOn and toggleOff. Parts: `Checkbox.row`,
    /// `Checkbox.box[checked|mixed][invalid]`, `Checkbox.mark`. Role
    /// checkbox. Probe: `<id>`, `<id>/Box`, `<id>/Box/Mark`.
    pub fn check_field(
        &mut self,
        id: impl Into<Id>,
        checked: bool,
        desc: &CheckFieldDesc,
    ) -> SwitchResult {
        let id = id.into();
        let t = self.theme;
        let on = checked || desc.mixed;
        let bx = self.by_text(22.0);
        let off_fill = self.role("well");
        let on_fill = self.role("primary");
        let on_primary = self.role("onPrimary");
        let still = self.still();
        let name = if desc.label.is_empty() {
            id.as_str().to_owned()
        } else {
            desc.label.clone()
        };
        let (mixed, invalid) = (desc.mixed, desc.invalid);
        let built = self.ctl(
            id,
            Ctl {
                part: "Checkbox.row",
                disabled: desc.disabled,
                flags: vec![
                    checked.then_some("checked"),
                    mixed.then_some("mixed"),
                    invalid.then_some("invalid"),
                ],
                width: desc.width,
                padding: Insets::xy(t.space.xs, t.space.xxs),
                gap: t.space.md,
                cross: if desc.description.is_empty() {
                    Cross::Center
                } else {
                    Cross::Start
                },
                fill: None,
                wash_by: 0.06,
                lift: 0.0,
                press: 1.0,
                semantics: Some(Semantics {
                    role: Some("checkbox".into()),
                    name: Some(name),
                    checked: Some(checked),
                    value: mixed.then(|| "mixed".into()),
                    ..Semantics::default()
                }),
                ..Ctl::default()
            },
            |ui, ix| {
                if desc.side == LabelSide::Start {
                    ui.labelled(&desc.label, &desc.description, desc.disabled);
                }
                let part = format!(
                    "Checkbox.box{}",
                    flags(&[
                        checked.then_some("checked"),
                        mixed.then_some("mixed"),
                        invalid.then_some("invalid"),
                        ix.flag()
                    ])
                );
                ui.open(
                    "Box",
                    Element {
                        width: Size::px(bx),
                        height: Size::px(bx),
                        align: Align::Center,
                        cross: Cross::Center,
                        radius: (bx * 0.24).round().max(3.0),
                        background: Some(if on { on_fill } else { off_fill }.srgba()),
                        // A field: its rim (found at 3:1 while empty, danger
                        // when invalid, none once filled) is the part's dressing.
                        part: crate::dress::PartRef::parse(&part, ElementClass::Field).ok(),
                        ..Element::default()
                    },
                );
                ui.open(
                    "Mark",
                    Element {
                        width: Size::px(bx),
                        height: Size::px(bx),
                        align: Align::Center,
                        cross: Cross::Center,
                        part: crate::dress::PartRef::parse("Checkbox.mark", ElementClass::Badge)
                            .ok(),
                        ..Element::default()
                    },
                );
                ui.drawn(
                    "Glyph",
                    if mixed { "minus" } else { "tick" },
                    (bx * 0.78).round(),
                    on_primary,
                    Some((bx / 9.0).max(2.0)),
                );
                ui.close();
                // The mark pops in as it turns on, and leaves at once.
                let (g, s) = two_state(
                    "selection",
                    on,
                    vec![(Prop::Opacity, 0.0), (Prop::Scale, 0.4)],
                    vec![(Prop::Opacity, 1.0), (Prop::Scale, 1.0)],
                    vec![
                        StateTransition {
                            from: "off".into(),
                            to: "on".into(),
                            transition: if still {
                                tween(0.0, Ease::CubicBezier(crate::theme::EASE_OUT))
                            } else {
                                spring(260.0, 0.45)
                            },
                        },
                        StateTransition {
                            from: "on".into(),
                            to: "off".into(),
                            transition: tween(
                                if still { 0.0 } else { 90.0 },
                                Ease::CubicBezier(crate::theme::EASE_EXIT),
                            ),
                        },
                    ],
                );
                ui.set_props(Props {
                    states: vec![g],
                    state: vec![s],
                    ..Props::default()
                });
                ui.close();
                let (g, s) = two_state(
                    "selection",
                    on,
                    bg_values(off_fill),
                    bg_values(on_fill),
                    Vec::new(),
                );
                ui.set_props(Props {
                    states: vec![g],
                    state: vec![s],
                    ..Props::default()
                });
                if desc.side == LabelSide::End {
                    ui.labelled(&desc.label, &desc.description, desc.disabled);
                }
            },
        );
        let mut next = checked;
        if built.r.clicked && !desc.disabled {
            // A mixed box turns on, as a group's parent does.
            next = mixed || !checked;
            self.feedback(if next {
                SoundEvent::ToggleOn
            } else {
                SoundEvent::ToggleOff
            });
        }
        SwitchResult {
            response: built.r,
            on: next,
            changed: next != checked || (mixed && built.r.clicked && !desc.disabled),
        }
    }

    fn radio_ring(&mut self, selected: bool, ix: Ix, size: f32) {
        let dot = (size * 0.46).round();
        let (p, w, op) = (
            self.role("primary"),
            self.role("well"),
            self.role("onPrimary"),
        );
        let still = self.still();
        let part = format!(
            "Radio.ring{}",
            flags(&[selected.then_some("selected"), Some("round"), ix.flag()])
        );
        self.open(
            "Ring",
            Element {
                width: Size::px(size),
                height: Size::px(size),
                align: Align::Center,
                cross: Cross::Center,
                radius: size / 2.0,
                background: Some(if selected { p } else { w }.srgba()),
                // A field, as a checkbox's box: its rim is the part's dressing.
                part: crate::dress::PartRef::parse(&part, ElementClass::Field).ok(),
                ..Element::default()
            },
        );
        self.open(
            "Dot",
            Element {
                width: Size::px(dot),
                height: Size::px(dot),
                radius: dot / 2.0,
                background: Some(op.srgba()),
                part: crate::dress::PartRef::parse("Radio.dot[round]", ElementClass::Badge).ok(),
                ..Element::default()
            },
        );
        self.close();
        let (g, s) = two_state(
            "selection",
            selected,
            vec![(Prop::Scale, 0.0), (Prop::Opacity, 0.0)],
            vec![(Prop::Scale, 1.0), (Prop::Opacity, 1.0)],
            vec![StateTransition {
                from: "off".into(),
                to: "on".into(),
                transition: if still {
                    tween(0.0, Ease::CubicBezier(crate::theme::EASE_OUT))
                } else {
                    spring(240.0, 0.4)
                },
            }],
        );
        self.set_props(Props {
            states: vec![g],
            state: vec![s],
            ..Props::default()
        });
        self.close();
        let (g, s) = two_state(
            "selection",
            selected,
            bg_values(w),
            bg_values(p),
            Vec::new(),
        );
        self.set_props(Props {
            states: vec![g],
            state: vec![s],
            ..Props::default()
        });
    }

    /// A radio group: one of several choices, as rings with labels or as
    /// cards. A press chooses; Left and Right step the choice while one has
    /// the focus, skipping disabled ones. Plays select. Parts:
    /// `Radio.option[selected]`, `Radio.ring[selected]`, `Radio.dot`,
    /// `Radio.card[selected]`. Role radiogroup with radio children. Probe:
    /// `<id>`, `<id>/Options/Option#i`.
    pub fn radio_group(&mut self, id: impl Into<Id>, desc: &RadioGroupDesc) -> ChoiceIndexResult {
        let t = self.theme;
        let cards = desc.variant == RadioVariant::Cards;
        let before = desc.selected;
        let mut selected = before;
        let mut focused = -1;
        let mut moved = 0;
        let mut move_from = -1;
        let size = self.by_text(20.0);
        self.open(
            id,
            Element {
                direction: Direction::Column,
                width: desc.width.unwrap_or_default(),
                gap: t.space.sm,
                cross: Cross::Stretch,
                ..Element::default()
            },
        );
        if !desc.label.is_empty() {
            self.text(
                "Legend",
                &desc.label,
                Text {
                    step: Some(t.type_ramp.sm),
                    color: Ink::Role("inkMuted"),
                    nowrap: true,
                    ..Text::default()
                },
            );
        }
        self.open(
            "Options",
            Element {
                direction: if desc.row {
                    Direction::Row
                } else {
                    Direction::Column
                },
                width: if desc.row { Size::fit() } else { Size::grow() },
                gap: if cards { t.space.sm } else { t.space.xxs },
                wrap: desc.row,
                cross: Cross::Stretch,
                ..Element::default()
            },
        );
        let (raised, primary, ink) = (
            self.role("panelRaised"),
            self.role("primary"),
            self.role("ink"),
        );
        for (i, o) in desc.options.iter().enumerate() {
            let on = i as i32 == selected;
            let off = desc.disabled || o.disabled;
            let built = self.ctl(
                crate::ui_idi("Option", i as u32),
                Ctl {
                    part: if cards { "Radio.card" } else { "Radio.option" },
                    cls: if cards {
                        ElementClass::Card
                    } else {
                        ElementClass::Button
                    },
                    disabled: off,
                    flags: vec![on.then_some("selected")],
                    width: (!desc.row).then(Size::grow),
                    padding: if cards {
                        Insets::all(t.space.md)
                    } else {
                        Insets::xy(t.space.xs, t.space.xxs)
                    },
                    gap: t.space.md,
                    cross: if o.description.is_empty() {
                        Cross::Center
                    } else {
                        Cross::Start
                    },
                    fill: cards.then(|| {
                        if on {
                            mix(raised, primary, 0.1)
                        } else {
                            raised
                        }
                    }),
                    wash_by: if cards { 0.05 } else { 0.06 },
                    lift: if cards { -2.0 } else { 0.0 },
                    press: if cards { 0.985 } else { 1.0 },
                    adjustable: true,
                    radius: Some(if cards {
                        t.radius.panel
                    } else {
                        t.radius.control
                    }),
                    semantics: Some(Semantics {
                        role: Some("radio".into()),
                        name: Some(o.label.clone()),
                        checked: Some(on),
                        ..Semantics::default()
                    }),
                    ..Ctl::default()
                },
                |ui, ix| {
                    if cards && !o.icon.is_empty() {
                        let s = ui.by_text(24.0);
                        ui.glyph_icon("Icon", &o.icon, s, if on { primary } else { ink });
                    }
                    if !cards {
                        ui.radio_ring(on, ix, size);
                    }
                    ui.labelled(&o.label, &o.description, off);
                    if cards {
                        ui.radio_ring(on, ix, size);
                        if on {
                            ui.chosen_ring("Chosen", 2.0, "primary", "Radio.chosen");
                        }
                    }
                },
            );
            if built.r.focused {
                focused = i as i32;
            }
            if built.r.clicked && !off && i as i32 != selected {
                selected = i as i32;
                self.feedback(SoundEvent::Select);
            }
            if built.r.adjust != 0 {
                moved = self.forward(built.r.adjust);
                move_from = i as i32;
            }
        }
        self.close();
        self.close();
        let path = self.last_path();
        self.semantics(&Semantics {
            role: Some("radiogroup".into()),
            name: (!desc.label.is_empty()).then(|| desc.label.clone()),
            ..Semantics::default()
        });
        if moved != 0 && move_from >= 0 {
            // Arrows move the choice and the focus with it, skipping disabled ones.
            let opts = &desc.options;
            let next = step_enabled(
                opts.len(),
                move_from,
                moved,
                |i| desc.disabled || opts[i].disabled,
                true,
            );
            if next != move_from {
                selected = next;
                self.feedback(SoundEvent::Select);
                self.set_focus(&format!("{path}/Options/Option#{next}"));
            }
        }
        ChoiceIndexResult {
            selected,
            changed: selected != before,
            focused,
        }
    }

    /// A segmented control: a track whose raised thumb slides to the chosen
    /// segment and takes its width on the style's spring (layout
    /// animation). A press chooses; Left and Right step the choice while it
    /// has the focus, skipping disabled segments, mirrored right to left.
    /// The segments never shrink below their labels: when they do not fit,
    /// the track scrolls sideways under a fade at the cut edge. Plays tab.
    /// Parts: `Tabs.track`, `Tabs.indicator` (the mark that slides to the
    /// selected tab) and `Tabs.tab[selected]`. Role radiogroup of radios. Probe:
    /// `<id>`, `<id>/Strip/Track/Item#i`, `<id>/Strip/Track/Thumb`.
    pub fn segmented(&mut self, id: impl Into<Id>, desc: &SegmentedDesc) -> ChoiceIndexResult {
        let id = id.into();
        let key = id.as_str().to_owned();
        let t = self.theme;
        let sizes = self.sizes();
        let touch = self.touch_frame();
        let small = desc.size == ControlSize::Small;
        let before = desc.selected;
        let mut selected = before;
        let mut focused = -1;
        // A segment is at least 44 tall under a finger: the track's padding is around it.
        let inner = ((if small {
            sizes.control_sm
        } else {
            sizes.control_md
        }) - 4.0)
            .max(if touch { TOUCH } else { 0.0 });
        let pad = (inner * 0.08).round().max(2.0);
        let h = inner + 2.0 * pad;
        let n = desc.items.len();
        // Each segment's own width (remembered from last tick): segments share
        // the row but never shrink below it.
        let nat: Vec<f32> = (0..n)
            .map(|i| self.memory(&format!("{key}~nat{i}"), 0.0) as f32)
            .collect();
        let widest = nat.iter().copied().fold(0.0, f32::max);
        let nat_sum = if desc.equal {
            widest * n as f32
        } else {
            nat.iter().sum()
        };
        let mut measured = vec![-1.0f32; n];
        let mut adjust = 0;
        // The track as laid out last tick: where the thumb goes.
        let me_path = format!("{}/Strip/Track", self.child_path(&key));
        let me = self.bounds_of(&me_path);
        let sel_b = self.bounds_of(&format!("{me_path}/Item#{selected}"));
        let (well, raised, ink, muted) = (
            self.role("well"),
            self.role("panelRaised"),
            self.role("ink"),
            self.role("inkMuted"),
        );
        let rtl = self.rtl();
        let still = self.still();
        self.open(
            id,
            Element {
                direction: Direction::Row,
                width: desc.width.unwrap_or(Size::grow()),
                height: Size::px(h),
                ..Element::default()
            },
        );
        self.open_scroll(
            "Strip",
            crate::ui::ScrollDesc {
                width: Size::grow(),
                height: Size::px(h),
                axes: crate::ui::ScrollAxes::Horizontal,
                ..crate::ui::ScrollDesc::default()
            },
        );
        let track_part = format!(
            "Tabs.track{}",
            flags(&[desc.disabled.then_some("disabled")])
        );
        self.open(
            "Track",
            Element {
                direction: Direction::Row,
                width: Size::Grow {
                    min: nat_sum + 2.0 * pad,
                    max: f32::INFINITY,
                },
                height: Size::px(h),
                padding: Insets::all(pad),
                cross: Cross::Stretch,
                radius: t.radius.control + pad,
                background: Some(well.srgba()),
                part: crate::dress::PartRef::parse(&track_part, ElementClass::Tab).ok(),
                ..Element::default()
            },
        );
        if let (Some(sel), Some(me)) = (sel_b, me) {
            self.open(
                "Thumb",
                Element {
                    width: Size::px(sel.width),
                    height: Size::px(sel.height),
                    radius: t.radius.control,
                    background: Some(raised.srgba()),
                    part: crate::dress::PartRef::parse("Tabs.indicator", ElementClass::Tab).ok(),
                    ..Element::default()
                },
            );
            self.close();
            self.set_props(Props {
                position: Position::Absolute,
                // Insets are inside the track's padding, from its start.
                insets: Edges {
                    start: Some(if rtl {
                        me.x + me.width - pad - (sel.x + sel.width)
                    } else {
                        sel.x - me.x - pad
                    }),
                    top: Some(sel.y - me.y - pad),
                    ..Edges::default()
                },
                overlaps: vec![me_path.clone()],
                inert: true,
                layout_motion: (!still).then_some(Transition::Token(MotionToken::Ui)),
                ..Props::default()
            });
        }
        let pad_x = self.by_text(if small { t.space.sm } else { t.space.md });
        let min_w = if touch { TOUCH } else { 0.0 };
        for (i, it) in desc.items.iter().enumerate() {
            let on = i as i32 == selected;
            let off = desc.disabled || it.disabled;
            let name = if it.label.is_empty() {
                it.aria_label.clone()
            } else {
                it.label.clone()
            };
            let built = self.ctl(
                crate::ui_idi("Item", i as u32),
                Ctl {
                    part: "Tabs.tab",
                    cls: ElementClass::Tab,
                    disabled: off,
                    flags: vec![on.then_some("selected")],
                    width: Some(if desc.equal {
                        Size::Grow {
                            min: widest.max(min_w),
                            max: f32::INFINITY,
                        }
                    } else {
                        Size::Fit {
                            min: min_w,
                            max: f32::INFINITY,
                        }
                    }),
                    height: Some(Size::grow()),
                    padding: Insets::xy(pad_x, 0.0),
                    gap: t.space.xs,
                    fill: None,
                    wash_by: if on { 0.0 } else { 0.05 },
                    lift: 0.0,
                    press: 0.97,
                    adjustable: true,
                    props: Props {
                        z: 1,
                        ..Props::default()
                    },
                    semantics: Some(Semantics {
                        role: Some("radio".into()),
                        name: Some(name),
                        checked: Some(on),
                        ..Semantics::default()
                    }),
                    ..Ctl::default()
                },
                |ui, _| {
                    let c = if on { ink } else { muted };
                    if !it.icon.is_empty() {
                        let s = ui.by_text(if small { 16.0 } else { 18.0 });
                        ui.glyph_icon("Icon", &it.icon, s, c);
                    }
                    if !it.label.is_empty() {
                        ui.text(
                            "Label",
                            &it.label,
                            Text {
                                step: Some(if small {
                                    t.type_ramp.sm
                                } else {
                                    t.type_ramp.body
                                }),
                                color: Ink::C(c),
                                nowrap: true,
                                align: TextAlign::Center,
                                ..Text::default()
                            },
                        );
                    }
                    if !it.badge.is_empty() {
                        ui.badge("Badge", &it.badge, crate::widgets::Tone::Primary);
                    }
                },
            );
            // Its own width: its parts' as laid out, never what a share squeezed it to.
            let parts: Vec<f32> = ["Icon", "Label", "Badge"]
                .iter()
                .filter_map(|c| {
                    self.bounds_of(&format!("{}/{c}", built.path))
                        .map(|b| b.width)
                })
                .collect();
            if !parts.is_empty() {
                measured[i] = (parts.iter().sum::<f32>()
                    + (parts.len() - 1) as f32 * t.space.xs
                    + 2.0 * pad_x)
                    .ceil();
            }
            if built.r.focused {
                focused = i as i32;
            }
            if built.r.clicked && !off {
                selected = i as i32;
            }
            if built.r.adjust != 0 {
                adjust = self.forward(built.r.adjust);
            }
        }
        self.close();
        let track_path = self.last_path();
        self.close();
        // The fades: where the track runs past the strip's edge (last tick's).
        let sb = self.bounds_of(parent_of(&track_path));
        let tb = self.bounds_of(&track_path);
        let (left_cut, right_cut) = match (sb, tb) {
            (Some(s), Some(b)) => (b.x < s.x - 0.5, b.x + b.width > s.x + s.width + 0.5),
            _ => (false, false),
        };
        let ui_spring = Transition::Token(MotionToken::Ui);
        let fs = self.animate(
            &format!("{key}~fs"),
            if if rtl { right_cut } else { left_cut } {
                1.0
            } else {
                0.0
            },
            ui_spring,
        );
        let fe = self.animate(
            &format!("{key}~fe"),
            if if rtl { left_cut } else { right_cut } {
                1.0
            } else {
                0.0
            },
            ui_spring,
        );
        self.edge_fade("FadeStart", true, fs, 24.0, "panel");
        self.edge_fade("FadeEnd", false, fe, 24.0, "panel");
        self.close();
        for (i, w) in measured.iter().enumerate() {
            if *w >= 0.0 {
                self.remember(&format!("{key}~nat{i}"), f64::from(*w));
            } else {
                self.remember(&format!("{key}~nat{i}"), f64::from(nat[i]));
            }
        }
        self.semantics(&Semantics {
            role: Some("radiogroup".into()),
            name: (!desc.label.is_empty()).then(|| desc.label.clone()),
            ..Semantics::default()
        });
        if adjust != 0 {
            let items = &desc.items;
            let next = step_enabled(
                n,
                selected,
                adjust,
                |i| desc.disabled || items[i].disabled,
                false,
            );
            if next != selected {
                selected = next;
                self.set_focus(&format!("{track_path}/Item#{next}"));
            }
        }
        if selected != before {
            self.feedback(SoundEvent::Tab);
        }
        ChoiceIndexResult {
            selected,
            changed: selected != before,
            focused,
        }
    }
}
