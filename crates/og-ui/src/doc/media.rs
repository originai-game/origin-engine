//! Media Queries Level 4 predicates, evaluated in the document length space.
//! Unsupported features fail cooking instead of behaving like a false query.
use cssparser::{ParseError, Parser, Token};

#[derive(Clone, Copy, Debug)]
pub struct Context {
    pub width: f32,
    pub height: f32,
    pub reduced_motion: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Reads {
    pub width: bool,
    pub height: bool,
    pub reduced_motion: bool,
}

impl Reads {
    pub fn extend(&mut self, other: Self) {
        self.width |= other.width;
        self.height |= other.height;
        self.reduced_motion |= other.reduced_motion;
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
    AspectRatio(Comparison, f64, f64),
    Portrait(bool),
    ReducedMotion(bool),
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

    pub fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::All, b) => b,
            (a, Self::All) => a,
            (a, b) => Self::And(vec![a, b]),
        }
    }

    /// Records only evaluated predicates. Short-circuited branches gain their
    /// subscriptions on the evaluation that first reaches them.
    pub fn matches(&self, context: Context, reads: &mut Reads) -> bool {
        match self {
            Self::All => true,
            Self::Not(q) => !q.matches(context, reads),
            Self::And(q) => q.iter().all(|q| q.matches(context, reads)),
            Self::Or(q) => q.iter().any(|q| q.matches(context, reads)),
            Self::Feature(feature) => match *feature {
                Feature::Width(cmp, value) => {
                    reads.width = true;
                    cmp.test(context.width.into(), value)
                }
                Feature::Height(cmp, value) => {
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
            },
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
    input.expect_parenthesis_block()?;
    input.parse_nested_block(|input| {
        let state = input.state();
        let nested = matches!(input.next()?, Token::ParenthesisBlock) || {
            input.reset(&state);
            input.try_parse(|p| p.expect_ident_matching("not")).is_ok()
        };
        input.reset(&state);
        if nested {
            condition(input, true, scope)
        } else {
            feature(input, scope)
        }
    })
}

#[derive(Clone, Copy)]
enum Value {
    Length(f64),
    Number(f64),
    Ratio(f64, f64),
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
        ("aspect-ratio", Value::Ratio(n, d)) => Feature::AspectRatio(cmp, n, d),
        ("aspect-ratio", Value::Number(n)) if n >= 0.0 => Feature::AspectRatio(cmp, n, 1.0),
        _ => {
            return Err(ParseError::custom(format!(
                "unsupported media feature or value for {name}; width/height need a length, aspect-ratio a nonnegative ratio"
            )));
        }
    };
    Ok(Query::Feature(feature))
}

/// The size feature a container condition names: horizontal writing only,
/// so the inline axis is the width.
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
        "inline-size" => Ok(format!("{prefix}width")),
        "block-size" => Ok(format!("{prefix}height")),
        _ => Err(ParseError::custom(format!(
            "unsupported container feature {name}; container queries take width, height, inline-size, block-size, aspect-ratio and orientation"
        ))),
    }
}

fn feature(input: &mut Parser<'_>, scope: Scope) -> Result<Query, Error> {
    if let Ok(name) = input.try_parse(|p| p.expect_ident_cloned()) {
        let name = logical(name.to_ascii_lowercase(), scope)?;
        if input.is_exhausted() {
            return match name.as_str() {
                "width" | "height" | "aspect-ratio" => {
                    ranged(&name, Comparison::Greater, Value::Number(0.0))
                }
                "orientation" => Ok(Query::All),
                "prefers-reduced-motion" => Ok(Query::Feature(Feature::ReducedMotion(true))),
                _ => Err(ParseError::custom(format!(
                    "unsupported boolean media feature {name}"
                ))),
            };
        }
        if input.try_parse(|p| p.expect_colon()).is_ok() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_and_boolean_logic_use_css_boundaries() {
        let context = Context {
            width: 600.0,
            height: 400.0,
            reduced_motion: false,
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
                reduced_motion: false
            },
            &mut reads
        ));
        assert_eq!(
            reads,
            Reads {
                width: true,
                height: false,
                reduced_motion: true
            }
        );
        let mut reads = Reads::default();
        assert!(q.matches(
            Context {
                width: 400.0,
                height: 300.0,
                reduced_motion: false
            },
            &mut reads
        ));
        assert_eq!(
            reads,
            Reads {
                width: true,
                height: true,
                reduced_motion: false
            }
        );
    }

    #[test]
    fn invalid_or_unsupported_queries_cannot_silently_match() {
        for source in [
            "",
            "phone",
            "(width: 12)",
            "(width: 50vw)",
            "(width: calc(1px + 2px))",
            "(color)",
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
