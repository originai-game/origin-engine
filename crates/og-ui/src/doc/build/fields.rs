//! Render-field replacement from evaluated inputs. No binding evaluation,
//! widget input processing, animation sampling or document emission lives here.
use super::*;

pub(super) struct BoxSource<'a> {
    pub document: Option<&'a UiDocument>,
    /// The element's kind; none for a box a control draws itself.
    pub kind: Option<Kind>,
    pub tag: Option<&'a str>,
    pub root: bool,
    pub line: usize,
    /// An `<iframe>`'s document ([`DocNode::frame`]).
    pub frame: Option<&'a str>,
}

#[derive(Clone, Debug)]
pub struct RenderFields {
    pub index: usize,
    host_key: u64,
    /// The element's kind; none for a box a control draws itself.
    kind: Option<Kind>,
    element: Element,
    widget_props: Props,
    document_props: Props,
    pub text: String,
    response: crate::Response,
    amounts: (f32, f32),
    press: f32,
    texture: Option<og_draw::TextureHandle>,
    art: Option<String>,
    /// Hidden by something other than the document's `display`.
    hidden: bool,
    /// A text's runs with its generated content (`::before`, `::after`,
    /// `::first-letter`, `::first-line`), as its build made them.
    pub generated: Option<std::sync::Arc<GeneratedRuns>>,
}

/// A text with its generated content, and the runs that style it.
pub type GeneratedRuns = (String, Vec<(crate::TextSpan, crate::TextSpanStyle)>);

impl RenderFields {
    pub fn capture(
        ui: &Ui,
        host_key: u64,
        index: usize,
        kind: Option<Kind>,
        response: crate::Response,
        document_props: Props,
        texture: Option<og_draw::TextureHandle>,
    ) -> Self {
        let node = &ui.tree.nodes[index];
        Self {
            index,
            host_key,
            kind,
            element: *node.el,
            widget_props: (*node.props).clone(),
            document_props,
            text: node.probe_text.clone(),
            response,
            amounts: node.part().map_or((0.0, 0.0), |p| (p.hover, p.press)),
            press: match node.widget {
                crate::layout::Widget::Button { press, .. } => press,
                _ => 0.0,
            },
            texture,
            art: (kind == Some(Kind::Art)).then(|| ui.docs.last_art_path()),
            hidden: node.hidden,
            generated: None,
        }
    }
}

impl RenderFields {
    /// A list or grid opens with its styled width, height, padding and gap;
    /// its baseline is the box without them.
    pub fn unstyle_virtual(&mut self) {
        self.element.width = Size::grow();
        self.element.height = Size::grow();
        self.element.padding = Insets::default();
        self.element.gap = 0.0;
    }
}

/// Its CSS corners when a corner differs from the element's one circular
/// `radius`: per-corner, elliptical or % radii, or a `corner-shape`.
fn corners(r: &Resolved, radius: f32) -> Option<crate::decor::Corners> {
    use crate::doc::boxes::{CORNER_RADII, CORNER_SHAPES, corner_radius, corner_shape};
    let written = |p: &str| r.str(p).filter(|v| !v.trim().is_empty());
    if CORNER_RADII
        .iter()
        .chain(&CORNER_SHAPES)
        .all(|p| written(p).is_none())
    {
        return None;
    }
    let mut c = crate::decor::Corners {
        radii: [[(radius, 0.0); 2]; 4],
        shape: [1.0; 4],
    };
    for k in 0..4 {
        if let Some(Ok(v)) = written(CORNER_RADII[k]).map(corner_radius) {
            c.radii[k] = v;
        }
        if let Some(Ok(t)) = written(CORNER_SHAPES[k]).map(crate::doc::css::tokens)
            && let [t] = t.as_slice()
            && let Ok(v) = corner_shape(t)
        {
            c.shape[k] = v;
        }
    }
    // Written but the same as the one radius (a role's `corner-shape:
    // round` beside its `border-radius`): nothing differs.
    let plain = crate::decor::Corners {
        radii: [[(radius, 0.0); 2]; 4],
        shape: [1.0; 4],
    };
    (c != plain).then_some(c)
}

impl Ui {
    /// The CSS border, outline, shadows and overflow of an element, with
    /// what they do to its box: a border's width pads it, an outline set
    /// replaces the focus ring, a scrolling overflow scrolls it, and a
    /// shadow takes its background under it.
    pub(super) fn box_decor(
        &self,
        doc: Option<&crate::doc::UiDocument>,
        r: &Resolved,
        el: &mut Element,
        kind: Option<Kind>,
        warn: &mut Vec<String>,
    ) -> Option<crate::decor::BoxDecor> {
        use crate::decor::{BoxDecor, Line, LineStyle, Shadow};
        let theme = self.theme();
        let current = r.current(theme);
        let cx = r.color_context(theme);
        let mut d = BoxDecor::default();
        for (k, side) in crate::doc::boxes::SIDES.iter().enumerate() {
            let style = r
                .str(&format!("border{side}Style"))
                .and_then(LineStyle::from_css);
            if let Some(style) = style {
                let width = r.num(&format!("border{side}Width")).unwrap_or(3.0).max(0.0);
                if width > 0.0 {
                    d.borders[k] = Some(Line {
                        width,
                        style,
                        color: r.color(&format!("border{side}Color")).unwrap_or(current),
                    });
                }
            }
        }
        let w = |k: usize| d.borders[k].map_or(0.0, |b| b.width);
        el.padding.top += w(0);
        el.padding.right += w(1);
        el.padding.bottom += w(2);
        el.padding.left += w(3);
        if let Some(style) = r.str("outlineStyle") {
            // Any outline but `auto` is the element's own focus mark.
            el.own_ring = style != "auto";
            if let Some(style) = LineStyle::from_css(style) {
                let width = r.num("outlineWidth").unwrap_or(3.0).max(0.0);
                if width > 0.0 {
                    d.outline = Some((
                        Line {
                            width,
                            style,
                            color: r.color("outlineColor").unwrap_or(current),
                        },
                        r.num("outlineOffset").unwrap_or(0.0),
                    ));
                }
            }
        }
        if let Some(text) = r.str("boxShadow") {
            for s in crate::doc::boxes::shadows(text).unwrap_or_default() {
                let color = match s.color.as_deref() {
                    None => Some(current),
                    Some([og_core::ogt::Value::String(text)]) => crate::css_color::parse_str(text)
                        .ok()
                        .map(|c| c.resolve(&cx)),
                    Some([_, og_core::ogt::Value::Word(token)]) => theme.color(token),
                    Some(_) => None,
                };
                if let Some(color) = color {
                    d.shadows.push(Shadow {
                        inset: s.inset,
                        offset: s.offset,
                        blur: s.blur,
                        spread: s.spread,
                        color,
                    });
                }
            }
            // Outer shadows go under the background: it is drawn with them.
            if d.shadows.iter().any(|s| !s.inset) && el.part.is_none() && el.frame.is_none() {
                d.background = el
                    .background
                    .take()
                    .map(|c| crate::paint::Paint::Solid(crate::paint::ColorRef::Literal(c)));
            }
        }
        d.corners = corners(r, el.radius);
        let css = self.css_paint(doc, r, &cx, warn);
        // A clip, mask, blend, filter or material cuts or composites the
        // background too, and a border shape shapes it: it is drawn with
        // them.
        if (css.as_ref().is_some_and(|c| {
            c.clip.is_some()
                || c.border_shape.is_some()
                || !c.masks.is_empty()
                || c.mask_border.is_some()
                || c.color_clip != crate::doc::paint_css::BoxEdge::Border
                || c.images
                    .iter()
                    .any(|l| l.blend != crate::paint::Blend::Normal)
                || c.blend.is_some()
                || c.isolate
                || c.layer.is_some()
                || !c.filters.is_empty()
        }) || d.corners.is_some())
            && d.background.is_none()
            && el.part.is_none()
            && el.frame.is_none()
        {
            d.background = el
                .background
                .take()
                .map(|c| crate::paint::Paint::Solid(crate::paint::ColorRef::Literal(c)));
        }
        d.css = css.map(Box::new);
        // CSS Overflow 3: `visible` beside a scrolling or clipping axis
        // computes to `auto`.
        let (x, y) = (r.str("overflowX"), r.str("overflowY"));
        if x.is_some() || y.is_some() {
            // `overlay` is a legacy alias of `auto` (CSS Overflow 3 §3.1).
            fn alias(a: Option<&str>) -> &str {
                match a {
                    Some("overlay") => "auto",
                    a => a.unwrap_or("visible"),
                }
            }
            let (mut x, mut y) = (alias(x), alias(y));
            let implied = x == "visible" && y != "visible";
            if implied {
                x = "auto";
            }
            if y == "visible" && x != "visible" {
                y = "auto";
            }
            let scrolls = |a: &str| matches!(a, "auto" | "scroll");
            if kind.is_some_and(Kind::container) {
                el.scroll = scrolls(y);
                el.scroll_x = scrolls(x);
                el.scroll_x_implied = implied && el.scroll_x;
            }
            d.clip = matches!(x, "hidden" | "clip") || matches!(y, "hidden" | "clip");
        }
        // Paint containment clips what it and its subtree draw to its box
        // (CSS Containment 2 §3.4).
        let paint = crate::doc::contain_css::contain(|p| r.str(p).map(str::to_owned))
            .has(crate::layout::Contain::PAINT);
        if paint {
            d.clip = true;
        }
        // `overflow-clip-margin` moves the edge of an `overflow: clip` or
        // paint containment cut, not of `hidden` (CSS Overflow 4 §3.2).
        let clips = [r.str("overflowX"), r.str("overflowY")].contains(&Some("clip"));
        if (clips || paint)
            && let Some(Ok(m)) = r
                .str("overflowClipMargin")
                .map(crate::doc::contain_css::clip_margin)
        {
            d.clip_margin = Some(m);
        }
        if let Some(Ok(s)) = r
            .str("scrollSnapType")
            .map(crate::doc::paint_css::snap_type)
        {
            el.snap = s;
        }
        if let Some(Ok(a)) = r
            .str("scrollSnapAlign")
            .map(crate::doc::paint_css::snap_align)
        {
            el.snap_align = a;
        }
        if r.str("scrollbarWidth") == Some("none") {
            el.hide_bars = true;
        }
        el.scroll_css = crate::doc::scroll_css::build(
            |p| r.str(p),
            &crate::css_color::Context {
                current,
                ..crate::css_color::Context::of(theme)
            },
        );
        // Authored corners also apply to CSS material backgrounds.
        if r.has("radius") {
            d.radius = Some(el.radius);
        }
        // A grid's gap rules (CSS Gap Decorations 1); a multi-column
        // container draws its column rules itself.
        if matches!(super::inner_display(r.str("display")), "grid" | "flex") {
            for (k, axis) in ["column", "row"].iter().enumerate() {
                for (e, name) in crate::css::multicol_css::RULE_INSETS[k].iter().enumerate() {
                    if let Some(v) = r.str(name).and_then(crate::css::multicol_css::inset) {
                        d.gap_insets[k][e] = v;
                    }
                }
                let style = r
                    .str(&format!("{axis}RuleStyle"))
                    .and_then(LineStyle::from_css);
                let width = match r.str(&format!("{axis}RuleWidth")) {
                    None | Some("medium") => 3.0,
                    Some("thin") => 1.0,
                    Some("thick") => 5.0,
                    Some(v) => v.trim_end_matches("px").parse().unwrap_or(0.0),
                };
                let brk = match r.str(&format!("{axis}RuleBreak")) {
                    Some("none") => crate::decor::GapBreak::None,
                    Some("intersection") => crate::decor::GapBreak::Intersection,
                    _ => crate::decor::GapBreak::Normal,
                };
                if let Some(style) = style.filter(|_| width > 0.0) {
                    let color = r.color(&format!("{axis}RuleColor")).unwrap_or(current);
                    d.gap_rules[k] = Some((
                        Line {
                            width,
                            style,
                            color,
                        },
                        brk,
                    ));
                }
            }
        }
        // A layer's content box, and the edge a content-box clip margin
        // starts from.
        if d.css.is_some() || d.clip_margin.is_some() {
            let p = el.padding;
            d.content = [p.left, p.top, p.right, p.bottom];
        }
        (d != BoxDecor::default()).then_some(d)
    }

    /// The element's CSS paint (`docs/ui.md`, CSS paint); values the
    /// stylesheet already checked, so one that fails here is skipped.
    fn css_paint(
        &self,
        doc: Option<&crate::doc::UiDocument>,
        r: &Resolved,
        cx: &crate::css_color::Context,
        warn: &mut Vec<String>,
    ) -> Option<crate::doc::paint_css::CssPaint> {
        use crate::doc::paint_css as p;
        let current = cx.current;
        let mut c = p::CssPaint::default();
        let (_, nearest, orient) = r.object_css();
        // Asset paths were resolved beside their stylesheet before the cascade.
        let mut one = |what: &str, i: p::CssImage| -> Option<p::BgImage> {
            let Some((source, density)) = i.select(self.env.viewport.dpr()) else {
                return (what == "mask" && matches!(i, p::CssImage::Set(_)))
                    .then_some(p::BgImage::Transparent);
            };
            Some(match source {
                p::ImageSource::Gradient(g) => p::BgImage::Gradient(g.bound(cx)),
                p::ImageSource::Url(u)
                    if self
                        .docs
                        .library
                        .textures
                        .contains_key(u.trim_start_matches('/')) =>
                {
                    p::BgImage::Texture {
                        texture: self.docs.library.textures[u.trim_start_matches('/')],
                        uv: [0.0, 0.0, 1.0, 1.0],
                        density,
                        nearest,
                        orient,
                    }
                }
                // A remote image not bound yet is still being fetched: the
                // layer draws nothing until it lands, as a page's image
                // before it loads. Its host warns when it fails.
                p::ImageSource::Url(u) if crate::css::request::is_remote(u) => return None,
                p::ImageSource::Url(u) => {
                    warn.push(format!("{what}: url({u}) names no texture asset"));
                    return None;
                }
            })
        };
        // A cross-fade's parts and a filter()'s image each resolved, its
        // functions as the `filter` property's (no `url(#id)`).
        let mut image = |what: &str, i: p::CssImage| {
            let theme = self.theme();
            let stages = |fns: &[p::FilterFn]| {
                p::filter_layers(
                    fns,
                    |c| c.map_or(current.linear(), |c| c.resolve(theme)),
                    |_| None,
                )
            };
            p::bg_image(i.in_scheme(cx.dark), &mut |i| one(what, i), &stages)
        };
        let layer = |l: p::CssLayer, image: p::BgImage| p::BgLayer {
            image,
            size: l.size,
            position: l.position,
            repeat: l.repeat,
            origin: l.origin,
            clip: l.clip,
            attach: l.attach,
            blend: l.blend,
            luminance: l.luminance,
        };
        let mut errors = Vec::new();
        let mut longhands = |layers: &mut [p::CssLayer], props: &[(&str, &str)], mask: bool| {
            let values: Vec<(&str, Option<&str>)> =
                props.iter().map(|(k, prop)| (*k, r.str(prop))).collect();
            if let Err(e) = p::longhands(layers, &values, mask) {
                errors.push(e);
            }
        };
        if let Some(Ok((mut layers, _))) = r.str("backgroundImage").and_then(p::background) {
            longhands(&mut layers, &p::BACKGROUND_LONGHANDS, false);
            c.color_clip = layers.last().map(|l| l.clip).unwrap_or_default();
            for l in layers {
                if let Some(i) = image("background", l.image.clone()) {
                    c.images.push(layer(l, i));
                }
            }
        } else if let Some(v) = r.str("backgroundClip").filter(|v| !v.is_empty()) {
            // A colour alone, cut by its clip.
            let mut l = [p::CssLayer::new(p::CssImage::None, false)];
            if p::longhands(&mut l, &[("clip", Some(v))], false).is_ok() {
                c.color_clip = l[0].clip;
            }
        }
        if let Some(Ok(mut layers)) = r.str("maskImage").map(p::masks) {
            longhands(&mut layers, &p::MASK_LONGHANDS, true);
            for l in layers {
                if let Some(i) = image("mask", l.image.clone()) {
                    c.masks.push(layer(l, i));
                }
            }
        }
        let mut border = |props: &[(&str, &str)], mask: bool| -> Option<p::BorderImg> {
            let get = |k: usize| r.str(props[k].1).filter(|v| !v.trim().is_empty());
            let source = image(props[0].1, p::border_source(get(0)?).ok()?)?;
            p::border_spec(
                [None, get(1), get(2), get(3), get(4)],
                props.get(5).and_then(|_| get(5)),
                mask,
            )
            .ok()
            .map(|spec| p::BorderImg {
                image: source,
                spec,
            })
        };
        c.border_image = border(&p::BORDER_IMAGE_PROPS, false);
        c.mask_border = border(&p::MASK_BORDER_PROPS, true);
        warn.extend(errors);
        // `url(#id)`: an inline SVG `<filter>` of the document.
        let fe_image = |href: &str| -> Result<(og_draw::TextureHandle, [f32; 4]), String> {
            let path = doc.map_or_else(
                || href.trim_start_matches('/').to_owned(),
                |doc| crate::doc::at_rules::resolve(&doc.source, href),
            );
            self.docs
                .library
                .textures
                .get(&path)
                .map(|t| (*t, [0.0, 0.0, 1.0, 1.0]))
                .ok_or_else(|| format!("href \"{href}\" names no texture asset"))
        };
        let url = |warn: &mut Vec<String>, what: &str, u: &str| match doc
            .ok_or_else(|| "an SVG filter reference needs its document".to_string())
            .and_then(|doc| super::svg::css_filter(doc, u, &fe_image))
        {
            Ok(f) => Some(f),
            Err(e) => {
                warn.push(format!("{what}: {e}"));
                None
            }
        };
        let mut fails = Vec::new();
        let glass = r
            .str("material")
            .and_then(|s| p::glass_material(s).ok())
            .flatten();
        let fns = p::filter_fns(r.str("backdropFilter").unwrap_or("none"), "backdrop-filter");
        // Glass folds the filter into its own backdrop when it can; any
        // other chain runs as staged filters, and glass draws over what
        // they leave (its own blur and colour unset).
        let folded = match (&glass, &fns) {
            (Some(glass), Ok(fns)) => p::glass_effect(glass.clone(), fns),
            _ => None,
        };
        if folded.is_none()
            && let Ok(fns) = &fns
            && let Some(stages) =
                p::backdrop_stages(fns, crate::paint::ColorRef::Literal(current), |u| {
                    url(&mut fails, "backdrop-filter", u)
                })
        {
            c.effects
                .push(crate::paint::Effect::BackdropFilter(std::sync::Arc::new(
                    stages,
                )));
        }
        if let Some(glass) = glass {
            c.effects
                .extend(folded.or_else(|| p::glass_effect(glass, &[])));
        }
        if let Some(Ok(fns)) = r.str("filter").map(|v| p::filter_fns(v, "filter")) {
            let theme = self.theme();
            c.filters = p::filter_layers(
                &fns,
                |c| c.map_or(current.linear(), |c| c.resolve(theme)),
                |u| url(&mut fails, "filter", u),
            );
        }
        warn.extend(fails);
        c.blend = r.str("mixBlendMode").and_then(|v| p::blend(v).ok());
        // `will-change` naming a property that makes a stacking context
        // makes one now (CSS Will Change 1 §2), so it is drawn on a layer
        // of its own: the group its change composites.
        c.isolate = r.str("isolation").and_then(|v| p::isolation(v).ok()) == Some(true)
            || r.str("willChange")
                .and_then(|v| crate::doc::contain_css::will_change(v).ok())
                == Some(true);
        if let Some(Ok(s)) = r
            .str("textShadow")
            .map(|v| p::text_shadows(v, crate::paint::ColorRef::Literal(current)))
        {
            // `currentColor` inside a colour function is this element's
            // colour too (a halo in `contrast-color(currentColor)`), not
            // the style's ink it would resolve to when drawn.
            let theme = self.theme();
            c.text_shadows = s
                .into_iter()
                .map(|mut sh| {
                    if let crate::paint::ColorRef::Css(e) = &sh.color
                        && e.uses_current()
                    {
                        let cx = crate::css_color::Context {
                            current,
                            ..crate::css_color::Context::of(theme)
                        };
                        sh.color = crate::paint::ColorRef::Literal(e.resolve(&cx));
                    }
                    sh
                })
                .collect();
        }
        // `clip` applies to an absolutely positioned box, under `clip-path`.
        let legacy = r
            .str("clipRect")
            .filter(|_| matches!(r.str("position"), Some("absolute" | "screen")))
            .and_then(|v| p::legacy_clip(v).ok().flatten());
        let clip_path = r.str("clipPath").map(str::to_owned).or(legacy);
        if let Some((clip, geo)) = clip_path.and_then(|v| p::clip(&v).ok()).flatten() {
            c.clip_box = geo;
            c.clip = Some(match clip {
                p::CssClip::Url(u) => match doc
                    .ok_or_else(|| "an SVG clip reference needs its document".to_string())
                    .and_then(|doc| super::svg::css_clip(doc, &u))
                {
                    Ok((rule, parts, false)) => p::CssClip::Path(rule, parts),
                    Ok((rule, parts, true)) => p::CssClip::Units(rule, parts),
                    Err(e) => {
                        warn.push(format!("clip-path: {e}"));
                        p::CssClip::Path(crate::paint::FillRule::NonZero, Vec::new())
                    }
                },
                clip => clip,
            });
        }
        if let Some(Ok(Some(shapes))) = r.str("borderShape").map(p::border_shape) {
            c.border_shape = Some(Box::new(p::BorderShape {
                shapes: shapes.into_iter().map(|(s, b)| (s, b, [0.0; 4])).collect(),
                side: 0,
            }));
        }
        if let Some(Ok(Some(m))) = r.str("material").map(p::material) {
            match self.docs.css_material(m) {
                Ok(l) => c.layer = Some(l),
                Err(e) => warn.push(e),
            }
        }
        (c != p::CssPaint::default()).then_some(c)
    }

    pub fn apply_document_fields(
        &mut self,
        doc: &crate::doc::UiDocument,
        n: &DocNode,
        r: &Resolved,
        base: &RenderFields,
    ) {
        let i = base.index;
        *self.tree.el_mut(i) = base.element;
        if let Some(host) = self.tree.nodes.iter_mut().rfind(|n| n.key == base.host_key)
            && host.part_host.as_deref() != r.part_host.as_deref()
        {
            host.part_host = r.part_host.clone();
        }
        *self.tree.props_mut(i) = base.widget_props.clone();
        let css = r.text.as_deref();
        self.tree.nodes[i].accent = css.and_then(|t| t.accent);
        self.tree.nodes[i].caret = css.and_then(|t| t.caret);
        self.tree.nodes[i].caret_shape = css.and_then(|t| t.caret_shape);
        // `display: none` that a container query decides keeps its element
        // built, so settlement can show it again.
        self.tree.nodes[i].hidden = base.hidden || (r.has("visible") && !r.flag("visible"));
        match base.kind {
            Some(Kind::Label) => {
                let desc = r.label_desc(self.theme(), self.fonts(), r.own_flow().rtl);
                let shown;
                let base = match r.text.as_deref() {
                    Some(t) => {
                        shown = RenderFields {
                            text: t.shown(&base.text, &self.env.locale),
                            ..base.clone()
                        };
                        &shown
                    }
                    None => base,
                };
                if let Some(g) = &base.generated {
                    let (text, runs) = &**g;
                    let spans: Vec<crate::TextSpan> =
                        runs.iter().map(|(span, _)| span.clone()).collect();
                    self.apply_text_spans(i, text, &spans, r.rich_desc(&desc), |span| {
                        runs.iter()
                            .find(|(s, _)| s == span)
                            .map(|(_, style)| style.clone())
                            .unwrap_or_default()
                    });
                } else if r.get("spans").is_none()
                    && let Some((span, style)) =
                        r.css_span(&base.text, &doc.sheet, self.fonts(), &desc)
                {
                    self.apply_text_spans(
                        i,
                        &base.text,
                        std::slice::from_ref(&span),
                        r.rich_desc(&desc),
                        |_| style.clone(),
                    );
                } else if let Some(V::Spans(Ok(spans))) = r.get("spans") {
                    self.apply_text_spans(i, &base.text, spans, r.rich_desc(&desc), |span| {
                        r.span_styles
                            .iter()
                            .find(|(attribute, _)| attribute == span)
                            .map(|(_, style)| style.clone())
                            .unwrap_or_default()
                    });
                } else {
                    let width = desc.width;
                    self.apply_label(i, &base.text, desc);
                    self.tree.css_text_content(i, r, &doc.sheet);
                    // Path text aligns along its path, not in its box: it
                    // keeps the box its path gives (a `circle()` centres in
                    // it), never the column's width a centred line takes.
                    // Nor does a label out of the flow, which is no block
                    // of its column: it shrinks to fit its text, as CSS
                    // sizes an absolutely positioned box (CSS 2.1 §10.3.7).
                    let out_of_flow = match r.str("position") {
                        Some("absolute" | "screen") => true,
                        Some("relative" | "sticky") => false,
                        _ => ["start", "top", "end", "bottom"]
                            .iter()
                            .any(|s| r.num(s).is_some()),
                    };
                    if r.has("textPath") || out_of_flow {
                        self.tree.el_mut(i).width = width;
                    }
                }
            }
            Some(Kind::Button) => {
                self.apply_button(
                    i,
                    &base.text,
                    r.button_desc(),
                    base.response,
                    base.amounts,
                    base.press,
                );
                // A button holding elements (a row of an icon, a name and
                // its detail) grows to fit them, its control height the
                // least it takes, as an HTML button does.
                if !n.children.is_empty()
                    && let crate::layout::Size::Fixed(h) = self.tree.nodes[i].el.height
                {
                    self.tree.el_mut(i).height = crate::layout::Size::Fit {
                        min: h,
                        max: f32::INFINITY,
                    };
                }
            }
            Some(Kind::Image) => {
                self.apply_image(
                    i,
                    crate::ImageDesc {
                        texture: base.texture,
                        tint: None,
                        width: r.size("width").unwrap_or_default(),
                        height: r.size("height").unwrap_or_default(),
                        aspect: r
                            .nums("ratio")
                            .filter(|n| n.len() == 2 && n[1] > 0.0)
                            .map_or(r.num("aspect").unwrap_or(0.0), |n| n[0] / n[1]),
                        label: base.text.clone(),
                        uv: r.nums("uv").and_then(|n| <[f32; 4]>::try_from(n).ok()),
                    },
                );
                self.tree.object_fit(i, r);
            }
            Some(Kind::Icon) => {
                let name = match (r.str("icon"), r.str("cue")) {
                    (Some(icon), _) => icon.to_string(),
                    (None, Some(token)) => self.cue_icon(token).unwrap_or_default(),
                    (None, None) => String::new(),
                };
                self.apply_icon(
                    i,
                    &name,
                    crate::widgets::IconDesc {
                        size: r.num("iconSize").unwrap_or(0.0),
                        tint: r.color("color"),
                        secondary: r.str("secondary").map(str::to_string),
                        secondary_tint: None,
                        label: r.str("name").map(str::to_string),
                    },
                )
            }
            Some(Kind::Art) => {
                let path = base.art.clone().expect("art render fields");
                let mut spec = crate::slot::SlotSpec::from_props(
                    &path,
                    |name| r.str(name).map(str::to_string),
                    |name| r.nums(name).map(<[f32]>::to_vec),
                );
                let (at, nearest, orient) = r.object_css();
                if let Some([focal, _]) = at {
                    spec.focal = focal;
                }
                if let Some(slot) = self.docs.art_slot_mut(&path) {
                    *slot = spec.clone();
                }
                self.apply_art(
                    i,
                    &crate::widgets::ArtDesc {
                        spec,
                        texture: base.texture,
                        size: None,
                        width: r.size("width").unwrap_or_default(),
                        height: r.size("height").unwrap_or_default(),
                        label: r.str("name").unwrap_or_default().to_string(),
                        offset: at.map_or([0.0; 2], |a| a[1]),
                        nearest,
                        orient,
                    },
                );
            }
            Some(Kind::Details) => {
                // Its fields are its content column's; the build gives its
                // root (the parent) the CSS width too (`DetailsDesc.width`).
                let root = self.tree.nodes[i].parent;
                if root != crate::layout::NONE {
                    self.tree.el_mut(root).width = r.size("width").unwrap_or(Size::grow());
                }
            }
            Some(Kind::Spacer) => {
                let size = r.size("width").or(r.size("height")).unwrap_or(Size::grow());
                self.tree.el_mut(i).width = size;
                self.tree.el_mut(i).height = size;
            }
            _ => {}
        }
        self.apply_box_fields(
            i,
            r,
            BoxSource {
                document: Some(doc),
                kind: base.kind,
                tag: n.selector_tag.as_deref(),
                root: n.parent == crate::layout::NONE,
                line: n.line,
                frame: n.frame.as_deref(),
            },
        );
        self.apply_document_semantics(n, r, base);
    }

    pub(super) fn apply_box_fields(&mut self, i: usize, r: &Resolved, source: BoxSource<'_>) {
        // Widget-owned sizing is established above; document box overrides
        // replace their previous values, including when a declaration vanishes.
        let content = r.str("fieldSizing") == Some("content");
        let field_line = self.field_measurement(i, content);
        let mut el = *self.tree.nodes[i].el;
        if field_line.is_some() {
            el.width = Size::fit();
            el.height = Size::fit();
        }
        // A content-sized select fits the option it shows (or its
        // placeholder) instead of filling the line.
        if content && source.kind == Some(Kind::Select) {
            el.width = Size::fit();
        }
        let em = self.style.em_of(r, &self.env);
        if source.kind.is_some_and(Kind::container) {
            el = Self::box_of(r, el, em);
            if !matches!(
                source.kind,
                Some(Kind::Panel | Kind::Scroll | Kind::List | Kind::Cells)
            ) {
                // A box is a column (CSS block), or a row as `display: flex`.
                let display = super::inner_display(r.str("display"));
                let initial = match (source.kind, display) {
                    (Some(Kind::Box), "flex") => "row",
                    (Some(Kind::Box), _)
                        if r.str("display") == Some("listItem")
                            && r.str("markerPosition") == Some("inside") =>
                    {
                        "row"
                    }
                    // A button's content is a row in it.
                    (Some(Kind::Button), _) => "row",
                    (kind, _) => kind.map_or("", Kind::name),
                };
                let direction = r.str("direction").unwrap_or(initial);
                el.direction = match direction {
                    "row" | "rowReverse" => Direction::Row,
                    _ => Direction::Column,
                };
                el.reverse = direction.ends_with("Reverse");
                // A flex box's lines share its free room across it unless
                // told otherwise (`align-content: normal`, CSS Flexbox §8.4).
                if source.kind == Some(Kind::Box) && display == "flex" && el.align_content.is_none()
                {
                    el.align_content = Some(crate::Align::Stretch);
                }
                // Its items stretch across their line unless told otherwise
                // (`align-items: normal`, CSS Flexbox §8.3): an item with an
                // `auto` cross size fills it.
                if source.kind == Some(Kind::Box) && display == "flex" && !r.has("cross") {
                    el.cross = crate::layout::Cross::Stretch;
                }
                // A `display: block` box flows its children as CSS blocks.
                el.block = initial == "box"
                    && el.direction == Direction::Column
                    && !matches!(display, "grid" | "table");
                // A block container lays its children out in its flow: the
                // flex properties do nothing on it (CSS Flexbox 1 §7, CSS
                // Box Alignment 3 §5.1, §6.1).
                if el.block
                    && source.kind == Some(Kind::Box)
                    && !(r.has("visible") && !r.flag("visible"))
                {
                    let ignored: Vec<_> = [
                        ("cross", "align-items"),
                        ("align", "justify-content"),
                        ("direction", "flex-direction"),
                    ]
                    .into_iter()
                    .filter(|(k, _)| r.has(k))
                    .map(|(_, css)| css)
                    .collect();
                    if !ignored.is_empty() {
                        self.out.warnings.push(format!(
                            "line {}: {} does nothing on a block box; give it display: flex",
                            source.line,
                            ignored.join(", ")
                        ));
                    }
                }
                // A formatting context of its own: a flow-root, a box whose
                // overflow is neither `visible` nor `clip` (CSS Overflow 3
                // §3), or one containing its layout or paint (box_of).
                let independent = ["overflowX", "overflowY"]
                    .iter()
                    .any(|p| matches!(r.str(p), Some("hidden" | "auto" | "scroll" | "overlay")));
                el.bfc = source.kind == Some(Kind::Box)
                    && (r.str("display") == Some("flowRoot") || independent)
                    || el
                        .contain
                        .has(crate::layout::Contain::LAYOUT | crate::layout::Contain::PAINT);
                el.contents =
                    source.kind == Some(Kind::Box) && r.str("display") == Some("contents");
                el.table = if source.kind == Some(Kind::Box) {
                    super::table_part(r.str("display"), r.str("captionSide"))
                } else {
                    crate::TablePart::None
                };
                // A separated table's `border-spacing` lies between its
                // cells and around them inside its padding (CSS 2 §17.6.1);
                // `collapse` has none (§17.6.2).
                if el.table == crate::TablePart::Table
                    && r.str("borderCollapse") != Some("collapse")
                    && let Some(v) = r.str("borderSpacing")
                {
                    let px: Vec<f32> = v
                        .split_whitespace()
                        .map(|w| w.trim_end_matches("px").parse().unwrap_or(0.0))
                        .collect();
                    let (h, w) = (px[0], *px.get(1).unwrap_or(&px[0]));
                    el.gaps = [Some(h), Some(w)];
                    el.padding.left += h;
                    el.padding.right += h;
                    el.padding.top += w;
                    el.padding.bottom += w;
                } else if el.table == crate::TablePart::Table {
                    el.gaps = [Some(0.0), Some(0.0)];
                }
                // A table cell's content sits in the middle of its row
                // unless `vertical-align` says otherwise (HTML's UA sheet).
                // CSS Inline 3 §4.2: `top` (and `text-top`) is the start,
                // `bottom` (and `text-bottom`) the end, `middle` the centre;
                // a baseline sits at the start.
                if el.table == crate::TablePart::Cell && !r.has("align") {
                    use crate::layout::{Baseline as B, BaselineShift as S};
                    let v =
                        r.text.as_ref().and_then(|t| t.vertical_align).or_else(|| {
                            r.str("verticalAlign").and_then(crate::VerticalAlign::parse)
                        });
                    el.align = match v {
                        None => crate::Align::Center,
                        Some(v) => match (v.shift, v.baseline) {
                            (S::Top, _) | (_, Some(B::TextTop)) => crate::Align::Start,
                            (S::Bottom, _) | (_, Some(B::TextBottom)) => crate::Align::End,
                            (S::Center, _) | (_, Some(B::Middle)) => crate::Align::Center,
                            _ => crate::Align::Start,
                        },
                    };
                }
                // Its lines' `text-align` (they lie in lines when one of
                // its children is inline-level).
                el.line_align = match (
                    r.text.as_deref().and_then(|t| t.align.as_deref()),
                    r.flow.rtl,
                ) {
                    (Some("center"), _) => crate::Align::Center,
                    (Some("end"), _) | (Some("right"), false) | (Some("left"), true) => {
                        crate::Align::End
                    }
                    _ => crate::Align::Start,
                };
                // A block-level box in a block's flow fills its width
                // (CSS 2 §10.3.3: `width: auto`); an inline-level one, a table
                // and a positioned box shrink to fit (§10.3.9, §17.5.2, §10.3.7).
                let parent = self.tree.nodes[i].parent;
                let fills = !super::outer_inline(r.str("display"))
                    && super::floated(r).is_none()
                    && !super::positioned(r)
                    && !matches!(display, "table")
                    && el.table == crate::TablePart::None;
                // The document's root element is a block of the initial
                // containing block (CSS 2 §10.1): it fills the column or
                // stack it is mounted in.
                let host = |p: &crate::layout::Node| {
                    source.root
                        && p.props.layout != crate::LayoutMode::Grid
                        && (p.props.layout == crate::LayoutMode::Stack
                            || p.el.direction == Direction::Column)
                };
                // It fills its parent's inline axis (the height in a vertical
                // writing mode); a box whose writing mode crosses its
                // parent's is an orthogonal flow, sized by its content and
                // its own inline axis (CSS Writing Modes 4 §7.3).
                let vertical = r.flow.mode.vertical();
                let across = parent != crate::layout::NONE
                    && self.tree.nodes[parent].props.flow.mode.vertical() != vertical;
                let axis = if vertical { "height" } else { "width" };
                if source.kind == Some(Kind::Box)
                    && fills
                    && !across
                    && !r.has(axis)
                    && parent != crate::layout::NONE
                    && (self.tree.nodes[parent].el.block || host(&self.tree.nodes[parent]))
                {
                    if vertical {
                        el.height = Size::grow();
                    } else {
                        el.width = Size::grow();
                    }
                }
                // An `<iframe>` is a replaced box (HTML §4.8.5): on an axis
                // CSS leaves auto it takes its natural size, 300×150, or
                // with `frame-sizing: content-…` its document's laid-out
                // content size, when that document opts in (CSS Sizing 4).
                if let Some(src) = source.frame {
                    let opted = self
                        .docs
                        .library
                        .documents
                        .values()
                        .any(|d| d.source == src && d.embedded_sizing);
                    let sizing = r.str("frameSizing").filter(|_| opted);
                    let content_width = match sizing {
                        Some("contentWidth") => true,
                        Some("contentInlineSize") => !vertical,
                        Some("contentBlockSize") => vertical,
                        _ => false,
                    };
                    let content_height = match sizing {
                        Some("contentHeight") => true,
                        Some("contentBlockSize") => !vertical,
                        Some("contentInlineSize") => vertical,
                        _ => false,
                    };
                    if !r.has("width") {
                        el.width = if content_width {
                            Size::fit()
                        } else {
                            Size::Fixed(300.0)
                        };
                    }
                    if !r.has("height") {
                        el.height = if content_height {
                            Size::fit()
                        } else {
                            Size::Fixed(150.0)
                        };
                    }
                }
            }
        } else {
            for name in [
                "height",
                "padding",
                "paddingTop",
                "paddingRight",
                "paddingBottom",
                "paddingLeft",
                "marginTop",
                "marginRight",
                "marginBottom",
                "marginLeft",
                "gap",
                "align",
                "cross",
                "background",
                "radius",
                "aspect",
                "interactive",
                "focusable",
                "adjustable",
                "wrap",
                "columnSpan",
                "contain",
                "contentVisibility",
                "resize",
                "overflowClipMargin",
                "breakBefore",
                "breakAfter",
                "breakInside",
                "boxDecorationBreak",
                "orphans",
                "widows",
            ] {
                if r.has(name) {
                    el = Self::box_of(
                        &Resolved {
                            vals: r
                                .vals
                                .iter()
                                .filter(|(s, _)| PROPS[*s].name == name)
                                .cloned()
                                .collect(),
                            ..Default::default()
                        },
                        el,
                        em,
                    );
                }
            }
            if r.has("width")
                && !matches!(
                    source.kind,
                    Some(Kind::Label | Kind::Button | Kind::Image | Kind::Art)
                )
            {
                el.width = r.size("width").unwrap_or(el.width);
            }
        }
        // `display: inline-*` (any element): it lies in its parent's lines,
        // and `vertical-align` places it there.
        el.inline = super::outer_inline(r.str("display"));
        // `float` and `clear` (CSS 2 §9.5), their sides in layout space:
        // a right-to-left containing block's content (its floats with it)
        // ends up mirrored, by its flip or the surface's mirror
        // (`layout::flow`), so `inline-start` lays out on the left and,
        // in a right-to-left one, `left` on the right. A float is
        // block-level.
        let parent = self.tree.nodes[i].parent;
        let rtl = if parent == crate::layout::NONE {
            r.flow.rtl
        } else {
            self.tree.nodes[parent].props.flow.rtl
        };
        el.float = match r.str("float") {
            Some("left") if rtl => crate::layout::FloatSide::Right,
            Some("right") if !rtl => crate::layout::FloatSide::Right,
            Some("inline-end") => crate::layout::FloatSide::Right,
            Some("left" | "right" | "inline-start") => crate::layout::FloatSide::Left,
            _ => crate::layout::FloatSide::None,
        };
        self.tree.nodes[i].shape_outside = r
            .str("shapeOutside")
            .and_then(|s| crate::doc::paint_css::outside(s).ok().flatten())
            .and_then(|value| {
                use crate::doc::paint_css::{CssClip, GeoBox, OutsideShape};
                let (shape, reference, image) = match value {
                    OutsideShape::Shape(shape, reference) => (shape, reference, None),
                    OutsideShape::Image(path) => {
                        let Some(alpha) = self.docs.library.alphas.get(path.trim_start_matches('/'))
                        else {
                            // An image that cannot be read is no shape
                            // (CSS Shapes 1 §6.1): the float's box wraps.
                            self.out.warnings.push(format!(
                                "shape-outside: url({path}) has no image alpha; the float's box is its shape"
                            ));
                            return None;
                        };
                        let threshold = r
                            .str("shapeImageThreshold")
                            .and_then(|s| crate::doc::paint_css::shape_threshold(s).ok())
                            .unwrap_or(0.0);
                        let image = crate::layout::float::ShapeImage {
                            alpha: alpha.clone(),
                            threshold,
                        };
                        (CssClip::Box, GeoBox::Content, Some(image))
                    }
                };
                Some(crate::layout::float::Outside {
                    shape,
                    reference,
                    image,
                    margin: r
                        .str("shapeMargin")
                        .and_then(|s| crate::doc::paint_css::shape_margin(s).ok())
                        .unwrap_or(crate::doc::paint_css::Pos::Px(0.0)),
                    rtl,
                })
            });
        el.clear = match r.str("clear") {
            Some("both") => crate::layout::float::LEFT | crate::layout::float::RIGHT,
            Some("left") if rtl => crate::layout::float::RIGHT,
            Some("right") if !rtl => crate::layout::float::RIGHT,
            Some("inline-end") => crate::layout::float::RIGHT,
            Some("left" | "right" | "inline-start") => crate::layout::float::LEFT,
            _ => 0,
        };
        if el.float != crate::layout::FloatSide::None {
            el.inline = false;
        }
        // A block-level text (`<p>`, a heading, a text displayed `block`)
        // in a block's flow fills its line, as a CSS block box does (CSS 2
        // §10.3.3); an inline-level one keeps fitting its text.
        if source.kind == Some(Kind::Label) {
            el.block = match r.str("display") {
                d @ Some(_) if super::outer_inline(d) => false,
                Some("block" | "flowRoot" | "listItem") => true,
                _ => matches!(
                    source.tag,
                    Some("p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
                ),
            };
            let parent = self.tree.nodes[i].parent;
            // Its inline axis: down a vertical line (CSS Writing Modes 4);
            // across its parent's writing mode it is an orthogonal flow.
            let vertical = r.flow.mode.vertical();
            let axis = if vertical { "height" } else { "width" };
            if el.block
                && el.float == crate::layout::FloatSide::None
                && !super::positioned(r)
                && !r.has(axis)
                && !r.has("textPath")
                && parent != crate::layout::NONE
                && self.tree.nodes[parent].el.block
                && self.tree.nodes[parent].props.flow.mode.vertical() == vertical
            {
                if vertical {
                    el.height = Size::grow();
                } else {
                    el.width = Size::grow();
                }
            }
        }
        el.dominant = r.text.as_ref().map(|t| t.dominant).unwrap_or_default();
        if el.inline {
            // `top` and `bottom` against the line box; the others against
            // the line's baseline (`Ui::align_baselines`).
            use crate::layout::{BaselineShift as S, BaselineSource};
            let mut v = r
                .text
                .as_ref()
                .and_then(|t| t.vertical_align)
                .or_else(|| r.str("verticalAlign").and_then(crate::VerticalAlign::parse))
                .unwrap_or_default();
            // CSS Inline 3 §4.2.1: `auto` is an inline block's last
            // baseline, any other box's first.
            if v.source == BaselineSource::Auto {
                v.source = if r.str("display") == Some("inlineBlock") {
                    BaselineSource::Last
                } else {
                    BaselineSource::First
                };
            }
            el.valign = v;
            let plain = crate::VerticalAlign {
                source: v.source,
                ..Default::default()
            };
            el.align_self = match v.shift {
                S::Top => Some(crate::Cross::Start),
                S::Bottom => Some(crate::Cross::End),
                _ if v == plain => el.align_self,
                _ => Some(crate::Cross::Baseline),
            };
        }
        let mut warn = Vec::new();
        let mut decor = self.box_decor(source.document, r, &mut el, source.kind, &mut warn);
        for w in warn {
            self.out.warnings.push(format!("line {}: {w}", source.line));
        }
        let mut el = Self::sized(r, Self::item_of(r, el));
        // A flex item that declares no `flex` is CSS's initial `0 1 auto`;
        // a box the engine sizes to grow (a control filling its row) keeps
        // its kind.
        if el.flex.is_none()
            && !matches!(el.width, Size::Grow { .. })
            && !matches!(el.height, Size::Grow { .. })
        {
            el.flex = Some(crate::layout::FlexItem::default());
        }
        // A clip's reference box from the laid-out box: its insets from
        // the border box and the corner it takes (CSS Masking 1 §4.1; the
        // border width pads the element, so padding holds it).
        if let Some(d) = decor.as_mut()
            && let Some(c) = d.css.as_mut()
        {
            use crate::doc::paint_css::GeoBox;
            let borders = d.borders.map(|b| b.map_or(0.0, |b| b.width));
            let p = el.padding;
            let m = el.margin.map(|m| m.intrinsic());
            let insets = match c.clip_box {
                GeoBox::Border => [0.0; 4],
                GeoBox::Padding => borders,
                GeoBox::Content => [p.top, p.right, p.bottom, p.left],
                GeoBox::Margin => m.map(|v| -v),
            };
            let grow = -insets.iter().copied().fold(f32::INFINITY, f32::min);
            c.clip_ref = (
                insets,
                if el.radius > 0.0 {
                    (el.radius + grow).max(0.0)
                } else {
                    0.0
                },
            );
        }
        let surface_rtl = self.env.direction == og_i18n::Direction::Rtl;
        let flow = r.own_flow();
        // A border shape's reference boxes, and the side it takes its
        // width and colour from (CSS Borders 4 §7.3, §7.6).
        if let Some(d) = decor.as_mut()
            && let Some(bs) = d.css.as_mut().and_then(|c| c.border_shape.as_mut())
        {
            use crate::doc::paint_css::GeoBox;
            let borders = d.borders.map(|b| b.map_or(0.0, |b| b.width));
            let p = el.padding;
            let m = el.margin.map(|m| m.intrinsic());
            for (_, geo, insets) in &mut bs.shapes {
                *insets = match geo {
                    None => borders.map(|b| b / 2.0),
                    Some(GeoBox::Border) => [0.0; 4],
                    Some(GeoBox::Padding) => borders,
                    Some(GeoBox::Content) => [p.top, p.right, p.bottom, p.left],
                    Some(GeoBox::Margin) => m.map(|v| -v),
                };
            }
            // Its frame's sides: block-start, inline-end, block-end,
            // inline-start; right to left, the inline ones swap.
            let s = flow.mode.sides().map(usize::from);
            let (start, end) = if flow.rtl { (s[1], s[3]) } else { (s[3], s[1]) };
            let order = [s[0], start, s[2], end];
            bs.side = order
                .into_iter()
                .find(|&k| d.borders[k].is_some())
                .unwrap_or(order[0]);
        }
        // A vertical element's inline axis runs down: the surface's mirror
        // turns its block axis round instead (`vertical-rl` lays out as
        // `vertical-lr` before the mirror).
        el.flip = if flow.mode.vertical() {
            flow.rtl
        } else {
            flow.rtl != surface_rtl
        };
        el.vertical = flow.mode.vertical().then(|| {
            flow.mode
                .sides()
                .map(|s| if surface_rtl && s % 2 == 1 { 4 - s } else { s })
        });
        if surface_rtl {
            crate::layout::flow::mirror_el(&mut el);
        }
        if let Some(line) = field_line {
            // At least a caret along the inline axis and a line across it:
            // in a vertical writing mode the content-sized inline axis is
            // the height and a line's thickness is the width.
            let (across, down) = if el.vertical.is_some() {
                (line, 1.0)
            } else {
                (1.0, line)
            };
            if let Size::Fit { min, .. } = &mut el.width {
                *min = min.max(el.padding.horizontal() + across);
            }
            if let Size::Fit { min, .. } = &mut el.height {
                *min = min.max(el.padding.vertical() + down);
            }
        }
        *self.tree.el_mut(i) = el;
        self.tree.nodes[i].decor = decor.map(Box::new);
    }

    fn apply_document_semantics(&mut self, n: &DocNode, r: &Resolved, base: &RenderFields) {
        let i = base.index;
        let surface_rtl = self.env.direction == og_i18n::Direction::Rtl;
        let flow = r.own_flow();
        let mut props = self.resolved_props(n, r);
        if surface_rtl {
            crate::layout::flow::mirror_props(&mut props);
        }
        if r.flag("dataOgInstant") {
            // `data-og-instant`: its first entrance lands settled (the
            // screen behind the web's play gate); the motion state then
            // knows it, so a later build does not replay it, and a later
            // entrance (after it left) plays.
            if self.docs.state.entered.insert(self.tree.nodes[i].key)
                && let Some(e) = props.enter.as_mut()
            {
                e.transition = crate::motion::Transition::Instant;
                e.delay_ms = 0.0;
            }
        }
        props.states = base.document_props.states.clone();
        props.state = base.document_props.state.clone();
        if props != Props::default() {
            props.states.extend(
                base.widget_props
                    .states
                    .iter()
                    .filter(|g| !n.groups.iter().any(|x| x.name == g.name))
                    .cloned(),
            );
            *self.tree.props_mut(i) = props;
        }
        self.tree.props_mut(i).flow = flow;
        // A vertical text sets its lines down: a plain text by its own
        // setting, a rich text by its node's (`Ui::rich_vertical`).
        // Text on a path follows its path's physical geometry: its box is
        // the path's extent, whatever the writing mode; its glyphs are set
        // as a vertical line's along the path.
        let build = self.tree.build();
        let node = &mut self.tree.nodes[i];
        match node.content.write(build) {
            crate::layout::Content::Text(t) if node.props.text_path.is_some() => {
                node.el.write(build).vertical = None;
                t.vertical = flow.vertical_text(&t.text);
            }
            crate::layout::Content::Text(t) => t.vertical = flow.vertical_text(&t.text),
            _ => {}
        }
        // The roles and levels HTML's tags give (landmarks, lists, headings).
        if let Some(role) = n.role {
            self.tree.nodes[i].a11y.role = Some(role);
        }
        if n.level > 0 {
            self.tree.nodes[i].a11y.level = Some(n.level);
        }
        // A split screen's region (`Props::player`).
        if let Some(k) = r.num("player") {
            self.tree.props_mut(i).player = Some(k.clamp(0.0, 3.0) as u8);
        }
        // `role`, `aria-live` and `aria-atomic` over the tag's.
        if let Some(role) = r
            .str("role")
            .and_then(|r| crate::a11y::ROLES.iter().find(|&&k| k == r))
        {
            self.tree.nodes[i].a11y.role = Some(role);
        }
        self.tree.nodes[i].a11y.live = r.str("ariaLive").and_then(crate::a11y::Live::parse);
        self.tree.nodes[i].a11y.atomic = r.get("ariaAtomic").map(|_| r.flag("ariaAtomic"));
        // `aria-label` (or a `<label for>`) names it over what it shows.
        if let Some(name) = r.str("name").filter(|n| !n.is_empty()) {
            self.tree.nodes[i].a11y.name = Some(name.to_string());
        }
        self.tree.apply_aria(i, r);
        // The part its `part` attribute names, over the control's own: the
        // control's states, variant, class and motion stay.
        if let Some(named) = r.part {
            let el = self.tree.el_mut(i);
            el.part = Some(match el.part {
                Some(own) => {
                    // A rebuild applies the named part again over itself:
                    // the control's own part is the one it kept.
                    let base = if own.own.is_empty() {
                        own.part
                    } else {
                        own.own
                    };
                    let mut p = crate::dress::PartRef {
                        component: named.component,
                        part: named.part,
                        own: if base == named.part { "" } else { base },
                        ..own
                    };
                    for f in named.flags() {
                        p = p.flag(f);
                    }
                    for (k, v) in named.attrs.iter().filter(|(k, _)| !k.is_empty()) {
                        p = p.attr(k, v);
                    }
                    p
                }
                None => named,
            });
        }
        if !self.tree.building {
            self.apply_part_appearance(i);
            self.layout
                .prepare_layout_node(i, &self.env, &mut self.tree);
        }
    }
}

impl crate::ui::Tree {
    /// A label's CSS `text-wrap-style`, `overflow-wrap`, `text-align-last`,
    /// `text-justify` and `text-overflow: <string>`.
    pub fn css_text_content(
        &mut self,
        i: usize,
        r: &Resolved,
        sheet: &crate::css::at_rules::Sheet,
    ) {
        use crate::layout::WrapStyle;
        let style = match r.text.as_deref().and_then(|t| t.wrap_style.as_deref()) {
            Some("balance") => WrapStyle::Balance,
            Some("pretty") => WrapStyle::Pretty,
            _ => WrapStyle::Auto,
        };
        use crate::layout::OverflowWrap;
        let overflow_wrap = match r.text.as_deref() {
            Some(t) if t.word_break.as_deref() == Some("break-word") => OverflowWrap::Anywhere,
            Some(t) => match t.overflow_wrap.as_deref() {
                Some("anywhere") => OverflowWrap::Anywhere,
                Some("break-word") => OverflowWrap::BreakWord,
                _ => OverflowWrap::Normal,
            },
            None => OverflowWrap::Normal,
        };
        let align_last = r.text_align_last(r.own_flow().rtl);
        let justify = match r.text.as_deref().and_then(|t| t.justify.as_deref()) {
            Some("none") => crate::layout::TextJustify::None,
            Some("inter-word") => crate::layout::TextJustify::InterWord,
            Some("inter-character" | "distribute") => crate::layout::TextJustify::InterCharacter,
            _ => crate::layout::TextJustify::Auto,
        };
        let cut = r
            .str("ellipsisText")
            .filter(|s| !s.is_empty())
            .map(str::to_owned);
        let synthesize_bold = r.text.as_deref().and_then(|t| t.synthesis[0]) != Some(false);
        let css = r.text.as_deref();
        let stretch = css.and_then(|t| t.stretch);
        let variations = css.map_or_else(Default::default, |t| {
            crate::shape::Variations::new(&t.variations)
        });
        let optical_sizing = css.and_then(|t| t.optical_sizing) != Some(false);
        let palette = css.map_or(Default::default(), |t| {
            sheet.palette(t.palette.as_deref(), t.family.as_deref())
        });
        let emoji = css.and_then(|t| t.emoji).unwrap_or_default();
        let size_adjust = css.and_then(|t| t.size_adjust).flatten();
        if let crate::layout::Content::Text(t) = self.content_mut(i) {
            t.emoji = emoji;
            t.size_adjust = size_adjust;
            t.synthesize_bold = synthesize_bold;
            t.stretch = stretch;
            t.variations = variations;
            t.optical_sizing = optical_sizing;
            t.palette = palette;
            t.wrap_style = style;
            t.overflow_wrap = overflow_wrap;
            t.align_last = align_last;
            t.justify = justify;
            t.cut = cut;
        }
    }

    /// An `<image>`'s CSS `object-fit`, `object-position`,
    /// `image-rendering` and `image-orientation`, checked when the CSS was
    /// lowered.
    pub fn object_fit(&mut self, i: usize, r: &Resolved) {
        let (at, nearest, orient) = r.object_css();
        let fit = match r.str("fit") {
            Some("contain") => og_draw::UiFit::Contain,
            Some("cover") => og_draw::UiFit::Cover,
            Some("none") => og_draw::UiFit::None,
            Some("scaleDown") => og_draw::UiFit::ScaleDown,
            _ => og_draw::UiFit::Fill,
        };
        if let crate::layout::Content::Image { object, .. } = self.content_mut(i) {
            *object = crate::layout::ObjectFit {
                fit,
                focal: at.map_or([0.5; 2], |a| a[0]),
                offset: at.map_or([0.0; 2], |a| a[1]),
                nearest,
                orient,
            };
        }
    }
}

impl crate::ui::Tree {
    /// An element's ARIA attributes over what its kind tells the tree
    /// (docs/ui.md §2): a role, a description, a value and its range, and
    /// states. Applied with its other fields, so a bound one changes the
    /// tree only when its element is rebuilt.
    fn apply_aria(&mut self, i: usize, r: &Resolved) {
        if let Some(role) = r.str("role")
            && let Some(&k) = crate::a11y::ROLES.iter().find(|&&k| k == role)
        {
            self.nodes[i].a11y.role = Some(k);
        }
        if r.flag("ariaDisabled") {
            self.nodes[i].disabled = true;
        }
        let a = &mut self.nodes[i].a11y;
        if let Some(d) = r.str("ariaDescription").filter(|d| !d.is_empty()) {
            a.description = Some(d.to_string());
        }
        if let Some(v) = r.str("ariaValuetext") {
            a.value = Some(v.to_string());
        }
        if let Some(now) = r.num("ariaValuenow") {
            a.range = Some(crate::a11y::A11yRange {
                min: r.num("ariaValuemin"),
                max: r.num("ariaValuemax"),
                now,
            });
        }
        let state = |name: &str| r.has(name).then(|| r.flag(name));
        a.checked = state("ariaChecked").or(a.checked);
        a.selected = state("ariaSelected").or(a.selected);
        a.expanded = state("ariaExpanded").or(a.expanded);
        a.pressed = state("ariaPressed").or(a.pressed);
        a.busy |= r.flag("ariaBusy");
        if let Some(level) = r.num("ariaLevel").filter(|l| *l >= 1.0) {
            a.level = Some(level.min(255.0) as u8);
        }
        a.current = r.str("ariaCurrent").is_some_and(|c| c != "false");
    }
}

impl Ui {
    /// Resolves the colours in `contextual` (their places in `r.vals`)
    /// in the element: `currentColor` is its `color`, declared or
    /// inherited (CSS Color 4 §6.4), and `light-dark()` takes its used
    /// colour scheme (CSS Color 5 §2, its `color-scheme` or the one it
    /// inherits). `color` itself goes first: its `currentColor` is the
    /// inherited colour, and the others' is it.
    pub(super) fn resolve_element_colors(
        &self,
        r: &mut Resolved,
        contextual: &[(usize, Arc<crate::css_color::Color>)],
    ) {
        use crate::doc::text_css;
        let theme = self.theme();
        let key = text_css::key("color");
        let inherited = r
            .cascade
            .computed
            .get(&key)
            .cloned()
            .flatten()
            .and_then(|v| crate::style_file::parse_color(&v));
        let scheme = r
            .cascade
            .text
            .iter()
            .rev()
            .find(|(n, _)| n == "color-scheme")
            .map(|(_, v)| v.clone())
            .or_else(|| {
                r.cascade
                    .computed
                    .get(&text_css::key("color-scheme"))
                    .cloned()
                    .flatten()
            });
        let style_dark = theme.scheme == crate::Scheme::Dark;
        let dark = scheme.map_or(style_dark, |v| text_css::scheme_dark(&v, style_dark));
        let mut cx = crate::css_color::Context {
            current: inherited.unwrap_or(theme.palette.ink),
            dark,
            ..crate::css_color::Context::of(theme)
        };
        let color = schema::prop("color").map(|(spec, _)| spec);
        let resolve = |r: &mut Resolved, cx: &crate::css_color::Context, of_color: bool| {
            for (i, c) in contextual {
                if (Some(r.vals[*i].0) == color) == of_color {
                    r.vals[*i].1 = V::Color(c.resolve(cx));
                }
            }
        };
        resolve(r, &cx, true);
        let own = r.color("color");
        cx.current = own.or(inherited).unwrap_or(theme.palette.ink);
        resolve(r, &cx, false);
        // What it draws in an inherited colour follows the parent's.
        if own.is_none() && inherited.is_some() {
            r.cascade.inherited_reads.insert(key);
        }
        if contextual.iter().any(|(_, c)| c.uses_scheme()) {
            r.cascade
                .inherited_reads
                .insert(text_css::key("color-scheme"));
        }
        r.used_color = Some((cx.current, dark));
    }
}

impl Resolved {
    /// Its `currentColor`: its `color`, or the one it inherits, or the
    /// style's ink.
    pub(crate) fn current(&self, theme: &crate::Theme) -> Srgba {
        self.color("color")
            .or(self.used_color.map(|(c, _)| c))
            .unwrap_or(theme.palette.ink)
    }

    /// The context its CSS colours resolve in ([`Self::current`], its used
    /// colour scheme).
    pub(crate) fn color_context<'a>(
        &self,
        theme: &'a crate::Theme,
    ) -> crate::css_color::Context<'a> {
        crate::css_color::Context {
            current: self.current(theme),
            dark: self
                .used_color
                .map_or(theme.scheme == crate::Scheme::Dark, |(_, d)| d),
            ..crate::css_color::Context::of(theme)
        }
    }

    pub fn intrinsic(&self) -> Self {
        Self {
            vals: self
                .vals
                .iter()
                .filter(|(spec, _)| {
                    let name = PROPS[*spec].name;
                    !super::super::css::STYLE.iter().any(|(p, _)| *p == name)
                })
                .cloned()
                .collect(),
            text: self.text.clone(),
            ..Self::default()
        }
    }

    /// Its CSS `text-align`, when one is set or inherited. CSS Text 3
    /// §6.1: left and right are physical; start and end follow the
    /// direction (the line drawer mirrors Start and End).
    fn text_align(&self, rtl: bool) -> Option<TextAlign> {
        let a = self
            .text
            .as_deref()
            .and_then(|t| t.align.as_deref())
            .or(self.str("textAlign"))?;
        Some(text_align_of(a, rtl))
    }

    /// Its CSS `text-align-last` (CSS Text 3 §6.3): `auto` is `text-align`,
    /// but start for `justify`; `justify-all` justifies the last line too.
    fn text_align_last(&self, rtl: bool) -> Option<TextAlign> {
        let t = self.text.as_deref()?;
        match t.align_last.as_deref() {
            Some("auto") | None => {
                (t.align.as_deref() == Some("justify-all")).then_some(TextAlign::Justify)
            }
            Some(a) => Some(text_align_of(a, rtl)),
        }
    }

    pub fn label_desc(&self, theme: &crate::Theme, fonts: &crate::Fonts, rtl: bool) -> LabelDesc {
        let step = self
            .str("step")
            .and_then(|s| crate::style_file::ramp_step(&mut theme.type_ramp.clone(), s).copied());
        let css = self.text.as_deref();
        let font = {
            let role = self
                .str("font")
                .and_then(FontRole::from_name)
                .or_else(|| css.and_then(|t| t.font_role))
                .unwrap_or_default();
            // A named family a document declares, the role after it.
            match css.and_then(|t| t.family.as_deref()) {
                Some(names) => fonts.family_list(names, role),
                None => role,
            }
        };
        let step = match css {
            Some(t) => {
                // `line-height: normal` in a face whose @font-face sets its
                // line metrics is theirs.
                let query = crate::FontQuery::new(t.weight.unwrap_or(0), t.italic == Some(true));
                let normal = fonts.for_styled(font, query, "").0.metrics.normal_line;
                Some(t.step_in(step.unwrap_or(theme.type_ramp.body), normal))
            }
            None => step,
        };
        let align = self.text_align(rtl).unwrap_or_default();
        LabelDesc {
            step,
            color: self.color("color"),
            on_scene: false,
            uppercase: self.flag("uppercase"),
            align,
            nowrap: css.and_then(|t| t.nowrap).unwrap_or(self.flag("nowrap")),
            width: self.size("width").unwrap_or_default(),
            font,
            heading: self.flag("heading"),
            ellipsis: self.flag("ellipsis"),
            max_lines: self.num("maxLines").unwrap_or(0.0) as u32,
            fit_min: self.str("fitMin").and_then(|s| {
                crate::style_file::ramp_step(&mut theme.type_ramp.clone(), s).copied()
            }),
        }
    }

    /// The rich text a label of `desc` lays out as: its paragraph's
    /// `text-indent` and `line-break` too.
    pub fn rich_desc(&self, desc: &LabelDesc) -> crate::RichTextDesc {
        let (indent, line_break) = self
            .text
            .as_deref()
            .map(|t| t.paragraph())
            .unwrap_or_default();
        let css = self.text.as_deref();
        crate::RichTextDesc {
            step: desc.step,
            color: desc.color,
            align: desc.align,
            nowrap: desc.nowrap,
            width: desc.width,
            font: desc.font,
            indent,
            line_break,
            emphasis: css.and_then(|t| t.emphasis),
            dominant: css.map(|t| t.dominant).unwrap_or_default(),
            ruby: css.map(|t| t.ruby).unwrap_or_default(),
            spacing: css.map(|t| t.spacing).unwrap_or_default(),
            hanging: css.map(|t| t.hanging).unwrap_or_default(),
            text_box: css.map(|t| t.text_box).unwrap_or_default(),
        }
    }

    /// Its text as one span styled by its CSS (`font-style`,
    /// `text-decoration`, `-webkit-text-stroke`, `-webkit-text-fill-color`,
    /// CJK spacing, hanging punctuation) when the plain text path cannot
    /// draw that style, or small capitals or super- and subscripts its face
    /// of `desc` lacks the forms of (rich text synthesises them).
    pub fn css_span(
        &self,
        text: &str,
        sheet: &crate::css::at_rules::Sheet,
        fonts: &crate::Fonts,
        desc: &LabelDesc,
    ) -> Option<(crate::TextSpan, crate::TextSpanStyle)> {
        let t = self.text.as_deref()?;
        // CJK spacing and hanging punctuation too, which the plain text
        // path does not set.
        if !t.rich()
            && !t.spacing.applies(text, t.language.as_deref())
            && t.hanging == Default::default()
        {
            // A label's CSS sets its step (`label_desc`).
            let step = desc.step?;
            let query = crate::FontQuery::new(t.weight.unwrap_or(step.weight), false);
            let font = fonts.for_styled(desc.font, query, text).0;
            let allow = self
                .text_style(t, sheet)
                .synthesis
                .unwrap_or(crate::Synth::ALL);
            if crate::rich::synthetic(text, step.features, font, allow).is_empty() {
                return None;
            }
        }
        let span = crate::TextSpan {
            name: "css".into(),
            position: 0,
            length: text.chars().count() as u32,
            properties: Default::default(),
        };
        Some((span, self.text_style(t, sheet)))
    }

    /// The run style its text CSS `t` gives: its font's style, weight,
    /// width, variations, palette and synthesis, its fill, stroke,
    /// decoration and word spacing.
    pub fn text_style(
        &self,
        t: &crate::css::text_css::TextCss,
        sheet: &crate::css::at_rules::Sheet,
    ) -> crate::TextSpanStyle {
        crate::TextSpanStyle {
            italic: t.italic,
            weight: t.weight,
            oblique: t.slant,
            stretch: t.stretch,
            variations: (!t.variations.is_empty())
                .then(|| crate::shape::Variations::new(&t.variations)),
            optical_sizing: t.optical_sizing,
            palette: t
                .palette
                .is_some()
                .then(|| sheet.palette(t.palette.as_deref(), t.family.as_deref())),
            emoji: t.emoji,
            size_adjust: t.size_adjust,
            synthesis: t
                .synthesis
                .iter()
                .any(Option::is_some)
                .then(|| crate::Synth {
                    bold: t.synthesis[0] != Some(false),
                    italic: t.synthesis[1] != Some(false),
                    small_caps: t.synthesis[2] != Some(false),
                    position: t.synthesis[3] != Some(false),
                }),
            color: t.fill.map(crate::paint::ColorRef::Literal),
            // A stroke centred on the glyphs' edge, over the fill (Compat
            // §5.4); `paint-order: stroke` paints it under, so only its
            // outer half shows. Its colour defaults to `color`.
            outline: t.stroke.map(|(w, c)| {
                let stroke_first = t.paint_order.as_deref().is_some_and(|o| {
                    o.split_whitespace()
                        .find(|w| *w == "fill" || *w == "stroke")
                        == Some("stroke")
                });
                let color = crate::paint::ColorRef::Literal(
                    c.or(self.color("color"))
                        .unwrap_or(crate::Srgba::rgb(0, 0, 0)),
                );
                if stroke_first {
                    (w / 2.0, color, og_draw::UiAlign::Outside)
                } else {
                    (w, color, og_draw::UiAlign::Center)
                }
            }),
            decoration: t.decoration.any().then(|| t.decoration.clone()),
            hyphen: t.hyphen.as_deref().map(Into::into),
            word_spacing: t.word_spacing.filter(|w| *w != 0.0),
            ..Default::default()
        }
    }

    pub fn button_desc(&self) -> ButtonDesc {
        ButtonDesc {
            variant: match self.str("variant") {
                Some("primary") => ButtonVariant::Primary,
                Some("quiet") => ButtonVariant::Quiet,
                Some("danger") => ButtonVariant::Danger,
                Some("ghost") => ButtonVariant::Ghost,
                _ => ButtonVariant::Secondary,
            },
            size: Default::default(),
            disabled: self.flag("disabled"),
            width: self.size("width"),
            autofocus: self.flag("autofocus"),
            square: self.flag("square"),
            align: self.text_align(self.flow.rtl),
            part: self.part,
        }
    }
}

/// A CSS `text-align` keyword: left and right are physical, start and end
/// follow the direction (the line drawer mirrors Start and End).
fn text_align_of(a: &str, rtl: bool) -> TextAlign {
    match a {
        "center" => TextAlign::Center,
        "end" => TextAlign::End,
        "left" if rtl => TextAlign::End,
        "right" if !rtl => TextAlign::End,
        "justify" | "justify-all" => TextAlign::Justify,
        _ => TextAlign::Start,
    }
}

/// A file input's `::file-selector-button` label while its text says
/// nothing.
pub const FILE_LABEL: &str = "Choose file…";

/// What a file input shows beside its button while it names no file.
pub const NO_FILE: &str = "No file chosen";
