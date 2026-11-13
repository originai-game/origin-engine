//! CSS Position 3 insets and CSS Anchor Positioning 1, read from their
//! text: `anchor()` and `anchor-size()` resolve against other elements'
//! boxes after layout, so these declarations keep their text
//! ([`crate::anchor`] parses and places them).

use super::Value;
use super::tracks::Lowered;
use crate::anchor;

/// The inset props, top, right, bottom, left.
const SIDES: [&str; 4] = ["insetTop", "insetRight", "insetBottom", "insetLeft"];

/// `value` split at its top-level spaces (a function's spaces kept).
pub(crate) fn words(value: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0, None);
    for (k, c) in value.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            c if c.is_whitespace() && depth == 0 => {
                if let Some(s) = start.take() {
                    out.push(&value[s..k]);
                }
                continue;
            }
            _ => {}
        }
        start.get_or_insert(k);
    }
    if let Some(s) = start {
        out.push(&value[s..]);
    }
    out
}

/// One inset's value, checked: `auto`, a length, `<n>%`, `anchor()` or a
/// `calc()` of one plus px.
fn inset(v: &str) -> Result<Vec<Value>, String> {
    if v != "auto" && !v.starts_with("var(") {
        anchor::parse_inset(v)?;
    }
    Ok(vec![Value::String(v.to_string())])
}

pub(crate) fn lower(name: &str, value: &str) -> Option<Result<Vec<Lowered>, String>> {
    let v = value.trim();
    if v.contains("bind(") {
        return None;
    }
    let text = |s: &str| vec![Value::String(s.trim().to_string())];
    let sides = |w: &[&str], order: [usize; 4]| -> Result<Vec<Lowered>, String> {
        order
            .iter()
            .zip(w)
            .map(|(&k, w)| Ok((SIDES[k], inset(w)?)))
            .collect()
    };
    Some(match name {
        // CSS 2 §9.5 and CSS Logical Properties 1 §2.2.
        "float" => match v {
            "none" | "left" | "right" | "inline-start" | "inline-end" => {
                Ok(vec![("float", text(v))])
            }
            v => Err(format!(
                "'{v}' is no float: none, left, right, inline-start or inline-end"
            )),
        },
        "clear" => match v {
            "none" | "left" | "right" | "both" | "inline-start" | "inline-end" => {
                Ok(vec![("clear", text(v))])
            }
            v => Err(format!(
                "'{v}' is no clear: none, left, right, both, inline-start or inline-end"
            )),
        },
        "top" | "inset-block-start" => sides(&[v], [0; 4]),
        "right" | "inset-inline-end" => sides(&[v], [1; 4]),
        "bottom" | "inset-block-end" => sides(&[v], [2; 4]),
        "left" | "inset-inline-start" => sides(&[v], [3; 4]),
        // `inset` as `margin` is (top, right, bottom, left); the logical
        // pairs start, end (horizontal writing: block is top and bottom).
        "inset" | "inset-block" | "inset-inline" => {
            let w = words(v);
            let w: Vec<&str> = match (name, w.as_slice()) {
                ("inset", [a]) => vec![a, a, a, a],
                ("inset", [a, b]) => vec![a, b, a, b],
                ("inset", [a, b, c]) => vec![a, b, c, b],
                ("inset", [a, b, c, d]) => vec![a, b, c, d],
                ("inset-block", [a]) => vec![a, a],
                ("inset-block", [a, b]) => vec![a, b],
                ("inset-inline", [a]) => vec![a, a],
                ("inset-inline", [a, b]) => vec![a, b],
                _ => {
                    return Some(Err(format!(
                        "{name} takes {} insets",
                        if name == "inset" { "1 to 4" } else { "1 or 2" }
                    )));
                }
            };
            match name {
                "inset" => sides(&w, [0, 1, 2, 3]),
                "inset-block" => sides(&w, [0, 2, 0, 2]),
                _ => sides(&w, [3, 1, 3, 1]),
            }
        }
        "anchor-name" => {
            let names: Vec<&str> = v.split(',').map(str::trim).collect();
            if v == "none" {
                Ok(vec![("anchorName", text(""))])
            } else if let Some(bad) = names.iter().find(|n| !n.starts_with("--") || n.len() < 3) {
                Err(format!(
                    "'{bad}' is no anchor name: a dashed ident (anchor-name: --menu)"
                ))
            } else {
                Ok(vec![("anchorName", text(&names.join(" ")))])
            }
        }
        "position-anchor" => match v {
            "auto" => Ok(vec![("positionAnchor", text(""))]),
            n if n.starts_with("--") && n.len() > 2 && !n.contains(' ') => {
                Ok(vec![("positionAnchor", text(n))])
            }
            n => Err(format!(
                "'{n}' is no anchor: auto or a dashed ident (position-anchor: --menu)"
            )),
        },
        "position-area" => anchor::parse_area(v).map(|_| vec![("positionArea", text(v))]),
        "position-try-fallbacks" => anchor::parse_tries(v).map(|_| vec![("positionTry", text(v))]),
        "position-try-order" => order(v).map(|o| vec![("positionTryOrder", text(o))]),
        // `position-try: [<order>] <fallbacks>`.
        "position-try" => {
            let (o, rest) = match v.split_once(char::is_whitespace) {
                Some((o, rest)) if order(o).is_ok() => (o, rest.trim()),
                _ => ("normal", v),
            };
            order(o).and_then(|o| {
                anchor::parse_tries(rest)
                    .map(|_| vec![("positionTryOrder", text(o)), ("positionTry", text(rest))])
            })
        }
        "position-visibility" => match v {
            "always" | "anchors-visible" | "no-overflow" => {
                Ok(vec![("positionVisibility", text(v))])
            }
            v => Err(format!(
                "'{v}' is no position-visibility: always, anchors-visible or no-overflow"
            )),
        },
        "width" | "height" | "inline-size" | "block-size" if v.contains("anchor-size(") => {
            let prop = match name {
                "width" | "inline-size" => "anchorWidth",
                _ => "anchorHeight",
            };
            anchor::parse_size(v).map(|_| vec![(prop, text(v))])
        }
        _ => return None,
    })
}

/// `position-try-order`'s word.
fn order(v: &str) -> Result<&'static str, String> {
    Ok(match v {
        "normal" => "normal",
        "most-width" | "most-inline-size" => "most-width",
        "most-height" | "most-block-size" => "most-height",
        v => {
            return Err(format!(
                "'{v}' is no position-try-order: normal, most-width, most-height, most-block-size or most-inline-size"
            ));
        }
    })
}
