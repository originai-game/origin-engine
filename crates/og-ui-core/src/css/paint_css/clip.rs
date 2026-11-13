use super::*;

fn rule(t: &[Tok]) -> (FillRule, &[Tok]) {
    match t.split_first() {
        Some((Tok::Ident(w), rest)) if w == "evenodd" => (FillRule::EvenOdd, rest),
        Some((Tok::Ident(w), rest)) if w == "nonzero" => (FillRule::NonZero, rest),
        _ => (FillRule::NonZero, t),
    }
}

pub(crate) fn shape_margin(text: &str) -> Result<Pos, String> {
    let t = tokens(text)?;
    let [value] = t.as_slice() else {
        return Err("shape-margin takes one non-negative length or percentage".into());
    };
    let value = pos(value)?;
    if value.at(1.0) < 0.0 {
        return Err("shape-margin cannot be negative".into());
    }
    Ok(value)
}

/// A `shape-outside` value: a basic shape over its reference box, or an
/// image whose alpha channel gives the contour.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum OutsideShape {
    Shape(CssClip, GeoBox),
    /// `url(<file>)`: the texture asset's path.
    Image(String),
}

/// Basic-shape float exclusion uses the margin box by default, not the
/// border box used by clip-path. SVG geometry boxes are not shape boxes.
pub(crate) fn outside(text: &str) -> Result<Option<OutsideShape>, String> {
    let t = tokens(text)?;
    if t.iter().any(|t| matches!(t, Tok::Ident(s) if matches!(s.as_str(), "fill-box" | "stroke-box" | "view-box"))) {
        return Err("shape-outside takes margin-box, border-box, padding-box or content-box".into());
    }
    if let Some(Tok::Func(f, args)) = t.first()
        && f == "url"
    {
        let [_] = t.as_slice() else {
            return Err("shape-outside: an image takes no shape box".into());
        };
        let path = url(args).map_err(|_| "shape-outside: url() names an image file")?;
        if path.is_empty() || path.starts_with('#') {
            return Err("shape-outside: url() names an image file".into());
        }
        return Ok(Some(OutsideShape::Image(path)));
    }
    let mut value = clip(text)?;
    if let Some((_, geo)) = &mut value
        && !t
            .iter()
            .any(|t| matches!(t, Tok::Ident(s) if GeoBox::parse(s).is_some()))
    {
        *geo = GeoBox::Margin;
    }
    Ok(value.map(|(shape, geo)| OutsideShape::Shape(shape, geo)))
}

/// `shape-image-threshold`: a number or percentage, clamped to 0-1.
pub(crate) fn shape_threshold(text: &str) -> Result<f32, String> {
    const FORM: &str = "shape-image-threshold takes a number or percentage";
    let t = tokens(text)?;
    let [value] = t.as_slice() else {
        return Err(FORM.into());
    };
    let (v, unit) = number(value)?;
    match unit {
        "" => Ok(v.clamp(0.0, 1.0)),
        "%" => Ok((v / 100.0).clamp(0.0, 1.0)),
        _ => Err(FORM.into()),
    }
}

/// `clip-path`: `[<basic-shape> || <geometry-box>] | url(#id) | none`;
/// the basic shapes `inset()`, `rect()`, `xywh()`, `circle()`,
/// `ellipse()`, `polygon()`, `path()` and `shape()`.
pub(crate) fn clip(text: &str) -> Result<Option<(CssClip, GeoBox)>, String> {
    const FORMS: &str = "a clip-path is [inset() | rect() | xywh() | circle() | ellipse() | polygon() | path() | shape()] [<geometry-box>], a <geometry-box>, url(#id) or none";
    let t = tokens(text)?;
    let mut shape = None;
    let mut geo = None;
    for tok in &t {
        match tok {
            Tok::Ident(w) if w == "none" && t.len() == 1 => return Ok(None),
            Tok::Ident(w) if geo.is_none() => {
                geo = Some(GeoBox::parse(w).ok_or_else(|| format!("'{w}': {FORMS}"))?);
            }
            Tok::Func(f, a) if shape.is_none() => shape = Some((f.as_str(), a)),
            _ => return Err(FORMS.into()),
        }
    }
    let geo = geo.unwrap_or_default();
    let Some((f, args)) = shape else {
        return Ok(Some((CssClip::Box, geo)));
    };
    if f == "url" {
        if t.len() > 1 {
            return Err("clip-path: url(#id) takes no box".into());
        }
        let id = match args.as_slice() {
            [a] => match a.as_slice() {
                [Tok::Hash(h)] => h.clone(),
                [Tok::Str(s)] if s.starts_with('#') => s.clone(),
                _ => return Err("clip-path: url(#id) names a <clipPath>".into()),
            },
            _ => return Err("clip-path: url(#id) names a <clipPath>".into()),
        };
        return Ok(Some((CssClip::Url(format!("url({id})")), geo)));
    }
    let split_at = |a: &[Tok]| -> Result<(Vec<Tok>, [Pos; 2]), String> {
        match a
            .iter()
            .position(|t| matches!(t, Tok::Ident(w) if w == "at"))
        {
            Some(k) => Ok((a[..k].to_vec(), position(&a[k + 1..])?)),
            None => Ok((a.to_vec(), [Pos::Frac(0.5); 2])),
        }
    };
    // `… round <r>`: the lengths before it and the radius.
    let rounded = |a: &[Tok]| -> Result<(usize, f32), String> {
        let k = a
            .iter()
            .position(|t| matches!(t, Tok::Ident(w) if w == "round"))
            .unwrap_or(a.len());
        let radius = match &a[k..] {
            [] => 0.0,
            [_, r] => px(r)?,
            _ => return Err(format!("{f}(… round <n>px)")),
        };
        Ok((k, radius))
    };
    let shape = match (f, args.as_slice()) {
        ("inset", [a]) => {
            let (k, radius) = rounded(a)?;
            let p: Vec<Pos> = a[..k].iter().map(pos).collect::<Result<_, _>>()?;
            let sides = match p.as_slice() {
                [a] => [*a; 4],
                [a, b] => [*a, *b, *a, *b],
                [a, b, c] => [*a, *b, *c, *b],
                [a, b, c, d] => [*a, *b, *c, *d],
                _ => return Err("inset() takes one to four lengths".into()),
            };
            CssClip::Inset(sides, radius)
        }
        ("rect", [a]) => {
            let (k, radius) = rounded(a)?;
            let p: Vec<Option<Pos>> = a[..k]
                .iter()
                .map(|t| match t {
                    Tok::Ident(w) if w == "auto" => Ok(None),
                    t => pos(t).map(Some),
                })
                .collect::<Result<_, _>>()?;
            let [t, r, b, l] = p[..] else {
                return Err("rect(<top> <right> <bottom> <left> [round <r>])".into());
            };
            CssClip::Rect([t, r, b, l], radius)
        }
        ("xywh", [a]) => {
            let (k, radius) = rounded(a)?;
            let p: Vec<Pos> = a[..k].iter().map(pos).collect::<Result<_, _>>()?;
            let [x, y, w, h] = p[..] else {
                return Err("xywh(<x> <y> <width> <height> [round <r>])".into());
            };
            CssClip::Xywh([x, y, w, h], radius)
        }
        ("shape", items) => shape_fn(items)?,
        ("circle", [a]) => {
            let (r, at) = split_at(a)?;
            let r = match r.as_slice() {
                [] => None,
                [r] => Some(pos(r)?),
                _ => return Err("circle(<r> at <x> <y>)".into()),
            };
            CssClip::Ellipse([r, r], true, at)
        }
        ("ellipse", [a]) => {
            let (r, at) = split_at(a)?;
            match r.as_slice() {
                [] => CssClip::Ellipse([None; 2], false, at),
                [x, y] => CssClip::Ellipse([Some(pos(x)?), Some(pos(y)?)], false, at),
                _ => return Err("ellipse(<rx> <ry> at <x> <y>)".into()),
            }
        }
        ("polygon", points) => {
            let (fill, first) = rule(points.first().map_or(&[][..], |p| p.as_slice()));
            let skip = first.is_empty() as usize;
            let mut out = Vec::new();
            for (k, p) in points.iter().enumerate().skip(skip) {
                let p = if k == 0 { first } else { p.as_slice() };
                match p {
                    [x, y] => out.push([pos(x)?, pos(y)?]),
                    _ => return Err("polygon([<rule>,] <x> <y>, …)".into()),
                }
            }
            if out.len() < 3 {
                return Err("a polygon takes at least three points".into());
            }
            CssClip::Polygon(fill, out)
        }
        ("path", args) => {
            let (fill, d) = match args {
                [d] => (FillRule::NonZero, d.as_slice()),
                [r, d] => (rule(r).0, d.as_slice()),
                _ => return Err("path([<rule>,] \"<svg path data>\")".into()),
            };
            let [Tok::Str(d)] = d else {
                return Err("path(\"<svg path data>\")".into());
            };
            let p = crate::svg::Path::parse(d)?;
            CssClip::Path(fill, crate::svg::subpaths(&p))
        }
        (f, _) => return Err(format!("{f}() is not a clip-path shape")),
    };
    Ok(Some((shape, geo)))
}

/// `shape([<rule>,] from <x> <y>, <command>, …)` (CSS Shapes 2 §3.2.1) as
/// SVG path commands: `move`, `line`, `hline`, `vline`, `curve … with
/// <c1> [/ <c2>]`, `smooth … [with <c>]`, `arc … of <rx> [<ry>] [cw | ccw]
/// [large | small] [rotate <angle>]`, each `to` or `by`, and `close`.
fn shape_fn(items: &[Vec<Tok>]) -> Result<CssClip, String> {
    const FORM: &str = "shape([<rule>,] from <x> <y>, <command>, …)";
    let (fill, first) = rule(items.first().map_or(&[][..], |p| p.as_slice()));
    let skip = first.is_empty() as usize;
    let mut cmds: Vec<(char, Vec<Pos>)> = Vec::new();
    for (k, item) in items.iter().enumerate().skip(skip) {
        let item = if k == 0 { first } else { item.as_slice() };
        let word = |i: usize| match item.get(i) {
            Some(Tok::Ident(w)) => Some(w.as_str()),
            _ => None,
        };
        // The command letter, lower case for `by`.
        let rel = |upper: char| -> Result<char, String> {
            match word(1) {
                Some("to") => Ok(upper),
                Some("by") => Ok(upper.to_ascii_lowercase()),
                _ => Err(format!(
                    "{FORM}: '{}' takes to or by",
                    word(0).unwrap_or("")
                )),
            }
        };
        let pair = |i: usize| -> Result<Vec<Pos>, String> {
            match (item.get(i), item.get(i + 1)) {
                (Some(x), Some(y)) => Ok(vec![pos(x)?, pos(y)?]),
                _ => Err(format!("{FORM}: a point is <x> <y>")),
            }
        };
        let cmd = match word(0) {
            Some("from") if cmds.is_empty() => ('M', pair(1)?),
            _ if cmds.is_empty() => return Err(format!("{FORM}: it starts with from")),
            Some("close") => ('Z', Vec::new()),
            Some("move") => (rel('M')?, pair(2)?),
            Some("line") => (rel('L')?, pair(2)?),
            Some("hline") => (rel('H')?, vec![pos(item.get(2).ok_or(FORM)?)?]),
            Some("vline") => (rel('V')?, vec![pos(item.get(2).ok_or(FORM)?)?]),
            Some(c @ ("curve" | "smooth")) => {
                let end = pair(2)?;
                let with = item
                    .iter()
                    .position(|t| matches!(t, Tok::Ident(w) if w == "with"));
                let controls: Vec<Pos> = match with {
                    None => Vec::new(),
                    Some(w) => item[w + 1..]
                        .iter()
                        .filter(|t| !matches!(t, Tok::Slash))
                        .map(pos)
                        .collect::<Result<_, _>>()?,
                };
                let letter = match (c, controls.len()) {
                    ("curve", 2) => 'Q',
                    ("curve", 4) => 'C',
                    ("smooth", 0) => 'T',
                    ("smooth", 2) => 'S',
                    _ => return Err(format!("{FORM}: {c} … with <x> <y> [/ <x> <y>]")),
                };
                let letter = rel(letter)?;
                (letter, [controls, end].concat())
            }
            Some("arc") => {
                let end = pair(2)?;
                let of = item
                    .iter()
                    .position(|t| matches!(t, Tok::Ident(w) if w == "of"))
                    .ok_or_else(|| format!("{FORM}: arc … of <radius>"))?;
                let mut radii = Vec::new();
                let (mut sweep, mut large, mut rotate) = (0.0, 0.0, 0.0);
                let mut i = of + 1;
                while i < item.len() {
                    match &item[i] {
                        Tok::Ident(w) if w == "cw" => sweep = 1.0,
                        Tok::Ident(w) if w == "ccw" => sweep = 0.0,
                        Tok::Ident(w) if w == "large" => large = 1.0,
                        Tok::Ident(w) if w == "small" => large = 0.0,
                        Tok::Ident(w) if w == "rotate" => {
                            i += 1;
                            rotate = angle(item.get(i).ok_or(FORM)?)?.to_degrees();
                        }
                        t => radii.push(pos(t)?),
                    }
                    i += 1;
                }
                let (rx, ry) = match radii[..] {
                    [r] => (r, r),
                    [rx, ry] => (rx, ry),
                    _ => return Err(format!("{FORM}: arc … of <rx> [<ry>]")),
                };
                let letter = rel('A')?;
                (
                    letter,
                    vec![
                        rx,
                        ry,
                        Pos::Px(rotate),
                        Pos::Px(large),
                        Pos::Px(sweep),
                        end[0],
                        end[1],
                    ],
                )
            }
            _ => {
                return Err(format!(
                    "{FORM}: '{}' is not a command",
                    item.first().map(describe).unwrap_or_default()
                ));
            }
        };
        cmds.push(cmd);
    }
    if cmds.len() < 2 {
        return Err(format!("{FORM}: it takes a command after from"));
    }
    Ok(CssClip::Shape(fill, cmds))
}
