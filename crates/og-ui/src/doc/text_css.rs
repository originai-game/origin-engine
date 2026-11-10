//! CSS text and font properties on document elements (CSS Fonts 4, CSS
//! Text 3 and 4, CSS Text Decoration 3, CSS Color Adjust 1, CSS UI 4).
//!
//! Most are inherited, so they ride the inherited custom-property channel
//! as SVG's presentation attributes do: an element that declares one
//! writes its computed value into its computed map under ` text <name>`, a
//! name no stylesheet can write, and every element reads the effective
//! value from its own map, recording the read so a parent's change
//! re-resolves the texts under it and nothing else. Lengths are computed
//! where they are declared: `font-size: 1.5em` is 1.5 times the parent's
//! size, and a child inherits the px, not the `em` (CSS Fonts 4 §2.5).
//!
//! The effective values meet the text where it is built: size, weight,
//! line height and tracking change the text's type step; a style,
//! decoration, stroke, word spacing or stretch makes it a one-span rich
//! text; a transform rewrites what is shown and a declared white space
//! what is laid out.

use super::StyleDeclaration;
use super::variables;
use crate::Srgba;
use crate::theme::TypeStep;

/// The inherited properties, by their CSS names.
pub(crate) const INHERITED: &[&str] = &[
    "font-family",
    "font-size",
    "font-weight",
    "font-style",
    "font-stretch",
    "line-height",
    "letter-spacing",
    "word-spacing",
    "text-transform",
    "white-space-collapse",
    "text-wrap-mode",
    "text-wrap-style",
    "text-align",
    "text-align-last",
    "text-indent",
    "tab-size",
    "hyphens",
    "word-break",
    "overflow-wrap",
    "text-justify",
    "font-variant-caps",
    "font-variant-numeric",
    "font-variant-ligatures",
    "font-variant-east-asian",
    "font-variant-alternates",
    "font-variant-position",
    "font-variant-emoji",
    "font-feature-settings",
    "font-variation-settings",
    "font-kerning",
    "font-optical-sizing",
    "font-synthesis-weight",
    "font-synthesis-style",
    "font-synthesis-small-caps",
    "font-synthesis-position",
    "font-size-adjust",
    "font-language-override",
    "text-underline-position",
    "text-underline-offset",
    "text-decoration-skip-ink",
    "-webkit-text-stroke-width",
    "-webkit-text-stroke-color",
    "-webkit-text-fill-color",
    "paint-order",
    "caret-color",
    "accent-color",
    "color-scheme",
    // The named families of `font-family` (its generic family is the
    // `font` role): a quoted, comma-separated list or `none`.
    "-og-font-name",
];

/// The text properties that do not inherit (CSS Text Decoration 3:
/// decorations propagate to the text inside instead).
pub(crate) const LOCAL: &[&str] = &[
    "text-decoration-line",
    "text-decoration-style",
    "text-decoration-color",
    "text-decoration-thickness",
    "vertical-align",
];

/// Whether this module owns `name` (after shorthands expand).
pub(crate) fn longhand(name: &str) -> bool {
    INHERITED.contains(&name) || LOCAL.contains(&name)
}

/// The computed-map name an inherited value is carried under.
pub(crate) fn key(name: &str) -> String {
    format!(" text {name}")
}

fn declaration(name: &str, value: &str, important: bool) -> Result<StyleDeclaration, String> {
    Ok(StyleDeclaration {
        name: name.into(),
        value: variables::Value::parse(value)?,
        source: value.into(),
        important,
        pending_shorthand: None,
    })
}

/// Expands the shorthands this module owns into their longhands (each
/// omitted part takes its initial value, as CSS's shorthands do) and
/// checks a longhand written without `var()`; other declarations pass.
/// Font shorthands with variables expand after substitution in the cascade.
pub(crate) fn expand(d: StyleDeclaration) -> Result<Vec<StyleDeclaration>, String> {
    let name = match d.name.as_str() {
        "word-wrap" => "overflow-wrap",
        "font-width" => "font-stretch",
        n => n,
    };
    let shorthand = matches!(
        name,
        "font"
            | "font-variant"
            | "font-synthesis"
            | "white-space"
            | "text-wrap"
            | "text-decoration"
            | "-webkit-text-stroke"
    );
    if name == "font" && d.value.has_variables() {
        return Ok(vec![d]);
    }
    if name == "font-family" {
        let v = d.source.trim();
        if !d.value.has_variables() && !wide(v) {
            family_list(v)?;
        }
        return Ok(vec![d]);
    }
    if !shorthand {
        if longhand(name) && !d.value.has_variables() {
            check(name, &d.source)?;
        }
        return Ok(vec![StyleDeclaration {
            name: name.into(),
            ..d
        }]);
    }
    if d.value.has_variables() {
        return Err(format!(
            "{name}: write var() in its longhands, not in the shorthand"
        ));
    }
    let v = d.source.trim();
    let parts = shorthand_parts(name, v)?;
    parts
        .into_iter()
        .map(|(n, v)| {
            if longhand(&n) {
                check(&n, &v)?;
            }
            declaration(&n, &v, d.important)
        })
        .collect()
}

fn shorthand_parts(name: &str, v: &str) -> Result<Vec<(String, String)>, String> {
    let words: Vec<&str> = v.split_whitespace().collect();
    let all = |props: &[(&str, &str)]| -> Vec<(String, String)> {
        props
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    };
    Ok(match name {
        "white-space" => {
            // CSS Text 4 §3.3: the legacy keywords.
            let (collapse, mode) = match v {
                "normal" => ("collapse", "wrap"),
                "pre" => ("preserve", "nowrap"),
                "nowrap" => ("collapse", "nowrap"),
                "pre-wrap" => ("preserve", "wrap"),
                "break-spaces" => ("break-spaces", "wrap"),
                "pre-line" => ("preserve-breaks", "wrap"),
                _ => {
                    let mut c = "collapse";
                    let mut m = "wrap";
                    for w in &words {
                        match *w {
                            "collapse" | "discard" | "preserve" | "preserve-breaks"
                            | "preserve-spaces" | "break-spaces" => c = w,
                            "wrap" | "nowrap" => m = w,
                            _ => return Err(format!("white-space: '{w}' is not a value")),
                        }
                    }
                    (c, m)
                }
            };
            all(&[("white-space-collapse", collapse), ("text-wrap-mode", mode)])
        }
        "text-wrap" => {
            let (mut mode, mut style) = ("wrap", "auto");
            for w in &words {
                match *w {
                    "wrap" | "nowrap" => mode = w,
                    "auto" | "balance" | "pretty" | "stable" => style = w,
                    _ => return Err(format!("text-wrap: '{w}' is not a value")),
                }
            }
            all(&[("text-wrap-mode", mode), ("text-wrap-style", style)])
        }
        "text-decoration" => {
            let (mut lines, mut style, mut color, mut thick) = (
                Vec::new(),
                "solid".to_string(),
                "currentcolor".to_string(),
                "auto".to_string(),
            );
            for item in split_top(v) {
                match item.as_str() {
                    "none" => {}
                    "underline" | "overline" | "line-through" | "blink" => lines.push(item),
                    "solid" | "double" | "dotted" | "dashed" | "wavy" => style = item,
                    "auto" | "from-font" => thick = item,
                    t if t.ends_with("px") || t.ends_with("em") || t.ends_with('%') || t == "0" => {
                        thick = item
                    }
                    _ => {
                        crate::css_color::parse_str(&item)
                            .map_err(|_| format!("text-decoration: '{item}' is not a value"))?;
                        color = item;
                    }
                }
            }
            let lines = if lines.is_empty() {
                "none".to_string()
            } else {
                lines.join(" ")
            };
            vec![
                ("text-decoration-line".into(), lines),
                ("text-decoration-style".into(), style),
                ("text-decoration-color".into(), color),
                ("text-decoration-thickness".into(), thick),
            ]
        }
        "-webkit-text-stroke" => {
            let (mut width, mut color) = ("0".to_string(), "currentcolor".to_string());
            for item in split_top(v) {
                if item.starts_with(|c: char| c.is_ascii_digit() || c == '.')
                    || matches!(item.as_str(), "thin" | "medium" | "thick")
                {
                    width = item;
                } else {
                    color = item;
                }
            }
            vec![
                ("-webkit-text-stroke-width".into(), width),
                ("-webkit-text-stroke-color".into(), color),
            ]
        }
        "font-synthesis" => {
            let has = |w: &str| words.contains(&w);
            if !(v == "none"
                || words
                    .iter()
                    .all(|w| ["weight", "style", "small-caps", "position"].contains(w)))
            {
                return Err(format!("font-synthesis: '{v}' is not a value"));
            }
            let on = |w: &str| if has(w) { "auto" } else { "none" };
            all(&[
                ("font-synthesis-weight", on("weight")),
                ("font-synthesis-style", on("style")),
                ("font-synthesis-small-caps", on("small-caps")),
                ("font-synthesis-position", on("position")),
            ])
        }
        "font-variant" => {
            let mut out: Vec<(String, Vec<String>)> = [
                "font-variant-caps",
                "font-variant-numeric",
                "font-variant-ligatures",
                "font-variant-east-asian",
                "font-variant-alternates",
                "font-variant-position",
                "font-variant-emoji",
            ]
            .iter()
            .map(|n| (n.to_string(), Vec::new()))
            .collect();
            if v != "normal" && v != "none" {
                for w in split_top(v) {
                    let at = VARIANTS
                        .iter()
                        .position(|(_, vals)| vals.contains(&w.as_str()))
                        .or_else(|| w.contains('(').then_some(4))
                        .ok_or_else(|| format!("font-variant: '{w}' is not a value"))?;
                    out[at].1.push(w);
                }
            } else if v == "none" {
                out[2].1.push("none".into());
            }
            out.into_iter()
                .map(|(n, w)| {
                    let v = if w.is_empty() {
                        "normal".into()
                    } else {
                        w.join(" ")
                    };
                    (n, v)
                })
                .collect()
        }
        "font" => font(v)?,
        _ => unreachable!(),
    })
}

/// Each font-variant longhand's keywords, in shorthand order.
const VARIANTS: &[(&str, &[&str])] = &[
    (
        "font-variant-caps",
        &[
            "small-caps",
            "all-small-caps",
            "petite-caps",
            "all-petite-caps",
            "unicase",
            "titling-caps",
        ],
    ),
    (
        "font-variant-numeric",
        &[
            "lining-nums",
            "oldstyle-nums",
            "proportional-nums",
            "tabular-nums",
            "diagonal-fractions",
            "stacked-fractions",
            "ordinal",
            "slashed-zero",
        ],
    ),
    (
        "font-variant-ligatures",
        &[
            "common-ligatures",
            "no-common-ligatures",
            "discretionary-ligatures",
            "no-discretionary-ligatures",
            "historical-ligatures",
            "no-historical-ligatures",
            "contextual",
            "no-contextual",
        ],
    ),
    (
        "font-variant-east-asian",
        &[
            "jis78",
            "jis83",
            "jis90",
            "jis04",
            "simplified",
            "traditional",
            "full-width",
            "proportional-width",
            "ruby",
        ],
    ),
    ("font-variant-alternates", &["historical-forms"]),
    ("font-variant-position", &["sub", "super"]),
    ("font-variant-emoji", &["text", "emoji", "unicode"]),
];

/// Comma-free items of a value, functions kept whole.
pub(crate) fn split_top(v: &str) -> Vec<String> {
    let mut out = Vec::new();
    let (mut depth, mut cur) = (0, String::new());
    for c in v.chars() {
        match c {
            '(' => {
                depth += 1;
                cur.push(c)
            }
            ')' => {
                depth -= 1;
                cur.push(c)
            }
            c if c.is_whitespace() && depth == 0 => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// CSS Fonts 4 §2.8: `[<style> || <small-caps> || <weight> || <stretch>]?
/// <size> [/ <line-height>]? <family>#`, or a system font keyword (the
/// style's body step).
pub(crate) fn font(v: &str) -> Result<Vec<(String, String)>, String> {
    const HOW: &str = "font: [style] [small-caps] [weight] [stretch] <size>[/<line-height>] <family>, or var(--text-<step>)";
    let system = [
        "caption",
        "icon",
        "menu",
        "message-box",
        "small-caption",
        "status-bar",
    ];
    if system.contains(&v) {
        return Ok(vec![("font".into(), "var(--text-body)".into())]);
    }
    let (head, family) = {
        // The family starts after the size: the first token that is a size.
        let mut items = split_top(v);
        let at = items
            .iter()
            .position(|w| {
                let s = w.split('/').next().unwrap_or("");
                is_size(s)
            })
            .ok_or(HOW)?;
        if items.get(at + 1).is_some_and(|s| s.starts_with('/')) {
            let slash = items.remove(at + 1);
            items[at].push_str(&slash);
        }
        if items[at].ends_with('/') {
            if at + 1 >= items.len() {
                return Err(HOW.into());
            }
            let height = items.remove(at + 1);
            items[at].push_str(&height);
        }
        let family = items[at + 1..].join(" ");
        (items[..=at].to_vec(), family)
    };
    let (mut style, mut caps, mut weight, mut stretch) = ("normal", "normal", "normal", "normal");
    let last = head.len() - 1;
    for w in &head[..last] {
        match w.as_str() {
            "normal" => {}
            "italic" | "oblique" => style = w,
            "small-caps" => caps = w,
            "bold" | "bolder" | "lighter" => weight = w,
            n if n.parse::<f32>().is_ok() => weight = w,
            s if STRETCH.iter().any(|(k, _)| *k == s) => stretch = w,
            _ => {
                return Err(format!(
                    "font: '{w}' is not a style, weight or stretch ({HOW})"
                ));
            }
        }
    }
    let (size, line) = match head[last].split_once('/') {
        Some((s, l)) => (s.to_string(), l.to_string()),
        None => (head[last].clone(), "normal".into()),
    };
    let mut out = vec![
        ("font-style".to_string(), style.to_string()),
        ("font-variant-caps".into(), caps.into()),
        ("font-weight".into(), weight.into()),
        ("font-stretch".into(), stretch.into()),
        ("font-size".into(), size),
        ("line-height".into(), line),
    ];
    if family.is_empty() {
        return Err(HOW.into());
    }
    family_list(&family)?;
    out.push(("font-family".into(), family));
    Ok(out)
}

fn wide(v: &str) -> bool {
    matches!(
        v,
        "inherit" | "initial" | "unset" | "revert" | "revert-layer"
    )
}

/// A `font-family` list (CSS Fonts 4 §2.1) as its named families, quoted
/// and comma-separated (`none` when it names none), and the role its first
/// generic family (or `var(--font-<role>)` token) is, `body` when it names
/// none: a named family is drawn from a document's `@font-face` faces
/// ([`crate::FontRole::Family`]), the role after it.
pub(crate) fn family_list(v: &str) -> Result<(String, String), String> {
    use super::css::Tok;
    let tokens = super::css::tokens(&format!("families({v})"))?;
    let [Tok::Func(_, items)] = tokens.as_slice() else {
        return Err("font-family: expected a comma-separated family list".into());
    };
    let mut names = Vec::new();
    let mut role = None;
    for tokens in items {
        let (item, quoted) = match tokens.as_slice() {
            [Tok::Str(name)] => (name.clone(), true),
            [] => return Err("font-family: an empty family in the list".into()),
            words if words.iter().all(|t| matches!(t, Tok::Ident(_))) => (
                words
                    .iter()
                    .map(super::css::describe)
                    .collect::<Vec<_>>()
                    .join(" "),
                false,
            ),
            _ => {
                return Err(
                    "font-family: expected a quoted name, identifiers or a generic family".into(),
                );
            }
        };
        let lower = item.to_ascii_lowercase();
        let generic = if quoted {
            None
        } else {
            let item = lower.as_str();

            match item {
                "" => return Err("font-family: an empty family in the list".into()),
                "monospace" | "ui-monospace" => Some("mono"),
                "ui-rounded" => Some("rounded"),
                "fantasy" | "cursive" => Some("display"),
                "serif" | "sans-serif" | "system-ui" | "ui-serif" | "ui-sans-serif" | "math"
                | "emoji" | "fangsong" => Some("body"),
                _ => None,
            }
        };
        if let Some(g) = generic {
            role.get_or_insert(g);
            continue;
        }
        if !quoted && wide(&item) {
            return Err("font-family: a CSS-wide keyword cannot occur in a family list".into());
        }
        if role.is_none() && !names.contains(&item) {
            names.push(item);
        }
    }
    let names = if names.is_empty() {
        "none".to_string()
    } else {
        names
            .iter()
            .map(|n| format!("\"{}\"", n.replace('\\', "\\\\").replace('"', "\\\"")))
            .collect::<Vec<_>>()
            .join(", ")
    };
    Ok((names, format!("var(--font-{})", role.unwrap_or("body"))))
}

/// The names of a computed `-og-font-name` list.
pub(crate) fn names_of(v: &str) -> Vec<String> {
    if v == "none" {
        return Vec::new();
    }
    match super::css::tokens(&format!("families({v})")).as_deref() {
        Ok([super::css::Tok::Func(_, items)]) => items
            .iter()
            .filter_map(|tokens| match tokens.as_slice() {
                [super::css::Tok::Str(name)] => Some(name.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn is_size(s: &str) -> bool {
    SIZE_KEYWORDS.iter().any(|(k, _)| *k == s)
        || matches!(s, "larger" | "smaller")
        || length_of(s, 16.0, 16.0).is_some()
}

const SIZE_KEYWORDS: &[(&str, f32)] = &[
    ("xx-small", 3.0 / 5.0),
    ("x-small", 3.0 / 4.0),
    ("small", 8.0 / 9.0),
    ("medium", 1.0),
    ("large", 6.0 / 5.0),
    ("x-large", 3.0 / 2.0),
    ("xx-large", 2.0),
    ("xxx-large", 3.0),
];

const STRETCH: &[(&str, f32)] = &[
    ("ultra-condensed", 50.0),
    ("extra-condensed", 62.5),
    ("condensed", 75.0),
    ("semi-condensed", 87.5),
    ("normal", 100.0),
    ("semi-expanded", 112.5),
    ("expanded", 125.0),
    ("extra-expanded", 150.0),
    ("ultra-expanded", 200.0),
];

/// A length in px against `em` (the font size it is relative to) and
/// `pct` (what 100% is). `rem` and viewport units are px by now.
pub(crate) fn length_of(s: &str, em: f32, pct: f32) -> Option<f32> {
    let s = s.trim();
    if s == "0" {
        return Some(0.0);
    }
    let (n, k) = if let Some(n) = s.strip_suffix("px") {
        (n, 1.0)
    } else if let Some(n) = s.strip_suffix("em") {
        (n, em)
    } else if let Some(n) = s.strip_suffix('%') {
        (n, pct / 100.0)
    } else if let Some(n) = s.strip_suffix("ex") {
        (n, em * 0.5)
    } else if let Some(n) = s.strip_suffix("ch") {
        (n, em * 0.5)
    } else {
        (s.strip_suffix("pt")?, 4.0 / 3.0)
    };
    n.parse::<f32>().ok().map(|v| v * k)
}

fn words_in(name: &str, v: &str, allowed: &[&str]) -> Result<(), String> {
    for w in v.split_whitespace() {
        if !allowed.contains(&w) {
            return Err(format!("{name} takes {}, not {w}", allowed.join(", ")));
        }
    }
    Ok(())
}

/// Checks a longhand's value as written (no `var()`).
fn check(name: &str, v: &str) -> Result<(), String> {
    let v = v.trim();
    if matches!(
        v,
        "inherit" | "initial" | "unset" | "revert" | "revert-layer"
    ) {
        return Ok(());
    }
    let length = |v: &str| length_of(v, 16.0, 16.0).is_some();
    let ok = match name {
        "font-size" => is_size(v),
        "font-weight" => {
            matches!(v, "normal" | "bold" | "bolder" | "lighter")
                || v.parse::<f32>().is_ok_and(|w| (1.0..=1000.0).contains(&w))
        }
        "font-style" => v == "normal" || v == "italic" || v.starts_with("oblique"),
        "font-stretch" => {
            STRETCH.iter().any(|(k, _)| *k == v)
                || v.strip_suffix('%')
                    .is_some_and(|n| n.parse::<f32>().is_ok())
        }
        "line-height" => v == "normal" || v.parse::<f32>().is_ok() || length(v),
        "letter-spacing" | "word-spacing" => v == "normal" || length(v),
        "text-indent" => {
            let mut it = v.split_whitespace();
            it.next().is_some_and(length) && it.all(|w| matches!(w, "hanging" | "each-line"))
        }
        "text-underline-offset" | "-webkit-text-stroke-width" | "text-decoration-thickness" => {
            matches!(v, "auto" | "from-font" | "thin" | "medium" | "thick") || length(v)
        }
        "tab-size" => v.parse::<f32>().is_ok_and(|n| n >= 0.0) || length(v),
        "text-transform" => {
            return words_in(
                name,
                v,
                &[
                    "none",
                    "uppercase",
                    "lowercase",
                    "capitalize",
                    "full-width",
                    "full-size-kana",
                    "math-auto",
                ],
            );
        }
        "white-space-collapse" => matches!(
            v,
            "collapse"
                | "discard"
                | "preserve"
                | "preserve-breaks"
                | "preserve-spaces"
                | "break-spaces"
        ),
        "text-wrap-mode" => matches!(v, "wrap" | "nowrap"),
        "text-wrap-style" => matches!(v, "auto" | "balance" | "pretty" | "stable"),
        "text-align" => matches!(
            v,
            "start"
                | "end"
                | "left"
                | "right"
                | "center"
                | "justify"
                | "match-parent"
                | "justify-all"
        ),
        "text-align-last" => matches!(
            v,
            "auto" | "start" | "end" | "left" | "right" | "center" | "justify" | "match-parent"
        ),
        "hyphens" => matches!(v, "none" | "manual" | "auto"),
        "word-break" => matches!(
            v,
            "normal" | "break-all" | "keep-all" | "break-word" | "auto-phrase"
        ),
        "overflow-wrap" => matches!(v, "normal" | "anywhere" | "break-word"),
        "text-justify" => matches!(
            v,
            "auto" | "none" | "inter-word" | "inter-character" | "distribute"
        ),
        "font-kerning" => matches!(v, "auto" | "normal" | "none"),
        "font-optical-sizing" => matches!(v, "auto" | "none"),
        "font-synthesis-weight"
        | "font-synthesis-style"
        | "font-synthesis-small-caps"
        | "font-synthesis-position" => matches!(v, "auto" | "none"),
        "font-size-adjust" => {
            v == "none"
                || v.split_whitespace()
                    .last()
                    .is_some_and(|n| n.parse::<f32>().is_ok() || n == "from-font")
        }
        "font-language-override" => v == "normal" || v.starts_with('"'),
        "font-feature-settings" => v == "normal" || feature_settings(v).is_ok(),
        "font-variation-settings" => v == "normal" || variation_settings(v).is_ok(),
        "text-underline-position" => {
            return words_in(name, v, &["auto", "from-font", "under", "left", "right"]);
        }
        "text-decoration-skip-ink" => matches!(v, "auto" | "none" | "all"),
        "text-decoration-line" => {
            return words_in(
                name,
                v,
                &[
                    "none",
                    "underline",
                    "overline",
                    "line-through",
                    "blink",
                    "spelling-error",
                    "grammar-error",
                ],
            );
        }
        "text-decoration-style" => matches!(v, "solid" | "double" | "dotted" | "dashed" | "wavy"),
        "vertical-align" => {
            matches!(
                v,
                "baseline"
                    | "sub"
                    | "super"
                    | "text-top"
                    | "text-bottom"
                    | "middle"
                    | "top"
                    | "bottom"
            ) || length(v)
        }
        "paint-order" => v == "normal" || words_in(name, v, &["fill", "stroke", "markers"]).is_ok(),
        "color-scheme" => {
            v == "normal"
                || v.split_whitespace()
                    .all(|w| w == "only" || super::syntax::identifier(w))
        }
        "caret-color" | "accent-color" if v == "auto" => true,
        "text-decoration-color"
        | "-webkit-text-stroke-color"
        | "-webkit-text-fill-color"
        | "caret-color"
        | "accent-color" => crate::css_color::parse_str(v).is_ok(),
        n if n.starts_with("font-variant-") => {
            let allowed = VARIANTS
                .iter()
                .find(|(k, _)| *k == n)
                .map_or(&[][..], |(_, w)| *w);
            v == "normal"
                || (n == "font-variant-ligatures" && v == "none")
                || (n == "font-variant-alternates" && v.contains('('))
                || v.split_whitespace().all(|w| allowed.contains(&w))
        }
        _ => true,
    };
    if ok {
        Ok(())
    } else {
        Err(format!("{name}: '{v}' is not a value"))
    }
}

/// `"liga" 0, "tnum"`: OpenType tags with their values.
pub(crate) fn feature_settings(v: &str) -> Result<Vec<([u8; 4], u16)>, String> {
    v.split(',')
        .map(|item| {
            let item = item.trim();
            let (tag, rest) = tag_of(item)?;
            let value = match rest.trim() {
                "" | "on" => 1,
                "off" => 0,
                n => n
                    .parse()
                    .map_err(|_| format!("'{n}' is not a feature value"))?,
            };
            Ok((tag, value))
        })
        .collect()
}

/// `"wght" 650, "wdth" 80`: axes with their values.
pub(crate) fn variation_settings(v: &str) -> Result<Vec<([u8; 4], f32)>, String> {
    v.split(',')
        .map(|item| {
            let (tag, rest) = tag_of(item.trim())?;
            let value = rest
                .trim()
                .parse()
                .map_err(|_| format!("'{}' is not an axis value", rest.trim()))?;
            Ok((tag, value))
        })
        .collect()
}

fn tag_of(item: &str) -> Result<([u8; 4], &str), String> {
    let q = item
        .chars()
        .next()
        .filter(|c| *c == '"' || *c == '\'')
        .ok_or("a feature tag is quoted: \"liga\"")?;
    let rest = &item[1..];
    let end = rest.find(q).ok_or("a feature tag is quoted")?;
    let tag = &rest.as_bytes()[..end];
    if tag.len() != 4 || !tag.iter().all(|b| (0x20..=0x7e).contains(b)) {
        return Err(format!("'{}' is not a four-letter tag", &rest[..end]));
    }
    Ok(([tag[0], tag[1], tag[2], tag[3]], &rest[end + 1..]))
}

// ---- computing

/// A decoration line's style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DecorationStyle {
    #[default]
    Solid,
    Double,
    Dotted,
    Dashed,
    Wavy,
}

/// The decoration of a text (CSS Text Decoration 3), its lengths in px.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Decoration {
    pub underline: bool,
    pub overline: bool,
    pub through: bool,
    pub style: DecorationStyle,
    /// `None`: the text's colour.
    pub color: Option<Srgba>,
    /// `None`: auto (the font's, about a fifteenth of an em).
    pub thickness: Option<f32>,
    /// The underline's offset from the baseline (`None`: auto).
    pub offset: Option<f32>,
    /// `text-underline-position: under`.
    pub under: bool,
}

impl Decoration {
    pub fn any(&self) -> bool {
        self.underline || self.overline || self.through
    }
}

/// How white space is handled.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Collapse {
    #[default]
    Collapse,
    Discard,
    Preserve,
    PreserveBreaks,
    PreserveSpaces,
    BreakSpaces,
}

/// One element's effective text style: what it declares over what it
/// inherits. Every field is `None` (or default) where neither sets it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextCss {
    pub size: Option<f32>,
    pub weight: Option<u16>,
    pub italic: Option<bool>,
    /// Oblique angle in degrees (`oblique 10deg`); italic is 14.
    pub slant: Option<f32>,
    pub stretch: Option<f32>,
    /// A multiple of the size (`normal`: the step's own).
    pub line_height: Option<LineHeight>,
    pub letter_spacing: Option<f32>,
    pub word_spacing: Option<f32>,
    pub transform: Option<String>,
    pub collapse: Option<Collapse>,
    pub nowrap: Option<bool>,
    pub wrap_style: Option<String>,
    pub align: Option<String>,
    pub align_last: Option<String>,
    pub indent: Option<(f32, bool, bool)>,
    pub tab_size: Option<f32>,
    pub hyphens: Option<String>,
    pub word_break: Option<String>,
    pub overflow_wrap: Option<String>,
    pub justify: Option<String>,
    pub features: Vec<([u8; 4], u16)>,
    pub variations: Vec<([u8; 4], f32)>,
    pub kerning: Option<bool>,
    pub optical_sizing: Option<bool>,
    pub synthesis: [Option<bool>; 4],
    pub decoration: Decoration,
    pub stroke: Option<(f32, Option<Srgba>)>,
    pub fill: Option<Srgba>,
    pub paint_order: Option<String>,
    pub caret: Option<Srgba>,
    pub accent: Option<Srgba>,
    /// The used colour scheme is dark (`None`: the style's).
    pub dark: Option<bool>,
    pub vertical_align: Option<String>,
    /// The first named family of `font-family` a document declares an
    /// `@font-face` for (CSS Fonts 4 §5: an unavailable family is skipped).
    pub family: Option<String>,
    pub font_role: Option<crate::FontRole>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LineHeight {
    Normal,
    /// A multiple of the font size (inherited as the number).
    Factor(f32),
    Px(f32),
}

/// What [`inherit`] reads of the element's resolution.
pub(crate) struct Inputs<'a> {
    /// The element's own text declarations, substituted, in order.
    pub local: &'a [(String, String)],
    pub computed: &'a mut variables::Computed,
    pub reads: &'a mut std::collections::BTreeSet<String>,
    pub diagnostics: &'a mut Vec<String>,
    pub theme: &'a crate::Theme,
    /// The text colour (`currentColor`).
    pub current: Srgba,
    /// The type step the element's own `font` gives (or the body's).
    pub step: TypeStep,
    /// Whether a document declares an `@font-face` for a family name.
    pub declared: &'a dyn Fn(&str) -> bool,
}

/// Computes the element's inherited text values into its computed map
/// and answers its effective text style.
pub(crate) fn inherit(inp: Inputs<'_>) -> TextCss {
    let Inputs {
        local,
        computed,
        reads,
        diagnostics,
        theme,
        current,
        step,
        declared,
    } = inp;
    // The parent's font size (px), what `em` and `%` here are against.
    let parent_num = |name: &str| {
        computed
            .get(&key(name))
            .cloned()
            .flatten()
            .and_then(|v| v.parse::<f32>().ok())
    };
    let parent_size = parent_num("font-size");
    let parent_weight = parent_num("font-weight").unwrap_or(f32::from(step.weight.max(400)));
    for (name, value) in local {
        let value = value.trim();
        let computed_value = match value {
            "inherit" | "unset" if INHERITED.contains(&name.as_str()) => continue,
            "initial" | "revert" | "revert-layer" | "unset" => {
                computed.remove(&key(name));
                continue;
            }
            "inherit" => continue,
            v => compute(
                name,
                v,
                parent_size.unwrap_or(step.size),
                parent_weight,
                // The element's own size, computed first if it declares one.
                computed
                    .get(&key("font-size"))
                    .cloned()
                    .flatten()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(step.size),
                theme,
                current,
            ),
        };
        match computed_value {
            Ok(v) => {
                computed.insert(key(name), Some(v));
            }
            Err(e) => diagnostics.push(format!("{name}: {e}")),
        }
    }
    let own_size = computed
        .get(&key("font-size"))
        .cloned()
        .flatten()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(step.size);
    // Decorations: the element's own lines over those of its ancestors'
    // (propagated, CSS Text Decoration 3 §2), the colour and offset its own.
    let own = |n: &str| {
        local
            .iter()
            .rev()
            .find(|(k, _)| k == n)
            .map(|(_, v)| v.clone())
    };
    let parent_lines = computed.get(&key("decoration")).cloned().flatten();
    reads.insert(key("decoration"));
    let mut lines = parent_lines.clone().unwrap_or_default();
    if let Some(l) = own("text-decoration-line").filter(|l| l != "none") {
        let style = own("text-decoration-style").unwrap_or_else(|| "solid".into());
        let color = own("text-decoration-color")
            .and_then(|c| crate::css_color::parse_str(&c).ok())
            .map(|c| c.resolve(&crate::css_color::Context::of(theme)))
            .map(crate::doc::css::hex)
            .unwrap_or_else(|| "-".into());
        let thick = own("text-decoration-thickness")
            .and_then(|v| length_of(&v, own_size, own_size))
            .map_or("-".into(), |v| v.to_string());
        lines = format!("{lines}|{l};{style};{color};{thick}");
    }
    if lines != parent_lines.unwrap_or_default() {
        computed.insert(key("decoration"), Some(lines.clone()));
    }
    let mut get = |name: &str| -> Option<String> {
        let k = key(name);
        if !local.iter().any(|(n, _)| n == name) {
            reads.insert(k.clone());
        }
        computed.get(&k).cloned().flatten()
    };
    let num = |s: Option<String>| s.and_then(|v| v.parse::<f32>().ok());
    let color = |s: Option<String>| -> Option<Srgba> {
        let s = s?;
        if s == "auto" {
            return None;
        }
        crate::css_color::parse_str(&s)
            .ok()
            .map(|c| c.resolve(&crate::css_color::Context::of(theme)))
    };
    let size = num(get("font-size"));
    let family_list = get("font-family").and_then(|v| family_list(&v).ok());
    let font_role = family_list.as_ref().and_then(|(_, role)| {
        role.strip_prefix("var(--font-")
            .and_then(|r| r.strip_suffix(')'))
            .and_then(crate::FontRole::from_name)
    });
    let family = family_list
        .map(|(names, _)| names)
        .or_else(|| get("-og-font-name"))
        .and_then(|names| names_of(&names).into_iter().find(|n| declared(n)));
    let mut t = TextCss {
        size,
        weight: num(get("font-weight")).map(|w| w.round().clamp(1.0, 1000.0) as u16),
        stretch: num(get("font-stretch")),
        line_height: get("line-height").map(|v| match v.as_str() {
            "normal" => LineHeight::Normal,
            v if v.ends_with("px") => LineHeight::Px(v[..v.len() - 2].parse().unwrap_or(0.0)),
            v => LineHeight::Factor(v.parse().unwrap_or(1.2)),
        }),
        letter_spacing: num(get("letter-spacing")),
        word_spacing: num(get("word-spacing")),
        transform: get("text-transform").filter(|v| v != "none"),
        collapse: get("white-space-collapse").map(|v| match v.as_str() {
            "discard" => Collapse::Discard,
            "preserve" => Collapse::Preserve,
            "preserve-breaks" => Collapse::PreserveBreaks,
            "preserve-spaces" => Collapse::PreserveSpaces,
            "break-spaces" => Collapse::BreakSpaces,
            _ => Collapse::Collapse,
        }),
        nowrap: get("text-wrap-mode").map(|v| v == "nowrap"),
        wrap_style: get("text-wrap-style"),
        align: get("text-align"),
        align_last: get("text-align-last"),
        indent: get("text-indent").map(|v| {
            let mut it = v.split_whitespace();
            let px = it.next().and_then(|n| n.parse().ok()).unwrap_or(0.0);
            let rest: Vec<&str> = it.collect();
            (px, rest.contains(&"hanging"), rest.contains(&"each-line"))
        }),
        tab_size: num(get("tab-size")),
        hyphens: get("hyphens"),
        word_break: get("word-break"),
        overflow_wrap: get("overflow-wrap"),
        justify: get("text-justify"),
        kerning: get("font-kerning").map(|v| v != "none"),
        optical_sizing: get("font-optical-sizing").map(|v| v != "none"),
        caret: color(get("caret-color")),
        accent: color(get("accent-color")),
        fill: color(get("-webkit-text-fill-color")),
        paint_order: get("paint-order").filter(|v| v != "normal"),
        family,
        font_role,
        ..TextCss::default()
    };
    match get("font-style").as_deref() {
        Some("normal") => t.italic = Some(false),
        Some("italic") => {
            t.italic = Some(true);
            t.slant = Some(14.0);
        }
        Some(o) if o.starts_with("oblique") => {
            t.italic = Some(true);
            t.slant = Some(
                o.split_whitespace()
                    .nth(1)
                    .and_then(|a| a.strip_suffix("deg"))
                    .and_then(|a| a.parse().ok())
                    .unwrap_or(14.0),
            );
        }
        _ => {}
    }
    for (k, n) in ["weight", "style", "small-caps", "position"]
        .iter()
        .enumerate()
    {
        t.synthesis[k] = get(&format!("font-synthesis-{n}")).map(|v| v == "auto");
    }
    // Font variants as OpenType features (CSS Fonts 4 §6), then the
    // low-level settings over them.
    let mut features: Vec<([u8; 4], u16)> = Vec::new();
    let mut on = |tag: &[u8; 4], v: u16| {
        features.retain(|(t, _)| t != tag);
        features.push((*tag, v));
    };
    for name in [
        "font-variant-caps",
        "font-variant-numeric",
        "font-variant-ligatures",
        "font-variant-east-asian",
        "font-variant-position",
        "font-variant-alternates",
    ] {
        let Some(v) = get(name) else { continue };
        for w in v.split_whitespace() {
            for (tag, value) in variant_features(w) {
                on(tag, value);
            }
        }
    }
    if t.kerning == Some(false) {
        on(b"kern", 0);
    }
    if let Some(v) = get("font-feature-settings").filter(|v| v != "normal")
        && let Ok(list) = feature_settings(&v)
    {
        for (tag, value) in list {
            on(&tag, value);
        }
    }
    t.features = features;
    if let Some(v) = get("font-variation-settings").filter(|v| v != "normal") {
        t.variations = variation_settings(&v).unwrap_or_default();
    }
    for part in lines.split('|').filter(|p| !p.is_empty()) {
        let f: Vec<&str> = part.split(';').collect();
        let d = &mut t.decoration;
        for l in f[0].split_whitespace() {
            match l {
                "underline" => d.underline = true,
                "overline" => d.overline = true,
                "line-through" => d.through = true,
                _ => {}
            }
        }
        d.style = match f.get(1).copied() {
            Some("double") => DecorationStyle::Double,
            Some("dotted") => DecorationStyle::Dotted,
            Some("dashed") => DecorationStyle::Dashed,
            Some("wavy") => DecorationStyle::Wavy,
            _ => DecorationStyle::Solid,
        };
        d.color = f.get(2).and_then(|c| crate::style_file::parse_color(c));
        d.thickness = f.get(3).and_then(|v| v.parse().ok());
    }
    t.decoration.offset = num(get("text-underline-offset"));
    t.decoration.under = get("text-underline-position").is_some_and(|v| v.contains("under"));
    if let Some(w) = num(get("-webkit-text-stroke-width")).filter(|w| *w > 0.0) {
        t.stroke = Some((w, color(get("-webkit-text-stroke-color"))));
    }
    t.dark = get("color-scheme").and_then(|v| {
        let words: Vec<&str> = v.split_whitespace().filter(|w| *w != "only").collect();
        let theme_dark = theme.scheme == crate::Scheme::Dark;
        let has = |w: &str| words.contains(&w);
        match (has("light"), has("dark")) {
            (true, true) | (false, false) => None,
            (true, false) => Some(false),
            (false, true) => Some(true),
        }
        .filter(|d| *d != theme_dark)
    });
    t.vertical_align = own("vertical-align");
    t
}

/// A variant keyword's OpenType features (CSS Fonts 4 §6.4–6.11).
fn variant_features(w: &str) -> Vec<(&'static [u8; 4], u16)> {
    match w {
        "small-caps" => vec![(b"smcp", 1)],
        "all-small-caps" => vec![(b"smcp", 1), (b"c2sc", 1)],
        "petite-caps" => vec![(b"pcap", 1)],
        "all-petite-caps" => vec![(b"pcap", 1), (b"c2pc", 1)],
        "unicase" => vec![(b"unic", 1)],
        "titling-caps" => vec![(b"titl", 1)],
        "lining-nums" => vec![(b"lnum", 1)],
        "oldstyle-nums" => vec![(b"onum", 1)],
        "proportional-nums" => vec![(b"pnum", 1)],
        "tabular-nums" => vec![(b"tnum", 1)],
        "diagonal-fractions" => vec![(b"frac", 1)],
        "stacked-fractions" => vec![(b"afrc", 1)],
        "ordinal" => vec![(b"ordn", 1)],
        "slashed-zero" => vec![(b"zero", 1)],
        "none" => vec![
            (b"liga", 0),
            (b"clig", 0),
            (b"dlig", 0),
            (b"hlig", 0),
            (b"calt", 0),
        ],
        "common-ligatures" => vec![(b"liga", 1), (b"clig", 1)],
        "no-common-ligatures" => vec![(b"liga", 0), (b"clig", 0)],
        "discretionary-ligatures" => vec![(b"dlig", 1)],
        "no-discretionary-ligatures" => vec![(b"dlig", 0)],
        "historical-ligatures" => vec![(b"hlig", 1)],
        "no-historical-ligatures" => vec![(b"hlig", 0)],
        "contextual" => vec![(b"calt", 1)],
        "no-contextual" => vec![(b"calt", 0)],
        "jis78" => vec![(b"jp78", 1)],
        "jis83" => vec![(b"jp83", 1)],
        "jis90" => vec![(b"jp90", 1)],
        "jis04" => vec![(b"jp04", 1)],
        "simplified" => vec![(b"smpl", 1)],
        "traditional" => vec![(b"trad", 1)],
        "full-width" => vec![(b"fwid", 1)],
        "proportional-width" => vec![(b"pwid", 1)],
        "ruby" => vec![(b"ruby", 1)],
        "sub" => vec![(b"subs", 1)],
        "super" => vec![(b"sups", 1)],
        "historical-forms" => vec![(b"hist", 1)],
        _ => Vec::new(),
    }
}

pub(crate) fn font_size(v: &str, parent: f32, medium: f32) -> Option<f32> {
    let size = if let Some((_, k)) = SIZE_KEYWORDS.iter().find(|(k, _)| *k == v) {
        medium * k
    } else if v == "larger" {
        parent * 1.2
    } else if v == "smaller" {
        parent / 1.2
    } else {
        length_of(v, parent, parent)?
    };
    Some(size.max(0.0))
}

/// One declared value as it is inherited: lengths in px against `em`
/// (the parent's size for `font-size`, the element's own otherwise).
#[allow(clippy::too_many_arguments)]
fn compute(
    name: &str,
    v: &str,
    parent: f32,
    parent_weight: f32,
    own: f32,
    theme: &crate::Theme,
    current: Srgba,
) -> Result<String, String> {
    check(name, v)?;
    let px = |v: &str, em: f32| -> Result<String, String> {
        length_of(v, em, em)
            .map(|p| super::calc::number(f64::from(p)))
            .ok_or_else(|| format!("'{v}' is not a length"))
    };
    Ok(match name {
        "font-size" => {
            let p = font_size(v, parent, theme.type_ramp.body.size).ok_or("a font size")?;
            super::calc::number(f64::from(p))
        }
        "font-weight" => match v {
            "normal" => "400".into(),
            "bold" => "700".into(),
            // CSS Fonts 4 §2.2's table, against the inherited weight.
            "bolder" => match parent_weight {
                w if w < 350.0 => "400",
                w if w < 550.0 => "700",
                _ => "900",
            }
            .into(),
            "lighter" => match parent_weight {
                w if w < 100.0 => return Ok(super::calc::number(f64::from(w))),
                w if w < 550.0 => "100",
                w if w < 750.0 => "400",
                _ => "700",
            }
            .into(),
            n => n.into(),
        },
        "font-stretch" => STRETCH.iter().find(|(k, _)| *k == v).map_or_else(
            || v.trim_end_matches('%').to_string(),
            |(_, p)| p.to_string(),
        ),
        "line-height" => match v {
            "normal" => "normal".into(),
            n if n.parse::<f32>().is_ok() => n.into(),
            v => format!("{}px", px(v, own)?),
        },
        "letter-spacing" | "word-spacing" if v == "normal" => "0".into(),
        "letter-spacing"
        | "word-spacing"
        | "text-underline-offset"
        | "-webkit-text-stroke-width" => match v {
            "auto" | "from-font" => "auto".into(),
            "thin" => "1".into(),
            "medium" => "3".into(),
            "thick" => "5".into(),
            v => px(v, own)?,
        },
        "text-indent" => {
            let mut it = v.split_whitespace();
            let l = it.next().unwrap_or("0");
            let rest: Vec<&str> = it.collect();
            format!("{} {}", px(l, own)?, rest.join(" "))
                .trim()
                .to_string()
        }
        "tab-size" => match v.parse::<f32>() {
            Ok(n) => super::calc::number(f64::from(n)),
            Err(_) => px(v, own)?,
        },
        "caret-color"
        | "accent-color"
        | "-webkit-text-stroke-color"
        | "-webkit-text-fill-color"
            if v != "auto" =>
        {
            let c = crate::css_color::parse_str(v)?;
            let c = if c.uses_current() {
                // currentColor computes to the element's colour.
                crate::css_color::parse_str(&crate::doc::css::hex(current))?
            } else {
                c
            };
            match c.literal() {
                Some(l) => crate::doc::css::hex(l),
                None => v.to_string(),
            }
        }
        _ => v.to_string(),
    })
}

// ---- applying

impl TextCss {
    /// The type step with its size, weight, line height and tracking.
    pub fn step(&self, mut step: TypeStep) -> TypeStep {
        let ratio = step.line_height / step.size.max(1e-3);
        if let Some(s) = self.size {
            // A size keeps the step's leading ratio unless a line height says.
            step.size = s;
            step.line_height = s * ratio;
            step.cjk_line_height = s * (step.cjk_line_height / step.size.max(1e-3)).max(ratio);
        }
        match self.line_height {
            Some(LineHeight::Factor(f)) => step.line_height = step.size * f,
            Some(LineHeight::Px(p)) => step.line_height = p,
            Some(LineHeight::Normal) => step.line_height = step.size * 1.2,
            None => {}
        }
        if let Some(LineHeight::Factor(_) | LineHeight::Px(_)) = self.line_height {
            step.cjk_line_height = step.line_height;
        }
        if let Some(l) = self.letter_spacing {
            step.tracking_em = l / step.size.max(1e-3);
            step.cjk_tracking_em = step.tracking_em;
        }
        if let Some(w) = self.weight {
            step.weight = w;
        }
        if let Some(t) = &self.transform {
            step.uppercase = t.split_whitespace().any(|w| w == "uppercase");
        }
        let mut tags: Vec<&str> = step.features.tags().collect();
        for (tag, v) in &self.features {
            let t = std::str::from_utf8(tag).unwrap_or("");
            if let Some(known) = crate::shape::Features::TAGS.iter().find(|k| **k == t) {
                tags.retain(|k| k != known);
                if *v != 0 {
                    tags.push(known);
                }
            }
        }
        if let Ok(f) = crate::shape::Features::parse(tags) {
            step.features = f;
        }
        step
    }

    /// Whether the text needs a span of its own (a style, a decoration,
    /// a stroke or a fill the plain text path does not draw).
    pub fn rich(&self) -> bool {
        self.italic == Some(true) || self.decoration.any() || self.stroke.is_some()
    }

    /// `text` as shown: transformed (CSS Text 3 §2.1) and its white space
    /// processed (§4.1). `locale` cases it. Shown text shown again is
    /// unchanged, so settlement can restyle a built text.
    pub fn shown(&self, text: &str, locale: &str) -> String {
        // The document's text is already HTML-collapsed: only a declared
        // white-space changes it.
        let Some(collapse) = self.collapse else {
            return self.finish(text.to_string(), locale);
        };
        let s = match collapse {
            Collapse::Collapse | Collapse::Discard => {
                let mut out = String::with_capacity(text.len());
                let mut space = false;
                for c in text.chars() {
                    if matches!(c, ' ' | '\t' | '\n' | '\r') {
                        space = collapse == Collapse::Collapse;
                    } else {
                        if space && !out.is_empty() {
                            out.push(' ');
                        }
                        space = false;
                        out.push(c);
                    }
                }
                out
            }
            Collapse::PreserveBreaks => {
                // Spaces and tabs collapse; a newline stays a forced break.
                text.split('\n')
                    .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            Collapse::PreserveSpaces => text.replace(['\n', '\t', '\r'], " "),
            Collapse::Preserve | Collapse::BreakSpaces => {
                // Tabs to the next tab stop (tab-size spaces).
                let n = self.tab_size.unwrap_or(8.0).round().max(0.0) as usize;
                let mut out = String::with_capacity(text.len());
                let mut col = 0;
                for c in text.chars() {
                    match c {
                        '\t' if n > 0 => {
                            let k = n - col % n;
                            out.extend(std::iter::repeat_n('\u{a0}', k));
                            col += k;
                        }
                        '\n' => {
                            out.push('\n');
                            col = 0;
                        }
                        // Preserved spaces do not collapse: no-break, so a
                        // run keeps its width (break-spaces lets each one
                        // break).
                        ' ' if collapse == Collapse::Preserve => {
                            out.push('\u{a0}');
                            col += 1;
                        }
                        c => {
                            out.push(c);
                            col += 1;
                        }
                    }
                }
                out
            }
        };
        self.finish(s, locale)
    }

    fn finish(&self, mut s: String, locale: &str) -> String {
        // word-break: break-all lets every character break; keep-all keeps
        // CJK words whole (CSS Text 3 §5.2), by zero-width spaces and joiners.
        match self.word_break.as_deref() {
            Some("break-all") => {
                s = s
                    .chars()
                    .filter(|c| *c != '\u{200b}')
                    .flat_map(|c| [c, '\u{200b}'])
                    .collect::<String>()
                    .trim_end_matches('\u{200b}')
                    .to_string()
            }
            Some("keep-all") => {
                s = s
                    .chars()
                    .filter(|c| *c != '\u{2060}')
                    .flat_map(|c| {
                        if crate::text::is_cjk(c) {
                            vec![c, '\u{2060}']
                        } else {
                            vec![c]
                        }
                    })
                    .collect()
            }
            _ => {}
        }
        // overflow-wrap anywhere and break-word: a unit wider than the line
        // breaks between graphemes, which the line breaker always does.
        if self.hyphens.as_deref() == Some("none") {
            s = s.replace('\u{ad}', "");
        }
        if let Some(t) = &self.transform {
            for w in t.split_whitespace() {
                s = transform(w, &s, locale);
            }
        }
        s
    }
}

/// CSS Text 3 §2.1's transforms.
pub(crate) fn transform(w: &str, s: &str, locale: &str) -> String {
    let turkic = locale.starts_with("tr") || locale.starts_with("az");
    match w {
        "uppercase" => {
            if turkic {
                s.replace('i', "İ").to_uppercase()
            } else {
                s.to_uppercase()
            }
        }
        "lowercase" => {
            if turkic {
                s.replace('I', "ı").replace('İ', "i").to_lowercase()
            } else {
                s.to_lowercase()
            }
        }
        "capitalize" => {
            // The first typographic letter unit of each word, titlecased.
            let mut out = String::with_capacity(s.len());
            let mut start = true;
            for c in s.chars() {
                if c.is_alphanumeric() {
                    if start {
                        out.extend(c.to_uppercase());
                    } else {
                        out.push(c);
                    }
                    start = false;
                } else {
                    start = c.is_whitespace() || matches!(c, '-' | '\u{a0}');
                    out.push(c);
                }
            }
            out
        }
        "full-width" => s
            .chars()
            .map(|c| match c {
                ' ' => '\u{3000}',
                '!'..='~' => char::from_u32(c as u32 - 0x21 + 0xff01).unwrap_or(c),
                c => c,
            })
            .collect(),
        "full-size-kana" => s
            .chars()
            .map(|c| {
                const SMALL: &str = "ぁぃぅぇぉゕゖっゃゅょゎァィゥェォヵㇰヶㇱㇲッㇳㇴㇵㇶㇷㇸㇹㇺャュョㇻㇼㇽㇾㇿヮｧｨｩｪｫｯｬｭｮ";
                const FULL: &str = "あいうえおかけつやゆよわアイウエオカクケシスツトヌハヒフヘホムヤユヨラリルレロワｱｲｳｴｵﾂﾔﾕﾖ";
                SMALL
                    .chars()
                    .position(|k| k == c)
                    .and_then(|i| FULL.chars().nth(i))
                    .unwrap_or(c)
            })
            .collect(),
        _ => s.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transforms_and_white_space_follow_css_text() {
        assert_eq!(
            transform("capitalize", "hello wide-world 3d", "en"),
            "Hello Wide-World 3d"
        );
        assert_eq!(transform("uppercase", "istanbul", "tr"), "İSTANBUL");
        assert_eq!(transform("lowercase", "DIŞ", "tr"), "dış");
        assert_eq!(transform("full-width", "A1 b", "en"), "Ａ１\u{3000}ｂ");
        assert_eq!(transform("full-size-kana", "ァッ", "ja"), "アツ");
        let t = |c| TextCss {
            collapse: Some(c),
            ..TextCss::default()
        };
        assert_eq!(t(Collapse::Collapse).shown("  a \n\t b  ", "en"), "a b");
        assert_eq!(
            t(Collapse::PreserveBreaks).shown(" a  b\n c", "en"),
            "a b\nc"
        );
        assert_eq!(t(Collapse::Preserve).shown("a  b", "en"), "a\u{a0}\u{a0}b");
        // A tab goes to the next stop of tab-size columns.
        let tab = TextCss {
            collapse: Some(Collapse::BreakSpaces),
            tab_size: Some(4.0),
            ..TextCss::default()
        };
        assert_eq!(tab.shown("ab\tc", "en"), "ab\u{a0}\u{a0}c");
    }

    #[test]
    fn shorthands_expand_to_their_longhands() {
        let d = |n: &str, v: &str| StyleDeclaration {
            name: n.into(),
            value: variables::Value::parse(v).unwrap(),
            source: v.into(),
            important: false,
            pending_shorthand: None,
        };
        let names = |ds: Vec<StyleDeclaration>| -> Vec<(String, String)> {
            ds.into_iter().map(|d| (d.name, d.source)).collect()
        };
        let font = names(expand(d("font", "italic bold 20px/1.5 monospace")).unwrap());
        assert!(font.contains(&("font-size".into(), "20px".into())));
        assert!(font.contains(&("line-height".into(), "1.5".into())));
        assert!(font.contains(&("font-weight".into(), "bold".into())));
        assert!(font.contains(&("font-family".into(), "monospace".into())));
        assert_eq!(
            names(expand(d("font", "var(--text-caption)")).unwrap()).len(),
            1
        );
        let ws = names(expand(d("white-space", "pre-line")).unwrap());
        assert_eq!(
            ws,
            [
                (
                    "white-space-collapse".to_string(),
                    "preserve-breaks".to_string()
                ),
                ("text-wrap-mode".into(), "wrap".into())
            ]
        );
        let td = names(expand(d("text-decoration", "underline wavy red 2px")).unwrap());
        assert!(td.contains(&("text-decoration-style".into(), "wavy".into())));
        assert!(td.contains(&("text-decoration-color".into(), "red".into())));
        assert!(expand(d("font-weight", "heavy")).is_err());
        assert!(expand(d("font-feature-settings", "\"liga\" 0, \"tnum\"")).is_ok());
    }
}
