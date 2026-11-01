//! Documents (docs/ui.md §10): a screen (`type screen`) or a component
//! (`type component`) as `.ogt` text, an element tree with props, slots,
//! parts, state groups, transitions, timelines, breakpoint overrides,
//! list templates, conditions, triggers and bindings. [`UiDocument::parse`]
//! reads and validates one against the reflected props ([`schema`]); og-cook
//! cooks it into an `og.UiDocument` blob (the validated text); a [`crate::Ui`]
//! mounts it ([`crate::Ui::mount`], `build.rs`) every tick, mixing with
//! immediate-mode code: slots filled by code, events back to code.
//!
//! The grammar is the README's §Documents; [`UiDocument::set_prop`] edits
//! one prop of one element in the text, keeping every other line (what
//! `ui:save` writes back).

pub mod build;
pub mod schema;

use crate::bind::{Expr, Format, Template};
use crate::element::Prop;
use crate::layout::Size;
use crate::motion::{
    Ease, EventKind, Key, KeyEase, MotionToken, PresenceSpec, SpringSpec, StateTransition,
    Timeline, TimelineEvent, TimelineTrack, Transition, TransitionKind,
};
use crate::style::Srgba;
use og_core::ogt::{Document, Entry, Number, Value};
use schema::{PROPS, Ty};

/// The `.ogt` types of documents.
pub const SCREEN_TYPE: &str = "screen";
pub const COMPONENT_TYPE: &str = "component";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocKind {
    Screen,
    Component,
}

/// The kinds of layer a screen declares (docs/ui.md §8.1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum LayerKind {
    Hud,
    #[default]
    Menu,
    Modal,
    Popover,
    Toast,
    Tooltip,
    System,
}

impl LayerKind {
    pub const NAMES: [&'static str; 7] = [
        "hud", "menu", "modal", "popover", "toast", "tooltip", "system",
    ];
    pub fn name(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    pub fn from_name(n: &str) -> Option<LayerKind> {
        use LayerKind::*;
        [Hud, Menu, Modal, Popover, Toast, Tooltip, System]
            .into_iter()
            .find(|k| k.name() == n)
    }
}

/// Where a layer's input goes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputMode {
    /// Input reaches gameplay; the layer takes only what lands on its controls (a HUD).
    Game,
    /// Input stays in the UI: gameplay sees none, focus stays in the layer.
    #[default]
    Menu,
    /// The UI takes focus and navigation, and gameplay still sees input.
    All,
}

impl InputMode {
    pub fn name(self) -> &'static str {
        match self {
            InputMode::Game => "game",
            InputMode::Menu => "menu",
            InputMode::All => "all",
        }
    }
}

/// What happens to the layers under one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Under {
    #[default]
    None,
    /// A scrim over them.
    Dim,
    /// A backdrop blur (the UI pass's; until it lands, a scrim).
    Blur,
    /// Not built.
    Hide,
}

impl Under {
    pub fn name(self) -> &'static str {
        match self {
            Under::None => "none",
            Under::Dim => "dim",
            Under::Blur => "blur",
            Under::Hide => "hide",
        }
    }
}

/// What Back does on a layer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Back {
    /// Pops it.
    #[default]
    Pop,
    /// Nothing (a root screen, a HUD).
    None,
    /// Emits a named event to code (`back emit confirmQuit`).
    Emit(String),
}

/// A screen's layer declaration.
#[derive(Clone, Debug, PartialEq)]
pub struct LayerSpec {
    pub kind: LayerKind,
    pub input: InputMode,
    /// It captures the pointer (gameplay never sees it, even on its gaps).
    pub capture: bool,
    pub pause: bool,
    pub under: Under,
    pub back: Back,
    /// The element focus starts on, a path below the root (empty: the first control).
    pub focus: String,
    /// Focus returns to where it was when the layer is on top again.
    pub remember: bool,
    pub enter: Option<PresenceSpec>,
    pub exit: Option<PresenceSpec>,
}

impl LayerSpec {
    /// The kind's defaults.
    pub fn of(kind: LayerKind) -> LayerSpec {
        let menu = !matches!(kind, LayerKind::Hud | LayerKind::Toast | LayerKind::Tooltip);
        LayerSpec {
            kind,
            input: match kind {
                LayerKind::Hud | LayerKind::Toast | LayerKind::Tooltip => InputMode::Game,
                LayerKind::System => InputMode::All,
                _ => InputMode::Menu,
            },
            capture: matches!(kind, LayerKind::Menu | LayerKind::Modal),
            pause: false,
            under: if matches!(kind, LayerKind::Menu | LayerKind::Modal) {
                Under::Dim
            } else {
                Under::None
            },
            back: if menu { Back::Pop } else { Back::None },
            focus: String::new(),
            remember: true,
            enter: None,
            exit: None,
        }
    }
}

/// A colour a document names.
#[derive(Clone, Debug, PartialEq)]
pub enum Paint {
    /// A palette role, resolved through the style when drawn.
    Token(String),
    Color(Srgba),
}

/// A prop's literal value, typed by its [`Ty`].
#[derive(Clone, Debug, PartialEq)]
pub enum Lit {
    Num(f32),
    Nums(Vec<f32>),
    Bool(bool),
    Str(String),
    Size(Size),
    Paint(Paint),
    Transition(Transition),
    Presence(PresenceSpec),
    Tracks(Vec<crate::element::Track>),
    Strs(Vec<String>),
}

/// A prop's value in a document.
#[derive(Clone, Debug, PartialEq)]
pub enum PropValue {
    Lit(Lit),
    /// `bind <expr> [format [digits]]`.
    Bind(Expr, Option<Format>),
    /// A text with arguments.
    Template(Template),
}

/// One `prop value` line.
#[derive(Clone, Debug, PartialEq)]
pub struct PropSet {
    /// Its index in [`schema::PROPS`].
    pub spec: usize,
    pub value: PropValue,
    /// The source line (1-based).
    pub line: usize,
    /// The value as written (for the dump and the write-back).
    pub source: String,
}

impl PropSet {
    pub fn name(&self) -> &'static str {
        PROPS[self.spec].name
    }
}

/// When an `at` block applies.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AtQuery {
    pub breakpoints: Vec<crate::style::Breakpoint>,
    pub orientation: Option<crate::element::Orientation>,
}

/// A state value: an animatable prop or a colour.
#[derive(Clone, Debug, PartialEq)]
pub enum StateValue {
    Prop(Prop, f32),
    Background(Paint),
    Ink(Paint),
}

/// One state of a document's group.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DocState {
    pub name: String,
    pub values: Vec<StateValue>,
    pub on_enter: String,
    pub on_exit: String,
}

/// A state group: its states, transitions, and the binding that picks
/// its state (a bool takes the second state when true, a string the state
/// of that name).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DocGroup {
    pub name: String,
    pub bind: Option<Expr>,
    pub states: Vec<DocState>,
    pub transitions: Vec<StateTransition>,
}

/// What a trigger does.
#[derive(Clone, Debug, PartialEq)]
pub enum Act {
    /// Plays a timeline on the element.
    Play(String),
    /// Sets one of its state groups.
    State(String, String),
    /// An event to code, by name.
    Emit(String),
    /// Pushes a screen.
    Push(String),
    /// Pops the top layer.
    Pop,
    /// Fires a UI action (`back`, `confirm`, a game's own).
    Action(String),
    /// Writes a value into the viewmodel (`set selection item.id`).
    Set(String, Expr),
}

/// What starts a trigger.
#[derive(Clone, Debug, PartialEq)]
pub enum When {
    /// The element was clicked (or activated by keys and pads).
    Click,
    /// A control's value changed.
    Changed,
    /// It appeared this tick.
    Appear,
    /// A binding's value changed since the last tick.
    Change(Expr),
    /// A UI action fired while its layer is on top (`on action details push Details`).
    Action(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Trigger {
    pub when: When,
    pub act: Act,
    pub line: usize,
}

/// What a node of the tree is.
#[derive(Clone, Debug, PartialEq)]
pub enum NodeKind {
    /// An element of a [`schema::KINDS`] kind.
    Element(&'static str),
    /// `for <var> in <list> { template }`.
    For { var: String, list: Expr },
    /// `if <expr> { … }`.
    If(Expr),
    /// `use <Component> <id> { args }`.
    Use(String),
}

/// One node of a document's tree.
#[derive(Clone, Debug, PartialEq)]
pub struct DocNode {
    pub kind: NodeKind,
    /// Its element id (empty: anonymous).
    pub id: String,
    pub line: usize,
    pub props: Vec<PropSet>,
    /// Breakpoint and orientation overrides, in order (later wins).
    pub at: Vec<(AtQuery, Vec<PropSet>)>,
    pub groups: Vec<DocGroup>,
    pub triggers: Vec<Trigger>,
    /// A component instance's arguments (`use`): its props by name.
    pub args: Vec<(String, PropValue, usize)>,
    pub children: Vec<usize>,
    pub parent: usize,
}

/// A prop of the document's contract (`props { gold number "…" }`).
#[derive(Clone, Debug, PartialEq)]
pub struct ContractProp {
    pub name: String,
    pub ty: String,
    pub doc: String,
}

/// A parsed, validated document.
#[derive(Clone, Debug, PartialEq)]
pub struct UiDocument {
    pub kind: DocKind,
    pub name: String,
    /// The asset id it was read from (`ui/shop.ogt`; empty: in memory).
    pub source: String,
    /// The text as written.
    pub text: String,
    pub layer: LayerSpec,
    pub props: Vec<ContractProp>,
    pub parts: Vec<String>,
    /// The action bar's hints: an action and what it does here.
    pub actions: Vec<(String, String)>,
    /// Timelines by their own names (the UI defines them as `<doc>/<name>`).
    pub timelines: Vec<(String, Timeline)>,
    /// Viewmodels to preview and push it with (`ui:push:shop:rich`).
    pub fixtures: Vec<(String, serde_json::Value)>,
    pub nodes: Vec<DocNode>,
    pub root: usize,
}

pub(crate) const NONE: usize = usize::MAX;

fn words(e: &Entry) -> Vec<String> {
    e.values
        .iter()
        .map(|v| match v {
            Value::Number(n) => n.text().to_string(),
            Value::String(s) | Value::Word(s) => s.clone(),
        })
        .collect()
}

/// Values as written (for the dump and the write-back).
fn written(vals: &[Value]) -> String {
    vals.iter()
        .map(|v| match v {
            Value::Number(n) => n.text().to_string(),
            Value::Word(w) => w.clone(),
            Value::String(s) => quote(s),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A string as `.ogt` writes it.
pub(crate) fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn as_name(v: &Value) -> Option<String> {
    match v {
        Value::String(s) | Value::Word(s) => Some(s.clone()),
        Value::Number(n) => Some(n.text().to_string()),
    }
}

fn nums(vals: &[Value], n: usize, what: &str) -> Result<Vec<f32>, String> {
    if vals.len() != n {
        return Err(format!("takes {n} number(s) ({what})"));
    }
    vals.iter()
        .map(|v| {
            v.as_number()
                .map(Number::as_f32)
                .ok_or_else(|| format!("takes numbers ({what})"))
        })
        .collect()
}

pub(crate) fn parse_transition(vals: &[Value]) -> Result<Transition, String> {
    let w: Vec<String> = vals.iter().filter_map(as_name).collect();
    let num = |i: usize| -> Result<f32, String> {
        vals.get(i)
            .and_then(Value::as_number)
            .map(Number::as_f32)
            .ok_or_else(|| {
                "a transition: spring <ms> <bounce>, tween <ms> [ease], token <name> or instant"
                    .to_string()
            })
    };
    match w.first().map(String::as_str) {
        Some("instant") => Ok(Transition::Instant),
        Some("spring") => Ok(Transition::Spring(SpringSpec::new(
            num(1)?,
            if vals.len() > 2 { num(2)? } else { 0.0 },
        ))),
        Some("tween") => Ok(Transition::Tween {
            duration_ms: num(1)?,
            ease: match w.get(2) {
                Some(e) => parse_ease(e)?,
                None => Ease::Out,
            },
        }),
        Some("token") => {
            let n = w.get(1).ok_or("token <ui|panel|emphasis|playful|enter|exit>")?;
            MotionToken::from_name(n)
                .map(Transition::Token)
                .ok_or_else(|| format!("'{n}' is not a motion token: ui, panel, emphasis, playful, enter, exit"))
        }
        Some(t) => MotionToken::from_name(t)
            .map(Transition::Token)
            .ok_or_else(|| format!("'{t}' is not a transition: spring <ms> <bounce>, tween <ms> [ease], token <name> or instant")),
        None => Err("a transition: spring <ms> <bounce>, tween <ms> [ease], token <name> or instant".into()),
    }
}

fn parse_ease(e: &str) -> Result<Ease, String> {
    Ok(match e {
        "linear" => Ease::Linear,
        "out" => Ease::Out,
        "exit" => Ease::Exit,
        "inOut" => Ease::InOut,
        other => {
            return Err(format!(
                "'{other}' is not an ease: linear, out, exit, inOut"
            ));
        }
    })
}

fn parse_presence(vals: &[Value], enter: bool) -> Result<PresenceSpec, String> {
    let kind = vals.first().and_then(Value::as_word).unwrap_or("");
    let k = match kind {
        "fade" => TransitionKind::Fade,
        "slideUp" => TransitionKind::SlideUp,
        "slideDown" => TransitionKind::SlideDown,
        "slideLeft" => TransitionKind::SlideLeft,
        "slideRight" => TransitionKind::SlideRight,
        "scale" => TransitionKind::Scale,
        "pop" => TransitionKind::Pop,
        other => {
            return Err(format!(
                "'{other}' is not a presence: fade, slideUp, slideDown, slideLeft, slideRight, scale, pop"
            ));
        }
    };
    let mut rest = &vals[1..];
    let mut distance = 24.0;
    if let Some(n) = rest.first().and_then(Value::as_number) {
        distance = n.as_f32();
        rest = &rest[1..];
    }
    let t = if rest.is_empty() {
        Transition::Token(if enter {
            MotionToken::Enter
        } else {
            MotionToken::Exit
        })
    } else {
        parse_transition(rest)?
    };
    Ok(PresenceSpec::from_kind(k, distance, t))
}

fn parse_size(vals: &[Value]) -> Result<Size, String> {
    const HOW: &str =
        "a size: fit [min max], grow [min max], fixed <px> (or a number), percent <0–1>";
    if let [Value::Number(n)] = vals {
        return Ok(Size::Fixed(n.as_f32()));
    }
    let w = vals.first().and_then(Value::as_word).ok_or(HOW)?;
    let n = |i: usize| vals.get(i).and_then(Value::as_number).map(Number::as_f32);
    let range = || (n(1).unwrap_or(0.0), n(2).unwrap_or(f32::INFINITY));
    Ok(match w {
        "fit" => {
            let (min, max) = range();
            Size::Fit { min, max }
        }
        "grow" => {
            let (min, max) = range();
            Size::Grow { min, max }
        }
        "fixed" => Size::Fixed(n(1).ok_or(HOW)?),
        "percent" => Size::Percent(n(1).ok_or(HOW)?),
        _ => return Err(HOW.into()),
    })
}

fn parse_paint(vals: &[Value]) -> Result<Paint, String> {
    const HOW: &str = "a paint: token <role> (a palette role) or \"#rrggbb[aa]\"";
    match vals {
        [Value::Word(t), Value::Word(role)] if t == "token" => {
            if crate::Palette::NAMES.contains(&role.as_str()) {
                Ok(Paint::Token(role.clone()))
            } else {
                Err(format!(
                    "'{role}' is not a palette role: {}",
                    crate::Palette::NAMES.join(", ")
                ))
            }
        }
        [Value::String(s)] | [Value::Word(s)] if s.starts_with('#') => {
            crate::style_file::parse_color(s)
                .map(Paint::Color)
                .ok_or_else(|| format!("'{s}' is not a colour (#rrggbb or #rrggbbaa)"))
        }
        [Value::Word(role)] | [Value::String(role)]
            if crate::Palette::NAMES.contains(&role.as_str()) =>
        {
            Ok(Paint::Token(role.clone()))
        }
        _ => Err(HOW.into()),
    }
}

fn parse_track(v: &Value) -> Result<crate::element::Track, String> {
    use crate::element::Track;
    let s = match v {
        Value::Number(n) => return Ok(Track::Px(n.as_f32())),
        Value::Word(w) | Value::String(w) => w.as_str(),
    };
    if s == "auto" {
        return Ok(Track::Auto);
    }
    if let Some(f) = s.strip_suffix("fr") {
        return f
            .parse()
            .map(Track::Fr)
            .map_err(|_| format!("'{s}' is not a track"));
    }
    if let Some(p) = s.strip_suffix('%') {
        return p
            .parse::<f32>()
            .map(|p| Track::Percent(p / 100.0))
            .map_err(|_| format!("'{s}' is not a track"));
    }
    if let Some(r) = s.strip_prefix("minmax:") {
        let (a, b) = r.split_once(':').ok_or("minmax:<min>:<max>")?;
        return Ok(Track::MinMax {
            min: a.parse().map_err(|_| "minmax:<min>:<max>")?,
            max: b.parse().map_err(|_| "minmax:<min>:<max>")?,
        });
    }
    if let Some(r) = s.strip_prefix("repeat:") {
        let _ = r;
        return Err("write each track (a grid's `columns 1fr 1fr 1fr`)".into());
    }
    Err(format!(
        "'{s}' is not a track: a number (px), <n>fr, auto, <n>%, minmax:<min>:<max>"
    ))
}

/// A literal of type `ty` from its values.
pub(crate) fn parse_lit(ty: Ty, vals: &[Value]) -> Result<Lit, String> {
    let one_name = || -> Result<String, String> {
        match vals {
            [v] => as_name(v).ok_or_else(|| "takes one name".to_string()),
            _ => Err("takes one name (a word or a string)".into()),
        }
    };
    Ok(match ty {
        Ty::Num => Lit::Num(nums(vals, 1, "a number")?[0]),
        Ty::Num2 => Lit::Nums(nums(vals, 2, "x y")?),
        Ty::Num4 => Lit::Nums(nums(vals, 4, "four numbers")?),
        Ty::Insets => {
            let v = nums(vals, vals.len(), "1, 2 or 4 numbers")?;
            if !matches!(v.len(), 1 | 2 | 4) {
                return Err("takes 1, 2 or 4 numbers (all, x y, or left top right bottom)".into());
            }
            Lit::Nums(v)
        }
        Ty::Bool => match vals {
            [v] => Lit::Bool(v.as_bool().ok_or("takes true or false")?),
            [] => Lit::Bool(true),
            _ => return Err("takes true or false".into()),
        },
        Ty::Text => match vals {
            [v] => Lit::Str(as_name(v).unwrap_or_default()),
            _ => return Err("takes one string".into()),
        },
        Ty::Word(allowed) => {
            let w = one_name()?;
            if !allowed.contains(&w.as_str()) {
                return Err(format!("'{w}' is not one of {}", allowed.join(", ")));
            }
            Lit::Str(w)
        }
        Ty::Name => Lit::Str(one_name()?),
        Ty::Size => Lit::Size(parse_size(vals)?),
        Ty::Paint => Lit::Paint(parse_paint(vals)?),
        Ty::Transition => Lit::Transition(parse_transition(vals)?),
        Ty::Presence => Lit::Presence(parse_presence(vals, true)?),
        Ty::Tracks => Lit::Tracks(vals.iter().map(parse_track).collect::<Result<_, _>>()?),
        Ty::Names => Lit::Strs(vals.iter().filter_map(as_name).collect()),
    })
}

/// A prop's value from its line's values (literal, `bind`, or a template).
pub(crate) fn parse_value(ty: Ty, vals: &[Value]) -> Result<PropValue, String> {
    if let Some(Value::Word(w)) = vals.first()
        && w == "bind"
    {
        let src = vals
            .get(1)
            .and_then(as_name)
            .ok_or("bind <path> or bind \"<expression>\"")?;
        let e = Expr::parse(&src)?;
        let f = match vals.get(2).and_then(Value::as_word) {
            Some(name) => {
                let digits = vals
                    .get(3)
                    .and_then(Value::as_number)
                    .map(|n| n.as_f32() as u8);
                Some(Format::new(name, digits)?)
            }
            None => None,
        };
        return Ok(PropValue::Bind(e, f));
    }
    if ty == Ty::Text
        && let [Value::String(s)] = vals
        && s.contains('{')
    {
        return Ok(PropValue::Template(Template::parse(s)?));
    }
    Ok(PropValue::Lit(parse_lit(ty, vals)?))
}

/// One parse: the errors, each with its line.
struct Ctx {
    errors: Vec<String>,
    nodes: Vec<DocNode>,
}

impl Ctx {
    fn err(&mut self, line: usize, what: impl std::fmt::Display) {
        self.errors.push(format!("line {line}: {what}"));
    }

    fn prop(&mut self, kind: &str, e: &Entry) -> Option<PropSet> {
        let Some((spec, p)) = schema::prop(&e.key) else {
            self.err(
                e.line,
                format!(
                    "'{}' is not a prop (the schema: og.ui.documentSchema, README §Documents)",
                    e.key
                ),
            );
            return None;
        };
        if !p.applies(kind) {
            let on = match p.on {
                schema::On::Kinds(k) => k.join(", "),
                schema::On::All => String::new(),
            };
            self.err(
                e.line,
                format!("'{}' is not a prop of {kind} (it is {on}'s)", e.key),
            );
            return None;
        }
        match parse_value(p.ty, &e.values) {
            Ok(value) => Some(PropSet {
                spec,
                value,
                line: e.line,
                source: written(&e.values),
            }),
            Err(why) => {
                self.err(e.line, format!("{}: {why}", e.key));
                None
            }
        }
    }

    fn group(&mut self, e: &Entry) -> Option<DocGroup> {
        let w = words(e);
        let name = w.first().cloned().unwrap_or_default();
        if name.is_empty() {
            self.err(e.line, "state <group> [bind <expr>] { <state> … }");
            return None;
        }
        let bind = match w.get(1).map(String::as_str) {
            Some("bind") => match w.get(2).map(|s| Expr::parse(s)) {
                Some(Ok(x)) => Some(x),
                Some(Err(why)) => {
                    self.err(e.line, why);
                    None
                }
                None => {
                    self.err(e.line, "bind needs its expression");
                    None
                }
            },
            Some(other) => {
                self.err(
                    e.line,
                    format!("'{other}': a state group is `state <group> [bind <expr>] {{`"),
                );
                None
            }
            None => None,
        };
        let mut g = DocGroup {
            name,
            bind,
            ..DocGroup::default()
        };
        for s in e.block.as_ref().map_or(&[][..], |b| &b.entries) {
            if s.key == "transition" {
                let w = words(s);
                if w.len() < 3 {
                    self.err(s.line, "transition <from|*> <to|*> <transition>");
                    continue;
                }
                match parse_transition(&s.values[2..]) {
                    Ok(t) => g.transitions.push(StateTransition {
                        from: w[0].clone(),
                        to: w[1].clone(),
                        transition: t,
                    }),
                    Err(why) => self.err(s.line, why),
                }
                continue;
            }
            let mut st = DocState {
                name: s.key.clone(),
                ..DocState::default()
            };
            for v in s.block.as_ref().map_or(&[][..], |b| &b.entries) {
                match v.key.as_str() {
                    "enter" | "exit" => {
                        let n = v.values.first().and_then(as_name).unwrap_or_default();
                        if v.key == "enter" {
                            st.on_enter = n;
                        } else {
                            st.on_exit = n;
                        }
                    }
                    "background" | "ink" => match parse_paint(&v.values) {
                        Ok(p) => st.values.push(if v.key == "background" {
                            StateValue::Background(p)
                        } else {
                            StateValue::Ink(p)
                        }),
                        Err(why) => self.err(v.line, why),
                    },
                    k => match (
                        Prop::from_name(k),
                        v.values.first().and_then(Value::as_number),
                    ) {
                        (Some(p), Some(n)) => st.values.push(StateValue::Prop(p, n.as_f32())),
                        (Some(_), None) => self.err(v.line, format!("{k} takes a number")),
                        (None, _) => self.err(
                            v.line,
                            format!(
                                "'{k}' is not a state value: {}, background, ink, enter, exit",
                                Prop::ALL
                                    .iter()
                                    .map(|p| p.name())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        ),
                    },
                }
            }
            if g.states.iter().any(|x| x.name == st.name) {
                self.err(s.line, format!("the state '{}' is declared twice", st.name));
            }
            g.states.push(st);
        }
        if g.states.is_empty() {
            self.err(
                e.line,
                format!("the state group '{}' has no states", g.name),
            );
        }
        for t in &g.transitions {
            for s in [&t.from, &t.to] {
                if s != "*" && !g.states.iter().any(|x| &x.name == s) {
                    self.errors.push(format!(
                        "line {}: the transition names '{s}', which is not a state of '{}'",
                        e.line, g.name
                    ));
                }
            }
        }
        Some(g)
    }

    fn trigger(&mut self, e: &Entry) -> Option<Trigger> {
        let w = words(e);
        const HOW: &str = "on <click|changed|appear|change <binding>|action <name>> <play <timeline> | state <group> <state> | emit <name> | push <screen> | pop | action <name> | set <path> <expr>>";
        let (when, rest) = match w.first().map(String::as_str) {
            Some("click") => (When::Click, &w[1..]),
            Some("changed") => (When::Changed, &w[1..]),
            Some("appear") => (When::Appear, &w[1..]),
            Some("action") if w.len() > 1 => (When::Action(w[1].clone()), &w[2..]),
            Some("change") if w.len() > 1 => match Expr::parse(&w[1]) {
                Ok(x) => (When::Change(x), &w[2..]),
                Err(why) => {
                    self.err(e.line, why);
                    return None;
                }
            },
            _ => {
                self.err(e.line, HOW);
                return None;
            }
        };
        let arg = |i: usize| rest.get(i).cloned();
        let act = match rest.first().map(String::as_str) {
            Some("play") => arg(1).map(Act::Play),
            Some("state") => arg(1).zip(arg(2)).map(|(g, s)| Act::State(g, s)),
            Some("emit") => arg(1).map(Act::Emit),
            Some("push") => arg(1).map(Act::Push),
            Some("pop") => Some(Act::Pop),
            Some("action") => arg(1).map(Act::Action),
            Some("set") => match (arg(1), arg(2).map(|s| Expr::parse(&s))) {
                (Some(p), Some(Ok(x))) => Some(Act::Set(p, x)),
                (_, Some(Err(why))) => {
                    self.err(e.line, why);
                    return None;
                }
                _ => None,
            },
            _ => None,
        };
        match act {
            Some(act) => Some(Trigger {
                when,
                act,
                line: e.line,
            }),
            None => {
                self.err(e.line, HOW);
                None
            }
        }
    }

    fn at(&mut self, kind: &str, e: &Entry) -> Option<(AtQuery, Vec<PropSet>)> {
        use crate::element::Orientation;
        use crate::style::Breakpoint;
        let mut q = AtQuery::default();
        for w in words(e) {
            match w.as_str() {
                "phonePortrait" => q.breakpoints.push(Breakpoint::PhonePortrait),
                "phoneLandscape" => q.breakpoints.push(Breakpoint::PhoneLandscape),
                "phone" => q
                    .breakpoints
                    .extend([Breakpoint::PhonePortrait, Breakpoint::PhoneLandscape]),
                "tablet" => q.breakpoints.push(Breakpoint::Tablet),
                "desktop" => q.breakpoints.push(Breakpoint::Desktop),
                "portrait" => q.orientation = Some(Orientation::Portrait),
                "landscape" => q.orientation = Some(Orientation::Landscape),
                other => {
                    self.err(e.line, format!("'{other}' is not a breakpoint: phonePortrait, phoneLandscape, phone, tablet, desktop, portrait, landscape"));
                    return None;
                }
            }
        }
        let props = e
            .block
            .as_ref()
            .map_or(&[][..], |b| &b.entries)
            .iter()
            .filter_map(|p| self.prop(kind, p))
            .collect();
        Some((q, props))
    }

    /// A node from its entry (an element, `for`, `if` or `use`).
    fn node(&mut self, e: &Entry, parent: usize) -> Option<usize> {
        let w = words(e);
        let (kind, id) = match e.key.as_str() {
            "for" => {
                if w.len() != 3 || w[1] != "in" {
                    self.err(e.line, "for <item> in <list> { <template> }");
                    return None;
                }
                match Expr::parse(&w[2]) {
                    Ok(list) => (
                        NodeKind::For {
                            var: w[0].clone(),
                            list,
                        },
                        String::new(),
                    ),
                    Err(why) => {
                        self.err(e.line, why);
                        return None;
                    }
                }
            }
            "if" => match w.first().map(|s| Expr::parse(s)) {
                Some(Ok(x)) => (NodeKind::If(x), String::new()),
                Some(Err(why)) => {
                    self.err(e.line, why);
                    return None;
                }
                None => {
                    self.err(e.line, "if <expr> { … }");
                    return None;
                }
            },
            "use" => {
                let Some(c) = w.first() else {
                    self.err(e.line, "use <Component> [id] { <prop> <value> … }");
                    return None;
                };
                (
                    NodeKind::Use(c.clone()),
                    w.get(1).cloned().unwrap_or_else(|| c.clone()),
                )
            }
            k => match schema::kind(k) {
                Some(spec) => {
                    if w.len() > 1 {
                        self.err(
                            e.line,
                            format!("{k} takes one id (its props go in its block)"),
                        );
                    }
                    (
                        NodeKind::Element(spec.name),
                        w.first().cloned().unwrap_or_default(),
                    )
                }
                None => {
                    self.err(
                        e.line,
                        format!(
                            "'{k}' is not an element: {}, or for, if, use",
                            schema::KINDS
                                .iter()
                                .map(|k| k.name)
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    );
                    return None;
                }
            },
        };
        if id.contains('/') {
            self.err(
                e.line,
                format!("the id '{id}' has a '/' (paths join ids with it)"),
            );
        }
        let i = self.nodes.len();
        self.nodes.push(DocNode {
            kind: kind.clone(),
            id,
            line: e.line,
            props: Vec::new(),
            at: Vec::new(),
            groups: Vec::new(),
            triggers: Vec::new(),
            args: Vec::new(),
            children: Vec::new(),
            parent,
        });
        let kname = match &kind {
            NodeKind::Element(k) => *k,
            _ => "",
        };
        let container = schema::kind(kname).is_some_and(|k| k.container)
            || matches!(kind, NodeKind::For { .. } | NodeKind::If(_));
        for c in e.block.as_ref().map_or(&[][..], |b| &b.entries) {
            match (&kind, c.key.as_str()) {
                (NodeKind::Use(_), k)
                    if c.block.is_none()
                        && schema::kind(k).is_none()
                        && !matches!(k, "for" | "if" | "use" | "state" | "on" | "at") =>
                {
                    match schema::prop(k) {
                        // An element prop applies to the instance's root.
                        Some(_)
                            if !k.is_empty()
                                && self.nodes[i].args.iter().all(|a| a.0 != k)
                                && is_element_prop(k) =>
                        {
                            if let Some(p) = self.prop("column", c) {
                                self.nodes[i].props.push(p);
                            }
                        }
                        _ => match parse_value(Ty::Text, &c.values)
                            .or_else(|_| parse_value(Ty::Names, &c.values))
                        {
                            Ok(v) => {
                                let v = match v {
                                    PropValue::Lit(Lit::Strs(mut s)) if s.len() == 1 => {
                                        PropValue::Lit(Lit::Str(s.remove(0)))
                                    }
                                    PropValue::Lit(Lit::Str(s)) => match c.values.first() {
                                        Some(Value::Number(n)) => {
                                            PropValue::Lit(Lit::Num(n.as_f32()))
                                        }
                                        Some(Value::Word(b)) if b == "true" || b == "false" => {
                                            PropValue::Lit(Lit::Bool(b == "true"))
                                        }
                                        _ => PropValue::Lit(Lit::Str(s)),
                                    },
                                    other => other,
                                };
                                self.nodes[i].args.push((k.to_string(), v, c.line));
                            }
                            Err(why) => self.err(c.line, format!("{k}: {why}")),
                        },
                    }
                }
                (_, "state") => {
                    if let Some(g) = self.group(c) {
                        self.nodes[i].groups.push(g);
                    }
                }
                (_, "on") => {
                    if let Some(t) = self.trigger(c) {
                        self.nodes[i].triggers.push(t);
                    }
                }
                (_, "at") => {
                    if let Some(a) = self.at(if kname.is_empty() { "column" } else { kname }, c) {
                        self.nodes[i].at.push(a);
                    }
                }
                // An element (a kind name that is also one of its own props,
                // `icon coin` in an icon, is the prop).
                (_, k)
                    if (schema::kind(k).is_some() || matches!(k, "for" | "if" | "use"))
                        && (container
                            || c.block.is_some()
                            || !schema::prop(k).is_some_and(|(_, p)| p.applies(kname))) =>
                {
                    if !container {
                        self.err(c.line, format!("{kname} has no children (a container: column, row, stack, grid, panel, scroll, list, cells, slot)"));
                        continue;
                    }
                    if let Some(ch) = self.node(c, i) {
                        self.nodes[i].children.push(ch);
                    }
                }
                (NodeKind::Element(_), _) => {
                    if let Some(p) = self.prop(kname, c) {
                        self.nodes[i].props.push(p);
                    }
                }
                (_, k) => self.err(
                    c.line,
                    format!("'{k}': a `for` or `if` holds elements only"),
                ),
            }
        }
        if matches!(kind, NodeKind::Element("list" | "cells")) {
            let fors = self.nodes[i]
                .children
                .iter()
                .filter(|&&c| matches!(self.nodes[c].kind, NodeKind::For { .. }))
                .count();
            if fors != 1 || self.nodes[i].children.len() != 1 {
                self.err(e.line, format!("{kname} holds exactly one `for` (its rows are built from it, only those in view)"));
            }
        }
        // Sibling ids are unique (paths name elements by them).
        let kids = self.nodes[i].children.clone();
        let mut seen = std::collections::BTreeSet::new();
        for c in kids {
            let id = &self.nodes[c].id;
            if !id.is_empty() && !seen.insert(id.clone()) {
                let line = self.nodes[c].line;
                self.err(
                    line,
                    format!("the id '{id}' repeats among its siblings: ids name elements in paths"),
                );
            }
        }
        Some(i)
    }
}

/// A prop of the box or the element model (every kind's), which a `use`
/// instance gives its component's root.
fn is_element_prop(name: &str) -> bool {
    schema::prop(name).is_some_and(|(_, p)| p.on == schema::On::All)
}

fn parse_timeline(ctx: &mut Ctx, e: &Entry) -> Option<Timeline> {
    let mut t = Timeline::default();
    for c in e.block.as_ref().map_or(&[][..], |b| &b.entries) {
        let w = words(c);
        match c.key.as_str() {
            "track" => {
                let (Some(target), Some(prop)) = (w.first(), w.get(1)) else {
                    ctx.err(
                        c.line,
                        "track <target|\"\"> <prop> { key <ms> <value> [ease] … }",
                    );
                    continue;
                };
                let Some(prop) = Prop::from_name(prop) else {
                    ctx.err(
                        c.line,
                        format!(
                            "'{prop}' is not an animatable prop: {}",
                            Prop::ALL
                                .iter()
                                .map(|p| p.name())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    );
                    continue;
                };
                let mut keys = Vec::new();
                for k in c.block.as_ref().map_or(&[][..], |b| &b.entries) {
                    if k.key != "key" {
                        ctx.err(k.line, "a track holds `key <ms> <value> [ease]` lines");
                        continue;
                    }
                    let n = |i: usize| {
                        k.values
                            .get(i)
                            .and_then(Value::as_number)
                            .map(Number::as_f32)
                    };
                    let (Some(at_ms), Some(value)) = (n(0), n(1)) else {
                        ctx.err(k.line, "key <ms> <value> [linear|out|exit|inOut|hold|spring <ms> <bounce>|bezier a b c d]");
                        continue;
                    };
                    let ease = match k.values.get(2).and_then(Value::as_word) {
                        None => KeyEase::Ease(Ease::Out),
                        Some("linear") => KeyEase::Linear,
                        Some("hold") => KeyEase::Hold,
                        Some("spring") => KeyEase::Spring(SpringSpec::new(
                            n(3).unwrap_or(300.0),
                            n(4).unwrap_or(0.0),
                        )),
                        Some("bezier") => match (n(3), n(4), n(5), n(6)) {
                            (Some(a), Some(b), Some(c2), Some(d)) => {
                                KeyEase::Ease(Ease::Bezier([a, b, c2, d]))
                            }
                            _ => {
                                ctx.err(k.line, "bezier takes four numbers");
                                continue;
                            }
                        },
                        Some(e2) => match parse_ease(e2) {
                            Ok(e2) => KeyEase::Ease(e2),
                            Err(why) => {
                                ctx.err(k.line, why);
                                continue;
                            }
                        },
                    };
                    keys.push(Key { at_ms, value, ease });
                }
                if keys.windows(2).any(|w| w[1].at_ms < w[0].at_ms) {
                    ctx.err(c.line, "a track's keys are in time order");
                }
                t.tracks.push(TimelineTrack {
                    target: target.clone(),
                    prop,
                    keys,
                });
            }
            "event" => {
                let at = c
                    .values
                    .first()
                    .and_then(Value::as_number)
                    .map(Number::as_f32);
                let (Some(at_ms), Some(target), Some(kind)) = (at, w.get(1), w.get(2)) else {
                    ctx.err(
                        c.line,
                        "event <ms> <target|\"\"> <sound|haptic|burst|state|callback> …",
                    );
                    continue;
                };
                let n = |i: usize| {
                    c.values
                        .get(i)
                        .and_then(Value::as_number)
                        .map(Number::as_f32)
                };
                let kind = match kind.as_str() {
                    "sound" => EventKind::Sound {
                        name: w.get(3).cloned().unwrap_or_default(),
                        pitch: n(4).unwrap_or(1.0),
                    },
                    "haptic" => {
                        EventKind::Haptic(w.get(3).cloned().unwrap_or_else(|| "light".into()))
                    }
                    "burst" => EventKind::Burst {
                        count: n(3).unwrap_or(10.0) as u32,
                        radius: n(4).unwrap_or(40.0),
                        color: w.get(5).and_then(|s| crate::style_file::parse_color(s)),
                    },
                    "state" => match (w.get(3), w.get(4)) {
                        (Some(g), Some(s)) => EventKind::State {
                            group: g.clone(),
                            state: s.clone(),
                        },
                        _ => {
                            ctx.err(c.line, "event <ms> <target> state <group> <state>");
                            continue;
                        }
                    },
                    "callback" => EventKind::Callback(w.get(3).cloned().unwrap_or_default()),
                    other => {
                        ctx.err(
                            c.line,
                            format!(
                                "'{other}' is not an event: sound, haptic, burst, state, callback"
                            ),
                        );
                        continue;
                    }
                };
                t.events.push(TimelineEvent {
                    at_ms,
                    target: target.clone(),
                    kind,
                });
            }
            "label" => match (w.first(), c.values.get(1).and_then(Value::as_number)) {
                (Some(l), Some(ms)) => t.labels.push((l.clone(), ms.as_f32())),
                _ => ctx.err(c.line, "label <name> <ms>"),
            },
            "duration" => match c.values.first().and_then(Value::as_number) {
                Some(ms) => t.duration_ms = ms.as_f32(),
                None => ctx.err(c.line, "duration <ms>"),
            },
            k => ctx.err(
                c.line,
                format!("'{k}' is not part of a timeline: track, event, label, duration"),
            ),
        }
    }
    Some(t)
}

impl UiDocument {
    /// Parses and validates a document (`source`: its asset id, for errors
    /// and the write-back). Every error has its line and its fix.
    pub fn parse(source: &str, text: &str) -> Result<UiDocument, Vec<String>> {
        let doc = Document::parse(text).map_err(|e| vec![format!("{source}: {}", e.reason())])?;
        let kind = match doc.ty.as_str() {
            SCREEN_TYPE => DocKind::Screen,
            COMPONENT_TYPE => DocKind::Component,
            other => {
                return Err(vec![format!(
                    "{source}: an .ogt of type '{other}' is not a document (screen or component)"
                )]);
            }
        };
        if doc.version != 1 {
            return Err(vec![format!(
                "{source}: version {} of a document is not known (1)",
                doc.version
            )]);
        }
        let mut ctx = Ctx {
            errors: Vec::new(),
            nodes: Vec::new(),
        };
        let mut d = UiDocument {
            kind,
            name: String::new(),
            source: source.to_string(),
            text: text.to_string(),
            layer: LayerSpec::of(LayerKind::Menu),
            props: Vec::new(),
            parts: Vec::new(),
            actions: Vec::new(),
            timelines: Vec::new(),
            fixtures: Vec::new(),
            nodes: Vec::new(),
            root: NONE,
        };
        // The layer kind first: the other layer lines change its defaults.
        if let Some(e) = doc.body.get("layer") {
            match e
                .values
                .first()
                .and_then(Value::as_word)
                .and_then(LayerKind::from_name)
            {
                Some(k) => d.layer = LayerSpec::of(k),
                None => ctx.err(
                    e.line,
                    format!("layer is one of {}", LayerKind::NAMES.join(", ")),
                ),
            }
        }
        let screen_only = |ctx: &mut Ctx, e: &Entry| {
            if kind != DocKind::Screen {
                ctx.err(
                    e.line,
                    format!(
                        "'{}' is a screen's (a component is placed by the screen that uses it)",
                        e.key
                    ),
                );
            }
        };
        for e in &doc.body.entries {
            let w = words(e);
            let first = w.first().cloned().unwrap_or_default();
            match e.key.as_str() {
                "name" => d.name = first,
                "layer" => screen_only(&mut ctx, e),
                "input" => {
                    screen_only(&mut ctx, e);
                    d.layer.input = match first.as_str() {
                        "game" => InputMode::Game,
                        "menu" => InputMode::Menu,
                        "all" => InputMode::All,
                        _ => {
                            ctx.err(e.line, "input is game, menu or all");
                            continue;
                        }
                    };
                }
                "capture" => d.layer.capture = e.values.first().and_then(Value::as_bool).unwrap_or(true),
                "pause" => d.layer.pause = e.values.first().and_then(Value::as_bool).unwrap_or(true),
                "remember" => d.layer.remember = e.values.first().and_then(Value::as_bool).unwrap_or(true),
                "under" => {
                    d.layer.under = match first.as_str() {
                        "none" => Under::None,
                        "dim" => Under::Dim,
                        "blur" => Under::Blur,
                        "hide" => Under::Hide,
                        _ => {
                            ctx.err(e.line, "under is none, dim, blur or hide");
                            continue;
                        }
                    };
                }
                "back" => {
                    d.layer.back = match (first.as_str(), w.get(1)) {
                        ("pop", None) => Back::Pop,
                        ("none", None) => Back::None,
                        ("emit", Some(n)) => Back::Emit(n.clone()),
                        _ => {
                            ctx.err(e.line, "back is pop, none or emit <name>");
                            continue;
                        }
                    };
                }
                "focus" => d.layer.focus = first,
                "enter" | "exit" => match parse_presence(&e.values, e.key == "enter") {
                    Ok(p) if e.key == "enter" => d.layer.enter = Some(p),
                    Ok(p) => d.layer.exit = Some(p),
                    Err(why) => ctx.err(e.line, why),
                },
                "props" => {
                    for p in e.block.as_ref().map_or(&[][..], |b| &b.entries) {
                        let w = words(p);
                        let ty = w.first().cloned().unwrap_or_else(|| "any".into());
                        if !["number", "string", "bool", "list", "object", "any"].contains(&ty.as_str()) {
                            ctx.err(p.line, format!("'{ty}' is not a prop type: number, string, bool, list, object, any"));
                        }
                        d.props.push(ContractProp { name: p.key.clone(), ty, doc: w.get(1).cloned().unwrap_or_default() });
                    }
                }
                "parts" => d.parts = w,
                "actions" => {
                    for a in e.block.as_ref().map_or(&[][..], |b| &b.entries) {
                        d.actions.push((a.key.clone(), words(a).first().cloned().unwrap_or_default()));
                    }
                }
                "timeline" => {
                    if let Some(t) = parse_timeline(&mut ctx, e) {
                        if d.timelines.iter().any(|(n, _)| *n == first) {
                            ctx.err(e.line, format!("the timeline '{first}' is declared twice"));
                        }
                        d.timelines.push((first, t));
                    }
                }
                "fixture" => match w.get(1).map(|j| serde_json::from_str::<serde_json::Value>(j)) {
                    Some(Ok(v)) => d.fixtures.push((first, v)),
                    Some(Err(why)) => ctx.err(e.line, format!("the fixture's JSON: {why}")),
                    None => ctx.err(e.line, "fixture <name> \"<json>\""),
                },
                k if schema::kind(k).is_some() || k == "use" => {
                    if d.root != NONE {
                        ctx.err(e.line, "a document has one root element");
                        continue;
                    }
                    d.root = ctx.node(e, NONE).unwrap_or(NONE);
                }
                k => ctx.err(e.line, format!("'{k}' is not part of a document: name, layer, input, capture, pause, under, back, focus, remember, enter, exit, props, parts, actions, timeline, fixture, and one root element")),
            }
        }
        if d.name.is_empty() {
            ctx.errors
                .push(format!("{source}: a document has a name (name \"shop\")"));
        }
        if d.root == NONE && ctx.errors.is_empty() {
            ctx.errors.push(format!(
                "{source}: a document has one root element (column, row, stack, panel …)"
            ));
        }
        d.nodes = std::mem::take(&mut ctx.nodes);
        d.check(&mut ctx.errors);
        if ctx.errors.is_empty() {
            Ok(d)
        } else {
            Err(ctx
                .errors
                .into_iter()
                .map(|e| format!("{source}: {e}"))
                .collect())
        }
    }

    /// The checks that read the whole tree: timelines named, parts
    /// declared and used, bindings rooted in the contract.
    fn check(&self, errors: &mut Vec<String>) {
        let timelines: Vec<&str> = self.timelines.iter().map(|(n, _)| n.as_str()).collect();
        let mut used_parts = Vec::new();
        for (i, n) in self.nodes.iter().enumerate() {
            let line = n.line;
            for t in &n.triggers {
                if let Act::Play(tl) = &t.act
                    && !timelines.contains(&tl.as_str())
                    && !tl.contains('/')
                {
                    errors.push(format!("line {}: the timeline '{tl}' is not declared in this document (timeline {tl} {{ … }}; another document's is <doc>/<name>)", t.line));
                }
            }
            for g in &n.groups {
                for s in &g.states {
                    for tl in [&s.on_enter, &s.on_exit] {
                        if !tl.is_empty() && !timelines.contains(&tl.as_str()) && !tl.contains('/')
                        {
                            errors.push(format!(
                                "line {line}: the state '{}' plays '{tl}', which is not declared",
                                s.name
                            ));
                        }
                    }
                }
            }
            if let Some(p) = n.props.iter().find(|p| p.name() == "part")
                && let PropValue::Lit(Lit::Str(name)) = &p.value
            {
                used_parts.push(name.clone());
                if !self.parts.contains(name) {
                    errors.push(format!(
                        "line {}: the part '{name}' is not in the document's `parts` line",
                        p.line
                    ));
                }
            }
            if !self.props.is_empty() {
                let locals = self.locals(i);
                for (path, line) in self.node_paths_at(n) {
                    let root = path.split('.').next().unwrap_or("");
                    let known = self.props.iter().any(|p| p.name == root)
                        || locals.iter().any(|l| l == root)
                        || root.starts_with(|c: char| c.is_ascii_uppercase())
                        || root.starts_with('$');
                    if !known {
                        errors.push(format!("line {line}: '{path}' reads '{root}', which is not a prop of the document (props {{ … }}), a list item, or a resource (Capitalised)"));
                    }
                }
            }
        }
        for p in &self.parts {
            if !used_parts.contains(p) {
                errors.push(format!(
                    "the part '{p}' is listed in `parts` but no element is it (part {p})"
                ));
            }
        }
    }

    /// The `for` variables in scope at node `i`.
    fn locals(&self, mut i: usize) -> Vec<String> {
        let mut out = Vec::new();
        while i != NONE {
            if let NodeKind::For { var, .. } = &self.nodes[i].kind {
                out.push(var.clone());
            }
            i = self.nodes[i].parent;
        }
        out
    }

    /// Every path a node's bindings read, each with the line that reads it.
    fn node_paths_at(&self, n: &DocNode) -> Vec<(String, usize)> {
        let mut out = Vec::new();
        let mut add = |v: &PropValue, line: usize| {
            let mut p = Vec::new();
            match v {
                PropValue::Bind(e, _) => e.paths(&mut p),
                PropValue::Template(t) => t.paths(&mut p),
                PropValue::Lit(_) => {}
            }
            out.extend(p.into_iter().map(|p| (p, line)));
        };
        for p in n.props.iter().chain(n.at.iter().flat_map(|a| &a.1)) {
            add(&p.value, p.line);
        }
        for (_, v, line) in &n.args {
            add(v, *line);
        }
        let mut rest = Vec::new();
        let bare = DocNode {
            props: Vec::new(),
            at: Vec::new(),
            args: Vec::new(),
            ..n.clone()
        };
        self.node_paths(&bare, &mut rest);
        out.extend(rest.into_iter().map(|p| (p, n.line)));
        out
    }

    /// Every path a node's bindings read (its own, not its children's).
    pub fn node_paths(&self, n: &DocNode, out: &mut Vec<String>) {
        let value = |v: &PropValue, out: &mut Vec<String>| match v {
            PropValue::Bind(e, _) => e.paths(out),
            PropValue::Template(t) => t.paths(out),
            PropValue::Lit(_) => {}
        };
        for p in n.props.iter().chain(n.at.iter().flat_map(|a| &a.1)) {
            value(&p.value, out);
        }
        for (_, v, _) in &n.args {
            value(v, out);
        }
        for g in &n.groups {
            if let Some(b) = &g.bind {
                b.paths(out);
            }
        }
        for t in &n.triggers {
            if let When::Change(e) = &t.when {
                e.paths(out);
            }
            if let Act::Set(_, e) = &t.act {
                e.paths(out);
            }
        }
        match &n.kind {
            NodeKind::For { list, .. } => list.paths(out),
            NodeKind::If(e) => e.paths(out),
            _ => {}
        }
    }

    /// The node at a path of ids below the root (`Header/Title`; the
    /// root's own id first is allowed too), through `for` templates
    /// (`Items/Card#3/Buy` is the template's `Buy`) and `if` blocks.
    pub fn node_at(&self, path: &str) -> Option<usize> {
        let mut at = self.root;
        let mut parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        if parts.first() == Some(&self.nodes.get(at)?.id.as_str()) {
            parts.remove(0);
        }
        for p in parts {
            let id = p.split('#').next().unwrap_or(p);
            at = self.child_by_id(at, id)?;
        }
        Some(at)
    }

    fn child_by_id(&self, at: usize, id: &str) -> Option<usize> {
        for &c in &self.nodes[at].children {
            let n = &self.nodes[c];
            if matches!(n.kind, NodeKind::For { .. } | NodeKind::If(_)) {
                if let Some(x) = self.child_by_id(c, id) {
                    return Some(x);
                }
            } else if n.id == id {
                return Some(c);
            }
        }
        None
    }

    /// Writes `prop value` into node `node`'s block in the text, keeping
    /// every other line: its line is replaced when the node sets the prop,
    /// else one is added after the node's opening line. The document is
    /// parsed again from the new text (refused, unchanged, if it no longer
    /// parses).
    pub fn set_prop(&mut self, node: usize, prop: &str, value: &str) -> Result<(), String> {
        let n = self.nodes.get(node).ok_or("no such node")?;
        let (_, spec) = schema::prop(prop).ok_or_else(|| format!("'{prop}' is not a prop"))?;
        let line = format!("{prop} {}", value_text(spec.ty, value));
        let mut lines: Vec<String> = self.text.split('\n').map(str::to_string).collect();
        let indent = |l: &str| l.len() - l.trim_start().len();
        if let Some(p) = n.props.iter().rev().find(|p| p.name() == prop) {
            let old = &lines[p.line - 1];
            lines[p.line - 1] = format!("{}{line}", &old[..indent(old)]);
        } else {
            let open = &lines[n.line - 1];
            let pad = " ".repeat(indent(open) + 2);
            if open.trim_end().ends_with('{') {
                lines.insert(n.line, format!("{pad}{line}"));
            } else {
                let base = open[..indent(open)].to_string();
                let head = open.trim_end().to_string();
                lines[n.line - 1] = format!("{head} {{");
                lines.insert(n.line, format!("{pad}{line}"));
                lines.insert(n.line + 1, format!("{base}}}"));
            }
        }
        let text = lines.join("\n");
        let doc = UiDocument::parse(&self.source, &text).map_err(|e| e.join("; "))?;
        *self = doc;
        Ok(())
    }

    /// A fixture by name (`default`: the first).
    pub fn fixture(&self, name: &str) -> Option<&serde_json::Value> {
        self.fixtures
            .iter()
            .find(|(n, _)| n == name)
            .or_else(|| (name == "default").then(|| self.fixtures.first()).flatten())
            .map(|(_, v)| v)
    }

    /// Its static structure as JSON (dump_ui's `documents`, og.ui.document).
    pub fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        let mut slots = Vec::new();
        let mut arts = Vec::new();
        for n in &self.nodes {
            match n.kind {
                NodeKind::Element("slot") => slots.push(n.id.clone()),
                NodeKind::Element("art") => arts.push(n.id.clone()),
                _ => {}
            }
        }
        json!({
            "name": self.name,
            "kind": match self.kind { DocKind::Screen => "screen", DocKind::Component => "component" },
            "source": self.source,
            "layer": {
                "kind": self.layer.kind.name(), "input": self.layer.input.name(),
                "capture": self.layer.capture, "pause": self.layer.pause,
                "under": self.layer.under.name(),
                "back": match &self.layer.back { Back::Pop => "pop".to_string(), Back::None => "none".into(), Back::Emit(n) => format!("emit {n}") },
                "focus": self.layer.focus, "remember": self.layer.remember,
            },
            "props": self.props.iter().map(|p| json!({ "name": p.name, "type": p.ty, "doc": p.doc })).collect::<Vec<_>>(),
            "parts": self.parts,
            "slots": slots,
            "arts": arts,
            "actions": self.actions.iter().map(|(a, h)| json!({ "action": a, "hint": h })).collect::<Vec<_>>(),
            "timelines": self.timelines.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>(),
            "fixtures": self.fixtures.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>(),
            "nodes": self.nodes.len(),
        })
    }
}

/// A value as a document line writes it for a prop of type `ty`: a text
/// prop's bare value is quoted; anything else is taken as written.
pub(crate) fn value_text(ty: Ty, value: &str) -> String {
    let v = value.trim();
    let quoted = v.starts_with('"') && v.ends_with('"') && v.len() >= 2;
    match ty {
        Ty::Text if !quoted && !v.starts_with("bind ") => quote(v),
        Ty::Name if !quoted && v.contains(char::is_whitespace) => quote(v),
        _ => v.to_string(),
    }
}

/// A probe's value text for a prop (`ui:set`): parsed as a document line
/// would be.
pub(crate) fn parse_probe_value(prop: &str, value: &str) -> Result<(usize, PropValue), String> {
    let (spec, p) = schema::prop(prop)
        .ok_or_else(|| format!("'{prop}' is not a prop (og.ui.documentSchema lists them)"))?;
    let text = format!("type x\nversion 1\n---\nv {}\n", value_text(p.ty, value));
    let doc = Document::parse(&text).map_err(|e| format!("{prop}={value}: {}", e.reason()))?;
    let vals = &doc.body.get("v").ok_or("no value")?.values;
    parse_value(p.ty, vals)
        .map(|v| (spec, v))
        .map_err(|e| format!("{prop}: {e}"))
}
