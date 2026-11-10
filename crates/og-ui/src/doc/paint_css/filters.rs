use super::*;

/// One CSS filter function (Filter Effects 1 §13), in the order written.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum FilterFn {
    /// `blur(<σ>)`, CSS px.
    Blur(f32),
    /// A colour function as its matrix.
    Matrix(og_render::UiColorMatrix),
    /// `drop-shadow([<colour>] <x> <y> [<blur>])`; no colour is
    /// `currentColor`.
    DropShadow {
        offset: [f32; 2],
        blur: f32,
        color: Option<ColorRef>,
    },
    /// `url(#id)`: an inline SVG `<filter>`, written `url(#id)`.
    Url(String),
}

/// `filter` or `backdrop-filter` (`what`): its functions in order, or none.
pub(crate) fn filter_fns(text: &str, what: &str) -> Result<Vec<FilterFn>, String> {
    let t = tokens(text)?;
    if matches!(t.as_slice(), [Tok::Ident(w)] if w == "none") {
        return Ok(Vec::new());
    }
    t.into_iter()
        .map(|t| match t {
            Tok::Func(f, a) => {
                let a: Vec<Tok> = a.into_iter().flatten().collect();
                filter_fn(&f, &a).map_err(|e| format!("{f}(): {e}"))
            }
            t => Err(format!("'{}' is not a {what} function", describe(&t))),
        })
        .collect()
}

fn filter_fn(f: &str, a: &[Tok]) -> Result<FilterFn, String> {
    use FilterFn::Matrix;
    // An amount: a number or percentage, not negative; 1 when left out.
    let amount = |most: f32| -> Result<f32, String> {
        let v = match a {
            [] => 1.0,
            [t] => match number(t)? {
                (v, "%") => v / 100.0,
                (v, "") => v,
                _ => return Err(format!("'{}' is not an amount: <n> or <n>%", describe(t))),
            },
            _ => return Err("one amount".into()),
        };
        if v < 0.0 {
            return Err("an amount is not negative".into());
        }
        Ok(v.min(most))
    };
    let diag = |k: f32, o: f32| og_render::UiColorMatrix {
        rows: [[k, 0.0, 0.0, o], [0.0, k, 0.0, o], [0.0, 0.0, k, o]],
        opacity: 1.0,
    };
    // A 3×3 over the colour.
    let m3 = |m: [[f32; 3]; 3]| og_render::UiColorMatrix {
        rows: m.map(|r| [r[0], r[1], r[2], 0.0]),
        opacity: 1.0,
    };
    Ok(match f {
        "url" => FilterFn::Url(match a {
            [Tok::Hash(h)] => format!("url({h})"),
            [Tok::Str(s)] if s.starts_with('#') => format!("url({s})"),
            _ => return Err("url(#id) names a <filter>".into()),
        }),
        "blur" => FilterFn::Blur(match a {
            [] => 0.0,
            [t] => px(t)?.max(0.0),
            _ => return Err("one length".into()),
        }),
        "brightness" => Matrix(diag(amount(f32::MAX)?, 0.0)),
        "contrast" => {
            let k = amount(f32::MAX)?;
            Matrix(diag(k, 0.5 - 0.5 * k))
        }
        "invert" => {
            let k = amount(1.0)?;
            Matrix(diag(1.0 - 2.0 * k, k))
        }
        "opacity" => Matrix(og_render::UiColorMatrix {
            opacity: amount(1.0)?,
            ..Default::default()
        }),
        "grayscale" => {
            let k = 1.0 - amount(1.0)?;
            Matrix(m3([
                [
                    0.2126 + 0.7874 * k,
                    0.7152 - 0.7152 * k,
                    0.0722 - 0.0722 * k,
                ],
                [
                    0.2126 - 0.2126 * k,
                    0.7152 + 0.2848 * k,
                    0.0722 - 0.0722 * k,
                ],
                [
                    0.2126 - 0.2126 * k,
                    0.7152 - 0.7152 * k,
                    0.0722 + 0.9278 * k,
                ],
            ]))
        }
        "sepia" => {
            let k = 1.0 - amount(1.0)?;
            Matrix(m3([
                [0.393 + 0.607 * k, 0.769 - 0.769 * k, 0.189 - 0.189 * k],
                [0.349 - 0.349 * k, 0.686 + 0.314 * k, 0.168 - 0.168 * k],
                [0.272 - 0.272 * k, 0.534 - 0.534 * k, 0.131 + 0.869 * k],
            ]))
        }
        "saturate" => {
            let s = amount(f32::MAX)?;
            Matrix(m3([
                [0.213 + 0.787 * s, 0.715 - 0.715 * s, 0.072 - 0.072 * s],
                [0.213 - 0.213 * s, 0.715 + 0.285 * s, 0.072 - 0.072 * s],
                [0.213 - 0.213 * s, 0.715 - 0.715 * s, 0.072 + 0.928 * s],
            ]))
        }
        "hue-rotate" => {
            let h = match a {
                [] => 0.0,
                [t] => angle(t)?,
                _ => return Err("one angle".into()),
            };
            let (sn, c) = og_core::math::sin_cos(h);
            Matrix(m3([
                [
                    0.213 + c * 0.787 - sn * 0.213,
                    0.715 - c * 0.715 - sn * 0.715,
                    0.072 - c * 0.072 + sn * 0.928,
                ],
                [
                    0.213 - c * 0.213 + sn * 0.143,
                    0.715 + c * 0.285 + sn * 0.140,
                    0.072 - c * 0.072 - sn * 0.283,
                ],
                [
                    0.213 - c * 0.213 - sn * 0.787,
                    0.715 - c * 0.715 + sn * 0.715,
                    0.072 + c * 0.928 + sn * 0.072,
                ],
            ]))
        }
        "drop-shadow" => {
            let mut lengths = Vec::new();
            let mut color = None;
            for t in a {
                if matches!(t, Tok::Num(..)) {
                    lengths.push(px(t)?);
                } else if color.is_none() {
                    color = Some(self::color(t)?);
                } else {
                    return Err("one colour".into());
                }
            }
            match lengths.as_slice() {
                [x, y, rest @ ..] if rest.len() <= 1 => FilterFn::DropShadow {
                    offset: [*x, *y],
                    blur: rest.first().copied().unwrap_or(0.0).max(0.0),
                    color,
                },
                _ => return Err("[<colour>] <x> <y> [<blur>]".into()),
            }
        }
        f => {
            return Err(format!(
                "not a filter: url(#id), blur, brightness, contrast, drop-shadow, grayscale, hue-rotate, invert, opacity, saturate, sepia (writing {f})"
            ));
        }
    })
}

/// `m` then `n`: one matrix, as if `n` read `m`'s result unclamped.
fn then(m: &og_render::UiColorMatrix, n: &og_render::UiColorMatrix) -> og_render::UiColorMatrix {
    let mut rows = [[0.0; 4]; 3];
    for (i, row) in rows.iter_mut().enumerate() {
        for (j, v) in row.iter_mut().enumerate() {
            *v = (0..3).map(|k| n.rows[i][k] * m.rows[k][j]).sum::<f32>()
                + if j == 3 { n.rows[i][3] } else { 0.0 };
        }
    }
    og_render::UiColorMatrix {
        rows,
        opacity: m.opacity * n.opacity,
    }
}

/// Whether `m` keeps every colour in 0..1: then the clamp after it does
/// nothing, and the next function folds into it exactly.
fn contained(m: &og_render::UiColorMatrix) -> bool {
    m.rows.iter().all(|r| {
        let lo = r[3] + r[..3].iter().map(|v| v.min(0.0)).sum::<f32>();
        let hi = r[3] + r[..3].iter().map(|v| v.max(0.0)).sum::<f32>();
        lo >= -1e-5 && hi <= 1.0 + 1e-5
    })
}

/// `filter`'s functions as layers, the first innermost: a blur, a drop
/// shadow (its colour resolved by `color`), an SVG `<filter>` (found by
/// `url`), and each run of colour functions as one colour matrix, split
/// where a function could leave 0..1 so the clamp CSS does after each one
/// still falls between them. A `url(#id)` that names no filter drops the
/// whole list, as Filter Effects 1 §11 says.
pub(crate) fn filter_layers(
    fns: &[FilterFn],
    color: impl Fn(Option<&ColorRef>) -> [f32; 4],
    mut url: impl FnMut(&str) -> Option<std::sync::Arc<crate::paint::FilterUse>>,
) -> Vec<Layer> {
    let layer = |b: Builtin, params| Layer {
        material: Material::Builtin(b),
        params,
        filter: None,
    };
    let mut out = Vec::new();
    let mut run: Option<og_render::UiColorMatrix> = None;
    for f in fns {
        if let FilterFn::Matrix(m) = f {
            run = Some(match run.take() {
                Some(r) if contained(&r) => then(&r, m),
                Some(r) => {
                    out.push(layer(Builtin::ColorMatrix, r.params()));
                    *m
                }
                None => *m,
            });
            continue;
        }
        if let Some(r) = run.take() {
            out.push(layer(Builtin::ColorMatrix, r.params()));
        }
        let mut p = [[0.0; 4]; og_render::UI_MATERIAL_PARAMS];
        match f {
            FilterFn::Blur(sigma) => {
                // The layer blur's radius is twice its standard deviation.
                p[0][0] = 2.0 * sigma;
                out.push(layer(Builtin::Blur, p));
            }
            FilterFn::DropShadow {
                offset,
                blur,
                color: c,
            } => {
                p[0] = [offset[0], offset[1], *blur, 0.0];
                p[1] = color(c.as_ref());
                out.push(layer(Builtin::DropShadow, p));
            }
            FilterFn::Url(u) => {
                let Some(f) = url(u) else {
                    return Vec::new();
                };
                out.push(Layer {
                    filter: Some(f),
                    ..Layer::default()
                });
            }
            FilterFn::Matrix(_) => unreachable!("folded above"),
        }
    }
    if let Some(r) = run {
        out.push(layer(Builtin::ColorMatrix, r.params()));
    }
    out
}

pub(super) fn px(t: &Tok) -> Result<f32, String> {
    match number(t)? {
        (v, "px") | (v @ 0.0, "") => Ok(v),
        _ => Err(format!("'{}' is not a length in px", describe(t))),
    }
}

/// The glass material carries no paint or filter defaults of its own.
pub(crate) fn glass_material(text: &str) -> Result<Option<crate::paint::GlassParts>, String> {
    let t = tokens(text)?;
    if !matches!(t.first(), Some(Tok::Ident(v)) if v == "glass") {
        return Ok(None);
    }
    let [Tok::Ident(_), Tok::Ident(weight), Tok::Ident(elevation)] = t.as_slice() else {
        return Err("glass needs a weight (regular | thick) and elevation (lift | float)".into());
    };
    Ok(Some(crate::paint::GlassParts {
        weight: Some(match weight.as_str() {
            "regular" => og_render::UiGlassWeight::Regular,
            "thick" => og_render::UiGlassWeight::Thick,
            _ => return Err("glass weight: regular | thick".into()),
        }),
        elevation: Some(match elevation.as_str() {
            "lift" => og_render::UiElevation::Lift,
            "float" => og_render::UiElevation::Float,
            _ => return Err("glass elevation: lift | float".into()),
        }),
        own_filter: true,
        ..Default::default()
    }))
}

/// A glass surface's filter in the native backdrop shader. A filter chain
/// that needs intermediate clamping cannot be folded into this shader.
pub(crate) fn glass_effect(
    mut glass: crate::paint::GlassParts,
    fns: &[FilterFn],
) -> Result<Effect, String> {
    let mut blur = 0.0;
    for f in fns {
        match f {
            FilterFn::Blur(sigma) if glass.matrix.is_none() => {
                blur = f32::sqrt(blur * blur + sigma * sigma);
            }
            FilterFn::Matrix(m) => {
                glass.matrix = Some(match glass.matrix {
                    None => *m,
                    Some(previous) if contained(&previous) => then(&previous, m),
                    _ => {
                        return Err(
                            "glass backdrop-filter needs an intermediate colour clamp".into()
                        );
                    }
                });
            }
            _ => return Err("glass backdrop-filter requires blur before colour functions".into()),
        }
    }
    Ok(Effect::Backdrop {
        blur: blur * og_render::ui::GLASS_BLUR_SCALE,
        saturation: 1.0,
        brightness: 1.0,
        tint: ColorRef::Literal(Srgba::TRANSPARENT),
        glass,
    })
}

/// `backdrop-filter`'s functions as its stages, in order: a run of
/// colour functions is one matrix while it keeps colour in 0..1 (as
/// `filter`'s layers fold), a `drop-shadow()` with no colour takes
/// `current`, and a `url(#id)` is the `<filter>` `url` finds; one that
/// names none drops the list. `None` for no functions.
pub(crate) fn backdrop_stages(
    fns: &[FilterFn],
    current: ColorRef,
    mut url: impl FnMut(&str) -> Option<std::sync::Arc<crate::paint::FilterUse>>,
) -> Option<Vec<crate::paint::BackdropStage>> {
    use crate::paint::BackdropStage as S;
    let mut out: Vec<S> = Vec::new();
    for f in fns {
        match f {
            FilterFn::Matrix(m) => match out.last_mut() {
                Some(S::Matrix(r)) if contained(r) => *r = then(r, m),
                _ => out.push(S::Matrix(*m)),
            },
            FilterFn::Blur(s) => out.push(S::Blur(*s)),
            FilterFn::DropShadow {
                offset,
                blur,
                color,
            } => out.push(S::DropShadow {
                offset: *offset,
                blur: *blur,
                color: color.clone().unwrap_or_else(|| current.clone()),
            }),
            FilterFn::Url(u) => out.push(S::Url(url(u)?)),
        }
    }
    (!out.is_empty()).then_some(out)
}

/// `isolation`: whether it isolates.
pub(crate) fn isolation(text: &str) -> Result<bool, String> {
    match text.trim() {
        "auto" => Ok(false),
        "isolate" => Ok(true),
        w => Err(format!("'{w}' is not an isolation: auto | isolate")),
    }
}

/// `mix-blend-mode`.
pub(crate) fn blend(text: &str) -> Result<Blend, String> {
    Ok(match text.trim() {
        "normal" => Blend::Normal,
        "multiply" => Blend::Multiply,
        "screen" => Blend::Screen,
        "plus-lighter" => Blend::Add,
        "overlay" => Blend::Overlay,
        "soft-light" => Blend::SoftLight,
        "color-dodge" => Blend::ColorDodge,
        "darken" => Blend::Darken,
        "lighten" => Blend::Lighten,
        "color-burn" => Blend::ColorBurn,
        "hard-light" => Blend::HardLight,
        "difference" => Blend::Difference,
        "exclusion" => Blend::Exclusion,
        "hue" => Blend::Hue,
        "saturation" => Blend::Saturation,
        "color" => Blend::Color,
        "luminosity" => Blend::Luminosity,
        w => {
            return Err(format!("'{w}' is not a blend mode: {BLEND_MODES}"));
        }
    })
}

const BLEND_MODES: &str = "normal, multiply, screen, overlay, darken, lighten, color-dodge, color-burn, hard-light, soft-light, difference, exclusion, hue, saturation, color, luminosity, plus-lighter";

/// `text-shadow: <x> <y> [<blur>] [<colour>], …` (unset colour: the text's,
/// given as `current`).
pub(crate) fn text_shadows(text: &str, current: ColorRef) -> Result<Vec<TextShadow>, String> {
    if text.trim() == "none" {
        return Ok(Vec::new());
    }
    super::boxes::items(text)
        .into_iter()
        .map(|item| {
            let mut lengths = Vec::new();
            let mut c = None;
            for t in tokens(item)? {
                if matches!(t, Tok::Num(..)) {
                    lengths.push(px(&t)?);
                } else if c.is_none() {
                    c = Some(color(&t)?);
                } else {
                    return Err(format!("a text-shadow takes one colour: {item}"));
                }
            }
            match lengths.as_slice() {
                [x, y, rest @ ..] if rest.len() <= 1 => Ok(TextShadow {
                    offset: [*x, *y],
                    blur: rest.first().copied().unwrap_or(0.0).max(0.0),
                    color: c.unwrap_or_else(|| current.clone()),
                }),
                _ => Err(format!(
                    "a text-shadow is <x> <y> [<blur>] [<colour>]: {item}"
                )),
            }
        })
        .collect()
}
