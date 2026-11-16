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
            vertical_align,
            combine
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
                | "text-combine-upright"
                | "-og-text-motion"
                | "color"
                | "font"
                | "font-family"
        ) && !box_prop(name)
        {
            return Err(format!("unsupported text span property {name}"));
        }
        if !declaration.value.has_variables() {
            let value = declaration.value.substitute(&mut |_| None)?;
            literal.push_str(&format!("{name}: {value};"));
        }
    }
    // Validated here; which family is declared is known when it builds.
    apply_body(&mut rule, &literal, None)?;
    Ok(rule)
}

fn apply_body(rule: &mut SpanRule, body: &str, fonts: Option<&crate::Fonts>) -> Result<(), String> {
    for (name, value) in css::split_decls(body)? {
        match name.as_str() {
            "font" if !value.trim().starts_with("var(") => {
                let expanded = super::text_css::font(&value)?;
                let body = expanded
                    .into_iter()
                    .filter(|(name, _)| {
                        !matches!(name.as_str(), "font-stretch" | "font-variant-caps")
                    })
                    .map(|(name, value)| format!("{name}: {value};"))
                    .collect::<String>();
                apply_body(rule, &body, fonts)?;
            }
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
            "text-combine-upright" => {
                rule.style.combine = Some(
                    crate::layout::flow::combine(&value)
                        .ok_or("text-combine-upright takes none, all or digits <2..4>")?,
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
                let (names, role) = super::text_css::family_list(&value)?;
                let role = role
                    .strip_prefix("var(--font-")
                    .and_then(|r| r.strip_suffix(')'))
                    .and_then(crate::FontRole::from_name)
                    .unwrap_or_default();
                rule.style.font = Some(fonts.map_or(role, |fonts| fonts.family_list(&names, role)));
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
                let b = rule.style.inline_box.get_or_insert_default();
                if !inline_box(b, &name, &value)? {
                    return Err(format!(
                        "{name} does not style a text span; use color, font, font-family, font-size, line-height, letter-spacing, font-weight, font-style, text-decoration-line, vertical-align, text-combine-upright, -og-text-motion, or its box's background-color, border, padding, border-radius, box-shadow and box-decoration-break"
                    ));
                }
            }
        }
    }
    Ok(())
}

/// The properties of a span's own box ([`crate::rich::InlineBoxStyle`]),
/// each with its sides' longhands (`border-top`, `padding-left`, …).
const BOX: [&str; 8] = [
    "background",
    "background-color",
    "border",
    "padding",
    "border-radius",
    "box-shadow",
    "box-decoration-break",
    "-webkit-box-decoration-break",
];

const SIDES: [&str; 4] = ["top", "right", "bottom", "left"];

/// Whether `name` is a property of a span's own box: one of [`BOX`],
/// or a side's `padding-*`, `border-*` or its width, style or colour.
fn box_prop(name: &str) -> bool {
    let side = |r: &str| SIDES.contains(&r);
    BOX.contains(&name)
        || name.strip_prefix("padding-").is_some_and(side)
        || name.strip_prefix("border-").is_some_and(|rest| {
            matches!(rest, "width" | "style" | "color")
                || match rest.split_once('-') {
                    Some((s, part)) => side(s) && matches!(part, "width" | "style" | "color"),
                    None => side(rest),
                }
        })
}

/// A length in CSS px (`0` alone needs no unit).
fn px(t: &css::Tok) -> Result<f32, String> {
    let bad = || format!("'{}' is not a length in px", css::describe(t));
    match t {
        css::Tok::Num(n, u) => {
            let v: f32 = n.parse().map_err(|_| bad())?;
            match u.as_str() {
                "px" => Ok(v),
                "" if v == 0.0 => Ok(0.0),
                _ => Err(bad()),
            }
        }
        _ => Err(bad()),
    }
}

/// A border's width: `thin`, `medium`, `thick` or px.
fn border_width(t: &css::Tok) -> Result<f32, String> {
    match t {
        css::Tok::Ident(w) if w == "thin" => Ok(1.0),
        css::Tok::Ident(w) if w == "medium" => Ok(3.0),
        css::Tok::Ident(w) if w == "thick" => Ok(5.0),
        t => px(t).and_then(|v| {
            if v < 0.0 {
                Err("a border width is not negative".into())
            } else {
                Ok(v)
            }
        }),
    }
}

/// A colour (`None`: `currentColor`).
fn tok_colour(t: &css::Tok) -> Result<Option<crate::paint::ColorRef>, String> {
    if matches!(t, css::Tok::Ident(c) if c.eq_ignore_ascii_case("currentcolor")) {
        return Ok(None);
    }
    paint(&css::paint_vals(std::slice::from_ref(t))?)
}

fn paint(vals: &[og_core::ogt::Value]) -> Result<Option<crate::paint::ColorRef>, String> {
    match parse_lit(Ty::Paint, vals)? {
        Lit::Paint(p) => Ok(Some(p.color_ref())),
        _ => Err("not a colour".into()),
    }
}

fn colour(value: &str) -> Result<Option<crate::paint::ColorRef>, String> {
    match css::tokens(value)?.as_slice() {
        [t] => tok_colour(t),
        _ => Err(format!("'{value}' is not a colour")),
    }
}

/// A border's width, style and colour, any of them in any order (a
/// shorthand resets those it leaves out).
type BorderPart = (
    Option<f32>,
    Option<crate::decor::LineStyle>,
    Option<crate::paint::ColorRef>,
);

fn border(value: &str) -> Result<BorderPart, String> {
    let (mut w, mut style, mut c) = (None, None, None);
    let (mut seen_style, mut seen_colour) = (false, false);
    for t in css::tokens(value)? {
        if !seen_style
            && let css::Tok::Ident(word) = &t
            && super::boxes::STYLES.contains(&word.as_str())
        {
            style = crate::decor::LineStyle::from_css(word);
            seen_style = true;
        } else if w.is_none()
            && let Ok(v) = border_width(&t)
        {
            w = Some(v);
        } else if !seen_colour {
            c = tok_colour(&t)?;
            seen_colour = true;
        } else {
            return Err("a border takes one width, one style and one colour".into());
        }
    }
    Ok((w, style, c))
}

/// Sets the box property `name` of an inline element's box; `false`
/// when it is no box property.
fn inline_box(
    b: &mut crate::rich::InlineBoxStyle,
    name: &str,
    value: &str,
) -> Result<bool, String> {
    let sides = |v: &str| -> Result<Vec<css::Tok>, String> { css::tokens(v) };
    let side = |n: &str| SIDES.iter().position(|s| *s == n);
    match name {
        "background" | "background-color" => {
            let v = value.trim();
            b.background = match v {
                "none" | "transparent" => None,
                _ => Some(colour(v)?.ok_or(format!("{name} of a text span takes a colour"))?),
            };
        }
        "padding" => {
            let v = sides(value)?
                .iter()
                .map(px)
                .collect::<Result<Vec<_>, _>>()?;
            b.padding = super::boxes::four(name, v)?;
            if b.padding.iter().any(|p| *p < 0.0) {
                return Err("padding is not negative".into());
            }
        }
        "border-radius" => {
            let (h, v) = match value.split_once('/') {
                Some((h, v)) => (h, Some(v)),
                None => (value, None),
            };
            let h = super::boxes::four(
                name,
                sides(h)?.iter().map(px).collect::<Result<Vec<_>, _>>()?,
            )?;
            let v = match v {
                Some(v) => super::boxes::four(
                    name,
                    sides(v)?.iter().map(px).collect::<Result<Vec<_>, _>>()?,
                )?,
                None => h,
            };
            b.radius = [0, 1, 2, 3].map(|k| [h[k].max(0.0), v[k].max(0.0)]);
        }
        "box-shadow" => {
            b.shadows = Vec::new();
            for s in super::boxes::shadows(value)? {
                let c = match &s.color {
                    Some(vals) => paint(vals)?,
                    None => None,
                };
                b.shadows.push((
                    crate::decor::Shadow {
                        inset: s.inset,
                        offset: s.offset,
                        blur: s.blur,
                        spread: s.spread,
                        color: crate::Srgba::TRANSPARENT,
                    },
                    c,
                ));
            }
        }
        "box-decoration-break" | "-webkit-box-decoration-break" => {
            b.clone = match value.trim() {
                "slice" => false,
                "clone" => true,
                _ => return Err("box-decoration-break takes slice or clone".into()),
            };
        }
        "border" => {
            let (w, s, c) = border(value)?;
            b.border_width = [w; 4];
            b.border_style = [s; 4];
            b.border_color = [c.clone(), c.clone(), c.clone(), c];
        }
        "border-width" => {
            let v = sides(value)?
                .iter()
                .map(border_width)
                .collect::<Result<Vec<_>, _>>()?;
            b.border_width = super::boxes::four(name, v)?.map(Some);
        }
        "border-style" => {
            let v = sides(value)?
                .iter()
                .map(|t| {
                    super::boxes::want_style(t, false)
                        .map(|s| crate::decor::LineStyle::from_css(&s))
                })
                .collect::<Result<Vec<_>, _>>()?;
            b.border_style = super::boxes::four(name, v)?;
        }
        "border-color" => {
            let v = sides(value)?
                .iter()
                .map(tok_colour)
                .collect::<Result<Vec<_>, _>>()?;
            b.border_color = super::boxes::four(name, v)?;
        }
        _ => {
            if let Some(k) = name.strip_prefix("padding-").and_then(side) {
                let [t] = sides(value)?
                    .try_into()
                    .map_err(|_| format!("{name} takes one length"))?;
                b.padding[k] = px(&t)?;
                return Ok(true);
            }
            let Some(rest) = name.strip_prefix("border-") else {
                return Ok(false);
            };
            let (s, part) = rest.split_once('-').unwrap_or((rest, ""));
            let Some(k) = side(s) else {
                return Ok(false);
            };
            match part {
                "" => {
                    let (w, s, c) = border(value)?;
                    b.border_width[k] = w;
                    b.border_style[k] = s;
                    b.border_color[k] = c;
                }
                "width" => {
                    let [t] = sides(value)?
                        .try_into()
                        .map_err(|_| format!("{name} takes one width"))?;
                    b.border_width[k] = Some(border_width(&t)?);
                }
                "style" => {
                    let [t] = sides(value)?
                        .try_into()
                        .map_err(|_| format!("{name} takes one style"))?;
                    b.border_style[k] =
                        crate::decor::LineStyle::from_css(&super::boxes::want_style(&t, false)?);
                }
                "color" => b.border_color[k] = colour(value)?,
                _ => return Ok(false),
            }
        }
    }
    Ok(true)
}

/// A run's style from text declarations (`name: value;`), as a
/// `::part` rule with them would give it.
pub fn style_of(body: &str, theme: &Theme, fonts: &crate::Fonts) -> Result<TextSpanStyle, String> {
    let mut rule = SpanRule {
        name: String::new(),
        attributes: Vec::new(),
        query: AtQuery::default(),
        condition: None,
        style: TextSpanStyle::default(),
        declarations: Vec::new(),
        step: None,
    };
    apply_body(&mut rule, body, Some(fonts))?;
    let mut style = TextSpanStyle::default();
    rule.apply(theme, &mut style);
    Ok(style)
}

pub(crate) fn resolve(
    rules: &[&SpanRule],
    span: &TextSpan,
    inherited: &super::variables::Computed,
    theme: &Theme,
    fonts: &crate::Fonts,
) -> (TextSpanStyle, super::build::cascade::Cascade) {
    let rules: Vec<_> = rules
        .iter()
        .filter(|rule| rule.matches(span))
        .map(|rule| super::StyleRule {
            inline: false,
            context: super::TreeContext::Tree,
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
    let mut cascade =
        super::build::cascade::resolve_values(&rules, &[], inherited, None, &|_| None, |_| true);
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
        if let Err(error) = apply_body(&mut rule, &format!("{name}: {value};"), Some(fonts)) {
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
