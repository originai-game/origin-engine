//! CSS Anchor Positioning 1: an absolutely positioned element placed
//! against other elements' boxes. `anchor-name` names an anchor;
//! `position-anchor` picks an element's default one (else the implicit
//! anchor a popover's invoker is); insets take `anchor()`, sizes
//! `anchor-size()`; `position-area` places it in one of the nine areas
//! the anchor's edges cut its containing block into; `position-try-
//! fallbacks` lists other placements to try when it overflows; and
//! `position-visibility` hides it with its anchor or when it overflows.
//! Resolved after the tree is laid out, against this frame's boxes.

use crate::layout::{Rect, Size};

/// An element's anchor positioning (`None` on [`crate::Props::anchor`]:
/// none, its insets are px).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnchorSpec {
    /// `position-anchor`: its default anchor's name (`--name`); empty: its
    /// implicit anchor ([`crate::Props::anchor_to`]).
    pub anchor: String,
    /// The placement itself: insets, area and sizes.
    pub base: Placement,
    /// `position-try-fallbacks`, tried in order while it overflows.
    pub tries: Vec<Try>,
    pub order: TryOrder,
    pub visibility: Visibility,
    /// The `@position-try` rules its fallbacks name, by name.
    pub named: Vec<(String, Placement)>,
}

/// What one position option sets.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Placement {
    /// Its insets, top, right, bottom, left (physical).
    pub insets: [Option<Inset>; 4],
    pub area: Option<Area>,
    /// `anchor-size()` widths and heights.
    pub sizes: [Option<AnchorSize>; 2],
}

/// An inset: px, a fraction of the containing block, or an anchor's edge.
#[derive(Clone, Debug, PartialEq)]
pub enum Inset {
    /// `auto`, written (an `@position-try` rule's `top: auto` clears the
    /// element's own).
    Auto,
    Px(f32),
    /// A fraction (0–1) of the containing block's size along the axis, plus px.
    Percent(f32, f32),
    /// `anchor(<name>? <side>, <fallback>?)`, plus px (`calc(anchor(…) + 4px)`).
    Anchor {
        name: String,
        side: Side,
        plus: f32,
        fallback: Option<f32>,
    },
}

/// `anchor()`'s side (CSS Anchor §4.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
    Center,
    /// The inset's own side, the opposite one.
    Inside,
    Outside,
    /// The axis's start and end (top and left; bottom and right).
    Start,
    End,
    /// A fraction along the axis from its start.
    Percent(f32),
}

/// `anchor-size(<name>? <dimension>)` times `scale`, plus px.
#[derive(Clone, Debug, PartialEq)]
pub struct AnchorSize {
    pub name: String,
    /// 0: the anchor's width, 1: its height.
    pub axis: usize,
    pub scale: f32,
    pub plus: f32,
}

/// A `position-area`: the columns and rows (0, 1, 2: start, the anchor's
/// span, end) it covers, inclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Area {
    pub cols: (u8, u8),
    pub rows: (u8, u8),
}

/// One `position-try-fallbacks` entry.
#[derive(Clone, Debug, PartialEq)]
pub enum Try {
    /// `flip-block`, `flip-inline`, `flip-start`, in order.
    Flips(Vec<Flip>),
    /// A `position-area`.
    Area(Area),
    /// An `@position-try` rule, then flips.
    Named(String, Vec<Flip>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flip {
    Block,
    Inline,
    Start,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TryOrder {
    #[default]
    Normal,
    MostWidth,
    MostHeight,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Visibility {
    Always,
    /// Hidden while its default anchor is clipped out of view (the default).
    #[default]
    AnchorsVisible,
    /// Hidden while it overflows its containing block.
    NoOverflow,
}

/// Where the anchored element ended up, per frame: the option that won (0
/// its own placement, k the k-th fallback) and the side of its anchor it
/// sits on (`None`: over it), for a popover's arrow.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Placed {
    pub option: u8,
    pub side: Option<PlacedSide>,
    pub hidden: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlacedSide {
    Top,
    Right,
    Bottom,
    Left,
}

// ---- parsing

fn px_of(t: &str) -> Option<f32> {
    let t = t.trim();
    if t == "0" {
        return Some(0.0);
    }
    t.strip_suffix("px")?.trim().parse().ok()
}

fn dashed(t: &str) -> bool {
    t.starts_with("--") && t.len() > 2
}

/// `anchor(<name>? <side>, <fallback>?)`.
fn anchor_fn(body: &str) -> Result<(String, Side, Option<f32>), String> {
    let (main, fallback) = match body.split_once(',') {
        Some((a, b)) => (
            a,
            Some(
                px_of(b)
                    .ok_or_else(|| format!("anchor()'s fallback '{}' is no length", b.trim()))?,
            ),
        ),
        None => (body, None),
    };
    let mut name = String::new();
    let mut side = None;
    for w in main.split_whitespace() {
        if dashed(w) && name.is_empty() {
            name = w.to_string();
            continue;
        }
        let s = match w {
            "top" => Side::Top,
            "right" => Side::Right,
            "bottom" => Side::Bottom,
            "left" => Side::Left,
            "center" => Side::Center,
            "inside" => Side::Inside,
            "outside" => Side::Outside,
            "start" | "self-start" => Side::Start,
            "end" | "self-end" => Side::End,
            p if p.ends_with('%') => Side::Percent(
                p.trim_end_matches('%')
                    .parse::<f32>()
                    .map_err(|_| format!("'{p}' is no anchor side"))?
                    / 100.0,
            ),
            w => {
                return Err(format!(
                    "'{w}' is no anchor side: top, right, bottom, left, center, start, end, inside, outside or <n>%"
                ));
            }
        };
        if side.replace(s).is_some() {
            return Err("anchor() takes one side".into());
        }
    }
    Ok((
        name,
        side.ok_or("anchor() names its side: anchor(--name bottom)")?,
        fallback,
    ))
}

/// A call `f(…)` at the start of `t`: its body and what follows.
fn call<'a>(t: &'a str, f: &str) -> Option<(&'a str, &'a str)> {
    let rest = t.strip_prefix(f)?.strip_prefix('(')?;
    let mut depth = 1;
    for (k, c) in rest.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&rest[..k], &rest[k + 1..]));
                }
            }
            _ => {}
        }
    }
    None
}

/// A `+ <px>` / `- <px>` tail (or none).
fn tail(t: &str) -> Result<f32, String> {
    let t = t.trim();
    if t.is_empty() {
        return Ok(0.0);
    }
    let (sign, rest) = match t.split_at(1) {
        ("+", r) => (1.0, r),
        ("-", r) => (-1.0, r),
        _ => return Err(format!("'{t}': add or take px: calc(anchor(…) + 4px)")),
    };
    px_of(rest)
        .map(|v| sign * v)
        .ok_or_else(|| format!("'{}' is no px length", rest.trim()))
}

/// An inset's value: `<n>px`, `<n>%`, `anchor(…)`, or `calc()` of one of
/// those plus or minus px.
pub fn parse_inset(v: &str) -> Result<Inset, String> {
    let v = v.trim();
    let inner = call(v, "calc")
        .filter(|(_, r)| r.trim().is_empty())
        .map(|(b, _)| b.trim());
    let body = inner.unwrap_or(v);
    if let Some((a, rest)) = call(body, "anchor") {
        let (name, side, fallback) = anchor_fn(a)?;
        return Ok(Inset::Anchor {
            name,
            side,
            plus: tail(rest)?,
            fallback,
        });
    }
    if let Some(p) = px_of(body) {
        return Ok(Inset::Px(p));
    }
    let (head, rest) = match body.find([' ']) {
        Some(k) => body.split_at(k),
        None => (body, ""),
    };
    if let Some(p) = head.strip_suffix('%').and_then(|p| p.parse::<f32>().ok()) {
        return Ok(Inset::Percent(p / 100.0, tail(rest)?));
    }
    Err(format!(
        "'{v}' is no inset: <n>px, <n>%, anchor([--name] <side>[, <fallback>]) or calc() of one plus px"
    ))
}

/// A size's `anchor-size([--name] [width | height | block | inline |
/// self-block | self-inline])`, optionally times a number or plus px.
pub fn parse_size(v: &str) -> Result<AnchorSize, String> {
    let v = v.trim();
    let inner = call(v, "calc")
        .filter(|(_, r)| r.trim().is_empty())
        .map(|(b, _)| b.trim());
    let body = inner.unwrap_or(v);
    let (scale, body) = match body.split_once('*') {
        Some((a, b)) if a.trim().parse::<f32>().is_ok() => (a.trim().parse().unwrap(), b.trim()),
        _ => (1.0, body),
    };
    let (a, rest) = call(body, "anchor-size").ok_or_else(|| {
        format!("'{v}' is no anchor-size(): anchor-size([--name] width | height)")
    })?;
    let (mut name, mut axis) = (String::new(), None);
    for w in a.split_whitespace() {
        match w {
            w if dashed(w) && name.is_empty() => name = w.to_string(),
            "width" | "inline" | "self-inline" => axis = Some(0),
            "height" | "block" | "self-block" => axis = Some(1),
            w => return Err(format!("'{w}' is no anchor-size() dimension")),
        }
    }
    let (scale, plus) = match rest.trim() {
        r if r.starts_with('*') => (
            scale
                * r[1..]
                    .trim()
                    .parse::<f32>()
                    .map_err(|_| "anchor-size() * <number>")?,
            0.0,
        ),
        r => (scale, tail(r)?),
    };
    Ok(AnchorSize {
        name,
        axis: axis.unwrap_or(usize::MAX),
        scale,
        plus,
    })
}

/// A `position-area` (CSS Anchor §3.1): one or two keywords, physical
/// (`top`, `left`, `center`, `span-left`, `span-all`…) or logical
/// (`block-start`, `span-inline-end`, `start`, `end`…).
pub fn parse_area(v: &str) -> Result<Option<Area>, String> {
    let v = v.trim();
    if v == "none" {
        return Ok(None);
    }
    // Each keyword: its axis (0 columns, 1 rows, 2 either) and range.
    let kw = |w: &str| -> Option<(u8, (u8, u8))> {
        Some(match w {
            "left" | "inline-start" | "x-start" | "self-inline-start" => (0, (0, 0)),
            "right" | "inline-end" | "x-end" | "self-inline-end" => (0, (2, 2)),
            "span-left" | "span-inline-start" | "span-x-start" => (0, (0, 1)),
            "span-right" | "span-inline-end" | "span-x-end" => (0, (1, 2)),
            "top" | "block-start" | "y-start" | "self-block-start" => (1, (0, 0)),
            "bottom" | "block-end" | "y-end" | "self-block-end" => (1, (2, 2)),
            "span-top" | "span-block-start" | "span-y-start" => (1, (0, 1)),
            "span-bottom" | "span-block-end" | "span-y-end" => (1, (1, 2)),
            "center" => (2, (1, 1)),
            "span-all" => (2, (0, 2)),
            "start" | "self-start" => (3, (0, 0)),
            "end" | "self-end" => (3, (2, 2)),
            "span-start" | "span-self-start" => (3, (0, 1)),
            "span-end" | "span-self-end" => (3, (1, 2)),
            _ => return None,
        })
    };
    let how = || {
        format!(
            "'{v}' is no position-area: one or two of top, bottom, left, right, center, span-top…, span-all, block-start…, start, end"
        )
    };
    let words: Vec<(u8, (u8, u8))> = v
        .split_whitespace()
        .map(|w| kw(w).ok_or_else(how))
        .collect::<Result<_, _>>()?;
    let (cols, rows) = match words.as_slice() {
        // One keyword: the other axis is span-all for a side, center for
        // center (§3.1.1).
        [(0, c)] => (*c, (0, 2)),
        [(1, r)] => ((0, 2), *r),
        [(2, x)] => (*x, *x),
        [(3, x)] => (*x, *x),
        // Two: physical axes in either order; `center`, `span-all` and the
        // generic `start`/`end` fill the other; generic pairs are block,
        // then inline.
        [(0, c), (1..=3, r)] | [(1..=3, r), (0, c)] => (*c, *r),
        [(1, r), (2 | 3, c)] | [(2 | 3, c), (1, r)] => (*c, *r),
        [(2 | 3, r), (2 | 3, c)] => (*c, *r),
        _ => return Err(how()),
    };
    Ok(Some(Area { cols, rows }))
}

/// `position-try-fallbacks`: `none` or a comma list of flips, an
/// `@position-try` name with flips, or a `position-area`.
pub fn parse_tries(v: &str) -> Result<Vec<Try>, String> {
    let v = v.trim();
    if v == "none" || v.is_empty() {
        return Ok(Vec::new());
    }
    v.split(',')
        .map(|entry| {
            let entry = entry.trim();
            let mut name = None;
            let mut flips = Vec::new();
            for w in entry.split_whitespace() {
                match w {
                    "flip-block" => flips.push(Flip::Block),
                    "flip-inline" => flips.push(Flip::Inline),
                    "flip-start" => flips.push(Flip::Start),
                    w if dashed(w) && name.is_none() => name = Some(w.to_string()),
                    _ => {
                        return match parse_area(entry)? {
                            Some(a) if name.is_none() && flips.is_empty() => Ok(Try::Area(a)),
                            _ => Err(format!("'{entry}' is no position option: flips, an @position-try name or a position-area")),
                        };
                    }
                }
            }
            Ok(match name {
                Some(n) => Try::Named(n, flips),
                None if !flips.is_empty() => Try::Flips(flips),
                None => return Err("an empty position option".into()),
            })
        })
        .collect()
}

/// An `@position-try` rule's declarations (name, value text) as the
/// placement it sets; what it leaves out comes from the element's own.
pub fn placement_of<'a>(decls: impl IntoIterator<Item = (&'a str, &'a str)>) -> Placement {
    let mut p = Placement::default();
    for (name, v) in decls {
        let v = v.trim();
        let sides: &[usize] = match name {
            "top" | "inset-block-start" => &[0],
            "right" | "inset-inline-end" => &[1],
            "bottom" | "inset-block-end" => &[2],
            "left" | "inset-inline-start" => &[3],
            "inset" => &[0, 1, 2, 3],
            "inset-block" => &[0, 2],
            "inset-inline" => &[3, 1],
            "position-area" => {
                p.area = parse_area(v).ok().flatten();
                continue;
            }
            "width" | "inline-size" | "height" | "block-size" => {
                let axis = usize::from(matches!(name, "height" | "block-size"));
                p.sizes[axis] = parse_size(v).ok();
                continue;
            }
            _ => continue,
        };
        // A shorthand's values in its sides' order (one value: every side).
        let words: Vec<&str> = v.split_whitespace().collect();
        for (k, &side) in sides.iter().enumerate() {
            let w = match (name, words.len()) {
                (_, 1) => words[0],
                ("inset", 2) => words[k % 2],
                ("inset", 3) => words[[0, 1, 2, 1][k]],
                _ => words.get(k).copied().unwrap_or(v),
            };
            p.insets[side] = if w == "auto" {
                Some(Inset::Auto)
            } else {
                parse_inset(w).ok()
            };
        }
    }
    p
}

// ---- resolution

impl Placement {
    /// Itself with `flip` applied (CSS Anchor §5.1): the block flip swaps
    /// top and bottom (and an anchor() side), the inline one left and
    /// right, the start one the axes.
    fn flipped(&self, flip: Flip) -> Placement {
        let side = |s: Side, f: Flip| match (f, s) {
            (Flip::Block, Side::Top) => Side::Bottom,
            (Flip::Block, Side::Bottom) => Side::Top,
            (Flip::Inline, Side::Left) => Side::Right,
            (Flip::Inline, Side::Right) => Side::Left,
            (Flip::Start, Side::Top) => Side::Left,
            (Flip::Start, Side::Left) => Side::Top,
            (Flip::Start, Side::Bottom) => Side::Right,
            (Flip::Start, Side::Right) => Side::Bottom,
            (Flip::Block | Flip::Inline, Side::Start) => Side::End,
            (Flip::Block | Flip::Inline, Side::End) => Side::Start,
            (Flip::Block | Flip::Inline, Side::Percent(p)) => Side::Percent(1.0 - p),
            (_, s) => s,
        };
        let inset = |i: &Option<Inset>| {
            i.clone().map(|i| match i {
                Inset::Anchor {
                    name,
                    side: s,
                    plus,
                    fallback,
                } => Inset::Anchor {
                    name,
                    side: side(s, flip),
                    plus,
                    fallback,
                },
                i => i,
            })
        };
        let [t, r, b, l] = &self.insets;
        let mirror = |(a, b): (u8, u8)| (2 - b, 2 - a);
        match flip {
            Flip::Block => Placement {
                insets: [inset(b), inset(r), inset(t), inset(l)],
                area: self.area.map(|a| Area {
                    rows: mirror(a.rows),
                    ..a
                }),
                sizes: self.sizes.clone(),
            },
            Flip::Inline => Placement {
                insets: [inset(t), inset(l), inset(b), inset(r)],
                area: self.area.map(|a| Area {
                    cols: mirror(a.cols),
                    ..a
                }),
                sizes: self.sizes.clone(),
            },
            Flip::Start => Placement {
                insets: [inset(l), inset(b), inset(r), inset(t)],
                area: self.area.map(|a| Area {
                    cols: a.rows,
                    rows: a.cols,
                }),
                sizes: [self.sizes[1].clone(), self.sizes[0].clone()],
            },
        }
    }
}

/// The boxes one resolution reads.
pub(crate) struct Frame<'a> {
    /// The containing block (its padding box).
    pub cb: Rect,
    /// An anchor's box by name (empty: the default one).
    pub anchor: &'a dyn Fn(&str) -> Option<Rect>,
    /// The element's authored width and height, and its content's sizes
    /// (preferred, least) per axis.
    pub size: [Size; 2],
    pub content: [(f32, f32); 2],
}

/// One option's box, and its inset-modified containing block.
fn place(p: &Placement, f: &Frame<'_>) -> (Rect, Rect) {
    let cb = f.cb;
    let start = [cb.x, cb.y];
    let len = [cb.width, cb.height];
    // An inset in px from its containing block edge (`None`: auto).
    let resolve = |k: usize| -> Option<f32> {
        let axis = usize::from(k % 2 == 0);
        let far = k == 1 || k == 2;
        let i = p.insets[k].as_ref()?;
        Some(match i {
            Inset::Auto => return None,
            Inset::Px(v) => *v,
            Inset::Percent(q, plus) => q * len[axis] + plus,
            Inset::Anchor {
                name,
                side,
                plus,
                fallback,
            } => {
                let Some(a) = (f.anchor)(name) else {
                    return *fallback;
                };
                let (a0, al) = if axis == 0 {
                    (a.x, a.width)
                } else {
                    (a.y, a.height)
                };
                let frac = match side {
                    Side::Top | Side::Left => 0.0,
                    Side::Bottom | Side::Right => 1.0,
                    Side::Center => 0.5,
                    Side::Start => 0.0,
                    Side::End => 1.0,
                    Side::Inside => f32::from(u8::from(far)),
                    Side::Outside => f32::from(u8::from(!far)),
                    Side::Percent(q) => *q,
                };
                let at = a0 + al * frac;
                (if far {
                    start[axis] + len[axis] - at
                } else {
                    at - start[axis]
                }) + plus
            }
        })
    };
    let insets = [resolve(0), resolve(1), resolve(2), resolve(3)];
    let anchor = (f.anchor)("");
    // The area's box: the 3×3 grid of the anchor's and the block's edges.
    let area = p.area.zip(anchor).map(|(ar, a)| {
        let lines = |axis: usize| {
            let (a0, al) = if axis == 0 {
                (a.x, a.width)
            } else {
                (a.y, a.height)
            };
            let c0 = start[axis];
            let c1 = c0 + len[axis];
            [c0.min(a0), a0, a0 + al, c1.max(a0 + al)]
        };
        let (xs, ys) = (lines(0), lines(1));
        let (c, r) = (ar.cols, ar.rows);
        Rect::new(
            xs[c.0 as usize],
            ys[r.0 as usize],
            xs[c.1 as usize + 1] - xs[c.0 as usize],
            ys[r.1 as usize + 1] - ys[r.0 as usize],
        )
    });
    let mut out = [0.0f32; 4];
    let mut imcb = [0.0f32; 4];
    for axis in 0..2 {
        let (lo_k, hi_k) = if axis == 0 { (3, 1) } else { (0, 2) };
        // Its room: the area, else the block less its insets.
        let (r0, r1) = match area {
            Some(a) if axis == 0 => (a.x, a.right()),
            Some(a) => (a.y, a.bottom()),
            None => (
                start[axis] + insets[lo_k].unwrap_or(0.0),
                start[axis] + len[axis] - insets[hi_k].unwrap_or(0.0),
            ),
        };
        let (r0, r1) = match area {
            Some(_) => (
                r0 + insets[lo_k].unwrap_or(0.0),
                r1 - insets[hi_k].unwrap_or(0.0),
            ),
            None => (r0, r1),
        };
        let room = (r1 - r0).max(0.0);
        imcb[axis] = r0;
        imcb[axis + 2] = room;
        let (pref, least) = f.content[axis];
        let anchor_size = p.sizes[axis].as_ref().and_then(|s| {
            let a = (f.anchor)(&s.name)?;
            let dim = if s.axis == usize::MAX { axis } else { s.axis };
            Some((if dim == 0 { a.width } else { a.height }) * s.scale + s.plus)
        });
        let both = insets[lo_k].is_some() && insets[hi_k].is_some() && area.is_none();
        let (lo_b, hi_b) = match f.size[axis] {
            Size::Fit { min, max } | Size::Grow { min, max } => (min, max.max(min)),
            _ => (0.0, f32::INFINITY),
        };
        let size = match (anchor_size, f.size[axis]) {
            (Some(s), _) => s,
            (None, Size::Fixed(v)) => v,
            (None, Size::Percent(q)) => q * len[axis],
            (None, Size::Calc(c)) => c.of(len[axis]),
            (None, Size::MinContent) => least,
            (None, Size::MaxContent) => pref,
            (None, Size::Grow { .. }) => room.clamp(lo_b, hi_b),
            // Auto between two insets fills them (CSS Position 3 §5.1),
            // else it shrinks to fit its room.
            (None, Size::Fit { .. }) if both => room.max(least).clamp(lo_b, hi_b),
            (None, Size::Fit { .. }) => pref.min(room).max(least).clamp(lo_b, hi_b),
        };
        // Where it sits in its room: by its insets, else toward the
        // anchor in an area (§3.1.2), else at its static start.
        let at = match area {
            Some(_) => {
                let (lo, hi) = if axis == 0 {
                    p.area.unwrap().cols
                } else {
                    p.area.unwrap().rows
                };
                // A centre or span-all region aligns as `anchor-center`.
                match (lo, hi, anchor) {
                    (0, 0, _) | (0, 1, _) => r1 - size,
                    (2, 2, _) | (1, 2, _) => r0,
                    (_, _, Some(a)) => {
                        let (a0, al) = if axis == 0 {
                            (a.x, a.width)
                        } else {
                            (a.y, a.height)
                        };
                        a0 + (al - size) / 2.0
                    }
                    _ => r0 + (room - size) / 2.0,
                }
            }
            None => match (insets[lo_k], insets[hi_k]) {
                (Some(_), _) => r0,
                (None, Some(_)) => r1 - size,
                (None, None) => r0,
            },
        };
        out[axis] = at;
        out[axis + 2] = size;
    }
    (
        Rect::new(out[0], out[1], out[2], out[3]),
        Rect::new(imcb[0], imcb[1], imcb[2], imcb[3]),
    )
}

/// The box that wins among `spec`'s options (CSS Anchor §5): its own
/// placement, else the first fallback that does not overflow the
/// containing block, else its own.
pub(crate) fn resolve(spec: &AnchorSpec, f: &Frame<'_>) -> (Rect, Placed) {
    let tries = |name: &str| {
        spec.named
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, p)| p.clone())
    };
    let mut options: Vec<Placement> = vec![spec.base.clone()];
    for t in &spec.tries {
        let flips = |p: Placement, fl: &[Flip]| fl.iter().fold(p, |p, &fl| p.flipped(fl));
        options.push(match t {
            Try::Flips(fl) => flips(spec.base.clone(), fl),
            Try::Area(a) => Placement {
                area: Some(*a),
                ..spec.base.clone()
            },
            Try::Named(name, fl) => match tries(name) {
                Some(p) => flips(
                    Placement {
                        insets: std::array::from_fn(|k| {
                            p.insets[k].clone().or_else(|| spec.base.insets[k].clone())
                        }),
                        area: p.area.or(spec.base.area),
                        sizes: std::array::from_fn(|k| {
                            p.sizes[k].clone().or_else(|| spec.base.sizes[k].clone())
                        }),
                    },
                    fl,
                ),
                None => continue,
            },
        });
    }
    let placed: Vec<(Rect, Rect)> = options.iter().map(|p| place(p, f)).collect();
    let fits = |r: &Rect| {
        let cb = f.cb;
        r.x >= cb.x - 0.5
            && r.y >= cb.y - 0.5
            && r.right() <= cb.right() + 0.5
            && r.bottom() <= cb.bottom() + 0.5
    };
    let mut order: Vec<usize> = (0..placed.len()).collect();
    match spec.order {
        TryOrder::Normal => {}
        TryOrder::MostWidth => {
            order[1..].sort_by(|&a, &b| placed[b].1.width.total_cmp(&placed[a].1.width))
        }
        TryOrder::MostHeight => {
            order[1..].sort_by(|&a, &b| placed[b].1.height.total_cmp(&placed[a].1.height))
        }
    }
    let won = order.iter().copied().find(|&k| fits(&placed[k].0));
    let k = won.unwrap_or(0);
    let rect = placed[k].0;
    let anchor = (f.anchor)("");
    let side = anchor.and_then(|a| {
        if rect.bottom() <= a.y + 0.5 {
            Some(PlacedSide::Top)
        } else if rect.y >= a.bottom() - 0.5 {
            Some(PlacedSide::Bottom)
        } else if rect.right() <= a.x + 0.5 {
            Some(PlacedSide::Left)
        } else if rect.x >= a.right() - 0.5 {
            Some(PlacedSide::Right)
        } else {
            None
        }
    });
    let hidden = spec.visibility == Visibility::NoOverflow && won.is_none();
    (
        rect,
        Placed {
            option: k as u8,
            side,
            hidden,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame<'a>(anchor: &'a dyn Fn(&str) -> Option<Rect>) -> Frame<'a> {
        Frame {
            cb: Rect::new(0.0, 0.0, 400.0, 300.0),
            anchor,
            size: [Size::Fit {
                min: 0.0,
                max: f32::INFINITY,
            }; 2],
            content: [(60.0, 20.0), (40.0, 40.0)],
        }
    }

    #[test]
    fn areas_insets_and_fallbacks() {
        let a = |_: &str| Some(Rect::new(100.0, 250.0, 50.0, 20.0));
        let f = frame(&a);
        // Below the anchor, centred on it (anchor-center), as wide as the
        // anchor's column lets it: it overflows the bottom, so the block
        // flip puts it above.
        let spec = AnchorSpec {
            base: Placement {
                area: parse_area("bottom center").unwrap(),
                ..Default::default()
            },
            tries: parse_tries("flip-block").unwrap(),
            ..Default::default()
        };
        let (r, placed) = resolve(&spec, &f);
        assert_eq!(r, Rect::new(100.0, 210.0, 50.0, 40.0));
        assert_eq!((placed.option, placed.side), (1, Some(PlacedSide::Top)));
        // `span-right` starts at the anchor's left edge; insets by anchor().
        let spec = AnchorSpec {
            base: Placement {
                area: parse_area("top span-right").unwrap(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(resolve(&spec, &f).0.x, 100.0);
        let spec = AnchorSpec {
            base: Placement {
                insets: [
                    None,
                    None,
                    Some(parse_inset("calc(anchor(top) + 4px)").unwrap()),
                    Some(parse_inset("anchor(--a right)").unwrap()),
                ],
                sizes: [Some(parse_size("anchor-size(width)").unwrap()), None],
                ..Default::default()
            },
            ..Default::default()
        };
        // bottom: 300 - 250 + 4 = 54 → its bottom at 246; left at 150.
        assert_eq!(resolve(&spec, &f).0, Rect::new(150.0, 206.0, 50.0, 40.0));
        assert!(parse_area("top top").is_err() && parse_inset("anchor(--a)").is_err());
    }
}
