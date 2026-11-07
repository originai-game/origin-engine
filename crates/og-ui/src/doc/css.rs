//! The CSS dialect of documents (`version 2`, docs/ui.md §10.4): an
//! element's look and layout as CSS declarations (`width: stretch;
//! max-width: 640px`), its interaction states as pseudo-classes (`:hover {
//! … }`), breakpoints as `@media phone { … }`, tokens as `var(--panel)` and
//! the game's extensions as `-og-` properties. Its structure and behaviour
//! (elements, `text`, `variant`, `bind`, `on`, `for`, `if`, `use`, named
//! state groups, timelines) keep their `.ogt` lines.
//!
//! [`lower`] reads a version 2 text into the `.ogt` entries the document
//! builder reads (each entry on its source line, so errors, the probe's
//! `source` and the write-back name the version 2 file), [`migrate`] writes
//! a version 1 text as version 2, keeping its comments and line order, and
//! [`decl_line`] writes one line of declarations (the write-back). One
//! table, [`STYLE`], drives all three.
//!
//! The subset is strict: a property or value it does not know is an error
//! that names the form to use; there is no cascade beyond the element's own
//! declarations (its `@media` blocks and states override them, later wins)
//! and no `!important`.

use og_core::ogt::{self, Block, Document, Entry, Line, Number, Value};

/// How a property's value is written.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Conv {
    /// A length in CSS px: `12px` (`0` alone).
    Len,
    /// A plain number.
    Num,
    /// An angle: `0.5rad`, `30deg`, `0.25turn` (radians in the element).
    Angle,
    /// `var(--role)` or `#rrggbb[aa]`.
    Paint,
    /// `fit-content`, `stretch`, `<n>px`, `<n>%`, with `min-`/`max-`.
    Size,
    /// One to four lengths, CSS order (top right bottom left).
    Insets,
    /// One of two words for true and false.
    Bool(&'static str, &'static str),
    /// A word written in kebab case (`spaceBetween` ↔ `space-between`).
    Kebab,
    /// A word mapped through a table (element ↔ CSS).
    Kw(&'static [(&'static str, &'static str)]),
    /// A token: `var(--<namespace>-<name>)`.
    Var(&'static str),
    /// A quoted string.
    Str,
    /// An identifier, or a string when it is not one.
    Ident,
    /// Quoted strings.
    Strs,
    /// Grid tracks: `120px 1fr auto 25% minmax(80px, 200px)`.
    Tracks,
    /// `none`, `<ms>ms [ease]`, `-og-spring(<ms>ms, <bounce>)`, `var(--motion-<name>)`.
    Transition,
    /// `<kind> [<distance>px] [<transition>]`.
    Presence,
    /// Two fractions as percentages: `50% 100%`.
    Origin,
    /// A Rive artboard's id as `rive("<file>.riv#<Artboard>")` (its
    /// property is `-og-frame`, which otherwise names a style part).
    Rive,
    /// Written with its partner in one property (`translate: <x> <y>`,
    /// `scale: <x> <y>`, `-og-skew`, `grid-column`/`grid-row`,
    /// `grid-auto-flow`).
    Joint,
}

const CURSORS: &[(&str, &str)] = &[
    ("auto", "auto"),
    ("default", "default"),
    ("pointer", "pointer"),
    ("text", "text"),
    ("grab", "grab"),
    ("grabbing", "grabbing"),
    ("move", "move"),
    ("notAllowed", "not-allowed"),
    ("crosshair", "crosshair"),
    ("help", "help"),
    ("wait", "wait"),
    ("progress", "progress"),
    ("resizeEw", "ew-resize"),
    ("resizeNs", "ns-resize"),
    ("resizeNesw", "nesw-resize"),
    ("resizeNwse", "nwse-resize"),
    ("zoomIn", "zoom-in"),
    ("zoomOut", "zoom-out"),
    ("none", "none"),
];

const POSITIONS: &[(&str, &str)] = &[
    ("flow", "static"),
    ("absolute", "absolute"),
    ("screen", "fixed"),
    ("anchored", "-og-anchored"),
];

/// Every element prop written as a declaration: (prop, CSS property, how).
/// A prop not here is an attribute line (`text "Buy"`, `variant primary`).
pub const STYLE: &[(&str, &str)] = &[
    ("width", "width"),
    ("height", "height"),
    ("padding", "padding"),
    ("gap", "gap"),
    ("align", "justify-content"),
    ("cross", "align-items"),
    ("wrap", "flex-wrap"),
    ("direction", "flex-direction"),
    ("background", "background"),
    ("radius", "border-radius"),
    ("frame", "-og-frame"),
    ("rive", "-og-frame"),
    ("aspect", "aspect-ratio"),
    ("fade", "-og-scroll-fade"),
    ("fadeInto", "-og-scroll-fade-into"),
    ("x", "translate"),
    ("y", "translate"),
    ("scale", "scale"),
    ("scaleX", "scale"),
    ("scaleY", "scale"),
    ("rotate", "rotate"),
    ("skewX", "-og-skew"),
    ("skewY", "-og-skew"),
    ("pivot", "transform-origin"),
    ("opacity", "opacity"),
    ("z", "z-index"),
    ("position", "position"),
    ("start", "inset-inline-start"),
    ("top", "top"),
    ("end", "inset-inline-end"),
    ("bottom", "bottom"),
    ("centerX", "-og-center-x"),
    ("centerY", "-og-center-y"),
    ("anchorTo", "-og-anchor"),
    ("anchorAt", "-og-anchor-at"),
    ("anchorSelf", "-og-anchor-self"),
    ("hitSlop", "-og-hit-slop"),
    ("columns", "grid-template-columns"),
    ("rows", "grid-template-rows"),
    ("areas", "grid-template-areas"),
    ("flow", "grid-auto-flow"),
    ("dense", "grid-auto-flow"),
    ("area", "grid-area"),
    ("cell", "grid-column"),
    ("span", "grid-column"),
    ("overlaps", "-og-overlaps"),
    ("motion", "transition"),
    ("layoutMotion", "-og-layout-transition"),
    ("enter", "-og-enter"),
    ("exit", "-og-exit"),
    ("cursor", "cursor"),
    ("reduced", "-og-reduced-motion"),
    ("step", "font"),
    ("fitMin", "-og-fit-min"),
    ("font", "font-family"),
    ("color", "color"),
    ("onScene", "-og-on-scene"),
    ("uppercase", "text-transform"),
    ("secondaryTint", "-og-secondary-tint"),
    ("textAlign", "text-align"),
    ("nowrap", "white-space"),
    ("maxLines", "line-clamp"),
    ("ellipsis", "text-overflow"),
    ("tint", "-og-tint"),
    ("fit", "object-fit"),
];

fn conv(prop: &str) -> Conv {
    match prop {
        "width" | "height" => Conv::Size,
        "padding" => Conv::Insets,
        "gap" | "radius" | "fade" | "start" | "top" | "end" | "bottom" | "hitSlop" => Conv::Len,
        "aspect" | "opacity" | "z" | "scale" | "maxLines" => Conv::Num,
        "rotate" => Conv::Angle,
        "background" | "fadeInto" | "color" | "secondaryTint" | "tint" | "ink" => Conv::Paint,
        "align" | "cross" | "direction" | "frame" | "anchorAt" | "anchorSelf" | "reduced"
        | "textAlign" | "fit" => Conv::Kebab,
        "wrap" => Conv::Bool("wrap", "nowrap"),
        "centerX" | "centerY" | "onScene" => Conv::Bool("true", "false"),
        "uppercase" => Conv::Bool("uppercase", "none"),
        "nowrap" => Conv::Bool("nowrap", "normal"),
        "ellipsis" => Conv::Bool("ellipsis", "clip"),
        "position" => Conv::Kw(POSITIONS),
        "cursor" => Conv::Kw(CURSORS),
        "step" | "fitMin" => Conv::Var("text"),
        "font" => Conv::Var("font"),
        "anchorTo" => Conv::Str,
        "rive" => Conv::Rive,
        "area" => Conv::Ident,
        "areas" | "overlaps" => Conv::Strs,
        "columns" | "rows" => Conv::Tracks,
        "motion" | "layoutMotion" => Conv::Transition,
        "enter" | "exit" => Conv::Presence,
        "pivot" => Conv::Origin,
        _ => Conv::Joint,
    }
}

/// The CSS property an element prop is written as; `None`: an attribute.
pub fn css_name(prop: &str) -> Option<&'static str> {
    STYLE.iter().find(|(p, _)| *p == prop).map(|(_, c)| *c)
}

/// The values a state of a group sets, as their CSS properties.
const STATE_VALUES: &[(&str, &str)] = &[
    ("x", "translate"),
    ("y", "translate"),
    ("scale", "scale"),
    ("scaleX", "scale"),
    ("scaleY", "scale"),
    ("rotate", "rotate"),
    ("skewX", "-og-skew"),
    ("skewY", "-og-skew"),
    ("opacity", "opacity"),
    ("radius", "border-radius"),
    ("z", "z-index"),
    ("background", "background"),
    ("ink", "color"),
];

/// The interaction states and their pseudo-classes.
const PSEUDO: &[(&str, &str)] = &[
    ("hover", "hover"),
    ("pressed", "active"),
    ("focused", "focus-visible"),
    ("disabled", "disabled"),
];

/// The neutral value of an animatable value (a state's `normal` takes it
/// when the element declares none).
fn neutral(key: &str) -> Option<&'static str> {
    match key {
        "x" | "y" | "rotate" | "skewX" | "skewY" => Some("0"),
        "scale" | "scaleX" | "scaleY" | "opacity" => Some("1"),
        _ => None,
    }
}

/// CSS that is not part of the subset, and what to write instead.
fn unsupported(name: &str) -> Option<&'static str> {
    Some(match name {
        "display" => "an element's display is its kind: `row Name {`, `column`, `stack` or `grid`",
        "flex" | "flex-grow" | "flex-shrink" | "flex-basis" => {
            "write `width: stretch` (or `height`) to take the room left"
        }
        "margin" | "margin-top" | "margin-bottom" | "margin-left" | "margin-right"
        | "margin-inline" | "margin-block" => {
            "og-ui has no margin: give the parent a `gap` or a `padding`"
        }
        "float" | "clear" => "write `display` as the kind: a `row` or a `grid`",
        "left" => "write `inset-inline-start` (it mirrors right to left)",
        "right" => "write `inset-inline-end` (it mirrors right to left)",
        "font-size" | "font-weight" | "line-height" | "letter-spacing" => {
            "text takes a step of the style's type ramp: `font: var(--text-caption)`"
        }
        "box-shadow" | "border" | "border-color" | "border-width" | "outline" => {
            "shadows and borders are the style's: name a part (`part frame`) or `-og-frame`, and dress it in the style pack"
        }
        "overflow" | "overflow-x" | "overflow-y" => {
            "a scrolling element is `scroll Name {`; what clips is the style's"
        }
        "transform" => {
            "write each part as its own property: `translate: 0 -4px`, `scale: 1.05`, `rotate: 0.1rad`, `-og-skew`"
        }
        "visibility" => "write the attribute `visible false` (or `visible bind …`)",
        "background-color" | "background-image" => "write `background: var(--panel)` or `#rrggbb`",
        "transition-duration" | "transition-timing-function" | "animation" => {
            "write `transition: 200ms ease-out`, `-og-spring(300ms, 0.1)` or `var(--motion-ui)`; a timeline plays an animation"
        }
        "inset-inline" | "inset" | "inset-block" => {
            "write each side: `top`, `bottom`, `inset-inline-start`, `inset-inline-end`"
        }
        _ => return None,
    })
}

// ---- words and numbers

/// `spaceBetween` → `space-between`.
pub(crate) fn kebab(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_uppercase() {
            out.push('-');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// `space-between` → `spaceBetween`.
pub(crate) fn camel(s: &str) -> String {
    let mut out = String::new();
    let mut up = false;
    for c in s.chars() {
        if c == '-' {
            up = true;
        } else if up {
            out.push(c.to_ascii_uppercase());
            up = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// A decimal text times 10^k, exactly (`0.25`, 2 → `25`).
pub(crate) fn shift(text: &str, k: i32) -> String {
    if text.contains(['e', 'E']) {
        let v: f64 = text.parse().unwrap_or(0.0);
        return format!("{}", v * 10f64.powi(k));
    }
    let (neg, t) = match text.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, text),
    };
    let (int, frac) = t.split_once('.').unwrap_or((t, ""));
    let digits: String = format!("{int}{frac}");
    let point = int.len() as i32 + k;
    let mut s = if point <= 0 {
        format!("0.{}{digits}", "0".repeat((-point) as usize))
    } else if point as usize >= digits.len() {
        format!("{digits}{}", "0".repeat(point as usize - digits.len()))
    } else {
        format!(
            "{}.{}",
            &digits[..point as usize],
            &digits[point as usize..]
        )
    };
    if s.contains('.') {
        s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    let s = s.trim_start_matches('0');
    let s = if s.is_empty() || s.starts_with('.') {
        format!("0{s}")
    } else {
        s.to_string()
    };
    if neg && s != "0" { format!("-{s}") } else { s }
}

pub(crate) fn number(text: &str) -> Result<Value, String> {
    Number::parse(text)
        .map(Value::Number)
        .map_err(|_| format!("'{text}' is not a number"))
}

fn text_of(v: &Value) -> String {
    match v {
        Value::Number(n) => n.text().to_string(),
        Value::String(s) | Value::Word(s) => s.clone(),
    }
}

pub(crate) fn quoted(s: &str) -> String {
    ogt::value_text(&Value::String(s.to_string())).unwrap_or_default()
}

fn is_ident(s: &str) -> bool {
    let mut c = s.chars();
    matches!(c.next(), Some(f) if f.is_ascii_alphabetic() || f == '_')
        && c.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

// ---- element values to CSS

fn px(v: &Value) -> Result<String, String> {
    match v {
        Value::Number(n) if n.text() == "0" => Ok("0".into()),
        Value::Number(n) => Ok(format!("{}px", n.text())),
        other => Err(format!("'{}' is not a length", text_of(other))),
    }
}

fn bound(vals: &[Value]) -> Option<Result<String, String>> {
    match vals {
        [Value::Word(b), e] if b == "bind" => {
            let src = text_of(e);
            let simple = !src.is_empty()
                && src
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '$'));
            Some(Ok(if simple {
                format!("bind({src})")
            } else {
                format!("bind({})", quoted(&src))
            }))
        }
        [Value::Word(b), ..] if b == "bind" => Some(Err(
            "a declaration binds without a format (a format is a text's: `text bind gold compact`)"
                .into(),
        )),
        _ => None,
    }
}

fn paint_css(vals: &[Value]) -> Result<String, String> {
    match vals {
        [Value::Word(t), Value::Word(role)] if t == "token" => {
            Ok(format!("var(--{})", kebab(role)))
        }
        [Value::String(s)] | [Value::Word(s)] if s.starts_with('#') => Ok(s.clone()),
        [Value::Word(role)] | [Value::String(role)] => Ok(format!("var(--{})", kebab(role))),
        _ => Err("a paint: token <role> or \"#rrggbb\"".into()),
    }
}

fn transition_css(vals: &[Value]) -> Result<String, String> {
    let w: Vec<String> = vals.iter().map(text_of).collect();
    let num = |i: usize| w.get(i).cloned().ok_or("a transition needs its numbers");
    match w.first().map(String::as_str) {
        Some("instant") => Ok("none".into()),
        Some("spring") => Ok(match w.get(2) {
            Some(b) => format!("-og-spring({}ms, {b})", num(1)?),
            None => format!("-og-spring({}ms)", num(1)?),
        }),
        Some("tween") => {
            let ease = match w.get(2).map(String::as_str) {
                None => "",
                Some("linear") => " linear",
                Some("out") => " ease-out",
                Some("exit") => " ease-in",
                Some("inOut") => " ease-in-out",
                Some(e) => return Err(format!("'{e}' is not an ease")),
            };
            Ok(format!("{}ms{ease}", num(1)?))
        }
        Some("token") => Ok(format!("var(--motion-{})", kebab(&num(1)?))),
        Some(t) => Ok(format!("var(--motion-{})", kebab(t))),
        None => Err("a transition".into()),
    }
}

fn presence_css(vals: &[Value]) -> Result<String, String> {
    let kind = vals
        .first()
        .map(text_of)
        .ok_or("a presence: fade, slideUp …")?;
    let mut out = kebab(&kind);
    let mut rest = &vals[1..];
    if let Some(Value::Number(n)) = rest.first() {
        out.push(' ');
        out.push_str(&px(&Value::Number(n.clone()))?);
        rest = &rest[1..];
    }
    if !rest.is_empty() {
        out.push(' ');
        out.push_str(&transition_css(rest)?);
    }
    Ok(out)
}

fn track_css(v: &Value) -> Result<String, String> {
    match v {
        Value::Number(_) => px(v),
        Value::Word(s) | Value::String(s) => match s.strip_prefix("minmax:") {
            Some(r) => {
                let (a, b) = r.split_once(':').ok_or("minmax:<min>:<max>")?;
                let len = |t: &str| {
                    if t == "0" {
                        "0".to_string()
                    } else {
                        format!("{t}px")
                    }
                };
                Ok(format!("minmax({}, {})", len(a), len(b)))
            }
            None => Ok(s.clone()),
        },
    }
}

/// One element value as its CSS value (`conv` of a prop not written jointly).
fn value_css(prop: &str, c: Conv, vals: &[Value]) -> Result<String, String> {
    if let Some(b) = bound(vals) {
        return b;
    }
    let one = || match vals {
        [v] => Ok(v),
        _ => Err(format!("{prop} takes one value")),
    };
    match c {
        Conv::Len => px(one()?),
        Conv::Num => match one()? {
            Value::Number(n) => Ok(n.text().to_string()),
            v => Err(format!("'{}' is not a number", text_of(v))),
        },
        Conv::Angle => match one()? {
            Value::Number(n) if n.text() == "0" => Ok("0".into()),
            Value::Number(n) => Ok(format!("{}rad", n.text())),
            v => Err(format!("'{}' is not an angle", text_of(v))),
        },
        Conv::Paint => paint_css(vals),
        Conv::Insets => {
            let n: Vec<String> = vals.iter().map(px).collect::<Result<_, _>>()?;
            Ok(match n.as_slice() {
                [a] => a.clone(),
                [x, y] => format!("{y} {x}"),
                [l, t, r, b] => format!("{t} {r} {b} {l}"),
                _ => return Err("padding takes 1, 2 or 4 numbers".into()),
            })
        }
        Conv::Bool(t, f) => match vals {
            [] => Ok(t.into()),
            [v] => match v.as_bool() {
                Some(true) => Ok(t.into()),
                Some(false) => Ok(f.into()),
                None => Err(format!("{prop} takes true or false")),
            },
            _ => Err(format!("{prop} takes true or false")),
        },
        Conv::Kebab => Ok(kebab(&text_of(one()?))),
        Conv::Kw(table) => {
            let w = text_of(one()?);
            table
                .iter()
                .find(|(e, _)| *e == w)
                .map(|(_, c)| c.to_string())
                .ok_or_else(|| format!("'{w}' is not a {prop}"))
        }
        Conv::Var(ns) => Ok(format!("var(--{ns}-{})", kebab(&text_of(one()?)))),
        Conv::Str => Ok(quoted(&text_of(one()?))),
        Conv::Rive => Ok(format!("rive({})", quoted(&text_of(one()?)))),
        Conv::Ident => {
            let w = text_of(one()?);
            Ok(if is_ident(&w) { w } else { quoted(&w) })
        }
        Conv::Strs => Ok(vals
            .iter()
            .map(|v| quoted(&text_of(v)))
            .collect::<Vec<_>>()
            .join(" ")),
        Conv::Tracks => Ok(vals
            .iter()
            .map(track_css)
            .collect::<Result<Vec<_>, _>>()?
            .join(" ")),
        Conv::Transition => transition_css(vals),
        Conv::Presence => presence_css(vals),
        Conv::Origin => match vals {
            [Value::Number(a), Value::Number(b)] => {
                Ok(format!("{}% {}%", shift(a.text(), 2), shift(b.text(), 2)))
            }
            _ => Err("pivot takes two fractions".into()),
        },
        Conv::Size => {
            if let [Value::Number(n)] = vals {
                return px(&Value::Number(n.clone()));
            }
            let w = vals.first().map(text_of).unwrap_or_default();
            match w.as_str() {
                "fixed" => px(vals.get(1).ok_or("fixed <px>")?),
                "percent" => Ok(format!(
                    "{}%",
                    shift(&vals.get(1).map(text_of).ok_or("percent <0–1>")?, 2)
                )),
                _ => Err("a size".into()),
            }
        }
        Conv::Joint => Err(format!("{prop} is written with its partner")),
    }
}

/// The declarations of `props` (element props, or a state's values when
/// `state`), in order, as one line: `width: stretch; max-width: 640px`.
/// Props written jointly (`x` and `y` as `translate`) share one
/// declaration; one missing its partner takes `partner(prop)`'s value
/// (the element's own, else its neutral value).
pub(crate) fn decl_line(
    props: &[(String, Vec<Value>)],
    state: bool,
    partner: &dyn Fn(&str) -> Option<Vec<Value>>,
) -> Result<String, String> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut done: Vec<&str> = Vec::new();
    let get = |k: &str| props.iter().find(|(p, _)| p == k).map(|(_, v)| v.clone());
    let fill = |k: &str| -> Result<String, String> {
        let v = get(k)
            .or_else(|| partner(k))
            .or_else(|| neutral(k).map(|n| vec![Value::Number(Number::parse(n).unwrap())]))
            .ok_or_else(|| format!("{k} has no value"))?;
        if let Some(b) = bound(&v) {
            return b;
        }
        match conv_joint(k) {
            Conv::Angle => value_css(k, Conv::Angle, &v),
            Conv::Len => value_css(k, Conv::Len, &v),
            _ => value_css(k, Conv::Num, &v),
        }
    };
    for (prop, vals) in props {
        let prop = prop.as_str();
        if done.contains(&prop) {
            continue;
        }
        let css = if state {
            STATE_VALUES
                .iter()
                .find(|(p, _)| *p == prop)
                .map(|(_, c)| *c)
        } else {
            css_name(prop)
        }
        .ok_or_else(|| format!("'{prop}' has no CSS form"))?;
        match prop {
            "x" | "y" => {
                done.extend(["x", "y"]);
                out.push((css.into(), format!("{} {}", fill("x")?, fill("y")?)));
            }
            "scaleX" | "scaleY" => {
                done.extend(["scaleX", "scaleY"]);
                out.push((
                    css.into(),
                    format!("{} {}", fill("scaleX")?, fill("scaleY")?),
                ));
            }
            "skewX" | "skewY" => {
                done.extend(["skewX", "skewY"]);
                out.push((css.into(), format!("{} {}", fill("skewX")?, fill("skewY")?)));
            }
            "flow" | "dense" => {
                done.extend(["flow", "dense"]);
                let mut v = Vec::new();
                if let Some(f) = get("flow") {
                    v.push(value_css("flow", Conv::Kebab, &f)?);
                }
                if let Some(d) = get("dense")
                    && d.first().and_then(Value::as_bool).unwrap_or(true)
                {
                    v.push("dense".into());
                }
                if v.is_empty() {
                    v.push("row".into());
                }
                out.push((css.into(), v.join(" ")));
            }
            "cell" | "span" => {
                done.extend(["cell", "span"]);
                let pair = |k: &str| -> Result<Option<[String; 2]>, String> {
                    match get(k).as_deref() {
                        None => Ok(None),
                        Some([a, b]) => Ok(Some([text_of(a), text_of(b)])),
                        Some(_) => Err(format!("{k} takes two numbers")),
                    }
                };
                let (cell, span) = (pair("cell")?, pair("span")?);
                for (axis, name) in [(0, "grid-column"), (1, "grid-row")] {
                    let start = cell.as_ref().map(|c| c[axis].clone()).filter(|s| s != "0");
                    let n = span.as_ref().map(|c| c[axis].clone()).filter(|s| s != "0");
                    let v = match (start, n) {
                        (Some(s), Some(n)) => format!("{s} / span {n}"),
                        (Some(s), None) => s,
                        (None, Some(n)) => format!("span {n}"),
                        (None, None) => continue,
                    };
                    out.push((name.into(), v));
                }
            }
            "width" | "height" if !state => {
                done.push(prop);
                if bound(vals).is_some() {
                    out.push((css.into(), value_css(prop, Conv::Len, vals)?));
                    continue;
                }
                let w = vals.first().map(text_of).unwrap_or_default();
                match w.as_str() {
                    "fit" | "grow" => {
                        out.push((
                            css.into(),
                            if w == "fit" { "fit-content" } else { "stretch" }.into(),
                        ));
                        if let Some(min) = vals.get(1)
                            && text_of(min) != "0"
                        {
                            out.push((format!("min-{css}"), px(min)?));
                        }
                        if let Some(max) = vals.get(2) {
                            out.push((format!("max-{css}"), px(max)?));
                        }
                    }
                    _ => out.push((css.into(), value_css(prop, Conv::Size, vals)?)),
                }
            }
            _ => {
                done.push(prop);
                let c = if prop == "ink" {
                    Conv::Paint
                } else {
                    conv(prop)
                };
                out.push((css.into(), value_css(prop, c, vals)?));
            }
        }
    }
    Ok(out
        .into_iter()
        .map(|(n, v)| format!("{n}: {v}"))
        .collect::<Vec<_>>()
        .join("; "))
}

fn conv_joint(k: &str) -> Conv {
    match k {
        "x" | "y" => Conv::Len,
        "skewX" | "skewY" => Conv::Angle,
        _ => Conv::Num,
    }
}

// ---- reading CSS values

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Tok {
    Ident(String),
    /// A number with its unit (`px`, `%`, `ms`, empty).
    Num(String, String),
    Str(String),
    Hash(String),
    Func(String, Vec<Vec<Tok>>),
    Slash,
}

pub(crate) fn describe(t: &Tok) -> String {
    match t {
        Tok::Ident(s) | Tok::Hash(s) => s.clone(),
        Tok::Num(n, u) => format!("{n}{u}"),
        Tok::Str(s) => quoted(s),
        Tok::Func(f, _) => format!("{f}(…)"),
        Tok::Slash => "/".into(),
    }
}

/// The value of one declaration as tokens.
pub(crate) fn tokens(s: &str) -> Result<Vec<Tok>, String> {
    let c: Vec<char> = s.chars().collect();
    let mut i = 0;
    let (toks, end) = tokens_in(&c, &mut i, false)?;
    if end {
        return Err("a ')' closes nothing".into());
    }
    Ok(toks.into_iter().flatten().collect())
}

/// Tokens up to the end (or, in a function, its `)`), split at commas.
fn tokens_in(c: &[char], i: &mut usize, func: bool) -> Result<(Vec<Vec<Tok>>, bool), String> {
    let mut args: Vec<Vec<Tok>> = vec![Vec::new()];
    while *i < c.len() {
        let ch = c[*i];
        if ch.is_whitespace() {
            *i += 1;
        } else if ch == ')' {
            *i += 1;
            let _ = func;
            return Ok((args, true));
        } else if ch == ',' {
            *i += 1;
            args.push(Vec::new());
        } else if ch == '/' {
            *i += 1;
            args.last_mut().unwrap().push(Tok::Slash);
        } else if ch == '"' {
            let start = *i;
            *i += 1;
            while *i < c.len() && c[*i] != '"' {
                if c[*i] == '\\' {
                    *i += 1;
                }
                *i += 1;
            }
            if *i >= c.len() {
                return Err("a string is not closed".into());
            }
            *i += 1;
            let lit: String = c[start..*i].iter().collect();
            match ogt::parse_line(&format!("k {lit}"), 0) {
                Ok(Line::Entry { entry, .. }) => match entry.values.as_slice() {
                    [Value::String(s)] => args.last_mut().unwrap().push(Tok::Str(s.clone())),
                    _ => return Err(format!("{lit} is not a string")),
                },
                _ => return Err(format!("{lit} is not a string")),
            }
        } else if ch == '#' {
            let start = *i;
            *i += 1;
            while *i < c.len() && c[*i].is_ascii_alphanumeric() {
                *i += 1;
            }
            args.last_mut()
                .unwrap()
                .push(Tok::Hash(c[start..*i].iter().collect()));
        } else if ch.is_ascii_digit()
            || (ch == '-' || ch == '.')
                && c.get(*i + 1)
                    .is_some_and(|d| d.is_ascii_digit() || *d == '.')
        {
            let start = *i;
            *i += 1;
            while *i < c.len()
                && (c[*i].is_ascii_digit()
                    || c[*i] == '.'
                    || (matches!(c[*i], 'e' | 'E')
                        && c.get(*i + 1)
                            .is_some_and(|d| d.is_ascii_digit() || *d == '-')))
            {
                if matches!(c[*i], 'e' | 'E') {
                    *i += 1;
                }
                *i += 1;
            }
            let n: String = c[start..*i].iter().collect();
            let us = *i;
            while *i < c.len() && (c[*i].is_ascii_alphabetic() || c[*i] == '%') {
                *i += 1;
            }
            let unit: String = c[us..*i].iter().collect();
            args.last_mut().unwrap().push(Tok::Num(n, unit));
        } else if ch.is_ascii_alphabetic() || matches!(ch, '-' | '_' | '$') {
            let start = *i;
            while *i < c.len()
                && (c[*i].is_ascii_alphanumeric() || matches!(c[*i], '-' | '_' | '.' | '$'))
            {
                *i += 1;
            }
            let id: String = c[start..*i].iter().collect();
            if c.get(*i) == Some(&'(') {
                *i += 1;
                let (inner, closed) = tokens_in(c, i, true)?;
                if !closed {
                    return Err(format!("{id}( is not closed"));
                }
                args.last_mut().unwrap().push(Tok::Func(id, inner));
            } else {
                args.last_mut().unwrap().push(Tok::Ident(id));
            }
        } else {
            return Err(format!("'{ch}' is not part of a value"));
        }
    }
    Ok((args, false))
}

pub(crate) fn want_len(t: &Tok) -> Result<Value, String> {
    match t {
        Tok::Num(n, u)
            if u == "px"
                || (u.is_empty()
                    && n.trim_start_matches('-')
                        .trim_matches(['0', '.'])
                        .is_empty()) =>
        {
            number(n)
        }
        Tok::Num(n, u) if u.is_empty() => Err(format!("'{n}' needs its unit: {n}px")),
        Tok::Func(f, a) if f == "bind" => Err(format!(
            "bind({}) binds a whole declaration",
            a.iter()
                .flatten()
                .map(describe)
                .collect::<Vec<_>>()
                .join(" ")
        )),
        t => Err(format!("'{}' is not a length (px)", describe(t))),
    }
}

pub(crate) fn want_num(t: &Tok) -> Result<Value, String> {
    match t {
        Tok::Num(n, u) if u.is_empty() => number(n),
        t => Err(format!("'{}' is not a number", describe(t))),
    }
}

fn want_angle(t: &Tok) -> Result<Value, String> {
    match t {
        Tok::Num(n, u) if u == "rad" => number(n),
        Tok::Num(n, u)
            if u.is_empty()
                && n.trim_start_matches('-')
                    .trim_matches(['0', '.'])
                    .is_empty() =>
        {
            number(n)
        }
        Tok::Num(n, u) if u == "deg" || u == "turn" => {
            let v: f64 = n.parse().map_err(|_| format!("'{n}' is not a number"))?;
            let r = if u == "deg" {
                v.to_radians()
            } else {
                v * std::f64::consts::TAU
            };
            Ok(Value::Number(
                Number::from_f32(r as f32).map_err(|e| e.reason().to_string())?,
            ))
        }
        t => Err(format!(
            "'{}' is not an angle (rad, deg or turn)",
            describe(t)
        )),
    }
}

pub(crate) fn want_ms(t: &Tok) -> Result<Value, String> {
    match t {
        Tok::Num(n, u) if u == "ms" => number(n),
        Tok::Num(n, u) if u == "s" => number(&shift(n, 3)),
        t => Err(format!("'{}' is not a duration (ms or s)", describe(t))),
    }
}

fn want_ident<'a>(t: &'a Tok, what: &str) -> Result<&'a str, String> {
    match t {
        Tok::Ident(s) => Ok(s),
        t => Err(format!("'{}' is not {what}", describe(t))),
    }
}

/// `var(--<ns>-<name>)`'s name (camel case).
fn want_var(t: &Tok, ns: &str) -> Result<String, String> {
    match t {
        Tok::Func(f, a) if f == "var" => match a.as_slice() {
            [v] => match v.as_slice() {
                [Tok::Ident(n)] => {
                    let n = n.strip_prefix("--").ok_or("var(--name)")?;
                    match ns {
                        "" => Ok(camel(n)),
                        _ => n
                            .strip_prefix(ns)
                            .and_then(|r| r.strip_prefix('-'))
                            .map(camel)
                            .ok_or_else(|| format!("var(--{n}) is not a --{ns}- token")),
                    }
                }
                _ => Err("var(--name)".into()),
            },
            _ => Err("var(--name) takes no fallback".into()),
        },
        t => Err(format!(
            "'{}' is not a token: var(--{ns}{}…)",
            describe(t),
            if ns.is_empty() { "" } else { "-" }
        )),
    }
}

fn paint_vals(t: &[Tok]) -> Result<Vec<Value>, String> {
    match t {
        [Tok::Hash(h)] => Ok(vec![Value::String(h.clone())]),
        [v @ Tok::Func(f, _)] if f == "var" => Ok(vec![
            Value::Word("token".into()),
            Value::Word(want_var(v, "")?),
        ]),
        _ => Err("a colour: var(--role) or #rrggbb[aa]".into()),
    }
}

fn transition_vals(t: &[Tok]) -> Result<Vec<Value>, String> {
    let t = match t.first() {
        Some(Tok::Ident(a)) if a == "all" => &t[1..],
        _ => t,
    };
    match t {
        [Tok::Ident(n)] if n == "none" => Ok(vec![Value::Word("instant".into())]),
        [v @ Tok::Func(f, _)] if f == "var" => Ok(vec![
            Value::Word("token".into()),
            Value::Word(want_var(v, "motion")?),
        ]),
        [Tok::Func(f, a)] if f == "-og-spring" => {
            let mut out = vec![Value::Word("spring".into())];
            match a.as_slice() {
                [d] => out.push(want_ms(d.first().ok_or("-og-spring(<ms>ms, <bounce>)")?)?),
                [d, b] => {
                    out.push(want_ms(d.first().ok_or("-og-spring(<ms>ms, <bounce>)")?)?);
                    out.push(want_num(b.first().ok_or("-og-spring(<ms>ms, <bounce>)")?)?);
                }
                _ => return Err("-og-spring(<ms>ms, <bounce>)".into()),
            }
            Ok(out)
        }
        [d, rest @ ..] if matches!(d, Tok::Num(..)) => {
            let mut out = vec![Value::Word("tween".into()), want_ms(d)?];
            match rest {
                [] => {}
                [Tok::Ident(e)] => out.push(Value::Word(
                    match e.as_str() {
                        "linear" => "linear",
                        "ease-out" => "out",
                        "ease-in" => "exit",
                        "ease-in-out" => "inOut",
                        _ => {
                            return Err(format!(
                                "'{e}' is not an ease: linear, ease-out, ease-in, ease-in-out"
                            ));
                        }
                    }
                    .into(),
                )),
                _ => return Err("<ms>ms [ease]".into()),
            }
            Ok(out)
        }
        _ => Err(
            "a transition: none, <ms>ms [ease], -og-spring(<ms>ms, <bounce>) or var(--motion-<name>)"
                .into(),
        ),
    }
}

fn track_vals(t: &Tok) -> Result<Value, String> {
    match t {
        Tok::Num(n, u) if u == "px" => number(n),
        Tok::Num(n, u) if u == "fr" || u == "%" => Ok(Value::String(format!("{n}{u}"))),
        Tok::Ident(a) if a == "auto" => Ok(Value::String("auto".into())),
        Tok::Func(f, a) if f == "minmax" => match a.as_slice() {
            [x, y] => {
                let len = |v: &[Tok]| match v {
                    [t] => want_len(t).map(|v| text_of(&v)),
                    _ => Err("minmax(<min>px, <max>px)".to_string()),
                };
                Ok(Value::String(format!("minmax:{}:{}", len(x)?, len(y)?)))
            }
            _ => Err("minmax(<min>px, <max>px)".into()),
        },
        Tok::Func(f, _) if f == "repeat" => Err("write each track: `1fr 1fr 1fr`".into()),
        t => Err(format!(
            "'{}' is not a track: <n>px, <n>fr, auto, <n>%, minmax(<min>px, <max>px)",
            describe(t)
        )),
    }
}

fn one(t: &[Tok], name: &str) -> Result<Tok, String> {
    match t {
        [v] => Ok(v.clone()),
        _ => Err(format!("{name} takes one value")),
    }
}

/// A grid line: `<start>`, `<start> / span <n>` or `span <n>` (0: auto).
fn grid_line(t: &[Tok]) -> Result<(Option<Value>, Option<Value>), String> {
    let span = |t: &[Tok]| match t {
        [Tok::Ident(s), n] if s == "span" => want_num(n).map(Some),
        _ => Err("span <n>".to_string()),
    };
    match t {
        [n @ Tok::Num(..)] => Ok((Some(want_num(n)?), None)),
        [n @ Tok::Num(..), Tok::Slash, rest @ ..] => Ok((Some(want_num(n)?), span(rest)?)),
        [Tok::Ident(s), _] if s == "span" => Ok((None, span(t)?)),
        _ => Err("a grid line: <n>, <n> / span <n> or span <n>".into()),
    }
}

/// The entries one element declaration (or state value when `state`)
/// sets, in order. Sizes, grid lines and `grid-auto-flow` come back as
/// parts the block joins ([`Joined`]).
fn lower_decl(name: &str, value: &str, state: bool) -> Result<Vec<Part>, String> {
    let t = tokens(value)?;
    let e = |key: &str, values: Vec<Value>| Part::Entry(key.to_string(), values);
    if t.is_empty() {
        return Err(format!("{name} has no value"));
    }
    // A bound declaration: the whole value is the binding.
    if let [Tok::Func(f, a)] = t.as_slice()
        && f == "bind"
        && !matches!(name, "translate" | "scale" | "-og-skew")
    {
        let src = match a.as_slice() {
            [v] => match v.as_slice() {
                [Tok::Ident(p)] => Value::Word(p.clone()),
                [Tok::Str(s)] => Value::String(s.clone()),
                _ => return Err("bind(path) or bind(\"expression\")".into()),
            },
            _ => return Err("bind(path) or bind(\"expression\")".into()),
        };
        let prop = if state {
            return Err("a state's values are literals".into());
        } else {
            match name {
                "grid-column" | "grid-row" | "grid-auto-flow" => {
                    return Err(format!("{name} binds each part: write it as values"));
                }
                n => STYLE
                    .iter()
                    .find(|(_, c)| *c == n)
                    .map(|(p, _)| *p)
                    .ok_or_else(|| format!("'{n}' is not a property"))?,
            }
        };
        return Ok(vec![e(prop, vec![Value::Word("bind".into()), src])]);
    }
    // A part of a joint property: its value, or `bind(…)`.
    let part = |k: &str, t: &Tok| -> Result<Vec<Value>, String> {
        if let Tok::Func(f, a) = t
            && f == "bind"
        {
            return match a.as_slice() {
                [v] => match v.as_slice() {
                    [Tok::Ident(p)] => Ok(vec![Value::Word("bind".into()), Value::Word(p.clone())]),
                    [Tok::Str(s)] => Ok(vec![Value::Word("bind".into()), Value::String(s.clone())]),
                    _ => Err("bind(path) or bind(\"expression\")".into()),
                },
                _ => Err("bind(path) or bind(\"expression\")".into()),
            };
        }
        if k.starts_with("skew") {
            want_angle(t).map(|v| vec![v])
        } else if k.starts_with("scale") {
            want_num(t).map(|v| vec![v])
        } else {
            want_len(t).map(|v| vec![v])
        }
    };
    match name {
        "translate" | "-og-skew" => {
            let (a, b) = if name == "translate" {
                ("x", "y")
            } else {
                ("skewX", "skewY")
            };
            let (x, y) = match t.as_slice() {
                [x] => (x.clone(), Tok::Num("0".into(), String::new())),
                [x, y] => (x.clone(), y.clone()),
                _ => return Err(format!("{name}: <x> <y>")),
            };
            Ok(vec![e(a, part(a, &x)?), e(b, part(b, &y)?)])
        }
        "scale" => match t.as_slice() {
            [s] => Ok(vec![e("scale", part("scale", s)?)]),
            [x, y] => Ok(vec![
                e("scaleX", part("scaleX", x)?),
                e("scaleY", part("scaleY", y)?),
            ]),
            _ => Err("scale: <both> or <x> <y>".into()),
        },
        "rotate" => Ok(vec![e("rotate", vec![want_angle(&one(&t, name)?)?])]),
        "opacity" => Ok(vec![e("opacity", vec![want_num(&one(&t, name)?)?])]),
        "z-index" => Ok(vec![e("z", vec![want_num(&one(&t, name)?)?])]),
        "border-radius" => Ok(vec![e("radius", vec![want_len(&one(&t, name)?)?])]),
        "background" => Ok(vec![e("background", paint_vals(&t)?)]),
        "color" => Ok(vec![e(
            if state { "ink" } else { "color" },
            paint_vals(&t)?,
        )]),
        _ if state => Err(format!(
            "'{name}' is not a state's: {}",
            [
                "translate",
                "scale",
                "rotate",
                "-og-skew",
                "opacity",
                "border-radius",
                "z-index",
                "background",
                "color"
            ]
            .join(", ")
        )),
        "width" | "height" | "min-width" | "max-width" | "min-height" | "max-height" => {
            let axis = if name.ends_with("width") { 0 } else { 1 };
            let v = one(&t, name)?;
            if let Some(bound) = name.strip_suffix("-width").or(name.strip_suffix("-height")) {
                return Ok(vec![Part::Size(
                    axis,
                    SizeBit::Range(bound == "min", want_len(&v)?),
                )]);
            }
            Ok(vec![Part::Size(
                axis,
                SizeBit::Kind(match &v {
                    Tok::Ident(k) if k == "fit-content" => vec![Value::Word("fit".into())],
                    Tok::Ident(k) if k == "stretch" => vec![Value::Word("grow".into())],
                    Tok::Ident(k) if k == "auto" => {
                        return Err(format!(
                            "{name}: fit-content (its content), stretch (the room left), <n>px or <n>%"
                        ));
                    }
                    Tok::Num(n, u) if u == "%" => {
                        vec![Value::Word("percent".into()), number(&shift(n, -2))?]
                    }
                    v => vec![want_len(v)?],
                }),
            )])
        }
        "padding" => {
            let n: Vec<Value> = t.iter().map(want_len).collect::<Result<_, _>>()?;
            let vals = match n.as_slice() {
                [a] => vec![a.clone()],
                [v, h] => vec![h.clone(), v.clone()],
                [top, h, b] => vec![h.clone(), top.clone(), h.clone(), b.clone()],
                [top, r, b, l] => vec![l.clone(), top.clone(), r.clone(), b.clone()],
                _ => return Err("padding takes 1 to 4 lengths".into()),
            };
            Ok(vec![e("padding", vals)])
        }
        "grid-column" | "grid-row" => {
            let (start, span) = grid_line(&t)?;
            let axis = if name == "grid-column" { 0 } else { 1 };
            Ok(vec![Part::Grid(axis, start, span)])
        }
        "grid-auto-flow" => {
            let mut out = Vec::new();
            for v in &t {
                match want_ident(v, "row, column or dense")? {
                    "dense" => out.push(e("dense", vec![Value::Word("true".into())])),
                    f @ ("row" | "column") => out.push(e("flow", vec![Value::Word(f.into())])),
                    f => return Err(format!("'{f}' is not a grid flow: row, column, dense")),
                }
            }
            Ok(out)
        }
        _ => {
            let Some(prop) = STYLE.iter().find(|(_, c)| *c == name).map(|(p, _)| *p) else {
                return Err(match unsupported(name) {
                    Some(how) => format!("'{name}' is not supported: {how}"),
                    None => format!("'{name}' is not a property{}", hint_for(name)),
                });
            };
            let vals = match conv(prop) {
                Conv::Len => vec![want_len(&one(&t, name)?)?],
                Conv::Num => vec![want_num(&one(&t, name)?)?],
                Conv::Angle => vec![want_angle(&one(&t, name)?)?],
                Conv::Paint => paint_vals(&t)?,
                Conv::Bool(yes, no) => {
                    match want_ident(&one(&t, name)?, "a word")?.to_string().as_str() {
                        w if w == yes => vec![Value::Word("true".into())],
                        w if w == no => vec![Value::Word("false".into())],
                        w => return Err(format!("{name} is {yes} or {no}, not '{w}'")),
                    }
                }
                Conv::Kebab => match one(&t, name)? {
                    Tok::Func(f, a) if prop == "frame" && f == "rive" => {
                        // One argument: the artboard's id, relative to
                        // assets/ (`ui/chest.riv#Chest`, docs/ui.md §11.2).
                        let id = match a.as_slice() {
                            [v] => match v.as_slice() {
                                [Tok::Str(s)] => Some(s.as_str()),
                                _ => None,
                            },
                            _ => None,
                        };
                        let well_formed =
                            id.and_then(|s| s.split_once('#'))
                                .is_some_and(|(file, board)| {
                                    file.ends_with(".riv")
                                        && !board.is_empty()
                                        && !board.contains('#')
                                });
                        match id {
                            Some(id) if well_formed => {
                                return Ok(vec![e("rive", vec![Value::String(id.to_owned())])]);
                            }
                            _ => return Err("-og-frame: rive(\"<file>.riv#<Artboard>\") takes one quoted id relative to assets/, such as rive(\"ui/chest.riv#Chest\")".into()),
                        }
                    }
                    v => vec![Value::Word(camel(want_ident(&v, "a word")?))],
                },
                Conv::Kw(table) => {
                    let v = one(&t, name)?;
                    let w = want_ident(&v, "a word")?;
                    let el = table
                        .iter()
                        .find(|(_, c)| *c == w)
                        .map(|(e, _)| *e)
                        .ok_or_else(|| {
                            format!(
                                "'{w}' is not a {name}: {}",
                                table.iter().map(|(_, c)| *c).collect::<Vec<_>>().join(", ")
                            )
                        })?;
                    vec![Value::Word(el.into())]
                }
                Conv::Var(ns) => vec![Value::Word(want_var(&one(&t, name)?, ns)?)],
                Conv::Str => match one(&t, name)? {
                    Tok::Str(s) => vec![Value::String(s)],
                    v => {
                        return Err(format!(
                            "{name} takes a quoted path, not '{}'",
                            describe(&v)
                        ));
                    }
                },
                Conv::Ident => match one(&t, name)? {
                    Tok::Str(s) => vec![Value::String(s)],
                    Tok::Ident(s) => vec![Value::Word(s)],
                    v => return Err(format!("{name} takes a name, not '{}'", describe(&v))),
                },
                Conv::Strs => t
                    .iter()
                    .map(|v| match v {
                        Tok::Str(s) => Ok(Value::String(s.clone())),
                        v => Err(format!(
                            "{name} takes quoted strings, not '{}'",
                            describe(v)
                        )),
                    })
                    .collect::<Result<_, _>>()?,
                Conv::Tracks => t.iter().map(track_vals).collect::<Result<_, _>>()?,
                Conv::Transition => transition_vals(&t)?,
                Conv::Presence => {
                    let kind = want_ident(
                        t.first().ok_or("a presence")?,
                        "a presence: fade, slide-up …",
                    )?;
                    let mut out = vec![Value::Word(camel(kind))];
                    let mut rest = &t[1..];
                    if let Some(d @ Tok::Num(..)) = rest.first() {
                        out.push(want_len(d)?);
                        rest = &rest[1..];
                    }
                    if !rest.is_empty() {
                        out.extend(transition_vals(rest)?);
                    }
                    out
                }
                Conv::Origin => match t.as_slice() {
                    [Tok::Num(a, u), Tok::Num(b, v)] if u == "%" && v == "%" => {
                        vec![number(&shift(a, -2))?, number(&shift(b, -2))?]
                    }
                    _ => return Err("transform-origin: <x>% <y>%".into()),
                },
                Conv::Size | Conv::Insets | Conv::Joint | Conv::Rive => {
                    unreachable!("handled above")
                }
            };
            Ok(vec![e(prop, vals)])
        }
    }
}

/// What a property name was meant to be, when it is a near miss.
fn hint_for(name: &str) -> String {
    let c = camel(name);
    if let Some(css) = css_name(&c).or(css_name(name)) {
        return format!(" (write `{css}`)");
    }
    if crate::doc::schema::prop(&c).is_some() || crate::doc::schema::prop(name).is_some() {
        return format!(" ({c} is an attribute: `{c} <value>`, without a colon)");
    }
    let near: Vec<&str> = STYLE
        .iter()
        .map(|(_, c)| *c)
        .filter(|c| c.contains(name) || name.contains(c.trim_start_matches("-og-")))
        .collect();
    if near.is_empty() {
        " (the properties: docs/ui.md §10.4)".into()
    } else {
        format!(" (did you mean {}?)", near.join(", "))
    }
}

enum SizeBit {
    Kind(Vec<Value>),
    /// `min-` (true) or `max-`, a length.
    Range(bool, Value),
}

enum Part {
    Entry(String, Vec<Value>),
    Size(usize, SizeBit),
    /// An axis, its start, its span.
    Grid(usize, Option<Value>, Option<Value>),
}

// ---- reading a version 2 text

#[derive(Clone, Copy, PartialEq, Debug)]
enum Ctx {
    Doc,
    /// An element, `use`, `for`, `if` or `@media`: declarations are props.
    Tree,
    /// A named state group: its states and `transition:`.
    Group,
    /// A state of a group, or a pseudo-class: declarations are its values.
    State,
    /// Anything else (`props`, `actions`, a timeline): `.ogt` lines.
    Other,
}

/// A size being joined: its entry's index in the block, its kind, its
/// `min-` and its `max-`.
type SizeParts = (usize, Vec<Value>, Option<Value>, Option<Value>);

struct Frame {
    ctx: Ctx,
    entry: Entry,
    /// A pseudo-class's state name (`hover`), when this frame is one.
    pseudo: Option<String>,
    /// Sizes and grid lines, joined at the block's end.
    size: [Option<SizeParts>; 2],
    grid: [Option<usize>; 2],
    grid_parts: [[Option<Value>; 2]; 2],
    /// The element's pseudo-class states, its interaction group's place
    /// and transition.
    states: Vec<Entry>,
    states_at: Option<usize>,
    state_transition: Option<Entry>,
}

impl Frame {
    fn new(ctx: Ctx, entry: Entry) -> Frame {
        Frame {
            ctx,
            entry,
            pseudo: None,
            size: [None, None],
            grid: [None, None],
            grid_parts: [[None, None], [None, None]],
            states: Vec::new(),
            states_at: None,
            state_transition: None,
        }
    }

    fn block(&mut self) -> &mut Block {
        self.entry.block.get_or_insert_with(Block::new)
    }

    fn push(&mut self, e: Entry) {
        self.block().push(e);
    }

    fn len(&self) -> usize {
        self.entry.block.as_ref().map_or(0, |b| b.entries.len())
    }

    fn part(&mut self, p: Part, line: usize) -> Result<(), String> {
        match p {
            Part::Entry(k, values) => {
                let mut e = Entry::new(k);
                e.values = values;
                e.line = line;
                self.push(e);
            }
            Part::Size(axis, bit) => {
                if self.size[axis].is_none() {
                    let mut e = Entry::new(if axis == 0 { "width" } else { "height" });
                    e.line = line;
                    let at = self.len();
                    self.push(e);
                    self.size[axis] = Some((at, vec![Value::Word("fit".into())], None, None));
                }
                let s = self.size[axis].as_mut().unwrap();
                match bit {
                    SizeBit::Kind(k) => s.1 = k,
                    SizeBit::Range(true, v) => s.2 = Some(v),
                    SizeBit::Range(false, v) => s.3 = Some(v),
                }
            }
            Part::Grid(axis, start, span) => {
                for (i, v) in [start, span].into_iter().enumerate() {
                    let Some(v) = v else { continue };
                    if self.grid[i].is_none() {
                        let mut e = Entry::new(if i == 0 { "cell" } else { "span" });
                        e.line = line;
                        self.grid[i] = Some(self.len());
                        self.push(e);
                    }
                    self.grid_parts[i][axis] = Some(v);
                }
            }
        }
        Ok(())
    }

    /// Joins sizes and grid lines, and the pseudo-class states into the
    /// interaction group (its `normal` state from the element's own
    /// values).
    fn finish(&mut self) -> Result<(), String> {
        for axis in 0..2 {
            let Some((at, kind, min, max)) = self.size[axis].take() else {
                continue;
            };
            let mut v = kind;
            let ranged = matches!(v.first(), Some(Value::Word(w)) if w == "fit" || w == "grow");
            if (min.is_some() || max.is_some()) && !ranged {
                return Err(format!(
                    "line {}: min- and max- bound a fit-content or stretch size",
                    self.entry.block.as_ref().unwrap().entries[at].line
                ));
            }
            if ranged && (min.is_some() || max.is_some()) {
                v.push(min.unwrap_or(Value::Number(Number::parse("0").unwrap())));
                if let Some(m) = max {
                    v.push(m);
                }
            }
            self.entry.block.as_mut().unwrap().entries[at].values = v;
        }
        for i in 0..2 {
            let Some(at) = self.grid[i].take() else {
                continue;
            };
            let zero = || Value::Number(Number::parse("0").unwrap());
            let [a, b] = std::mem::take(&mut self.grid_parts[i]);
            self.entry.block.as_mut().unwrap().entries[at].values =
                vec![a.unwrap_or_else(zero), b.unwrap_or_else(zero)];
        }
        if self.states.is_empty() {
            if let Some(t) = &self.state_transition {
                return Err(format!(
                    "line {}: -og-state-transition moves between the element's :hover, :active, :focus-visible and :disabled states; it declares none",
                    t.line
                ));
            }
            return Ok(());
        }
        let line = self.states[0].line;
        // The values any state sets, in order: `normal` returns each to
        // the element's own, else to its neutral value.
        let mut keys: Vec<String> = Vec::new();
        for s in &self.states {
            for v in s.block.as_ref().map_or(&[][..], |b| &b.entries) {
                if STATE_VALUES.iter().any(|(k, _)| *k == v.key) && !keys.contains(&v.key) {
                    keys.push(v.key.clone());
                }
            }
        }
        let mut normal = Entry::new("normal");
        normal.line = line;
        let own = self.entry.block.as_ref().map_or(&[][..], |b| &b.entries);
        for k in keys {
            let el = if k == "ink" { "color" } else { k.as_str() };
            let mine = own
                .iter()
                .rev()
                .find(|e| e.key == el && e.block.is_none())
                .map(|e| e.values.clone())
                .filter(|v| !matches!(v.first(), Some(Value::Word(b)) if b == "bind"));
            let values = match (mine, neutral(&k)) {
                (Some(v), _) => v,
                (None, Some(n)) => vec![Value::Number(Number::parse(n).unwrap())],
                (None, None) => {
                    return Err(format!(
                        "line {line}: a state sets {} but the element declares none to return to: declare it on the element",
                        STATE_VALUES
                            .iter()
                            .find(|(s, _)| *s == k)
                            .map_or("it", |(_, c)| c)
                    ));
                }
            };
            let mut e = Entry::new(k);
            e.values = values;
            e.line = line;
            normal.block.get_or_insert_with(Block::new).push(e);
        }
        normal.block.get_or_insert_with(Block::new);
        let mut group = Entry::new("state");
        group.values = vec![Value::Word("interaction".into())];
        group.line = line;
        let b = group.block.get_or_insert_with(Block::new);
        b.push(normal);
        for s in std::mem::take(&mut self.states) {
            b.push(s);
        }
        if let Some(t) = self.state_transition.take() {
            b.push(t);
        }
        let at = self.states_at.take().unwrap_or(0);
        self.block().entries.insert(at, group);
        Ok(())
    }
}

/// A declaration line: `name: value[; name: value]*`, without its comment.
pub(crate) fn is_decl(t: &str) -> bool {
    let name: String = t
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    !name.is_empty()
        && name.starts_with(|c: char| c.is_ascii_lowercase() || c == '-')
        && t[name.len()..].trim_start().starts_with(':')
}

/// A line's declarations, split at `;` outside strings and parentheses, and
/// cut at a `#` comment (a `#` that starts no colour).
pub(crate) fn split_decls(t: &str) -> Result<Vec<(String, String)>, String> {
    let c: Vec<char> = t.chars().collect();
    let mut parts = Vec::new();
    let (mut cur, mut depth, mut i, mut in_str) = (String::new(), 0, 0, false);
    while i < c.len() {
        let ch = c[i];
        if in_str {
            cur.push(ch);
            if ch == '\\' && i + 1 < c.len() {
                cur.push(c[i + 1]);
                i += 1;
            } else if ch == '"' {
                in_str = false;
            }
        } else if ch == '"' {
            in_str = true;
            cur.push(ch);
        } else if ch == '#' && !c.get(i + 1).is_some_and(|d| d.is_ascii_hexdigit()) {
            break;
        } else if ch == '(' {
            depth += 1;
            cur.push(ch);
        } else if ch == ')' {
            depth -= 1;
            cur.push(ch);
        } else if ch == ';' && depth == 0 {
            parts.push(std::mem::take(&mut cur));
        } else {
            cur.push(ch);
        }
        i += 1;
    }
    parts.push(cur);
    let mut out = Vec::new();
    for p in parts {
        let p = p.trim();
        if p.is_empty() {
            continue;
        }
        let (name, value) = p
            .split_once(':')
            .ok_or_else(|| format!("'{p}' is not a declaration (name: value)"))?;
        let value = value.trim();
        if value.ends_with("!important") {
            return Err("there is no !important: an element's later declaration wins".into());
        }
        out.push((name.trim().to_string(), value.to_string()));
    }
    Ok(out)
}

/// `@media phone, tablet and (orientation: portrait) {`'s `at` words.
fn media_words(q: &str) -> Result<Vec<Value>, String> {
    let mut words = Vec::new();
    let mut rest = q.to_string();
    if let Some(i) = rest.find("(orientation:") {
        let end = rest[i..]
            .find(')')
            .ok_or("(orientation: portrait|landscape)")?
            + i;
        let o = rest[i + "(orientation:".len()..end].trim().to_string();
        if o != "portrait" && o != "landscape" {
            return Err(format!("'{o}' is not an orientation: portrait, landscape"));
        }
        rest.replace_range(i..=end, "");
        rest = rest.trim().trim_end_matches("and").trim().to_string();
        let mut bps = media_names(&rest)?;
        bps.push(Value::Word(o));
        words = bps;
    } else {
        words.extend(media_names(&rest)?);
    }
    if words.is_empty() {
        return Err("@media <breakpoint>[, <breakpoint>] [and (orientation: …)] {".into());
    }
    Ok(words)
}

fn media_names(s: &str) -> Result<Vec<Value>, String> {
    s.split(',')
        .map(str::trim)
        .filter(|w| !w.is_empty())
        .map(|w| match w {
            "phone" | "tablet" | "desktop" | "phone-portrait" | "phone-landscape" => {
                Ok(Value::Word(camel(w)))
            }
            _ if w.contains("max-width") || w.contains("min-width") => Err(
                "breakpoints are named (the style sets their widths): phone, phone-portrait, phone-landscape, tablet, desktop".into(),
            ),
            _ => Err(format!(
                "'{w}' is not a breakpoint: phone, phone-portrait, phone-landscape, tablet, desktop, (orientation: portrait|landscape)"
            )),
        })
        .collect()
}

/// The context a block opened in `parent` by `key` reads its lines in.
fn child_ctx(parent: Ctx, e: &Entry) -> Ctx {
    match parent {
        Ctx::Doc | Ctx::Tree => match e.key.as_str() {
            "state" if parent == Ctx::Tree => Ctx::Group,
            "for" | "if" | "use" => Ctx::Tree,
            k if crate::doc::schema::kind(k).is_some() => Ctx::Tree,
            _ => Ctx::Other,
        },
        Ctx::Group => Ctx::State,
        _ => Ctx::Other,
    }
}

/// Reads a version 2 document into the `.ogt` entries of the builder, each
/// on its source line. Errors are `line N: …`.
pub(crate) fn lower(text: &str) -> Result<Document, Vec<String>> {
    let mut lines = text
        .split('\n')
        .enumerate()
        .map(|(i, l)| (i + 1, l.strip_suffix('\r').unwrap_or(l)));
    let mut head = String::new();
    for (_, l) in lines.by_ref() {
        head.push_str(l);
        head.push('\n');
        if l.trim() == "---" {
            break;
        }
    }
    let header = Document::parse(&head).map_err(|e| vec![e.reason().to_string()])?;
    let mut errors = Vec::new();
    let mut stack = vec![Frame::new(Ctx::Doc, Entry::new(""))];
    for (n, raw) in lines {
        let t = raw.trim();
        let err = |errors: &mut Vec<String>, why: String| errors.push(format!("line {n}: {why}"));
        let ctx = stack.last().unwrap().ctx;
        // A pseudo-class's or a media query's line may end in a comment.
        let code = comment_of(t).0;
        if let Some(p) = code.strip_prefix(':') {
            let Some(name) = p.strip_suffix('{').map(str::trim) else {
                err(
                    &mut errors,
                    "a pseudo-class opens its block: `:hover {`".into(),
                );
                continue;
            };
            let state = PSEUDO.iter().find(|(_, c)| *c == name).map(|(s, _)| *s);
            let el = stack.last().is_some_and(|f| {
                f.ctx == Ctx::Tree
                    && f.entry.key != "at"
                    && crate::doc::schema::kind(&f.entry.key).is_some()
            });
            match state {
                Some(s) if el => {
                    let mut e = Entry::new(s);
                    e.line = n;
                    e.block = Some(Block::new());
                    let mut f = Frame::new(Ctx::State, e);
                    f.pseudo = Some(s.to_string());
                    stack.push(f);
                }
                Some(_) => {
                    err(
                        &mut errors,
                        format!(":{name} is an element's (not in @media, for, if or use)"),
                    );
                    let mut f = Frame::new(Ctx::Other, Entry::new("_"));
                    f.entry.line = n;
                    stack.push(f);
                }
                None => {
                    err(
                        &mut errors,
                        format!(
                            "':{name}' is not a state og-ui tracks: :hover, :active, :focus-visible, :disabled; a bound state is `state <group> bind <expr> {{ … }}`"
                        ),
                    );
                    stack.push(Frame::new(Ctx::Other, Entry::new("_")));
                }
            }
            continue;
        }
        if let Some(q) = code.strip_prefix("@media") {
            let Some(q) = q.trim().strip_suffix('{') else {
                err(&mut errors, "@media <breakpoint> {".into());
                continue;
            };
            let mut e = Entry::new("at");
            e.line = n;
            e.block = Some(Block::new());
            match media_words(q.trim()) {
                Ok(w) => e.values = w,
                Err(why) => err(&mut errors, why),
            }
            if ctx != Ctx::Tree {
                err(&mut errors, "@media is an element's".into());
            }
            stack.push(Frame::new(Ctx::Tree, e));
            continue;
        }
        if t.starts_with('@') {
            err(
                &mut errors,
                format!(
                    "'{}' is not supported: @media is the one at-rule",
                    t.split_whitespace().next().unwrap_or(t)
                ),
            );
            continue;
        }
        if is_decl(t) {
            let decls = match split_decls(t) {
                Ok(d) => d,
                Err(why) => {
                    err(&mut errors, why);
                    continue;
                }
            };
            let f = stack.last_mut().unwrap();
            for (name, value) in decls {
                let r = match f.ctx {
                    Ctx::Tree if name == "-og-state-transition" => transition_vals(&tokens(&value).unwrap_or_default()).map(|v| {
                        let mut e = Entry::new("transition");
                        e.values = vec![Value::Word("any".into()), Value::Word("any".into())];
                        e.values.extend(v);
                        e.line = n;
                        f.state_transition = Some(e);
                    }),
                    Ctx::Tree => lower_decl(&name, &value, false).and_then(|parts| {
                        parts.into_iter().try_for_each(|p| f.part(p, n))
                    }),
                    Ctx::State => lower_decl(&name, &value, true).and_then(|parts| {
                        parts.into_iter().try_for_each(|p| f.part(p, n))
                    }),
                    Ctx::Group if name == "transition" => tokens(&value).and_then(|t| {
                        let (ends, rest) = match t.as_slice() {
                            [Tok::Ident(a), Tok::Ident(b), rest @ ..]
                                if !rest.is_empty() && !a.starts_with('-') && a != "all" && a != "none" =>
                            {
                                (vec![Value::Word(a.clone()), Value::Word(b.clone())], rest.to_vec())
                            }
                            _ => (vec![Value::Word("any".into()), Value::Word("any".into())], t.clone()),
                        };
                        let mut e = Entry::new("transition");
                        e.values = ends;
                        e.values.extend(transition_vals(&rest)?);
                        e.line = n;
                        f.push(e);
                        Ok(())
                    }),
                    Ctx::Group => Err("a state group holds its states and `transition: [<from> <to>] <transition>`".into()),
                    _ => Err(format!("'{name}: …' is a declaration; declarations go in an element or a state")),
                };
                if let Err(why) = r {
                    err(&mut errors, why);
                }
            }
            continue;
        }
        match ogt::parse_line(raw, n) {
            Err(e) => err(&mut errors, e.reason().to_string()),
            Ok(Line::Blank) => {}
            Ok(Line::Close) => {
                if stack.len() == 1 {
                    err(&mut errors, "unmatched '}'".into());
                    continue;
                }
                let mut f = stack.pop().unwrap();
                if let Err(why) = f.finish() {
                    errors.push(why);
                }
                let parent = stack.last_mut().unwrap();
                if f.pseudo.is_some() {
                    if parent.states_at.is_none() {
                        parent.states_at = Some(parent.len());
                    }
                    parent.states.push(f.entry);
                } else if f.entry.key != "_" {
                    parent.push(f.entry);
                }
            }
            Ok(Line::Entry { entry, opens }) => {
                if matches!(ctx, Ctx::Tree | Ctx::State) && entry.block.is_none() {
                    let style = if ctx == Ctx::Tree {
                        css_name(&entry.key).is_some()
                    } else {
                        STATE_VALUES.iter().any(|(k, _)| *k == entry.key)
                    };
                    if style {
                        let how = decl_line(
                            &[(entry.key.clone(), entry.values.clone())],
                            ctx == Ctx::State,
                            &|_| None,
                        )
                        .map(|d| format!(": write `{d}`"))
                        .unwrap_or_default();
                        err(
                            &mut errors,
                            format!("'{}' is a style property{how}", entry.key),
                        );
                        continue;
                    }
                }
                if ctx == Ctx::Tree && opens && entry.key == "at" {
                    err(&mut errors, "a breakpoint is `@media phone {`".into());
                }
                if ctx == Ctx::Tree
                    && opens
                    && entry.key == "state"
                    && matches!(entry.values.first(), Some(Value::Word(w)) if w == "interaction")
                {
                    err(&mut errors, "the interaction states are pseudo-classes: `:hover {`, `:active {`, `:focus-visible {`, `:disabled {`".into());
                }
                if opens {
                    let c = child_ctx(ctx, &entry);
                    stack.push(Frame::new(c, entry));
                } else {
                    stack.last_mut().unwrap().push(entry);
                }
            }
        }
    }
    if stack.len() > 1 {
        let f = stack.last().unwrap();
        errors.push(format!(
            "line {}: block '{}' opened here is not closed",
            f.entry.line, f.entry.key
        ));
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut root = stack.pop().unwrap();
    Ok(Document {
        ty: header.ty,
        version: header.version,
        body: root.entry.block.take().unwrap_or_default(),
    })
}

/// Gives each lone half of a jointly written pair (`y` without `x`) its
/// partner as [`migrate`] writes it: the element's own value, else the
/// neutral one. A state or `at` block reads the element it is in; what
/// it adds changes nothing drawn (a state that sets `x` to the element's
/// own `x` leaves it where it is).
pub(crate) fn partner_neutrals(block: &mut Block) {
    fn walk(block: &mut Block, own: Option<&[Entry]>) {
        let pairs = [("x", "y"), ("scaleX", "scaleY"), ("skewX", "skewY")];
        let mine: Vec<Entry> = own.map_or_else(|| block.entries.clone(), <[Entry]>::to_vec);
        for (a, b) in pairs {
            let has =
                |k: &str, bl: &Block| bl.entries.iter().any(|e| e.key == k && e.block.is_none());
            for (lone, other, first) in [(b, a, true), (a, b, false)] {
                if has(lone, block) && !has(other, block) {
                    let i = block
                        .entries
                        .iter()
                        .position(|e| e.key == lone && e.block.is_none())
                        .unwrap();
                    let values = mine
                        .iter()
                        .rev()
                        .find(|e| e.key == other && e.block.is_none())
                        .map(|e| e.values.clone())
                        .unwrap_or_else(|| {
                            vec![Value::Number(
                                Number::parse(neutral(other).unwrap()).unwrap(),
                            )]
                        });
                    let mut e = Entry::new(other);
                    e.values = values;
                    e.line = block.entries[i].line;
                    block.entries.insert(if first { i } else { i + 1 }, e);
                }
            }
        }
        let snapshot = block.entries.clone();
        for e in &mut block.entries {
            let Some(b) = &mut e.block else { continue };
            match e.key.as_str() {
                "at" => walk(b, Some(own.unwrap_or(&snapshot))),
                "state" => {
                    for s in &mut b.entries {
                        if let Some(sb) = &mut s.block {
                            walk(sb, Some(own.unwrap_or(&snapshot)));
                        }
                    }
                }
                _ => walk(b, None),
            }
        }
    }
    walk(block, None);
}

// ---- writing a version 1 text as version 2

/// A rewritten line with the comment of the line `old` it replaces, at
/// the comment's column when the new code leaves room for it.
pub(crate) fn with_comment(code: String, comment: &str, old: &str) -> String {
    let col = old.chars().count() - comment.chars().count();
    let n = code.chars().count();
    if code.trim().is_empty() {
        format!("{code}{comment}")
    } else if n < col {
        format!("{code}{}{comment}", " ".repeat(col - n))
    } else {
        format!("{code} {comment}")
    }
}

/// The code of a `.ogt` line and its `#` comment (from a `#` that starts a
/// token outside a string), each trimmed.
pub(crate) fn comment_of(line: &str) -> (&str, Option<&str>) {
    let b = line.as_bytes();
    let mut in_str = false;
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\\' if in_str => i += 1,
            b'"' => in_str = !in_str,
            b'#' if !in_str && (i == 0 || b[i - 1] == b' ' || b[i - 1] == b'\t') => {
                return (line[..i].trim_end(), Some(&line[i..]));
            }
            _ => {}
        }
        i += 1;
    }
    (line.trim_end(), None)
}

enum Act {
    Replace(String),
    Drop,
}

struct Plan {
    acts: std::collections::BTreeMap<usize, Act>,
    /// Lines that move up one level (inside an interaction group that is dropped).
    dedent: std::collections::BTreeSet<usize>,
    /// Each block's open line to its close line.
    close: std::collections::BTreeMap<usize, usize>,
    errors: Vec<String>,
}

impl Plan {
    fn set(&mut self, line: usize, a: Act) {
        self.acts.insert(line, a);
    }

    fn drop_block(&mut self, e: &Entry) {
        let end = self.close.get(&e.line).copied().unwrap_or(e.line);
        for l in e.line..=end {
            self.acts.insert(l, Act::Drop);
        }
    }

    /// The declarations of a block's props (element props, or state
    /// values), on their lines: props that share a CSS property move to
    /// the line of the first.
    fn decls(&mut self, block: &Block, state: bool, own: &[Entry]) {
        let style = |k: &str| {
            if state {
                STATE_VALUES.iter().any(|(s, _)| *s == k)
            } else {
                css_name(k).is_some()
            }
        };
        let props: Vec<&Entry> = block
            .entries
            .iter()
            .filter(|e| e.block.is_none() && style(&e.key))
            .collect();
        let joint = |k: &str| -> &'static [&'static str] {
            match k {
                "x" | "y" => &["x", "y"],
                "scaleX" | "scaleY" => &["scaleX", "scaleY"],
                "skewX" | "skewY" => &["skewX", "skewY"],
                "flow" | "dense" => &["flow", "dense"],
                _ => &[],
            }
        };
        let mut placed: Vec<usize> = Vec::new();
        let partner = |k: &str| -> Option<Vec<Value>> {
            let el = if k == "ink" { "color" } else { k };
            own.iter()
                .rev()
                .find(|e| e.key == el && e.block.is_none())
                .map(|e| e.values.clone())
        };
        for (i, e) in props.iter().enumerate() {
            if placed.contains(&i) {
                continue;
            }
            let mut on_line = vec![(e.key.clone(), e.values.clone())];
            // Its partners: the last of each, later in the block.
            for k in joint(&e.key).iter().filter(|k| **k != e.key) {
                if let Some(j) = props.iter().rposition(|p| p.key == *k)
                    && j > i
                {
                    on_line.push((props[j].key.clone(), props[j].values.clone()));
                    placed.push(j);
                    self.acts.insert(props[j].line, Act::Drop);
                }
            }
            match decl_line(&on_line, state, &partner) {
                Ok(d) => self.set(e.line, Act::Replace(d)),
                Err(why) => self.errors.push(format!("line {}: {why}", e.line)),
            }
        }
    }

    fn tree(&mut self, block: &Block, ctx: Ctx, own: &[Entry]) {
        match ctx {
            Ctx::Tree => self.decls(block, false, own),
            Ctx::State => self.decls(block, true, own),
            _ => {}
        }
        for e in &block.entries {
            let Some(b) = &e.block else { continue };
            match (ctx, e.key.as_str()) {
                (Ctx::Tree, "at") => {
                    let mut bps = Vec::new();
                    let mut orient = None;
                    for v in &e.values {
                        match text_of(v).as_str() {
                            o @ ("portrait" | "landscape") => orient = Some(o.to_string()),
                            w => bps.push(kebab(w)),
                        }
                    }
                    let mut q = bps.join(", ");
                    if let Some(o) = orient {
                        q = if q.is_empty() {
                            format!("(orientation: {o})")
                        } else {
                            format!("{q} and (orientation: {o})")
                        };
                    }
                    self.set(e.line, Act::Replace(format!("@media {q} {{")));
                    self.tree(b, Ctx::Tree, &block.entries);
                }
                (Ctx::Tree, "state") if matches!(e.values.first(), Some(Value::Word(w)) if w == "interaction") =>
                {
                    self.interaction(e, b, &block.entries);
                }
                (Ctx::Tree, "state") => {
                    for s in &b.entries {
                        match (&s.block, s.key.as_str()) {
                            (None, "transition") => {
                                let w: Vec<String> = s.values.iter().map(text_of).collect();
                                match transition_css(s.values.get(2..).unwrap_or(&[])) {
                                    Ok(t) if w.len() > 2 && w[0] == "any" && w[1] == "any" => {
                                        self.set(s.line, Act::Replace(format!("transition: {t}")));
                                    }
                                    Ok(t) if w.len() > 2 => self.set(
                                        s.line,
                                        Act::Replace(format!("transition: {} {} {t}", w[0], w[1])),
                                    ),
                                    _ => self.errors.push(format!(
                                        "line {}: transition <from> <to> <transition>",
                                        s.line
                                    )),
                                }
                            }
                            (Some(sb), _) => self.tree(sb, Ctx::State, &block.entries),
                            _ => {}
                        }
                    }
                }
                (Ctx::Doc | Ctx::Tree, _) => {
                    let c = child_ctx(ctx, e);
                    // An element's own values are its block's; `for`, `if`
                    // and `use` blocks hold the elements of their parent.
                    self.tree(b, c, &b.entries);
                }
                _ => {}
            }
        }
    }

    fn interaction(&mut self, group: &Entry, b: &Block, own: &[Entry]) {
        self.set(group.line, Act::Drop);
        if let Some(&end) = self.close.get(&group.line) {
            self.set(end, Act::Drop);
            for l in group.line + 1..end {
                self.dedent.insert(l);
            }
        }
        for s in &b.entries {
            match (&s.block, s.key.as_str()) {
                (Some(sb), "normal") => {
                    for v in &sb.entries {
                        let el = if v.key == "ink" { "color" } else { v.key.as_str() };
                        let mine = own.iter().rev().find(|e| e.key == el && e.block.is_none());
                        let same = match mine {
                            Some(m) => m.values == v.values,
                            None => neutral(&v.key).is_some_and(|n| {
                                v.values.first().and_then(Value::as_number).is_some_and(|x| x.as_f64() == n.parse::<f64>().unwrap())
                            }),
                        };
                        if !same {
                            self.errors.push(format!(
                                "line {}: the normal state's {} is not the element's own: declare it on the element by hand",
                                v.line, v.key
                            ));
                        }
                    }
                    self.drop_block(s);
                }
                (Some(sb), name) => match PSEUDO.iter().find(|(k, _)| *k == name) {
                    Some((_, p)) => {
                        self.set(s.line, Act::Replace(format!(":{p} {{")));
                        self.tree(sb, Ctx::State, own);
                    }
                    None => self.errors.push(format!(
                        "line {}: '{name}' is not an interaction state (normal, hover, pressed, focused, disabled)",
                        s.line
                    )),
                },
                (None, "transition") => {
                    let w: Vec<String> = s.values.iter().map(text_of).collect();
                    match transition_css(s.values.get(2..).unwrap_or(&[])) {
                        Ok(t) if w.len() > 2 && w[0] == "any" && w[1] == "any" => {
                            self.set(s.line, Act::Replace(format!("-og-state-transition: {t}")));
                        }
                        _ => self.errors.push(format!(
                            "line {}: an interaction transition other than `any any` is migrated by hand",
                            s.line
                        )),
                    }
                }
                _ => {}
            }
        }
    }
}

/// Each block's open line to its close line, read by `.ogt`'s lines.
fn closes(text: &str) -> std::collections::BTreeMap<usize, usize> {
    let mut out = std::collections::BTreeMap::new();
    let mut open = Vec::new();
    let mut body = false;
    for (i, l) in text.split('\n').enumerate() {
        let n = i + 1;
        if !body {
            body = l.trim() == "---";
            continue;
        }
        match ogt::parse_line(l, n) {
            Ok(Line::Entry { opens: true, .. }) => open.push(n),
            Ok(Line::Close) => {
                if let Some(o) = open.pop() {
                    out.insert(o, n);
                }
            }
            _ => {}
        }
    }
    out
}

/// A version 1 document's text as version 2, line for line: its comments,
/// blank lines, attributes and structure as written, its style props as
/// declarations, its `at` blocks as `@media`, its interaction group as
/// pseudo-classes (the `normal` state is the element's own values).
pub fn migrate(text: &str) -> Result<String, Vec<String>> {
    let doc = Document::parse(text).map_err(|e| vec![e.reason().to_string()])?;
    if doc.version != 1 {
        return Err(vec![format!(
            "version {} is not a version 1 document",
            doc.version
        )]);
    }
    let mut plan = Plan {
        acts: Default::default(),
        dedent: Default::default(),
        close: closes(text),
        errors: Vec::new(),
    };
    plan.tree(&doc.body, Ctx::Doc, &[]);
    if !plan.errors.is_empty() {
        return Err(plan.errors);
    }
    let mut out = Vec::new();
    let mut header = true;
    for (i, l) in text.split('\n').enumerate() {
        let n = i + 1;
        if header {
            if l.trim() == "---" {
                header = false;
            }
            let t = l.trim();
            out.push(
                if t.starts_with("version") && t.split_whitespace().nth(1) == Some("1") {
                    l.replacen('1', "2", 1)
                } else {
                    l.to_string()
                },
            );
            continue;
        }
        let indent_len = l.len() - l.trim_start().len();
        let mut indent = &l[..indent_len];
        if plan.dedent.contains(&n) && indent.len() >= 2 {
            indent = &indent[..indent.len() - 2];
        }
        let (code, comment) = comment_of(l.trim_start());
        match plan.acts.get(&n) {
            Some(Act::Drop) => {
                if let Some(c) = comment
                    && !code.is_empty()
                {
                    out.push(format!("{indent}{c}"));
                }
            }
            Some(Act::Replace(s)) => out.push(match comment {
                Some(c) => with_comment(format!("{indent}{s}"), c, l),
                None => format!("{indent}{s}"),
            }),
            None => out.push(if code.is_empty() && comment.is_none() {
                String::new()
            } else {
                format!("{indent}{}", l.trim_start())
            }),
        }
    }
    Ok(out.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_shifts_are_exact() {
        assert_eq!(shift("0.25", 2), "25");
        assert_eq!(shift("0.5", 2), "50");
        assert_eq!(shift("1", 2), "100");
        assert_eq!(shift("33.3", -2), "0.333");
        assert_eq!(shift("-4", -2), "-0.04");
        assert_eq!(shift("0.3", 3), "300");
        assert_eq!(shift("0", 2), "0");
    }

    #[test]
    fn kebab_and_camel_are_inverse() {
        for w in [
            "spaceBetween",
            "inkMuted",
            "xl2",
            "displaySm",
            "team1",
            "slideUp",
        ] {
            assert_eq!(camel(&kebab(w)), w);
        }
    }
}
