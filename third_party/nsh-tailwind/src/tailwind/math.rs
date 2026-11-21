//! CSS math in an arbitrary length (`w-[calc(2rem+4px)]`,
//! `size-[max(4rem,min(8.286rem,16vh))]`): `calc()`, `min()`, `max()` and
//! `clamp()` over `px`, `rem` and the viewport units, as CSS Values §10
//! has them. A class resolves against the window, so each value is a
//! length in px, the window's width and its height; one that reads
//! neither is a plain length when the class compiles. Percentages (of a
//! containing block the class cannot see) and `em` are not taken: the
//! class stays unknown.

use crate::css;

/// A length: a linear one, or the least or greatest of several.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Calc {
    /// `px` + `w` × the window's width + `h` × its height.
    Linear { px: f32, w: f32, h: f32 },
    Min(Box<[Calc]>),
    Max(Box<[Calc]>),
    /// Their sum, each times its factor.
    Sum(Box<[(f32, Calc)]>),
}

impl Calc {
    /// Its value in a `width` by `height` window, px.
    pub(crate) fn eval(&self, width: f32, height: f32) -> f32 {
        match self {
            Calc::Linear { px, w, h } => px + w * width + h * height,
            Calc::Min(v) => v.iter().map(|c| c.eval(width, height)).fold(f32::INFINITY, f32::min),
            Calc::Max(v) => v.iter().map(|c| c.eval(width, height)).fold(f32::NEG_INFINITY, f32::max),
            Calc::Sum(v) => v.iter().map(|(k, c)| k * c.eval(width, height)).sum(),
        }
    }

    /// Which of the window's sides it reads: (width, height).
    pub(crate) fn reads(&self) -> (bool, bool) {
        match self {
            Calc::Linear { w, h, .. } => (*w != 0.0, *h != 0.0),
            Calc::Min(v) | Calc::Max(v) => v.iter().fold((false, false), |a, c| or(a, c.reads())),
            Calc::Sum(v) => v.iter().fold((false, false), |a, (_, c)| or(a, c.reads())),
        }
    }

    /// Its value when it reads no side of the window.
    pub(crate) fn constant(&self) -> Option<f32> {
        (self.reads() == (false, false)).then(|| self.eval(0.0, 0.0))
    }
}

fn or(a: (bool, bool), b: (bool, bool)) -> (bool, bool) {
    (a.0 || b.0, a.1 || b.1)
}

/// `value` (an arbitrary value, its `_` already spaces) as CSS math, with
/// `rem` px to the rem: only a value that uses a math function or a
/// viewport unit, so a plain length keeps its own path.
pub(crate) fn parse(value: &str, rem: f32) -> Option<Calc> {
    let value = value.trim();
    let math = ["calc(", "min(", "max(", "clamp("].iter().any(|f| value.starts_with(f));
    if !math && !value.ends_with("vw") && !value.ends_with("vh") && !value.ends_with("vmin") && !value.ends_with("vmax") {
        return None;
    }
    let mut p = Parser { s: value.as_bytes(), at: 0, rem };
    let (calc, number) = p.sum()?;
    p.space();
    (p.at == p.s.len() && !number).then_some(calc)
}

struct Parser<'a> {
    s: &'a [u8],
    at: usize,
    rem: f32,
}

impl Parser<'_> {
    fn space(&mut self) {
        while self.s.get(self.at).is_some_and(u8::is_ascii_whitespace) {
            self.at += 1;
        }
    }

    fn eat(&mut self, c: u8) -> bool {
        self.space();
        if self.s.get(self.at) == Some(&c) {
            self.at += 1;
            true
        } else {
            false
        }
    }

    /// A sum of products; the flag says it is a bare number, not a length.
    fn sum(&mut self) -> Option<(Calc, bool)> {
        let (first, number) = self.product()?;
        let mut terms = vec![(1.0, first)];
        loop {
            let sign = if self.eat(b'+') {
                1.0
            } else if self.eat(b'-') {
                -1.0
            } else {
                break;
            };
            let (t, n) = self.product()?;
            // A length and a number do not add.
            if n != number {
                return None;
            }
            terms.push((sign, t));
        }
        Some(match terms.len() {
            1 => (terms.pop()?.1, number),
            _ => (Calc::Sum(terms.into()), number),
        })
    }

    /// Lengths times and over numbers.
    fn product(&mut self) -> Option<(Calc, bool)> {
        let (mut calc, mut number) = self.unary()?;
        loop {
            if self.eat(b'*') {
                let (rhs, n) = self.unary()?;
                match (number, n) {
                    (_, true) => calc = scale(calc, rhs.constant()?),
                    (true, false) => calc = scale(rhs, calc.constant()?),
                    (false, false) => return None,
                }
                number &= n;
            } else if self.eat(b'/') {
                let (rhs, n) = self.unary()?;
                let k = rhs.constant().filter(|k| n && *k != 0.0)?;
                calc = scale(calc, 1.0 / k);
            } else {
                return Some((calc, number));
            }
        }
    }

    fn unary(&mut self) -> Option<(Calc, bool)> {
        if self.eat(b'-') {
            let (c, n) = self.unary()?;
            return Some((scale(c, -1.0), n));
        }
        self.atom()
    }

    fn atom(&mut self) -> Option<(Calc, bool)> {
        self.space();
        if self.eat(b'(') {
            let v = self.sum()?;
            return self.eat(b')').then_some(v);
        }
        let start = self.at;
        while self.s.get(self.at).is_some_and(|c| c.is_ascii_alphabetic()) {
            self.at += 1;
        }
        let name = std::str::from_utf8(&self.s[start..self.at]).ok()?;
        if !name.is_empty() {
            if !self.eat(b'(') {
                return None;
            }
            let mut args = vec![self.sum()?];
            while self.eat(b',') {
                args.push(self.sum()?);
            }
            if !self.eat(b')') {
                return None;
            }
            let number = args[0].1;
            if args.iter().any(|a| a.1 != number) {
                return None;
            }
            let mut v: Vec<Calc> = args.into_iter().map(|a| a.0).collect();
            let calc = match (name, v.len()) {
                ("calc", 1) => v.pop()?,
                ("min", _) => Calc::Min(v.into()),
                ("max", _) => Calc::Max(v.into()),
                // clamp(lo, x, hi) = max(lo, min(x, hi))
                ("clamp", 3) => {
                    let hi = v.pop()?;
                    let x = v.pop()?;
                    let lo = v.pop()?;
                    Calc::Max([lo, Calc::Min([x, hi].into())].into())
                }
                _ => return None,
            };
            return Some((calc, number));
        }
        // A number and its unit.
        let start = self.at;
        while self.s.get(self.at).is_some_and(|c| c.is_ascii_digit() || *c == b'.') {
            self.at += 1;
        }
        let n = css::number(std::str::from_utf8(&self.s[start..self.at]).ok()?)?;
        let unit_at = self.at;
        while self.s.get(self.at).is_some_and(|c| c.is_ascii_alphabetic()) {
            self.at += 1;
        }
        let lin = |px, w, h| Some((Calc::Linear { px, w, h }, false));
        match std::str::from_utf8(&self.s[unit_at..self.at]).ok()? {
            "" => Some((Calc::Linear { px: n, w: 0.0, h: 0.0 }, true)),
            "px" => lin(n, 0.0, 0.0),
            "rem" => lin(n * self.rem, 0.0, 0.0),
            "vw" | "dvw" | "svw" | "lvw" => lin(0.0, n / 100.0, 0.0),
            "vh" | "dvh" | "svh" | "lvh" => lin(0.0, 0.0, n / 100.0),
            "vmin" => Some((
                Calc::Min([Calc::Linear { px: 0.0, w: n / 100.0, h: 0.0 }, Calc::Linear { px: 0.0, w: 0.0, h: n / 100.0 }].into()),
                false,
            )),
            "vmax" => Some((
                Calc::Max([Calc::Linear { px: 0.0, w: n / 100.0, h: 0.0 }, Calc::Linear { px: 0.0, w: 0.0, h: n / 100.0 }].into()),
                false,
            )),
            _ => None,
        }
    }
}

/// `c` times `k`.
fn scale(c: Calc, k: f32) -> Calc {
    match c {
        Calc::Linear { px, w, h } => Calc::Linear { px: px * k, w: w * k, h: h * k },
        c => Calc::Sum([(k, c)].into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each against an 800 by 600 window at 14 px to the rem, its value
    /// worked out by hand; `None`: not CSS math this takes.
    #[test]
    fn css_math_evaluates_against_the_window() {
        for (v, want) in [
            ("max(4rem,min(8.286rem,16vh))", Some(96.0)),
            ("calc(2rem + 4px)", Some(32.0)),
            ("calc(100vw - 2 * 10px)", Some(780.0)),
            ("clamp(10px, 50vw, 300px)", Some(300.0)),
            ("min(10px,2rem)", Some(10.0)),
            ("50vh", Some(300.0)),
            ("10vmin", Some(60.0)),
            ("calc((1rem + 2px) / 2)", Some(8.0)),
            ("-min(5px, 1rem)", None),
            ("calc(50% - 1rem)", None),
            ("calc(1rem + 2)", None),
            ("max(1em, 2px)", None),
            ("calc(1rem", None),
            ("4rem", None),
        ] {
            let got = parse(v, 14.0).map(|c| c.eval(800.0, 600.0));
            assert_eq!(got.map(|g| (g * 1000.0).round() / 1000.0), want, "{v}");
        }
    }
}
