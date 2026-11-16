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
    "hyphenate-character",
    "hyphenate-limit-chars",
    "word-break",
    "line-break",
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
    "font-palette",
    "font-synthesis-weight",
    "font-synthesis-style",
    "font-synthesis-small-caps",
    "font-synthesis-position",
    "font-size-adjust",
    "font-language-override",
    "text-underline-position",
    "text-underline-offset",
    "text-decoration-skip-ink",
    "text-decoration-skip-spaces",
    "-webkit-text-stroke-width",
    "-webkit-text-stroke-color",
    "-webkit-text-fill-color",
    "paint-order",
    "caret-color",
    "caret-shape",
    "accent-color",
    "color-scheme",
    // CSS Color Adjust 1 §3.2: whether forced colours mode forces its
    // colours (the cascade reads it, `css::forced_colors`).
    "forced-color-adjust",
    // CSS Color HDR 1 §3.1: how far over SDR white the frame may go (the
    // documents' roots' value is the frame's, `Ui::range_limit`).
    "dynamic-range-limit",
    // CSS Text Decoration 4 §3: emphasis marks.
    "text-emphasis-style",
    "text-emphasis-color",
    "text-emphasis-position",
    // CSS Ruby 1 §4, §5: how a text's ruby is set.
    "ruby-position",
    "ruby-align",
    "ruby-overhang",
    // The named families of `font-family` (its generic family is the
    // `font` role): a quoted, comma-separated list or `none`.
    "-og-font-name",
    // The element's `lang` attribute (empty: unknown), which an element
    // without one inherits.
    "-og-lang",
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
    if d.name == "glyph-orientation-vertical" {
        // SVG 1.1's name: an alias of `text-orientation` (CSS Writing
        // Modes 4 §5.1.3), mapped as it is read.
        let v = d.source.trim();
        let mapped = crate::layout::flow::glyph_orientation(v).ok_or_else(|| {
            format!("glyph-orientation-vertical takes auto, 0deg or 90deg, not {v}")
        })?;
        return Ok(vec![declaration("text-orientation", mapped, d.important)?]);
    }
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
            | "text-emphasis"
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
                    "underline" | "overline" | "line-through" | "blink" | "spelling-error"
                    | "grammar-error" => lines.push(item),
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
        "text-emphasis" => {
            // CSS Text Decoration 4 §3.3: a style and a colour, each its
            // initial value when omitted.
            let (mut style, mut color) = (Vec::new(), "currentcolor".to_string());
            for item in split_top(v) {
                if item == "none"
                    || item.starts_with(['"', '\''])
                    || EMPHASIS.contains(&item.as_str())
                {
                    style.push(item);
                } else {
                    crate::css_color::parse_str(&item)
                        .map_err(|_| format!("text-emphasis: '{item}' is not a value"))?;
                    color = item;
                }
            }
            let style = if style.is_empty() {
                "none".to_string()
            } else {
                style.join(" ")
            };
            all(&[
                ("text-emphasis-style", style.as_str()),
                ("text-emphasis-color", color.as_str()),
            ])
        }
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
        "inherit" | "initial" | "unset" | "revert" | "revert-layer" | "revert-rule"
    )
}

/// A CSS Fonts 4 §2.1 list, preserving generic positions and quoted names.
/// The separate role supplies the final fallback when no entry has a glyph.
pub(crate) fn family_list(v: &str) -> Result<(Vec<crate::FontFamily>, String), String> {
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
                "monospace" => Some("mono"),
                "fantasy" | "cursive" => Some("display"),
                "serif" | "sans-serif" | "emoji" | "fangsong" => Some("body"),
                // CSS Fonts 4 §2.1.3: these match only a face that suits
                // them, else the list goes on. The style's UI face is the
                // system's and its sans-serif; its own mono and rounded
                // faces are its ui-monospace and ui-rounded
                // (`Fonts::family_list`); it has no serif or math face.
                "system-ui" | "ui-sans-serif" => {
                    names.push(crate::FontFamily::Ui(crate::FontRole::Body));
                    continue;
                }
                "ui-monospace" => {
                    names.push(crate::FontFamily::Ui(crate::FontRole::Mono));
                    continue;
                }
                "ui-rounded" => {
                    names.push(crate::FontFamily::Ui(crate::FontRole::Rounded));
                    continue;
                }
                "ui-serif" | "math" => continue,
                _ => None,
            }
        };
        if let Some(g) = generic {
            role.get_or_insert(g);
            names.push(crate::FontFamily::Generic(
                crate::FontRole::from_name(g).unwrap(),
            ));
            continue;
        }
        if !quoted && wide(&item) {
            return Err("font-family: a CSS-wide keyword cannot occur in a family list".into());
        }
        names.push(crate::FontFamily::Named(item));
    }
    Ok((names, format!("var(--font-{})", role.unwrap_or("body"))))
}

fn is_size(s: &str) -> bool {
    SIZE_KEYWORDS.iter().any(|(k, _)| *k == s) || matches!(s, "larger" | "smaller") || is_length(s)
}

fn is_length(s: &str) -> bool {
    let (value, _) = super::calc::absolute(s, super::calc::Basis::default(), "");
    length_of(&value, 16.0, 16.0).is_some()
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

/// A `font-stretch` (`font-width`) keyword's or percentage's width, a
/// percentage of the normal (CSS Fonts 4 §2.4); a negative one is none.
pub(crate) fn stretch_percent(v: &str) -> Option<f32> {
    STRETCH
        .iter()
        .find(|(k, _)| *k == v)
        .map(|(_, p)| *p)
        .or_else(|| v.strip_suffix('%')?.parse::<f32>().ok())
        .filter(|p| *p >= 0.0)
}

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

/// `text-emphasis-style`'s keywords (CSS Text Decoration 4 §3.1).
const EMPHASIS: &[&str] = &[
    "filled",
    "open",
    "dot",
    "circle",
    "double-circle",
    "triangle",
    "sesame",
];

/// Checks a longhand's value as written (no `var()`).
fn check(name: &str, v: &str) -> Result<(), String> {
    let v = v.trim();
    if matches!(
        v,
        "inherit" | "initial" | "unset" | "revert" | "revert-layer" | "revert-rule"
    ) {
        return Ok(());
    }
    let length = is_length;
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
        "hyphenate-character" => v == "auto" || hyphen_string(v).is_some(),
        "hyphenate-limit-chars" => HyphenLimits::parse(v).is_some(),
        "word-break" => matches!(
            v,
            "normal" | "break-all" | "keep-all" | "break-word" | "auto-phrase"
        ),
        "overflow-wrap" => matches!(v, "normal" | "anywhere" | "break-word"),
        "line-break" => matches!(v, "auto" | "loose" | "normal" | "strict" | "anywhere"),
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
        "font-size-adjust" => size_adjust(v).is_some(),
        "font-language-override" => v == "normal" || language_tag(v).is_some(),
        "font-feature-settings" => v == "normal" || feature_settings(v).is_ok(),
        "font-variation-settings" => v == "normal" || variation_settings(v).is_ok(),
        "font-palette" => {
            og_css::tokens(v).is_ok_and(|t| matches!(t.as_slice(), [t] if palette(t)))
        }
        "text-underline-position" => {
            return words_in(name, v, &["auto", "from-font", "under", "left", "right"]);
        }
        "text-decoration-skip-ink" => matches!(v, "auto" | "none" | "all"),
        "text-decoration-skip-spaces" => SkipSpaces::parse(v).is_some(),
        "text-emphasis-style" => v == "none" || Emphasis::parse(v, "over", None).is_some(),
        "text-emphasis-position" => {
            let w: Vec<&str> = v.split_whitespace().collect();
            let sides = w.iter().filter(|w| matches!(**w, "over" | "under")).count();
            let ends = w.iter().filter(|w| matches!(**w, "right" | "left")).count();
            sides == 1 && ends <= 1 && w.len() == sides + ends
        }
        "ruby-position" => {
            let w: Vec<&str> = v.split_whitespace().collect();
            v == "inter-character"
                || (!w.is_empty()
                    && w.len() <= 2
                    && w.iter()
                        .all(|w| matches!(*w, "alternate" | "over" | "under"))
                    && w.iter().filter(|w| **w != "alternate").count() <= 1
                    && w.iter().filter(|w| **w == "alternate").count() <= 1)
        }
        "ruby-align" => matches!(v, "start" | "center" | "space-between" | "space-around"),
        "ruby-overhang" => matches!(v, "auto" | "none" | "spaces"),
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
        "caret-shape" => matches!(v, "auto" | "bar" | "block" | "underscore"),
        "forced-color-adjust" => matches!(v, "auto" | "none" | "preserve-parent-color"),
        "dynamic-range-limit" => range_limit(v).is_some(),
        "caret-color" | "accent-color" if v == "auto" => true,
        "text-decoration-color"
        | "text-emphasis-color"
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
    /// `text-decoration-thickness`.
    pub thickness: DecorationLength,
    /// `text-underline-offset`: from the baseline, or from the under edge
    /// when `under`.
    pub offset: DecorationLength,
    /// `text-underline-position: under`.
    pub under: bool,
    /// `text-decoration-skip-ink` of the box the lines come from.
    pub skip_ink: SkipInk,
    /// The text's own `text-decoration-skip-spaces`.
    pub skip_spaces: SkipSpaces,
}

/// `text-decoration-skip-spaces` (CSS Text Decoration 4 §2.10.3): which
/// spacers every decoration line skips, with the letter and word
/// spacing next to them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkipSpaces {
    /// Spacers at the start of a line.
    pub start: bool,
    /// Spacers at the end of a line.
    pub end: bool,
    /// Every spacer and word separator.
    pub all: bool,
}

impl Default for SkipSpaces {
    /// The initial `start end`.
    fn default() -> SkipSpaces {
        SkipSpaces {
            start: true,
            end: true,
            all: false,
        }
    }
}

impl SkipSpaces {
    /// `none | all | [ start || end ]`.
    pub fn parse(v: &str) -> Option<SkipSpaces> {
        let none = SkipSpaces {
            start: false,
            end: false,
            all: false,
        };
        match v.trim() {
            "none" => return Some(none),
            "all" => {
                return Some(SkipSpaces {
                    start: true,
                    end: true,
                    all: true,
                });
            }
            _ => {}
        }
        let mut out = none;
        for w in v.split_whitespace() {
            match w {
                "start" if !out.start => out.start = true,
                "end" if !out.end => out.end = true,
                _ => return None,
            }
        }
        (out.start || out.end).then_some(out)
    }
}

/// `text-decoration-skip-ink` (CSS Text Decoration 4 §2.10.4): whether
/// underlines and overlines break where they would cross glyph ink.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkipInk {
    /// Broken around the ink of every script but CJK's.
    #[default]
    Auto,
    None,
    /// Broken around every glyph's ink, CJK's too.
    All,
}

/// A decoration length: the engine's choice, the font's own metric, or px.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum DecorationLength {
    #[default]
    Auto,
    FromFont,
    Px(f32),
}

impl DecorationLength {
    fn parse(v: Option<&str>) -> DecorationLength {
        match v {
            Some("from-font") => DecorationLength::FromFont,
            v => v
                .and_then(|v| v.trim_end_matches("px").parse().ok())
                .map_or(DecorationLength::Auto, DecorationLength::Px),
        }
    }
}

impl Decoration {
    pub fn any(&self) -> bool {
        self.underline || self.overline || self.through
    }

    /// How far a line of `style` drawn `th` thick reaches below its top
    /// ([`crate::rich::decoration_line`]'s geometry).
    pub fn extent(&self, th: f32) -> f32 {
        match self.style {
            DecorationStyle::Double => th * 3.0,
            DecorationStyle::Wavy => th * 3.4,
            _ => th,
        }
    }

    /// Its lines' thickness and its underline's top below the baseline,
    /// px, for text `size` px tall in a font of metrics `m`, its px
    /// lengths scaled by `scale` (CSS Text Decoration 4 §2.4, §2.6, §2.7).
    /// `auto` is a fourteenth of an em thick, 0.12 em below the baseline;
    /// `from-font` takes the font's `post` underline. `under` starts the
    /// offset at the text's under edge, the font's descent, so the line
    /// clears every descender.
    pub fn placed(&self, m: &crate::FontMetrics, size: f32, scale: f32, dpr: f32) -> (f32, f32) {
        let font = m.underline();
        let thickness = match (self.thickness, font) {
            (DecorationLength::Px(t), _) => t * scale,
            (DecorationLength::FromFont, Some((_, t))) => (t * size).max(1.0 / dpr),
            _ => (size * 0.07).max(1.0 / dpr),
        };
        let offset = match (self.offset, font) {
            (DecorationLength::Px(o), _) => o * scale,
            _ if self.under => 0.0,
            (DecorationLength::FromFont, Some((top, _))) => top * size,
            _ => size * 0.12,
        };
        let edge = if self.under {
            m.under_edge() * size
        } else {
            0.0
        };
        (thickness, edge + offset)
    }
}

/// CSS `text-emphasis` (CSS Text Decoration 4 §3): the mark drawn once
/// for each typographic character unit, its colour and its side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Emphasis {
    pub mark: EmphasisMark,
    /// `text-emphasis-color`; `None`: the text's colour (`currentcolor`).
    pub color: Option<Srgba>,
    /// `text-emphasis-position: under` (else over), across a line.
    pub under: bool,
    /// `text-emphasis-position: … left` (else right), down a line.
    pub left: bool,
}

/// `text-emphasis-style`'s mark.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmphasisMark {
    /// A shape, filled or open. `None`: `filled` or `open` alone, a circle
    /// across a line and a sesame down one (§3.1).
    Shape(Option<EmphasisShape>, bool),
    /// A `<string>`: its first character, drawn as text.
    Char(char),
}

/// The shapes of §3.1: U+2022/25E6, U+25CF/25CB, U+25C9/25CE, U+25B2/25B3
/// and U+FE45/FE46, which the engine draws as shapes in any font.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmphasisShape {
    Dot,
    Circle,
    DoubleCircle,
    Triangle,
    Sesame,
}

impl Emphasis {
    /// The emphasis a computed `text-emphasis-style` and
    /// `text-emphasis-position` give (`None` for `none` or a value that
    /// is not one).
    pub(crate) fn parse(style: &str, position: &str, color: Option<Srgba>) -> Option<Emphasis> {
        let style = style.trim();
        let mark = if let Some(q) = style.chars().next().filter(|c| matches!(c, '"' | '\'')) {
            let inner = style.strip_prefix(q)?.strip_suffix(q)?;
            EmphasisMark::Char(inner.chars().next()?)
        } else {
            let (mut shape, mut fill) = (None, None);
            for w in style.split_whitespace() {
                match w {
                    "filled" | "open" if fill.is_none() => fill = Some(w == "filled"),
                    "dot" | "circle" | "double-circle" | "triangle" | "sesame"
                        if shape.is_none() =>
                    {
                        shape = Some(match w {
                            "dot" => EmphasisShape::Dot,
                            "circle" => EmphasisShape::Circle,
                            "double-circle" => EmphasisShape::DoubleCircle,
                            "triangle" => EmphasisShape::Triangle,
                            _ => EmphasisShape::Sesame,
                        })
                    }
                    _ => return None,
                }
            }
            if shape.is_none() && fill.is_none() {
                return None;
            }
            EmphasisMark::Shape(shape, fill.unwrap_or(true))
        };
        let has = |w: &str| position.split_whitespace().any(|p| p == w);
        Some(Emphasis {
            mark,
            color,
            under: has("under"),
            left: has("left"),
        })
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
    /// A synthesised oblique's angle in degrees (`oblique 10deg`, 14 when
    /// it names none, CSS Fonts 4 §2.4); `None` for italic, which slants
    /// as the engine's markup italic does.
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
    /// `hyphenate-character`: the string shown at a hyphenated break
    /// (`None`: `auto`, a hyphen-minus).
    pub hyphen: Option<String>,
    /// `hyphenate-limit-chars`.
    pub hyphen_limits: HyphenLimits,
    pub word_break: Option<String>,
    pub line_break: Option<String>,
    pub overflow_wrap: Option<String>,
    pub justify: Option<String>,
    pub features: Vec<([u8; 4], u16)>,
    pub variations: Vec<([u8; 4], f32)>,
    pub kerning: Option<bool>,
    pub optical_sizing: Option<bool>,
    /// `font-palette`: `light`, `dark` or an `@font-palette-values` name
    /// (`None`: `normal`), resolved against the document's sheet
    /// ([`super::at_rules::Sheet::palette`]).
    pub palette: Option<String>,
    /// The language its runs are shaped in (BCP 47): the one that shapes
    /// in `font-language-override`'s OpenType language system (`ro` for
    /// "ROM"; one no language shapes in first as HarfBuzz's private use
    /// writes it, `x-hbotabcd`), else the element's `lang`.
    pub language: Option<String>,
    /// `font-variant-emoji` (`None`: `normal`, which presents as
    /// `unicode` does).
    pub emoji: Option<crate::text::EmojiStyle>,
    /// `font-size-adjust` (`Some(None)`: `none`).
    pub size_adjust: Option<Option<crate::text::SizeAdjust>>,
    pub synthesis: [Option<bool>; 4],
    pub decoration: Decoration,
    pub stroke: Option<(f32, Option<Srgba>)>,
    pub fill: Option<Srgba>,
    pub paint_order: Option<String>,
    pub caret: Option<Srgba>,
    pub caret_shape: Option<CaretShape>,
    pub accent: Option<Srgba>,
    /// The used colour scheme is dark (`None`: the style's).
    pub dark: Option<bool>,
    pub vertical_align: Option<String>,
    /// Named and generic families in authored order, skipping unavailable names.
    pub family: Option<Vec<crate::FontFamily>>,
    pub font_role: Option<crate::FontRole>,
    /// `text-emphasis` (CSS Text Decoration 4 §3).
    pub emphasis: Option<Emphasis>,
    /// `ruby-position`, `ruby-align` and `ruby-overhang` (CSS Ruby 1).
    pub ruby: crate::rich::RubyStyle,
}

/// CSS UI 4 `caret-shape`: how a focused field draws its caret (`auto`
/// draws a bar).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CaretShape {
    #[default]
    Auto,
    Bar,
    /// Over the next character's advance, under the text.
    Block,
    /// A thin line under the next character's advance.
    Underscore,
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
    /// The document's `@font-feature-values` rules.
    pub feature_values: &'a [super::at_rules::FontFeatureValues],
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
        feature_values,
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
    // The element's own font size first: every other length's `em` and
    // `%` is against it, wherever its rule declares it.
    let sized = local.iter().filter(|(n, _)| n == "font-size");
    for (name, value) in sized.chain(local.iter().filter(|(n, _)| n != "font-size")) {
        let value = value.trim();
        let computed_value = match value {
            "inherit" | "unset" if INHERITED.contains(&name.as_str()) => continue,
            "initial" | "revert" | "revert-layer" | "revert-rule" | "unset" => {
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
            .map(super::css::hex)
            .unwrap_or_else(|| "-".into());
        let thick = own("text-decoration-thickness").map_or("-".into(), |v| {
            if v.trim() == "from-font" {
                "from-font".into()
            } else {
                length_of(&v, own_size, own_size).map_or("-".into(), |v| v.to_string())
            }
        });
        // §2.10.4: the skip-ink of the box the lines come from.
        reads.insert(key("text-decoration-skip-ink"));
        let skip = computed
            .get(&key("text-decoration-skip-ink"))
            .cloned()
            .flatten()
            .unwrap_or_else(|| "auto".into());
        lines = format!("{lines}|{l};{style};{color};{thick};{skip}");
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
        .or_else(|| get("-og-font-name").and_then(|v| self::family_list(&v).ok().map(|v| v.0)))
        .map(|names| {
            names
                .into_iter()
                .filter(|n| match n {
                    crate::FontFamily::Named(name) => declared(name),
                    crate::FontFamily::Generic(_) | crate::FontFamily::Ui(_) => true,
                })
                .collect()
        });
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
        hyphen: get("hyphenate-character").and_then(|v| hyphen_string(&v)),
        hyphen_limits: get("hyphenate-limit-chars")
            .and_then(|v| HyphenLimits::parse(&v))
            .unwrap_or_default(),
        word_break: get("word-break"),
        line_break: get("line-break"),
        overflow_wrap: get("overflow-wrap"),
        justify: get("text-justify"),
        kerning: get("font-kerning").map(|v| v != "none"),
        optical_sizing: get("font-optical-sizing").map(|v| v != "none"),
        palette: get("font-palette").filter(|v| v != "normal"),
        emoji: match get("font-variant-emoji").as_deref() {
            Some("text") => Some(crate::text::EmojiStyle::Text),
            Some("emoji") => Some(crate::text::EmojiStyle::Emoji),
            Some("unicode") => Some(crate::text::EmojiStyle::Unicode),
            _ => None,
        },
        size_adjust: get("font-size-adjust").and_then(|v| size_adjust(&v)),
        language: get("font-language-override")
            .and_then(|v| language_tag(&v))
            .map(|tag| {
                crate::shape::language_of_tag(&tag).map_or_else(
                    || format!("x-hbot{}", tag.to_ascii_lowercase()),
                    str::to_string,
                )
            })
            .or_else(|| get("-og-lang").map(|l| l.trim().trim_matches('"').to_string())),
        caret: color(get("caret-color")),
        caret_shape: get("caret-shape").map(|v| match v.as_str() {
            "bar" => CaretShape::Bar,
            "block" => CaretShape::Block,
            "underscore" => CaretShape::Underscore,
            _ => CaretShape::Auto,
        }),
        accent: color(get("accent-color")),
        fill: color(get("-webkit-text-fill-color")),
        paint_order: get("paint-order").filter(|v| v != "normal"),
        family,
        font_role,
        ..TextCss::default()
    };
    match get("font-style").as_deref() {
        Some("normal") => t.italic = Some(false),
        Some("italic") => t.italic = Some(true),
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
    // Alternates' functions take the values their names have for the
    // text's families (CSS Fonts 4 §6.8); a name none defines is ignored.
    if let Some(v) = get("font-variant-alternates") {
        let names: Vec<String> = t
            .family
            .iter()
            .flatten()
            .filter_map(|f| match f {
                crate::FontFamily::Named(n) => Some(n.clone()),
                _ => None,
            })
            .collect();
        for item in split_top(&v) {
            let Some((f, args)) = item.strip_suffix(')').and_then(|i| i.split_once('(')) else {
                continue;
            };
            for name in args.split(',').map(str::trim) {
                let Some(values) = super::at_rules::feature_value(feature_values, f, name, &names)
                else {
                    continue;
                };
                for (tag, value) in alternate_features(f, values) {
                    on(&tag, value);
                }
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
    // A spelling or grammar error's line (CSS Text Decoration 4 §2.1):
    // the UA's wavy underline in the theme's danger or success colour,
    // which no other decoration property changes (§2.8, §2.10).
    let mut error = None;
    for part in lines.split('|').filter(|p| !p.is_empty()) {
        let f: Vec<&str> = part.split(';').collect();
        let d = &mut t.decoration;
        for l in f[0].split_whitespace() {
            match l {
                "underline" => d.underline = true,
                "overline" => d.overline = true,
                "line-through" => d.through = true,
                "spelling-error" => error = Some(theme.palette.danger),
                "grammar-error" => error = Some(theme.palette.success),
                // `blink`: the text does not blink, as §2.1 allows.
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
        d.thickness = DecorationLength::parse(f.get(3).copied());
        d.skip_ink = match f.get(4).copied() {
            Some("none") => SkipInk::None,
            Some("all") => SkipInk::All,
            _ => SkipInk::Auto,
        };
    }
    t.decoration.skip_spaces = get("text-decoration-skip-spaces")
        .and_then(|v| SkipSpaces::parse(&v))
        .unwrap_or_default();
    t.decoration.offset = DecorationLength::parse(get("text-underline-offset").as_deref());
    t.decoration.under = get("text-underline-position").is_some_and(|v| v.contains("under"));
    if let Some(ink) = error {
        t.decoration = Decoration {
            underline: true,
            style: DecorationStyle::Wavy,
            color: Some(ink),
            skip_ink: SkipInk::None,
            ..Decoration::default()
        };
    }
    if let Some(w) = num(get("-webkit-text-stroke-width")).filter(|w| *w > 0.0) {
        t.stroke = Some((w, color(get("-webkit-text-stroke-color"))));
    }
    let theme_dark = theme.scheme == crate::Scheme::Dark;
    t.dark = get("color-scheme")
        .map(|v| scheme_dark(&v, theme_dark))
        .filter(|d| *d != theme_dark);
    t.vertical_align = own("vertical-align");
    let mark_color = get("text-emphasis-color")
        .filter(|c| !c.eq_ignore_ascii_case("currentcolor"))
        .and_then(|c| color(Some(c)));
    let position = get("text-emphasis-position").unwrap_or_default();
    t.emphasis =
        get("text-emphasis-style").and_then(|style| Emphasis::parse(&style, &position, mark_color));
    t.ruby = crate::rich::RubyStyle::parse(
        get("ruby-position").as_deref(),
        get("ruby-align").as_deref(),
        get("ruby-overhang").as_deref(),
    );
    t
}

/// The used colour scheme of a `color-scheme` value (CSS Color Adjust 1
/// §2): one that names only `dark` (or only `light`) is it; `normal`, or
/// both, is the style's (`style_dark`), the scheme the player chose.
pub(crate) fn scheme_dark(value: &str, style_dark: bool) -> bool {
    let has = |w: &str| value.split_whitespace().any(|v| v == w);
    match (has("light"), has("dark")) {
        (true, false) => false,
        (false, true) => true,
        _ => style_dark,
    }
}

/// A `dynamic-range-limit` value as its computed weights (CSS Color HDR 1
/// §3.3): a keyword is all of its own; `dynamic-range-limit-mix()`
/// normalizes its percentages to sum to 100 % and sums each argument's
/// weights by them, nested mixes included. `None`: not a value.
pub(crate) fn range_limit(v: &str) -> Option<og_draw::RangeLimit> {
    fn of(t: &[og_css::Tok]) -> Option<og_draw::RangeLimit> {
        use og_css::Tok;
        let keyword = |standard, constrained, no_limit| og_draw::RangeLimit {
            standard,
            constrained,
            no_limit,
        };
        match t {
            [Tok::Ident(w)] => match w.to_ascii_lowercase().as_str() {
                "standard" => Some(keyword(1.0, 0.0, 0.0)),
                "constrained" => Some(keyword(0.0, 1.0, 0.0)),
                "no-limit" => Some(keyword(0.0, 0.0, 1.0)),
                _ => None,
            },
            [Tok::Func(f, args)] if f.eq_ignore_ascii_case("dynamic-range-limit-mix") => {
                let mut parts = Vec::new();
                for arg in args {
                    let at = arg
                        .iter()
                        .position(|t| matches!(t, Tok::Num(_, u) if u == "%"))?;
                    let Tok::Num(n, _) = &arg[at] else {
                        return None;
                    };
                    let p: f32 = n.parse().ok().filter(|p| (0.0..=100.0).contains(p))?;
                    let rest: Vec<Tok> = arg
                        .iter()
                        .enumerate()
                        .filter(|(k, _)| *k != at)
                        .map(|(_, t)| t.clone())
                        .collect();
                    parts.push((of(&rest)?, p));
                }
                let sum: f32 = parts.iter().map(|(_, p)| p).sum();
                if parts.is_empty() || sum <= 0.0 {
                    return None;
                }
                let mut mixed = keyword(0.0, 0.0, 0.0);
                for (l, p) in parts {
                    let w = p / sum;
                    mixed.standard += l.standard * w;
                    mixed.constrained += l.constrained * w;
                    mixed.no_limit += l.no_limit * w;
                }
                Some(mixed)
            }
            _ => None,
        }
    }
    of(&og_css::tokens(v.trim()).ok()?)
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

/// `font-language-override`'s OpenType language system tag: a quoted one
/// to four letters (CSS Fonts 4 §6.13), trailing spaces dropped.
/// A `font-size-adjust` value (CSS Fonts 5 §2.5): `Some(None)` for
/// `none`, else its metric and its number or `from-font`.
fn size_adjust(v: &str) -> Option<Option<crate::text::SizeAdjust>> {
    use crate::text::AspectMetric;
    let words: Vec<&str> = v.split_whitespace().collect();
    if words == ["none"] {
        return Some(None);
    }
    let (metric, value) = match words.as_slice() {
        [value] => (AspectMetric::ExHeight, *value),
        [metric, value] => (
            match *metric {
                "ex-height" => AspectMetric::ExHeight,
                "cap-height" => AspectMetric::CapHeight,
                "ch-width" => AspectMetric::ChWidth,
                "ic-width" => AspectMetric::IcWidth,
                "ic-height" => AspectMetric::IcHeight,
                _ => return None,
            },
            *value,
        ),
        _ => return None,
    };
    let value = match value {
        "from-font" => None,
        n => Some(
            n.parse::<f32>()
                .ok()
                .filter(|n| n.is_finite() && *n >= 0.0)
                .map(|n| (n * 1000.0).round() as u32)?,
        ),
    };
    Some(Some(crate::text::SizeAdjust { metric, value }))
}

fn language_tag(v: &str) -> Option<String> {
    let inner = v
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .or_else(|| v.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))?;
    let tag = inner.trim_end();
    (!tag.is_empty() && inner.len() <= 4 && tag.bytes().all(|b| b.is_ascii_alphanumeric()))
        .then(|| tag.to_string())
}

/// The OpenType features an alternates function sets from a name's
/// values (CSS Fonts 4 §6.8): `salt`, `ssNN` for each value, `cvNN` set to
/// its second value (else 1), `swsh` and `cswh`, `ornm` and `nalt`. A
/// value outside a feature's range sets nothing.
fn alternate_features(function: &str, values: &[u16]) -> Vec<([u8; 4], u16)> {
    let numbered = |prefix: &str, n: u16| -> [u8; 4] {
        let t = format!("{prefix}{n:02}");
        t.as_bytes().try_into().expect("four bytes")
    };
    let first = values.first().copied().unwrap_or(0);
    match function {
        "stylistic" => vec![(*b"salt", first)],
        "styleset" => values
            .iter()
            .filter(|v| (1..=20).contains(*v))
            .map(|&v| (numbered("ss", v), 1))
            .collect(),
        "character-variant" if (1..=99).contains(&first) => {
            vec![(numbered("cv", first), values.get(1).copied().unwrap_or(1))]
        }
        "swash" => vec![(*b"swsh", first), (*b"cswh", first)],
        "ornaments" => vec![(*b"ornm", first)],
        "annotation" => vec![(*b"nalt", first)],
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
        "text-underline-offset" if v == "from-font" => v.into(),
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
                crate::css_color::parse_str(&super::css::hex(current))?
            } else {
                c
            };
            match c.literal() {
                Some(l) => super::css::hex(l),
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
        step.features = step.features.with_settings(&self.features);
        if self.language.is_some() {
            step.features = step.features.with_language(self.language.as_deref());
        }
        step
    }

    /// Whether the text needs a span of its own (a style, word spacing, a
    /// decoration, a stroke or a fill the plain text path does not draw),
    /// or is a
    /// paragraph Parley lays out (an indent, a `line-break`).
    pub fn rich(&self) -> bool {
        let p = self.paragraph();
        self.italic == Some(true)
            || self.word_spacing.is_some_and(|w| w != 0.0)
            || self.decoration.any()
            || self.stroke.is_some()
            || self.emphasis.is_some()
            || self.hyphen.is_some()
            || p.0 != Default::default()
            || p.1 != Default::default()
    }

    /// Its paragraph's `text-indent` and `line-break`.
    pub fn paragraph(&self) -> (crate::rich::TextIndent, crate::rich::LineBreak) {
        use crate::rich::{LineBreak, TextIndent};
        let indent = self
            .indent
            .map_or(TextIndent::default(), |(px, hanging, each_line)| {
                TextIndent {
                    px,
                    hanging,
                    each_line,
                }
            });
        let line_break = match self.line_break.as_deref() {
            Some("loose") => LineBreak::Loose,
            Some("normal") => LineBreak::Normal,
            Some("strict") => LineBreak::Strict,
            Some("anywhere") => LineBreak::Anywhere,
            _ => LineBreak::Auto,
        };
        (indent, line_break)
    }

    /// Whether its white space keeps spaces or line breaks the document's
    /// HTML collapsing removed.
    pub fn preserves(&self) -> bool {
        self.collapse
            .is_some_and(|c| !matches!(c, Collapse::Collapse | Collapse::Discard))
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
                let mut chars = text.chars().peekable();
                while let Some(c) = chars.next() {
                    let next = chars.peek();
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
                        // run keeps its width; a wrapping line may break
                        // after a run (break-spaces after each space).
                        ' ' if collapse == Collapse::Preserve
                            && (self.nowrap == Some(true) || next == Some(&' ')) =>
                        {
                            out.push('\u{a0}');
                            col += 1;
                        }
                        // break-spaces: each space keeps its room, even at
                        // a line's end, and a line may break after it.
                        ' ' if collapse == Collapse::BreakSpaces => {
                            out.push_str("\u{a0}\u{200b}");
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
        // hyphens: auto (CSS Text 4 §6.1): soft hyphens at the hyphenation
        // points of the language's dictionary (shipped in its font pack),
        // which the line breakers take as manual ones.
        if self.hyphens.as_deref() == Some("auto") {
            let l = self.hyphen_limits;
            s = og_i18n::hyphenation::hyphenate(&s, locale, l.word.unwrap_or(5), l.before, l.after);
        }
        s
    }
}

/// `hyphenate-limit-chars` (CSS Text 4 §6.3): the fewest characters of
/// a word that hyphenates and of its parts before and after a break
/// (`None`: `auto`, five for a word and the language's own minimums).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HyphenLimits {
    pub word: Option<usize>,
    pub before: Option<usize>,
    pub after: Option<usize>,
}

impl HyphenLimits {
    /// `[ auto | <integer> ]{1,3}`: an omitted after is the before, an
    /// omitted before `auto`.
    pub fn parse(v: &str) -> Option<HyphenLimits> {
        let values: Vec<Option<usize>> = v
            .split_whitespace()
            .map(|w| match w {
                "auto" => Some(None),
                n => n.parse::<usize>().ok().map(Some),
            })
            .collect::<Option<_>>()?;
        match values[..] {
            [word] => Some(HyphenLimits {
                word,
                ..Default::default()
            }),
            [word, before] => Some(HyphenLimits {
                word,
                before,
                after: before,
            }),
            [word, before, after] => Some(HyphenLimits {
                word,
                before,
                after,
            }),
            _ => None,
        }
    }
}

/// `hyphenate-character`'s `<string>`.
fn hyphen_string(v: &str) -> Option<String> {
    let mut p = cssparser::Parser::new(v);
    let s = p.expect_string().ok()?.to_string();
    p.is_exhausted().then_some(s)
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

/// Whether a token is a `font-palette` value: `normal`, `light`, `dark`,
/// a dashed name or a `palette-mix()` of them (CSS Fonts 4 §9.1).
fn palette(t: &og_css::Tok) -> bool {
    use og_css::Tok;
    match t {
        Tok::Ident(w) => matches!(w.as_str(), "normal" | "light" | "dark") || w.starts_with("--"),
        Tok::Func(f, args) if f == "palette-mix" => super::at_rules::palette_mix_method(args)
            .is_some_and(|(_, items)| {
                items.iter().all(|i| match i.as_slice() {
                    [p] => palette(p),
                    [p, Tok::Num(n, u)] | [Tok::Num(n, u), p] if u == "%" => {
                        palette(p) && n.parse::<f32>().is_ok_and(|n| (0.0..=100.0).contains(&n))
                    }
                    _ => false,
                })
            }),
        _ => false,
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
        // pre keeps every space unbreakable; pre-wrap may break after a run.
        let pre = TextCss {
            nowrap: Some(true),
            ..t(Collapse::Preserve)
        };
        assert_eq!(pre.shown("a  b", "en"), "a\u{a0}\u{a0}b");
        assert_eq!(t(Collapse::Preserve).shown("a  b", "en"), "a\u{a0} b");
        // A tab goes to the next stop of tab-size columns.
        let tab = TextCss {
            collapse: Some(Collapse::BreakSpaces),
            tab_size: Some(4.0),
            ..TextCss::default()
        };
        assert_eq!(tab.shown("ab\tc", "en"), "ab\u{a0}\u{a0}c");
        assert_eq!(tab.shown("a b", "en"), "a\u{a0}\u{200b}b");
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
