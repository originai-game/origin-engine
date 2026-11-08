//! CSS declaration values shared by document stylesheets and style packs.
//! [`STYLE`] maps reflected properties to their CSS spellings; [`decl_line`]
//! writes probe edits using the same value conversions as the reader.

use og_core::ogt::{self, Block, Entry, Line, Number, Value};

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
    /// A margin edge: `<n>px`, `<n>em` (of its own type step) or `0`, as
    /// px and em.
    Margin,
    /// The CSS value as written, checked when it is read (`box-shadow`,
    /// `-og-text-path`'s shapes).
    Raw,
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
    ("marginTop", "margin-top"),
    ("marginRight", "margin-right"),
    ("marginBottom", "margin-bottom"),
    ("marginLeft", "margin-left"),
    ("gap", "gap"),
    ("align", "justify-content"),
    ("cross", "align-items"),
    ("wrap", "flex-wrap"),
    ("direction", "flex-direction"),
    ("labelDirection", "flex-direction"),
    ("background", "background"),
    ("radius", "border-radius"),
    ("borderTopWidth", "border-top-width"),
    ("borderTopStyle", "border-top-style"),
    ("borderTopColor", "border-top-color"),
    ("borderRightWidth", "border-right-width"),
    ("borderRightStyle", "border-right-style"),
    ("borderRightColor", "border-right-color"),
    ("borderBottomWidth", "border-bottom-width"),
    ("borderBottomStyle", "border-bottom-style"),
    ("borderBottomColor", "border-bottom-color"),
    ("borderLeftWidth", "border-left-width"),
    ("borderLeftStyle", "border-left-style"),
    ("borderLeftColor", "border-left-color"),
    ("outlineWidth", "outline-width"),
    ("outlineStyle", "outline-style"),
    ("outlineColor", "outline-color"),
    ("outlineOffset", "outline-offset"),
    ("boxShadow", "box-shadow"),
    ("overflowX", "overflow-x"),
    ("overflowY", "overflow-y"),
    ("scrollbarWidth", "scrollbar-width"),
    ("frame", "-og-frame"),
    ("rive", "-og-frame"),
    ("backdrop", "-og-backdrop"),
    ("display", "display"),
    ("markerType", "-og-marker-type"),
    ("markerPosition", "-og-marker-position"),
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
    ("userSelect", "user-select"),
    ("textPath", "-og-text-path"),
    ("textPathOffset", "-og-text-path-offset"),
    ("textPathSide", "-og-text-path-side"),
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
    ("cue", "-og-cue"),
    ("fit", "object-fit"),
    ("uv", "object-view-box"),
    ("uv", "-og-uv"),
    ("appearance", "appearance"),
    ("fillOrigin", "-og-fill-origin"),
    ("pickerMaxHeight", "-og-picker-max-height"),
];

/// Reflected fields affected even when a winning declaration is invalid.
pub(crate) fn targets(name: &str) -> impl Iterator<Item = &'static str> + '_ {
    let alias = match name {
        "display" => Some("visible"),
        "-og-control-size" => Some("size"),
        _ => None,
    };
    let both: Vec<&'static str> = match name {
        "margin" => vec!["marginTop", "marginRight", "marginBottom", "marginLeft"],
        "margin-block" => vec!["marginTop", "marginBottom"],
        "margin-inline" => vec!["marginRight", "marginLeft"],
        "margin-inline-start" => vec!["marginLeft"],
        "margin-inline-end" => vec!["marginRight"],
        "margin-block-start" => vec!["marginTop"],
        "margin-block-end" => vec!["marginBottom"],
        _ => super::boxes::longhands(name),
    };
    let target = match name {
        "grid-row" => "grid-column",
        "min-width" | "max-width" => "width",
        "min-height" | "max-height" => "height",
        _ => name,
    };
    alias.into_iter().chain(both).chain(
        STYLE
            .iter()
            .filter_map(move |(prop, css)| (*css == target).then_some(*prop)),
    )
}

fn conv(prop: &str) -> Conv {
    match prop {
        "width" | "height" => Conv::Size,
        "padding" => Conv::Insets,
        p if p.starts_with("border") && p.ends_with("Width") => Conv::Len,
        p if p.starts_with("border") && p.ends_with("Color") => Conv::Paint,
        p if p.starts_with("border") && p.ends_with("Style") => Conv::Kebab,
        "outlineWidth" | "outlineOffset" => Conv::Len,
        "outlineColor" => Conv::Paint,
        "outlineStyle" | "overflowX" | "overflowY" | "scrollbarWidth" => Conv::Kebab,
        "boxShadow" | "uv" => Conv::Raw,
        "marginTop" | "marginRight" | "marginBottom" | "marginLeft" => Conv::Margin,
        "gap" | "radius" | "fade" | "start" | "top" | "end" | "bottom" | "hitSlop"
        | "pickerMaxHeight" => Conv::Len,
        "aspect" | "opacity" | "z" | "scale" | "maxLines" | "fillOrigin" => Conv::Num,
        "rotate" => Conv::Angle,
        "background" | "backdrop" | "fadeInto" | "color" | "secondaryTint" | "tint" | "ink" => {
            Conv::Paint
        }
        "align" | "cross" | "direction" | "frame" | "anchorAt" | "anchorSelf" | "reduced"
        | "textAlign" | "fit" | "display" | "markerType" | "markerPosition" | "userSelect"
        | "appearance" | "labelDirection" => Conv::Kebab,
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
        "textPath" | "textPathOffset" => Conv::Raw,
        "textPathSide" => Conv::Kebab,
        "area" | "cue" => Conv::Ident,
        "areas" | "overlaps" => Conv::Strs,
        "columns" | "rows" => Conv::Tracks,
        "motion" | "layoutMotion" => Conv::Transition,
        "enter" | "exit" => Conv::Presence,
        "pivot" => Conv::Origin,
        _ => Conv::Joint,
    }
}

/// How a declared prop's value is written in CSS (the catalogue's
/// column); `None` for a prop written with its partner or as an attribute.
/// `words` are a word prop's values as the element names them.
pub fn value_form(prop: &str, words: &[&str]) -> Option<String> {
    if let Some(j) = joint_form(css_name(prop)?) {
        return Some(j.into());
    }
    Some(match conv(prop) {
        Conv::Len => "<n>px".into(),
        Conv::Num => "<number>".into(),
        Conv::Angle => "<n>deg | <n>rad | <n>turn".into(),
        Conv::Paint => "var(--<role>) | #rrggbb[aa]".into(),
        Conv::Size => {
            "fit-content | stretch | <n>px | <n>% | calc(…) | min(…) | max(…) | clamp(…) (min-/max- bounds)"
                .into()
        }
        Conv::Insets => "1 to 4 lengths (top right bottom left)".into(),
        Conv::Margin => "<n>px | <n>em | <n>% | auto | calc(…)".into(),
        Conv::Raw if prop == "boxShadow" => {
            "[inset] <x> <y> [<blur> [<spread>]] [<colour>], … | none".into()
        }
        Conv::Bool(t, f) => format!("{t} | {f}"),
        Conv::Kebab => words
            .iter()
            .map(|w| kebab(w))
            .collect::<Vec<_>>()
            .join(" | "),
        Conv::Kw(t) => t.iter().map(|(_, c)| *c).collect::<Vec<_>>().join(" | "),
        Conv::Var(ns) => format!("var(--{ns}-<name>)"),
        Conv::Str => "\"<text>\"".into(),
        Conv::Ident => "<name>".into(),
        Conv::Strs => "\"<a>\" \"<b>\" …".into(),
        Conv::Tracks => "120px 1fr auto 25% minmax(80px, 200px)".into(),
        Conv::Transition => {
            "none | <ms>ms [ease] | -og-spring(<ms>ms, <bounce>) | var(--motion-<name>)".into()
        }
        Conv::Presence => "<kind> [<distance>px] [<transition>]".into(),
        Conv::Origin => "<x>% <y>%".into(),
        Conv::Rive => "rive(\"<file>.riv#<Artboard>\")".into(),
        Conv::Raw if prop == "uv" => "inset(<t>% <r>% <b>% <l>%) | -og-uv: <u0> <v0> <u1> <v1>".into(),
        Conv::Raw if prop == "textPath" => {
            "none | path(\"<svg path data>\") | circle([<n>px])".into()
        }
        Conv::Raw => "<n>px | <n>%".into(),
        Conv::Joint => return None,
    })
}

/// How a property that sets several props at once is written.
fn joint_form(css: &str) -> Option<&'static str> {
    Some(match css {
        "translate" => "<x>px [<y>px]",
        "scale" => "<both> | <x> <y>",
        "-og-skew" => "<x>deg [<y>deg]",
        "grid-column" => "<n> | <n> / span <n> | span <n>",
        "grid-auto-flow" => "row | column [dense]",
        _ => return None,
    })
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
pub(crate) fn unsupported(name: &str) -> Option<&'static str> {
    Some(match name {
        "flex" | "flex-grow" | "flex-shrink" | "flex-basis" => {
            "write `width: stretch` (or `height`) to take the room left"
        }
        "float" | "clear" => "write `display` as the kind: a `row` or a `grid`",
        "left" => "write `inset-inline-start` (it mirrors right to left)",
        "right" => "write `inset-inline-end` (it mirrors right to left)",
        "font-size" | "font-weight" | "line-height" | "letter-spacing" => {
            "text takes a step of the style's type ramp: `font: var(--text-caption)`"
        }
        "transform" => {
            "write each part as its own property: `translate: 0 -4px`, `scale: 1.05`, `rotate: 0.1rad`, `-og-skew`"
        }
        "visibility" => "write the attribute `visible false` (or `visible bind …`)",
        "background-color" | "background-image" => "write `background: var(--panel)` or `#rrggbb`",
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
        Value::String(s) if s.starts_with("var(--") => Ok(s.clone()),
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
    if let Some(v) = rest.first()
        && px(v).is_ok()
    {
        out.push(' ');
        out.push_str(&px(v)?);
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
                    if t == "0" || t.starts_with("var(--") {
                        t.to_string()
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
        Conv::Margin => match vals {
            [_, _, _, Value::Number(a)] if a.text() != "0" => Ok("auto".into()),
            [Value::Number(p), Value::Number(e), Value::Number(c), _] => {
                let pct = super::calc::number(c.as_f32() as f64 * 100.0);
                let terms: Vec<String> = [
                    (pct, "%"),
                    (p.text().to_string(), "px"),
                    (e.text().to_string(), "em"),
                ]
                .into_iter()
                .filter(|(v, _)| v != "0")
                .map(|(v, u)| format!("{v}{u}"))
                .collect();
                Ok(match terms.as_slice() {
                    [] => "0".into(),
                    [t] => t.clone(),
                    _ => format!("calc({})", terms.join(" + ").replace("+ -", "- ")),
                })
            }
            _ => Err(format!("{prop} takes a length or auto")),
        },
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
        Conv::Raw => Ok(text_of(one()?)),
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
            if let [v] = vals
                && px(v).is_ok()
            {
                return px(v);
            }
            let w = vals.first().map(text_of).unwrap_or_default();
            match w.as_str() {
                "fixed" => px(vals.get(1).ok_or("fixed <px>")?),
                "percent" => Ok(format!(
                    "{}%",
                    shift(&vals.get(1).map(text_of).ok_or("percent <0–1>")?, 2)
                )),
                "calc" => {
                    let n: Vec<f64> = vals[2..]
                        .iter()
                        .map(|v| v.as_number().map_or(0.0, |n| n.as_f32() as f64))
                        .collect();
                    let sum = |k: usize| {
                        let (pct, px) = (
                            n.get(2 * k).copied().unwrap_or(0.0),
                            n.get(2 * k + 1).copied().unwrap_or(0.0),
                        );
                        super::calc::sum_text(
                            &[("%", pct * 100.0), ("px", px)]
                                .into_iter()
                                .filter(|(_, v)| *v != 0.0)
                                .collect::<Vec<_>>(),
                        )
                    };
                    let op = vals.get(1).map(text_of).unwrap_or_default();
                    Ok(match op.as_str() {
                        "sum" => sum(0),
                        "clamp" => format!("clamp({}, {}, {})", sum(0), sum(1), sum(2)),
                        op => {
                            let k = n.len() / 2;
                            format!("{op}({})", (0..k).map(sum).collect::<Vec<_>>().join(", "))
                        }
                    })
                }
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
        } else if matches!(ch, '+' | '-') && c.get(*i + 1).is_none_or(|d| d.is_whitespace()) {
            // A math operator (CSS needs spaces around + and -).
            *i += 1;
            args.last_mut().unwrap().push(Tok::Ident(ch.to_string()));
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
        Tok::Func(f, _) if f == "var" => {
            let name = kebab(&want_var(t, "")?);
            if crate::Theme::default().length(&name).is_none() {
                return Err(format!("unresolved length token --{name}"));
            }
            Ok(Value::String(format!("var(--{name})")))
        }
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

/// A margin edge as its px, em, fraction of the parent's content width and
/// auto flag: `12px`, `0.67em`, `5%`, `0`, `auto` or a reduced
/// `calc(50% - 12px)`.
fn margin_vals(t: &Tok) -> Result<Vec<Value>, String> {
    let n = |v: f64| number(&super::calc::number(v));
    if matches!(t, Tok::Ident(w) if w == "auto") {
        return [0.0, 0.0, 0.0, 1.0].into_iter().map(n).collect();
    }
    if let Tok::Func(f, _) = t
        && f == "var"
    {
        let v = want_len(t)?;
        return Ok(vec![v, n(0.0)?, n(0.0)?, n(0.0)?]);
    }
    let [pct, px, em] = want_lin(t)?;
    [px, em, pct, 0.0].into_iter().map(n).collect()
}

/// A size's reduced math function as its ogt form, `calc <op> <pct> <px>
/// …`: `calc(100% - 12px)` → `calc sum 1 -12`, `min(100%, 600px)` → `calc
/// min 1 0 0 600`.
fn size_calc(name: &str, f: &str, args: &[Vec<Tok>]) -> Result<Vec<Value>, String> {
    let lengths: Vec<[f64; 3]> = if f == "calc" {
        vec![want_lin(&Tok::Func(f.into(), args.to_vec()))?]
    } else {
        args.iter()
            .map(|arg| match arg.as_slice() {
                [t] => want_lin(t),
                _ => Err(format!("{f}() takes one length an argument")),
            })
            .collect::<Result<_, _>>()?
    };
    let count_ok = match f {
        "calc" => lengths.len() == 1,
        "clamp" => lengths.len() == 3,
        _ => (1..=3).contains(&lengths.len()),
    };
    if !count_ok {
        return Err(format!(
            "{name}: {f}() takes {}",
            if f == "clamp" {
                "three lengths"
            } else {
                "one to three lengths"
            }
        ));
    }
    if lengths.iter().any(|l| l[2] != 0.0) {
        return Err(format!("{name} takes px and %, not em"));
    }
    let op = if f == "calc" { "sum" } else { f };
    let mut out = vec![Value::Word("calc".into()), Value::Word(op.into())];
    for [pct, px, _] in lengths {
        out.push(number(&super::calc::number(pct))?);
        out.push(number(&super::calc::number(px))?);
    }
    Ok(out)
}

/// A length as its fraction, px and em terms: `12px`, `50%`, `1.5em`, `0`,
/// or a `calc()` doc/calc.rs reduced to a sum (`calc(50% - 12px + 1em)`).
pub(crate) fn want_lin(t: &Tok) -> Result<[f64; 3], String> {
    fn term(t: &Tok, sign: f64, out: &mut [f64; 3]) -> Result<(), String> {
        let Tok::Num(n, u) = t else {
            return Err(format!("'{}' is not a length (px, %, em)", describe(t)));
        };
        let v: f64 = n.parse().map_err(|_| format!("'{n}' is not a number"))?;
        let k = match u.as_str() {
            "%" => 0,
            "px" => 1,
            "em" => 2,
            "" if v == 0.0 => 1,
            _ => return Err(format!("'{n}{u}' is not a length (px, %, em)")),
        };
        out[k] += sign * if k == 0 { v / 100.0 } else { v };
        Ok(())
    }
    let mut out = [0.0; 3];
    match t {
        Tok::Func(f, args) if f == "calc" => {
            let [toks] = args.as_slice() else {
                return Err("calc() takes one expression".into());
            };
            let mut sign = 1.0;
            for (k, t) in toks.iter().enumerate() {
                match t {
                    Tok::Ident(op) if k % 2 == 1 && (op == "+" || op == "-") => {
                        sign = if op == "-" { -1.0 } else { 1.0 };
                    }
                    t if k % 2 == 0 => term(t, sign, &mut out)?,
                    t => return Err(format!("'{}' is not part of a reduced calc()", describe(t))),
                }
            }
        }
        Tok::Func(f, _) if matches!(f.as_str(), "min" | "max" | "clamp") => {
            return Err(format!(
                "{f}() over lengths in more than one unit takes a width or a height; a margin takes calc()"
            ));
        }
        t => term(t, 1.0, &mut out)?,
    }
    Ok(out)
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

pub(crate) fn paint_vals(t: &[Tok]) -> Result<Vec<Value>, String> {
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
        Tok::Func(f, _) if f == "var" => want_len(t),
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
    if name == "display" {
        // A box's (`<div>`…) layout is its display; every other element's
        // layout is its tag, so there display only shows or hides it.
        let layout = match value.trim() {
            "none" => {
                return Ok(vec![e("visible", vec![Value::Word("false".into())])]);
            }
            "initial" => {
                return Ok(vec![e("visible", vec![Value::Word("true".into())])]);
            }
            "block" | "inline-block" | "flow-root" | "list-item" => "block",
            "flex" | "inline-flex" => "flex",
            "grid" | "inline-grid" => "grid",
            other => {
                return Err(format!(
                    "display: {other} is not supported; a <div> takes block, flex or grid (or none), and other elements' layout is their tag"
                ));
            }
        };
        return Ok(vec![
            e("visible", vec![Value::Word("true".into())]),
            e("display", vec![Value::Word(layout.into())]),
        ]);
    }
    if name == "-og-control-size" {
        return Ok(vec![e("size", vec![Value::Word(value.trim().into())])]);
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
        "-og-backdrop" if !state => Ok(vec![e("backdrop", paint_vals(&t)?)]),
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
                    Tok::Func(f, args)
                        if matches!(f.as_str(), "calc" | "min" | "max" | "clamp") =>
                    {
                        size_calc(name, f, args)?
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
        "margin"
        | "margin-block"
        | "margin-inline"
        | "margin-inline-start"
        | "margin-inline-end"
        | "margin-block-start"
        | "margin-block-end" => {
            // CSS order: top, right, bottom, left (left to right).
            let sides: [Option<&Tok>; 4] = match (name, t.as_slice()) {
                ("margin-block-start", [a]) => [Some(a), None, None, None],
                ("margin-inline-end", [a]) => [None, Some(a), None, None],
                ("margin-block-end", [a]) => [None, None, Some(a), None],
                ("margin-inline-start", [a]) => [None, None, None, Some(a)],
                ("margin", [a]) => [Some(a); 4],
                ("margin", [v, h]) => [Some(v), Some(h), Some(v), Some(h)],
                ("margin", [top, h, b]) => [Some(top), Some(h), Some(b), Some(h)],
                ("margin", [top, r, b, l]) => [Some(top), Some(r), Some(b), Some(l)],
                ("margin-block", [a]) => [Some(a), None, Some(a), None],
                ("margin-block", [a, b]) => [Some(a), None, Some(b), None],
                ("margin-inline", [a]) => [None, Some(a), None, Some(a)],
                ("margin-inline", [start, end]) => [None, Some(end), None, Some(start)],
                _ => {
                    return Err(format!(
                        "{name} takes 1 to {} values",
                        if name == "margin" { 4 } else { 2 }
                    ));
                }
            };
            let mut out = Vec::new();
            for (side, prop) in
                sides
                    .into_iter()
                    .zip(["marginTop", "marginRight", "marginBottom", "marginLeft"])
            {
                if let Some(side) = side {
                    out.push(e(prop, margin_vals(side)?));
                }
            }
            Ok(out)
        }
        n if super::boxes::longhands(n).len() > 1
            || n == "box-shadow"
            || n.ends_with("-width") && (n.starts_with("border-") || n == "outline-width")
            || n.ends_with("-color") && (n.starts_with("border-") || n == "outline-color") =>
        {
            super::boxes::lower(n, value, &t)
                .map(|parts| parts.into_iter().map(|(k, v)| e(k, v)).collect())
        }
        "grid-column" | "grid-row" => {
            let (start, span) = grid_line(&t)?;
            let axis = if name == "grid-column" { 0 } else { 1 };
            Ok(vec![Part::Grid(axis, start, span)])
        }
        "-og-text-path" => {
            crate::text_path::parse_shape(value)?;
            Ok(vec![e(
                "textPath",
                vec![Value::String(value.trim().to_owned())],
            )])
        }
        "-og-text-path-offset" => {
            crate::text_path::parse_offset(value)?;
            Ok(vec![e(
                "textPathOffset",
                vec![Value::String(value.trim().to_owned())],
            )])
        }
        "object-view-box" => {
            // CSS's inset() of the texture, in percentages: the box's
            // u0 v0 u1 v1 (px need the texture's size, which the box lacks).
            let [Tok::Func(f, args)] = t.as_slice() else {
                return Err("object-view-box: none | inset(<t>% [<r>% [<b>% [<l>%]]])".into());
            };
            if f != "inset" {
                return Err(format!("object-view-box takes inset(…), not {f}(…)"));
            }
            let pct: Vec<f64> = args
                .iter()
                .flatten()
                .map(|v| match v {
                    Tok::Num(n, u) if u == "%" => n
                        .parse::<f64>()
                        .map(|n| n / 100.0)
                        .map_err(|_| format!("'{n}%' is not a number")),
                    Tok::Num(n, _) if n.parse::<f64>() == Ok(0.0) => Ok(0.0),
                    v => Err(format!(
                        "object-view-box: inset() takes percentages of the image, not '{}'",
                        describe(v)
                    )),
                })
                .collect::<Result<_, _>>()?;
            let [top, right, bottom, left] = match pct[..] {
                [a] => [a; 4],
                [v, h] => [v, h, v, h],
                [top, h, b] => [top, h, b, h],
                [top, r, b, l] => [top, r, b, l],
                _ => return Err("inset() takes 1 to 4 percentages".into()),
            };
            if left + right >= 1.0 || top + bottom >= 1.0 {
                return Err("object-view-box: inset() leaves none of the image".into());
            }
            Ok(vec![e(
                "uv",
                [left, top, 1.0 - right, 1.0 - bottom]
                    .into_iter()
                    .map(|v| {
                        let s = format!("{v:.6}");
                        number(s.trim_end_matches('0').trim_end_matches('.'))
                    })
                    .collect::<Result<_, _>>()?,
            )])
        }
        "-og-uv" => {
            let n: Vec<Value> = t.iter().map(want_num).collect::<Result<_, _>>()?;
            if n.len() != 4 {
                return Err("-og-uv: <u0> <v0> <u1> <v1>, fractions of the texture".into());
            }
            Ok(vec![e("uv", n)])
        }
        "flex-direction" => {
            let w = camel(want_ident(&one(&t, name)?, "a direction")?);
            let both = vec![Value::Word(w)];
            Ok(vec![
                e("direction", both.clone()),
                e("labelDirection", both),
            ])
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
                        // assets/ (`ui/chest.riv#Chest`, docs/ui.md §3.3).
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
                    if let Some(d) = rest.first()
                        && (matches!(d, Tok::Num(..))
                            || matches!(d, Tok::Func(f, _) if f == "var" && want_len(d).is_ok()))
                    {
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
                Conv::Margin => margin_vals(&one(&t, name)?)?,
                Conv::Size | Conv::Insets | Conv::Joint | Conv::Rive | Conv::Raw => {
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
        " (the properties: docs/ui.md §3.2)".into()
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

/// A size being joined: its entry's index in the block, its kind, its
/// `min-` and its `max-`.
type SizeParts = (usize, Vec<Value>, Option<Value>, Option<Value>);

pub(crate) fn declarations(text: &str, line: usize) -> Result<Vec<Entry>, String> {
    declarations_at([(line, text)])
}

pub(crate) fn declarations_at<'a>(
    lines: impl IntoIterator<Item = (usize, &'a str)>,
) -> Result<Vec<Entry>, String> {
    let mut frame = Frame::new(Entry::new("style"));
    for (line, text) in lines {
        for (name, value) in split_decls(text)? {
            for part in lower_decl(&name, &value, false)? {
                frame.part(part, line)?;
            }
        }
    }
    frame.finish()?;
    Ok(frame.entry.block.map_or_else(Vec::new, |b| b.entries))
}

struct Frame {
    entry: Entry,
    /// Sizes and grid lines, joined at the block's end.
    size: [Option<SizeParts>; 2],
    grid: [Option<usize>; 2],
    grid_parts: [[Option<Value>; 2]; 2],
}

impl Frame {
    fn new(entry: Entry) -> Frame {
        Frame {
            entry,
            size: [None, None],
            grid: [None, None],
            grid_parts: [[None, None], [None, None]],
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

    /// Joins sizes and grid lines.
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
