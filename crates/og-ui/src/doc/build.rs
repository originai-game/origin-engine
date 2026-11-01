//! Mounting a document (docs/ui.md §10.2): its tree is built into the
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

/// Something a document did that code may want: a click, a changed
/// control, a named event (`emit`, a timeline's callback), an action, a
/// two-way write (`set`), a Back the layer turned into an event.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DocEvent {
    /// The document's mount: the screen's name, or a component instance's path.
    pub mount: String,
    /// The element's path.
    pub path: String,
    /// `click`, `changed`, `select`, `emit`, `action`, `set`, `back`.
    pub kind: String,
    /// The event's or the action's name, or the path a `set` wrote.
    pub name: String,
    pub value: Value,
}

impl DocEvent {
    pub fn to_json(&self) -> Value {
        json!({ "mount": self.mount, "path": self.path, "kind": self.kind, "name": self.name, "value": self.value })
    }
}

/// What a mount answers: its events this tick, and whether its viewmodel changed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DocResponse {
    pub events: Vec<DocEvent>,
    /// Its root's path (empty: it was not built).
    pub path: String,
}

/// A probe edit of a prop (`ui:set`): presentation only, marked in the dump.
#[derive(Clone, Debug, PartialEq)]
pub struct Edit {
    pub path: String,
    pub prop: String,
    pub spec: usize,
    pub value: PropValue,
    /// The value as given.
    pub source: String,
}

/// One element a document built this tick, for the probe.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Built {
    pub path: String,
    /// The document's name and the node's line.
    pub doc: String,
    pub node: usize,
    pub line: usize,
    /// The mount it belongs to (a screen's name is its layer).
    pub mount: String,
    /// `(prop, expression as written, value now)`.
    pub bindings: Vec<(String, String, Value)>,
    /// Its probe edits: `(prop, value as given)`.
    pub edits: Vec<(String, String)>,
    /// The state groups it declares, with the state each is in by its binding.
    pub states: Vec<(String, String)>,
}

/// The documents a UI knows and what their mounts keep between ticks.
#[derive(Clone, Debug, Default)]
pub struct DocHost {
    /// By name.
    pub(crate) docs: BTreeMap<String, Arc<UiDocument>>,
    pub(crate) edits: Vec<Edit>,
    /// A trigger's last value (`on change`), by element key and trigger.
    pub(crate) last: BTreeMap<(u64, usize), Value>,
    /// The states triggers set (`on click state selection on`), by element key and group.
    pub(crate) states: BTreeMap<(u64, String), String>,
    /// Document elements seen last tick (for `on appear`).
    pub(crate) seen: BTreeSet<u64>,
    pub(crate) seen_now: BTreeSet<u64>,
    /// This tick's builds, slots and art slots.
    pub(crate) built: Vec<Built>,
    pub(crate) slots: BTreeMap<String, usize>,
    pub(crate) arts: Vec<(String, crate::slot::SlotSpec)>,
    /// Events queued for code (screens built at the end of a tick reach
    /// code the next).
    pub(crate) queued: Vec<DocEvent>,
    /// Layer operations triggers asked for this tick, done after the build.
    pub(crate) ops: Vec<crate::layers::LayerOp>,
    /// Plays a document or the probe started: `(path, timeline)` → play.
    pub(crate) plays: BTreeMap<(String, String), u32>,
    /// The tick the layers were last built.
    pub(crate) built_tick: u64,
    pub(crate) layers: crate::layers::LayerStack,
    /// The mount being built is the top layer that takes input (its `on action` triggers fire).
    pub(crate) accepts: bool,
}

/// A resolved value of a prop.
#[derive(Clone, Debug, PartialEq)]
enum V {
    Num(f32),
    Nums(Vec<f32>),
    Bool(bool),
    Str(String),
    Size(Size),
    Color(Srgba),
    Transition(crate::motion::Transition),
    Presence(crate::motion::PresenceSpec),
    Tracks(Vec<Track>),
    Strs(Vec<String>),
}

/// An element's resolved props, by schema index.
#[derive(Default)]
struct Resolved {
    vals: Vec<(usize, V)>,
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
            V::Num(n) => Some(*n),
            V::Bool(b) => Some(f32::from(u8::from(*b))),
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
            V::Num(n) => Some(Size::Fixed(*n)),
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
        Paint::Token(t) => ui.theme().palette.get(t),
        Paint::Color(c) => Some(*c),
    }
}

/// A bound JSON value as a prop of type `ty`.
fn coerce(ui: &Ui, ty: Ty, v: &Value) -> Option<V> {
    Some(match ty {
        Ty::Num => V::Num(number(v)? as f32),
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
        Ty::Transition | Ty::Presence | Ty::Tracks => return None,
    })
}

fn lit(ui: &Ui, l: &Lit) -> Option<V> {
    Some(match l {
        Lit::Num(n) => V::Num(*n),
        Lit::Nums(n) => V::Nums(n.clone()),
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
    events: Vec<DocEvent>,
    writes: Vec<(String, Value)>,
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
        e.eval(&self.scope())
    }

    /// Where a write to `path` lands in the viewmodel: through an item
    /// variable to its list's entry.
    fn write_path(&self, path: &str) -> Option<String> {
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
            kind: kind.to_string(),
            name: name.to_string(),
            value,
        });
    }

    fn value(&self, ui: &Ui, ty: Ty, v: &PropValue) -> Option<V> {
        match v {
            PropValue::Lit(l) => lit(ui, l),
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
    /// Registers a document (its timelines as `<name>/<timeline>`); one of
    /// the same name is replaced, the UI's state kept (a hot reload).
    pub fn define_document(&mut self, doc: UiDocument) {
        for (n, t) in &doc.timelines {
            self.define_timeline(&format!("{}/{n}", doc.name), t.clone());
        }
        self.docs.docs.insert(doc.name.clone(), Arc::new(doc));
    }

    /// A document by name.
    pub fn document(&self, name: &str) -> Option<&UiDocument> {
        self.docs.docs.get(name).map(|d| &**d)
    }

    /// Every document's name.
    pub fn documents(&self) -> Vec<String> {
        self.docs.docs.keys().cloned().collect()
    }

    /// Mounts a component here: `props` is its viewmodel (writes of its
    /// two-way bindings land in it), `id` its root's id.
    pub fn component(
        &mut self,
        id: &str,
        name: &str,
        props: &mut Value,
        env: &dyn Env,
    ) -> DocResponse {
        let Some(doc) = self.docs.docs.get(name).cloned() else {
            self.warnings.push(format!(
                "og.ui.component: no document named '{name}' (load its .ogt with og.load.document)"
            ));
            return DocResponse::default();
        };
        let mount = format!("{}/{id}", self.path_of(self.parent()));
        let game = self.game_input();
        let accepts = std::mem::replace(&mut self.docs.accepts, game);
        let r = self.mount_doc(doc, mount, id, props, env);
        self.docs.accepts = accepts;
        r
    }

    /// Builds `doc` here with root id `id`.
    pub(crate) fn mount_doc(
        &mut self,
        doc: Arc<UiDocument>,
        mount: String,
        id: &str,
        vm: &mut Value,
        env: &dyn Env,
    ) -> DocResponse {
        let root = doc.root;
        let mut b = Build {
            doc: doc.clone(),
            mount: mount.clone(),
            vm,
            env,
            locals: Vec::new(),
            events: Vec::new(),
            writes: Vec::new(),
        };
        let path = self.build_node(&mut b, root, Some(id));
        let (events, writes) = (b.events, b.writes);
        for (p, v) in writes {
            set_path(vm, &p, v);
        }
        DocResponse {
            events,
            path: path.unwrap_or_default(),
        }
    }

    /// Opens a document's slot so code fills it (its children follow until
    /// [`Ui::close`]): a full path, or `<mount>/<slot id>`. False when no
    /// such slot was built this tick.
    pub fn open_slot(&mut self, path: &str) -> bool {
        match self.docs.slots.get(path) {
            Some(&i) if i < self.nodes.len() => {
                self.stack.push(i);
                true
            }
            _ => {
                self.warnings.push(format!(
                    "og.ui.openSlot: no slot '{path}' this tick (mount the document first; slots: {})",
                    self.docs.slots.keys().cloned().collect::<Vec<_>>().join(", ")
                ));
                false
            }
        }
    }

    /// Events of documents built where code did not see them (screens
    /// built at the end of a tick), taken.
    pub fn take_doc_events(&mut self) -> Vec<DocEvent> {
        std::mem::take(&mut self.docs.queued)
    }

    /// What documents built this tick (the probe's `dump_ui`).
    pub fn doc_built(&self) -> &[Built] {
        &self.docs.built
    }

    /// The art slots documents declared this tick, by path.
    pub fn art_slots(&self) -> &[(String, crate::slot::SlotSpec)] {
        &self.docs.arts
    }

    /// The probe's edits in effect.
    pub fn doc_edits(&self) -> &[Edit] {
        &self.docs.edits
    }

    /// `ui:set`: sets a prop of the element at `path` (presentation only:
    /// it applies over the document's own line until cleared or saved).
    pub fn set_doc_prop(&mut self, path: &str, prop: &str, value: &str) -> Result<(), String> {
        let (spec, v) = parse_probe_value(prop, value)?;
        self.docs
            .edits
            .retain(|e| !(e.path == path && e.prop == prop));
        self.docs.edits.push(Edit {
            path: path.to_string(),
            prop: prop.to_string(),
            spec,
            value: v,
            source: value.to_string(),
        });
        Ok(())
    }

    /// Drops the probe's edits of `path` (all of them: empty).
    pub fn clear_doc_edits(&mut self, path: &str) {
        self.docs
            .edits
            .retain(|e| !path.is_empty() && e.path != path);
    }

    /// The document element built at `path` last tick: its document's name
    /// and node.
    pub fn doc_element(&self, path: &str) -> Option<(&str, usize)> {
        self.docs
            .built
            .iter()
            .find(|b| b.path == path)
            .map(|b| (b.doc.as_str(), b.node))
    }

    /// Writes the probe's edits into their documents' text (`ui:save`):
    /// each edited element's line of that prop is replaced, or one added.
    /// Answers the documents changed, each with its source and new text;
    /// the edits written are dropped (the documents now say it).
    pub fn save_doc_edits(&mut self) -> Result<Vec<(String, String, String)>, String> {
        let edits = self.docs.edits.clone();
        let mut changed: BTreeMap<String, UiDocument> = BTreeMap::new();
        let mut kept = Vec::new();
        for e in edits {
            let Some((doc, node)) = self.doc_element(&e.path).map(|(d, n)| (d.to_string(), n))
            else {
                kept.push(e);
                continue;
            };
            let d = match changed.get_mut(&doc) {
                Some(d) => d,
                None => {
                    let d = (**self.docs.docs.get(&doc).ok_or("no document")?).clone();
                    changed.entry(doc.clone()).or_insert(d)
                }
            };
            d.set_prop(node, &e.prop, &e.source)
                .map_err(|why| format!("{}: {}.{}: {why}", d.source, e.path, e.prop))?;
        }
        self.docs.edits = kept;
        let mut out = Vec::new();
        for (name, d) in changed {
            out.push((name, d.source.clone(), d.text.clone()));
            self.define_document(d);
        }
        Ok(out)
    }

    /// Plays a document timeline on the element at `path` (`ui:play`): a
    /// name of the element's document, or `<doc>/<name>`.
    pub fn play_doc_timeline(
        &mut self,
        path: &str,
        timeline: &str,
        from_ms: f32,
        paused: bool,
    ) -> Result<u32, String> {
        let full = self.timeline_name(path, timeline)?;
        let id = self.play_timeline(
            path,
            &full,
            crate::motion::PlayDesc {
                from_ms,
                hold: true,
                ..Default::default()
            },
        );
        if paused {
            self.pause(id);
        }
        self.docs
            .plays
            .insert((path.to_string(), timeline.to_string()), id);
        Ok(id)
    }

    /// `ui:seek`: a timeline playing on the element to `ms` (started there,
    /// paused, when it is not playing).
    pub fn seek_doc_timeline(
        &mut self,
        path: &str,
        timeline: &str,
        ms: f32,
    ) -> Result<u32, String> {
        if let Some(&id) = self
            .docs
            .plays
            .get(&(path.to_string(), timeline.to_string()))
            && self.seek(id, ms)
        {
            self.pause(id);
            return Ok(id);
        }
        self.play_doc_timeline(path, timeline, ms, true)
    }

    fn timeline_name(&self, path: &str, timeline: &str) -> Result<String, String> {
        if timeline.contains('/') {
            return if self.timeline(timeline).is_some() {
                Ok(timeline.to_string())
            } else {
                Err(format!("no timeline '{timeline}'"))
            };
        }
        let doc = self
            .doc_element(path)
            .map(|(d, _)| d.to_string())
            .or_else(|| {
                // A path below a mount: the nearest built ancestor's document.
                self.docs
                    .built
                    .iter()
                    .filter(|b| path.starts_with(&format!("{}/", b.path)))
                    .max_by_key(|b| b.path.len())
                    .map(|b| b.doc.clone())
            });
        match doc {
            Some(d) if self.timeline(&format!("{d}/{timeline}")).is_some() => {
                Ok(format!("{d}/{timeline}"))
            }
            _ if self.timeline(timeline).is_some() => Ok(timeline.to_string()),
            Some(d) => Err(format!(
                "the document '{d}' declares no timeline '{timeline}' ({})",
                self.document(&d)
                    .map(|x| x
                        .timelines
                        .iter()
                        .map(|t| t.0.clone())
                        .collect::<Vec<_>>()
                        .join(", "))
                    .unwrap_or_default()
            )),
            None => Err(format!(
                "no document element at '{path}' and no timeline '{timeline}'"
            )),
        }
    }

    /// The tick's document bookkeeping starts (from `begin`).
    pub(crate) fn docs_begin(&mut self) {
        let d = &mut self.docs;
        d.seen = std::mem::take(&mut d.seen_now);
        d.built.clear();
        d.slots.clear();
        d.arts.clear();
    }

    /// Resolves a node's props: its lines, its matching `at` blocks, then
    /// the probe's edits of `path`.
    fn resolve(&self, b: &Build<'_>, n: &DocNode, path: &str, built: &mut Built) -> Resolved {
        let mut r = Resolved::default();
        let bp = self.breakpoint();
        let vp = self.viewport();
        let orientation = if vp.width >= vp.height {
            crate::element::Orientation::Landscape
        } else {
            crate::element::Orientation::Portrait
        };
        let mut sets: Vec<&super::PropSet> = n.props.iter().collect();
        for (q, props) in &n.at {
            let bp_ok = q.breakpoints.is_empty() || q.breakpoints.contains(&bp);
            let or_ok = q.orientation.is_none_or(|o| o == orientation);
            if bp_ok && or_ok {
                sets.extend(props.iter());
            }
        }
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
        if r.has("interactive") {
            el.interactive = r.flag("interactive");
        }
        el
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
        p.reduced = match r.str("reduced") {
            Some("fade") => Some(crate::Reduced::Fade),
            Some("none") => Some(crate::Reduced::None),
            Some("keep") => Some(crate::Reduced::Keep),
            _ => None,
        };
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

    /// Builds node `i` (and its subtree); `root_id` names a mount's root.
    /// Answers its path (none when it is not shown).
    fn build_node(&mut self, b: &mut Build<'_>, i: usize, root_id: Option<&str>) -> Option<String> {
        let doc = b.doc.clone();
        let n = &doc.nodes[i];
        match &n.kind {
            NodeKind::If(e) => {
                if truthy(&b.eval(e)) {
                    for &c in &n.children {
                        self.build_node(b, c, None);
                    }
                }
                return None;
            }
            NodeKind::For { var, list } => {
                let items = b.eval(list);
                let origin = match list {
                    crate::bind::Expr::Path(p) => b.write_path(p),
                    _ => None,
                };
                let items = items.as_array().cloned().unwrap_or_default();
                for (k, item) in items.into_iter().enumerate() {
                    self.build_item(b, n, var, item, k, origin.as_deref());
                }
                return None;
            }
            NodeKind::Use(comp) => {
                let Some(cdoc) = self.docs.docs.get(comp).cloned() else {
                    self.warnings.push(format!(
                        "{}:{}: no component '{comp}' (load its document)",
                        doc.source, n.line
                    ));
                    return None;
                };
                let mut props = serde_json::Map::new();
                let mut links = Vec::new();
                for (k, v, _) in &n.args {
                    let x = match v {
                        PropValue::Lit(l) => match l {
                            Lit::Num(x) => json!(x),
                            Lit::Bool(x) => json!(x),
                            Lit::Str(s) => json!(s),
                            Lit::Strs(s) => json!(s),
                            _ => Value::Null,
                        },
                        PropValue::Bind(e, f) => {
                            if let crate::bind::Expr::Path(p) = e
                                && let Some(w) = b.write_path(p)
                            {
                                links.push((k.clone(), w));
                            }
                            let x = b.eval(e);
                            match f {
                                Some(f) => {
                                    json!(crate::bind::format_value(&x, *f, b.env.language()))
                                }
                                None => x,
                            }
                        }
                        PropValue::Template(t) => json!(t.format(&b.scope())),
                    };
                    props.insert(k.clone(), x);
                }
                let mut vm = Value::Object(props);
                let id = root_id.unwrap_or(if n.id.is_empty() {
                    comp.as_str()
                } else {
                    n.id.as_str()
                });
                let mount = format!("{}/{id}", self.path_of(self.parent()));
                let before = vm.clone();
                let r = self.mount_doc(cdoc, mount, id, &mut vm, b.env);
                // The instance's own element props apply to its root.
                if !n.props.is_empty()
                    && let Some(&last) = Some(&self.last_closed)
                    && last != NONE
                {
                    let mut built = Built::default();
                    let rp = self.resolve(b, n, &r.path, &mut built);
                    let el = Ui::box_of(&rp, self.nodes[last].el);
                    let props = self.props_of(b, n, self.nodes[last].key, &rp, &mut built);
                    self.nodes[last].el = el;
                    if props != Props::default() {
                        self.nodes[last].props = props;
                    }
                }
                // Its two-way writes reach the bound paths of the user.
                for (k, w) in links {
                    if vm[&k] != before[&k] {
                        b.writes.push((w, vm[&k].clone()));
                    }
                }
                b.events.extend(r.events);
                return Some(r.path);
            }
            NodeKind::Element(_) => {}
        }
        let NodeKind::Element(kind) = n.kind else {
            return None;
        };
        let id = match root_id {
            Some(r) => r.to_string(),
            None if n.id.is_empty() => {
                // Anonymous: its kind and its place among same-kind siblings.
                let k = doc.nodes[n.parent]
                    .children
                    .iter()
                    .take_while(|&&c| c != i)
                    .filter(|&&c| doc.nodes[c].id.is_empty() && doc.nodes[c].kind == n.kind)
                    .count();
                format!("{kind}#{k}")
            }
            None => n.id.clone(),
        };
        let parent_path = self.path_of(self.parent());
        let path = if parent_path == "root" {
            id.clone()
        } else {
            format!("{parent_path}/{id}")
        };
        let key = crate::id::key_of_path(&path);
        let mut built = Built {
            path: path.clone(),
            doc: doc.name.clone(),
            node: i,
            line: n.line,
            mount: b.mount.clone(),
            ..Built::default()
        };
        let r = self.resolve(b, n, &path, &mut built);
        if r.has("visible") && !r.flag("visible") {
            return None;
        }
        self.docs.seen_now.insert(key);
        let props = self.props_of(b, n, key, &r, &mut built);
        let t = self.theme;
        let text_of = |r: &Resolved, b: &Build<'_>| -> String {
            if let Some(k) = r.str("key")
                && let Some(s) = b.env.translate(k)
            {
                return s;
            }
            r.str("text").unwrap_or_default().to_string()
        };
        let mut resp = crate::ui::Response::default();
        let container = schema::kind(kind).is_some_and(|k| k.container);
        if container {
            match kind {
                "panel" => {
                    self.open_panel(
                        id.as_str(),
                        crate::ui::PanelDesc {
                            row: false,
                            width: r.size("width").unwrap_or_default(),
                            height: r.size("height").unwrap_or_default(),
                            ..Default::default()
                        },
                    );
                    let at = self.parent();
                    self.nodes[at].el = Ui::box_of(&r, self.nodes[at].el);
                    self.nodes[at].props = props;
                }
                "scroll" => {
                    self.open_scroll(
                        id.as_str(),
                        crate::ui::ScrollDesc {
                            width: r.size("width").unwrap_or(Size::grow()),
                            height: r.size("height").unwrap_or(Size::grow()),
                            ..Default::default()
                        },
                    );
                    let at = self.parent();
                    self.nodes[at].el = Ui::box_of(&r, self.nodes[at].el);
                    self.nodes[at].props = props;
                }
                "list" | "cells" => {
                    self.build_virtual(b, n, kind, &id, &r, props, &path);
                    self.docs.built.push(built);
                    self.fire(b, n, key, &path, resp);
                    return Some(path);
                }
                _ => {
                    let el = Ui::box_of(
                        &r,
                        Element {
                            direction: if kind == "row" {
                                Direction::Row
                            } else {
                                Direction::Column
                            },
                            ..Element::default()
                        },
                    );
                    self.open_with(id.as_str(), el, props);
                }
            }
            if kind == "slot" {
                let at = self.parent();
                self.docs.slots.insert(path.clone(), at);
                let short = format!("{}/{}", b.mount, n.id);
                self.docs.slots.entry(short).or_insert(at);
            }
            self.docs.built.push(built);
            for &c in &n.children {
                self.build_node(b, c, None);
            }
            resp = self.close();
            self.fire(b, n, key, &path, resp);
            return Some(path);
        }
        // A leaf: its widget, then its props and box over the widget's.
        match kind {
            "label" => {
                let text = text_of(&r, b);
                let desc = LabelDesc {
                    step: r.str("step").and_then(|s| {
                        crate::style_file::ramp_step(&mut t.type_ramp.clone(), s).copied()
                    }),
                    color: r.color("color"),
                    align: match r.str("textAlign") {
                        Some("center") => TextAlign::Center,
                        Some("end") => TextAlign::End,
                        _ => TextAlign::Start,
                    },
                    nowrap: r.flag("nowrap"),
                    width: r.size("width").unwrap_or_default(),
                    font: r
                        .str("font")
                        .and_then(FontRole::from_name)
                        .unwrap_or_default(),
                    heading: r.flag("heading"),
                    ellipsis: r.flag("ellipsis"),
                    max_lines: r.num("maxLines").unwrap_or(0.0) as u32,
                };
                self.label(id.as_str(), &text, desc);
            }
            "button" => {
                let text = text_of(&r, b);
                resp = self.button(
                    id.as_str(),
                    &text,
                    ButtonDesc {
                        variant: match r.str("variant") {
                            Some("primary") => ButtonVariant::Primary,
                            Some("quiet") => ButtonVariant::Quiet,
                            Some("danger") => ButtonVariant::Danger,
                            _ => ButtonVariant::Secondary,
                        },
                        size: match r.str("size") {
                            Some("small") => ControlSize::Small,
                            Some("large") => ControlSize::Large,
                            _ => ControlSize::Medium,
                        },
                        disabled: r.flag("disabled"),
                        width: r.size("width"),
                        autofocus: r.flag("autofocus"),
                    },
                );
            }
            "art" => {
                // An art slot: its art fitted by its spec, or the style's
                // placeholder for its kind (docs/ui.md §13.1).
                self.art_spec(&path, &r);
                let spec = self
                    .docs
                    .arts
                    .last()
                    .map(|(_, s)| s.clone())
                    .expect("pushed");
                let texture = r.num("texture").and_then(|h| b.env.texture(f64::from(h)));
                self.art(
                    id.as_str(),
                    &crate::widgets::ArtDesc {
                        spec,
                        texture,
                        size: None,
                        width: r.size("width").unwrap_or_default(),
                        height: r.size("height").unwrap_or_default(),
                        label: r.str("name").unwrap_or_default().to_string(),
                    },
                );
            }
            "image" => {
                let texture = r.num("texture").and_then(|h| b.env.texture(f64::from(h)));
                let tint = r.color("tint");
                self.image(
                    id.as_str(),
                    crate::ui::ImageDesc {
                        texture,
                        tint,
                        width: r.size("width").unwrap_or_default(),
                        height: r.size("height").unwrap_or_default(),
                        aspect: r
                            .nums("ratio")
                            .filter(|n| n.len() == 2 && n[1] > 0.0)
                            .map_or(r.num("aspect").unwrap_or(0.0), |n| n[0] / n[1]),
                        label: r.str("name").unwrap_or_default().to_string(),
                        ..Default::default()
                    },
                );
            }
            "icon" => {
                let name = r.str("icon").unwrap_or_default().to_string();
                self.icon(
                    id.as_str(),
                    &name,
                    crate::widgets::IconDesc {
                        size: r.num("iconSize").unwrap_or(0.0),
                        tint: r.color("color"),
                        label: r.str("name").map(str::to_string),
                    },
                );
            }
            "progress" => {
                let v = r.num("value").unwrap_or(0.0);
                self.progress(
                    id.as_str(),
                    v,
                    crate::ui::ProgressDesc {
                        width: r.size("width"),
                    },
                );
            }
            "slider" => {
                let mut v = r.num("value").unwrap_or(0.0);
                resp = self.slider(
                    id.as_str(),
                    &mut v,
                    crate::ui::SliderDesc {
                        min: r.num("min").unwrap_or(0.0),
                        max: r.num("max").unwrap_or(1.0),
                        step: r.num("stepBy").unwrap_or(0.0),
                        width: r.size("width"),
                    },
                );
                if resp.changed {
                    self.write_back(b, n, "value", json!(v));
                }
            }
            "toggle" => {
                let mut on = r.flag("checked");
                let text = text_of(&r, b);
                resp = self.toggle(id.as_str(), &mut on, &text);
                if resp.changed {
                    self.write_back(b, n, "checked", json!(on));
                }
            }
            "field" => {
                let mut s = r.str("input").unwrap_or_default().to_string();
                resp = self.text_field(
                    id.as_str(),
                    &mut s,
                    &crate::ui::FieldDesc {
                        placeholder: r.str("placeholder").unwrap_or_default().to_string(),
                        width: r.size("width"),
                        max_chars: r.num("maxChars").unwrap_or(0.0) as u32,
                    },
                );
                if resp.changed {
                    self.write_back(b, n, "input", json!(s));
                }
            }
            "badge" => {
                let text = text_of(&r, b);
                let tone = match r.str("tone") {
                    Some("info") => crate::widgets::Tone::Info,
                    Some("success") => crate::widgets::Tone::Success,
                    Some("warn") => crate::widgets::Tone::Warn,
                    Some("danger") => crate::widgets::Tone::Danger,
                    Some("accent") => crate::widgets::Tone::Primary,
                    _ => crate::widgets::Tone::Neutral,
                };
                self.badge(id.as_str(), &text, tone);
            }
            "spacer" => {
                let s = r.size("width").or(r.size("height"));
                self.spacer(id.as_str(), s);
            }
            "divider" => self.divider(id.as_str(), r.flag("vertical")),
            other => {
                self.warnings
                    .push(format!("{}:{}: '{other}' is not built", doc.source, n.line));
                return None;
            }
        }
        let last = self.last_closed;
        if last != NONE && last < self.nodes.len() {
            let mut el = self.nodes[last].el;
            // The widget's own box stays where the document says nothing.
            for name in [
                "height",
                "padding",
                "gap",
                "align",
                "cross",
                "background",
                "radius",
                "frame",
                "aspect",
                "interactive",
                "wrap",
            ] {
                if r.has(name) {
                    el = Ui::box_of(
                        &Resolved {
                            vals: r
                                .vals
                                .iter()
                                .filter(|(s, _)| PROPS[*s].name == name)
                                .cloned()
                                .collect(),
                        },
                        el,
                    );
                }
            }
            if r.has("width")
                && !matches!(
                    kind,
                    "label" | "button" | "image" | "art" | "progress" | "slider" | "field"
                )
            {
                el.width = r.size("width").unwrap_or(el.width);
            }
            self.nodes[last].el = el;
            if props != Props::default() {
                let mut p = props;
                // A widget's own states (a button's interaction) stay.
                let own = std::mem::take(&mut self.nodes[last].props.states);
                p.states.extend(
                    own.into_iter()
                        .filter(|g| !n.groups.iter().any(|x| x.name == g.name)),
                );
                self.nodes[last].props = p;
            }
        }
        self.docs.built.push(built);
        self.fire(b, n, key, &path, resp);
        Some(path)
    }

    /// One item of a `for` outside a virtualised container.
    fn build_item(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        var: &str,
        item: Value,
        k: usize,
        origin: Option<&str>,
    ) {
        b.locals
            .push((var.to_string(), item, origin.map(|o| format!("{o}.{k}"))));
        b.locals.push(("$index".into(), json!(k), None));
        let doc = b.doc.clone();
        for &c in &n.children {
            // Each item's elements are told apart by its index (`Item#3`).
            let id = match &doc.nodes[c].kind {
                NodeKind::Element(kind) if doc.nodes[c].id.is_empty() => {
                    Some(format!("{kind}#{k}"))
                }
                NodeKind::Element(_) | NodeKind::Use(_) => Some(format!("{}#{k}", doc.nodes[c].id)),
                _ => None,
            };
            self.build_node(b, c, id.as_deref());
        }
        b.locals.pop();
        b.locals.pop();
    }

    /// A `list` or `cells` and its `for`: only the rows in view are built.
    #[allow(clippy::too_many_arguments)]
    fn build_virtual(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        kind: &str,
        id: &str,
        r: &Resolved,
        props: Props,
        path: &str,
    ) {
        let doc = b.doc.clone();
        let Some(&f) = n.children.first() else { return };
        let NodeKind::For { var, list } = &doc.nodes[f].kind else {
            return;
        };
        let items = b.eval(list).as_array().cloned().unwrap_or_default();
        let origin = match list {
            crate::bind::Expr::Path(p) => b.write_path(p),
            _ => None,
        };
        let count = items.len() as u32;
        let padding = r.nums("padding").map(insets_of).unwrap_or_default();
        let range = if kind == "list" {
            self.open_list(
                id,
                crate::widgets::ListDesc {
                    count,
                    row_height: r.num("rowHeight").unwrap_or(0.0),
                    gap: r.num("gap").unwrap_or(0.0),
                    width: r.size("width").unwrap_or(Size::grow()),
                    height: r.size("height"),
                    padding,
                },
            )
        } else {
            self.open_grid(
                id,
                crate::widgets::GridDesc {
                    count,
                    cell_width: r.num("cellWidth").unwrap_or(0.0),
                    cell_height: r.num("cellHeight").unwrap_or(0.0),
                    columns: r.num("cellColumns").unwrap_or(0.0) as u32,
                    gap: r.num("gap").unwrap_or(0.0),
                    width: r.size("width").unwrap_or(Size::grow()),
                    height: r.size("height"),
                    padding,
                },
            )
        };
        let at = self.parent();
        if props != Props::default() {
            self.nodes[at].props = props;
        }
        for k in range.first..range.last.min(count) {
            let resp = if kind == "list" {
                self.list_item(k, false)
            } else {
                self.grid_item(k, false)
            };
            let row = format!(
                "{path}/{}",
                if kind == "list" {
                    crate::ui_idi("Row", k)
                } else {
                    crate::ui_idi("Cell", k)
                }
                .as_str()
            );
            if resp.clicked {
                b.event(&row, "select", id, json!(k));
            }
            self.build_item(
                b,
                &doc.nodes[f],
                var,
                items[k as usize].clone(),
                k as usize,
                origin.as_deref(),
            );
            self.close();
        }
        self.close_list();
    }

    fn art_spec(&mut self, path: &str, r: &Resolved) {
        let kind = r
            .str("kind")
            .and_then(crate::slot::SlotKind::from_name)
            .unwrap_or(crate::slot::SlotKind::ALL[0]);
        let mut s = crate::slot::SlotSpec::new(path, kind);
        if let Some(n) = r.nums("ratio").filter(|n| n.len() == 2) {
            s.aspect =
                crate::slot::SlotAspect::One(crate::slot::Aspect::new(n[0] as u16, n[1] as u16));
        }
        if let Some(n) = r.nums("focal").filter(|n| n.len() == 2) {
            s.focal = [n[0], n[1]];
        }
        if let Some(n) = r.nums("safe").filter(|n| n.len() == 4) {
            s.safe = Some([n[0], n[1], n[2], n[3]]);
        }
        if let Some(f) = r.str("fit") {
            s.fit = match f {
                "contain" => crate::slot::Fit::Contain,
                "fill" => crate::slot::Fit::Fill,
                "nineSlice" => crate::slot::Fit::NineSlice,
                "tile" => crate::slot::Fit::Tile,
                _ => crate::slot::Fit::Cover,
            };
        }
        self.docs.arts.push((path.to_string(), s));
    }

    /// A control's two-way prop: its binding's path takes the new value.
    fn write_back(&mut self, b: &mut Build<'_>, n: &DocNode, prop: &str, v: Value) {
        let bound = n.props.iter().rev().find(|p| p.name() == prop);
        if let Some(p) = bound
            && let PropValue::Bind(crate::bind::Expr::Path(path), _) = &p.value
            && let Some(w) = b.write_path(path)
        {
            b.writes.push((w.clone(), v.clone()));
            b.event("", "set", &w, v.clone());
        }
        b.event("", "changed", prop, v);
    }

    /// Fires a node's triggers for what happened to it this tick.
    fn fire(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        key: u64,
        path: &str,
        resp: crate::ui::Response,
    ) {
        // Paths of the events this element pushed without one.
        for e in b.events.iter_mut().filter(|e| e.path.is_empty()) {
            e.path = path.to_string();
        }
        if resp.clicked {
            b.event(path, "click", &n.id, Value::Null);
        }
        let appeared = !self.docs.seen.contains(&key);
        for (k, t) in n.triggers.iter().enumerate() {
            let go = match &t.when {
                When::Click => resp.clicked,
                When::Changed => resp.changed,
                When::Appear => appeared,
                When::Action(a) => self.docs.accepts && self.action_fired(a),
                When::Change(e) => {
                    let v = b.eval(e);
                    let old = self.docs.last.insert((key, k), v.clone());
                    old.is_some_and(|o| o != v)
                }
            };
            if !go {
                continue;
            }
            match &t.act {
                Act::Play(tl) => {
                    let name = qualify(&b.doc.name, tl);
                    let id = self.play_timeline(path, &name, crate::motion::PlayDesc::default());
                    self.docs.plays.insert((path.to_string(), tl.clone()), id);
                }
                Act::State(g, s) => {
                    self.docs.states.insert((key, g.clone()), s.clone());
                }
                Act::Emit(name) => b.event(path, "emit", name, Value::Null),
                Act::Push(screen) => self
                    .docs
                    .ops
                    .push(crate::layers::LayerOp::Push(screen.clone(), Value::Null)),
                Act::Pop => self.docs.ops.push(crate::layers::LayerOp::Pop),
                Act::Action(a) => {
                    self.fire_action(a);
                    b.event(path, "action", a, Value::Null);
                }
                Act::Set(p, e) => {
                    let v = b.eval(e);
                    if let Some(w) = b.write_path(p) {
                        b.writes.push((w.clone(), v.clone()));
                        b.event(path, "set", &w, v);
                    }
                }
            }
        }
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
    use crate::slot::SlotAspect;
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
        "focal": s.focal,
        "safe": s.safe,
        "fit": match s.fit {
            crate::slot::Fit::Cover => "cover",
            crate::slot::Fit::Contain => "contain",
            crate::slot::Fit::Fill => "fill",
            crate::slot::Fit::NineSlice => "nineSlice",
            crate::slot::Fit::Tile => "tile",
        },
    })
}
