//! CSS colours as CSS Color Level 4 and 5 define them: hex, every named
//! colour, `rgb()`, `hsl()`, `hwb()`, `lab()`, `lch()`, `oklab()`,
//! `oklch()` and `color()` over every predefined space, `color-mix()`,
//! relative colours (`oklch(from var(--accent) l c calc(h + 180))`),
//! `light-dark()`, `currentColor` and `none` components.
//!
//! A colour that reads no token, no `currentColor` and no `light-dark()`
//! reduces to sRGB when it is parsed ([`Color::literal`]); one that does
//! stays an expression resolved against the active style when it is drawn
//! ([`Color::resolve`]). Out-of-gamut results are mapped into sRGB by CSS
//! Color 4's gamut mapping (OKLCh chroma reduction to within one JND).
//! Conversion matrices are CSS Color 4's sample code (W3C, permissive).

use crate::doc::css::{Tok, describe, tokens};
use crate::theme::Srgba;

/// A colour space a colour is written in or mixed in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Space {
    Srgb,
    SrgbLinear,
    DisplayP3,
    A98,
    ProPhoto,
    Rec2020,
    XyzD50,
    XyzD65,
    Lab,
    Lch,
    Oklab,
    Oklch,
    Hsl,
    Hwb,
}

impl Space {
    fn named(name: &str) -> Option<Space> {
        Some(match name {
            "srgb" => Space::Srgb,
            "srgb-linear" => Space::SrgbLinear,
            "display-p3" => Space::DisplayP3,
            "a98-rgb" => Space::A98,
            "prophoto-rgb" => Space::ProPhoto,
            "rec2020" => Space::Rec2020,
            "xyz-d50" => Space::XyzD50,
            "xyz" | "xyz-d65" => Space::XyzD65,
            "lab" => Space::Lab,
            "lch" => Space::Lch,
            "oklab" => Space::Oklab,
            "oklch" => Space::Oklch,
            "hsl" => Space::Hsl,
            "hwb" => Space::Hwb,
            _ => return None,
        })
    }

    /// The component holding a hue, if any.
    fn hue(self) -> Option<usize> {
        match self {
            Space::Hsl | Space::Hwb => Some(0),
            Space::Lch | Space::Oklch => Some(2),
            _ => None,
        }
    }

    fn rgb_like(self) -> bool {
        matches!(
            self,
            Space::Srgb
                | Space::SrgbLinear
                | Space::DisplayP3
                | Space::A98
                | Space::ProPhoto
                | Space::Rec2020
        )
    }
}

/// How `color-mix()` interpolates hues.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Hue {
    #[default]
    Shorter,
    Longer,
    Increasing,
    Decreasing,
}

/// A relative colour's channel: a number, a channel keyword of the origin
/// colour, `none`, or arithmetic over them (`calc()`, `min()`, `max()`,
/// `clamp()`).
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(f32),
    /// The origin's channel: 0 to 2, or 3 for `alpha`.
    Channel(usize),
    None,
    Op(char, Box<Expr>, Box<Expr>),
    Min(Vec<Expr>),
    Max(Vec<Expr>),
}

/// A parsed CSS colour.
#[derive(Clone, Debug, PartialEq)]
pub enum Color {
    /// Components in `space`'s units (see [`Space`]), `None` where `none`
    /// was written.
    Abs {
        space: Space,
        c: [Option<f32>; 3],
        alpha: Option<f32>,
    },
    /// `var(--role)`: a palette role resolved through the style.
    Token(String),
    Current,
    LightDark(Box<Color>, Box<Color>),
    Mix {
        space: Space,
        hue: Hue,
        a: Box<Color>,
        pa: Option<f32>,
        b: Box<Color>,
        pb: Option<f32>,
    },
    Relative {
        /// The function's space; `rgb()` keeps its 0–255 channels
        /// (`rgb255`).
        space: Space,
        rgb255: bool,
        from: Box<Color>,
        c: [Expr; 3],
        alpha: Option<Expr>,
    },
}

/// What a colour is resolved against.
#[derive(Clone, Copy, Debug)]
pub struct Context<'a> {
    pub theme: &'a crate::Theme,
    /// `currentColor` (the element's `color`).
    pub current: Srgba,
    /// The used colour scheme is dark (`light-dark()` takes its second).
    pub dark: bool,
}

impl<'a> Context<'a> {
    /// The style's own: its ink as `currentColor`, its scheme.
    pub fn of(theme: &'a crate::Theme) -> Context<'a> {
        Context {
            theme,
            current: theme.palette.ink,
            dark: theme.scheme == crate::Scheme::Dark,
        }
    }
}

type Abs = (Space, [Option<f32>; 3], Option<f32>);

impl Color {
    /// The colour as sRGB when it depends on nothing (no token,
    /// `currentColor` or `light-dark()`).
    pub fn literal(&self) -> Option<Srgba> {
        self.fixed().then(|| {
            let theme = crate::Theme::default();
            to_srgba(self.eval(&Context::of(&theme)))
        })
    }

    fn fixed(&self) -> bool {
        match self {
            Color::Abs { .. } => true,
            Color::Token(_) | Color::Current | Color::LightDark(..) => false,
            Color::Mix { a, b, .. } => a.fixed() && b.fixed(),
            Color::Relative { from, .. } => from.fixed(),
        }
    }

    /// Whether it reads `currentColor`.
    pub fn uses_current(&self) -> bool {
        match self {
            Color::Current => true,
            Color::Abs { .. } | Color::Token(_) => false,
            Color::LightDark(a, b) | Color::Mix { a, b, .. } => {
                a.uses_current() || b.uses_current()
            }
            Color::Relative { from, .. } => from.uses_current(),
        }
    }

    /// Its palette roles, for the cook's check.
    pub fn tokens(&self, out: &mut Vec<String>) {
        match self {
            Color::Token(t) => out.push(t.clone()),
            Color::Abs { .. } | Color::Current => {}
            Color::LightDark(a, b) | Color::Mix { a, b, .. } => {
                a.tokens(out);
                b.tokens(out);
            }
            Color::Relative { from, .. } => from.tokens(out),
        }
    }

    /// The sRGB colour it is in `cx`, gamut mapped.
    pub fn resolve(&self, cx: &Context) -> Srgba {
        to_srgba(self.eval(cx))
    }

    fn eval(&self, cx: &Context) -> Abs {
        match self {
            Color::Abs { space, c, alpha } => (*space, *c, *alpha),
            Color::Token(name) => srgb_abs(cx.theme.color(name).unwrap_or(Srgba::rgb(255, 0, 255))),
            Color::Current => srgb_abs(cx.current),
            Color::LightDark(l, d) => {
                if cx.dark {
                    d.eval(cx)
                } else {
                    l.eval(cx)
                }
            }
            Color::Mix {
                space,
                hue,
                a,
                pa,
                b,
                pb,
            } => mix(*space, *hue, a.eval(cx), *pa, b.eval(cx), *pb),
            Color::Relative {
                space,
                rgb255,
                from,
                c,
                alpha,
            } => {
                let (fs, fc, fa) = from.eval(cx);
                let (_, o, _) = convert((fs, fc, fa), *space);
                let scale = if *rgb255 { 255.0 } else { 1.0 };
                let chans = [
                    o[0].unwrap_or(0.0) * scale,
                    o[1].unwrap_or(0.0) * scale,
                    o[2].unwrap_or(0.0) * scale,
                    fa.unwrap_or(1.0),
                ];
                let ev = |e: &Expr| e.eval(&chans);
                let mut out = [ev(&c[0]), ev(&c[1]), ev(&c[2])];
                for v in out.iter_mut().flatten() {
                    *v /= scale;
                }
                if let Some(h) = space.hue() {
                    out[h] = out[h].map(|v| v.rem_euclid(360.0));
                }
                let a = match alpha {
                    Some(e) => ev(e).map(|v| v.clamp(0.0, 1.0)),
                    None => fa,
                };
                (*space, out, a)
            }
        }
    }
}

impl Expr {
    fn eval(&self, ch: &[f32; 4]) -> Option<f32> {
        Some(match self {
            Expr::Num(n) => *n,
            Expr::Channel(i) => ch[*i],
            Expr::None => return None,
            Expr::Op(op, a, b) => {
                let (a, b) = (a.eval(ch)?, b.eval(ch)?);
                match op {
                    '+' => a + b,
                    '-' => a - b,
                    '*' => a * b,
                    _ => a / b,
                }
            }
            Expr::Min(v) => v
                .iter()
                .map(|e| e.eval(ch))
                .collect::<Option<Vec<_>>>()?
                .into_iter()
                .fold(f32::INFINITY, f32::min),
            Expr::Max(v) => v
                .iter()
                .map(|e| e.eval(ch))
                .collect::<Option<Vec<_>>>()?
                .into_iter()
                .fold(f32::NEG_INFINITY, f32::max),
        })
    }
}

fn srgb_abs(c: Srgba) -> Abs {
    (
        Space::Srgb,
        c.rgb.map(|v| Some(f32::from(v) / 255.0)),
        Some(c.alpha),
    )
}

fn to_srgba(abs: Abs) -> Srgba {
    let (_, c, a) = convert(abs, Space::Srgb);
    let rgb = gamut_map(
        [
            c[0].unwrap_or(0.0),
            c[1].unwrap_or(0.0),
            c[2].unwrap_or(0.0),
        ],
        abs,
    );
    Srgba {
        rgb: rgb.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8),
        alpha: a.unwrap_or(1.0).clamp(0.0, 1.0),
    }
}

// ---- parsing

/// Parses one colour value (`#f80`, `oklch(70% 0.1 200)`, …).
pub fn parse_str(text: &str) -> Result<Color, String> {
    match tokens(text)?.as_slice() {
        [t] => parse(t),
        _ => Err(format!("'{}' is not a colour", text.trim())),
    }
}

/// Parses one colour token.
pub(crate) fn parse(t: &Tok) -> Result<Color, String> {
    match t {
        Tok::Hash(h) => hex(h)
            .ok_or_else(|| format!("'{h}' is not a colour (#rgb, #rgba, #rrggbb or #rrggbbaa)")),
        Tok::Ident(w) => {
            let lower = w.to_ascii_lowercase();
            match lower.as_str() {
                "currentcolor" => Ok(Color::Current),
                "transparent" => Ok(Color::Abs {
                    space: Space::Srgb,
                    c: [Some(0.0); 3],
                    alpha: Some(0.0),
                }),
                n => named(n)
                    .map(|rgb| srgb_color(rgb.map(|v| f32::from(v) / 255.0), 1.0))
                    .ok_or_else(|| format!("'{w}' is not a colour")),
            }
        }
        Tok::Func(f, args) => {
            let f = f.to_ascii_lowercase();
            match f.as_str() {
                "var" => match args.as_slice() {
                    [v] => match v.as_slice() {
                        [Tok::Ident(n)] if n.starts_with("--") => {
                            Ok(Color::Token(crate::doc::css::camel(&n[2..])))
                        }
                        _ => Err("var(--<role>)".into()),
                    },
                    _ => Err("a colour's var(--<role>) takes no fallback".into()),
                },
                "light-dark" => match args.as_slice() {
                    [a, b] => Ok(Color::LightDark(Box::new(one(a)?), Box::new(one(b)?))),
                    _ => Err("light-dark(<light>, <dark>)".into()),
                },
                "color-mix" => color_mix(args),
                "rgb" | "rgba" | "hsl" | "hsla" | "hwb" | "lab" | "lch" | "oklab" | "oklch"
                | "color" => function(&f, args),
                _ => Err(format!("{f}() is not a colour")),
            }
        }
        t => Err(format!("'{}' is not a colour", describe(t))),
    }
}

fn one(t: &[Tok]) -> Result<Color, String> {
    match t {
        [c] => parse(c),
        _ => Err("one colour".into()),
    }
}

fn srgb_color(c: [f32; 3], a: f32) -> Color {
    Color::Abs {
        space: Space::Srgb,
        c: c.map(Some),
        alpha: Some(a),
    }
}

fn hex(h: &str) -> Option<Color> {
    let c = crate::style_file::parse_color(h)?;
    Some(srgb_color(c.rgb.map(|v| f32::from(v) / 255.0), c.alpha))
}

/// A number of a component: its value with `percent` the value 100%
/// stands for. `hue` reads angles.
fn component(t: &Tok, percent: f32, hue: bool) -> Result<Option<f32>, String> {
    match t {
        Tok::Ident(w) if w.eq_ignore_ascii_case("none") => Ok(None),
        Tok::Num(n, u) => {
            let v: f32 = n.parse().map_err(|_| format!("'{n}' is not a number"))?;
            Ok(Some(match u.as_str() {
                "" => v,
                "%" if !hue => v / 100.0 * percent,
                "deg" if hue => v,
                "rad" if hue => v.to_degrees(),
                "grad" if hue => v * 0.9,
                "turn" if hue => v * 360.0,
                _ => return Err(format!("'{n}{u}' is not a colour component")),
            }))
        }
        Tok::Func(f, a) if f == "calc" => {
            let e = expr_of(a, &[], percent, hue)?;
            Ok(e.eval(&[0.0; 4]))
        }
        t => Err(format!("'{}' is not a colour component", describe(t))),
    }
}

fn alpha_of(t: &Tok) -> Result<Option<f32>, String> {
    component(t, 1.0, false).map(|v| v.map(|a| a.clamp(0.0, 1.0)))
}

/// The channel keywords, percentage references and hue component of a
/// function's space.
fn channels(f: &str, space: Space) -> ([&'static str; 3], [f32; 3]) {
    match f {
        "rgb" | "rgba" => (["r", "g", "b"], [255.0; 3]),
        "hsl" | "hsla" => (["h", "s", "l"], [0.0, 100.0, 100.0]),
        "hwb" => (["h", "w", "b"], [0.0, 100.0, 100.0]),
        "lab" => (["l", "a", "b"], [100.0, 125.0, 125.0]),
        "lch" => (["l", "c", "h"], [100.0, 150.0, 0.0]),
        "oklab" => (["l", "a", "b"], [1.0, 0.4, 0.4]),
        "oklch" => (["l", "c", "h"], [1.0, 0.4, 0.0]),
        _ if matches!(space, Space::XyzD50 | Space::XyzD65) => (["x", "y", "z"], [1.0; 3]),
        _ => (["r", "g", "b"], [1.0; 3]),
    }
}

fn function(f: &str, args: &[Vec<Tok>]) -> Result<Color, String> {
    let space = match f {
        "rgb" | "rgba" => Space::Srgb,
        "hsl" | "hsla" => Space::Hsl,
        "hwb" => Space::Hwb,
        "lab" => Space::Lab,
        "lch" => Space::Lch,
        "oklab" => Space::Oklab,
        "oklch" => Space::Oklch,
        _ => Space::Srgb,
    };
    // The legacy comma form: rgb(255, 0, 0[, 0.5]), hsl(120, 50%, 50%).
    if args.len() > 1 {
        if !matches!(f, "rgb" | "rgba" | "hsl" | "hsla") || !(3..=4).contains(&args.len()) {
            return Err(format!("{f}() takes its components separated by spaces"));
        }
        let t: Vec<&Tok> = args
            .iter()
            .map(|a| match a.as_slice() {
                [t] => Ok(t),
                _ => Err(format!("{f}(): one value between commas")),
            })
            .collect::<Result<_, _>>()?;
        if t.iter().any(|t| matches!(t, Tok::Ident(w) if w == "none")) {
            return Err(format!("{f}(): the comma form takes no none"));
        }
        return absolute(f, space, &t[..3], t.get(3).copied());
    }
    let toks = args.first().map(Vec::as_slice).unwrap_or_default();
    let (body, alpha) = match toks.iter().position(|t| *t == Tok::Slash) {
        Some(i) => match &toks[i + 1..] {
            [a] => (&toks[..i], Some(a)),
            _ => return Err(format!("{f}(): one alpha after /")),
        },
        None => (toks, None),
    };
    if let Some(Tok::Ident(w)) = body.first()
        && w == "from"
    {
        let from = Box::new(parse(body.get(1).ok_or(format!("{f}(from <colour> …)"))?)?);
        let (space, rest) = if f == "color" {
            match body.get(2) {
                Some(Tok::Ident(s)) => (
                    Space::named(s)
                        .filter(|s| s.rgb_like() || matches!(s, Space::XyzD50 | Space::XyzD65))
                        .ok_or(format!("color(): '{s}' is not a colour space"))?,
                    &body[3..],
                ),
                _ => return Err("color(from <colour> <space> …)".into()),
            }
        } else {
            (space, &body[2..])
        };
        let (names, pct) = channels(f, space);
        let kws: Vec<&str> = names.iter().copied().chain(["alpha"]).collect();
        let [a, b, c] = rest else {
            return Err(format!("{f}(from <colour> …) takes three channels"));
        };
        let hue = space.hue();
        let ch = |k: usize, t: &Tok| expr_one(t, &kws, pct[k], hue == Some(k));
        let alpha = alpha.map(|t| expr_one(t, &kws, 1.0, false)).transpose()?;
        return Ok(Color::Relative {
            space,
            rgb255: matches!(f, "rgb" | "rgba"),
            from,
            c: [ch(0, a)?, ch(1, b)?, ch(2, c)?],
            alpha,
        });
    }
    if f == "color" {
        let (s, rest) = match body.split_first() {
            Some((Tok::Ident(s), rest)) => (s, rest),
            _ => return Err("color(<space> <c1> <c2> <c3> [/ <alpha>])".into()),
        };
        let space = Space::named(s)
            .filter(|s| s.rgb_like() || matches!(s, Space::XyzD50 | Space::XyzD65))
            .ok_or(format!("color(): '{s}' is not a predefined colour space"))?;
        let [a, b, c] = rest else {
            return Err("color() takes three components".into());
        };
        return Ok(Color::Abs {
            space,
            c: [
                component(a, 1.0, false)?,
                component(b, 1.0, false)?,
                component(c, 1.0, false)?,
            ],
            alpha: match alpha {
                Some(a) => alpha_of(a)?,
                None => Some(1.0),
            },
        });
    }
    let t: Vec<&Tok> = body.iter().collect();
    if t.len() != 3 {
        return Err(format!("{f}() takes three components"));
    }
    absolute(f, space, &t, alpha)
}

fn absolute(f: &str, space: Space, t: &[&Tok], alpha: Option<&Tok>) -> Result<Color, String> {
    let (_, pct) = channels(f, space);
    let hue = space.hue();
    let mut c = [None; 3];
    for k in 0..3 {
        c[k] = component(t[k], pct[k], hue == Some(k))?;
    }
    if matches!(f, "rgb" | "rgba") {
        for v in c.iter_mut().flatten() {
            *v = (*v / 255.0).clamp(0.0, 1.0);
        }
    }
    if let Some(h) = hue {
        c[h] = c[h].map(|v| v.rem_euclid(360.0));
    }
    // Lightness clamps; chroma is never negative.
    match space {
        Space::Lab | Space::Lch => c[0] = c[0].map(|v| v.clamp(0.0, 100.0)),
        Space::Oklab | Space::Oklch => c[0] = c[0].map(|v| v.clamp(0.0, 1.0)),
        _ => {}
    }
    if matches!(space, Space::Lch | Space::Oklch) {
        c[1] = c[1].map(|v| v.max(0.0));
    }
    Ok(Color::Abs {
        space,
        c,
        alpha: match alpha {
            Some(a) => alpha_of(a)?,
            None => Some(1.0),
        },
    })
}

fn expr_one(t: &Tok, kws: &[&str], percent: f32, hue: bool) -> Result<Expr, String> {
    expr_of(&[vec![t.clone()]], kws, percent, hue)
}

/// A channel expression of comma-separated `args` (one for `calc()`).
fn expr_of(args: &[Vec<Tok>], kws: &[&str], percent: f32, hue: bool) -> Result<Expr, String> {
    let [toks] = args else {
        return Err("calc() takes one expression".into());
    };
    let mut p = ExprParser {
        t: toks,
        i: 0,
        kws,
        percent,
        hue,
    };
    let e = p.sum()?;
    if p.i != toks.len() {
        return Err(format!(
            "'{}' is not part of a channel",
            describe(&toks[p.i])
        ));
    }
    Ok(e)
}

struct ExprParser<'a> {
    t: &'a [Tok],
    i: usize,
    kws: &'a [&'a str],
    percent: f32,
    hue: bool,
}

impl ExprParser<'_> {
    fn sum(&mut self) -> Result<Expr, String> {
        let mut e = self.product()?;
        while let Some(Tok::Ident(op)) = self.t.get(self.i)
            && (op == "+" || op == "-")
        {
            let op = op.chars().next().unwrap_or('+');
            self.i += 1;
            e = Expr::Op(op, Box::new(e), Box::new(self.product()?));
        }
        Ok(e)
    }

    fn product(&mut self) -> Result<Expr, String> {
        let mut e = self.atom()?;
        loop {
            let op = match self.t.get(self.i) {
                Some(Tok::Ident(op)) if op == "*" => '*',
                Some(Tok::Slash) => '/',
                _ => return Ok(e),
            };
            self.i += 1;
            e = Expr::Op(op, Box::new(e), Box::new(self.atom()?));
        }
    }

    fn atom(&mut self) -> Result<Expr, String> {
        let t = self
            .t
            .get(self.i)
            .ok_or("a channel expression ends early")?;
        self.i += 1;
        Ok(match t {
            Tok::Ident(w) if w == "none" => Expr::None,
            Tok::Ident(w) => match self.kws.iter().position(|k| k == w) {
                Some(k) => Expr::Channel(k),
                None => return Err(format!("'{w}' is not a channel")),
            },
            Tok::Num(..) => match component(t, self.percent, self.hue)? {
                Some(v) => Expr::Num(v),
                None => Expr::None,
            },
            Tok::Func(f, a) => {
                let all = |a: &[Vec<Tok>]| -> Result<Vec<Expr>, String> {
                    a.iter()
                        .map(|x| expr_of(std::slice::from_ref(x), self.kws, self.percent, self.hue))
                        .collect()
                };
                match f.as_str() {
                    "calc" => expr_of(a, self.kws, self.percent, self.hue)?,
                    "min" => Expr::Min(all(a)?),
                    "max" => Expr::Max(all(a)?),
                    "clamp" => match all(a)?.as_slice() {
                        [lo, v, hi] => {
                            Expr::Max(vec![lo.clone(), Expr::Min(vec![v.clone(), hi.clone()])])
                        }
                        _ => return Err("clamp(<min>, <value>, <max>)".into()),
                    },
                    _ => return Err(format!("{f}() is not a channel expression")),
                }
            }
            t => return Err(format!("'{}' is not part of a channel", describe(t))),
        })
    }
}

fn color_mix(args: &[Vec<Tok>]) -> Result<Color, String> {
    const HOW: &str = "color-mix(in <space> [<hue> hue], <colour> [<n>%], <colour> [<n>%])";
    let [method, a, b] = args else {
        return Err(HOW.into());
    };
    let (space, hue) = match method.as_slice() {
        [Tok::Ident(i), Tok::Ident(s), rest @ ..] if i == "in" => {
            let space =
                Space::named(s).ok_or(format!("color-mix(): '{s}' is not a colour space"))?;
            let hue = match rest {
                [] => Hue::Shorter,
                [Tok::Ident(h), Tok::Ident(w)] if w == "hue" && space.hue().is_some() => {
                    match h.as_str() {
                        "shorter" => Hue::Shorter,
                        "longer" => Hue::Longer,
                        "increasing" => Hue::Increasing,
                        "decreasing" => Hue::Decreasing,
                        _ => return Err(HOW.into()),
                    }
                }
                _ => return Err(HOW.into()),
            };
            (space, hue)
        }
        _ => return Err(HOW.into()),
    };
    let part = |t: &[Tok]| -> Result<(Color, Option<f32>), String> {
        let pct = |t: &Tok| match t {
            Tok::Num(n, u) if u == "%" => n
                .parse::<f32>()
                .ok()
                .filter(|p| (0.0..=100.0).contains(p))
                .ok_or_else(|| format!("color-mix(): '{n}%' is not 0% to 100%")),
            t => Err(format!(
                "color-mix(): '{}' is not a percentage",
                describe(t)
            )),
        };
        match t {
            [c] => Ok((parse(c)?, None)),
            [c, p] if matches!(p, Tok::Num(..)) => Ok((parse(c)?, Some(pct(p)?))),
            [p, c] => Ok((parse(c)?, Some(pct(p)?))),
            _ => Err(HOW.into()),
        }
    };
    let (a, pa) = part(a)?;
    let (b, pb) = part(b)?;
    if pa == Some(0.0) && pb == Some(0.0) {
        return Err("color-mix(): the percentages sum to 0%".into());
    }
    Ok(Color::Mix {
        space,
        hue,
        a: Box::new(a),
        pa,
        b: Box::new(b),
        pb,
    })
}

// ---- mixing

fn mix(space: Space, hue: Hue, a: Abs, pa: Option<f32>, b: Abs, pb: Option<f32>) -> Abs {
    let (p1, p2) = match (pa, pb) {
        (None, None) => (50.0, 50.0),
        (Some(p), None) => (p, 100.0 - p),
        (None, Some(p)) => (100.0 - p, p),
        (Some(x), Some(y)) => (x, y),
    };
    let sum = p1 + p2;
    if sum <= 0.0 {
        return (space, [Some(0.0); 3], Some(0.0));
    }
    let mult = if sum < 100.0 { sum / 100.0 } else { 1.0 };
    let t = p2 / sum;
    let (_, mut c1, a1) = convert(a, space);
    let (_, mut c2, a2) = convert(b, space);
    for k in 0..3 {
        match (c1[k], c2[k]) {
            (None, Some(v)) => c1[k] = Some(v),
            (Some(v), None) => c2[k] = Some(v),
            _ => {}
        }
    }
    let alpha = match (a1, a2) {
        (None, None) => None,
        (x, y) => {
            let (x, y) = (x.or(y).unwrap_or(1.0), y.or(x).unwrap_or(1.0));
            Some(x + (y - x) * t)
        }
    };
    let (w1, w2) = (a1.or(a2).unwrap_or(1.0), a2.or(a1).unwrap_or(1.0));
    let mut out = [None; 3];
    for k in 0..3 {
        let (Some(x), Some(y)) = (c1[k], c2[k]) else {
            continue;
        };
        out[k] = Some(if space.hue() == Some(k) {
            let (x, y) = hue_pair(x, y, hue);
            (x + (y - x) * t).rem_euclid(360.0)
        } else {
            let premul = x * w1 + (y * w2 - x * w1) * t;
            match alpha {
                Some(al) if al > 0.0 => premul / al,
                _ => x + (y - x) * t,
            }
        });
    }
    (space, out, alpha.map(|a| a * mult))
}

fn hue_pair(mut x: f32, mut y: f32, hue: Hue) -> (f32, f32) {
    let d = y - x;
    match hue {
        Hue::Shorter => {
            if d > 180.0 {
                x += 360.0
            } else if d < -180.0 {
                y += 360.0
            }
        }
        Hue::Longer => {
            if 0.0 < d && d < 180.0 {
                x += 360.0
            } else if -180.0 < d && d <= 0.0 {
                y += 360.0
            }
        }
        Hue::Increasing => {
            if d < 0.0 {
                y += 360.0
            }
        }
        Hue::Decreasing => {
            if d > 0.0 {
                x += 360.0
            }
        }
    }
    (x, y)
}

// ---- conversion

type M = [[f64; 3]; 3];

fn mul(m: &M, v: [f64; 3]) -> [f64; 3] {
    [0, 1, 2].map(|r| m[r][0] * v[0] + m[r][1] * v[1] + m[r][2] * v[2])
}

const SRGB_XYZ: M = [
    [506752.0 / 1228815.0, 87881.0 / 245763.0, 12673.0 / 70218.0],
    [87098.0 / 409605.0, 175762.0 / 245763.0, 12673.0 / 175545.0],
    [7918.0 / 409605.0, 87881.0 / 737289.0, 1001167.0 / 1053270.0],
];
const XYZ_SRGB: M = [
    [12831.0 / 3959.0, -329.0 / 214.0, -1974.0 / 3959.0],
    [
        -851781.0 / 878810.0,
        1648619.0 / 878810.0,
        36519.0 / 878810.0,
    ],
    [705.0 / 12673.0, -2585.0 / 12673.0, 705.0 / 667.0],
];
const P3_XYZ: M = [
    [
        608311.0 / 1250200.0,
        189793.0 / 714400.0,
        198249.0 / 1000160.0,
    ],
    [
        35783.0 / 156275.0,
        247089.0 / 357200.0,
        198249.0 / 2500400.0,
    ],
    [0.0, 32229.0 / 714400.0, 5220557.0 / 5000800.0],
];
const XYZ_P3: M = [
    [
        446124.0 / 178915.0,
        -333277.0 / 357830.0,
        -72051.0 / 178915.0,
    ],
    [-14852.0 / 17905.0, 63121.0 / 35810.0, 423.0 / 17905.0],
    [11844.0 / 330415.0, -50337.0 / 660830.0, 316169.0 / 330415.0],
];
const A98_XYZ: M = [
    [
        573536.0 / 994567.0,
        263643.0 / 1420810.0,
        187206.0 / 994567.0,
    ],
    [
        591459.0 / 1989134.0,
        6239551.0 / 9945670.0,
        374412.0 / 4972835.0,
    ],
    [
        53769.0 / 1989134.0,
        351524.0 / 4972835.0,
        4929758.0 / 4972835.0,
    ],
];
const XYZ_A98: M = [
    [
        1829569.0 / 896150.0,
        -506331.0 / 2688450.0,
        -308931.0 / 896150.0,
    ],
    [
        -851781.0 / 878810.0,
        1648619.0 / 878810.0,
        36519.0 / 878810.0,
    ],
    [
        16779.0 / 1248040.0,
        -147721.0 / 1248040.0,
        1266979.0 / 1248040.0,
    ],
];
const PRO_XYZ50: M = [
    [0.7977666449006423, 0.13518129740053308, 0.0313477341283922],
    [0.2880748288194013, 0.711835234241873, 0.00008993693872564],
    [0.0, 0.0, 0.8251046025104602],
];
const XYZ50_PRO: M = [
    [
        1.3457868816471583,
        -0.25557208737979464,
        -0.05110186497554526,
    ],
    [-0.5446307051249019, 1.5082477428451468, 0.02052744743642139],
    [0.0, 0.0, 1.2119675456389452],
];
const R2020_XYZ: M = [
    [
        63426534.0 / 99577255.0,
        20160776.0 / 139408157.0,
        47086771.0 / 278816314.0,
    ],
    [
        26158966.0 / 99577255.0,
        472592308.0 / 697040785.0,
        8267143.0 / 139408157.0,
    ],
    [0.0, 19567812.0 / 697040785.0, 295819943.0 / 278816314.0],
];
const XYZ_R2020: M = [
    [
        30757411.0 / 17917100.0,
        -6372589.0 / 17917100.0,
        -4539589.0 / 17917100.0,
    ],
    [
        -19765991.0 / 29648200.0,
        47925759.0 / 29648200.0,
        467509.0 / 29648200.0,
    ],
    [
        792561.0 / 44930125.0,
        -1921689.0 / 44930125.0,
        42328811.0 / 44930125.0,
    ],
];
const D65_D50: M = [
    [
        1.0479297925449969,
        0.022946870601609652,
        -0.05019226628920524,
    ],
    [
        0.02962780877005599,
        0.9904344267538799,
        -0.017073799063418826,
    ],
    [
        -0.009243040646204504,
        0.015055191490298152,
        0.7518742814281371,
    ],
];
const D50_D65: M = [
    [0.955473421488075, -0.02309845494876471, 0.06325924320057072],
    [
        -0.0283697093338637,
        1.0099953980813041,
        0.021041441191917323,
    ],
    [
        0.012314014864481998,
        -0.020507649298898964,
        1.330365926242124,
    ],
];
const XYZ_LMS: M = [
    [0.819022437996703, 0.3619062600528904, -0.1288737815209879],
    [0.0329836539323885, 0.9292868615863434, 0.0361446663506424],
    [0.0481771893596242, 0.2642395317527308, 0.6335478284694309],
];
const LMS_OKLAB: M = [
    [0.210454268309314, 0.7936177747023054, -0.0040720430116193],
    [1.9779985324311684, -2.42859224204858, 0.450593709617411],
    [0.0259040424655478, 0.7827717124575296, -0.8086757549230774],
];
const OKLAB_LMS: M = [
    [1.0, 0.3963377773761749, 0.2158037573099136],
    [1.0, -0.1055613458156586, -0.0638541728258133],
    [1.0, -0.0894841775298119, -1.2914855480194092],
];
const LMS_XYZ: M = [
    [1.2268798758459243, -0.5578149944602171, 0.2813910456659647],
    [-0.0405757452148008, 1.112286803280317, -0.0717110580655164],
    [-0.0763729366746601, -0.4214933324022432, 1.5869240198367816],
];
const D50_WHITE: [f64; 3] = [0.3457 / 0.3585, 1.0, (1.0 - 0.3457 - 0.3585) / 0.3585];

fn per(v: [f64; 3], f: impl Fn(f64) -> f64) -> [f64; 3] {
    v.map(f)
}

fn signed(x: f64, f: impl Fn(f64) -> f64) -> f64 {
    x.signum() * f(x.abs())
}

fn srgb_lin(x: f64) -> f64 {
    signed(x, |a| {
        if a <= 0.04045 {
            a / 12.92
        } else {
            ((a + 0.055) / 1.055).powf(2.4)
        }
    })
}

fn srgb_gam(x: f64) -> f64 {
    signed(x, |a| {
        if a > 0.0031308 {
            1.055 * a.powf(1.0 / 2.4) - 0.055
        } else {
            12.92 * a
        }
    })
}

const R2020_A: f64 = 1.09929682680944;
const R2020_B: f64 = 0.018053968510807;

/// Linear-light to XYZ D65 of an RGB space's gamma-encoded channels.
fn rgb_to_xyz(space: Space, v: [f64; 3]) -> [f64; 3] {
    match space {
        Space::Srgb => mul(&SRGB_XYZ, per(v, srgb_lin)),
        Space::SrgbLinear => mul(&SRGB_XYZ, v),
        Space::DisplayP3 => mul(&P3_XYZ, per(v, srgb_lin)),
        Space::A98 => mul(&A98_XYZ, per(v, |x| signed(x, |a| a.powf(563.0 / 256.0)))),
        Space::ProPhoto => mul(
            &D50_D65,
            mul(
                &PRO_XYZ50,
                per(v, |x| {
                    signed(x, |a| {
                        if a <= 16.0 / 512.0 {
                            a / 16.0
                        } else {
                            a.powf(1.8)
                        }
                    })
                }),
            ),
        ),
        Space::Rec2020 => mul(
            &R2020_XYZ,
            per(v, |x| {
                signed(x, |a| {
                    if a < R2020_B * 4.5 {
                        a / 4.5
                    } else {
                        ((a + R2020_A - 1.0) / R2020_A).powf(1.0 / 0.45)
                    }
                })
            }),
        ),
        _ => v,
    }
}

fn xyz_to_rgb(space: Space, xyz: [f64; 3]) -> [f64; 3] {
    match space {
        Space::Srgb => per(mul(&XYZ_SRGB, xyz), srgb_gam),
        Space::SrgbLinear => mul(&XYZ_SRGB, xyz),
        Space::DisplayP3 => per(mul(&XYZ_P3, xyz), srgb_gam),
        Space::A98 => per(mul(&XYZ_A98, xyz), |x| signed(x, |a| a.powf(256.0 / 563.0))),
        Space::ProPhoto => per(mul(&XYZ50_PRO, mul(&D65_D50, xyz)), |x| {
            signed(x, |a| {
                if a >= 1.0 / 512.0 {
                    a.powf(1.0 / 1.8)
                } else {
                    16.0 * a
                }
            })
        }),
        Space::Rec2020 => per(mul(&XYZ_R2020, xyz), |x| {
            signed(x, |a| {
                if a > R2020_B {
                    R2020_A * a.powf(0.45) - (R2020_A - 1.0)
                } else {
                    4.5 * a
                }
            })
        }),
        _ => xyz,
    }
}

fn polar(lab: [f64; 3]) -> [f64; 3] {
    let h = lab[2].atan2(lab[1]).to_degrees().rem_euclid(360.0);
    [lab[0], lab[1].hypot(lab[2]), h]
}

fn cartesian(lch: [f64; 3]) -> [f64; 3] {
    let h = lch[2].to_radians();
    [lch[0], lch[1] * h.cos(), lch[1] * h.sin()]
}

fn hsl_to_srgb([h, s, l]: [f64; 3]) -> [f64; 3] {
    let (s, l) = (s / 100.0, l / 100.0);
    let f = |n: f64| {
        let k = (n + h / 30.0).rem_euclid(12.0);
        let a = s * l.min(1.0 - l);
        l - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0)
    };
    [f(0.0), f(8.0), f(4.0)]
}

fn srgb_to_hsl([r, g, b]: [f64; 3]) -> [f64; 3] {
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let (mut h, mut s, l) = (f64::NAN, 0.0, (min + max) / 2.0);
    let d = max - min;
    if d != 0.0 {
        s = if l == 0.0 || l == 1.0 {
            0.0
        } else {
            (max - l) / l.min(1.0 - l)
        };
        h = if max == r {
            (g - b) / d + if g < b { 6.0 } else { 0.0 }
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        } * 60.0;
    }
    if s < 0.0 {
        h += 180.0;
        s = s.abs();
    }
    [h.rem_euclid(360.0), s * 100.0, l * 100.0]
}

fn hwb_to_srgb([h, w, b]: [f64; 3]) -> [f64; 3] {
    let (w, b) = (w / 100.0, b / 100.0);
    if w + b >= 1.0 {
        let g = w / (w + b);
        return [g; 3];
    }
    hsl_to_srgb([h, 100.0, 50.0]).map(|c| c * (1.0 - w - b) + w)
}

fn srgb_to_hwb(rgb: [f64; 3]) -> [f64; 3] {
    let h = srgb_to_hsl(rgb)[0];
    let (max, min) = (
        rgb[0].max(rgb[1]).max(rgb[2]),
        rgb[0].min(rgb[1]).min(rgb[2]),
    );
    let hue = if max == min { f64::NAN } else { h };
    [hue, min * 100.0, (1.0 - max) * 100.0]
}

fn to_xyz(space: Space, c: [f64; 3]) -> [f64; 3] {
    match space {
        Space::XyzD65 => c,
        Space::XyzD50 => mul(&D50_D65, c),
        Space::Lab | Space::Lch => {
            let [l, a, b] = if space == Space::Lch { cartesian(c) } else { c };
            let (e, k) = (216.0 / 24389.0, 24389.0 / 27.0);
            let f1 = (l + 16.0) / 116.0;
            let (f0, f2) = (a / 500.0 + f1, f1 - b / 200.0);
            let x = if f0.powi(3) > e {
                f0.powi(3)
            } else {
                (116.0 * f0 - 16.0) / k
            };
            let y = if l > k * e { f1.powi(3) } else { l / k };
            let z = if f2.powi(3) > e {
                f2.powi(3)
            } else {
                (116.0 * f2 - 16.0) / k
            };
            mul(&D50_D65, [x * D50_WHITE[0], y, z * D50_WHITE[2]])
        }
        Space::Oklab | Space::Oklch => {
            let lab = if space == Space::Oklch {
                cartesian(c)
            } else {
                c
            };
            mul(&LMS_XYZ, mul(&OKLAB_LMS, lab).map(|v| v.powi(3)))
        }
        Space::Hsl => rgb_to_xyz(Space::Srgb, hsl_to_srgb(c)),
        Space::Hwb => rgb_to_xyz(Space::Srgb, hwb_to_srgb(c)),
        rgb => rgb_to_xyz(rgb, c),
    }
}

fn from_xyz(space: Space, xyz: [f64; 3]) -> [f64; 3] {
    match space {
        Space::XyzD65 => xyz,
        Space::XyzD50 => mul(&D65_D50, xyz),
        Space::Lab | Space::Lch => {
            let d50 = mul(&D65_D50, xyz);
            let (e, k) = (216.0 / 24389.0, 24389.0 / 27.0);
            let f = [0, 1, 2].map(|i| {
                let v = d50[i] / D50_WHITE[i];
                if v > e {
                    v.cbrt()
                } else {
                    (k * v + 16.0) / 116.0
                }
            });
            let lab = [
                116.0 * f[1] - 16.0,
                500.0 * (f[0] - f[1]),
                200.0 * (f[1] - f[2]),
            ];
            if space == Space::Lch { polar(lab) } else { lab }
        }
        Space::Oklab | Space::Oklch => {
            let lab = mul(&LMS_OKLAB, mul(&XYZ_LMS, xyz).map(f64::cbrt));
            if space == Space::Oklch {
                polar(lab)
            } else {
                lab
            }
        }
        Space::Hsl => srgb_to_hsl(xyz_to_rgb(Space::Srgb, xyz)),
        Space::Hwb => srgb_to_hwb(xyz_to_rgb(Space::Srgb, xyz)),
        rgb => xyz_to_rgb(rgb, xyz),
    }
}

/// `abs` in `to`. `none` components survive a conversion between
/// analogous spaces (CSS Color 4 §12.2); a hue that has no meaning (a grey
/// in a polar space) becomes `none`.
fn convert(abs: Abs, to: Space) -> Abs {
    let (from, c, a) = abs;
    if from == to {
        return abs;
    }
    let v = c.map(|x| f64::from(x.unwrap_or(0.0)));
    let mut out = from_xyz(to, to_xyz(from, v)).map(|x| Some(x as f32));
    // Analogous components: lightness, chroma/colourfulness, hue.
    let role = |s: Space, k: usize| -> Option<u8> {
        match (s, k) {
            (Space::Lab | Space::Oklab | Space::Lch | Space::Oklch, 0) | (Space::Hsl, 2) => Some(0),
            (Space::Lch | Space::Oklch, 1) | (Space::Hsl, 1) => Some(1),
            (s, k) if s.hue() == Some(k) => Some(2),
            (s, k) if s.rgb_like() => Some(10 + k as u8),
            _ => None,
        }
    };
    for (k, v) in c.iter().enumerate() {
        if v.is_none()
            && let Some(r) = role(from, k)
            && let Some(j) = (0..3).find(|&j| role(to, j) == Some(r))
        {
            out[j] = None;
        }
    }
    if let Some(h) = to.hue() {
        let chroma = match to {
            Space::Lch => out[1].map(|c| c < 1e-3),
            Space::Oklch => out[1].map(|c| c < 4e-6),
            Space::Hsl => out[1].map(|s| s.abs() < 1e-4),
            _ => out[0].map(|x| x.is_nan()),
        };
        if chroma == Some(true) || out[h].is_some_and(f32::is_nan) {
            out[h] = None;
        }
    }
    (to, out, a)
}

/// CSS Color 4 §13.2: sRGB `rgb` of `abs` brought into gamut by reducing
/// OKLCh chroma until clipping changes it by under one JND.
fn gamut_map(rgb: [f32; 3], abs: Abs) -> [f32; 3] {
    let inside = |c: [f64; 3]| c.iter().all(|v| (-1e-5..=1.0 + 1e-5).contains(v));
    let r = rgb.map(f64::from);
    if inside(r) {
        return rgb;
    }
    let (_, ok, _) = convert(abs, Space::Oklch);
    let l = f64::from(ok[0].unwrap_or(0.0));
    if l >= 1.0 {
        return [1.0; 3];
    }
    if l <= 0.0 {
        return [0.0; 3];
    }
    let (c0, h) = (
        f64::from(ok[1].unwrap_or(0.0)),
        f64::from(ok[2].unwrap_or(0.0)),
    );
    let to_rgb = |c: f64| from_xyz(Space::Srgb, to_xyz(Space::Oklch, [l, c, h]));
    let clip = |v: [f64; 3]| v.map(|x| x.clamp(0.0, 1.0));
    let delta = |a: [f64; 3], b: [f64; 3]| {
        let lab = |v| from_xyz(Space::Oklab, to_xyz(Space::Srgb, v));
        let (x, y) = (lab(a), lab(b));
        ((x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2) + (x[2] - y[2]).powi(2)).sqrt()
    };
    const JND: f64 = 0.02;
    let (mut lo, mut hi, mut lo_in) = (0.0, c0, true);
    let current = to_rgb(c0);
    let mut clipped = clip(current);
    if delta(clipped, current) < JND {
        return clipped.map(|v| v as f32);
    }
    while hi - lo > 1e-4 {
        let c = (lo + hi) / 2.0;
        let cur = to_rgb(c);
        if lo_in && inside(cur) {
            lo = c;
            continue;
        }
        clipped = clip(cur);
        let e = delta(clipped, cur);
        if e < JND {
            if JND - e < 1e-4 {
                break;
            }
            lo_in = false;
            lo = c;
        } else {
            hi = c;
        }
    }
    clipped.map(|v| v as f32)
}

/// CSS Color 4's named colours.
fn named(name: &str) -> Option<[u8; 3]> {
    NAMED.binary_search_by_key(&name, |(n, _)| n).ok().map(|i| {
        let v = NAMED[i].1;
        [(v >> 16) as u8, (v >> 8) as u8, v as u8]
    })
}

/// Whether `name` is one of CSS's named colours (or `transparent`,
/// `currentcolor`).
pub fn is_named(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n == "transparent" || n == "currentcolor" || named(&n).is_some()
}

const NAMED: &[(&str, u32)] = &[
    ("aliceblue", 0xf0f8ff),
    ("antiquewhite", 0xfaebd7),
    ("aqua", 0x00ffff),
    ("aquamarine", 0x7fffd4),
    ("azure", 0xf0ffff),
    ("beige", 0xf5f5dc),
    ("bisque", 0xffe4c4),
    ("black", 0x000000),
    ("blanchedalmond", 0xffebcd),
    ("blue", 0x0000ff),
    ("blueviolet", 0x8a2be2),
    ("brown", 0xa52a2a),
    ("burlywood", 0xdeb887),
    ("cadetblue", 0x5f9ea0),
    ("chartreuse", 0x7fff00),
    ("chocolate", 0xd2691e),
    ("coral", 0xff7f50),
    ("cornflowerblue", 0x6495ed),
    ("cornsilk", 0xfff8dc),
    ("crimson", 0xdc143c),
    ("cyan", 0x00ffff),
    ("darkblue", 0x00008b),
    ("darkcyan", 0x008b8b),
    ("darkgoldenrod", 0xb8860b),
    ("darkgray", 0xa9a9a9),
    ("darkgreen", 0x006400),
    ("darkgrey", 0xa9a9a9),
    ("darkkhaki", 0xbdb76b),
    ("darkmagenta", 0x8b008b),
    ("darkolivegreen", 0x556b2f),
    ("darkorange", 0xff8c00),
    ("darkorchid", 0x9932cc),
    ("darkred", 0x8b0000),
    ("darksalmon", 0xe9967a),
    ("darkseagreen", 0x8fbc8f),
    ("darkslateblue", 0x483d8b),
    ("darkslategray", 0x2f4f4f),
    ("darkslategrey", 0x2f4f4f),
    ("darkturquoise", 0x00ced1),
    ("darkviolet", 0x9400d3),
    ("deeppink", 0xff1493),
    ("deepskyblue", 0x00bfff),
    ("dimgray", 0x696969),
    ("dimgrey", 0x696969),
    ("dodgerblue", 0x1e90ff),
    ("firebrick", 0xb22222),
    ("floralwhite", 0xfffaf0),
    ("forestgreen", 0x228b22),
    ("fuchsia", 0xff00ff),
    ("gainsboro", 0xdcdcdc),
    ("ghostwhite", 0xf8f8ff),
    ("gold", 0xffd700),
    ("goldenrod", 0xdaa520),
    ("gray", 0x808080),
    ("green", 0x008000),
    ("greenyellow", 0xadff2f),
    ("grey", 0x808080),
    ("honeydew", 0xf0fff0),
    ("hotpink", 0xff69b4),
    ("indianred", 0xcd5c5c),
    ("indigo", 0x4b0082),
    ("ivory", 0xfffff0),
    ("khaki", 0xf0e68c),
    ("lavender", 0xe6e6fa),
    ("lavenderblush", 0xfff0f5),
    ("lawngreen", 0x7cfc00),
    ("lemonchiffon", 0xfffacd),
    ("lightblue", 0xadd8e6),
    ("lightcoral", 0xf08080),
    ("lightcyan", 0xe0ffff),
    ("lightgoldenrodyellow", 0xfafad2),
    ("lightgray", 0xd3d3d3),
    ("lightgreen", 0x90ee90),
    ("lightgrey", 0xd3d3d3),
    ("lightpink", 0xffb6c1),
    ("lightsalmon", 0xffa07a),
    ("lightseagreen", 0x20b2aa),
    ("lightskyblue", 0x87cefa),
    ("lightslategray", 0x778899),
    ("lightslategrey", 0x778899),
    ("lightsteelblue", 0xb0c4de),
    ("lightyellow", 0xffffe0),
    ("lime", 0x00ff00),
    ("limegreen", 0x32cd32),
    ("linen", 0xfaf0e6),
    ("magenta", 0xff00ff),
    ("maroon", 0x800000),
    ("mediumaquamarine", 0x66cdaa),
    ("mediumblue", 0x0000cd),
    ("mediumorchid", 0xba55d3),
    ("mediumpurple", 0x9370db),
    ("mediumseagreen", 0x3cb371),
    ("mediumslateblue", 0x7b68ee),
    ("mediumspringgreen", 0x00fa9a),
    ("mediumturquoise", 0x48d1cc),
    ("mediumvioletred", 0xc71585),
    ("midnightblue", 0x191970),
    ("mintcream", 0xf5fffa),
    ("mistyrose", 0xffe4e1),
    ("moccasin", 0xffe4b5),
    ("navajowhite", 0xffdead),
    ("navy", 0x000080),
    ("oldlace", 0xfdf5e6),
    ("olive", 0x808000),
    ("olivedrab", 0x6b8e23),
    ("orange", 0xffa500),
    ("orangered", 0xff4500),
    ("orchid", 0xda70d6),
    ("palegoldenrod", 0xeee8aa),
    ("palegreen", 0x98fb98),
    ("paleturquoise", 0xafeeee),
    ("palevioletred", 0xdb7093),
    ("papayawhip", 0xffefd5),
    ("peachpuff", 0xffdab9),
    ("peru", 0xcd853f),
    ("pink", 0xffc0cb),
    ("plum", 0xdda0dd),
    ("powderblue", 0xb0e0e6),
    ("purple", 0x800080),
    ("rebeccapurple", 0x663399),
    ("red", 0xff0000),
    ("rosybrown", 0xbc8f8f),
    ("royalblue", 0x4169e1),
    ("saddlebrown", 0x8b4513),
    ("salmon", 0xfa8072),
    ("sandybrown", 0xf4a460),
    ("seagreen", 0x2e8b57),
    ("seashell", 0xfff5ee),
    ("sienna", 0xa0522d),
    ("silver", 0xc0c0c0),
    ("skyblue", 0x87ceeb),
    ("slateblue", 0x6a5acd),
    ("slategray", 0x708090),
    ("slategrey", 0x708090),
    ("snow", 0xfffafa),
    ("springgreen", 0x00ff7f),
    ("steelblue", 0x4682b4),
    ("tan", 0xd2b48c),
    ("teal", 0x008080),
    ("thistle", 0xd8bfd8),
    ("tomato", 0xff6347),
    ("turquoise", 0x40e0d0),
    ("violet", 0xee82ee),
    ("wheat", 0xf5deb3),
    ("white", 0xffffff),
    ("whitesmoke", 0xf5f5f5),
    ("yellow", 0xffff00),
    ("yellowgreen", 0x9acd32),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(s: &str) -> [u8; 4] {
        let c = parse_str(s)
            .unwrap_or_else(|e| panic!("{s}: {e}"))
            .literal()
            .unwrap();
        [
            c.rgb[0],
            c.rgb[1],
            c.rgb[2],
            (c.alpha * 255.0).round() as u8,
        ]
    }

    /// Expected values are the CSS Color 4 spec's and browsers' (Chrome
    /// and Firefox agree), so a wrong matrix, a hue in the wrong unit or a
    /// mix that is not premultiplied shows.
    #[test]
    fn colour_functions_match_the_spec_examples() {
        assert!(NAMED.windows(2).all(|w| w[0].0 < w[1].0) && NAMED.len() == 148);
        assert_eq!(lit("rebeccapurple"), [102, 51, 153, 255]);
        assert_eq!(lit("#f808"), [255, 136, 0, 136]);
        assert_eq!(lit("rgb(255 0 0 / 50%)"), [255, 0, 0, 128]);
        assert_eq!(lit("rgba(0, 128, 255, 0.25)"), [0, 128, 255, 64]);
        assert_eq!(lit("hsl(120deg 100% 25%)"), [0, 128, 0, 255]);
        assert_eq!(lit("hsl(0.5turn, 100%, 50%)"), [0, 255, 255, 255]);
        assert_eq!(lit("hwb(90 20% 30%)"), [115, 179, 51, 255]);
        // Computed independently from CSS Color 4's formulas (D50 Lab,
        // Bradford to D65): a matrix or white-point slip moves it.
        assert_eq!(lit("lab(50% 40 59.5)"), [191, 87, 0, 255]);
        assert_eq!(lit("oklch(62.8% 0.2577 29.23)"), [255, 0, 0, 255]);
        assert_eq!(lit("oklab(0.628 0.2249 0.1258)"), [255, 0, 0, 255]);
        assert_eq!(
            lit("color(srgb-linear 0.2159 0.2159 0.2159)"),
            [128, 128, 128, 255]
        );
        // Display P3 red is outside sRGB: mapped, not clipped to pure red.
        let p3 = lit("color(display-p3 1 0 0)");
        assert_eq!(p3[0], 255);
        assert!(p3[1] < 30 && p3[2] < 30, "{p3:?}");
        // Mixed in sRGB: halfway; with transparency premultiplied, so a
        // transparent side lends no colour.
        assert_eq!(lit("color-mix(in srgb, red, blue)"), [128, 0, 128, 255]);
        assert_eq!(
            lit("color-mix(in srgb, red 25%, transparent)"),
            [255, 0, 0, 64]
        );
        // Percentages under 100% scale the alpha.
        assert_eq!(lit("color-mix(in srgb, red 30%, blue 20%)")[3], 128);
        // Longer hue goes the other way round.
        let short = lit("color-mix(in hsl, hsl(10 100% 50%), hsl(350 100% 50%))");
        let long = lit("color-mix(in hsl longer hue, hsl(10 100% 50%), hsl(350 100% 50%))");
        assert_eq!(short, [255, 0, 0, 255]);
        assert_eq!(long, [0, 255, 255, 255]);
        // Relative colours: the complement, and alpha from a channel.
        assert_eq!(lit("hsl(from red calc(h + 180) s l)"), [0, 255, 255, 255]);
        assert_eq!(lit("rgb(from #ff8000 r g b / 50%)"), [255, 128, 0, 128]);
        assert_eq!(lit("rgb(from #204060 calc(r * 2) g b)"), [64, 64, 96, 255]);
        assert!(parse_str("color-mix(in srgb, red 0%, blue 0%)").is_err());
        assert!(parse_str("rgb(1, 2 3)").is_err());
    }

    #[test]
    fn tokens_and_light_dark_resolve_against_the_style() {
        let light = crate::Theme::new(crate::Scheme::Light);
        let dark = crate::Theme::new(crate::Scheme::Dark);
        let c = parse_str("light-dark(white, var(--ink))").unwrap();
        assert!(c.literal().is_none());
        assert_eq!(c.resolve(&Context::of(&light)).rgb, [255; 3]);
        assert_eq!(c.resolve(&Context::of(&dark)), dark.palette.ink);
        let mix = parse_str("color-mix(in srgb, var(--panel), black)").unwrap();
        let p = dark.palette.panel.rgb;
        assert_eq!(
            mix.resolve(&Context::of(&dark)).rgb,
            p.map(|v| (f32::from(v) / 2.0).round() as u8)
        );
    }
}
