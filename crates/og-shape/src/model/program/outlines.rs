//! Profiles and paths from JSON: 2D outlines, 3D paths and their pieces.

use super::*;

/// The fields each profile takes.
pub(crate) const PROFILES: &[(&str, &[&str])] = &[
    ("rect", &["width", "height", "radius"]),
    ("circle", &["radius", "segments"]),
    ("ellipse", &["rx", "ry", "segments"]),
    ("slot", &["length", "width"]),
    ("polygon", &["sides", "radius", "flats"]),
    ("poly", &["points"]),
    ("gear", &["teeth", "module", "pressureAngle"]),
    ("union", &[]),
    ("subtract", &[]),
    ("intersect", &[]),
    ("offset", &["profile", "distance"]),
    ("translate", &["profile", "x", "y"]),
    ("rotate", &["profile", "angle"]),
    ("scale", &["profile", "x", "y"]),
    ("superellipse", &["rx", "ry", "exponent", "segments"]),
    ("spline", &["points"]),
    ("path", &[]),
    ("stroke", &["path", "width", "join", "roundCaps", "closed"]),
    ("hull", &[]),
    ("mirror", &["profile", "axis"]),
    ("text", &["text", "font", "size", "spacing"]),
];

/// A path's pieces before they are built, in `D` dimensions.
pub(crate) enum RawPiece<P> {
    Line(P),
    Arc(P, P),
    TangentArc(P),
    Spline(Vec<P>),
}

/// What a path list holds, for the errors.
pub(crate) const PATH_WANT: &str = "a list: a start point, then points (straight to them), \
    {\"arc\": {\"through\", \"to\"}}, {\"tangentArc\": {\"to\"}} or \
    {\"spline\": {\"points\"}}";

/// A path list: its start and its pieces, points read by `point`
/// (`[x, y]` or `[x, y, z]`). Errors name `at[i]`.
pub(crate) fn raw_path<P>(
    v: &Value,
    at: &str,
    point: impl Fn(&Value) -> Option<P>,
    want: &str,
) -> Result<(P, Vec<RawPiece<P>>), ModelError> {
    let bad = |w: &str, what: &str| ModelError::BadField(w.into(), what.into());
    let l = v.as_array().ok_or_else(|| bad(at, PATH_WANT))?;
    let (first, rest) = l
        .split_first()
        .ok_or_else(|| bad(at, "at least a start point"))?;
    let start =
        point(first).ok_or_else(|| bad(&format!("{at}[0]"), &format!("the start, {want}")))?;
    let mut pieces = Vec::new();
    for (j, item) in rest.iter().enumerate() {
        let w = format!("{at}[{}]", j + 1);
        let pt = |o: &Map<String, Value>, k: &str| {
            o.get(k)
                .and_then(&point)
                .ok_or_else(|| bad(&format!("{w}.{k}"), want))
        };
        let piece = match item {
            Value::Array(_) => RawPiece::Line(point(item).ok_or_else(|| bad(&w, want))?),
            Value::Object(o) if o.len() == 1 => {
                let Some((k, body)) = o.iter().next() else {
                    return Err(bad(&w, PATH_WANT));
                };
                let fields: &[&str] = match k.as_str() {
                    "arc" => &["through", "to"],
                    "tangentArc" => &["to"],
                    "spline" => &["points"],
                    _ => return Err(bad(&w, PATH_WANT)),
                };
                let b = body
                    .as_object()
                    .ok_or_else(|| bad(&format!("{w}.{k}"), "an object"))?;
                only(b, &format!("{w}.{k}"), fields)?;
                let w = format!("{w}.{k}");
                match k.as_str() {
                    "arc" => RawPiece::Arc(pt(b, "through")?, pt(b, "to")?),
                    "tangentArc" => RawPiece::TangentArc(pt(b, "to")?),
                    _ => RawPiece::Spline(
                        b.get("points")
                            .and_then(Value::as_array)
                            .filter(|l| !l.is_empty())
                            .and_then(|l| l.iter().map(&point).collect::<Option<Vec<_>>>())
                            .ok_or_else(|| {
                                bad(&format!("{w}.points"), &format!("a list of {want}"))
                            })?,
                    ),
                }
            }
            _ => return Err(bad(&w, PATH_WANT)),
        };
        pieces.push(piece);
    }
    Ok((start, pieces))
}

/// A 2D path list as a [`crate::profile::Path`] (not closed). A three-point arc
/// on one line and a tangent arc with no direction before it are refused.
pub(crate) fn path2(v: &Value, at: &str) -> Result<crate::profile::Path, ModelError> {
    let (start, pieces) = raw_path(v, at, xy, "[x, y]")?;
    let mut p = crate::profile::Path::new(start.x, start.y);
    for (j, piece) in pieces.into_iter().enumerate() {
        let w = format!("{at}[{}]", j + 1);
        let last = *p.points().last().unwrap_or(&start);
        p = match piece {
            RawPiece::Line(q) => p.line_to(q.x, q.y),
            RawPiece::Arc(t, q) => {
                if crate::math::predicates::orient2d(last, t, q).is_eq() {
                    return Err(ModelError::BadField(
                        format!("{w}.arc"),
                        "three points not on one line (the arc's start, through, to)".into(),
                    ));
                }
                p.arc_to(t.x, t.y, q.x, q.y)
            }
            RawPiece::TangentArc(q) => {
                if p.heading().is_none() {
                    return Err(ModelError::BadField(
                        format!("{w}.tangentArc"),
                        "a piece before it, to leave tangent to".into(),
                    ));
                }
                p.tangent_arc_to(q.x, q.y)
            }
            RawPiece::Spline(l) => p.spline_through(&l),
        };
    }
    Ok(p)
}

/// A 3D path: a list of points and pieces (flattened by the curve rule),
/// or `{"helix": {"radius", "pitch", "turns"}}` about +z from
/// `(radius, 0, 0)`.
pub(crate) fn path3(v: &Value, at: &str) -> Result<Flat, ModelError> {
    if let Some(o) = v.as_object() {
        only(o, at, &["helix"])?;
        let h = o.get("helix").and_then(Value::as_object).ok_or_else(|| {
            ModelError::BadField(at.into(), "{\"helix\": {radius, pitch, turns}}".into())
        })?;
        only(h, &format!("{at}.helix"), &["radius", "pitch", "turns"])?;
        let n = |k: &str| {
            h.get(k)
                .and_then(Value::as_f64)
                .ok_or_else(|| ModelError::BadField(format!("{at}.helix.{k}"), "a number".into()))
        };
        return Ok(crate::generators::path3::helix(&Helix {
            radius: n("radius")?,
            pitch: n("pitch")?,
            turns: n("turns")?,
        })?);
    }
    let (start, pieces) = raw_path(v, at, vec3_of, "[x, y, z]")?;
    let pieces = pieces
        .into_iter()
        .map(|p| match p {
            RawPiece::Line(q) => Piece::Line(q),
            RawPiece::Arc(through, to) => Piece::Arc { through, to },
            RawPiece::TangentArc(q) => Piece::TangentArc(q),
            RawPiece::Spline(l) => Piece::Spline(l),
        })
        .collect();
    Ok(crate::generators::path3::flatten(&Path3 { start, pieces })?)
}

/// A profile from its JSON. Errors name the field inside the profile
/// (`rect.width`); the caller says where the profile is.
pub(crate) fn profile(v: &Value, fonts: &Fonts) -> Result<Profile, ModelError> {
    let kinds = || {
        PROFILES
            .iter()
            .map(|(k, _)| *k)
            .collect::<Vec<_>>()
            .join(", ")
    };
    let o = v.as_object().filter(|o| o.len() == 1).ok_or_else(|| {
        ModelError::BadField(
            String::new(),
            format!("an object with one key: {}", kinds()),
        )
    })?;
    let (k, v) = o
        .iter()
        .next()
        .ok_or_else(|| ModelError::BadField(String::new(), "empty".into()))?;
    let bad = |key: &str, want: &str| ModelError::BadField(format!("{k}.{key}"), want.into());
    let Some((_, fields)) = PROFILES.iter().find(|(name, _)| name == k) else {
        return Err(ModelError::BadField(
            String::new(),
            format!("no profile {k:?}; profiles: {}", kinds()),
        ));
    };
    if !fields.is_empty() {
        let fo = v
            .as_object()
            .ok_or_else(|| ModelError::BadField(k.clone(), "an object".into()))?;
        only(fo, k, fields)?;
    }
    let f = |key: &str| -> Result<f64, ModelError> {
        match v.get(key) {
            None | Some(Value::Null) => Ok(0.0),
            Some(x) => x.as_f64().ok_or_else(|| bad(key, "a number")),
        }
    };
    let u = |key: &str| -> Result<u32, ModelError> {
        match v.get(key) {
            None | Some(Value::Null) => Ok(0),
            Some(x) => x
                .as_u64()
                .or_else(|| {
                    x.as_f64()
                        .filter(|n| {
                            n.is_finite()
                                && *n >= 0.0
                                && n.fract() == 0.0
                                && *n <= f64::from(u32::MAX)
                        })
                        .map(|n| n as u64)
                })
                .and_then(|n| u32::try_from(n).ok())
                .ok_or_else(|| bad(key, "a whole number")),
        }
    };
    // A segment count: 0 for the chord rule, else at least `least`.
    let count = |key: &str, least: u32| -> Result<u32, ModelError> {
        let n = u(key)?;
        if n != 0 && n < least {
            return Err(bad(key, &format!("0 (the chord rule) or at least {least}")));
        }
        Ok(n)
    };
    let one = |f: f64| if f == 0.0 { 1.0 } else { f };
    let inner = |key: &str| -> Result<Profile, ModelError> {
        let p = v.get(key).ok_or_else(|| bad(key, "missing"))?;
        profile(p, fonts).map_err(|e| nest(&format!("{k}.{key}"), e))
    };
    let list = || {
        v.as_array()
            .filter(|l| !l.is_empty())
            .ok_or_else(|| ModelError::BadField(k.clone(), "a list of profiles".into()))
    };
    Ok(match k.as_str() {
        "rect" => crate::profile::rect(&crate::profile::Rect {
            width: f("width")?,
            height: f("height")?,
            radius: f("radius")?,
        })?,
        "circle" => crate::profile::circle(&crate::profile::Circle {
            radius: f("radius")?,
            segments: count("segments", 3)?,
        })?,
        "ellipse" => crate::profile::ellipse(&crate::profile::Ellipse {
            rx: f("rx")?,
            ry: f("ry")?,
            segments: count("segments", 3)?,
        })?,
        "superellipse" => crate::profile::superellipse(&crate::profile::Superellipse {
            rx: f("rx")?,
            ry: f("ry")?,
            exponent: f("exponent")?,
            segments: count("segments", 4)?,
        })?,
        "spline" => crate::profile::spline(&crate::profile::Spline {
            points: v
                .get("points")
                .and_then(Value::as_array)
                .and_then(|l| l.iter().map(xy).collect::<Option<Vec<_>>>())
                .ok_or_else(|| bad("points", "a list of [x, y]"))?,
        })?,
        "path" => path2(v, "path")?.close()?,
        "stroke" => {
            let p = path2(v.get("path").unwrap_or(&Value::Null), "stroke.path")?;
            let join = match v.get("join").map(Value::as_str) {
                None | Some(Some("round")) => crate::profile::Join::Round,
                Some(Some("miter")) => crate::profile::Join::Miter,
                Some(Some("bevel")) => crate::profile::Join::Bevel,
                Some(_) => return Err(bad("join", "\"round\", \"miter\" or \"bevel\"")),
            };
            let flag = |key: &str| match v.get(key) {
                None | Some(Value::Null) => Ok(false),
                Some(b) => b.as_bool().ok_or_else(|| bad(key, "true or false")),
            };
            crate::profile::stroke(
                p.points(),
                &crate::profile::StrokeDesc {
                    width: f("width")?,
                    join,
                    round_caps: flag("roundCaps")?,
                    closed: flag("closed")?,
                },
            )?
        }
        "hull" => {
            let l = list()?;
            let mut all = Profile::default();
            for (i, q) in l.iter().enumerate() {
                let q = profile(q, fonts).map_err(|e| nest(&format!("hull[{i}]"), e))?;
                all.regions.extend(q.regions);
            }
            all.hull()?
        }
        "mirror" => match v.get("axis").map(Value::as_str) {
            None | Some(Some("y")) => inner("profile")?.mirror(),
            Some(Some("x")) => inner("profile")?.scale(1.0, -1.0),
            Some(_) => {
                return Err(bad(
                    "axis",
                    "\"y\" (across the y axis: x becomes -x) or \"x\" (y becomes -y)",
                ));
            }
        },
        "text" => {
            let name = v
                .get("font")
                .and_then(Value::as_str)
                .ok_or_else(|| bad("font", "the name of a font the caller supplies"))?;
            let font = fonts.get(name).ok_or_else(|| {
                let have: Vec<&str> = fonts.keys().map(String::as_str).collect();
                bad(
                    "font",
                    &format!(
                        "a font supplied by the caller; {name:?} was not (supplied: {})",
                        if have.is_empty() {
                            "none".to_string()
                        } else {
                            have.join(", ")
                        }
                    ),
                )
            })?;
            crate::profile::text(&crate::profile::Text {
                text: v
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| bad("text", "a string"))?
                    .into(),
                font: font.clone(),
                size: f("size")?,
                spacing: f("spacing")?,
            })?
        }
        "slot" => crate::profile::slot(&crate::profile::Slot {
            length: f("length")?,
            width: f("width")?,
        })?,
        "polygon" => crate::profile::polygon(&crate::profile::Polygon {
            sides: u("sides")?,
            radius: f("radius")?,
            flats: f("flats")?,
        })?,
        "gear" => crate::profile::gear(&crate::profile::Gear {
            teeth: u("teeth")?,
            module: f("module")?,
            pressure_angle: f("pressureAngle")?,
        })?,
        "poly" => {
            let pts = v
                .get("points")
                .and_then(Value::as_array)
                .ok_or_else(|| bad("points", "a list of [x, y] or {x, y, radius, bulge}"))?;
            let mut points = Vec::new();
            for (i, p) in pts.iter().enumerate() {
                let want = "[x, y] or {x, y, radius?, bulge?}";
                let num = |o: &Map<String, Value>, key: &str| -> Result<f64, ModelError> {
                    match o.get(key) {
                        None | Some(Value::Null) => Ok(0.0),
                        Some(x) => x
                            .as_f64()
                            .ok_or_else(|| bad(&format!("points[{i}].{key}"), "a number")),
                    }
                };
                let point = match p {
                    Value::Array(xy) => match xy[..] {
                        [ref x, ref y] => crate::profile::PolyPoint {
                            x: x.as_f64()
                                .ok_or_else(|| bad(&format!("points[{i}]"), want))?,
                            y: y.as_f64()
                                .ok_or_else(|| bad(&format!("points[{i}]"), want))?,
                            ..Default::default()
                        },
                        _ => return Err(bad(&format!("points[{i}]"), want)),
                    },
                    Value::Object(o) => {
                        only(
                            o,
                            &format!("poly.points[{i}]"),
                            &["x", "y", "radius", "bulge"],
                        )?;
                        if !(o.contains_key("x") && o.contains_key("y")) {
                            return Err(bad(&format!("points[{i}]"), want));
                        }
                        crate::profile::PolyPoint {
                            x: num(o, "x")?,
                            y: num(o, "y")?,
                            radius: num(o, "radius")?,
                            bulge: num(o, "bulge")?,
                        }
                    }
                    _ => return Err(bad(&format!("points[{i}]"), want)),
                };
                points.push(point);
            }
            crate::profile::poly(&crate::profile::Poly { points })?
        }
        "union" | "subtract" | "intersect" => {
            let l = list()?;
            let item = |i: usize| profile(&l[i], fonts).map_err(|e| nest(&format!("{k}[{i}]"), e));
            let mut acc = item(0)?;
            for i in 1..l.len() {
                let q = item(i)?;
                acc = match k.as_str() {
                    "union" => acc.union(&q),
                    "subtract" => acc.subtract(&q),
                    _ => acc.intersect(&q),
                };
            }
            acc
        }
        "offset" => inner("profile")?.offset(f("distance")?, crate::profile::Join::Round)?,
        "translate" => inner("profile")?.translate(DVec2::new(f("x")?, f("y")?)),
        "rotate" => inner("profile")?.rotate(f("angle")?),
        "scale" => {
            let (x, y) = (one(f("x")?), one(f("y")?));
            if !(x.is_finite() && y.is_finite() && x > 0.0 && y > 0.0) {
                return Err(bad(
                    "x, y",
                    "positive factors (0 keeps 1); mirror with a negative is not taken",
                ));
            }
            inner("profile")?.scale(x, y)
        }
        _ => unreachable!("every profile kind is matched"),
    })
}

/// `e` from inside the field `at`: field names get `at.` in front.
pub(crate) fn nest(at: &str, e: ModelError) -> ModelError {
    match e {
        ModelError::BadField(f, w) if f.is_empty() => ModelError::BadField(at.into(), w),
        ModelError::BadField(f, w) => ModelError::BadField(format!("{at}.{f}"), w),
        ModelError::UnknownField {
            within,
            field,
            accepted,
        } => ModelError::UnknownField {
            within: format!("{at}: {within}"),
            field,
            accepted,
        },
        ModelError::Profile(p) => ModelError::BadField(at.into(), p.to_string()),
        e => e,
    }
}
