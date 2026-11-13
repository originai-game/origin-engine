use super::effect::backdrop_graph;
use super::*;

impl ElementDraw {
    /// Whether its subtree draws offscreen.
    pub(crate) fn layered(&self) -> bool {
        self.layer.is_some()
            || !self.filters.is_empty()
            || matches!(self.clip, Clip::Mask(_))
            || self
                .effects
                .iter()
                .any(|e| matches!(e, Effect::LayerBlur { .. }))
    }

    /// Whether CSS mask layers cut its subtree: it draws in a plain layer
    /// of its own, outside its other layers, which the masks are applied
    /// to and which carries its opacity and blend.
    fn masked(&self) -> bool {
        self.boxes.as_ref().is_some_and(|b| !b.masks.is_empty())
    }

    /// Whether a layer's composite carries its opacity and blend.
    pub(crate) fn composited(&self) -> bool {
        self.layered() || self.masked()
    }

    /// Its background or mask layers drawn in `at` with `blend`, each cut
    /// to its painting area; `text`'s clip is drawn as the box's (the
    /// caller cuts those layers to the text).
    fn box_layers(
        &self,
        layers: &[BoxLayer],
        theme: &Theme,
        at: &Placed,
        opacity: f32,
        blend: impl Fn(&BoxLayer) -> Blend,
    ) -> Vec<DrawCmd> {
        let mut out = Vec::new();
        for l in layers {
            let (inset, mask_rect) = match l.clip {
                LayerClip::Box(i) => (i, false),
                LayerClip::Text => ([0.0; 4], false),
                LayerClip::Rect(i) => (i, true),
            };
            let r = inset_rect(at.rect, inset);
            if r.is_empty() {
                continue;
            }
            let geometry = if mask_rect {
                og_draw::UiGeometry::Rect {
                    radii: [0.0; 4],
                    corner: Corner::Round,
                }
            } else {
                inset_geometry(self.geometry(&self.shape, at), inset)
            };
            out.push(DrawCmd::Shape {
                shape: Box::new(og_draw::UiShape {
                    geometry,
                    center: Vec2::new(r.x + r.width / 2.0, r.y + r.height / 2.0),
                    half: Vec2::new(r.width / 2.0, r.height / 2.0),
                    transform: [1.0, 0.0, 0.0, 1.0],
                    paint: l.resolve(theme, at.rect, r),
                    mode: og_draw::UiMode::Fill,
                    opacity,
                    blend: blend(l),
                    layer: 0,
                    clip: None,
                }),
                clip: at.clip,
            });
        }
        out
    }

    /// Its backgrounds cut to its text (`background-clip: text`) around
    /// `text`, the commands that draw it: the glyphs' coverage drawn
    /// opaque in a layer of their own, the layers kept only there, then
    /// the text as it is drawn (CSS Backgrounds 4 §2.1). `text` unchanged
    /// when it has none.
    pub(crate) fn clip_to_text(
        &self,
        theme: &Theme,
        at: &Placed,
        text: Vec<DrawCmd>,
    ) -> Vec<DrawCmd> {
        let Some(b) = self
            .boxes
            .as_ref()
            .filter(|b| b.backgrounds.iter().any(|l| l.clip == LayerClip::Text))
        else {
            return text;
        };
        let mut out = vec![plain_layer(None)];
        let opaque = |c: &mut DrawCmd| {
            if let DrawCmd::StyledText { text, .. } = c {
                *c = (**text).clone();
            }
            if let DrawCmd::Text { color, glow, .. } = c {
                *color = [1.0; 4];
                *glow = None;
            }
        };
        let mut glyphs = 0;
        for c in &text {
            if matches!(c, DrawCmd::Text { .. } | DrawCmd::StyledText { .. }) {
                let mut c = c.clone();
                opaque(&mut c);
                out.push(c);
                glyphs += 1;
            }
        }
        if glyphs == 0 {
            return text;
        }
        let on_text: Vec<BoxLayer> = b
            .backgrounds
            .iter()
            .filter(|l| l.clip == LayerClip::Text)
            .cloned()
            .collect();
        // The bottom layer keeps itself on the glyphs; the rest blend over
        // it there as they do in the box.
        for (k, cmd) in self
            .box_layers(&on_text, theme, at, 1.0, |l| l.blend)
            .into_iter()
            .enumerate()
        {
            let mut cmd = cmd;
            if k == 0
                && let DrawCmd::Shape { shape, .. } = &mut cmd
            {
                shape.blend = Blend::SourceIn;
            }
            out.push(cmd);
        }
        if on_text.len() > 1 {
            // Later layers drew over the glyphs' edges too: the glyphs cut
            // them again.
            out.push(plain_layer(None));
            for c in &text {
                if matches!(c, DrawCmd::Text { .. } | DrawCmd::StyledText { .. }) {
                    let mut c = c.clone();
                    opaque(&mut c);
                    out.push(c);
                }
            }
            out.push(end_layer(None, at, 1.0, Blend::DestinationIn));
        }
        let o = if self.composited() { 1.0 } else { self.opacity };
        out.push(end_layer(None, at, o, Blend::Normal));
        out.extend(text);
        out
    }

    /// Its layers: the outer one (its own, else its last filter's), which
    /// carries the composite, and the filters inside it, the first
    /// innermost.
    /// The number of its first element image in layer `l`'s filter
    /// (one of its layers; [`filter_uses`]).
    fn element_image_base(&self, l: &Layer) -> u32 {
        let own = l.filter.as_deref();
        filter_uses(&self.effects, self.layer.as_ref(), &self.filters)
            .into_iter()
            .take_while(|u| !own.is_some_and(|o| std::ptr::eq(*u, o)))
            .map(|u| u.filter.elements.len() as u32)
            .sum()
    }

    fn stages(&self) -> (Option<&Layer>, &[Layer]) {
        match (&self.layer, self.filters.split_last()) {
            (Some(l), _) => (Some(l), &self.filters),
            (None, Some((last, rest))) => (Some(last), rest),
            (None, None) => (None, &[]),
        }
    }

    /// Whether a point `p` (CSS px from its box's top left, the box `size`)
    /// is on the geometry SVG `pointer-events` names (`bits` of
    /// [`crate::element::SvgPointer`]): inside its fill by its fill rule,
    /// within half a stroke's width of its stroke or outlines, or in its
    /// bounding box; `painted` takes only what is painted (SVG 2 §16.6).
    pub fn covers(&self, size: [f32; 2], p: Vec2, bits: u8) -> bool {
        use crate::element::SvgPointer as S;
        let (k, q) = match self.view_box {
            Some([x, y, w, h]) => {
                let k = (size[0] / w.max(1e-6)).min(size[1] / h.max(1e-6)).max(1e-6);
                let mid = Vec2::new(x + w * 0.5, y + h * 0.5);
                (k, (p - Vec2::from(size) * 0.5) / k + mid)
            }
            None => (1.0, p),
        };
        let frame = self.view_rect.unwrap_or(match self.view_box {
            Some(vb) => vb,
            None => [0.0, 0.0, size[0], size[1]],
        });
        let tol = 0.25 / k;
        let lines = |kind: &ShapeKind| outline(kind, frame, tol);
        if bits & S::BOX != 0 {
            let all: Vec<Vec2> = std::iter::once(&self.shape)
                .chain(self.outlines.iter().map(|(s, _)| s))
                .flat_map(|s| lines(s).0.into_iter().flatten())
                .collect();
            let lo = all.iter().fold(Vec2::splat(f32::MAX), |a, b| a.min(*b));
            let hi = all.iter().fold(Vec2::splat(f32::MIN), |a, b| a.max(*b));
            return q.cmpge(lo).all() && q.cmple(hi).all();
        }
        let painted = bits & S::PAINTED != 0;
        let (shape, closed, rule) = lines(&self.shape);
        if bits & S::FILL != 0 && (!painted || !self.fills.is_empty()) && closed {
            let winding: i32 = shape.iter().map(|c| winding(c, q)).sum();
            if match rule {
                FillRule::EvenOdd => winding % 2 != 0,
                _ => winding != 0,
            } {
                return true;
            }
        }
        if bits & S::STROKE != 0 && (!painted || !self.strokes.is_empty()) {
            let width = self.strokes.iter().map(|s| s.width).fold(0.0, f32::max);
            let own = match &self.shape {
                ShapeKind::Path {
                    filled: false,
                    width: w,
                    ..
                } => *w,
                _ => width,
            };
            let near = |cs: &[Vec<Vec2>], closed: bool, half: f32| {
                cs.iter().any(|c| distance(c, closed, q) <= half)
            };
            if own > 0.0 && near(&shape, closed, own * 0.5) {
                return true;
            }
            if self.outlines.iter().any(|(s, st)| {
                let (c, closed, _) = lines(s);
                near(&c, closed, st.width * 0.5)
            }) {
                return true;
            }
        }
        false
    }

    /// The view box's fit in `at`'s box: its scale and the box point
    /// (CSS px from the box's top left) its centre lands on.
    fn fit(&self, at: &Placed) -> Option<(f32, Vec2, Vec2)> {
        let [_, _, w, h] = self.view_box?;
        let k = (at.rect.width / w.max(1e-6)).min(at.rect.height / h.max(1e-6));
        let vb = self.view_box?;
        Some((
            k,
            Vec2::new(vb[0] + vb[2] * 0.5, vb[1] + vb[3] * 0.5),
            Vec2::new(at.rect.width, at.rect.height) * 0.5,
        ))
    }

    /// The rect its shape fills in `at`: its view rect through the view
    /// box's fit, else the box.
    fn frame(&self, at: &Placed) -> Rect {
        match (self.fit(at), self.view_rect) {
            (Some((k, mid, half)), Some([x, y, w, h])) => {
                let o = (Vec2::new(x, y) - mid) * k + half;
                Rect {
                    x: at.rect.x + o.x,
                    y: at.rect.y + o.y,
                    width: w * k,
                    height: h * k,
                }
            }
            _ => at.rect,
        }
    }

    /// `kind`'s geometry placed in `at`, through the view box when it has one.
    fn geometry(&self, kind: &ShapeKind, at: &Placed) -> og_draw::UiGeometry {
        let size = [at.rect.width, at.rect.height];
        let Some((k, mid, half)) = self.fit(at) else {
            return kind.geometry(at.radius, size);
        };
        match kind {
            ShapeKind::Path {
                points,
                closed,
                filled,
                curves,
                fill_rule,
                contours,
                width,
                cap,
                join,
            } => ShapeKind::Path {
                points: points
                    .iter()
                    .map(|p| ((Vec2::from(*p) - mid) * k + half).into())
                    .collect(),
                closed: *closed,
                filled: *filled,
                curves: *curves,
                fill_rule: *fill_rule,
                contours: contours.clone(),
                width: width * k,
                cap: *cap,
                join: *join,
            }
            .geometry(at.radius, size),
            ShapeKind::Rect {
                radii: Some(r),
                corner,
            } if self.view_rect.is_some() => ShapeKind::Rect {
                radii: Some(r.map(|v| v * k)),
                corner: *corner,
            }
            .geometry(at.radius, size),
            other => {
                let f = self.frame(at);
                other.geometry(at.radius, [f.width, f.height])
            }
        }
    }

    /// A paint resolved for `at`: a gradient written in view-box units
    /// becomes fractions of the box.
    fn paint(&self, paint: &Paint, theme: &Theme, at: &Placed) -> og_draw::UiPaint {
        let mut out = paint.resolve(theme);
        if let (Some((k, mid, half)), og_draw::UiPaint::Gradient(g)) = (self.fit(at), &mut out) {
            let size = Vec2::new(at.rect.width, at.rect.height).max(Vec2::splat(1e-6));
            g.from = ((g.from - mid) * k + half) / size;
            g.to = match g.kind {
                GradientKind::Linear => ((g.to - mid) * k + half) / size,
                _ => g.to * k / size,
            };
        }
        out
    }

    fn shape(
        &self,
        at: &Placed,
        paint: og_draw::UiPaint,
        mode: og_draw::UiMode,
        opacity: f32,
        blend: Blend,
    ) -> DrawCmd {
        self.shape_of(&self.shape, at, paint, mode, opacity, blend)
    }

    fn shape_of(
        &self,
        kind: &ShapeKind,
        at: &Placed,
        paint: og_draw::UiPaint,
        mode: og_draw::UiMode,
        opacity: f32,
        blend: Blend,
    ) -> DrawCmd {
        let r = if matches!(kind, ShapeKind::Path { .. }) {
            at.rect
        } else {
            self.frame(at)
        };
        DrawCmd::Shape {
            shape: Box::new(og_draw::UiShape {
                geometry: self.geometry(kind, at),
                center: Vec2::new(r.x + r.width / 2.0, r.y + r.height / 2.0),
                half: Vec2::new(r.width / 2.0, r.height / 2.0),
                transform: [1.0, 0.0, 0.0, 1.0],
                paint,
                mode,
                opacity,
                blend,
                layer: 0,
                clip: None,
            }),
            clip: at.clip,
        }
    }

    /// A `backdrop-filter`: a backdrop layer over its region, its url
    /// filters' element images inside it, composited in its shape.
    fn backdrop_filter(
        &self,
        stages: &[BackdropStage],
        theme: &Theme,
        at: &Placed,
        opacity: f32,
    ) -> Vec<DrawCmd> {
        let (region, primitives, _) =
            backdrop_graph(stages, [at.rect.width, at.rect.height], theme);
        if primitives.is_empty() {
            return Vec::new();
        }
        let map = crate::element::Affine::translate(at.rect.x, at.rect.y);
        let mut graph = og_draw::UiFilter {
            primitives,
            ..Default::default()
        };
        crate::svg::filter::map(&mut graph, &map);
        let mut out = vec![DrawCmd::BeginLayer {
            material: og_draw::UiMaterial::Plain,
            params: [[0.0; 4]; og_draw::UI_MATERIAL_PARAMS],
            mask: None,
            expand: 0.0,
            rect: Rect::default(),
            filter: Some(Box::new((map.map_rect(&region), graph))),
            role: crate::draw::LayerRole::Backdrop,
        }];
        let rect = if matches!(self.shape, ShapeKind::Path { .. }) {
            at.rect
        } else {
            self.frame(at)
        };
        out.push(DrawCmd::EndLayer {
            geometry: Some(self.geometry(&self.shape, at)),
            rect,
            opacity,
            blend: Blend::Normal,
            clip: at.clip,
            ground: None,
        });
        out
    }

    /// Opens the subtree's masks, clips and filter layers, even when its
    /// own paint is hidden but a descendant restores visibility.
    pub(crate) fn begin(&self, theme: &Theme, at: &Placed) -> Vec<DrawCmd> {
        if let Some(n) = self.element_image {
            return vec![DrawCmd::BeginLayer {
                material: og_draw::UiMaterial::Plain,
                params: [[0.0; 4]; og_draw::UI_MATERIAL_PARAMS],
                mask: None,
                expand: 0.0,
                rect: Rect::default(),
                filter: None,
                role: crate::draw::LayerRole::Image(n),
            }];
        }
        let mut out = Vec::new();
        if self.masked() {
            out.push(plain_layer(None));
        }
        // A cut (CSS `clip-path`) takes all it draws, to `leave`; a layered
        // one is cut by its composite.
        if let Some(cut) = self.cut(at) {
            out.push(cut);
        }
        if self.layered() {
            let (outer, inner) = self.stages();
            let (material, params) = match outer {
                Some(l) => (l.material.into(), l.params),
                None => match self.effects.iter().find_map(|e| match e {
                    Effect::LayerBlur { radius } => Some(*radius),
                    _ => None,
                }) {
                    Some(radius) => {
                        let mut p = [[0.0; 4]; og_draw::UI_MATERIAL_PARAMS];
                        p[0][0] = radius;
                        (og_draw::UiMaterial::Builtin(Builtin::Blur), p)
                    }
                    None => (
                        og_draw::UiMaterial::Plain,
                        [[0.0; 4]; og_draw::UI_MATERIAL_PARAMS],
                    ),
                },
            };
            let mask = match &self.clip {
                // Written in view-box units, it is fitted as a fill is.
                Clip::Mask(Mask::Gradient(g)) => {
                    match self.paint(&Paint::Gradient(g.clone()), theme, at) {
                        og_draw::UiPaint::Gradient(g) => Some(og_draw::UiMask::Gradient(g)),
                        _ => None,
                    }
                }
                Clip::Mask(Mask::Image { texture, uv }) => Some(og_draw::UiMask::Image {
                    texture: *texture,
                    uv: *uv,
                }),
                Clip::Mask(Mask::Atlas { uv }) => theme
                    .art
                    .atlas
                    .map(|texture| og_draw::UiMask::Image { texture, uv: *uv }),
                _ => None,
            };
            // Its element images are numbered across its filters
            // ([`filter_uses`]); its children draw them.
            let filter = |l: Option<&Layer>| {
                let l = l?;
                let f = l.filter.as_ref()?;
                let (region, mut graph) = f.resolve(at, self.fit(at));
                let base = self.element_image_base(l);
                for p in &mut graph.primitives {
                    if let og_draw::UiFilterOp::Layer { layer } = &mut p.op
                        && let Some(n) = crate::svg::filter::element_of(*layer)
                    {
                        *layer = crate::svg::filter::element_layer(n + base);
                    }
                }
                Some(Box::new((region, graph)))
            };
            // A mask is fitted to the element's box, so that box is the
            // layer's rectangle (draw.rs), not the union of what it draws.
            let rect = if mask.is_some() {
                at.rect
            } else {
                Rect::default()
            };
            out.push(DrawCmd::BeginLayer {
                material,
                params,
                mask,
                expand: expand_of(material, &params),
                rect,
                filter: filter(outer),
                role: Default::default(),
            });
            // The inner stages open inside it, the first innermost.
            for l in inner.iter().rev() {
                let material = l.material.into();
                out.push(DrawCmd::BeginLayer {
                    material,
                    params: l.params,
                    mask: None,
                    expand: expand_of(material, &l.params),
                    rect: Rect::default(),
                    filter: filter(Some(l)),
                    role: Default::default(),
                });
            }
        }
        out
    }

    /// What draws before the element's own commands: a layer's start, then
    /// its shadows, outer glows and backdrops, then its fills, inner
    /// shadows and inner glows.
    pub(crate) fn enter(&self, theme: &Theme, at: &Placed) -> Vec<DrawCmd> {
        let mut out = self.begin(theme, at);
        if self.element_image.is_some() {
            return out;
        }
        // Inside a layer the composite carries the opacity and blend.
        let (opacity, blend) = if self.composited() {
            (at.alpha, Blend::Normal)
        } else {
            (self.opacity * at.alpha, self.blend)
        };
        let o = og_draw::UiMode::Fill;
        for e in &self.effects {
            let (mode, color) = match e {
                Effect::DropShadow {
                    offset,
                    blur,
                    spread,
                    color,
                } => (
                    og_draw::UiMode::Shadow {
                        offset: Vec2::from(*offset),
                        blur: *blur,
                        spread: *spread,
                    },
                    color,
                ),
                Effect::OuterGlow {
                    blur,
                    spread,
                    color,
                } => (
                    og_draw::UiMode::Shadow {
                        offset: Vec2::ZERO,
                        blur: *blur,
                        spread: *spread,
                    },
                    color,
                ),
                _ => continue,
            };
            out.push(self.shape(
                at,
                og_draw::UiPaint::Solid(color.resolve(theme)),
                mode,
                opacity,
                Blend::Normal,
            ));
        }
        for e in &self.effects {
            if let Effect::BackdropFilter(stages) = e {
                out.extend(self.backdrop_filter(stages, theme, at, opacity));
            }
            if let Effect::Backdrop {
                blur,
                saturation,
                brightness,
                tint,
                glass,
            } = e
            {
                let (mode, tint) = glass.resolve(
                    *blur,
                    *saturation,
                    *brightness,
                    tint.resolve(theme),
                    glass.own_tint,
                    theme,
                );
                out.push(self.shape(
                    at,
                    og_draw::UiPaint::Solid(tint),
                    mode,
                    opacity,
                    Blend::Normal,
                ));
            }
        }
        for f in &self.fills {
            if let Paint::Vector { vector, tint } = f {
                let mut tint = tint.resolve(theme);
                tint[3] *= opacity;
                out.push(DrawCmd::Vector {
                    vector: *vector,
                    rect: at.rect,
                    transform: [1.0, 0.0, 0.0, 1.0],
                    tint,
                    clip: at.clip,
                });
                continue;
            }
            out.push(self.shape(at, self.paint(f, theme, at), o, opacity, blend));
        }
        if let Some(b) = &self.boxes {
            // The backgrounds, cut to the text drawn with it
            // (`clip_to_text`) left out here; blending among themselves in a
            // group of their own when a layer blends.
            let boxed: Vec<BoxLayer> = b
                .backgrounds
                .iter()
                .filter(|l| l.clip != LayerClip::Text)
                .cloned()
                .collect();
            if b.isolate {
                out.push(plain_layer(None));
                out.extend(self.box_layers(&boxed, theme, at, 1.0, |l| l.blend));
                out.push(end_layer(None, at, opacity, blend));
            } else {
                out.extend(self.box_layers(&boxed, theme, at, opacity, |_| blend));
            }
        }
        for e in &self.effects {
            let (mode, color) = match e {
                Effect::InnerShadow {
                    offset,
                    blur,
                    spread,
                    color,
                } => (
                    og_draw::UiMode::InnerShadow {
                        offset: Vec2::from(*offset),
                        blur: *blur,
                        spread: *spread,
                    },
                    color,
                ),
                Effect::InnerGlow {
                    blur,
                    spread,
                    color,
                } => (
                    og_draw::UiMode::InnerShadow {
                        offset: Vec2::ZERO,
                        blur: *blur,
                        spread: *spread,
                    },
                    color,
                ),
                _ => continue,
            };
            out.push(self.shape(
                at,
                og_draw::UiPaint::Solid(color.resolve(theme)),
                mode,
                opacity,
                Blend::Normal,
            ));
        }
        if let Some(b) = self.boxes.as_ref().and_then(|b| b.border.as_ref()) {
            out.extend(self.box_layers(std::slice::from_ref(b), theme, at, opacity, |_| blend));
        }
        if let Some(RiveDraw(player)) = &self.rive {
            let r = at.rect;
            // An artboard that cuts its content to its frame cuts it to the
            // box, in the command's own space, so the item's transform turns
            // the cut with the element (a screen-space clip would cut a turned
            // element by its unturned box).
            let cut = player.artboard.clip;
            if cut {
                out.push(DrawCmd::PushClip {
                    geometry: og_draw::UiGeometry::Rect {
                        radii: [0.0; 4],
                        corner: og_draw::UiCorner::Round,
                    },
                    center: (r.x + r.width / 2.0, r.y + r.height / 2.0),
                    half: (r.width / 2.0, r.height / 2.0),
                });
            }
            for d in player.draw([r.x, r.y, r.width, r.height], opacity) {
                out.push(rive_cmd(d, at.clip));
            }
            if cut {
                out.push(DrawCmd::PopClip);
            }
        }
        out
    }

    /// What starts the children: the cut to the shape, when it clips.
    pub(crate) fn children(&self, at: &Placed) -> Vec<DrawCmd> {
        if self.clip != Clip::Shape || self.element_image.is_some() {
            return Vec::new();
        }
        vec![self.clip_to(&self.shape, at)]
    }

    /// Its CSS cut, unless a layer's composite cuts it.
    fn cut(&self, at: &Placed) -> Option<DrawCmd> {
        match &self.clip {
            Clip::Cut(shape) if !self.layered() => Some(self.clip_to(shape, at)),
            _ => None,
        }
    }

    /// The clip to `shape`.
    fn clip_to(&self, shape: &ShapeKind, at: &Placed) -> DrawCmd {
        let r = if matches!(shape, ShapeKind::Path { .. }) {
            at.rect
        } else {
            self.frame(at)
        };
        DrawCmd::PushClip {
            geometry: self.geometry(shape, at),
            center: (r.x + r.width / 2.0, r.y + r.height / 2.0),
            half: (r.width / 2.0, r.height / 2.0),
        }
    }

    /// What draws after the subtree: the end of the cut, the strokes, then
    /// the layer's composite.
    pub(crate) fn leave(&self, theme: &Theme, at: &Placed) -> Vec<DrawCmd> {
        if self.element_image.is_some() {
            return vec![end_layer(None, at, 1.0, Blend::Normal)];
        }
        let mut out = Vec::new();
        if self.clip == Clip::Shape {
            out.push(DrawCmd::PopClip);
        }
        out.extend(self.strokes(theme, at));
        if self.cut(at).is_some() {
            out.push(DrawCmd::PopClip);
        }
        out.extend(self.composite(theme, at));
        out
    }

    /// [`ElementDraw::leave`] for a draw whose strokes were drawn with its
    /// fills ([`ElementDraw::strokes`] after [`ElementDraw::enter`]): a
    /// style part's border, under its label, so the two batch on their
    /// levels (docs/ui.md §3.3).
    pub(crate) fn leave_stroked(&self, theme: &Theme, at: &Placed) -> Vec<DrawCmd> {
        if self.element_image.is_some() {
            return self.leave(theme, at);
        }
        let mut out = Vec::new();
        if self.clip == Clip::Shape || self.cut(at).is_some() {
            out.push(DrawCmd::PopClip);
        }
        out.extend(self.composite(theme, at));
        out
    }

    /// Its strokes.
    pub(crate) fn strokes(&self, theme: &Theme, at: &Placed) -> Vec<DrawCmd> {
        let mut out = Vec::new();
        let (opacity, blend) = if self.composited() {
            (at.alpha, Blend::Normal)
        } else {
            (self.opacity * at.alpha, self.blend)
        };
        // Widths and dashes in view-box units scale with its fit (uniform:
        // `meet` scales both axes alike).
        let k = self.fit(at).map_or(1.0, |(k, ..)| k);
        let mode = |s: &Stroke| {
            og_draw::UiMode::Stroke(og_draw::UiStroke {
                width: s.width * k,
                align: s.align,
                dash: s.dash.map(|d| d * k),
                dash_offset: s.dash_offset * k,
                dash_cap: s.dash_cap,
            })
        };
        for s in &self.strokes {
            out.push(self.shape(at, self.paint(&s.paint, theme, at), mode(s), opacity, blend));
        }
        for (kind, s) in &self.outlines {
            out.push(self.shape_of(
                kind,
                at,
                self.paint(&s.paint, theme, at),
                mode(s),
                opacity,
                blend,
            ));
        }
        if let Some(borders) = &self.borders {
            let placed = Placed {
                alpha: opacity,
                ..*at
            };
            for mut cmd in borders.lines(&placed) {
                if let DrawCmd::Shape { shape, .. } = &mut cmd {
                    shape.blend = blend;
                }
                out.push(cmd);
            }
        }
        out
    }

    /// The layer's composite, when it draws offscreen.
    fn composite(&self, theme: &Theme, at: &Placed) -> Vec<DrawCmd> {
        let mut out = Vec::new();
        let masked = self.masked();
        if self.layered() {
            // Each inner stage composites its whole layer into the next.
            for _ in self.stages().1 {
                out.push(DrawCmd::EndLayer {
                    geometry: None,
                    rect: at.rect,
                    opacity: 1.0,
                    blend: Blend::Normal,
                    clip: at.clip,
                    ground: None,
                });
            }
            // The composite covers the layer; a mask or a material clips
            // to the element's shape only when it clips, and to its CSS cut.
            let geometry = match &self.clip {
                Clip::Shape => Some(self.geometry(&self.shape, at)),
                Clip::Cut(shape) => Some(self.geometry(shape, at)),
                _ => None,
            };
            out.push(DrawCmd::EndLayer {
                geometry,
                rect: at.rect,
                opacity: if masked { 1.0 } else { self.opacity },
                blend: if masked { Blend::Normal } else { self.blend },
                clip: at.clip,
                ground: None,
            });
        }
        if let Some(b) = self.boxes.as_ref().filter(|_| masked) {
            // The mask (CSS Masking 1 §6.1): its layers, the bottom first,
            // each combined with those under it by its operator, in a
            // layer that keeps the subtree's where its alpha is.
            out.push(plain_layer(None));
            out.extend(self.box_layers(&b.masks, theme, at, 1.0, |l| l.blend));
            out.push(end_layer(None, at, 1.0, Blend::DestinationIn));
            let shaped = matches!(self.clip, Clip::Shape);
            out.push(end_layer(
                shaped.then(|| self.geometry(&self.shape, at)),
                at,
                self.opacity,
                self.blend,
            ));
        }
        out
    }

    /// Its text effects over the line box `bounds`. `cell`: a pixel
    /// font's pixel at the text's size (CSS px; 0 for an outline font),
    /// which an outline's width and a shadow's offset are whole multiples
    /// of, at least one, and a shadow is hard: a fractional stroke would
    /// soften the cells' edges ([`snap_to_cells`]).
    pub(crate) fn text_effects(
        &self,
        theme: &Theme,
        bounds: Rect,
        cell: f32,
    ) -> Option<og_draw::UiTextEffects> {
        let t = self.text.as_ref()?;
        Some(og_draw::UiTextEffects {
            text: 0,
            shadows: t
                .shadows
                .iter()
                .map(|s| og_draw::UiTextShadow {
                    offset: Vec2::from(s.offset.map(|v| snap_to_cells(v, cell))),
                    blur: if cell > 0.0 { 0.0 } else { s.blur },
                    color: s.color.resolve(theme),
                })
                .collect(),
            outline: t.outline.as_ref().map(|o| og_draw::UiTextOutline {
                width: snap_to_cells(o.width, cell),
                color: o.color.resolve(theme),
                align: o.align,
            }),
            fill: t.fill.as_ref().map(|g| g.resolve(theme)),
            fill_box: [bounds.x, bounds.y, bounds.width, bounds.height],
            transform: [1.0, 0.0, 0.0, 1.0],
        })
    }

    /// Wraps the text commands an element pushed in its effects.
    /// `grid`: the text's font's pixel grid (0: an outline font).
    pub(crate) fn style_text(&self, theme: &Theme, cmds: &mut [DrawCmd], grid: u32) {
        if self.text.is_none() {
            return;
        }
        for c in cmds.iter_mut() {
            if let DrawCmd::Text { bounds, size, .. } = c {
                let cell = if grid > 0 { *size / grid as f32 } else { 0.0 };
                let Some(fx) = self.text_effects(theme, *bounds, cell) else {
                    return;
                };
                let text = std::mem::replace(c, DrawCmd::PopClip);
                *c = DrawCmd::StyledText {
                    text: Box::new(text),
                    effects: Box::new(fx),
                };
            }
        }
    }
}

/// One thing a Rive artboard draws as a draw command, cut to `clip`: a
/// shape; a text piece (styled when it carries a gradient, a stroke or a
/// map of its own); a cut to an outline and its end.
fn rive_cmd(d: crate::rive::Drawn, clip: Rect) -> DrawCmd {
    use crate::rive::Drawn;
    match d {
        Drawn::Shape(shape) => DrawCmd::Shape { shape, clip },
        Drawn::Clip(path) => {
            let (lo, hi) = path.bounds().unwrap_or_default();
            let c = (lo + hi) * 0.5;
            DrawCmd::PushClip {
                geometry: og_draw::UiGeometry::Path(std::sync::Arc::new(path.map(|q| q - c))),
                center: (c.x, c.y),
                half: ((hi.x - lo.x) / 2.0, (hi.y - lo.y) / 2.0),
            }
        }
        Drawn::Unclip => DrawCmd::PopClip,
        Drawn::Text(t) => {
            let [bx, by, bw, bh] = t.bounds;
            let text = DrawCmd::Text {
                x: t.pen.x,
                baseline: t.pen.y,
                size: t.size,
                text: t.text,
                color: t.color,
                font: t.font,
                bounds: Rect::new(bx, by, bw, bh),
                clip,
                tracking: t.tracking,
                weight: 0.0,
                glow: None,
                placements: t.placements,
            };
            if t.gradient.is_none() && t.outline.is_none() && t.transform == [1.0, 0.0, 0.0, 1.0] {
                return text;
            }
            let (fill, fill_box) = match t.gradient {
                Some((g, b)) => (Some(g), b),
                None => (None, t.bounds),
            };
            DrawCmd::StyledText {
                text: Box::new(text),
                effects: Box::new(og_draw::UiTextEffects {
                    text: 0,
                    shadows: Vec::new(),
                    outline: t.outline.map(|(width, color)| og_draw::UiTextOutline {
                        width,
                        color,
                        align: og_draw::UiAlign::Center,
                    }),
                    fill,
                    fill_box,
                    transform: t.transform,
                }),
            }
        }
    }
}

/// A text effect's length (CSS px) in whole cells of `cell` px, at least
/// one when it is not zero; `v` itself when `cell` is 0 (an outline font).
pub(crate) fn snap_to_cells(v: f32, cell: f32) -> f32 {
    if cell <= 0.0 || v == 0.0 {
        return v;
    }
    (v.abs() / cell).round().max(1.0) * cell * v.signum()
}

impl ShapeKind {
    /// Whether the area a closed shape over a `size` box fills, by its
    /// fill rule, holds `p` (CSS px from the box's top left); an open
    /// line holds nothing.
    pub(crate) fn holds(&self, size: [f32; 2], p: Vec2) -> bool {
        let (lines, closed, rule) = outline(self, [0.0, 0.0, size[0], size[1]], 0.25);
        let w: i32 = lines.iter().map(|c| winding(c, p)).sum();
        closed
            && match rule {
                FillRule::EvenOdd => w % 2 != 0,
                _ => w != 0,
            }
    }
}

/// A shape's contours as polylines in its own units (`frame`: the rect a
/// rect or ellipse fills), whether they are closed, and its fill rule.
fn outline(kind: &ShapeKind, frame: [f32; 4], tol: f32) -> (Vec<Vec<Vec2>>, bool, FillRule) {
    let [x, y, w, h] = frame;
    match kind {
        ShapeKind::Path {
            points,
            closed,
            filled,
            curves,
            fill_rule,
            contours,
            ..
        } => {
            let whole = [points.len() as u32];
            let counts: &[u32] = if contours.is_empty() {
                &whole
            } else {
                contours
            };
            let mut rest: &[[f32; 2]] = points;
            let mut out = Vec::new();
            for &n in counts {
                let (c, more) = rest.split_at((n as usize).min(rest.len()));
                rest = more;
                let Some(first) = c.first() else { continue };
                let mut line = vec![Vec2::from(*first)];
                if *curves {
                    for seg in c[1..].as_chunks::<3>().0 {
                        let [p0, p1, p2, p3] = [
                            *line.last().unwrap(),
                            seg[0].into(),
                            seg[1].into(),
                            seg[2].into(),
                        ];
                        let len = p0.distance(p1) + p1.distance(p2) + p2.distance(p3);
                        let steps = ((len / tol.max(1e-3)).sqrt().ceil() as usize).clamp(1, 64);
                        for k in 1..=steps {
                            let t = k as f32 / steps as f32;
                            let u = 1.0 - t;
                            line.push(
                                p0 * (u * u * u)
                                    + p1 * (3.0 * u * u * t)
                                    + p2 * (3.0 * u * t * t)
                                    + p3 * (t * t * t),
                            );
                        }
                    }
                } else {
                    line.extend(c[1..].iter().map(|p| Vec2::from(*p)));
                }
                out.push(line);
            }
            (out, *closed || *filled, *fill_rule)
        }
        ShapeKind::Ellipse => {
            let (c, r) = (Vec2::new(x + w * 0.5, y + h * 0.5), Vec2::new(w, h) * 0.5);
            let line = (0..64)
                .map(|k| {
                    let a = k as f32 / 64.0 * std::f32::consts::TAU;
                    c + r * Vec2::new(a.cos(), a.sin())
                })
                .collect();
            (vec![line], true, FillRule::NonZero)
        }
        _ => {
            let line = vec![
                Vec2::new(x, y),
                Vec2::new(x + w, y),
                Vec2::new(x + w, y + h),
                Vec2::new(x, y + h),
            ];
            (vec![line], true, FillRule::NonZero)
        }
    }
}

/// The winding number of a closed polyline around `q`.
fn winding(c: &[Vec2], q: Vec2) -> i32 {
    let mut w = 0;
    for (k, a) in c.iter().enumerate() {
        let b = c[(k + 1) % c.len()];
        let cross = (b.x - a.x) * (q.y - a.y) - (q.x - a.x) * (b.y - a.y);
        if a.y <= q.y && b.y > q.y && cross > 0.0 {
            w += 1;
        } else if a.y > q.y && b.y <= q.y && cross < 0.0 {
            w -= 1;
        }
    }
    w
}

/// The distance from `q` to a polyline.
fn distance(c: &[Vec2], closed: bool, q: Vec2) -> f32 {
    let segs = if closed {
        c.len()
    } else {
        c.len().saturating_sub(1)
    };
    let mut best = c.first().map_or(f32::MAX, |p| p.distance(q));
    for k in 0..segs {
        let (a, b) = (c[k], c[(k + 1) % c.len()]);
        let ab = b - a;
        let t = ((q - a).dot(ab) / ab.length_squared().max(1e-12)).clamp(0.0, 1.0);
        best = best.min((a + ab * t).distance(q));
    }
    best
}
