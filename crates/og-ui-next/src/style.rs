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

/// An element's computed look: what its rules set, folded in order.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub layout: CompactStyle,
    pub fill: Rgba,
    pub ink: Rgba,
    pub radius: f32,
    pub opacity: f32,
    pub font_size: f32,
    /// An SVG shape's fill and stroke paint, and its stroke's width.
    pub paint: Rgba,
    pub stroke: Rgba,
    pub stroke_width: f32,
    /// The sides of its `inset` (0) and `padding` (1) that add the HUD's
    /// safe-area inset (`env(-og-hud-inset-left)`), bits top, right,
    /// bottom, left: the one runtime input a record leaves open.
    pub safe: [u8; 2],
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
        Record {
            layout: CompactStyle::default(),
            fill: [0.0; 4],
            ink: [0.0, 0.0, 0.0, 1.0],
            radius: 0.0,
            opacity: 1.0,
            font_size: 16.0,
            paint: [0.0, 0.0, 0.0, 1.0],
            stroke: [0.0; 4],
            stroke_width: 1.0,
            safe: [0; 2],
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
        op::GAP => l.set_gap(a, b),
        op::SAFE => r.safe[c & 1] = (r.safe[c & 1] & !(a as u8)) | b as u8,
        op::PAD => {
            l.set_pad_top(a);
            l.set_pad_right(b);
            l.set_pad_bottom(cc);
            l.set_pad_left(d);
        }
        op::MARGIN => {
            l.set_margin_top(a);
            l.set_margin_right(b);
            l.set_margin_bottom(cc);
            l.set_margin_left(d);
        }
        op::WIDTH => l.set_width(dim(s)),
        op::HEIGHT => l.set_height(dim(s)),
        op::MIN_W => l.set_min_width(lpa(s)),
        op::MIN_H => l.set_min_height(lpa(s)),
        op::MAX_W => l.set_max_width(lpa(s)),
        op::MAX_H => l.set_max_height(lpa(s)),
        op::ABSOLUTE => l.set_position_absolute(),
        op::INSET => {
            let side = |k: usize| {
                if s.code & (1 << k) != 0 {
                    LengthPercentageAuto::length(s.v[k])
                } else {
                    LengthPercentageAuto::auto()
                }
            };
            l.set_absolute_inset(Rect {
                top: side(0),
                right: side(1),
                bottom: side(2),
                left: side(3),
            });
        }
        op::ALIGN => l.set_align_items(ALIGNS[c]),
        op::JUSTIFY => l.set_justify_content(JUSTIFIES[c]),
        op::GROW => l.set_flex_grow(a),
        op::SHRINK => l.set_flex_shrink(a),
        op::BASIS => l.set_flex_basis(dim(s)),
        op::FILL => r.fill = s.v,
        op::INK => r.ink = s.v,
        op::RADIUS => r.radius = a,
        op::OPACITY => r.opacity = a,
        op::FONT_SIZE => r.font_size = a,
        op::PAINT => r.paint = if c == 1 { r.ink } else { [a, b, cc, d] },
        op::STROKE => r.stroke = if c == 1 { r.ink } else { [a, b, cc, d] },
        op::STROKE_WIDTH => r.stroke_width = a,
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
    let parts: Vec<f32> = words(v).map(px).collect::<Option<_>>()?;
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

fn index<T: Copy>(table: &[(&str, T)], v: &str) -> Option<u8> {
    table.iter().position(|(n, _)| *n == v).map(|i| i as u8)
}

/// Lays the step a side-wise property (`padding-left`, `margin-inline`)
/// sets over the last full one of `op`, or a new one from zero.
fn set_sides(steps: &mut Vec<Step>, o: u8, which: &[usize], x: f32) {
    let mut v = steps
        .iter()
        .rev()
        .find(|s| s.op == o)
        .map_or([0.0; 4], |s| s.v);
    for &k in which {
        v[k] = x;
    }
    steps.push(step(o, 0, v));
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
            let mut g = steps
                .iter()
                .rev()
                .find(|s| s.op == op::GAP)
                .map_or([0.0; 4], |s| s.v);
            g[usize::from(name == "row-gap")] = x;
            steps.push(step(op::GAP, 0, g));
            true
        }
        "padding" => sides(v).is_some_and(|s| push(op::PAD, 0, s)),
        "margin" => sides(v).is_some_and(|s| push(op::MARGIN, 0, s)),
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
            let vals: Vec<f32> = match words(v).map(px).collect::<Option<Vec<f32>>>() {
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
            "fixed" | "absolute" => push(op::ABSOLUTE, 0, [0.0; 4]),
            "relative" | "static" => true,
            _ => false,
        },
        "top" | "right" | "bottom" | "left" | "inset" => {
            let mut last = steps
                .iter()
                .rev()
                .find(|s| s.op == op::INSET)
                .copied()
                .unwrap_or(step(op::INSET, 0, [0.0; 4]));
            if name == "inset" {
                let Some(s) = sides(v) else { return false };
                last = step(op::INSET, 0b1111, s);
            } else {
                let k = ["top", "right", "bottom", "left"]
                    .iter()
                    .position(|s| *s == name)
                    .unwrap();
                let Some(x) = px(v) else { return false };
                last.v[k] = x;
                last.code |= 1 << k;
            }
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
        "background" | "background-color" => color(v).is_some_and(|c| push(op::FILL, 0, c)),
        "color" => color(v).is_some_and(|c| push(op::INK, 0, c)),
        "border-radius" => px(v).is_some_and(|x| push(op::RADIUS, 0, [x, 0.0, 0.0, 0.0])),
        "opacity" => v
            .parse()
            .is_ok_and(|x| push(op::OPACITY, 0, [x, 0.0, 0.0, 0.0])),
        "font-size" => px(v).is_some_and(|x| push(op::FONT_SIZE, 0, [x, 0.0, 0.0, 0.0])),
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
