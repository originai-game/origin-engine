//! The descriptor at-rules a stylesheet declares beside its style rules:
//! `@font-face`, `@property`, `@counter-style` and `@position-try`, and
//! the `@import`ed stylesheets the cook supplied (docs/ui.md §3.2).
//! [`crate::doc::stylesheet`] reads them; the cascade, fonts, counters and
//! anchor positioning read what they declare.
use super::StyleDeclaration;

/// What a document's stylesheets declare besides rules.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sheet {
    /// `@import`ed stylesheets by the path written, as the cook read them
    /// from beside the document.
    pub imports: Vec<(String, String)>,
    /// Asset URLs resolved against the stylesheet that authored them.
    pub assets: std::collections::BTreeSet<String>,
    /// The http(s) images its `url()`s name, by request key
    /// ([`super::request::UrlRequest::key`]): the host fetches each and
    /// binds it under its key.
    pub remote: std::collections::BTreeSet<String>,
    pub font_faces: Vec<FontFace>,
    /// `@property` registrations by custom property name.
    pub properties: Vec<(String, Property)>,
    pub counter_styles: Vec<(String, CounterStyle)>,
    /// `@font-palette-values --name` rules in source order.
    pub palettes: Vec<FontPaletteValues>,
    /// `@position-try --name` fallbacks: their declarations, in order.
    pub position_tries: Vec<(String, Vec<StyleDeclaration>)>,
    /// Cascade layer names in precedence order, lowest first (a layer's
    /// sublayers before its own rules).
    pub layers: Vec<String>,
    /// Keyframes by key; conditional definitions use `<name>@<n>`.
    pub keyframes: Vec<(String, crate::motion::css::Keyframes)>,
    /// Definitions in source order: name, enclosing conditions and key.
    /// The last definition whose conditions hold defines the name.
    pub keyframe_rules: Vec<(String, super::AtQuery, String)>,
    /// `@custom-media` definitions in source order, unresolved: the last
    /// of a name defines it ([`super::media::Query::resolve`]).
    pub custom_media: Vec<(String, super::media::Query)>,
    /// `@function` custom functions by name; the last of a name defines it.
    pub functions: Vec<(String, Function)>,
}

/// One `@function --name(<params>) returns <type> { … }` (CSS Custom
/// Functions and Mixins 1 §2).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Function {
    pub params: Vec<Param>,
    /// The result's syntax; none: `*`.
    pub returns: Option<String>,
    /// Its custom properties and `result`, in order, each with the
    /// `@media` queries it sits under (`@supports` holds or not at cook).
    pub body: Vec<(Vec<super::media::Query>, StyleDeclaration)>,
}

/// A `@function` parameter: its name, its syntax and its default value.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Param {
    pub name: String,
    pub syntax: Option<String>,
    pub default: Option<String>,
}

/// One `@font-face`: a family the document names in `font-family`, drawn
/// with the first `src` the engine's font path loads.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FontFace {
    pub family: String,
    /// `url()` sources, resolved relative to the document, in order;
    /// `local()` names are kept with a `local:` prefix.
    pub src: Vec<String>,
    /// `font-weight` as a range (a single weight is both ends); `None`:
    /// `auto` (the initial value), a variable font's `wght` range, else
    /// 400.
    pub weight: Option<(f32, f32)>,
    pub style: String,
    /// `font-stretch` (`font-width`) as a range of percentages of the
    /// normal width (one width is both ends); `None`: `auto`, a variable
    /// font's `wdth` range, else 100 %.
    pub stretch: Option<(f32, f32)>,
    /// `unicode-range` as inclusive code point ranges; empty: all.
    pub unicode_range: Vec<(u32, u32)>,
    pub display: String,
}

/// One `@font-palette-values` (CSS Fonts 4 §9.1): a palette `font-palette`
/// names, for the families it lists.
#[derive(Clone, Debug, PartialEq)]
pub struct FontPaletteValues {
    pub name: String,
    /// `font-family`: the families it applies to, as written.
    pub families: Vec<String>,
    /// `base-palette` (`None`: the font's first palette).
    pub base: Option<crate::shape::PaletteBase>,
    /// `override-colors`: palette entries and their colours, straight sRGB.
    pub overrides: Vec<(u16, [u8; 4])>,
}

/// One `@property` registration (CSS Properties and Values API).
#[derive(Clone, Debug, PartialEq)]
pub struct Property {
    pub syntax: String,
    pub inherits: bool,
    pub initial: Option<String>,
}

/// One `@counter-style` (CSS Counter Styles 3).
#[derive(Clone, Debug, PartialEq)]
pub struct CounterStyle {
    pub system: System,
    pub symbols: Vec<String>,
    /// `additive-symbols`: (weight, symbol), weights descending.
    pub additive: Vec<(i64, String)>,
    pub negative: (String, String),
    pub prefix: String,
    pub suffix: String,
    /// `range`: inclusive bounds; empty: the system's own.
    pub range: Vec<(i64, i64)>,
    /// `pad`: minimum length and the symbol to pad with.
    pub pad: Option<(usize, String)>,
    pub fallback: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum System {
    Cyclic,
    Numeric,
    Alphabetic,
    Symbolic,
    Additive,
    /// `fixed [first]`.
    Fixed(i64),
    /// `extends <name>`: the other style's algorithm with these descriptors.
    Extends(String),
}

fn unquote(value: &str) -> String {
    let v = value.trim();
    if v.len() >= 2
        && (v.starts_with('"') && v.ends_with('"') || v.starts_with('\'') && v.ends_with('\''))
    {
        v[1..v.len() - 1].to_string()
    } else {
        v.to_string()
    }
}

/// Space- or comma-separated tokens, quoted strings kept whole.
fn words(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut p = cssparser::Parser::new(value);
    while let Ok(token) = p.next() {
        match token {
            cssparser::Token::QuotedString(s) | cssparser::Token::Ident(s) => {
                out.push(s.to_string())
            }
            cssparser::Token::Number { value, .. } => out.push(value.to_string()),
            cssparser::Token::Delim(c) => out.push(c.to_string()),
            cssparser::Token::Comma => {}
            cssparser::Token::Function(name) => {
                let name = name.to_string();
                let inner: Result<String, cssparser::ParseError<()>> = p.parse_nested_block(|p| {
                    let start = p.position();
                    while p.next().is_ok() {}
                    Ok(p.slice_from(start).to_string())
                });
                out.push(format!("{name}({})", inner.unwrap_or_default()));
            }
            _ => {}
        }
    }
    out
}

fn integer(word: &str, what: &str) -> Result<i64, String> {
    match word {
        "infinite" if what == "range" => Ok(i64::MAX),
        _ => word
            .parse::<f64>()
            .ok()
            .filter(|n| n.fract() == 0.0)
            .map(|n| n as i64)
            .ok_or_else(|| format!("@counter-style {what} takes integers, not {word}")),
    }
}

impl Sheet {
    /// `@font-face { … }`.
    pub fn font_face(&mut self, body: &str, base: &str) -> Result<(), String> {
        let mut face = FontFace {
            style: "normal".into(),
            display: "auto".into(),
            ..FontFace::default()
        };
        for d in super::variables::declarations(body)? {
            let v = d.source.trim();
            match d.name.as_str() {
                "font-family" => face.family = unquote(v),
                "src" => {
                    for source in v.split(',') {
                        let source = source.trim();
                        let inner = |f: &str| {
                            source
                                .strip_prefix(f)
                                .and_then(|r| r.split_once(')'))
                                .map(|(url, _)| unquote(url))
                        };
                        if let Some(url) = inner("url(") {
                            face.src.push(resolve(base, &url));
                        } else if let Some(name) = inner("local(") {
                            face.src.push(format!("local:{name}"));
                        } else {
                            return Err(format!(
                                "@font-face src takes url() or local(), not {source}"
                            ));
                        }
                    }
                }
                "font-weight" | "font-stretch" | "font-width" if v == "auto" => {}
                "font-weight" => {
                    let w: Vec<f32> = v
                        .split_whitespace()
                        .map(|w| match w {
                            "normal" => Ok(400.0),
                            "bold" => Ok(700.0),
                            w => w.parse().map_err(|_| {
                                format!("@font-face font-weight takes numbers, not {w}")
                            }),
                        })
                        .collect::<Result<_, _>>()?;
                    face.weight = match w.as_slice() {
                        [a] => Some((*a, *a)),
                        [a, b] => Some((*a, *b)),
                        _ => return Err("@font-face font-weight takes one or two weights".into()),
                    };
                }
                "font-stretch" | "font-width" => {
                    let w: Vec<f32> = v
                        .split_whitespace()
                        .map(|w| {
                            super::text_css::stretch_percent(w).ok_or_else(|| {
                                format!(
                                    "@font-face {} takes a width keyword or a percentage, not {w}",
                                    d.name
                                )
                            })
                        })
                        .collect::<Result<_, _>>()?;
                    face.stretch = match w.as_slice() {
                        [a] => Some((*a, *a)),
                        [a, b] => Some((a.min(*b), a.max(*b))),
                        _ => return Err(format!("@font-face {} takes one or two widths", d.name)),
                    };
                }
                "font-style" => face.style = v.split_whitespace().next().unwrap_or("normal").into(),
                "font-display" => face.display = v.into(),
                "unicode-range" => {
                    for range in v.split(',') {
                        let r = range
                            .trim()
                            .trim_start_matches(['U', 'u'])
                            .trim_start_matches('+');
                        let hex = |s: &str| {
                            u32::from_str_radix(s, 16)
                                .map_err(|_| format!("unicode-range {range} is not hexadecimal"))
                        };
                        face.unicode_range
                            .push(if let Some((a, b)) = r.split_once('-') {
                                (hex(a)?, hex(b)?)
                            } else if r.contains('?') {
                                (hex(&r.replace('?', "0"))?, hex(&r.replace('?', "F"))?)
                            } else {
                                (hex(r)?, hex(r)?)
                            });
                    }
                }
                // Descriptors that tune what the engine's font path already
                // decides (metrics overrides, features): accepted, unused.
                "font-feature-settings"
                | "font-variation-settings"
                | "ascent-override"
                | "descent-override"
                | "line-gap-override"
                | "size-adjust"
                | "font-named-instance"
                | "font-variant" => {}
                other => return Err(format!("@font-face has no descriptor {other}")),
            }
        }
        if face.family.is_empty() || face.src.is_empty() {
            return Err("@font-face needs font-family and src".into());
        }
        self.font_faces.push(face);
        Ok(())
    }

    /// `@property --name { syntax; inherits; initial-value }`.
    pub fn property(&mut self, name: &str, body: &str) -> Result<(), String> {
        if !name.starts_with("--") || name.len() < 3 {
            return Err(format!(
                "@property names a custom property (--name), not {name}"
            ));
        }
        let (mut syntax, mut inherits, mut initial) = (None, None, None);
        for d in super::variables::declarations(body)? {
            match d.name.as_str() {
                "syntax" => syntax = Some(unquote(&d.source)),
                "inherits" => {
                    inherits = Some(match d.source.trim() {
                        "true" => true,
                        "false" => false,
                        v => {
                            return Err(format!("@property inherits takes true or false, not {v}"));
                        }
                    })
                }
                "initial-value" => initial = Some(d.source.trim().to_string()),
                other => return Err(format!("@property has no descriptor {other}")),
            }
        }
        let (Some(syntax), Some(inherits)) = (syntax, inherits) else {
            return Err(format!("@property {name} needs syntax and inherits"));
        };
        if syntax != "*" && initial.is_none() {
            return Err(format!(
                "@property {name}: a syntax other than \"*\" needs initial-value"
            ));
        }
        if let Some(value) = &initial
            && !matches_syntax(&syntax, value)
        {
            return Err(format!(
                "@property {name}: initial-value {value} does not match syntax {syntax}"
            ));
        }
        self.properties.retain(|(n, _)| n != name);
        self.properties.push((
            name.to_string(),
            Property {
                syntax,
                inherits,
                initial,
            },
        ));
        Ok(())
    }

    /// `@counter-style name { … }`.
    pub fn counter_style(&mut self, name: &str, body: &str) -> Result<(), String> {
        if !super::syntax::identifier(name)
            || matches!(
                name,
                "none" | "inherit" | "initial" | "unset" | "decimal" | "disc" | "circle" | "square"
            )
        {
            return Err(format!("@counter-style cannot be named {name}"));
        }
        let mut style = CounterStyle {
            system: System::Symbolic,
            symbols: Vec::new(),
            additive: Vec::new(),
            negative: ("-".into(), String::new()),
            prefix: String::new(),
            suffix: ". ".into(),
            range: Vec::new(),
            pad: None,
            fallback: "decimal".into(),
        };
        for d in super::variables::declarations(body)? {
            let w = words(&d.source);
            match d.name.as_str() {
                "system" => {
                    style.system = match w.first().map(String::as_str) {
                        Some("cyclic") => System::Cyclic,
                        Some("numeric") => System::Numeric,
                        Some("alphabetic") => System::Alphabetic,
                        Some("symbolic") => System::Symbolic,
                        Some("additive") => System::Additive,
                        Some("fixed") => {
                            System::Fixed(w.get(1).map_or(Ok(1), |n| integer(n, "system"))?)
                        }
                        Some("extends") => System::Extends(
                            w.get(1).cloned().ok_or("system: extends needs a name")?,
                        ),
                        _ => {
                            return Err(format!(
                                "unknown @counter-style system {}",
                                d.source.trim()
                            ));
                        }
                    }
                }
                "symbols" => style.symbols = w,
                "additive-symbols" => {
                    for pair in d.source.split(',') {
                        let w = words(pair);
                        let [weight, symbol] = w.as_slice() else {
                            return Err("additive-symbols takes `<weight> <symbol>` pairs".into());
                        };
                        style
                            .additive
                            .push((integer(weight, "additive-symbols")?, symbol.clone()));
                    }
                }
                "negative" => {
                    style.negative = (
                        w.first().cloned().unwrap_or_default(),
                        w.get(1).cloned().unwrap_or_default(),
                    )
                }
                "prefix" => style.prefix = w.first().cloned().unwrap_or_default(),
                "suffix" => style.suffix = w.first().cloned().unwrap_or_default(),
                "range" => {
                    if d.source.trim() != "auto" {
                        for pair in d.source.split(',') {
                            let w = words(pair);
                            let [a, b] = w.as_slice() else {
                                return Err("range takes `<low> <high>` pairs or auto".into());
                            };
                            let low = if a == "infinite" {
                                i64::MIN
                            } else {
                                integer(a, "range")?
                            };
                            style.range.push((low, integer(b, "range")?));
                        }
                    }
                }
                "pad" => {
                    let [n, symbol] = w.as_slice() else {
                        return Err("pad takes `<length> <symbol>`".into());
                    };
                    style.pad = Some((integer(n, "pad")?.max(0) as usize, symbol.clone()));
                }
                "fallback" => {
                    style.fallback = w.first().cloned().unwrap_or_else(|| "decimal".into())
                }
                "speak-as" => {}
                other => return Err(format!("@counter-style has no descriptor {other}")),
            }
        }
        let needs = match &style.system {
            System::Additive => style.additive.is_empty(),
            System::Alphabetic | System::Numeric => style.symbols.len() < 2,
            System::Extends(_) => false,
            _ => style.symbols.is_empty(),
        };
        if needs {
            return Err(format!(
                "@counter-style {name} needs the symbols its system counts with"
            ));
        }
        self.counter_styles.retain(|(n, _)| n != name);
        self.counter_styles.push((name.to_string(), style));
        Ok(())
    }

    /// `@position-try --name { … }`: kept for anchor positioning, which
    /// tries each in `position-try-fallbacks` order.
    pub fn position_try(&mut self, name: &str, body: &str) -> Result<(), String> {
        if !name.starts_with("--") || name.len() < 3 {
            return Err(format!(
                "@position-try names a dashed identifier (--name), not {name}"
            ));
        }
        let declarations = super::variables::declarations(body)?;
        for d in &declarations {
            let positioning = d.name.starts_with("inset")
                || d.name.starts_with("margin")
                || matches!(d.name.as_str(), "top" | "right" | "bottom" | "left")
                || d.name.contains("width")
                || d.name.contains("height")
                || d.name.contains("inline-size")
                || d.name.contains("block-size")
                || matches!(
                    d.name.as_str(),
                    "align-self"
                        | "justify-self"
                        | "place-self"
                        | "position-anchor"
                        | "position-area"
                );
            if !positioning || d.important {
                return Err(format!(
                    "@position-try {name} takes inset, margin, sizing, self-alignment, position-anchor and position-area declarations without !important, not {}",
                    d.name
                ));
            }
        }
        self.position_tries.retain(|(n, _)| n != name);
        self.position_tries.push((name.to_string(), declarations));
        Ok(())
    }

    /// `@font-palette-values --name { … }`.
    pub fn font_palette_values(&mut self, name: &str, body: &str) -> Result<(), String> {
        let name = name.trim();
        if !name.starts_with("--") || name.len() < 3 || name.contains(char::is_whitespace) {
            return Err(format!(
                "@font-palette-values takes a dashed name (--name), not '{name}'"
            ));
        }
        let mut rule = FontPaletteValues {
            name: name.to_string(),
            families: Vec::new(),
            base: None,
            overrides: Vec::new(),
        };
        for d in super::variables::declarations(body)? {
            let v = d.source.trim();
            match d.name.as_str() {
                "font-family" => {
                    rule.families = v
                        .split(',')
                        .map(|f| unquote(f.trim()))
                        .filter(|f| !f.is_empty())
                        .collect();
                }
                "base-palette" => {
                    rule.base = Some(match v {
                        "light" => crate::shape::PaletteBase::Light,
                        "dark" => crate::shape::PaletteBase::Dark,
                        n => crate::shape::PaletteBase::Index(n.parse().map_err(|_| {
                            format!("base-palette takes light, dark or an index, not {n}")
                        })?),
                    });
                }
                "override-colors" => {
                    for entry in v.split(',') {
                        let entry = entry.trim();
                        let (index, colour) =
                            entry.split_once(char::is_whitespace).ok_or_else(|| {
                                format!("override-colors takes an index and a colour, not {entry}")
                            })?;
                        let index: u16 = index.parse().map_err(|_| {
                            format!("override-colors: '{index}' is not a palette index")
                        })?;
                        let c = crate::paint::svg::color(colour)
                            .map_err(|e| format!("override-colors: {e}"))?;
                        rule.overrides
                            .push((index, c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)));
                    }
                }
                other => {
                    return Err(format!(
                        "@font-palette-values takes font-family, base-palette and override-colors, not {other}"
                    ));
                }
            }
        }
        self.palettes.push(rule);
        Ok(())
    }

    /// The palette `font-palette: value` gives a text in `families`: `light`
    /// and `dark` as they say, a name as the last `@font-palette-values`
    /// of that name whose `font-family` lists one of the text's named
    /// families (none: `normal`, CSS Fonts 4 §9.2), and `palette-mix()`
    /// its palettes' entries mixed as `color-mix()` mixes colours (§9.1;
    /// in OKLab when it names no method).
    pub fn palette(
        &self,
        value: Option<&str>,
        families: Option<&[crate::FontFamily]>,
    ) -> crate::shape::Palette {
        use crate::shape::Palette;
        let Some(value) = value.map(str::trim).filter(|v| *v != "normal") else {
            return Palette::NONE;
        };
        let spec = og_css::tokens(value)
            .ok()
            .and_then(|t| match t.as_slice() {
                [t] => Some(self.palette_spec(t, families)),
                _ => None,
            })
            .flatten();
        spec.map_or(Palette::NONE, Palette::new)
    }

    /// One `font-palette` value as a palette (`None`: `normal`).
    fn palette_spec(
        &self,
        t: &og_css::Tok,
        families: Option<&[crate::FontFamily]>,
    ) -> Option<crate::shape::PaletteSpec> {
        use crate::shape::{PaletteBase, PaletteSpec};
        use og_css::Tok;
        let plain = |base| PaletteSpec::Cpal {
            base,
            overrides: Vec::new(),
        };
        match t {
            Tok::Ident(w) if w == "normal" => None,
            Tok::Ident(w) if w == "light" => Some(plain(PaletteBase::Light)),
            Tok::Ident(w) if w == "dark" => Some(plain(PaletteBase::Dark)),
            Tok::Ident(name) if name.starts_with("--") => {
                let named = |f: &str| {
                    families.unwrap_or_default().iter().any(
                        |n| matches!(n, crate::FontFamily::Named(n) if n.eq_ignore_ascii_case(f)),
                    )
                };
                self.palettes
                    .iter()
                    .rev()
                    .find(|r| &r.name == name && r.families.iter().any(|f| named(f)))
                    .map(|r| PaletteSpec::Cpal {
                        base: r.base.unwrap_or(PaletteBase::Index(0)),
                        overrides: r.overrides.clone(),
                    })
            }
            Tok::Func(f, args) if f == "palette-mix" => {
                let (method, items) = palette_mix_method(args)?;
                let mut out = Vec::new();
                for item in items {
                    let (palette, pct) = match item.as_slice() {
                        [p] => (p, None),
                        [p, Tok::Num(n, u)] | [Tok::Num(n, u), p] if u == "%" => {
                            let n: f32 = n.parse().ok().filter(|n| (0.0..=100.0).contains(n))?;
                            (p, Some((n * 10.0).round() as u16))
                        }
                        _ => return None,
                    };
                    let spec = self
                        .palette_spec(palette, families)
                        .unwrap_or(PaletteSpec::FIRST);
                    out.push((spec, pct));
                }
                // One palette mixed with itself is that palette (§9.1's
                // simplification), as long as nothing thins it.
                let given: u16 = out.iter().filter_map(|(_, p)| *p).sum();
                if out.iter().all(|(s, _)| *s == out[0].0)
                    && (given >= 1000 || out.iter().any(|(_, p)| p.is_none()))
                {
                    let one = out.swap_remove(0).0;
                    return (one != PaletteSpec::FIRST).then_some(one);
                }
                Some(PaletteSpec::Mix { method, items: out })
            }
            _ => None,
        }
    }

    pub fn counter(&self, name: &str) -> Option<&CounterStyle> {
        self.counter_styles
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, s)| s)
    }

    pub fn registered(&self, name: &str) -> Option<&Property> {
        self.properties
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, p)| p)
    }
}

/// A path written beside the document, from the document's own directory.
pub fn resolve(base: &str, path: &str) -> String {
    if path.contains(':') || path.starts_with('/') || path.starts_with('#') {
        return path.trim_start_matches('/').to_string();
    }
    let mut parts: Vec<&str> = base
        .rsplit_once('/')
        .map_or(Vec::new(), |(dir, _)| dir.split('/').collect());
    for c in path.split('/') {
        match c {
            "." | "" => {}
            ".." => {
                parts.pop();
            }
            c => parts.push(c),
        }
    }
    parts.join("/")
}

/// Rewrite URL tokens, including those nested in image-set(), without
/// interpreting quoted strings or comments as CSS functions.
pub fn rebase_urls(base: &str, css: &str) -> String {
    map_urls(css, &mut |url| {
        if url.starts_with('#') || url.contains(':') {
            url.to_owned()
        } else {
            format!("/{}", resolve(base, url))
        }
    })
}

/// Asset URLs in a declaration or stylesheet, excluding local fragments
/// and embedded data. The caller resolves them against their source file.
pub fn urls(css: &str) -> Vec<String> {
    let mut out = Vec::new();
    map_urls_in(
        css,
        &mut |url| {
            if !url.starts_with('#') && !url.contains(':') && !out.iter().any(|s| s == url) {
                out.push(url.to_owned());
            }
            url.to_owned()
        },
        false,
        true,
    );
    out
}

/// The http(s) images a declaration or stylesheet names, each as its
/// request key (super::request::UrlRequest::key), which the host fetches
/// and binds.
pub fn remote_urls(css: &str) -> Vec<String> {
    let mut out = Vec::new();
    map_urls_in(
        css,
        &mut |url| {
            if super::request::is_remote(url) && !out.iter().any(|s| s == url) {
                out.push(url.to_owned());
            }
            url.to_owned()
        },
        false,
        true,
    );
    out
}

fn map_urls(css: &str, map: &mut impl FnMut(&str) -> String) -> String {
    map_urls_in(css, map, false, false)
}

fn map_urls_in(
    css: &str,
    map: &mut dyn FnMut(&str) -> String,
    image_set: bool,
    assets_only: bool,
) -> String {
    use cssparser::{ToCss, Token};
    if image_set
        && assets_only
        && let Ok(image) = super::paint_css::border_source(&format!("image-set({css})"))
    {
        for url in image.urls() {
            map(url);
        }
        return css.to_owned();
    }
    let mut p = cssparser::Parser::new(css);
    let mut out = String::new();
    let mut copied = p.position();
    loop {
        let start = p.position();
        let Ok(token) = p.next_including_whitespace_and_comments().cloned() else {
            break;
        };
        let replacement = match token {
            Token::QuotedString(url) if image_set => {
                Some(Token::QuotedString(map(&url).into()).to_css_string())
            }
            Token::UnquotedUrl(url) => Some(format!(
                "url({})",
                Token::QuotedString(map(&url).into()).to_css_string()
            )),
            // `url("…" <request modifiers>)`: a remote URL and its
            // modifiers become one request key (super::request); a
            // modifier this cannot read stays as written, for the value's
            // own parse to name.
            Token::Function(ref name) if name.eq_ignore_ascii_case("url") => {
                let url = p.parse_nested_block(|p| {
                    let url = p.expect_string()?.to_string();
                    let mut modifiers = Vec::new();
                    while !p.is_exhausted() {
                        let m = match p.next()?.clone() {
                            Token::Function(m) => m,
                            _ => return Err(cssparser::ParseError::custom(())),
                        };
                        let value = p.parse_nested_block(|p| {
                            Ok::<_, cssparser::ParseError<()>>(match p.next()?.clone() {
                                Token::Ident(v) | Token::QuotedString(v) => v.to_string(),
                                _ => return Err(cssparser::ParseError::custom(())),
                            })
                        })?;
                        modifiers.push((m.to_string(), value));
                    }
                    Ok::<_, cssparser::ParseError<()>>((url, modifiers))
                });
                url.ok()
                    .and_then(|(url, modifiers)| {
                        if modifiers.is_empty() {
                            return Some(url);
                        }
                        super::request::UrlRequest::new(&url, &modifiers)
                            .ok()
                            .map(|r| r.key())
                    })
                    .map(|url| {
                        format!(
                            "url({})",
                            Token::QuotedString(map(&url).into()).to_css_string()
                        )
                    })
            }
            Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock => {
                let image_set = matches!(&token, Token::Function(name) if name.eq_ignore_ascii_case("image-set"));
                let opening = p.slice(start..p.position()).to_owned();
                let close = match token {
                    Token::SquareBracketBlock => ']',
                    Token::CurlyBracketBlock => '}',
                    _ => ')',
                };
                let inner = p
                    .parse_nested_block(|p| {
                        let start = p.position();
                        while p.next_including_whitespace_and_comments().is_ok() {}
                        Ok::<_, cssparser::ParseError<()>>(map_urls_in(
                            p.slice_from(start),
                            map,
                            image_set,
                            assets_only,
                        ))
                    })
                    .ok();
                inner.map(|inner| format!("{opening}{inner}{close}"))
            }
            _ => None,
        };
        if let Some(replacement) = replacement {
            out.push_str(p.slice(copied..start));
            out.push_str(&replacement);
            copied = p.position();
        }
    }
    out.push_str(p.slice_from(copied));
    out
}

/// Whether `value` matches a registered syntax (`<length> | auto`,
/// `<color>+`, `*`…): its components checked against the CSS types the
/// document can write.
pub fn matches_syntax(syntax: &str, value: &str) -> bool {
    let syntax = syntax.trim();
    if syntax == "*" {
        return true;
    }
    let value = value.trim();
    syntax.split('|').map(str::trim).any(|alt| {
        let (component, multiplier) = match alt.strip_suffix('+') {
            Some(c) => (c, Some(' ')),
            None => match alt.strip_suffix('#') {
                Some(c) => (c, Some(',')),
                None => (alt, None),
            },
        };
        match multiplier {
            Some(sep) => value
                .split(sep)
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .all(|v| component_matches(component, v)),
            None => component_matches(component, value),
        }
    })
}

fn component_matches(component: &str, value: &str) -> bool {
    let number = |v: &str| v.parse::<f64>().is_ok();
    let dimension = |v: &str, units: &[&str]| {
        units.iter().any(|u| v.strip_suffix(u).is_some_and(number))
            || v == "0"
            || v.starts_with("calc(")
    };
    const LENGTHS: &[&str] = &[
        "px", "em", "rem", "vw", "vh", "vmin", "vmax", "%", "cqw", "cqh", "cqi", "cqb", "cqmin",
        "cqmax", "ch", "ex", "lh", "pt", "pc", "in", "cm", "mm", "q",
    ];
    match component {
        "<length>" => dimension(value, LENGTHS) && !value.ends_with('%'),
        "<percentage>" => value.ends_with('%') && number(&value[..value.len() - 1]),
        "<length-percentage>" => dimension(value, LENGTHS),
        "<number>" => number(value),
        "<integer>" => value.parse::<i64>().is_ok(),
        "<angle>" => dimension(value, &["deg", "rad", "grad", "turn"]),
        "<time>" => dimension(value, &["ms", "s"]),
        "<resolution>" => dimension(value, &["dpi", "dpcm", "dppx", "x"]),
        "<color>" => {
            value.starts_with('#')
                || value.contains('(')
                || value.chars().all(|c| c.is_ascii_alphabetic())
        }
        "<custom-ident>" => super::syntax::identifier(value),
        "<string>" => value.starts_with('"') || value.starts_with('\''),
        "<url>" => value.starts_with("url("),
        "<image>" => value.starts_with("url(") || value.contains("gradient("),
        "<transform-function>" | "<transform-list>" => value.contains('('),
        ident => ident == value,
    }
}

/// `palette-mix()`'s `<color-interpolation-method>` (what follows `in`,
/// OKLab when it is not given) and its palettes; `None` when the method
/// is not one or no palette follows it.
pub(crate) fn palette_mix_method(
    args: &[Vec<og_css::Tok>],
) -> Option<(String, &[Vec<og_css::Tok>])> {
    use og_css::Tok;
    let (method, items) = match args.first().map(Vec::as_slice) {
        Some([Tok::Ident(i), rest @ ..]) if i == "in" => {
            og_css::color::interpolation(rest).ok()?;
            let words: Vec<String> = rest.iter().map(og_css::describe).collect();
            (words.join(" "), &args[1..])
        }
        _ => ("oklab".to_string(), args),
    };
    (!items.is_empty()).then_some((method, items))
}
