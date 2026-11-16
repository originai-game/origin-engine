//! Media Queries Level 4 predicates, evaluated in the document length space.
//! Unsupported features fail cooking instead of behaving like a false query.
use cssparser::{ParseError, Parser, Token};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Context {
    pub width: f32,
    pub height: f32,
    pub reduced_motion: bool,
    /// The player's colour vision setting (`-og-color-vision`).
    pub color_vision: og_draw::ColorVision,
    /// The primary pointer is coarse (a touch screen): `pointer: coarse`
    /// and `hover: none`, else `pointer: fine` and `hover: hover`.
    pub coarse_pointer: bool,
    /// The theme's scheme is dark (`prefers-color-scheme: dark`).
    pub dark: bool,
    /// The theme is its high-contrast variant (`prefers-contrast: more`).
    pub high_contrast: bool,
    /// `prefers-reduced-transparency: reduce`.
    pub reduced_transparency: bool,
    /// Device pixels per CSS pixel (`resolution`, in dppx).
    pub resolution: f32,
    /// The configured input mode is gamepad (`-og-input: pad`).
    pub pad: bool,
    /// What the surface presents: the Display P3 gamut (`color-gamut: p3`)
    /// and extended range (`dynamic-range: high`).
    pub display: Display,
}

/// The surface's gamut and range, as the renderer configured it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Display {
    pub wide_gamut: bool,
    pub high_range: bool,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
            reduced_motion: false,
            color_vision: og_draw::ColorVision::Typical,
            coarse_pointer: false,
            dark: false,
            high_contrast: false,
            reduced_transparency: false,
            resolution: 1.0,
            pad: false,
            display: Display::default(),
        }
    }
}

/// How many independent inputs a media query reads ([`Reads::axes`]).
pub const AXES: usize = 11;

impl Context {
    /// Which inputs differ from `old`, in [`Reads::axes`] order.
    pub fn changes(&self, old: &Self) -> [bool; AXES] {
        [
            old.width != self.width,
            old.height != self.height,
            old.reduced_motion != self.reduced_motion,
            old.color_vision != self.color_vision,
            old.coarse_pointer != self.coarse_pointer,
            old.dark != self.dark,
            old.high_contrast != self.high_contrast,
            old.reduced_transparency != self.reduced_transparency,
            old.resolution != self.resolution,
            old.pad != self.pad,
            old.display != self.display,
        ]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Reads {
    pub width: bool,
    pub height: bool,
    pub reduced_motion: bool,
    pub color_vision: bool,
    pub coarse_pointer: bool,
    pub scheme: bool,
    pub contrast: bool,
    pub transparency: bool,
    pub resolution: bool,
    pub pad: bool,
    pub display: bool,
}

impl Reads {
    pub fn extend(&mut self, other: Self) {
        for (axis, read) in other.axes().into_iter().enumerate() {
            if read {
                *self.axis(axis) = true;
            }
        }
    }

    /// The inputs read, in [`Context::changes`] order.
    pub fn axes(self) -> [bool; AXES] {
        [
            self.width,
            self.height,
            self.reduced_motion,
            self.color_vision,
            self.coarse_pointer,
            self.scheme,
            self.contrast,
            self.transparency,
            self.resolution,
            self.pad,
            self.display,
        ]
    }

    fn axis(&mut self, axis: usize) -> &mut bool {
        [
            &mut self.width,
            &mut self.height,
            &mut self.reduced_motion,
            &mut self.color_vision,
            &mut self.coarse_pointer,
            &mut self.scheme,
            &mut self.contrast,
            &mut self.transparency,
            &mut self.resolution,
            &mut self.pad,
            &mut self.display,
        ]
        .into_iter()
        .nth(axis)
        .unwrap()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Comparison {
    Less,
    LessEqual,
    Equal,
    GreaterEqual,
    Greater,
}

impl Comparison {
    fn test(self, a: f64, b: f64) -> bool {
        match self {
            Self::Less => a < b,
            Self::LessEqual => a <= b,
            Self::Equal => a == b,
            Self::GreaterEqual => a >= b,
            Self::Greater => a > b,
        }
    }
    fn reverse(self) -> Self {
        match self {
            Self::Less => Self::Greater,
            Self::LessEqual => Self::GreaterEqual,
            Self::Equal => Self::Equal,
            Self::GreaterEqual => Self::LessEqual,
            Self::Greater => Self::Less,
        }
    }
    fn direction(self) -> i8 {
        match self {
            Self::Less | Self::LessEqual => -1,
            Self::Equal => 0,
            _ => 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Feature {
    Width(Comparison, f64),
    Height(Comparison, f64),
    /// A container's `inline-size` and `block-size`: its width or its
    /// height as its own writing mode names them (CSS Containment 3 §6.1).
    InlineSize(Comparison, f64),
    BlockSize(Comparison, f64),
    AspectRatio(Comparison, f64, f64),
    Portrait(bool),
    ReducedMotion(bool),
    /// `(-og-color-vision: <mode>)`; `None`, the boolean form: any mode
    /// but `none`.
    ColorVision(Option<og_draw::ColorVision>),
    /// `pointer`/`any-pointer` (`coarse`: true, `fine`: false) and
    /// `hover`/`any-hover` (`none`: true, `hover`: false); `None`, the
    /// boolean form, which every pointing device matches.
    Coarse(Option<bool>),
    /// The engine's forced gamepad input mode.
    Pad,
    /// `prefers-color-scheme`: true for dark.
    Dark(bool),
    /// `prefers-contrast`: `more` (true), `no-preference` (false); `None`,
    /// the boolean form, any preference.
    Contrast(Option<bool>),
    ReducedTransparency(bool),
    /// `resolution` in dppx.
    Resolution(Comparison, f64),
    /// `(color-gamut: p3)`: the surface presents Display P3.
    WideGamut,
    /// `(dynamic-range: high)` and `video-dynamic-range`: the surface
    /// presents extended range.
    HighRange,
    /// A feature whose value is fixed for every play surface (the surface
    /// is a fast-updating, scripted, full-surface display with no forced
    /// colours, at least sRGB and standard range), already evaluated at parse.
    Fixed(bool),
    /// A container `style(--name)` (`None`: the property has a value) or
    /// `style(--name: value)` query, against the query container's
    /// computed custom property.
    Style(String, Option<String>),
    /// A container `scroll-state(<feature>: <value>)` query: some bit of
    /// `bits` is set among the container's [`scroll`] flags of `group`;
    /// `bits` 0 is the `none` value, no bit of the group.
    ScrollState {
        group: u16,
        bits: u16,
    },
    /// A container `style()` range (CSS Conditional 5 §6.2): custom
    /// property names or literal values, each pair joined by its
    /// comparison, compared once each reduces to one unit.
    StyleRange(Vec<String>, Vec<Comparison>),
    /// `(--name)`, a custom media query (Media Queries 5 §3), until
    /// [`Query::resolve`] puts its `@custom-media` definition in its place.
    Custom(String),
    /// A container `anchored(fallback[: <value>])` query (CSS Anchor
    /// Positioning 2 §2.3.1), against the container's applied
    /// `position-try-fallbacks` entry.
    Fallback(Fallback),
}

/// An `anchored()` `fallback` value: any entry applied (the boolean
/// form), none, an `@position-try` name and try-tactics in order, or a
/// `<position-area-query>`'s columns and rows (`None`: `any`).
#[derive(Clone, Debug, PartialEq)]
pub enum Fallback {
    Any,
    None,
    Tactics(Option<String>, Vec<crate::anchor::Flip>),
    Area(Option<(u8, u8)>, Option<(u8, u8)>),
}

impl Fallback {
    fn parse(text: &str) -> Result<Self, String> {
        use crate::anchor::{Try, parse_area, parse_tries};
        let text = text.trim().to_ascii_lowercase();
        if text == "none" {
            return Ok(Self::None);
        }
        let words: Vec<&str> = text.split_whitespace().collect();
        // `any` stands for one axis; one keyword with an axis of its own
        // leaves the other `any` (§2.3.2).
        let axis = |w: &str| {
            parse_area(w).ok().flatten().map(|a| match a {
                a if a.rows == (0, 2) && a.cols != (0, 2) => (Some(a.cols), None),
                a if a.cols == (0, 2) && a.rows != (0, 2) => (None, Some(a.rows)),
                // center, span-all, start, end: both axes, as written.
                a => (Some(a.cols), Some(a.rows)),
            })
        };
        let area = match words.as_slice() {
            ["any", "any"] | ["any"] => Some((None, None)),
            [w] if !w.starts_with("flip-") && !w.starts_with("--") => axis(w),
            ["any", w] | [w, "any"] => {
                let first = words[0] != "any";
                axis(w).map(|(c, r)| match (c, r) {
                    (Some(c), None) => (Some(c), None),
                    (None, Some(r)) => (None, Some(r)),
                    // A generic keyword's axis by its place: block, then
                    // inline.
                    (Some(x), Some(_)) if first => (None, Some(x)),
                    (Some(x), Some(_)) => (Some(x), None),
                    _ => (None, None),
                })
            }
            _ => None,
        };
        if let Some((cols, rows)) = area {
            return Ok(Self::Area(cols, rows));
        }
        match parse_tries(&text)?.as_slice() {
            [Try::Area(a)] => Ok(Self::Area(Some(a.cols), Some(a.rows))),
            [Try::Flips(f)] => Ok(Self::Tactics(None, f.clone())),
            [Try::Named(n, f)] => Ok(Self::Tactics(Some(n.clone()), f.clone())),
            _ => Err(format!(
                "anchored(fallback: {text}): write none, a --name with try-tactics, or a position-area"
            )),
        }
    }

    /// Whether `applied` (the container's applied fallback entry; none:
    /// its own placement) matches: tactics and their order alike, or the
    /// same physical area where the query does not say `any`.
    pub fn matches(&self, applied: Option<&crate::anchor::Try>) -> bool {
        use crate::anchor::Try;
        match (self, applied) {
            (Self::Any, applied) => applied.is_some(),
            (Self::None, applied) => applied.is_none(),
            (Self::Tactics(None, f), Some(Try::Flips(g))) => f == g,
            (Self::Tactics(Some(n), f), Some(Try::Named(m, g))) => n == m && f == g,
            (Self::Area(cols, rows), Some(Try::Area(a))) => {
                cols.is_none_or(|c| c == a.cols) && rows.is_none_or(|r| r == a.rows)
            }
            _ => false,
        }
    }
}

/// The scroll-state flags a container publishes, one bit per edge in the
/// order top, right, bottom, left (`x` axis bits 0 and 2, … for snapped).
pub mod scroll {
    pub const SCROLLABLE: u16 = 0xf;
    pub const SNAPPED: u16 = 0x3 << 4;
    pub const STUCK: u16 = 0xf << 6;
    pub const SCROLLED: u16 = 0xf << 10;
    pub const TOP: u16 = 1;
    pub const RIGHT: u16 = 2;
    pub const BOTTOM: u16 = 4;
    pub const LEFT: u16 = 8;
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Query {
    #[default]
    All,
    Feature(Feature),
    Not(Box<Query>),
    And(Vec<Query>),
    Or(Vec<Query>),
}

/// Which at-rule a condition belongs to: container conditions take the
/// size features and the logical `inline-size` and `block-size`, never
/// media types or user preferences.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Scope {
    Media,
    Container,
    /// Inside `style()`.
    Style,
    /// Inside `scroll-state()`.
    ScrollState,
    /// Inside `anchored()`.
    Anchored,
}

impl Query {
    /// An `@container` condition (the prelude after any container name).
    pub fn parse_container(source: &str) -> Result<Self, String> {
        Parser::new(source)
            .parse_entirely(|p| condition(p, true, Scope::Container))
            .map_err(|error| format!("invalid container query {source:?}: {error:?}"))
    }

    pub fn parse(source: &str) -> Result<Self, String> {
        let queries = Parser::new(source)
            .parse_comma_separated(query)
            .map_err(|error| format!("invalid media query {source:?}: {error:?}"))?;
        Ok(if queries.len() == 1 {
            queries.into_iter().next().unwrap()
        } else {
            Self::Or(queries)
        })
    }

    /// `@custom-media <name> <media-query-list> | true | false` after the
    /// at-keyword: its name and query, unresolved.
    pub fn parse_custom(source: &str) -> Result<(String, Self), String> {
        let source = source.trim();
        let (name, rest) = source
            .split_once(char::is_whitespace)
            .unwrap_or((source, ""));
        if !name.starts_with("--") || name.len() < 3 {
            return Err(format!(
                "@custom-media names a query --name, not {name}: @custom-media --wide (width > 800px);"
            ));
        }
        let query = match rest.trim() {
            "true" => Self::All,
            "false" => Self::Feature(Feature::Fixed(false)),
            rest => Self::parse(rest)?,
        };
        Ok((name.to_string(), query))
    }

    /// Each `(--name)` replaced by the last `@custom-media` definition of
    /// it in `defined`, itself resolved (Media Queries 5 §3.1). A name
    /// never defined, or one whose definition reaches itself, is an error.
    pub fn resolve(self, defined: &[(String, Self)]) -> Result<Self, String> {
        fn walk(
            q: Query,
            defined: &[(String, Query)],
            stack: &mut Vec<String>,
        ) -> Result<Query, String> {
            let all = |qs: Vec<Query>, stack: &mut Vec<String>| {
                qs.into_iter()
                    .map(|q| walk(q, defined, stack))
                    .collect::<Result<Vec<_>, _>>()
            };
            Ok(match q {
                Query::Feature(Feature::Custom(name)) => {
                    if stack.contains(&name) {
                        return Err(format!("@custom-media {name} refers to itself"));
                    }
                    let Some((_, q)) = defined.iter().rev().find(|(n, _)| *n == name) else {
                        return Err(format!(
                            "({name}) names no @custom-media; define it: @custom-media {name} (width > 800px);"
                        ));
                    };
                    stack.push(name);
                    let q = walk(q.clone(), defined, stack)?;
                    stack.pop();
                    q
                }
                Query::Not(q) => Query::Not(Box::new(walk(*q, defined, stack)?)),
                Query::And(qs) => Query::And(all(qs, stack)?),
                Query::Or(qs) => Query::Or(all(qs, stack)?),
                q => q,
            })
        }
        walk(self, defined, &mut Vec::new())
    }

    pub fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::All, b) => b,
            (a, Self::All) => a,
            (a, b) => Self::And(vec![a, b]),
        }
    }

    /// Evaluates the boolean structure with `leaf` answering each feature,
    /// short-circuiting as CSS does.
    pub fn test(&self, leaf: &mut dyn FnMut(&Feature) -> bool) -> bool {
        match self {
            Self::All => true,
            Self::Not(q) => !q.test(leaf),
            Self::And(q) => q.iter().all(|q| q.test(leaf)),
            Self::Or(q) => q.iter().any(|q| q.test(leaf)),
            Self::Feature(feature) => leaf(feature),
        }
    }

    /// Every feature the query names, evaluated or not.
    pub fn features(&self, out: &mut dyn FnMut(&Feature)) {
        match self {
            Self::All => {}
            Self::Not(q) => q.features(out),
            Self::And(qs) | Self::Or(qs) => qs.iter().for_each(|q| q.features(out)),
            Self::Feature(f) => out(f),
        }
    }

    /// Records only evaluated predicates. Short-circuited branches gain their
    /// subscriptions on the evaluation that first reaches them.
    pub fn matches(&self, context: Context, reads: &mut Reads) -> bool {
        self.test(&mut |feature| context.feature(feature, reads))
    }
}

impl Context {
    /// One media or container size feature against this context.
    pub fn feature(&self, feature: &Feature, reads: &mut Reads) -> bool {
        let context = self;
        {
            match *feature {
                Feature::Width(cmp, value) => {
                    reads.width = true;
                    cmp.test(context.width.into(), value)
                }
                Feature::Height(cmp, value) => {
                    reads.height = true;
                    cmp.test(context.height.into(), value)
                }
                // Horizontal writing: a container reads them through its
                // own writing mode first (`container::Condition::matches`).
                Feature::InlineSize(cmp, value) => {
                    reads.width = true;
                    cmp.test(context.width.into(), value)
                }
                Feature::BlockSize(cmp, value) => {
                    reads.height = true;
                    cmp.test(context.height.into(), value)
                }
                Feature::AspectRatio(cmp, numerator, denominator) => {
                    reads.width = true;
                    reads.height = true;
                    cmp.test(
                        f64::from(context.width) * denominator,
                        f64::from(context.height) * numerator,
                    )
                }
                Feature::Portrait(portrait) => {
                    reads.width = true;
                    reads.height = true;
                    (context.height >= context.width) == portrait
                }
                Feature::ReducedMotion(value) => {
                    reads.reduced_motion = true;
                    context.reduced_motion == value
                }
                Feature::Coarse(value) => {
                    reads.coarse_pointer = true;
                    value.is_none_or(|v| v == context.coarse_pointer)
                }
                Feature::Pad => {
                    reads.pad = true;
                    context.pad
                }
                Feature::Dark(value) => {
                    reads.scheme = true;
                    context.dark == value
                }
                Feature::Contrast(value) => {
                    reads.contrast = true;
                    value.map_or(context.high_contrast, |v| v == context.high_contrast)
                }
                Feature::ReducedTransparency(value) => {
                    reads.transparency = true;
                    context.reduced_transparency == value
                }
                Feature::Resolution(cmp, value) => {
                    reads.resolution = true;
                    cmp.test(context.resolution.into(), value)
                }
                Feature::WideGamut => {
                    reads.display = true;
                    context.display.wide_gamut
                }
                Feature::HighRange => {
                    reads.display = true;
                    context.display.high_range
                }
                Feature::Fixed(value) => value,
                // Container-only features never parse in a media query.
                Feature::Style(..)
                | Feature::StyleRange(..)
                | Feature::ScrollState { .. }
                | Feature::Fallback(_)
                | Feature::Custom(_) => false,
                Feature::ColorVision(value) => {
                    reads.color_vision = true;
                    match value {
                        Some(v) => context.color_vision == v,
                        None => context.color_vision != og_draw::ColorVision::Typical,
                    }
                }
            }
        }
    }
}

type Error = ParseError<String>;

fn query(input: &mut Parser<'_>) -> Result<Query, Error> {
    let state = input.state();
    let negate = input.try_parse(|p| p.expect_ident_matching("not")).is_ok();
    let only = !negate && input.try_parse(|p| p.expect_ident_matching("only")).is_ok();
    if let Ok(name) = input.try_parse(|p| p.expect_ident_cloned()) {
        if !name.eq_ignore_ascii_case("screen") && !name.eq_ignore_ascii_case("all") {
            return Err(ParseError::custom(format!(
                "unsupported media type {name}; use screen or all"
            )));
        }
        let value = if input.is_exhausted() {
            Query::All
        } else {
            input.expect_ident_matching("and")?;
            condition(input, false, Scope::Media)?
        };
        return Ok(if negate {
            Query::Not(Box::new(value))
        } else {
            value
        });
    }
    if only {
        return Err(ParseError::custom("only requires screen or all"));
    }
    input.reset(&state);
    condition(input, true, Scope::Media)
}

fn condition(input: &mut Parser<'_>, allow_or: bool, scope: Scope) -> Result<Query, Error> {
    if input.try_parse(|p| p.expect_ident_matching("not")).is_ok() {
        let q = Query::Not(Box::new(group(input, scope)?));
        input.expect_exhausted()?;
        return Ok(q);
    }
    let mut queries = vec![group(input, scope)?];
    let mut operator = None;
    while !input.is_exhausted() {
        let word = input.expect_ident_cloned()?.to_ascii_lowercase();
        if word != "and" && !(allow_or && word == "or") {
            return Err(ParseError::custom(format!(
                "expected {} between media conditions, found {word}",
                if allow_or { "and or or" } else { "and" }
            )));
        }
        if operator.as_ref().is_some_and(|previous| previous != &word) {
            return Err(ParseError::custom(
                "group mixed and/or conditions in parentheses",
            ));
        }
        operator = Some(word);
        queries.push(group(input, scope)?);
    }
    Ok(match operator.as_deref() {
        None => queries.pop().unwrap(),
        Some("or") => Query::Or(queries),
        _ => Query::And(queries),
    })
}

fn group(input: &mut Parser<'_>, scope: Scope) -> Result<Query, Error> {
    let scope = match input.next()?.clone() {
        Token::ParenthesisBlock => scope,
        Token::Function(name) if scope == Scope::Container => {
            match name.to_ascii_lowercase().as_str() {
                "style" => Scope::Style,
                "scroll-state" => Scope::ScrollState,
                "anchored" => Scope::Anchored,
                _ => {
                    return Err(ParseError::custom(format!(
                        "unsupported container function {name}(); use style(), scroll-state() or anchored()"
                    )));
                }
            }
        }
        token => {
            return Err(ParseError::custom(format!(
                "expected a parenthesized condition, found {token:?}"
            )));
        }
    };
    input.parse_nested_block(|input| {
        let state = input.state();
        let nested = matches!(input.next()?, Token::ParenthesisBlock) || {
            input.reset(&state);
            input.try_parse(|p| p.expect_ident_matching("not")).is_ok()
        };
        input.reset(&state);
        match scope {
            _ if nested => condition(input, true, scope),
            Scope::Style => style_feature(input),
            Scope::ScrollState => scroll_feature(input),
            Scope::Anchored => anchored_feature(input),
            _ => feature(input, scope),
        }
    })
}

/// `--name` or `--name: <value>` inside `style()`. Values compare as their
/// tokens with whitespace runs collapsed, as substituted custom properties
/// do.
fn style_feature(input: &mut Parser<'_>) -> Result<Query, Error> {
    if let Ok(range) = input.try_parse(style_range) {
        return Ok(range);
    }
    let name = input.expect_ident_cloned()?.to_string();
    if !name.starts_with("--") {
        return Err(ParseError::custom(format!(
            "style({name}): style queries take custom properties (--name)"
        )));
    }
    if input.is_exhausted() {
        return Ok(Query::Feature(Feature::Style(name, None)));
    }
    input.expect_colon()?;
    let start = input.position();
    while input.next().is_ok() {}
    let value = normalize(input.slice_from(start));
    Ok(Query::Feature(Feature::Style(name, Some(value))))
}

/// `a < b`, `a >= b`, `a = b` or `a < b <= c` inside `style()`.
fn style_range(input: &mut Parser<'_>) -> Result<Query, Error> {
    let mut operands = vec![style_operand(input)?];
    let mut comparisons = Vec::new();
    while !input.is_exhausted() {
        comparisons.push(comparison(input)?);
        operands.push(style_operand(input)?);
    }
    match comparisons[..] {
        [_] => {}
        [a, b] if a.direction() != 0 && a.direction() == b.direction() => {}
        [_, _] => {
            return Err(ParseError::custom(
                "style(): chained ranges must point in the same direction".to_string(),
            ));
        }
        _ => return Err(ParseError::custom("not a style() range".to_string())),
    }
    Ok(Query::Feature(Feature::StyleRange(operands, comparisons)))
}

/// One side of a `style()` range: a custom property name, a number,
/// dimension or percentage, or a math function.
fn style_operand(input: &mut Parser<'_>) -> Result<String, Error> {
    let start = input.position();
    match input.next()?.clone() {
        Token::Ident(name) if name.starts_with("--") => {}
        Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {}
        Token::Function(_) => input.parse_nested_block(|p| {
            while p.next().is_ok() {}
            Ok::<_, Error>(())
        })?,
        token => {
            return Err(ParseError::custom(format!(
                "a style() range compares custom properties and numbers, not {token:?}"
            )));
        }
    }
    Ok(input.slice_from(start).trim().to_string())
}

/// A style range's value: its number in its canonical unit (px, deg, ms,
/// dppx, %, or none), or `None` when it is not one number.
pub fn range_value(text: &str) -> Option<(f64, String)> {
    let reduced = og_css::calc::reduce(&format!("calc({})", text.trim())).ok()?;
    let reduced = reduced.trim();
    let at = reduced
        .find(|c: char| c.is_ascii_alphabetic() || c == '%')
        .unwrap_or(reduced.len());
    let (number, unit) = reduced.split_at(at);
    Some((number.parse().ok()?, unit.to_string()))
}

impl Comparison {
    /// Whether `values` hold each comparison in turn (`a < b <= c`).
    pub fn chain(comparisons: &[Self], values: &[f64]) -> bool {
        comparisons
            .iter()
            .zip(values.windows(2))
            .all(|(c, v)| c.test(v[0], v[1]))
    }
}

/// A custom property value as style queries compare it.
pub fn normalize(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `scrollable`, `snapped`, `stuck` or `scrolled`, with or without a
/// value, inside `scroll-state()`. Horizontal-tb, left-to-right writing:
/// block-start is the top and inline-start the left.
/// `fallback` or `fallback: <value>` inside `anchored()`.
fn anchored_feature(input: &mut Parser<'_>) -> Result<Query, Error> {
    let name = input.expect_ident_cloned()?.to_ascii_lowercase();
    if name != "fallback" {
        return Err(ParseError::custom(format!(
            "unsupported anchored() feature {name}; use fallback"
        )));
    }
    if input.is_exhausted() {
        return Ok(Query::Feature(Feature::Fallback(Fallback::Any)));
    }
    input.expect_colon()?;
    let start = input.position();
    while input.next().is_ok() {}
    Fallback::parse(input.slice_from(start))
        .map(|f| Query::Feature(Feature::Fallback(f)))
        .map_err(ParseError::custom)
}

fn scroll_feature(input: &mut Parser<'_>) -> Result<Query, Error> {
    use scroll::*;
    let name = input.expect_ident_cloned()?.to_ascii_lowercase();
    let group = match name.as_str() {
        "scrollable" => SCROLLABLE,
        "snapped" => SNAPPED,
        "stuck" => STUCK,
        "scrolled" => SCROLLED,
        _ => {
            return Err(ParseError::custom(format!(
                "unsupported scroll-state feature {name}; use scrollable, snapped, stuck or scrolled"
            )));
        }
    };
    if input.is_exhausted() {
        return Ok(Query::Feature(Feature::ScrollState { group, bits: group }));
    }
    input.expect_colon()?;
    let word = input.expect_ident_cloned()?.to_ascii_lowercase();
    let edges = if group == SNAPPED {
        match word.as_str() {
            "none" => Some(0),
            "x" | "inline" => Some(1),
            "y" | "block" => Some(2),
            "both" => Some(3),
            _ => None,
        }
    } else {
        match word.as_str() {
            "none" => Some(0),
            "top" | "block-start" => Some(TOP),
            "right" | "inline-end" => Some(RIGHT),
            "bottom" | "block-end" => Some(BOTTOM),
            "left" | "inline-start" => Some(LEFT),
            "x" | "inline" if group != STUCK => Some(LEFT | RIGHT),
            "y" | "block" if group != STUCK => Some(TOP | BOTTOM),
            _ => None,
        }
    };
    let Some(edges) = edges else {
        return Err(ParseError::custom(format!(
            "unsupported scroll-state({name}) value {word}"
        )));
    };
    let bits = edges << group.trailing_zeros();
    // `snapped: both` needs both axes; the others need any named edge.
    Ok(if group == SNAPPED && edges == 3 {
        Query::And(vec![
            Query::Feature(Feature::ScrollState {
                group,
                bits: 1 << 4,
            }),
            Query::Feature(Feature::ScrollState {
                group,
                bits: 2 << 4,
            }),
        ])
    } else {
        Query::Feature(Feature::ScrollState { group, bits })
    })
}

#[derive(Clone, Copy)]
enum Value {
    Length(f64),
    Number(f64),
    Ratio(f64, f64),
    /// A `<resolution>` in dppx.
    Resolution(f64),
}

fn value(input: &mut Parser<'_>) -> Result<Value, Error> {
    match input.next()?.clone() {
        Token::Dimension { value, unit, .. } => {
            let scale = match unit.to_ascii_lowercase().as_str() {
                "px" => 1.0,
                "em" | "rem" => 16.0,
                "in" => 96.0,
                "cm" => 96.0 / 2.54,
                "mm" => 96.0 / 25.4,
                "q" => 96.0 / 101.6,
                "pt" => 96.0 / 72.0,
                "pc" => 16.0,
                "dppx" | "x" => return Ok(Value::Resolution(value.into())),
                "dpi" => return Ok(Value::Resolution(f64::from(value) / 96.0)),
                "dpcm" => return Ok(Value::Resolution(f64::from(value) * 2.54 / 96.0)),
                _ => {
                    return Err(ParseError::custom(format!(
                        "unsupported media length unit {unit}; use px, em, rem or an absolute length"
                    )));
                }
            };
            Ok(Value::Length(f64::from(value) * scale))
        }
        Token::Number { value, .. } => {
            if input.try_parse(|p| p.expect_delim('/')).is_ok() {
                let denominator = input.expect_number()?;
                if value < 0.0 || denominator < 0.0 {
                    return Err(ParseError::custom(
                        "aspect-ratio values must be nonnegative",
                    ));
                }
                if denominator == 0.0 {
                    return Err(ParseError::custom(
                        "zero-denominator media ratios are unsupported; use a positive denominator",
                    ));
                }
                Ok(Value::Ratio(value.into(), denominator.into()))
            } else {
                Ok(Value::Number(value.into()))
            }
        }
        token => Err(ParseError::custom(format!(
            "expected a media length or ratio, found {token:?}"
        ))),
    }
}

fn comparison(input: &mut Parser<'_>) -> Result<Comparison, Error> {
    let token = input.next()?.clone();
    let inclusive = if matches!(token, Token::Delim('<' | '>')) {
        input
            .try_parse(|p| match p.next_including_whitespace()? {
                Token::Delim('=') => Ok(()),
                _ => Err(ParseError::<String>::custom("expected adjacent =")),
            })
            .is_ok()
    } else {
        false
    };
    match (token, inclusive) {
        (Token::Delim('<'), false) => Ok(Comparison::Less),
        (Token::Delim('<'), true) => Ok(Comparison::LessEqual),
        (Token::Delim('>'), false) => Ok(Comparison::Greater),
        (Token::Delim('>'), true) => Ok(Comparison::GreaterEqual),
        (Token::Delim('='), _) => Ok(Comparison::Equal),
        (token, _) => Err(ParseError::custom(format!(
            "expected <, <=, =, >= or >, found {token:?}"
        ))),
    }
}

fn ranged(name: &str, cmp: Comparison, value: Value) -> Result<Query, Error> {
    let feature = match (name, value) {
        ("width", Value::Length(n) | Value::Number(n))
            if matches!(value, Value::Length(_)) || n == 0.0 =>
        {
            Feature::Width(cmp, n)
        }
        ("height", Value::Length(n) | Value::Number(n))
            if matches!(value, Value::Length(_)) || n == 0.0 =>
        {
            Feature::Height(cmp, n)
        }
        (LOGICAL_INLINE, Value::Length(n) | Value::Number(n))
            if matches!(value, Value::Length(_)) || n == 0.0 =>
        {
            Feature::InlineSize(cmp, n)
        }
        (LOGICAL_BLOCK, Value::Length(n) | Value::Number(n))
            if matches!(value, Value::Length(_)) || n == 0.0 =>
        {
            Feature::BlockSize(cmp, n)
        }
        ("aspect-ratio", Value::Ratio(n, d)) => Feature::AspectRatio(cmp, n, d),
        ("aspect-ratio", Value::Number(n)) if n >= 0.0 => Feature::AspectRatio(cmp, n, 1.0),
        ("resolution", Value::Resolution(n)) => Feature::Resolution(cmp, n),
        (DPPX, Value::Number(n)) => Feature::Resolution(cmp, n),
        // Fixed integer features: 8 bits per colour component, no palette,
        // not monochrome, a bitmap display, 3D transforms (Compatibility
        // §3.1), one viewport segment (no host reports a fold).
        (
            "color"
            | "monochrome"
            | "color-index"
            | "grid"
            | "-webkit-transform-3d"
            | "horizontal-viewport-segments"
            | "vertical-viewport-segments",
            Value::Number(n),
        ) if n.fract() == 0.0 => {
            let actual = match name {
                "color" => 8.0,
                "monochrome" | "color-index" | "grid" => 0.0,
                _ => 1.0,
            };
            Feature::Fixed(cmp.test(actual, n))
        }
        _ => {
            return Err(ParseError::custom(format!(
                "unsupported media feature or value for {name}; width/height need a length, aspect-ratio a nonnegative ratio, resolution a resolution, -webkit-device-pixel-ratio a number, color/monochrome/color-index/grid/-webkit-transform-3d/*-viewport-segments an integer"
            )));
        }
    };
    Ok(Query::Feature(feature))
}

/// `-webkit-device-pixel-ratio` as parsed: `resolution` in dppx, a number.
const DPPX: &str = "@dppx";

/// A media feature that equals another here: Media Queries 4's deprecated
/// `device-width`, `device-height` and `device-aspect-ratio` read the play
/// surface (the output the game renders to, as a browser reports the
/// screen of a page it fills), and Compatibility's
/// `-webkit-device-pixel-ratio` is `resolution` in dppx (§3.2).
fn alias(name: String) -> String {
    let unprefixed = name.strip_prefix("-webkit-").unwrap_or(&name);
    let (prefix, bare) = ["min-", "max-"]
        .iter()
        .find_map(|p| unprefixed.strip_prefix(p).map(|rest| (*p, rest)))
        .unwrap_or(("", unprefixed));
    let feature = match (bare, unprefixed.len() < name.len()) {
        ("device-pixel-ratio", true) => DPPX,
        ("device-width", false) => "width",
        ("device-height", false) => "height",
        ("device-aspect-ratio", false) => "aspect-ratio",
        _ => return name,
    };
    format!("{prefix}{feature}")
}

/// A container condition's `inline-size` and `block-size` as parsed: names
/// no identifier spells, so a media query never reaches them.
const LOGICAL_INLINE: &str = "@inline-size";
const LOGICAL_BLOCK: &str = "@block-size";

/// The size feature a container condition names; `inline-size` and
/// `block-size` stay logical until the container's writing mode reads them.
fn logical(name: String, scope: Scope) -> Result<String, Error> {
    if scope == Scope::Media {
        return Ok(name);
    }
    let (prefix, bare) = ["min-", "max-"]
        .iter()
        .find_map(|p| name.strip_prefix(p).map(|rest| (*p, rest)))
        .unwrap_or(("", name.as_str()));
    match bare {
        "width" | "height" | "aspect-ratio" | "orientation" => Ok(name),
        "inline-size" => Ok(format!("{prefix}{LOGICAL_INLINE}")),
        "block-size" => Ok(format!("{prefix}{LOGICAL_BLOCK}")),
        _ => Err(ParseError::custom(format!(
            "unsupported container feature {name}; container queries take width, height, inline-size, block-size, aspect-ratio and orientation"
        ))),
    }
}

fn feature(input: &mut Parser<'_>, scope: Scope) -> Result<Query, Error> {
    if let Ok(name) = input.try_parse(|p| p.expect_ident_cloned()) {
        let name = logical(name.to_ascii_lowercase(), scope)?;
        let name = if scope == Scope::Media {
            alias(name)
        } else {
            name
        };
        if input.is_exhausted() && scope == Scope::Media && name.starts_with("--") {
            return Ok(Query::Feature(Feature::Custom(name)));
        }
        if input.is_exhausted() {
            return match name.as_str() {
                "width" | "height" | "aspect-ratio" | LOGICAL_INLINE | LOGICAL_BLOCK => {
                    ranged(&name, Comparison::Greater, Value::Number(0.0))
                }
                "orientation" => Ok(Query::All),
                "prefers-reduced-motion" => Ok(Query::Feature(Feature::ReducedMotion(true))),
                "-og-color-vision" => Ok(Query::Feature(Feature::ColorVision(None))),
                // Every surface has a pointer, fine or coarse; it hovers
                // only when fine (`hover: none` on a touch screen).
                "pointer" | "any-pointer" => Ok(Query::Feature(Feature::Coarse(None))),
                "hover" | "any-hover" => Ok(Query::Feature(Feature::Coarse(Some(false)))),
                "prefers-contrast" => Ok(Query::Feature(Feature::Contrast(None))),
                "prefers-reduced-transparency" => {
                    Ok(Query::Feature(Feature::ReducedTransparency(true)))
                }
                "resolution" => Ok(Query::Feature(Feature::Resolution(
                    Comparison::Greater,
                    0.0,
                ))),
                // True for any value but 0, `none` and a value the feature
                // names false (Media Queries 4 §2.4.4): every surface has a
                // scheme, a colour depth, a gamut, a range and updates.
                "color"
                | "color-gamut"
                | "update"
                | "scripting"
                | "display-mode"
                | "display-state"
                | "prefers-color-scheme"
                | "dynamic-range"
                | "device-posture"
                | "horizontal-viewport-segments"
                | "vertical-viewport-segments"
                | "-webkit-transform-3d" => Ok(Query::Feature(Feature::Fixed(true))),
                DPPX => Ok(Query::Feature(Feature::Resolution(
                    Comparison::Greater,
                    0.0,
                ))),
                "forced-colors"
                | "monochrome"
                | "color-index"
                | "grid"
                | "inverted-colors"
                | "prefers-reduced-data" => Ok(Query::Feature(Feature::Fixed(false))),
                _ => Err(ParseError::custom(format!(
                    "unsupported boolean media feature {name}"
                ))),
            };
        }
        if input.try_parse(|p| p.expect_colon()).is_ok() {
            if name == "-og-input" {
                let word = input.expect_ident_cloned()?;
                return if word.eq_ignore_ascii_case("pad") {
                    Ok(Query::Feature(Feature::Pad))
                } else {
                    Err(ParseError::custom(format!(
                        "unsupported -og-input value {word}; use pad; use pointer and hover for mouse and touch"
                    )))
                };
            }
            if name == "-og-color-vision" {
                let word = input.expect_ident_cloned()?.to_ascii_lowercase();
                return og_draw::ColorVision::from_name(&word)
                    .map(|v| Query::Feature(Feature::ColorVision(Some(v))))
                    .ok_or_else(|| {
                        ParseError::custom(format!(
                            "unsupported -og-color-vision value {word}; use none, protanopia, deuteranopia or tritanopia"
                        ))
                    });
            }
            if matches!(
                name.as_str(),
                "pointer" | "any-pointer" | "hover" | "any-hover"
            ) {
                let word = input.expect_ident_cloned()?.to_ascii_lowercase();
                let coarse = match (name.trim_start_matches("any-"), word.as_str()) {
                    ("pointer", "coarse") | ("hover", "none") => true,
                    ("pointer", "fine") | ("hover", "hover") => false,
                    _ => {
                        return Err(ParseError::custom(format!(
                            "unsupported {name} value {word}; pointer takes coarse or fine, hover takes hover or none"
                        )));
                    }
                };
                return Ok(Query::Feature(Feature::Coarse(Some(coarse))));
            }
            if let Some(feature) = keyword_feature(&name, input)? {
                return Ok(Query::Feature(feature));
            }
            if name == "orientation" || name == "prefers-reduced-motion" {
                let word = input.expect_ident_cloned()?.to_ascii_lowercase();
                return match (name.as_str(), word.as_str()) {
                    ("orientation", "portrait") => Ok(Query::Feature(Feature::Portrait(true))),
                    ("orientation", "landscape") => Ok(Query::Feature(Feature::Portrait(false))),
                    ("prefers-reduced-motion", "reduce") => {
                        Ok(Query::Feature(Feature::ReducedMotion(true)))
                    }
                    ("prefers-reduced-motion", "no-preference") => {
                        Ok(Query::Feature(Feature::ReducedMotion(false)))
                    }
                    _ => Err(ParseError::custom(format!(
                        "unsupported {name} value {word}"
                    ))),
                };
            }
            let (name, cmp) = if let Some(n) = name.strip_prefix("min-") {
                (n, Comparison::GreaterEqual)
            } else if let Some(n) = name.strip_prefix("max-") {
                (n, Comparison::LessEqual)
            } else {
                (name.as_str(), Comparison::Equal)
            };
            let value = value(input)?;
            return ranged(name, cmp, value);
        }
        let cmp = comparison(input)?;
        let value = value(input)?;
        return ranged(&name, cmp, value);
    }
    let left = value(input)?;
    let first = comparison(input)?;
    let name = logical(input.expect_ident_cloned()?.to_ascii_lowercase(), scope)?;
    let a = ranged(&name, first.reverse(), left)?;
    if input.is_exhausted() {
        return Ok(a);
    }
    let second = comparison(input)?;
    if first.direction() == 0 || first.direction() != second.direction() {
        return Err(ParseError::custom(format!(
            "{name}: chained ranges must point in the same direction"
        )));
    }
    let right = value(input)?;
    Ok(a.and(ranged(&name, second, right)?))
}

/// The keyword-valued user preference and display features; `None` when
/// `name` is not one of them (nothing is consumed then).
fn keyword_feature(name: &str, input: &mut Parser<'_>) -> Result<Option<Feature>, Error> {
    let values: &[&str] = match name {
        "prefers-color-scheme" => &["light", "dark"],
        "prefers-contrast" => &["no-preference", "more", "less", "custom"],
        "prefers-reduced-transparency" => &["no-preference", "reduce"],
        "forced-colors" => &["none", "active"],
        "color-gamut" => &["srgb", "p3", "rec2020"],
        "dynamic-range" | "video-dynamic-range" => &["standard", "high"],
        "display-mode" => &[
            "fullscreen",
            "standalone",
            "minimal-ui",
            "browser",
            "picture-in-picture",
        ],
        "display-state" => &["fullscreen", "maximized", "minimized", "normal"],
        "update" => &["none", "slow", "fast"],
        "scripting" => &["none", "initial-only", "enabled"],
        "inverted-colors" => &["none", "inverted"],
        "prefers-reduced-data" => &["no-preference", "reduce"],
        "device-posture" => &["continuous", "folded"],
        _ => return Ok(None),
    };
    let word = input.expect_ident_cloned()?.to_ascii_lowercase();
    if !values.contains(&word.as_str()) {
        return Err(ParseError::custom(format!(
            "unsupported {name} value {word}; use {}",
            values.join(", ")
        )));
    }
    let word = word.as_str();
    Ok(Some(match name {
        "prefers-color-scheme" => Feature::Dark(word == "dark"),
        // The theme has standard and high contrast only: `less` and
        // `custom` never match.
        "prefers-contrast" => match word {
            "more" => Feature::Contrast(Some(true)),
            "no-preference" => Feature::Contrast(Some(false)),
            _ => Feature::Fixed(false),
        },
        "prefers-reduced-transparency" => Feature::ReducedTransparency(word == "reduce"),
        // Every surface covers sRGB and standard range; P3 and high range
        // are what the renderer configured. Rec. 2020 is never presented.
        "color-gamut" if word == "p3" => Feature::WideGamut,
        "dynamic-range" | "video-dynamic-range" if word == "high" => Feature::HighRange,
        // The play surface fills the display area with no browser chrome,
        // updates every frame and runs scripts; no forced or inverted
        // colours or data saving; no host reports a fold, so the device
        // is flat (`continuous`).
        _ => Feature::Fixed(matches!(
            (name, word),
            ("forced-colors", "none")
                | ("color-gamut", "srgb")
                | ("dynamic-range" | "video-dynamic-range", "standard")
                | ("display-mode" | "display-state", "fullscreen")
                | ("update", "fast")
                | ("scripting", "enabled")
                | ("inverted-colors", "none")
                | ("prefers-reduced-data", "no-preference")
                | ("device-posture", "continuous")
        )),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_and_boolean_logic_use_css_boundaries() {
        let context = Context {
            width: 600.0,
            height: 400.0,
            reduced_motion: false,
            color_vision: og_draw::ColorVision::Typical,
            coarse_pointer: false,
            ..Context::default()
        };
        for (source, expected) in [
            ("(600px >= width > 599.5px)", true),
            ("(600px > width >= 599.5px)", false),
            ("only SCREEN and (min-width: 37.5em)", true),
            ("not screen and (width: 600px)", false),
            ("not ((width < 600px) or (height > 400px))", true),
            ("(width > -1px)", true),
            ("(aspect-ratio: 1.5)", true),
            ("(aspect-ratio: 3/2)", true),
            ("(aspect-ratio: 0/1)", false),
            ("(width) and (height) and (orientation)", true),
            ("(prefers-reduced-motion)", false),
            ("(prefers-reduced-motion: no-preference)", true),
        ] {
            assert_eq!(
                Query::parse(source)
                    .unwrap()
                    .matches(context, &mut Reads::default()),
                expected,
                "{source}"
            );
        }
    }

    #[test]
    fn reads_include_false_predicates_but_not_short_circuited_branches() {
        let q =
            Query::parse("(width < 500px) and (height > 100px), (prefers-reduced-motion)").unwrap();
        let mut reads = Reads::default();
        assert!(!q.matches(
            Context {
                width: 600.0,
                height: 400.0,
                reduced_motion: false,
                color_vision: og_draw::ColorVision::Typical,
                coarse_pointer: false,
                ..Context::default()
            },
            &mut reads
        ));
        assert_eq!(
            reads,
            Reads {
                width: true,
                height: false,
                reduced_motion: true,
                color_vision: false,
                coarse_pointer: false,
                ..Reads::default()
            }
        );
        let mut reads = Reads::default();
        assert!(q.matches(
            Context {
                width: 400.0,
                height: 300.0,
                reduced_motion: false,
                color_vision: og_draw::ColorVision::Typical,
                coarse_pointer: false,
                ..Context::default()
            },
            &mut reads
        ));
        assert_eq!(
            reads,
            Reads {
                width: true,
                height: true,
                reduced_motion: false,
                color_vision: false,
                coarse_pointer: false,
                ..Reads::default()
            }
        );
    }

    #[test]
    fn pointer_and_hover_follow_a_coarse_primary_pointer() {
        for (coarse, source, expected) in [
            (true, "(pointer: coarse)", true),
            (true, "(any-pointer: fine)", false),
            (true, "(hover: none)", true),
            // Boolean `hover` is false when its value is `none` (§7.2).
            (true, "(hover)", false),
            (false, "(hover)", true),
            (true, "(pointer)", true),
            (false, "(pointer: coarse)", false),
            (false, "(pointer: fine) and (hover: hover)", true),
            (false, "not (any-hover: none)", true),
        ] {
            let mut reads = Reads::default();
            let context = Context {
                width: 600.0,
                height: 400.0,
                reduced_motion: false,
                color_vision: og_draw::ColorVision::Typical,
                coarse_pointer: coarse,
                ..Context::default()
            };
            let q = Query::parse(source).unwrap();
            assert_eq!(q.matches(context, &mut reads), expected, "{source}");
            assert!(reads.coarse_pointer && !reads.width, "{source}");
        }
        assert!(Query::parse("(pointer: none)").is_err());
        assert!(Query::parse_container("(pointer: coarse)").is_err());
    }

    #[test]
    fn preference_and_display_features_read_the_theme_and_surface() {
        let context = Context {
            width: 600.0,
            height: 400.0,
            dark: true,
            high_contrast: true,
            resolution: 2.0,
            ..Context::default()
        };
        for (source, expected, read) in [
            ("(prefers-color-scheme: dark)", true, Some(5)),
            ("(prefers-color-scheme: light)", false, Some(5)),
            ("(prefers-contrast)", true, Some(6)),
            ("(prefers-contrast: more)", true, Some(6)),
            ("(prefers-contrast: no-preference)", false, Some(6)),
            ("(prefers-contrast: less)", false, None),
            ("(prefers-reduced-transparency)", false, Some(7)),
            (
                "(prefers-reduced-transparency: no-preference)",
                true,
                Some(7),
            ),
            ("(min-resolution: 2dppx)", true, Some(8)),
            ("(resolution > 192dpi)", false, Some(8)),
            ("(1.5x < resolution)", true, Some(8)),
            ("(forced-colors: none)", true, None),
            ("(forced-colors)", false, None),
            ("(color-gamut: srgb)", true, None),
            ("(color-gamut: p3)", false, Some(10)),
            ("(color-gamut: rec2020)", false, None),
            ("(dynamic-range: high)", false, Some(10)),
            ("(video-dynamic-range: standard)", true, None),
            ("(display-mode: fullscreen)", true, None),
            ("(display-state: fullscreen)", true, None),
            ("(display-state: normal)", false, None),
            ("(update: fast)", true, None),
            ("(scripting: enabled)", true, None),
            ("(min-color: 8)", true, None),
            ("(monochrome)", false, None),
        ] {
            let mut reads = Reads::default();
            let q = Query::parse(source).unwrap();
            assert_eq!(q.matches(context, &mut reads), expected, "{source}");
            let axes = reads.axes();
            assert_eq!(
                axes.iter().position(|&r| r),
                read,
                "{source} reads {axes:?}"
            );
        }
        let light = Context {
            dark: false,
            ..context
        };
        assert_eq!(
            light.changes(&context),
            [
                false, false, false, false, false, true, false, false, false, false, false
            ]
        );
        let p3 = Context {
            display: Display {
                wide_gamut: true,
                high_range: false,
            },
            ..context
        };
        let hdr = Context {
            display: Display {
                wide_gamut: true,
                high_range: true,
            },
            ..context
        };
        for (source, on_p3, on_hdr) in [
            ("(color-gamut: srgb)", true, true),
            ("(color-gamut: p3)", true, true),
            ("(color-gamut: rec2020)", false, false),
            ("(dynamic-range: high)", false, true),
            ("(dynamic-range: standard)", true, true),
            ("not (dynamic-range: high)", true, false),
        ] {
            let q = Query::parse(source).unwrap();
            assert_eq!(
                q.matches(p3, &mut Reads::default()),
                on_p3,
                "{source} on P3"
            );
            assert_eq!(
                q.matches(hdr, &mut Reads::default()),
                on_hdr,
                "{source} on HDR"
            );
        }
        assert!(hdr.changes(&p3)[10]);
        assert!(Query::parse_container("(prefers-color-scheme: dark)").is_err());
    }

    #[test]
    fn invalid_or_unsupported_queries_cannot_silently_match() {
        for source in [
            "",
            "phone",
            "(width: 12)",
            "(width: 50vw)",
            "(width: calc(1px + 2px))",
            "(color-gamut: cmyk)",
            "(prefers-color-scheme: sepia)",
            "(resolution: 2px)",
            "(orientation: diagonal)",
            "(width > 1px) or (height > 1px) and (width)",
            "(1px < width > 2px)",
            "(width < = 2px)",
            "only (width)",
            "(width),",
            "(aspect-ratio: -1/2)",
            "(aspect-ratio: 0/0)",
        ] {
            assert!(Query::parse(source).is_err(), "accepted {source:?}");
        }
    }
}
