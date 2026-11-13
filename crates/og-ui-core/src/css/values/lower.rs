use super::*;

/// An entry setting `key` to `values`.
fn e(key: &str, values: Vec<Value>) -> Part {
    Part::Entry(key.to_string(), values)
}

/// A part of a joint property: its value, or `bind(…)`.
fn part(k: &str, t: &Tok) -> Result<Vec<Value>, String> {
    if let Tok::Func(f, a) = t
        && f == "bind"
    {
        return match a.as_slice() {
            [v] => match v.as_slice() {
                [Tok::Ident(p)] => Ok(vec![Value::Word("bind".into()), Value::Word(p.clone())]),
                [Tok::Str(s)] => Ok(vec![Value::Word("bind".into()), Value::String(s.clone())]),
                _ => Err("bind(path) or bind(\"expression\")".into()),
            },
            _ => Err("bind(path) or bind(\"expression\")".into()),
        };
    }
    if k.starts_with("scale") {
        want_num(t).map(|v| vec![v])
    } else {
        want_len(t).map(|v| vec![v])
    }
}

/// The entries one element declaration (or state value when `state`)
/// sets, in order. Sizes, grid lines and `grid-auto-flow` come back as
/// parts the block joins ([`Joined`]).
pub(super) fn lower_decl(name: &str, value: &str, state: bool) -> Result<Vec<Part>, String> {
    if !value.contains("bind(")
        && let Some(parts) = super::tracks::lower(name, value)
    {
        return Ok(parts?.into_iter().map(|(k, v)| e(k, v)).collect());
    }
    if let Some(parts) = super::position_css::lower(name, value) {
        return Ok(parts?.into_iter().map(|(k, v)| e(k, v)).collect());
    }
    if let Some(parts) = super::multicol_css::lower(name, value) {
        return Ok(parts?.into_iter().map(|(k, v)| e(k, v)).collect());
    }
    if let Some(parts) = super::view_transition::lower(name, value) {
        return Ok(parts?.into_iter().map(|(k, v)| e(k, v)).collect());
    }
    if let Some(parts) = super::contain_css::lower(name, value) {
        return Ok(parts?.into_iter().map(|(k, v)| e(k, v)).collect());
    }
    let t = tokens(value)?;
    if t.is_empty() {
        return Err(format!("{name} has no value"));
    }
    if name == "display" {
        return lower_display(value);
    }
    // A bound declaration: the whole value is the binding.
    if let [Tok::Func(f, a)] = t.as_slice()
        && f == "bind"
        && !matches!(name, "translate" | "scale")
    {
        return lower_bind(name, a, state);
    }
    match name {
        "translate" => lower_translate(name, t, state),
        "scale" => lower_scale(t),
        "rotate" => Ok(vec![e("rotate", vec![want_angle(&one(&t, name)?)?])]),
        "opacity" => Ok(vec![e("opacity", vec![want_num(&one(&t, name)?)?])]),
        "z-index" => Ok(vec![e("z", vec![want_num(&one(&t, name)?)?])]),
        "border-radius" if state => Ok(vec![e("radius", vec![want_len(&one(&t, name)?)?])]),
        n if !state && !super::boxes::corner_props(n).is_empty() => {
            super::boxes::lower_corners(n, &t)
                .map(|parts| parts.into_iter().map(|(k, v)| e(k, v)).collect())
        }
        "background" | "background-image"
            if !state && super::paint_css::background(value).is_some() =>
        {
            lower_background_image(name, value)
        }
        "background" | "background-color" => Ok(vec![e("background", paint_vals(&t)?)]),
        "color" => Ok(vec![e(
            if state { "ink" } else { "color" },
            paint_vals(&t)?,
        )]),
        _ if state => Err(format!(
            "'{name}' is not a state's: {}",
            [
                "translate",
                "scale",
                "rotate",
                "opacity",
                "border-radius",
                "z-index",
                "background",
                "color"
            ]
            .join(", ")
        )),
        "background-image" if value.trim() == "none" => Ok(vec![e(
            "backgroundImage",
            vec![Value::String("none".into())],
        )]),
        n if let Some(parts) = super::scroll_css::lower(n, value) => Ok(parts?
            .into_iter()
            .map(|(k, v)| e(k, vec![Value::String(v)]))
            .collect()),
        n if let Some(prop) = super::interact_css::prop_of(n) => {
            super::interact_css::check(n, value)?;
            Ok(vec![e(prop, vec![Value::String(value.trim().to_owned())])])
        }
        n if let Some(prop) = super::image_css::prop_of(n) => {
            super::image_css::check(n, value)?;
            Ok(vec![e(prop, vec![Value::String(value.trim().to_owned())])])
        }
        n if let Some(prop) = super::contain_css::prop_of(n) => {
            super::contain_css::check(n, value)?;
            Ok(vec![e(prop, vec![Value::String(value.trim().to_owned())])])
        }
        "scroll-snap-type" | "scroll-snap-align" => {
            let prop = match name {
                "scroll-snap-type" => super::paint_css::snap_type(value).map(|_| "scrollSnapType"),
                _ => super::paint_css::snap_align(value).map(|_| "scrollSnapAlign"),
            }?;
            Ok(vec![e(prop, vec![Value::String(value.trim().to_owned())])])
        }
        // CSS UI 4 §5.2: clip, ellipsis or a string the cut ends in (one
        // value: the end edge's).
        "text-overflow" if !state => lower_text_overflow(name, t),
        n if !state && let Some(parts) = super::paint_css::lower(n, value) => Ok(parts?
            .into_iter()
            .map(|(prop, v)| e(prop, vec![Value::String(v)]))
            .collect()),
        "list-style-image" => {
            super::image_css::list_style_image(value)?;
            Ok(vec![e(
                "markerImage",
                vec![Value::String(value.trim().into())],
            )])
        }
        "-og-sound" => {
            crate::sound_css::parse(value)?;
            Ok(vec![e(
                "sound",
                vec![Value::String(value.trim().to_owned())],
            )])
        }
        "backdrop-filter"
        | "filter"
        | "mix-blend-mode"
        | "isolation"
        | "text-shadow"
        | "clip-path"
        | "shape-outside"
        | "shape-margin"
        | "shape-image-threshold"
        | "-og-material" => lower_effects(name, value),
        "width" | "height" | "min-width" | "max-width" | "min-height" | "max-height"
        | "inline-size" | "block-size" | "min-inline-size" | "max-inline-size"
        | "min-block-size" | "max-block-size" => lower_size(name, t),
        "aspect-ratio" => lower_aspect_ratio(t),
        "padding-top"
        | "padding-right"
        | "padding-bottom"
        | "padding-left"
        | "padding-block"
        | "padding-inline"
        | "padding-block-start"
        | "padding-block-end"
        | "padding-inline-start"
        | "padding-inline-end" => lower_padding_side(name, t),
        "padding" => lower_padding(t),
        "margin"
        | "margin-block"
        | "margin-inline"
        | "margin-inline-start"
        | "margin-inline-end"
        | "margin-block-start"
        | "margin-block-end" => lower_margin(name, t),
        n if super::boxes::longhands(n).len() > 1
            || super::boxes::logical(n).is_some()
            || n == "box-shadow"
            || n.ends_with("-width") && (n.starts_with("border-") || n == "outline-width")
            || n.ends_with("-color") && (n.starts_with("border-") || n == "outline-color") =>
        {
            super::boxes::lower(n, value, &t)
                .map(|parts| parts.into_iter().map(|(k, v)| e(k, v)).collect())
        }
        "grid-column" | "grid-row" => {
            let (start, span) = grid_line(&t)?;
            let axis = if name == "grid-column" { 0 } else { 1 };
            Ok(vec![Part::Grid(axis, start, span)])
        }
        "-og-text-path" => {
            crate::text_path::parse_shape(value)?;
            Ok(vec![e(
                "textPath",
                vec![Value::String(value.trim().to_owned())],
            )])
        }
        "-og-text-path-offset" => {
            crate::text_path::parse_offset(value)?;
            Ok(vec![e(
                "textPathOffset",
                vec![Value::String(value.trim().to_owned())],
            )])
        }
        "object-view-box" => lower_object_view_box(t),
        "flex-direction" => {
            let w = camel(want_ident(&one(&t, name)?, "a direction")?);
            let both = vec![Value::Word(w)];
            Ok(vec![
                e("direction", both.clone()),
                e("labelDirection", both),
            ])
        }
        "flex-wrap" => lower_flex_wrap(name, t),
        "flex-flow" => lower_flex_flow(t),
        "flex" => lower_flex(t),
        "flex-basis" => lower_flex_basis(name, value, t),
        "order" => match one(&t, name)? {
            Tok::Num(n, u) if u.is_empty() && !n.contains('.') => {
                Ok(vec![e("order", vec![number(&n)?])])
            }
            v => Err(format!("order takes an integer, not '{}'", describe(&v))),
        },
        "gap" => lower_gap(t),
        "row-gap" | "column-gap" => lower_gap_axis(name, t),
        "justify-content" | "align-content" | "align-items" | "align-self" | "justify-items"
        | "justify-self" => alignment(name, &t),
        "place-content" | "place-items" | "place-self" => lower_place(name, t),
        "grid-auto-flow" => lower_grid_auto_flow(t),
        _ => lower_other(name, value, t),
    }
}

/// Lowers `display`: whether the element is shown, and how.
fn lower_display(value: &str) -> Result<Vec<Part>, String> {
    let word = match value.trim() {
        "none" => {
            return Ok(vec![e("visible", vec![Value::Word("false".into())])]);
        }
        "initial" => {
            return Ok(vec![e("visible", vec![Value::Word("true".into())])]);
        }
        v => display_word(v)?,
    };
    Ok(vec![
        e("visible", vec![Value::Word("true".into())]),
        e("display", vec![Value::Word(word.into())]),
    ])
}

/// Lowers a bound declaration: the whole value is the binding.
#[allow(clippy::ptr_arg)]
fn lower_bind(name: &str, a: &Vec<Vec<Tok>>, state: bool) -> Result<Vec<Part>, String> {
    let src = match a.as_slice() {
        [v] => match v.as_slice() {
            [Tok::Ident(p)] => Value::Word(p.clone()),
            [Tok::Str(s)] => Value::String(s.clone()),
            _ => return Err("bind(path) or bind(\"expression\")".into()),
        },
        _ => return Err("bind(path) or bind(\"expression\")".into()),
    };
    let prop = if state {
        return Err("a state's values are literals".into());
    } else {
        match name {
            "grid-column" | "grid-row" | "grid-auto-flow" => {
                return Err(format!("{name} binds each part: write it as values"));
            }
            n => STYLE
                .iter()
                .find(|(_, c)| *c == n)
                .map(|(p, _)| *p)
                .ok_or_else(|| format!("'{n}' is not a property"))?,
        }
    };
    Ok(vec![e(prop, vec![Value::Word("bind".into()), src])])
}

/// Lowers `"scale"`.
fn lower_scale(t: Vec<Tok>) -> Result<Vec<Part>, String> {
    match t.as_slice() {
        [s] => Ok(vec![e("scale", part("scale", s)?)]),
        [x, y] => Ok(vec![
            e("scaleX", part("scaleX", x)?),
            e("scaleY", part("scaleY", y)?),
        ]),
        _ => Err("scale: <both> or <x> <y>".into()),
    }
}

/// Lowers `"gap"`.
fn lower_gap(t: Vec<Tok>) -> Result<Vec<Part>, String> {
    match t.as_slice() {
        [g] => Ok(vec![e("gap", vec![want_len(g)?])]),
        [r, c] => Ok(vec![
            e("rowGap", vec![want_len(r)?]),
            e("columnGap", vec![want_len(c)?]),
        ]),
        _ => Err("gap: <row-gap> [<column-gap>]".into()),
    }
}

/// Lowers `"row-gap" | "column-gap"`.
fn lower_gap_axis(name: &str, t: Vec<Tok>) -> Result<Vec<Part>, String> {
    match one(&t, name)? {
        Tok::Ident(w) if w == "normal" => Ok(Vec::new()),
        v => Ok(vec![e(
            if name == "row-gap" {
                "rowGap"
            } else {
                "columnGap"
            },
            vec![want_len(&v)?],
        )]),
    }
}

/// Lowers `"flex-basis"`.
fn lower_flex_basis(name: &str, value: &str, t: Vec<Tok>) -> Result<Vec<Part>, String> {
    match one(&t, name)? {
        Tok::Ident(w) if w == "auto" => Ok(Vec::new()),
        Tok::Ident(w) if w == "content" => {
            Ok(vec![e("flexBasis", vec![Value::Word("maxContent".into())])])
        }
        _ => {
            let parts = lower_decl("width", value, false)?;
            Ok(parts
                .into_iter()
                .map(|p| match p {
                    Part::Size(_, SizeBit::Kind(v)) => e("flexBasis", v),
                    p => p,
                })
                .collect())
        }
    }
}

/// Lowers `"translate"`.
fn lower_translate(name: &str, t: Vec<Tok>, state: bool) -> Result<Vec<Part>, String> {
    let (x, y) = match t.as_slice() {
        [Tok::Ident(w)] if name == "translate" && w == "none" => (
            Tok::Num("0".into(), String::new()),
            Tok::Num("0".into(), String::new()),
        ),
        [x] => (x.clone(), Tok::Num("0".into(), String::new())),
        [x, y] => (x.clone(), y.clone()),
        _ => return Err(format!("{name}: <x> <y>")),
    };
    // A percentage is of the element's own box, as CSS's: its
    // fraction moves it after layout beside the px (`xShift`), so
    // `translate: -50% -50%` centres it on its place.
    // The px pair first, as every reader of `translate` takes it;
    // the shifts after.
    let (mut px, mut shifts) = (Vec::new(), Vec::new());
    for (k, shift, t) in [("x", "xShift", &x), ("y", "yShift", &y)] {
        let fraction = match t {
            Tok::Num(n, u) if u == "%" && state => {
                return Err(format!("a state's translate takes px, not {n}%"));
            }
            Tok::Num(n, u) if u == "%" => {
                px.push(e(k, vec![number("0")?]));
                let f = n
                    .parse::<f64>()
                    .map_err(|_| format!("'{n}%' is not a number"))?;
                (f / 100.0).to_string()
            }
            t => {
                px.push(e(k, part(k, t)?));
                "0".to_string()
            }
        };
        if !state {
            shifts.push(e(shift, vec![number(&fraction)?]));
        }
    }
    px.extend(shifts);
    Ok(px)
}

/// Lowers the background_image family.
fn lower_background_image(name: &str, value: &str) -> Result<Vec<Part>, String> {
    let (_, colour) = super::paint_css::background(value).unwrap()?;
    let mut out = vec![e(
        "backgroundImage",
        vec![Value::String(value.trim().to_owned())],
    )];
    if let Some(c) = colour {
        out.push(e("background", paint_vals(&[c])?));
    } else if name == "background" && value.trim() == "none" {
        out.push(e(
            "background",
            paint_vals(&[Tok::Ident("transparent".into())])?,
        ));
    }
    // The shorthand resets the longhands to its own layers' (an
    // empty value), so an earlier `background-size` stops applying
    // and a later one still does.
    if name == "background" {
        for (_, prop) in super::paint_css::BACKGROUND_LONGHANDS {
            out.push(e(prop, vec![Value::String(String::new())]));
        }
    }
    Ok(out)
}

/// Lowers `"text-overflow" if !state`.
fn lower_text_overflow(name: &str, t: Vec<Tok>) -> Result<Vec<Part>, String> {
    let (on, cut) = match one(&t, name)? {
        Tok::Ident(w) if w == "clip" => (false, String::new()),
        Tok::Ident(w) if w == "ellipsis" => (true, String::new()),
        Tok::Str(s) => (true, s),
        t => {
            return Err(format!(
                "text-overflow is clip, ellipsis or a string, not '{}'",
                describe(&t)
            ));
        }
    };
    Ok(vec![
        e("ellipsis", vec![Value::Word(on.to_string())]),
        e("ellipsisText", vec![Value::String(cut)]),
    ])
}

/// Lowers the effects family.
fn lower_effects(name: &str, value: &str) -> Result<Vec<Part>, String> {
    use super::paint_css as p;
    let prop = match name {
        "backdrop-filter" => p::filter_fns(value, "backdrop-filter").map(|_| "backdropFilter"),
        "filter" => p::filter_fns(value, "filter").map(|_| "filter"),
        "mix-blend-mode" => p::blend(value).map(|_| "mixBlendMode"),
        "isolation" => p::isolation(value).map(|_| "isolation"),
        "text-shadow" => p::text_shadows(
            value,
            crate::paint::ColorRef::Literal(crate::theme::Srgba::TRANSPARENT),
        )
        .map(|_| "textShadow"),
        "clip-path" => p::clip(value).map(|_| "clipPath"),
        "shape-outside" => p::outside(value).map(|_| "shapeOutside"),
        "shape-margin" => p::shape_margin(value).map(|_| "shapeMargin"),
        "shape-image-threshold" => p::shape_threshold(value).map(|_| "shapeImageThreshold"),
        _ => p::glass_material(value).and_then(|glass| {
            if glass.is_some() {
                Ok("material")
            } else {
                p::material(value).map(|_| "material")
            }
        }),
    }?;
    Ok(vec![e(prop, vec![Value::String(value.trim().to_owned())])])
}

/// Lowers the size family.
fn lower_size(name: &str, t: Vec<Tok>) -> Result<Vec<Part>, String> {
    let axis = usize::from(!(name.ends_with("width") || name.ends_with("inline-size")));
    let v = one(&t, name)?;
    let bound = name
        .strip_prefix("min-")
        .map(|_| true)
        .or(name.strip_prefix("max-").map(|_| false));
    // `auto` (and a bound's `none`) is the initial value: the
    // element's own size, unbounded (CSS Sizing 3 §3.1).
    if matches!(&v, Tok::Ident(k) if k == "auto" || (k == "none" && bound == Some(false))) {
        return Ok(Vec::new());
    }
    let mut parts = Vec::new();
    let vals = match &v {
        Tok::Ident(k) if k == "fit-content" => vec![Value::Word("fit".into())],
        Tok::Ident(k) if k == "stretch" => vec![Value::Word("grow".into())],
        Tok::Ident(k) if k == "min-content" => vec![Value::Word("minContent".into())],
        Tok::Ident(k) if k == "max-content" => vec![Value::Word("maxContent".into())],
        // fit-content(<length>): its content, at most the length and
        // at least its min-content (CSS Sizing 3 §3.2).
        Tok::Func(f, args) if f == "fit-content" && bound.is_none() => {
            let [arg] = args.as_slice() else {
                return Err(format!("{name}: fit-content(<length>)"));
            };
            let [len] = arg.as_slice() else {
                return Err(format!("{name}: fit-content(<length>)"));
            };
            match len {
                Tok::Num(n, u) if u == "%" => {
                    parts.push(Part::Size(
                        axis,
                        SizeBit::Range(
                            false,
                            vec![Value::Word("percent".into()), number(&shift(n, -2))?],
                        ),
                    ));
                    vec![Value::Word("fit".into())]
                }
                len => vec![Value::Word("fit".into()), number("0")?, want_len(len)?],
            }
        }
        Tok::Num(n, u) if u == "%" => {
            vec![Value::Word("percent".into()), number(&shift(n, -2))?]
        }
        Tok::Func(f, args)
            if matches!(
                f.as_str(),
                "calc" | "min" | "max" | "clamp" | "round" | "mod" | "rem" | "abs" | "hypot"
            ) =>
        {
            size_calc(name, f, args)?
        }
        v => vec![want_len(v)?],
    };
    parts.push(Part::Size(
        axis,
        match bound {
            Some(min) => SizeBit::Range(min, vals),
            None => SizeBit::Kind(vals),
        },
    ));
    Ok(parts)
}

/// Lowers `"aspect-ratio"`.
fn lower_aspect_ratio(t: Vec<Tok>) -> Result<Vec<Part>, String> {
    // `auto`, `<ratio>` or both (CSS Sizing 4 §2.1); `auto` alone is
    // the initial value, and a ratio is width / height.
    let ratio: Vec<&Tok> = t
        .iter()
        .filter(|v| !matches!(v, Tok::Ident(a) if a == "auto"))
        .collect();
    let n = |v: &Tok| -> Result<f64, String> {
        match v {
            Tok::Num(n, u) if u.is_empty() => {
                n.parse().map_err(|_| format!("'{n}' is not a number"))
            }
            v => Err(format!(
                "aspect-ratio: auto | <width> [/ <height>], not '{}'",
                describe(v)
            )),
        }
    };
    let r = match ratio.as_slice() {
        [] => return Ok(Vec::new()),
        [a] => n(a)?,
        [a, Tok::Slash, b] => {
            let b = n(b)?;
            if b == 0.0 { 0.0 } else { n(a)? / b }
        }
        _ => return Err("aspect-ratio: auto | <width> [/ <height>]".into()),
    };
    Ok(vec![e("aspect", vec![number(&super::calc::number(r))?])])
}

/// Lowers the padding_side family.
fn lower_padding_side(name: &str, t: Vec<Tok>) -> Result<Vec<Part>, String> {
    // CSS order: top, right, bottom, left (left to right).
    let sides: [Option<&Tok>; 4] = match (name, t.as_slice()) {
        ("padding-top" | "padding-block-start", [a]) => [Some(a), None, None, None],
        ("padding-right" | "padding-inline-end", [a]) => [None, Some(a), None, None],
        ("padding-bottom" | "padding-block-end", [a]) => [None, None, Some(a), None],
        ("padding-left" | "padding-inline-start", [a]) => [None, None, None, Some(a)],
        ("padding-block", [a]) => [Some(a), None, Some(a), None],
        ("padding-block", [a, b]) => [Some(a), None, Some(b), None],
        ("padding-inline", [a]) => [None, Some(a), None, Some(a)],
        ("padding-inline", [start, end]) => [None, Some(end), None, Some(start)],
        _ => {
            return Err(format!(
                "{name} takes one length (two for -block and -inline)"
            ));
        }
    };
    let mut out = Vec::new();
    for (side, prop) in
        sides
            .into_iter()
            .zip(["paddingTop", "paddingRight", "paddingBottom", "paddingLeft"])
    {
        if let Some(side) = side {
            out.push(e(prop, vec![want_len(side)?]));
        }
    }
    Ok(out)
}

/// Lowers `"padding"`.
fn lower_padding(t: Vec<Tok>) -> Result<Vec<Part>, String> {
    let n: Vec<Value> = t.iter().map(want_len).collect::<Result<_, _>>()?;
    let vals = match n.as_slice() {
        [a] => vec![a.clone()],
        [v, h] => vec![h.clone(), v.clone()],
        [top, h, b] => vec![h.clone(), top.clone(), h.clone(), b.clone()],
        [top, r, b, l] => vec![l.clone(), top.clone(), r.clone(), b.clone()],
        _ => return Err("padding takes 1 to 4 lengths".into()),
    };
    Ok(vec![e("padding", vals)])
}

/// Lowers the margin family.
fn lower_margin(name: &str, t: Vec<Tok>) -> Result<Vec<Part>, String> {
    // CSS order: top, right, bottom, left (left to right).
    let sides: [Option<&Tok>; 4] = match (name, t.as_slice()) {
        ("margin-block-start", [a]) => [Some(a), None, None, None],
        ("margin-inline-end", [a]) => [None, Some(a), None, None],
        ("margin-block-end", [a]) => [None, None, Some(a), None],
        ("margin-inline-start", [a]) => [None, None, None, Some(a)],
        ("margin", [a]) => [Some(a); 4],
        ("margin", [v, h]) => [Some(v), Some(h), Some(v), Some(h)],
        ("margin", [top, h, b]) => [Some(top), Some(h), Some(b), Some(h)],
        ("margin", [top, r, b, l]) => [Some(top), Some(r), Some(b), Some(l)],
        ("margin-block", [a]) => [Some(a), None, Some(a), None],
        ("margin-block", [a, b]) => [Some(a), None, Some(b), None],
        ("margin-inline", [a]) => [None, Some(a), None, Some(a)],
        ("margin-inline", [start, end]) => [None, Some(end), None, Some(start)],
        _ => {
            return Err(format!(
                "{name} takes 1 to {} values",
                if name == "margin" { 4 } else { 2 }
            ));
        }
    };
    let mut out = Vec::new();
    for (side, prop) in
        sides
            .into_iter()
            .zip(["marginTop", "marginRight", "marginBottom", "marginLeft"])
    {
        if let Some(side) = side {
            out.push(e(prop, margin_vals(side)?));
        }
    }
    Ok(out)
}

/// Lowers `"object-view-box"`.
fn lower_object_view_box(t: Vec<Tok>) -> Result<Vec<Part>, String> {
    // CSS's inset() of the texture, in percentages: the box's
    // u0 v0 u1 v1 (px need the texture's size, which the box lacks).
    let [Tok::Func(f, args)] = t.as_slice() else {
        return Err("object-view-box: none | inset(<t>% [<r>% [<b>% [<l>%]]])".into());
    };
    if f != "inset" {
        return Err(format!("object-view-box takes inset(…), not {f}(…)"));
    }
    let pct: Vec<f64> = args
        .iter()
        .flatten()
        .map(|v| match v {
            Tok::Num(n, u) if u == "%" => n
                .parse::<f64>()
                .map(|n| n / 100.0)
                .map_err(|_| format!("'{n}%' is not a number")),
            Tok::Num(n, _) if n.parse::<f64>() == Ok(0.0) => Ok(0.0),
            v => Err(format!(
                "object-view-box: inset() takes percentages of the image, not '{}'",
                describe(v)
            )),
        })
        .collect::<Result<_, _>>()?;
    let [top, right, bottom, left] = match pct[..] {
        [a] => [a; 4],
        [v, h] => [v, h, v, h],
        [top, h, b] => [top, h, b, h],
        [top, r, b, l] => [top, r, b, l],
        _ => return Err("inset() takes 1 to 4 percentages".into()),
    };
    if left + right >= 1.0 || top + bottom >= 1.0 {
        return Err("object-view-box: inset() leaves none of the image".into());
    }
    Ok(vec![e(
        "uv",
        [left, top, 1.0 - right, 1.0 - bottom]
            .into_iter()
            .map(|v| {
                let s = format!("{v:.6}");
                number(s.trim_end_matches('0').trim_end_matches('.'))
            })
            .collect::<Result<_, _>>()?,
    )])
}

/// Lowers `"flex-wrap"`.
fn lower_flex_wrap(name: &str, t: Vec<Tok>) -> Result<Vec<Part>, String> {
    let (wrap, reverse) = match want_ident(&one(&t, name)?, "nowrap, wrap or wrap-reverse")? {
        "nowrap" => (false, false),
        "wrap" => (true, false),
        "wrap-reverse" => (true, true),
        w => {
            return Err(format!(
                "'{w}' is not a flex-wrap: nowrap, wrap, wrap-reverse"
            ));
        }
    };
    Ok(vec![
        e("wrap", vec![bool_word(wrap)]),
        e("wrapReverse", vec![bool_word(reverse)]),
    ])
}

/// Lowers `"flex-flow"`.
fn lower_flex_flow(t: Vec<Tok>) -> Result<Vec<Part>, String> {
    // <flex-direction> || <flex-wrap> (CSS Flexbox §5.3).
    let mut out = Vec::new();
    for v in &t {
        let w = want_ident(v, "a direction or a wrap")?;
        match w {
            "row" | "row-reverse" | "column" | "column-reverse" => {
                out.extend(lower_decl("flex-direction", w, false)?)
            }
            "nowrap" | "wrap" | "wrap-reverse" => out.extend(lower_decl("flex-wrap", w, false)?),
            w => return Err(format!("'{w}' is not a flex-flow: a direction and a wrap")),
        }
    }
    Ok(out)
}

/// Lowers `"flex"`.
fn lower_flex(t: Vec<Tok>) -> Result<Vec<Part>, String> {
    // none | auto | <grow> [<shrink>] || <basis> (CSS Flexbox §7.1);
    // a number alone has a 0 basis, and `auto` a basis of auto.
    let num = |v: &Tok| matches!(v, Tok::Num(_, u) if u.is_empty());
    let (grow, shrink, basis): (Value, Value, Option<&Tok>) = match t.as_slice() {
        [Tok::Ident(w)] if w == "none" => (number("0")?, number("0")?, None),
        [Tok::Ident(w)] if w == "auto" => (number("1")?, number("1")?, None),
        [Tok::Ident(w)] if w == "initial" => (number("0")?, number("1")?, None),
        [g] if num(g) => (
            want_num(g)?,
            number("1")?,
            Some(&Tok::Num("0".into(), "px".into())),
        ),
        [g, s] if num(g) && num(s) => (
            want_num(g)?,
            want_num(s)?,
            Some(&Tok::Num("0".into(), "px".into())),
        ),
        [g, b] if num(g) => (want_num(g)?, number("1")?, Some(b)),
        [b, g] if num(g) => (want_num(g)?, number("1")?, Some(b)),
        [g, s, b] if num(g) && num(s) => (want_num(g)?, want_num(s)?, Some(b)),
        [b, g, s] if num(g) && num(s) => (want_num(g)?, want_num(s)?, Some(b)),
        [b] => (number("1")?, number("1")?, Some(b)),
        _ => return Err("flex: none | auto | <grow> [<shrink>] [<basis>]".into()),
    };
    let mut out = vec![e("flexGrow", vec![grow]), e("flexShrink", vec![shrink])];
    if let Some(b) = basis {
        out.extend(lower_decl("flex-basis", &css_text(b), false)?);
    }
    Ok(out)
}

/// Lowers `"place-content" | "place-items" | "place-self"`.
fn lower_place(name: &str, t: Vec<Tok>) -> Result<Vec<Part>, String> {
    // <align> [<justify>]: one value sets both (CSS Box Alignment §5.3, §6.3, §6.4).
    let (align, justify) = match name {
        "place-content" => ("align-content", "justify-content"),
        "place-items" => ("align-items", "justify-items"),
        _ => ("align-self", "justify-self"),
    };
    let split = t
        .iter()
        .enumerate()
        .skip(1)
        .find(|(k, v)| {
            !matches!(t[k - 1], Tok::Ident(ref w) if w == "safe" || w == "unsafe" || w == "first" || w == "last")
                && matches!(v, Tok::Ident(_))
        })
        .map(|(k, _)| k)
        .unwrap_or(t.len());
    let (a, j) = t.split_at(split);
    let j = if j.is_empty() { a } else { j };
    let mut out = alignment(align, a)?;
    out.extend(alignment(justify, j)?);
    Ok(out)
}

/// Lowers `"grid-auto-flow"`.
fn lower_grid_auto_flow(t: Vec<Tok>) -> Result<Vec<Part>, String> {
    let mut out = Vec::new();
    for v in &t {
        match want_ident(v, "row, column or dense")? {
            "dense" => out.push(e("dense", vec![Value::Word("true".into())])),
            f @ ("row" | "column") => out.push(e("flow", vec![Value::Word(f.into())])),
            f => return Err(format!("'{f}' is not a grid flow: row, column, dense")),
        }
    }
    Ok(out)
}

/// Lowers `_`.
fn lower_other(name: &str, value: &str, t: Vec<Tok>) -> Result<Vec<Part>, String> {
    let Some(prop) = STYLE.iter().find(|(_, c)| *c == name).map(|(p, _)| *p) else {
        return Err(match unsupported(name) {
            Some(how) => format!("'{name}' is not supported: {how}"),
            None => format!("'{name}' is not a property{}", hint_for(name)),
        });
    };
    let vals = match conv(prop) {
        Conv::Len => vec![want_len(&one(&t, name)?)?],
        Conv::Num => vec![want_num(&one(&t, name)?)?],
        Conv::Angle => vec![want_angle(&one(&t, name)?)?],
        Conv::Paint => paint_vals(&t)?,
        Conv::Bool(yes, no) => match want_ident(&one(&t, name)?, "a word")?.to_string().as_str() {
            w if w == yes => vec![Value::Word("true".into())],
            w if w == no => vec![Value::Word("false".into())],
            w => return Err(format!("{name} is {yes} or {no}, not '{w}'")),
        },
        Conv::Kebab => vec![Value::Word(camel(want_ident(&one(&t, name)?, "a word")?))],
        Conv::Kw(table) => {
            let v = one(&t, name)?;
            let w = want_ident(&v, "a word")?;
            let el = table
                .iter()
                .find(|(_, c)| *c == w)
                .map(|(e, _)| *e)
                .ok_or_else(|| {
                    format!(
                        "'{w}' is not a {name}: {}",
                        table.iter().map(|(_, c)| *c).collect::<Vec<_>>().join(", ")
                    )
                })?;
            vec![Value::Word(el.into())]
        }
        Conv::Var(ns) => vec![Value::Word(want_var(&one(&t, name)?, ns)?)],
        Conv::Ident => match one(&t, name)? {
            Tok::Str(s) => vec![Value::String(s)],
            Tok::Ident(s) => vec![Value::Word(s)],
            v => return Err(format!("{name} takes a name, not '{}'", describe(&v))),
        },
        Conv::Strs => t
            .iter()
            .map(|v| match v {
                Tok::Str(s) => Ok(Value::String(s.clone())),
                v => Err(format!(
                    "{name} takes quoted strings, not '{}'",
                    describe(v)
                )),
            })
            .collect::<Result<_, _>>()?,
        Conv::Tracks => {
            super::tracks::parse(value)?;
            vec![Value::String(value.trim().to_string())]
        }
        Conv::Transition => transition_vals(&t)?,
        Conv::Origin => match t.as_slice() {
            [Tok::Num(a, u), Tok::Num(b, v)] if u == "%" && v == "%" => {
                vec![number(&shift(a, -2))?, number(&shift(b, -2))?]
            }
            _ => return Err("transform-origin: <x>% <y>%".into()),
        },
        Conv::Margin => margin_vals(&one(&t, name)?)?,
        Conv::Size | Conv::Insets | Conv::Joint | Conv::Raw => {
            unreachable!("handled above")
        }
    };
    Ok(vec![e(prop, vals)])
}

fn bool_word(b: bool) -> Value {
    Value::Word(if b { "true" } else { "false" }.into())
}

/// A token as CSS text (a lowered shorthand hands its part on).
fn css_text(t: &Tok) -> String {
    match t {
        Tok::Func(f, args) => format!(
            "{f}({})",
            args.iter()
                .map(|a| a.iter().map(css_text).collect::<Vec<_>>().join(" "))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        t => describe(t),
    }
}

/// A CSS Box Alignment value as its prop's word (and its `safe` flag):
/// `[safe | unsafe] <position>`, a distribution, `normal`, `stretch` or a
/// baseline. `auto` on an item is the initial value (its parent's).
fn alignment(name: &str, t: &[Tok]) -> Result<Vec<Part>, String> {
    let words: Vec<&str> = t
        .iter()
        .map(|v| want_ident(v, "an alignment"))
        .collect::<Result<_, _>>()?;
    let (safe, words) = match words.as_slice() {
        ["safe", rest @ ..] => (true, rest),
        ["unsafe", rest @ ..] => (false, rest),
        w => (false, w),
    };
    let content = name.ends_with("-content");
    let justify = name.starts_with("justify");
    let word = match (words, content) {
        (["auto"], _) if name.ends_with("-self") => return Ok(Vec::new()),
        (["normal"], true) if justify => "normal",
        (["normal"], true) => "stretch",
        (["normal" | "legacy"], false) => "stretch",
        (["stretch"], _) => "stretch",
        (["flex-start"], true) => "flexStart",
        (["flex-end"], true) => "flexEnd",
        (["start" | "left"], _) => "start",
        (["end" | "right"], _) => "end",
        (["flex-start" | "self-start"], false) => "start",
        (["flex-end" | "self-end"], false) => "end",
        (["center"], _) => "center",
        (["space-between"], true) => "spaceBetween",
        (["space-around"], true) => "spaceAround",
        (["space-evenly"], true) => "spaceEvenly",
        (["baseline"] | ["first", "baseline"], false) => "baseline",
        (["last", "baseline"], false) => "lastBaseline",
        _ => {
            return Err(format!("'{}' is not a {name} value", words.join(" ")));
        }
    };
    let prop = match name {
        "justify-content" => "align",
        "align-items" => "cross",
        "align-content" => "alignContent",
        "justify-items" => "justifyItems",
        "align-self" => "alignSelf",
        _ => "justifySelf",
    };
    // align-content takes no flex-relative words: its lines' start is
    // the flex start.
    let word = match (prop, word) {
        ("alignContent", "flexStart") => "start",
        ("alignContent", "flexEnd") => "end",
        (_, w) => w,
    };
    let mut out = vec![e(prop, vec![Value::Word(word.into())])];
    match name {
        "justify-content" => out.push(e("safeJustify", vec![bool_word(safe)])),
        "align-items" | "align-self" => out.push(e("safeAlign", vec![bool_word(safe)])),
        _ => {}
    }
    Ok(out)
}

/// What a property name was meant to be, when it is a near miss.
fn hint_for(name: &str) -> String {
    let c = camel(name);
    if let Some(css) = css_name(&c).or(css_name(name)) {
        return format!(" (write `{css}`)");
    }
    if super::super::schema::prop(&c).is_some() || super::super::schema::prop(name).is_some() {
        return format!(" ({c} is an attribute: `{c} <value>`, without a colon)");
    }
    let near: Vec<&str> = STYLE
        .iter()
        .map(|(_, c)| *c)
        .filter(|c| c.contains(name) || name.contains(c.trim_start_matches("-og-")))
        .collect();
    if near.is_empty() {
        " (the properties: docs/ui.md §3.2)".into()
    } else {
        format!(" (did you mean {}?)", near.join(", "))
    }
}

pub(super) enum SizeBit {
    Kind(Vec<Value>),
    /// `min-` (true) or `max-`, a size's values.
    Range(bool, Vec<Value>),
}

pub(super) enum Part {
    Entry(String, Vec<Value>),
    Size(usize, SizeBit),
    /// An axis, its start, its span.
    Grid(usize, Option<Value>, Option<Value>),
}

/// A size being joined: its entry's index in the block and its kind (once
/// one is declared), its `min-` and its `max-`, and the line of the first.
pub(super) type SizeParts = (
    Option<(usize, Vec<Value>)>,
    Option<Vec<Value>>,
    Option<Vec<Value>>,
    usize,
);
