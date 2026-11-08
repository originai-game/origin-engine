//! CSS Values 4 math functions: `calc()`, `min()`, `max()` and `clamp()`,
//! reduced once their `var()`s are substituted (at cook for a constant
//! value, when the element resolves for one that reads a variable, such as
//! a bound `style:--fill`).
//!
//! As in browsers, a length reduces to a linear sum of its `%`, `px` and
//! `em` terms: `calc(100% - 12px)` stays `calc(100% - 12px)`, and layout
//! resolves the percentage against the basis it already has. Viewport units
//! and `rem` become px first ([`absolute`]), when the element resolves.
//! `min()`, `max()` and `clamp()` whose arguments mix units stay functions
//! for layout to evaluate; times, angles and plain numbers take one unit.

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
    ("ms", "ms", 1.0),
    ("s", "ms", 1000.0),
    ("deg", "deg", 1.0),
    ("rad", "deg", 180.0 / std::f64::consts::PI),
    ("turn", "deg", 360.0),
    ("", "", 1.0),
];

/// The order a sum's terms are written in.
const ORDER: [&str; 6] = ["%", "px", "em", "ms", "deg", ""];

/// The units resolved before reduction: the viewport's and the root's.
const ABSOLUTE: [&str; 5] = ["vw", "vh", "vmin", "vmax", "rem"];

const FUNCTIONS: [&str; 4] = ["calc", "min", "max", "clamp"];

fn is_length(unit: &str) -> bool {
    matches!(unit, "%" | "px" | "em")
}

/// What viewport units and `rem` are worth, in px.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Basis {
    pub width: f32,
    pub height: f32,
    /// The root font size (the body type step).
    pub rem: f32,
}

/// Which viewport axes a value read through its units.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Read {
    pub width: bool,
    pub height: bool,
}

/// Whether `text` has a number in a unit [`absolute`] resolves.
pub(crate) fn has_absolute(text: &str) -> bool {
    numbers(text).any(|(_, _, unit)| ABSOLUTE.contains(&unit))
}

/// `text` with every `vw`, `vh`, `vmin`, `vmax` and `rem` number written in
/// px, and the viewport axes it read.
pub(crate) fn absolute(text: &str, basis: Basis) -> (String, Read) {
    let mut read = Read::default();
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for (start, end, unit) in numbers(text) {
        let (w, h) = (basis.width as f64 / 100.0, basis.height as f64 / 100.0);
        let per = match unit {
            "vw" => {
                read.width = true;
                w
            }
            "vh" => {
                read.height = true;
                h
            }
            "vmin" | "vmax" => {
                read.width = true;
                read.height = true;
                if unit == "vmin" { w.min(h) } else { w.max(h) }
            }
            "rem" => basis.rem as f64,
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
        Dim::Fn(name, args) => format!(
            "{name}({})",
            args.iter()
                .map(|a| sum_text(a))
                .collect::<Vec<_>>()
                .join(", ")
        ),
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

    /// A function's arguments up to the end of its body.
    fn function(&mut self, name: &'static str) -> Result<Dim, String> {
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
        let count = match name {
            "calc" => 1..=1,
            "clamp" => 3..=3,
            _ => 1..=usize::MAX,
        };
        if !count.contains(&args.len()) {
            return Err(format!(
                "{name}() takes {}",
                match name {
                    "calc" => "one expression",
                    "clamp" => "a minimum, a preferred value and a maximum",
                    _ => "one or more values",
                }
            ));
        }
        if name == "calc" {
            return Ok(args.remove(0));
        }
        let sums: Vec<Vec<(&'static str, f64)>> = args
            .into_iter()
            .map(|a| match a {
                Dim::Sum(t) => Ok(t),
                Dim::Fn(inner, _) => Err(nested(inner)),
            })
            .collect::<Result<_, _>>()?;
        // One unit throughout: the value now.
        let unit = sums[0][0].0;
        if sums.iter().all(|t| t.len() == 1 && t[0].0 == unit) {
            let v: Vec<f64> = sums.iter().map(|t| t[0].1).collect();
            let picked = match name {
                "min" => v.iter().copied().reduce(f64::min),
                "max" => v.iter().copied().reduce(f64::max),
                _ => Some(v[1].min(v[2]).max(v[0])),
            };
            return Ok(Dim::Sum(vec![(unit, picked.unwrap_or(0.0))]));
        }
        if let Some(t) = sums.iter().find(|t| t.iter().any(|(u, _)| !is_length(u))) {
            let other = sums
                .iter()
                .find(|s| s[0].0 != t[0].0)
                .map_or("", |s| s[0].0);
            return Err(mixed(t[0].0, other));
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
        let &(_, unit, k) = UNITS
            .iter()
            .find(|(w, ..)| *w == written)
            .ok_or_else(|| format!("calc() does not take {written}"))?;
        Ok(Dim::Sum(vec![(unit, v * k)]))
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
            "calc(2 * 3q)",
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
        };
        let (text, read) = absolute("calc(100vw - 2rem) 50vh", basis);
        assert_eq!(text, "calc(1280px - 32px) 360px");
        assert_eq!(
            read,
            Read {
                width: true,
                height: true
            }
        );
        let (text, read) = absolute("-10vmin 1.5vmax #12vw0a 'a 3vw'", basis);
        assert_eq!(text, "-72px 19.2px #12vw0a 'a 3vw'");
        assert_eq!(
            read,
            Read {
                width: true,
                height: true
            }
        );
        let (text, read) = absolute(
            "calc(50% - 1vw) 2em",
            Basis {
                width: 390.0,
                ..basis
            },
        );
        assert_eq!(text, "calc(50% - 3.9px) 2em");
        assert_eq!(
            read,
            Read {
                width: true,
                height: false
            }
        );
        assert!(has_absolute("1rem") && !has_absolute("1em 10px"));
    }
}
