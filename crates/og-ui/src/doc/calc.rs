//! CSS Values 4 math functions: `calc()`, `min()`, `max()` and `clamp()`,
//! reduced once their `var()`s are substituted (at cook for a constant
//! value, when the element resolves for one that reads a variable, such as
//! a bound `style:--fill`). og-ui resolves them before layout, so every
//! term must reduce to one unit: `calc(var(--fill) * 100%)` and
//! `calc(var(--i) * 80ms)` do, `calc(100% - 12px)` does not.

/// A number with its canonical unit: lengths in their own unit (`px`,
/// `%`, `em`), times in `ms`, angles in `deg`, `""` for a plain number.
#[derive(Clone, Debug, PartialEq)]
struct Dim {
    v: f64,
    unit: &'static str,
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

const FUNCTIONS: [&str; 4] = ["calc", "min", "max", "clamp"];

/// `text` with every math function reduced to its value (`calc(0.5 *
/// 100%)` → `50%`). Other text is unchanged.
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

fn format(d: &Dim) -> String {
    let mut n = format!("{:.6}", d.v);
    while n.ends_with('0') {
        n.pop();
    }
    if n.ends_with('.') {
        n.pop();
    }
    if n == "-0" {
        n = "0".into();
    }
    format!("{n}{}", d.unit)
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
    fn function(&mut self, name: &str) -> Result<Dim, String> {
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
        let unit = args[0].unit;
        if let Some(other) = args.iter().find(|a| a.unit != unit) {
            return Err(mixed(unit, other.unit));
        }
        let pick = |f: fn(f64, f64) -> f64| Dim {
            v: args.iter().map(|a| a.v).reduce(f).unwrap_or(0.0),
            unit,
        };
        match (name, args.len()) {
            ("calc", 1) => Ok(args.remove(0)),
            ("min", _) => Ok(pick(f64::min)),
            ("max", _) => Ok(pick(f64::max)),
            ("clamp", 3) => Ok(Dim {
                v: args[1].v.min(args[2].v).max(args[0].v),
                unit,
            }),
            _ => Err(format!(
                "{name}() takes {}",
                match name {
                    "calc" => "one expression",
                    _ => "a minimum, a preferred value and a maximum",
                }
            )),
        }
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
            // A bare 0 adds to any unit.
            let unit = match (a.unit, b.unit) {
                (x, y) if x == y => x,
                (x, "") if b.v == 0.0 => x,
                ("", y) if a.v == 0.0 => y,
                (x, y) => return Err(mixed(x, y)),
            };
            a = Dim {
                v: if op == b'+' { a.v + b.v } else { a.v - b.v },
                unit,
            };
        }
    }

    fn product(&mut self) -> Result<Dim, String> {
        let mut a = self.value()?;
        loop {
            if self.eat(b'*') {
                let b = self.value()?;
                a = match (a.unit, b.unit) {
                    ("", u) | (u, "") => Dim {
                        v: a.v * b.v,
                        unit: u,
                    },
                    (x, y) => {
                        return Err(format!(
                            "calc() multiplies a {x} by a {y}; one side is a plain number"
                        ));
                    }
                };
            } else if self.eat(b'/') {
                let b = self.value()?;
                if !b.unit.is_empty() {
                    return Err(format!(
                        "calc() divides by a {}; divide by a plain number",
                        b.unit
                    ));
                }
                if b.v == 0.0 {
                    return Err("calc() divides by zero".into());
                }
                a.v /= b.v;
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
        Ok(Dim { v: v * k, unit })
    }
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
        "calc() mixes {} and {}; og-ui reduces it before layout, so every term takes one unit",
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
        for bad in [
            "calc(100% - 12px)",
            "calc(2px * 3px)",
            "calc(4px / 2px)",
            "calc(1px / 0)",
            "clamp(1px, 2px)",
            "calc(1px+2px)",
            "calc(3 * 2vw)",
        ] {
            assert!(reduce(bad).is_err(), "{bad}");
        }
    }
}
