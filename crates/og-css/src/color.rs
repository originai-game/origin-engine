//! CSS colours as CSS Color Level 4 and 5 define them. Absolute colours
//! (hex, every named colour, `rgb()`, `hsl()`, `hwb()`, `lab()`, `lch()`,
//! `oklab()`, `oklch()` and `color()` over every predefined space, with
//! `none` components), every conversion and `color-mix()`'s interpolation
//! (CSS Color 4 §12: missing components, hue directions, premultiplied
//! alpha) are linebender's `color` crate. This module adds what it leaves
//! to the host: `var(--role)`, `currentColor`, `light-dark()`,
//! `contrast-color()`, `color-mix()`'s percentages over any number of
//! colours, relative colours (`oklch(from var(--accent) l c calc(h + 180))`)
//! and CSS Color 4 §13.2 gamut mapping.
//!
//! A colour that reads no token, no `currentColor` and no `light-dark()`
//! reduces to sRGB when it is parsed ([`Color::literal`]); one that does
//! stays an expression resolved against the active style when it is drawn
//! ([`Color::resolve`]).

use crate::{Srgba, Tok, calc, camel, describe, tok_text, tokens};
use color::{ColorSpaceTag as Space, DynamicColor, Flags, HueDirection, Missing, OpaqueColor};

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
    /// An absolute colour in the space it was written in, `none`
    /// components marked missing.
    Abs(DynamicColor),
    /// `var(--role)`: a palette role resolved through the style.
    Token(String),
    Current,
    LightDark(Box<Color>, Box<Color>),
    /// `contrast-color(<ground>)`: white or black, whichever reads more on
    /// the ground ([`Srgba::contrast_color`]).
    Contrast(Box<Color>),
    /// `color-mix()`: one or more colours, each with its percentage if it
    /// was given, mixed in order.
    Mix {
        space: Space,
        hue: HueDirection,
        items: Vec<(Color, Option<f32>)>,
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
    /// `alpha(from <colour> / <alpha>)`: the origin in its own space with
    /// a new alpha (CSS Color 5 §4.3); `None` keeps the origin's.
    Alpha {
        from: Box<Color>,
        alpha: Option<Expr>,
    },
}

/// The palette roles a colour's `var(--role)` names, and the style's own
/// ink and scheme: what a [`Context`] reads from a style (og-ui's `Theme`).
pub trait ColorTokens {
    /// The colour of palette role `name` (camelCase), if the style has it.
    fn color(&self, name: &str) -> Option<Srgba>;
    /// The style's ink: `currentColor` where an element sets none.
    fn ink(&self) -> Srgba;
    /// The style's colour scheme is dark.
    fn dark(&self) -> bool;
    /// The gamut the output presents: [`Color::linear`] maps into it.
    fn gamut(&self) -> Gamut {
        Gamut::Srgb
    }
}

/// The gamut a colour is drawn in: what the display output presents.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Gamut {
    #[default]
    Srgb,
    /// Display P3: colours outside sRGB keep their gamut up to P3's.
    P3,
}

impl Gamut {
    fn space(self) -> Space {
        match self {
            Gamut::Srgb => Space::Srgb,
            Gamut::P3 => Space::DisplayP3,
        }
    }
}

/// What a colour is resolved against.
#[derive(Clone, Copy)]
pub struct Context<'a> {
    /// Where `var(--role)` is looked up, borrowed for the resolve.
    pub tokens: &'a dyn ColorTokens,
    /// `currentColor` (the element's `color`).
    pub current: Srgba,
    /// The used colour scheme is dark (`light-dark()` takes its second).
    pub dark: bool,
    /// The output's gamut ([`Color::linear`]).
    pub gamut: Gamut,
}

impl std::fmt::Debug for Context<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("current", &self.current)
            .field("dark", &self.dark)
            .field("gamut", &self.gamut)
            .finish_non_exhaustive()
    }
}

impl<'a> Context<'a> {
    /// The style's own: its ink as `currentColor`, its scheme.
    pub fn of(tokens: &'a dyn ColorTokens) -> Context<'a> {
        Context {
            tokens,
            current: tokens.ink(),
            dark: tokens.dark(),
            gamut: tokens.gamut(),
        }
    }
}

/// No palette: what a colour that depends on nothing is resolved against
/// ([`Color::literal`] never reads it).
struct NoTokens;

impl ColorTokens for NoTokens {
    fn color(&self, _: &str) -> Option<Srgba> {
        None
    }
    fn ink(&self) -> Srgba {
        Srgba::BLACK
    }
    fn dark(&self) -> bool {
        false
    }
}

impl Color {
    /// The colour as sRGB when it depends on nothing (no token,
    /// `currentColor` or `light-dark()`).
    pub fn literal(&self) -> Option<Srgba> {
        self.fixed()
            .then(|| to_srgba(self.eval(&Context::of(&NoTokens))))
    }

    fn fixed(&self) -> bool {
        match self {
            Color::Abs(_) => true,
            Color::Token(_) | Color::Current | Color::LightDark(..) => false,
            Color::Contrast(c) => c.fixed(),
            Color::Mix { items, .. } => items.iter().all(|(c, _)| c.fixed()),
            Color::Relative { from, .. } | Color::Alpha { from, .. } => from.fixed(),
        }
    }

    /// Whether it reads `currentColor`.
    pub fn uses_current(&self) -> bool {
        match self {
            Color::Current => true,
            Color::Abs(_) | Color::Token(_) => false,
            Color::LightDark(a, b) => a.uses_current() || b.uses_current(),
            Color::Mix { items, .. } => items.iter().any(|(c, _)| c.uses_current()),
            Color::Contrast(from) | Color::Relative { from, .. } | Color::Alpha { from, .. } => {
                from.uses_current()
            }
        }
    }

    /// Whether it reads the colour scheme (`light-dark()`).
    pub fn uses_scheme(&self) -> bool {
        match self {
            Color::LightDark(..) => true,
            Color::Abs(_) | Color::Token(_) | Color::Current => false,
            Color::Mix { items, .. } => items.iter().any(|(c, _)| c.uses_scheme()),
            Color::Contrast(from) | Color::Relative { from, .. } | Color::Alpha { from, .. } => {
                from.uses_scheme()
            }
        }
    }

    /// Its palette roles, for the cook's check.
    pub fn tokens(&self, out: &mut Vec<String>) {
        match self {
            Color::Token(t) => out.push(t.clone()),
            Color::Abs(_) | Color::Current => {}
            Color::LightDark(a, b) => {
                a.tokens(out);
                b.tokens(out);
            }
            Color::Mix { items, .. } => items.iter().for_each(|(c, _)| c.tokens(out)),
            Color::Contrast(from) | Color::Relative { from, .. } | Color::Alpha { from, .. } => {
                from.tokens(out)
            }
        }
    }

    /// The sRGB colour it is in `cx`, gamut mapped; outside sRGB in a P3
    /// `cx`, it also keeps its gamut mapping into Display P3
    /// ([`Srgba::p3`]), which [`Srgba::linear`] draws.
    pub fn resolve(&self, cx: &Context) -> Srgba {
        let c = self.eval(cx);
        let p3 = (cx.gamut == Gamut::P3 && !in_srgb(c))
            .then(|| gamut_map(c, Gamut::P3).map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8));
        Srgba { p3, ..to_srgba(c) }
    }

    /// Linear RGBA (straight alpha) in `cx`'s gamut, what og-render draws:
    /// [`Self::resolve`]'s colour where it is inside sRGB or the gamut is
    /// sRGB; otherwise gamut mapped into Display P3 and given as extended
    /// linear sRGB (a channel may be negative).
    pub fn linear(&self, cx: &Context) -> [f32; 4] {
        let c = self.eval(cx);
        if cx.gamut == Gamut::Srgb || in_srgb(c) {
            return to_srgba(c).linear();
        }
        let p3 = gamut_map(c, Gamut::P3);
        let [r, g, b] = Space::DisplayP3.convert(Space::LinearSrgb, p3);
        [r, g, b, c.components[3].clamp(0.0, 1.0)]
    }

    /// Whether it depends on nothing and lies outside sRGB (`color(display-p3
    /// 1 0 0)`, a wide `oklch()`): [`Self::literal`] loses its gamut, so a
    /// caller keeps it as an expression for [`Self::linear`].
    pub fn wide(&self) -> bool {
        self.fixed() && !in_srgb(self.eval(&Context::of(&NoTokens)))
    }

    fn eval(&self, cx: &Context) -> DynamicColor {
        match self {
            Color::Abs(c) => *c,
            Color::Token(name) => cx
                .tokens
                .color(name)
                .unwrap_or(Srgba::rgb(255, 0, 255))
                .into(),
            Color::Current => cx.current.into(),
            Color::LightDark(l, d) => {
                if cx.dark {
                    d.eval(cx)
                } else {
                    l.eval(cx)
                }
            }
            Color::Contrast(ground) => to_srgba(ground.eval(cx)).contrast_color().into(),
            Color::Mix { space, hue, items } => mix(
                *space,
                *hue,
                items.iter().map(|(c, p)| (c.eval(cx), *p)).collect(),
            ),
            Color::Alpha { from, alpha } => {
                let mut c = from.eval(cx);
                let a = c.components[3];
                match alpha.as_ref().map(|e| e.eval(&[0.0, 0.0, 0.0, a])) {
                    None => {}
                    Some(Some(v)) => c.components[3] = v.clamp(0.0, 1.0),
                    Some(None) => {
                        let mut missing = c.flags.missing();
                        missing.insert(3);
                        c.flags.set_missing(missing);
                        c.components[3] = 0.0;
                    }
                }
                c
            }
            Color::Relative {
                space,
                rgb255,
                from,
                c,
                alpha,
            } => {
                // The origin's missing components read as zero (CSS Color 5
                // §4.1); `color` keeps them zero.
                let o = from.eval(cx).convert(*space).components;
                let scale = if *rgb255 { 255.0 } else { 1.0 };
                let chans = [o[0] * scale, o[1] * scale, o[2] * scale, o[3]];
                let ev = |e: &Expr| e.eval(&chans);
                let mut out = [ev(&c[0]), ev(&c[1]), ev(&c[2])].map(|v| v.map(|v| v / scale));
                if let Some(h) = hue_channel(*space) {
                    out[h] = out[h].map(|v| v.rem_euclid(360.0));
                }
                let a = match alpha {
                    Some(e) => ev(e).map(|v| v.clamp(0.0, 1.0)),
                    None => Some(o[3]),
                };
                let mut missing = Missing::EMPTY;
                let mut components = [0.0; 4];
                for (k, v) in out.into_iter().chain([a]).enumerate() {
                    match v {
                        Some(v) => components[k] = v,
                        None => missing.insert(k),
                    }
                }
                DynamicColor {
                    cs: *space,
                    flags: Flags::from_missing(missing),
                    components,
                }
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

fn to_srgba(c: DynamicColor) -> Srgba {
    Srgba {
        rgb: gamut_map(c, Gamut::Srgb).map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8),
        // A missing alpha is zero (CSS Color 4 §4.4), as `color` keeps it.
        alpha: c.components[3].clamp(0.0, 1.0),
        p3: None,
    }
}

/// The component holding a space's hue, if it has one.
fn hue_channel(space: Space) -> Option<usize> {
    match space {
        Space::Hsl | Space::Hwb => Some(0),
        Space::Lch | Space::Oklch => Some(2),
        _ => None,
    }
}

/// A colour space by its CSS name (`color()`'s and `color-mix()`'s).
fn space_named(name: &str) -> Option<Space> {
    Some(match name {
        "srgb" => Space::Srgb,
        "srgb-linear" => Space::LinearSrgb,
        "display-p3" => Space::DisplayP3,
        "a98-rgb" => Space::A98Rgb,
        "prophoto-rgb" => Space::ProphotoRgb,
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

/// A `<color-interpolation-method>` (`in oklch longer hue`), as
/// `color-mix()` and gradients take it: the tokens after `in`.
pub fn interpolation(t: &[Tok]) -> Result<(Space, HueDirection), String> {
    const HOW: &str = "in <colour space> [shorter | longer | increasing | decreasing hue]";
    let [Tok::Ident(s), rest @ ..] = t else {
        return Err(HOW.into());
    };
    let space = space_named(s).ok_or(format!("'{s}' is not a colour space"))?;
    let hue = match rest {
        [] => HueDirection::Shorter,
        [Tok::Ident(h), Tok::Ident(w)] if w == "hue" && hue_channel(space).is_some() => {
            match h.as_str() {
                "shorter" => HueDirection::Shorter,
                "longer" => HueDirection::Longer,
                "increasing" => HueDirection::Increasing,
                "decreasing" => HueDirection::Decreasing,
                _ => return Err(HOW.into()),
            }
        }
        _ => return Err(HOW.into()),
    };
    Ok((space, hue))
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
pub fn parse(t: &Tok) -> Result<Color, String> {
    match t {
        Tok::Ident(w) if w.eq_ignore_ascii_case("currentcolor") => Ok(Color::Current),
        Tok::Func(f, args) => match f.to_ascii_lowercase().as_str() {
            "var" => match args.as_slice() {
                [v] => match v.as_slice() {
                    [Tok::Ident(n)] if n.starts_with("--") => Ok(Color::Token(camel(&n[2..]))),
                    _ => Err("var(--<role>)".into()),
                },
                _ => Err("a colour's var(--<role>) takes no fallback".into()),
            },
            "light-dark" => match args.as_slice() {
                [a, b] => Ok(Color::LightDark(Box::new(one(a)?), Box::new(one(b)?))),
                _ => Err("light-dark(<light>, <dark>)".into()),
            },
            "color-mix" => color_mix(args),
            "alpha" => alpha(args),
            "contrast-color" => match args.as_slice() {
                [a] => Ok(Color::Contrast(Box::new(one(a)?))),
                _ => Err("contrast-color(<colour>)".into()),
            },
            f @ ("rgb" | "rgba" | "hsl" | "hsla" | "hwb" | "lab" | "lch" | "oklab" | "oklch"
            | "color") => match args.as_slice() {
                [body] if matches!(body.first(), Some(Tok::Ident(w)) if w == "from") => {
                    relative(f, body)
                }
                _ => absolute(t),
            },
            f => Err(format!("{f}() is not a colour")),
        },
        Tok::Ident(w) if let Some(role) = system_role(w) => Ok(Color::Token(role.into())),
        Tok::Ident(_) | Tok::Hash(_) => absolute(t),
        t => Err(format!("'{}' is not a colour", describe(t))),
    }
}

/// A colour `color` parses: hex, a name or a colour function, its math
/// functions (`rgb(calc(255 / 2) 0 0)`) reduced first.
fn absolute(t: &Tok) -> Result<Color, String> {
    let text = calc::reduce(&tok_text(t))?;
    color::parse_color(&text)
        .map(Color::Abs)
        .map_err(|e| format!("'{text}' is not a colour ({e})"))
}

fn one(t: &[Tok]) -> Result<Color, String> {
    match t {
        [c] => parse(c),
        _ => Err("one colour".into()),
    }
}

/// A channel's number: its value with `percent` the value 100% stands
/// for; `hue` reads angles in degrees.
fn number(t: &Tok, percent: f32, hue: bool) -> Result<f32, String> {
    let Tok::Num(n, u) = t else {
        return Err(format!("'{}' is not a colour component", describe(t)));
    };
    let v: f32 = n.parse().map_err(|_| format!("'{n}' is not a number"))?;
    Ok(match u.as_str() {
        "" => v,
        "%" if !hue => v / 100.0 * percent,
        "deg" if hue => v,
        "rad" if hue => v.to_degrees(),
        "grad" if hue => v * 0.9,
        "turn" if hue => v * 360.0,
        _ => return Err(format!("'{n}{u}' is not a colour component")),
    })
}

/// The channel keywords and percentage references of a function's space.
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

/// `<fn>(from <colour> …)`: CSS Color 5's relative colour syntax.
fn relative(f: &str, toks: &[Tok]) -> Result<Color, String> {
    let (body, alpha) = match toks.iter().position(|t| *t == Tok::Slash) {
        Some(i) => match &toks[i + 1..] {
            [a] => (&toks[..i], Some(a)),
            _ => return Err(format!("{f}(): one alpha after /")),
        },
        None => (toks, None),
    };
    let from = Box::new(parse(body.get(1).ok_or(format!("{f}(from <colour> …)"))?)?);
    let (space, rest) = match f {
        "rgb" | "rgba" => (Space::Srgb, &body[2..]),
        "hsl" | "hsla" => (Space::Hsl, &body[2..]),
        "hwb" => (Space::Hwb, &body[2..]),
        "lab" => (Space::Lab, &body[2..]),
        "lch" => (Space::Lch, &body[2..]),
        "oklab" => (Space::Oklab, &body[2..]),
        "oklch" => (Space::Oklch, &body[2..]),
        _ => match body.get(2) {
            Some(Tok::Ident(s)) => (
                space_named(s)
                    .filter(|s| {
                        !matches!(
                            s,
                            Space::Lab
                                | Space::Lch
                                | Space::Oklab
                                | Space::Oklch
                                | Space::Hsl
                                | Space::Hwb
                        )
                    })
                    .ok_or(format!("color(): '{s}' is not a predefined colour space"))?,
                &body[3..],
            ),
            _ => return Err("color(from <colour> <space> …)".into()),
        },
    };
    let (names, pct) = channels(f, space);
    let kws: Vec<&str> = names.iter().copied().chain(["alpha"]).collect();
    let [a, b, c] = rest else {
        return Err(format!("{f}(from <colour> …) takes three channels"));
    };
    let hue = hue_channel(space);
    let ch = |k: usize, t: &Tok| expr_one(t, &kws, pct[k], hue == Some(k));
    let alpha = alpha.map(|t| expr_one(t, &kws, 1.0, false)).transpose()?;
    Ok(Color::Relative {
        space,
        rgb255: matches!(f, "rgb" | "rgba"),
        from,
        c: [ch(0, a)?, ch(1, b)?, ch(2, c)?],
        alpha,
    })
}

/// `alpha(from <colour> [/ <alpha>])`: only `alpha` is a channel keyword.
fn alpha(args: &[Vec<Tok>]) -> Result<Color, String> {
    const HOW: &str = "alpha(from <colour> [/ <alpha>])";
    let [body] = args else { return Err(HOW.into()) };
    let (from, alpha) = match body.as_slice() {
        [Tok::Ident(w), c] if w == "from" => (c, None),
        [Tok::Ident(w), c, Tok::Slash, a] if w == "from" => (c, Some(a)),
        _ => return Err(HOW.into()),
    };
    Ok(Color::Alpha {
        from: Box::new(parse(from)?),
        alpha: alpha
            .map(|t| expr_one(t, &["", "", "", "alpha"], 1.0, false))
            .transpose()?,
    })
}

/// The palette role a CSS Color 4 §6.2 system colour is: the style's
/// colours are the "system" a game's UI runs in.
fn system_role(name: &str) -> Option<&'static str> {
    Some(match name.to_ascii_lowercase().as_str() {
        "canvas" => "backdrop",
        "canvastext" | "buttontext" | "fieldtext" | "marktext" => "ink",
        "accentcolor" => "primary",
        "accentcolortext" => "onPrimary",
        "buttonface" | "field" => "control",
        "buttonborder" => "lineStrong",
        "graytext" => "inkFaint",
        "highlight" | "selecteditem" => "selection",
        "highlighttext" | "selecteditemtext" => "onSelection",
        "linktext" | "visitedtext" | "activetext" => "highlight",
        "mark" => "primarySoft",
        _ => return None,
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
            Tok::Num(..) => Expr::Num(number(t, self.percent, self.hue)?),
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
    const HOW: &str = "color-mix([in <space> [<hue> hue],] <colour> [<n>%], …): one colour or more";
    // CSS Color 5 §3.1: without a method, the mix is in OKLab.
    let (space, hue, items) = match args.first().map(Vec::as_slice) {
        Some([Tok::Ident(i), rest @ ..]) if i == "in" => {
            let (space, hue) = interpolation(rest).map_err(|e| format!("color-mix(): {e}"))?;
            (space, hue, &args[1..])
        }
        _ => (Space::Oklab, HueDirection::Shorter, args),
    };
    if items.is_empty() {
        return Err(HOW.into());
    }
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
    let items = items
        .iter()
        .map(|t| match t.as_slice() {
            [c] => Ok((parse(c)?, None)),
            [c, p] if matches!(p, Tok::Num(..)) => Ok((parse(c)?, Some(pct(p)?))),
            [p, c] => Ok((parse(c)?, Some(pct(p)?))),
            _ => Err(HOW.to_string()),
        })
        .collect::<Result<_, _>>()?;
    Ok(Color::Mix { space, hue, items })
}

// ---- mixing

/// `color-mix()`: CSS Color 5 §3.3 over `color`'s CSS Color 4 §12
/// interpolation. The percentages are normalised (CSS Values 5 §6.1, forced):
/// omitted ones share what the given ones leave of 100%, and all are scaled
/// to sum to 100%; what they summed to under 100% is the alpha multiplier
/// (all 0%: transparent). Then each colour is mixed into the mix of those
/// before it, in order, by its share of their combined percentage.
fn mix(space: Space, hue: HueDirection, items: Vec<(DynamicColor, Option<f32>)>) -> DynamicColor {
    let given = items.iter().filter_map(|(_, p)| *p).sum::<f32>().min(100.0);
    let omitted = items.iter().filter(|(_, p)| p.is_none()).count().max(1) as f32;
    let mut parts: Vec<(DynamicColor, f32)> = items
        .into_iter()
        .map(|(c, p)| (c, p.unwrap_or((100.0 - given) / omitted)))
        .collect();
    let total: f32 = parts.iter().map(|(_, p)| p).sum();
    if total > 0.0 {
        parts.iter_mut().for_each(|(_, p)| *p *= 100.0 / total);
    }
    let alpha = 1.0 - (100.0 - total).max(0.0) / 100.0;
    let mut parts = parts.into_iter();
    let Some((first, mut weight)) = parts.next() else {
        return DynamicColor::from_alpha_color(color::AlphaColor::<color::Srgb>::TRANSPARENT);
    };
    let mut mixed = interpolating(first, space);
    for (c, p) in parts {
        let combined = weight + p;
        let t = if combined > 0.0 { p / combined } else { 0.5 };
        mixed = interpolating(mixed, space)
            .interpolate(interpolating(c, space), space, hue)
            .eval(t);
        weight = combined;
    }
    if alpha < 1.0 {
        mixed.multiply_alpha(alpha)
    } else {
        mixed
    }
}

/// `color-mix(in <method>, …)` of straight sRGB colours (0 to 1), each with
/// its percentage if it was given: what `palette-mix()` does to each of a
/// palette's entries (CSS Fonts 4 §9.1). `method` is what follows `in`
/// (`oklab`, `oklch longer hue`). The mix is clamped to sRGB.
pub fn mix_srgb(method: &str, items: &[([f32; 4], Option<f32>)]) -> Result<[f32; 4], String> {
    let (space, hue) = interpolation(&tokens(method)?)?;
    let items = items
        .iter()
        .map(|&(c, p)| {
            let c = color::AlphaColor::<color::Srgb>::new(c);
            (DynamicColor::from_alpha_color(c), p)
        })
        .collect();
    let c = mix(space, hue, items)
        .to_alpha_color::<color::Srgb>()
        .components;
    Ok(c.map(|v| v.clamp(0.0, 1.0)))
}

/// `c` in `space`, ready for `color`'s interpolation there: a hue made
/// missing where it is powerless (CSS Color 4 §12.4). `color` 0.3 takes a
/// hue as powerless under an absolute chroma of 1e-6, which suits OKLCh
/// but not LCH's 0–150 scale: an sRGB grey keeps up to 3e-5 of chroma in
/// LCH, and its arbitrary hue would turn the mix.
pub fn interpolating(c: DynamicColor, space: Space) -> DynamicColor {
    let mut c = c.convert(space);
    if space == Space::Lch && c.components[1] < 1e-3 {
        let mut missing = c.flags.missing();
        missing.insert(2);
        c.flags.set_missing(missing);
        c.components[2] = 0.0;
    }
    c
}

// ---- gamut mapping

fn inside(v: [f32; 3]) -> bool {
    v.iter().all(|v| (-1e-5..=1.0 + 1e-5).contains(v))
}

fn in_srgb(c: DynamicColor) -> bool {
    let [r, g, b, _] = c.convert(Space::Srgb).components;
    inside([r, g, b])
}

/// CSS Color 4 §13.2: `c` in `gamut`'s space (encoded sRGB or Display P3),
/// brought into gamut by reducing its OKLCh chroma until clipping changes
/// it by under one JND (deltaEOK).
fn gamut_map(c: DynamicColor, gamut: Gamut) -> [f32; 3] {
    let dest = gamut.space();
    let [r, g, b, _] = c.convert(dest).components;
    if inside([r, g, b]) {
        return [r, g, b];
    }
    let [l, c0, h, _] = c.convert(Space::Oklch).components;
    if l >= 1.0 {
        return [1.0; 3];
    }
    if l <= 0.0 {
        return [0.0; 3];
    }
    let to_rgb = |c: f32| Space::Oklch.convert(dest, [l, c, h]);
    let delta = |a: [f32; 3], b: [f32; 3]| {
        let lab = |v| OpaqueColor::<color::Oklab>::new(dest.convert(Space::Oklab, v));
        lab(a).difference(lab(b))
    };
    const JND: f32 = 0.02;
    let (mut lo, mut hi, mut lo_in) = (0.0, c0, true);
    let current = to_rgb(c0);
    let mut clipped = dest.clip(current);
    if delta(clipped, current) < JND {
        return clipped;
    }
    while hi - lo > 1e-4 {
        let c = (lo + hi) / 2.0;
        let cur = to_rgb(c);
        if lo_in && inside(cur) {
            lo = c;
            continue;
        }
        clipped = dest.clip(cur);
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
    clipped
}

/// Whether `name` is one of CSS's named colours (or `transparent`,
/// `currentcolor`).
pub fn is_named(name: &str) -> bool {
    name.bytes().all(|b| b.is_ascii_alphabetic())
        && (name.eq_ignore_ascii_case("currentcolor")
            || system_role(name).is_some()
            || color::parse_color(name).is_ok())
}

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
    /// and Firefox agree), so a wrong space, a hue in the wrong unit, a
    /// mix that is not premultiplied or a percentage rule slip shows.
    #[test]
    fn colour_functions_match_the_spec_examples() {
        assert_eq!(lit("rebeccapurple"), [102, 51, 153, 255]);
        assert_eq!(lit("#f808"), [255, 136, 0, 136]);
        assert!(is_named("Transparent") && is_named("currentColor") && !is_named("rgb"));
        assert_eq!(lit("rgb(255 0 0 / 50%)"), [255, 0, 0, 128]);
        assert_eq!(lit("rgba(0, 128, 255, 0.25)"), [0, 128, 255, 64]);
        assert_eq!(lit("hsl(0.5turn, 100%, 50%)"), [0, 255, 255, 255]);
        // A component's math reduces before `color` reads it.
        assert_eq!(lit("rgb(calc(255 / 5) 0 0)"), [51, 0, 0, 255]);
        assert_eq!(lit("oklch(62.8% 0.2577 29.23)"), [255, 0, 0, 255]);
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
        // A `none` hue takes the other side's.
        assert_eq!(
            lit("color-mix(in oklch, oklch(0.7 0.15 none), oklch(0.7 0.15 140))"),
            lit("oklch(0.7 0.15 140)")
        );
        // An sRGB grey's hue is powerless in LCH: the mix keeps red's.
        assert_eq!(
            lit("color-mix(in lch, gray, red)"),
            lit("color-mix(in lch, lch(from gray l 0 none), red)")
        );
        // Relative colours: the complement, and alpha from a channel.
        assert_eq!(lit("hsl(from red calc(h + 180) s l)"), [0, 255, 255, 255]);
        assert_eq!(lit("rgb(from #ff8000 r g b / 50%)"), [255, 128, 0, 128]);
        assert_eq!(lit("rgb(from #204060 calc(r * 2) g b)"), [64, 64, 96, 255]);
        assert!(parse_str("rgb(1, 2 3)").is_err());
        assert!(parse_str("notacolour").is_err());
    }

    /// CSS Color 5 §3.3: each colour is mixed into the mix of those before
    /// it by its share of their combined percentage, so three equal colours
    /// average (a pairwise halving would give blue half: 64 64 128); omitted
    /// percentages share what is left, a sum under 100% is the alpha, and
    /// all 0% is transparent (no longer a parse error).
    #[test]
    fn color_mix_takes_any_number_of_colours() {
        assert_eq!(
            lit("color-mix(in srgb, red, lime, blue)"),
            [85, 85, 85, 255]
        );
        assert_eq!(
            lit("color-mix(in srgb, red 20%, lime 30%, blue)"),
            [51, 77, 128, 255]
        );
        assert_eq!(
            lit("color-mix(in srgb, red 10%, lime 10%, blue 20%)"),
            [64, 64, 128, 102]
        );
        // One colour: itself, in the space, its percentage the alpha.
        assert_eq!(lit("color-mix(in srgb, red 50%)"), [255, 0, 0, 128]);
        assert_eq!(lit("color-mix(in srgb, red 0%, blue 0%)")[3], 0);
        // Without a method the mix is in OKLab.
        assert_eq!(
            lit("color-mix(red, blue)"),
            lit("color-mix(in oklab, red, blue)")
        );
        assert_ne!(
            lit("color-mix(red, blue)"),
            lit("color-mix(in srgb, red, blue)")
        );
        // Two colours keep their CSS Color 4 results.
        assert_eq!(
            lit("color-mix(in srgb, red 30%, blue 20%)"),
            [153, 0, 102, 128]
        );
        assert!(parse_str("color-mix(in srgb)").is_err());
        assert!(parse_str("color-mix(in srgb, red 120%, blue)").is_err());
    }

    /// `contrast-color()` is white or black by the greater WCAG 2 ratio, so
    /// mid grey takes black (5.3:1 against white's 3.9:1, where a
    /// lightness-over-half rule takes white), and every ground reads at
    /// least √21:1 with its pick. A role's ground stays an expression
    /// until it is drawn.
    #[test]
    fn contrast_color_picks_the_ink_that_reads_most() {
        assert_eq!(lit("contrast-color(white)"), [0, 0, 0, 255]);
        assert_eq!(lit("contrast-color(black)"), [255, 255, 255, 255]);
        assert_eq!(lit("contrast-color(gray)"), [0, 0, 0, 255]);
        assert_eq!(lit("contrast-color(#2060c0)"), [255, 255, 255, 255]);
        // Its alpha is not read; the result is opaque.
        assert_eq!(
            lit("contrast-color(rgb(0 0 0 / 10%))"),
            [255, 255, 255, 255]
        );
        for r in (0..=255).step_by(17) {
            for g in (0..=255).step_by(17) {
                for b in (0..=255).step_by(51) {
                    let ground = Srgba::rgb(r, g, b);
                    let ink = ground.contrast_color();
                    assert!(ink.contrast(ground) >= 21f32.sqrt() - 1e-4, "{ground:?}");
                }
            }
        }
        let c = parse_str("contrast-color(var(--primary))").unwrap();
        assert!(c.literal().is_none());
        let mut roles = vec![];
        c.tokens(&mut roles);
        assert_eq!(roles, ["primary"]);
        // Inside a mix, against a style's role.
        struct Light;
        impl ColorTokens for Light {
            fn color(&self, _: &str) -> Option<Srgba> {
                Some(Srgba::rgb(250, 250, 240))
            }
            fn ink(&self) -> Srgba {
                Srgba::BLACK
            }
            fn dark(&self) -> bool {
                false
            }
        }
        let m = parse_str("color-mix(in srgb, contrast-color(var(--panel)), white)").unwrap();
        assert_eq!(m.resolve(&Context::of(&Light)).rgb, [128; 3]);
    }

    /// APCA-W3 0.0.98G-4g's own published examples (apca-w3's README).
    #[test]
    fn apca_matches_its_reference_values() {
        let grey = Srgba::rgb(136, 136, 136);
        let near = |a: f32, b: f32| (a - b).abs() < 0.01;
        assert!(
            near(grey.apca(Srgba::WHITE), 63.056),
            "{}",
            grey.apca(Srgba::WHITE)
        );
        assert!(near(Srgba::WHITE.apca(grey), -68.541));
        assert!(near(Srgba::BLACK.apca(Srgba::WHITE), 106.041));
        assert!(near(Srgba::WHITE.apca(Srgba::BLACK), -107.885));
        assert_eq!(grey.apca(Srgba::rgb(137, 137, 137)), 0.0);
    }

    /// Against P3 a colour outside sRGB keeps its gamut (Display P3 red is
    /// linear sRGB 1.2249, -0.0421, -0.0196 by CSS Color 4's matrices) and one
    /// outside P3 too is mapped onto P3's edge; sRGB colours draw as they
    /// do against sRGB, to the bit.
    #[test]
    fn a_p3_output_keeps_wide_colours_and_draws_srgb_ones_unchanged() {
        let cx = |gamut| Context {
            gamut,
            ..Context::of(&NoTokens)
        };
        let p3_red = parse_str("color(display-p3 1 0 0)").unwrap();
        assert!(p3_red.wide());
        // sRGB still gamut maps it, keeping lightness: a slightly lighter red.
        let srgb = p3_red.literal().unwrap().rgb;
        assert!(srgb[0] == 255 && srgb[1] < 16 && srgb[2] < 16, "{srgb:?}");
        let got = p3_red.linear(&cx(Gamut::P3));
        for (g, want) in got.iter().zip([1.2249, -0.0421, -0.0196, 1.0]) {
            assert!((g - want).abs() < 2e-3, "{got:?}");
        }
        assert_eq!(
            p3_red.linear(&cx(Gamut::Srgb)),
            p3_red.literal().unwrap().linear()
        );
        let beyond = parse_str("oklch(0.7 0.5 150 / 0.5)").unwrap();
        let [r, g, b, a] = beyond.linear(&cx(Gamut::P3));
        let p3 = Space::LinearSrgb.convert(Space::DisplayP3, [r, g, b]);
        assert!(
            p3.iter().all(|v| (-1e-3..=1.0 + 1e-3).contains(v)),
            "{p3:?}"
        );
        assert!(
            r < -1e-3 || g > 1.0 + 1e-3 || b < -1e-3,
            "outside sRGB: {r} {g} {b}"
        );
        assert_eq!(a, 0.5);
        for srgb in [
            "rgb(10 200 30)",
            "color-mix(in srgb, red, blue)",
            "oklch(0.6 0.1 40)",
        ] {
            let c = parse_str(srgb).unwrap();
            assert!(!c.wide(), "{srgb}");
            assert_eq!(
                c.linear(&cx(Gamut::P3)),
                c.resolve(&cx(Gamut::Srgb)).linear()
            );
        }
    }
}
