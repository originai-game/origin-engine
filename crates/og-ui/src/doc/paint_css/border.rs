use super::*;

/// A `border-image-width` or `-outset` value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BiLen {
    Px(f32),
    /// Of the border image area's side.
    Frac(f32),
    /// Times the border's width.
    Times(f32),
    Auto,
}

/// `border-image-*` or `mask-border-*` other than its source, top, right,
/// bottom, left as CSS writes them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BorderSpec {
    /// Image px (`Px`, a number) or a fraction of the image.
    pub slice: [Pos; 4],
    pub fill: bool,
    pub width: [BiLen; 4],
    pub outset: [BiLen; 4],
    pub repeat: [og_render::UiEdge; 2],
    pub luminance: bool,
}

/// One to four values as CSS's sides: top, right, bottom, left.
fn sides<T: Copy>(t: &[Tok], f: impl Fn(&Tok) -> Result<T, String>) -> Result<[T; 4], String> {
    let v: Vec<T> = t.iter().map(f).collect::<Result<_, _>>()?;
    Ok(match v.as_slice() {
        [a] => [*a; 4],
        [a, b] => [*a, *b, *a, *b],
        [a, b, c] => [*a, *b, *c, *b],
        [a, b, c, d] => [*a, *b, *c, *d],
        _ => return Err("one to four values: top, right, bottom, left".into()),
    })
}

fn bi_len(t: &Tok, pct: bool, auto: bool) -> Result<BiLen, String> {
    match t {
        Tok::Ident(w) if w == "auto" && auto => Ok(BiLen::Auto),
        t => match number(t)? {
            (v, _) if v < 0.0 => Err("a border image's width or outset is not negative".into()),
            (v, "") => Ok(BiLen::Times(v)),
            (v, "px") => Ok(BiLen::Px(v)),
            (v, "%") if pct => Ok(BiLen::Frac(v / 100.0)),
            _ => Err(format!(
                "'{}' is not <n>px, <n>{}",
                describe(t),
                if pct { ", <n>% or auto" } else { "" }
            )),
        },
    }
}

/// A border image's parts by longhand: source, slice, width, outset,
/// repeat (and a mask border's mode), each `None` when not written.
/// `mask` reads `mask-border`'s initial values.
pub(crate) fn border_spec(
    parts: [Option<&str>; 5],
    mode: Option<&str>,
    mask: bool,
) -> Result<BorderSpec, String> {
    use og_render::UiEdge as E;
    let toks = |t: Option<&str>| -> Result<Option<Vec<Tok>>, String> {
        t.filter(|t| !t.trim().is_empty()).map(tokens).transpose()
    };
    let mut b = BorderSpec {
        slice: [if mask { Pos::Px(0.0) } else { Pos::Frac(1.0) }; 4],
        fill: false,
        width: [if mask { BiLen::Auto } else { BiLen::Times(1.0) }; 4],
        outset: [BiLen::Times(0.0); 4],
        repeat: [E::Stretch; 2],
        luminance: false,
    };
    if let Some(t) = toks(parts[1])? {
        b.fill = t.iter().any(|t| matches!(t, Tok::Ident(w) if w == "fill"));
        let n: Vec<Tok> = t
            .into_iter()
            .filter(|t| !matches!(t, Tok::Ident(w) if w == "fill"))
            .collect();
        b.slice = sides(&n, |t| match number(t)? {
            (v, _) if v < 0.0 => Err("a slice is not negative".into()),
            (v, "") => Ok(Pos::Px(v)),
            (v, "%") => Ok(Pos::Frac(v / 100.0)),
            _ => Err(format!("'{}' is not a slice: <n> or <n>%", describe(t))),
        })?;
    }
    if let Some(t) = toks(parts[2])? {
        b.width = sides(&t, |t| bi_len(t, true, true))?;
    }
    if let Some(t) = toks(parts[3])? {
        b.outset = sides(&t, |t| bi_len(t, false, false))?;
    }
    if let Some(t) = toks(parts[4])? {
        let w = |t: &Tok| match t {
            Tok::Ident(w) => Ok(match w.as_str() {
                "stretch" => E::Stretch,
                "repeat" => E::Repeat,
                "round" => E::Round,
                "space" => E::Space,
                _ => return Err(format!("'{w}' is not stretch, repeat, round or space")),
            }),
            t => Err(format!("'{}' is not a repeat", describe(t))),
        };
        b.repeat = match t.as_slice() {
            [a] => [w(a)?; 2],
            [a, b] => [w(a)?, w(b)?],
            _ => return Err("one or two of stretch, repeat, round, space".into()),
        };
    }
    if let Some(m) = mode.filter(|m| !m.trim().is_empty()) {
        b.luminance = match m.trim() {
            "luminance" => true,
            "alpha" => false,
            m => return Err(format!("'{m}' is not a mask border mode: alpha, luminance")),
        };
    }
    Ok(b)
}

/// A `border-image` or `mask-border` shorthand split into its longhands'
/// text: source, slice, width, outset, repeat, then a mask border's mode.
pub(crate) fn border_shorthand(text: &str, mask: bool) -> Result<[String; 6], String> {
    let t = tokens(text)?;
    let mut out: [String; 6] = Default::default();
    let push = |s: &mut String, tok: &Tok| {
        if !s.is_empty() {
            s.push(' ');
        }
        s.push_str(&tok_text(tok));
    };
    // 0 before the slice, 1 in it, 2 the width, 3 the outset.
    let mut part = 0;
    for tok in &t {
        let ident = match tok {
            Tok::Ident(w) => Some(w.as_str()),
            _ => None,
        };
        if is_image(tok) || ident == Some("none") {
            if !out[0].is_empty() {
                return Err("one image".into());
            }
            push(&mut out[0], tok);
            if part > 0 {
                part = 4;
            }
        } else if matches!(tok, Tok::Slash) {
            if !(1..3).contains(&part) {
                return Err(
                    "a width or outset follows the slice: <slice> / <width> / <outset>".into(),
                );
            }
            part += 1;
        } else if matches!(ident, Some("stretch" | "repeat" | "round" | "space")) {
            push(&mut out[4], tok);
            part = 4;
        } else if mask && matches!(ident, Some("alpha" | "luminance")) {
            push(&mut out[5], tok);
        } else if matches!(tok, Tok::Num(..)) || ident == Some("fill") || ident == Some("auto") {
            if part == 0 || part == 4 {
                if !out[1].is_empty() {
                    return Err("one slice".into());
                }
                part = 1;
            }
            push(&mut out[part.min(3)], tok);
        } else {
            return Err(format!("'{}' is not part of a border image", describe(tok)));
        }
    }
    border_spec(
        [
            None,
            Some(&out[1]),
            Some(&out[2]),
            Some(&out[3]),
            Some(&out[4]),
        ],
        Some(&out[5]),
        mask,
    )?;
    if !out[0].is_empty() && out[0] != "none" {
        image(&tokens(&out[0])?[0])?;
    }
    Ok(out)
}

/// A token as CSS text again.
fn tok_text(t: &Tok) -> String {
    match t {
        Tok::Ident(s) | Tok::Hash(s) => s.clone(),
        Tok::Num(n, u) => format!("{n}{u}"),
        Tok::Str(s) => format!("{s:?}"),
        Tok::Func(f, args) => format!(
            "{f}({})",
            args.iter()
                .map(|a| a.iter().map(tok_text).collect::<Vec<_>>().join(" "))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Tok::Slash => "/".into(),
    }
}

/// A border image source: `none`, `url(…)` or a gradient.
pub(crate) fn border_source(text: &str) -> Result<CssImage, String> {
    match tokens(text)?.as_slice() {
        [t] => image(t),
        _ => Err("one image: url(…), a gradient or none".into()),
    }
}

/// The longhands of `border-image` and `mask-border`: (CSS name after the
/// prefix, prop). `mode` is the mask border's only.
pub(crate) const BORDER_IMAGE_PROPS: [(&str, &str); 5] = [
    ("source", "borderImageSource"),
    ("slice", "borderImageSlice"),
    ("width", "borderImageWidth"),
    ("outset", "borderImageOutset"),
    ("repeat", "borderImageRepeat"),
];
pub(crate) const MASK_BORDER_PROPS: [(&str, &str); 6] = [
    ("source", "maskBorderSource"),
    ("slice", "maskBorderSlice"),
    ("width", "maskBorderWidth"),
    ("outset", "maskBorderOutset"),
    ("repeat", "maskBorderRepeat"),
    ("mode", "maskBorderMode"),
];

/// The props a C4 declaration writes, each with its text, or `None` when
/// `name` is not one of them: the background and mask layer longhands,
/// `mask` and `mask-image`, and `border-image`, `mask-border` and their
/// longhands. A bad value is named.
pub(crate) fn lower(
    name: &str,
    value: &str,
) -> Option<Result<Vec<(&'static str, String)>, String>> {
    let v = value.trim().to_owned();
    let one = |prop: &'static str| vec![(prop, v.clone())];
    let check_layer = |mask: bool, key: &str| -> Result<(), String> {
        let mut l = [CssLayer::new(CssImage::None, mask)];
        longhands(&mut l, &[(key, Some(value))], mask)
    };
    let border = |props: &[(&'static str, &'static str)],
                  mask: bool|
     -> Result<Vec<(&'static str, String)>, String> {
        let parts = border_shorthand(value, mask)?;
        Ok(props.iter().zip(parts).map(|((_, p), v)| (*p, v)).collect())
    };
    Some(match name {
        "mask" | "mask-image" => masks(value).map(|_| {
            let mut out = one("maskImage");
            if name == "mask" {
                // The shorthand resets the longhands and the mask border.
                out.extend(MASK_LONGHANDS.iter().map(|(_, p)| (*p, String::new())));
                out.extend(MASK_BORDER_PROPS.iter().map(|(_, p)| (*p, String::new())));
            }
            out
        }),
        "border-image" => border(&BORDER_IMAGE_PROPS, false),
        "mask-border" => border(&MASK_BORDER_PROPS, true),
        n => {
            if let Some(key) = n.strip_prefix("background-")
                && let Some((_, prop)) = BACKGROUND_LONGHANDS.iter().find(|(k, _)| *k == key)
            {
                check_layer(false, key).map(|_| one(prop))
            } else if let Some(key) = n.strip_prefix("mask-border-")
                && let Some((_, prop)) = MASK_BORDER_PROPS.iter().find(|(k, _)| *k == key)
            {
                border_check(key, value, true).map(|_| one(prop))
            } else if let Some(key) = n.strip_prefix("mask-")
                && let Some((_, prop)) = MASK_LONGHANDS.iter().find(|(k, _)| *k == key)
            {
                check_layer(true, key).map(|_| one(prop))
            } else if let Some(key) = n.strip_prefix("border-image-")
                && let Some((_, prop)) = BORDER_IMAGE_PROPS.iter().find(|(k, _)| *k == key)
            {
                border_check(key, value, false).map(|_| one(prop))
            } else {
                return None;
            }
        }
    })
}

fn border_check(key: &str, value: &str, mask: bool) -> Result<(), String> {
    let mut parts = [None; 5];
    let mut mode = None;
    match key {
        "source" => return border_source(value).map(|_| ()),
        "mode" => mode = Some(value),
        k => {
            let i = ["source", "slice", "width", "outset", "repeat"]
                .iter()
                .position(|p| *p == k)
                .unwrap_or(1);
            parts[i] = Some(value);
        }
    }
    border_spec(parts, mode, mask).map(|_| ())
}
