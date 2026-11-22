//! The generator operations: a solid from an operation's fields.

use super::*;

/// The generator's solid, with UVs (`crate::generators::solids::textured`) when
/// `uvs`.
pub(crate) fn generate(a: &Args, uvs: bool) -> Result<crate::mesh::Solid, ModelError> {
    use crate::generators::solids as g;
    use crate::generators::solids::Generator as G;
    let segments = || a.u32_or("segments", 0);
    // Either the generator alone or `textured` of it.
    macro_rules! pick {
        ($gen:expr, $plain:expr) => {
            if uvs { g::textured(&$gen)? } else { $plain? }
        };
    }
    Ok(match a.op {
        "box" => {
            let d = BoxDesc {
                width: a.num("width")?,
                depth: a.num("depth")?,
                height: a.num("height")?,
            };
            pick!(G::Box(d), g::cuboid(&d))
        }
        "cylinder" => {
            let d = CylinderDesc {
                radius: a.num("radius")?,
                radius_top: a.num_or("radiusTop", 0.0)?,
                height: a.num("height")?,
                segments: segments()?,
            };
            pick!(G::Cylinder(d), g::cylinder(&d))
        }
        "cone" => {
            let d = ConeDesc {
                radius: a.num("radius")?,
                height: a.num("height")?,
                segments: segments()?,
            };
            pick!(G::Cone(d), g::cone_desc(&d))
        }
        "sphere" => {
            let d = SphereDesc {
                radius: a.num("radius")?,
                axis: a.axis()?,
                segments: segments()?,
            };
            pick!(G::Sphere(d), g::sphere_desc(&d))
        }
        "dome" => {
            let d = DomeDesc {
                radius: a.num("radius")?,
                height: a.num_or("height", 0.0)?,
                axis: a.axis()?,
                segments: segments()?,
            };
            pick!(G::Dome(d), g::dome_desc(&d))
        }
        "tube" => {
            let d = TubeDesc {
                outer: a.num("outer")?,
                inner: a.num("inner")?,
                height: a.num("height")?,
                segments: segments()?,
            };
            pick!(G::Tube(d), g::tube_desc(&d))
        }
        "torus" => {
            let d = TorusDesc {
                radius: a.num("radius")?,
                tube_radius: a.num("tube")?,
                axis: a.axis()?,
                angle: a.num_or("angle", 0.0)?,
                segments: segments()?,
                tube_segments: a.u32_or("tubeSegments", 0)?,
            };
            pick!(G::Torus(d), g::torus_desc(&d))
        }
        "wedge" => {
            let (w, h, d) = (a.num("width")?, a.num("height")?, a.num("depth")?);
            pick!(G::Wedge(w, h, d), g::wedge(w, h, d))
        }
        "extrude" => {
            let p = a.profile("profile")?;
            let d = Extrude {
                height: a.num("height")?,
                twist: a.num_or("twist", 0.0)?,
                taper: a.num_or("taper", 0.0)?,
                both_sides: a.bool_or("bothSides", false)?,
            };
            pick!(G::Extrude(&p, d), g::extrude(&p, &d))
        }
        "revolve" => {
            let p = a.profile("profile")?;
            let d = Revolve {
                angle: a.num_or("angle", 0.0)?,
                axis: a.axis()?,
                segments: segments()?,
            };
            pick!(G::Revolve(&p, d), g::revolve(&p, &d))
        }
        "sweep" => {
            let p = a.profile("profile")?;
            let route = a.route("path")?;
            let d = Sweep {
                path: route.points,
                tangents: route.tangents,
                up: match a.opt("up") {
                    Some(_) => a.vec3("up")?,
                    None => DVec3::ZERO,
                },
                twist: a.num_or("twist", 0.0)?,
                scale_end: a.num_or("scaleEnd", 0.0)?,
            };
            pick!(G::Sweep(&p, &d), crate::generators::paths::sweep(&p, &d))
        }
        "loft" => {
            let list = a
                .get("sections")?
                .as_array()
                .ok_or_else(|| a.bad("sections", "a list of {profile, z}"))?;
            let mut sections = Vec::new();
            let mut placed = Vec::new();
            for (i, sec) in list.iter().enumerate() {
                let within = format!("sections[{i}]");
                let o = sec
                    .as_object()
                    .ok_or_else(|| a.bad(&within, "{profile, z} or {profile, at}"))?;
                only(o, &within, &["profile", "z", "at"])?;
                let p = o
                    .get("profile")
                    .ok_or_else(|| a.bad(&format!("{within}.profile"), "missing"))?;
                let p =
                    profile(p, a.fonts).map_err(|e| a.inside(&format!("{within}.profile"), e))?;
                match (o.get("z"), o.get("at")) {
                    (Some(z), None) => {
                        let z = z
                            .as_f64()
                            .ok_or_else(|| a.bad(&format!("{within}.z"), "a number"))?;
                        sections.push((p, z));
                    }
                    (None, Some(at)) => {
                        let f = frame_of(at).map_err(|w| a.bad(&format!("{within}.at"), &w))?;
                        placed.push((p, f));
                    }
                    _ => {
                        return Err(a.bad(&within, "one of z (a height) or at (a frame), not both"));
                    }
                }
            }
            let closed = a.bool_or("closed", false)?;
            if !placed.is_empty() {
                // Sections placed in space: every one by its frame.
                if !sections.is_empty() {
                    return Err(a.bad("sections", "every section by z, or every section by at"));
                }
                let straight = match a.opt("curve").map(|v| v.as_str()) {
                    None | Some(Some("catmullRom")) => false,
                    Some(Some("straight")) => true,
                    Some(_) => {
                        return Err(a.bad(
                            "curve",
                            "\"straight\" or \"catmullRom\" for sections placed by at",
                        ));
                    }
                };
                return crate::model::checked::loft_along_with(
                    &LoftAlong {
                        sections: placed,
                        samples: a.u32_or("samples", 0)?,
                        steps: a.u32_or("steps", 0)?,
                        straight,
                        closed,
                    },
                    uvs,
                );
            }
            if closed {
                return Err(a.bad(
                    "closed",
                    "a loop needs its sections placed by at (frames round the loop)",
                ));
            }
            let curve = match a.opt("curve").map(|v| v.as_str()) {
                None => LoftCurve::Auto,
                Some(Some("auto")) => LoftCurve::Auto,
                Some(Some("straight")) => LoftCurve::Straight,
                Some(Some("catmullRom")) => LoftCurve::CatmullRom,
                Some(Some("spline")) => LoftCurve::Spline,
                Some(_) => {
                    return Err(a.bad(
                        "curve",
                        "\"auto\", \"straight\", \"catmullRom\" or \"spline\"",
                    ));
                }
            };
            crate::model::checked::loft_with(
                &Loft {
                    sections,
                    samples: a.u32_or("samples", 0)?,
                    steps: a.u32_or("steps", 0)?,
                    curve,
                },
                uvs,
            )?
        }
        "thread" => {
            let d = Thread {
                radius: a.num("radius")?,
                pitch: a.num("pitch")?,
                length: a.num("length")?,
                depth: a.num("depth")?,
            };
            pick!(G::Thread(d), crate::generators::paths::thread(&d))
        }
        "wire" => {
            let route = a.route("path")?;
            let fillet = a.num_or("fillet", 0.0)?;
            let curved = route.corner.iter().any(|c| !c);
            let fillets = match a.opt("fillets") {
                Some(_) if curved => {
                    return Err(a.bad(
                        "fillets",
                        "one radius a point, so a path of points only; with arcs, splines or a \
                         helix give `fillet`, which bends the corners between pieces",
                    ));
                }
                Some(_) => a.nums("fillets")?,
                // Inside a curve the path is already round: no bend there.
                None if curved && fillet != 0.0 => route
                    .corner
                    .iter()
                    .map(|&c| if c { fillet } else { 0.0 })
                    .collect(),
                None => Vec::new(),
            };
            let d = Wire {
                path: route.points,
                diameter: a.num("diameter")?,
                closed: a.bool_or("closed", false)?,
                fillet: if fillets.is_empty() { fillet } else { 0.0 },
                fillets,
                segments: segments()?,
                tangents: route.tangents,
            };
            pick!(G::Wire(&d), crate::generators::wire::wire(&d))
        }
        "body" => {
            let d = Body {
                plan: a.view("plan")?,
                top: a.view("top")?,
                bottom: a.view("bottom")?,
                n: match a.opt("n") {
                    Some(_) => a.view("n")?,
                    None => View::default(),
                },
            };
            pick!(G::Body(&d), crate::generators::body::body(&d))
        }
        "knurl" => {
            let d = Knurl {
                diameter: a.num("diameter")?,
                height: a.num("height")?,
                pitch: a.num_or("pitch", 0.0)?,
                straight: a.bool_or("straight", false)?,
            };
            pick!(G::Knurl(d), crate::generators::parts::knurl(&d))
        }
        "gear" => {
            let d = GearDesc {
                teeth: a.u32_or("teeth", 0)?,
                module: a.num("module")?,
                pressure_angle: a.num_or("pressureAngle", 0.0)?,
                width: a.num("width")?,
                bore: a.num_or("bore", 0.0)?,
                hub: a.num_or("hub", 0.0)?,
                hub_height: a.num_or("hubHeight", 0.0)?,
            };
            pick!(G::Gear(d), crate::generators::parts::gear_solid(&d))
        }
        _ => return Err(ModelError::Program(format!("no generator {:?}", a.op))),
    })
}

/// An extrusion up to a face of a part (`until: {part, face}`): its
/// height runs from the profile's plane to the face's plane along the
/// extrusion's +z. A face parallel to the profile gives a flat top at
/// that height; a tilted one, a top cut along its plane (named `top`,
/// its edges `top` too). `on` is the extrusion's frame. Refused: `height`
/// with it, or `twist`, `taper`, `bothSides`; a face that is not flat; a
/// plane the extrusion would meet at or below its profile.
pub(crate) fn extrude_until(
    m: &Model,
    a: &Args,
    on: &Frame,
    uvs: bool,
) -> Result<crate::mesh::Solid, ModelError> {
    for k in ["height", "twist", "taper", "bothSides"] {
        if a.opt(k).is_some() {
            return Err(a.bad(k, "not with `until`, which sets the height"));
        }
    }
    let u = a.get("until")?;
    let o = u
        .as_object()
        .ok_or_else(|| a.bad("until", "{part, face}"))?;
    only(o, "until", &["part", "face"])?;
    let part = o
        .get("part")
        .and_then(id_of)
        .ok_or_else(|| a.bad("until.part", "a part's id"))?;
    let face = o
        .get("face")
        .and_then(Value::as_str)
        .ok_or_else(|| a.bad("until.face", "a face name"))?;
    let plane = on.inverse().compose(&m.face(part, face)?);
    let (q, n) = (plane.origin, plane.z);
    let prof = a.profile("profile")?;
    if n.z == 0.0 {
        return Err(a.bad(
            "until",
            "a face the extrusion meets: this one is parallel to the extrusion's direction",
        ));
    }
    // Where each profile point's line along +z meets the plane.
    let t = |p: DVec2| (q - DVec3::new(p.x, p.y, 0.0)).dot(n) / n.z;
    let heights: Vec<f64> = prof
        .contours()
        .flat_map(|c| &c.points)
        .map(|&p| t(p))
        .collect();
    let lo = heights.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = heights.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if !(lo > 0.0 && hi.is_finite()) {
        return Err(a.bad(
            "until",
            &format!(
                "a face whose plane lies above the whole profile; it meets the extrusion {lo:.4} \
                 from the profile's plane"
            ),
        ));
    }
    let flat = n.x == 0.0 && n.y == 0.0;
    let height = if flat { q.z } else { hi + hi.max(1.0) };
    let d = crate::generators::solids::Extrude {
        height,
        ..Default::default()
    };
    // With UVs, the tilted top (the split's cut) is planar in its own
    // plane's x and y (`crate::edit::split`).
    let s = if uvs {
        crate::generators::solids::textured(&crate::generators::solids::Generator::Extrude(
            &prof, d,
        ))?
    } else {
        crate::generators::solids::extrude(&prof, &d)?
    };
    if flat {
        return Ok(s);
    }
    // Keep the side of the plane the profile is on.
    let cut = Frame::facing(q, n).ok_or_else(|| a.bad("until", "a face with a normal"))?;
    let p0 = prof.regions[0].outer.points[0];
    let keep = if (DVec3::new(p0.x, p0.y, 0.0) - q).dot(n) < 0.0 {
        crate::edit::Keep::Below
    } else {
        crate::edit::Keep::Above
    };
    let mut s = crate::edit::split(&s, &cut, keep)?;
    let top = s.names.intern("top");
    let made: Vec<u32> = s
        .names
        .all()
        .filter(|x| x.starts_with("split."))
        .filter_map(|x| s.names.get(x))
        .collect();
    for tags in &mut s.face_tags {
        if tags.iter().any(|x| made.contains(x)) && !tags.contains(&top) {
            let tags = std::sync::Arc::make_mut(tags);
            tags.push(top);
            tags.sort_unstable();
        }
    }
    if let Some(c) = s.names.get("cut") {
        for tags in s.edge_tags.values_mut() {
            if tags.contains(&c) && !tags.contains(&top) {
                tags.push(top);
                tags.sort_unstable();
            }
        }
    }
    s.canonicalize()
        .map_err(|_| ModelError::BadValue("the extrusion's positions".into()))?;
    Ok(s)
}
