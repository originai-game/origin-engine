//! CSS's box decorations and overflow in documents: `border` and its
//! longhands, `outline`, `box-shadow`, `overflow` and `scrollbar-width`,
//! lowered onto element props (CSS Backgrounds and Borders 3, CSS Basic UI
//! 4 §outline, CSS Overflow 3). A border's width is room inside the box,
//! as padding is (og-ui sizes boxes `border-box`); its paint, the
//! outline's and the shadows are the element's draw.
use super::css::{Tok, describe, number};
use og_core::ogt::Value;

/// The sides in CSS's order.
pub(crate) const SIDES: [&str; 4] = ["Top", "Right", "Bottom", "Left"];

pub(crate) const STYLES: [&str; 10] = [
    "none", "hidden", "solid", "dashed", "dotted", "double", "groove", "ridge", "inset", "outset",
];

/// A part of a border or outline shorthand: a width, a style or a colour.
enum Part {
    Width(Value),
    Style(String),
    Color(Vec<Value>),
}

fn width(t: &Tok) -> Option<Value> {
    match t {
        Tok::Ident(w) => Some(Value::Number(
            og_core::ogt::Number::parse(match w.as_str() {
                "thin" => "1",
                "medium" => "3",
                "thick" => "5",
                _ => return None,
            })
            .ok()?,
        )),
        Tok::Num(..) => super::css::want_len(t).ok(),
        Tok::Func(f, _) if f == "var" => super::css::want_len(t).ok(),
        _ => None,
    }
}

fn part(t: &Tok) -> Result<Part, String> {
    if let Some(w) = width(t) {
        return Ok(Part::Width(w));
    }
    if let Tok::Ident(s) = t
        && STYLES.contains(&s.as_str())
    {
        return Ok(Part::Style(s.clone()));
    }
    super::css::paint_vals(std::slice::from_ref(t))
        .map(Part::Color)
        .map_err(|_| {
            format!(
                "'{}' is not a border width, style ({}) or colour",
                describe(t),
                STYLES.join(", ")
            )
        })
}

/// `border`, `border-<side>`, `outline`: up to one width, one style and
/// one colour, in any order. Unset parts take their initial values
/// (medium, none, currentColor), as the shorthand resets them.
pub(crate) fn shorthand(name: &str, t: &[Tok]) -> Result<(Value, String, Vec<Value>), String> {
    let (mut w, mut s, mut c) = (None, None, None);
    for tok in t {
        if matches!(tok, Tok::Ident(x) if x.eq_ignore_ascii_case("currentcolor")) && c.is_none() {
            c = Some(vec![Value::Word("currentColor".into())]);
            continue;
        }
        if name == "outline" && matches!(tok, Tok::Ident(x) if x == "auto") && s.is_none() {
            s = Some("auto".into());
            continue;
        }
        match part(tok)? {
            Part::Width(v) if w.is_none() => w = Some(v),
            Part::Style(v) if s.is_none() => s = Some(v),
            Part::Color(v) if c.is_none() => c = Some(v),
            _ => return Err(format!("{name} takes one width, one style and one colour")),
        }
    }
    if t.is_empty() || t.len() > 3 {
        return Err(format!("{name} takes a width, a style and a colour"));
    }
    Ok((
        w.unwrap_or_else(|| number("3").expect("a number")),
        s.unwrap_or_else(|| "none".into()),
        c.unwrap_or_else(|| vec![Value::Word("currentColor".into())]),
    ))
}

/// One to four values for the four sides, CSS's way: all, vertical and
/// horizontal, top / horizontal / bottom, or top right bottom left.
pub(crate) fn four<T: Clone>(name: &str, v: Vec<T>) -> Result<[T; 4], String> {
    Ok(match v.as_slice() {
        [a] => [a.clone(), a.clone(), a.clone(), a.clone()],
        [a, b] => [a.clone(), b.clone(), a.clone(), b.clone()],
        [a, b, c] => [a.clone(), b.clone(), c.clone(), b.clone()],
        [a, b, c, d] => [a.clone(), b.clone(), c.clone(), d.clone()],
        _ => return Err(format!("{name} takes 1 to 4 values")),
    })
}

/// A border width (`thin`, `medium`, `thick` or a length).
pub(crate) fn want_width(t: &Tok) -> Result<Value, String> {
    width(t).ok_or_else(|| format!("'{}' is not a border width", describe(t)))
}

/// A border or outline style.
pub(crate) fn want_style(t: &Tok, outline: bool) -> Result<String, String> {
    match t {
        Tok::Ident(s) if STYLES.contains(&s.as_str()) && !(outline && s == "hidden") => {
            Ok(s.clone())
        }
        // An outline's `auto` is the platform's ring: og-ui's focus ring.
        Tok::Ident(s) if outline && s == "auto" => Ok(s.clone()),
        t => Err(format!(
            "'{}' is not a {} style: {}",
            describe(t),
            if outline { "outline" } else { "border" },
            STYLES.join(", ")
        )),
    }
}

/// One `box-shadow`: `[inset] <x> <y> [<blur> [<spread>]] [<colour>]`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Shadow {
    pub inset: bool,
    pub offset: [f32; 2],
    pub blur: f32,
    pub spread: f32,
    /// The colour's values; none: currentColor.
    pub color: Option<Vec<Value>>,
}

/// The top-level comma-separated items of `text` (commas inside a
/// function's parentheses stay in their item).
pub(crate) fn items(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0i32, 0);
    for (i, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                out.push(text[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(text[start..].trim());
    out
}

/// A `box-shadow` value: `none`, or shadows separated by commas.
pub(crate) fn shadows(text: &str) -> Result<Vec<Shadow>, String> {
    if text.trim() == "none" {
        return Ok(Vec::new());
    }
    items(text).into_iter().map(shadow).collect()
}

fn shadow(item: &str) -> Result<Shadow, String> {
    let t = super::css::tokens(item)?;
    let mut s = Shadow {
        inset: false,
        offset: [0.0; 2],
        blur: 0.0,
        spread: 0.0,
        color: None,
    };
    let mut lengths = Vec::new();
    for tok in &t {
        match tok {
            Tok::Ident(w) if w == "inset" && !s.inset => s.inset = true,
            Tok::Num(..) => match super::css::want_len(tok)? {
                Value::Number(n) => lengths.push(n.as_f32()),
                v => return Err(format!("'{v:?}' is not a shadow length")),
            },
            _ if s.color.is_none() => {
                s.color = Some(super::css::paint_vals(std::slice::from_ref(tok)).map_err(|_| {
                    format!("'{}' is not part of a box-shadow: [inset] <x> <y> [<blur> [<spread>]] [<colour>]", describe(tok))
                })?)
            }
            _ => return Err(format!("a box-shadow takes one colour: {item}")),
        }
    }
    match lengths.as_slice() {
        [x, y, rest @ ..] if rest.len() <= 2 => {
            s.offset = [*x, *y];
            s.blur = rest.first().copied().unwrap_or(0.0);
            s.spread = rest.get(1).copied().unwrap_or(0.0);
        }
        _ => {
            return Err(format!(
                "a box-shadow takes two to four lengths (x y [blur [spread]]): {item}"
            ));
        }
    }
    if s.blur < 0.0 {
        return Err(format!("a box-shadow's blur is not negative: {item}"));
    }
    Ok(s)
}

const OVERFLOW: [&str; 5] = ["visible", "hidden", "clip", "scroll", "auto"];

fn side_props(part: &str) -> Vec<&'static str> {
    // Interned names: the schema's own, by side.
    match part {
        "Width" => vec![
            "borderTopWidth",
            "borderRightWidth",
            "borderBottomWidth",
            "borderLeftWidth",
        ],
        "Style" => vec![
            "borderTopStyle",
            "borderRightStyle",
            "borderBottomStyle",
            "borderLeftStyle",
        ],
        _ => vec![
            "borderTopColor",
            "borderRightColor",
            "borderBottomColor",
            "borderLeftColor",
        ],
    }
}

fn side_index(side: &str) -> Option<usize> {
    SIDES.iter().position(|s| s.eq_ignore_ascii_case(side))
}

/// A logical border property (CSS Logical 1 §6): the physical sides it
/// means in horizontal-tb, left to right (CSS order), and its part (``,
/// `-width`, `-style`, `-color`).
pub(crate) fn logical(name: &str) -> Option<(&'static [usize], &str)> {
    let rest = name.strip_prefix("border-")?;
    const FLOWS: [(&str, &[usize]); 6] = [
        ("block-start", &[0]),
        ("block-end", &[2]),
        ("inline-start", &[3]),
        ("inline-end", &[1]),
        ("block", &[0, 2]),
        ("inline", &[3, 1]),
    ];
    FLOWS.iter().find_map(|(flow, sides)| {
        let part = rest.strip_prefix(flow)?;
        matches!(part, "" | "-width" | "-style" | "-color").then_some((*sides, part))
    })
}

/// The props a shorthand sets (several), or the one a longhand sets.
pub(crate) fn longhands(name: &str) -> Vec<&'static str> {
    if let Some((sides, part)) = logical(name) {
        let parts: &[&str] = match part {
            "-width" => &["Width"],
            "-style" => &["Style"],
            "-color" => &["Color"],
            _ => &["Width", "Style", "Color"],
        };
        return sides
            .iter()
            .flat_map(|&k| parts.iter().map(move |p| side_props(p)[k]))
            .collect();
    }
    match name {
        "border" => ["Width", "Style", "Color"]
            .into_iter()
            .flat_map(side_props)
            .collect(),
        "border-width" => side_props("Width"),
        "border-style" => side_props("Style"),
        "border-color" => side_props("Color"),
        "outline" => vec!["outlineWidth", "outlineStyle", "outlineColor"],
        "overflow" => vec!["overflowX", "overflowY"],
        n if n.ends_with("-radius") || n.starts_with("corner-") => corner_props(n),
        _ => match name.strip_prefix("border-").and_then(side_index) {
            Some(k) => vec![
                side_props("Width")[k],
                side_props("Style")[k],
                side_props("Color")[k],
            ],
            None => Vec::new(),
        },
    }
}

/// A colour, or none for `currentColor` (the prop left unset).
fn colour(t: &Tok) -> Result<Option<Vec<Value>>, String> {
    match t {
        Tok::Ident(c) if c.eq_ignore_ascii_case("currentcolor") => Ok(None),
        t => super::css::paint_vals(std::slice::from_ref(t)).map(Some),
    }
}

type Entries = Vec<(&'static str, Vec<Value>)>;

fn push_triple(
    out: &mut Entries,
    props: [&'static str; 3],
    (w, s, c): (Value, String, Vec<Value>),
) {
    out.push((props[0], vec![w]));
    out.push((props[1], vec![Value::Word(s)]));
    if !matches!(c.as_slice(), [Value::Word(w)] if w == "currentColor") {
        out.push((props[2], c));
    }
}

/// Lowers a border, outline, box-shadow or overflow declaration onto its
/// props. A colour of `currentColor` sets nothing: unset, the prop draws
/// in the element's text colour.
pub(crate) fn lower(name: &str, value: &str, t: &[Tok]) -> Result<Entries, String> {
    let mut out = Entries::new();
    if let Some((sides, part)) = logical(name) {
        // `border-block` sets both sides alike; `border-block-width: a b`
        // sets start then end.
        let each: Vec<&[Tok]> = match (sides.len(), part) {
            (2, "") | (1, _) => vec![t; sides.len()],
            (2, _) => match t {
                [a] => vec![std::slice::from_ref(a); 2],
                [a, b] => vec![std::slice::from_ref(a), std::slice::from_ref(b)],
                _ => return Err(format!("{name} takes one or two values (start end)")),
            },
            _ => unreachable!(),
        };
        for (&k, t) in sides.iter().zip(each) {
            let side = format!("border-{}{part}", SIDES[k].to_lowercase());
            if part == "-style" {
                let [v] = t else {
                    return Err(format!("{name} takes one style"));
                };
                out.push((
                    side_props("Style")[k],
                    vec![Value::Word(want_style(v, false)?)],
                ));
            } else {
                out.extend(lower(&side, value, t)?);
            }
        }
        return Ok(out);
    }
    let one = |what: &str| match t {
        [v] => Ok(v),
        _ => Err(format!("{name} takes one {what}")),
    };
    match name {
        "box-shadow" => {
            shadows(value)?;
            out.push(("boxShadow", vec![Value::String(value.trim().to_string())]));
        }
        "overflow" => {
            let words: Vec<&str> = t
                .iter()
                .map(|v| match v {
                    Tok::Ident(w) if OVERFLOW.contains(&w.as_str()) => Ok(w.as_str()),
                    v => Err(format!(
                        "'{}' is not an overflow: {}",
                        describe(v),
                        OVERFLOW.join(", ")
                    )),
                })
                .collect::<Result<_, _>>()?;
            let (x, y) = match words.as_slice() {
                [a] => (*a, *a),
                [a, b] => (*a, *b),
                _ => return Err("overflow takes one or two values (x y)".into()),
            };
            out.push(("overflowX", vec![Value::Word(x.into())]));
            out.push(("overflowY", vec![Value::Word(y.into())]));
        }
        "border" | "outline" => {
            let parts = shorthand(name, t)?;
            if name == "border" {
                for k in 0..4 {
                    push_triple(
                        &mut out,
                        [
                            side_props("Width")[k],
                            side_props("Style")[k],
                            side_props("Color")[k],
                        ],
                        parts.clone(),
                    );
                }
            } else {
                if parts.1 == "hidden" {
                    return Err("an outline's style is not hidden".into());
                }
                push_triple(
                    &mut out,
                    ["outlineWidth", "outlineStyle", "outlineColor"],
                    parts,
                );
            }
        }
        "border-width" => {
            let w: Vec<Value> = t.iter().map(want_width).collect::<Result<_, _>>()?;
            for (p, v) in side_props("Width").into_iter().zip(four(name, w)?) {
                out.push((p, vec![v]));
            }
        }
        "border-style" => {
            let s: Vec<String> = t
                .iter()
                .map(|v| want_style(v, false))
                .collect::<Result<_, _>>()?;
            for (p, v) in side_props("Style").into_iter().zip(four(name, s)?) {
                out.push((p, vec![Value::Word(v)]));
            }
        }
        "border-color" => {
            let c: Vec<Option<Vec<Value>>> = t.iter().map(colour).collect::<Result<_, _>>()?;
            for (p, v) in side_props("Color").into_iter().zip(four(name, c)?) {
                if let Some(v) = v {
                    out.push((p, v));
                }
            }
        }
        "outline-width" => out.push(("outlineWidth", vec![want_width(one("width")?)?])),
        "outline-color" => {
            if let Some(c) = colour(one("colour")?)? {
                out.push(("outlineColor", c));
            }
        }
        _ => {
            let rest = name.strip_prefix("border-").unwrap_or(name);
            let (side, part) = rest.split_once('-').unwrap_or((rest, ""));
            let k = side_index(side).ok_or_else(|| format!("'{name}' is not a property"))?;
            match part {
                "" => push_triple(
                    &mut out,
                    [
                        side_props("Width")[k],
                        side_props("Style")[k],
                        side_props("Color")[k],
                    ],
                    shorthand(name, t)?,
                ),
                "width" => out.push((side_props("Width")[k], vec![want_width(one("width")?)?])),
                "color" => {
                    if let Some(c) = colour(one("colour")?)? {
                        out.push((side_props("Color")[k], c));
                    }
                }
                _ => return Err(format!("'{name}' is not a property")),
            }
        }
    }
    Ok(out)
}

/// The corners in CSS's order, as `border-<v>-<h>-radius` names them.
pub(crate) const CORNERS: [&str; 4] = ["top-left", "top-right", "bottom-right", "bottom-left"];

/// The props of each corner's radii and shape (CSS order).
pub(crate) const CORNER_RADII: [&str; 4] = [
    "radiusTopLeft",
    "radiusTopRight",
    "radiusBottomRight",
    "radiusBottomLeft",
];
pub(crate) const CORNER_SHAPES: [&str; 4] = [
    "cornerTopLeftShape",
    "cornerTopRightShape",
    "cornerBottomRightShape",
    "cornerBottomLeftShape",
];

/// The physical corner a corner name means (`top-left`, the logical
/// `start-start` in horizontal-tb, left to right), in CSS order.
fn corner_index(name: &str) -> Option<usize> {
    let physical = match name {
        "start-start" => "top-left",
        "start-end" => "top-right",
        "end-end" => "bottom-right",
        "end-start" => "bottom-left",
        n => n,
    };
    CORNERS.iter().position(|c| *c == physical)
}

/// The corner props a `border-radius` or `corner-shape` property sets: the
/// four for the shorthands (`corner-top-shape` its side's two), one for a
/// longhand; empty when it is neither.
pub(crate) fn corner_props(name: &str) -> Vec<&'static str> {
    if name == "border-radius" {
        return CORNER_RADII.to_vec();
    }
    if name == "corner-shape" {
        return CORNER_SHAPES.to_vec();
    }
    if let Some(c) = name
        .strip_prefix("border-")
        .and_then(|n| n.strip_suffix("-radius"))
        .and_then(corner_index)
    {
        return vec![CORNER_RADII[c]];
    }
    let Some(rest) = name
        .strip_prefix("corner-")
        .and_then(|n| n.strip_suffix("-shape"))
    else {
        return Vec::new();
    };
    if let Some(c) = corner_index(rest) {
        return vec![CORNER_SHAPES[c]];
    }
    // A side's two corners: corner-top-shape, corner-block-start-shape…
    let pair = match rest {
        "top" | "block-start" => [0, 1],
        "right" | "inline-end" => [1, 2],
        "bottom" | "block-end" => [2, 3],
        "left" | "inline-start" => [3, 0],
        _ => return Vec::new(),
    };
    pair.iter().map(|&k| CORNER_SHAPES[k]).collect()
}

/// A radius: px (or `0`) or a percentage of the box's side, as (px, fraction).
fn radius_len(t: &Tok) -> Result<(f32, f32), String> {
    let bad = || format!("'{}' is not a radius (<n>px or <n>%)", describe(t));
    match t {
        Tok::Num(n, u) => {
            let v: f32 = n.parse().map_err(|_| bad())?;
            if v < 0.0 {
                return Err(format!("a radius is not negative: {}", describe(t)));
            }
            match u.as_str() {
                "%" => Ok((0.0, v / 100.0)),
                "px" => Ok((v, 0.0)),
                "" if v == 0.0 => Ok((0.0, 0.0)),
                _ => Err(bad()),
            }
        }
        _ => Err(bad()),
    }
}

/// One corner's radii, as its prop holds them: `<h> [<v>]`.
pub(crate) fn corner_radius(text: &str) -> Result<[(f32, f32); 2], String> {
    let t = super::css::tokens(text)?;
    match t.as_slice() {
        [h] => {
            let h = radius_len(h)?;
            Ok([h, h])
        }
        [h, v] => Ok([radius_len(h)?, radius_len(v)?]),
        _ => Err("a corner radius takes one or two lengths (horizontal vertical)".into()),
    }
}

/// A `corner-shape` value: its superellipse parameter K (CSS Borders 4).
pub(crate) fn corner_shape(t: &Tok) -> Result<f32, String> {
    Ok(match t {
        Tok::Ident(w) => match w.as_str() {
            "round" => 1.0,
            "squircle" => 2.0,
            "bevel" => 0.0,
            "scoop" => -1.0,
            "notch" => f32::NEG_INFINITY,
            "square" => f32::INFINITY,
            _ => return Err(format!("'{w}' is not a corner shape: {SHAPES}")),
        },
        Tok::Func(f, a) if f == "superellipse" => match a.as_slice() {
            [v] => match v.as_slice() {
                [Tok::Ident(i)] if i == "infinity" => f32::INFINITY,
                [Tok::Ident(i)] if i == "-infinity" => f32::NEG_INFINITY,
                [Tok::Num(n, u)] if u.is_empty() => n
                    .parse()
                    .map_err(|_| format!("superellipse({n}) takes a number"))?,
                _ => return Err("superellipse(<number> | infinity | -infinity)".into()),
            },
            _ => return Err("superellipse(<number> | infinity | -infinity)".into()),
        },
        t => return Err(format!("'{}' is not a corner shape: {SHAPES}", describe(t))),
    })
}

const SHAPES: &str = "round, squircle, bevel, scoop, notch, square or superellipse(<K>)";

/// Lowers `border-radius`, its longhands, `corner-shape` and its longhands
/// onto the corner props. A uniform `border-radius: <n>px` is the element's
/// `radius` (which states and transitions animate) and clears the corners.
pub(crate) fn lower_corners(name: &str, t: &[Tok]) -> Result<Entries, String> {
    let props = corner_props(name);
    let mut out = Entries::new();
    let text = |v: String| vec![Value::String(v)];
    if name.contains("shape") {
        let k: Vec<f32> = t.iter().map(corner_shape).collect::<Result<_, _>>()?;
        let k = match props.len() {
            4 => four(name, k)?.to_vec(),
            n if k.len() == 1 => vec![k[0]; n],
            _ => return Err(format!("{name} takes one corner shape")),
        };
        for (p, k) in props.into_iter().zip(k) {
            out.push((p, text(shape_text(k))));
        }
        return Ok(out);
    }
    if props.len() == 1 {
        if t.is_empty() || t.len() > 2 {
            return Err(format!(
                "{name} takes one or two lengths (horizontal vertical)"
            ));
        }
        for v in t {
            radius_len(v)?;
        }
        out.push((props[0], text(written(t))));
        return Ok(out);
    }
    // border-radius: 1 to 4 horizontal radii [/ 1 to 4 vertical].
    let split = t.iter().position(|v| matches!(v, Tok::Slash));
    let (h, v) = match split {
        Some(k) => (&t[..k], Some(&t[k + 1..])),
        None => (t, None),
    };
    if let ([one], None) = (h, v)
        && matches!(one, Tok::Num(_, u) if u != "%")
    {
        out.push(("radius", vec![super::css::want_len(one)?]));
        for p in CORNER_RADII {
            out.push((p, text(String::new())));
        }
        return Ok(out);
    }
    for x in h.iter().chain(v.into_iter().flatten()) {
        radius_len(x)?;
    }
    let h = four(name, h.to_vec())?;
    let v = match v {
        Some(v) => four(name, v.to_vec())?,
        None => h.clone(),
    };
    for (k, p) in CORNER_RADII.into_iter().enumerate() {
        out.push((p, text(written(&[h[k].clone(), v[k].clone()]))));
    }
    Ok(out)
}

fn written(t: &[Tok]) -> String {
    t.iter().map(describe).collect::<Vec<_>>().join(" ")
}

fn shape_text(k: f32) -> String {
    if k == f32::INFINITY {
        "superellipse(infinity)".into()
    } else if k == f32::NEG_INFINITY {
        "superellipse(-infinity)".into()
    } else {
        format!("superellipse({k})")
    }
}
