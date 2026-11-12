use super::*;

/// A `<use>`'s own placement: `translate(x y)`, then a symbol's view box
/// fitted into its width and height (`xMidYMid meet`, or `none`).
pub(super) fn use_transform(r: &Resolved) -> Result<Affine, String> {
    let num = |p: &str| -> Result<f32, String> {
        r.str(p)
            .map_or(Ok(0.0), |v| crate::svg::length(v, 0.0))
            .map_err(|e| format!("{p}: {e}"))
    };
    let at = Affine([1.0, 0.0, 0.0, 1.0, num("svgX")?, num("svgY")?]);
    let Some(vb) = r.str("svgViewBox").and_then(view_box) else {
        return Ok(at);
    };
    let (w, h) = (num("svgWidth")?, num("svgHeight")?);
    let (w, h) = (
        if w > 0.0 { w } else { vb[2] },
        if h > 0.0 { h } else { vb[3] },
    );
    Ok(at.then(&fit_box(
        vb,
        [0.0, 0.0, w, h],
        r.str("svgPreserveAspectRatio"),
    )))
}

/// The transform that fits view box `vb` into `into` by
/// `preserveAspectRatio` (`none`, else centred `meet`; `slice` covers).
pub(super) fn fit_box(vb: [f32; 4], into: [f32; 4], par: Option<&str>) -> Affine {
    let (sx, sy) = (into[2] / vb[2], into[3] / vb[3]);
    let par = par.unwrap_or("xMidYMid meet");
    let (sx, sy) = if par.trim() == "none" {
        (sx, sy)
    } else if par.contains("slice") {
        (sx.max(sy), sx.max(sy))
    } else {
        (sx.min(sy), sx.min(sy))
    };
    let tx = into[0] + (into[2] - vb[2] * sx) / 2.0 - vb[0] * sx;
    let ty = into[1] + (into[3] - vb[3] * sy) / 2.0 - vb[1] * sy;
    Affine([sx, 0.0, 0.0, sy, tx, ty])
}

fn invert(a: &Affine) -> Option<Affine> {
    let [a, b, c, d, e, f] = a.0;
    let det = a * d - b * c;
    (det.abs() > 1e-12).then(|| {
        let (ia, ib, ic, id) = (d / det, -b / det, -c / det, a / det);
        Affine([ia, ib, ic, id, -(ia * e + ic * f), -(ib * e + id * f)])
    })
}

/// The definition `#id` of `doc` whose tag is one of `tags`.
pub(super) fn def<'d>(doc: &'d UiDocument, id: &str, tags: &[&str]) -> Option<&'d DocNode> {
    doc.nodes
        .iter()
        .find(|n| n.id == id && n.selector_tag.as_deref().is_some_and(|t| tags.contains(&t)))
}

/// The id a `url(#id)` names.
pub(super) fn url_id(v: &str) -> Option<&str> {
    v.trim()
        .strip_prefix("url(")?
        .strip_suffix(')')?
        .trim()
        .strip_prefix('#')
}

/// A definition's shape: its tag, literal attributes and transform.
type DefShape = (String, Vec<(String, String)>, Affine);

/// The shapes of a definition (its children, and its groups' children),
/// each its tag, literal attributes and transform in the definition's
/// space.
pub fn def_shapes(
    doc: &UiDocument,
    node: &DocNode,
    t: Affine,
    out: &mut Vec<DefShape>,
) -> Result<(), String> {
    for &c in &node.children {
        let n = &doc.nodes[c];
        let Some(tag) = n.selector_tag.as_deref() else {
            continue;
        };
        let attrs = literal_attrs(n)?;
        let own = match attrs.iter().find(|(k, _)| k == "transform") {
            Some((_, v)) => t.then(&Affine::parse(v)?),
            None => t,
        };
        if tag == "g" {
            def_shapes(doc, n, own, out)?;
        } else if crate::doc::extern_element::SVG_SHAPES.contains(&tag) {
            out.push((tag.to_string(), attrs, own));
        }
    }
    Ok(())
}

/// A rect, circle or ellipse under an axis-aligned transform as a box
/// shape and its view rect (what og-render clips to natively); `None`
/// for any other shape or a turned one.
pub(super) fn basic_box<'a>(
    tag: &str,
    get: &dyn Fn(&str) -> Option<&'a str>,
    local: &crate::svg::Path,
    t: &Affine,
) -> Option<(ShapeKind, [f32; 4])> {
    let [a, b, c, d, ..] = t.0;
    if b != 0.0 || c != 0.0 || !matches!(tag, "rect" | "circle" | "ellipse") {
        return None;
    }
    let [x, y, w, h] = bounds(local);
    let p = t.apply(Vec2::new(x, y));
    let q = t.apply(Vec2::new(x + w, y + h));
    let rect = [
        p.x.min(q.x),
        p.y.min(q.y),
        (q.x - p.x).abs(),
        (q.y - p.y).abs(),
    ];
    let shape = if tag == "rect" {
        let rx = get("rx")
            .or(get("ry"))
            .and_then(|v| crate::svg::length(v, w).ok());
        let r = rx.unwrap_or(0.0).min(w / 2.0).min(h / 2.0) * a.abs().min(d.abs());
        ShapeKind::Rect {
            radii: Some([r; 4]),
            corner: crate::paint::Corner::Round,
        }
    } else {
        ShapeKind::Ellipse
    };
    Some((shape, rect))
}

/// A clip's or mask's shapes as one path under `t`: its rule (`clip-rule`
/// of a clip's shapes, `fill-rule` of a mask's, when every shape takes it,
/// else nonzero) and its subpaths. The shapes are one path, so two
/// evenodd shapes that overlap cut a hole where they do; a mask's black or
/// unfilled shapes reveal nothing.
#[allow(clippy::type_complexity)]
pub(super) fn def_path(
    shapes: &[DefShape],
    mask: bool,
    id: &str,
    vb: [f32; 4],
    t: &Affine,
) -> Result<(crate::paint::FillRule, Vec<(Vec<[f32; 2]>, bool)>), String> {
    let tags = [if mask { "mask" } else { "clipPath" }];
    let rule_attr = if tags[0] == "mask" {
        "fill-rule"
    } else {
        "clip-rule"
    };
    let rules: Vec<bool> = shapes
        .iter()
        .map(|(_, attrs, _)| {
            match attrs
                .iter()
                .find(|(k, _)| k == rule_attr)
                .map(|(_, v)| v.trim())
            {
                None | Some("nonzero") => Ok(false),
                Some("evenodd") => Ok(true),
                Some(v) => Err(format!("{rule_attr} \"{v}\" is nonzero or evenodd")),
            }
        })
        .collect::<Result<_, _>>()?;
    if rules.iter().any(|&e| e) && !rules.iter().all(|&e| e) {
        return Err(format!(
            "#{id}: its shapes mix nonzero and evenodd {rule_attr}; give them one rule"
        ));
    }
    let rule = if rules.first() == Some(&true) {
        crate::paint::FillRule::EvenOdd
    } else {
        crate::paint::FillRule::NonZero
    };
    let mut parts = Vec::new();
    for (tag, attrs, own) in shapes {
        let get = |a: &str| attrs.iter().find(|(k, _)| k == a).map(|(_, v)| v.as_str());
        // A mask's black or unfilled shapes reveal nothing.
        if tags[0] == "mask"
            && get("fill").is_some_and(|f| {
                f == "none" || crate::svg::color(f).is_ok_and(|c| c[0] + c[1] + c[2] < 0.05)
            })
        {
            continue;
        }
        if let Some(local) = shape_path(tag, &get, vb)? {
            parts.extend(subpaths(&local.transformed(&t.then(own))));
        }
    }
    Ok((rule, parts))
}

/// CSS `clip-path: url(#id)` on an element: the `<clipPath>`'s shapes as
/// one path in the reference box's user space (CSS px from its top left),
/// its rule, and whether they are fractions of the box
/// (`clipPathUnits="objectBoundingBox"`).
#[allow(clippy::type_complexity)]
pub(crate) fn css_clip(
    doc: &UiDocument,
    reference: &str,
) -> Result<(crate::paint::FillRule, Vec<(Vec<[f32; 2]>, bool)>, bool), String> {
    let id = url_id(reference).ok_or_else(|| format!("\"{reference}\" is not url(#id)"))?;
    let node =
        def(doc, id, &["clipPath"]).ok_or_else(|| format!("url(#{id}) names no <clipPath>"))?;
    let attrs = literal_attrs(node)?;
    let units = attrs
        .iter()
        .any(|(k, v)| k == "clipPathUnits" && v == "objectBoundingBox");
    let mut shapes = Vec::new();
    def_shapes(doc, node, Affine::IDENTITY, &mut shapes)?;
    // A percentage is of the box: of 1 in its units, else of 100 px.
    let vb = if units {
        [0.0, 0.0, 1.0, 1.0]
    } else {
        [0.0, 0.0, 100.0, 100.0]
    };
    let (rule, parts) = def_path(&shapes, false, id, vb, &Affine::IDENTITY)?;
    Ok((rule, parts, units))
}

/// A `<filter>` and its descendants as the filter reader takes them.
fn filter_node<'d>(
    doc: &'d UiDocument,
    n: &'d DocNode,
) -> Result<crate::svg::filter::Node<'d>, String> {
    Ok(crate::svg::filter::Node {
        tag: n.selector_tag.as_deref().unwrap_or_default(),
        attrs: literal_attrs(n)?,
        children: n
            .children
            .iter()
            .map(|&c| filter_node(doc, &doc.nodes[c]))
            .collect::<Result<_, _>>()?,
    })
}

/// The `<filter>` `reference` (`url(#id)`) names; a user-unit percentage
/// is of `viewport`.
pub fn filter_def(
    doc: &UiDocument,
    reference: &str,
    viewport: [f32; 2],
    image: crate::svg::filter::FindImage,
) -> Result<crate::svg::filter::Filter, String> {
    let id = url_id(reference).ok_or_else(|| format!("\"{reference}\" is not url(#id)"))?;
    let node = def(doc, id, &["filter"]).ok_or_else(|| format!("url(#{id}) names no <filter>"))?;
    crate::svg::filter::parse(&filter_node(doc, node)?, viewport, image)
        .map_err(|e| format!("#{id}: {e}"))
}

/// CSS `filter: url(#id)` or `backdrop-filter: url(#id)` on a box: the
/// `<filter>` in the box's user space (CSS px from its border box's top
/// left; a percentage of user units is of 100 px).
pub(crate) fn css_filter(
    doc: &UiDocument,
    reference: &str,
    image: crate::svg::filter::FindImage,
) -> Result<std::sync::Arc<crate::paint::FilterUse>, String> {
    let vb = [0.0, 0.0, 100.0, 100.0];
    let filter = filter_def(doc, reference, [vb[2], vb[3]], image)?;
    Ok(std::sync::Arc::new(crate::paint::FilterUse {
        filter,
        svg: None,
    }))
}

/// A pattern's tiles under a shape of local bounds `bbox` and transform
/// `t`: for each of its shapes, all its tiles as one filled path, with
/// that shape's paint. At most 4096 tiles.
pub(super) fn pattern(
    doc: &UiDocument,
    id: &str,
    bbox: [f32; 4],
    t: &Affine,
    vb: [f32; 4],
    alpha: f32,
) -> Result<Vec<(ShapeKind, Paint)>, String> {
    let node = def(doc, id, &["pattern"]).ok_or_else(|| format!("url(#{id}) names nothing"))?;
    let attrs = literal_attrs(node)?;
    let get = |a: &str| attrs.iter().find(|(k, _)| k == a).map(|(_, v)| v.as_str());
    let bbox_units = get("patternUnits") != Some("userSpaceOnUse");
    let len = |a: &str, whole: f32, offset: f32| -> Result<f32, String> {
        let Some(v) = get(a) else { return Ok(0.0) };
        Ok(if bbox_units {
            let f = match v.trim().strip_suffix('%') {
                Some(p) => p.trim().parse::<f32>().map_err(|e| e.to_string())? / 100.0,
                None => v.trim().parse::<f32>().map_err(|e| e.to_string())?,
            };
            offset + f * whole
        } else {
            crate::svg::length(v, whole)?
        })
    };
    let tile = [
        len("x", bbox[2], bbox[0])?,
        len("y", bbox[3], bbox[1])?,
        len("width", bbox[2], 0.0)?,
        len("height", bbox[3], 0.0)?,
    ];
    if tile[2] <= 0.0 || tile[3] <= 0.0 {
        return Ok(Vec::new());
    }
    if get("patternContentUnits") == Some("objectBoundingBox") {
        return Err(format!(
            "#{id}: patternContentUnits objectBoundingBox is not supported"
        ));
    }
    let pt = match get("patternTransform") {
        Some(v) => Affine::parse(v)?,
        None => Affine::IDENTITY,
    };
    let content = match get("viewBox").and_then(view_box) {
        Some(pvb) => fit_box(
            pvb,
            [0.0, 0.0, tile[2], tile[3]],
            get("preserveAspectRatio"),
        ),
        None => Affine::IDENTITY,
    };
    // The shape's bounds in pattern space, and the tiles over them.
    let inv = invert(&pt).ok_or("patternTransform is not invertible")?;
    let corners = [
        [bbox[0], bbox[1]],
        [bbox[0] + bbox[2], bbox[1]],
        [bbox[0], bbox[1] + bbox[3]],
        [bbox[0] + bbox[2], bbox[1] + bbox[3]],
    ]
    .map(|[x, y]| inv.apply(Vec2::new(x, y)));
    let lo = corners.iter().fold(Vec2::splat(f32::MAX), |a, c| a.min(*c));
    let hi = corners.iter().fold(Vec2::splat(f32::MIN), |a, c| a.max(*c));
    let i0 = ((lo.x - tile[0]) / tile[2]).floor() as i64;
    let i1 = ((hi.x - tile[0]) / tile[2]).ceil() as i64;
    let j0 = ((lo.y - tile[1]) / tile[3]).floor() as i64;
    let j1 = ((hi.y - tile[1]) / tile[3]).ceil() as i64;
    if (i1 - i0) * (j1 - j0) > 4096 {
        return Err(format!(
            "#{id}: more than 4096 tiles; make the pattern larger"
        ));
    }
    let mut shapes = Vec::new();
    def_shapes(doc, node, Affine::IDENTITY, &mut shapes)?;
    let mut out = Vec::new();
    for (tag, attrs, own) in &shapes {
        let get = |a: &str| attrs.iter().find(|(k, _)| k == a).map(|(_, v)| v.as_str());
        let mut style = Style::default();
        for attr in PRESENTATION {
            if let Some(v) = get(attr) {
                let parent = style.clone();
                style
                    .apply(attr, v, &parent)
                    .map_err(|e| format!("{attr}: {e}"))?;
            }
        }
        let Some(paint) = solid(&style.fill, style.fill_opacity * alpha) else {
            continue;
        };
        let Some(local) = shape_path(tag, &get, vb)? else {
            continue;
        };
        let mut parts = Vec::new();
        for i in i0..i1 {
            for j in j0..j1 {
                let at = Affine([
                    1.0,
                    0.0,
                    0.0,
                    1.0,
                    tile[0] + i as f32 * tile[2],
                    tile[1] + j as f32 * tile[3],
                ]);
                let to = t.then(&pt).then(&at).then(&content).then(own);
                parts.extend(subpaths(&local.transformed(&to)));
            }
        }
        out.push((filled(parts, crate::paint::FillRule::NonZero), paint));
    }
    Ok(out)
}

/// A solid paint (`None` for none, and for a reference).
pub fn solid(p: &PaintRef, alpha: f32) -> Option<Paint> {
    match p {
        PaintRef::Color(c) => Some(Paint::Solid(ColorRef::Literal(srgba(*c, alpha)))),
        PaintRef::Current => Some(Paint::Solid(ColorRef::Token {
            name: "ink".into(),
            alpha,
        })),
        PaintRef::Token(name) => Some(Paint::Solid(ColorRef::Token {
            name: crate::doc::css::camel(name),
            alpha,
        })),
        PaintRef::None | PaintRef::Url(_) => None,
    }
}

pub(crate) fn srgba(c: [f32; 4], alpha: f32) -> Srgba {
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    Srgba {
        rgb: [byte(c[0]), byte(c[1]), byte(c[2])],
        alpha: (c[3] * alpha).clamp(0.0, 1.0),
    }
}

/// A gradient node's literal attributes by their SVG names.
pub(super) fn literal_attrs(n: &DocNode) -> Result<Vec<(String, String)>, String> {
    n.selector_attrs
        .iter()
        .filter(|(name, _)| name != "class" && !name.starts_with("class:"))
        .map(|(name, e)| match e {
            crate::bind::Expr::Lit(v) => Ok((name.clone(), crate::bind::text(v))),
            _ => Err(format!(
                "<{}> {name}: a gradient's attributes are literal",
                n.selector_tag.as_deref().unwrap_or("gradient")
            )),
        })
        .collect()
}

/// The gradient `#id` of `doc`, placed for a shape of local bounds `bbox`
/// under `t`, in view-box units.
pub(super) fn gradient(
    doc: &UiDocument,
    id: &str,
    bbox: [f32; 4],
    t: &Affine,
    alpha: f32,
) -> Result<Gradient, String> {
    let find = |id: &str| {
        doc.nodes.iter().find(|n| {
            n.id == id
                && matches!(
                    n.selector_tag.as_deref(),
                    Some("linearGradient" | "radialGradient")
                )
        })
    };
    let first = find(id).ok_or_else(|| format!("url(#{id}) names no gradient"))?;
    let mut chain = vec![first];
    let mut attrs = vec![literal_attrs(first)?];
    while let Some(href) = attrs
        .last()
        .and_then(|a| a.iter().find(|(k, _)| k == "href" || k == "xlink:href"))
        .map(|(_, v)| v.clone())
    {
        let next = href
            .strip_prefix('#')
            .and_then(find)
            .ok_or_else(|| format!("gradient \"{id}\": href \"{href}\" names no gradient"))?;
        if chain.len() > 16 || chain.iter().any(|c| std::ptr::eq(*c, next)) {
            return Err(format!("gradient \"{id}\": an href loop"));
        }
        chain.push(next);
        attrs.push(literal_attrs(next)?);
    }
    let stops: Vec<Vec<(String, String)>> =
        chain
            .iter()
            .find(|n| !n.children.is_empty())
            .map_or(Ok(Vec::new()), |n| {
                n.children
                    .iter()
                    .map(|&c| literal_attrs(&doc.nodes[c]))
                    .collect()
            })?;
    let get = |k: &str| {
        attrs
            .iter()
            .find_map(|a| a.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str()))
    };
    let stop_gets: Vec<_> = stops
        .iter()
        .map(|s| move |k: &str| s.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str()))
        .collect();
    let radial = first.selector_tag.as_deref() == Some("radialGradient");
    let placed = GradientDef::new(id, radial, &get, &stop_gets)?.place(bbox, t, alpha)?;
    Ok(Gradient {
        kind: if placed.radial {
            GradientKind::Radial
        } else {
            GradientKind::Linear
        },
        from: placed.from.into(),
        to: placed.to.into(),
        stops: placed
            .stops
            .into_iter()
            .map(|(offset, c)| Stop {
                offset,
                color: ColorRef::Literal(srgba(c, 1.0)),
            })
            .collect(),
        spread: placed.spread,
        dither: true,
    })
}

/// A path's subpaths as `ShapeKind::Path` points: each its start, then
/// three points per cubic.
pub(crate) fn subpaths(p: &crate::svg::Path) -> Vec<(Vec<[f32; 2]>, bool)> {
    p.subpaths
        .iter()
        .filter(|(_, cubics, _)| !cubics.is_empty())
        .map(|(start, cubics, closed)| {
            let mut pts: Vec<[f32; 2]> = vec![(*start).into()];
            pts.extend(cubics.iter().flatten().map(|v: &Vec2| <[f32; 2]>::from(*v)));
            (pts, *closed)
        })
        .collect()
}
