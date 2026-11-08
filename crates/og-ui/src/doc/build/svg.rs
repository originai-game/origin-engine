//! Inline SVG in documents (`docs/ui.md` §3.3): `<svg>`, `<g>` and the
//! basic shapes as retained elements, their attributes bindable.
//!
//! Each shape is one element whose draw is its path in the root `<svg>`'s
//! user space ([`crate::paint::ElementDraw::view_box`]): its fill is the
//! element's shape, drawn analytically by og-render's UI pass, and each of
//! its subpaths is a stroked outline. The view box is fitted into the box
//! when the frame is painted, so a box that resizes repaints only its
//! shapes and nothing is built again.
//!
//! SVG's presentation attributes inherit as CSS's inherited properties do:
//! an `<svg>` or `<g>` writes its effective values (and its accumulated
//! transform) into its computed map under names no stylesheet can write,
//! and each SVG element under it records reading them, so a group's bound
//! `fill` re-resolves that group and the shapes that read it, nothing
//! else.
use super::*;
use crate::doc::extern_element::PRESENTATION;
use crate::doc::schema_extra::svg_prop;
use crate::paint::{
    ColorRef, ElementDraw, Gradient, GradientKind, Paint, ShapeKind, Stop, Stroke, StrokeAlign,
};
use crate::svg::{Affine, GradientDef, PaintRef, Style, bounds, shape_path};
use og_core::glam::Vec2;

/// The kinds this module builds.
pub(super) const KINDS: &[&str] = &["svg", "svgGroup", "svgShape", "svgDef"];

const VIEW_BOX: &str = " svg viewBox";
const TRANSFORM: &str = " svg transform";

fn key(attr: &str) -> String {
    format!(" svg {attr}")
}

/// HTML's size of an `<svg>` with neither a size nor a view box.
const DEFAULT_SIZE: [f32; 2] = [300.0, 150.0];

fn fixed(s: Option<Size>) -> Option<f32> {
    match s? {
        Size::Fixed(v) => Some(v),
        _ => None,
    }
}

/// The view box `min-x min-y width height`.
fn view_box(text: &str) -> Option<[f32; 4]> {
    let n = crate::svg::numbers(text).ok()?;
    (n.len() == 4 && n[2] > 0.0 && n[3] > 0.0).then(|| [n[0], n[1], n[2], n[3]])
}

fn matrix(a: &Affine) -> String {
    let [a, b, c, d, e, f] = a.0;
    format!("matrix({a} {b} {c} {d} {e} {f})")
}

/// A presentation value the element sets: its CSS, else its attribute.
fn own(r: &Resolved, attr: &str) -> Option<String> {
    r.cascade
        .svg
        .iter()
        .rev()
        .find(|(name, _)| name == attr)
        .map(|(_, v)| v.clone())
        .or_else(|| svg_prop(attr).and_then(|p| r.str(p)).map(str::to_string))
}

/// An SVG element's inherited state, written into its computed map for
/// the SVG elements under it (see the module documentation).
pub(super) fn inherit(kind: &str, r: &mut Resolved) {
    if kind == "svgDef" {
        return;
    }
    let reads = &mut r.cascade.inherited_reads;
    reads.insert(VIEW_BOX.into());
    reads.insert(TRANSFORM.into());
    reads.extend(PRESENTATION.iter().map(|a| key(a)));
    let mut writes: Vec<(String, String)> = Vec::new();
    for attr in PRESENTATION {
        if let Some(v) = own(r, attr) {
            writes.push((key(attr), v));
        }
    }
    let computed = &r.cascade.computed;
    let inherited = |name: &str| computed.get(name).cloned().flatten();
    if kind == "svg" {
        let vb = match r.str("svgViewBox") {
            Some(v) => v.to_string(),
            None => {
                let w = fixed(r.size("width")).unwrap_or(DEFAULT_SIZE[0]);
                let h = fixed(r.size("height")).unwrap_or(DEFAULT_SIZE[1]);
                format!("0 0 {w} {h}")
            }
        };
        if view_box(&vb).is_none() {
            r.cascade
                .diagnostics
                .push(format!("viewBox \"{vb}\" is not four numbers with a size"));
        }
        writes.push((VIEW_BOX.into(), vb));
        writes.push((TRANSFORM.into(), matrix(&Affine::IDENTITY)));
    } else {
        let parent = inherited(TRANSFORM)
            .and_then(|t| Affine::parse(&t).ok())
            .unwrap_or(Affine::IDENTITY);
        let total = match r.str("svgTransform").map(Affine::parse) {
            Some(Ok(t)) => parent.then(&t),
            Some(Err(e)) => {
                r.cascade.diagnostics.push(format!("transform: {e}"));
                parent
            }
            None => parent,
        };
        writes.push((TRANSFORM.into(), matrix(&total)));
    }
    let computed = Arc::make_mut(&mut r.cascade.computed);
    for (name, value) in writes {
        computed.insert(name, Some(value));
    }
}

fn srgba(c: [f32; 4], alpha: f32) -> Srgba {
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    Srgba {
        rgb: [byte(c[0]), byte(c[1]), byte(c[2])],
        alpha: (c[3] * alpha).clamp(0.0, 1.0),
    }
}

/// A gradient node's literal attributes by their SVG names.
fn literal_attrs(n: &DocNode) -> Result<Vec<(String, String)>, String> {
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
fn gradient(
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
fn subpaths(p: &crate::svg::Path) -> Vec<(Vec<[f32; 2]>, bool)> {
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

/// A shape's draw, in its `<svg>`'s user space.
fn shape_draw(
    doc: &UiDocument,
    tag: &str,
    r: &Resolved,
    vb: [f32; 4],
) -> Result<ElementDraw, String> {
    let computed = &r.cascade.computed;
    let value = |name: &str| computed.get(name).cloned().flatten();
    let mut style = Style::default();
    for attr in PRESENTATION {
        if let Some(v) = value(&key(attr)) {
            let parent = style.clone();
            style
                .apply(attr, &v, &parent)
                .map_err(|e| format!("{attr}: {e}"))?;
        }
    }
    let t = value(TRANSFORM)
        .and_then(|t| Affine::parse(&t).ok())
        .unwrap_or(Affine::IDENTITY);
    let get = |attr: &str| svg_prop(attr).and_then(|p| r.str(p));
    let mut draw = ElementDraw {
        view_box: Some(vb),
        ..ElementDraw::default()
    };
    let Some(local) = shape_path(tag, &get, vb)? else {
        return Ok(draw);
    };
    let bbox = bounds(&local);
    let path = local.transformed(&t);
    let parts = subpaths(&path);
    let paint = |p: &PaintRef, alpha: f32| -> Result<Option<Paint>, String> {
        Ok(match p {
            PaintRef::None => None,
            PaintRef::Color(c) => Some(Paint::Solid(ColorRef::Literal(srgba(*c, alpha)))),
            PaintRef::Current => Some(Paint::Solid(ColorRef::Token {
                name: "ink".into(),
                alpha,
            })),
            PaintRef::Token(name) => Some(Paint::Solid(ColorRef::Token {
                name: crate::doc::css::camel(name),
                alpha,
            })),
            PaintRef::Url(id) => Some(Paint::Gradient(gradient(doc, id, bbox, &t, alpha)?)),
        })
    };
    let rule = if style.even_odd {
        crate::paint::FillRule::EvenOdd
    } else {
        crate::paint::FillRule::NonZero
    };
    let cap = match style.cap {
        crate::svg::Cap::Butt => crate::paint::Cap::Butt,
        crate::svg::Cap::Round => crate::paint::Cap::Round,
        crate::svg::Cap::Square => crate::paint::Cap::Square,
    };
    let join = match style.join {
        crate::svg::Join::Miter => crate::paint::Join::Miter(style.miter_limit),
        crate::svg::Join::Round => crate::paint::Join::Round,
        crate::svg::Join::Bevel => crate::paint::Join::Bevel,
    };
    let width = style.stroke_width * t.scale();
    draw.shape = ShapeKind::Path {
        points: parts.iter().flat_map(|(p, _)| p.iter().copied()).collect(),
        closed: true,
        filled: true,
        curves: true,
        fill_rule: rule,
        contours: parts.iter().map(|(p, _)| p.len() as u32).collect(),
        width,
        cap,
        join,
    };
    // A line has no inside; a polyline fills as SVG does, closed.
    if tag != "line"
        && let Some(fill) = paint(&style.fill, style.fill_opacity)?
    {
        draw.fills.push(fill);
    }
    if width > 0.0
        && let Some(stroke) = paint(&style.stroke, style.stroke_opacity)?
    {
        // Dashes measure the outline in user units, or in `pathLength`.
        let per = match get("pathLength").map(|v| v.trim().parse::<f32>()) {
            Some(Ok(l)) if l > 0.0 => local.length(vb[2].max(vb[3]) / 2000.0) / l,
            Some(_) => return Err("pathLength is a positive number".into()),
            None => 1.0,
        } * t.scale();
        let dash = match style.dashes.as_slice() {
            [] => [0.0; 2],
            [d, g] => [d * per, g * per],
            more => {
                return Err(format!(
                    "stroke-dasharray takes one dash and one gap, not {} lengths",
                    more.len()
                ));
            }
        };
        for (points, closed) in parts {
            draw.outlines.push((
                ShapeKind::Path {
                    points,
                    closed,
                    filled: false,
                    curves: true,
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
                    dash_offset: style.dash_offset * per,
                    dash_cap: cap,
                    paint: stroke.clone(),
                },
            ));
        }
    }
    Ok(draw)
}

impl Ui {
    /// Builds an SVG element: `<svg>` as a box of its size, a group or a
    /// shape as a box over the whole `<svg>`, a definition as nothing.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build_svg(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        kind: &'static str,
        id: &str,
        r: &Resolved,
        props: Props,
        path: String,
        key: u64,
        built: Built,
        build_children: impl FnOnce(&mut Ui, &mut Build<'_>, Option<Props>),
    ) -> Option<String> {
        if kind == "svgDef" {
            return None;
        }
        let computed = r.cascade.computed.clone();
        let vb = computed
            .get(VIEW_BOX)
            .cloned()
            .flatten()
            .and_then(|v| view_box(&v))
            .unwrap_or([0.0, 0.0, DEFAULT_SIZE[0], DEFAULT_SIZE[1]]);
        let mut el = Element {
            direction: Direction::Column,
            ..Element::default()
        };
        if kind == "svg" {
            el.width = Size::px(vb[2]);
            el.height = Size::px(vb[3]);
        }
        let mut el = Self::box_of(r, el, self.em_of(r));
        if kind != "svg" {
            el.width = Size::grow();
            el.height = Size::grow();
        }
        self.open(id, el);
        let i = self.parent();
        let mut p = Self::resolved_props(n, r);
        p.states = props.states;
        p.state = props.state;
        if kind != "svg" {
            // Over the whole `<svg>`: its user space is the root's box.
            p.position = crate::element::Position::Absolute;
            p.insets = crate::element::Edges {
                start: Some(0.0),
                top: Some(0.0),
                end: Some(0.0),
                bottom: Some(0.0),
                ..Default::default()
            };
        }
        self.nodes[i].props = p;
        if let Some(role) = n.role {
            self.nodes[i].a11y.role = Some(role);
        }
        if let Some(name) = r.str("name").filter(|n| !n.is_empty()) {
            self.nodes[i].a11y.name = Some(name.to_string());
        }
        if kind == "svgShape" {
            let tag = n.selector_tag.as_deref().unwrap_or("path");
            match shape_draw(&b.doc, tag, r, vb) {
                Ok(draw) => self.set_draw(draw),
                Err(e) => self.warnings.push(format!(
                    "{}:{} ({path}): <{tag}>: {e}",
                    b.doc.source, n.line
                )),
            }
        }
        self.note_built(built);
        if kind != "svgShape" {
            let inherited = std::mem::replace(&mut b.inherited, computed);
            build_children(self, b, None);
            b.inherited = inherited;
        }
        let resp = self.close();
        self.fire(b, n, key, &path, resp);
        Some(path)
    }
}

#[cfg(test)]
mod tests {
    use crate::doc::UiDocument;
    use crate::reactive::SourceId;
    use crate::reactive::session::Command;
    use crate::{DrawCmd, InputFrame, Ui, UiDesc, Viewport};
    use serde_json::json;
    use std::collections::BTreeMap;

    const RING: &str = r##"
<meta name="component" content="ring">
<div id="Box">
  <svg id="Ring" viewBox="0 0 40 40" style:--s={$size}>
    <g id="G" stroke="#ff0000">
      <circle id="Track" cx="20" cy="20" r="18" fill="none" stroke-width="1"/>
      <circle id="Fill" cx="20" cy="20" r="18" fill="none" pathLength="1"
        stroke-dasharray="1 1" stroke-dashoffset={1 - $ready}/>
    </g>
    <rect id="Bar" x="0" y={40 - $h} width="8" height={$h} fill="url(#Grad)"/>
    <linearGradient id="Grad"><stop offset="0" stop-color="#000"/><stop offset="1" stop-color="#fff"/></linearGradient>
  </svg>
</div>
<style>#Ring { width: calc(var(--s) * 1px); height: calc(var(--s) * 1px); }</style>
"##;

    /// The frame's stroke widths and whether a gradient fill drew.
    fn strokes(ui: &Ui) -> Vec<f32> {
        ui.frame()
            .items
            .iter()
            .filter_map(|it| match &it.cmd {
                DrawCmd::Shape { shape, .. } => match shape.mode {
                    og_render::UiMode::Stroke(s) => Some(s.width),
                    _ => None,
                },
                _ => None,
            })
            .collect()
    }

    #[test]
    fn inline_svg_shapes_bind_attributes_and_repaint_only_what_changed() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("ring.svelte", RING).unwrap());
        let mut inputs = BTreeMap::new();
        for (name, v) in [("ready", 0.25), ("h", 10.0), ("size", 80.0)] {
            let source = SourceId::new("test", name);
            ui.reactive
                .execute(Command::Signal {
                    source: source.clone(),
                    initial: json!(v),
                    readable: false,
                })
                .unwrap();
            inputs.insert(name.to_string(), source);
        }
        ui.mount_document(
            SourceId::new("test", "ring"),
            "ring",
            inputs,
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui, w: f32| {
            ui.begin(InputFrame::default(), Viewport::new(w, 400.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end().stats
        };
        for _ in 0..3 {
            tick(&mut ui, 400.0);
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        // A 1-unit stroke in a 2× box is 2 px, as in browsers.
        let widths = strokes(&ui);
        assert_eq!(widths.len(), 2, "{widths:?}");
        assert!(widths.iter().all(|w| (w - 2.0).abs() < 1e-4), "{widths:?}");
        let gradient = ui.frame().items.iter().any(|it| {
            matches!(&it.cmd, DrawCmd::Shape { shape, .. }
                if matches!(shape.paint, og_render::UiPaint::Gradient(_)))
        });
        assert!(gradient, "the bar's url(#Grad) fill draws");
        let idle = tick(&mut ui, 400.0);
        assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
        // One bound attribute: that shape alone (the store and the
        // expression `1 - $ready` are its two evaluations).
        ui.reactive
            .execute(Command::Set {
                source: SourceId::new("test", "ready"),
                value: json!(0.75),
                force: false,
            })
            .unwrap();
        let changed = tick(&mut ui, 400.0);
        assert_eq!(
            (changed.bindings, changed.reconciled),
            (2, 1),
            "{changed:?}"
        );
        // The ring's dash offset is a quarter of its circumference, in px.
        let offset = ui.frame().items.iter().find_map(|it| match &it.cmd {
            DrawCmd::Shape { shape, .. } => match shape.mode {
                og_render::UiMode::Stroke(s) if s.dash[0] > 0.0 => Some((s.dash, s.dash_offset)),
                _ => None,
            },
            _ => None,
        });
        let (dash, off) = offset.expect("the dashed ring");
        let circumference = 2.0 * std::f32::consts::PI * 18.0 * 2.0;
        assert!((dash[0] - circumference).abs() < 0.5, "{dash:?}");
        assert!((off - 0.25 * circumference).abs() < 0.5, "{off}");
        // Only the box changes: its shapes' attributes stay, their draws
        // are placed in the new box (a 1-unit stroke at 3× is 3 px).
        ui.reactive
            .execute(Command::Set {
                source: SourceId::new("test", "size"),
                value: json!(120.0),
                force: false,
            })
            .unwrap();
        for _ in 0..3 {
            tick(&mut ui, 400.0);
        }
        let widths = strokes(&ui);
        assert_eq!(widths.len(), 2, "{widths:?}");
        assert!(widths.iter().all(|w| (w - 3.0).abs() < 1e-4), "{widths:?}");
        let idle = tick(&mut ui, 400.0);
        assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
    }
}
