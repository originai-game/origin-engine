//! Forced colours mode (CSS Color Adjust 1 §3): with the viewport's
//! `forced_colors` on, an element whose `forced-color-adjust` is `auto`
//! has its colours forced to the system colours, which are the palette's
//! roles (CSS Color 4 §6.2, `og_css::color`). The cascade applies
//! [`force`] to each winning author declaration.

use og_css::color::Color;
use og_css::{Tok, tok_text, tokens};

/// What forcing does to one winning declaration.
#[derive(Debug, PartialEq)]
pub(crate) enum Forced {
    /// It stands as written: no colour of its own, or only system colours.
    Kept,
    /// Rolled back to what it is without author rules (CSS Cascade 5
    /// §7.3 `revert`): the inherited or initial value, which the user
    /// agent's own styles set.
    Reverted,
    /// These declarations stand in its place.
    Is(Vec<(String, String)>),
}

/// The colour properties forcing replaces whole: each value is one colour
/// (or a keyword that is none).
const COLORS: &[&str] = &[
    "color",
    "border-top-color",
    "border-right-color",
    "border-bottom-color",
    "border-left-color",
    "border-block-start-color",
    "border-block-end-color",
    "border-inline-start-color",
    "border-inline-end-color",
    "outline-color",
    "column-rule-color",
    "row-rule-color",
    "text-decoration-color",
    "text-emphasis-color",
    "caret-color",
    "fill",
    "stroke",
    "flood-color",
    "lighting-color",
    "stop-color",
    "-webkit-text-stroke-color",
    "-webkit-text-fill-color",
    "-webkit-tap-highlight-color",
];

/// The shorthands with a colour among their parts: a part that is a
/// colour other than a system one becomes `currentcolor`, each one's
/// initial colour.
const WITH_COLOR: &[&str] = &[
    "border",
    "border-top",
    "border-right",
    "border-bottom",
    "border-left",
    "border-block",
    "border-block-start",
    "border-block-end",
    "border-inline",
    "border-inline-start",
    "border-inline-end",
    "border-color",
    "border-block-color",
    "border-inline-color",
    "outline",
    "column-rule",
    "row-rule",
    "rule",
    "rule-color",
    "text-decoration",
    "text-emphasis",
    "-webkit-text-stroke",
];

/// Whether forcing reads `name`: its value depends on the mode.
pub(crate) fn affected(name: &str) -> bool {
    COLORS.contains(&name)
        || WITH_COLOR.contains(&name)
        || matches!(
            name,
            "background"
                | "background-color"
                | "background-image"
                | "box-shadow"
                | "text-shadow"
                | "accent-color"
                | "scrollbar-color"
                | "color-scheme"
        )
}

/// Whether `c`'s computed value is a system colour: one of the palette's
/// roles, `currentcolor` (the forced `color`), or `light-dark()` of two.
fn system(c: &Color) -> bool {
    match c {
        Color::Token(_) | Color::Current => true,
        Color::LightDark(a, b) => system(a) && system(b),
        _ => false,
    }
}

/// The colour `t` is when it is one other than a system colour.
fn forced_color(t: &Tok) -> Option<Color> {
    og_css::color::parse(t).ok().filter(|c| !system(c))
}

/// `value` of `name` in forced colours mode (§3.1). `ink` is the
/// element's own `color` when it declares one, as written: a background
/// is the system colour paired with it (CSS Color 4 §6.2: `ButtonFace`
/// behind `ButtonText`), `Canvas` behind any other, with its own alpha so
/// a transparent background stays transparent.
pub(crate) fn force(name: &str, value: &str, ink: Option<&str>) -> Forced {
    let value = value.trim();
    match name {
        "box-shadow" | "text-shadow" if value != "none" => {
            Forced::Is(vec![(name.into(), "none".into())])
        }
        "accent-color" | "scrollbar-color" if value != "auto" => {
            Forced::Is(vec![(name.into(), "auto".into())])
        }
        "color-scheme" if value != "light dark" => {
            Forced::Is(vec![(name.into(), "light dark".into())])
        }
        "background-image" if value != "none" && !value.contains("url(") => {
            Forced::Is(vec![(name.into(), "none".into())])
        }
        "background-color" => match tokens(value).as_deref() {
            Ok([t]) if let Some(c) = forced_color(t) => {
                Forced::Is(vec![(name.into(), ground(&c, ink))])
            }
            _ => Forced::Kept,
        },
        "background" => background(value, ink),
        n if COLORS.contains(&n) => match tokens(value).as_deref() {
            Ok([t]) if forced_color(t).is_some() => Forced::Reverted,
            _ => Forced::Kept,
        },
        n if WITH_COLOR.contains(&n) => match tokens(value) {
            Ok(parts) if parts.iter().any(|t| forced_color(t).is_some()) => {
                let parts: Vec<String> = parts
                    .iter()
                    .map(|t| match forced_color(t) {
                        Some(_) => "currentcolor".into(),
                        None => tok_text(t),
                    })
                    .collect();
                Forced::Is(vec![(name.into(), parts.join(" "))])
            }
            _ => Forced::Kept,
        },
        _ => Forced::Kept,
    }
}

/// The `background` shorthand: its images are `none` unless one is a
/// `url()`, its colour the ground ([`ground`]).
fn background(value: &str, ink: Option<&str>) -> Forced {
    let Ok(parts) = tokens(value) else {
        return Forced::Kept;
    };
    let colour = match crate::css::paint_css::background(value) {
        // Layers: their colour is the last one's, transparent without.
        Some(Ok((_, colour))) => colour,
        Some(Err(_)) => return Forced::Kept,
        None if parts.len() == 1 => parts.first().cloned(),
        None => return Forced::Kept,
    };
    let url = value.contains("url(");
    let images = !url && parts.iter().any(crate::css::paint_css::is_image);
    let forced = colour.as_ref().and_then(forced_color);
    if !images && forced.is_none() {
        return Forced::Kept;
    }
    let ground = match (&forced, &colour) {
        (Some(c), _) => ground(c, ink),
        (None, Some(t)) => tok_text(t),
        (None, None) => "transparent".into(),
    };
    if url {
        // The images stay; the colour under them is forced.
        Forced::Is(vec![
            ("background".into(), value.into()),
            ("background-color".into(), ground),
        ])
    } else {
        Forced::Is(vec![("background".into(), ground)])
    }
}

/// The system colour a background is forced to: the one paired with the
/// element's own `color` (`ink`), with `c`'s alpha.
fn ground(c: &Color, ink: Option<&str>) -> String {
    const PAIRS: [(&str, &str); 7] = [
        ("canvastext", "Canvas"),
        ("buttontext", "ButtonFace"),
        ("fieldtext", "Field"),
        ("marktext", "Mark"),
        ("highlighttext", "Highlight"),
        ("selecteditemtext", "SelectedItem"),
        ("accentcolortext", "AccentColor"),
    ];
    let ink = ink.map(|i| i.trim().to_ascii_lowercase());
    let system = PAIRS
        .iter()
        .find(|(text, _)| ink.as_deref() == Some(*text))
        .map_or("Canvas", |(_, ground)| ground);
    let alpha = c.literal().map_or(1.0, |c| c.alpha);
    if alpha >= 1.0 {
        system.into()
    } else if alpha <= 0.0 {
        "transparent".into()
    } else {
        format!("alpha(from {system} / {alpha})")
    }
}

/// An element's `forced-color-adjust` takes forcing (`auto`); `none` and
/// `preserve-parent-color` opt it out (§3.2: the latter's inherited
/// `color` is its parent's used one, which is what an element that
/// declares none draws in).
pub(crate) fn adjusts(value: &str) -> bool {
    !matches!(value.trim(), "none" | "preserve-parent-color")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is(pairs: &[(&str, &str)]) -> Forced {
        Forced::Is(
            pairs
                .iter()
                .map(|(n, v)| (n.to_string(), v.to_string()))
                .collect(),
        )
    }

    /// §3.1 per property: colours other than system ones revert, a
    /// shorthand's colour becomes its initial `currentcolor`, a background
    /// takes the system colour paired with the element's own and keeps its
    /// alpha, shadows are none, images without `url()` none, and
    /// `accent-color` and `scrollbar-color` auto.
    #[test]
    fn colours_are_forced_to_system_colours() {
        for (name, value, ink, want) in [
            ("color", "#ff0000", None, Forced::Reverted),
            ("color", "CanvasText", None, Forced::Kept),
            (
                "color",
                "light-dark(Canvas, CanvasText)",
                None,
                Forced::Kept,
            ),
            ("color", "currentcolor", None, Forced::Kept),
            ("border-top-color", "rgb(0 0 255)", None, Forced::Reverted),
            ("outline-color", "transparent", None, Forced::Reverted),
            ("caret-color", "auto", None, Forced::Kept),
            ("fill", "none", None, Forced::Kept),
            (
                "border",
                "2px solid #00ff00",
                None,
                is(&[("border", "2px solid currentcolor")]),
            ),
            ("border", "2px solid ButtonBorder", None, Forced::Kept),
            (
                "background-color",
                "#ff0000",
                None,
                is(&[("background-color", "Canvas")]),
            ),
            (
                "background-color",
                "#ff000080",
                Some("ButtonText"),
                is(&[("background-color", "alpha(from ButtonFace / 0.5019608)")]),
            ),
            (
                "background-color",
                "transparent",
                None,
                is(&[("background-color", "transparent")]),
            ),
            ("background-color", "Highlight", None, Forced::Kept),
            (
                "background",
                "rgb(255 0 0 / 25%)",
                None,
                is(&[("background", "alpha(from Canvas / 0.25)")]),
            ),
            (
                "background",
                "linear-gradient(red, blue)",
                None,
                is(&[("background", "transparent")]),
            ),
            (
                "background",
                "linear-gradient(red, blue) #00ff00",
                Some("HighlightText"),
                is(&[("background", "Highlight")]),
            ),
            (
                "background",
                "url(a.png) #00ff00",
                None,
                is(&[
                    ("background", "url(a.png) #00ff00"),
                    ("background-color", "Canvas"),
                ]),
            ),
            (
                "background-image",
                "linear-gradient(red, blue)",
                None,
                is(&[("background-image", "none")]),
            ),
            ("background-image", "url(a.png)", None, Forced::Kept),
            (
                "box-shadow",
                "0 0 4px red",
                None,
                is(&[("box-shadow", "none")]),
            ),
            ("text-shadow", "none", None, Forced::Kept),
            (
                "accent-color",
                "#ff0000",
                None,
                is(&[("accent-color", "auto")]),
            ),
            (
                "scrollbar-color",
                "red blue",
                None,
                is(&[("scrollbar-color", "auto")]),
            ),
            ("width", "10px", None, Forced::Kept),
        ] {
            assert_eq!(force(name, value, ink), want, "{name}: {value}");
        }
    }
}
