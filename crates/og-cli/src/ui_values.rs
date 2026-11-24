//! The values a game's TSX writes by hand where the theme has a step or a
//! role (crates/og-ui-next/CONVENTIONS.md §Theme and packs, Steps and
//! roles). `og check` reports each as warning OG9109 at the class, naming
//! the class that replaces it:
//!
//! - a bracketed value off the theme's scale (`text-[1.714rem]`,
//!   `w-[17.143rem]`, `tracking-[0.14em]`, `rounded-[4.576px]`): the step
//!   (`text-title`, `w-60`, `tracking-label`, `rounded-control`), the
//!   nearest one where none is exact, at the theme's rem and spacing;
//! - a bracketed percentage (`w-[40%]`): the fraction (`w-2/5`);
//! - a shadow written out (`shadow-[0px_0px_12px_#38e8ff]`): the named
//!   shadows, rings and inset shadows with a role's colour (`shadow-glow`);
//! - a colour written out, a `var(--…)` or a literal `white` or `black`
//!   where [`crate::ui_grounds`] does not already report it (a border, a
//!   ring, a stroke, a shadow's colour): the role;
//! - an opacity modifier on a role (`bg-well/79`, `border-rim/50`): the
//!   whole role, or the role that names that strength (`fg-muted`).
//!
//! What it leaves alone is structure, not a value: a grid template of
//! `auto`, `fr` and `minmax(0, …fr)` tracks, `flex`, `grow`, `row` and
//! `col` numbers, an `env()` inset, a `data-[…]` variant, and an inline
//! `style` (a bar's width from data).

use serde_json::Value;

/// The check's code in `og check`'s warnings.
pub const CODE: u32 = 9109;

/// What replaces a hand-written class.
#[derive(Clone, Debug, PartialEq)]
pub struct Fix {
    /// The class or classes to write instead; empty when the fix is a
    /// restructuring the message describes.
    pub classes: String,
    /// Whether the replacement draws the same value (`false`: the nearest
    /// step or role, which moves it).
    pub exact: bool,
    pub message: String,
}

/// The theme's scales, as `Pack::theme_json` gives a scheme's theme.
#[derive(Clone, Debug, Default)]
pub struct Scale {
    rem: f32,
    spacing: f32,
    font: Vec<(String, f32)>,
    tracking: Vec<(String, f32)>,
    leading: Vec<(String, f32)>,
    radius: Vec<(String, f32)>,
    blur: Vec<(String, f32)>,
    /// Each colour's value in every scheme given.
    colors: Vec<(String, Vec<[u8; 4]>)>,
    /// The named shadows and inset shadows: `shadow-<name>` is one of
    /// them before it is a colour.
    shadows: Vec<String>,
}

/// Names a scale lists twice: the step other components already use wins.
const ALIASES: [&str; 3] = ["meta", "body", "button"];

impl Scale {
    /// The scales of `themes` (one JSON per scheme): lengths from the
    /// first, colours from all.
    pub fn new(themes: &[String]) -> Scale {
        let themes: Vec<Value> = themes
            .iter()
            .filter_map(|t| serde_json::from_str(t).ok())
            .collect();
        let mut s = Scale {
            rem: 16.0,
            spacing: 4.0,
            ..Scale::default()
        };
        let Some(first) = themes.first() else {
            return s;
        };
        let num = |key: &str| first.get(key).and_then(Value::as_f64).map(|n| n as f32);
        s.rem = num("rem").unwrap_or(16.0);
        s.spacing = num("spacing").unwrap_or(4.0);
        let section = |key: &str, of: &dyn Fn(&Value) -> Option<f64>| {
            let mut out: Vec<(String, f32)> = first
                .get(key)
                .and_then(Value::as_object)
                .into_iter()
                .flatten()
                .filter_map(|(k, v)| Some((k.clone(), of(v)? as f32)))
                .collect();
            // Ascending, an alias after the name it repeats.
            out.sort_by(|a, b| {
                (a.1, ALIASES.contains(&a.0.as_str()), &a.0)
                    .partial_cmp(&(b.1, ALIASES.contains(&b.0.as_str()), &b.0))
                    .unwrap()
            });
            out
        };
        s.font = section("fontSize", &|v| v.get(0).unwrap_or(v).as_f64());
        s.tracking = section("tracking", &Value::as_f64);
        s.leading = section("leading", &Value::as_f64);
        s.radius = section("radius", &Value::as_f64);
        s.blur = section("blur", &Value::as_f64);
        for key in ["shadow", "insetShadow"] {
            let names = first.get(key).and_then(Value::as_object);
            s.shadows
                .extend(names.into_iter().flatten().map(|(k, _)| k.clone()));
        }
        for t in &themes {
            for (name, v) in t
                .get("colors")
                .and_then(Value::as_object)
                .into_iter()
                .flatten()
            {
                let Some(c) = v.as_str().and_then(colour) else {
                    continue;
                };
                match s.colors.iter_mut().find(|(n, _)| n == name) {
                    Some((_, list)) => list.push(c),
                    None => s.colors.push((name.clone(), vec![c])),
                }
            }
        }
        s
    }

    fn has_colour(&self, name: &str) -> bool {
        self.colors.iter().any(|(n, _)| n == name)
    }

    /// The role a written colour is: one whose value it is in a scheme,
    /// else the nearest. Answers the name and whether it is that colour
    /// at that strength.
    fn role(&self, c: [u8; 4], edge: bool) -> (String, bool) {
        let [r, g, b, a] = c;
        let (light, dark) = (r.min(g).min(b) >= 245, r.max(g).max(b) <= 16);
        if (light || dark) && a < 255 {
            let n = (f32::from(a) / 255.0 * 100.0).round() as u32;
            let name = if light { white(edge, n) } else { black(n) };
            return (name.to_owned(), false);
        }
        // An edge's own roles first, a state's own colour before the
        // shades of it, then short names.
        let rank = |n: &str| {
            (
                !(edge && n.contains("rim")),
                [
                    "-hi", "-lo", "-hover", "-active", "-pressed", "-soft", "-ink", "-tone",
                ]
                .iter()
                .any(|s| n.ends_with(s)),
                n.len(),
            )
        };
        let mut best: Option<(u32, &str)> = None;
        for (name, values) in &self.colors {
            for v in values {
                let d: u32 = (0..4)
                    .map(|k| (i32::from(v[k]) - i32::from(c[k])).unsigned_abs().pow(2))
                    .sum();
                let better = match best {
                    None => true,
                    Some((bd, bn)) => d < bd || d == bd && rank(name) < rank(bn),
                };
                if better {
                    best = Some((d, name));
                }
            }
        }
        match best {
            Some((d, name)) => (name.to_owned(), d <= 3),
            None => ("fg".to_owned(), false),
        }
    }

    /// [`Scale::role`] for a shadow's colour class: a role whose name is
    /// a named shadow's gives way to its neighbour.
    fn shadow_role(&self, c: [u8; 4], edge: bool) -> String {
        let (role, _) = self.role(c, edge);
        if !self.shadows.contains(&role) {
            return role;
        }
        match role.as_str() {
            "lit" => "sheen",
            "rim" => "rim-strong",
            "glow" => "accent",
            _ => "fg",
        }
        .to_owned()
    }
}

/// The role a translucent white stands for, at `n` percent.
fn white(edge: bool, n: u32) -> &'static str {
    match (edge, n) {
        (true, 0..=10) => "rim-soft",
        (true, 11..=30) => "rim",
        (true, _) => "rim-strong",
        (false, 0..=6) => "lit-soft",
        (false, 7..=14) => "lit",
        (false, _) => "sheen",
    }
}

/// The role a translucent black stands for, at `n` percent.
fn black(n: u32) -> &'static str {
    if n <= 30 { "shade" } else { "shade-deep" }
}

/// `#rgb`, `#rrggbb`, `#rrggbbaa` or `rgb(r g b / a)` (a class writes the
/// spaces as `_`).
fn colour(text: &str) -> Option<[u8; 4]> {
    let t = text.trim();
    if let Some(h) = t.strip_prefix('#') {
        let d: Vec<u8> = h
            .chars()
            .map(|c| c.to_digit(16).map(|d| d as u8))
            .collect::<Option<_>>()?;
        return match d.len() {
            3 => Some([d[0] * 17, d[1] * 17, d[2] * 17, 255]),
            6 => Some([d[0] * 16 + d[1], d[2] * 16 + d[3], d[4] * 16 + d[5], 255]),
            8 => Some([
                d[0] * 16 + d[1],
                d[2] * 16 + d[3],
                d[4] * 16 + d[5],
                d[6] * 16 + d[7],
            ]),
            _ => None,
        };
    }
    let inner = t
        .strip_prefix("rgba(")
        .or_else(|| t.strip_prefix("rgb("))?
        .strip_suffix(')')?;
    let (rgb, alpha) = match inner.split_once('/') {
        Some((c, a)) => (c, Some(a)),
        None => (inner, None),
    };
    let mut parts: Vec<f32> = rgb
        .split(['_', ',', ' '])
        .filter(|p| !p.is_empty())
        .map(|p| p.parse().ok())
        .collect::<Option<_>>()?;
    let alpha = match alpha {
        Some(a) => Some(a.trim_matches(['_', ' '])),
        None if parts.len() == 4 => {
            let a = parts.pop()?;
            return Some([
                parts[0] as u8,
                parts[1] as u8,
                parts[2] as u8,
                (a * 255.0).round() as u8,
            ]);
        }
        None => None,
    };
    if parts.len() != 3 {
        return None;
    }
    let a = match alpha {
        Some(a) => match a.strip_suffix('%') {
            Some(p) => p.parse::<f32>().ok()? / 100.0,
            None => a.parse::<f32>().ok()?,
        },
        None => 1.0,
    };
    Some([
        parts[0] as u8,
        parts[1] as u8,
        parts[2] as u8,
        (a * 255.0).round() as u8,
    ])
}

/// A number and its unit (`1.714rem`, `-50%`, `22deg`, `1.5`).
fn quantity(text: &str) -> Option<(f32, &str)> {
    let t = text.trim();
    let at = t
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+'))
        .unwrap_or(t.len());
    let n: f32 = t[..at].parse().ok()?;
    let unit = &t[at..];
    unit.chars()
        .all(|c| c.is_ascii_alphabetic() || c == '%')
        .then_some((n, unit))
}

/// A number without trailing zeros (`1.5`, `60`).
fn plain(n: f32) -> String {
    let s = format!("{n:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// The nearest of `steps` (ascending) to `v`, the smaller on a tie, and
/// whether it is `v`.
fn nearest(steps: &[(String, f32)], v: f32) -> Option<(&str, f32, bool)> {
    let mut best: Option<&(String, f32)> = None;
    for s in steps {
        if best.is_none_or(|b| (s.1 - v).abs() < (b.1 - v).abs() - 1e-4) {
            best = Some(s);
        }
    }
    best.map(|(n, s)| (n.as_str(), *s, (s - v).abs() <= 0.011 * v.abs().max(1.0)))
}

/// The utilities that take a length on the spacing scale, and whether
/// each takes a fraction too.
const LENGTHS: [(&str, bool); 50] = [
    ("w", true),
    ("h", true),
    ("size", true),
    ("min-w", true),
    ("min-h", true),
    ("max-w", true),
    ("max-h", true),
    ("basis", true),
    ("inset", true),
    ("inset-x", true),
    ("inset-y", true),
    ("top", true),
    ("right", true),
    ("bottom", true),
    ("left", true),
    ("start", true),
    ("end", true),
    ("translate-x", true),
    ("translate-y", true),
    ("gap", false),
    ("gap-x", false),
    ("gap-y", false),
    ("p", false),
    ("px", false),
    ("py", false),
    ("pt", false),
    ("pr", false),
    ("pb", false),
    ("pl", false),
    ("ps", false),
    ("pe", false),
    ("m", false),
    ("mx", false),
    ("my", false),
    ("mt", false),
    ("mr", false),
    ("mb", false),
    ("ml", false),
    ("ms", false),
    ("me", false),
    ("space-x", false),
    ("space-y", false),
    ("indent", false),
    ("scroll-m", false),
    ("scroll-p", false),
    ("scroll-mt", false),
    ("scroll-mb", false),
    ("scroll-pt", false),
    ("scroll-pb", false),
    ("underline-offset", false),
];

/// The utilities whose value is a whole number of px.
const WIDTHS: [&str; 16] = [
    "border",
    "border-t",
    "border-r",
    "border-b",
    "border-l",
    "border-x",
    "border-y",
    "border-s",
    "border-e",
    "ring",
    "inset-ring",
    "ring-offset",
    "outline",
    "outline-offset",
    "stroke",
    "divide-x",
];

/// The utilities whose value is a colour, and whether it draws an edge.
const COLOURS: [(&str, bool); 30] = [
    ("inset-shadow", false),
    ("text-shadow", false),
    ("drop-shadow", false),
    ("inset-ring", true),
    ("ring-offset", true),
    ("border-x", true),
    ("border-y", true),
    ("border-t", true),
    ("border-r", true),
    ("border-b", true),
    ("border-l", true),
    ("border-s", true),
    ("border-e", true),
    ("border", true),
    ("divide", true),
    ("outline", true),
    ("ring", true),
    ("shadow", false),
    ("stroke", true),
    ("fill", false),
    ("decoration", false),
    ("caret", false),
    ("accent", false),
    ("placeholder", false),
    ("bg", false),
    ("from", false),
    ("via", false),
    ("to", false),
    ("text", false),
    ("text-stroke", true),
];

/// The colour utilities [`crate::ui_grounds`] reports a literal or a
/// hand-written colour in.
const GROUNDS: [&str; 5] = ["bg", "from", "via", "to", "text"];

/// Structure, not a value: numbers of rows, columns and flex factors.
const STRUCTURE: [&str; 16] = [
    "flex",
    "grow",
    "shrink",
    "order",
    "row",
    "col",
    "row-span",
    "col-span",
    "row-start",
    "col-start",
    "row-end",
    "col-end",
    "data",
    "aria",
    "content",
    "line-clamp",
];

/// A class's variants (with their `:`) and its utility.
fn parts(class: &str) -> (&str, &str) {
    let (mut depth, mut start) = (0, 0);
    for (i, c) in class.char_indices() {
        match c {
            '[' | '(' => depth += 1,
            ']' | ')' => depth -= 1,
            ':' if depth == 0 => start = i + 1,
            _ => {}
        }
    }
    (&class[..start], &class[start..])
}

fn has_written_colour(v: &str) -> bool {
    v.contains('#') || v.contains("rgb") || v.contains("hsl") || v.contains("var(--")
}

/// What replaces `class`, when it writes by hand what the theme names.
pub fn hand_written(class: &str, scale: &Scale) -> Option<Fix> {
    let (variants, utility) = parts(class);
    let important = utility.starts_with('!');
    let utility = utility.trim_start_matches('!');
    let (neg, utility) = match utility.strip_prefix('-') {
        Some(u) => (true, u),
        None => (false, utility),
    };
    let found = match utility.find("-[") {
        Some(at) if utility.ends_with(']') => bracket(
            &utility[..at],
            &utility[at + 2..utility.len() - 1],
            neg,
            scale,
        ),
        Some(_) => None,
        None if neg => None,
        None => modifier(utility, scale),
    }?;
    let (classes, exact, why) = found;
    let prefix = format!("{variants}{}", if important { "!" } else { "" });
    let classes = classes
        .split(' ')
        .filter(|c| !c.is_empty())
        .map(|c| format!("{prefix}{c}"))
        .collect::<Vec<_>>()
        .join(" ");
    let message = match (classes.is_empty(), exact) {
        (true, _) => format!("`{class}` {why}"),
        (false, true) => format!("`{class}` {why}: use `{classes}`"),
        (false, false) => format!("`{class}` {why}: use `{classes}`, the nearest the theme has"),
    };
    Some(Fix {
        classes,
        exact,
        message,
    })
}

type Found = (String, bool, String);

/// `name-[value]`.
fn bracket(name: &str, value: &str, neg: bool, s: &Scale) -> Option<Found> {
    if STRUCTURE.contains(&name) || value.contains("env(") {
        return None;
    }
    let sign = if neg { "-" } else { "" };
    let by_hand = |what: &str| format!("writes {what} by hand");
    if matches!(name, "shadow" | "inset-shadow") && value.contains('_') {
        let (classes, exact) = shadow(value, name == "inset-shadow", s);
        return Some((classes, exact, by_hand("a shadow")));
    }
    if matches!(name, "grid-cols" | "grid-rows" | "auto-rows" | "auto-cols") {
        return grid(name, value, s);
    }
    if has_written_colour(value) {
        let (_, edge) = COLOURS.iter().find(|(p, _)| *p == name)?;
        let written = value.strip_prefix("color:").unwrap_or(value);
        if let Some(role) = written
            .strip_prefix("var(--color-")
            .and_then(|r| r.strip_suffix(')'))
        {
            return Some((
                format!("{name}-{role}"),
                true,
                by_hand("a theme colour's variable"),
            ));
        }
        // ui_grounds reports a ground's or an ink's.
        if GROUNDS.contains(&name) && !value.contains("var(--") {
            return None;
        }
        return Some(match colour(written) {
            Some(c) => {
                let (role, exact) = s.role(c, *edge);
                (format!("{name}-{role}"), exact, by_hand("a colour"))
            }
            None => (
                String::new(),
                false,
                "writes a colour by hand: use theme roles (a gradient is `bg-linear-to-r from-<role> to-<role>`), so a pack restyles it".to_owned(),
            ),
        });
    }
    let (n, unit) = quantity(value).or_else(|| {
        // `7/10`, an aspect ratio.
        (name == "aspect" && value.split('/').all(|p| p.parse::<u32>().is_ok()))
            .then_some((0.0, "/"))
    })?;
    let n = if neg { -n } else { n };
    let px = match unit {
        "px" => Some(n),
        "rem" => Some(n * s.rem),
        _ => None,
    };
    let step = |steps: &[(String, f32)], v: f32, what: &str| {
        nearest(steps, v).map(|(step, _, exact)| (format!("{name}-{step}"), exact, by_hand(what)))
    };
    match name {
        "aspect" if unit == "/" => Some((format!("aspect-{value}"), true, by_hand("a ratio"))),
        "text" => step(&s.font, px?, "a type size"),
        "tracking" if unit == "em" && n == 0.0 => {
            Some(("tracking-normal".to_owned(), true, by_hand("a tracking")))
        }
        "tracking" if unit == "em" => {
            // A size step's own letter spacing is no label's tracking.
            let named: Vec<_> = s
                .tracking
                .iter()
                .filter(|(k, v)| (v - n).abs() < 1e-4 || !s.font.iter().any(|(f, _)| f == k))
                .cloned()
                .collect();
            step(&named, n, "a tracking")
        }
        "leading" if unit.is_empty() => step(&s.leading, n, "a leading"),
        "blur" | "backdrop-blur" => step(&s.blur, px?, "a blur"),
        _ if name == "rounded" || name.starts_with("rounded-") => {
            // A round end: on a box that is not square, 50% is an
            // ellipse and `full` a capsule.
            if unit == "%" && n >= 50.0 || px.is_some_and(|p| p >= 999.0) {
                return Some((format!("{name}-full"), unit != "%", by_hand("a radius")));
            }
            step(&s.radius, px?, "a radius")
        }
        _ if WIDTHS.contains(&name) => {
            let w = px?.round().max(0.0);
            let class = match (name.starts_with("border") || name == "ring", w as u32) {
                (true, 1) => name.to_owned(),
                (_, w) => format!("{name}-{w}"),
            };
            Some((class, (w - px?).abs() < 0.01, by_hand("a width")))
        }
        "opacity" if unit.is_empty() || unit == "%" => {
            let p = if unit == "%" { n } else { n * 100.0 };
            Some((
                format!("opacity-{}", p.round()),
                true,
                by_hand("an opacity"),
            ))
        }
        "scale" | "scale-x" | "scale-y" if unit.is_empty() => Some((
            format!("{sign}{name}-{}", (n.abs() * 100.0).round()),
            true,
            by_hand("a scale"),
        )),
        "rotate" | "skew-x" | "skew-y" | "hue-rotate" if unit == "deg" => {
            let flip = if n < 0.0 { "-" } else { "" };
            Some((
                format!("{flip}{name}-{}", plain(n.abs())),
                true,
                by_hand("an angle"),
            ))
        }
        "duration" | "delay" if unit == "ms" || unit == "s" => {
            let ms = if unit == "s" { n * 1000.0 } else { n };
            Some((format!("{name}-{}", ms.round()), true, by_hand("a time")))
        }
        "z" if unit.is_empty() => Some((
            format!("{sign}z-{}", plain(n.abs())),
            true,
            by_hand("a layer"),
        )),
        "from" | "via" | "to" if unit == "%" => Some((
            format!("{name}-{}%", plain(n)),
            true,
            by_hand("a gradient stop"),
        )),
        _ => {
            let (_, fraction) = LENGTHS
                .iter()
                .find(|(l, _)| *l == name || (name == "leading" && *l == "h"))?;
            let flip = if n < 0.0 { "-" } else { "" };
            if unit == "%" {
                if !*fraction {
                    return None;
                }
                let p = n.abs().round() as u32;
                let exact = (n.abs() - p as f32).abs() < 0.01;
                let class = match p {
                    0 => format!("{name}-0"),
                    100 => format!("{flip}{name}-full"),
                    p => {
                        let g = gcd(p, 100);
                        format!("{flip}{name}-{}/{}", p / g, 100 / g)
                    }
                };
                return Some((class, exact, by_hand("a share")));
            }
            let px = px?;
            let steps = (px.abs() / s.spacing * 4.0).round() / 4.0;
            let class = if px.abs() == 1.0 {
                format!("{flip}{name}-px")
            } else {
                format!("{flip}{name}-{}", plain(steps))
            };
            // A device pixel is the least the eye places.
            let exact = (steps * s.spacing - px.abs()).abs() <= 0.51;
            Some((class, exact, by_hand("a length")))
        }
    }
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// A grid template: its tracks are structure; a length in one is a value.
fn grid(name: &str, value: &str, s: &Scale) -> Option<Found> {
    let by_hand = "writes a track list by hand".to_owned();
    // As many equal tiles as fit: the tile's least size is a step.
    for (repeat, fill) in [
        ("repeat(auto-fill,minmax(", "fill"),
        ("repeat(auto-fit,minmax(", "fit"),
    ] {
        if let Some((n, unit)) = value
            .strip_prefix(repeat)
            .and_then(|v| v.strip_suffix(",1fr))"))
            .and_then(quantity)
            .filter(|(_, u)| matches!(*u, "px" | "rem") && name.starts_with("grid-"))
        {
            let px = if unit == "rem" { n * s.rem } else { n };
            let steps = (px / s.spacing * 4.0).round() / 4.0;
            let exact = (steps * s.spacing - px).abs() <= 0.51;
            return Some((format!("{name}-{fill}-{}", plain(steps)), exact, by_hand));
        }
    }
    if let Some(n) = value
        .strip_prefix("repeat(")
        .and_then(|v| v.strip_suffix(",minmax(0,1fr))"))
        .and_then(|n| n.parse::<u32>().ok())
        .filter(|_| name.starts_with("grid-"))
    {
        return Some((format!("{name}-{n}"), true, by_hand));
    }
    if value == "minmax(0,1fr)" && name.starts_with("grid-") {
        return Some((format!("{name}-1"), true, by_hand));
    }
    let length = value
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '%'))
        .any(|w| quantity(w).is_some_and(|(n, u)| n != 0.0 && matches!(u, "px" | "rem" | "em")));
    length.then(|| (
        String::new(),
        false,
        "writes a length into a track list: give the child the size as a step (`w-48`, `min-w-60`) and keep the tracks `auto`, `fr` and `minmax(0,1fr)`".to_owned(),
    ))
}

/// One layer of a written shadow.
struct Layer {
    inset: bool,
    x: f32,
    y: f32,
    blur: f32,
    spread: f32,
    colour: Option<[u8; 4]>,
    role: Option<String>,
}

fn layers(value: &str, inset_all: bool, s: &Scale) -> Vec<Layer> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0, 0);
    let mut cut = Vec::new();
    for (i, c) in value.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                cut.push(&value[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    cut.push(&value[start..]);
    for text in cut {
        let mut l = Layer {
            inset: inset_all,
            x: 0.0,
            y: 0.0,
            blur: 0.0,
            spread: 0.0,
            colour: None,
            role: None,
        };
        let mut lengths = Vec::new();
        // `rgb(0_0_0/0.5)` holds `_` too: split outside parentheses.
        let (mut depth, mut start) = (0, 0);
        let mut words = Vec::new();
        for (i, c) in text.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                '_' if depth == 0 => {
                    words.push(&text[start..i]);
                    start = i + 1;
                }
                _ => {}
            }
        }
        words.push(&text[start..]);
        for w in words.into_iter().filter(|w| !w.is_empty()) {
            if w == "inset" {
                l.inset = true;
            } else if let Some((n, u)) =
                quantity(w).filter(|(_, u)| matches!(*u, "" | "px" | "rem"))
            {
                lengths.push(if u == "rem" { n * s.rem } else { n });
            } else if let Some(role) = w
                .strip_prefix("var(--color-")
                .and_then(|r| r.strip_suffix(')'))
            {
                l.role = Some(role.to_owned());
            } else {
                l.colour = colour(w);
            }
        }
        lengths.resize(4, 0.0);
        (l.x, l.y, l.blur, l.spread) = (lengths[0], lengths[1], lengths[2], lengths[3]);
        out.push(l);
    }
    out
}

/// The named shadows, rings and inset shadows a written shadow is: one
/// `shadow-*`, one `inset-shadow-*`, one ring and one inset ring at most,
/// as an element holds. What is left over is the slot's pack art.
fn shadow(value: &str, inset_all: bool, s: &Scale) -> (String, bool) {
    let (mut outer, mut inner, mut ring, mut inset_ring) = (None, None, None, None);
    // An element holds one of each: the first written stands, but inside
    // a face a mark of state stands over a recess, and that over a lit
    // edge or a lip.
    let weight = |class: &str| match class {
        "inset-shadow-lit" | "inset-shadow-lip" => 0,
        "inset-shadow-sunk" => 1,
        _ => 2,
    };
    let put = |slot: &mut Option<String>, class: String| {
        if slot
            .as_deref()
            .is_none_or(|had| weight(had) < weight(&class) && class.starts_with("inset-shadow"))
        {
            *slot = Some(class);
        }
    };
    for l in layers(value, inset_all, s) {
        let dark = l
            .colour
            .is_some_and(|c| c[0] < 40 && c[1] < 40 && c[2] < 48);
        let white = l
            .colour
            .is_some_and(|c| c[0] == c[1] && c[1] == c[2] && c[0] > 215);
        // The role a coloured layer is lit in; a neutral one keeps the
        // named shadow's own colour.
        let role = |edge: bool| match (&l.role, l.colour) {
            (Some(r), _) => Some(r.clone()),
            (None, Some(c)) if edge => Some(s.role(c, edge).0),
            (None, Some(c)) => Some(s.shadow_role(c, edge)),
            (None, None) => None,
        };
        let flat = l.x == 0.0 && l.y == 0.0 && l.blur == 0.0;
        let width = l.spread.round().max(1.0) as u32;
        match (l.inset, flat) {
            (false, true) | (true, true) => {
                let name = if l.inset { "inset-ring" } else { "ring" };
                let width = if width == 1 && !l.inset {
                    name.to_owned()
                } else {
                    format!("{name}-{width}")
                };
                let class = match role(true) {
                    Some(r) => format!("{width} {name}-{r}"),
                    None => width,
                };
                put(if l.inset { &mut inset_ring } else { &mut ring }, class);
            }
            (true, false) => {
                let edge = l.blur == 0.0;
                let tint = |kind: &str| match role(false) {
                    Some(r) => format!("inset-shadow-{kind} inset-shadow-{r}"),
                    None => format!("inset-shadow-{kind}"),
                };
                let class = match (edge, dark, white) {
                    (true, false, true) if l.y > 0.0 => "inset-shadow-lit".to_owned(),
                    (true, true, _) if l.y < 0.0 => "inset-shadow-lip".to_owned(),
                    (true, _, _) if l.x != 0.0 => tint("bar"),
                    (true, _, _) if l.y < 0.0 => tint("underline"),
                    (true, _, _) => tint("overline"),
                    (false, true, _) => "inset-shadow-sunk".to_owned(),
                    (false, _, _) => tint("glow"),
                };
                put(&mut inner, class);
            }
            (false, false) => {
                let class = if dark || l.colour.is_none() && l.role.is_none() {
                    match l.blur {
                        0.0 => "shadow-key",
                        b if b <= 4.0 => "shadow-raise",
                        b if b <= 16.0 => "shadow-plate",
                        b if b <= 28.0 => "shadow-float",
                        _ => "shadow-overlay",
                    }
                    .to_owned()
                } else {
                    let size = match l.blur {
                        b if b <= 9.5 => "glow-sm",
                        b if b <= 13.0 => "glow",
                        _ => "glow-lg",
                    };
                    match role(false) {
                        Some(r) if r != "hud-accent" => format!("shadow-{size} shadow-{r}"),
                        _ => format!("shadow-{size}"),
                    }
                };
                put(&mut outer, class);
            }
        }
    }
    // `shadow-raise`, `-plate` and `-key` carry their own lit edge or lip.
    if outer.as_deref().is_some_and(|o| {
        ["shadow-raise", "shadow-plate", "shadow-key", "shadow-float"].contains(&o)
    }) && inner
        .as_deref()
        .is_some_and(|i| i == "inset-shadow-lit" || i == "inset-shadow-lip")
    {
        inner = None;
    }
    let classes = [outer, inner, ring, inset_ring]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
    (classes, false)
}

/// `prop-white`, `prop-black` or `prop-<role>/<n>`.
fn modifier(utility: &str, s: &Scale) -> Option<Found> {
    let (prop, edge, rest) = COLOURS.iter().find_map(|(p, edge)| {
        utility
            .strip_prefix(p)
            .and_then(|r| r.strip_prefix('-'))
            .map(|r| (*p, *edge, r))
    })?;
    let (name, n) = match rest.rsplit_once('/') {
        Some((c, m)) => (c, Some(m.parse::<u32>().ok()?)),
        None => (rest, None),
    };
    let ground = GROUNDS.contains(&prop);
    if name == "white" || name == "black" {
        // ui_grounds reports a ground's or an ink's.
        if ground {
            return None;
        }
        let role = match (name, n) {
            ("white", Some(n)) => white(edge, n),
            ("white", None) if edge && prop != "stroke" => "rim-strong",
            ("white", None) => "fg",
            (_, Some(n)) => black(n),
            (_, None) if prop.ends_with("shadow") => "shade-deep",
            (_, None) => "bg",
        };
        return Some((
            format!("{prop}-{role}"),
            false,
            format!("is a literal {name}, the same in every pack and scheme"),
        ));
    }
    let n = n?;
    if !s.has_colour(name) || prop == "text" {
        return None;
    }
    let role = match (name, n) {
        ("fg", 0..=55) => "fg-subtle",
        ("fg", 56..=89) => "fg-muted",
        _ => name,
    };
    let why = match n {
        // A wash: the state it tints is the slot's, or the whole colour.
        0..=40 if role == name => format!(
            "washes `{name}` by hand at {n}% (a chosen or tinted state is the slot's variant or `{prop}-surface-hover`; a fill is the whole role)"
        ),
        _ => format!("thins `{name}` by hand to {n}%"),
    };
    Some((format!("{prop}-{role}"), false, why))
}
