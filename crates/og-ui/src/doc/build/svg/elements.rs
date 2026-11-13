use super::*;
use crate::doc::schema::Kind;

impl Ui {
    /// The elements the `feImage`s of the open element's filters name
    /// (`uses`, numbered as [`crate::paint::filter_uses`] numbers them),
    /// each built as `<use>` would draw it (Filter Effects 1 §15.17): in
    /// the filtered element's user space, moved by its `feImage`'s `x` and
    /// `y`, inheriting from the filtered element. Each is a child of the
    /// open element under an anonymous box whose draw is its element image
    /// (`ElementDraw::element_image`), so it draws everything the element
    /// draws (gradients, patterns, text, images, groups, `<use>`) into a
    /// layer only the filter reads, takes no input and is in no tree a
    /// reader walks. `of` is the filtered element's node, `computed` its
    /// computed values.
    pub(crate) fn build_element_images(
        &mut self,
        b: &Build<'_>,
        of: &DocNode,
        computed: &crate::doc::variables::Computed,
        uses: &[&crate::paint::FilterUse],
        space: ImageSpace,
    ) {
        if self.docs.state().element_images >= 4 {
            return;
        }
        let doc = b.doc.clone();
        let Some(of) = doc.nodes.iter().position(|d| std::ptr::eq(d, of)) else {
            return;
        };
        let mut n = 0u32;
        for u in uses {
            for (k, id) in u.filter.elements.iter().enumerate() {
                let at = n;
                n += 1;
                let Some(target) = doc.nodes.iter().position(|d| {
                    d.id == *id
                        && matches!(d.kind, NodeKind::Element(k) if k.svg() && k != Kind::SvgDef)
                }) else {
                    // A missing element: the filter reads nothing there.
                    continue;
                };
                let mut up = of;
                let mut inside = false;
                while up != crate::doc::NONE {
                    inside |= up == target;
                    up = doc.nodes[up].parent;
                }
                if inside {
                    self.out.warnings_mut().push(format!(
                        "{}:{}: <feImage href=\"#{id}\"> draws an element the filter is on or inside",
                        doc.source, doc.nodes[of].line
                    ));
                    continue;
                }
                // Its feImage's x and y, in the filter's primitive units.
                let layer = crate::svg::filter::element_layer(k as u32);
                let sub = u
                    .filter
                    .primitives
                    .iter()
                    .find(|p| matches!(p.op, og_draw::UiFilterOp::Layer { layer: l } if l == layer))
                    .map_or([f32::NAN; 4], |p| p.subregion);
                let bbox = match space {
                    ImageSpace::Svg(_, bbox) => bbox,
                    ImageSpace::Css => [0.0; 4],
                };
                // (user units, fraction of the box) per axis.
                let shift = |v: f32, axis: usize| match (v.is_nan(), u.filter.primitive_bbox) {
                    (true, _) => (0.0, 0.0),
                    (false, true) => (bbox[axis], v),
                    (false, false) => (v, 0.0),
                };
                let (sx, sy) = (shift(sub[0], 0), shift(sub[1], 1));
                let mut inherited = computed.clone();
                self.open(
                    format!("~image{at}"),
                    match space {
                        ImageSpace::Svg(..) => Element {
                            width: Size::grow(),
                            height: Size::grow(),
                            ..Element::default()
                        },
                        ImageSpace::Css => Element {
                            width: Size::px(1.0),
                            height: Size::px(1.0),
                            ..Element::default()
                        },
                    },
                );
                let i = self.tree.parent();
                let p = self.tree.props_mut(i);
                p.position = crate::element::Position::Absolute;
                p.inert = true;
                p.interact.pointer_events = crate::element::PointerEvents::None;
                match space {
                    ImageSpace::Svg(t, _) => {
                        // Over the whole `<svg>`, as its shapes are.
                        p.insets = crate::element::Edges {
                            start: Some(0.0),
                            top: Some(0.0),
                            end: Some(0.0),
                            bottom: Some(0.0),
                            ..Default::default()
                        };
                        let to = Affine([
                            1.0,
                            0.0,
                            0.0,
                            1.0,
                            sx.0 + sx.1 * bbox[2],
                            sy.0 + sy.1 * bbox[3],
                        ]);
                        inherited.insert(TRANSFORM.into(), Some(matrix(&t.then(&to))));
                    }
                    ImageSpace::Css => {
                        // A one-px view box at the box's top left (moved
                        // by the feImage): one user unit is a CSS px.
                        use crate::layout::anchor::{AnchorSpec, Inset};
                        let inset = |(px, q): (f32, f32)| Some(Inset::Percent(q, px));
                        p.anchor = Some(Box::new(AnchorSpec {
                            base: crate::layout::anchor::Placement {
                                insets: [inset(sy), None, None, inset(sx)],
                                ..Default::default()
                            },
                            ..Default::default()
                        }));
                        inherited.insert(VIEW_BOX.into(), Some("0 0 1 1".into()));
                        inherited.insert(TRANSFORM.into(), Some(matrix(&Affine::IDENTITY)));
                    }
                }
                self.set_draw(ElementDraw {
                    element_image: Some(at),
                    ..ElementDraw::default()
                });
                let mut ib = Build {
                    doc: doc.clone(),
                    mount: b.mount.clone(),
                    vm: b.vm,
                    env: b.env,
                    locals: b.locals.clone(),
                    row_key: b.row_key.clone(),
                    inherited: std::sync::Arc::new(inherited),
                    events: Vec::new(),
                    writes: Vec::new(),
                    cached: None,
                    recording: Default::default(),
                };
                self.docs.state_mut().element_images += 1;
                self.build_node(&mut ib, target, None);
                self.docs.state_mut().element_images -= 1;
                self.close();
            }
        }
    }

    /// An SVG `<text>`: one line shaped as any label, at its view-box size,
    /// placed and scaled by layout through [`crate::element::ViewPlace`].
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build_svg_text(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &str,
        r: &Resolved,
        props: Props,
        path: &str,
        key: u64,
        vb: [f32; 4],
    ) {
        let computed = r.cascade.computed.clone();
        let value = |attr: &str| computed.get(&self::key(attr)).cloned().flatten();
        let mut warnings = Vec::new();
        let mut style = Style::default();
        for attr in PRESENTATION {
            if let Some(v) = value(attr) {
                let parent = style.clone();
                if let Err(e) = style.apply(attr, &v, &parent) {
                    warnings.push(format!("{attr}: {e}"));
                }
            }
        }
        let num = |attr: &str, default: f32, warnings: &mut Vec<String>| match value(attr)
            .as_deref()
            .or_else(|| svg_prop(attr).and_then(|p| r.str(p)))
            .map(|v| v.trim().trim_end_matches("px").parse::<f32>())
        {
            Some(Ok(v)) => v,
            Some(Err(_)) => {
                warnings.push(format!("{attr} is a number of user units"));
                default
            }
            None => default,
        };
        let own = |attr: &str, warnings: &mut Vec<String>| {
            svg_prop(attr)
                .and_then(|p| r.str(p))
                .map_or(Ok(0.0), |v| v.trim().trim_end_matches("px").parse::<f32>())
                .unwrap_or_else(|_| {
                    warnings.push(format!("{attr} is a number of user units"));
                    0.0
                })
        };
        let t = computed
            .get(TRANSFORM)
            .cloned()
            .flatten()
            .and_then(|t| Affine::parse(&t).ok())
            .unwrap_or(Affine::IDENTITY);
        let font_size = num("font-size", 16.0, &mut warnings).max(0.0);
        // dx and dy: user units or em, a list summed (a line's dy is the
        // sum of the dys since its y).
        let shift = |attr: &str, warnings: &mut Vec<String>| -> f32 {
            let Some(v) = svg_prop(attr).and_then(|p| r.str(p)) else {
                return 0.0;
            };
            v.split([' ', ','])
                .filter(|t| !t.is_empty())
                .map(|t| match t.strip_suffix("em") {
                    Some(n) => n.parse::<f32>().map(|n| n * font_size),
                    None => t.trim_end_matches("px").parse::<f32>(),
                })
                .sum::<Result<f32, _>>()
                .unwrap_or_else(|_| {
                    warnings.push(format!("{attr} \"{v}\" is user units or em"));
                    0.0
                })
        };
        let x = own("x", &mut warnings) + shift("dx", &mut warnings);
        let y = own("y", &mut warnings) + shift("dy", &mut warnings);
        let at = t.apply(og_core::glam::Vec2::new(x, y));
        let size = font_size * t.scale();
        let weight = match value("font-weight").as_deref().map(str::trim) {
            None | Some("normal") => 0,
            Some("bold" | "bolder") => 700,
            Some(v) => v.parse::<u16>().unwrap_or_else(|_| {
                warnings.push(format!("font-weight \"{v}\" is normal, bold or 100–900"));
                0
            }),
        };
        let italic = matches!(
            value("font-style").as_deref().map(str::trim),
            Some("italic" | "oblique")
        );
        let text = r.str("text").unwrap_or_default().to_string();
        let font = r.label_desc(self.env.theme(), self.env.fonts(), false).font;
        let (ascender, descender) = {
            let m = &self
                .env
                .fonts()
                .for_styled(font, weight, italic, &text)
                .0
                .metrics;
            m.extents(&text)
        };
        let anchor_x = match value("text-anchor").as_deref().map(str::trim) {
            None | Some("start") => 0.0,
            Some("middle") => 0.5,
            Some("end") => 1.0,
            Some(v) => {
                warnings.push(format!("text-anchor \"{v}\" is start, middle or end"));
                0.0
            }
        };
        let base = value("dominant-baseline").unwrap_or_default();
        let anchor_y = baseline_shift(base.trim(), ascender, descender, size).unwrap_or_else(|e| {
            warnings.push(e);
            ascender * size
        });
        let mut step = self.env.theme().type_ramp.body;
        step.size = size;
        // One line exactly the font's extents tall: the baseline sits at
        // the ascender, so `dominant-baseline` is measured from its top.
        step.line_height = (ascender - descender) * size;
        step.cjk_line_height = step.line_height;
        if weight > 0 {
            step.weight = weight;
        }
        let color = solid(&style.fill, style.fill_opacity).and_then(|p| match p {
            Paint::Solid(c) => c.srgb(self.env.theme()),
            _ => None,
        });
        // `fill="none"`: transparent, still laid out and named.
        let color = color.or(Some(Srgba {
            rgb: [0; 3],
            alpha: 0.0,
        }));
        // Lines a kept line break starts align on the anchor too.
        let align = match anchor_x {
            0.5 => crate::layout::TextAlign::Center,
            1.0 => crate::layout::TextAlign::End,
            _ => crate::layout::TextAlign::Start,
        };
        let desc = crate::ui::LabelDesc {
            step: Some(step),
            color,
            nowrap: true,
            align,
            font,
            ..Default::default()
        };
        let text_path = match text_path(&b.doc, r, &t, vb) {
            Ok(tp) => tp,
            Err(e) => {
                warnings.push(e);
                None
            }
        };
        let width = desc.width;
        // A whole bold or italic text is one span over it (bold synthesised
        // where the role has no such face, as a `<tspan>`'s is).
        let bold = weight >= 600;
        let whole;
        let spans = match r.get("spans") {
            Some(V::Spans(Ok(spans))) => Some(spans.as_slice()),
            _ if italic || bold => {
                whole = [crate::TextSpan {
                    name: "svgWhole".into(),
                    position: 0,
                    length: text.chars().count() as u32,
                    properties: Default::default(),
                }];
                Some(&whole[..])
            }
            _ => None,
        };
        match spans {
            Some(spans) => {
                self.rich_text_spans(
                    id,
                    &text,
                    spans,
                    crate::RichTextDesc {
                        step: desc.step,
                        color: desc.color,
                        align: desc.align,
                        nowrap: true,
                        width: desc.width,
                        font,
                    },
                    |span| {
                        let mut s = r
                            .span_styles
                            .iter()
                            .find(|(attribute, _)| attribute == span)
                            .map(|(_, style)| style.clone())
                            .unwrap_or_default();
                        if italic {
                            s.italic.get_or_insert(true);
                        }
                        if bold {
                            s.bold.get_or_insert(true);
                        }
                        s
                    },
                );
            }
            None => self.label(id, &text, desc),
        }
        let i = self.tree.last_closed();
        // Its anchor places it, so it keeps its text's width (a centred
        // label in a column would take the column's).
        self.tree.el_mut(i).width = width;
        let mut p = Self::resolved_props(n, r);
        p.states = props.states;
        p.state = props.state;
        p.position = crate::element::Position::Absolute;
        p.view_place = Some(match &text_path {
            // On a path: its box from the view box's origin, the path in
            // it (scaled with the view box as the shapes are).
            Some(_) => crate::element::ViewPlace {
                view_box: vb,
                at: [vb[0], vb[1]],
                anchor_x: 0.0,
                anchor_y: 0.0,
            },
            None => crate::element::ViewPlace {
                view_box: vb,
                at: [at.x, at.y],
                anchor_x,
                anchor_y,
            },
        });
        p.text_path = text_path;
        *self.tree.props_mut(i) = p;
        for e in warnings {
            let tag = n.selector_tag.as_deref().unwrap_or("text");
            self.out.warnings_mut().push(format!(
                "{}:{} ({path}): <{tag}>: {e}",
                b.doc.source, n.line
            ));
        }
        self.fire(b, n, key, path, crate::ui::Response::default());
    }

    /// A `<foreignObject x y width height>`: a column box `width` by
    /// `height` user units laid out at that size, its corner on `(x, y)`
    /// fitted into the `<svg>`'s box and scaled with the view box, its
    /// document elements cut to it (SVG's overflow hidden).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build_svg_foreign(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &str,
        r: &Resolved,
        props: Props,
        path: String,
        key: u64,
        built: Built,
        vb: [f32; 4],
        build_children: impl FnOnce(&mut Ui, &mut Build<'_>, Option<Props>),
    ) -> Option<String> {
        let computed = r.cascade.computed.clone();
        let t = computed
            .get(TRANSFORM)
            .cloned()
            .flatten()
            .and_then(|t| Affine::parse(&t).ok())
            .unwrap_or(Affine::IDENTITY);
        let mut warnings = Vec::new();
        let len = |p: &str, whole: f32, warnings: &mut Vec<String>| {
            r.str(p)
                .map_or(Ok(0.0), |v| crate::svg::length(v, whole))
                .unwrap_or_else(|e| {
                    warnings.push(format!(
                        "{}: {e}",
                        crate::doc::schema::props::svg::svg_attr(p).unwrap_or(p)
                    ));
                    0.0
                })
        };
        let (x, y) = (
            len("svgX", vb[2], &mut warnings),
            len("svgY", vb[3], &mut warnings),
        );
        let (w, h) = (
            len("svgWidth", vb[2], &mut warnings).max(0.0),
            len("svgHeight", vb[3], &mut warnings).max(0.0),
        );
        let [a, bb, c, d, ..] = t.0;
        if bb != 0.0 || c != 0.0 || (a - 1.0).abs() > 1e-6 || (d - 1.0).abs() > 1e-6 {
            warnings.push(
                "a <foreignObject> takes only a translation from its transform (its content is laid out, not drawn as shapes)".into(),
            );
        }
        let at = Vec2::new(x + t.0[4], y + t.0[5]);
        let el = Element {
            direction: Direction::Column,
            ..Element::default()
        };
        let mut el = Self::box_of(r, el, self.style.em_of(r, &self.env));
        el.width = Size::px(w);
        el.height = Size::px(h);
        self.open(id, el);
        let i = self.tree.parent();
        let mut p = Self::resolved_props(n, r);
        p.states = props.states;
        p.state = props.state;
        p.position = crate::element::Position::Absolute;
        p.view_place = Some(crate::element::ViewPlace {
            view_box: vb,
            at: [at.x, at.y],
            anchor_x: 0.0,
            anchor_y: 0.0,
        });
        *self.tree.props_mut(i) = p;
        self.set_draw(ElementDraw {
            shape: ShapeKind::Rect {
                radii: Some([0.0; 4]),
                corner: crate::paint::Corner::Round,
            },
            clip: crate::paint::Clip::Shape,
            ..ElementDraw::default()
        });
        for e in warnings {
            self.out.warnings_mut().push(format!(
                "{}:{} ({path}): <foreignObject>: {e}",
                b.doc.source, n.line
            ));
        }
        self.docs.note_built(built);
        let inherited = std::mem::replace(&mut b.inherited, computed);
        build_children(self, b, None);
        b.inherited = inherited;
        let resp = self.close();
        self.fire(b, n, key, &path, resp);
        Some(path)
    }

    /// Builds an SVG element: `<svg>` as a box of its size, a group or a
    /// shape as a box over the whole `<svg>`, a definition as nothing.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build_svg(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        kind: Kind,
        id: &str,
        r: &Resolved,
        props: Props,
        path: String,
        key: u64,
        built: Built,
        build_children: impl FnOnce(&mut Ui, &mut Build<'_>, Option<Props>),
    ) -> Option<String> {
        if kind == Kind::SvgDef {
            return None;
        }
        if animated(&b.doc, n) {
            // Its animations' clock starts as it first shows.
            let clock = smil_clock(&path);
            if self.has_anim(clock) {
                self.anim(clock);
            } else {
                let t = self.interaction.state().tick;
                self.set_anim(
                    clock,
                    crate::ui::Anim {
                        t,
                        ..Default::default()
                    },
                );
            }
            if r.smil
                && let Some(owned) = self.tree.owning_mut().last_mut()
            {
                owned.restyle = true;
            }
        }
        let computed = r.cascade.computed.clone();
        let vb = computed
            .get(VIEW_BOX)
            .cloned()
            .flatten()
            .and_then(|v| view_box(&v))
            .unwrap_or([0.0, 0.0, DEFAULT_SIZE[0], DEFAULT_SIZE[1]]);
        if kind == Kind::SvgText {
            self.build_svg_text(b, n, id, r, props, &path, key, vb);
            self.docs.note_built(built);
            return Some(path);
        }
        if kind == Kind::SvgForeign {
            return self.build_svg_foreign(
                b,
                n,
                id,
                r,
                props,
                path,
                key,
                built,
                vb,
                build_children,
            );
        }
        let mut el = Element {
            direction: Direction::Column,
            ..Element::default()
        };
        if kind == Kind::Svg {
            el.width = Size::px(vb[2]);
            el.height = Size::px(vb[3]);
        }
        let mut el = Self::box_of(r, el, self.style.em_of(r, &self.env));
        if kind != Kind::Svg {
            el.width = Size::grow();
            el.height = Size::grow();
        }
        self.open(id, el);
        let i = self.tree.parent();
        let mut p = Self::resolved_props(n, r);
        p.states = props.states;
        p.state = props.state;
        if kind != Kind::Svg {
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
        *self.tree.props_mut(i) = p;
        if let Some(role) = n.role {
            self.tree.semantics_mut(i).role = Some(role);
        }
        if let Some(name) = r.str("name").filter(|n| !n.is_empty()) {
            self.tree.semantics_mut(i).name = Some(name.to_string());
            // A named `<svg>` is an image, as browsers expose one.
            if kind == Kind::Svg && n.role.is_none() {
                self.tree.semantics_mut(i).role = Some("image");
            }
        }
        let t = computed
            .get(TRANSFORM)
            .cloned()
            .flatten()
            .and_then(|t| Affine::parse(&t).ok())
            .unwrap_or(Affine::IDENTITY);
        let warn = |ui: &mut Ui, e: String| {
            let tag = n.selector_tag.as_deref().unwrap_or("svg");
            ui.out.warnings_mut().push(format!(
                "{}:{} ({path}): <{tag}>: {e}",
                b.doc.source, n.line
            ));
        };
        // A clip or a mask cuts the element's children; a cut shape
        // draws as the cut element's child, an anonymous box over it.
        let cut = match cut(&b.doc, r, &t, vb) {
            Ok(c) => c,
            Err(e) => {
                warn(self, e);
                None
            }
        };
        // `filter="url(#id)"`: the element and its subtree drawn through
        // an SVG `<filter>`, the innermost of its layers.
        let filter = match r.str("svgFilter").map(str::trim) {
            Some(v) if kind != Kind::Svg && v != "none" && !v.is_empty() => {
                match self.docs.svg_filter(b, n, kind, r, v, &t, vb, &self.env) {
                    Ok(f) => Some(f),
                    Err(e) => {
                        warn(self, format!("filter: {e}"));
                        None
                    }
                }
            }
            _ => None,
        };
        let filtered = |mut d: ElementDraw| {
            if let Some(f) = &filter {
                d.filters.insert(
                    0,
                    crate::paint::Layer {
                        filter: Some(f.clone()),
                        ..Default::default()
                    },
                );
            }
            d
        };
        if cut.is_none() && filter.is_some() && kind != Kind::SvgShape {
            self.set_draw(filtered(ElementDraw {
                view_box: Some(vb),
                ..ElementDraw::default()
            }));
        }
        if let Some(c) = &cut {
            self.set_draw(filtered(ElementDraw {
                shape: match c {
                    Cut::Shape(s) | Cut::Box(s, _) => s.clone(),
                    Cut::Gradient(_) => ShapeKind::default(),
                },
                clip: match c {
                    Cut::Shape(_) | Cut::Box(..) => crate::paint::Clip::Shape,
                    Cut::Gradient(g) => {
                        crate::paint::Clip::Mask(crate::paint::Mask::Gradient(g.clone()))
                    }
                },
                view_box: Some(vb),
                view_rect: match c {
                    Cut::Box(_, r) => Some(*r),
                    _ => None,
                },
                ..ElementDraw::default()
            }));
        }
        let over = |ui: &mut Ui, id: &str| {
            ui.open(
                id,
                Element {
                    width: Size::grow(),
                    height: Size::grow(),
                    ..Element::default()
                },
            );
            let i = ui.tree.parent();
            ui.tree.props_mut(i).position = crate::element::Position::Absolute;
            ui.tree.props_mut(i).insets = crate::element::Edges {
                start: Some(0.0),
                top: Some(0.0),
                end: Some(0.0),
                bottom: Some(0.0),
                ..Default::default()
            };
        };
        if kind == Kind::SvgShape && n.selector_tag.as_deref() == Some("image") {
            match self.docs.image_draw(b, r, &t, vb, &self.env) {
                Ok(Some(draw)) if cut.is_none() => self.set_draw(filtered(draw)),
                Ok(Some(draw)) => self.set_draw(draw),
                Ok(None) => {}
                Err(e) => warn(self, e),
            }
        } else if kind == Kind::SvgShape {
            let tag = n.selector_tag.as_deref().unwrap_or("path");
            match shape_draw(&b.doc, tag, r, vb) {
                Ok(ShapeOut {
                    draw,
                    tiles,
                    tile_box,
                    markers,
                }) => {
                    if cut.is_some() {
                        over(self, "~shape");
                        self.set_draw(draw);
                    } else {
                        self.set_draw(filtered(draw));
                    }
                    let boxed = !tiles.is_empty();
                    if boxed {
                        over(self, "~pattern");
                        if let Some((shape, view_rect)) = tile_box {
                            self.set_draw(ElementDraw {
                                shape,
                                clip: crate::paint::Clip::Shape,
                                view_box: Some(vb),
                                view_rect,
                                ..ElementDraw::default()
                            });
                        }
                    }
                    for (k, (shape, paint)) in tiles.into_iter().enumerate() {
                        over(self, &format!("~tile{k}"));
                        self.set_draw(ElementDraw {
                            shape,
                            fills: vec![paint],
                            view_box: Some(vb),
                            ..ElementDraw::default()
                        });
                        self.close();
                    }
                    if boxed {
                        self.close();
                    }
                    for (k, draw) in markers.into_iter().enumerate() {
                        over(self, &format!("~marker{k}"));
                        self.set_draw(draw);
                        self.close();
                    }
                    if cut.is_some() {
                        self.close();
                    }
                }
                Err(e) => warn(self, e),
            }
        }
        if let Some(f) = &filter {
            let bbox = f.svg.map_or([0.0; 4], |(_, bbox)| bbox);
            self.build_element_images(b, n, &computed, &[&**f], ImageSpace::Svg(t, bbox));
        }
        self.docs.note_built(built);
        if kind != Kind::SvgShape {
            let inherited = std::mem::replace(&mut b.inherited, computed);
            build_children(self, b, None);
            b.inherited = inherited;
        }
        let resp = self.close();
        self.fire(b, n, key, &path, resp);
        Some(path)
    }
}

impl crate::ui::Docs {
    /// An `<image href x y width height preserveAspectRatio>`: an atlas
    /// icon by name, or a render texture by handle, filling its rect.
    /// An SVG element's `filter` reference: the `<filter>` in its user
    /// space (`t` takes it into the view box) and its bounding box there:
    /// a shape's geometry, a group's shapes' (literal ones).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn svg_filter(
        &self,
        b: &Build<'_>,
        n: &DocNode,
        kind: Kind,
        r: &Resolved,
        reference: &str,
        t: &Affine,
        vb: [f32; 4],
        env: &crate::ui::Env,
    ) -> Result<std::sync::Arc<crate::paint::FilterUse>, String> {
        let image = |href: &str| -> Result<(og_draw::TextureHandle, [f32; 4]), String> {
            match href.parse::<f64>() {
                Ok(h) => b
                    .env
                    .texture(h)
                    .map(|t| (t, [0.0, 0.0, 1.0, 1.0]))
                    .ok_or_else(|| format!("href {h} names no texture")),
                Err(_) => match env.icons().find(href) {
                    Some(crate::ui::IconCell::Image(texture, uv)) => Ok((texture, uv)),
                    _ => Err(format!("href \"{href}\" names no atlas image")),
                },
            }
        };
        let filter = filter_def(&b.doc, reference, [vb[2], vb[3]], &image)?;
        let union = |a: [f32; 4], b: [f32; 4]| {
            if a[2] <= 0.0 && a[3] <= 0.0 {
                return b;
            }
            let x = a[0].min(b[0]);
            let y = a[1].min(b[1]);
            [
                x,
                y,
                (a[0] + a[2]).max(b[0] + b[2]) - x,
                (a[1] + a[3]).max(b[1] + b[3]) - y,
            ]
        };
        let mut bbox = [0.0; 4];
        let tag = n.selector_tag.as_deref().unwrap_or("path");
        if kind == Kind::SvgShape {
            let get = |attr: &str| svg_prop(attr).and_then(|p| r.str(p));
            if tag == "image" {
                let len =
                    |a: &str, whole: f32| get(a).map_or(Ok(0.0), |v| crate::svg::length(v, whole));
                bbox = [
                    len("x", vb[2])?,
                    len("y", vb[3])?,
                    len("width", vb[2])?,
                    len("height", vb[3])?,
                ];
            } else if let Some(local) = shape_path(tag, &get, vb)? {
                bbox = bounds(&local);
            }
        } else {
            let mut shapes = Vec::new();
            def_shapes(&b.doc, n, Affine::IDENTITY, &mut shapes)?;
            for (tag, attrs, own) in &shapes {
                let get = |a: &str| attrs.iter().find(|(k, _)| k == a).map(|(_, v)| v.as_str());
                if let Some(local) = shape_path(tag, &get, vb)? {
                    bbox = union(bbox, bounds(&local.transformed(own)));
                }
            }
        }
        Ok(std::sync::Arc::new(crate::paint::FilterUse {
            filter,
            svg: Some((*t, bbox)),
        }))
    }

    pub(crate) fn image_draw(
        &self,
        b: &Build<'_>,
        r: &Resolved,
        t: &Affine,
        vb: [f32; 4],
        env: &crate::ui::Env,
    ) -> Result<Option<ElementDraw>, String> {
        let get = |p: &str| r.str(p);
        let len = |p: &str, whole: f32| -> Result<f32, String> {
            get(p)
                .map_or(Ok(0.0), |v| crate::svg::length(v, whole))
                .map_err(|e| format!("{p}: {e}"))
        };
        let (w, h) = (len("svgWidth", vb[2])?, len("svgHeight", vb[3])?);
        if w <= 0.0 || h <= 0.0 {
            return Ok(None);
        }
        let [_, bb, c, _, ..] = t.0;
        if bb != 0.0 || c != 0.0 {
            return Err("an <image> takes no turn or skew".into());
        }
        let (x, y) = (len("svgX", vb[2])?, len("svgY", vb[3])?);
        let p = t.apply(Vec2::new(x, y));
        let q = t.apply(Vec2::new(x + w, y + h));
        let rect = [
            p.x.min(q.x),
            p.y.min(q.y),
            (q.x - p.x).abs(),
            (q.y - p.y).abs(),
        ];
        let href = get("svgHref").unwrap_or_default().trim();
        let (texture, uv) = match href.parse::<f64>() {
            Ok(h) => (
                b.env
                    .texture(h)
                    .ok_or_else(|| format!("href {h} names no texture"))?,
                [0.0, 0.0, 1.0, 1.0],
            ),
            Err(_) => match env.icons().find(href) {
                Some(crate::ui::IconCell::Image(texture, uv)) => (texture, uv),
                Some(_) => {
                    return Err(format!(
                        "\"{href}\" is a glyph icon, not an image: write <og-icon icon=\"{href}\"> beside the svg"
                    ));
                }
                None => return Err(format!("href \"{href}\" names no atlas image")),
            },
        };
        let fit = match get("svgPreserveAspectRatio").map(str::trim) {
            Some("none") => og_draw::UiFit::Fill,
            Some(v) if v.contains("slice") => og_draw::UiFit::Cover,
            _ => og_draw::UiFit::Contain,
        };
        Ok(Some(ElementDraw {
            shape: ShapeKind::Rect {
                radii: Some([0.0; 4]),
                corner: crate::paint::Corner::Round,
            },
            fills: vec![Paint::Image(crate::paint::ImagePaint {
                texture,
                uv,
                size: [
                    (uv[2] - uv[0]).abs() * 1024.0,
                    (uv[3] - uv[1]).abs() * 1024.0,
                ],
                density: 1.0,
                tint: ColorRef::Literal(Srgba {
                    rgb: [255; 3],
                    alpha: 1.0,
                }),
                fit,
                focal: [0.5, 0.5],
                offset: [0.0; 2],
                nearest: false,
            })],
            view_box: Some(vb),
            view_rect: Some(rect),
            ..ElementDraw::default()
        }))
    }
}
