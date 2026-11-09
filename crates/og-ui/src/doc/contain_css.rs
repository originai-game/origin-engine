//! CSS Containment 2's properties (`contain`, `content-visibility`), CSS
//! Sizing 4's `contain-intrinsic-*` (§5), CSS UI 4's `resize` (§5.2) and
//! CSS Will Change 1's `will-change` on document elements.
//!
//! Each is one reflected prop holding its CSS value as written, checked
//! here when the stylesheet is lowered and read here again when the
//! element is built ([`contain`]). `contain-intrinsic-size` and its
//! logical forms lower to the two physical longhands.

use super::Value;
use super::css::{Tok, describe, tokens};
use super::tracks::Lowered;
use crate::layout::{Contain, ContentVisibility};

/// The props this module reads, by their CSS names.
pub(crate) fn prop_of(css: &str) -> Option<&'static str> {
    Some(match css {
        "contain" => "contain",
        "content-visibility" => "contentVisibility",
        "contain-intrinsic-width" => "containIntrinsicWidth",
        "contain-intrinsic-height" => "containIntrinsicHeight",
        "resize" => "resize",
        "will-change" => "willChange",
        _ => return None,
    })
}

/// How each prop's value is written (the catalogue's column).
pub(crate) fn form(prop: &str) -> Option<&'static str> {
    Some(match prop {
        "contain" => "none | strict | content | [size | inline-size] || layout || style || paint",
        "contentVisibility" => "visible | auto | hidden",
        "containIntrinsicWidth" | "containIntrinsicHeight" => {
            "none | <n>px | auto <n>px | auto none"
        }
        "resize" => "none | both | horizontal | vertical | block | inline",
        "willChange" => "auto | <property>, …",
        _ => return None,
    })
}

/// Checks a declaration's value.
pub(crate) fn check(css: &str, value: &str) -> Result<(), String> {
    match css {
        "contain" => bits(value).map(|_| ()),
        "content-visibility" => visibility(value).map(|_| ()),
        "contain-intrinsic-width" | "contain-intrinsic-height" => intrinsic(value).map(|_| ()),
        "resize" => resize(value).map(|_| ()),
        "will-change" => will_change(value).map(|_| ()),
        _ => Ok(()),
    }
}

/// `contain-intrinsic-size` (`<width> [<height>]`), `-block-size` and
/// `-inline-size`: the physical longhands (horizontal writing mode).
pub(crate) fn lower(name: &str, value: &str) -> Option<Result<Vec<Lowered>, String>> {
    let text = |p: &'static str, v: &str| (p, vec![Value::String(v.trim().to_string())]);
    Some(match name {
        "contain-intrinsic-size" => (|| {
            let parts = split_pairs(value)?;
            let (w, h) = match parts.as_slice() {
                [one] => (one.clone(), one.clone()),
                [w, h] => (w.clone(), h.clone()),
                _ => return Err("contain-intrinsic-size: one or two sizes".into()),
            };
            intrinsic(&w)?;
            intrinsic(&h)?;
            Ok(vec![
                text("containIntrinsicWidth", &w),
                text("containIntrinsicHeight", &h),
            ])
        })(),
        "contain-intrinsic-inline-size" => {
            intrinsic(value).map(|_| vec![text("containIntrinsicWidth", value)])
        }
        "contain-intrinsic-block-size" => {
            intrinsic(value).map(|_| vec![text("containIntrinsicHeight", value)])
        }
        _ => return None,
    })
}

/// A `contain-intrinsic-size` value's sizes, each `auto <x>` kept whole.
fn split_pairs(value: &str) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::new();
    let mut auto = false;
    for w in value.split_whitespace() {
        if w.eq_ignore_ascii_case("auto") {
            if auto {
                return Err("contain-intrinsic-size: 'auto auto'".into());
            }
            auto = true;
            continue;
        }
        out.push(if auto {
            format!("auto {w}")
        } else {
            w.to_string()
        });
        auto = false;
    }
    if auto || out.is_empty() {
        return Err("contain-intrinsic-size: auto takes a length or none after it".into());
    }
    Ok(out)
}

fn words(value: &str) -> Result<Vec<String>, String> {
    tokens(value)?
        .iter()
        .map(|t| match t {
            Tok::Ident(w) => Ok(w.to_ascii_lowercase()),
            t => Err(format!("'{}' is not a keyword", describe(t))),
        })
        .collect()
}

/// `contain`: its containment bits (CSS Containment 2 §3).
pub(crate) fn bits(value: &str) -> Result<u8, String> {
    let w = words(value)?;
    let all = Contain::SIZE_W | Contain::SIZE_H | Contain::LAYOUT | Contain::STYLE | Contain::PAINT;
    match w.as_slice() {
        [one] if one == "none" => return Ok(0),
        [one] if one == "strict" => return Ok(all),
        [one] if one == "content" => return Ok(all & !(Contain::SIZE_W | Contain::SIZE_H)),
        [] => return Err("contain: a keyword".into()),
        _ => {}
    }
    let mut out = 0u8;
    let mut sized = false;
    for w in &w {
        let b = match w.as_str() {
            "size" | "inline-size" if sized => {
                return Err("contain: size and inline-size are one choice".into());
            }
            "size" => Contain::SIZE_W | Contain::SIZE_H,
            "inline-size" => Contain::SIZE_W,
            "layout" => Contain::LAYOUT,
            "style" => Contain::STYLE,
            "paint" => Contain::PAINT,
            w => {
                return Err(format!(
                    "contain: '{w}' is not none, strict, content, size, inline-size, layout, style or paint"
                ));
            }
        };
        sized |= matches!(w.as_str(), "size" | "inline-size");
        if out & b != 0 {
            return Err(format!("contain: '{w}' twice"));
        }
        out |= b;
    }
    Ok(out)
}

pub(crate) fn visibility(value: &str) -> Result<ContentVisibility, String> {
    match words(value)?.as_slice() {
        [w] if w == "visible" => Ok(ContentVisibility::Visible),
        [w] if w == "auto" => Ok(ContentVisibility::Auto),
        [w] if w == "hidden" => Ok(ContentVisibility::Hidden),
        _ => Err("content-visibility: visible, auto or hidden".into()),
    }
}

/// One `contain-intrinsic-*` value: whether `auto`, and its px (`None`:
/// `none`).
pub(crate) fn intrinsic(value: &str) -> Result<(bool, Option<f32>), String> {
    let t = tokens(value)?;
    let (auto, rest) = match t.split_first() {
        Some((Tok::Ident(w), rest)) if w.eq_ignore_ascii_case("auto") => (true, rest),
        _ => (false, t.as_slice()),
    };
    let len = match rest {
        [Tok::Ident(w)] if w.eq_ignore_ascii_case("none") => None,
        [Tok::Num(n, u)] if u == "px" || (u.is_empty() && n == "0") => Some(
            n.parse::<f32>()
                .ok()
                .filter(|v| *v >= 0.0)
                .ok_or("contain-intrinsic-size: a length is not negative")?,
        ),
        _ => {
            return Err("contain-intrinsic-size: none, a px length, or auto before either".into());
        }
    };
    Ok((auto, len))
}

/// `resize`: whether the user may resize it across and down.
pub(crate) fn resize(value: &str) -> Result<[bool; 2], String> {
    match words(value)?.as_slice() {
        [w] => match w.as_str() {
            "none" => Ok([false, false]),
            "both" => Ok([true, true]),
            "horizontal" | "inline" => Ok([true, false]),
            "vertical" | "block" => Ok([false, true]),
            w => Err(format!(
                "resize: '{w}' is not none, both, horizontal, vertical, block or inline"
            )),
        },
        _ => Err("resize: one keyword".into()),
    }
}

/// `will-change`: whether it names a property that makes a stacking
/// context or a group of its own (`transform`, `opacity`, `filter`…):
/// such an element is drawn on a layer of its own while it changes.
pub(crate) fn will_change(value: &str) -> Result<bool, String> {
    let mut layer = false;
    let items = super::interact_css::commas(value);
    for item in &items {
        let w = match words(item)?.as_slice() {
            [w] => w.clone(),
            _ => return Err("will-change: auto or properties separated by commas".into()),
        };
        match w.as_str() {
            "auto" if items.len() == 1 => {}
            "auto" | "will-change" | "none" | "inherit" | "initial" | "unset" | "default" => {
                return Err(format!("will-change: '{w}' cannot be listed"));
            }
            "transform" | "translate" | "rotate" | "scale" | "opacity" | "filter"
            | "backdrop-filter" | "mix-blend-mode" | "isolation" | "clip-path" | "mask"
            | "mask-image" | "perspective" | "contain" => layer = true,
            _ => {}
        }
    }
    Ok(layer)
}

/// An element's containment from its props: `contain`, with the layout,
/// style and paint containment `content-visibility: auto` and `hidden`
/// add (CSS Containment 2 §4), and its intrinsic sizes.
pub(crate) fn contain(get: impl Fn(&str) -> Option<String>) -> Contain {
    let mut c = Contain {
        bits: get("contain").and_then(|v| bits(&v).ok()).unwrap_or(0),
        visibility: get("contentVisibility")
            .and_then(|v| visibility(&v).ok())
            .unwrap_or_default(),
        intrinsic: [(false, None); 2],
    };
    if c.visibility != ContentVisibility::Visible {
        c.bits |= Contain::LAYOUT | Contain::STYLE | Contain::PAINT;
    }
    for (k, name) in ["containIntrinsicWidth", "containIntrinsicHeight"]
        .into_iter()
        .enumerate()
    {
        if let Some(Ok(v)) = get(name).map(|v| intrinsic(&v)) {
            c.intrinsic[k] = v;
        }
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contain_keywords_follow_containment_2() {
        let all =
            Contain::SIZE_W | Contain::SIZE_H | Contain::LAYOUT | Contain::STYLE | Contain::PAINT;
        assert_eq!(bits("strict"), Ok(all));
        assert_eq!(
            bits("content"),
            Ok(Contain::LAYOUT | Contain::STYLE | Contain::PAINT)
        );
        assert_eq!(
            bits("paint inline-size"),
            Ok(Contain::PAINT | Contain::SIZE_W)
        );
        assert!(bits("size inline-size").is_err());
        assert!(bits("none paint").is_err());
        assert_eq!(intrinsic("auto 300px"), Ok((true, Some(300.0))));
        assert_eq!(intrinsic("auto none"), Ok((true, None)));
        assert!(intrinsic("auto").is_err());
        let lowered = lower("contain-intrinsic-size", "auto 10px 20px")
            .unwrap()
            .unwrap();
        assert_eq!(lowered[0].1, vec![Value::String("auto 10px".into())]);
        assert_eq!(lowered[1].1, vec![Value::String("20px".into())]);
        assert_eq!(will_change("scroll-position, opacity"), Ok(true));
        assert!(will_change("auto, opacity").is_err());
    }
}
