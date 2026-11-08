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
    /// The layout key this element publishes as a query container.
    container: Option<u64>,
    /// An immediate build's inputs under a query container (settlement replay).
    replay: Option<Arc<containers::Replay>>,
    /// A numbered `<li>`'s number ([`super::lists::ordinal`]).
    ordinal: Option<i64>,
    /// Its effective CSS text and font style ([`super::text_css`]); `None`
    /// for an element that neither declares nor shows text.
    text: Option<Arc<super::text_css::TextCss>>,
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
    p.resolve(ui.theme())
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
            } else if s.contains('(') || crate::css_color::is_named(s) {
                Paint::Color(
                    crate::css_color::parse_str(s)
                        .ok()?
                        .resolve(&crate::css_color::Context::of(ui.theme())),
                )
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

/// A grid's tracks, areas and auto-placement.
/// A `justify-content`/`align-content` word as laid out.
fn align_of(a: &str) -> Align {
    match a {
        "center" => Align::Center,
        "end" | "flexEnd" => Align::End,
        "spaceBetween" => Align::SpaceBetween,
        "spaceAround" => Align::SpaceAround,
        "spaceEvenly" => Align::SpaceEvenly,
        "stretch" => Align::Stretch,
        _ => Align::Start,
    }
}

/// An `align-items`/`-self`/`justify-items`/`-self` word as laid out.
fn cross_of(c: &str) -> Cross {
    match c {
        "center" => Cross::Center,
        "end" => Cross::End,
        "stretch" => Cross::Stretch,
        "baseline" => Cross::Baseline,
        "lastBaseline" => Cross::LastBaseline,
        _ => Cross::Start,
    }
}

fn grid_of(r: &Resolved) -> GridSpec {
    let tracks = |name: &str| match r.get(name) {
        Some(V::Tracks(t)) => t.clone(),
        _ => Vec::new(),
    };
    GridSpec {
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
        column_gap: r.num("columnGap"),
        row_gap: r.num("rowGap"),
        ..GridSpec::default()
    }
}

/// The link id of inline link `k` ([`super::InlineRun::link`]): `link` for
/// an `<a>` element's own `href`, `link<k>` for its text's `<a>`s.
fn inline_link(k: usize) -> String {
    if k == usize::MAX {
        "link".into()
    } else {
        format!("link{k}")
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
    /// While resolving an element under a query container, what `eval` and
    /// `value` computed, so settlement can resolve it again without a model.
    recording: std::cell::RefCell<Option<retained::CachedInputs>>,
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
        let value = e.eval(&self.scope());
        if let Some(recording) = self.recording.borrow_mut().as_mut() {
            recording.remember_expression(e, &value);
        }
        value
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
        let value = self.evaluate(ui, ty, v);
        if !matches!(v, PropValue::Lit(_))
            && let Some(recording) = self.recording.borrow_mut().as_mut()
        {
            recording.remember_property(v, ty, &value);
        }
        value
    }

    fn evaluate(&self, ui: &Ui, ty: Ty, v: &PropValue) -> Option<V> {
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
        for k in 0..self.emitted.len() {
            self.apply_part_appearance(self.emitted[k]);
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
        let em = step.map_or(self.theme.type_ramp.body.size, |s| s.size);
        let n = &mut self.nodes[i];
        n.el = Self::box_of(&r, n.el, em);
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
            color_vision: self.theme().color_vision,
            coarse_pointer: vp.coarse_pointer,
        };
        let mut media_reads = super::media::Reads::default();
        let chain = super::container::chain(inherited);
        // Under a container, an immediate build records every input this
        // element can read, including branches the current sizes skip.
        let recording = b.cached.is_none() && !chain.is_empty();
        let mut decisions = Vec::new();
        if recording {
            *b.recording.borrow_mut() = Some(Default::default());
            for (_, props) in &n.at {
                for p in props {
                    b.value(self, PROPS[p.spec].ty, &p.value);
                }
            }
            let conditions = n.rules.iter().map(|r| &r.condition);
            for e in conditions
                .chain(n.span_rules.iter().map(|r| &r.condition))
                .flatten()
            {
                b.eval(e);
            }
            for s in n.rules.iter().filter_map(|rule| rule.structure.as_ref()) {
                decisions.push((Arc::as_ptr(s) as usize, structure(s)));
            }
        }
        let structure = &mut |s: &super::selectors::Selector| {
            decisions
                .iter()
                .find(|(at, _)| *at == s as *const _ as usize)
                .map_or_else(|| structure(s), |&(_, matched)| matched)
        };
        let sizes = &self.docs.container_sizes;
        let mut container_reads = BTreeSet::new();
        let mut applies = |q: &super::AtQuery, reads: &mut super::media::Reads| {
            q.media.matches(context, reads)
                && q.container
                    .iter()
                    .all(|c| c.matches(&chain, sizes, &mut container_reads))
        };
        let mut sets: Vec<&super::PropSet> = n.props.iter().collect();
        for (q, props) in &n.at {
            if applies(q, &mut media_reads) {
                sets.extend(props.iter());
            }
        }
        // Bound custom properties: a value a rule reads with var(); null
        // (or a list) leaves the property unset, so var()'s fallback holds.
        // Their expressions are the element's attributes too, so a change
        // re-resolves the element as an attribute selector's does.
        let inline: Vec<(String, String)> = n
            .custom
            .iter()
            .filter_map(|(name, e)| {
                let text = match b.eval(e) {
                    Value::String(s) => s,
                    Value::Number(x) => x.to_string(),
                    Value::Bool(x) => x.to_string(),
                    _ => return None,
                };
                Some((name.clone(), text))
            })
            .collect();
        let body = self.theme().type_ramp.body;
        let face = &self.fonts().body.metrics;
        let ratio = |s: &str| face.measure(s, 100.0) / 100.0;
        let [small, large] = vp.extremes();
        let basis = super::calc::Basis {
            width: vp.width,
            height: vp.height,
            small,
            large,
            rem: body.size,
            rlh: body.line_height,
            ch: ratio("0"),
            cap: face.ascender,
            ic: ratio("\u{6C34}"),
            safe: [
                vp.safe_area.top,
                vp.safe_area.right,
                vp.safe_area.bottom,
                vp.safe_area.left,
            ],
            ..Default::default()
        };
        // An attribute attr() reads, as its text.
        let attr = |name: &str| {
            let (_, e) = n.selector_attrs.iter().find(|(a, _)| a == name)?;
            match b.eval(e) {
                Value::String(s) => Some(s),
                Value::Number(x) => Some(x.to_string()),
                Value::Bool(x) => Some(x.to_string()),
                _ => None,
            }
        };
        r.cascade = cascade::resolve(
            &n.rules,
            &inline,
            inherited,
            self.theme(),
            basis,
            &attr,
            |rule| {
                applies(&rule.query, &mut media_reads)
                    && rule.condition.as_ref().is_none_or(|e| truthy(&b.eval(e)))
                    && rule.structure.as_ref().is_none_or(|s| structure(s))
            },
        );
        if let Some((kind, names)) =
            super::container::declared(&r.cascade.container, &mut r.cascade.diagnostics)
        {
            let key = crate::id::key_of_path(path);
            super::container::push(Arc::make_mut(&mut r.cascade.computed), key, kind, &names);
            r.container = Some(key);
        }
        (r.transitions, r.animations) =
            super::motion::resolve(&r.cascade.motion, &b.doc.name, &mut r.cascade.diagnostics);
        // A conditional `@keyframes` defines its name while its conditions
        // hold; choosing another definition restarts the animation, as a
        // name change does.
        if b.doc
            .keyframe_rules
            .iter()
            .any(|(_, q, _)| *q != super::AtQuery::default())
        {
            let prefix = format!("{}/", b.doc.name);
            for a in &mut r.animations {
                let Some(name) = a.name.strip_prefix(&prefix) else {
                    continue;
                };
                let key = b
                    .doc
                    .keyframe_rules
                    .iter()
                    .rev()
                    .filter(|(n, ..)| n == name)
                    .find(|(_, q, _)| applies(q, &mut media_reads))
                    .map_or_else(|| format!("{name}@none"), |(.., key)| key.clone());
                a.name = format!("{prefix}{key}");
            }
        }
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
                    applies(&rule.query, &mut media_reads)
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
        // A translated string's MessageFormat 2 markup (`{#b}…{/b}`) styles
        // its runs as inline elements would.
        if n.inline.is_empty()
            && let Some(t) = r
                .str("key")
                .and_then(|k| b.env.translate(k))
                .filter(|t| super::mf2::has_markup(t))
        {
            let mut spans = Vec::new();
            let mut position = 0u32;
            for run in super::mf2::runs(&t) {
                let length = run.text.chars().count() as u32;
                let span = crate::TextSpan {
                    name: "inline".into(),
                    position,
                    length,
                    properties: Default::default(),
                };
                position += length;
                let on = |b: bool| b.then_some(true);
                r.span_styles.push((
                    span.clone(),
                    crate::TextSpanStyle {
                        bold: on(run.bold),
                        italic: on(run.italic),
                        underline: on(run.underline),
                        strike: on(run.strike),
                        ..Default::default()
                    },
                ));
                spans.push(span);
            }
            if let Some((spec, _)) = schema::prop("spans") {
                r.vals.push((spec, V::Spans(Ok(spans))));
            }
        }
        if !n.inline.is_empty() {
            // A text element's inline markup: one span per run, styled by
            // its elements (a run of an `<a href>` is a link).
            let mut spans = Vec::new();
            let mut position = 0u32;
            // A translated `<a i18n>` is one run: the string's text.
            let translated = r
                .str("key")
                .and_then(|k| b.env.translate(k))
                .filter(|_| n.inline.len() == 1);
            for run in &n.inline {
                let text = match &run.text {
                    _ if translated.is_some() => translated.clone().unwrap_or_default(),
                    crate::bind::Expr::Lit(v) => crate::bind::text(v),
                    e => crate::bind::text(&b.eval(e)),
                };
                let length = text.chars().count() as u32;
                let span = crate::TextSpan {
                    name: run.part.clone().unwrap_or_else(|| "inline".into()),
                    position,
                    length,
                    properties: Default::default(),
                };
                position += length;
                // A named span's `::part(name)` rules, under its elements'.
                let mut style = if run.part.is_some() {
                    let rules: Vec<_> = n
                        .span_rules
                        .iter()
                        .filter(|rule| {
                            applies(&rule.query, &mut media_reads)
                                && rule.condition.as_ref().is_none_or(|e| truthy(&b.eval(e)))
                        })
                        .collect();
                    let (style, cascade) =
                        super::spans::resolve(&rules, &span, &r.cascade.computed, self.theme());
                    r.cascade.reads.extend(cascade.reads);
                    style
                } else {
                    crate::TextSpanStyle::default()
                };
                let on = |b: bool, was: Option<bool>| if b { Some(true) } else { was };
                style.bold = on(run.bold, style.bold);
                style.italic = on(run.italic, style.italic);
                style.underline = on(run.underline, style.underline);
                style.strike = on(run.strike, style.strike);
                if let Some(c) = run.color.as_deref().and_then(|c| {
                    let c = crate::svg::color(c).ok()?;
                    Some(crate::paint::ColorRef::Literal(svg::srgba(c, 1.0)))
                }) {
                    style.color = Some(c);
                }
                style.link = run.link.map(inline_link).or(style.link);
                style.atom = run.atom.clone();
                r.span_styles.push((span.clone(), style));
                spans.push(span);
            }
            if let Some((spec, _)) = schema::prop("spans") {
                r.vals.push((spec, V::Spans(Ok(spans))));
            }
        }
        r.cascade.viewport_reads.extend(media_reads);
        if !container_reads.is_empty() {
            // The chain is inherited state: a container appearing or leaving
            // above this element re-resolves it like a custom property would.
            r.cascade
                .inherited_reads
                .insert(super::container::CHAIN.to_string());
        }
        r.cascade.container_reads = container_reads;
        if let NodeKind::Element(kind) = n.kind
            && svg::KINDS.contains(&kind)
        {
            svg::inherit(kind, &mut r);
        }
        // Text style: written for those under it by an element declaring
        // it, read by an element that shows text.
        // Controls read accent-color and caret-color the same way.
        let shows_text = matches!(
            n.kind,
            NodeKind::Element(
                "label"
                    | "button"
                    | "field"
                    | "toggle"
                    | "slider"
                    | "radio"
                    | "progress"
                    | "meter"
                    | "select"
                    | "color"
            )
        );
        if shows_text || !r.cascade.text.is_empty() {
            let mut ramp = self.theme.type_ramp;
            let step = r
                .str("step")
                .and_then(|s| crate::style_file::ramp_step(&mut ramp, s).copied())
                .unwrap_or(self.theme.type_ramp.body);
            let current = r.color("color").unwrap_or(self.theme.palette.ink);
            let local = std::mem::take(&mut r.cascade.text);
            let cascade = &mut r.cascade;
            let text = super::text_css::inherit(super::text_css::Inputs {
                local: &local,
                computed: Arc::make_mut(&mut cascade.computed),
                reads: &mut cascade.inherited_reads,
                diagnostics: &mut cascade.diagnostics,
                theme: &self.theme,
                current,
                step,
            });
            cascade.text = local;
            r.text = (text != Default::default()).then(|| Arc::new(text));
        }
        if recording {
            let cached = b.recording.borrow_mut().take().unwrap_or_default();
            r.replay = Some(Arc::new(containers::Replay { cached, decisions }));
        }
        r
    }

    /// The size of an em for `r`'s element: its type step's, else the body's.
    fn em_of(&self, r: &Resolved) -> f32 {
        let mut ramp = self.theme.type_ramp;
        let body = ramp.body.size;
        if let Some(s) = r.text.as_ref().and_then(|t| t.size) {
            return s;
        }
        r.str("step")
            .and_then(|name| crate::style_file::ramp_step(&mut ramp, name))
            .map_or(body, |s| s.size)
    }

    /// The box of an element from its props, over `base`; `em` sizes its
    /// em margins.
    fn box_of(r: &Resolved, mut el: Element, em: f32) -> Element {
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
        el.wrap_reverse = r.flag("wrapReverse");
        el.align_content = r.str("alignContent").map(align_of);
        el.justify_items = r.str("justifyItems").map(cross_of);
        el.gaps = [r.num("columnGap"), r.num("rowGap")];
        el.safe = u8::from(r.flag("safeJustify")) | u8::from(r.flag("safeAlign")) << 1;
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

    /// `el` with its CSS flex and grid item properties (any element can be
    /// an item): its flex factors, `order`, and own alignments.
    fn item_of(r: &Resolved, mut el: Element) -> Element {
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
    fn sized(r: &Resolved, mut el: Element) -> Element {
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
            across: None,
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
        // A `<div>` with `display: grid` lays out as a `<grid>` does.
        let grid_box = n.kind == NodeKind::Element("box") && r.str("display") == Some("grid");
        match n.kind {
            NodeKind::Element("stack") => p.layout = LayoutMode::Stack,
            NodeKind::Element("grid") => {
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
            column_span: pair("span").map_or(0, |c| c[0] as u16),
            row_span: pair("span").map_or(0, |c| c[1] as u16),
            area: r.str("area").unwrap_or_default().to_string(),
        };
        if let Some(V::Strs(o)) = r.get("overlaps") {
            p.overlaps = o.clone();
        }
        gestures::props(n, r, &mut p);
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
mod color;
mod containers;
mod fields;
pub(crate) mod gestures;
mod kbd;
mod mount;
mod nodes;
mod popover;
mod retained;
mod structure;
pub(crate) mod svg;
mod text_motion;
mod touch;
mod viewport;
mod world;
pub use mount::{Built, DocEvent, DocHost, DocResponse, Edit};
