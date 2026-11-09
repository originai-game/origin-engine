//! CSS Values 4 math functions: `calc()`, `min()`, `max()` and `clamp()`,
//! reduced once their `var()`s are substituted (at cook for a constant
//! value, when the element resolves for one that reads a variable, such as
//! a bound `style:--fill`).
//!
//! As in browsers, a length reduces to a linear sum of its `%`, `px` and
//! `em` terms: `calc(100% - 12px)` stays `calc(100% - 12px)`, and layout
//! resolves the percentage against the basis it already has. Viewport,
//! root and font-relative units become px first ([`absolute`]), when the
//! element resolves. The other functions of CSS Values 4 §10 (stepped
//! values, trigonometry, exponents and signs) evaluate here; `min()`,
//! `max()`, `clamp()`, `round()`, `mod()`, `rem()`, `abs()` and `hypot()`
//! whose length arguments mix `%` with px stay functions for layout.

/// A sum of terms, each a coefficient of a canonical unit: lengths in
/// `%`, `px` and `em`, times in `ms`, angles in `deg`, `""` for a plain
/// number. Terms are kept in [`ORDER`] and are never empty.
#[derive(Clone, Debug, PartialEq)]
enum Dim {
    Sum(Vec<(&'static str, f64)>),
    /// A `min()`, `max()` or `clamp()` of length sums in more than one unit,
    /// left for layout.
    Fn(&'static str, Vec<Vec<(&'static str, f64)>>),
}

const UNITS: &[(&str, &str, f64)] = &[
    ("px", "px", 1.0),
    ("%", "%", 1.0),
    ("em", "em", 1.0),
    // CSS Values 4 §6.2: the physical units at 96px to the inch.
    ("cm", "px", 96.0 / 2.54),
    ("mm", "px", 96.0 / 25.4),
    ("Q", "px", 96.0 / 101.6),
    ("in", "px", 96.0),
    ("pt", "px", 96.0 / 72.0),
    ("pc", "px", 16.0),
    ("ms", "ms", 1.0),
    ("s", "ms", 1000.0),
    ("deg", "deg", 1.0),
    ("grad", "deg", 0.9),
    ("rad", "deg", 180.0 / std::f64::consts::PI),
    ("turn", "deg", 360.0),
    ("", "", 1.0),
];

/// What one physical unit is worth in px (`None`: not a physical unit).
pub(crate) fn physical(unit: &str) -> Option<f64> {
    UNITS
        .iter()
        .find(|(w, c, _)| *w == unit && *c == "px" && *w != "px")
        .map(|&(_, _, k)| k)
}

/// The order a sum's terms are written in.
const ORDER: [&str; 6] = ["%", "px", "em", "ms", "deg", ""];

/// The units resolved before reduction: the viewport's (with the small,
/// large and dynamic forms of CSS Values 4 §6.1.2, one viewport here), the
/// root's and the element's font's but `em` (which a margin keeps to the
/// element's resolve, [`keeps_em`]).
const ABSOLUTE: [&str; 41] = [
    "vw", "vh", "vi", "vb", "vmin", "vmax", "svw", "svh", "svi", "svb", "svmin", "svmax", "lvw",
    "lvh", "lvi", "lvb", "lvmin", "lvmax", "dvw", "dvh", "dvi", "dvb", "dvmin", "dvmax", "rem",
    "ch", "ex", "cap", "ic", "lh", "rlh", "rch", "rex", "rcap", "ric", "cqw", "cqh", "cqi", "cqb",
    "cqmin", "cqmax",
];

const FUNCTIONS: [&str; 21] = [
    "calc", "min", "max", "clamp", "round", "mod", "rem", "abs", "sign", "sin", "cos", "tan",
    "asin", "acos", "atan2", "atan", "pow", "sqrt", "hypot", "log", "exp",
];

/// Math functions over lengths whose result layout can still evaluate
/// when their arguments mix `%` with px (`Size::Calc`).
const LAYOUT_FUNCTIONS: [&str; 11] = [
    "min",
    "max",
    "clamp",
    "round",
    "round-up",
    "round-down",
    "round-to-zero",
    "mod",
    "rem",
    "abs",
    "hypot",
];

fn is_length(unit: &str) -> bool {
    matches!(unit, "%" | "px" | "em")
}

/// What the units [`absolute`] resolves are worth for one element.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Basis {
    /// The viewport's dynamic size (the surface's now), and its small and
    /// large ones (CSS Values 4 §6.1.2).
    pub width: f32,
    pub height: f32,
    pub small: [f32; 2],
    pub large: [f32; 2],
    /// The root font size (the body type step).
    pub rem: f32,
    /// The element's font size and line height (its type step's), px; 0:
    /// the root's.
    pub em: f32,
    pub lh: f32,
    /// The root's line height, px.
    pub rlh: f32,
    /// The body face's `0` advance, cap height and CJK water ideograph
    /// advance, and its x-height, each in em (CSS Values 4 §6.1.1).
    pub ch: f32,
    pub cap: f32,
    pub ic: f32,
    pub ex: f32,
    /// The safe area's insets (top, right, bottom, left), for `env()`.
    pub safe: [f32; 4],
    /// The nearest query container eligible on each axis (width, height)
    /// and its settled content size (0 until measured); none, and the
    /// container units fall back to the viewport's.
    pub containers: [Option<(u64, f32)>; 2],
}

impl Basis {
    /// The element's em and line height, the root's when it has none.
    fn em(&self) -> (f32, f32) {
        if self.em > 0.0 {
            (self.em, self.lh)
        } else {
            (self.rem, self.rlh)
        }
    }

    /// The font ratios a face lacking them takes (CSS Values 4 §6.1.1:
    /// 0.5em for `ch` and `ex`, the ascent for `cap`, 1em for `ic`).
    fn ratio(v: f32, fallback: f32) -> f32 {
        if v > 0.0 { v } else { fallback }
    }
}

/// Which viewport axes and query containers a value read through its units.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Read {
    pub width: bool,
    pub height: bool,
    pub containers: [Option<u64>; 2],
}

/// Whether `text` has a number in a unit [`absolute`] resolves, or an
/// `env()`.
pub(crate) fn has_absolute(text: &str) -> bool {
    text.contains("env(") || numbers(text).any(|(_, _, unit)| ABSOLUTE.contains(&unit))
}

/// A property that keeps its `em` terms to the element's resolve (a
/// margin's em reads the element's own type step there).
pub(crate) fn keeps_em(name: &str) -> bool {
    name.starts_with("margin")
}

/// Whether property `name`'s `text` resolves only when its element does:
/// it reads the viewport, the safe area, the element's font or an
/// attribute.
pub(crate) fn per_element(name: &str, text: &str) -> bool {
    has_absolute(text)
        || text.contains("attr(")
        || (!keeps_em(name) && numbers(text).any(|(_, _, unit)| unit == "em"))
}

/// `text` with every viewport, root and font-relative number written in px
/// (`em` too unless property `name` [`keeps_em`]), every `env()` replaced,
/// and the viewport axes it read.
pub(crate) fn absolute(text: &str, basis: Basis, name: &str) -> (String, Read) {
    let mut read = Read::default();
    let text = env(text, basis, &mut read);
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    let (em, lh) = basis.em();
    let root = basis.rem;
    let ch = Basis::ratio(basis.ch, 0.5);
    let ex = Basis::ratio(basis.ex, 0.5);
    let cap = Basis::ratio(basis.cap, 0.7);
    let ic = Basis::ratio(basis.ic, 1.0);
    for (start, end, unit) in numbers(&text) {
        // `s`, `l` and `d` read the small, large and dynamic viewport; a
        // bare `v` unit the dynamic one, as browsers' do (§6.1.2).
        let size = |v: [f32; 2]| {
            if v[0] > 0.0 && v[1] > 0.0 {
                v
            } else {
                [basis.width, basis.height]
            }
        };
        let (viewport, [w, h]) = match unit.split_at_checked(1) {
            Some(("s", u)) if u.starts_with('v') => (u, size(basis.small)),
            Some(("l", u)) if u.starts_with('v') => (u, size(basis.large)),
            Some(("d", u)) if u.starts_with('v') => (u, [basis.width, basis.height]),
            _ => (unit, [basis.width, basis.height]),
        };
        let (w, h) = (w as f64 / 100.0, h as f64 / 100.0);
        let per = match viewport {
            "vw" | "vi" => {
                read.width = true;
                w
            }
            "vh" | "vb" => {
                read.height = true;
                h
            }
            "vmin" | "vmax" => {
                read.width = true;
                read.height = true;
                if viewport == "vmin" {
                    w.min(h)
                } else {
                    w.max(h)
                }
            }
            "rem" => root as f64,
            "em" if !keeps_em(name) => em as f64,
            "lh" => lh as f64,
            "rlh" => basis.rlh as f64,
            "ch" => (ch * em) as f64,
            "ex" => (ex * em) as f64,
            "cap" => (cap * em) as f64,
            "ic" => (ic * em) as f64,
            "rch" => (ch * root) as f64,
            "rex" => (ex * root) as f64,
            "rcap" => (cap * root) as f64,
            "ric" => (ic * root) as f64,
            "cqw" | "cqi" | "cqh" | "cqb" | "cqmin" | "cqmax" => {
                let mut axis = |i: usize| match basis.containers[i] {
                    Some((key, size)) => {
                        read.containers[i] = Some(key);
                        size as f64 / 100.0
                    }
                    None => {
                        if i == 0 {
                            read.width = true;
                            w
                        } else {
                            read.height = true;
                            h
                        }
                    }
                };
                match unit {
                    "cqw" | "cqi" => axis(0),
                    "cqh" | "cqb" => axis(1),
                    "cqmin" => axis(0).min(axis(1)),
                    _ => axis(0).max(axis(1)),
                }
            }
            _ => continue,
        };
        let n: f64 = text[start..end - unit.len()].parse().unwrap_or(0.0);
        out.push_str(&text[last..start]);
        out.push_str(&number(n * per));
        out.push_str("px");
        last = end;
    }
    out.push_str(&text[last..]);
    (out, read)
}

/// `text` with each `env(<name>[, <fallback>])` replaced (CSS Environment
/// Variables): the safe-area insets as px (the `max` ones too, one safe
/// area here), any other name by its fallback, else nothing.
fn env(text: &str, basis: Basis, read: &mut Read) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("env(") {
        let boundary = at == 0 || {
            let p = rest.as_bytes()[at - 1];
            !(p.is_ascii_alphanumeric() || p == b'-')
        };
        let Some(close) = closing(rest, at + 3).filter(|_| boundary) else {
            out.push_str(&rest[..at + 4]);
            rest = &rest[at + 4..];
            continue;
        };
        out.push_str(&rest[..at]);
        let body = &rest[at + 4..close];
        let (name, fallback) = match split_top(body) {
            Some((n, f)) => (n.trim(), Some(f.trim())),
            None => (body.trim(), None),
        };
        let side = name
            .strip_prefix("safe-area-inset-")
            .or_else(|| name.strip_prefix("safe-area-max-inset-"));
        let value = match side {
            Some(s) => ["top", "right", "bottom", "left"]
                .iter()
                .position(|w| *w == s)
                .map(|k| {
                    read.width = true;
                    read.height = true;
                    format!("{}px", number(basis.safe[k] as f64))
                }),
            None => None,
        };
        out.push_str(&value.unwrap_or_else(|| fallback.unwrap_or("").to_string()));
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    out
}

/// `text` with each `attr(<name> [<type>][, <fallback>])` (CSS Values 5
/// §7.7) replaced by the element's attribute `name`: as a string with no
/// type (or `raw-string`), as written for `type(…)`, and a number with a
/// unit keyword (`px`, `%`, `deg`…) or `number`. An absent attribute, or
/// one that is no number where a number is asked, takes the fallback;
/// without one the declaration is invalid.
pub(crate) fn attr(text: &str, lookup: &dyn Fn(&str) -> Option<String>) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("attr(") {
        let boundary = at == 0 || {
            let p = rest.as_bytes()[at - 1];
            !(p.is_ascii_alphanumeric() || p == b'-')
        };
        let Some(close) = closing(rest, at + 4).filter(|_| boundary) else {
            out.push_str(&rest[..at + 5]);
            rest = &rest[at + 5..];
            continue;
        };
        out.push_str(&rest[..at]);
        let body = &rest[at + 5..close];
        let (head, fallback) = match split_top(body) {
            Some((h, f)) => (h.trim(), Some(f.trim())),
            None => (body.trim(), None),
        };
        let (name, ty) = match head.split_once(char::is_whitespace) {
            Some((n, t)) => (n, t.trim()),
            None => (head, ""),
        };
        let value = lookup(name).and_then(|v| {
            let v = v.trim().to_string();
            let numeric = || v.parse::<f64>().ok().map(number);
            match ty {
                "" | "raw-string" => Some(format!(
                    "\"{}\"",
                    v.replace('\\', "\\\\").replace('"', "\\\"")
                )),
                "number" => numeric(),
                t if t.starts_with("type(") => (!v.is_empty()).then_some(v.clone()),
                unit if UNITS.iter().any(|(w, ..)| w.eq_ignore_ascii_case(unit))
                    || ABSOLUTE.contains(&unit) =>
                {
                    numeric().map(|n| format!("{n}{unit}"))
                }
                _ => None,
            }
        });
        match value.or_else(|| fallback.map(str::to_string)) {
            Some(v) => out.push_str(&v),
            None => {
                return Err(format!(
                    "attr({head}) has no value: the element has no {name} attribute of that type, and no fallback"
                ));
            }
        }
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// `s` split at its first comma outside parentheses.
fn split_top(s: &str) -> Option<(&str, &str)> {
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => return Some((&s[..i], &s[i + 1..])),
            _ => {}
        }
    }
    None
}

/// Every number token in `text` outside strings and colours: its start, its
/// end and its unit.
fn numbers(text: &str) -> impl Iterator<Item = (usize, usize, &str)> {
    let b = text.as_bytes();
    let mut i = 0;
    let mut quote = None;
    std::iter::from_fn(move || {
        while i < b.len() {
            let c = b[i];
            if let Some(q) = quote {
                if c == q {
                    quote = None;
                }
                i += 1;
                continue;
            }
            if c == b'"' || c == b'\'' {
                quote = Some(c);
                i += 1;
                continue;
            }
            let prev = if i == 0 { b' ' } else { b[i - 1] };
            let starts =
                c.is_ascii_digit() || (c == b'.' && b.get(i + 1).is_some_and(u8::is_ascii_digit));
            if !starts || prev.is_ascii_alphanumeric() || prev == b'#' || prev == b'_' {
                i += 1;
                continue;
            }
            let start = if prev == b'-' || prev == b'+' {
                i - 1
            } else {
                i
            };
            while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'.') {
                i += 1;
            }
            let us = i;
            while i < b.len() && (b[i].is_ascii_alphabetic() || b[i] == b'%') {
                i += 1;
            }
            return Some((start, i, &text[us..i]));
        }
        None
    })
}

/// `text` with every math function reduced to its value (`calc(0.5 *
/// 100%)` → `50%`, `calc(100% - 2 * 6px)` → `calc(100% - 12px)`). Other
/// text is unchanged.
pub(crate) fn reduce(text: &str) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some((at, name)) = next_function(rest) {
        out.push_str(&rest[..at]);
        let open = at + name.len();
        let close = closing(rest, open).ok_or_else(|| format!("{name}( is not closed"))?;
        let body = &rest[open + 1..close];
        // `anchor()` and `anchor-size()` read other elements' boxes:
        // layout resolves their math (CSS Anchor Positioning 1 §4).
        if body.contains("anchor(") || body.contains("anchor-size(") {
            out.push_str(&rest[at..=close]);
            rest = &rest[close + 1..];
            continue;
        }
        let mut p = Parser {
            s: body.as_bytes(),
            i: 0,
        };
        let value = p.function(name)?;
        out.push_str(&format(&value));
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Whether `text` holds a math function.
pub(crate) fn has_math(text: &str) -> bool {
    next_function(text).is_some()
}

/// The first math function in `s`: where its name starts, and the name.
fn next_function(s: &str) -> Option<(usize, &'static str)> {
    let b = s.as_bytes();
    (0..b.len()).find_map(|i| {
        let boundary = i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'-');
        FUNCTIONS
            .iter()
            .find(|f| boundary && s[i..].starts_with(*f) && b.get(i + f.len()) == Some(&b'('))
            .map(|f| (i, *f))
    })
}

/// The index of the `)` closing the `(` at `open`.
fn closing(s: &str, open: usize) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in s.bytes().enumerate().skip(open) {
        match c {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

pub(crate) fn number(v: f64) -> String {
    let mut n = format!("{v:.6}");
    while n.ends_with('0') {
        n.pop();
    }
    if n.ends_with('.') {
        n.pop();
    }
    if n == "-0" {
        n = "0".into();
    }
    n
}

/// A sum as CSS: one term bare (`12px`), more in a `calc()`
/// (`calc(100% - 12px)`).
pub(crate) fn sum_text(terms: &[(&str, f64)]) -> String {
    match terms {
        [] => "0".into(),
        [(u, v)] => format!("{}{u}", number(*v)),
        _ => {
            let mut s = String::from("calc(");
            for (k, (u, v)) in terms.iter().enumerate() {
                match k {
                    0 => s.push_str(&format!("{}{u}", number(*v))),
                    _ if *v < 0.0 => s.push_str(&format!(" - {}{u}", number(-v))),
                    _ => s.push_str(&format!(" + {}{u}", number(*v))),
                }
            }
            s.push(')');
            s
        }
    }
}

fn format(d: &Dim) -> String {
    match d {
        Dim::Sum(terms) => sum_text(terms),
        Dim::Fn(name, args) => {
            let list = args
                .iter()
                .map(|a| sum_text(a))
                .collect::<Vec<_>>()
                .join(", ");
            match name.strip_prefix("round-") {
                Some(strategy) => format!("round({strategy}, {list})"),
                None => format!("{name}({list})"),
            }
        }
    }
}

/// `a + k·b`, term by term: units of one kind add, a bare 0 adds to any.
fn add(
    a: Vec<(&'static str, f64)>,
    b: Vec<(&'static str, f64)>,
    k: f64,
) -> Result<Vec<(&'static str, f64)>, String> {
    let zero = |t: &[(&str, f64)]| matches!(t, [("", v)] if *v == 0.0);
    if zero(&b) {
        return Ok(a);
    }
    if zero(&a) {
        return Ok(b.into_iter().map(|(u, v)| (u, k * v)).collect());
    }
    let kind = |t: &[(&'static str, f64)]| -> &'static str {
        if t.iter().all(|(u, _)| is_length(u)) {
            "length"
        } else {
            t[0].0
        }
    };
    let (ka, kb) = (kind(&a), kind(&b));
    if ka != kb {
        return Err(mixed(a[0].0, b[0].0));
    }
    let mut out = a;
    for (u, v) in b {
        match out.iter_mut().find(|(w, _)| *w == u) {
            Some((_, w)) => *w += k * v,
            None => out.push((u, k * v)),
        }
    }
    out.sort_by_key(|(u, _)| ORDER.iter().position(|o| o == u));
    // Keep one term for a sum that cancels out, so its unit survives.
    if out.len() > 1 {
        let first = out[0];
        out.retain(|(_, v)| *v != 0.0);
        if out.is_empty() {
            out.push((first.0, 0.0));
        }
    }
    Ok(out)
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn space(&mut self) {
        while self.s.get(self.i).is_some_and(u8::is_ascii_whitespace) {
            self.i += 1;
        }
    }

    fn eat(&mut self, c: u8) -> bool {
        self.space();
        if self.s.get(self.i) == Some(&c) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    /// A word at the cursor (`nearest`, `pi`), consumed.
    fn word(&mut self) -> Option<&str> {
        self.space();
        let start = self.i;
        while self
            .s
            .get(self.i)
            .is_some_and(|c| c.is_ascii_alphabetic() || *c == b'-')
        {
            self.i += 1;
        }
        if self.i == start || self.s.get(self.i) == Some(&b'(') {
            self.i = start;
            return None;
        }
        std::str::from_utf8(&self.s[start..self.i]).ok()
    }

    /// A function's arguments up to the end of its body.
    fn function(&mut self, name: &'static str) -> Result<Dim, String> {
        // `round()`'s rounding strategy (CSS Values 4 §10.3).
        let mut name = name;
        if name == "round" {
            let at = self.i;
            match self.word() {
                Some("nearest") => name = "round",
                Some("up") => name = "round-up",
                Some("down") => name = "round-down",
                Some("to-zero") => name = "round-to-zero",
                _ => self.i = at,
            }
            if self.i != at && !self.eat(b',') {
                return Err("round(<strategy>, A, B): a comma follows the strategy".into());
            }
        }
        let mut args = vec![self.sum()?];
        while self.eat(b',') {
            args.push(self.sum()?);
        }
        self.space();
        if self.i < self.s.len() {
            return Err(format!(
                "'{}' is not part of {name}()",
                String::from_utf8_lossy(&self.s[self.i..])
            ));
        }
        let (count, how) = match name {
            "calc" => (1..=1, "one expression"),
            "clamp" => (3..=3, "a minimum, a preferred value and a maximum"),
            "mod" | "rem" | "atan2" | "pow" => (2..=2, "two values"),
            n if n.starts_with("round") => (1..=2, "a value and its step"),
            "log" => (1..=2, "a value and its base"),
            "min" | "max" | "hypot" => (1..=usize::MAX, "one or more values"),
            _ => (1..=1, "one value"),
        };
        if !count.contains(&args.len()) {
            return Err(format!("{name}() takes {how}"));
        }
        if name == "calc" {
            return Ok(match args.remove(0) {
                Dim::Sum(t) => Dim::Sum(t.into_iter().map(|(u, v)| (u, censor(v))).collect()),
                d => d,
            });
        }
        let mut sums: Vec<Vec<(&'static str, f64)>> = args
            .into_iter()
            .map(|a| match a {
                Dim::Sum(t) => Ok(t),
                Dim::Fn(inner, _) => Err(nested(inner)),
            })
            .collect::<Result<_, _>>()?;
        // `round(A)` steps by 1 of A's unit.
        if name.starts_with("round") && sums.len() == 1 {
            let unit = sums[0][0].0;
            sums.push(vec![(unit, 1.0)]);
        }
        // One unit throughout: the value now.
        let unit = sums[0][0].0;
        if sums.iter().all(|t| t.len() == 1 && t[0].0 == unit) {
            let v: Vec<f64> = sums.iter().map(|t| t[0].1).collect();
            return evaluate(name, unit, &v).map(|(u, v)| Dim::Sum(vec![(u, censor(v))]));
        }
        if let Some(t) = sums.iter().find(|t| t.iter().any(|(u, _)| !is_length(u))) {
            let other = sums
                .iter()
                .find(|s| s[0].0 != t[0].0)
                .map_or("", |s| s[0].0);
            return Err(mixed(t[0].0, other));
        }
        if !LAYOUT_FUNCTIONS.contains(&name) {
            return Err(format!(
                "{name}() of lengths in more than one unit ({}) has no value before layout; give it lengths in one unit",
                sums.iter()
                    .map(|s| sum_text(s))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        Ok(Dim::Fn(name, sums))
    }

    fn sum(&mut self) -> Result<Dim, String> {
        let mut a = self.product()?;
        loop {
            // CSS needs spaces around + and -, so `-2px` stays a number.
            self.space();
            let op = match self.s.get(self.i) {
                Some(&c @ (b'+' | b'-'))
                    if self.s.get(self.i + 1).is_some_and(u8::is_ascii_whitespace) =>
                {
                    c
                }
                _ => return Ok(a),
            };
            self.i += 1;
            let b = self.product()?;
            a = Dim::Sum(add(
                terms(a)?,
                terms(b)?,
                if op == b'+' { 1.0 } else { -1.0 },
            )?);
        }
    }

    fn product(&mut self) -> Result<Dim, String> {
        let mut a = self.value()?;
        loop {
            if self.eat(b'*') {
                let b = self.value()?;
                let (x, y) = (terms(a)?, terms(b)?);
                let (k, t) = match (x.as_slice(), y.as_slice()) {
                    ([("", k)], _) => (*k, y),
                    (_, [("", k)]) => (*k, x),
                    _ => {
                        return Err(format!(
                            "calc() multiplies a {} by a {}; one side is a plain number",
                            x[0].0, y[0].0
                        ));
                    }
                };
                a = Dim::Sum(t.into_iter().map(|(u, v)| (u, k * v)).collect());
            } else if self.eat(b'/') {
                let b = terms(self.value()?)?;
                let k = match b.as_slice() {
                    [("", k)] => *k,
                    _ => {
                        return Err(format!(
                            "calc() divides by a {}; divide by a plain number",
                            b[0].0
                        ));
                    }
                };
                if k == 0.0 {
                    return Err("calc() divides by zero".into());
                }
                a = Dim::Sum(terms(a)?.into_iter().map(|(u, v)| (u, v / k)).collect());
            } else {
                return Ok(a);
            }
        }
    }

    fn value(&mut self) -> Result<Dim, String> {
        self.space();
        if self.eat(b'(') {
            let v = self.sum()?;
            if !self.eat(b')') {
                return Err("a '(' in calc() is not closed".into());
            }
            return Ok(v);
        }
        let rest = std::str::from_utf8(&self.s[self.i..]).unwrap_or("");
        if let Some((_, name)) = next_function(rest).filter(|(at, _)| *at == 0) {
            let open = self.i + name.len();
            let text = std::str::from_utf8(self.s).unwrap_or("");
            let close = closing(text, open).ok_or_else(|| format!("{name}( is not closed"))?;
            let mut inner = Parser {
                s: &self.s[open + 1..close],
                i: 0,
            };
            self.i = close + 1;
            return inner.function(name);
        }
        let at = self.i;
        let negative = self.s.get(self.i) == Some(&b'-');
        if negative {
            self.i += 1;
        }
        if let Some(w) = self.word() {
            // CSS Values 4 §10.7.1's constants.
            let v = match w.to_ascii_lowercase().as_str() {
                "e" => std::f64::consts::E,
                "pi" => std::f64::consts::PI,
                "infinity" => f64::INFINITY,
                "nan" => f64::NAN,
                _ => return Err(format!("calc() does not take '{w}'")),
            };
            return Ok(Dim::Sum(vec![("", if negative { -v } else { v })]));
        }
        self.i = at;
        let start = self.i;
        if matches!(self.s.get(self.i), Some(b'-' | b'+')) {
            self.i += 1;
        }
        while self
            .s
            .get(self.i)
            .is_some_and(|c| c.is_ascii_digit() || *c == b'.')
        {
            self.i += 1;
        }
        let number = std::str::from_utf8(&self.s[start..self.i]).unwrap_or("");
        let v: f64 = number.parse().map_err(|_| {
            format!(
                "'{}' is not a number in calc()",
                rest.chars().take(12).collect::<String>()
            )
        })?;
        let us = self.i;
        while self
            .s
            .get(self.i)
            .is_some_and(|c| c.is_ascii_alphabetic() || *c == b'%')
        {
            self.i += 1;
        }
        let written = std::str::from_utf8(&self.s[us..self.i]).unwrap_or("");
        // Units are ASCII case-insensitive (CSS Values 4 §6).
        let &(_, unit, k) = UNITS
            .iter()
            .find(|(w, ..)| w.eq_ignore_ascii_case(written))
            .ok_or_else(|| format!("calc() does not take {written}"))?;
        Ok(Dim::Sum(vec![(unit, v * k)]))
    }
}

/// A math function of arguments in one `unit`: its unit and its value
/// (CSS Values 4 §10.3 to §10.7).
fn evaluate(name: &str, unit: &'static str, v: &[f64]) -> Result<(&'static str, f64), String> {
    let number = |what: &str| -> Result<(), String> {
        if unit.is_empty() {
            Ok(())
        } else {
            Err(format!("{name}() takes {what}, not a {unit}"))
        }
    };
    let radians = |x: f64| -> Result<f64, String> {
        match unit {
            "" => Ok(x),
            "deg" => Ok(x.to_radians()),
            u => Err(format!("{name}() takes an angle or a number, not a {u}")),
        }
    };
    let (a, b) = (v[0], v.get(1).copied().unwrap_or(0.0));
    Ok(match name {
        "min" => (unit, v.iter().copied().fold(f64::INFINITY, f64::min)),
        "max" => (unit, v.iter().copied().fold(f64::NEG_INFINITY, f64::max)),
        "clamp" => (unit, v[1].min(v[2]).max(v[0])),
        "round" | "round-up" | "round-down" | "round-to-zero" => (unit, round(name, a, b)),
        "mod" => (unit, a - b * (a / b).floor()),
        "rem" => (unit, a - b * (a / b).trunc()),
        "abs" => (unit, a.abs()),
        "sign" => ("", if a == 0.0 { a } else { a.signum() }),
        "hypot" => (unit, v.iter().map(|x| x * x).sum::<f64>().sqrt()),
        "sin" => ("", radians(a)?.sin()),
        "cos" => ("", radians(a)?.cos()),
        "tan" => ("", radians(a)?.tan()),
        "asin" | "acos" | "atan" => {
            number("a number")?;
            let r = match name {
                "asin" => a.asin(),
                "acos" => a.acos(),
                _ => a.atan(),
            };
            ("deg", r.to_degrees())
        }
        "atan2" => ("deg", a.atan2(b).to_degrees()),
        "pow" => {
            number("numbers")?;
            ("", a.powf(b))
        }
        "sqrt" => {
            number("a number")?;
            ("", a.sqrt())
        }
        "exp" => {
            number("a number")?;
            ("", a.exp())
        }
        "log" => {
            number("numbers")?;
            (
                "",
                if v.len() == 2 {
                    a.ln() / b.ln()
                } else {
                    a.ln()
                },
            )
        }
        _ => return Err(format!("{name}() is not a math function")),
    })
}

/// `round()`'s nearest multiple of `b` to `a` by its strategy: the
/// nearest (a tie upwards), up, down, or towards zero; a zero step gives
/// NaN, as CSS's does.
pub(crate) fn round(strategy: &str, a: f64, b: f64) -> f64 {
    let b = b.abs();
    if b == 0.0 {
        return f64::NAN;
    }
    let q = a / b;
    b * match strategy {
        "round-up" => q.ceil(),
        "round-down" => q.floor(),
        "round-to-zero" => q.trunc(),
        _ => (q + 0.5).floor(),
    }
}

/// A top-level result as CSS censors it (CSS Values 4 §10.9): NaN is 0
/// and an infinity the largest finite value.
fn censor(v: f64) -> f64 {
    if v.is_nan() {
        0.0
    } else {
        v.clamp(-(f32::MAX as f64), f32::MAX as f64)
    }
}

/// A value's terms; a `min()`, `max()` or `clamp()` left for layout takes
/// part in no arithmetic.
fn terms(d: Dim) -> Result<Vec<(&'static str, f64)>, String> {
    match d {
        Dim::Sum(t) => Ok(t),
        Dim::Fn(name, _) => Err(nested(name)),
    }
}

fn nested(name: &str) -> String {
    format!(
        "this {name}() mixes units, so layout evaluates it; it stands alone as a value, not inside other math"
    )
}

fn mixed(a: &str, b: &str) -> String {
    let name = |u: &str| {
        if u.is_empty() {
            "a plain number".to_string()
        } else {
            u.to_string()
        }
    };
    format!(
        "calc() mixes {} and {}; only lengths (%, px, em, rem and viewport units) add to each other",
        name(a),
        name(b)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn math_functions_reduce_to_one_value() {
        for (text, want) in [
            ("calc(0.25 * 100%)", "25%"),
            ("calc(3 * 80ms)", "240ms"),
            ("calc(0.5s + 20ms)", "520ms"),
            ("calc((1 + 2) * 4px)", "12px"),
            ("calc(10px - 2 * 3px)", "4px"),
            ("calc(10px - -3px)", "13px"),
            ("calc(100px / 4)", "25px"),
            ("min(10px, 4px, 7px)", "4px"),
            ("max(1, 2)", "2"),
            ("clamp(0%, 130%, 100%)", "100%"),
            ("clamp(0%, calc(-0.2 * 100%), 100%)", "0%"),
            ("calc(0.5turn)", "180deg"),
            ("translate(calc(2 * 3px), 4px)", "translate(6px, 4px)"),
            ("12px minmax(80px, 1fr)", "12px minmax(80px, 1fr)"),
        ] {
            assert_eq!(reduce(text).as_deref(), Ok(want), "{text}");
        }
    }

    #[test]
    fn lengths_in_several_units_reduce_to_a_linear_sum() {
        for (text, want) in [
            ("calc(100% - 12px)", "calc(100% - 12px)"),
            ("calc(12px + 100%)", "calc(100% + 12px)"),
            ("calc((100% - 2 * 8px) / 2)", "calc(50% - 8px)"),
            ("calc(1em + 50% - 4px + 1em)", "calc(50% - 4px + 2em)"),
            ("calc(100% - 12px + 12px)", "100%"),
            ("calc(50% - 50%)", "0%"),
            ("min(100%, 600px)", "min(100%, 600px)"),
            (
                "clamp(200px, calc(50% + 0px), 40em)",
                "clamp(200px, 50%, 40em)",
            ),
            (
                "max(calc(100% - 2 * 10px), 3em)",
                "max(calc(100% - 20px), 3em)",
            ),
            // Reducing a reduced value changes nothing.
            ("calc(50% - 8px)", "calc(50% - 8px)"),
        ] {
            assert_eq!(reduce(text).as_deref(), Ok(want), "{text}");
        }
        for bad in [
            "calc(2px * 3px)",
            "calc(4px / 2px)",
            "calc(1px / 0)",
            "clamp(1px, 2px)",
            "calc(1px+2px)",
            "calc(10px + 2)",
            "calc(10px + 20ms)",
            "min(10px, 20ms)",
            "calc(min(100%, 10px) + 2px)",
            "calc(2 * 3qx)",
        ] {
            assert!(reduce(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn viewport_units_and_rem_become_px_with_the_axes_they_read() {
        let basis = Basis {
            width: 1280.0,
            height: 720.0,
            rem: 16.0,
            ..Basis::default()
        };
        let (text, read) = absolute("calc(100vw - 2rem) 50vh", basis, "width");
        assert_eq!(text, "calc(1280px - 32px) 360px");
        assert_eq!(
            read,
            Read {
                width: true,
                height: true,
                ..Read::default()
            }
        );
        let (text, read) = absolute("-10vmin 1.5vmax #12vw0a 'a 3vw'", basis, "width");
        assert_eq!(text, "-72px 19.2px #12vw0a 'a 3vw'");
        assert_eq!(
            read,
            Read {
                width: true,
                height: true,
                ..Read::default()
            }
        );
        let (text, read) = absolute(
            "calc(50% - 1vw) 2em",
            Basis {
                width: 390.0,
                ..basis
            },
            "margin-left",
        );
        assert_eq!(text, "calc(50% - 3.9px) 2em");
        assert_eq!(
            read,
            Read {
                width: true,
                height: false,
                ..Read::default()
            }
        );
        assert!(has_absolute("1rem") && !has_absolute("1em 10px"));
        // The small, large and dynamic viewports are each their own (a
        // phone's browser bar shown, hidden, and as it is now); the
        // font-relative units read the element's step, a root one the
        // body's, and the face's ratios (0.5em for a face without them).
        let (text, _) = absolute(
            "100svh 100lvh 100dvh 100vh 10svmax 50lvw",
            Basis {
                height: 700.0,
                small: [1280.0, 600.0],
                large: [1300.0, 760.0],
                ..basis
            },
            "height",
        );
        assert_eq!(text, "600px 760px 700px 700px 128px 650px");
        let font = Basis {
            em: 20.0,
            lh: 30.0,
            rlh: 24.0,
            ch: 0.5,
            safe: [44.0, 0.0, 34.0, 0.0],
            ..basis
        };
        let (text, _) = absolute(
            "100dvh 50svw 10lvmax 2em 2ch 3ex 1lh 1rlh 2rch",
            font,
            "width",
        );
        assert_eq!(text, "720px 640px 128px 40px 20px 30px 30px 24px 16px");
        let (text, read) = absolute(
            "calc(env(safe-area-inset-top) + 8px) env(safe-area-inset-left, 3px) env(nope, 5px)",
            font,
            "padding",
        );
        assert_eq!(text, "calc(44px + 8px) 0px 5px");
        assert!(read.width && read.height);
        assert!(per_element("width", "2em") && !per_element("margin-top", "2em"));
        // Container units: an inline-size container answers the width axis
        // only; the height falls back to the viewport's.
        let (text, read) = absolute(
            "10cqw 10cqi 10cqh 10cqmin 10cqmax",
            Basis {
                containers: [Some((7, 300.0)), None],
                ..basis
            },
            "width",
        );
        assert_eq!(text, "30px 30px 72px 30px 72px");
        assert_eq!(
            read,
            Read {
                width: false,
                height: true,
                containers: [Some(7), None],
            }
        );
    }

    #[test]
    fn stepped_trigonometric_and_exponential_functions_evaluate() {
        for (text, want) in [
            ("round(7px, 5px)", "5px"),
            ("round(7.5px, 5px)", "10px"),
            ("round(-7.5px, 5px)", "-5px"),
            ("round(up, 6px, 5px)", "10px"),
            ("round(down, -6px, 5px)", "-10px"),
            ("round(to-zero, -6px, 5px)", "-5px"),
            ("round(2.5)", "3"),
            ("mod(-7px, 3px)", "2px"),
            ("rem(-7px, 3px)", "-1px"),
            ("mod(7, -3)", "-2"),
            ("abs(-4ms)", "4ms"),
            ("calc(sign(-3px) * 10px)", "-10px"),
            ("calc(10px * sin(90deg))", "10px"),
            ("cos(0.5turn)", "-1"),
            ("tan(45deg)", "1"),
            ("atan2(1px, 1px)", "45deg"),
            ("acos(0)", "90deg"),
            ("pow(2, 10)", "1024"),
            ("sqrt(16)", "4"),
            ("hypot(3px, 4px)", "5px"),
            ("log(e)", "1"),
            ("log(8, 2)", "3"),
            ("exp(0)", "1"),
            ("calc(2 * pi)", "6.283185"),
            ("calc(1in - 48pt)", "32px"),
            ("calc(2.54cm + 10mm + 4Q)", "137.574803px"),
            ("calc(6pc)", "96px"),
            ("calc(100grad + 0.25turn)", "180deg"),
            ("round(1px, 0px)", "0px"),
            (
                "calc(1px * infinity)",
                "340282346638528859811704183484516925440px",
            ),
            ("round(50%, 10px)", "round(50%, 10px)"),
            (
                "round(up, 50% - 3px, 8px)",
                "round(up, calc(50% - 3px), 8px)",
            ),
            ("mod(100%, 7px)", "mod(100%, 7px)"),
            ("abs(calc(50% - 100px))", "abs(calc(50% - 100px))"),
        ] {
            assert_eq!(reduce(text).as_deref(), Ok(want), "{text}");
        }
        for bad in [
            "sign(50% - 10px)",
            "sin(10px)",
            "pow(2px, 2)",
            "mod(1px)",
            "round(sideways, 1px, 2px)",
            "atan2(1px, 1deg)",
            "calc(2 * tau)",
        ] {
            assert!(reduce(bad).is_err(), "{bad}");
        }
    }
}
