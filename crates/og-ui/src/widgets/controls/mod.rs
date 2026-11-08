//! The controls (docs/ui.md §2, Controls): the switch, the checkbox, the
//! radio group, the segmented control, the slider, the number stepper and
//! the dropdown, one native set the TypeScript API, the documents and the
//! inspector all call. What they share lives here: one interactive part
//! (`ctl`) that a style dresses by its interaction state, whose motion is a
//! §7.3 state group (so a style's springs move it and the probe can force
//! each state), with its accessibility and a touch target of at least 44
//! CSS px on a coarse pointer; marks drawn as paths; the value formats the
//! slider and the stepper share; a floating layer placed beside its anchor.

use crate::a11y::Semantics;
use crate::element::{Edges, Position, Prop, Props, Transform, Values};
use crate::id::Id;
use crate::layout::{Align, Cross, Direction, Element, FontRole, Insets, Rect, Size, TextAlign};
use crate::motion::{Ease, MotionToken, StateGroup, StateTransition, Transition, VisualState};
use crate::paint::{
    Cap, ColorRef, ElementDraw, Gradient, Join, Paint, ShapeKind, Stop, Stroke, StrokeAlign,
};
use crate::style::{Breakpoint, Srgba, TypeStep};
use crate::tokens::{ElementClass, SoundEvent};
use crate::ui::{ControlSize, LabelDesc, Response, Ui};
use crate::widgets::IconDesc;

mod choices;
mod details;
mod dropdown;
mod ranges;

pub use choices::{
    CheckFieldDesc, ChoiceIndexResult, LabelSide, RadioGroupDesc, RadioOption, RadioVariant,
    SegmentItem, SegmentedDesc, SwitchResult, ToggleSwitchDesc,
};
pub use details::{DetailsDesc, DetailsResult};
pub use dropdown::{DropdownDesc, DropdownOption, DropdownResult, kept_options};
pub(crate) use ranges::RangeSliderDesc;
pub use ranges::{
    NumberResult, NumberStepperDesc, SliderMark, ValueFormat, ValueSliderDesc, ValueSliderResult,
};

/// The smallest touch target on a coarse pointer (docs/ui.md §6, XAG).
const TOUCH: f32 = 44.0;

// ── colour, as the controls mix it ──────────────────────────────────────────

/// A colour as the controls compute with it: sRGB 0–1 and alpha, mixed in
/// sRGB (a style's hover is its rest moved part of the way to the ink).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct C {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl C {
    fn of(s: Srgba) -> C {
        let [r, g, b] = s.rgb;
        C {
            r: f32::from(r) / 255.0,
            g: f32::from(g) / 255.0,
            b: f32::from(b) / 255.0,
            a: s.alpha,
        }
    }

    fn srgba(self) -> Srgba {
        let ch = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        Srgba {
            rgb: [ch(self.r), ch(self.g), ch(self.b)],
            alpha: self.a.clamp(0.0, 1.0),
        }
    }

    fn alpha(self, a: f32) -> C {
        C { a, ..self }
    }
}

/// a towards b by t (0–1), in sRGB, alpha too.
fn mix(a: C, b: C, t: f32) -> C {
    let k = t.clamp(0.0, 1.0);
    C {
        r: a.r + (b.r - a.r) * k,
        g: a.g + (b.g - a.g) * k,
        b: a.b + (b.b - a.b) * k,
        a: a.a + (b.a - a.a) * k,
    }
}

/// A colour as the four background values of a visual state.
fn bg_values(c: C) -> Values {
    vec![
        (Prop::BackgroundR, c.r),
        (Prop::BackgroundG, c.g),
        (Prop::BackgroundB, c.b),
        (Prop::BackgroundA, c.a),
    ]
}

fn vstate(name: &str, values: Values) -> VisualState {
    VisualState {
        name: name.to_owned(),
        values,
        on_enter: String::new(),
        on_exit: String::new(),
    }
}

/// A two-state group (`selection`: off, on) whose values a style's motion
/// moves between, and its current state.
fn two_state(
    name: &str,
    on: bool,
    off: Values,
    on_values: Values,
    transitions: Vec<StateTransition>,
) -> (StateGroup, (String, String)) {
    (
        StateGroup {
            name: name.to_owned(),
            states: vec![vstate("off", off), vstate("on", on_values)],
            transitions,
        },
        (name.to_owned(), if on { "on" } else { "off" }.to_owned()),
    )
}

fn tween(ms: f32, ease: Ease) -> Transition {
    Transition::Tween {
        duration_ms: ms,
        ease,
    }
}

fn spring(ms: f32, bounce: f32) -> Transition {
    Transition::Spring(crate::motion::SpringSpec::new(ms, bounce))
}

/// A selector's flags as `[a][b]`.
fn flags(f: &[Option<&str>]) -> String {
    f.iter()
        .flatten()
        .filter(|x| !x.is_empty())
        .map(|x| format!("[{x}]"))
        .collect()
}

/// A path's parent path (empty at the top).
fn parent_of(path: &str) -> &str {
    path.rfind('/').map_or("", |k| &path[..k])
}

/// What a label is drawn in: a colour role (resolved by the style) or a
/// colour the control computed.
#[derive(Clone, Copy)]
enum Ink<'a> {
    Role(&'a str),
    C(C),
}

/// A label as the controls set one.
#[derive(Clone, Copy)]
struct Text<'a> {
    step: Option<TypeStep>,
    color: Ink<'a>,
    nowrap: bool,
    ellipsis: bool,
    width: Option<Size>,
    align: TextAlign,
    font: FontRole,
    max_lines: u32,
}

impl Default for Text<'_> {
    fn default() -> Self {
        Text {
            step: None,
            color: Ink::Role("ink"),
            nowrap: false,
            ellipsis: false,
            width: None,
            align: TextAlign::Start,
            font: FontRole::Body,
            max_lines: 0,
        }
    }
}

/// An element's interaction state, as its `interaction` group says (last
/// tick; a probe-forced state included).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ix {
    Normal,
    Hover,
    Pressed,
    Focused,
    Disabled,
}

impl Ix {
    fn name(self) -> &'static str {
        match self {
            Ix::Normal => "normal",
            Ix::Hover => "hover",
            Ix::Pressed => "pressed",
            Ix::Focused => "focused",
            Ix::Disabled => "disabled",
        }
    }

    /// The flag an inner part takes from its row's state (none at rest).
    fn flag(self) -> Option<&'static str> {
        (self != Ix::Normal).then(|| self.name())
    }
}

/// One interactive part of a control (`Ui::ctl`).
struct Ctl<'a> {
    /// The part a style dresses (`Toggle.row`); the interaction state and
    /// `flags` are added as qualifiers.
    part: &'a str,
    /// The element class an undressed part takes its shape and elevation from.
    cls: ElementClass,
    flags: Vec<Option<&'a str>>,
    disabled: bool,
    /// Keys and pads reach it.
    focusable: bool,
    /// With focusable: Left and Right are its own while it has the focus.
    adjustable: bool,
    /// With focusable: the arrows it claims while it has the focus
    /// ([`crate::ARROW_UP`] …), reported as `adjust` and `adjust_y`.
    arrows: u8,
    /// Its fill at rest (moving towards `wash` as it is hovered and
    /// pressed); `None` for none.
    fill: Option<C>,
    /// What hover (half) and press (whole) move the fill towards, `wash_by`
    /// of the way; `None` keeps the fill. The ink by default.
    wash: Option<Option<C>>,
    wash_by: f32,
    direction: Direction,
    width: Option<Size>,
    height: Option<Size>,
    padding: Insets,
    gap: f32,
    align: Align,
    cross: Cross,
    radius: Option<f32>,
    semantics: Option<Semantics>,
    lift: f32,
    press: f32,
    dim: f32,
    states: Vec<StateGroup>,
    state: Vec<(String, String)>,
    /// Further element props (a placement, z).
    props: Props,
    /// The smallest side a coarse pointer gets: the control's reach grows to it.
    touch: f32,
    /// A part drawn in its state that takes no input: presses fall through
    /// to what holds it (a slider's knob).
    inert: bool,
}

impl Default for Ctl<'_> {
    fn default() -> Self {
        Ctl {
            part: "",
            cls: ElementClass::Button,
            flags: Vec::new(),
            disabled: false,
            focusable: true,
            adjustable: false,
            arrows: 0,
            fill: None,
            wash: None,
            wash_by: 0.1,
            direction: Direction::Row,
            width: None,
            height: None,
            padding: Insets::all(0.0),
            gap: 0.0,
            align: Align::Center,
            cross: Cross::Center,
            radius: None,
            semantics: None,
            lift: -1.0,
            press: 0.97,
            dim: 0.5,
            states: Vec::new(),
            state: Vec::new(),
            props: Props::default(),
            touch: TOUCH,
            inert: false,
        }
    }
}

/// What `Ui::ctl` answers.
struct Built {
    /// Its response (zero clicks, presses and adjusts while disabled).
    r: Response,
    /// Its full path.
    path: String,
    /// Its node this tick (what a floating layer is anchored to).
    node: usize,
}

impl Ui {
    /// The text kept under `key` in the open container (empty when none), taken out.
    fn take_text(&mut self, key: &str) -> String {
        let k = self.key_for(key);
        self.kept_texts.remove(&k).unwrap_or_default()
    }

    /// Keeps `text` under `key` in the open container for next tick.
    fn keep_text(&mut self, key: &str, text: String) {
        let k = self.key_for(key);
        self.kept_texts.insert(k, text);
    }

    /// The text of the label just before the next element in the open row
    /// (a settings row's "Master" before its slider), as the tree names a
    /// control without text of its own.
    fn label_before(&self) -> Option<String> {
        let parent = self.parent();
        let n = self.nodes.get(parent)?;
        if n.el.direction != Direction::Row {
            return None;
        }
        // Written again in place, the control already stands among its
        // siblings: the label before is the one just ahead of it.
        let kids = &n.built_children;
        let end = self
            .in_place
            .and_then(|(key, _)| kids.iter().position(|&c| self.nodes[c].key == key))
            .unwrap_or(kids.len());
        let &last = kids[..end].last()?;
        (self.kinds.get(last)?.0 == "label").then(|| self.nodes[last].probe_text.clone())
    }

    // ── the frame's facts ───────────────────────────────────────────────

    fn role(&self, name: &str) -> C {
        C::of(
            self.theme
                .color(name)
                .unwrap_or(Srgba::rgb(0xff, 0x00, 0xff)),
        )
    }

    /// A finger is what presses: a coarse pointer, a phone or a tablet.
    fn touch_frame(&self) -> bool {
        self.viewport.coarse_pointer
            || matches!(
                self.breakpoint(),
                Breakpoint::PhonePortrait | Breakpoint::PhoneLandscape | Breakpoint::Tablet
            )
    }

    /// A size that holds text, at the player's text scale.
    pub(crate) fn by_text(&self, h: f32) -> f32 {
        (h * self.theme.text_scale.max(1.0)).round()
    }

    pub(crate) fn still(&self) -> bool {
        self.theme.reduced_motion
    }

    fn rtl(&self) -> bool {
        self.direction() == og_i18n::Direction::Rtl
    }

    /// Response.adjust in the reading direction.
    fn forward(&self, adjust: i32) -> i32 {
        if self.rtl() { -adjust } else { adjust }
    }

    /// An element's box in the last completed frame, by path.
    fn bounds_of(&self, path: &str) -> Option<Rect> {
        if path.is_empty() {
            return None;
        }
        self.last_bounds(crate::id::key_of_path(path))
    }

    /// The path a child `id` of the open element has.
    pub(crate) fn child_path(&self, id: &str) -> String {
        let p = self.open_path();
        if p.is_empty() {
            id.to_owned()
        } else {
            format!("{p}/{id}")
        }
    }

    /// The interaction state of the element at `path` last tick.
    fn interaction_at(&self, path: &str) -> Ix {
        match self.visual_state(path, "interaction").as_deref() {
            Some("hover") => Ix::Hover,
            Some("pressed") => Ix::Pressed,
            Some("focused") => Ix::Focused,
            Some("disabled") => Ix::Disabled,
            _ => Ix::Normal,
        }
    }

    fn text(&mut self, id: impl Into<Id>, value: &str, d: Text) {
        let color = match d.color {
            Ink::Role(name) => self.theme.color(name),
            Ink::C(c) => Some(c.srgba()),
        };
        self.label(
            id,
            value,
            LabelDesc {
                step: Some(d.step.unwrap_or(self.theme.type_ramp.body)),
                color,
                align: d.align,
                nowrap: d.nowrap,
                width: d.width.unwrap_or_default(),
                font: d.font,
                ellipsis: d.ellipsis,
                max_lines: d.max_lines,
                ..LabelDesc::default()
            },
        );
    }

    /// Props for the element just built as a layer over its parent's
    /// content: absolute, declared to overlap the parent's subtree.
    fn over(&mut self, mut p: Props) {
        let last = self.last_path();
        let parent = parent_of(&last);
        p.position = Position::Absolute;
        if !parent.is_empty() {
            p.overlaps.push(parent.to_owned());
        }
        self.set_props(p);
    }

    // ── the interactive part ────────────────────────────────────────────

    /// The interaction group a control part moves by: a small lift under
    /// the pointer and the focus, a press that sinks it, a disabled part
    /// dimmed, and its fill at each (rest, hover, pressed). A press lands
    /// fast.
    fn interaction_group(&self, fills: [C; 3], lift: f32, press: f32, dim: f32) -> StateGroup {
        let still = self.still();
        let lift = if still { 0.0 } else { lift };
        let press = if still { 1.0 } else { press };
        let st = |name: &str, y: f32, s: f32, o: f32, c: C| {
            let mut v = vec![(Prop::Y, y), (Prop::Scale, s), (Prop::Opacity, o)];
            v.extend(bg_values(c));
            vstate(name, v)
        };
        StateGroup {
            name: "interaction".into(),
            states: vec![
                st("normal", 0.0, 1.0, 1.0, fills[0]),
                st("hover", lift, 1.0, 1.0, fills[1]),
                st("pressed", 0.0, press, 1.0, fills[2]),
                st("focused", lift, 1.0, 1.0, fills[1]),
                st("disabled", 0.0, 1.0, dim, fills[0]),
            ],
            // A press lands at once and lets go on the spring: a control
            // answers the finger in the tick it is touched.
            transitions: vec![StateTransition {
                from: "*".into(),
                to: "pressed".into(),
                transition: tween(
                    if still { 0.0 } else { 60.0 },
                    Ease::CubicBezier(crate::theme::EASE_OUT),
                ),
            }],
        }
    }

    /// A control's interactive part: a container keys, pads, the pointer
    /// and a finger reach, dressed as `part` with its interaction state
    /// (as its interaction group was last tick, so a probe-forced state is
    /// dressed too) and the given flags, moving between those states as a
    /// §7.3 group. On a coarse pointer its reach grows to 44 CSS px.
    fn ctl(&mut self, id: impl Into<Id>, d: Ctl, build: impl FnOnce(&mut Ui, Ix)) -> Built {
        let id = id.into();
        let own = self.child_path(id.as_str());
        let disabled = d.disabled;
        let reach = !d.inert && !disabled;
        let ix = if disabled {
            Ix::Disabled
        } else if d.inert {
            Ix::Normal
        } else {
            self.interaction_at(&own)
        };
        let ink = self.role("ink");
        let wash = d.wash.unwrap_or(Some(ink));
        let by = d.wash_by;
        let rest = d.fill.unwrap_or(wash.unwrap_or(ink).alpha(0.0));
        let tint = |k: f32| match wash {
            None => rest,
            Some(w) if rest.a > 0.0 => mix(rest, w.alpha(rest.a.max(1.0)), k * by),
            Some(w) => w.alpha(k * by * 1.2),
        };
        let fills = [rest, tint(0.5), tint(1.0)];
        let mut fl = d.flags.clone();
        fl.push(ix.flag());
        let part = format!("{}{}", d.part, flags(&fl));
        let touch = self.touch_frame();
        let radius = d.radius.unwrap_or(self.theme.radius.control);
        self.open(
            id,
            Element {
                direction: d.direction,
                width: d.width.unwrap_or_default(),
                // A row that fits its content is at least a finger's height
                // on a coarse pointer.
                height: d.height.unwrap_or(if touch {
                    Size::Fit {
                        min: d.touch,
                        max: f32::INFINITY,
                    }
                } else {
                    Size::fit()
                }),
                padding: d.padding,
                gap: d.gap,
                align: d.align,
                cross: d.cross,
                interactive: reach,
                focusable: reach && d.focusable,
                adjustable: !disabled && d.adjustable,
                arrows: if disabled { 0 } else { d.arrows },
                // A transparent rest still has a fill for the states to move.
                background: Some(rest.srgba()),
                radius,
                part: crate::dress::PartRef::parse(&part, d.cls).ok(),
                ..Element::default()
            },
        );
        build(self, ix);
        let r = self.close();
        let path = self.last_path();
        let node = self.last_closed;
        let want = if touch { d.touch } else { 0.0 };
        let slop = if want > 0.0 {
            self.bounds_of(&path).map_or(0.0, |b| {
                ((want - b.width.min(b.height)) / 2.0).ceil().max(0.0)
            })
        } else {
            0.0
        };
        let mut props = d.props;
        if slop > 0.0 {
            props.hit_slop = props.hit_slop.max(slop);
        }
        let mut states = vec![self.interaction_group(fills, d.lift, d.press, d.dim)];
        states.extend(d.states);
        states.append(&mut props.states);
        props.states = states;
        let mut state = d.state;
        state.append(&mut props.state);
        props.state = state;
        self.set_props(props);
        if d.semantics.is_some() || disabled {
            let mut s = d.semantics.unwrap_or_default();
            s.disabled = disabled;
            self.semantics(&s);
        }
        let r = if disabled {
            Response {
                clicked: false,
                pressed: false,
                adjust: 0,
                ..r
            }
        } else {
            r
        };
        Built { r, path, node }
    }

    // ── marks ───────────────────────────────────────────────────────────

    /// A line through `points` (fractions of a `size` square), as a mark.
    fn line_shape(points: &[[f32; 2]], size: f32, weight: f32, closed: bool) -> ShapeKind {
        ShapeKind::Path {
            points: points.iter().map(|p| [p[0] * size, p[1] * size]).collect(),
            closed,
            filled: false,
            fill_rule: Default::default(),
            contours: Vec::new(),
            curves: false,
            width: weight,
            cap: Cap::Round,
            join: Join::Round,
        }
    }

    fn solid(c: C) -> Vec<Paint> {
        vec![Paint::Solid(ColorRef::Literal(c.srgba()))]
    }

    /// A mark drawn as a path in a `size` square: a tick, a minus (or
    /// dash), a plus, a cross, a warning or a lens; other names a dot.
    fn drawn(&mut self, id: impl Into<Id>, name: &str, size: f32, color: C, weight: Option<f32>) {
        let s = size;
        let w = weight.unwrap_or((s / 11.0).max(1.5));
        let fill = Self::solid(color);
        let line = |pts: &[[f32; 2]], closed: bool| Self::line_shape(pts, s, w, closed);
        self.open(
            id,
            Element {
                width: Size::px(s),
                height: Size::px(s),
                ..Element::default()
            },
        );
        let draw = |shape: ShapeKind| ElementDraw {
            shape,
            fills: fill.clone(),
            ..ElementDraw::default()
        };
        let layer = |ui: &mut Ui, id: &str, shape: ShapeKind, size: f32, at: [f32; 2]| {
            ui.open(
                id,
                Element {
                    width: Size::px(size),
                    height: Size::px(size),
                    ..Element::default()
                },
            );
            ui.set_draw(draw(shape));
            ui.close();
            ui.set_props(Props {
                position: Position::Absolute,
                insets: Edges {
                    start: Some(at[0]),
                    top: Some(at[1]),
                    ..Edges::default()
                },
                ..Props::default()
            });
        };
        match name {
            "tick" => self.set_draw(draw(line(&[[0.2, 0.53], [0.42, 0.74], [0.8, 0.28]], false))),
            "cross" => self.set_draw(draw(line(
                &[
                    [0.27, 0.27],
                    [0.73, 0.73],
                    [0.5, 0.5],
                    [0.73, 0.27],
                    [0.27, 0.73],
                ],
                false,
            ))),
            "plus" => self.set_draw(draw(line(
                &[
                    [0.5, 0.22],
                    [0.5, 0.78],
                    [0.5, 0.5],
                    [0.22, 0.5],
                    [0.78, 0.5],
                ],
                false,
            ))),
            "minus" | "dash" => self.set_draw(draw(line(&[[0.24, 0.5], [0.76, 0.5]], false))),
            "search" => {
                layer(
                    self,
                    "Lens",
                    ShapeKind::Arc {
                        start: 0.0,
                        sweep: std::f32::consts::TAU,
                        thickness: w,
                        cap: Cap::default(),
                    },
                    s * 0.62,
                    [s * 0.1, s * 0.1],
                );
                layer(
                    self,
                    "Handle",
                    line(&[[0.64, 0.64], [0.86, 0.86]], false),
                    s,
                    [0.0, 0.0],
                );
            }
            "warning" => {
                self.set_draw(draw(line(&[[0.5, 0.12], [0.9, 0.84], [0.1, 0.84]], true)));
                layer(
                    self,
                    "Bang",
                    line(
                        &[
                            [0.5, 0.4],
                            [0.5, 0.58],
                            [0.5, 0.58],
                            [0.5, 0.72],
                            [0.5, 0.7201],
                        ],
                        false,
                    ),
                    s,
                    [0.0, 0.0],
                );
            }
            _ => self.set_draw(draw(ShapeKind::Ellipse)),
        }
        self.close();
    }

    /// The marks the controls draw themselves.
    fn has_mark(name: &str) -> bool {
        matches!(
            name,
            "tick" | "cross" | "plus" | "minus" | "dash" | "search" | "warning" | "dot"
        )
    }

    /// An icon of the style's set when it has it, else a mark drawn as a
    /// path (never a missing glyph).
    fn glyph_icon(&mut self, id: impl Into<Id>, name: &str, size: f32, color: C) {
        if !self.has_icon(name) && Self::has_mark(name) {
            self.drawn(id, name, size, color, None);
            return;
        }
        self.icon(
            id,
            name,
            IconDesc {
                size,
                tint: Some(color.srgba()),
                label: Some(String::new()),
                ..IconDesc::default()
            },
        );
    }

    /// A chevron drawn as a path pointing down, turned half a turn by the
    /// `expansion` group while expanded (on the style's motion).
    fn chevron_down(
        &mut self,
        id: impl Into<Id>,
        size: f32,
        color: C,
        expanded: bool,
        weight: f32,
    ) {
        self.chevron(id, size, color, expanded, weight, (0.0, 2.0));
    }

    /// A chevron turned `turns.0` quarter turns from down while collapsed
    /// and `turns.1` while expanded (-1: towards the right), moving between
    /// them as the `expansion` group.
    fn chevron(
        &mut self,
        id: impl Into<Id>,
        size: f32,
        color: C,
        expanded: bool,
        weight: f32,
        turns: (f32, f32),
    ) {
        let s = size;
        self.open(
            id,
            Element {
                width: Size::px(s),
                height: Size::px(s),
                ..Element::default()
            },
        );
        self.set_draw(ElementDraw {
            shape: ShapeKind::Path {
                points: vec![
                    [s * 0.38, s * 0.22],
                    [s * 0.66, s * 0.5],
                    [s * 0.38, s * 0.78],
                ],
                closed: false,
                filled: false,
                fill_rule: Default::default(),
                contours: Vec::new(),
                curves: false,
                width: weight,
                cap: Cap::Round,
                join: Join::Round,
            },
            fills: Self::solid(color),
            ..ElementDraw::default()
        });
        self.close();
        let rad = |q: f32| (90.0 + 90.0 * q).to_radians();
        let turn = |name: &str, q: f32| vstate(name, vec![(Prop::Rotate, rad(q))]);
        self.set_props(Props {
            transform: Transform {
                rotate: rad(if expanded { turns.1 } else { turns.0 }),
                ..Transform::IDENTITY
            },
            states: vec![StateGroup {
                name: "expansion".into(),
                states: vec![turn("collapsed", turns.0), turn("expanded", turns.1)],
                transitions: Vec::new(),
            }],
            state: vec![(
                "expansion".into(),
                if expanded { "expanded" } else { "collapsed" }.into(),
            )],
            ..Props::default()
        });
    }

    /// An edge fade over the parent's start or end (a scrolled strip's cut
    /// edge), `k` of the way in.
    fn edge_fade(&mut self, id: &str, start: bool, k: f32, size: f32, ink: &str) {
        if k < 0.01 {
            return;
        }
        self.open(
            id,
            Element {
                width: Size::px(size),
                height: Size::grow(),
                ..Element::default()
            },
        );
        let outer = start != self.rtl();
        let tok = |a: f32| ColorRef::Token {
            name: ink.to_owned(),
            alpha: a,
        };
        self.set_draw(ElementDraw {
            fills: vec![Paint::Gradient(Gradient {
                from: [if outer { 0.0 } else { 1.0 }, 0.5],
                to: [if outer { 1.0 } else { 0.0 }, 0.5],
                stops: vec![
                    Stop {
                        offset: 0.0,
                        color: tok(k),
                    },
                    Stop {
                        offset: 1.0,
                        color: tok(0.0),
                    },
                ],
                ..Gradient::default()
            })],
            ..ElementDraw::default()
        });
        self.close();
        self.over(Props {
            insets: Edges {
                start: start.then_some(0.0),
                end: (!start).then_some(0.0),
                top: Some(0.0),
                bottom: Some(0.0),
                ..Edges::default()
            },
            inert: true,
            z: 3,
            ..Props::default()
        });
    }

    /// A ring over the parent's box (a chosen card's).
    fn chosen_ring(&mut self, id: &str, width: f32, ink: &str, part: &str) {
        self.open(
            id,
            Element {
                width: Size::grow(),
                height: Size::grow(),
                part: crate::dress::PartRef::parse(part, None::<ElementClass>).ok(),
                ..Element::default()
            },
        );
        self.set_draw(ElementDraw {
            shape: ShapeKind::default(),
            strokes: vec![Self::stroke(width, ink)],
            ..ElementDraw::default()
        });
        self.close();
        self.over(Props {
            insets: Edges {
                start: Some(0.0),
                top: Some(0.0),
                end: Some(0.0),
                bottom: Some(0.0),
                ..Edges::default()
            },
            inert: true,
            z: 3,
            ..Props::default()
        });
    }

    /// A stroke inside an element's edge in a colour role.
    fn stroke(width: f32, token: &str) -> Stroke {
        Stroke {
            width,
            align: StrokeAlign::Inside,
            dash: [0.0; 2],
            dash_offset: 0.0,
            dash_cap: Cap::default(),
            paint: Paint::Solid(ColorRef::token(token)),
        }
    }

    // ── the label and helper around a field ─────────────────────────────

    /// The label above a control and its helper or error line below.
    fn field_chrome(
        &mut self,
        id: impl Into<Id>,
        label: &str,
        helper: &str,
        error: &str,
        width: Size,
        build: impl FnOnce(&mut Ui),
    ) {
        let t = self.theme;
        self.open(
            id,
            Element {
                direction: Direction::Column,
                width,
                gap: t.space.xs,
                cross: Cross::Stretch,
                ..Element::default()
            },
        );
        if !label.is_empty() {
            self.open(
                "Head",
                Element {
                    direction: Direction::Row,
                    width: Size::grow(),
                    gap: t.space.xs,
                    cross: Cross::Baseline,
                    ..Element::default()
                },
            );
            self.text(
                "Label",
                label,
                Text {
                    step: Some(t.type_ramp.sm),
                    color: Ink::Role("inkMuted"),
                    nowrap: true,
                    ellipsis: true,
                    width: Some(Size::grow()),
                    ..Text::default()
                },
            );
            self.close();
        }
        build(self);
        if !error.is_empty() || !helper.is_empty() {
            self.open(
                "Foot",
                Element {
                    direction: Direction::Row,
                    width: Size::grow(),
                    gap: t.space.sm,
                    cross: Cross::Start,
                    ..Element::default()
                },
            );
            if !error.is_empty() {
                let danger = self.role("danger");
                self.drawn("ErrorIcon", "warning", 14.0, danger, None);
                self.text(
                    "Error",
                    error,
                    Text {
                        step: Some(t.type_ramp.caption),
                        color: Ink::Role("danger"),
                        width: Some(Size::grow()),
                        ..Text::default()
                    },
                );
                self.semantics(&Semantics {
                    role: Some("alert".into()),
                    name: Some(error.to_owned()),
                    ..Semantics::default()
                });
            } else {
                self.text(
                    "Helper",
                    helper,
                    Text {
                        step: Some(t.type_ramp.caption),
                        color: Ink::Role("inkMuted"),
                        width: Some(Size::grow()),
                        ..Text::default()
                    },
                );
            }
            self.close();
        }
        self.close();
    }

    // ── floating layers ─────────────────────────────────────────────────

    /// A floating surface on node `anchor` (a dropdown's list): a popover
    /// layer, its path `id`, under the anchor (above it where there is more
    /// room, kept inside the safe area; og-ui's `Float::Popover`), built from
    /// `content` and `build`. A focus scope: Back and a press outside it and
    /// its anchor dismiss it, and the focus goes back where it was when it
    /// closes. It fades and drops in as it opens.
    pub(crate) fn floating(
        &mut self,
        id: &str,
        open: bool,
        anchor: usize,
        content: Element,
        build: impl FnOnce(&mut Ui),
    ) -> crate::widgets::ModalResponse {
        let key = crate::id::child_key(self.nodes[0].key, &id.replace('/', "_"));
        let mut dismissed = false;
        if open {
            // Last tick's hits: what the press landed on.
            if self.input.pressed {
                let hot = self.state.hot;
                let at = self.input.pointer;
                // A press on what takes no input inside it (a disabled
                // option, its padding) is inside it too.
                let inside = hot == self.nodes[anchor].key
                    || self
                        .hit(hot)
                        .is_some_and(|h| h.scope == key || h.key == key)
                    || self
                        .last_bounds(key)
                        .is_some_and(|r| r.contains(at.x, at.y));
                dismissed |= !inside;
            }
            if self.state.scope == key && self.nav.back {
                self.nav.back = false;
                dismissed = true;
            }
        }
        let visible = open && !dismissed;
        let k = self.animate_key(
            crate::id::child_key(key, "~open"),
            if visible { 1.0 } else { 0.0 },
            tween(140.0, Ease::CubicBezier(crate::theme::EASE_OUT)),
            Some(0.0),
        );
        let k = if self.still() || !visible { 1.0 } else { k };
        if visible {
            let layer = self.open_layer(
                "menu",
                Id::from(id),
                content,
                crate::layout::Float::Popover,
                anchor,
            );
            self.nodes[layer].scope = true;
            build(self);
            self.close();
            self.set_props(Props {
                transform: Transform {
                    y: (1.0 - k) * -6.0,
                    ..Transform::IDENTITY
                },
                opacity: (k * 1.6).min(1.0),
                ..Props::default()
            });
        }
        crate::widgets::ModalResponse { visible, dismissed }
    }
}

// ── numbers ─────────────────────────────────────────────────────────────────

/// How many decimals a step needs: 1 → 0, 0.5 → 1, 0.05 → 2; continuous (0) shows 2.
fn decimals_of(step: f32) -> usize {
    if step.is_nan() || step <= 0.0 {
        return 2;
    }
    let s = f64::from(step);
    for k in 0..=6 {
        let m = s * 10f64.powi(k);
        if (m.round() - m).abs() < 1e-6 {
            return k as usize;
        }
    }
    6
}

/// `v` rounded to `dec` decimals as a number (a step of 0.1 lands on 0.3,
/// not 0.30000000000000004).
fn round_to(v: f32, dec: usize) -> f32 {
    let m = 10f64.powi(dec.min(10) as i32);
    ((f64::from(v) * m).round() / m) as f32
}

/// `value` snapped to `step` from `min` and kept in range.
fn snap(value: f32, min: f32, max: f32, step: f32) -> f32 {
    let lo = min.min(max);
    let hi = min.max(max);
    let mut v = value.clamp(lo, hi);
    if step > 0.0 {
        v = (lo + ((v - lo) / step).round() * step).clamp(lo, hi);
    }
    round_to(v, decimals_of(step) + 2)
}

/// A number as a control shows it: its decimals by the step (0.25 → 2),
/// grouped, a true minus, a unit after.
fn format_value(
    value: f32,
    decimals: Option<usize>,
    step: f32,
    unit: &str,
    percent: bool,
) -> String {
    if !value.is_finite() {
        return "—".into();
    }
    if percent {
        return format!("{}%", (f64::from(value) * 100.0).round() as i64);
    }
    let dec = decimals.unwrap_or_else(|| decimals_of(step));
    let fixed = format!("{:.*}", dec, f64::from(value).abs());
    let (int, frac) = fixed.split_once('.').unwrap_or((&fixed, ""));
    let mut grouped = String::new();
    for (k, ch) in int.chars().enumerate() {
        if k > 0 && (int.len() - k) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    let neg = value < 0.0 && fixed.parse::<f64>().is_ok_and(|v| v != 0.0);
    format!(
        "{}{grouped}{}{unit}",
        if neg { "\u{2212}" } else { "" },
        if frac.is_empty() {
            String::new()
        } else {
            format!(".{frac}")
        }
    )
}

/// The next enabled index from `from` by `dir` (wrapping, or staying at an
/// end), or `from` when none is.
fn step_enabled(
    count: usize,
    from: i32,
    dir: i32,
    disabled: impl Fn(usize) -> bool,
    wrap: bool,
) -> i32 {
    if count == 0 {
        return -1;
    }
    let n = count as i32;
    let mut i = from;
    for _ in 0..count {
        i += dir.signum();
        if i < 0 || i >= n {
            if !wrap {
                return from;
            }
            i = (i + n).rem_euclid(n);
        }
        if !disabled(i as usize) {
            return i;
        }
    }
    from
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_read_as_a_control_shows_them() {
        assert_eq!(format_value(1234.5, None, 0.5, "", false), "1,234.5");
        assert_eq!(format_value(-0.001, Some(1), 0.0, "", false), "0.0");
        assert_eq!(format_value(-3.0, None, 1.0, " m", false), "\u{2212}3 m");
        assert_eq!(format_value(0.25, None, 0.0, "", true), "25%");
        assert_eq!(snap(0.1 * 3.0, 0.0, 1.0, 0.1), 0.3);
        assert_eq!(snap(7.4, 0.0, 10.0, 5.0), 5.0);
        assert_eq!(decimals_of(0.05), 2);
    }

    #[test]
    fn arrows_skip_disabled_choices() {
        let off = |i: usize| i == 1;
        assert_eq!(step_enabled(4, 0, 1, off, true), 2);
        assert_eq!(step_enabled(4, 3, 1, off, true), 0);
        assert_eq!(step_enabled(4, 3, 1, off, false), 3);
        assert_eq!(step_enabled(4, 2, -1, off, false), 0);
    }
}
