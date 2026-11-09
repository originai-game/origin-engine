use super::{AtQuery, Expr, Lit, Ty, css, parse_lit, schema};
use crate::{TextSpan, TextSpanStyle, TextSpanValue, Theme};

/// A scoped `text::part(name)[property=value]` rule, in cascade order.
#[derive(Clone, Debug, PartialEq)]
pub struct SpanRule {
    pub name: String,
    pub attributes: Vec<(String, Option<TextSpanValue>)>,
    pub query: AtQuery,
    pub condition: Option<Expr>,
    pub style: TextSpanStyle,
    pub declarations: Vec<super::StyleDeclaration>,
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
            color,
            font,
            weight,
            bold,
            italic,
            underline,
            strike,
            outline,
            motion,
            size,
            line_height,
            tracking_em,
            vertical_align
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
        declarations: super::variables::declarations(body)?,
        step: None,
    };
    let mut literal = String::new();
    for declaration in &rule.declarations {
        let name = declaration.name.as_str();
        if name.starts_with("--") {
            continue;
        }
        if !matches!(
            name,
            "font-weight"
                | "font-style"
                | "text-decoration-line"
                | "font-size"
                | "line-height"
                | "letter-spacing"
                | "vertical-align"
                | "-og-text-motion"
                | "color"
                | "font"
                | "font-family"
        ) {
            return Err(format!("unsupported text span property {name}"));
        }
        if !declaration.value.has_variables() {
            let value = declaration.value.substitute(&mut |_| None)?;
            literal.push_str(&format!("{name}: {value};"));
        }
    }
    // Validated here; which family is declared is known when it builds.
    apply_body(&mut rule, &literal, &|_| true)?;
    Ok(rule)
}

/// `declared` tells a family some loaded document's `@font-face` declares.
fn apply_body(
    rule: &mut SpanRule,
    body: &str,
    declared: &dyn Fn(&str) -> bool,
) -> Result<(), String> {
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
            // A span's size (CSS px), line height (px, or a multiple of its
            // size) and tracking (em, or px of its size).
            "font-size" => {
                rule.style.size = Some(
                    value
                        .strip_suffix("px")
                        .and_then(|v| v.trim().parse::<f32>().ok())
                        .filter(|v| *v > 0.0)
                        .ok_or("font-size takes <n>px")?,
                );
            }
            "line-height" => {
                let v = value.trim();
                rule.style.line_height = Some(match v.strip_suffix("px") {
                    Some(px) => px
                        .trim()
                        .parse::<f32>()
                        .map_err(|_| "line-height takes <n>px or a number")?,
                    None => {
                        let k = v
                            .parse::<f32>()
                            .map_err(|_| "line-height takes <n>px or a number")?;
                        k * rule.style.size.ok_or("a unitless line-height multiplies a span's font-size: write font-size first")?
                    }
                });
            }
            "letter-spacing" => {
                let v = value.trim();
                rule.style.tracking_em = Some(if let Some(em) = v.strip_suffix("em") {
                    em.trim()
                        .parse::<f32>()
                        .map_err(|_| "letter-spacing takes <n>em or <n>px")?
                } else if let Some(px) = v.strip_suffix("px") {
                    px.trim().parse::<f32>().map_err(|_| "letter-spacing takes <n>em or <n>px")?
                        / rule.style.size.ok_or("letter-spacing in px needs the span's font-size: write it first, or use em")?
                } else if v == "normal" {
                    0.0
                } else {
                    return Err("letter-spacing takes <n>em, <n>px or normal".into());
                });
            }
            "vertical-align" => {
                rule.style.vertical_align = Some(
                    crate::VerticalAlign::parse(&value)
                        .ok_or("vertical-align takes baseline, sub, super, text-top, text-bottom, middle, top, bottom, <n>px, <n>em or <n>%")?,
                );
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
            "font-family" => {
                // The first named family a loaded document declares, as an
                // element's (CSS Fonts 4 §5.1), else the generic family's role.
                let (names, role) = super::text_css::family_list(&value)?;
                let role = role
                    .strip_prefix("var(--font-")
                    .and_then(|r| r.strip_suffix(')'))
                    .and_then(crate::FontRole::from_name)
                    .unwrap_or_default();
                let first = names
                    .split(',')
                    .map(|n| n.trim().trim_matches('"'))
                    .find(|n| *n != "none" && !n.is_empty() && declared(n));
                rule.style.font = Some(match first {
                    Some(name) => crate::FontRole::Family(crate::Family::new(name, role)),
                    None => role,
                });
            }
            "color" | "font" => {
                for entry in css::declarations(&format!("{name}: {value};"), 1)? {
                    let (_, spec) = schema::prop(&entry.key).ok_or("unknown span text property")?;
                    match (spec.ty, parse_lit(spec.ty, &entry.values)?) {
                        (Ty::Paint, Lit::Paint(p)) => rule.style.color = Some(p.color_ref()),
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
                    "{name} does not style a text span; use color, font, font-family, font-size, line-height, letter-spacing, font-weight, font-style, text-decoration-line, vertical-align or -og-text-motion"
                ));
            }
        }
    }
    Ok(())
}

/// A run's style from text declarations (`name: value;`), as a
/// `::part` rule with them would give it.
pub(crate) fn style_of(
    body: &str,
    theme: &Theme,
    declared: &dyn Fn(&str) -> bool,
) -> Result<TextSpanStyle, String> {
    let mut rule = SpanRule {
        name: String::new(),
        attributes: Vec::new(),
        query: AtQuery::default(),
        condition: None,
        style: TextSpanStyle::default(),
        declarations: Vec::new(),
        step: None,
    };
    apply_body(&mut rule, body, declared)?;
    let mut style = TextSpanStyle::default();
    rule.apply(theme, &mut style);
    Ok(style)
}

pub(crate) fn resolve(
    rules: &[&SpanRule],
    span: &TextSpan,
    inherited: &super::variables::Computed,
    theme: &Theme,
    declared: &dyn Fn(&str) -> bool,
) -> (TextSpanStyle, super::build::cascade::Cascade) {
    let rules: Vec<_> = rules
        .iter()
        .filter(|rule| rule.matches(span))
        .map(|rule| super::StyleRule {
            layer: u32::MAX,
            starting: false,
            structure: None,
            query: AtQuery::default(),
            condition: None,
            props: Vec::new(),
            declarations: rule.declarations.clone(),
            line: 0,
        })
        .collect();
    let mut cascade = super::build::cascade::resolve_values(
        &rules,
        &[],
        inherited,
        theme,
        None,
        &|_| None,
        |_| true,
    );
    let mut rule = SpanRule {
        name: span.name.clone(),
        attributes: Vec::new(),
        query: AtQuery::default(),
        condition: None,
        style: TextSpanStyle::default(),
        declarations: Vec::new(),
        step: None,
    };
    for (name, value, _) in cascade.values.drain(..) {
        if let Err(error) = apply_body(&mut rule, &format!("{name}: {value};"), declared) {
            cascade.diagnostics.push(format!(
                "::part({}): {name} invalid at computed-value time: {error}",
                span.name
            ));
        }
    }
    let mut style = TextSpanStyle::default();
    rule.apply(theme, &mut style);
    (style, cascade)
}
