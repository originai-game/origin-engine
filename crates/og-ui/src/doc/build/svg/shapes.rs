use super::*;

fn cap_of(c: crate::svg::Cap) -> crate::paint::Cap {
    match c {
        crate::svg::Cap::Butt => crate::paint::Cap::Butt,
        crate::svg::Cap::Round => crate::paint::Cap::Round,
        crate::svg::Cap::Square => crate::paint::Cap::Square,
    }
}

/// A path (in the `<svg>`'s user space) filled with `fill` and stroked
/// with `stroke` by `style`: `scale` is its transform's (the stroke's
/// width scales by it), `per` the user units of one dash unit. A dash
/// pattern of one dash and one gap is og-render's; a longer one is cut
/// into its dashes here.
#[allow(clippy::too_many_arguments)]
pub(super) fn painted(
    path: &crate::svg::Path,
    style: &Style,
    scale: f32,
    per: f32,
    fill: Option<Paint>,
    stroke: Option<Paint>,
    vb: [f32; 4],
) -> ElementDraw {
    let parts = subpaths(path);
    let rule = if style.even_odd {
        crate::paint::FillRule::EvenOdd
    } else {
        crate::paint::FillRule::NonZero
    };
    let cap = cap_of(style.cap);
    let join = match style.join {
        crate::svg::Join::Miter => crate::paint::Join::Miter(style.miter_limit),
        crate::svg::Join::Round => crate::paint::Join::Round,
        crate::svg::Join::Bevel => crate::paint::Join::Bevel,
    };
    let width = style.stroke_width * scale;
    let mut draw = ElementDraw {
        view_box: Some(vb),
        shape: ShapeKind::Path {
            points: parts.iter().flat_map(|(p, _)| p.iter().copied()).collect(),
            closed: true,
            filled: true,
            curves: true,
            fill_rule: rule,
            contours: parts.iter().map(|(p, _)| p.len() as u32).collect(),
            width,
            cap,
            join,
        },
        ..ElementDraw::default()
    };
    if let Some(fill) = fill {
        draw.fills.push(fill);
    }
    let Some(stroke) = stroke.filter(|_| width > 0.0) else {
        return draw;
    };
    let outline =
        |points: Vec<[f32; 2]>, closed: bool, curves: bool, dash: [f32; 2], offset: f32| {
            (
                ShapeKind::Path {
                    points,
                    closed,
                    filled: false,
                    curves,
                    fill_rule: rule,
                    contours: Vec::new(),
                    width,
                    cap,
                    join,
                },
                Stroke {
                    width,
                    align: StrokeAlign::Center,
                    dash,
                    dash_offset: offset,
                    dash_cap: cap,
                    paint: stroke.clone(),
                },
            )
        };
    match style.dashes.as_slice() {
        [] => {
            for (points, closed) in parts {
                draw.outlines
                    .push(outline(points, closed, true, [0.0; 2], 0.0));
            }
        }
        // Zero-length dashes (an empty `pathLength` meter's "0 1"): butt
        // caps draw nothing; og-render reads a zero dash as solid.
        [d, g] if *d <= 0.0 && *g > 0.0 && style.cap == crate::svg::Cap::Butt => {}
        [d, g] if *d > 0.0 || *g <= 0.0 => {
            for (points, closed) in parts {
                draw.outlines.push(outline(
                    points,
                    closed,
                    true,
                    [d * per, g * per],
                    style.dash_offset * per,
                ));
            }
        }
        more => {
            // Every dash its own open polyline, its caps the stroke's.
            let pattern: Vec<f32> = more.iter().map(|d| d * per).collect();
            let tolerance = vb[2].max(vb[3]) / 4000.0;
            for dash in path.dashed(&pattern, style.dash_offset * per, tolerance) {
                let points = dash.into_iter().map(<[f32; 2]>::from).collect();
                draw.outlines
                    .push(outline(points, false, false, [0.0; 2], 0.0));
            }
        }
    }
    draw
}

/// A presentation value as SVG writes it, a marker's `context-fill` and
/// `context-stroke` (SVG 2) taken from the shape that draws it.
fn style_of(attrs: &[(String, String)], context: Option<&Style>) -> Result<Style, String> {
    let mut style = Style::default();
    for attr in PRESENTATION {
        let Some((_, v)) = attrs.iter().find(|(k, _)| k == attr) else {
            continue;
        };
        let parent = style.clone();
        match (v.trim(), context) {
            ("context-fill", Some(c)) => style.apply(attr, "none", &parent).map(|_| {
                if *attr == "fill" {
                    style.fill = c.fill.clone();
                } else {
                    style.stroke = c.fill.clone();
                }
            }),
            ("context-stroke", Some(c)) => style.apply(attr, "none", &parent).map(|_| {
                if *attr == "fill" {
                    style.fill = c.stroke.clone();
                } else {
                    style.stroke = c.stroke.clone();
                }
            }),
            (v, _) => style.apply(attr, v, &parent),
        }
        .map_err(|e| format!("{attr}: {e}"))?;
    }
    Ok(style)
}

/// Which vertices a marker property draws at.
#[derive(Clone, Copy, PartialEq)]
enum Which {
    Start,
    Mid,
    End,
}

/// The draws of a shape's `marker-start`, `marker-mid` and `marker-end`:
/// each `<marker>`'s shapes at the vertices, turned by its `orient`, its
/// `refX refY` on the vertex, scaled by the stroke's width (markerUnits
/// strokeWidth) and its view box fitted into `markerWidth markerHeight`.
pub(super) fn markers(
    doc: &UiDocument,
    value: &dyn Fn(&str) -> Option<String>,
    local: &crate::svg::Path,
    t: &Affine,
    style: &Style,
    vb: [f32; 4],
) -> Result<Vec<ElementDraw>, String> {
    let verts = local.vertices();
    let n = verts.len();
    let mut out = Vec::new();
    for (attr, which) in [
        ("marker-start", Which::Start),
        ("marker-mid", Which::Mid),
        ("marker-end", Which::End),
    ] {
        let Some(v) = value(attr).filter(|v| v.trim() != "none") else {
            continue;
        };
        let id = url_id(&v).ok_or_else(|| format!("{attr} \"{v}\" is url(#id) or none"))?;
        let node = def(doc, id, &["marker"])
            .ok_or_else(|| format!("{attr}: url(#{id}) names no <marker>"))?;
        let attrs = literal_attrs(node)?;
        let get = |a: &str| attrs.iter().find(|(k, _)| k == a).map(|(_, v)| v.as_str());
        let num = |a: &str, default: f32| -> Result<f32, String> {
            get(a)
                .map_or(Ok(default), |v| crate::svg::length(v, 1.0))
                .map_err(|e| format!("#{id} {a}: {e}"))
        };
        let (mw, mh) = (num("markerWidth", 3.0)?, num("markerHeight", 3.0)?);
        if mw <= 0.0 || mh <= 0.0 {
            continue;
        }
        let scale = match get("markerUnits").map(str::trim) {
            None | Some("strokeWidth") => style.stroke_width,
            Some("userSpaceOnUse") => 1.0,
            Some(u) => {
                return Err(format!(
                    "#{id} markerUnits \"{u}\" is strokeWidth or userSpaceOnUse"
                ));
            }
        };
        let content = match get("viewBox").and_then(view_box) {
            Some(mvb) => fit_box(mvb, [0.0, 0.0, mw, mh], get("preserveAspectRatio")),
            None => Affine::IDENTITY,
        };
        let refp = content.apply(Vec2::new(num("refX", 0.0)?, num("refY", 0.0)?));
        let orient = get("orient").unwrap_or("0").trim();
        let fixed = match orient {
            "auto" | "auto-start-reverse" => None,
            o => Some(
                o.trim_end_matches("deg")
                    .trim()
                    .parse::<f32>()
                    .map_err(|_| {
                        format!("#{id} orient \"{o}\" is auto, auto-start-reverse or degrees")
                    })?
                    .to_radians(),
            ),
        };
        let mut shapes = Vec::new();
        def_shapes(doc, node, Affine::IDENTITY, &mut shapes)?;
        let at: Vec<usize> = match which {
            Which::Start => (n > 0).then_some(0).into_iter().collect(),
            Which::End => n.checked_sub(1).into_iter().collect(),
            Which::Mid => (1..n.saturating_sub(1)).collect(),
        };
        for i in at {
            let (p, came, goes) = verts[i];
            let angle = fixed.unwrap_or_else(|| {
                let d = match (came, goes) {
                    (Some(a), Some(b)) => (a + b).try_normalize().unwrap_or(a),
                    (Some(a), None) => a,
                    (None, Some(b)) => b,
                    (None, None) => Vec2::X,
                };
                let a = d.y.atan2(d.x);
                if orient == "auto-start-reverse" && which == Which::Start {
                    a + std::f32::consts::PI
                } else {
                    a
                }
            });
            let (sin, cos) = angle.sin_cos();
            let m = t
                .then(&Affine([1.0, 0.0, 0.0, 1.0, p.x, p.y]))
                .then(&Affine([cos, sin, -sin, cos, 0.0, 0.0]))
                .then(&Affine([scale, 0.0, 0.0, scale, 0.0, 0.0]))
                .then(&Affine([1.0, 0.0, 0.0, 1.0, -refp.x, -refp.y]))
                .then(&content);
            for (tag, attrs, own) in &shapes {
                let get = |a: &str| attrs.iter().find(|(k, _)| k == a).map(|(_, v)| v.as_str());
                let Some(shape) = shape_path(tag, &get, vb)? else {
                    continue;
                };
                let s = style_of(attrs, Some(style))?;
                let at = m.then(own);
                let fill = if *tag == "line" {
                    None
                } else {
                    solid(&s.fill, s.fill_opacity)
                };
                let stroke = solid(&s.stroke, s.stroke_opacity);
                out.push(painted(
                    &shape.transformed(&at),
                    &s,
                    at.scale(),
                    at.scale(),
                    fill,
                    stroke,
                    vb,
                ));
            }
        }
    }
    Ok(out)
}

/// A `<text>`'s `<textPath>` as og-ui's text on a path: the path `href`
/// names (a `<path>` or any basic shape, its own transform applied) or its
/// own `path` data, under the text's transform, in the `<svg>`'s user
/// space from the view box's origin. `None`: the text is not on a path.
pub fn text_path(
    doc: &UiDocument,
    r: &Resolved,
    t: &Affine,
    vb: [f32; 4],
) -> Result<Option<crate::text_path::TextPath>, String> {
    let (local, length_scale) = if let Some(d) = r.str("svgPath") {
        (
            crate::svg::Path::parse(d).map_err(|e| format!("<textPath> path: {e}"))?,
            1.0,
        )
    } else if let Some(href) = r.str("svgTextPathHref") {
        shape_by_href(doc, href, vb).map_err(|e| format!("<textPath {e}"))?
    } else {
        return Ok(None);
    };
    let path = local.transformed(t);
    let tolerance = vb[2].max(vb[3]) / 4000.0;
    let origin = Vec2::new(vb[0], vb[1]);
    let subs: Vec<Vec<Vec2>> = path
        .flatten(tolerance)
        .into_iter()
        .map(|(mut pts, closed)| {
            if closed && let Some(&p) = pts.first() {
                pts.push(p);
            }
            pts.into_iter().map(|p| p - origin).collect()
        })
        .collect();
    let (offset, percent) = match r.str("svgStartOffset").map(str::trim) {
        None => (0.0, false),
        Some(v) => match v.strip_suffix('%') {
            Some(p) => (
                p.trim()
                    .parse::<f32>()
                    .map_err(|_| format!("startOffset \"{v}\""))?
                    / 100.0,
                true,
            ),
            None => (
                crate::svg::length(v, 0.0)? * length_scale * t.scale(),
                false,
            ),
        },
    };
    let reverse = match r.str("svgSide").map(str::trim) {
        None | Some("left") => false,
        Some("right") => true,
        Some(v) => return Err(format!("<textPath> side \"{v}\" is left or right")),
    };
    match r.str("svgMethod").map(str::trim) {
        None | Some("align" | "stretch") => {}
        Some(v) => return Err(format!("<textPath> method \"{v}\" is align or stretch")),
    }
    match r.str("svgSpacing").map(str::trim) {
        None | Some("exact" | "auto") => {}
        Some(v) => return Err(format!("<textPath> spacing \"{v}\" is exact or auto")),
    }
    Ok(Some(crate::text_path::TextPath {
        shape: crate::text_path::PathShape::Data(std::sync::Arc::new(subs)),
        offset,
        percent,
        reverse,
    }))
}
