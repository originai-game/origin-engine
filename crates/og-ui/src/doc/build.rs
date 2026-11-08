//! Mounting a document (docs/ui.md §2.1): its tree is built into the
//! immediate-mode [`Ui`] where it is mounted, every tick, so documents and
//! code mix in one tree. Props are resolved in order (the line, the
//! matching `at` blocks, a probe edit), bindings are evaluated against the
//! viewmodel ([`crate::bind`]), `for` builds a list's rows (only those in
//! view under `list` and `cells`), `slot`s are left for code to fill
//! ([`Ui::open_slot`]), triggers fire, and controls write their values back
//! into the viewmodel (two-way bindings). What the build saw is kept for
//! the probe: every element's document line, bindings and their values,
//! the slots and art slots, the probe's edits.

use super::schema::{self, PROPS, Ty};
use super::{
    Act, DocNode, Lit, NodeKind, Paint, PropValue, StateValue, UiDocument, When, parse_probe_value,
};
use crate::bind::{Env, Scope, number, set_path, text, truthy};
use crate::element::{
    Edges, GridFlow, GridPlace, GridSpec, LayoutMode, Position, Prop, Props, Track,
};
use crate::layout::{Align, Cross, Direction, Element, FontRole, Insets, NONE, Size, TextAlign};
use crate::motion::{StateGroup, VisualState};
use crate::style::Srgba;
use crate::ui::{ButtonDesc, ButtonVariant, ControlSize, LabelDesc, Ui};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// A resolved value of a prop.
#[derive(Clone, Debug, PartialEq)]
enum V {
    /// In f64, as JSON carries it: a bound handle (a texture's, `(1 << 32) |
    /// index`) needs every one of its 53 bits, which f32 would round to a
    /// power of two (texture 0). Layout reads it as f32 ([`Resolved::num`]).
    Num(f64),
    Nums(Vec<f32>),
    Bool(bool),
    Str(String),
    Size(Size),
    Color(Srgba),
    Transition(crate::motion::Transition),
    Presence(crate::motion::PresenceSpec),
    Tracks(Vec<Track>),
    Strs(Vec<String>),
    Spans(Result<Vec<crate::TextSpan>, String>),
}

/// An element's resolved props, by schema index.
#[derive(Clone, Default)]
struct Resolved {
    vals: Vec<(usize, V)>,
    cascade: cascade::Cascade,
    span_styles: Vec<(crate::TextSpan, crate::TextSpanStyle)>,
    transitions: Vec<crate::motion::css::CssTransition>,
    animations: Vec<crate::motion::css::CssAnimation>,
}

#[derive(Clone)]
struct ResolvedNode {
    id: String,
    path: String,
    key: u64,
    values: Resolved,
    probe: Built,
}

impl Resolved {
    fn get(&self, name: &str) -> Option<&V> {
        self.vals
            .iter()
            .rev()
            .find(|(s, _)| PROPS[*s].name == name)
            .map(|(_, v)| v)
    }
    fn has(&self, name: &str) -> bool {
        self.get(name).is_some()
    }
    fn num(&self, name: &str) -> Option<f32> {
        match self.get(name)? {
            V::Num(n) => Some(*n as f32),
            V::Bool(b) => Some(f32::from(u8::from(*b))),
            _ => None,
        }
    }
    /// A handle prop (a texture's) as bound, every bit kept.
    fn handle(&self, name: &str) -> Option<f64> {
        match self.get(name)? {
            V::Num(n) => Some(*n),
            _ => None,
        }
    }
    fn flag(&self, name: &str) -> bool {
        matches!(self.get(name), Some(V::Bool(true)))
    }
    fn str(&self, name: &str) -> Option<&str> {
        match self.get(name)? {
            V::Str(s) => Some(s),
            _ => None,
        }
    }
    fn size(&self, name: &str) -> Option<Size> {
        match self.get(name)? {
            V::Size(s) => Some(*s),
            V::Num(n) => Some(Size::Fixed(*n as f32)),
            _ => None,
        }
    }
    fn color(&self, name: &str) -> Option<Srgba> {
        match self.get(name)? {
            V::Color(c) => Some(*c),
            _ => None,
        }
    }
    fn nums(&self, name: &str) -> Option<&[f32]> {
        match self.get(name)? {
            V::Nums(n) => Some(n),
            _ => None,
        }
    }
}

fn paint(ui: &Ui, p: &Paint) -> Option<Srgba> {
    match p {
        Paint::Token(t) => ui.theme().color(t),
        Paint::Color(c) => Some(*c),
    }
}

/// A bound JSON value as a prop of type `ty`.
fn coerce(ui: &Ui, ty: Ty, v: &Value) -> Option<V> {
    Some(match ty {
        Ty::Num => V::Num(number(v)?),
        Ty::Bool => V::Bool(truthy(v)),
        Ty::Text | Ty::Name | Ty::Word(_) => {
            let s = text(v);
            if let Ty::Word(w) = ty
                && !w.contains(&s.as_str())
            {
                return None;
            }
            V::Str(s)
        }
        Ty::Size => match v {
            Value::String(s) => match s.as_str() {
                "grow" => V::Size(Size::grow()),
                "fit" => V::Size(Size::fit()),
                _ => V::Size(Size::Fixed(s.parse().ok()?)),
            },
            _ => V::Size(Size::Fixed(number(v)? as f32)),
        },
        Ty::Paint => {
            let s = v.as_str()?;
            let p = if s.starts_with('#') {
                Paint::Color(crate::style_file::parse_color(s)?)
            } else {
                Paint::Token(s.to_string())
            };
            V::Color(paint(ui, &p)?)
        }
        Ty::Num2 | Ty::Num4 | Ty::Insets => match v {
            Value::Array(a) => V::Nums(a.iter().filter_map(number).map(|x| x as f32).collect()),
            _ => V::Nums(vec![number(v)? as f32]),
        },
        Ty::Names => match v {
            Value::Array(a) => V::Strs(a.iter().map(text).collect()),
            _ => V::Strs(vec![text(v)]),
        },
        Ty::Spans => V::Spans(serde_json::from_value(v.clone()).map_err(|e| e.to_string())),
        Ty::Transition | Ty::Presence | Ty::Tracks => return None,
    })
}

fn lit(ui: &Ui, l: &Lit) -> Option<V> {
    Some(match l {
        Lit::Num(n) => V::Num(f64::from(*n)),
        Lit::Nums(n) => V::Nums(n.clone()),
        Lit::Lengths(ty, vals) => {
            let vals = super::resolve_lengths(vals, ui.theme()).ok()?;
            return lit(ui, &super::parse_lit(*ty, &vals).ok()?);
        }
        Lit::Bool(b) => V::Bool(*b),
        Lit::Str(s) => V::Str(s.clone()),
        Lit::Size(s) => V::Size(*s),
        Lit::Paint(p) => V::Color(paint(ui, p)?),
        Lit::Transition(t) => V::Transition(*t),
        Lit::Presence(p) => V::Presence(p.clone()),
        Lit::Tracks(t) => V::Tracks(t.clone()),
        Lit::Strs(s) => V::Strs(s.clone()),
    })
}

fn insets_of(n: &[f32]) -> Insets {
    match n {
        [a] => Insets::all(*a),
        [x, y] => Insets::xy(*x, *y),
        [x, y, _] => Insets::xy(*x, *y),
        [l, t, r, b, ..] => Insets {
            left: *l,
            top: *t,
            right: *r,
            bottom: *b,
        },
        [] => Insets::default(),
    }
}

fn anchor_of(s: &str) -> crate::layout::Anchor {
    use crate::layout::Anchor::*;
    match s {
        "top" => Top,
        "topRight" => TopRight,
        "left" => Left,
        "center" => Center,
        "right" => Right,
        "bottomLeft" => BottomLeft,
        "bottom" => Bottom,
        "bottomRight" => BottomRight,
        _ => TopLeft,
    }
}

/// One mount's build: the document, its viewmodel and the item variables in scope.
struct Build<'a> {
    doc: Arc<UiDocument>,
    mount: String,
    vm: &'a Value,
    env: &'a dyn Env,
    /// `(name, value, the viewmodel path it came from)`.
    locals: Vec<(String, Value, Option<String>)>,
    row_key: String,
    inherited: Arc<super::variables::Computed>,
    events: Vec<DocEvent>,
    writes: Vec<(String, Value)>,
    cached: Option<&'a retained::CachedInputs>,
}

impl Build<'_> {
    fn scope(&self) -> Scope<'_> {
        Scope {
            vm: self.vm,
            locals: self
                .locals
                .iter()
                .map(|(n, v, _)| (n.as_str(), v.clone()))
                .collect(),
            env: self.env,
        }
    }

    fn eval(&self, e: &crate::bind::Expr) -> Value {
        if let Some(cached) = self.cached {
            return cached.expression(e);
        }
        e.eval(&self.scope())
    }

    /// Where a write to `path` lands in the viewmodel: through an item
    /// variable to its list's entry.
    fn write_path(&self, path: &str) -> Option<String> {
        if self.cached.is_some() {
            return Some(path.to_owned());
        }
        let (root, rest) = path.split_once('.').unwrap_or((path, ""));
        for (n, _, origin) in self.locals.iter().rev() {
            if n == root {
                let o = origin.as_ref()?;
                return Some(if rest.is_empty() {
                    o.clone()
                } else {
                    format!("{o}.{rest}")
                });
            }
        }
        if root.starts_with('$') || root.starts_with(|c: char| c.is_ascii_uppercase()) {
            return None;
        }
        Some(path.to_string())
    }

    fn event(&mut self, path: &str, kind: &str, name: &str, value: Value) {
        self.events.push(DocEvent {
            mount: self.mount.clone(),
            path: path.to_string(),
            row_key: self.row_key.clone(),
            kind: kind.to_string(),
            name: name.to_string(),
            value,
        });
    }

    fn value(&self, ui: &Ui, ty: Ty, v: &PropValue) -> Option<V> {
        if let Some(cached) = self.cached
            && !matches!(v, PropValue::Lit(_))
        {
            return cached.property(v, ty);
        }
        match v {
            PropValue::Lit(l) => lit(ui, l),
            PropValue::Model(path) => coerce(ui, ty, &self.scope().get(path)),
            PropValue::Bind(e, f) => {
                let x = self.eval(e);
                match (f, ty) {
                    (Some(f), Ty::Text | Ty::Name) => Some(V::Str(crate::bind::format_value(
                        &x,
                        *f,
                        self.env.language(),
                    ))),
                    _ => coerce(ui, ty, &x),
                }
            }
            PropValue::Template(t) => Some(V::Str(t.format(&self.scope()))),
        }
    }
}

impl Ui {
    /// Apply a part's presentation with the same values documents use.
    pub(crate) fn apply_part_appearances(&mut self) {
        if !self
            .dress
            .dressings()
            .iter()
            .any(|d| !d.draw.appearance.is_empty())
        {
            return;
        }
        for i in 0..self.nodes.len() {
            self.apply_part_appearance(i);
        }
    }

    pub(crate) fn apply_part_appearance(&mut self, i: usize) {
        let Some(part) = self.nodes[i]
            .el
            .part
            .or_else(|| self.nodes[i].el.frame.map(crate::StylePart::part_ref))
        else {
            return;
        };
        let appearance = self.dress.resolve(&part).draw.appearance;
        if appearance.is_empty() {
            return;
        }
        let r = Resolved {
            vals: appearance
                .iter()
                .filter_map(|(index, value)| lit(self, value).map(|v| (*index, v)))
                .collect(),
            ..Default::default()
        };
        let mut ramp = self.theme.type_ramp;
        let step = r
            .str("step")
            .and_then(|name| crate::style_file::ramp_step(&mut ramp, name).copied());
        let n = &mut self.nodes[i];
        n.el = Self::box_of(&r, n.el);
        if let Some(x) = r.num("x") {
            n.props.transform.x = x;
        }
        if let Some(y) = r.num("y") {
            n.props.transform.y = y;
        }
        if let Some(opacity) = r.num("opacity") {
            n.props.opacity = opacity.clamp(0.0, 1.0);
        }
        if let Some(V::Transition(t)) = r.get("motion") {
            n.props.motion = Some(*t);
        }
        if let crate::layout::Content::Text(text) = &mut n.content {
            if let Some(color) = r.color("color") {
                text.color = color;
            }
            if let Some(step) = step {
                text.step = step;
            }
        }
    }

    /// Resolves a node's props: its lines, its matching `at` blocks, then
    /// the probe's edits of `path`.
    fn resolve(
        &self,
        b: &Build<'_>,
        n: &DocNode,
        path: &str,
        built: &mut Built,
        inherited: &super::variables::Computed,
        structure: &mut dyn FnMut(&super::selectors::Selector) -> bool,
    ) -> Resolved {
        let mut r = Resolved::default();
        let vp = self.viewport();
        let context = super::media::Context {
            width: vp.width,
            height: vp.height,
            reduced_motion: self.theme().reduced_motion,
        };
        let mut media_reads = super::media::Reads::default();
        let mut sets: Vec<&super::PropSet> = n.props.iter().collect();
        for (q, props) in &n.at {
            if q.media.matches(context, &mut media_reads) {
                sets.extend(props.iter());
            }
        }
        r.cascade = cascade::resolve(&n.rules, inherited, self.theme(), |rule| {
            rule.query.media.matches(context, &mut media_reads)
                && rule.condition.as_ref().is_none_or(|e| truthy(&b.eval(e)))
                && rule.structure.as_ref().is_none_or(|s| structure(s))
        });
        (r.transitions, r.animations) =
            super::motion::resolve(&r.cascade.motion, &b.doc.name, &mut r.cascade.diagnostics);
        sets.retain(|prop| !r.cascade.unset.contains(&prop.spec));
        sets.extend(&r.cascade.props);
        for p in sets {
            let ty = PROPS[p.spec].ty;
            if let Some(v) = b.value(self, ty, &p.value) {
                if !matches!(p.value, PropValue::Lit(_)) {
                    built.bindings.push((
                        p.name().to_string(),
                        p.source.clone(),
                        match &v {
                            V::Num(x) => json!(x),
                            V::Bool(x) => json!(x),
                            V::Str(s) => json!(s),
                            _ => Value::Null,
                        },
                    ));
                }
                r.vals.push((p.spec, v));
            }
        }
        for e in self.docs.edits.iter().filter(|e| e.path == path) {
            if let Some(v) = b.value(self, PROPS[e.spec].ty, &e.value) {
                r.vals.push((e.spec, v));
                built.edits.push((e.prop.clone(), e.source.clone()));
            }
        }
        if let Some(V::Spans(Ok(spans))) = r.get("spans") {
            let rules: Vec<_> = n
                .span_rules
                .iter()
                .filter(|rule| {
                    rule.query.media.matches(context, &mut media_reads)
                        && rule.condition.as_ref().is_none_or(|e| truthy(&b.eval(e)))
                })
                .collect();
            let resolved: Vec<_> = spans
                .iter()
                .map(|span| {
                    let (style, cascade) =
                        super::spans::resolve(&rules, span, &r.cascade.computed, self.theme());
                    (span.clone(), style, cascade)
                })
                .collect();
            for (span, style, cascade) in resolved {
                r.span_styles.push((span, style));
                r.cascade.reads.extend(cascade.reads);
                // Part variables inherit the text element's computed map. Only
                // names not declared on this text depend on its parent map.
                r.cascade.inherited_reads.extend(
                    cascade
                        .inherited_reads
                        .into_iter()
                        .filter(|name| !r.cascade.local_names.contains(name)),
                );
                r.cascade.diagnostics.extend(cascade.diagnostics);
            }
        }
        r.cascade.viewport_reads = media_reads;
        r
    }

    /// The box of an element from its props, over `base`.
    fn box_of(r: &Resolved, mut el: Element) -> Element {
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
        if let Some(g) = r.num("gap") {
            el.gap = g;
        }
        if let Some(a) = r.str("align") {
            el.align = match a {
                "center" => Align::Center,
                "end" => Align::End,
                "spaceBetween" => Align::SpaceBetween,
                _ => Align::Start,
            };
        }
        if let Some(c) = r.str("cross") {
            el.cross = match c {
                "center" => Cross::Center,
                "end" => Cross::End,
                "stretch" => Cross::Stretch,
                "baseline" => Cross::Baseline,
                _ => Cross::Start,
            };
        }
        if r.has("wrap") {
            el.wrap = r.flag("wrap");
        }
        if let Some(c) = r.color("background") {
            el.background = Some(c);
        }
        if let Some(x) = r.num("radius") {
            el.radius = x;
        }
        if let Some(f) = r.str("frame") {
            el.frame = crate::style_file::StylePart::from_name(f);
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

    /// Element fields depend only on resolved values, not a binding scope.
    fn resolved_props(n: &DocNode, r: &Resolved) -> Props {
        let mut p = Props::default();
        let t = &mut p.transform;
        if let Some(v) = r.num("x") {
            t.x = v;
        }
        if let Some(v) = r.num("y") {
            t.y = v;
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
        if let Some(v) = r.num("skewX") {
            t.skew_x = v;
        }
        if let Some(v) = r.num("skewY") {
            t.skew_y = v;
        }
        if let Some([x, y]) = r.nums("pivot").and_then(|n| <[f32; 2]>::try_from(n).ok()) {
            t.pivot_x = x;
            t.pivot_y = y;
        }
        if let Some(v) = r.num("opacity") {
            p.opacity = v.clamp(0.0, 1.0);
        }
        if let Some(v) = r.num("z") {
            p.z = v as i32;
        }
        p.position = match r.str("position") {
            Some("absolute") => Position::Absolute,
            Some("screen") => Position::Screen,
            Some("anchored") => Position::Anchored,
            _ => Position::Flow,
        };
        p.insets = Edges {
            start: r.num("start"),
            top: r.num("top"),
            end: r.num("end"),
            bottom: r.num("bottom"),
            center_x: r.flag("centerX"),
            center_y: r.flag("centerY"),
        };
        if p.position == Position::Flow && p.insets.is_set() {
            p.position = Position::Absolute;
        }
        if let Some(a) = r.str("anchorTo") {
            p.anchor_to = a.to_string();
            if p.position == Position::Flow {
                p.position = Position::Anchored;
            }
        }
        if let Some(a) = r.str("anchorAt") {
            p.anchor_at = anchor_of(a);
        }
        if let Some(a) = r.str("anchorSelf") {
            p.anchor_self = anchor_of(a);
        }
        if let Some(v) = r.num("hitSlop") {
            p.hit_slop = v;
        }
        match n.kind {
            NodeKind::Element("stack") => p.layout = LayoutMode::Stack,
            NodeKind::Element("grid") => {
                p.layout = LayoutMode::Grid;
                let tracks = |name: &str| match r.get(name) {
                    Some(V::Tracks(t)) => t.clone(),
                    _ => Vec::new(),
                };
                p.grid = GridSpec {
                    columns: tracks("columns"),
                    rows: tracks("rows"),
                    areas: match r.get("areas") {
                        Some(V::Strs(s)) => s.clone(),
                        _ => Vec::new(),
                    },
                    flow: if r.str("flow") == Some("column") {
                        GridFlow::Column
                    } else {
                        GridFlow::Row
                    },
                    dense: r.flag("dense"),
                    ..GridSpec::default()
                };
            }
            _ => {}
        }
        let pair = |name: &str| r.nums(name).and_then(|n| <[f32; 2]>::try_from(n).ok());
        p.place = GridPlace {
            column: pair("cell").map_or(0, |c| c[0] as u16),
            row: pair("cell").map_or(0, |c| c[1] as u16),
            column_span: pair("span").map_or(0, |c| c[0] as u16),
            row_span: pair("span").map_or(0, |c| c[1] as u16),
            area: r.str("area").unwrap_or_default().to_string(),
        };
        if let Some(V::Strs(o)) = r.get("overlaps") {
            p.overlaps = o.clone();
        }
        if let Some(m) = r.str("match") {
            p.match_key = m.to_string();
        }
        if let Some(V::Transition(t)) = r.get("motion") {
            p.motion = Some(*t);
        }
        p.transitions = r.transitions.clone();
        p.animations = r.animations.clone();
        if let Some(V::Transition(t)) = r.get("layoutMotion") {
            p.layout_motion = Some(*t);
        }
        if let Some(V::Presence(e)) = r.get("enter") {
            p.enter = Some(e.clone());
        }
        if let Some(V::Presence(e)) = r.get("exit") {
            p.exit = Some(e.clone());
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
        if let Some(c) = r.str("cursor").and_then(crate::element::Cursor::from_name) {
            p.cursor = c;
        }
        p.customize = r.flag("customize").then(|| crate::element::Customize {
            key: r.str("customizeKey").unwrap_or_default().to_owned(),
            ..crate::element::Customize::default()
        });
        p.reduced = match r.str("reduced") {
            Some("fade") => Some(crate::Reduced::Fade),
            Some("none") => Some(crate::Reduced::None),
            Some("keep") => Some(crate::Reduced::Keep),
            _ => None,
        };
        p
    }

    /// The element-model props of an element (§6.1) and its state groups.
    fn props_of(
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
                None => self.docs.states.get(&(key, g.name.clone())).cloned(),
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

/// A document's timeline by its full name (`shop/open`).
fn qualify(doc: &str, tl: &str) -> String {
    if tl.is_empty() || tl.contains('/') {
        tl.to_string()
    } else {
        format!("{doc}/{tl}")
    }
}

/// An art slot's spec as JSON (the probe's `slot`, the art brief's input).
pub fn slot_json(s: &crate::slot::SlotSpec) -> Value {
    use crate::slot::{Mask, SlotAspect, Treatment};
    let i = s.kind.info();
    // Four places, so 0.2 reads 0.2 and not its f32's 0.20000000298.
    let short = |v: &[f32]| -> Vec<f64> {
        v.iter()
            .map(|x| (f64::from(*x) * 1e4).round() / 1e4)
            .collect()
    };
    let aspect = |a: &crate::slot::Aspect| json!([a.w, a.h]);
    json!({
        "name": s.name,
        "kind": s.kind.name(),
        "aspect": match &s.aspect {
            SlotAspect::One(a) => aspect(a),
            SlotAspect::AnyOf(v) => Value::Array(v.iter().map(aspect).collect()),
            SlotAspect::Range(a, b) => json!({ "min": aspect(a), "max": aspect(b) }),
            SlotAspect::Free => json!("free"),
        },
        "focal": short(&s.focal),
        "safe": s.safe.map(|r| short(&r)),
        "fit": match s.fit {
            crate::slot::Fit::Cover => "cover",
            crate::slot::Fit::Contain => "contain",
            crate::slot::Fit::Fill => "fill",
            crate::slot::Fit::NineSlice => "nineSlice",
            crate::slot::Fit::Tile => "tile",
        },
        "transparent": i.transparent,
        "mask": match s.mask {
            Mask::None => json!("none"),
            Mask::Round(r) => json!({ "round": r }),
            Mask::Circle => json!("circle"),
            Mask::Squircle => json!("squircle"),
            Mask::Hexagon => json!("hexagon"),
            Mask::Shape(c) => json!({ "shape": c.name() }),
            Mask::Alpha => json!("alpha"),
        },
        "treatments": s.treatments.iter().map(|t| match t {
            Treatment::Vignette => "vignette",
            Treatment::Duotone => "duotone",
            Treatment::Frame => "frame",
            Treatment::RarityGlow => "rarityGlow",
        }).collect::<Vec<_>>(),
        "tiling": i.tiling || s.fit == crate::slot::Fit::Tile,
        "insets": s.insets.map(|r| short(&r)),
        "cells": s.cells,
        "states": s.states,
        "legiblePx": i.legible_px,
        "generated": i.generated,
        "set": s.set,
    })
}

pub mod cascade;
mod fields;
mod mount;
mod nodes;
mod retained;
mod structure;
pub use mount::{Built, DocEvent, DocHost, DocResponse, Edit};
