//! CSS View Transitions for layers (docs/ui.md §2.1, CSS View Transitions
//! Level 2): a screen document that opts in with `@view-transition` covers
//! the frame as its layer joins the stack and again as it leaves, the cover
//! closing over the old view and opening on the new one (the engine's
//! screen transition, [`crate::Ui::cover`]).
//!
//! ```css
//! @view-transition { navigation: auto; types: wipe from-right; }
//! ::view-transition-group(root) { animation-duration: 600ms; }
//! ::view-transition { background-color: #101820; }
//! ```
//!
//! `types` names how it covers: `fade` (the default), `wipe`, `iris`,
//! `dissolve` or `slide`, and `from-left`, `from-right`, `from-top` or
//! `from-bottom` for where a wipe or slide comes from. The root group's
//! `animation-duration` is the whole transition (cover and reveal); the
//! `::view-transition` overlay's background is the cover's colour. Only the
//! root transitions: `view-transition-name` (shared-element morphs) is not
//! read.

use super::syntax::Error;
use super::*;
use crate::{ScreenTransitionDesc, ScreenWipe, Side};

/// Whether a rule's selector or prelude is a view-transition one.
pub(super) fn selects(selector: &str) -> bool {
    selector.starts_with("@view-transition") || selector.starts_with("::view-transition")
}

/// Reads one view-transition rule into `doc`'s layer spec; true for an
/// `@view-transition` that turns it on.
pub(super) fn apply(
    doc: &mut UiDocument,
    selector: &str,
    body: &str,
    at: usize,
    conditional: bool,
) -> Result<bool, Error> {
    if conditional {
        return Err((
            at,
            format!(
                "{selector}: write view-transition rules at the top level, not inside @media or @container"
            ),
        ));
    }
    let declarations = variables::declarations(body).map_err(|e| (at, e))?;
    let desc = doc
        .layer
        .transition
        .get_or_insert_with(ScreenTransitionDesc::default);
    let mut opted = false;
    match selector {
        "@view-transition" => {
            for d in &declarations {
                let value = d.source.trim();
                match d.name.as_str() {
                    "navigation" => match value {
                        "auto" => opted = true,
                        "none" => {}
                        _ => return Err((at, "navigation is auto or none".into())),
                    },
                    "types" => {
                        for word in value.split_whitespace() {
                            match word {
                                "fade" => desc.kind = ScreenWipe::Fade,
                                "wipe" => desc.kind = ScreenWipe::Wipe,
                                "iris" => desc.kind = ScreenWipe::Iris,
                                "dissolve" => desc.kind = ScreenWipe::Dissolve,
                                "slide" => desc.kind = ScreenWipe::Slide,
                                "from-left" => desc.from = Side::Left,
                                "from-right" => desc.from = Side::Right,
                                "from-top" => desc.from = Side::Top,
                                "from-bottom" => desc.from = Side::Bottom,
                                other => {
                                    return Err((
                                        at,
                                        format!(
                                            "view-transition type {other}: use fade, wipe, iris, dissolve or slide, and from-left/right/top/bottom"
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                    other => {
                        return Err((
                            at,
                            format!("@view-transition takes navigation and types, not {other}"),
                        ));
                    }
                }
            }
            return Ok(opted);
        }
        "::view-transition-group(root)" => {
            for d in &declarations {
                if d.name != "animation-duration" {
                    return Err((
                        at,
                        format!(
                            "::view-transition-group(root) takes animation-duration (the whole transition), not {}",
                            d.name
                        ),
                    ));
                }
                let toks = css::tokens(d.source.trim()).map_err(|e| (at, e))?;
                let [t] = toks.as_slice() else {
                    return Err((at, "animation-duration is one duration".into()));
                };
                let ms = css::want_ms(t).map_err(|e| (at, e))?;
                desc.duration_ms = ms.as_number().map_or(0.0, |n| n.as_f64()) as f32;
            }
        }
        "::view-transition" => {
            for d in &declarations {
                if !matches!(d.name.as_str(), "background" | "background-color") {
                    return Err((
                        at,
                        format!(
                            "::view-transition takes background-color (the cover's colour), not {}",
                            d.name
                        ),
                    ));
                }
                desc.color = Some(crate::style_file::parse_color(d.source.trim()).ok_or((
                    at,
                    format!("'{}' is not a #rrggbb[aa] colour", d.source.trim()),
                ))?);
            }
        }
        other => {
            return Err((
                at,
                format!(
                    "{other}: a layer's view transition is its root's; write @view-transition, ::view-transition-group(root) or ::view-transition"
                ),
            ));
        }
    }
    Ok(false)
}
