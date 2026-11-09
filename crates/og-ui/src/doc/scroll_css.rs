//! CSS scrolling and scrollbar properties on document elements, beyond
//! `overflow` and scroll snap's type and alignment: `overscroll-behavior`
//! (CSS Overscroll 1), `scroll-behavior`, `scroll-margin*`,
//! `scroll-padding*` and `scroll-snap-stop` (CSS Scroll Snap 1, CSSOM
//! View), `scrollbar-color` and `scrollbar-gutter` (CSS Scrollbars 1,
//! CSS Overflow 3 §4).
//!
//! Each longhand is one reflected prop holding its CSS value as written,
//! checked here when the stylesheet is lowered (shorthands and logical
//! sides expand to the physical longhands, horizontal-tb) and read here
//! into the element's [`ScrollCss`] when it is built.

use super::css::{Tok, describe, tokens};
use crate::scroll::{Overscroll, ScrollCss};

/// Each longhand's reflected prop and CSS name.
pub(crate) const PROPS: [(&str, &str); 14] = [
    ("overscrollBehaviorX", "overscroll-behavior-x"),
    ("overscrollBehaviorY", "overscroll-behavior-y"),
    ("scrollBehavior", "scroll-behavior"),
    ("scrollMarginTop", "scroll-margin-top"),
    ("scrollMarginRight", "scroll-margin-right"),
    ("scrollMarginBottom", "scroll-margin-bottom"),
    ("scrollMarginLeft", "scroll-margin-left"),
    ("scrollPaddingTop", "scroll-padding-top"),
    ("scrollPaddingRight", "scroll-padding-right"),
    ("scrollPaddingBottom", "scroll-padding-bottom"),
    ("scrollPaddingLeft", "scroll-padding-left"),
    ("scrollSnapStop", "scroll-snap-stop"),
    ("scrollbarColor", "scrollbar-color"),
    ("scrollbarGutter", "scrollbar-gutter"),
];

const SIDES: [&str; 4] = ["Top", "Right", "Bottom", "Left"];

/// A shorthand's or logical side's longhands (CSS names), in the order its
/// values fill them: `None` for a name this module does not own.
fn expand(css: &str) -> Option<Vec<String>> {
    if let Some(p) = PROPS.iter().find(|(_, c)| *c == css) {
        return Some(vec![p.1.to_owned()]);
    }
    if let Some(axis) = css.strip_prefix("overscroll-behavior") {
        return Some(match axis {
            "" => vec![
                "overscroll-behavior-x".into(),
                "overscroll-behavior-y".into(),
            ],
            "-inline" => vec!["overscroll-behavior-x".into()],
            "-block" => vec!["overscroll-behavior-y".into()],
            _ => return None,
        });
    }
    let base = ["scroll-margin", "scroll-padding"]
        .into_iter()
        .find(|b| css.starts_with(b))?;
    let side = |s: &str| format!("{base}-{s}");
    Some(match &css[base.len()..] {
        // top right bottom left, as margin's 1 to 4 values fill them.
        "" => ["top", "right", "bottom", "left"].map(side).to_vec(),
        "-block" => vec![side("top"), side("bottom")],
        "-inline" => vec![side("left"), side("right")],
        "-block-start" => vec![side("top")],
        "-block-end" => vec![side("bottom")],
        "-inline-start" => vec![side("left")],
        "-inline-end" => vec![side("right")],
        _ => return None,
    })
}

fn prop(css: &str) -> &'static str {
    PROPS.iter().find(|p| p.1 == css).map_or("", |p| p.0)
}

/// The reflected props a declaration of `css` sets (for the cascade's
/// targets), empty for a name this module does not own.
pub(crate) fn targets(css: &str) -> Vec<&'static str> {
    expand(css)
        .map(|l| l.iter().map(|c| prop(c)).collect())
        .unwrap_or_default()
}

/// How each prop's value is written (the catalogue's column).
pub(crate) fn form(prop: &str) -> Option<&'static str> {
    Some(match prop {
        "overscrollBehaviorX" | "overscrollBehaviorY" => "auto | contain | none",
        "scrollBehavior" => "auto | smooth",
        p if p.starts_with("scrollMargin") => "<length>",
        p if p.starts_with("scrollPadding") => "auto | <length> | <n>%",
        "scrollSnapStop" => "normal | always",
        "scrollbarColor" => "auto | <thumb colour> <track colour>",
        "scrollbarGutter" => "auto | stable [both-edges]",
        _ => return None,
    })
}

/// Lowers a declaration this module owns into its longhands' values, each
/// checked: `None` for any other name.
pub(crate) fn lower(css: &str, value: &str) -> Option<Result<Vec<(&'static str, String)>, String>> {
    let longhands = expand(css)?;
    Some((|| {
        let parts = values(value)?;
        let fill: Vec<&str> = match (longhands.len(), parts.as_slice()) {
            (1, [one]) => vec![one],
            (1, _) => return Ok(vec![(prop(&longhands[0]), check(&longhands[0], value)?)]),
            // A pair: the first and second, the second defaulting to the first.
            (2, [a]) => vec![a, a],
            (2, [a, b]) => vec![a, b],
            // Four sides, as `margin` fills them.
            (4, [a]) => vec![a, a, a, a],
            (4, [a, b]) => vec![a, b, a, b],
            (4, [a, b, c]) => vec![a, b, c, b],
            (4, [a, b, c, d]) => vec![a, b, c, d],
            _ => return Err(format!("{css} takes 1 to {} values", longhands.len())),
        };
        longhands
            .iter()
            .zip(fill)
            .map(|(l, v)| Ok((prop(l), check(l, v)?)))
            .collect()
    })())
}

/// Top-level space-separated items, functions kept whole.
fn values(v: &str) -> Result<Vec<&str>, String> {
    let (mut depth, mut from, mut out) = (0i32, None, Vec::new());
    for (k, c) in v.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            c if c.is_whitespace() && depth == 0 => {
                if let Some(f) = from.take() {
                    out.push(&v[f..k]);
                }
                continue;
            }
            _ => {}
        }
        from.get_or_insert(k);
    }
    if let Some(f) = from {
        out.push(&v[f..]);
    }
    if out.is_empty() {
        return Err("no value".into());
    }
    Ok(out)
}

fn word(value: &str) -> Result<String, String> {
    match tokens(value)?.as_slice() {
        [Tok::Ident(w)] => Ok(w.to_ascii_lowercase()),
        [t] => Err(format!("'{}' is not a keyword", describe(t))),
        _ => Err("one keyword".into()),
    }
}

/// A longhand's value checked, as it is kept.
fn check(css: &str, value: &str) -> Result<String, String> {
    let value = value.trim();
    match prop(css) {
        "overscrollBehaviorX" | "overscrollBehaviorY" => overscroll(value).map(|_| ()),
        "scrollBehavior" => match word(value)?.as_str() {
            "auto" | "smooth" => Ok(()),
            w => Err(format!("scroll-behavior: '{w}' is not auto or smooth")),
        },
        "scrollSnapStop" => match word(value)?.as_str() {
            "normal" | "always" => Ok(()),
            w => Err(format!("scroll-snap-stop: '{w}' is not normal or always")),
        },
        "scrollbarColor" => scrollbar_color(value).map(|_| ()),
        "scrollbarGutter" => gutter(value).map(|_| ()),
        p => length(value, p.starts_with("scrollPadding")).map(|_| ()),
    }
    .map_err(|e| format!("{css}: {e}"))?;
    Ok(value.to_owned())
}

fn overscroll(value: &str) -> Result<Overscroll, String> {
    match word(value)?.as_str() {
        "auto" => Ok(Overscroll::Auto),
        "contain" => Ok(Overscroll::Contain),
        "none" => Ok(Overscroll::None),
        w => Err(format!("'{w}' is not auto, contain or none")),
    }
}

/// A scroll margin's px, or a scroll padding's px and fraction of its
/// view (`auto` is 0, as UAs take it). Relative units are px by now.
fn length(value: &str, padding: bool) -> Result<(f32, f32), String> {
    match tokens(value)?.as_slice() {
        [Tok::Ident(w)] if padding && w.eq_ignore_ascii_case("auto") => Ok((0.0, 0.0)),
        [Tok::Num(n, u)] => {
            let v: f32 = n.parse().map_err(|_| format!("'{n}' is not a number"))?;
            match u.as_str() {
                "px" => Ok((v, 0.0)),
                "" if v == 0.0 => Ok((0.0, 0.0)),
                "%" if padding => Ok((0.0, v / 100.0)),
                u => Err(format!(
                    "'{n}{u}' is not a length{}",
                    if padding { " or a percentage" } else { "" }
                )),
            }
        }
        _ => Err(format!(
            "'{value}' is not one length{}",
            if padding { " or percentage" } else { "" }
        )),
    }
}

fn scrollbar_color(value: &str) -> Result<Option<[crate::css_color::Color; 2]>, String> {
    let t = tokens(value)?;
    match t.as_slice() {
        [Tok::Ident(w)] if w.eq_ignore_ascii_case("auto") => Ok(None),
        [thumb, track] => Ok(Some([
            crate::css_color::parse(thumb)?,
            crate::css_color::parse(track)?,
        ])),
        _ => Err("auto, or two colours: the thumb's and the track's".into()),
    }
}

/// `scrollbar-gutter`: 0 auto, 1 stable, 2 stable on both edges.
fn gutter(value: &str) -> Result<u8, String> {
    let t = tokens(value)?;
    let w: Vec<String> = t
        .iter()
        .map(|t| match t {
            Tok::Ident(w) => w.to_ascii_lowercase(),
            t => describe(t),
        })
        .collect();
    match w.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["auto"] => Ok(0),
        ["stable"] => Ok(1),
        ["stable", "both-edges"] | ["both-edges", "stable"] => Ok(2),
        _ => Err("auto | stable [both-edges]".into()),
    }
}

/// The element's [`ScrollCss`] from its props (`get`), its colours resolved
/// in `cx`. Values were checked when the CSS was lowered.
pub(crate) fn build<'a>(
    get: impl Fn(&str) -> Option<&'a str>,
    cx: &crate::css_color::Context,
) -> ScrollCss {
    let mut s = ScrollCss::default();
    let ov = |p| get(p).and_then(|v| overscroll(v).ok()).unwrap_or_default();
    s.overscroll = [ov("overscrollBehaviorX"), ov("overscrollBehaviorY")];
    s.smooth = get("scrollBehavior").is_some_and(|v| v.eq_ignore_ascii_case("smooth"));
    s.snap_stop = get("scrollSnapStop").is_some_and(|v| v.eq_ignore_ascii_case("always"));
    s.thin = get("scrollbarWidth").is_some_and(|v| v.eq_ignore_ascii_case("thin"));
    for (k, side) in SIDES.iter().enumerate() {
        let len = |p: String, pad| {
            get(&p)
                .and_then(|v| length(v, pad).ok())
                .unwrap_or_default()
        };
        s.margin[k] = len(format!("scrollMargin{side}"), false).0;
        s.padding[k] = len(format!("scrollPadding{side}"), true);
    }
    if let Some(Ok(Some([thumb, track]))) = get("scrollbarColor").map(scrollbar_color) {
        s.colors = Some([thumb.resolve(cx), track.resolve(cx)]);
    }
    s.gutter = get("scrollbarGutter")
        .and_then(|v| gutter(v).ok())
        .unwrap_or(0);
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shorthands_fill_their_sides_as_margin_does() {
        let got = lower("scroll-padding", "4px 10%").unwrap().unwrap();
        assert_eq!(
            got,
            vec![
                ("scrollPaddingTop", "4px".into()),
                ("scrollPaddingRight", "10%".into()),
                ("scrollPaddingBottom", "4px".into()),
                ("scrollPaddingLeft", "10%".into()),
            ]
        );
        let got = lower("scroll-margin-inline", "1px 2px").unwrap().unwrap();
        assert_eq!(
            got,
            vec![
                ("scrollMarginLeft", "1px".into()),
                ("scrollMarginRight", "2px".into())
            ]
        );
        let got = lower("overscroll-behavior", "contain").unwrap().unwrap();
        assert_eq!(got.len(), 2);
        // A margin is no percentage; a gutter's both-edges needs stable.
        assert!(lower("scroll-margin", "10%").unwrap().is_err());
        assert!(lower("scrollbar-gutter", "both-edges").unwrap().is_err());
        assert!(
            lower("scrollbar-gutter", "stable both-edges")
                .unwrap()
                .is_ok()
        );
        assert!(lower("scrollbar-color", "red").unwrap().is_err());
        assert!(lower("overflow", "auto").is_none());
    }
}
