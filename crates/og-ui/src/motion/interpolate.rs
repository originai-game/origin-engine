//! Interpolation, addition and accumulation of computed CSS values for
//! transitions and `@keyframes` (CSS Values 4 §3, Web Animations 1 §5,
//! CSS Transitions 1 and 2, CSS Transforms 2 §11–§16, Filter Effects 1
//! §11, CSS Backgrounds 3 §7.1, CSS Color 4 §12). Values travel as the
//! cascade's computed text, so every property animates by its value
//! types: numbers, lengths and percentages (mixed units through `calc()`),
//! angles, times, colours (through `color-mix()`, premultiplied, in sRGB
//! for legacy colours and Oklab otherwise), and functions, lists and
//! gradients component by component. Shadows, filters and transforms
//! follow their own list rules. A pair that does not match is discrete: it
//! flips at 50%, except that `display`, `visibility`, `content-visibility`
//! and `overlay` keep their shown value for the whole of a move to or from
//! their hidden one.

use crate::transform3d::{self as t3, LenPct, Mat4, TransformFn};

/// How a keyframe's value combines with the underlying value
/// (`animation-composition`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Composite {
    #[default]
    Replace,
    Add,
    Accumulate,
}

impl Composite {
    pub fn from_css(word: &str) -> Option<Composite> {
        Some(match word {
            "replace" => Composite::Replace,
            "add" => Composite::Add,
            "accumulate" => Composite::Accumulate,
            _ => return None,
        })
    }
}

/// Properties that do not animate at all (CSS's "not animatable").
pub(crate) fn animatable(property: &str) -> bool {
    !(property.starts_with("animation")
        || property.starts_with("transition")
        || property.starts_with("scroll-timeline")
        || property.starts_with("view-timeline")
        || property.starts_with("container")
        || matches!(
            property,
            "timeline-scope" | "direction" | "unicode-bidi" | "will-change" | "all"
        ))
}

/// A property's initial value, for a transition to or from a declaration
/// that is not there and a keyframe's missing end; `None` where it is not
/// fixed (an inherited or element-dependent one), which does not move.
pub(crate) fn initial(property: &str) -> Option<&'static str> {
    Some(match property {
        "box-shadow" | "text-shadow" | "filter" | "backdrop-filter" | "transform" | "clip-path"
        | "mask-image" | "mask" | "background-image" | "max-width" | "max-height"
        | "perspective" | "translate" | "rotate" | "scale" => "none",
        "background-color" => "transparent",
        "outline-color"
        | "text-decoration-color"
        | "column-rule-color"
        | "text-emphasis-color"
        | "-webkit-text-stroke-color" => "currentcolor",
        p if p.starts_with("border") && p.ends_with("-color") => "currentcolor",
        p if p.starts_with("padding") || p.starts_with("margin") || p.starts_with("scroll-") => {
            "0px"
        }
        "outline-offset"
        | "letter-spacing"
        | "word-spacing"
        | "text-indent"
        | "text-underline-offset"
        | "-webkit-text-stroke-width" => "0px",
        "width" | "height" | "min-width" | "min-height" | "inline-size" | "block-size" | "top"
        | "right" | "bottom" | "left" | "flex-basis" | "z-index" | "aspect-ratio" => "auto",
        "inset" | "inset-block" | "inset-inline" | "inset-block-start" | "inset-block-end"
        | "inset-inline-start" | "inset-inline-end" => "auto",
        "flex-grow" | "order" => "0",
        "flex-shrink" | "opacity" => "1",
        "background-position" | "mask-position" => "0% 0%",
        "background-size" | "mask-size" => "auto",
        "transform-origin" | "perspective-origin" => "50% 50%",
        "visibility" => "visible",
        "gap" | "row-gap" | "column-gap" => "normal",
        _ => return None,
    })
}

/// The hidden value of a property whose shown value holds through a
/// discrete move to or from it.
fn hidden(property: &str) -> Option<&'static str> {
    Some(match property {
        "display" => "none",
        "visibility" => "hidden",
        "content-visibility" => "hidden",
        "overlay" => "none",
        _ => return None,
    })
}

/// A property's value at `p` (0 is `a`, 1 is `b`; easings may go beyond),
/// discrete when the pair cannot interpolate.
pub(crate) fn mix(property: &str, a: &str, b: &str, p: f32) -> String {
    interpolate(property, a, b, p).unwrap_or_else(|| discrete(property, a, b, p).to_string())
}

/// The discrete step from `a` to `b` at `p`.
pub(crate) fn discrete<'a>(property: &str, a: &'a str, b: &'a str, p: f32) -> &'a str {
    if let Some(none) = hidden(property)
        && (a == none) != (b == none)
        && p > 0.0
        && p < 1.0
    {
        // CSS Transitions 2 §3 (visibility: when either is visible) and
        // CSS Display 4 §2.9: the shown value holds in between.
        if property == "visibility" && a != "visible" && b != "visible" {
            return if p < 0.5 { a } else { b };
        }
        return if a == none { b } else { a };
    }
    if p < 0.5 { a } else { b }
}

/// `a` to `b` at `p` by the property's value types, or `None` when the
/// pair is discrete.
pub(crate) fn interpolate(property: &str, a: &str, b: &str, p: f32) -> Option<String> {
    let (a, b) = (a.trim(), b.trim());
    if a == b {
        return Some(a.to_string());
    }
    if !animatable(property) || hidden(property).is_some() {
        return None;
    }
    match property {
        "transform" => return transform(a, b, p),
        "box-shadow" | "text-shadow" => {
            let (x, y) = shadow_pair(property, a, b)?;
            return Some(write(&lerp_list(property, &x, &y, p)?));
        }
        "filter" | "backdrop-filter" => {
            let (x, y) = filter_pair(a, b)?;
            return Some(write(&lerp_list(property, &x, &y, p)?));
        }
        _ => {}
    }
    let (mut x, mut y) = (tokens(a)?, tokens(b)?);
    repeat_lists(property, &mut x, &mut y);
    Some(write(&lerp_list(property, &x, &y, p)?))
}

/// `value` combined onto `underlying` (Web Animations 1 §5.4.4).
pub(crate) fn compose(property: &str, underlying: &str, value: &str, op: Composite) -> String {
    if op == Composite::Replace || !animatable(property) || hidden(property).is_some() {
        return value.to_string();
    }
    let add = op == Composite::Add;
    let combined = match property {
        "transform" => compose_transform(underlying, value, add),
        // Lists add by concatenation and accumulate item by item.
        "box-shadow" | "text-shadow" | "filter" | "backdrop-filter" if add => {
            match (underlying.trim(), value.trim()) {
                ("none", v) => Some(v.to_string()),
                (u, "none") => Some(u.to_string()),
                (u, v) if property.ends_with("shadow") => Some(format!("{u}, {v}")),
                (u, v) => Some(format!("{u} {v}")),
            }
        }
        "box-shadow" | "text-shadow" => shadow_pair(property, underlying, value)
            .and_then(|(x, y)| sum_list(property, &x, &y))
            .map(|t| write(&t)),
        "filter" | "backdrop-filter" => filter_pair(underlying, value)
            .and_then(|(x, y)| sum_list(property, &x, &y))
            .map(|t| write(&t)),
        _ => tokens(underlying)
            .zip(tokens(value))
            .and_then(|(x, y)| sum_list(property, &x, &y))
            .map(|t| write(&t)),
    };
    combined.unwrap_or_else(|| value.to_string())
}

// ---------------------------------------------------------------- tokens

#[derive(Clone, Debug, PartialEq)]
enum T {
    /// A number with its unit (`""`, `%`, `px`, `deg`…).
    Num(f32, String),
    /// A colour, as written.
    Color(String),
    Func(String, Vec<T>),
    Word(String),
    Comma,
    Slash,
    /// Emitted text (a `calc()` or `color-mix()` of two values).
    Raw(String),
}

const COLOR_FUNCTIONS: [&str; 14] = [
    "rgb",
    "rgba",
    "hsl",
    "hsla",
    "hwb",
    "lab",
    "lch",
    "oklab",
    "oklch",
    "color",
    "color-mix",
    "light-dark",
    "contrast-color",
    "device-cmyk",
];

fn tokens(text: &str) -> Option<Vec<T>> {
    let chars: Vec<char> = text.chars().collect();
    let (out, at) = scan(&chars, 0)?;
    (at == chars.len()).then_some(out)
}

/// Tokens from `at` up to an unmatched `)` or the end.
fn scan(c: &[char], mut at: usize) -> Option<(Vec<T>, usize)> {
    let mut out = Vec::new();
    while at < c.len() {
        let ch = c[at];
        match ch {
            _ if ch.is_whitespace() => at += 1,
            ',' => {
                out.push(T::Comma);
                at += 1;
            }
            '/' => {
                out.push(T::Slash);
                at += 1;
            }
            ')' => return Some((out, at)),
            '(' => return None,
            '"' | '\'' => {
                let end = c[at + 1..].iter().position(|&q| q == ch)? + at + 2;
                out.push(T::Word(c[at..end].iter().collect()));
                at = end;
            }
            _ => {
                let start = at;
                while at < c.len()
                    && !c[at].is_whitespace()
                    && !matches!(c[at], ',' | '/' | '(' | ')')
                {
                    at += 1;
                }
                let word: String = c[start..at].iter().collect();
                if at < c.len() && c[at] == '(' {
                    let name = word.to_ascii_lowercase();
                    let (args, close) = scan(c, at + 1)?;
                    if close >= c.len() {
                        return None;
                    }
                    at = close + 1;
                    if COLOR_FUNCTIONS.contains(&name.as_str()) {
                        out.push(T::Color(c[start..at].iter().collect()));
                    } else if name == "url" {
                        out.push(T::Word(c[start..at].iter().collect()));
                    } else {
                        out.push(T::Func(name, args));
                    }
                } else {
                    out.push(word_token(word));
                }
            }
        }
    }
    Some((out, at))
}

fn word_token(word: String) -> T {
    let numeric = {
        let b = word.as_bytes();
        let s = usize::from(matches!(b[0], b'+' | b'-'));
        b.get(s).is_some_and(|c| c.is_ascii_digit())
            || (b.get(s) == Some(&b'.') && b.get(s + 1).is_some_and(|c| c.is_ascii_digit()))
    };
    if numeric {
        let split = word
            .char_indices()
            .skip(1)
            .find(|&(i, ch)| {
                !(ch.is_ascii_digit()
                    || ch == '.'
                    || ((ch == 'e' || ch == 'E')
                        && word[i + 1..]
                            .chars()
                            .next()
                            .is_some_and(|n| n.is_ascii_digit() || n == '-' || n == '+'))
                    || ((ch == '-' || ch == '+') && matches!(&word[i - 1..i], "e" | "E")))
            })
            .map_or(word.len(), |(i, _)| i);
        if let Ok(v) = word[..split].parse::<f32>()
            && v.is_finite()
        {
            return T::Num(v, word[split..].to_ascii_lowercase());
        }
    }
    let lower = word.to_ascii_lowercase();
    if word.starts_with('#')
        || lower == "transparent"
        || lower == "currentcolor"
        || crate::css_color::is_named(&lower)
    {
        return T::Color(word);
    }
    T::Word(word)
}

fn num(v: f32) -> String {
    let r = (v * 10_000.0).round() / 10_000.0;
    if r == 0.0 { "0".into() } else { format!("{r}") }
}

fn write(t: &[T]) -> String {
    let mut out = String::new();
    for (i, tok) in t.iter().enumerate() {
        if i > 0 && !matches!(tok, T::Comma) {
            out.push(' ');
        }
        match tok {
            T::Num(v, unit) => {
                out.push_str(&num(*v));
                out.push_str(unit);
            }
            T::Color(s) | T::Word(s) | T::Raw(s) => out.push_str(s),
            T::Func(name, args) => {
                out.push_str(name);
                out.push('(');
                out.push_str(&write(args));
                out.push(')');
            }
            T::Comma => out.push(','),
            T::Slash => out.push('/'),
        }
    }
    out
}

fn one(t: &T) -> String {
    write(std::slice::from_ref(t))
}

// ---------------------------------------------------------- interpolation

const LENGTHS: [&str; 30] = [
    "px", "%", "em", "rem", "ex", "rex", "ch", "rch", "cap", "rcap", "ic", "ric", "lh", "rlh",
    "vw", "vh", "vmin", "vmax", "vi", "vb", "cqw", "cqh", "cqi", "cqb", "cqmin", "cqmax", "cm",
    "mm", "in", "pt",
];

fn is_length(unit: &str) -> bool {
    LENGTHS.contains(&unit)
        || matches!(unit, "pc" | "q")
        || unit.ends_with("vh")
        || unit.ends_with("vw")
}

/// Angles in degrees and times in ms, so mixed units meet.
fn canonical(v: f32, unit: &str) -> Option<(f32, &'static str)> {
    Some(match unit {
        "deg" => (v, "deg"),
        "rad" => (v.to_degrees(), "deg"),
        "grad" => (v * 0.9, "deg"),
        "turn" => (v * 360.0, "deg"),
        "s" => (v * 1000.0, "ms"),
        "ms" => (v, "ms"),
        _ => return None,
    })
}

/// Integer-valued properties round (CSS Values 4 §3.1).
fn integer(property: &str) -> bool {
    matches!(
        property,
        "z-index"
            | "order"
            | "column-count"
            | "orphans"
            | "widows"
            | "-webkit-line-clamp"
            | "line-clamp"
    ) || property.starts_with("grid-row")
        || property.starts_with("grid-column")
}

fn lerp(a: f32, b: f32, p: f32) -> f32 {
    a + (b - a) * p
}

fn lerp_list(property: &str, a: &[T], b: &[T], p: f32) -> Option<Vec<T>> {
    if a.len() != b.len() {
        return None;
    }
    a.iter()
        .zip(b)
        .map(|(x, y)| lerp_one(property, x, y, p))
        .collect()
}

fn length_like(t: &T) -> bool {
    match t {
        T::Num(v, u) => is_length(u) || (u.is_empty() && *v == 0.0),
        T::Func(name, _) => matches!(name.as_str(), "calc" | "min" | "max" | "clamp"),
        T::Raw(s) => s.starts_with("calc("),
        _ => false,
    }
}

fn lerp_one(property: &str, a: &T, b: &T, p: f32) -> Option<T> {
    if a == b {
        return Some(a.clone());
    }
    Some(match (a, b) {
        (T::Num(x, u), T::Num(y, v)) if u == v => {
            let r = lerp(*x, *y, p);
            T::Num(if integer(property) { r.round() } else { r }, u.clone())
        }
        // A unitless zero is a zero of the other side's unit.
        (T::Num(x, u), T::Num(y, v)) if u.is_empty() && *x == 0.0 => {
            T::Num(lerp(0.0, *y, p), v.clone())
        }
        (T::Num(x, u), T::Num(y, v)) if v.is_empty() && *y == 0.0 => {
            T::Num(lerp(*x, 0.0, p), u.clone())
        }
        (T::Num(x, u), T::Num(y, v))
            if canonical(*x, u).is_some() && canonical(*y, v).is_some() =>
        {
            let ((x, unit), (y, unit2)) = (canonical(*x, u)?, canonical(*y, v)?);
            if unit != unit2 {
                return None;
            }
            T::Num(lerp(x, y, p), unit.into())
        }
        _ if length_like(a) && length_like(b) => {
            // Mixed lengths and percentages: their weighted sum.
            T::Raw(format!(
                "calc({} * {} + {} * {})",
                one(a),
                num(1.0 - p),
                one(b),
                num(p)
            ))
        }
        (T::Color(x), T::Color(y)) => T::Raw(color_mix(x, y, p)),
        (T::Func(f, x), T::Func(g, y)) if f == g => {
            T::Func(f.clone(), lerp_list(property, x, y, p)?)
        }
        _ => return None,
    })
}

fn legacy(color: &str) -> bool {
    let lower = color.to_ascii_lowercase();
    let name = lower.split('(').next().unwrap_or("");
    !lower.contains('(') || matches!(name, "rgb" | "rgba" | "hsl" | "hsla" | "hwb")
}

/// `a` to `b` at `p` as `color-mix()` (CSS Color 4 §12, premultiplied).
fn color_mix(a: &str, b: &str, p: f32) -> String {
    let space = if legacy(a) && legacy(b) {
        "srgb"
    } else {
        "oklab"
    };
    // color-mix() takes percentages in 0–100%; overshoot clamps to an end.
    let pct = (p.clamp(0.0, 1.0) * 100.0).max(0.0);
    if pct <= 0.0 {
        return a.into();
    }
    if pct >= 100.0 {
        return b.into();
    }
    format!("color-mix(in {space}, {a}, {b} {}%)", num(pct))
}

/// Comma-separated repeatable lists (backgrounds, masks) repeat to their
/// least common multiple (CSS Values 4 §3.2).
fn repeat_lists(property: &str, a: &mut Vec<T>, b: &mut Vec<T>) {
    if !(property.starts_with("background") || property.starts_with("mask")) {
        return;
    }
    let items = |t: &[T]| {
        t.split(|x| *x == T::Comma)
            .map(<[T]>::to_vec)
            .collect::<Vec<_>>()
    };
    let (x, y) = (items(a), items(b));
    if x.len() == y.len() {
        return;
    }
    let gcd = |mut m: usize, mut n: usize| {
        while n != 0 {
            (m, n) = (n, m % n);
        }
        m
    };
    let lcm = x.len() * y.len() / gcd(x.len(), y.len());
    let join = |items: &[Vec<T>]| {
        let mut out = Vec::new();
        for k in 0..lcm {
            if k > 0 {
                out.push(T::Comma);
            }
            out.extend(items[k % items.len()].iter().cloned());
        }
        out
    };
    (*a, *b) = (join(&x), join(&y));
}

// ---------------------------------------------------------------- addition

fn sum_list(property: &str, a: &[T], b: &[T]) -> Option<Vec<T>> {
    if a.len() != b.len() {
        return None;
    }
    a.iter()
        .zip(b)
        .map(|(x, y)| sum_one(property, x, y))
        .collect()
}

fn sum_one(property: &str, a: &T, b: &T) -> Option<T> {
    Some(match (a, b) {
        (T::Num(x, u), T::Num(y, v)) if u == v => T::Num(x + y, u.clone()),
        (T::Num(x, u), T::Num(y, v)) if u.is_empty() && *x == 0.0 => T::Num(*y, v.clone()),
        (T::Num(x, u), T::Num(y, v)) if v.is_empty() && *y == 0.0 => T::Num(*x, u.clone()),
        (T::Num(x, u), T::Num(y, v))
            if canonical(*x, u).is_some() && canonical(*y, v).is_some() =>
        {
            let ((x, unit), (y, unit2)) = (canonical(*x, u)?, canonical(*y, v)?);
            if unit != unit2 {
                return None;
            }
            T::Num(x + y, unit.into())
        }
        _ if length_like(a) && length_like(b) => T::Raw(format!("calc({} + {})", one(a), one(b))),
        (T::Color(x), T::Color(y)) => T::Raw(color_sum(x, y)?),
        (T::Func(f, x), T::Func(g, y)) if f == g => {
            // Filter functions whose identity is 1 add around it.
            if matches!(
                f.as_str(),
                "brightness" | "contrast" | "opacity" | "saturate"
            ) && let ([T::Num(x, u)], [T::Num(y, v)]) = (x.as_slice(), y.as_slice())
                && u == v
            {
                let one = if u == "%" { 100.0 } else { 1.0 };
                return Some(T::Func(f.clone(), vec![T::Num(x + y - one, u.clone())]));
            }
            T::Func(f.clone(), sum_list(property, x, y)?)
        }
        // Words and other discrete parts: the effect value replaces.
        _ => b.clone(),
    })
}

/// Two fixed colours added in premultiplied sRGB; `None` for colours that
/// resolve later (tokens, `currentcolor`).
fn color_sum(a: &str, b: &str) -> Option<String> {
    let lit = |s: &str| crate::css_color::parse_str(s).ok()?.literal();
    let (x, y) = (lit(a)?, lit(b)?);
    let alpha = (x.alpha + y.alpha).min(1.0);
    let channel = |i: usize| {
        let premul = x.rgb[i] as f32 * x.alpha + y.rgb[i] as f32 * y.alpha;
        if alpha > 0.0 {
            (premul / alpha).clamp(0.0, 255.0)
        } else {
            0.0
        }
    };
    Some(format!(
        "rgb({} {} {} / {})",
        num(channel(0)),
        num(channel(1)),
        num(channel(2)),
        num(alpha)
    ))
}

// ----------------------------------------------------------------- shadows

/// Both shadow lists in `[inset] x y blur [spread] colour` form, the
/// shorter padded with transparent zero shadows (CSS Backgrounds 3 §7.1).
fn shadow_pair(property: &str, a: &str, b: &str) -> Option<(Vec<T>, Vec<T>)> {
    let box_ = property == "box-shadow";
    let parse = |text: &str| -> Option<Vec<Vec<T>>> {
        if text.trim() == "none" {
            return Some(Vec::new());
        }
        let t = tokens(text)?;
        t.split(|x| *x == T::Comma)
            .map(|item| {
                let inset = item.iter().any(|x| matches!(x, T::Word(w) if w == "inset"));
                let color = item
                    .iter()
                    .find(|x| matches!(x, T::Color(_)))
                    .cloned()
                    .unwrap_or_else(|| T::Color("currentcolor".into()));
                let mut lengths: Vec<T> = item.iter().filter(|x| length_like(x)).cloned().collect();
                let n = if box_ { 4 } else { 3 };
                if lengths.len() < 2 || lengths.len() > n {
                    return None;
                }
                lengths.resize(n, T::Num(0.0, "px".into()));
                let mut out = Vec::new();
                if inset {
                    out.push(T::Word("inset".into()));
                }
                out.extend(lengths);
                out.push(color);
                Some(out)
            })
            .collect()
    };
    let (mut x, mut y) = (parse(a)?, parse(b)?);
    let pad = |like: &[T]| {
        let mut out: Vec<T> = like
            .iter()
            .map(|t| match t {
                T::Color(_) => T::Color("transparent".into()),
                T::Word(w) => T::Word(w.clone()),
                _ => T::Num(0.0, "px".into()),
            })
            .collect();
        out.shrink_to_fit();
        out
    };
    while x.len() < y.len() {
        x.push(pad(&y[x.len()]));
    }
    while y.len() < x.len() {
        y.push(pad(&x[y.len()]));
    }
    let join = |items: Vec<Vec<T>>| {
        let mut out = Vec::new();
        for (k, item) in items.into_iter().enumerate() {
            if k > 0 {
                out.push(T::Comma);
            }
            out.extend(item);
        }
        out
    };
    Some((join(x), join(y)))
}

// ----------------------------------------------------------------- filters

fn filter_identity(f: &str) -> Option<T> {
    let n = |v: f32, u: &str| vec![T::Num(v, u.into())];
    Some(T::Func(
        f.into(),
        match f {
            "blur" => n(0.0, "px"),
            "brightness" | "contrast" | "opacity" | "saturate" => n(1.0, ""),
            "grayscale" | "invert" | "sepia" => n(0.0, ""),
            "hue-rotate" => n(0.0, "deg"),
            "drop-shadow" => vec![
                T::Num(0.0, "px".into()),
                T::Num(0.0, "px".into()),
                T::Num(0.0, "px".into()),
                T::Color("transparent".into()),
            ],
            _ => return None,
        },
    ))
}

/// Both filter lists, the shorter padded with identity functions; `None`
/// when their functions differ (Filter Effects 1 §11).
fn filter_pair(a: &str, b: &str) -> Option<(Vec<T>, Vec<T>)> {
    let parse = |text: &str| -> Option<Vec<T>> {
        if text.trim() == "none" {
            return Some(Vec::new());
        }
        tokens(text)?
            .into_iter()
            .map(|t| match t {
                T::Func(name, args) => {
                    // Omitted arguments take the function's initial value;
                    // a percentage is its fraction.
                    let mut args: Vec<T> = args
                        .into_iter()
                        .map(|a| match a {
                            T::Num(v, u) if u == "%" && name != "drop-shadow" => {
                                T::Num(v / 100.0, String::new())
                            }
                            a => a,
                        })
                        .collect();
                    if args.is_empty() {
                        let T::Func(_, d) = filter_identity(&name)? else {
                            unreachable!()
                        };
                        args = match name.as_str() {
                            "blur" | "hue-rotate" | "drop-shadow" => d,
                            _ => vec![T::Num(1.0, String::new())],
                        };
                    }
                    if name == "drop-shadow" {
                        args = shadow_pair("text-shadow", &write(&args), "none")?.0;
                    }
                    Some(T::Func(name, args))
                }
                _ => None,
            })
            .collect()
    };
    let (mut x, mut y) = (parse(a)?, parse(b)?);
    let name = |t: &T| match t {
        T::Func(n, _) => n.clone(),
        _ => String::new(),
    };
    let shared = x.len().min(y.len());
    if (0..shared).any(|i| name(&x[i]) != name(&y[i])) {
        return None;
    }
    while x.len() < y.len() {
        x.push(filter_identity(&name(&y[x.len()]))?);
    }
    while y.len() < x.len() {
        y.push(filter_identity(&name(&x[y.len()]))?);
    }
    Some((x, y))
}

// -------------------------------------------------------------- transforms

fn identity_of(f: &TransformFn) -> TransformFn {
    match f {
        TransformFn::Translate(..) => {
            TransformFn::Translate(LenPct::default(), LenPct::default(), 0.0)
        }
        TransformFn::Scale(..) => TransformFn::Scale(1.0, 1.0, 1.0),
        TransformFn::Rotate(axis, _) => TransformFn::Rotate(*axis, 0.0),
        TransformFn::Skew(..) => TransformFn::Skew(0.0, 0.0),
        TransformFn::Perspective(_) => TransformFn::Perspective(None),
        TransformFn::Matrix(_) => TransformFn::Matrix(t3::IDENTITY),
    }
}

fn unit_axis(a: [f32; 3]) -> Option<[f32; 3]> {
    let len = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    (len > 1e-6).then(|| [a[0] / len, a[1] / len, a[2] / len])
}

fn same_axis(a: [f32; 3], b: [f32; 3]) -> bool {
    match (unit_axis(a), unit_axis(b)) {
        (Some(x), Some(y)) => x.iter().zip(&y).all(|(p, q)| (p - q).abs() < 1e-4),
        _ => false,
    }
}

/// One pair of functions with a common primitive, at `p`.
fn lerp_fn(a: &TransformFn, b: &TransformFn, p: f32) -> Option<TransformFn> {
    let lp = |x: LenPct, y: LenPct| LenPct {
        px: lerp(x.px, y.px, p),
        frac: lerp(x.frac, y.frac, p),
    };
    Some(match (a, b) {
        (TransformFn::Translate(x0, y0, z0), TransformFn::Translate(x1, y1, z1)) => {
            TransformFn::Translate(lp(*x0, *x1), lp(*y0, *y1), lerp(*z0, *z1, p))
        }
        (TransformFn::Scale(x0, y0, z0), TransformFn::Scale(x1, y1, z1)) => {
            TransformFn::Scale(lerp(*x0, *x1, p), lerp(*y0, *y1, p), lerp(*z0, *z1, p))
        }
        (TransformFn::Rotate(a0, r0), TransformFn::Rotate(a1, r1))
            if same_axis(*a0, *a1) || *r0 == 0.0 || *r1 == 0.0 =>
        {
            let axis = if *r0 == 0.0 { *a1 } else { *a0 };
            TransformFn::Rotate(axis, lerp(*r0, *r1, p))
        }
        (TransformFn::Skew(x0, y0), TransformFn::Skew(x1, y1)) => {
            TransformFn::Skew(lerp(*x0, *x1, p), lerp(*y0, *y1, p))
        }
        (TransformFn::Perspective(d0), TransformFn::Perspective(d1)) => {
            // By the inverse of the distance; none is 0.
            let inv = |d: &Option<f32>| d.map_or(0.0, |d| 1.0 / d.max(1.0));
            let i = lerp(inv(d0), inv(d1), p);
            TransformFn::Perspective((i > 0.0).then_some(1.0 / i))
        }
        _ => return None,
    })
}

fn len_pct_text(v: LenPct) -> String {
    match (v.px, v.frac) {
        (px, 0.0) => format!("{}px", num(px)),
        (0.0, f) => format!("{}%", num(f * 100.0)),
        (px, f) => format!("calc({}px + {}%)", num(px), num(f * 100.0)),
    }
}

fn fn_text(f: &TransformFn) -> String {
    let deg = |r: f32| format!("{}deg", num(r.to_degrees()));
    match f {
        TransformFn::Translate(x, y, z) => format!(
            "translate3d({}, {}, {}px)",
            len_pct_text(*x),
            len_pct_text(*y),
            num(*z)
        ),
        TransformFn::Scale(x, y, z) => format!("scale3d({}, {}, {})", num(*x), num(*y), num(*z)),
        TransformFn::Rotate(a, r) => format!(
            "rotate3d({}, {}, {}, {})",
            num(a[0]),
            num(a[1]),
            num(a[2]),
            deg(*r)
        ),
        TransformFn::Skew(x, y) => format!("skew({}, {})", deg(*x), deg(*y)),
        TransformFn::Perspective(None) => "perspective(none)".into(),
        TransformFn::Perspective(Some(d)) => format!("perspective({}px)", num(*d)),
        TransformFn::Matrix(m) => matrix_text(m),
    }
}

fn matrix_text(m: &Mat4) -> String {
    let mut v = Vec::with_capacity(16);
    for c in 0..4 {
        for row in m {
            v.push(num(row[c]));
        }
    }
    format!("matrix3d({})", v.join(", "))
}

fn list_text(fns: &[TransformFn]) -> String {
    if fns.is_empty() {
        return "none".into();
    }
    fns.iter().map(fn_text).collect::<Vec<_>>().join(" ")
}

/// The product of a list, or `None` when it reads the reference box.
fn product(fns: &[TransformFn]) -> Option<Mat4> {
    let mut m = t3::IDENTITY;
    for f in fns {
        if let TransformFn::Translate(x, y, _) = f
            && (x.frac != 0.0 || y.frac != 0.0)
        {
            return None;
        }
        m = t3::mul(&m, &f.matrix((0.0, 0.0)));
    }
    Some(m)
}

/// The lists, the shorter padded with identities: the matching prefix
/// pairwise and the remainders, which only matrices can join.
fn pair_lists(a: &[TransformFn], b: &[TransformFn]) -> (Vec<TransformFn>, Vec<TransformFn>, usize) {
    let (mut x, mut y) = (a.to_vec(), b.to_vec());
    while x.len() < y.len() {
        x.push(identity_of(&y[x.len()]));
    }
    while y.len() < x.len() {
        y.push(identity_of(&x[y.len()]));
    }
    let k = (0..x.len())
        .find(|&i| lerp_fn(&x[i], &y[i], 0.0).is_none())
        .unwrap_or(x.len());
    (x, y, k)
}

fn transform(a: &str, b: &str, p: f32) -> Option<String> {
    let (x, y) = (t3::parse_transform(a).ok()?, t3::parse_transform(b).ok()?);
    let (x, y, k) = pair_lists(&x, &y);
    let mut out: Vec<TransformFn> = (0..k).map(|i| lerp_fn(&x[i], &y[i], p).unwrap()).collect();
    if k < x.len() {
        let (m0, m1) = (product(&x[k..])?, product(&y[k..])?);
        let (d0, d1) = (decompose(&m0)?, decompose(&m1)?);
        out.push(TransformFn::Matrix(recompose(&d0.lerp(&d1, p))));
    }
    Some(list_text(&out))
}

fn compose_transform(underlying: &str, value: &str, add: bool) -> Option<String> {
    let (x, y) = (
        t3::parse_transform(underlying).ok()?,
        t3::parse_transform(value).ok()?,
    );
    if add {
        return Some(list_text(&[x, y].concat()));
    }
    // Accumulation (CSS Transforms 2 §16): matching functions add (scales
    // one-based), the rest through their decomposed matrices.
    let (x, y, k) = pair_lists(&x, &y);
    let mut out = Vec::new();
    for i in 0..k {
        out.push(match (&x[i], &y[i]) {
            (TransformFn::Translate(x0, y0, z0), TransformFn::Translate(x1, y1, z1)) => {
                let s = |a: LenPct, b: LenPct| LenPct {
                    px: a.px + b.px,
                    frac: a.frac + b.frac,
                };
                TransformFn::Translate(s(*x0, *x1), s(*y0, *y1), z0 + z1)
            }
            (TransformFn::Scale(x0, y0, z0), TransformFn::Scale(x1, y1, z1)) => {
                TransformFn::Scale(x0 + x1 - 1.0, y0 + y1 - 1.0, z0 + z1 - 1.0)
            }
            (TransformFn::Rotate(a0, r0), TransformFn::Rotate(a1, r1)) => {
                TransformFn::Rotate(if *r0 == 0.0 { *a1 } else { *a0 }, r0 + r1)
            }
            (TransformFn::Skew(x0, y0), TransformFn::Skew(x1, y1)) => {
                TransformFn::Skew(x0 + x1, y0 + y1)
            }
            (a, b) => lerp_fn(a, b, 1.0)?,
        });
    }
    if k < x.len() {
        let (d0, d1) = (
            decompose(&product(&x[k..])?)?,
            decompose(&product(&y[k..])?)?,
        );
        out.push(TransformFn::Matrix(recompose(&d0.accumulate(&d1))));
    }
    Some(list_text(&out))
}

/// A decomposed 3D matrix (CSS Transforms 2 §12.1).
#[derive(Clone, Copy, Debug)]
struct Decomposed {
    translate: [f32; 3],
    scale: [f32; 3],
    skew: [f32; 3],
    perspective: [f32; 4],
    quaternion: [f32; 4],
}

impl Decomposed {
    fn lerp(&self, o: &Decomposed, p: f32) -> Decomposed {
        let l3 = |a: [f32; 3], b: [f32; 3]| std::array::from_fn(|i| lerp(a[i], b[i], p));
        Decomposed {
            translate: l3(self.translate, o.translate),
            scale: l3(self.scale, o.scale),
            skew: l3(self.skew, o.skew),
            perspective: std::array::from_fn(|i| lerp(self.perspective[i], o.perspective[i], p)),
            quaternion: slerp(self.quaternion, o.quaternion, p),
        }
    }

    fn accumulate(&self, o: &Decomposed) -> Decomposed {
        let s3 = |a: [f32; 3], b: [f32; 3]| std::array::from_fn(|i| a[i] + b[i]);
        let [x1, y1, z1, w1] = self.quaternion;
        let [x2, y2, z2, w2] = o.quaternion;
        Decomposed {
            translate: s3(self.translate, o.translate),
            scale: std::array::from_fn(|i| self.scale[i] + o.scale[i] - 1.0),
            skew: s3(self.skew, o.skew),
            perspective: std::array::from_fn(|i| {
                self.perspective[i] + o.perspective[i] - if i == 3 { 1.0 } else { 0.0 }
            }),
            quaternion: [
                w1 * x2 + x1 * w2 + y1 * z2 - z1 * y2,
                w1 * y2 - x1 * z2 + y1 * w2 + z1 * x2,
                w1 * z2 + x1 * y2 - y1 * x2 + z1 * w2,
                w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2,
            ],
        }
    }
}

fn slerp(a: [f32; 4], b: [f32; 4], p: f32) -> [f32; 4] {
    let mut dot: f32 = (0..4).map(|i| a[i] * b[i]).sum();
    let mut b = b;
    if dot < 0.0 {
        // The shorter way round.
        b = b.map(|v| -v);
        dot = -dot;
    }
    let dot = dot.min(1.0);
    if dot > 0.9995 {
        let q: [f32; 4] = std::array::from_fn(|i| lerp(a[i], b[i], p));
        let n = q.iter().map(|v| v * v).sum::<f32>().sqrt();
        return q.map(|v| v / n);
    }
    let theta = dot.acos();
    let sin = theta.sin();
    let wa = ((1.0 - p) * theta).sin() / sin;
    let wb = (p * theta).sin() / sin;
    std::array::from_fn(|i| a[i] * wa + b[i] * wb)
}

fn det3(m: [[f32; 3]; 3]) -> f32 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

fn invert(m: &Mat4) -> Option<Mat4> {
    // Cofactor expansion.
    let minor = |r: usize, c: usize| {
        let mut s = [[0.0; 3]; 3];
        let rows = (0..4).filter(|&i| i != r);
        for (i, row) in rows.enumerate() {
            for (j, col) in (0..4).filter(|&j| j != c).enumerate() {
                s[i][j] = m[row][col];
            }
        }
        det3(s)
    };
    let det: f32 = (0..4)
        .map(|c| m[0][c] * minor(0, c) * if c % 2 == 0 { 1.0 } else { -1.0 })
        .sum();
    if det.abs() < 1e-12 {
        return None;
    }
    let mut out = [[0.0; 4]; 4];
    for (r, row) in out.iter_mut().enumerate() {
        for (c, v) in row.iter_mut().enumerate() {
            let sign = if (r + c) % 2 == 0 { 1.0 } else { -1.0 };
            *v = sign * minor(c, r) / det;
        }
    }
    Some(out)
}

/// The spec's unmatrix, written over `s[column][row]`.
fn decompose(m: &Mat4) -> Option<Decomposed> {
    if m[3][3] == 0.0 {
        return None;
    }
    // s[i][j]: column i, row j, normalised.
    let s: Mat4 = std::array::from_fn(|i| std::array::from_fn(|j| m[j][i] / m[3][3]));
    let mut pm = s;
    for row in pm.iter_mut().take(3) {
        row[3] = 0.0;
    }
    pm[3][3] = 1.0;
    if det3(std::array::from_fn(|i| std::array::from_fn(|j| pm[i][j]))) == 0.0 {
        return None;
    }
    let perspective = if s[0][3] != 0.0 || s[1][3] != 0.0 || s[2][3] != 0.0 {
        let rhs = [s[0][3], s[1][3], s[2][3], s[3][3]];
        // rhs × transpose(inverse(pm)), in s's column-major terms.
        let inv = invert(&transpose(&pm))?;
        let inv_t = transpose(&inv);
        std::array::from_fn(|j| (0..4).map(|i| rhs[i] * inv_t[j][i]).sum())
    } else {
        [0.0, 0.0, 0.0, 1.0]
    };
    let translate = [s[3][0], s[3][1], s[3][2]];
    let mut row: [[f32; 3]; 3] = std::array::from_fn(|i| [s[i][0], s[i][1], s[i][2]]);
    let length = |v: [f32; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let combine = |a: [f32; 3], b: [f32; 3], x: f32, y: f32| -> [f32; 3] {
        std::array::from_fn(|i| a[i] * x + b[i] * y)
    };
    let mut scale = [0.0; 3];
    let mut skew = [0.0; 3];
    scale[0] = length(row[0]);
    if scale[0] == 0.0 {
        return None;
    }
    row[0] = row[0].map(|v| v / scale[0]);
    skew[0] = dot(row[0], row[1]);
    row[1] = combine(row[1], row[0], 1.0, -skew[0]);
    scale[1] = length(row[1]);
    if scale[1] == 0.0 {
        return None;
    }
    row[1] = row[1].map(|v| v / scale[1]);
    skew[0] /= scale[1];
    skew[1] = dot(row[0], row[2]);
    row[2] = combine(row[2], row[0], 1.0, -skew[1]);
    skew[2] = dot(row[1], row[2]);
    row[2] = combine(row[2], row[1], 1.0, -skew[2]);
    scale[2] = length(row[2]);
    if scale[2] == 0.0 {
        return None;
    }
    row[2] = row[2].map(|v| v / scale[2]);
    skew[1] /= scale[2];
    skew[2] /= scale[2];
    let cross = |a: [f32; 3], b: [f32; 3]| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    if dot(row[0], cross(row[1], row[2])) < 0.0 {
        for i in 0..3 {
            scale[i] = -scale[i];
            row[i] = row[i].map(|v| -v);
        }
    }
    let q = |v: f32| v.max(0.0).sqrt() * 0.5;
    let mut quaternion = [
        q(1.0 + row[0][0] - row[1][1] - row[2][2]),
        q(1.0 - row[0][0] + row[1][1] - row[2][2]),
        q(1.0 - row[0][0] - row[1][1] + row[2][2]),
        q(1.0 + row[0][0] + row[1][1] + row[2][2]),
    ];
    if row[2][1] < row[1][2] {
        quaternion[0] = -quaternion[0];
    }
    if row[0][2] < row[2][0] {
        quaternion[1] = -quaternion[1];
    }
    if row[1][0] < row[0][1] {
        quaternion[2] = -quaternion[2];
    }
    Some(Decomposed {
        translate,
        scale,
        skew,
        perspective,
        quaternion,
    })
}

fn transpose(m: &Mat4) -> Mat4 {
    std::array::from_fn(|i| std::array::from_fn(|j| m[j][i]))
}

/// The spec's recompose, back to `m[row][column]`.
fn recompose(d: &Decomposed) -> Mat4 {
    // Built over `s[column][row]`, post-multiplied as the spec does.
    let mul = |a: &Mat4, b: &Mat4| -> Mat4 {
        std::array::from_fn(|i| std::array::from_fn(|j| (0..4).map(|k| b[i][k] * a[k][j]).sum()))
    };
    let mut s = t3::IDENTITY;
    for i in 0..4 {
        s[i][3] = d.perspective[i];
    }
    for i in 0..4 {
        for j in 0..3 {
            s[3][i] += d.translate[j] * s[j][i];
        }
    }
    let [x, y, z, w] = d.quaternion;
    let rotation: Mat4 = [
        [
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y - z * w),
            2.0 * (x * z + y * w),
            0.0,
        ],
        [
            2.0 * (x * y + z * w),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z - x * w),
            0.0,
        ],
        [
            2.0 * (x * z - y * w),
            2.0 * (y * z + x * w),
            1.0 - 2.0 * (x * x + y * y),
            0.0,
        ],
        [0.0, 0.0, 0.0, 1.0],
    ];
    s = mul(&s, &rotation);
    let mut temp = t3::IDENTITY;
    if d.skew[2] != 0.0 {
        temp[2][1] = d.skew[2];
        s = mul(&s, &temp);
    }
    if d.skew[1] != 0.0 {
        temp[2][1] = 0.0;
        temp[2][0] = d.skew[1];
        s = mul(&s, &temp);
    }
    if d.skew[0] != 0.0 {
        temp[2][0] = 0.0;
        temp[1][0] = d.skew[0];
        s = mul(&s, &temp);
    }
    for (row, k) in s.iter_mut().zip(d.scale) {
        for v in row.iter_mut() {
            *v *= k;
        }
    }
    transpose(&s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_interpolate_by_type() {
        let at = |p: &str, a: &str, b: &str, k: f32| mix(p, a, b, k);
        // Lengths, mixed units, integers, angles in any unit.
        assert_eq!(at("width", "10px", "30px", 0.25), "15px");
        assert_eq!(
            at("width", "10px", "50%", 0.25),
            "calc(10px * 0.75 + 50% * 0.25)"
        );
        assert_eq!(at("z-index", "1", "4", 0.4), "2");
        assert_eq!(at("offset-rotate", "0.5turn", "200grad", 0.5), "180deg");
        // Colours mix in sRGB when both are legacy, in Oklab otherwise.
        assert_eq!(
            at("background-color", "red", "#00f", 0.3),
            "color-mix(in srgb, red, #00f 30%)"
        );
        assert_eq!(
            at("color", "oklch(0.7 0.1 30)", "red", 0.5),
            "color-mix(in oklab, oklch(0.7 0.1 30), red 50%)"
        );
        // Gradients with the same stops, component by component.
        assert_eq!(
            at(
                "background-image",
                "linear-gradient(0deg, red 0%, blue 100%)",
                "linear-gradient(90deg, red 20%, blue 100%)",
                0.5
            ),
            "linear-gradient(45deg, red 10%, blue 100%)"
        );
        // Unlike keywords flip at 50%; display keeps its shown value.
        assert_eq!(at("align-items", "start", "end", 0.49), "start");
        assert_eq!(at("align-items", "start", "end", 0.5), "end");
        assert_eq!(at("display", "none", "flex", 0.1), "flex");
        assert_eq!(at("display", "flex", "none", 0.9), "flex");
        assert_eq!(at("display", "flex", "none", 1.0), "none");
        assert_eq!(at("visibility", "hidden", "visible", 0.01), "visible");
    }

    #[test]
    fn shadows_and_filters_pad_and_pair() {
        assert_eq!(
            mix("box-shadow", "none", "inset 0 4px 8px red", 0.5),
            "inset 0px 2px 4px 0px color-mix(in srgb, transparent, red 50%)"
        );
        assert_eq!(
            mix("filter", "blur(4px)", "blur(8px) brightness(150%)", 0.5),
            "blur(6px) brightness(1.25)"
        );
        // Functions in another order are discrete.
        assert_eq!(
            mix("filter", "blur(4px) sepia(1)", "sepia(1) blur(4px)", 0.4),
            "blur(4px) sepia(1)"
        );
    }

    #[test]
    fn transforms_pair_primitives_and_fall_back_to_matrices() {
        assert_eq!(
            mix("transform", "none", "translateX(100px) rotate(90deg)", 0.5),
            "translate3d(50px, 0px, 0px) rotate3d(0, 0, 1, 45deg)"
        );
        // Rotations about different axes go through decomposition: halfway
        // between none-equivalent and a quarter turn about z is 45° about z.
        let m = mix(
            "transform",
            "skewX(0deg) scale(1)",
            "rotate(90deg) scale(2)",
            0.5,
        );
        assert!(m.starts_with("matrix3d("), "{m}");
        let parsed = t3::parse_transform(&m).unwrap();
        let mat = product(&parsed).unwrap();
        let d = decompose(&mat).unwrap();
        assert!((d.scale[0] - 1.5).abs() < 1e-3, "{m}");
        let angle = 2.0 * d.quaternion[3].acos();
        assert!((angle.to_degrees() - 45.0).abs() < 0.1, "{m}");
        // Decomposition recomposes the matrix it came from.
        let fns =
            t3::parse_transform("translate(10px, 20px) rotate(30deg) skewX(10deg) scale(2, 3)")
                .unwrap();
        let m = product(&fns).unwrap();
        let back = recompose(&decompose(&m).unwrap());
        for r in 0..4 {
            for c in 0..4 {
                assert!((m[r][c] - back[r][c]).abs() < 1e-3, "{m:?} vs {back:?}");
            }
        }
        // Accumulated scales are one-based; added lists concatenate.
        assert_eq!(
            compose("transform", "scale(2)", "scale(3)", Composite::Accumulate),
            "scale3d(4, 4, 1)"
        );
        assert_eq!(
            compose("transform", "scale(2)", "translateX(5px)", Composite::Add),
            "scale3d(2, 2, 1) translate3d(5px, 0px, 0px)"
        );
        assert_eq!(
            compose("width", "10px", "2em", Composite::Add),
            "calc(10px + 2em)"
        );
    }
}
