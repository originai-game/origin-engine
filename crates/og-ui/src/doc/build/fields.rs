//! Render-field replacement from evaluated inputs. No binding evaluation,
//! widget input processing, animation sampling or document emission lives here.
use super::*;

#[derive(Clone, Debug)]
pub(crate) struct RenderFields {
    pub(super) index: usize,
    kind: &'static str,
    element: Element,
    widget_props: Props,
    document_props: Props,
    pub(super) text: String,
    response: crate::Response,
    amounts: (f32, f32),
    press: f32,
    texture: Option<og_render::TextureHandle>,
    /// The textures of the asset handles its `background`'s `url()`s name,
    /// looked up while the host was at hand ([`css_textures`]).
    textures: Vec<(f64, og_render::TextureHandle)>,
    art: Option<String>,
    /// Hidden by something other than the document's `display`.
    hidden: bool,
}

impl RenderFields {
    pub(super) fn capture(
        ui: &Ui,
        index: usize,
        kind: &'static str,
        response: crate::Response,
        document_props: Props,
        texture: Option<og_render::TextureHandle>,
    ) -> Self {
        let node = &ui.nodes[index];
        Self {
            index,
            kind,
            element: node.el,
            widget_props: node.props.clone(),
            document_props,
            text: node.probe_text.clone(),
            response,
            amounts: node.el.part.map_or((0.0, 0.0), |p| (p.hover, p.press)),
            press: match node.widget {
                crate::layout::Widget::Button { press, .. } => press,
                _ => 0.0,
            },
            texture,
            textures: Vec::new(),
            art: (kind == "art").then(|| ui.last_art_path()),
            hidden: node.hidden,
        }
    }
}

impl RenderFields {
    /// With the textures `r`'s background `url(<handle>)`s name in `env`.
    pub(super) fn with_textures(mut self, r: &Resolved, env: &dyn crate::bind::Env) -> Self {
        self.textures = css_textures(r, env);
        self
    }

    /// A list or grid opens with its styled width, height, padding and gap;
    /// its baseline is the box without them.
    pub(super) fn unstyle_virtual(&mut self) {
        self.element.width = Size::grow();
        self.element.height = Size::grow();
        self.element.padding = Insets::default();
        self.element.gap = 0.0;
    }
}

/// The textures of the asset handles (`og.load.texture`) an element's
/// `background` names by `url(<handle>)`: settlement resolves its fields
/// again without the host, so they are looked up when it builds.
fn css_textures(r: &Resolved, env: &dyn crate::bind::Env) -> Vec<(f64, og_render::TextureHandle)> {
    let Some(Ok((layers, _))) = r
        .str("backgroundImage")
        .and_then(crate::doc::paint_css::background)
    else {
        return Vec::new();
    };
    layers
        .iter()
        .filter_map(|l| match &l.image {
            crate::doc::paint_css::CssImage::Url(u) => {
                let h = u.parse::<f64>().ok()?;
                Some((h, env.texture(h)?))
            }
            _ => None,
        })
        .collect()
}

impl Ui {
    /// The CSS border, outline, shadows and overflow of an element, with
    /// what they do to its box: a border's width pads it, an outline set
    /// replaces the focus ring, a scrolling overflow scrolls it, and a
    /// shadow takes its background under it.
    fn box_decor(
        &self,
        r: &Resolved,
        el: &mut Element,
        kind: &str,
        textures: &[(f64, og_render::TextureHandle)],
        warn: &mut Vec<String>,
    ) -> Option<crate::decor::BoxDecor> {
        use crate::decor::{BoxDecor, Line, LineStyle, Shadow};
        let theme = self.theme();
        let current = r.color("color").unwrap_or(theme.palette.ink);
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
                    Some([og_core::ogt::Value::String(hex)]) => crate::style_file::parse_color(hex),
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
        let css = self.css_paint(r, current, textures, warn);
        // A clip, mask, blend, filter or material cuts or composites the
        // background too: it is drawn with them.
        if css.as_ref().is_some_and(|c| {
            c.clip.is_some()
                || c.mask.is_some()
                || c.blend.is_some()
                || c.layer.is_some()
                || !c.filters.is_empty()
        }) && d.background.is_none()
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
            let (mut x, mut y) = (x.unwrap_or("visible"), y.unwrap_or("visible"));
            if x == "visible" && y != "visible" {
                x = "auto";
            }
            if y == "visible" && x != "visible" {
                y = "auto";
            }
            let scrolls = |a: &str| matches!(a, "auto" | "scroll");
            if kind == "scroll" || schema::kind(kind).is_some_and(|k| k.container) {
                el.scroll = scrolls(y);
                el.scroll_x = scrolls(x);
            }
            d.clip = matches!(x, "hidden" | "clip") || matches!(y, "hidden" | "clip");
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
        // A dressed part takes the style's corner unless CSS writes one.
        if r.has("radius") && (el.part.is_some() || el.frame.is_some()) {
            d.radius = Some(el.radius);
        }
        (d != BoxDecor::default()).then_some(d)
    }

    /// The element's CSS paint (`docs/ui.md`, CSS paint); values the
    /// stylesheet already checked, so one that fails here is skipped.
    fn css_paint(
        &self,
        r: &Resolved,
        current: Srgba,
        textures: &[(f64, og_render::TextureHandle)],
        warn: &mut Vec<String>,
    ) -> Option<crate::doc::paint_css::CssPaint> {
        use crate::doc::paint_css as p;
        let mut c = p::CssPaint::default();
        if let Some(Ok((mut layers, _))) = r.str("backgroundImage").and_then(p::background) {
            if let Err(e) = p::longhands(
                &mut layers,
                r.str("backgroundSize"),
                r.str("backgroundPosition"),
                r.str("backgroundRepeat"),
            ) {
                warn.push(e);
            }
            for l in layers {
                let (texture, uv) = match l.image {
                    p::CssImage::None => continue,
                    p::CssImage::Gradient(g) => {
                        c.images.push(p::BgLayer::Gradient(g));
                        continue;
                    }
                    // A texture by its handle, else an atlas icon by name,
                    // as `<image href>` resolves one.
                    p::CssImage::Url(u) => match u.parse::<f64>() {
                        Ok(h) => match textures.iter().find(|(k, _)| *k == h) {
                            Some((_, t)) => (*t, [0.0, 0.0, 1.0, 1.0]),
                            None => {
                                warn.push(format!("background: url({u}) names no texture"));
                                continue;
                            }
                        },
                        Err(_) => match self.icons().find(&u) {
                            Some(crate::ui::IconCell::Image(t, uv)) => (t, uv),
                            Some(_) => {
                                warn.push(format!(
                                    "background: url({u}) is a glyph icon, not an image: write <icon name=\"{u}\">"
                                ));
                                continue;
                            }
                            None => {
                                warn.push(format!("background: url({u}) names no atlas image"));
                                continue;
                            }
                        },
                    },
                };
                c.images.push(p::BgLayer::Image {
                    texture,
                    uv,
                    size: l.size,
                    position: l.position,
                    repeat: l.repeat,
                });
            }
        }
        c.mask = r.str("maskImage").and_then(|v| p::mask(v).ok()).flatten();
        if let Some(Ok(Some(e))) = r.str("backdropFilter").map(p::backdrop) {
            c.effects.push(e);
        }
        if let Some(Ok(fns)) = r.str("filter").map(|v| p::filter_fns(v, "filter")) {
            let theme = self.theme();
            c.filters =
                p::filter_layers(&fns, |c| c.map_or(current.linear(), |c| c.resolve(theme)));
        }
        c.blend = r.str("mixBlendMode").and_then(|v| p::blend(v).ok());
        if let Some(Ok(s)) = r
            .str("textShadow")
            .map(|v| p::text_shadows(v, crate::paint::ColorRef::Literal(current)))
        {
            c.text_shadows = s;
        }
        c.clip = r.str("clipPath").and_then(|v| p::clip(v).ok()).flatten();
        if let Some(Ok(Some(l))) = r.str("material").map(p::material) {
            c.layer = Some(l);
        }
        (c != p::CssPaint::default()).then_some(c)
    }

    pub(super) fn apply_document_fields(&mut self, n: &DocNode, r: &Resolved, base: &RenderFields) {
        let i = base.index;
        self.nodes[i].el = base.element;
        self.nodes[i].props = base.widget_props.clone();
        let css = r.text.as_deref();
        self.nodes[i].accent = css.and_then(|t| t.accent);
        self.nodes[i].caret = css.and_then(|t| t.caret);
        // `display: none` that a container query decides keeps its element
        // built, so settlement can show it again.
        self.nodes[i].hidden = base.hidden || (r.has("visible") && !r.flag("visible"));
        match base.kind {
            "label" => {
                let desc = r.label_desc(self.theme(), self.direction == og_i18n::Direction::Rtl);
                let shown;
                let base = match r.text.as_deref() {
                    Some(t) => {
                        shown = RenderFields {
                            text: t.shown(&base.text, &self.locale),
                            ..base.clone()
                        };
                        &shown
                    }
                    None => base,
                };
                if r.get("spans").is_none()
                    && let Some((span, style)) = r.css_span(&base.text)
                {
                    self.apply_text_spans(
                        i,
                        &base.text,
                        std::slice::from_ref(&span),
                        crate::RichTextDesc {
                            step: desc.step,
                            color: desc.color,
                            align: desc.align,
                            nowrap: desc.nowrap,
                            width: desc.width,
                            font: desc.font,
                        },
                        |_| style.clone(),
                    );
                } else if let Some(V::Spans(Ok(spans))) = r.get("spans") {
                    self.apply_text_spans(
                        i,
                        &base.text,
                        spans,
                        crate::RichTextDesc {
                            step: desc.step,
                            color: desc.color,
                            align: desc.align,
                            nowrap: desc.nowrap,
                            width: desc.width,
                            font: desc.font,
                        },
                        |span| {
                            r.span_styles
                                .iter()
                                .find(|(attribute, _)| attribute == span)
                                .map(|(_, style)| style.clone())
                                .unwrap_or_default()
                        },
                    );
                } else {
                    let width = desc.width;
                    self.apply_label(i, &base.text, desc);
                    // Path text aligns along its path, not in its box: it
                    // keeps the box its path gives (a `circle()` centres in
                    // it), never the column's width a centred line takes.
                    if r.has("textPath") {
                        self.nodes[i].el.width = width;
                    }
                }
            }
            "button" => self.apply_button(
                i,
                &base.text,
                r.button_desc(),
                base.response,
                base.amounts,
                base.press,
            ),
            "image" => self.apply_image(
                i,
                crate::ImageDesc {
                    texture: base.texture,
                    tint: r.color("tint"),
                    width: r.size("width").unwrap_or_default(),
                    height: r.size("height").unwrap_or_default(),
                    aspect: r
                        .nums("ratio")
                        .filter(|n| n.len() == 2 && n[1] > 0.0)
                        .map_or(r.num("aspect").unwrap_or(0.0), |n| n[0] / n[1]),
                    label: base.text.clone(),
                    uv: r.nums("uv").and_then(|n| <[f32; 4]>::try_from(n).ok()),
                },
            ),
            "icon" => {
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
                        secondary_tint: r.color("secondaryTint"),
                        label: r.str("name").map(str::to_string),
                    },
                )
            }
            "art" => {
                let path = base.art.clone().expect("art render fields");
                let spec = crate::slot::SlotSpec::from_props(
                    &path,
                    |name| r.str(name).map(str::to_string),
                    |name| r.nums(name).map(<[f32]>::to_vec),
                );
                if let Some(slot) = self.art_slot_mut(&path) {
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
                    },
                );
            }
            "details" => {
                // Its fields are its content column's; the build gives its
                // root (the parent) the CSS width too (`DetailsDesc.width`).
                let root = self.nodes[i].parent;
                if root != crate::layout::NONE {
                    self.nodes[root].el.width = r.size("width").unwrap_or(Size::grow());
                }
            }
            "spacer" => {
                let size = r.size("width").or(r.size("height")).unwrap_or(Size::grow());
                self.nodes[i].el.width = size;
                self.nodes[i].el.height = size;
            }
            _ => {}
        }
        // Widget-owned sizing is established above; document box overrides
        // replace their previous values, including when a declaration vanishes.
        let mut el = self.nodes[i].el;
        let em = self.em_of(r);
        if schema::kind(base.kind).is_some_and(|kind| kind.container) {
            el = Self::box_of(r, el, em);
            if !matches!(base.kind, "panel" | "scroll" | "list" | "cells") {
                // A box is a column (CSS block), or a row as `display: flex`.
                let initial = match (base.kind, r.str("display")) {
                    ("box", Some("flex")) => "row",
                    ("box", _) if r.str("markerPosition") == Some("inside") => "row",
                    // A button's content is a row in it.
                    ("button", _) => "row",
                    (kind, _) => kind,
                };
                let direction = r.str("direction").unwrap_or(initial);
                el.direction = match direction {
                    "row" | "rowReverse" => Direction::Row,
                    _ => Direction::Column,
                };
                el.reverse = direction.ends_with("Reverse");
                // A flex box's lines share its free room across it unless
                // told otherwise (`align-content: normal`, CSS Flexbox §8.4).
                if base.kind == "box"
                    && r.str("display") == Some("flex")
                    && el.align_content.is_none()
                {
                    el.align_content = Some(crate::Align::Stretch);
                }
                // A `display: block` box flows its children as CSS blocks.
                el.block = initial == "box"
                    && el.direction == Direction::Column
                    && r.str("display") != Some("grid");
                // A block-level box in a block's flow fills its width
                // (CSS 2 §10.3.3: `width: auto`).
                let parent = self.nodes[i].parent;
                if base.kind == "box"
                    && !r.has("width")
                    && parent != crate::layout::NONE
                    && self.nodes[parent].el.block
                {
                    el.width = Size::grow();
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
                "frame",
                "aspect",
                "interactive",
                "focusable",
                "adjustable",
                "wrap",
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
            if r.has("width") && !matches!(base.kind, "label" | "button" | "image" | "art") {
                el.width = r.size("width").unwrap_or(el.width);
            }
        }
        let mut warn = Vec::new();
        let decor = self.box_decor(r, &mut el, base.kind, &base.textures, &mut warn);
        for w in warn {
            self.warnings.push(format!("line {}: {w}", n.line));
        }
        let el = Self::sized(r, Self::item_of(r, el));
        self.nodes[i].el = el;
        self.nodes[i].decor = decor.map(Box::new);
        let mut props = Self::resolved_props(n, r);
        if r.flag("dataOgInstant") {
            // `data-og-instant`: its first entrance lands settled (the
            // screen behind the web's play gate); the motion state then
            // knows it, so a later build does not replay it, and a later
            // entrance (after it left) plays.
            if self.docs.entered.insert(self.nodes[i].key)
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
            self.nodes[i].props = props;
        }
        // The roles and levels HTML's tags give (landmarks, lists, headings).
        if let Some(role) = n.role {
            self.nodes[i].a11y.role = Some(role);
        }
        if n.level > 0 {
            self.nodes[i].a11y.level = Some(n.level);
        }
        // A split screen's region (`Props::player`).
        if let Some(k) = r.num("player") {
            self.nodes[i].props.player = Some(k.clamp(0.0, 3.0) as u8);
        }
        // `role`, `aria-live` and `aria-atomic` over the tag's.
        if let Some(role) = r
            .str("role")
            .and_then(|r| crate::a11y::ROLES.iter().find(|&&k| k == r))
        {
            self.nodes[i].a11y.role = Some(role);
        }
        self.nodes[i].a11y.live = r.str("ariaLive").and_then(crate::a11y::Live::parse);
        self.nodes[i].a11y.atomic = r.get("ariaAtomic").map(|_| r.flag("ariaAtomic"));
        // `aria-label` (or a `<label for>`) names it over what it shows.
        if let Some(name) = r.str("name").filter(|n| !n.is_empty()) {
            self.nodes[i].a11y.name = Some(name.to_string());
        }
        self.apply_aria(i, r);
        if !self.building {
            self.apply_part_appearance(i);
            self.prepare_layout_node(i);
        }
    }
}

impl Ui {
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

impl Resolved {
    pub(super) fn intrinsic(&self) -> Self {
        Self {
            vals: self
                .vals
                .iter()
                .filter(|(spec, _)| {
                    let name = PROPS[*spec].name;
                    name != "size" && !super::super::css::STYLE.iter().any(|(p, _)| *p == name)
                })
                .cloned()
                .collect(),
            text: self.text.clone(),
            ..Self::default()
        }
    }

    pub(super) fn label_desc(&self, theme: &crate::Theme, rtl: bool) -> LabelDesc {
        let step = self
            .str("step")
            .and_then(|s| crate::style_file::ramp_step(&mut theme.type_ramp.clone(), s).copied());
        let css = self.text.as_deref();
        let step = match css {
            Some(t) => Some(t.step(step.unwrap_or(theme.type_ramp.body))),
            None => step,
        };
        // CSS Text 3 §6.1: left and right are physical; start and end follow
        // the direction (the line drawer mirrors Start and End).
        let align = match css
            .and_then(|t| t.align.as_deref())
            .or(self.str("textAlign"))
        {
            Some("center") => TextAlign::Center,
            Some("end") => TextAlign::End,
            Some("left") if rtl => TextAlign::End,
            Some("right") if !rtl => TextAlign::End,
            Some("justify" | "justify-all") => TextAlign::Justify,
            _ => TextAlign::Start,
        };
        LabelDesc {
            step,
            color: self.color("color"),
            on_scene: self.flag("onScene"),
            uppercase: self.flag("uppercase"),
            align,
            nowrap: css.and_then(|t| t.nowrap).unwrap_or(self.flag("nowrap")),
            width: self.size("width").unwrap_or_default(),
            font: self
                .str("font")
                .and_then(FontRole::from_name)
                .unwrap_or_default(),
            heading: self.flag("heading"),
            ellipsis: self.flag("ellipsis"),
            max_lines: self.num("maxLines").unwrap_or(0.0) as u32,
            fit_min: self.str("fitMin").and_then(|s| {
                crate::style_file::ramp_step(&mut theme.type_ramp.clone(), s).copied()
            }),
        }
    }

    /// Its text as one span styled by its CSS (`font-style`,
    /// `text-decoration`, `-webkit-text-stroke`, `-webkit-text-fill-color`)
    /// when the plain text path cannot draw that style.
    pub(super) fn css_span(&self, text: &str) -> Option<(crate::TextSpan, crate::TextSpanStyle)> {
        let t = self.text.as_deref().filter(|t| t.rich())?;
        let span = crate::TextSpan {
            name: "css".into(),
            position: 0,
            length: text.chars().count() as u32,
            properties: Default::default(),
        };
        let style = crate::TextSpanStyle {
            italic: t.italic,
            weight: t.weight,
            color: t.fill.map(crate::paint::ColorRef::Literal),
            outline: t.stroke.map(|(w, c)| {
                (
                    w,
                    crate::paint::ColorRef::Literal(
                        c.or(t.fill)
                            .or(self.color("color"))
                            .unwrap_or(crate::Srgba::rgb(0, 0, 0)),
                    ),
                )
            }),
            decoration: t.decoration.any().then(|| t.decoration.clone()),
            ..Default::default()
        };
        Some((span, style))
    }

    /// Its `-og-control-size`.
    pub(crate) fn control_size(&self) -> ControlSize {
        match self.str("size") {
            Some("small") => ControlSize::Small,
            Some("large") => ControlSize::Large,
            _ => ControlSize::Medium,
        }
    }

    pub(super) fn button_desc(&self) -> ButtonDesc {
        ButtonDesc {
            variant: match self.str("variant") {
                Some("primary") => ButtonVariant::Primary,
                Some("quiet") => ButtonVariant::Quiet,
                Some("danger") => ButtonVariant::Danger,
                Some("ghost") => ButtonVariant::Ghost,
                _ => ButtonVariant::Secondary,
            },
            size: self.control_size(),
            disabled: self.flag("disabled"),
            width: self.size("width"),
            autofocus: self.flag("autofocus"),
            square: self.flag("square"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(styled: bool) -> UiDocument {
        let css = if styled {
            "#Root { padding: 13px; gap: 7px; } #Title { text-transform: uppercase; color: #f0a020; position: absolute; top: 230px; } #Action { -og-control-size: large; width: 173px; } #Check, #Range { width: 219px; padding: 5px; }"
        } else {
            ""
        };
        UiDocument::parse(
            "fields.svelte",
            &format!(
                r#"
<meta name="component" content="fields">
<column id="Root">
  <text id="Title">Mixed Case</text>
  <button id="Action" variant="primary">Apply</button>
  <label><input type="checkbox" switch id="Check" checked={{checked}}> Enabled</label>
  <input type="range" id="Range" value={{amount}} show-value="false" />
</column>
<style>{css}</style>
"#
            ),
        )
        .unwrap()
    }

    #[test]
    fn settled_document_fields_match_fresh_styles_without_rebuilding_controls() {
        let env = crate::bind::NoEnv;
        let mut patched = Ui::default();
        let mut direct = Ui::default();
        for ui in [&mut patched, &mut direct] {
            ui.set_fonts(crate::Fonts {
                body: crate::UiFont {
                    handle: Some(og_render::FontHandle::from_bits(1)),
                    metrics: crate::FontMetrics::fixed(0.55),
                },
                ..Default::default()
            });
        }
        for styled in [true, false, true] {
            let target = Arc::new(document(styled));
            let mut vm = json!({"checked": true, "amount": 0.37});
            for (ui, doc) in [
                (&mut patched, document(!styled)),
                (&mut direct, (*target).clone()),
            ] {
                ui.define_document(doc);
                ui.begin(
                    crate::InputFrame::default(),
                    crate::Viewport::new(640.0, 480.0),
                );
                ui.component("Root", "fields", &mut vm, &env);
            }
            let build = Build {
                doc: target.clone(),
                mount: "Root".into(),
                vm: &vm,
                env: &env,
                locals: Vec::new(),
                row_key: String::new(),
                inherited: Arc::default(),
                events: Vec::new(),
                writes: Vec::new(),
                cached: None,
                recording: Default::default(),
            };
            // Resolve before settlement; the callback has only evaluated values
            // and the existing controls, not a viewmodel or a second build.
            let replacements: Vec<_> = patched
                .docs
                .built
                .iter()
                .map(|node| {
                    let key = crate::id::key_of_path(&node.path);
                    let values = patched.resolve(
                        &build,
                        &target.nodes[node.node],
                        &node.path,
                        &mut Built::default(),
                        &build.inherited,
                        &mut |_| false,
                    );
                    (
                        node.node,
                        values,
                        patched.render_fields(key).unwrap().clone(),
                    )
                })
                .collect();
            patched.watch_layout_container(patched.nodes[0].key);
            // The first measure of this subscription must run on every test frame.
            patched.unwatch_layout_container(patched.nodes[0].key);
            patched.watch_layout_container(patched.nodes[0].key);
            patched
                .end_with_layout_queries(|ui, _| {
                    let state = ui.save_state();
                    let events = ui.docs.queued.clone();
                    let count = ui.nodes.len();
                    for (node, values, fields) in &replacements {
                        ui.apply_document_fields(&target.nodes[*node], values, fields);
                    }
                    assert_eq!(ui.save_state(), state);
                    assert_eq!(ui.docs.queued, events);
                    assert_eq!(ui.nodes.len(), count);
                    Ok(replacements
                        .iter()
                        .map(|(_, _, fields)| fields.index)
                        .collect())
                })
                .unwrap();
            direct.end();
            assert_eq!(patched.elements(), direct.elements(), "styled={styled}");
            assert_eq!(
                patched.frame().items,
                direct.frame().items,
                "styled={styled}"
            );
            assert_eq!(patched.a11y(), direct.a11y(), "styled={styled}");
        }
    }
}
