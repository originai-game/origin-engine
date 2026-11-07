use super::{AtQuery, Expr, Lit, Paint, Ty, css, parse_lit, schema};
use crate::{TextSpan, TextSpanStyle, TextSpanValue, Theme, paint::ColorRef};

/// A scoped `text::part(name)[property=value]` rule, in cascade order.
#[derive(Clone, Debug, PartialEq)]
pub struct SpanRule {
    pub name: String,
    pub attributes: Vec<(String, Option<TextSpanValue>)>,
    pub query: AtQuery,
    pub condition: Option<Expr>,
    pub style: TextSpanStyle,
    /// A type-ramp role resolved from the active theme, never frozen at cook time.
    pub step: Option<String>,
}

impl SpanRule {
    pub fn matches(&self, span: &TextSpan) -> bool {
        self.name == span.name
            && self.attributes.iter().all(|(name, expected)| {
                span.properties
                    .get(name)
                    .is_some_and(|value| expected.as_ref().is_none_or(|expected| expected == value))
            })
    }

    pub fn apply(&self, theme: &Theme, out: &mut TextSpanStyle) {
        macro_rules! inherit {
            ($($field:ident),*) => { $(
                if self.style.$field.is_some() { out.$field = self.style.$field.clone(); }
            )* };
        }
        inherit!(
            color, font, weight, bold, italic, underline, strike, outline, motion
        );
        if let Some(step) = &self.step {
            out.step = crate::style_file::ramp_step(&mut theme.type_ramp.clone(), step).copied();
        }
    }
}

pub(super) fn parse(selector: &str, body: &str) -> Result<SpanRule, String> {
    let (name, mut rest) = selector
        .split_once(')')
        .ok_or("close ::part(name) with )")?;
    if !super::syntax::identifier(name) {
        return Err("::part takes one named text span".into());
    }
    let mut attributes = Vec::new();
    while !rest.is_empty() {
        let next = rest
            .strip_prefix('[')
            .ok_or("after ::part(name), use [property=value] selectors")?;
        let (test, tail) = next
            .split_once(']')
            .ok_or("close span property selector with ]")?;
        let (key, value) = match test.split_once('=') {
            None => (test.trim(), None),
            Some((key, value)) => {
                let value = value.trim();
                let scalar = if value.len() >= 2
                    && ((value.starts_with('\'') && value.ends_with('\''))
                        || (value.starts_with('"') && value.ends_with('"')))
                {
                    TextSpanValue::String(value[1..value.len() - 1].into())
                } else {
                    serde_json::from_str::<TextSpanValue>(value).map_err(
                        |_| "span property values are quoted strings, numbers or booleans",
                    )?
                };
                (key.trim(), Some(scalar))
            }
        };
        if !super::syntax::identifier(key) {
            return Err("span property selectors need an identifier".into());
        }
        attributes.push((key.into(), value));
        rest = tail;
    }
    let mut rule = SpanRule {
        name: name.into(),
        attributes,
        query: AtQuery::default(),
        condition: None,
        style: TextSpanStyle::default(),
        step: None,
    };
    for (name, value) in css::split_decls(body)? {
        match name.as_str() {
            "font-weight" => {
                let weight = match value.as_str() {
                    "normal" => 400,
                    "bold" => 700,
                    _ => value
                        .parse::<u16>()
                        .map_err(|_| "font-weight takes normal, bold, or 1..1000")?,
                };
                if !(1..=1000).contains(&weight) {
                    return Err("font-weight must be in 1..1000".into());
                }
                rule.style.weight = Some(weight);
                rule.style.bold = Some(weight >= 700);
            }
            "font-style" => {
                rule.style.italic = Some(match value.as_str() {
                    "normal" => false,
                    "italic" => true,
                    _ => return Err("font-style takes normal or italic".into()),
                })
            }
            "text-decoration-line" => {
                let words: Vec<_> = value.split_whitespace().collect();
                if words.is_empty()
                    || words
                        .iter()
                        .any(|v| !["none", "underline", "line-through"].contains(v))
                    || (words.contains(&"none") && words.len() > 1)
                {
                    return Err("text-decoration-line takes none, underline, line-through, or the latter two together".into());
                }
                rule.style.underline = Some(words.contains(&"underline"));
                rule.style.strike = Some(words.contains(&"line-through"));
            }
            "-og-text-motion" => {
                rule.style.motion = Some(match value.as_str() {
                    "none" => crate::markup::SpanFx::None,
                    "wave" => crate::markup::SpanFx::Wave,
                    "shake" => crate::markup::SpanFx::Shake,
                    "pulse" => crate::markup::SpanFx::Pulse,
                    _ => return Err("-og-text-motion takes none, wave, shake or pulse".into()),
                })
            }
            "color" | "font" | "font-family" => {
                for entry in css::declarations(&format!("{name}: {value};"), 1)? {
                    let (_, spec) = schema::prop(&entry.key).ok_or("unknown span text property")?;
                    match (spec.ty, parse_lit(spec.ty, &entry.values)?) {
                        (Ty::Paint, Lit::Paint(p)) => {
                            rule.style.color = Some(match p {
                                Paint::Token(name) => ColorRef::token(&name),
                                Paint::Color(color) => ColorRef::Literal(color),
                            })
                        }
                        (_, Lit::Str(s)) if entry.key == "step" => rule.step = Some(s),
                        (_, Lit::Str(s)) if entry.key == "font" => {
                            rule.style.font =
                                Some(crate::FontRole::from_name(&s).ok_or("unknown font role")?);
                        }
                        _ => return Err("unsupported span text declaration".into()),
                    }
                }
            }
            _ => {
                return Err(format!(
                    "{name} does not style a text span; use color, font, font-family, font-weight, font-style, text-decoration-line or -og-text-motion"
                ));
            }
        }
    }
    Ok(rule)
}
