//! Mounting a document (docs/ui.md §2.1): its tree is built into the
//! immediate-mode [`Ui`] where it is mounted, every tick, so documents and
//! code mix in one tree. Props are resolved in order (the line, the
//! matching `at` blocks, a probe edit), bindings are evaluated against the
//! viewmodel ([`crate::bind`]), `for` builds a list's rows (only those in
//! view under `list` and `cells`), triggers fire, and controls write their values back
//! into the viewmodel (two-way bindings). What the build saw is kept for
//! the probe: every element's document line, bindings and their values,
//! the art slots, the probe's edits.

use super::schema::{self, PROPS, Ty};
use super::{Act, DocNode, Lit, NodeKind, Paint, PropValue, StateValue, UiDocument, When};
use super::{css, motion};
use crate::Rect;
use crate::bind::{Env, Scope, number, text, truthy};
use crate::doc::schema::Kind;
use crate::element::{Edges, GridFlow, GridPlace, GridSpec, LayoutMode, Position, Prop, Props};
use crate::layout::{Align, Cross, Direction, Element, FontRole, Insets, NONE, Size, TextAlign};
use crate::motion::{StateGroup, VisualState};
use crate::style::Srgba;
use crate::ui::{ButtonDesc, ButtonVariant, LabelDesc, Ui};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// A resolved value of a prop.
#[derive(Clone, Debug, PartialEq)]
pub enum V {
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
    Tracks(super::tracks::TrackList),
    Strs(Vec<String>),
    Spans(Result<Vec<crate::TextSpan>, String>),
}

/// An element's resolved props, by schema index.
#[derive(Clone, Default)]
pub struct Resolved {
    pub vals: Vec<(usize, V)>,
    pub cascade: cascade::Cascade,
    pub span_styles: Vec<(crate::TextSpan, crate::TextSpanStyle)>,
    pub transitions: Vec<crate::motion::css::CssTransition>,
    pub animations: Vec<crate::motion::css::CssAnimation>,
    /// Scroll and view timelines it declares, and its `timeline-scope`.
    timelines: Vec<crate::motion::scroll::NamedTimeline>,
    timeline_scope: Vec<String>,
    /// The layout key this element publishes as a query container.
    pub container: Option<u64>,
    /// A numbered `<li>`'s number ([`super::lists::ordinal`]).
    pub ordinal: Option<i64>,
    /// Its effective CSS text and font style ([`super::text_css`]); `None`
    /// for an element that neither declares nor shows text.
    pub text: Option<Arc<super::text_css::TextCss>>,
    /// Its SMIL animations are playing: it resolves again next frame.
    pub smil: bool,
    /// Its `transform` and 3D properties ([`crate::transform3d`]).
    space3d: Option<Arc<crate::transform3d::Space3d>>,
    /// The `@position-try` rules its `position-try-fallbacks` names.
    position_tries: Vec<(String, crate::anchor::Placement)>,
    /// Its `@starting-style` values that differ from its style, on the
    /// build it first appears in (the before-change style of its
    /// transitions, CSS Transitions 2 §3).
    starting: Vec<(String, String, usize)>,
    /// Its generated content and pseudo-element styles.
    pub generated: Option<Arc<super::generated::Generated>>,
    /// Its transitions and animations of computed values, for the motion
    /// state ([`animate`]).
    pub style: Option<Arc<animate::StyleCommit>>,
    /// Its box pseudo-elements ([`DocNode::pseudo`]), each resolved under
    /// it: the name, its node and its values.
    pub pseudo: Vec<(&'static str, usize, Arc<Resolved>)>,
    /// Its direction and bidi ([`cascade::flow`]).
    flow: crate::layout::Flow,
    /// `dir="auto"`: its direction is its text's.
    dir_auto: bool,
    /// The part its `part` attribute names, with its classes and ARIA
    /// states as qualifiers ([`Ui::document_part`]).
    part: Option<crate::dress::PartRef>,
    part_host: Option<Arc<PartHost>>,
}

/// Authored host facts shared by all parts of a document control.
#[derive(Clone, Debug)]
pub struct PartHost {
    pub document: String,
    pub tag: String,
    pub attributes: BTreeMap<String, Value>,
    pub computed: Arc<super::variables::Computed>,
    /// A button's own plate ([`plate_prop`]) as its author declared it: its
    /// plate part takes these over the style's, and the host draws none.
    plate: Vec<(usize, V)>,
}

/// Whether `name` makes an element's box: its background, border, padding,
/// corners and shadows.
fn plate_prop(name: &str) -> bool {
    corner_prop(name)
        || name == "boxShadow"
        || name.starts_with("background")
        || name.starts_with("border")
        || name.starts_with("padding")
}

/// Whether `name` shapes an element's corners.
fn corner_prop(name: &str) -> bool {
    name == "radius"
        || crate::doc::boxes::CORNER_RADII.contains(&name)
        || crate::doc::boxes::CORNER_SHAPES.contains(&name)
}

#[derive(Clone)]
pub struct ResolvedNode {
    pub id: String,
    pub path: String,
    pub key: u64,
    pub values: Resolved,
    pub probe: Built,
}

impl Resolved {
    /// Its flow, a `dir="auto"` one in its text's direction.
    pub fn own_flow(&self) -> crate::layout::Flow {
        let mut flow = self.flow;
        if self.dir_auto {
            let text = self.str("text").unwrap_or_default();
            flow.rtl = og_i18n::paragraph_direction(text, og_i18n::Direction::Ltr)
                == og_i18n::Direction::Rtl;
        }
        flow
    }

    pub fn get(&self, name: &str) -> Option<&V> {
        self.vals
            .iter()
            .rev()
            .find(|(s, _)| PROPS[*s].name == name)
            .map(|(_, v)| v)
    }
    pub fn has(&self, name: &str) -> bool {
        self.get(name).is_some()
    }
    pub fn num(&self, name: &str) -> Option<f32> {
        match self.get(name)? {
            V::Num(n) => Some(*n as f32),
            V::Bool(b) => Some(f32::from(u8::from(*b))),
            _ => None,
        }
    }
    /// A handle prop (a texture's) as bound, every bit kept.
    pub fn handle(&self, name: &str) -> Option<f64> {
        match self.get(name)? {
            V::Num(n) => Some(*n),
            _ => None,
        }
    }
    pub fn flag(&self, name: &str) -> bool {
        matches!(self.get(name), Some(V::Bool(true)))
    }
    pub fn str(&self, name: &str) -> Option<&str> {
        match self.get(name)? {
            V::Str(s) => Some(s),
            _ => None,
        }
    }
    /// CSS `object-position` as its fractions and its px (`None`: not
    /// declared), and whether `image-rendering` samples nearest; both
    /// checked when the CSS was lowered.
    pub fn object_css(&self) -> (Option<[[f32; 2]; 2]>, bool) {
        use crate::doc::image_css as ic;
        let at = self
            .str("objectPosition")
            .and_then(|v| ic::object_position(v).ok())
            .map(|[x, y]| [[x.0, y.0], [x.1, y.1]]);
        let nearest = self
            .str("imageRendering")
            .and_then(|v| ic::image_rendering(v).ok())
            .unwrap_or(false);
        (at, nearest)
    }
    pub fn size(&self, name: &str) -> Option<Size> {
        match self.get(name)? {
            V::Size(s) => Some(*s),
            V::Num(n) => Some(Size::Fixed(*n as f32)),
            _ => None,
        }
    }
    pub fn color(&self, name: &str) -> Option<Srgba> {
        match self.get(name)? {
            V::Color(c) => Some(*c),
            _ => None,
        }
    }
    pub(crate) fn part_rect(&self, rect: Rect, owner: Rect) -> Rect {
        let extent = |name, current: f32, available: f32| match self.size(name) {
            Some(Size::Fixed(n)) => n.max(0.0),
            Some(Size::Percent(p)) => (available * p).max(0.0),
            Some(Size::Calc(c)) => c.of(available).max(0.0),
            Some(Size::Fit { min, max }) => current.clamp(min, max),
            Some(Size::Grow { min, max }) => available.clamp(min, max),
            _ => current,
        };
        let width = extent("width", rect.width, owner.width);
        let height = extent("height", rect.height, owner.height);
        Rect::new(
            rect.x + (rect.width - width) / 2.0 + self.num("x").unwrap_or(0.0),
            rect.y + (rect.height - height) / 2.0 + self.num("y").unwrap_or(0.0),
            width,
            height,
        )
    }
    pub fn nums(&self, name: &str) -> Option<&[f32]> {
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
pub fn coerce(ui: &Ui, ty: Ty, v: &Value) -> Option<V> {
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
        Ty::Transition | Ty::Tracks => return None,
    })
}

fn lit(ui: &Ui, l: &Lit) -> Option<V> {
    Some(match l {
        Lit::Num(n) => V::Num(f64::from(*n)),
        Lit::Nums(n) => V::Nums(n.clone()),
        Lit::Bool(b) => V::Bool(*b),
        Lit::Str(s) => V::Str(s.clone()),
        Lit::Size(s) => V::Size(*s),
        Lit::Paint(p) => V::Color(paint(ui, p)?),
        Lit::Transition(t) => V::Transition(*t),
        Lit::Tracks(t) => V::Tracks(t.clone()),
        Lit::Strs(s) => V::Strs(s.clone()),
    })
}

pub fn insets_of(n: &[f32]) -> Insets {
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

/// The inner display of a `display` word: `flow`, `flex`, `grid` or `table`.
pub(super) fn inner_display(d: Option<&str>) -> &'static str {
    match d {
        Some("flex" | "inlineFlex") => "flex",
        Some("grid" | "inlineGrid") => "grid",
        Some("table" | "inlineTable") => "table",
        _ => "flow",
    }
}

/// Whether a `display` word's outer display is `inline`.
pub(super) fn outer_inline(d: Option<&str>) -> bool {
    matches!(
        d,
        Some("inline" | "inlineBlock" | "inlineFlex" | "inlineGrid" | "inlineTable")
    )
}

/// Taken out of the flow by its position (`absolute`, `fixed`, an anchor)
/// or by an inset alone: its `auto` size shrinks to fit or fills between
/// its insets (CSS 2.1 §10.3.7), never the flow's width.
fn positioned(r: &Resolved) -> bool {
    match r.str("position") {
        Some("absolute" | "screen") => true,
        Some("relative" | "sticky") => false,
        _ => ["start", "top", "end", "bottom"]
            .iter()
            .any(|k| r.num(k).is_some()),
    }
}

/// Its CSS `float` side as written, when it floats.
fn floated(r: &Resolved) -> Option<&str> {
    r.str("float").filter(|f| *f != "none")
}

/// A box's table part from its `display` word.
pub(super) fn table_part(d: Option<&str>, caption_side: Option<&str>) -> crate::TablePart {
    use crate::TablePart as T;
    match d {
        Some("table" | "inlineTable") => T::Table,
        Some("tableHeaderGroup") => T::Header,
        Some("tableRowGroup") => T::Body,
        Some("tableFooterGroup") => T::Footer,
        Some("tableRow") => T::Row,
        Some("tableCell") => T::Cell,
        Some("tableCaption") => T::Caption {
            below: caption_side == Some("bottom"),
        },
        Some("tableColumn") => T::Column,
        Some("tableColumnGroup") => T::ColumnGroup,
        _ => T::None,
    }
}

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
        _ => super::tracks::TrackList::default(),
    };
    let (columns, rows) = (tracks("columns"), tracks("rows"));
    let auto = |name: &str| tracks(name).tracks;
    // `normal` (and a stretch) leaves the tracks to fill the grid.
    let content = |name: &str| {
        r.str(name)
            .filter(|a| !matches!(*a, "normal" | "stretch"))
            .map(align_of)
    };
    GridSpec {
        columns: columns.tracks,
        rows: rows.tracks,
        lines: [columns.lines, rows.lines],
        auto_columns: auto("autoColumns"),
        auto_rows: auto("autoRows"),
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
        content: [content("align"), content("alignContent")],
        align_items: r.str("cross").map(cross_of),
        ..GridSpec::default()
    }
}

/// The link id of inline link `k` ([`super::InlineRun::link`]): `link` for
/// an `<a>` element's own `href`, `link<k>` for its text's `<a>`s.
pub fn inline_link(k: usize) -> String {
    if k == usize::MAX {
        "link".into()
    } else {
        format!("link{k}")
    }
}

/// One element's build: its document, what it inherits, and the inputs
/// its mount compiled for it (`cached`). A static build (an element
/// image's, [`Ui::build_static`]) has none: its bindings read nothing.
pub struct Build<'a> {
    pub doc: Arc<UiDocument>,
    pub mount: String,
    pub env: &'a dyn Env,
    pub row_key: String,
    pub inherited: Arc<super::variables::Computed>,
    pub events: Vec<DocEvent>,
    pub writes: Vec<(String, Value)>,
    pub cached: Option<&'a instances::CachedInputs>,
}

impl Build<'_> {
    /// The scope a static build evaluates in: no values.
    fn scope(&self) -> Scope<'_> {
        Scope {
            vm: &Value::Null,
            locals: Vec::new(),
            env: self.env,
        }
    }

    pub fn eval(&self, e: &crate::bind::Expr) -> Value {
        match self.cached {
            Some(cached) => cached.expression(e),
            None => e.eval(&self.scope()),
        }
    }

    /// The path a write to `path` goes to: its mount's binding of it. A
    /// static build writes nothing.
    pub fn write_path(&self, path: &str) -> Option<String> {
        self.cached.map(|_| path.to_owned())
    }

    pub fn event(&mut self, path: &str, kind: &str, name: &str, value: Value) {
        self.events.push(DocEvent {
            mount: self.mount.clone(),
            path: path.to_string(),
            row_key: self.row_key.clone(),
            kind: kind.to_string(),
            name: name.to_string(),
            value,
        });
    }

    pub fn value(&self, ui: &Ui, ty: Ty, v: &PropValue) -> Option<V> {
        if let Some(cached) = self.cached
            && !matches!(v, PropValue::Lit(_))
        {
            return cached.property(v, ty);
        }
        self.evaluate(ui, ty, v)
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
    /// An exposed part belongs to the authored markup host. Classes and
    /// attributes qualify that host; widget interaction states are merged
    /// when the control is built.
    fn document_part(r: &Resolved, n: &DocNode, b: &Build<'_>) -> Option<crate::dress::PartRef> {
        use crate::dress::intern;
        let mut part = crate::dress::PartRef::of(
            intern(n.selector_tag.as_deref()?),
            intern(r.str("part")?),
            None,
        );
        for (name, e) in &n.selector_attrs {
            if name == "class" {
                if let Value::String(s) = b.eval(e) {
                    for c in s.split_whitespace() {
                        part = part.flag(intern(c));
                    }
                }
            } else if let Some(c) = name.strip_prefix("class:")
                && truthy(&b.eval(e))
            {
                part = part.flag(intern(c));
            } else if !name.starts_with("class:") {
                let value = b.eval(e);
                if !value.is_null() && value != Value::Bool(false) {
                    part = part.attr(intern(name), intern(&crate::bind::text(&value)));
                }
            }
        }
        Some(
            part.flag_if(r.flag("ariaSelected"), "selected")
                .flag_if(r.flag("ariaPressed") || r.flag("ariaChecked"), "on"),
        )
    }

    /// Apply a part's presentation with the same values documents use.
    pub(crate) fn apply_part_appearances(&mut self) {
        let moving: Vec<_> = self
            .interaction
            .state
            .motion
            .part_styles
            .iter()
            .filter(|(_, styles)| styles.values().any(|style| style.running))
            .filter_map(|(key, _)| self.tree.index.find(*key))
            .filter(|&i| self.tree.nodes[i].part_box.is_some())
            .collect();
        for i in moving {
            self.tree.rewrite(i);
        }
        for k in 0..self.tree.emitted.len() {
            self.apply_part_appearance(self.tree.emitted[k]);
        }
    }

    pub(crate) fn apply_part_appearance(&mut self, i: usize) {
        if self.tree.nodes[i].content_slot != NONE {
            self.apply_part_box(self.tree.nodes[i].content_slot);
            return;
        }
        if self.tree.nodes[i].part_box.is_some() {
            self.apply_part_box(i);
            return;
        }
        let Some(part) = self.tree.nodes[i]
            .el
            .part
            .or_else(|| self.tree.nodes[i].el.frame.map(crate::StylePart::part_ref))
        else {
            return;
        };
        let r = self.part_values(&part, Some(self.tree.nodes[i].key));
        let step = match &*self.tree.nodes[i].content {
            crate::layout::Content::Text(text) => r.part_type(&self.env.theme, text.step),
            _ => self.env.theme.type_ramp.body,
        };
        let ink = r.color("color");
        let em = step.size;
        let build = self.tree.build();
        let n = &mut self.tree.nodes[i];
        let el = n.el.write(build);
        *el = Self::box_of(&r, *el, em);
        if let Some(radius) = n.decor.as_ref().and_then(|d| d.radius) {
            el.radius = radius;
        }
        if let Some(x) = r.num("x") {
            n.props.write(build).transform.x = x;
        }
        if let Some(y) = r.num("y") {
            n.props.write(build).transform.y = y;
        }
        if let Some(x) = r.num("xShift") {
            n.props.write(build).transform.shift_x = x;
        }
        if let Some(y) = r.num("yShift") {
            n.props.write(build).transform.shift_y = y;
        }
        if let Some(V::Transition(t)) = r.get("motion") {
            n.props.write(build).motion = Some(*t);
        }
        if let crate::layout::Content::Text(text) = n.content.write(build) {
            if let Some(color) = ink {
                text.color = color;
            }
            text.step = step;
        }
    }

    /// What document media queries read: the viewport and the theme.
    pub fn media_context(&self) -> super::media::Context {
        let (vp, theme) = (self.viewport(), self.theme());
        super::media::Context {
            width: vp.width,
            height: vp.height,
            reduced_motion: theme.reduced_motion,
            color_vision: theme.color_vision,
            coarse_pointer: match theme.density {
                crate::DensityMode::Touch => true,
                crate::DensityMode::Mouse => false,
                crate::DensityMode::Auto | crate::DensityMode::Pad => vp.coarse_pointer,
            },
            dark: theme.scheme == crate::Scheme::Dark,
            high_contrast: theme.contrast == crate::Contrast::High,
            reduced_transparency: theme.reduced_transparency,
            resolution: vp.dpr,
            pad: theme.density == crate::DensityMode::Pad,
        }
    }

    /// Resolves a node's props: its lines, its matching `at` blocks, then
    /// the probe's edits of `path`.
    pub fn resolve(
        &self,
        b: &Build<'_>,
        n: &DocNode,
        path: &str,
        built: &mut Built,
        inherited: &super::variables::Computed,
        structure: &mut dyn FnMut(&super::selectors::Selector) -> bool,
    ) -> Resolved {
        let root_variables;
        let inherited = if inherited.is_empty() {
            root_variables = self.css_root_variables();
            root_variables.as_ref()
        } else {
            inherited
        };
        let mut r = Resolved::default();
        let vp = self.viewport();
        let context = self.media_context();
        let mut media_reads = super::media::Reads::default();
        let chain = super::container::chain(inherited);
        let scroll = |key: u64| {
            self.docs
                .containers
                .scroll_states
                .get(&key)
                .map_or(0, |s| s.1)
        };
        let probe = super::container::Probe {
            sizes: &self.docs.containers.sizes,
            scroll: &scroll,
            inherited,
        };
        let mut container_read = super::container::Read::default();
        let mut applies = |q: &super::AtQuery, reads: &mut super::media::Reads| {
            q.media.matches(context, reads)
                && q.container
                    .iter()
                    .all(|c| c.matches(&chain, &probe, &mut container_read))
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
            safe: {
                let s = vp.env_insets();
                [s.top, s.right, s.bottom, s.left]
            },
            hud: {
                let s = vp.hud_insets(self.env.hud_max_aspect);
                [s.top, s.right, s.bottom, s.left]
            },
            containers: super::container::unit_basis(&chain, &self.docs.containers.sizes),
            vertical: cascade::root_vertical(inherited).unwrap_or(false),
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
        let options = cascade::Options {
            inline: &inline,
            inherited,
            basis: Some(basis),
            attr: &attr,
            sheet: Some(&b.doc.sheet),
            starting: false,
            carry: self.docs.library.wide_keywords,
        };
        let mut rule_applies = |rule: &super::StyleRule| {
            applies(&rule.query, &mut media_reads)
                && rule.condition.as_ref().is_none_or(|e| truthy(&b.eval(e)))
                && rule.structure.as_ref().is_none_or(|s| structure(s))
        };
        let mut rules = std::borrow::Cow::Borrowed(&n.rules);
        let mut inline_errors = Vec::new();
        if n.rules.iter().any(|rule| {
            rule.declarations
                .iter()
                .any(|d| d.value.binding().is_some())
        }) {
            for rule in rules.to_mut() {
                rule.declarations = std::mem::take(&mut rule.declarations)
                    .into_iter()
                    .flat_map(|declaration| {
                        let Some(expression) = declaration.value.binding() else {
                            return vec![declaration];
                        };
                        let value = match b.eval(expression) {
                            Value::String(value) => value,
                            Value::Number(value) => value.to_string(),
                            Value::Bool(value) => value.to_string(),
                            _ => return Vec::new(),
                        };
                        let body = format!(
                            "{}: {value}{};",
                            declaration.name,
                            if declaration.important {
                                " !important"
                            } else {
                                ""
                            }
                        );
                        match super::stylesheet::inline(&b.doc, n, &body, 0) {
                            Ok(declarations) => declarations,
                            Err((_, error)) => {
                                inline_errors.push(format!("style:{}: {error}", declaration.name));
                                Vec::new()
                            }
                        }
                    })
                    .collect();
            }
        }
        r.cascade = cascade::values_with(rules.iter(), &options, &mut rule_applies);
        // The root's own `vi` and `vb` follow the writing mode it declares.
        let (basis, options) = if cascade::root_vertical(inherited).is_none()
            && cascade::declares_vertical(&r.cascade.values)
        {
            let basis = super::calc::Basis {
                vertical: true,
                ..basis
            };
            let options = cascade::Options {
                basis: Some(basis),
                ..options
            };
            r.cascade = cascade::values_with(rules.iter(), &options, &mut rule_applies);
            (basis, options)
        } else {
            (basis, options)
        };
        r.cascade.diagnostics.extend(inline_errors);
        // `@starting-style`: the before-change style of an element's first
        // style, for its transitions.
        let key = crate::id::key_of_path(path);
        let instant = !self.docs.state.entered.contains(&key)
            && sets
                .iter()
                .rev()
                .find(|p| p.name() == "dataOgInstant")
                .is_some_and(|p| {
                    matches!(
                        b.value(self, PROPS[p.spec].ty, &p.value),
                        Some(V::Bool(true))
                    )
                });
        if !instant
            && !self.docs.state.memory.seen.contains(&key)
            && n.rules.iter().any(|rule| rule.starting)
        {
            let start = cascade::values_with(
                rules.iter(),
                &cascade::Options {
                    starting: true,
                    carry: false,
                    ..options
                },
                &mut rule_applies,
            );
            r.starting = start
                .values
                .iter()
                .filter(|(name, value, _)| {
                    !r.cascade
                        .values
                        .iter()
                        .any(|(n, v, _)| n == name && v == value)
                })
                .cloned()
                .collect();
        }
        let motion: Vec<(String, String)> = r
            .cascade
            .values
            .iter()
            .filter(|(name, ..)| super::motion::longhand(name))
            .map(|(name, value, _)| {
                // Timeline insets and ranges in the element's units, as px.
                let value = if (name.starts_with("animation-range")
                    || name == "animation-timeline"
                    || name == "view-timeline-inset")
                    && super::calc::per_element(name, value)
                {
                    super::calc::absolute(value, basis, name).0
                } else {
                    value.clone()
                };
                (name.clone(), value)
            })
            .collect();
        (r.timelines, r.timeline_scope) = super::motion::timelines(&motion);

        (r.transitions, r.animations) =
            super::motion::resolve(&motion, &b.doc.name, &mut r.cascade.diagnostics);
        // A conditional `@keyframes` defines its name while its conditions
        // hold; choosing another definition restarts the animation, as a
        // name change does.
        if b.doc
            .sheet
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
                    .sheet
                    .keyframe_rules
                    .iter()
                    .rev()
                    .filter(|(n, ..)| n == name)
                    .find(|(_, q, _)| applies(q, &mut media_reads))
                    .map_or_else(|| format!("{name}@none"), |(.., key)| key.clone());
                a.name = format!("{prefix}{key}");
            }
        }
        // Transitions and animations of computed values, over the winners.
        r.style = self.animate_values(
            animate::StyleTarget::Element(key),
            &mut r.cascade,
            &r.starting,
            &r.transitions,
            &r.animations,
            basis,
            None,
        );
        let dir = attr("dir");
        r.flow = cascade::flow(
            &mut r.cascade,
            inherited,
            self.env.direction == og_i18n::Direction::Rtl,
            dir.as_deref(),
        );
        r.dir_auto = dir.is_some_and(|d| d.eq_ignore_ascii_case("auto"));
        cascade::route(&mut r.cascade, r.flow);
        if let Some((ty, names)) =
            super::container::declared(&r.cascade.container, &mut r.cascade.diagnostics)
        {
            let key = crate::id::key_of_path(path);
            super::container::push(
                Arc::make_mut(&mut r.cascade.computed),
                key,
                ty,
                &names,
                r.flow.mode.vertical(),
            );
            // Only size containers publish their box through settlement.
            if ty.size != super::container::Kind::Normal {
                r.container = Some(key);
            }
        }
        r.space3d = crate::transform3d::resolve(&r.cascade.transform, &mut r.cascade.diagnostics)
            .map(Arc::new);
        if !r.cascade.generated.is_empty() {
            let values = std::mem::take(&mut r.cascade.generated);
            if let Some((_, q)) = values.iter().find(|(name, _)| name == "quotes") {
                Arc::make_mut(&mut r.cascade.computed).insert(QUOTES.into(), Some(q.clone()));
            }
            r.generated = Some(Arc::new(super::generated::Generated::from_values(
                &values,
                &mut r.cascade.diagnostics,
            )));
            r.cascade.generated = values;
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
        for e in self.docs.state.edits.iter().filter(|e| e.path == path) {
            if let Some(v) = b.value(self, PROPS[e.spec].ty, &e.value) {
                r.vals.push((e.spec, v));
                built.edits.push((e.prop.clone(), e.source.clone()));
            }
        }
        r.part = Self::document_part(&r, n, b);
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
                    let (style, cascade) = super::spans::resolve(
                        &rules,
                        span,
                        &r.cascade.computed,
                        self.theme(),
                        self.fonts(),
                    );
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
                .map(|t| b.env.shown(t))
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
                .map(|t| b.env.shown(t))
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
                    let (style, cascade) = super::spans::resolve(
                        &rules,
                        &span,
                        &r.cascade.computed,
                        self.theme(),
                        self.fonts(),
                    );
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
                // HTML's UA sheet: `sub, sup { vertical-align: sub | super;
                // font-size: smaller }` (a size step of 1.2, as browsers take).
                if let Some(v) = run.valign {
                    style.vertical_align = style.vertical_align.or(Some(v));
                    style.size_scale = Some(1.0 / 1.2);
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
        media_reads.resolution |= [
            "backgroundImage",
            "maskImage",
            "borderImageSource",
            "maskBorderSource",
        ]
        .iter()
        .any(|name| {
            r.str(name).is_some_and(|value| {
                super::css::tokens(value).is_ok_and(|tokens| {
                    tokens.iter().any(|token| {
                        matches!(token, super::css::Tok::Func(name, _) if name == "image-set")
                    })
                })
            })
        });
        r.cascade.viewport_reads.extend(media_reads);
        if !container_read.sizes.is_empty()
            || !r.cascade.container_reads.is_empty()
            || !container_read.scroll.is_empty()
            || container_read
                .inherited
                .iter()
                .any(|e| e.starts_with("@cq:"))
        {
            // The chain is inherited state: a container appearing or leaving
            // above this element re-resolves it like a custom property would.
            r.cascade
                .inherited_reads
                .insert(super::container::CHAIN.to_string());
        }
        // `style()` reads its container's custom properties as inherited
        // values: a change there re-resolves this element.
        r.cascade.inherited_reads.extend(container_read.inherited);
        r.cascade.container_reads.extend(container_read.sizes);
        r.cascade.scroll_reads = container_read.scroll;
        if let NodeKind::Element(kind) = n.kind
            && kind.svg()
        {
            r.smil = svg::animate(self, &b.doc, n, path, &mut r);
            svg::inherit(kind, &mut r);
        }
        // Text style: written for those under it by an element declaring
        // it, read by an element that shows text.
        // Controls read accent-color and caret-color the same way.
        let shows_text = matches!(
            n.kind,
            NodeKind::Element(
                Kind::Label
                    | Kind::Button
                    | Kind::Field
                    | Kind::Toggle
                    | Kind::Slider
                    | Kind::Radio
                    | Kind::Progress
                    | Kind::Meter
                    | Kind::Select
                    | Kind::Color
            )
        );
        if shows_text || !r.cascade.text.is_empty() {
            let mut ramp = self.env.theme.type_ramp;
            let step = r
                .str("step")
                .and_then(|s| crate::style_file::ramp_step(&mut ramp, s).copied())
                .unwrap_or(self.env.theme.type_ramp.body);
            let current = r.color("color").unwrap_or(self.env.theme.palette.ink);
            let local = std::mem::take(&mut r.cascade.text);
            let cascade = &mut r.cascade;
            let text = super::text_css::inherit(super::text_css::Inputs {
                local: &local,
                computed: Arc::make_mut(&mut cascade.computed),
                reads: &mut cascade.inherited_reads,
                diagnostics: &mut cascade.diagnostics,
                theme: &self.env.theme,
                current,
                step,
                declared: &|name| self.style.family_declared(name, &self.env),
            });
            cascade.text = local;
            r.text = (text != Default::default()).then(|| Arc::new(text));
        }
        r.part_host = n.selector_tag.as_ref().map(|tag| {
            let mut computed = (*r.cascade.computed).clone();
            computed.properties.extend(
                r.cascade
                    .values
                    .iter()
                    .filter(|(name, ..)| cascade::inherited_property(name))
                    .map(|(name, value, _)| (name.clone(), value.clone())),
            );
            // A button is its plate: what its author writes for its box
            // (`button.buy { background: … }`) restyles the plate part
            // that covers it, as CSS on an HTML button restyles the button.
            // Its padding goes with it, so the plate is the button's whole
            // box; the button keeps its corners for what it draws itself
            // (the `overflow` clip, the outline).
            let mut plate = Vec::new();
            if n.kind == NodeKind::Element(Kind::Button) {
                let (own, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut r.vals)
                    .into_iter()
                    .partition(|(spec, _)| plate_prop(PROPS[*spec].name));
                r.vals = rest;
                r.vals.extend(
                    own.iter()
                        .filter(|(spec, _)| corner_prop(PROPS[*spec].name))
                        .cloned(),
                );
                plate = own;
                // Its fill replaces the style's whole background and its
                // border the style's painted frame, as the `background`
                // and `border` shorthands reset the image beside them.
                let has = |plate: &[(usize, V)], name: &str| {
                    plate.iter().any(|(spec, _)| PROPS[*spec].name == name)
                };
                let border = plate.iter().any(|(spec, _)| {
                    let name = PROPS[*spec].name;
                    name.starts_with("border") && !name.starts_with("borderImage")
                });
                for (written, image) in [
                    (has(&plate, "background"), "backgroundImage"),
                    (border, "borderImageSource"),
                ] {
                    if written
                        && !has(&plate, image)
                        && let Some((spec, _)) = schema::prop(image)
                    {
                        plate.push((spec, V::Str("none".into())));
                    }
                }
            }
            Arc::new(PartHost {
                document: b.doc.name.clone(),
                tag: tag.clone(),
                attributes: structure::selector_facts(n, |e| b.eval(e)),
                computed: Arc::new(computed),
                plate,
            })
        });
        // Its box pseudo-elements inherit from it and match as it does
        // (their selectors' subject is this element); what they read
        // beyond it is read through it.
        for &(which, at) in &n.pseudo {
            let inherited = r.cascade.computed.clone();
            let p = self.resolve(
                b,
                &b.doc.nodes[at],
                &format!("{path}/::{which}"),
                &mut Built::default(),
                &inherited,
                structure,
            );
            let c = &mut r.cascade;
            c.inherited_reads.extend(
                p.cascade
                    .inherited_reads
                    .iter()
                    .filter(|name| !c.local_names.contains(*name))
                    .cloned(),
            );
            c.reads.extend(p.cascade.reads.iter().cloned());
            c.viewport_reads.extend(p.cascade.viewport_reads);
            c.container_reads.extend(&p.cascade.container_reads);
            c.scroll_reads.extend(&p.cascade.scroll_reads);
            c.diagnostics.extend(
                p.cascade
                    .diagnostics
                    .iter()
                    .map(|d| format!("::{which}: {d}")),
            );
            r.pseudo.push((which, at, Arc::new(p)));
        }
        if let Some(t) = r.str("positionTry") {
            let named: Vec<String> = t
                .split([',', ' '])
                .filter(|w| w.starts_with("--"))
                .map(str::to_string)
                .collect();
            r.position_tries = b
                .doc
                .sheet
                .position_tries
                .iter()
                .filter(|(n, _)| named.contains(n))
                .map(|(n, decls)| {
                    let p = crate::anchor::placement_of(
                        decls.iter().map(|d| (d.name.as_str(), d.source.as_str())),
                    );
                    (n.clone(), p)
                })
                .collect();
        }
        r
    }
}

impl crate::ui::Style {
    /// Whether a family is installed, rather than merely declared in CSS.
    pub(crate) fn family_declared(&self, name: &str, env: &crate::ui::Env) -> bool {
        env.fonts
            .families
            .iter()
            .any(|f| f.family.eq_ignore_ascii_case(name))
    }
}

/// A document's timeline by its full name (`shop/open`).
pub fn qualify(doc: &str, tl: &str) -> String {
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
            crate::slot::Fit::None => "none",
            crate::slot::Fit::ScaleDown => "scaleDown",
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

mod animate;
pub(crate) use animate::StyleCommit;
pub mod cascade;
pub(crate) mod color;
pub(crate) mod containers;
pub(crate) mod datetime;
#[cfg(test)]
mod ext;
#[doc(hidden)]
pub mod fields;
pub(crate) mod generated;
pub(crate) mod gestures;
#[doc(hidden)]
pub mod host;
#[doc(hidden)]
pub mod instances;
pub(crate) mod kbd;
mod layout_props;
pub(crate) mod library;
pub(crate) mod media;
pub(crate) mod mount;
pub(crate) mod nodes;
mod parts;
pub(crate) use parts::PartIndex;
pub(crate) mod popover;
pub(crate) mod retained;
pub(crate) mod state;
#[doc(hidden)]
pub mod structure;
#[doc(hidden)]
pub mod svg;
pub(crate) mod text_motion;
pub(crate) mod touch;
pub(crate) mod viewport;
pub(crate) mod world;
pub use host::{Built, DocEvent, DocMemory, Edit};
pub use instances::RetainedHost;

/// The computed-value entry carrying the inherited `quotes`.
pub const QUOTES: &str = "@quotes";
