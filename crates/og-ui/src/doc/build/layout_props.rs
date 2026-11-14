use super::*;
use crate::doc::schema::Kind;
use crate::element::{DragSpec, DropSpec};

impl Ui {
    /// The box of an element from its props, over `base`; `em` sizes its
    /// em margins. What `r` does not declare stays as `el` has it: a part's
    /// values are laid over the element its document already styled.
    pub fn box_of(r: &Resolved, mut el: Element, em: f32) -> Element {
        for (name, edge) in ["marginTop", "marginRight", "marginBottom", "marginLeft"]
            .into_iter()
            .zip(&mut el.margin)
        {
            if let Some(&[px, ems, pct, auto]) =
                r.nums(name).and_then(|n| <&[f32; 4]>::try_from(n).ok())
            {
                *edge = crate::Margin {
                    len: crate::Length {
                        pct,
                        px: px + ems * em,
                    },
                    auto: auto != 0.0,
                };
            }
        }
        if let Some(s) = r.size("width") {
            el.width = s;
        }
        if let Some(s) = r.size("height") {
            el.height = s;
        }
        if let Some(n) = r.nums("padding") {
            el.padding = insets_of(n);
        } else if let Some(n) = r.num("padding") {
            el.padding = Insets::all(n);
        }
        // The CSS longhands, over the shorthand and a widget's own padding.
        for (name, side) in [
            ("paddingTop", &mut el.padding.top),
            ("paddingRight", &mut el.padding.right),
            ("paddingBottom", &mut el.padding.bottom),
            ("paddingLeft", &mut el.padding.left),
        ] {
            if let Some(v) = r.num(name) {
                *side = v;
            }
        }
        if let Some(g) = r.num("gap") {
            el.gap = g;
        }
        if let Some(a) = r.str("align") {
            el.align = align_of(a);
            el.align_logical = matches!(a, "start" | "end");
        }
        if let Some(c) = r.str("cross") {
            el.cross = cross_of(c);
        }
        if r.has("wrap") {
            el.wrap = r.flag("wrap");
        }
        if r.has("wrapReverse") {
            el.wrap_reverse = r.flag("wrapReverse");
        }
        if let Some(a) = r.str("alignContent") {
            el.align_content = Some(align_of(a));
        }
        if let Some(j) = r.str("justifyItems") {
            el.justify_items = Some(cross_of(j));
        }
        for (gap, name) in el.gaps.iter_mut().zip(["columnGap", "rowGap"]) {
            if let Some(g) = r.num(name) {
                *gap = Some(g);
            }
        }
        // CSS Multi-column: a column count or width makes it a
        // multi-column container; its gap is the column gap (`normal`: 1em).
        let px = |v: &str| match v {
            "thin" => Some(1.0),
            "medium" => Some(3.0),
            "thick" => Some(5.0),
            v => v.strip_suffix("px").unwrap_or(v).parse::<f32>().ok(),
        };
        let count = r
            .str("columnCount")
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(0);
        let width = r.str("columnWidth").and_then(px).unwrap_or(0.0);
        let columns = (count > 0 || width > 0.0).then(|| {
            let rule = r
                .str("columnRuleStyle")
                .and_then(crate::decor::LineStyle::from_css)
                .zip(
                    r.str("columnRuleWidth")
                        .map_or(Some(3.0), px)
                        .filter(|w| *w > 0.0),
                )
                .map(|(style, width)| crate::decor::Line {
                    width,
                    style,
                    color: r
                        .color("columnRuleColor")
                        .or(r.color("color"))
                        .unwrap_or(crate::theme::LIGHT.ink),
                });
            let current = r.color("columnRuleColor").or(r.color("color")).is_none();
            crate::Multicol {
                count,
                width,
                gap: el.gaps[0].or(r.num("gap")).unwrap_or(em),
                fill_auto: r.str("columnFill") == Some("auto"),
                rule,
                current,
            }
        });
        if r.has("columnCount") || r.has("columnWidth") {
            el.multicol = columns;
        }
        if let Some(v) = r.str("columnSpan") {
            el.span_all = v == "all";
        }
        let forced = |v: Option<&str>| {
            matches!(
                v,
                Some("column" | "page" | "always" | "all" | "left" | "right" | "recto" | "verso")
            )
        };
        {
            use crate::layout::multicol::{AVOID_INSIDE, BREAK_AFTER, BREAK_BEFORE};
            for (name, bit) in [
                ("breakBefore", BREAK_BEFORE),
                ("breakAfter", BREAK_AFTER),
                ("breakInside", AVOID_INSIDE),
            ] {
                if let Some(v) = r.str(name) {
                    let on = if bit == AVOID_INSIDE {
                        v.starts_with("avoid")
                    } else {
                        forced(Some(v))
                    };
                    el.breaks = if on {
                        el.breaks | bit
                    } else {
                        el.breaks & !bit
                    };
                }
            }
        }
        // CSS Containment: layout and paint containment start a formatting
        // context of their own (CSS Containment 2 §3.2, §3.4).
        if [
            "contain",
            "contentVisibility",
            "containIntrinsicWidth",
            "containIntrinsicHeight",
        ]
        .iter()
        .any(|p| r.has(p))
        {
            el.contain = crate::doc::contain_css::contain(|p| r.str(p).map(str::to_owned));
        }
        if el
            .contain
            .has(crate::layout::Contain::LAYOUT | crate::layout::Contain::PAINT)
        {
            el.bfc = true;
        }
        // CSS `resize` applies where overflow is not `visible` (CSS UI 4
        // §5.2).
        let clips = ["overflowX", "overflowY"]
            .iter()
            .any(|p| r.str(p).is_some_and(|v| v != "visible"));
        if r.has("resize") {
            el.resize = match r.str("resize").map(crate::doc::contain_css::resize) {
                Some(Ok([x, y])) if clips => u8::from(x) | u8::from(y) << 1,
                _ => 0,
            };
        }
        for (bit, name) in [(1, "safeJustify"), (2, "safeAlign")] {
            if r.has(name) {
                el.safe = if r.flag(name) {
                    el.safe | bit
                } else {
                    el.safe & !bit
                };
            }
        }
        if let Some(c) = r.color("background") {
            el.background = Some(c);
        }
        if let Some(x) = r.num("radius") {
            el.radius = x;
        }
        if let Some(a) = r.num("aspect") {
            el.aspect = a;
        }
        if let Some(f) = r.num("fade") {
            el.fade = f.max(0.0);
        }
        if let Some(c) = r.color("fadeInto") {
            el.fade_into = Some(c);
        }
        // A context menu's element takes the presses that open it.
        if r.has("contextmenu") {
            el.interactive = true;
        }
        if r.has("interactive") {
            el.interactive = r.flag("interactive");
        }
        if r.has("focusable") {
            el.focusable = r.flag("focusable");
        }
        if r.has("adjustable") {
            el.adjustable = r.flag("adjustable");
        }
        el
    }

    /// `el` with its CSS flex and grid item properties (any element can be
    /// an item): its flex factors, `order`, and own alignments.
    pub(super) fn item_of(r: &Resolved, mut el: Element) -> Element {
        el.flex = (r.has("flexGrow") || r.has("flexShrink") || r.has("flexBasis")).then(|| {
            crate::layout::FlexItem {
                grow: r.num("flexGrow").unwrap_or(0.0),
                shrink: r.num("flexShrink").unwrap_or(1.0),
                basis: r.size("flexBasis"),
            }
        });
        el.order = r.num("order").unwrap_or(0.0) as i32;
        el.align_self = r.str("alignSelf").map(cross_of);
        el.justify_self = r.str("justifySelf").map(cross_of);
        if r.has("safeAlign") && r.has("alignSelf") {
            el.safe = (el.safe & !2) | u8::from(r.flag("safeAlign")) << 1;
        }
        el
    }

    /// `el` with its CSS min- and max- bounds and its box sizing: a px
    /// bound of a fit-content or stretch size joins the size's range (as
    /// the house form writes it), others bound whatever size it takes; a
    /// `content-box` size and its bounds grow by its padding and border.
    pub(super) fn sized(r: &Resolved, mut el: Element) -> Element {
        let bounds = [["minWidth", "maxWidth"], ["minHeight", "maxHeight"]]
            .map(|names| names.map(|name| r.size(name)));
        el.bounds = bounds;
        for (axis, size) in [&mut el.width, &mut el.height].into_iter().enumerate() {
            if let Size::Fit { min, max } | Size::Grow { min, max } = size {
                if let Some(Size::Fixed(v)) = el.bounds[axis][0] {
                    *min = v;
                    el.bounds[axis][0] = None;
                }
                if let Some(Size::Fixed(v)) = el.bounds[axis][1] {
                    *max = v;
                    el.bounds[axis][1] = None;
                }
            }
        }
        if r.str("boxSizing") == Some("contentBox") {
            let pad = [el.padding.horizontal(), el.padding.vertical()];
            let grow = |s: Size, p: f32| match s {
                Size::Fixed(v) => Size::Fixed(v + p),
                Size::Percent(f) => Size::Calc(crate::Calc {
                    args: [
                        crate::Length { pct: f, px: p },
                        crate::Length::default(),
                        crate::Length::default(),
                    ],
                    count: 1,
                    ..crate::Calc::default()
                }),
                Size::Calc(mut c) => {
                    for a in &mut c.args[..c.count.max(1) as usize] {
                        a.px += p;
                    }
                    Size::Calc(c)
                }
                Size::Fit { min, max } => Size::Fit {
                    min: if min > 0.0 { min + p } else { min },
                    max: max + p,
                },
                Size::Grow { min, max } => Size::Grow {
                    min: if min > 0.0 { min + p } else { min },
                    max: max + p,
                },
                // A content size already holds its padding.
                s @ (Size::MinContent | Size::MaxContent) => s,
            };
            el.width = grow(el.width, pad[0]);
            el.height = grow(el.height, pad[1]);
            for axis in 0..2 {
                for b in &mut el.bounds[axis] {
                    *b = b.map(|s| grow(s, pad[axis]));
                }
            }
        }
        el
    }

    /// Its anchor positioning (CSS Anchor Positioning 1) as authored, and
    /// whether any of it resolves only after layout; px insets and anchor
    /// names go into `p`.
    pub fn anchor_spec(r: &Resolved, p: &mut Props) -> (crate::anchor::AnchorSpec, bool) {
        // The standard insets (`top`, `left`, `inset`, …): px ones are its
        // edges; a percentage or `anchor()` resolves after layout.
        let mut spec = crate::anchor::AnchorSpec::default();
        let mut resolved_later = false;
        for (k, name) in ["insetTop", "insetRight", "insetBottom", "insetLeft"]
            .into_iter()
            .enumerate()
        {
            let Some(v) = r.str(name).filter(|v| *v != "auto") else {
                continue;
            };
            let Ok(inset) = crate::anchor::parse_inset(v) else {
                continue;
            };
            if let crate::anchor::Inset::Px(px) = inset {
                *[
                    &mut p.insets.top,
                    &mut p.insets.end,
                    &mut p.insets.bottom,
                    &mut p.insets.start,
                ][k] = Some(px);
            } else {
                resolved_later = true;
            }
            spec.base.insets[k] = Some(inset);
        }
        if let Some(n) = r.str("anchorName").filter(|n| !n.is_empty()) {
            p.anchor_names = n.split_whitespace().map(str::to_string).collect();
        }
        if let Some(a) = r.str("positionAnchor") {
            spec.anchor = a.to_string();
        }
        if let Some(a) = r.str("positionArea") {
            spec.base.area = crate::anchor::parse_area(a).ok().flatten();
            resolved_later |= spec.base.area.is_some();
        }
        for (k, name) in ["anchorWidth", "anchorHeight"].into_iter().enumerate() {
            if let Some(v) = r.str(name) {
                spec.base.sizes[k] = crate::anchor::parse_size(v).ok();
                resolved_later |= spec.base.sizes[k].is_some();
            }
        }
        if let Some(t) = r.str("positionTry") {
            spec.tries = crate::anchor::parse_tries(t).unwrap_or_default();
        }
        spec.order = match r.str("positionTryOrder") {
            Some("most-width") => crate::anchor::TryOrder::MostWidth,
            Some("most-height") => crate::anchor::TryOrder::MostHeight,
            _ => crate::anchor::TryOrder::Normal,
        };
        spec.visibility = match r.str("positionVisibility") {
            Some("always") => crate::anchor::Visibility::Always,
            Some("no-overflow") => crate::anchor::Visibility::NoOverflow,
            _ => crate::anchor::Visibility::AnchorsVisible,
        };
        spec.named = r.position_tries.clone();
        resolved_later |= !spec.tries.is_empty();
        (spec, resolved_later)
    }

    /// Document event ownership is separate from a box's computed style.
    pub fn resolved_props(n: &DocNode, r: &Resolved) -> Props {
        let mut p = Self::box_props(&n.kind, r);
        p.enter = n.enter.clone();
        p.exit = n.exit.clone();
        props(n, r, &mut p);
        p
    }

    /// Shared layout and presentation for document boxes and widget parts.
    pub(super) fn box_props(kind: &NodeKind, r: &Resolved) -> Props {
        let mut p = Props::default();
        let t = &mut p.transform;
        if let Some(v) = r.num("x") {
            t.x = v;
        }
        if let Some(v) = r.num("y") {
            t.y = v;
        }
        if let Some(v) = r.num("xShift") {
            t.shift_x = v;
        }
        if let Some(v) = r.num("yShift") {
            t.shift_y = v;
        }
        if let Some(v) = r.num("scale") {
            t.scale_x = v;
            t.scale_y = v;
        }
        if let Some(v) = r.num("scaleX") {
            t.scale_x = v;
        }
        if let Some(v) = r.num("scaleY") {
            t.scale_y = v;
        }
        if let Some(v) = r.num("rotate") {
            t.rotate = v;
        }
        if let Some([x, y]) = r.nums("pivot").and_then(|n| <[f32; 2]>::try_from(n).ok()) {
            t.pivot_x = x;
            t.pivot_y = y;
        }
        p.space3d = r.space3d.clone();
        if let Some(v) = r.num("opacity") {
            p.opacity = v.clamp(0.0, 1.0);
        }
        if let Some(v) = r.num("z") {
            p.z = v as i32;
        }
        p.position = match r.str("position") {
            Some("absolute") => Position::Absolute,
            Some("screen") => Position::Screen,
            Some("relative") => Position::Relative,
            Some("sticky") => Position::Sticky,
            _ => Position::Flow,
        };
        p.insets = Edges {
            start: r.num("start"),
            top: r.num("top"),
            end: r.num("end"),
            bottom: r.num("bottom"),
            ..Edges::default()
        };
        if p.position == Position::Flow && p.insets.is_set() {
            p.position = Position::Absolute;
        }
        let (spec, resolved_later) = Self::anchor_spec(r, &mut p);
        if resolved_later && matches!(p.position, Position::Absolute | Position::Screen) {
            p.anchor = Some(Box::new(spec));
        }
        if let Some(v) = r.num("dataHitSlop") {
            p.hit_slop = v;
        }
        // A `<div>` with `display: grid` lays out as a `<grid>` does; a
        // table lays its cells out in a grid of its own.
        let grid_box =
            *kind == NodeKind::Element(Kind::Box) && inner_display(r.str("display")) == "grid";
        if *kind == NodeKind::Element(Kind::Box) && inner_display(r.str("display")) == "table" {
            p.layout = LayoutMode::Grid;
        }
        match kind {
            NodeKind::Element(Kind::Stack) => p.layout = LayoutMode::Stack,
            NodeKind::Element(Kind::Grid) => {
                p.layout = LayoutMode::Grid;
                p.grid = grid_of(r);
            }
            _ if grid_box => {
                p.layout = LayoutMode::Grid;
                p.grid = grid_of(r);
            }
            _ => {}
        }
        let pair = |name: &str| r.nums(name).and_then(|n| <[f32; 2]>::try_from(n).ok());
        p.place = GridPlace {
            column: pair("cell").map_or(0, |c| c[0] as u16),
            row: pair("cell").map_or(0, |c| c[1] as u16),
            // A table cell's `colspan` and `rowspan` (0: to its group's end).
            column_span: r
                .num("colspan")
                .map(|v| v.clamp(1.0, 1000.0) as u16)
                .or(pair("span").map(|c| c[0] as u16))
                .unwrap_or(0),
            row_span: r
                .num("rowspan")
                .map(|v| {
                    if v < 1.0 {
                        u16::MAX
                    } else {
                        v.min(65534.0) as u16
                    }
                })
                .or(pair("span").map(|c| c[1] as u16))
                .unwrap_or(0),
            area: r.str("area").unwrap_or_default().to_string(),
            lines: [["columnStart", "columnEnd"], ["rowStart", "rowEnd"]].map(|axis| {
                axis.map(|k| {
                    r.str(k)
                        .and_then(|v| crate::element::GridLine::parse(v).ok())
                        .unwrap_or_default()
                })
            }),
        };
        if let Some(m) = r.str("match") {
            p.match_key = m.to_string();
        }
        if let Some(n) = r.str("viewTransitionName") {
            p.view_transition_name = n.to_string();
        }
        if let Some(c) = r.str("viewTransitionClass") {
            p.view_transition_class = c.split_whitespace().map(str::to_string).collect();
        }
        if let Some(g) = r.str("viewTransitionGroup") {
            p.view_transition_group = g.to_string();
        }
        if let Some(V::Transition(t)) = r.get("motion") {
            p.motion = Some(*t);
        }
        // Computed values transition in the build ([`animate`]).
        p.transitions = r
            .transitions
            .iter()
            .filter(|t| t.channels != 0)
            .cloned()
            .collect();
        p.animations = r.animations.clone();
        p.timelines = r.timelines.clone();
        p.timeline_scope = r.timeline_scope.clone();
        if let Some(V::Transition(t)) = r.get("layoutMotion") {
            p.layout_motion = Some(*t);
        }
        // `@starting-style` values the channels hold.
        for (name, value, _) in &r.starting {
            if super::motion::channels(name).is_none() {
                continue;
            }
            for entry in super::css::declarations(&format!("{name}: {value};"), 0)
                .into_iter()
                .flatten()
            {
                if let Some(prop) = crate::element::Prop::from_name(&entry.key)
                    && let Some(v) = entry
                        .values
                        .first()
                        .and_then(og_core::ogt::Value::as_number)
                        .map(og_core::ogt::Number::as_f32)
                {
                    p.starting.push((prop, v));
                }
            }
        }
        p.inert = r.flag("inert");
        let link = |name: &str| r.str(name).unwrap_or_default().to_string();
        p.nav = crate::element::NavLinks {
            up: link("navUp"),
            down: link("navDown"),
            left: link("navLeft"),
            right: link("navRight"),
            next: link("navNext"),
            prev: link("navPrev"),
        };
        p.focus_group = r.str("focusGroup").map(|w| crate::element::FocusGroup {
            wrap: w == "wrap",
            memory: r.flag("focusMemory"),
        });
        let g = crate::element::GestureSpec {
            tap: r.flag("tap"),
            double_tap: r.flag("doubleTap"),
            long_press: r.flag("longPress"),
            long_press_ms: r.num("longPressMs").unwrap_or(0.0).max(0.0),
            priority: r.num("gesturePriority").unwrap_or(0.0) as i32,
            ..crate::element::GestureSpec::default()
        };
        p.gestures = g.any().then_some(g);
        {
            // Checked when the CSS was lowered.
            use crate::doc::interact_css as ic;
            if let Some(Ok((images, c))) = r.str("cursor").map(ic::cursor) {
                (p.cursor_images, p.cursor) = (images, c);
            }
            let i = &mut p.interact;
            if let Some(Ok(v)) = r.str("pointerEvents").map(ic::pointer_events) {
                i.pointer_events = v;
            }
            if let Some(Ok(v)) = r.str("touchAction").map(ic::touch_action) {
                i.touch_action = v;
            }
            if let Some(Ok(v)) = r.str("visibility").map(ic::visibility) {
                i.visibility = v;
            }
        }
        if let Some(shape) = r.str("textPath") {
            // Checked when the CSS was lowered.
            p.text_path = crate::text_path::TextPath::from_css(
                shape,
                r.str("textPathOffset"),
                r.str("textPathSide"),
            )
            .ok()
            .flatten();
        }
        if let Some(s) = r
            .str("userSelect")
            .and_then(crate::element::UserSelect::from_name)
        {
            p.select = s;
        }
        p.customize = r.flag("customize").then(|| crate::element::Customize {
            key: r.str("customizeKey").unwrap_or_default().to_owned(),
            ..crate::element::Customize::default()
        });
        // A transition written under `prefers-reduced-motion` plays as
        // written; any other takes the theme's fallback.
        p.reduced = r
            .cascade
            .reduced_motion_authored
            .then_some(crate::Reduced::Keep);
        p
    }

    /// The element-model props of an element (§6.1) and its state groups.
    pub fn props_of(
        &self,
        b: &Build<'_>,
        n: &DocNode,
        key: u64,
        r: &Resolved,
        built: &mut Built,
    ) -> Props {
        let mut p = Self::resolved_props(n, r);
        for g in &n.groups {
            let mut group = StateGroup {
                name: g.name.clone(),
                states: Vec::new(),
                transitions: g.transitions.clone(),
            };
            for s in &g.states {
                let mut values = Vec::new();
                for v in &s.values {
                    match v {
                        StateValue::Prop(prop, x) => values.push((*prop, *x)),
                        StateValue::Length(prop, value) => {
                            if let Some(V::Num(x)) = lit(self, value) {
                                values.push((*prop, x as f32));
                            }
                        }
                        StateValue::Background(c) | StateValue::Ink(c) => {
                            let bg = matches!(v, StateValue::Background(_));
                            if let Some(c) = paint(self, c) {
                                let ch =
                                    [c.rgb[0], c.rgb[1], c.rgb[2]].map(|x| f32::from(x) / 255.0);
                                let props = if bg {
                                    [
                                        Prop::BackgroundR,
                                        Prop::BackgroundG,
                                        Prop::BackgroundB,
                                        Prop::BackgroundA,
                                    ]
                                } else {
                                    [Prop::InkR, Prop::InkG, Prop::InkB, Prop::InkA]
                                };
                                for (k, pr) in props.into_iter().enumerate() {
                                    values.push((pr, if k < 3 { ch[k] } else { c.alpha }));
                                }
                            }
                        }
                    }
                }
                group.states.push(VisualState {
                    name: s.name.clone(),
                    values,
                    on_enter: qualify(&b.doc.name, &s.on_enter),
                    on_exit: qualify(&b.doc.name, &s.on_exit),
                });
            }
            // The state: its binding (a bool takes the second state when
            // true, a string the state of that name), else a trigger's.
            let state = match &g.bind {
                Some(e) => {
                    let v = b.eval(e);
                    match v {
                        Value::String(s) if g.states.iter().any(|x| x.name == s) => Some(s),
                        other => {
                            let k = usize::from(truthy(&other)).min(g.states.len() - 1);
                            Some(g.states[k].name.clone())
                        }
                    }
                }
                None => self
                    .docs
                    .state
                    .memory
                    .states
                    .get(&(key, g.name.clone()))
                    .cloned(),
            };
            if g.name != "interaction" {
                let s = state.unwrap_or_else(|| g.states[0].name.clone());
                built.states.push((g.name.clone(), s.clone()));
                p.state.push((g.name.clone(), s));
            }
            p.states.push(group);
        }
        p
    }
}

impl crate::ui::Style {
    /// The size of an em for `r`'s element: its type step's, else the body's.
    pub fn em_of(&self, r: &Resolved, env: &crate::ui::Env) -> f32 {
        let mut ramp = env.theme.type_ramp;
        let body = ramp.body.size;
        if let Some(s) = r.text.as_ref().and_then(|t| t.size) {
            return s;
        }
        r.str("step")
            .and_then(|name| crate::style_file::ramp_step(&mut ramp, name))
            .map_or(body, |s| s.size)
    }
}

/// The element's drag and drop props from its attributes and handlers.
pub(super) fn props(n: &DocNode, r: &Resolved, p: &mut Props) {
    p.pointer = n.triggers.iter().any(|t| {
        matches!(
            t.when,
            When::PointerDown | When::PointerMove | When::PointerUp | When::Wheel | When::Pinch
        )
    });
    if r.flag("draggable") {
        p.drag = Some(DragSpec {
            return_home: true,
            tag: r.str("dataDragType").unwrap_or_default().to_string(),
            ..DragSpec::default()
        });
    }
    if n.triggers
        .iter()
        .any(|t| matches!(t.when, When::DragOver | When::Drop))
    {
        p.drop = Some(DropSpec {
            accept: r
                .str("dataAccept")
                .unwrap_or_default()
                .split_whitespace()
                .map(str::to_string)
                .collect(),
        });
    }
}
