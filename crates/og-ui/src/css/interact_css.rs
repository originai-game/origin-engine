//! CSS interaction properties on document elements (CSS UI 4 §5,
//! Pointer Events 3 §9, CSS Display 3 §7): `cursor` with `url()` images,
//! `pointer-events`, `touch-action` and `visibility`.
//!
//! Each is one reflected prop holding its CSS value as written, checked
//! here when the stylesheet is lowered and read here again when the
//! element is built. The inherited ones (`cursor`, `pointer-events`,
//! `visibility`) are resolved down the tree by the frame's orders, as
//! `user-select` is, so a declared value costs nothing while idle.

use super::css::{Tok, describe, tokens};
use crate::element::{Cursor, CursorImage, Interact, PointerEvents, SvgPointer, Visibility};

/// The props this module reads, by their CSS names.
pub(crate) fn prop_of(css: &str) -> Option<&'static str> {
    Some(match css {
        "cursor" => "cursor",
        "pointer-events" => "pointerEvents",
        "touch-action" => "touchAction",
        "visibility" => "visibility",
        _ => return None,
    })
}

/// How each prop's value is written (the catalogue's column).
pub(crate) fn form(prop: &str) -> Option<&'static str> {
    Some(match prop {
        "cursor" => {
            "[url(<file>) [<x> <y>],]* auto | default | none | context-menu | help | pointer | progress | wait | cell | crosshair | text | vertical-text | alias | copy | move | no-drop | not-allowed | grab | grabbing | all-scroll | col-resize | row-resize | n-resize | e-resize | s-resize | w-resize | ne-resize | nw-resize | se-resize | sw-resize | ew-resize | ns-resize | nesw-resize | nwse-resize | zoom-in | zoom-out"
        }
        "pointerEvents" => {
            "auto | none | visiblePainted | visibleFill | visibleStroke | visible | painted | fill | stroke | bounding-box | all"
        }
        "touchAction" => {
            "auto | none | [pan-x | pan-left | pan-right] || [pan-y | pan-up | pan-down] || pinch-zoom | manipulation"
        }
        "visibility" => "visible | hidden | collapse",
        _ => return None,
    })
}

/// Checks a declaration's value.
pub(crate) fn check(css: &str, value: &str) -> Result<(), String> {
    match css {
        "cursor" => cursor(value).map(|_| ()),
        "pointer-events" => pointer_events(value).map(|_| ()),
        "touch-action" => touch_action(value).map(|_| ()),
        "visibility" => visibility(value).map(|_| ()),
        _ => Ok(()),
    }
}

/// Top-level comma-separated items, functions kept whole.
pub(crate) fn commas(v: &str) -> Vec<&str> {
    let (mut depth, mut from, mut out) = (0i32, 0, Vec::new());
    for (k, c) in v.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                out.push(v[from..k].trim());
                from = k + 1;
            }
            _ => {}
        }
    }
    out.push(v[from..].trim());
    out
}

fn word(value: &str) -> Result<String, String> {
    match tokens(value)?.as_slice() {
        [Tok::Ident(w)] => Ok(w.to_ascii_lowercase()),
        [t] => Err(format!("'{}' is not a keyword", describe(t))),
        _ => Err("one keyword".into()),
    }
}

/// `cursor`: its images in order, then the keyword (CSS UI 4 §5.1: the
/// last item is a keyword, every one before it an image).
pub(crate) fn cursor(value: &str) -> Result<(Vec<CursorImage>, Cursor), String> {
    let items = commas(value);
    let (last, images) = items.split_last().expect("one item at least");
    let keyword = word(last).and_then(|w| {
        Cursor::from_css(&w).ok_or_else(|| format!("cursor: '{w}' is not a cursor keyword"))
    })?;
    let mut out = Vec::new();
    for item in images {
        let t = tokens(item)?;
        let (url, rest) = match t.split_first() {
            Some((Tok::Func(f, a), rest)) if f == "url" => (super::paint_css::url(a)?, rest),
            _ => {
                return Err(format!(
                    "cursor: '{item}' is not url(<image>) [<x> <y>]; the keyword comes last"
                ));
            }
        };
        let num = |t: &Tok| match t {
            Tok::Num(n, u) if u.is_empty() || u == "px" => n.parse::<f32>().ok(),
            _ => None,
        };
        let hotspot = match rest {
            [] => None,
            [x, y] => Some([
                num(x).ok_or("cursor: a hotspot is two numbers")?,
                num(y).ok_or("cursor: a hotspot is two numbers")?,
            ]),
            _ => return Err("cursor: url(<image>) takes a hotspot of two numbers".into()),
        };
        out.push(CursorImage { url, hotspot });
    }
    Ok((out, keyword))
}

pub(crate) fn pointer_events(value: &str) -> Result<PointerEvents, String> {
    use SvgPointer as S;
    let w = word(value)?;
    let svg = |b| Ok(PointerEvents::Svg(SvgPointer(b)));
    match w.as_str() {
        "auto" => Ok(PointerEvents::Auto),
        "none" => Ok(PointerEvents::None),
        "visiblepainted" => svg(S::FILL | S::STROKE | S::PAINTED | S::VISIBLE),
        "visiblefill" => svg(S::FILL | S::VISIBLE),
        "visiblestroke" => svg(S::STROKE | S::VISIBLE),
        "visible" => svg(S::FILL | S::STROKE | S::VISIBLE),
        "painted" => svg(S::FILL | S::STROKE | S::PAINTED),
        "fill" => svg(S::FILL),
        "stroke" => svg(S::STROKE),
        "all" => svg(S::FILL | S::STROKE),
        "bounding-box" => svg(S::BOX),
        _ => Err(format!("pointer-events: '{w}' is not a value")),
    }
}

/// `touch-action`: `None` for `auto`, else the allowed gestures.
pub(crate) fn touch_action(value: &str) -> Result<Option<u8>, String> {
    let t = tokens(value)?;
    let words: Vec<String> = t
        .iter()
        .map(|t| match t {
            Tok::Ident(w) => Ok(w.to_ascii_lowercase()),
            t => Err(format!("touch-action: '{}' is not a keyword", describe(t))),
        })
        .collect::<Result<_, _>>()?;
    match words.as_slice() {
        [w] if w == "auto" => return Ok(None),
        [w] if w == "none" => return Ok(Some(0)),
        [w] if w == "manipulation" => {
            return Ok(Some(Interact::PAN_X | Interact::PAN_Y | Interact::PINCH));
        }
        _ => {}
    }
    let (mut x, mut y, mut pinch) = (None, None, false);
    for w in &words {
        let (slot, bits) = match w.as_str() {
            "pan-x" => (&mut x, Interact::PAN_X),
            "pan-left" => (&mut x, Interact::PAN_LEFT),
            "pan-right" => (&mut x, Interact::PAN_RIGHT),
            "pan-y" => (&mut y, Interact::PAN_Y),
            "pan-up" => (&mut y, Interact::PAN_UP),
            "pan-down" => (&mut y, Interact::PAN_DOWN),
            "pinch-zoom" if !pinch => {
                pinch = true;
                continue;
            }
            _ => {
                return Err(format!(
                    "touch-action: '{}' is not valid here; write auto, none, manipulation or pan-x/-left/-right, pan-y/-up/-down and pinch-zoom once each",
                    words.join(" ")
                ));
            }
        };
        if slot.replace(bits).is_some() {
            return Err(format!(
                "touch-action: '{}' pans one axis twice",
                words.join(" ")
            ));
        }
    }
    Ok(Some(
        x.unwrap_or(0) | y.unwrap_or(0) | if pinch { Interact::PINCH } else { 0 },
    ))
}

pub(crate) fn visibility(value: &str) -> Result<Visibility, String> {
    match word(value)?.as_str() {
        "visible" => Ok(Visibility::Visible),
        "hidden" => Ok(Visibility::Hidden),
        "collapse" => Ok(Visibility::Collapse),
        w => Err(format!(
            "visibility: '{w}' is not visible, hidden or collapse"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_images_fall_back_to_the_keyword() {
        let (images, k) = cursor("url(sword) 4 2, url(7), not-allowed").unwrap();
        assert_eq!(k, Cursor::NotAllowed);
        assert_eq!(images.len(), 2);
        assert_eq!(images[0].hotspot, Some([4.0, 2.0]));
        assert_eq!(images[1].hotspot, None);
        // The keyword is required, and comes last.
        assert!(cursor("url(sword)").is_err());
        assert!(cursor("pointer, url(sword)").is_err());
        assert!(cursor("url(a) 4, pointer").is_err());
    }

    #[test]
    fn touch_action_combines_one_pan_per_axis() {
        let pan = touch_action("pan-left pan-down pinch-zoom")
            .unwrap()
            .unwrap();
        assert_eq!(
            pan,
            Interact::PAN_LEFT | Interact::PAN_DOWN | Interact::PINCH
        );
        assert_eq!(touch_action("auto").unwrap(), None);
        assert_eq!(touch_action("none").unwrap(), Some(0));
        assert!(touch_action("pan-x pan-left").is_err());
        assert!(touch_action("none pan-x").is_err());
    }
}
