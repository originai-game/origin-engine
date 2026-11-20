//! CSS compiled once, when a sheet loads, into flat typed records: a rule's
//! declarations become a `Record` (layout as rune-taffy's `CompactStyle`,
//! paint as plain `Copy` fields), with every `var()` and `calc()` resolved
//! against the style's tokens. A selector is a compound of an id, classes
//! and a state (`:hover`, `:active`, `:disabled`, `:focus`), interned to
//! `u32`s; an element's record is the fold of the rules its id and classes
//! name, done when its classes change, never per frame. What the compiler
//! does not take (combinators, unknown properties) is listed, not guessed.

use crate::tokens::Tokens;
use rune_taffy::CompactStyle;
use rustc_hash::FxHashMap;
use taffy::{AlignItems, Dimension, FlexDirection, JustifyContent, LengthPercentageAuto, Rect};

/// Interned names: ids and classes share one table.
#[derive(Clone, Debug, Default)]
pub struct Names(FxHashMap<Box<str>, u32>);

impl Names {
    pub fn id(&mut self, name: &str) -> u32 {
        let n = self.0.len() as u32;
        *self.0.entry(name.into()).or_insert(n)
    }
    pub fn get(&self, name: &str) -> Option<u32> {
        self.0.get(name).copied()
    }
}

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

/// A linear RGBA colour.
pub type Rgba = [f32; 4];

/// What one rule sets: `Some` the properties it declared.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Decl {
    layout: Vec<Layout>,
    pub fill: Option<Rgba>,
    pub ink: Option<Rgba>,
    pub radius: Option<f32>,
    pub opacity: Option<f32>,
    pub font_size: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Layout {
    Display(taffy::Display),
    Direction(FlexDirection),
    Gap(f32, f32),
    Pad([f32; 4]),
    Margin([f32; 4]),
    Width(Dimension),
    Height(Dimension),
    Absolute,
    Inset([Option<f32>; 4]),
    Align(AlignItems),
    Justify(JustifyContent),
    Grow(f32),
}

/// An element's computed look: what its rules set, folded in order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Record {
    pub layout: CompactStyle,
    pub fill: Rgba,
    pub ink: Rgba,
    pub radius: f32,
    pub opacity: f32,
    pub font_size: f32,
}

#[derive(Clone, Debug)]
struct Rule {
    /// The id or classes the element must have, all of them.
    needs: Vec<u32>,
    state: State,
    decl: Decl,
}

/// A compiled sheet.
#[derive(Clone, Debug, Default)]
pub struct Sheet {
    pub names: Names,
    rules: Vec<Rule>,
    /// Selectors and declarations the compiler did not take.
    pub skipped: Vec<String>,
}

impl Sheet {
    /// Compiles `css` against `tokens`.
    pub fn compile(css: &str, tokens: &Tokens) -> Sheet {
        let mut s = Sheet::default();
        for (prelude, body) in crate::sheet::blocks(css) {
            if prelude.starts_with(':') && prelude.trim() == ":root" || prelude.starts_with('@') {
                continue;
            }
            let decl = s.declarations(body, tokens);
            for sel in crate::sheet::split_top(prelude, b',') {
                match s.selector(sel.trim()) {
                    Some((needs, state)) => s.rules.push(Rule {
                        needs,
                        state,
                        decl: decl.clone(),
                    }),
                    None => s.skipped.push(format!("selector {}", sel.trim())),
                }
            }
        }
        s
    }

    fn selector(&mut self, sel: &str) -> Option<(Vec<u32>, State)> {
        if sel.is_empty() || sel.contains([' ', '>', '+', '~', '[', '*']) {
            return None;
        }
        let (sel, state) = match sel.split_once(':') {
            None => (sel, State::Base),
            Some((s, "hover")) => (s, State::Hover),
            Some((s, "active")) => (s, State::Active),
            Some((s, "focus" | "focus-visible")) => (s, State::Focus),
            Some((s, "disabled")) => (s, State::Disabled),
            Some(_) => return None,
        };
        let mut needs = Vec::new();
        let mut rest = sel;
        while !rest.is_empty() {
            let sigil = rest.as_bytes()[0];
            if sigil != b'#' && sigil != b'.' {
                return None;
            }
            let end = rest[1..].find(['#', '.']).map_or(rest.len(), |e| e + 1);
            needs.push(self.names.id(&rest[..end]));
            rest = &rest[end..];
        }
        (!needs.is_empty()).then_some((needs, state))
    }

    fn declarations(&mut self, body: &str, tokens: &Tokens) -> Decl {
        let mut d = Decl::default();
        for (name, value) in crate::sheet::declarations(body) {
            if name.starts_with("--") {
                continue;
            }
            let Some(v) = tokens.substitute_str(value) else {
                self.skipped
                    .push(format!("{name}: {value} (an unknown token)"));
                continue;
            };
            if !apply(&mut d, name, v.trim()) {
                self.skipped.push(format!("{name}: {value}"));
            }
        }
        d
    }

    /// The record of an element with these names in this state: the base
    /// rules in sheet order, then the state's.
    pub fn record(&self, names: &[u32], state: State) -> Record {
        let mut r = Record {
            opacity: 1.0,
            font_size: 16.0,
            ink: [0.0, 0.0, 0.0, 1.0],
            ..Record::default()
        };
        for pass in [State::Base, state] {
            for rule in &self.rules {
                if rule.state == pass && rule.needs.iter().all(|n| names.contains(n)) {
                    fold(&mut r, &rule.decl);
                }
            }
            if state == State::Base {
                break;
            }
        }
        r
    }
}

fn fold(r: &mut Record, d: &Decl) {
    for l in &d.layout {
        let s = &mut r.layout;
        match *l {
            Layout::Display(v) => s.modify_cold(|t| t.display = v),
            Layout::Direction(v) => s.set_flex_direction(v),
            Layout::Gap(c, w) => s.set_gap(c, w),
            Layout::Pad([t, rt, b, lf]) => {
                s.set_pad_top(t);
                s.set_pad_right(rt);
                s.set_pad_bottom(b);
                s.set_pad_left(lf);
            }
            Layout::Margin([t, rt, b, lf]) => {
                s.set_margin_top(t);
                s.set_margin_right(rt);
                s.set_margin_bottom(b);
                s.set_margin_left(lf);
            }
            Layout::Width(v) => s.set_width(v),
            Layout::Height(v) => s.set_height(v),
            Layout::Absolute => s.set_position_absolute(),
            Layout::Inset(i) => {
                let lpa = |v: Option<f32>| {
                    v.map_or(LengthPercentageAuto::auto(), LengthPercentageAuto::length)
                };
                s.set_absolute_inset(Rect {
                    top: lpa(i[0]),
                    right: lpa(i[1]),
                    bottom: lpa(i[2]),
                    left: lpa(i[3]),
                });
            }
            Layout::Align(v) => s.set_align_items(v),
            Layout::Justify(v) => s.set_justify_content(v),
            Layout::Grow(v) => s.set_flex_grow(v),
        }
    }
    if let Some(v) = d.fill {
        r.fill = v;
    }
    if let Some(v) = d.ink {
        r.ink = v;
    }
    if let Some(v) = d.radius {
        r.radius = v;
    }
    if let Some(v) = d.opacity {
        r.opacity = v;
    }
    if let Some(v) = d.font_size {
        r.font_size = v;
    }
}

/// A length in CSS px: `12px`, `1.5rem` (16 px), `0`, or a `calc()` that
/// reduces to one.
fn px(v: &str) -> Option<f32> {
    let reduced;
    let v = if v.contains('(') {
        reduced = og_css::calc::reduce(v).ok()?;
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

fn dimension(v: &str) -> Option<Dimension> {
    if v == "auto" {
        return Some(Dimension::auto());
    }
    if let Some(p) = v.strip_suffix('%') {
        return p
            .trim()
            .parse::<f32>()
            .ok()
            .map(|p| Dimension::percent(p / 100.0));
    }
    px(v).map(Dimension::length)
}

/// Up to four lengths in CSS's top, right, bottom, left shorthand order.
fn sides(v: &str) -> Option<[f32; 4]> {
    let parts: Vec<f32> = crate::sheet::split_top(v, b' ')
        .filter(|p| !p.is_empty())
        .map(px)
        .collect::<Option<_>>()?;
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

fn apply(d: &mut Decl, name: &str, v: &str) -> bool {
    use Layout as L;
    let layout = match name {
        "display" => L::Display(match v {
            "flex" => taffy::Display::Flex,
            "grid" => taffy::Display::Grid,
            "block" => taffy::Display::Block,
            "none" => taffy::Display::None,
            _ => return false,
        }),
        "flex-direction" => L::Direction(match v {
            "row" => FlexDirection::Row,
            "column" => FlexDirection::Column,
            "row-reverse" => FlexDirection::RowReverse,
            "column-reverse" => FlexDirection::ColumnReverse,
            _ => return false,
        }),
        "gap" => {
            let Some(s) = sides(v) else { return false };
            L::Gap(s[1], s[0])
        }
        "padding" => match sides(v) {
            Some(s) => L::Pad(s),
            None => return false,
        },
        "margin" => match sides(v) {
            Some(s) => L::Margin(s),
            None => return false,
        },
        "width" | "inline-size" => match dimension(v) {
            Some(x) => L::Width(x),
            None => return false,
        },
        "height" | "block-size" => match dimension(v) {
            Some(x) => L::Height(x),
            None => return false,
        },
        "position" if v == "fixed" || v == "absolute" => L::Absolute,
        "position" if v == "relative" || v == "static" => return true,
        "top" | "right" | "bottom" | "left" => {
            let at = ["top", "right", "bottom", "left"]
                .iter()
                .position(|s| *s == name)
                .unwrap();
            let Some(x) = px(v) else { return false };
            let mut inset = d.layout.iter().rev().find_map(|l| match l {
                L::Inset(i) => Some(*i),
                _ => None,
            });
            let i = inset.get_or_insert([None; 4]);
            i[at] = Some(x);
            d.layout.retain(|l| !matches!(l, L::Inset(_)));
            L::Inset(*i)
        }
        "align-items" => L::Align(match v {
            "center" => AlignItems::CENTER,
            "start" | "flex-start" => AlignItems::FLEX_START,
            "end" | "flex-end" => AlignItems::FLEX_END,
            "stretch" => AlignItems::STRETCH,
            "baseline" => AlignItems::BASELINE,
            _ => return false,
        }),
        "justify-content" => L::Justify(match v {
            "center" => JustifyContent::CENTER,
            "start" | "flex-start" => JustifyContent::FLEX_START,
            "end" | "flex-end" => JustifyContent::FLEX_END,
            "space-between" => JustifyContent::SPACE_BETWEEN,
            "space-around" => JustifyContent::SPACE_AROUND,
            _ => return false,
        }),
        "flex-grow" => match v.parse() {
            Ok(g) => L::Grow(g),
            Err(_) => return false,
        },
        "background" | "background-color" => {
            d.fill = color(v);
            return d.fill.is_some();
        }
        "color" => {
            d.ink = color(v);
            return d.ink.is_some();
        }
        "border-radius" => {
            d.radius = px(v);
            return d.radius.is_some();
        }
        "opacity" => {
            d.opacity = v.parse().ok();
            return d.opacity.is_some();
        }
        "font-size" => {
            d.font_size = px(v);
            return d.font_size.is_some();
        }
        _ => return false,
    };
    d.layout.push(layout);
    true
}
