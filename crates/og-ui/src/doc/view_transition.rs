//! CSS View Transitions for layers (docs/ui.md §2.1, CSS View Transitions
//! Level 2). A screen document opts in with `@view-transition { navigation:
//! auto; }`: its layer joining the stack and leaving it are its
//! navigations, and the layer is the `root` snapshot. By default the layer
//! cross-fades (the UA's `-ua-view-transition-fade-in` and `-fade-out`,
//! 0.25s). `::view-transition-old(root)` animates the outgoing view (the
//! leaving layer), `::view-transition-new(root)` the incoming one (the
//! joining layer), with `animation` and the document's `@keyframes`;
//! `::view-transition-group(root)` and `::view-transition-image-pair(root)`
//! give both their inherited timing (`animation-duration`, its other
//! longhands but the name). `types` are labels that
//! `:active-view-transition-type()` matches, as in the spec.
//!
//! The engine's full-frame covers are the only `-og-` part: the built-in
//! keyframes `-og-iris`, `-og-dissolve`, `-og-wipe` and `-og-slide` (with
//! `-from-right`, `-from-top` or `-from-bottom`). On `old` a cover closes
//! over the outgoing view, and the incoming view shows once it has closed;
//! on `new` it opens on the incoming view. `::view-transition
//! { background-color }` is the cover's colour. Shared-element morphs
//! (`view-transition-name`) are not read.

use super::syntax::Error;
use super::*;
use crate::motion::css::CssAnimation;
use crate::{ScreenWipe, Side};

/// A screen's view transition (`LayerSpec::transition`).
#[derive(Clone, Debug, PartialEq)]
pub struct ViewTransition {
    /// The outgoing view's animations (`::view-transition-old(root)`).
    pub old: Vec<CssAnimation>,
    /// The incoming view's (`::view-transition-new(root)`).
    pub new: Vec<CssAnimation>,
    /// The covers' colour (`None`: the style's backdrop).
    pub color: Option<Srgba>,
}

/// The UA's cross-fade keyframes.
pub const FADE_IN: &str = "-ua-view-transition-fade-in";
pub const FADE_OUT: &str = "-ua-view-transition-fade-out";

/// The cover a built-in keyframes name plays, and where it comes from.
pub fn cover(name: &str) -> Option<(ScreenWipe, Side)> {
    let name = name.strip_prefix("-og-")?;
    let (kind, side) = match name.split_once("-from-") {
        Some((k, s)) => (k, s),
        None => (name, "left"),
    };
    let kind = match kind {
        "iris" => ScreenWipe::Iris,
        "dissolve" => ScreenWipe::Dissolve,
        "wipe" => ScreenWipe::Wipe,
        "slide" => ScreenWipe::Slide,
        _ => return None,
    };
    let side = match side {
        "left" => Side::Left,
        "right" => Side::Right,
        "top" => Side::Top,
        "bottom" => Side::Bottom,
        _ => return None,
    };
    Some((kind, side))
}

/// The built-in keyframes names (the UA's and the covers).
fn builtin(name: &str) -> bool {
    name == FADE_IN || name == FADE_OUT || cover(name).is_some()
}

/// The keyframes of the UA's cross-fade (`None` for any other name).
pub fn builtin_keyframes(name: &str) -> Option<crate::motion::css::Keyframes> {
    use crate::motion::css::{Keyframe, KeyframeTrack, Keyframes};
    let offset = match name {
        FADE_IN => 0.0,
        FADE_OUT => 1.0,
        _ => return None,
    };
    Some(Keyframes {
        tracks: vec![KeyframeTrack {
            prop: crate::element::Prop::Opacity,
            keys: vec![Keyframe {
                offset,
                value: 0.0,
                ease: None,
                composite: None,
            }],
        }],
        styles: Vec::new(),
        events: Vec::new(),
    })
}

/// Whether a rule's selector or prelude is a view-transition one.
pub(super) fn selects(selector: &str) -> bool {
    selector.starts_with("@view-transition") || selector.contains("::view-transition")
}

/// A rule's animation longhands, `(name, value)` in source order.
type Longhands = Vec<(String, String)>;

/// Which part a pseudo-element rule styles.
#[derive(Clone, Copy, PartialEq)]
enum Part {
    Overlay,
    Group,
    Old,
    New,
}

/// Reads a screen's view-transition rules, `(selector, body, at,
/// conditional)` in source order, into its layer spec.
pub(super) fn apply(
    doc: &mut UiDocument,
    rules: &[(&str, &str, usize, bool)],
) -> Result<(), Error> {
    let mut opted = false;
    let mut types: Vec<String> = Vec::new();
    for &(selector, body, at, conditional) in rules {
        if conditional {
            return Err((
                at,
                format!(
                    "{selector}: write view-transition rules at the top level, not inside @media or @container"
                ),
            ));
        }
        if selector != "@view-transition" {
            continue;
        }
        for d in variables::declarations(body).map_err(|e| (at, e))? {
            let value = d.source.trim();
            match d.name.as_str() {
                "navigation" => match value {
                    "auto" => opted = true,
                    "none" => opted = false,
                    _ => return Err((at, "navigation is auto or none".into())),
                },
                "types" => types = value.split_whitespace().map(str::to_string).collect(),
                other => {
                    return Err((
                        at,
                        format!("@view-transition takes navigation and types, not {other}"),
                    ));
                }
            }
        }
    }
    // (part, typed, longhands) in cascade order: a type-qualified rule
    // outranks a plain one (its pseudo-class), then source order.
    let mut parts: Vec<(Part, bool, Longhands)> = Vec::new();
    let known: Vec<String> = doc.keyframe_rules.iter().map(|(n, ..)| n.clone()).collect();
    let keyframes = |name: &str| builtin(name) || known.iter().any(|k| k == name);
    let rules: Vec<(&str, &str, usize, bool)> = rules
        .iter()
        .flat_map(|&(list, body, at, c)| list.split(',').map(move |s| (s.trim(), body, at, c)))
        .collect();
    for &(selector, body, at, _) in &rules {
        if selector == "@view-transition" {
            continue;
        }
        let Some((part, typed, applies)) = pseudo(selector, &types).map_err(|e| (at, e))? else {
            return Err((
                at,
                format!("{selector}: write @view-transition or a ::view-transition pseudo-element"),
            ));
        };
        if !applies {
            continue;
        }
        let mut longhands = Vec::new();
        for d in variables::declarations(body).map_err(|e| (at, e))? {
            let allowed = match part {
                Part::Overlay => matches!(d.name.as_str(), "background" | "background-color"),
                Part::Group => d.name.starts_with("animation") && d.name != "animation-name",
                Part::Old | Part::New => d.name.starts_with("animation"),
            };
            if !allowed {
                return Err((
                    at,
                    format!(
                        "{selector} takes {}, not {}",
                        match part {
                            Part::Overlay => "background-color (the cover's colour)",
                            Part::Group => "the animation timing its old and new images inherit",
                            Part::Old | Part::New => "animation",
                        },
                        d.name
                    ),
                ));
            }
            if part == Part::Overlay {
                longhands.push(("background-color".into(), d.source.trim().to_string()));
                continue;
            }
            for d in super::motion::expand(d, &keyframes).map_err(|e| (at, e))? {
                longhands.push((d.name, d.source));
            }
        }
        parts.push((part, typed, longhands));
    }
    if !opted {
        doc.layer.transition = None;
        return Ok(());
    }
    if doc.kind != DocKind::Screen {
        return Err((
            rules[0].2,
            "a view transition is a screen's: its layer is the root that joins and leaves the stack"
                .into(),
        ));
    }
    parts.sort_by_key(|(_, typed, _)| *typed);
    let of = |want: Part| -> Vec<(String, String)> {
        parts
            .iter()
            .filter(|(p, ..)| *p == want)
            .flat_map(|(_, _, l)| l.clone())
            .collect()
    };
    let group = of(Part::Group);
    let mut diagnostics = Vec::new();
    let mut images = |own: Part, ua: &str| -> Vec<CssAnimation> {
        let mut winners = vec![
            ("animation-name".to_string(), ua.to_string()),
            ("animation-duration".to_string(), "0.25s".to_string()),
            ("animation-fill-mode".to_string(), "both".to_string()),
        ];
        winners.extend(group.clone());
        winners.extend(of(own));
        let (_, mut animations) = super::motion::resolve(&winners, &doc.name, &mut diagnostics);
        for a in &mut animations {
            if let Some(bare) = a.name.split_once('/').map(|(_, n)| n.to_string())
                && builtin(&bare)
            {
                a.name = bare;
            }
        }
        animations
    };
    let old = images(Part::Old, FADE_OUT);
    let new = images(Part::New, FADE_IN);
    if let Some(error) = diagnostics.first() {
        return Err((rules[0].2, error.clone()));
    }
    let color = match of(Part::Overlay).last() {
        Some((_, value)) => Some(
            crate::style_file::parse_color(value)
                .ok_or((rules[0].2, format!("'{value}' is not a #rrggbb[aa] colour")))?,
        ),
        None => None,
    };
    doc.layer.transition = Some(ViewTransition { old, new, color });
    Ok(())
}

/// A pseudo-element selector: its part, whether it names a transition
/// type, and whether it applies (its types are among the declared ones).
/// `None` when it is no view-transition selector.
fn pseudo(selector: &str, types: &[String]) -> Result<Option<(Part, bool, bool)>, String> {
    let at = selector.find("::view-transition").ok_or("")?;
    let (prefix, element) = selector.split_at(at);
    let prefix = prefix
        .trim()
        .trim_start_matches("html")
        .trim_start_matches(":root");
    let (typed, applies) = match prefix {
        "" => (false, true),
        p => {
            let list = p
                .strip_prefix(":active-view-transition-type(")
                .and_then(|r| r.strip_suffix(')'))
                .ok_or_else(|| {
                    format!("{selector}: a view-transition rule is qualified only by :active-view-transition-type(…)")
                })?;
            let names: Vec<&str> = list.split(',').map(str::trim).collect();
            (true, names.iter().any(|n| types.iter().any(|t| t == n)))
        }
    };
    let element = element.trim();
    let part = match element {
        "::view-transition" => Part::Overlay,
        _ => {
            let (name, arg) = element
                .strip_suffix(')')
                .and_then(|e| e.split_once('('))
                .ok_or_else(|| format!("{element}: name its snapshot, as (root)"))?;
            if !matches!(arg.trim(), "root" | "*") {
                return Err(format!(
                    "{element}: only the root transitions; view-transition-name morphs are not read"
                ));
            }
            match name {
                "::view-transition-group" | "::view-transition-image-pair" => Part::Group,
                "::view-transition-old" => Part::Old,
                "::view-transition-new" => Part::New,
                _ => return Ok(None),
            }
        }
    };
    Ok(Some((part, typed, applies)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(css: &str) -> Result<Option<ViewTransition>, String> {
        UiDocument::parse(
            "t.svelte",
            &format!(
                "<meta name=\"screen\" content=\"s\"><column/><style>@keyframes rise {{ from {{ translate: 0 40px; opacity: 0; }} }}\n{css}</style>"
            ),
        )
        .map(|d| d.layer.transition)
        .map_err(|e| e.join("\n"))
    }

    /// The UA's cross-fade by default; old and new take animation, the
    /// group's timing is inherited, types gate typed rules, and the
    /// built-in names stay unqualified.
    #[test]
    fn view_transitions_read_the_spec_pseudo_elements() {
        let t = spec("@view-transition { navigation: auto; }")
            .unwrap()
            .unwrap();
        assert_eq!(
            (t.old[0].name.as_str(), t.new[0].name.as_str()),
            (FADE_OUT, FADE_IN)
        );
        assert_eq!((t.old[0].duration_ms, t.new[0].duration_ms), (250.0, 250.0));
        let t = spec(
            "@view-transition { navigation: auto; types: slide; }
             ::view-transition-group(root) { animation-duration: 600ms; }
             ::view-transition-new(root) { animation-name: rise; }
             :root:active-view-transition-type(slide)::view-transition-old(root) { animation: 300ms -og-iris; }
             :active-view-transition-type(other)::view-transition-old(root) { animation: 1s -og-wipe; }
             ::view-transition-old(root) { animation: 900ms -og-dissolve; }
             ::view-transition { background-color: #102030; }",
        )
        .unwrap()
        .unwrap();
        assert_eq!(t.new[0].name, "s/rise");
        assert_eq!(
            t.new[0].duration_ms, 600.0,
            "the group's duration is inherited"
        );
        // The typed rule outranks the later plain one; the other type's does not apply.
        assert_eq!(
            (t.old[0].name.as_str(), t.old[0].duration_ms),
            ("-og-iris", 300.0)
        );
        assert_eq!(t.color.unwrap().rgb, [0x10, 0x20, 0x30]);
        assert_eq!(
            cover("-og-wipe-from-right"),
            Some((ScreenWipe::Wipe, Side::Right))
        );
        // Pseudo-element rules without the opt-in transition nothing.
        assert!(
            spec("::view-transition-new(root) { animation: 1s rise; }")
                .unwrap()
                .is_none()
        );
        for (css, wanted) in [
            (
                "@view-transition { navigation: auto; } ::view-transition-old(card) { animation: 1s rise; }",
                "only the root",
            ),
            (
                "@view-transition { navigation: auto; } ::view-transition-new(root) { opacity: 0; }",
                "takes animation",
            ),
            (
                "@view-transition { navigation: auto; } ::view-transition-new(root) { animation: 1s nowhere; }",
                "nowhere",
            ),
            (
                "@media (width > 1px) { @view-transition { navigation: auto; } }",
                "top level",
            ),
        ] {
            let error = spec(css).unwrap_err();
            assert!(error.contains(wanted), "{css}: {error}");
        }
        let error = UiDocument::parse(
            "c.svelte",
            "<meta name=\"component\" content=\"c\"><column/><style>@view-transition { navigation: auto; }</style>",
        )
        .unwrap_err()
        .join("\n");
        assert!(error.contains("a view transition is a screen's"), "{error}");
    }
}
