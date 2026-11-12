use super::*;

/// The widget memory holding when the element at `path` first showed:
/// its SMIL animations' clock starts there.
pub fn smil_clock(path: &str) -> u64 {
    crate::id::child_key(og_core::hash::hash_bytes(path.as_bytes()), "~smil")
}

/// Whether `n` holds SMIL animations.
pub fn animated(doc: &UiDocument, n: &DocNode) -> bool {
    n.children.iter().any(|&c| {
        doc.nodes[c]
            .selector_tag
            .as_deref()
            .is_some_and(|t| crate::svg::smil::TAGS.contains(&t))
    })
}

/// The path and transform of the shape `href="#id"` names, in its own
/// user space.
pub(super) fn shape_by_href(
    doc: &UiDocument,
    href: &str,
    vb: [f32; 4],
) -> Result<(crate::svg::Path, f32), String> {
    let id = href.trim().strip_prefix('#').unwrap_or(href.trim());
    let node = doc
        .nodes
        .iter()
        .find(|n| {
            n.id == id
                && n.selector_tag.as_deref().is_some_and(|t| {
                    crate::doc::extern_element::SVG_SHAPES.contains(&t) && t != "image"
                })
        })
        .ok_or_else(|| format!("href=\"{href}\": no path or shape has id \"{id}\""))?;
    let attrs = literal_attrs(node)?;
    let get = |a: &str| attrs.iter().find(|(k, _)| k == a).map(|(_, v)| v.as_str());
    let tag = node.selector_tag.as_deref().unwrap_or("path");
    let shape = shape_path(tag, &get, vb)?.ok_or_else(|| format!("#{id} is empty"))?;
    let own = match get("transform") {
        Some(v) => Affine::parse(v)?,
        None => Affine::IDENTITY,
    };
    // A length along it measures the path in its pathLength.
    let k = match get("pathLength").map(|v| v.trim().parse::<f32>()) {
        Some(Ok(l)) if l > 0.0 => shape.length(vb[2].max(vb[3]) / 2000.0) / l,
        _ => 1.0,
    };
    Ok((shape.transformed(&own), k))
}

/// An animation element's begins in seconds of its element's clock,
/// following `id.begin` and `id.end` to the animations they name.
fn begin_times(doc: &UiDocument, a: &crate::svg::smil::Animation, depth: u32) -> Vec<f32> {
    use crate::svg::smil::{Animation, Begin};
    let mut out = Vec::new();
    for b in &a.begins {
        match b {
            Begin::At(t) => out.push(*t),
            Begin::Sync { id, end, offset } if depth < 8 => {
                let Some(node) = doc.nodes.iter().find(|n| {
                    &n.id == id
                        && n.selector_tag
                            .as_deref()
                            .is_some_and(|t| crate::svg::smil::TAGS.contains(&t))
                }) else {
                    continue;
                };
                let tag = node.selector_tag.as_deref().unwrap_or("animate");
                let Some(other) = literal_attrs(node)
                    .ok()
                    .and_then(|attrs| Animation::parse(tag, &attrs, Some(Default::default())).ok())
                else {
                    continue;
                };
                let Some(first) = begin_times(doc, &other, depth + 1)
                    .into_iter()
                    .reduce(f32::min)
                else {
                    continue;
                };
                match (end, other.active()) {
                    (false, _) => out.push(first + offset),
                    (true, Some(d)) => out.push(first + d + offset),
                    (true, None) => {}
                }
            }
            Begin::Sync { .. } => {}
        }
    }
    out
}

/// Applies `n`'s SMIL animations (`<animate>`, `<set>`,
/// `<animateTransform>`, `<animateMotion>`) at the UI clock: each
/// overrides an attribute, a presentation property or the transform as
/// SVG's override style sheet does. Presentation only: the clock is the
/// element's widget memory and the values never leave the UI. Returns
/// whether any still moves (or waits to begin), so it resolves again
/// next frame. Under reduced motion each shows its frozen end, or none.
pub(in crate::doc::build) fn animate(
    ui: &Ui,
    doc: &UiDocument,
    n: &DocNode,
    path: &str,
    r: &mut Resolved,
) -> bool {
    use crate::svg::smil::{Animation, Target};
    if !animated(doc, n) {
        return false;
    }
    let vb = r
        .cascade
        .computed
        .get(VIEW_BOX)
        .cloned()
        .flatten()
        .and_then(|v| view_box(&v))
        .unwrap_or([0.0, 0.0, DEFAULT_SIZE[0], DEFAULT_SIZE[1]]);
    let tps = ui.ticks_per_second().max(1) as f32;
    let start = ui
        .interaction
        .state
        .anims
        .get(&smil_clock(path))
        .map_or(ui.interaction.state.tick, |a| a.t);
    let still = ui.theme().reduced_motion;
    let now = ui.interaction.state.tick.saturating_sub(start) as f32 / tps;
    let mut running = false;
    let mut values: Vec<(String, String)> = Vec::new();
    let mut motion: Option<String> = None;
    for &c in &n.children {
        let node = &doc.nodes[c];
        let Some(tag) = node
            .selector_tag
            .as_deref()
            .filter(|t| crate::svg::smil::TAGS.contains(t))
        else {
            continue;
        };
        let parsed = literal_attrs(node).and_then(|attrs| {
            let mpath = match node
                .children
                .iter()
                .map(|&m| &doc.nodes[m])
                .find(|m| m.selector_tag.as_deref() == Some("mpath"))
            {
                Some(m) => {
                    let attrs = literal_attrs(m)?;
                    let href = attrs
                        .iter()
                        .find(|(k, _)| k == "href")
                        .map(|(_, v)| v.as_str())
                        .ok_or("<mpath> names its path: href=\"#id\"")?;
                    Some(
                        shape_by_href(doc, href, vb)
                            .map_err(|e| format!("<mpath {e}"))?
                            .0,
                    )
                }
                None => None,
            };
            Animation::parse(tag, &attrs, mpath)
        });
        let a = match parsed {
            Ok(a) => a,
            Err(e) => {
                r.cascade.diagnostics.push(format!("<{tag}>: {e}"));
                continue;
            }
        };
        let begins = begin_times(doc, &a, 0);
        let at = if still {
            match (a.active(), a.freeze) {
                (Some(d), true) => begins.iter().copied().reduce(f32::min).map(|b| (b, d)),
                _ => None,
            }
        } else {
            running |= begins.iter().any(|b| *b > now);
            Animation::current(&begins, now).map(|b| (b, now - b))
        };
        let Some((_, local)) = at else {
            continue;
        };
        let (progress, live) = a.progress(local);
        running |= live && !still;
        let Some(progress) = progress else {
            continue;
        };
        let current = |values: &[(String, String)], name: &str| -> String {
            if let Some((_, v)) = values.iter().rev().find(|(k, _)| k == name) {
                return v.clone();
            }
            if name == "transform" {
                return r.str("svgTransform").unwrap_or_default().to_string();
            }
            if name == "opacity" {
                return r.num("opacity").unwrap_or(1.0).to_string();
            }
            own(r, name)
                .or_else(|| svg_prop(name).and_then(|p| r.str(p)).map(str::to_string))
                .unwrap_or_default()
        };
        match &a.target {
            Target::Attr(name) => {
                let v = a.sample(&current(&values, name), progress);
                values.push((name.clone(), v));
            }
            Target::Transform(_) => {
                let v = a.sample(&current(&values, "transform"), progress);
                values.push(("transform".into(), v));
            }
            Target::Motion { .. } => motion = Some(a.sample("", progress)),
        }
    }
    if let Some(m) = motion {
        let base = values
            .iter()
            .rev()
            .find(|(k, _)| k == "transform")
            .map(|(_, v)| v.clone())
            .or_else(|| r.str("svgTransform").map(str::to_string))
            .unwrap_or_default();
        values.push(("transform".into(), format!("{m} {base}")));
    }
    for (name, v) in values {
        if PRESENTATION.contains(&name.as_str()) || TEXT_PRESENTATION.contains(&name.as_str()) {
            r.cascade.svg.push((name, v));
        } else if name == "opacity" {
            match v.trim().parse::<f64>() {
                Ok(o) => {
                    if let Some((spec, _)) = schema::prop("opacity") {
                        r.vals.push((spec, V::Num(o.clamp(0.0, 1.0))));
                    }
                }
                Err(_) => r
                    .cascade
                    .diagnostics
                    .push(format!("<animate> opacity \"{v}\" is not a number")),
            }
        } else if let Some((spec, _)) = svg_prop(&name).and_then(schema::prop) {
            r.vals.push((spec, V::Str(v)));
        } else {
            r.cascade.diagnostics.push(format!(
                "<animate attributeName=\"{name}\">: no SVG attribute {name} to animate"
            ));
        }
    }
    running
}
