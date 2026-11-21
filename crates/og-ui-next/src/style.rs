//! CSS compiled by the cook into flat typed records. A rule's declarations
//! become `Step`s: plain 20-byte `Pod` values (an op, a code and four
//! floats) with every `var()` and `calc()` resolved against the style's
//! tokens, shorthands and logical properties expanded. A selector is a
//! compound of interned ids and classes and a state (`:hover`, `:active`,
//! `:focus`, `:disabled`). The compiled sheet is a few flat arrays written
//! into the blob as they are ([`Sheet::to_bytes`]); loading one is copying
//! them back ([`Sheet::from_bytes`]). An element's record is the fold of the
//! rules its names match, made when its names change, never per frame.
//! What the compiler does not take is listed in [`Sheet::skipped`] for the
//! sheet's author to fix, not guessed.

use crate::tokens::Tokens;
use bytemuck::{Pod, Zeroable};
use rune_taffy::CompactStyle;
use rustc_hash::FxHashMap;
use taffy::{
    AlignItems, Dimension, FlexDirection, FlexWrap, JustifyContent, LengthPercentageAuto, Rect,
};

/// An element state a rule may apply under.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum State {
    #[default]
    Base = 0,
    Hover = 1,
    Active = 2,
    Focus = 3,
    Disabled = 4,
}

/// What a rule applies under: a built-in state, or a custom state
/// (`:state(selected)`) that the element's `state:selected={…}` binding sets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Under<S = String> {
    State(State),
    Custom(S),
}

/// A rule's state field with this bit set holds a custom state's name id.
const CUSTOM: u32 = 1 << 31;

impl State {
    fn from_u32(v: u32) -> State {
        match v {
            1 => State::Hover,
            2 => State::Active,
            3 => State::Focus,
            4 => State::Disabled,
            _ => State::Base,
        }
    }
}

/// A linear RGBA colour.
pub type Rgba = [f32; 4];

/// One compiled declaration.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct Step {
    op: u8,
    /// An enum's value, a length's kind ([`len`]) or the sides an inset sets.
    code: u8,
    _pad: [u8; 2],
    v: [f32; 4],
}

mod op {
    pub const DISPLAY: u8 = 1;
    pub const DIRECTION: u8 = 2;
    pub const WRAP: u8 = 3;
    pub const GAP: u8 = 4;
    pub const PAD: u8 = 5;
    pub const MARGIN: u8 = 6;
    pub const WIDTH: u8 = 7;
    pub const HEIGHT: u8 = 8;
    pub const MIN_W: u8 = 9;
    pub const MIN_H: u8 = 10;
    pub const MAX_W: u8 = 11;
    pub const MAX_H: u8 = 12;
    pub const ABSOLUTE: u8 = 13;
    pub const INSET: u8 = 14;
    pub const ALIGN: u8 = 15;
    pub const JUSTIFY: u8 = 16;
    pub const GROW: u8 = 17;
    pub const SHRINK: u8 = 18;
    pub const BASIS: u8 = 19;
    pub const FILL: u8 = 32;
    pub const INK: u8 = 33;
    pub const RADIUS: u8 = 34;
    pub const OPACITY: u8 = 35;
    pub const FONT_SIZE: u8 = 36;
    /// SVG paint; code 1: `currentColor`, the element's ink.
    pub const PAINT: u8 = 37;
    pub const STROKE: u8 = 38;
    pub const STROKE_WIDTH: u8 = 39;
    /// Of the sides a declaration sets (bits of `v[0]`: top, right,
    /// bottom, left), those that add the HUD's safe-area inset (`v[1]`);
    /// code 0: `inset`, 1: `padding`.
    pub const SAFE: u8 = 20;
    /// Overflow, x then y in codes 0..3 (visible, clip, hidden, scroll).
    pub const OVERFLOW: u8 = 21;
    pub const ASPECT: u8 = 22;
    pub const WEIGHT: u8 = 40;
    /// Letter spacing: `v[0]` px plus `v[1]` em.
    pub const TRACKING: u8 = 41;
    /// Line height: code 0 a multiple of the font size, 1 px.
    pub const LINE: u8 = 42;
    /// Wrapping: code 0 wraps, 1 stays on one line.
    pub const NOWRAP: u8 = 43;
    /// Text alignment: code 0 start, 1 center, 2 end.
    pub const TEXT_ALIGN: u8 = 44;
    /// What a line too long for its box shows: code 0 clips, 1 an
    /// ellipsis, 2 scrolls (`-og-scroll`, `v[0]` the cycle and `v[1]` the
    /// pause, ms).
    pub const TEXT_OVERFLOW: u8 = 45;
    pub const TRANSFORM: u8 = 46;
    /// A linear gradient over the fill: `v[0]` its angle (radians, CSS: 0
    /// up, clockwise); its stops are the [`STOP`] steps after it.
    pub const GRADIENT: u8 = 47;
    /// A gradient's stop: `v` its colour, its offset (0..1) in the pad.
    pub const STOP: u8 = 48;
    /// Border widths, as [`PAD`] (code: the sides it sets, 0 all).
    pub const BORDER: u8 = 49;
    pub const BORDER_COLOR: u8 = 50;
    /// A shadow layer: `v` x, y, blur, spread; code 1 inset. Its colour is
    /// the [`SHADOW_COLOR`] step after it.
    pub const SHADOW: u8 = 51;
    pub const SHADOW_COLOR: u8 = 52;
    /// `box-shadow: …` starts over: no layers.
    pub const SHADOWS: u8 = 53;
    /// Outline: `v` width, offset; its colour [`OUTLINE_COLOR`].
    pub const OUTLINE: u8 = 54;
    pub const OUTLINE_COLOR: u8 = 55;
    /// code 0: translate x y; 1: scale x y; 2: rotate (radians); 3: origin
    /// (fractions of the box).
    pub const MOVE: u8 = 56;
    pub const BACKDROP_BLUR: u8 = 57;
    /// Text shadow: `v` x, y, blur; its colour [`TEXT_SHADOW_COLOR`].
    pub const TEXT_SHADOW: u8 = 58;
    pub const TEXT_SHADOW_COLOR: u8 = 59;
    /// Text decoration: code its lines; `v[0]` thickness (0: auto).
    pub const DECORATION: u8 = 60;
    pub const DECORATION_COLOR: u8 = 61;
}

/// A length's kind in a step's code.
mod len {
    pub const AUTO: u8 = 0;
    pub const PX: u8 = 1;
    pub const PERCENT: u8 = 2;
}

fn step(op: u8, code: u8, v: [f32; 4]) -> Step {
    Step {
        op,
        code,
        _pad: [0; 2],
        v,
    }
}

/// An element's text properties. Text inherits: a field its own rules do
/// not set (a bit clear in `set`) is its parent's, applied when the
/// element's record is applied, never matched.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Text {
    pub weight: u16,
    pub tracking_px: f32,
    pub tracking_em: f32,
    /// A multiple of the font size, or with `line_px` CSS px.
    pub line: f32,
    pub line_px: bool,
    pub nowrap: bool,
    /// 0 start, 1 center, 2 end.
    pub align: u8,
    /// 0 clip, 1 ellipsis, 2 scroll.
    pub overflow: u8,
    /// `-og-scroll`'s cycle and pause, ms.
    pub scroll: [f32; 2],
    /// 0 none, 1 uppercase, 2 lowercase, 3 capitalize.
    pub transform: u8,
    /// Which fields its own rules set: bit k for [`Text::WEIGHT`] and on.
    pub set: u16,
    /// Its font size was set by its own rules.
    pub sized: bool,
}

impl Text {
    pub const WEIGHT: u16 = 1;
    pub const TRACKING: u16 = 2;
    pub const LINE: u16 = 4;
    pub const NOWRAP: u16 = 8;
    pub const ALIGN: u16 = 16;
    pub const OVERFLOW: u16 = 32;
    pub const TRANSFORM: u16 = 64;

    /// Its letter spacing at `size`, CSS px.
    pub fn tracking(&self, size: f32) -> f32 {
        self.tracking_px + self.tracking_em * size
    }

    /// Its line height at `size`, CSS px.
    pub fn line_height(&self, size: f32) -> f32 {
        if self.line_px {
            self.line
        } else {
            self.line * size
        }
    }

    /// This text over `parent`'s: the fields it does not set are the
    /// parent's.
    pub fn over(&self, parent: &Text) -> Text {
        let mut t = *parent;
        let own = |bit| self.set & bit != 0;
        if own(Text::WEIGHT) {
            t.weight = self.weight;
        }
        if own(Text::TRACKING) {
            (t.tracking_px, t.tracking_em) = (self.tracking_px, self.tracking_em);
        }
        if own(Text::LINE) {
            (t.line, t.line_px) = (self.line, self.line_px);
        }
        if own(Text::NOWRAP) {
            t.nowrap = self.nowrap;
        }
        if own(Text::ALIGN) {
            t.align = self.align;
        }
        if own(Text::OVERFLOW) {
            (t.overflow, t.scroll) = (self.overflow, self.scroll);
        }
        if own(Text::TRANSFORM) {
            t.transform = self.transform;
        }
        t.set = parent.set | self.set;
        t.sized = self.sized;
        t
    }
}

impl Default for Text {
    fn default() -> Self {
        Text {
            weight: 400,
            tracking_px: 0.0,
            tracking_em: 0.0,
            line: 1.2,
            line_px: false,
            nowrap: false,
            align: 0,
            overflow: 0,
            scroll: [0.0; 2],
            transform: 0,
            set: 0,
            sized: false,
        }
    }
}

/// The most stops a background gradient keeps.
pub const GRADIENT_STOPS: usize = 8;

/// A linear gradient background, over the fill colour.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gradient {
    /// Radians, CSS's: 0 up, clockwise.
    pub angle: f32,
    /// `to <corner>`: x and y, -1 or 1, aimed at the box's corner at paint
    /// time; `[0, 0]`: `angle`.
    pub corner: [i8; 2],
    pub count: u8,
    /// (offset 0..1, linear RGBA), `count` of them.
    pub stops: [(f32, Rgba); GRADIENT_STOPS],
}

impl Gradient {
    pub fn stops(&self) -> &[(f32, Rgba)] {
        &self.stops[..usize::from(self.count)]
    }
}

/// The most box-shadow layers a record keeps.
pub const SHADOWS: usize = 4;

/// A shadow layer (`box-shadow`, `ring-*`, `text-shadow`); a colour of 0
/// alpha draws nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Shadow {
    pub offset: [f32; 2],
    pub blur: f32,
    pub spread: f32,
    pub color: Rgba,
    pub inset: bool,
}

/// An outline: outside the border, `offset` out, taking no space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Outline {
    pub width: f32,
    pub offset: f32,
    pub color: Rgba,
}

/// A paint-time transform around the box (layout never moves).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub translate: [f32; 2],
    pub scale: [f32; 2],
    /// Radians, clockwise.
    pub rotate: f32,
    /// Fractions of the box.
    pub origin: [f32; 2],
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            translate: [0.0; 2],
            scale: [1.0; 2],
            rotate: 0.0,
            origin: [0.5; 2],
        }
    }
}

impl Transform {
    pub fn is_identity(&self) -> bool {
        self.translate == [0.0; 2] && self.scale == [1.0; 2] && self.rotate == 0.0
    }
}

/// A text's decoration lines.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Decoration {
    /// Bits: [`Decoration::UNDERLINE`], `OVERLINE`, `LINE_THROUGH`.
    pub line: u8,
    /// 0 alpha: the text's ink.
    pub color: Rgba,
    /// 0: from the font size.
    pub thickness: f32,
}

impl Decoration {
    pub const UNDERLINE: u8 = 1;
    pub const OVERLINE: u8 = 2;
    pub const LINE_THROUGH: u8 = 4;
}

/// An element's paint: every field the paint pass reads, `Copy`, so motion
/// holds an animated copy of it per node and repaint reads either.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Skin {
    pub fill: Rgba,
    /// A gradient over `fill` (`background: linear-gradient(…), <colour>`).
    pub gradient: Option<Gradient>,
    pub ink: Rgba,
    /// Corner radii: top left, top right, bottom right, bottom left.
    pub radius: [f32; 4],
    /// Border widths, top, right, bottom, left (also in the layout).
    pub border: [f32; 4],
    pub border_color: Rgba,
    /// Box shadows, `shadow_count` of them, first on top.
    pub shadows: [Shadow; SHADOWS],
    pub shadow_count: u8,
    pub outline: Outline,
    pub transform: Transform,
    /// `backdrop-filter: blur()`, CSS px; 0 none.
    pub backdrop_blur: f32,
    pub text_shadow: Shadow,
    pub decoration: Decoration,
    pub opacity: f32,
    /// An SVG shape's fill and stroke paint, and its stroke's width.
    pub paint: Rgba,
    pub stroke: Rgba,
    pub stroke_width: f32,
}

impl Default for Skin {
    fn default() -> Self {
        Skin {
            fill: [0.0; 4],
            gradient: None,
            ink: [0.0, 0.0, 0.0, 1.0],
            radius: [0.0; 4],
            border: [0.0; 4],
            border_color: [0.0; 4],
            shadows: [Shadow::default(); SHADOWS],
            shadow_count: 0,
            outline: Outline::default(),
            transform: Transform::default(),
            backdrop_blur: 0.0,
            text_shadow: Shadow::default(),
            decoration: Decoration::default(),
            opacity: 1.0,
            paint: [0.0, 0.0, 0.0, 1.0],
            stroke: [0.0; 4],
            stroke_width: 1.0,
        }
    }
}

/// An element's computed look: what its rules set, folded in order.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub layout: CompactStyle,
    pub skin: Skin,
    pub font_size: f32,
    /// The sides of its `inset` (0) and `padding` (1) that add the HUD's
    /// safe-area inset (`env(-og-hud-inset-left)`), bits top, right,
    /// bottom, left: the one runtime input a record leaves open.
    pub safe: [u8; 2],
    pub text: Text,
    /// Its ink was set by its own rules; else it inherits its parent's.
    pub inked: bool,
    /// `position: fixed`: placed against the viewport.
    pub fixed: bool,
}

impl Record {
    /// Its layout with the HUD's safe-area `insets` (top, right, bottom,
    /// left) added where its rules read them.
    pub fn layout_in(&self, insets: [f32; 4]) -> CompactStyle {
        if self.safe == [0; 2] {
            return self.layout.clone();
        }
        let mut s = taffy::Style::from(&self.layout);
        let add = |mask: u8, k: usize| if mask & (1 << k) != 0 { insets[k] } else { 0.0 };
        let [t, r, b, l] = [0, 1, 2, 3].map(|k| add(self.safe[0], k));
        let grow = |v: &mut LengthPercentageAuto, by: f32| {
            if by != 0.0 && !v.is_auto() {
                *v = LengthPercentageAuto::length(v.into_raw().value() + by);
            }
        };
        grow(&mut s.inset.top, t);
        grow(&mut s.inset.right, r);
        grow(&mut s.inset.bottom, b);
        grow(&mut s.inset.left, l);
        let [t, r, b, l] = [0, 1, 2, 3].map(|k| add(self.safe[1], k));
        let pad = |v: &mut taffy::LengthPercentage, by: f32| {
            if by != 0.0 {
                *v = taffy::LengthPercentage::length(v.into_raw().value() + by);
            }
        };
        pad(&mut s.padding.top, t);
        pad(&mut s.padding.right, r);
        pad(&mut s.padding.bottom, b);
        pad(&mut s.padding.left, l);
        CompactStyle::from(&s)
    }
}

impl Default for Record {
    fn default() -> Self {
        // Every box is its anchored descendants' containing block, as in
        // og-ui: `position` other than absolute and fixed is relative.
        let mut base = taffy::Style::from(&CompactStyle::default());
        base.position = taffy::Position::Relative;
        Record {
            layout: CompactStyle::from(&base),
            skin: Skin::default(),
            font_size: 16.0,
            safe: [0; 2],
            text: Text::default(),
            inked: false,
            fixed: false,
        }
    }
}

/// A rule: the names it needs and its steps, as spans of the sheet's arrays.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
struct Rule {
    needs: [u32; 2],
    steps: [u32; 2],
    state: u32,
}

/// A compiled sheet.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sheet {
    names: Vec<Box<str>>,
    ids: FxHashMap<Box<str>, u32>,
    rules: Vec<Rule>,
    needs: Vec<u32>,
    steps: Vec<Step>,
    /// Selectors and declarations the compiler did not take (the cook's
    /// warnings; not kept in the blob).
    pub skipped: Vec<String>,
}

impl Sheet {
    /// Compiles `css` against `tokens` (the cook's step).
    pub fn compile(css: &str, tokens: &Tokens) -> Sheet {
        let mut s = Sheet::default();
        for (prelude, body) in crate::sheet::blocks(css) {
            if prelude.trim() == ":root" || prelude.starts_with('@') {
                continue;
            }
            let from = s.steps.len() as u32;
            s.declarations(body, tokens);
            let steps = [from, s.steps.len() as u32 - from];
            for sel in crate::sheet::split_top(prelude, b',') {
                let sel = sel.trim();
                match s.selector(sel) {
                    Some((needs, state)) => {
                        let state = match state {
                            Under::State(st) => st as u32,
                            Under::Custom(name) => CUSTOM | s.intern(&name),
                        };
                        let at = s.needs.len() as u32;
                        s.needs.extend_from_slice(&needs);
                        s.rules.push(Rule {
                            needs: [at, needs.len() as u32],
                            steps,
                            state,
                        });
                    }
                    None => s.skipped.push(format!("selector {sel}")),
                }
            }
        }
        s
    }

    fn intern(&mut self, name: &str) -> u32 {
        if let Some(&i) = self.ids.get(name) {
            return i;
        }
        let i = self.names.len() as u32;
        self.names.push(name.into());
        self.ids.insert(name.into(), i);
        i
    }

    /// The id of an id (`#Clock`) or class (`.hud`) the sheet names.
    pub fn name(&self, name: &str) -> Option<u32> {
        self.ids.get(name).copied()
    }

    fn selector(&mut self, sel: &str) -> Option<(Vec<u32>, Under)> {
        if sel.is_empty() || sel.contains([' ', '>', '+', '~', '[', '*']) {
            return None;
        }
        let (sel, state) = match sel.split_once(':') {
            None => (sel, Under::State(State::Base)),
            Some((s, "hover")) => (s, Under::State(State::Hover)),
            Some((s, "active")) => (s, Under::State(State::Active)),
            Some((s, "focus" | "focus-visible")) => (s, Under::State(State::Focus)),
            Some((s, "disabled")) => (s, Under::State(State::Disabled)),
            Some((s, custom)) => {
                let name = custom.strip_prefix("state(")?.strip_suffix(')')?;
                if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                    return None;
                }
                (s, Under::Custom(name.to_owned()))
            }
        };
        let mut needs = Vec::new();
        let mut rest = sel;
        while !rest.is_empty() {
            if !rest.starts_with(['#', '.']) {
                return None;
            }
            let end = rest[1..].find(['#', '.']).map_or(rest.len(), |e| e + 1);
            needs.push(self.intern(&rest[..end]));
            rest = &rest[end..];
        }
        (!needs.is_empty()).then_some((needs, state))
    }

    fn declarations(&mut self, body: &str, tokens: &Tokens) {
        for (name, value) in crate::sheet::declarations(body) {
            if name.starts_with("--") {
                continue;
            }
            let Some(v) = tokens.substitute_str(value) else {
                self.skipped
                    .push(format!("{name}: {value} (an unknown token)"));
                continue;
            };
            if !expand(&mut self.steps, name, v.trim()) {
                self.skipped.push(format!("{name}: {value}"));
            }
        }
    }

    /// Every rule in sheet order: the names it needs, what it applies
    /// under and its steps.
    pub fn rules<'s>(&'s self, mut f: impl FnMut(&'s [u32], Under<&'s str>, &'s [Step])) {
        for rule in &self.rules {
            let under = if rule.state & CUSTOM != 0 {
                Under::Custom(&*self.names[(rule.state & !CUSTOM) as usize])
            } else {
                Under::State(State::from_u32(rule.state))
            };
            f(
                &self.needs[span(rule.needs)],
                under,
                &self.steps[span(rule.steps)],
            );
        }
    }

    /// The text of name `id` (`.frame`).
    pub fn name_of(&self, id: u32) -> &str {
        &self.names[id as usize]
    }

    /// The record of an element with these names in this state: the base
    /// rules in sheet order, then the state's.
    pub fn record(&self, names: &[u32], state: State) -> Record {
        let mut r = Record::default();
        let passes: &[State] = if state == State::Base {
            &[State::Base]
        } else {
            &[State::Base, state]
        };
        for &pass in passes {
            for rule in &self.rules {
                let needs = &self.needs[span(rule.needs)];
                if rule.state == pass as u32 && needs.iter().all(|n| names.contains(n)) {
                    for s in &self.steps[span(rule.steps)] {
                        fold(&mut r, s);
                    }
                }
            }
        }
        r
    }

    /// The sheet as the blob keeps it: its arrays as they are in memory,
    /// each after its length, then the names.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut put = |bytes: &[u8], count: usize| {
            out.extend_from_slice(&(count as u32).to_le_bytes());
            out.extend_from_slice(bytes);
        };
        put(bytemuck::cast_slice(&self.rules), self.rules.len());
        put(bytemuck::cast_slice(&self.needs), self.needs.len());
        put(bytemuck::cast_slice(&self.steps), self.steps.len());
        let names = self.names.join("\n");
        put(names.as_bytes(), names.len());
        out
    }

    /// A sheet [`Sheet::to_bytes`] wrote.
    pub fn from_bytes(bytes: &[u8]) -> Option<Sheet> {
        let mut at = 0;
        let mut take = |size: usize| -> Option<&[u8]> {
            let n = u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?) as usize;
            let b = bytes.get(at + 4..at + 4 + n * size)?;
            at += 4 + n * size;
            Some(b)
        };
        let rules = collect_pod(take(size_of::<Rule>())?);
        let needs = collect_pod(take(4)?);
        let steps = collect_pod(take(size_of::<Step>())?);
        let names_text = std::str::from_utf8(take(1)?).ok()?;
        let names: Vec<Box<str>> = if names_text.is_empty() {
            Vec::new()
        } else {
            names_text.split('\n').map(Into::into).collect()
        };
        let ids = names
            .iter()
            .enumerate()
            .map(|(i, n)| (n.clone(), i as u32))
            .collect();
        Some(Sheet {
            names,
            ids,
            rules,
            needs,
            steps,
            skipped: Vec::new(),
        })
    }
}

/// `bytes` copied into a new, aligned `Vec<T>`.
fn collect_pod<T: Pod>(bytes: &[u8]) -> Vec<T> {
    let mut v = vec![T::zeroed(); bytes.len() / size_of::<T>()];
    bytemuck::cast_slice_mut::<T, u8>(&mut v).copy_from_slice(bytes);
    v
}

fn span([start, n]: [u32; 2]) -> std::ops::Range<usize> {
    start as usize..(start + n) as usize
}

fn dim(s: &Step) -> Dimension {
    match s.code {
        len::PX => Dimension::length(s.v[0]),
        len::PERCENT => Dimension::percent(s.v[0]),
        _ => Dimension::auto(),
    }
}

fn lpa(s: &Step) -> LengthPercentageAuto {
    match s.code {
        len::PX => LengthPercentageAuto::length(s.v[0]),
        len::PERCENT => LengthPercentageAuto::percent(s.v[0]),
        _ => LengthPercentageAuto::auto(),
    }
}

const DISPLAYS: [taffy::Display; 4] = [
    taffy::Display::Flex,
    taffy::Display::Grid,
    taffy::Display::Block,
    taffy::Display::None,
];
const DIRECTIONS: [FlexDirection; 4] = [
    FlexDirection::Row,
    FlexDirection::Column,
    FlexDirection::RowReverse,
    FlexDirection::ColumnReverse,
];
const WRAPS: [FlexWrap; 3] = [FlexWrap::NoWrap, FlexWrap::Wrap, FlexWrap::WrapReverse];
const ALIGNS: [AlignItems; 5] = [
    AlignItems::CENTER,
    AlignItems::FLEX_START,
    AlignItems::FLEX_END,
    AlignItems::STRETCH,
    AlignItems::BASELINE,
];
const JUSTIFIES: [JustifyContent; 6] = [
    JustifyContent::CENTER,
    JustifyContent::FLEX_START,
    JustifyContent::FLEX_END,
    JustifyContent::SPACE_BETWEEN,
    JustifyContent::SPACE_AROUND,
    JustifyContent::SPACE_EVENLY,
];

pub(crate) fn fold(r: &mut Record, s: &Step) {
    let l = &mut r.layout;
    let c = s.code as usize;
    let [a, b, cc, d] = s.v;
    match s.op {
        op::DISPLAY => {
            let v = DISPLAYS[c];
            l.modify_cold(|t| t.display = v);
        }
        op::DIRECTION => l.set_flex_direction(DIRECTIONS[c]),
        op::WRAP => l.set_flex_wrap(WRAPS[c]),
        op::GAP => {
            // Code: the axes it sets (bit 0 column, bit 1 row); 0 both.
            let m = if s.code == 0 { 3 } else { s.code };
            let g = l.gap();
            let keep = |v: taffy::LengthPercentage| v.into_raw().value();
            l.set_gap(
                if m & 1 != 0 { a } else { keep(g.width) },
                if m & 2 != 0 { b } else { keep(g.height) },
            );
        }
        op::SAFE => r.safe[c & 1] = (r.safe[c & 1] & !(a as u8)) | b as u8,
        // Code: the sides it sets (top, right, bottom, left); 0 all four.
        op::PAD => {
            let m = if s.code == 0 { 15 } else { s.code };
            if m & 1 != 0 {
                l.set_pad_top(a);
            }
            if m & 2 != 0 {
                l.set_pad_right(b);
            }
            if m & 4 != 0 {
                l.set_pad_bottom(cc);
            }
            if m & 8 != 0 {
                l.set_pad_left(d);
            }
        }
        op::MARGIN => {
            let m = if s.code == 0 { 15 } else { s.code };
            // NaN is `auto`: the free space on that side, as flex places it.
            if [a, b, cc, d].iter().any(|v| v.is_nan()) {
                let side = |v: f32| {
                    if v.is_nan() {
                        LengthPercentageAuto::auto()
                    } else {
                        LengthPercentageAuto::length(v)
                    }
                };
                l.modify_cold(|st| {
                    if m & 1 != 0 {
                        st.margin.top = side(a);
                    }
                    if m & 2 != 0 {
                        st.margin.right = side(b);
                    }
                    if m & 4 != 0 {
                        st.margin.bottom = side(cc);
                    }
                    if m & 8 != 0 {
                        st.margin.left = side(d);
                    }
                });
                return;
            }
            if m & 1 != 0 {
                l.set_margin_top(a);
            }
            if m & 2 != 0 {
                l.set_margin_right(b);
            }
            if m & 4 != 0 {
                l.set_margin_bottom(cc);
            }
            if m & 8 != 0 {
                l.set_margin_left(d);
            }
        }
        op::WIDTH => l.set_width(dim(s)),
        op::HEIGHT => l.set_height(dim(s)),
        op::MIN_W => l.set_min_width(lpa(s)),
        op::MIN_H => l.set_min_height(lpa(s)),
        op::MAX_W => l.set_max_width(lpa(s)),
        op::MAX_H => l.set_max_height(lpa(s)),
        op::ABSOLUTE => {
            l.set_position_absolute();
            r.fixed = s.code == 1;
        }
        op::INSET => {
            // The sides it sets over those the record has.
            let had = l.inset();
            // NaN is `auto` (`inset: 4px auto auto 4px`, an anchor's).
            let side = |k: usize, was: LengthPercentageAuto| {
                if s.code & (1 << k) != 0 && s.v[k].is_nan() {
                    LengthPercentageAuto::auto()
                } else if s.code & (1 << k) != 0 {
                    LengthPercentageAuto::length(s.v[k])
                } else {
                    was
                }
            };
            l.set_absolute_inset(Rect {
                top: side(0, had.top),
                right: side(1, had.right),
                bottom: side(2, had.bottom),
                left: side(3, had.left),
            });
        }
        op::ALIGN => l.set_align_items(ALIGNS[c]),
        op::JUSTIFY => l.set_justify_content(JUSTIFIES[c]),
        op::GROW => l.set_flex_grow(a),
        op::SHRINK => l.set_flex_shrink(a),
        op::BASIS => l.set_flex_basis(dim(s)),
        op::FILL => {
            r.skin.fill = s.v;
            r.skin.gradient = None;
        }
        op::GRADIENT => {
            r.skin.gradient = Some(Gradient {
                angle: a,
                corner: [b as i8, cc as i8],
                count: 0,
                stops: [(0.0, [0.0; 4]); GRADIENT_STOPS],
            })
        }
        op::STOP => {
            if let Some(g) = &mut r.skin.gradient
                && usize::from(g.count) < GRADIENT_STOPS
            {
                let offset = f32::from(u16::from_le_bytes(s._pad)) / f32::from(u16::MAX);
                g.stops[usize::from(g.count)] = (offset, s.v);
                g.count += 1;
            }
        }
        op::INK => {
            r.skin.ink = s.v;
            r.inked = true;
        }
        op::RADIUS => r.skin.radius = s.v,
        op::BORDER => {
            let m = if s.code == 0 { 15 } else { s.code };
            for k in 0..4 {
                if m & (1 << k) != 0 {
                    r.skin.border[k] = s.v[k];
                }
            }
            r.layout.set_border(r.skin.border);
        }
        op::BORDER_COLOR => r.skin.border_color = s.v,
        op::SHADOWS => r.skin.shadow_count = 0,
        op::SHADOW => {
            if usize::from(r.skin.shadow_count) < SHADOWS {
                r.skin.shadows[usize::from(r.skin.shadow_count)] = Shadow {
                    offset: [a, b],
                    blur: cc,
                    spread: d,
                    color: [0.0; 4],
                    inset: s.code == 1,
                };
                r.skin.shadow_count += 1;
            }
        }
        op::SHADOW_COLOR => {
            if let Some(k) = usize::from(r.skin.shadow_count).checked_sub(1) {
                r.skin.shadows[k].color = s.v;
            }
        }
        // Code 0: the width (`outline`), 1: the offset.
        op::OUTLINE => match s.code {
            0 => r.skin.outline.width = a,
            _ => r.skin.outline.offset = b,
        },
        op::OUTLINE_COLOR => r.skin.outline.color = s.v,
        op::MOVE => match s.code {
            0 => r.skin.transform.translate = [a, b],
            1 => r.skin.transform.scale = [a, b],
            2 => r.skin.transform.rotate = a,
            _ => r.skin.transform.origin = [a, b],
        },
        op::BACKDROP_BLUR => r.skin.backdrop_blur = a,
        op::TEXT_SHADOW => {
            r.skin.text_shadow = Shadow {
                offset: [a, b],
                blur: cc,
                ..Shadow::default()
            }
        }
        op::TEXT_SHADOW_COLOR => r.skin.text_shadow.color = s.v,
        op::DECORATION => {
            r.skin.decoration.line = s.code;
            r.skin.decoration.thickness = a;
        }
        op::DECORATION_COLOR => r.skin.decoration.color = s.v,
        op::OPACITY => r.skin.opacity = a,
        op::FONT_SIZE => {
            r.font_size = a;
            r.text.sized = true;
        }
        op::OVERFLOW => {
            use taffy::Overflow as O;
            let o = |k: u8| [O::Visible, O::Clip, O::Hidden, O::Scroll][usize::from(k & 3)];
            // A step of an old blob names no axes: both.
            let axes = if a == 0.0 { 3 } else { a as u8 };
            let had = taffy::CoreStyle::overflow(&r.layout);
            r.layout.set_overflow(
                if axes & 1 != 0 { o(s.code) } else { had.x },
                if axes & 2 != 0 { o(s.code >> 2) } else { had.y },
            );
        }
        op::ASPECT => r.layout.set_aspect_ratio((a > 0.0).then_some(a)),
        op::WEIGHT => {
            r.text.weight = a as u16;
            r.text.set |= Text::WEIGHT;
        }
        op::TRACKING => {
            (r.text.tracking_px, r.text.tracking_em) = (a, b);
            r.text.set |= Text::TRACKING;
        }
        op::LINE => {
            (r.text.line, r.text.line_px) = (a, c == 1);
            r.text.set |= Text::LINE;
        }
        op::NOWRAP => {
            r.text.nowrap = c == 1;
            r.text.set |= Text::NOWRAP;
        }
        op::TEXT_ALIGN => {
            r.text.align = c as u8;
            r.text.set |= Text::ALIGN;
        }
        op::TEXT_OVERFLOW => {
            (r.text.overflow, r.text.scroll) = (c as u8, [a, b]);
            r.text.set |= Text::OVERFLOW;
        }
        op::TRANSFORM => {
            r.text.transform = c as u8;
            r.text.set |= Text::TRANSFORM;
        }
        op::PAINT => r.skin.paint = if c == 1 { r.skin.ink } else { [a, b, cc, d] },
        op::STROKE => r.skin.stroke = if c == 1 { r.skin.ink } else { [a, b, cc, d] },
        op::STROKE_WIDTH => r.skin.stroke_width = a,
        _ => {}
    }
}

/// A length in CSS px: `12px`, `1.5rem` (16 px), `0`, or a `calc()` that
/// reduces to one.
fn px(v: &str) -> Option<f32> {
    let reduced;
    let v = if v.contains('(') {
        reduced = og_css::calc::reduce(&rem_to_px(v)).ok()?;
        reduced.trim()
    } else {
        v
    };
    if v == "0" {
        return Some(0.0);
    }
    if let Some(n) = v.strip_suffix("px") {
        return n.trim().parse().ok();
    }
    if let Some(n) = v.strip_suffix("rem") {
        return n.trim().parse::<f32>().ok().map(|n| n * 16.0);
    }
    None
}

/// `text` with each `<n>rem` written as `<n * 16>px`, so `calc()` reduces it.
fn rem_to_px(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("rem") {
        let num_start = rest[..at]
            .rfind(|c: char| !(c.is_ascii_digit() || c == '.'))
            .map_or(0, |i| i + 1);
        let after = rest[at + 3..].chars().next();
        match rest[num_start..at].parse::<f64>() {
            Ok(n) if num_start < at && !after.is_some_and(|c| c.is_alphanumeric() || c == '-') => {
                out.push_str(&rest[..num_start]);
                out.push_str(&format!("{}px", n * 16.0));
            }
            _ => out.push_str(&rest[..at + 3]),
        }
        rest = &rest[at + 3..];
    }
    out.push_str(rest);
    out
}

/// A length's kind and value: `auto`, a percentage or px.
fn length(v: &str) -> Option<(u8, f32)> {
    if v == "auto" {
        return Some((len::AUTO, 0.0));
    }
    if let Some(p) = v.strip_suffix('%') {
        return p
            .trim()
            .parse::<f32>()
            .ok()
            .map(|p| (len::PERCENT, p / 100.0));
    }
    px(v).map(|x| (len::PX, x))
}

fn words(v: &str) -> impl Iterator<Item = &str> {
    crate::sheet::split_top(v, b' ').filter(|p| !p.is_empty())
}

/// Up to four lengths in CSS's top, right, bottom, left shorthand order.
fn sides(v: &str) -> Option<[f32; 4]> {
    sides_with(v, px)
}

/// A margin's or an inset's length: `auto` is NaN (a margin takes the
/// free space; an inset leaves the side to the others).
fn margin_px(w: &str) -> Option<f32> {
    if w == "auto" { Some(f32::NAN) } else { px(w) }
}

fn sides_with(v: &str, len: impl Fn(&str) -> Option<f32>) -> Option<[f32; 4]> {
    let parts: Vec<f32> = words(v).map(len).collect::<Option<_>>()?;
    Some(match parts[..] {
        [a] => [a; 4],
        [a, b] => [a, b, a, b],
        [a, b, c] => [a, b, c, b],
        [a, b, c, d] => [a, b, c, d],
        _ => return None,
    })
}

fn color(v: &str) -> Option<Rgba> {
    og_css::color::parse_str(v)
        .ok()?
        .literal()
        .map(|c| c.linear())
}

/// `v` split at its commas outside parentheses.
fn top_level(v: &str, at: char) -> impl Iterator<Item = &str> {
    let mut depth = 0i32;
    let mut start = 0;
    let mut out = Vec::new();
    for (i, ch) in v.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            c if c == at && depth == 0 => {
                out.push(v[start..i].trim());
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    out.push(v[start..].trim());
    out.into_iter().filter(|s| !s.is_empty())
}

/// `background`: a colour, `none`, or layers whose last is a colour and
/// whose one other is a `linear-gradient()`. Another image or a second
/// gradient is not taken.
fn background(steps: &mut Vec<Step>, v: &str) -> bool {
    if v == "none" {
        steps.push(step(op::FILL, 0, [0.0; 4]));
        return true;
    }
    let layers: Vec<&str> = top_level(v, ',').collect();
    let (base, images) = match layers.split_last() {
        Some((last, rest)) if !last.contains("gradient(") => (color(last), rest),
        _ => (Some([0.0; 4]), &layers[..]),
    };
    let Some(base) = base else { return false };
    let gradient = match images {
        [] => None,
        [g] => match linear_gradient(g) {
            Some(g) => Some(g),
            None => return false,
        },
        _ => return false,
    };
    steps.push(step(op::FILL, 0, base));
    if let Some((angle, corner, stops)) = gradient {
        let [x, y] = corner.map(f32::from);
        steps.push(step(op::GRADIENT, 0, [angle, x, y, 0.0]));
        for (offset, c) in stops {
            let mut s = step(op::STOP, 0, c);
            s._pad = ((offset.clamp(0.0, 1.0) * f32::from(u16::MAX)).round() as u16).to_le_bytes();
            steps.push(s);
        }
    }
    true
}

/// A `linear-gradient(…)`: its angle (radians) and stops (offset 0..1,
/// linear RGBA), unplaced stops spread evenly between their neighbours.
/// A parsed linear gradient: its angle, its `to` corner and its stops.
type Linear = (f32, [i8; 2], Vec<(f32, Rgba)>);

fn linear_gradient(v: &str) -> Option<Linear> {
    let body = v.strip_prefix("linear-gradient(")?.strip_suffix(')')?;
    let mut args: Vec<&str> = top_level(body, ',').collect();
    let first = *args.first()?;
    let mut corner = [0i8; 2];
    let angle = if let Some(to) = first.strip_prefix("to ") {
        let (mut x, mut y) = (0.0f32, 0.0f32);
        for w in to.split_whitespace() {
            match w {
                "top" => y = -1.0,
                "bottom" => y = 1.0,
                "left" => x = -1.0,
                "right" => x = 1.0,
                _ => return None,
            }
        }
        // A corner's angle depends on the box: paint aims it (`corner`);
        // 45° stands for it until then.
        if x != 0.0 && y != 0.0 {
            corner = [x as i8, y as i8];
        }
        use std::f32::consts::{FRAC_PI_2 as Q, FRAC_PI_4 as E, PI};
        match (x as i8, y as i8) {
            (0, -1) => Some(0.0),
            (1, -1) => Some(E),
            (1, 0) => Some(Q),
            (1, 1) => Some(Q + E),
            (0, 1) => Some(PI),
            (-1, 1) => Some(PI + E),
            (-1, 0) => Some(PI + Q),
            (-1, -1) => Some(PI + Q + E),
            _ => return None,
        }
    } else {
        [
            ("deg", 1.0f32.to_radians()),
            ("grad", 0.9f32.to_radians()),
            ("rad", 1.0),
            ("turn", std::f32::consts::TAU),
        ]
        .iter()
        .find_map(|(unit, k)| {
            first
                .strip_suffix(unit)
                .and_then(|n| n.trim().parse::<f32>().ok())
                .map(|n| n * k)
        })
    };
    if angle.is_some() {
        args.remove(0);
    }
    let angle = angle.unwrap_or(std::f32::consts::PI);
    if args.is_empty() || args.len() > GRADIENT_STOPS {
        return None;
    }
    let mut stops: Vec<(Option<f32>, Rgba)> = Vec::with_capacity(args.len());
    for a in args {
        // `<colour> [<percentage>]`: the colour may hold spaces.
        let (c, at) = match a.rsplit_once(' ') {
            Some((c, p)) if p.ends_with('%') => (
                c,
                Some(p.trim_end_matches('%').parse::<f32>().ok()? / 100.0),
            ),
            _ => (a, None),
        };
        stops.push((at, color(c.trim())?));
    }
    let n = stops.len();
    if stops[0].0.is_none() {
        stops[0].0 = Some(0.0);
    }
    if stops[n - 1].0.is_none() {
        stops[n - 1].0 = Some(1.0);
    }
    let mut out = Vec::with_capacity(n);
    let mut last = 0.0f32;
    let mut k = 0;
    while k < n {
        if let Some(at) = stops[k].0 {
            last = at.max(last);
            out.push((last, stops[k].1));
            k += 1;
            continue;
        }
        let end = (k..n).find(|&j| stops[j].0.is_some()).unwrap_or(n - 1);
        let to = stops[end].0.unwrap_or(1.0).max(last);
        let gap = (end - k + 1) as f32;
        for (i, j) in (k..end).enumerate() {
            out.push((last + (to - last) * (i + 1) as f32 / gap, stops[j].1));
        }
        k = end;
    }
    Some((angle, corner, out))
}

/// A `border` or `outline` line: `<width> [<style>] [<colour>]` in any
/// order, `none` or `0`; its width and colour, if it names one.
fn border_line(v: &str) -> Option<(f32, Option<Rgba>)> {
    if v == "none" || v == "0" {
        return Some((0.0, None));
    }
    let (mut w, mut c) = (None, None);
    for word in top_level(v, ' ') {
        match word {
            "solid" | "dashed" | "dotted" | "double" => {}
            "none" | "hidden" => w = Some(0.0),
            "thin" => w = Some(1.0),
            "medium" => w = Some(3.0),
            "thick" => w = Some(5.0),
            word => match px(word) {
                Some(x) => w = Some(x),
                None => c = Some(color(word)?),
            },
        }
    }
    // CSS's initial width is medium.
    Some((w.unwrap_or(3.0), c))
}

/// One shadow layer: `[inset] <x> <y> [<blur> [<spread>]] [<colour>]`;
/// whether it is inset, its lengths and its colour (black when unnamed).
fn shadow_layer(v: &str) -> Option<(bool, [f32; 4], Rgba)> {
    let (mut inset, mut lens, mut c) = (false, Vec::with_capacity(4), None);
    for word in top_level(v, ' ') {
        if word == "inset" {
            inset = true;
        } else if let Some(x) = px(word) {
            lens.push(x);
        } else {
            c = Some(color(word)?);
        }
    }
    if !(2..=4).contains(&lens.len()) {
        return None;
    }
    lens.resize(4, 0.0);
    Some((
        inset,
        [lens[0], lens[1], lens[2], lens[3]],
        c.unwrap_or([0.0, 0.0, 0.0, 1.0]),
    ))
}

/// `box-shadow`: `none`, or up to [`SHADOWS`] layers.
fn shadows(steps: &mut Vec<Step>, v: &str) -> bool {
    let mut layers = Vec::new();
    if v != "none" {
        for l in top_level(v, ',') {
            match shadow_layer(l) {
                Some(x) => layers.push(x),
                None => return false,
            }
        }
    }
    if layers.len() > SHADOWS {
        return false;
    }
    steps.push(step(op::SHADOWS, 0, [0.0; 4]));
    for (inset, v, c) in layers {
        steps.push(step(op::SHADOW, u8::from(inset), v));
        steps.push(step(op::SHADOW_COLOR, 0, c));
    }
    true
}

/// An angle in radians (`45deg`, `0.25turn`, `1rad`, `0`).
fn angle(v: &str) -> Option<f32> {
    if v == "0" {
        return Some(0.0);
    }
    [
        ("deg", 1.0f32.to_radians()),
        ("grad", 0.9f32.to_radians()),
        ("rad", 1.0),
        ("turn", std::f32::consts::TAU),
    ]
    .iter()
    .find_map(|(unit, k)| {
        v.strip_suffix(unit)?
            .trim()
            .parse::<f32>()
            .ok()
            .map(|n| n * k)
    })
}

fn index<T: Copy>(table: &[(&str, T)], v: &str) -> Option<u8> {
    table.iter().position(|(n, _)| *n == v).map(|i| i as u8)
}

/// Lays the step a side-wise property (`padding-left`, `margin-inline`)
/// sets over the last full one of `op`, or a new one from zero.
/// A step of `o` that sets the sides `which` to `x` and leaves the others
/// as the element's earlier rules set them (a step never reads another
/// step: the array holds every element's).
fn set_sides(steps: &mut Vec<Step>, o: u8, which: &[usize], x: f32) {
    let mut v = [0.0; 4];
    let mut mask = 0u8;
    for &k in which {
        v[k] = x;
        mask |= 1 << k;
    }
    steps.push(step(o, mask, v));
}

/// Expands one declaration into steps; false when it is not taken.
pub(crate) fn expand(steps: &mut Vec<Step>, name: &str, v: &str) -> bool {
    // The HUD's safe-area inset is the one value a record leaves to
    // runtime: compile it as 0, and after each `inset` or `padding`
    // declaration write which of the sides it sets read it.
    let sides = |n: &str| match n {
        "top" | "padding-top" => 1,
        "right" | "padding-right" => 2,
        "bottom" | "padding-bottom" => 4,
        "left" | "padding-left" => 8,
        "padding-inline" => 10,
        "padding-block" => 5,
        _ => 15,
    };
    let target = match name {
        "inset" | "top" | "right" | "bottom" | "left" => Some(0),
        n if n.starts_with("padding") => Some(1),
        _ => None,
    };
    if let Some(target) = target {
        const ENV: &str = "env(-og-hud-inset-";
        let (mut text, mut rest, mut mask) = (String::new(), v, 0u8);
        while let Some(at) = rest.find(ENV) {
            text.push_str(&rest[..at]);
            let after = &rest[at + ENV.len()..];
            let Some(end) = after.find(')') else {
                return false;
            };
            let Some(k) = ["top", "right", "bottom", "left"]
                .iter()
                .position(|s| *s == &after[..end])
            else {
                return false;
            };
            mask |= 1 << k;
            text.push_str("0px");
            rest = &after[end + 1..];
        }
        let v = if mask == 0 {
            v
        } else {
            text.push_str(rest);
            text.as_str()
        };
        if !expand_value(steps, name, v) {
            return false;
        }
        let set = sides(name);
        steps.push(step(
            op::SAFE,
            target,
            [f32::from(set), f32::from(mask & set), 0.0, 0.0],
        ));
        return true;
    }
    expand_value(steps, name, v)
}

fn expand_value(steps: &mut Vec<Step>, name: &str, v: &str) -> bool {
    let mut push = |o: u8, code: u8, x: [f32; 4]| {
        steps.push(step(o, code, x));
        true
    };
    let keyword = |table: &[&str]| table.iter().position(|n| *n == v).map(|i| i as u8);
    match name {
        "display" => match keyword(&["flex", "grid", "block", "none"]) {
            Some(c) => push(op::DISPLAY, c, [0.0; 4]),
            None => false,
        },
        "flex-direction" => match keyword(&["row", "column", "row-reverse", "column-reverse"]) {
            Some(c) => push(op::DIRECTION, c, [0.0; 4]),
            None => false,
        },
        "flex-wrap" => match keyword(&["nowrap", "wrap", "wrap-reverse"]) {
            Some(c) => push(op::WRAP, c, [0.0; 4]),
            None => false,
        },
        "gap" => match sides(v) {
            Some(s) => push(op::GAP, 0, [s[1], s[0], 0.0, 0.0]),
            None => false,
        },
        "row-gap" | "column-gap" => {
            let Some(x) = px(v) else { return false };
            let row = usize::from(name == "row-gap");
            let mut g = [0.0; 4];
            g[row] = x;
            steps.push(step(op::GAP, 1 << row, g));
            true
        }
        "padding" => sides(v).is_some_and(|s| push(op::PAD, 0, s)),
        "margin" => sides_with(v, margin_px).is_some_and(|s| push(op::MARGIN, 0, s)),
        "padding-top" | "padding-right" | "padding-bottom" | "padding-left" | "padding-inline"
        | "padding-block" | "margin-top" | "margin-right" | "margin-bottom" | "margin-left"
        | "margin-inline" | "margin-block" => {
            let (o, side) = name.split_once('-').unwrap();
            let o = if o == "padding" { op::PAD } else { op::MARGIN };
            let which: &[usize] = match side {
                "top" => &[0],
                "right" => &[1],
                "bottom" => &[2],
                "left" => &[3],
                // Horizontal writing: inline is left and right, block top and bottom.
                "inline" => &[1, 3],
                _ => &[0, 2],
            };
            let len = if o == op::MARGIN { margin_px } else { px };
            let vals: Vec<f32> = match words(v).map(len).collect::<Option<Vec<f32>>>() {
                Some(v) => v,
                None => return false,
            };
            match (which.len(), &vals[..]) {
                (_, &[x]) => set_sides(steps, o, which, x),
                // `-inline: start end` is left then right; `-block: start end` top then bottom.
                (2, &[x, y]) => {
                    let (start, end) = if which == [1, 3] { (3, 1) } else { (0, 2) };
                    set_sides(steps, o, &[start], x);
                    set_sides(steps, o, &[end], y);
                }
                _ => return false,
            }
            true
        }
        "width" | "inline-size" | "height" | "block-size" | "min-width" | "min-inline-size"
        | "min-height" | "min-block-size" | "max-width" | "max-inline-size" | "max-height"
        | "max-block-size" | "flex-basis" => {
            let o = match name {
                "width" | "inline-size" => op::WIDTH,
                "height" | "block-size" => op::HEIGHT,
                "min-width" | "min-inline-size" => op::MIN_W,
                "min-height" | "min-block-size" => op::MIN_H,
                "max-width" | "max-inline-size" => op::MAX_W,
                "max-height" | "max-block-size" => op::MAX_H,
                _ => op::BASIS,
            };
            // `none` as a maximum is no limit.
            let v = if v == "none" { "auto" } else { v };
            length(v).is_some_and(|(k, x)| push(o, k, [x, 0.0, 0.0, 0.0]))
        }
        "position" => match v {
            "absolute" => push(op::ABSOLUTE, 0, [0.0; 4]),
            // Code 1: placed against the viewport, not its parent.
            "fixed" => push(op::ABSOLUTE, 1, [0.0; 4]),
            "relative" | "static" => true,
            _ => false,
        },
        "top" | "right" | "bottom" | "left" | "inset" => {
            // Only the sides it names: fold keeps the element's others.
            let last = if name == "inset" {
                let Some(s) = sides_with(v, margin_px) else {
                    return false;
                };
                step(op::INSET, 0b1111, s)
            } else {
                let k = ["top", "right", "bottom", "left"]
                    .iter()
                    .position(|s| *s == name)
                    .unwrap();
                let Some(x) = margin_px(v) else { return false };
                let mut v = [0.0; 4];
                v[k] = x;
                step(op::INSET, 1 << k, v)
            };
            steps.push(last);
            true
        }
        "align-items" => {
            let table = [
                ("center", ()),
                ("start", ()),
                ("end", ()),
                ("stretch", ()),
                ("baseline", ()),
                ("flex-start", ()),
                ("flex-end", ()),
            ];
            match index(&table, v) {
                Some(i) => push(op::ALIGN, [0, 1, 2, 3, 4, 1, 2][i as usize], [0.0; 4]),
                None => false,
            }
        }
        "justify-content" => {
            let table = [
                ("center", ()),
                ("start", ()),
                ("end", ()),
                ("space-between", ()),
                ("space-around", ()),
                ("space-evenly", ()),
                ("flex-start", ()),
                ("flex-end", ()),
            ];
            match index(&table, v) {
                Some(i) => push(op::JUSTIFY, [0, 1, 2, 3, 4, 5, 1, 2][i as usize], [0.0; 4]),
                None => false,
            }
        }
        "flex-grow" => v
            .parse()
            .is_ok_and(|g| push(op::GROW, 0, [g, 0.0, 0.0, 0.0])),
        "flex-shrink" => v
            .parse()
            .is_ok_and(|g| push(op::SHRINK, 0, [g, 0.0, 0.0, 0.0])),
        "flex" => {
            // `none`, `auto` and `<grow> [<shrink>] [<basis>]`.
            let (g, sh, basis) = match v {
                "none" => (0.0, 0.0, "auto"),
                "auto" => (1.0, 1.0, "auto"),
                _ => {
                    let w: Vec<&str> = words(v).collect();
                    let num = |s: &str| s.parse::<f32>().ok();
                    match w[..] {
                        [g] if num(g).is_some() => (num(g).unwrap(), 1.0, "0px"),
                        [g, s] if num(s).is_some() => {
                            (num(g).unwrap_or(1.0), num(s).unwrap(), "0px")
                        }
                        [g, b] => (num(g).unwrap_or(1.0), 1.0, b),
                        [g, s, b] => (num(g).unwrap_or(1.0), num(s).unwrap_or(1.0), b),
                        _ => return false,
                    }
                }
            };
            let Some((k, x)) = length(basis) else {
                return false;
            };
            push(op::GROW, 0, [g, 0.0, 0.0, 0.0]);
            push(op::SHRINK, 0, [sh, 0.0, 0.0, 0.0]);
            push(op::BASIS, k, [x, 0.0, 0.0, 0.0])
        }
        "background-color" => color(v).is_some_and(|c| push(op::FILL, 0, c)),
        "background" => background(steps, v),
        "color" => color(v).is_some_and(|c| push(op::INK, 0, c)),
        "border-radius" => sides(v).is_some_and(|c| push(op::RADIUS, 0, c)),
        "border" | "border-top" | "border-right" | "border-bottom" | "border-left" => {
            let Some((w, c)) = border_line(v) else {
                return false;
            };
            let mask = match name {
                "border-top" => 1,
                "border-right" => 2,
                "border-bottom" => 4,
                "border-left" => 8,
                _ => 0,
            };
            push(op::BORDER, mask, [w; 4]);
            match c {
                Some(c) => push(op::BORDER_COLOR, 0, c),
                None => true,
            }
        }
        "border-width" => sides(v).is_some_and(|w| push(op::BORDER, 0, w)),
        "border-color" => color(v).is_some_and(|c| push(op::BORDER_COLOR, 0, c)),
        "box-shadow" => shadows(steps, v),
        "outline" => match border_line(v) {
            Some((w, c)) => {
                push(op::OUTLINE, 0, [w, 0.0, 0.0, 0.0]);
                c.is_none_or(|c| push(op::OUTLINE_COLOR, 0, c))
            }
            None => false,
        },
        "outline-offset" => px(v).is_some_and(|o| push(op::OUTLINE, 1, [0.0, o, 0.0, 0.0])),
        "outline-color" => color(v).is_some_and(|c| push(op::OUTLINE_COLOR, 0, c)),
        "translate" => {
            let w: Vec<f32> = match words(v).map(px).collect::<Option<_>>() {
                Some(w) => w,
                None => return false,
            };
            match w[..] {
                [x] => push(op::MOVE, 0, [x, 0.0, 0.0, 0.0]),
                [x, y] => push(op::MOVE, 0, [x, y, 0.0, 0.0]),
                _ => false,
            }
        }
        "scale" => {
            let num = |w: &str| match w.strip_suffix('%') {
                Some(p) => p.parse::<f32>().ok().map(|p| p / 100.0),
                None => w.parse::<f32>().ok(),
            };
            let w: Vec<f32> = match words(v).map(num).collect::<Option<_>>() {
                Some(w) => w,
                None => return false,
            };
            match w[..] {
                [k] => push(op::MOVE, 1, [k, k, 0.0, 0.0]),
                [x, y] => push(op::MOVE, 1, [x, y, 0.0, 0.0]),
                _ => false,
            }
        }
        "rotate" => angle(v).is_some_and(|a| push(op::MOVE, 2, [a, 0.0, 0.0, 0.0])),
        "transform-origin" => {
            let at = |w: &str| match w {
                "left" | "top" => Some(0.0),
                "center" => Some(0.5),
                "right" | "bottom" => Some(1.0),
                w => w.strip_suffix('%')?.parse::<f32>().ok().map(|p| p / 100.0),
            };
            let w: Vec<&str> = words(v).collect();
            let (x, y) = match w[..] {
                [one] if one == "top" || one == "bottom" => (Some(0.5), at(one)),
                [one] => (at(one), Some(0.5)),
                [a, b] if a == "top" || a == "bottom" => (at(b), at(a)),
                [a, b] => (at(a), at(b)),
                _ => return false,
            };
            match (x, y) {
                (Some(x), Some(y)) => push(op::MOVE, 3, [x, y, 0.0, 0.0]),
                _ => false,
            }
        }
        "backdrop-filter" => {
            if v == "none" {
                return push(op::BACKDROP_BLUR, 0, [0.0; 4]);
            }
            v.strip_prefix("blur(")
                .and_then(|b| b.strip_suffix(')'))
                .and_then(px)
                .is_some_and(|b| push(op::BACKDROP_BLUR, 0, [b, 0.0, 0.0, 0.0]))
        }
        "text-shadow" => {
            if v == "none" {
                push(op::TEXT_SHADOW, 0, [0.0; 4]);
                return push(op::TEXT_SHADOW_COLOR, 0, [0.0; 4]);
            }
            match shadow_layer(v) {
                Some((false, [x, y, blur, 0.0], c)) => {
                    push(op::TEXT_SHADOW, 0, [x, y, blur, 0.0]);
                    push(op::TEXT_SHADOW_COLOR, 0, c)
                }
                _ => false,
            }
        }
        "text-decoration" | "text-decoration-line" => {
            let (mut line, mut c, mut thick) = (0u8, None, 0.0);
            for w in top_level(v, ' ') {
                match w {
                    "none" => {}
                    "underline" => line |= Decoration::UNDERLINE,
                    "overline" => line |= Decoration::OVERLINE,
                    "line-through" => line |= Decoration::LINE_THROUGH,
                    "solid" => {}
                    w => match (px(w), color(w)) {
                        (Some(t), _) if name == "text-decoration" => thick = t,
                        (_, Some(k)) if name == "text-decoration" => c = Some(k),
                        _ => return false,
                    },
                }
            }
            push(op::DECORATION, line, [thick, 0.0, 0.0, 0.0]);
            c.is_none_or(|c| push(op::DECORATION_COLOR, 0, c))
        }
        "text-decoration-color" => color(v).is_some_and(|c| push(op::DECORATION_COLOR, 0, c)),
        "opacity" => v
            .parse()
            .is_ok_and(|x| push(op::OPACITY, 0, [x, 0.0, 0.0, 0.0])),
        "font-size" => px(v).is_some_and(|x| push(op::FONT_SIZE, 0, [x, 0.0, 0.0, 0.0])),
        "font-weight" => {
            let w = match v {
                "normal" => Some(400.0),
                "bold" => Some(700.0),
                v => v.parse::<f32>().ok().filter(|w| (1.0..=1000.0).contains(w)),
            };
            w.is_some_and(|w| push(op::WEIGHT, 0, [w, 0.0, 0.0, 0.0]))
        }
        "letter-spacing" => {
            if v == "normal" {
                return push(op::TRACKING, 0, [0.0; 4]);
            }
            if let Some(em) = v
                .strip_suffix("em")
                .and_then(|n| n.trim().parse::<f32>().ok())
            {
                return push(op::TRACKING, 0, [0.0, em, 0.0, 0.0]);
            }
            px(v).is_some_and(|x| push(op::TRACKING, 0, [x, 0.0, 0.0, 0.0]))
        }
        "line-height" => match v.parse::<f32>() {
            Ok(k) => push(op::LINE, 0, [k, 0.0, 0.0, 0.0]),
            Err(_) if v == "normal" => push(op::LINE, 0, [1.2, 0.0, 0.0, 0.0]),
            Err(_) => px(v).is_some_and(|x| push(op::LINE, 1, [x, 0.0, 0.0, 0.0])),
        },
        // One wrapping switch: `white-space` and `text-wrap-mode` set it;
        // spaces always collapse, except `pre-wrap`, which keeps them.
        "white-space" | "text-wrap-mode" | "text-wrap" => match v {
            "nowrap" | "pre" => push(op::NOWRAP, 1, [0.0; 4]),
            "normal" | "wrap" | "pre-wrap" | "pre-line" | "balance" | "pretty" => {
                push(op::NOWRAP, 0, [0.0; 4])
            }
            _ => false,
        },
        "white-space-collapse" => matches!(v, "collapse" | "preserve" | "preserve-breaks"),
        "text-align" => match v {
            "start" | "left" | "justify" => push(op::TEXT_ALIGN, 0, [0.0; 4]),
            "center" => push(op::TEXT_ALIGN, 1, [0.0; 4]),
            "end" | "right" => push(op::TEXT_ALIGN, 2, [0.0; 4]),
            _ => false,
        },
        "text-overflow" => {
            let mut w = words(v);
            match w.next() {
                Some("clip") => push(op::TEXT_OVERFLOW, 0, [0.0; 4]),
                Some("ellipsis") => push(op::TEXT_OVERFLOW, 1, [0.0; 4]),
                Some("-og-scroll") => {
                    let ms = |t: Option<&str>| match t {
                        Some(t) if t.ends_with("ms") => t[..t.len() - 2].parse::<f32>().ok(),
                        Some(t) if t.ends_with('s') => {
                            t[..t.len() - 1].parse::<f32>().ok().map(|s| s * 1000.0)
                        }
                        None => Some(0.0),
                        _ => None,
                    };
                    match (ms(w.next()), ms(w.next())) {
                        (Some(a), Some(b)) => push(op::TEXT_OVERFLOW, 2, [a, b, 0.0, 0.0]),
                        _ => false,
                    }
                }
                _ => false,
            }
        }
        "text-transform" => match v {
            "none" => push(op::TRANSFORM, 0, [0.0; 4]),
            "uppercase" => push(op::TRANSFORM, 1, [0.0; 4]),
            "lowercase" => push(op::TRANSFORM, 2, [0.0; 4]),
            "capitalize" => push(op::TRANSFORM, 3, [0.0; 4]),
            _ => false,
        },
        "overflow" | "overflow-x" | "overflow-y" => {
            let code = |w: &str| match w {
                "visible" => Some(0u8),
                "clip" => Some(1),
                "hidden" => Some(2),
                "auto" | "scroll" => Some(3),
                _ => None,
            };
            let mut ws = words(v);
            let (Some(x), y) = (ws.next().and_then(code), ws.next().map(code)) else {
                return false;
            };
            let y = match y {
                Some(Some(y)) => y,
                Some(None) => return false,
                None => x,
            };
            // `v[0]`: the axes it sets (1 x, 2 y); fold keeps the other.
            let (c, axes) = match name {
                "overflow-x" => (x, 1.0),
                "overflow-y" => (x << 2, 2.0),
                _ => (x | (y << 2), 3.0),
            };
            push(op::OVERFLOW, c, [axes, 0.0, 0.0, 0.0])
        }
        "aspect-ratio" => {
            let v = v.trim_start_matches("auto").trim();
            let k = match v.split_once('/') {
                Some((a, b)) => a
                    .trim()
                    .parse::<f32>()
                    .ok()
                    .zip(b.trim().parse::<f32>().ok())
                    .map(|(a, b)| a / b),
                None => v.parse().ok(),
            };
            k.is_some_and(|k| push(op::ASPECT, 0, [k, 0.0, 0.0, 0.0]))
        }
        "fill" | "stroke" => {
            let o = if name == "fill" {
                op::PAINT
            } else {
                op::STROKE
            };
            match v {
                "none" | "transparent" => push(o, 0, [0.0; 4]),
                _ if v.eq_ignore_ascii_case("currentcolor") => push(o, 1, [0.0; 4]),
                _ => color(v).is_some_and(|c| push(o, 0, c)),
            }
        }
        "stroke-width" => px(v).is_some_and(|x| push(op::STROKE_WIDTH, 0, [x, 0.0, 0.0, 0.0])),
        // A list draws no markers in og-ui: there is nothing to turn off.
        "list-style" | "list-style-type" => v == "none",
        _ => false,
    }
}
