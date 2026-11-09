//! Scriptless Svelte documents and scoped CSS, validated against [`schema`].
//! [`UiDocument::parse`] produces reflected nodes, expressions, and cascade
//! rules for the retained builder. The cook stores the validated document
//! with its base string-table dependencies in an `og.UiDocument` blob.
//! [`UiDocument::set_prop`] saves probe edits without rewriting other nodes.

pub mod at_rules;
pub(crate) mod boxes;
pub mod build;
pub(crate) mod calc;
pub mod catalogue;
pub(crate) mod contain_css;
pub mod container;
pub mod css;
mod extern_element;
pub mod generated;
pub(crate) mod image_css;
pub(crate) mod interact_css;
pub(crate) mod lists;
pub mod matching;
pub mod media;
pub(crate) mod mf2;
pub(crate) mod motion;
pub(crate) mod multicol_css;
pub(crate) mod paint_css;
pub(crate) mod position_css;
pub mod schema;
mod schema_extra;
mod script;
pub(crate) mod scroll_css;
pub mod selectors;
mod spans;
pub(crate) mod stylesheet;
mod svelte;
mod syntax;
pub(crate) mod text_css;
pub mod topology;
pub(crate) mod tracks;
pub mod types;
pub mod variables;
pub mod view_transition;
mod writeback;

use crate::bind::{Expr, Format, Template};
use crate::element::Prop;
use crate::layout::Size;
use crate::motion::{
    Ease, MotionToken, PresenceSpec, SpringSpec, StateTransition, Timeline, Transition,
    TransitionKind,
};
use crate::style::Srgba;
use og_core::ogt::{Document, Number, Value};
use schema::{PROPS, Ty};
pub use spans::SpanRule;

/// The reflected document kinds declared by metadata.
pub const SCREEN_TYPE: &str = "screen";
pub const COMPONENT_TYPE: &str = "component";

/// The engine's `$ui` store, which every mounted document reads unless it
/// binds its own `ui` (docs/ui.md §2): the device used last, a coarse
/// pointer and the action bar.
pub(crate) static UI_STORE: std::sync::LazyLock<crate::reactive::SourceId> =
    std::sync::LazyLock::new(|| crate::reactive::SourceId::new("og", "ui"));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocKind {
    Screen,
    Component,
}

/// The kinds of layer a screen declares (docs/ui.md §2.1).
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
    /// Its CSS view transition (`@view-transition`): the frame is covered
    /// as the layer joins the stack and as it leaves.
    pub transition: Option<view_transition::ViewTransition>,
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
            transition: None,
        }
    }
}

/// A colour a document names.
#[derive(Clone, Debug, PartialEq)]
pub enum Paint {
    /// A palette role, resolved through the style when drawn.
    Token(String),
    Color(Srgba),
    /// A CSS colour that reads roles, `currentColor` or `light-dark()`
    /// (`color-mix(in oklch, var(--accent) 40%, var(--panel))`).
    Css(std::sync::Arc<crate::css_color::Color>),
}

impl Paint {
    /// Its colour in `theme` (`None`: an unknown role).
    pub fn resolve(&self, theme: &crate::Theme) -> Option<Srgba> {
        match self {
            Paint::Token(t) => theme.color(t),
            Paint::Color(c) => Some(*c),
            Paint::Css(c) => Some(c.resolve(&crate::css_color::Context::of(theme))),
        }
    }

    /// The same colour as a draw's [`crate::paint::ColorRef`].
    pub fn color_ref(&self) -> crate::paint::ColorRef {
        match self {
            Paint::Token(t) => crate::paint::ColorRef::token(t),
            Paint::Color(c) => crate::paint::ColorRef::Literal(*c),
            Paint::Css(c) => crate::paint::ColorRef::Css(c.clone()),
        }
    }
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
    Tracks(tracks::TrackList),
    Strs(Vec<String>),
}

/// A prop's value in a document.
#[derive(Clone, Debug, PartialEq)]
pub enum PropValue {
    Lit(Lit),
    /// `bind <expr> [format [digits]]`.
    Bind(Expr, Option<Format>),
    /// An explicit writable source path (`bind:value={$volume}`).
    Model(String),
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

/// One authored CSS declaration. Values remain token streams until the
/// winning declaration's variables are resolved against its mounted parent.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleDeclaration {
    pub name: String,
    pub value: variables::Value,
    pub source: String,
    pub important: bool,
    /// This longhand's value must be extracted after var() substitution.
    pub pending_shorthand: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StyleRule {
    /// Its cascade layer's rank: 0 for the user-agent sheet, the layer's
    /// place in [`at_rules::Sheet::layers`] plus one, `u32::MAX` unlayered.
    /// Normal declarations of a higher rank win; `!important` ones of a
    /// lower rank.
    pub layer: u32,
    /// Inside `@starting-style`: it applies only to an element's first
    /// style, as the values its transitions start from.
    pub starting: bool,
    pub query: AtQuery,
    pub condition: Option<Expr>,
    /// A sibling or structural selector, matched on the instantiated tree
    /// ([`matching`]) instead of through `condition`.
    pub structure: Option<std::sync::Arc<selectors::Selector>>,
    /// Cook-time reflected values available without custom-property substitution.
    pub props: Vec<PropSet>,
    pub declarations: Vec<StyleDeclaration>,
    pub line: usize,
}

impl PropSet {
    pub fn name(&self) -> &'static str {
        PROPS[self.spec].name
    }
}

/// The enclosing stylesheet query conditions, intersected when nested.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AtQuery {
    pub media: media::Query,
    /// Enclosing `@container` conditions; each picks its own container.
    pub container: Vec<container::Condition>,
}

/// A state value: an animatable prop or a colour.
#[derive(Clone, Debug, PartialEq)]
pub enum StateValue {
    Prop(Prop, f32),
    Length(Prop, Lit),
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
    /// A form's submit (`<button type="submit">`): the form's action,
    /// sent unless one of its constraint-validated controls (`controls`,
    /// by node) is invalid, when the first of them takes the focus
    /// instead. `row`: the form sits in an `{#each}`, so only controls of
    /// the button's row count.
    Submit {
        action: String,
        controls: Vec<usize>,
        row: bool,
    },
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
    /// HTML's drag and drop (`draggable`): a drag of it starts or ends, a
    /// drag moves over it, or one is dropped on it. Their events carry the
    /// dragged element's and the target's row keys.
    DragStart,
    DragOver,
    Drop,
    DragEnd,
    /// Pointer Events and `wheel`: a press, a move while over it or held,
    /// a release, and the wheel over it; their events carry the point in
    /// its box.
    PointerDown,
    PointerMove,
    PointerUp,
    Wheel,
    /// Two touches pinching over it (`onpinch`): its event carries the
    /// scale since they went down (Safari's `GestureEvent.scale`) and
    /// their centre in its box.
    Pinch,
    /// A press held for its `data-hold` ms (`onhold`).
    Hold,
    /// A secondary press or long press asks for its menu (`oncontextmenu`).
    ContextMenu,
    /// A typewriter text finished revealing (`onrevealed`).
    Revealed,
    /// A popover or `<details>` opened or closed (`ontoggle`).
    Toggle,
    /// A `<dialog>` closed (`onclose`).
    Close,
}

impl When {
    /// The HTML event attribute (without `on`) a variant is written as.
    pub(crate) const EVENTS: &[(&str, When)] = &[
        ("dragstart", When::DragStart),
        ("dragover", When::DragOver),
        ("drop", When::Drop),
        ("dragend", When::DragEnd),
        ("pointerdown", When::PointerDown),
        ("pointermove", When::PointerMove),
        ("pointerup", When::PointerUp),
        ("wheel", When::Wheel),
        ("pinch", When::Pinch),
        ("hold", When::Hold),
        ("contextmenu", When::ContextMenu),
        ("revealed", When::Revealed),
        ("toggle", When::Toggle),
        ("close", When::Close),
    ];
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
    For {
        var: String,
        list: Expr,
        key: Expr,
        empty: Vec<usize>,
    },
    /// `if <expr> { … }`.
    If(Expr),
    /// `use <Component> <id> { args }`.
    Use(String),
    Snippet(String),
    Render {
        name: String,
        optional: bool,
    },
    /// `{#key expr}`: its children are made again when `expr` changes.
    Key(Expr),
}

/// One node of a document's tree.
#[derive(Clone, Debug, PartialEq)]
pub struct DocNode {
    pub kind: NodeKind,
    /// Its element id (empty: anonymous).
    pub id: String,
    pub line: usize,
    /// Byte offset of the authored node, used by source-preserving probe edits.
    pub source_offset: usize,
    /// Authored tag before reflected aliases; structural nodes have no tag.
    pub selector_tag: Option<String>,
    /// Authored selector inputs, independent of currently declared CSS rules.
    /// Names retain `class:` and `bind:` prefixes; event handlers are excluded.
    pub selector_attrs: Vec<(String, Expr)>,
    pub props: Vec<PropSet>,
    /// Breakpoint and orientation overrides, in order (later wins).
    pub at: Vec<(AtQuery, Vec<PropSet>)>,
    /// Scoped CSS rules in cascade order, with their reactive match condition.
    pub rules: Vec<StyleRule>,
    /// Scoped rich-text part rules, in cascade order.
    pub span_rules: Vec<SpanRule>,
    pub groups: Vec<DocGroup>,
    pub triggers: Vec<Trigger>,
    /// A component instance's arguments (`use`): its props by name.
    pub args: Vec<(String, PropValue, usize)>,
    pub children: Vec<usize>,
    pub parent: usize,
    /// The role its HTML tag gives it beyond its kind's (`<nav>`:
    /// navigation, `<li>`: listitem).
    pub role: Option<&'static str>,
    /// A heading's level, 1 to 6 (`<h1>`…`<h6>`); 0 for none.
    pub level: u8,
    /// A text element's inline content when it has any markup (`<b>`,
    /// `<a href>`…): its runs, in order; empty for plain text (the `text`
    /// prop alone).
    pub inline: Vec<InlineRun>,
    /// The literal `href`s of its inline `<a>`s, by [`InlineRun::link`].
    pub links: Vec<String>,
    /// Its `style:--name={value}` custom properties, in order: inline
    /// declarations that win over every rule but an `!important` one and
    /// inherit like any custom property.
    pub custom: Vec<(String, Expr)>,
    /// Its `use:name={params}` actions, in order: the game's handler of
    /// each hears it made, its params change and it go (docs/ui.md §2).
    pub uses: Vec<(String, Option<Expr>)>,
    /// Its box pseudo-elements (`before`, `after`, `backdrop`, `picker`)
    /// and the synthetic node of each: a `box` outside [`Self::children`]
    /// whose rules are the pseudo-element's, matched against this element.
    pub pseudo: Vec<(&'static str, usize)>,
}

/// One run of a text element's inline content.
#[derive(Clone, Debug, PartialEq)]
pub struct InlineRun {
    pub text: Expr,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    /// Its colour (an SVG `<tspan fill>`), CSS text.
    pub color: Option<String>,
    /// Its link: an index into [`DocNode::links`], or `usize::MAX` for the
    /// `<a>` element's own `href`.
    pub link: Option<usize>,
    /// The `<span part="name">` it is in: its span's name, which
    /// `text::part(name)` rules style.
    pub part: Option<String>,
    /// An inline `<img>`, `<icon>` or `<kbd data-action>`: its text is the
    /// placeholder U+FFFC.
    pub atom: Option<crate::rich::InlineAtom>,
    /// HTML's `<sub>` or `<sup>`: its `vertical-align`, its size the
    /// smaller one.
    pub valign: Option<crate::VerticalAlign>,
}

/// A prop of the document's contract (`props { gold number "…" }`).
#[derive(Clone, Debug, PartialEq)]
pub struct ContractProp {
    pub name: String,
    pub ty: String,
    pub doc: String,
    /// Its default (`let { gold = 0 } = $props()`): what it reads when the
    /// user passes none.
    pub default: Option<serde_json::Value>,
}

/// A parsed, validated document.
#[derive(Clone, Debug, PartialEq)]
pub struct UiDocument {
    pub kind: DocKind,
    pub name: String,
    /// The asset id it was read from (`ui/shop.svelte`; empty: in memory).
    pub source: String,
    /// The text as written.
    pub text: String,
    /// Cooked base string-table asset IDs, resolved relative to this document.
    pub strings: Vec<String>,
    pub layer: LayerSpec,
    pub props: Vec<ContractProp>,
    pub parts: Vec<String>,
    /// The action bar's hints: an action and what it does here.
    pub actions: Vec<(String, String)>,
    /// Timelines by their own names (the UI defines them as `<doc>/<name>`).
    pub timelines: Vec<(String, Timeline)>,
    /// Viewmodels to preview and push it with (`ui:push:shop:rich`).
    pub fixtures: Vec<(String, serde_json::Value)>,
    /// The script's `$derived` values, in order: names the whole tree reads.
    pub derived: Vec<(String, Expr)>,
    /// Its stylesheets' descriptor at-rules, layers and imports.
    pub sheet: at_rules::Sheet,
    /// Scoped rules for exposed component parts, in cascade order.
    pub part_rules: Vec<StyleRule>,
    pub nodes: Vec<DocNode>,
    pub root: usize,
    /// Author diagnostics that publish promotes to errors.
    pub warnings: Vec<String>,
}

pub(crate) const NONE: usize = usize::MAX;

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
                None => Ease::CubicBezier(crate::theme::EASE_OUT),
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
        "out" => Ease::CubicBezier(crate::theme::EASE_OUT),
        "exit" => Ease::CubicBezier(crate::theme::EASE_EXIT),
        "inOut" => Ease::CubicBezier([0.65, 0.0, 0.35, 1.0]),
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
    const HOW: &str = "a size: fit [min max], grow [min max], fixed <px> (or a number), percent <0–1>, minContent, maxContent";
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
        "minContent" => Size::MinContent,
        "maxContent" => Size::MaxContent,
        "fixed" => Size::Fixed(n(1).ok_or(HOW)?),
        "percent" => Size::Percent(n(1).ok_or(HOW)?),
        "calc" => {
            let op = match vals.get(1).and_then(Value::as_word) {
                Some("sum") => crate::CalcOp::Sum,
                Some("min") => crate::CalcOp::Min,
                Some("max") => crate::CalcOp::Max,
                Some("clamp") => crate::CalcOp::Clamp,
                Some("round") => crate::CalcOp::Round,
                Some("roundUp") => crate::CalcOp::RoundUp,
                Some("roundDown") => crate::CalcOp::RoundDown,
                Some("roundToZero") => crate::CalcOp::RoundToZero,
                Some("mod") => crate::CalcOp::Mod,
                Some("rem") => crate::CalcOp::Rem,
                Some("abs") => crate::CalcOp::Abs,
                Some("hypot") => crate::CalcOp::Hypot,
                _ => return Err(HOW.into()),
            };
            let mut c = crate::Calc {
                op,
                ..Default::default()
            };
            let terms = &vals[2..];
            if terms.is_empty() || terms.len() % 2 != 0 || terms.len() > 6 {
                return Err(HOW.into());
            }
            for (k, arg) in c.args.iter_mut().take(terms.len() / 2).enumerate() {
                *arg = crate::Length {
                    pct: n(2 + 2 * k).ok_or(HOW)?,
                    px: n(3 + 2 * k).ok_or(HOW)?,
                };
            }
            c.count = (terms.len() / 2) as u8;
            Size::Calc(c)
        }
        _ => return Err(HOW.into()),
    })
}

fn parse_paint(vals: &[Value]) -> Result<Paint, String> {
    const HOW: &str =
        "a paint: token <role> (a palette role or one the style declares) or \"#rrggbb[aa]\"";
    match vals {
        [Value::Word(t), Value::Word(role)] if t == "token" => {
            // A palette role, or one the style declares (docs/ui.md §3.1):
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
                .ok_or_else(|| {
                    format!("'{s}' is not a hex colour (#rgb, #rgba, #rrggbb or #rrggbbaa)")
                })
        }
        [Value::String(s)] if s.contains('(') || crate::css_color::is_named(s) => {
            let c = crate::css_color::parse_str(s)?;
            Ok(match c.literal() {
                Some(l) => Paint::Color(l),
                None => Paint::Css(std::sync::Arc::new(c)),
            })
        }
        [Value::Word(role)] | [Value::String(role)]
            if crate::Palette::NAMES.contains(&role.as_str()) =>
        {
            Ok(Paint::Token(role.clone()))
        }
        _ => Err(HOW.into()),
    }
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
        Ty::Tracks => Lit::Tracks(tracks::of_values(vals)?),
        Ty::Names => Lit::Strs(vals.iter().filter_map(as_name).collect()),
        Ty::Spans => return Err("spans takes a read-only expression: spans={$spans}".into()),
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

mod document;
pub(crate) use document::*;
