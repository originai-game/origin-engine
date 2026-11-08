//! Ranges: the slider and the number stepper. A slider's track takes a
//! press anywhere and a drag (the pointer and a finger), Left and Right
//! while it has the focus, with ticks, marks, a readout and a bubble; a
//! stepper repeats while held, faster the longer.

use super::*;

/// A labelled point on a slider's track.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SliderMark {
    pub value: f32,
    /// What it says under the track; empty for a tick only.
    pub label: String,
}

/// How a control shows a number.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ValueFormat {
    /// Decimals; `None` follows the step (1 → 0, 0.5 → 1, 0.05 → 2).
    pub decimals: Option<usize>,
    /// Written after it (`%`, ` m`); empty for none.
    pub unit: String,
    /// Show 0–1 as a whole percent.
    pub percent: bool,
    /// Words for whole values from min (`Off`, `Low`, `High`).
    pub labels: Vec<String>,
    /// Whole digits shown, zero-padded and not grouped (a clock's `05`, a
    /// year's `2026`); 0 groups thousands.
    pub digits: usize,
}

impl ValueFormat {
    /// A number as a slider or stepper shows it: a level's word, a
    /// percent, or the number with its unit.
    pub fn show(&self, value: f32, min: f32, step: f32) -> String {
        let k = (value - min).round();
        if k >= 0.0 && (k as usize) < self.labels.len() {
            return self.labels[k as usize].clone();
        }
        if self.digits > 0 && value.fract() == 0.0 && value >= 0.0 {
            return format!("{:0w$}{}", value as i64, self.unit, w = self.digits);
        }
        format_value(value, self.decimals, step, &self.unit, self.percent)
    }
}

/// A slider: range, step, label, readout, ticks and marks.
#[derive(Clone, Debug, PartialEq)]
pub struct ValueSliderDesc {
    pub min: f32,
    pub max: f32,
    /// Snap to multiples of this from min (0: continuous; keys step a twentieth).
    pub step: f32,
    pub label: String,
    /// Its value at the label row's end.
    pub show_value: bool,
    pub format: ValueFormat,
    /// A tick at every step (at most 20 steps).
    pub ticks: bool,
    pub marks: Vec<SliderMark>,
    /// Fill from this value rather than min (a bipolar slider).
    pub origin: Option<f32>,
    pub disabled: bool,
    /// Its width; `None` grows.
    pub width: Option<Size>,
    /// A bubble over the knob with the value while it is dragged or focused.
    pub bubble: bool,
}

impl Default for ValueSliderDesc {
    fn default() -> Self {
        ValueSliderDesc {
            min: 0.0,
            max: 1.0,
            step: 0.0,
            label: String::new(),
            show_value: true,
            format: ValueFormat::default(),
            ticks: false,
            marks: Vec::new(),
            origin: None,
            disabled: false,
            width: None,
            bubble: true,
        }
    }
}

/// A slider's value after this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ValueSliderResult {
    pub response: Response,
    /// The value, to keep and pass back.
    pub value: f32,
    pub changed: bool,
    /// A drag ended, or a key or pad step landed.
    pub committed: bool,
}

/// A range slider ([`Ui::range_slider`]): two knobs on one track.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RangeSliderDesc {
    pub min: f32,
    pub max: f32,
    /// Snap to multiples of this from min (0: continuous).
    pub step: f32,
    /// The least distance between the knobs.
    pub min_gap: f32,
    pub label: String,
    /// Both values at the label row's end.
    pub show_value: bool,
    pub format: ValueFormat,
    pub disabled: bool,
    pub width: Option<Size>,
}

/// A range slider's values after this tick.
pub(crate) struct RangeSliderResult {
    pub low: f32,
    pub high: f32,
    /// Either changed this tick.
    pub changed: bool,
}

/// A number stepper: minus, the value and plus.
#[derive(Clone, Debug, PartialEq)]
pub struct NumberStepperDesc {
    pub min: Option<f32>,
    pub max: Option<f32>,
    /// What each press adds or takes.
    pub step: f32,
    /// What a held press's repeats move by after a second; 0 is ten steps.
    pub large_step: f32,
    pub label: String,
    pub format: ValueFormat,
    /// Past max it goes to min and back (needs both).
    pub wrap: bool,
    pub disabled: bool,
    pub width: Option<Size>,
}

impl Default for NumberStepperDesc {
    fn default() -> Self {
        NumberStepperDesc {
            min: None,
            max: None,
            step: 1.0,
            large_step: 0.0,
            label: String::new(),
            format: ValueFormat::default(),
            wrap: false,
            disabled: false,
            width: None,
        }
    }
}

/// A stepper's value after this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NumberResult {
    pub value: f32,
    pub changed: bool,
}

/// A slider track's geometry: its height, the knob, the rail and the
/// rail's width last tick (the track's less a knob).
#[derive(Clone, Copy)]
struct Geo {
    h: f32,
    knob: f32,
    rail: f32,
    width: f32,
}

/// Ticks at every step of a stepped range (at most 20 steps), as fractions.
fn tick_fractions(min: f32, max: f32, step: f32, on: bool) -> Vec<f32> {
    if !on || step <= 0.0 || max <= min {
        return Vec::new();
    }
    let n = ((max - min) / step).round() as i32;
    if !(1..=20).contains(&n) {
        return Vec::new();
    }
    (0..=n).map(|i| i as f32 / n as f32).collect()
}

impl Ui {
    /// A fraction across a track box for the frame's pointer, the knob's
    /// half kept at each end; the writing direction's.
    fn fraction_at(&self, b: Rect, knob: f32) -> f32 {
        let p = self.input().pointer;
        let x = ((p.x - b.x - knob / 2.0) / (b.width - knob).max(1.0)).clamp(0.0, 1.0);
        if self.rtl() { 1.0 - x } else { x }
    }

    /// The rail, its fill between two fractions, and its ticks.
    fn rail(&mut self, g: Geo, from: f32, to: f32, ticks: &[f32], disabled: bool) {
        let rail = mix(self.role("well"), self.role("ink"), 0.1);
        let fill = if disabled {
            self.role("inkFaint")
        } else {
            self.role("primary")
        };
        let (on_primary, strong) = (self.role("onPrimary"), self.role("lineStrong"));
        let part = |s: &str| crate::dress::PartRef::parse(s, ElementClass::Bar).ok();
        self.open(
            "Rail",
            Element {
                width: Size::grow(),
                height: Size::px(g.rail),
                radius: g.rail / 2.0,
                background: Some(rail.srgba()),
                part: part("Slider.track"),
                ..Element::default()
            },
        );
        self.open(
            "Fill",
            Element {
                width: Size::percent((to - from).clamp(0.0, 1.0)),
                height: Size::grow(),
                radius: g.rail / 2.0,
                background: Some(fill.srgba()),
                part: part("Slider.fill"),
                ..Element::default()
            },
        );
        self.close();
        self.over(Props {
            insets: Edges {
                start: Some((from.clamp(0.0, 1.0) * g.width).round()),
                top: Some(0.0),
                bottom: Some(0.0),
                ..Edges::default()
            },
            inert: true,
            ..Props::default()
        });
        for (i, tk) in ticks.iter().enumerate() {
            let passed = *tk >= from - 1e-6 && *tk <= to + 1e-6;
            let p = format!("Slider.tick{}", flags(&[passed.then_some("passed")]));
            self.open(
                crate::ui_idi("Tick", i as u32),
                Element {
                    width: Size::px(2.0),
                    height: Size::px(g.rail + 6.0),
                    radius: 1.0,
                    background: Some(if passed { on_primary } else { strong }.srgba()),
                    part: part(&p),
                    ..Element::default()
                },
            );
            self.close();
            self.over(Props {
                insets: Edges {
                    start: Some((tk * g.width).round() - 1.0),
                    top: Some(-3.0),
                    ..Edges::default()
                },
                inert: true,
                z: 1,
                ..Props::default()
            });
        }
        self.close();
        self.over(Props {
            insets: Edges {
                start: Some(g.knob / 2.0),
                end: Some(g.knob / 2.0),
                top: Some((g.h - g.rail) / 2.0),
                ..Edges::default()
            },
            inert: true,
            ..Props::default()
        });
    }

    /// A slider's knob: a round part at a fraction of the rail, drawn in
    /// its state; it swells under a finger and takes no press of its own.
    fn knob_at(&mut self, f: f32, g: Geo, pressed: bool, focused: bool, disabled: bool) {
        self.knob_named("Knob", None, None, f, g, (pressed, focused, disabled));
    }

    /// A knob named `name`, with a flag (`low`, `high`); with `semantics`
    /// it is its own focus stop that Left and Right adjust (a range
    /// slider's), else inert inside its track's control.
    fn knob_named(
        &mut self,
        name: &str,
        extra: Option<&'static str>,
        semantics: Option<Semantics>,
        f: f32,
        g: Geo,
        (pressed, focused, disabled): (bool, bool, bool),
    ) -> Built {
        let stop = semantics.is_some();
        let big = pressed && self.touch_frame() && !self.still();
        let last = self.last_path();
        let track = parent_of(&last).to_owned();
        let fill = if disabled {
            self.role("control")
        } else {
            self.role("knob")
        };
        let primary = self.role("primary");
        self.ctl(
            name,
            Ctl {
                part: "Slider.knob",
                cls: ElementClass::Chip,
                flags: vec![
                    Some("round"),
                    extra,
                    pressed.then_some("pressed"),
                    (focused && !pressed).then_some("focused"),
                ],
                disabled,
                focusable: stop,
                adjustable: stop,
                semantics,
                width: Some(Size::px(g.knob)),
                height: Some(Size::px(g.knob)),
                radius: Some(g.knob / 2.0),
                fill: Some(fill),
                wash: Some(None),
                lift: 0.0,
                press: 1.0,
                touch: if stop { TOUCH } else { 0.0 },
                inert: !stop,
                props: Props {
                    position: Position::Absolute,
                    // Along the track by layout, so it follows a track
                    // whose width changed this frame.
                    insets: Edges {
                        across: Some(f.clamp(0.0, 1.0)),
                        top: Some((g.h - g.knob) / 2.0),
                        ..Edges::default()
                    },
                    overlaps: if track.is_empty() {
                        Vec::new()
                    } else {
                        vec![track]
                    },
                    transform: Transform {
                        scale_x: if big { 1.15 } else { 1.0 },
                        scale_y: if big { 1.15 } else { 1.0 },
                        ..Transform::IDENTITY
                    },
                    motion: Some(Transition::Token(MotionToken::Ui)),
                    z: 2,
                    ..Props::default()
                },
                ..Ctl::default()
            },
            |ui, _| {
                // A dot of primary inside the knob while it is held or focused.
                if pressed || focused {
                    let c = (g.knob * 0.42).round();
                    ui.open(
                        "Core",
                        Element {
                            width: Size::px(c),
                            height: Size::px(c),
                            radius: c / 2.0,
                            background: Some(primary.srgba()),
                            part: crate::dress::PartRef::parse("Slider.core", ElementClass::Badge)
                                .ok(),
                            ..Element::default()
                        },
                    );
                    ui.close();
                }
            },
        )
    }

    /// The value bubble over a knob.
    fn bubble(&mut self, f: f32, label: &str, g: Geo) {
        let t = self.theme;
        let h = self.by_text(26.0);
        let ink = self.role("ink");
        self.open(
            "Bubble",
            Element {
                height: Size::px(h),
                padding: Insets::xy(t.space.sm, 0.0),
                align: Align::Center,
                cross: Cross::Center,
                radius: t.radius.control,
                background: Some(ink.srgba()),
                part: crate::dress::PartRef::parse("Slider.bubble", ElementClass::Tooltip).ok(),
                ..Element::default()
            },
        );
        self.text(
            "Text",
            label,
            Text {
                step: Some(t.type_ramp.sm),
                color: Ink::Role("inkInverse"),
                nowrap: true,
                font: FontRole::Mono,
                ..Text::default()
            },
        );
        self.close();
        let w = (label.chars().count() as f32 * 8.0 + t.space.sm * 2.0).max(32.0);
        let top = -self.by_text(30.0);
        self.over(Props {
            insets: Edges {
                start: Some((f * g.width + g.knob / 2.0 - w / 2.0).round()),
                top: Some(top),
                ..Edges::default()
            },
            inert: true,
            z: 5,
            enter: Some(crate::motion::PresenceSpec {
                values: vec![(Prop::Opacity, 0.0), (Prop::Y, 4.0)],
                transition: Transition::Token(MotionToken::Ui),
                delay_ms: 0.0,
            }),
            exit: Some(crate::motion::PresenceSpec {
                values: vec![(Prop::Opacity, 0.0)],
                transition: tween(90.0, Ease::CubicBezier(crate::theme::EASE_EXIT)),
                delay_ms: 0.0,
            }),
            reduced: Some(crate::motion::Reduced::Fade),
            ..Props::default()
        });
    }

    /// A slider: a track, a fill from min (or from an origin, both ways)
    /// and a knob. A press anywhere on the track jumps the knob there and a
    /// drag follows; Left and Right step it while it has the focus, with
    /// ticks at its steps, labelled marks, a readout and a bubble over the
    /// knob while it moves. Plays sliderTick at each step passed. Parts:
    /// `Slider.reach`, `Slider.track`, `Slider.fill`, `Slider.knob`,
    /// `Slider.core`, `Slider.tick[passed]`, `Slider.bubble`. Role slider.
    /// Probe: `<id>`, `<id>/Track`, `<id>/Track/Knob`, `<id>/Head/Value`.
    pub fn value_slider(
        &mut self,
        id: impl Into<Id>,
        value: f32,
        desc: &ValueSliderDesc,
    ) -> ValueSliderResult {
        let id = id.into();
        let key = id.as_str().to_owned();
        let t = self.theme;
        let (min, max, step) = (desc.min, desc.max, desc.step);
        let span = if max - min == 0.0 { 1.0 } else { max - min };
        let disabled = desc.disabled;
        let touch = self.touch_frame();
        let knob = self.by_text(if touch { 26.0 } else { 20.0 });
        let unit = if step > 0.0 { step } else { span / 20.0 };
        // Last tick's track, for the pointer and the rail's width.
        let track_path = format!("{}/Track", self.child_path(&key));
        let tb = self.bounds_of(&track_path);
        let g = Geo {
            h: (knob + 8.0).max(if touch { TOUCH } else { 28.0 }),
            knob,
            rail: if touch { 6.0 } else { 4.0 },
            width: tb.map_or(0.0, |b| (b.width - knob).max(1.0)),
        };
        let down = self.input().down;
        let held_mem = self.memory(&format!("{key}~held"), 0.0) > 0.0;
        let held = held_mem && down && !disabled;
        let (lo, hi) = (min.min(max), min.max(max));
        let mut v = if value.is_finite() { value } else { min }.clamp(lo, hi);
        if let (true, Some(b)) = (held, tb) {
            v = snap(min + self.fraction_at(b, knob) * span, min, max, step);
        }
        let shown = desc.format.show(v, min, step);
        let flash = self.memory(&format!("{key}~flash"), 0.0);
        let now = self.seconds();
        let mut committed = false;
        let label_before = self.label_before();
        self.open(
            id,
            Element {
                direction: Direction::Column,
                width: desc.width.unwrap_or(Size::grow()),
                gap: t.space.xs,
                cross: Cross::Stretch,
                ..Element::default()
            },
        );
        if !desc.label.is_empty() || desc.show_value {
            self.open(
                "Head",
                Element {
                    direction: Direction::Row,
                    width: Size::grow(),
                    gap: t.space.sm,
                    cross: Cross::Baseline,
                    ..Element::default()
                },
            );
            if desc.label.is_empty() {
                self.spacer("Lead", Some(Size::grow()));
            } else {
                // A long label wraps rather than losing its end.
                self.text(
                    "Label",
                    &desc.label,
                    Text {
                        step: Some(t.type_ramp.sm),
                        color: Ink::Role(if disabled { "inkFaint" } else { "inkMuted" }),
                        width: Some(Size::grow()),
                        ..Text::default()
                    },
                );
            }
            if desc.show_value {
                self.text(
                    "Value",
                    &shown,
                    Text {
                        step: Some(t.type_ramp.sm),
                        color: Ink::Role(if disabled { "inkFaint" } else { "ink" }),
                        nowrap: true,
                        font: FontRole::Mono,
                        ..Text::default()
                    },
                );
            }
            self.close();
        }
        let f = (v - min) / span;
        let origin = desc.origin.map_or(min, |o| o.clamp(lo, hi));
        let of = (origin - min) / span;
        // The step's ticks, and one at each mark (a `<datalist>` option):
        // an unlabelled mark is a tick only.
        let mut ticks = tick_fractions(min, max, step, desc.ticks);
        for m in &desc.marks {
            let f = ((m.value - min) / span).clamp(0.0, 1.0);
            if !ticks.iter().any(|t| (t - f).abs() < 1e-4) {
                ticks.push(f);
            }
        }
        ticks.sort_by(f32::total_cmp);
        // Unlabelled, it is named by the label before it in its row, else by its id.
        let name = Some(if desc.label.is_empty() {
            label_before.unwrap_or_else(|| key.clone())
        } else {
            desc.label.clone()
        });
        let show_bubble = desc.bubble && !disabled;
        let built = self.ctl(
            "Track",
            Ctl {
                // The reach a finger and the pointer take; the bar it holds
                // is `Slider.track`, as CSS names a range's.
                part: "Slider.reach",
                cls: ElementClass::Badge,
                disabled,
                adjustable: true,
                width: Some(Size::grow()),
                height: Some(Size::px(g.h)),
                fill: None,
                wash: Some(None),
                lift: 0.0,
                press: 1.0,
                semantics: Some(Semantics {
                    role: Some("slider".into()),
                    name,
                    value: Some(shown.clone()),
                    value_min: Some(min),
                    value_max: Some(max),
                    value_now: Some(v),
                    ..Semantics::default()
                }),
                ..Ctl::default()
            },
            |ui, ix| {
                ui.rail(g, f.min(of), f.max(of), &ticks, disabled);
                let pressed = held || ix == Ix::Pressed;
                ui.knob_at(f, g, pressed, ix == Ix::Focused, disabled);
                if show_bubble && (pressed || flash > now) {
                    ui.bubble(f, &shown, g);
                }
            },
        );
        if let (true, false, Some(b)) = (built.r.pressed, held, tb) {
            v = snap(min + self.fraction_at(b, knob) * span, min, max, step);
        }
        let stepped = built.r.adjust != 0 && !disabled;
        if stepped {
            v = snap(
                v + unit * self.forward(built.r.adjust) as f32,
                min,
                max,
                step.max(0.0),
            );
            committed = true;
        }
        if held_mem && !built.r.pressed && !down {
            committed = true;
        }
        // A screen reader's set value lands as a step does.
        #[cfg(not(target_arch = "wasm32"))]
        if !disabled {
            let tkey = crate::id::key_of_path(&built.path);
            let mut set = v;
            if self.accessibility_number(tkey, &mut set, lo, hi, step) {
                v = snap(set, min, max, step);
                committed = true;
            }
        }
        if !desc.marks.is_empty() {
            self.slider_marks(&desc.marks, min, span, g);
        }
        self.close();
        // Kept beside the slider, where they were read.
        self.remember(
            &format!("{key}~flash"),
            if stepped { now + 0.9 } else { flash },
        );
        self.remember(
            &format!("{key}~held"),
            if built.r.pressed || held { 1.0 } else { 0.0 },
        );
        let changed = v != value;
        if changed {
            let before = ((value - min) / unit).round();
            let after = ((v - min) / unit).round();
            if before != after {
                self.feedback_rising(SoundEvent::SliderTick, ((v - min) / span).clamp(0.0, 1.0));
            }
        }
        ValueSliderResult {
            response: built.r,
            value: v,
            changed,
            committed,
        }
    }

    /// A range slider (a document's `<input type="range">` with `low` and
    /// `high`): two knobs on one track and the fill between them. A press
    /// on the track takes the nearer knob there and a drag moves it, never
    /// past the other and kept `min_gap` apart; each knob is its own focus
    /// stop that Left and Right step. Probe: `<id>/Track/Low`,
    /// `<id>/Track/High`, `<id>/Head/Value`.
    pub(crate) fn range_slider(
        &mut self,
        id: impl Into<Id>,
        (low, high): (f32, f32),
        desc: &RangeSliderDesc,
    ) -> RangeSliderResult {
        let id = id.into();
        let key = id.as_str().to_owned();
        let t = self.theme;
        let (min, max, step) = (desc.min.min(desc.max), desc.min.max(desc.max), desc.step);
        let span = if max - min == 0.0 { 1.0 } else { max - min };
        let disabled = desc.disabled;
        let touch = self.touch_frame();
        let knob = self.by_text(if touch { 26.0 } else { 20.0 });
        let unit = if step > 0.0 { step } else { span / 20.0 };
        let gap = desc.min_gap.max(0.0);
        let track_path = format!("{}/Track", self.child_path(&key));
        let tb = self.bounds_of(&track_path);
        let g = Geo {
            h: (knob + 8.0).max(if touch { TOUCH } else { 28.0 }),
            knob,
            rail: if touch { 6.0 } else { 4.0 },
            width: tb.map_or(0.0, |b| (b.width - knob).max(1.0)),
        };
        let down = self.input().down;
        // Which knob a held press moves: 1 low, 2 high.
        let active = self.memory(&format!("{key}~knob"), 0.0) as u8;
        let clean = |v: f32| if v.is_finite() { v } else { min };
        let (a, b) = (clean(low), clean(high));
        let mut lo = a.min(b).clamp(min, max);
        let mut hi = a.max(b).clamp(min, max);
        let place = |k: u8, at: f32, lo: &mut f32, hi: &mut f32| {
            let v = snap(at, min, max, step);
            if k == 1 {
                *lo = v.clamp(min, (*hi - gap).max(min));
            } else {
                *hi = v.clamp((*lo + gap).min(max), max);
            }
        };
        if let (true, false, Some(b)) = (active != 0 && down, disabled, tb) {
            place(
                active,
                min + self.fraction_at(b, knob) * span,
                &mut lo,
                &mut hi,
            );
        }
        let show = |v: f32| desc.format.show(v, min, step);
        let label_before = self.label_before();
        self.open(
            id,
            Element {
                direction: Direction::Column,
                width: desc.width.unwrap_or(Size::grow()),
                gap: t.space.xs,
                cross: Cross::Stretch,
                ..Element::default()
            },
        );
        if !desc.label.is_empty() || desc.show_value {
            self.open(
                "Head",
                Element {
                    direction: Direction::Row,
                    width: Size::grow(),
                    gap: t.space.sm,
                    cross: Cross::Baseline,
                    ..Element::default()
                },
            );
            if desc.label.is_empty() {
                self.spacer("Lead", Some(Size::grow()));
            } else {
                self.text(
                    "Label",
                    &desc.label,
                    Text {
                        step: Some(t.type_ramp.sm),
                        color: Ink::Role(if disabled { "inkFaint" } else { "inkMuted" }),
                        width: Some(Size::grow()),
                        ..Text::default()
                    },
                );
            }
            if desc.show_value {
                self.text(
                    "Value",
                    &format!("{} – {}", show(lo), show(hi)),
                    Text {
                        step: Some(t.type_ramp.sm),
                        color: Ink::Role(if disabled { "inkFaint" } else { "ink" }),
                        nowrap: true,
                        font: FontRole::Mono,
                        ..Text::default()
                    },
                );
            }
            self.close();
        }
        let name = if desc.label.is_empty() {
            label_before.unwrap_or_else(|| key.clone())
        } else {
            desc.label.clone()
        };
        let (fl, fh) = ((lo - min) / span, (hi - min) / span);
        let sem = |which: &str, v: f32| Semantics {
            role: Some("slider".into()),
            name: Some(format!("{name}, {which}")),
            value: Some(show(v)),
            value_min: Some(min),
            value_max: Some(max),
            value_now: Some(v),
            ..Semantics::default()
        };
        let (sem_lo, sem_hi) = (sem("minimum", lo), sem("maximum", hi));
        let mut knobs: Option<(Built, Built)> = None;
        let built = self.ctl(
            "Track",
            Ctl {
                part: "Slider.reach",
                cls: ElementClass::Badge,
                disabled,
                focusable: false,
                width: Some(Size::grow()),
                height: Some(Size::px(g.h)),
                fill: None,
                wash: Some(None),
                lift: 0.0,
                press: 1.0,
                ..Ctl::default()
            },
            |ui, _| {
                ui.rail(g, fl, fh, &[], disabled);
                let held = |k: u8| active == k && down;
                let k1 = ui.knob_named(
                    "Low",
                    Some("low"),
                    Some(sem_lo),
                    fl,
                    g,
                    (held(1), false, disabled),
                );
                let k2 = ui.knob_named(
                    "High",
                    Some("high"),
                    Some(sem_hi),
                    fh,
                    g,
                    (held(2), false, disabled),
                );
                knobs = Some((k1, k2));
            },
        );
        self.close();
        let (k1, k2) = knobs.expect("the track builds its knobs");
        let mut press = if k1.r.pressed {
            1
        } else if k2.r.pressed {
            2
        } else {
            0
        };
        if !disabled {
            for (k, b, v) in [(1, &k1, lo), (2, &k2, hi)] {
                if b.r.adjust != 0 {
                    place(
                        k,
                        v + unit * self.forward(b.r.adjust) as f32,
                        &mut lo,
                        &mut hi,
                    );
                }
            }
        }
        if let (true, 0, Some(b), false) = (built.r.pressed, press, tb, disabled) {
            // A press on the track takes the nearer knob there.
            let at = min + self.fraction_at(b, knob) * span;
            press = if (at - lo).abs() <= (at - hi).abs() {
                1
            } else {
                2
            };
            place(press, at, &mut lo, &mut hi);
        }
        let held = press != 0 || (active != 0 && down);
        self.remember(
            &format!("{key}~knob"),
            f64::from(if press != 0 {
                press
            } else if held {
                active
            } else {
                0
            }),
        );
        let changed = lo != a.min(b).clamp(min, max) || hi != a.max(b).clamp(min, max);
        if changed {
            self.feedback(SoundEvent::SliderTick);
        }
        RangeSliderResult {
            low: lo,
            high: hi,
            changed,
        }
    }

    /// Labelled points under a track.
    fn slider_marks(&mut self, list: &[SliderMark], min: f32, span: f32, g: Geo) {
        let h = self.by_text(18.0);
        let t = self.theme;
        self.open(
            "Marks",
            Element {
                direction: Direction::Row,
                width: Size::grow(),
                height: Size::px(h),
                ..Element::default()
            },
        );
        for (i, m) in list.iter().enumerate() {
            let f = ((m.value - min) / span).clamp(0.0, 1.0);
            let w = (m.label.chars().count() as f32 * 8.0).max(24.0);
            self.open(
                crate::ui_idi("Mark", i as u32),
                Element {
                    width: Size::px(w),
                    align: Align::Center,
                    ..Element::default()
                },
            );
            if !m.label.is_empty() {
                self.text(
                    "Text",
                    &m.label,
                    Text {
                        step: Some(t.type_ramp.caption),
                        color: Ink::Role("inkMuted"),
                        nowrap: true,
                        align: TextAlign::Center,
                        ..Text::default()
                    },
                );
            }
            self.close();
            // At its point, centred under it; the end ones kept inside.
            let at =
                (f * g.width + g.knob / 2.0 - w / 2.0).clamp(0.0, (g.width + g.knob - w).max(0.0));
            self.over(Props {
                insets: Edges {
                    start: Some(at.round()),
                    top: Some(0.0),
                    ..Edges::default()
                },
                ..Props::default()
            });
        }
        self.close();
    }

    /// A number stepper: minus, the value and plus. A press steps once, a
    /// held press repeats after 400 ms and speeds up (large steps after a
    /// second); Left and Right step it while the value has the focus. A
    /// bound's button goes disabled at the bound. Plays sliderTick per
    /// step. Parts: `Stepper.frame`, `Stepper.button[dec|inc]`,
    /// `Stepper.value`. Role spinbutton; its − and + are buttons under a
    /// finger and named images on a fine pointer. Probe: `<id>/Frame/Dec`,
    /// `<id>/Frame/Value`, `<id>/Frame/Inc`.
    pub fn number_stepper(
        &mut self,
        id: impl Into<Id>,
        value: f32,
        desc: &NumberStepperDesc,
    ) -> NumberResult {
        let id = id.into();
        let key = id.as_str().to_owned();
        let t = self.theme;
        let (min, max) = (desc.min, desc.max);
        let step = if desc.step > 0.0 { desc.step } else { 1.0 };
        let large = if desc.large_step > 0.0 {
            desc.large_step
        } else {
            step * 10.0
        };
        let disabled = desc.disabled;
        let wrap = desc.wrap && min.is_some() && max.is_some();
        let touch = self.touch_frame();
        let h = self.sizes().control_md.max(if touch { TOUCH } else { 0.0 });
        let now = self.seconds();
        // A held button repeats: after 0.4 s, every 0.12 s, then 0.05 s; past a second by the large step.
        let since = self.memory(&format!("{key}~since"), -1.0);
        let last_rep = self.memory(&format!("{key}~rep"), -1.0);
        let fit = |x: f32| {
            let mut y = round_to(x, decimals_of(step) + 2);
            if let (true, Some(lo), Some(hi)) = (wrap, min, max) {
                if y > hi {
                    y = lo;
                } else if y < lo {
                    y = hi;
                }
            }
            if let Some(lo) = min {
                y = y.max(lo);
            }
            if let Some(hi) = max {
                y = y.min(hi);
            }
            y
        };
        let mut v = if value.is_finite() {
            value
        } else {
            min.unwrap_or(0.0)
        };
        let shown = desc.format.show(v, min.unwrap_or(0.0), step);
        let mut held = 0i32;
        let mut pressed_now = 0i32;
        let (well, ink, faint) = (self.role("well"), self.role("ink"), self.role("inkFaint"));
        // The label and the buttons on one line while they fit; the buttons
        // on a line of their own when they do not.
        self.open(
            id,
            Element {
                direction: Direction::Row,
                width: desc.width.unwrap_or_default(),
                gap: t.space.md,
                cross: Cross::Center,
                wrap: true,
                ..Element::default()
            },
        );
        if !desc.label.is_empty() {
            self.text(
                "Label",
                &desc.label,
                Text {
                    color: Ink::Role(if disabled { "inkFaint" } else { "ink" }),
                    ..Text::default()
                },
            );
            self.spacer("Gap", Some(Size::grow()));
        }
        let fpart = format!("Stepper.frame{}", flags(&[disabled.then_some("disabled")]));
        self.open(
            "Frame",
            Element {
                direction: Direction::Row,
                height: Size::px(h),
                cross: Cross::Stretch,
                radius: t.radius.control,
                background: Some(well.srgba()),
                part: crate::dress::PartRef::parse(&fpart, ElementClass::Field).ok(),
                ..Element::default()
            },
        );
        let glyph = self.by_text(18.0);
        let mut button = |ui: &mut Ui, bid: &str, sign: i32, v: f32| {
            let at_bound = !wrap
                && if sign < 0 {
                    min.is_some_and(|lo| v <= lo)
                } else {
                    max.is_some_and(|hi| v >= hi)
                };
            let part = if sign < 0 {
                "Stepper.button[dec]"
            } else {
                "Stepper.button[inc]"
            };
            let off = disabled || at_bound;
            let b = ui.ctl(
                bid,
                Ctl {
                    part,
                    disabled: off,
                    // A button under a finger is a stop keys and pads reach too.
                    focusable: touch,
                    width: Some(Size::px(h)),
                    height: Some(Size::grow()),
                    fill: None,
                    wash_by: 0.1,
                    lift: 0.0,
                    press: 0.92,
                    semantics: Some(Semantics {
                        role: Some(if touch { "button" } else { "image" }.into()),
                        name: Some(if sign < 0 { "Decrease" } else { "Increase" }.into()),
                        ..Semantics::default()
                    }),
                    ..Ctl::default()
                },
                |ui, _| {
                    ui.drawn(
                        "Glyph",
                        if sign < 0 { "minus" } else { "plus" },
                        glyph,
                        if off { faint } else { ink },
                        None,
                    )
                },
            );
            if b.r.pressed {
                held = sign;
                if since < 0.0 {
                    pressed_now = sign;
                }
            }
        };
        button(self, "Dec", -1, v);
        let name = if desc.label.is_empty() {
            key.clone()
        } else {
            desc.label.clone()
        };
        let min_w = self.by_text(56.0);
        let val = self.ctl(
            "Value",
            Ctl {
                part: "Stepper.value",
                cls: ElementClass::Field,
                disabled,
                adjustable: true,
                width: Some(Size::Fit {
                    min: min_w,
                    max: f32::INFINITY,
                }),
                height: Some(Size::grow()),
                padding: Insets::xy(t.space.sm, 0.0),
                fill: None,
                wash: Some(None),
                lift: 0.0,
                press: 1.0,
                semantics: Some(Semantics {
                    role: Some("spinbutton".into()),
                    name: Some(name),
                    value: Some(shown.clone()),
                    value_min: min,
                    value_max: max,
                    value_now: Some(v),
                    ..Semantics::default()
                }),
                ..Ctl::default()
            },
            |ui, _| {
                ui.text(
                    "Text",
                    &shown,
                    Text {
                        color: Ink::Role(if disabled { "inkFaint" } else { "ink" }),
                        nowrap: true,
                        font: FontRole::Mono,
                        align: TextAlign::Center,
                        ..Text::default()
                    },
                )
            },
        );
        button(self, "Inc", 1, v);
        self.close();
        if val.r.adjust != 0 && !disabled {
            v = fit(v + step * self.forward(val.r.adjust) as f32);
        }
        // A screen reader's increment and decrement on the spinbutton, and its set value.
        let vkey = crate::id::key_of_path(&val.path);
        if self.nav.spin.0 == vkey && !disabled {
            v = fit(v + step * self.nav.spin.1 as f32);
        }
        #[cfg(not(target_arch = "wasm32"))]
        if !disabled {
            let mut set = v;
            if self.accessibility_number(
                vkey,
                &mut set,
                min.unwrap_or(f32::MIN),
                max.unwrap_or(f32::MAX),
                step,
            ) {
                v = fit(set);
            }
        }
        self.close();
        if !disabled {
            if pressed_now != 0 {
                v = fit(v + step * pressed_now as f32);
                self.remember(&format!("{key}~since"), now);
                self.remember(&format!("{key}~rep"), now);
            } else if held != 0 && since >= 0.0 {
                let age = now - since;
                let every = if age > 1.6 { 0.05 } else { 0.12 };
                if age > 0.4 && now - last_rep >= every {
                    v = fit(v + if age > 1.0 { large } else { step } * held as f32);
                    self.remember(&format!("{key}~rep"), now);
                } else {
                    self.remember(&format!("{key}~rep"), last_rep);
                }
                self.remember(&format!("{key}~since"), since);
            } else {
                self.remember(&format!("{key}~since"), -1.0);
                self.remember(&format!("{key}~rep"), -1.0);
            }
        }
        if v != value {
            self.feedback(SoundEvent::SliderTick);
        }
        NumberResult {
            value: v,
            changed: v != value,
        }
    }
}
