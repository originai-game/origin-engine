//! CSS properties of how an image is drawn on document elements (CSS
//! Images 3 §5): `image-rendering`, which picks the sampling of every
//! image the element draws (its `<image>`, `<art>`, background, border
//! image), and `object-position`, which places an `<image>` or `<art>`
//! inside its box under `object-fit`.
//!
//! Each is one reflected prop holding its CSS value as written, checked
//! here when the stylesheet is lowered and read here again when the
//! element is built. `image-rendering` inherits through the cascade
//! ([`super::build::cascade::CARRIED_BY_DEFAULT`]).

use super::css::{Tok, describe, tokens};

/// The props this module reads, by their CSS names.
pub(crate) fn prop_of(css: &str) -> Option<&'static str> {
    Some(match css {
        "image-rendering" => "imageRendering",
        "object-position" => "objectPosition",
        _ => return None,
    })
}

/// How each prop's value is written (the catalogue's column).
pub(crate) fn form(prop: &str) -> Option<&'static str> {
    Some(match prop {
        "imageRendering" => "auto | smooth | high-quality | pixelated | crisp-edges",
        "objectPosition" => {
            "<position>: one to four of left, center, right, top, bottom, <n>px, <n>%"
        }
        _ => return None,
    })
}

/// Checks a declaration's value.
pub(crate) fn check(css: &str, value: &str) -> Result<(), String> {
    match css {
        "image-rendering" => image_rendering(value).map(|_| ()),
        "object-position" => object_position(value).map(|_| ()),
        _ => Ok(()),
    }
}

/// `image-rendering`: whether its images are sampled at the nearest
/// texel. `pixelated` and `crisp-edges` are; `auto`, `smooth` and
/// `high-quality` keep the texture's own sampling (a `kind pixel`
/// texture is cooked nearest). The legacy `optimizeSpeed` and
/// `optimizeQuality` are `pixelated` and `smooth` (CSS Images 3 §5.3).
pub(crate) fn image_rendering(value: &str) -> Result<bool, String> {
    match tokens(value)?.as_slice() {
        [Tok::Ident(w)] => match w.to_ascii_lowercase().as_str() {
            "auto" | "smooth" | "high-quality" | "optimizequality" => Ok(false),
            "pixelated" | "crisp-edges" | "optimizespeed" => Ok(true),
            w => Err(format!(
                "image-rendering: '{w}' is not auto, smooth, high-quality, pixelated or crisp-edges"
            )),
        },
        [t] => Err(format!(
            "image-rendering: '{}' is not a keyword",
            describe(t)
        )),
        _ => Err("image-rendering: one keyword".into()),
    }
}

/// `object-position`: per axis, the fraction of the free space and the
/// px after it (`right 10px` is 1 and -10), as `background-position`.
pub(crate) fn object_position(value: &str) -> Result<[(f32, f32); 2], String> {
    let t = tokens(value)?;
    if t.is_empty() {
        return Err("object-position: a position".into());
    }
    super::paint_css::bg_position(&t).map_err(|e| format!("object-position: {e}"))
}

/// `list-style-image`: `none` or `url(name)`, a picture the game
/// registered (`UiDesc.images`); the name, or `None` for `none`.
pub fn list_style_image(value: &str) -> Result<Option<String>, String> {
    let v = value.trim();
    if v == "none" {
        return Ok(None);
    }
    let inner = v
        .strip_prefix("url(")
        .or_else(|| v.strip_prefix("image("))
        .and_then(|r| r.strip_suffix(')'))
        .ok_or_else(|| format!("list-style-image takes none or url(name), not {v}"))?;
    let name = inner.trim().trim_matches(|c| c == '"' || c == '\'');
    if name.is_empty() {
        return Err("list-style-image: url() names a registered picture".into());
    }
    Ok(Some(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_position_takes_edge_offsets() {
        assert_eq!(object_position("center").unwrap(), [(0.5, 0.0); 2]);
        assert_eq!(
            object_position("right 10px bottom 25%").unwrap(),
            [(1.0, -10.0), (0.75, 0.0)]
        );
        assert_eq!(
            object_position("top left").unwrap(),
            [(0.0, 0.0), (0.0, 0.0)]
        );
        assert!(object_position("").is_err());
        assert!(object_position("left left").is_err());
        assert!(image_rendering("pixelated").unwrap());
        assert!(!image_rendering("smooth").unwrap());
        assert!(image_rendering("pixel").is_err());
    }
}
