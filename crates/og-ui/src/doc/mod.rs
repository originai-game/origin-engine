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
pub mod css;
pub mod schema;
pub mod types;

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
    /// Its tap, double tap or long press was recognised (its `tap`,
    /// `doubleTap` or `longPress` prop).
    Tap,
    DoubleTap,
    LongPress,
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
        "glitch" => TransitionKind::Glitch,
        other => {
            return Err(format!(
                "'{other}' is not a presence: fade, slideUp, slideDown, slideLeft, slideRight, scale, pop, glitch"
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
    const HOW: &str =
        "a paint: token <role> (a palette role or one the style declares) or \"#rrggbb[aa]\"";
    match vals {
        [Value::Word(t), Value::Word(role)] if t == "token" => {
            // A palette role, or one the style declares (docs/ui.md §12.2):
            // a name the style lacks draws as a missing token.
            if crate::Palette::NAMES.contains(&role.as_str())
                || (role.starts_with(|c: char| c.is_ascii_lowercase())
                    && role.chars().all(|c| c.is_ascii_alphanumeric()))
            {
                Ok(Paint::Token(role.clone()))
            } else {
                Err(format!(
                    "'{role}' is not a colour role: a palette role ({}) or one the style declares",
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

mod document;
mod parse;
pub(crate) use document::*;
use parse::parse_timeline;
