//! A mounted document as the new core runs it: its elements made once in an
//! [`Arena`], each binding an effect of the update queue ([`crate::queue`])
//! that patches only the node it names. A frame drains the tree's ops,
//! refolds the records of the elements whose conditions changed (and the
//! text their subtree inherits), lays out what is dirty and paints again
//! only when something changed. A frame with nothing queued does no work
//! and allocates nothing.

use crate::arena::{Arena, Id, Run};
use crate::doc::DocStyle;
use crate::look::Look;
use crate::queue::{self, Op, Reads, Target};
use crate::style::{Record, Rgba, Text};
use og_draw::paint::{Paint, PaintKind, PaintQuad};
use og_reactive::bind::{Env, Expr, Scope, Screen};
use og_ui::doc::{Act, Lit, NodeKind, PropValue, UiDocument, When, schema::Kind};
use og_ui::{IconAtlas, IconCell};
use rustc_hash::FxHashMap;
use serde_json::Value;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

const NONE: u32 = u32::MAX;

/// A host's string tables: a key's message (og.i18n), `None` when absent.
pub type Strings = Box<dyn Fn(&str) -> Option<String>>;

/// What the bindings read beside the stores: the UI's box and input, which
/// `:size()` and `:touch` read, and the host's locale and string tables,
/// which `t()` reads.
pub struct Surface {
    pub size: Cell<(f32, f32)>,
    pub touch: Cell<bool>,
    pub language: String,
    pub strings: Option<Strings>,
}

impl Default for Surface {
    fn default() -> Self {
        Surface {
            size: Cell::default(),
            touch: Cell::default(),
            language: "en".into(),
            strings: None,
        }
    }
}

impl Env for Surface {
    fn screen(&self, class: Screen) -> bool {
        let (w, h) = self.size.get();
        class.holds(w, h, self.touch.get())
    }
    fn language(&self) -> &str {
        &self.language
    }
    fn translate(&self, key: &str) -> Option<String> {
        self.strings.as_ref()?(key)
    }
}

/// A document compiled against the look.
struct Compiled {
    doc: Arc<UiDocument>,
    style: DocStyle,
    /// A document node's index in `style.nodes`.
    style_of: Vec<u32>,
    /// A condition reads no store (`:size()`, `:dir()`): it is evaluated
    /// when its element's record is folded.
    fixed: Vec<bool>,
    /// The scrollers a condition asks `:state(more)` of: (condition,
    /// document node).
    more: Vec<(u32, u32)>,
}

/// One instance of a document.
struct Inst {
    doc: u32,
    /// The mount it is part of, in mounting order.
    mount: u32,
    /// Its scrollers (document nodes) whose content runs past their end.
    more: Vec<u32>,
}

/// The effective look an element passes to its children.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Inherit {
    size: f32,
    ink: Rgba,
    text: Text,
    opacity: f32,
}

struct El {
    node: Id,
    /// Its document node.
    at: u32,
    /// The key of the innermost `{#each}` row it is in; empty outside one.
    row: Rc<str>,
    /// Its `disabled` attribute as a signal; NONE when it has none.
    disabled: u32,
    /// It answers a press: a button, or an element with a click handler.
    clickable: bool,
    /// The renderer's id of an op node ([`ops`]); NONE for a document's.
    js: u32,
    /// An op node's events (bits by [`ops::Event`]) and its class entry.
    listens: u32,
    class: u32,
    /// The element its arena node is a child of (a fixed one: none).
    arena_parent: u32,
    /// The element it inherits from.
    parent: u32,
    inst: u32,
    style: u32,
    kind: Kind,
    leaf: Option<Id>,
    /// Its text, a text leaf's or an icon's name: rewritten in place, so a
    /// new text no longer than its capacity allocates nothing.
    text: String,
    hidden: u16,
    dead: bool,
    queued: bool,
    record: Record,
    /// The store conditions it holds, bits by condition index: its own, as
    /// each row of an `{#each}` sets its own of the same rule.
    holds: Box<[u64]>,
    eff: Inherit,
    /// A `<Line>`'s points and how they map onto its box; `None` for any
    /// other node.
    line: Option<Box<Line>>,
    /// Where an op node with `anchor` sits in the world, and where the host
    /// last placed it; `None` for one in flow.
    anchor: Option<Box<Anchor>>,
    /// An op node's own image props over its classes' box image; `None`
    /// for a node that sets none.
    image: Option<Box<ImageProps>>,
    /// An op node's `data-*` props that are on (names without `data-`,
    /// sorted), and its data context's entry ([`ops::Classes::data_id`]):
    /// these and its nearest groups', which its classes' `data-*:` and
    /// `group-data-*:` variants read; 0 none.
    attrs: Vec<Box<str>>,
    data: u32,
}

/// What an anchored node sits on (CONVENTIONS.md §World).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnchorTarget {
    /// An entity, by its bits.
    Entity(u64),
    Point([f32; 3]),
}

/// An anchored node: its target and world offset as its props set them,
/// and its top left on the surface and whether it shows as
/// [`Tree::place_anchors`] last placed it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Anchor {
    pub(crate) target: AnchorTarget,
    pub(crate) lift: [f32; 3],
    pub(crate) at: [f32; 2],
    pub(crate) shown: bool,
}

/// A `<Line>`'s own props (CONVENTIONS.md §Ops).
#[derive(Clone, Debug, Default)]
pub(crate) struct Line {
    /// x, y pairs: CSS px from the box's top left, or units of `view`.
    pub(crate) points: Vec<f32>,
    pub(crate) closed: bool,
    /// `viewBox` as SVG's min-x, min-y, width and height: the points'
    /// space, which is mapped onto the laid-out box as it paints; `None` px.
    pub(crate) view: Option<[f32; 4]>,
    /// `fit="contain"`: the view scales uniformly and is centred in the box
    /// (SVG's `meet`); otherwise it stretches to fill it.
    pub(crate) contain: bool,
}

impl Line {
    /// The points in the box's px, for a box `w` by `h`.
    fn mapped(&self, w: f32, h: f32) -> impl Iterator<Item = [f32; 2]> + '_ {
        let (s, o) = match self.view {
            Some([vx, vy, vw, vh]) if vw > 0.0 && vh > 0.0 => {
                let (sx, sy) = (w / vw, h / vh);
                let (s, pad) = if self.contain {
                    let k = sx.min(sy);
                    ([k, k], [(w - vw * k) / 2.0, (h - vh * k) / 2.0])
                } else {
                    ([sx, sy], [0.0, 0.0])
                };
                (s, [pad[0] - vx * s[0], pad[1] - vy * s[1]])
            }
            _ => ([1.0, 1.0], [0.0, 0.0]),
        };
        self.points
            .as_chunks::<2>()
            .0
            .iter()
            .map(move |&[x, y]| [o[0] + x * s[0], o[1] + y * s[1]])
    }
}

/// What an element's props set of its box image (CONVENTIONS.md §Images):
/// its source (TSX `src`, .svelte `<img texture>`) and the radial wipe
/// bound per tick, so a moving share never makes a class string.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ImageProps {
    /// The source: the texture handle as the game holds it, which
    /// [`Tree::set_textures`] resolves as it paints; NaN none.
    pub(crate) handle: f64,
    pub(crate) wipe: Option<f32>,
    pub(crate) wipe_start: Option<f32>,
    pub(crate) wipe_ccw: Option<bool>,
}

impl ImageProps {
    /// The box image `base` (its classes') with these props over it, the
    /// source's slot from `textures` (none: 0).
    fn over(&self, base: &crate::style::Slice, textures: Option<&Textures>) -> crate::style::Slice {
        let mut im = *base;
        if !self.handle.is_nan() {
            im.texture = textures.map_or(0, |f| f(self.handle));
        }
        im.wipe = self.wipe.unwrap_or(im.wipe);
        im.wipe_start = self.wipe_start.unwrap_or(im.wipe_start);
        im.wipe_ccw = self.wipe_ccw.unwrap_or(im.wipe_ccw);
        im
    }
}

struct IfBlock {
    tops: Vec<u32>,
    shown: bool,
}

struct EachBlock {
    doc: u32,
    inst: u32,
    node: usize,
    parent: u32,
    arena_parent: Id,
    scope: Vec<(Box<str>, u32)>,
    chain: Vec<u32>,
    list: u32,
    rows: Vec<(u32, Vec<u32>)>,
}

/// Something an element did that its document reports to the game
/// (`og.ui.docEvents`): `click` with its id, or `emit` with the handler
/// an `on:` attribute names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    /// The mount, in mounting order.
    pub mount: u32,
    /// The element's document id.
    pub path: Box<str>,
    pub row_key: Rc<str>,
    pub kind: &'static str,
    pub name: Box<str>,
}

/// What a frame did: whether it laid out and painted, and its ops.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Frame {
    pub ops: usize,
    pub refolded: usize,
    pub laid_out: bool,
    pub painted: bool,
}

pub struct Tree {
    id: u32,
    look: Rc<Look>,
    env: Rc<Surface>,
    docs: Vec<Compiled>,
    by_name: FxHashMap<Box<str>, u32>,
    library: FxHashMap<Box<str>, Arc<UiDocument>>,
    /// Documents' styles as the cook compiled them ([`crate::cooked`]),
    /// by document source.
    cooked: FxHashMap<Box<str>, Rc<[u8]>>,
    insts: Vec<Inst>,
    els: Vec<El>,
    ifs: Vec<IfBlock>,
    eaches: Vec<EachBlock>,
    arena: Arena,
    root: Inherit,
    ops: Vec<Op>,
    /// The size class as a signal ([`Screen`] bits), which the bindings
    /// that read `:size()` or `:touch` track.
    screen: u32,
    dirty: Vec<u32>,
    paint: bool,
    /// The HUD's safe-area insets (top, right, bottom, left) a record's
    /// `env(hud-inset-*)` adds at layout ([`Tree::set_insets`]).
    insets: [f32; 4],
    /// What it paints (og-draw's arena, drawn by og-render's paint
    /// pass), refilled in place: its generation moves each repaint.
    pub drawn: Paint,
    /// Each arena node's semantics for the accessibility tree, written by
    /// the op applier ([`crate::a11y`]).
    pub sem: crate::a11y::Semantics,
    /// Each element's place on screen, whether it shows, and the clip its
    /// children paint in (x0, y0, x1, y1).
    abs: Vec<(f32, f32, bool, [f32; 4])>,
    /// The elements with an anchor, in no order (a dead or unanchored one
    /// leaves at the next [`Tree::place_anchors`]).
    anchored: Vec<u32>,
    /// Each element's text laid out as glyph quads from its leaf's top
    /// left, kept while its text, width and run stay.
    glyphs: Vec<Glyphs>,
    /// The faces text was measured and laid out in (the body's, then its
    /// fallbacks'): when a locale links others, every text is again.
    faces: Vec<Option<og_draw::FontHandle>>,
    /// Characters text was laid out with that no face has, each once,
    /// until [`Tree::take_missing`]: the host loads a font pack that
    /// covers them (docs/ui.md §3.4), and the text draws in it once linked.
    missing: Vec<char>,
    /// The lines [`lay_out`] breaks a text into, kept between texts.
    breaks: Vec<(std::ops::Range<usize>, f32)>,
    /// Each element's transform in `drawn.transforms` (its own composed
    /// with its ancestors'; 0 none), what its children compose onto.
    xfs: Vec<u32>,
    /// The host's: an image's source (a texture handle) as the slot it
    /// draws, read as the tree paints, so a replaced texture follows.
    textures: Option<Textures>,
    /// The mount being made and the key of the row being made.
    mounting: u32,
    row: Rc<str>,
    /// The element the pointer went down on, while it is held.
    pressed: u32,
    /// The live elements depth first, a parent before its children (in
    /// paint order), made again when the structure changes; `el_of` maps
    /// an arena node to its element, `walk` is its stack.
    order: Vec<u32>,
    ordered: bool,
    el_of: Vec<u32>,
    walk: Vec<Id>,
    /// The op nodes: renderer id to element (NONE: free), their classes'
    /// records once per distinct class and style, and the events since the
    /// renderer last took them ([`ops`]).
    by_js: Vec<u32>,
    classes: ops::Classes,
    pub events: Vec<ops::Fired>,
    /// A batch changed a `data-*` prop, a class or the op tree's shape:
    /// the data contexts are worked out again after it.
    data_dirty: bool,
    /// The string tables op batches named (op 12), for the host to
    /// register ([`Tree::take_strings`]).
    strings: Vec<String>,
    /// What moves, keyed by arena node ([`crate::motion`]), at the UI
    /// tick's `time`; `shown` is what each element passes down this frame
    /// as motion shows it ([`Shown`]).
    motion: crate::motion::Motion,
    time: f64,
    shown: Vec<Shown>,
    /// Op nodes kept drawn while their exit runs: removed (false) or
    /// released too (true); `gone` is the settled exits' scratch list.
    leaving: Vec<(u32, bool)>,
    gone: Vec<Id>,
    /// Op nodes' hover, press, focus and modals ([`input`]), and where
    /// the pointer was in the last frame and how many frames had painted
    /// then (`paints` counts them).
    input: input::Input,
    last_pointer: (f32, f32, u64),
    paints: u64,
}

/// What an element passes down in a frame, as motion shows it: its
/// opacity times its parent's, its ink (its own, else its parent's), and
/// whether it is in an exit, which nothing hits.
#[derive(Clone, Copy)]
struct Shown {
    alpha: f32,
    ink: Rgba,
    out: bool,
}

/// An element's text as glyph quads (from its leaf's top left; colour and
/// clip set as they are copied), and what they were laid out from.
#[derive(Default)]
struct Glyphs {
    text: String,
    key: [u32; 5],
    quads: Vec<PaintQuad>,
    /// Each line's start x, baseline and width, for its decoration.
    lines: Vec<[f32; 3]>,
    /// The lines' texts as laid out, each ended by `\n`: what the probe's
    /// dump checks for a word broken where UAX #14 has no break.
    line_text: String,
}

mod input;
mod ops;
pub use input::{Cue, Play, SoundTable};
pub use ops::{EVENTS, Event as OpEvent, Fired, MAGIC, REFUSED, Resolve};

/// An image's source, a texture handle as the game holds it, as its
/// texture's slot (`og_draw::TextureHandle::slot`; 0 none)
/// ([`Tree::set_textures`]).
pub type Textures = Box<dyn Fn(f64) -> u32>;

static NEXT_TREE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

fn lit(l: &Lit) -> Value {
    match l {
        Lit::Str(s) => s.as_str().into(),
        Lit::Num(n) => serde_json::Number::from_f64(*n as f64).map_or(Value::Null, Value::Number),
        Lit::Bool(b) => (*b).into(),
        Lit::Strs(v) => v.iter().map(|s| Value::from(s.as_str())).collect(),
        _ => Value::Null,
    }
}

/// Whether `e` reads the UI's box or input (`:size()`, `:touch`).
fn reads_screen(e: &Expr) -> bool {
    match e {
        Expr::Screen(_) => true,
        Expr::Not(a) | Expr::Neg(a) | Expr::Format(_, a) | Expr::Numbers(a) => reads_screen(a),
        Expr::Bin(_, a, b) | Expr::Index(a, b) => reads_screen(a) || reads_screen(b),
        Expr::If(a, b, c) => reads_screen(a) || reads_screen(b) || reads_screen(c),
        _ => false,
    }
}

/// The signals of `scope` an expression reads, by their root name, and
/// `screen` (the size class) when it reads the UI's box: a binding of
/// `.step:state(current):size(compact)` runs again when the class moves.
fn reads(e: &Expr, scope: &[(Box<str>, u32)], screen: u32) -> Reads {
    let mut paths = Vec::new();
    e.paths(&mut paths);
    let mut out: Reads = Vec::new();
    for p in paths {
        let root = p.split('.').next().unwrap_or("");
        if out.iter().any(|(n, _)| &**n == root) {
            continue;
        }
        if let Some((n, s)) = scope.iter().rev().find(|(n, _)| &**n == root) {
            out.push((n.clone(), *s));
        }
    }
    if reads_screen(e) {
        // A name no path takes: the value is read only to be tracked.
        out.push(("\u{0}screen".into(), screen));
    }
    out
}

/// `e` over `reads`, as a derived signal.
fn derive(e: Expr, reads: Reads, env: Rc<Surface>) -> u32 {
    queue::derive(move || queue::eval(&e, &reads, &*env))
}

/// What a fixed condition reads: the surface, and the instance's
/// scrollers with more below.
struct Fixed<'a> {
    surface: &'a Surface,
    more: &'a [u32],
}

impl og_reactive::bind::Env for Fixed<'_> {
    fn screen(&self, class: Screen) -> bool {
        self.surface.screen(class)
    }
    fn language(&self) -> &str {
        &self.surface.language
    }
    fn interaction_at(&self, node: usize, state: og_reactive::bind::InteractionState) -> bool {
        state == og_reactive::bind::InteractionState::More && self.more.contains(&(node as u32))
    }
}

fn eval_static(e: &Expr, env: &Fixed) -> bool {
    og_reactive::bind::truthy(&e.eval(&Scope {
        vm: &Value::Null,
        locals: Vec::new(),
        env,
    }))
}

/// The scroller nodes `e` asks `:state(more)` of.
fn more_of(e: &Expr, out: &mut Vec<u32>) {
    match e {
        Expr::InteractionAt {
            node,
            state: og_reactive::bind::InteractionState::More,
        } => out.push(*node as u32),
        Expr::Not(a) | Expr::Neg(a) => more_of(a, out),
        Expr::Bin(_, a, b) => {
            more_of(a, out);
            more_of(b, out);
        }
        Expr::If(a, b, c) => {
            more_of(a, out);
            more_of(b, out);
            more_of(c, out);
        }
        _ => {}
    }
}

impl Tree {
    /// Gives the bindings the host's locale and string tables. Call it
    /// before mounting: a binding reads them when it runs.
    pub fn set_strings(&mut self, language: &str, strings: Strings) {
        let env = Rc::get_mut(&mut self.env).expect("set before anything is mounted");
        env.language = language.into();
        env.strings = Some(strings);
    }

    /// How an image's source resolves to its texture (the host's assets).
    pub fn set_textures(&mut self, textures: Textures) {
        self.textures = Some(textures);
        self.paint = true;
    }

    /// The textures [`Tree::set_textures`] resolves against changed: the
    /// next frame paints its images again.
    pub fn repaint_images(&mut self) {
        self.paint = true;
    }

    /// An empty tree over `look`, with the documents it may use.
    pub fn new(look: Rc<Look>, library: impl IntoIterator<Item = Arc<UiDocument>>) -> Tree {
        let mut root = look.root.clone();
        root.text.set = 0;
        Tree {
            id: NEXT_TREE.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            env: Rc::new(Surface::default()),
            root: Inherit {
                size: root.font_size,
                ink: root.skin.ink,
                text: root.text,
                opacity: 1.0,
            },
            look,
            docs: Vec::new(),
            by_name: FxHashMap::default(),
            library: library
                .into_iter()
                .map(|d| (d.name.as_str().into(), d))
                .collect(),
            cooked: FxHashMap::default(),
            insts: Vec::new(),
            els: Vec::new(),
            ifs: Vec::new(),
            eaches: Vec::new(),
            arena: Arena::new(),
            ops: Vec::new(),
            screen: queue::signal(0.into()),
            dirty: Vec::new(),
            paint: true,
            insets: [0.0; 4],
            drawn: Paint::default(),
            sem: crate::a11y::Semantics::default(),
            abs: Vec::new(),
            anchored: Vec::new(),
            glyphs: Vec::new(),
            xfs: Vec::new(),
            textures: None,
            mounting: 0,
            row: "".into(),
            pressed: NONE,
            order: Vec::new(),
            ordered: false,
            el_of: Vec::new(),
            walk: Vec::new(),
            by_js: Vec::new(),
            classes: ops::Classes::default(),
            events: Vec::new(),
            data_dirty: false,
            strings: Vec::new(),
            motion: crate::motion::Motion::new(),
            time: 0.0,
            shown: Vec::new(),
            leaving: Vec::new(),
            gone: Vec::new(),
            input: input::Input::new(),
            last_pointer: (f32::NAN, f32::NAN, 0),
            paints: 0,
            faces: Vec::new(),
            missing: Vec::new(),
            breaks: Vec::new(),
        }
    }

    /// The cook's compiled style of document `source` ([`crate::cooked`]):
    /// taken when it was compiled against this tree's look and parse.
    pub fn set_cooked(&mut self, source: &str, bytes: Rc<[u8]>) {
        self.cooked.insert(source.into(), bytes);
    }

    pub fn len(&self) -> usize {
        self.els.len()
    }

    pub fn is_empty(&self) -> bool {
        self.els.is_empty()
    }

    fn compiled(&mut self, name: &str) -> Option<u32> {
        if let Some(&d) = self.by_name.get(name) {
            return Some(d);
        }
        let doc = self.library.get(name)?.clone();
        // The cook's compile of it against this look, else one now.
        let style = self
            .cooked
            .get(doc.source.as_str())
            .and_then(|b| crate::cooked::decode(b, doc.nodes.len(), self.look.key))
            .unwrap_or_else(|| DocStyle::compile(&doc, self.look.tokens(), self.look.surfaces()));
        let mut style_of = vec![NONE; doc.nodes.len()];
        for (k, n) in style.nodes.iter().enumerate() {
            style_of[n.node as usize] = k as u32;
        }
        let fixed = style
            .conditions
            .iter()
            .map(|c| {
                let mut p = Vec::new();
                c.paths(&mut p);
                p.is_empty()
            })
            .collect();
        let mut more = Vec::new();
        for (k, c) in style.conditions.iter().enumerate() {
            let mut nodes = Vec::new();
            more_of(c, &mut nodes);
            more.extend(nodes.into_iter().map(|n| (k as u32, n)));
        }
        self.docs.push(Compiled {
            doc,
            style,
            style_of,
            fixed,
            more,
        });
        let d = self.docs.len() as u32 - 1;
        self.by_name.insert(name.into(), d);
        Some(d)
    }

    /// Mounts document `name` with its inputs (prop name to store signal).
    pub fn mount(&mut self, name: &str, inputs: &[(&str, u32)]) -> Result<(), String> {
        let mut scope: Vec<(Box<str>, u32)> = Vec::new();
        for (n, s) in inputs {
            scope.push(((*n).into(), *s));
            scope.push((format!("${n}").into(), *s));
        }
        self.row = "".into();
        self.instance(name, NONE, 0, scope, &mut Vec::new())
            .ok_or_else(|| format!("og-ui-next: no document {name}"))?;
        self.mounting += 1;
        Ok(())
    }

    fn instance(
        &mut self,
        name: &str,
        parent: u32,
        arena_parent: Id,
        mut scope: Vec<(Box<str>, u32)>,
        chain: &mut Vec<u32>,
    ) -> Option<()> {
        let d = self.compiled(name)?;
        let inst = self.insts.len() as u32;
        self.insts.push(Inst {
            doc: d,
            mount: self.mounting,
            more: Vec::new(),
        });
        // The document's own derived values, in order.
        let derived = self.docs[d as usize].doc.derived.clone();
        for (n, e) in derived {
            let r = reads(&e, &scope, self.screen);
            let s = derive(e, r, self.env.clone());
            scope.push((n.into(), s));
        }
        let root = self.docs[d as usize].doc.root;
        self.node(d, inst, root, parent, arena_parent, &scope, chain);
        Some(())
    }

    #[allow(clippy::too_many_arguments)]
    fn node(
        &mut self,
        d: u32,
        inst: u32,
        i: usize,
        parent: u32,
        arena_parent: Id,
        scope: &[(Box<str>, u32)],
        chain: &mut Vec<u32>,
    ) {
        let Some(n) = self.docs[d as usize].doc.nodes.get(i) else {
            return;
        };
        let kind = n.kind.clone();
        let children = n.children.clone();
        match kind {
            NodeKind::Element(k) => self.element(d, inst, i, k, parent, arena_parent, scope, chain),
            NodeKind::If(e) => {
                let b = self.ifs.len() as u32;
                self.ifs.push(IfBlock {
                    tops: Vec::new(),
                    shown: true,
                });
                chain.push(b);
                for c in children {
                    self.node(d, inst, c, parent, arena_parent, scope, chain);
                }
                chain.pop();
                let r = reads(&e, scope, self.screen);
                queue::bind(
                    self.id,
                    Target::Shown { node: b },
                    Rc::new(e),
                    r,
                    self.env.clone(),
                );
            }
            NodeKind::For {
                var,
                index,
                list,
                key,
                ..
            } => {
                let r = reads(&list, scope, self.screen);
                let list_sig = derive(list, r, self.env.clone());
                let b = self.eaches.len() as u32;
                self.eaches.push(EachBlock {
                    doc: d,
                    inst,
                    node: i,
                    parent,
                    arena_parent,
                    scope: scope.to_vec(),
                    chain: chain.clone(),
                    list: list_sig,
                    rows: Vec::new(),
                });
                self.rows(b, &var, index.as_deref(), &key);
                let keys = Self::keys_of(var.clone(), key, list_sig, self.env.clone());
                queue::bind(
                    self.id,
                    Target::Rebuild { node: b },
                    Rc::new(Expr::Path("keys".into())),
                    vec![("keys".into(), keys)],
                    self.env.clone(),
                );
            }
            NodeKind::Use(name) => {
                let Some(cd) = self.compiled(&name) else {
                    return;
                };
                let props = self.docs[cd as usize].doc.props.clone();
                let args = self.docs[d as usize].doc.nodes[i].args.clone();
                let mut inner: Vec<(Box<str>, u32)> = Vec::new();
                for p in props {
                    let arg = args
                        .iter()
                        .find(|(n, _, _)| *n == p.name)
                        .map(|(_, v, _)| v);
                    let s = match arg {
                        Some(PropValue::Bind(e, _)) => {
                            let r = reads(e, scope, self.screen);
                            derive(e.clone(), r, self.env.clone())
                        }
                        Some(PropValue::Lit(l)) => queue::signal(lit(l)),
                        _ => queue::signal(p.default.clone().unwrap_or(Value::Null)),
                    };
                    inner.push((p.name.into(), s));
                }
                self.instance(&name, parent, arena_parent, inner, chain);
            }
            NodeKind::Key(_) => {
                for c in children {
                    self.node(d, inst, c, parent, arena_parent, scope, chain);
                }
            }
            NodeKind::Snippet(_) | NodeKind::Render { .. } => {}
        }
    }

    /// The keys of a list signal's rows, as a derived signal.
    fn keys_of(var: String, key: Expr, list: u32, env: Rc<Surface>) -> u32 {
        queue::derive(move || {
            let Value::Array(items) = queue::read(list) else {
                return Value::Array(Vec::new());
            };
            items
                .into_iter()
                .map(|item| {
                    key.eval(&Scope {
                        vm: &Value::Null,
                        locals: vec![(var.as_str(), item)],
                        env: &*env,
                    })
                })
                .collect()
        })
    }

    /// Makes each row of list block `b`, its row a derived signal of the
    /// list by key, under an owner of its own.
    fn rows(&mut self, b: u32, var: &str, index: Option<&str>, key: &Expr) {
        let e = &self.eaches[b as usize];
        let (d, inst, node, parent, arena_parent, list) =
            (e.doc, e.inst, e.node, e.parent, e.arena_parent, e.list);
        let (mounting, outer_row) = (self.mounting, self.row.clone());
        self.mounting = self.insts[inst as usize].mount;
        let scope = e.scope.clone();
        let mut chain = e.chain.clone();
        let Value::Array(items) = queue::get(list) else {
            return;
        };
        let children = self.docs[d as usize].doc.nodes[node].children.clone();
        for (at, item) in items.into_iter().enumerate() {
            let k = key.eval(&Scope {
                vm: &Value::Null,
                locals: vec![(var, item)],
                env: &*self.env,
            });
            let first = self.els.len() as u32;
            self.row = queue::text_of(&k).into();
            let mut row_scope = scope.clone();
            let root = queue::owned(|| {
                let (var2, key2, env) = (var.to_string(), key.clone(), self.env.clone());
                // Its item found in the list in place; only it is copied.
                let row = queue::derive(move || {
                    queue::read_with(list, |items| {
                        let Value::Array(items) = items else {
                            return Value::Null;
                        };
                        items
                            .iter()
                            .find(|item| {
                                key2.eval(&Scope {
                                    vm: &Value::Null,
                                    locals: vec![(var2.as_str(), (*item).clone())],
                                    env: &*env,
                                }) == k
                            })
                            .cloned()
                            .unwrap_or(Value::Null)
                    })
                });
                row_scope.push((var.into(), row));
                if let Some(ix) = index {
                    row_scope.push((ix.into(), queue::signal(at.into())));
                }
                for &c in &children {
                    self.node(d, inst, c, parent, arena_parent, &row_scope, &mut chain);
                }
            });
            let tops = (first..self.els.len() as u32)
                .filter(|&e| {
                    self.els[e as usize].arena_parent == parent
                        || self.els[e as usize].arena_parent == NONE
                })
                .collect();
            self.eaches[b as usize].rows.push((root, tops));
        }
        self.mounting = mounting;
        self.row = outer_row;
    }

    #[allow(clippy::too_many_arguments)]
    fn element(
        &mut self,
        d: u32,
        inst: u32,
        i: usize,
        kind: Kind,
        parent: u32,
        arena_parent: Id,
        scope: &[(Box<str>, u32)],
        chain: &[u32],
    ) {
        let style = self.docs[d as usize].style_of[i];
        let e = self.els.len() as u32;
        let record = self.fold(inst, style, &[]);
        let (arena_at, arena_parent_el) = if record.fixed {
            (0, NONE)
        } else {
            (arena_parent, parent)
        };
        let node = self.arena.element(arena_at, record.layout.clone());
        let n = &self.docs[d as usize].doc.nodes[i];
        let prop = |name: &str| {
            n.props
                .iter()
                .find(|p| og_ui::doc::schema::PROPS[p.spec].name == name)
                .map(|p| p.value.clone())
        };
        let text = match kind {
            Kind::Label | Kind::Kbd | Kind::Button => prop("text"),
            Kind::Icon => prop("icon"),
            // An image's source: its box image's ([`Tree::set_text`]).
            Kind::Image => prop("texture"),
            _ => None,
        };
        let leaf = (text.is_some() && !matches!(kind, Kind::Icon | Kind::Image))
            .then(|| self.arena.text(node, "", 16.0));
        let children = n.children.clone();
        let clickable = kind == Kind::Button
            || n.triggers
                .iter()
                .any(|t| matches!(t.when, When::Click | When::Tap));
        let disabled = match prop("disabled") {
            Some(PropValue::Bind(x, _)) => {
                let r = reads(&x, scope, self.screen);
                derive(x, r, self.env.clone())
            }
            Some(PropValue::Lit(l)) => queue::signal(lit(&l)),
            _ => NONE,
        };
        self.els.push(El {
            node,
            at: i as u32,
            row: self.row.clone(),
            disabled,
            clickable,
            js: NONE,
            listens: 0,
            class: NONE,
            arena_parent: arena_parent_el,
            parent,
            inst,
            style,
            kind,
            leaf,
            text: String::new(),
            hidden: 0,
            dead: false,
            queued: false,
            record,
            holds: Box::default(),
            line: None,
            anchor: None,
            attrs: Vec::new(),
            data: 0,
            image: None,
            eff: self.root,
        });
        self.ordered = false;
        for &b in chain.iter() {
            self.ifs[b as usize].tops.push(e);
        }
        self.inherit(e);
        match text {
            Some(PropValue::Lit(l)) => self.set_text(e, &queue::text_of(&lit(&l))),
            Some(PropValue::Bind(x, _)) => {
                let r = reads(&x, scope, self.screen);
                queue::bind(
                    self.id,
                    Target::Text { node: e },
                    Rc::new(x),
                    r,
                    self.env.clone(),
                );
            }
            _ => {}
        }
        // Its conditions that read stores, each a binding.
        if style != NONE {
            let c = &self.docs[d as usize];
            let [at, len] = c.style.nodes[style as usize].spans;
            let conds: Vec<(u32, Rc<Expr>)> = c.style.spans[at as usize..(at + len) as usize]
                .iter()
                .filter(|s| s.condition != u32::MAX && !c.fixed[s.condition as usize])
                .map(|s| {
                    (
                        s.condition,
                        c.style.conditions[s.condition as usize].clone(),
                    )
                })
                .collect();
            if !conds.is_empty() {
                let words = c.style.conditions.len().div_ceil(64);
                self.els[e as usize].holds = vec![0; words].into_boxed_slice();
            }
            for (cond, x) in conds {
                let r = reads(&x, scope, self.screen);
                queue::bind(
                    self.id,
                    Target::Holds { node: e, cond },
                    x,
                    r,
                    self.env.clone(),
                );
            }
        }
        if leaf.is_none() || kind == Kind::Button {
            let mut inner = Vec::new();
            for c in children {
                self.node(d, inst, c, e, node, scope, &mut inner);
            }
        }
    }

    /// The record of an element of `inst` with `style`, under the store
    /// conditions `holds` says it holds (bits by condition index).
    fn fold(&self, inst: u32, style: u32, holds: &[u64]) -> Record {
        if style == NONE {
            return Record::default();
        }
        let ins = &self.insts[inst as usize];
        let c = &self.docs[ins.doc as usize];
        let env = Fixed {
            surface: &self.env,
            more: &ins.more,
        };
        c.style.record(style as usize, |cond| {
            if c.fixed[cond as usize] {
                eval_static(&c.style.conditions[cond as usize], &env)
            } else {
                holds.get((cond / 64) as usize).copied().unwrap_or(0) & (1 << (cond % 64)) != 0
            }
        })
    }

    /// `e`'s text; an image's is its source.
    fn set_text(&mut self, e: u32, text: &str) {
        if self.els[e as usize].kind == Kind::Image {
            return self.set_src(e, text);
        }
        let el = &mut self.els[e as usize];
        if &*el.text == text {
            return;
        }
        el.text.clear();
        el.text.push_str(text);
        if let Some(leaf) = el.leaf {
            self.arena.set_text(leaf, text);
        }
        self.paint = true;
    }

    /// `e`'s box image's source: a texture handle's number ("" none).
    fn set_src(&mut self, e: u32, src: &str) {
        let props = self.els[e as usize].image.get_or_insert_with(|| {
            Box::new(ImageProps {
                handle: f64::NAN,
                ..Default::default()
            })
        });
        props.handle = src.trim().parse::<f64>().unwrap_or(f64::NAN);
        self.paint = true;
    }

    /// Places each anchored node (CONVENTIONS.md §World): `project` gives
    /// the surface point (CSS px) of its target lifted by its world offset,
    /// `None` when that is off the screen, behind the camera or a dead
    /// entity's, which hides it, its subtree and its accessibility nodes.
    /// Allocates nothing; a node that did not move repaints nothing.
    pub fn place_anchors(
        &mut self,
        mut project: impl FnMut(AnchorTarget, [f32; 3]) -> Option<[f32; 2]>,
    ) {
        let els = &self.els;
        self.anchored
            .retain(|&e| !els[e as usize].dead && els[e as usize].anchor.is_some());
        let (w, h) = self.env.size.get();
        for &e in &self.anchored {
            let el = &mut self.els[e as usize];
            let a = el.anchor.as_mut().expect("an anchored node");
            let at = project(a.target, a.lift)
                .filter(|p| p[0] >= 0.0 && p[1] >= 0.0 && p[0] <= w && p[1] <= h);
            let shown = at.is_some();
            let at = at.unwrap_or(a.at);
            if shown != a.shown || at != a.at {
                (a.shown, a.at) = (shown, at);
                self.sem.place(el.node, shown.then_some(at));
                self.paint = true;
            }
        }
    }

    /// The anchored nodes, for the host to ask which entities they need.
    pub fn anchor_targets(&self) -> impl Iterator<Item = AnchorTarget> + '_ {
        self.anchored
            .iter()
            .filter_map(|&e| self.els[e as usize].anchor.as_ref().map(|a| a.target))
    }

    /// Applies `e`'s record over what it inherits, then its children's.
    fn inherit(&mut self, e: u32) {
        let parent = self.els[e as usize].parent;
        let from = if parent == NONE {
            self.root
        } else {
            self.els[parent as usize].eff
        };
        let el = &self.els[e as usize];
        let r = &el.record;
        let eff = Inherit {
            size: if r.text.sized { r.font_size } else { from.size },
            ink: if r.inked { r.skin.ink } else { from.ink },
            text: r.text.over(&from.text),
            opacity: from.opacity * r.skin.opacity,
        };
        let mut layout = r.layout_in(self.insets);
        // An anchored node is out of flow: the host places it.
        if el.hidden > 0 || el.anchor.is_some() {
            let mut s = taffy::Style::from(&layout);
            if el.hidden > 0 {
                s.display = taffy::Display::None;
            } else {
                s.position = taffy::Position::Absolute;
            }
            layout = rune_taffy::CompactStyle::from(&s);
        }
        let (node, leaf) = (el.node, el.leaf);
        self.arena.set_style(node, layout);
        if let Some(leaf) = leaf {
            self.arena.set_run(
                leaf,
                Run {
                    size: eff.size,
                    tracking: eff.text.tracking(eff.size),
                    line: eff.text.line_height(eff.size),
                    weight: eff.text.weight,
                    nowrap: eff.text.nowrap,
                },
            );
        }
        self.els[e as usize].eff = eff;
    }

    /// Re-applies `e` and every descendant whose inheritance changed.
    fn reapply(&mut self, e: u32) {
        let before = self.els[e as usize].eff;
        self.inherit(e);
        if self.els[e as usize].eff == before {
            return;
        }
        for c in 0..self.els.len() as u32 {
            if self.els[c as usize].parent == e && !self.els[c as usize].dead {
                self.reapply(c);
            }
        }
    }

    /// The viewport changed: the conditions that read it are folded again.
    pub fn resize(&mut self, width: f32, height: f32) {
        if self.env.size.get() == (width, height) {
            return;
        }
        let old = self.env.size.replace((width, height));
        let classes = [Screen::Narrow, Screen::Short, Screen::Compact];
        let moved = classes
            .iter()
            .any(|c| c.holds(old.0, old.1, false) != c.holds(width, height, false));
        if moved {
            let bits = classes
                .iter()
                .enumerate()
                .filter(|(_, c)| c.holds(width, height, false))
                .fold(0u32, |b, (k, _)| b | 1 << k);
            queue::set(self.screen, bits.into());
        }
        if moved || self.els.is_empty() {
            for e in 0..self.els.len() as u32 {
                self.queue(e);
            }
        }
        self.paint = true;
    }

    /// The HUD's safe-area insets on the tree's surface (top, right,
    /// bottom, left): the viewport's safe area with the host's covered
    /// region folded in, which `env(hud-inset-*)` reads. A change lays the
    /// elements that read them out again.
    pub fn set_insets(&mut self, insets: [f32; 4]) {
        if self.insets == insets {
            return;
        }
        self.insets = insets;
        for e in 0..self.els.len() as u32 {
            let el = &self.els[e as usize];
            if !el.dead && el.record.safe != [0; 2] {
                self.inherit(e);
            }
        }
        self.paint = true;
    }

    fn queue(&mut self, e: u32) {
        let el = &mut self.els[e as usize];
        if !el.queued {
            el.queued = true;
            self.dirty.push(e);
        }
    }

    /// The characters laid-out text drew that no face has, since the last
    /// call: a frame that found none allocates nothing.
    pub fn take_missing(&mut self) -> Vec<char> {
        std::mem::take(&mut self.missing)
    }

    /// Drains the queue, refolds what changed, lays out and paints. The
    /// fonts measure text; `icons` draws icons.
    pub fn frame(&mut self, fonts: &og_ui::Fonts, icons: &IconAtlas) -> Frame {
        let mut out = self.commit();
        let (laid_out, painted) = self.present(fonts, icons);
        out.laid_out = laid_out;
        out.painted = painted;
        out
    }

    /// The queue's ops applied and the changed records folded again: what
    /// a frame does before layout.
    pub fn commit(&mut self) -> Frame {
        let mut out = Frame::default();
        queue::drain(self.id, &mut self.ops);
        out.ops = self.ops.len();
        let mut ops = std::mem::take(&mut self.ops);
        for op in ops.drain(..) {
            match op {
                Op::Text { node, text, .. } => self.set_text(node, &text),
                Op::Holds { node, cond, on, .. } => {
                    let w = &mut self.els[node as usize].holds[(cond / 64) as usize];
                    let bit = 1u64 << (cond % 64);
                    let was = *w & bit != 0;
                    if was != on {
                        *w ^= bit;
                        self.queue(node);
                    }
                }
                Op::Shown { node, on, .. } => {
                    let b = &mut self.ifs[node as usize];
                    if b.shown != on {
                        b.shown = on;
                        let tops = b.tops.clone();
                        for e in tops {
                            let el = &mut self.els[e as usize];
                            el.hidden = if on {
                                el.hidden.saturating_sub(1)
                            } else {
                                el.hidden + 1
                            };
                            self.queue(e);
                        }
                    }
                }
                Op::Rebuild { node, .. } => self.rebuild(node),
            }
        }
        self.ops = ops;
        if !self.dirty.is_empty() {
            let mut dirty = std::mem::take(&mut self.dirty);
            dirty.sort_unstable();
            out.refolded = dirty.len();
            for &e in &dirty {
                let el = &self.els[e as usize];
                if el.dead || el.js != NONE {
                    continue;
                }
                let r = self.fold(el.inst, el.style, &el.holds);
                self.els[e as usize].record = r;
                self.els[e as usize].queued = false;
            }
            for &e in &dirty {
                if !self.els[e as usize].dead {
                    self.reapply(e);
                }
            }
            dirty.clear();
            self.dirty = dirty;
            self.paint = true;
        }
        out
    }

    /// The UI tick's time in seconds (unscaled: a paused game's menus
    /// still move) and the player's reduced-motion setting, which the next
    /// [`Tree::present`] samples motion at. Never a wall clock.
    pub fn set_time(&mut self, t: f64, reduced_motion: bool) {
        self.time = t;
        self.motion.set_reduced_motion(reduced_motion);
    }

    /// Samples motion at the tick's time, lays out what is dirty and paints
    /// again when anything changed or moves: whether it laid out, and
    /// whether it painted.
    pub fn present(&mut self, fonts: &og_ui::Fonts, icons: &IconAtlas) -> (bool, bool) {
        let moving = self.motion.tick_at(self.time);
        self.settle_exits();
        let (w, h) = self.env.size.get();
        let tf = text_font(fonts);
        let faces = || {
            std::iter::once(tf.handle).chain(
                tf.metrics
                    .runs
                    .iter()
                    .flat_map(|r| r.iter().map(|f| f.handle)),
            )
        };
        if !faces().eq(self.faces.iter().copied()) {
            self.faces = faces().collect();
            self.arena.remeasure();
            for g in &mut self.glyphs {
                g.key = [u32::MAX; 5];
            }
            self.paint = true;
        }
        let m = &tf.metrics;
        let laid_out = self.arena.layout(w, h, &mut |t, run, width| {
            crate::text::measure(m, t, run, width)
        });
        // A scroller's `:state(more)` follows its layout: when one moved,
        // what reads it is folded and laid out again in the same frame.
        if laid_out && self.scrolled() {
            self.commit();
            self.arena.layout(w, h, &mut |t, run, width| {
                crate::text::measure(m, t, run, width)
            });
        }
        if laid_out {
            self.clamp_scrolls();
        }
        // A box that moved slides there when it transitions its layout.
        let arena = &self.arena;
        self.motion.relayout(|n| arena.box_of(n));
        let painted = laid_out || self.paint || moving;
        if painted {
            self.repaint(fonts, icons);
            self.paint = false;
            self.paints += 1;
            let drawn = self.drawn_boxes();
            self.sem.set_drawn(drawn);
        }
        (laid_out, painted)
    }

    /// The box each element under a transform is drawn in (its painted
    /// box's corners through the transform, bounded), by arena node: the
    /// accessibility tree's bounds for it.
    fn drawn_boxes(&self) -> rustc_hash::FxHashMap<Id, [f32; 4]> {
        let mut out = rustc_hash::FxHashMap::default();
        for (e, el) in self.els.iter().enumerate() {
            let xf = self.xfs.get(e).copied().unwrap_or(0);
            let Some(&(x, y, true, _)) = self.abs.get(e) else {
                continue;
            };
            if xf == 0 || el.dead {
                continue;
            }
            let [a, b, c, d, tx, ty, _, _] = self.drawn.transforms[xf as usize];
            let bx = self
                .motion
                .box_of(el.node)
                .unwrap_or_else(|| self.arena.box_of(el.node));
            let corners = [(x, y), (x + bx.w, y), (x, y + bx.h), (x + bx.w, y + bx.h)]
                .map(|(px, py)| (a * px + c * py + tx, b * px + d * py + ty));
            let x0 = corners.iter().map(|p| p.0).fold(f32::INFINITY, f32::min);
            let y0 = corners.iter().map(|p| p.1).fold(f32::INFINITY, f32::min);
            let x1 = corners
                .iter()
                .map(|p| p.0)
                .fold(f32::NEG_INFINITY, f32::max);
            let y1 = corners
                .iter()
                .map(|p| p.1)
                .fold(f32::NEG_INFINITY, f32::max);
            out.insert(el.node, [x0, y0, x1 - x0, y1 - y0]);
        }
        out
    }

    /// Sets each asked scroller's `:state(more)` from its layout and
    /// queues the elements that read one that changed.
    fn scrolled(&mut self) -> bool {
        let mut moved = false;
        for e in 0..self.els.len() {
            let el = &self.els[e];
            if el.dead || el.style == NONE {
                continue;
            }
            let ins = &self.insts[el.inst as usize];
            let c = &self.docs[ins.doc as usize];
            if c.more.is_empty() {
                continue;
            }
            let node = c.style.nodes[el.style as usize].node;
            if !c.more.iter().any(|&(_, n)| n == node) {
                continue;
            }
            let more = self.arena.runs_past_end(el.node);
            let inst = el.inst;
            let more_now = &mut self.insts[inst as usize].more;
            if more_now.contains(&node) == more {
                continue;
            }
            if more {
                more_now.push(node);
            } else {
                more_now.retain(|&n| n != node);
            }
            moved = true;
            let d = self.insts[inst as usize].doc as usize;
            for r in 0..self.els.len() as u32 {
                let el = &self.els[r as usize];
                if el.inst != inst || el.style == NONE || el.dead {
                    continue;
                }
                let c = &self.docs[d];
                let [at, len] = c.style.nodes[el.style as usize].spans;
                let reads = c.style.spans[at as usize..(at + len) as usize]
                    .iter()
                    .any(|s| c.more.iter().any(|&(k, n)| k == s.condition && n == node));
                if reads {
                    self.queue(r);
                }
            }
        }
        moved
    }

    fn rebuild(&mut self, b: u32) {
        let rows = std::mem::take(&mut self.eaches[b as usize].rows);
        for (root, tops) in rows {
            queue::dispose(root);
            for e in tops {
                self.arena.detach(self.els[e as usize].node);
            }
            self.ordered = false;
        }
        // Everything the rows made is gone.
        let NodeKind::For {
            var, index, key, ..
        } = self.docs[self.eaches[b as usize].doc as usize].doc.nodes[self.eaches[b as usize].node]
            .kind
            .clone()
        else {
            return;
        };
        self.rows(b, &var, index.as_deref(), &key);
        self.paint = true;
    }

    /// Makes `order` again when the structure changed: the elements under
    /// the arena's root depth first, in their arena order.
    fn order(&mut self) {
        if self.ordered {
            return;
        }
        self.ordered = true;
        self.el_of.clear();
        self.el_of.resize(self.arena.len(), NONE);
        for (e, el) in self.els.iter().enumerate() {
            if !el.dead {
                self.el_of[el.node as usize] = e as u32;
            }
        }
        self.order.clear();
        self.walk.clear();
        self.walk.extend(self.arena.children(0).iter().rev());
        while let Some(n) = self.walk.pop() {
            let e = self.el_of.get(n as usize).copied().unwrap_or(NONE);
            if e != NONE {
                self.order.push(e);
            }
            self.walk.extend(self.arena.children(n).iter().rev());
        }
    }

    fn repaint(&mut self, fonts: &og_ui::Fonts, icons: &IconAtlas) {
        /// A line's stroke: its own colour faded, else the ink (already faded).
        fn s_line_color(stroke: &Option<Rgba>, ink: Rgba, alpha: f32) -> Rgba {
            stroke.map_or(ink, |mut c| {
                c[3] *= alpha;
                c
            })
        }
        let (vw, vh) = self.env.size.get();
        let screen = [0.0, 0.0, vw, vh];
        let p = &mut self.drawn;
        p.clear();
        (p.width, p.height) = (vw, vh);
        // Font slots: the body's (0), the icon set's (1), then the faces
        // a character the body lacks falls back to (its mixed font's runs:
        // the CJK companion, the locale's, the game's, the fallback), one
        // each from 2; the host adds each one's other pages.
        let tf = text_font(fonts);
        let icon_font = icons.font.as_ref().and_then(|f| f.handle);
        let runs = tf.metrics.runs.as_deref().unwrap_or(&[]);
        let firsts = || {
            [tf.handle, icon_font]
                .into_iter()
                .chain(runs.iter().map(|f| f.handle))
        };
        if p.fonts.len() != 2 + runs.len()
            || firsts()
                .zip(&p.fonts)
                .any(|(h, slot)| slot.first().copied() != h)
        {
            p.fonts = firsts().map(|h| h.iter().copied().collect()).collect();
        }
        let body = tf.handle;
        self.order();
        self.abs.clear();
        self.abs.resize(self.els.len(), (0.0, 0.0, false, [0.0; 4]));
        self.shown.clear();
        let top = Shown {
            alpha: 1.0,
            ink: self.root.ink,
            out: false,
        };
        // NaN: not reached yet this frame.
        let unseen = Shown {
            alpha: f32::NAN,
            ..top
        };
        self.shown.resize(self.els.len(), unseen);
        self.glyphs.resize_with(self.els.len(), Glyphs::default);
        self.xfs.resize(self.els.len(), 0);
        let m = &tf.metrics;
        for k in 0..self.order.len() {
            let e = self.order[k] as usize;
            let el = &self.els[e];
            // What motion shows: the box a layout transition slides, the
            // skin a transition or keyframes move.
            let b = self
                .motion
                .box_of(el.node)
                .unwrap_or_else(|| self.arena.box_of(el.node));
            let (px, py, pv, pclip) = if let Some(a) = &el.anchor {
                // Its box's top left at its placed point, over the screen;
                // it shows when its target is in view and its parent shows.
                let pv = el.arena_parent == NONE || self.abs[el.arena_parent as usize].2;
                (a.at[0] - b.x, a.at[1] - b.y, pv && a.shown, screen)
            } else if el.arena_parent == NONE {
                (0.0, 0.0, true, screen)
            } else {
                // A scroller's children move by its scroll.
                let [sx, sy] = self.scroll_of(el.arena_parent);
                let (px, py, pv, pclip) = self.abs[el.arena_parent as usize];
                (px - sx, py - sy, pv, pclip)
            };
            // What it inherits comes from the element it inherits from,
            // which a fixed one is not the arena child of: one not reached
            // yet gives what its record folds to.
            let from = match el.parent {
                NONE => top,
                p if !self.shown[p as usize].alpha.is_nan() => self.shown[p as usize],
                p => {
                    let eff = &self.els[p as usize].eff;
                    Shown {
                        alpha: eff.opacity,
                        ink: eff.ink,
                        out: false,
                    }
                }
            };
            // `display: none` leaves a zero box at its parent's corner:
            // nothing of it or under it paints.
            let r = &el.record;
            let s = self.motion.skin(el.node, &r.skin);
            let alpha = from.alpha * s.opacity;
            let own_ink = if r.inked { s.ink } else { from.ink };
            self.shown[e] = Shown {
                alpha,
                ink: own_ink,
                out: from.out || self.motion.exiting(el.node),
            };
            let visible = pv
                && !el.dead
                && el.hidden == 0
                && alpha > 0.0
                && r.layout.display() != taffy::Display::None;
            let (x, y) = (px + b.x, py + b.y);
            let clip = pclip;
            // What overflows a box that does not show it is cut to it.
            let o = taffy::CoreStyle::overflow(&r.layout);
            let scroller = self.scroll_axes(e as u32) != [false; 2];
            // CSS clips overflow at the padding box: what scrolls or runs
            // over never covers the box's own border.
            let [bt, br, bb, bl] = s.border;
            let inner =
                if scroller || o.x != taffy::Overflow::Visible || o.y != taffy::Overflow::Visible {
                    [
                        clip[0].max(x + bl),
                        clip[1].max(y + bt),
                        clip[2].min(x + b.w - br),
                        clip[3].min(y + b.h - bb),
                    ]
                } else {
                    clip
                };
            self.abs[e] = (x, y, visible, inner);
            let pxf = if el.arena_parent == NONE {
                0
            } else {
                self.xfs[el.arena_parent as usize]
            };
            self.xfs[e] = pxf;
            if !visible {
                continue;
            }
            let p = &mut self.drawn;
            let start = p.quads.len();
            let xf = transformed(p, pxf, &s.transform, [x, y, b.w, b.h]);
            self.xfs[e] = xf;
            let rect = [x, y, b.w, b.h];
            let radius = radii(s.radius, b.w, b.h);
            let faded = |mut c: Rgba| {
                c[3] *= alpha;
                c
            };
            // Outer shadows under the box, the last listed lowest.
            let shadows = &s.shadows[..usize::from(s.shadow_count)];
            for sh in shadows
                .iter()
                .rev()
                .filter(|sh| !sh.inset && sh.color[3] > 0.0)
            {
                let p4 = [sh.offset[0], sh.offset[1], sh.blur, sh.spread];
                p.quads.push(quad(
                    PaintKind::Shadow,
                    rect,
                    clip,
                    faded(sh.color),
                    radius,
                    p4,
                    0,
                    0,
                ));
            }
            let im = &match &el.image {
                Some(props) => props.over(&s.image, self.textures.as_ref()),
                None => s.image,
            };
            let image = og_draw::TextureHandle::from_slot(im.texture).filter(|_| im.wipe > 0.0);
            // A fill-tinted image takes the fill's colour in its place.
            let fill_tints = image.is_some() && im.tint == crate::style::Tint::Fill;
            if s.fill[3] > 0.0 && !fill_tints {
                p.quads.push(quad(
                    PaintKind::Solid,
                    rect,
                    clip,
                    faded(s.fill),
                    radius,
                    [0.0; 4],
                    0,
                    0,
                ));
            }
            if let Some(g) = &s.gradient {
                // A `to <corner>` line runs perpendicular to the diagonal
                // between the other two corners, so its angle is the box's.
                let angle = match g.corner {
                    [0, _] | [_, 0] => g.angle,
                    [cx, cy] => og_core::math::atan2(f32::from(cx) * b.h, -f32::from(cy) * b.w),
                };
                let gi = p.gradient(angle, g.stops().iter().copied());
                p.quads.push(quad(
                    PaintKind::Gradient,
                    rect,
                    clip,
                    [1.0, 1.0, 1.0, alpha],
                    radius,
                    [0.0; 4],
                    gi,
                    0,
                ));
            }
            // Its image over the fill, nine-sliced, radially wiped.
            if let Some(t) = image {
                let width = if im.width == [0.0; 4] {
                    im.slice
                } else {
                    im.width
                };
                let color = match im.tint {
                    crate::style::Tint::Own => [1.0, 1.0, 1.0, alpha],
                    crate::style::Tint::Ink => faded(own_ink),
                    crate::style::Tint::Fill => faded(s.fill),
                };
                let flags = (u32::from(im.fill) * og_draw::paint::IMAGE_FILL)
                    | (u32::from(im.tile) * og_draw::paint::IMAGE_TILE);
                let cut = if im.wipe < 1.0 { 1.0 - im.wipe } else { 0.0 };
                p.quads.push(PaintQuad {
                    uv: im.uv,
                    wipe: [cut, im.wipe_start, f32::from(u8::from(im.wipe_ccw)), 0.0],
                    ..quad(
                        PaintKind::Image,
                        rect,
                        clip,
                        color,
                        width,
                        im.slice,
                        t.to_bits(),
                        flags,
                    )
                });
            }
            for sh in shadows
                .iter()
                .rev()
                .filter(|sh| sh.inset && sh.color[3] > 0.0)
            {
                let p4 = [sh.offset[0], sh.offset[1], sh.blur, sh.spread];
                p.quads.push(quad(
                    PaintKind::InsetShadow,
                    rect,
                    clip,
                    faded(sh.color),
                    radius,
                    p4,
                    0,
                    0,
                ));
            }
            let sides = if s.border_side[3] > 0.0 {
                s.border_sides
            } else {
                0
            };
            if (s.border_color[3] > 0.0 || sides != 0) && s.border.iter().any(|&w| w > 0.0) {
                p.quads.push(PaintQuad {
                    uv: faded(s.border_side),
                    ..quad(
                        PaintKind::Border,
                        rect,
                        clip,
                        faded(s.border_color),
                        radius,
                        s.border,
                        0,
                        u32::from(sides),
                    )
                });
            }
            let o = &s.outline;
            if o.width > 0.0 && o.color[3] > 0.0 {
                // The ring of the box grown by the offset and the width; its
                // corners follow the box's, as browsers draw them.
                let out = o.offset + o.width;
                let grown = [x - out, y - out, b.w + 2.0 * out, b.h + 2.0 * out];
                let rr = radius.map(|k| if k > 0.0 { (k + out).max(0.0) } else { 0.0 });
                p.quads.push(quad(
                    PaintKind::Border,
                    grown,
                    clip,
                    faded(o.color),
                    rr,
                    [o.width; 4],
                    0,
                    0,
                ));
            }
            let mut ink = own_ink;
            ink[3] *= alpha;
            if let Some(line) = &el.line {
                let stroke = s_line_color(&r.stroke, ink, alpha);
                let n = line.points.len() / 2;
                if r.stroke_width >= 1.0 && n >= 2 && stroke[3] > 0.0 {
                    let first = p.points.len() as u32;
                    p.points.extend(line.mapped(b.w, b.h));
                    p.quads.push(quad(
                        PaintKind::Line,
                        rect,
                        clip,
                        stroke,
                        [0.0; 4],
                        [r.stroke_width, f32::from(u8::from(line.closed)), 0.0, 0.0],
                        first,
                        n as u32,
                    ));
                }
            }
            if el.kind == Kind::Icon && !el.text.is_empty() {
                let size = b.w.min(b.h).max(1.0);
                let square = [x + (b.w - size) / 2.0, y + (b.h - size) / 2.0, size, size];
                match icons.find_drawn(&el.text) {
                    Some((IconCell::Glyph(_, ch), _)) => {
                        let (asc, desc) = icons
                            .font
                            .as_ref()
                            .map_or((0.8, -0.2), |f| (f.metrics.ascender, f.metrics.descender));
                        let top = y + (b.h - (asc - desc) * size) / 2.0;
                        p.quads.push(quad(
                            PaintKind::Glyph,
                            [square[0], top + asc * size, size, 0.0],
                            clip,
                            ink,
                            [0.0; 4],
                            [0.0; 4],
                            ch as u32,
                            1,
                        ));
                    }
                    // A texture grid's cell: a picture in its own colours, a
                    // mask in the ink.
                    Some((IconCell::Image(t, uv), picture)) => {
                        let color = if picture { [1.0, 1.0, 1.0, alpha] } else { ink };
                        p.quads.push(PaintQuad {
                            uv,
                            ..quad(
                                PaintKind::Image,
                                square,
                                clip,
                                color,
                                [0.0; 4],
                                [0.0; 4],
                                t.to_bits(),
                                og_draw::paint::IMAGE_FILL,
                            )
                        });
                    }
                    Some((IconCell::Builtin(name), _)) => {
                        let set = crate::icons::icons();
                        if p.icons.is_none() {
                            p.icons = Some(set.segments.clone());
                        }
                        if let Some((first, count)) = set.find(name) {
                            p.quads.push(quad(
                                PaintKind::Icon,
                                square,
                                clip,
                                ink,
                                [0.0; 4],
                                [0.0; 4],
                                first,
                                count,
                            ));
                        }
                    }
                    None => {}
                }
            }
            // A glyph with the pack's art for its binding draws it in its
            // text's place: an ink mask one em square, centred on the leaf
            // (which holds an em space, so it lays out as one).
            if let Some(t) = self
                .input
                .per(e as u32)
                .and_then(|g| og_draw::TextureHandle::from_slot(g.art))
                && let Some(leaf) = el.leaf
            {
                let lb = self.arena.box_of(leaf);
                let size = el.eff.size;
                let square = [
                    x + lb.x + (lb.w - size) / 2.0,
                    y + lb.y + (lb.h - size) / 2.0,
                    size,
                    size,
                ];
                p.quads.push(PaintQuad {
                    uv: [0.0, 0.0, 1.0, 1.0],
                    ..quad(
                        PaintKind::Image,
                        square,
                        clip,
                        ink,
                        [0.0; 4],
                        [0.0; 4],
                        t.to_bits(),
                        og_draw::paint::IMAGE_FILL,
                    )
                });
            }
            // An `Input` showing its placeholder draws it at half its ink.
            let field = self.input.per(e as u32).filter(|f| f.input);
            let mut text_ink = ink;
            if field.is_some_and(|f| f.value.is_empty()) {
                text_ink[3] *= 0.5;
            }
            if let (Some(leaf), Some(_)) = (el.leaf, body)
                && !el.text.is_empty()
            {
                let lb = self.arena.box_of(leaf);
                let g = &mut self.glyphs[e];
                lay_out(
                    g,
                    &el.text,
                    &el.eff,
                    lb.w,
                    m,
                    &mut self.missing,
                    &mut self.breaks,
                );
                let (lx, ly) = (x + lb.x, y + lb.y);
                let sh = s.text_shadow;
                if sh.color[3] > 0.0 {
                    let mut c = sh.color;
                    c[3] *= alpha;
                    let [dx, dy] = sh.offset;
                    p.quads.extend(g.quads.iter().map(|q| PaintQuad {
                        rect: [
                            q.rect[0] + lx + dx,
                            q.rect[1] + ly + dy,
                            q.rect[2],
                            q.rect[3],
                        ],
                        clip,
                        color: c,
                        p: [q.p[0], sh.blur, 0.0, 0.0],
                        ..*q
                    }));
                }
                // The outline under the fill: each glyph thickened by half
                // the stroke (its field's reach bounds it, in the shader).
                let st = s.text_stroke;
                if st.width > 0.0 && st.color[3] > 0.0 {
                    let mut c = st.color;
                    c[3] *= alpha;
                    p.quads.extend(g.quads.iter().map(|q| PaintQuad {
                        rect: [q.rect[0] + lx, q.rect[1] + ly, q.rect[2], q.rect[3]],
                        clip,
                        color: c,
                        p: [q.p[0] + 0.5 * st.width, 0.0, 0.0, 0.0],
                        ..*q
                    }));
                }
                p.quads.extend(g.quads.iter().map(|q| PaintQuad {
                    rect: [q.rect[0] + lx, q.rect[1] + ly, q.rect[2], q.rect[3]],
                    clip,
                    color: text_ink,
                    ..*q
                }));
                let d = s.decoration;
                if d.line != 0 {
                    let mut c = if d.color[3] > 0.0 { d.color } else { own_ink };
                    c[3] *= alpha;
                    let size = el.eff.size;
                    let th = if d.thickness > 0.0 {
                        d.thickness
                    } else {
                        (size / 14.0).max(1.0)
                    };
                    use crate::style::Decoration as D;
                    for &[x0, base, w] in &g.lines {
                        let at = [
                            (D::UNDERLINE, base + 0.12 * size),
                            (D::OVERLINE, base - m.ascender * size),
                            (D::LINE_THROUGH, base - 0.3 * size),
                        ];
                        for (bit, cy) in at {
                            if d.line & bit != 0 {
                                let rect = [lx + x0, ly + cy - th / 2.0, w, th];
                                p.quads.push(quad(
                                    PaintKind::Solid,
                                    rect,
                                    clip,
                                    c,
                                    [0.0; 4],
                                    [0.0; 4],
                                    0,
                                    0,
                                ));
                            }
                        }
                    }
                }
            }
            // The focused `Input`'s caret: a bar the text's height before
            // the character it stands at, in the text's ink.
            if let (Some(leaf), Some(f)) = (el.leaf, field)
                && self.input.focus_el() == e as u32
            {
                let t = &el.eff.text;
                let size = el.eff.size;
                let typed = !f.value.is_empty();
                let before = if typed {
                    &f.value[..f.caret.min(f.value.len())]
                } else {
                    ""
                };
                let features = og_text::shape::Features::NONE;
                let track = t.tracking(size) / size.max(f32::MIN_POSITIVE);
                let pen = og_text::text::widest_line(before, m, size, track, features);
                let dx = if typed {
                    self.glyphs[e].lines.first().map_or(0.0, |l| l[0])
                } else {
                    0.0
                };
                let glyph = (m.ascender - m.descender) * size;
                let top = (t.line_height(size) - glyph) / 2.0;
                let lb = self.arena.box_of(leaf);
                let rect = [
                    x + lb.x + dx + pen,
                    y + lb.y + top,
                    (size / 14.0).max(1.0),
                    glyph,
                ];
                p.quads.push(quad(
                    PaintKind::Solid,
                    rect,
                    clip,
                    ink,
                    [0.0; 4],
                    [0.0; 4],
                    0,
                    0,
                ));
            }
            if xf != 0 {
                for q in &mut self.drawn.quads[start..] {
                    q.xf = xf;
                }
            }
        }
        self.drawn.generation += 1;
    }

    /// An element's box by its document id (`#Time`), for checks.
    pub fn find(&self, id: &str) -> Option<(f32, f32, f32, f32)> {
        let e = self.els.iter().position(|el| {
            let ins = &self.insts[el.inst as usize];
            let c = &self.docs[ins.doc as usize];
            el.style != NONE && c.doc.nodes[c.style.nodes[el.style as usize].node as usize].id == id
        })?;
        let (x, y, _, _) = *self.abs.get(e)?;
        let b = self.arena.box_of(self.els[e].node);
        Some((x, y, b.w, b.h))
    }

    /// The element a press at (x, y) reaches: of the shown, enabled
    /// elements that answer a press and whose box, inside the clip their
    /// parent paints them in, holds the point, the last made (a descendant
    /// comes after its ancestors and paints over them); NONE for none.
    fn hit(&self, x: f32, y: f32) -> u32 {
        self.hit_where(x, y, |el| el.clickable)
    }

    /// [`Tree::hit`] over the elements `want` accepts; a press needs them
    /// enabled too. Behind an open modal, nothing answers.
    fn hit_where(&self, x: f32, y: f32, want: impl Fn(&El) -> bool) -> u32 {
        for &e in self.order.iter().rev() {
            let (e, el) = (e as usize, &self.els[e as usize]);
            // An element on its way out, or in one, answers nothing.
            if el.dead || !want(el) || self.shown.get(e).is_some_and(|s| s.out) {
                continue;
            }
            let Some(&(bx, by, shown, _)) = self.abs.get(e) else {
                continue;
            };
            let all = [f32::MIN, f32::MIN, f32::MAX, f32::MAX];
            let [cx0, cy0, cx1, cy1] = match el.arena_parent {
                _ if el.anchor.is_some() => all,
                NONE => all,
                p => self.abs.get(p as usize).map_or(all, |a| a.3),
            };
            let b = self
                .motion
                .box_of(el.node)
                .unwrap_or_else(|| self.arena.box_of(el.node));
            let inside = x >= bx.max(cx0)
                && x < (bx + b.w).min(cx1)
                && y >= by.max(cy0)
                && y < (by + b.h).min(cy1);
            if !shown || !inside || !self.reachable(e as u32) {
                continue;
            }
            if el.clickable
                && el.disabled != NONE
                && og_reactive::bind::truthy(&queue::get(el.disabled))
            {
                continue;
            }
            return e as u32;
        }
        NONE
    }

    /// The pointer this frame, at (x, y) on the tree's surface: a press
    /// takes the element under it, and a release over that same element
    /// clicks it. A click reports `click` with its id, then `emit` for each
    /// handler its `onclick` or `ontap` names, to `out`.
    pub fn pointer(&mut self, x: f32, y: f32, pressed: bool, released: bool, out: &mut Vec<Event>) {
        // Op nodes take the pointer when it moved, a frame painted since
        // (a box may have come under it), or a button changed.
        let was = std::mem::replace(&mut self.last_pointer, (x, y, self.paints));
        if self.has_ops() && (was != self.last_pointer || pressed || released) {
            self.pointer_ops(x, y, (was.0, was.1) != (x, y), pressed, released);
        }
        if pressed {
            let e = self.hit(x, y);
            self.pressed = if e != NONE && self.els[e as usize].js == NONE {
                e
            } else {
                NONE
            };
        }
        if !released {
            return;
        }
        let held = std::mem::replace(&mut self.pressed, NONE);
        if held == NONE || self.hit(x, y) != held {
            return;
        }
        let el = &self.els[held as usize];
        let inst = &self.insts[el.inst as usize];
        let n = &self.docs[inst.doc as usize].doc.nodes[el.at as usize];
        let event = |kind, name: &str| Event {
            mount: inst.mount,
            path: n.id.as_str().into(),
            row_key: el.row.clone(),
            kind,
            name: name.into(),
        };
        out.push(event("click", &n.id));
        for t in &n.triggers {
            if let (When::Click | When::Tap, Act::Emit(name)) = (&t.when, &t.act) {
                out.push(event("emit", name));
            }
        }
    }

    /// Each live element with a document id: its id, its box on screen
    /// (x, y, width, height), whether it shows, and its text.
    pub fn dump(&self) -> Vec<(&str, [f32; 4], bool, &str)> {
        self.els
            .iter()
            .enumerate()
            .filter(|(_, el)| !el.dead && el.style != NONE)
            .filter_map(|(e, el)| {
                let ins = &self.insts[el.inst as usize];
                let c = &self.docs[ins.doc as usize];
                let id = c.doc.nodes[c.style.nodes[el.style as usize].node as usize]
                    .id
                    .as_str();
                if id.is_empty() {
                    return None;
                }
                let (x, y, shown, _) = *self.abs.get(e)?;
                let b = self.arena.box_of(el.node);
                Some((id, [x, y, b.w, b.h], shown, &*el.text))
            })
            .collect()
    }

    /// The text an element shows, by its document id.
    pub fn text(&self, id: &str) -> Option<&str> {
        self.els
            .iter()
            .find(|el| {
                let ins = &self.insts[el.inst as usize];
                let c = &self.docs[ins.doc as usize];
                el.style != NONE
                    && c.doc.nodes[c.style.nodes[el.style as usize].node as usize].id == id
            })
            .map(|el| &*el.text)
    }
}

#[allow(clippy::too_many_arguments)]
fn quad(
    kind: PaintKind,
    rect: [f32; 4],
    clip: [f32; 4],
    color: Rgba,
    radius: [f32; 4],
    p: [f32; 4],
    a: u32,
    b: u32,
) -> PaintQuad {
    PaintQuad {
        rect,
        clip,
        color,
        radius,
        p,
        kind: kind as u32,
        a,
        b,
        xf: 0,
        uv: [0.0; 4],
        wipe: [0.0; 4],
    }
}

/// Adds the transform of a box `rect` (x, y, w, h) composed onto its
/// parent's `parent` to `p.transforms`: its index, or `parent` when its own
/// is the identity. CSS's order: move to the origin, translate, rotate,
/// scale, move back.
fn transformed(p: &mut Paint, parent: u32, t: &crate::style::Transform, rect: [f32; 4]) -> u32 {
    if t.translate == [0.0; 2] && t.scale == [1.0; 2] && t.rotate == 0.0 {
        return parent;
    }
    if p.transforms.is_empty() {
        p.transforms.push(og_draw::paint::IDENTITY);
    }
    let (sin, cos) = og_core::math::sin_cos(t.rotate);
    let [sx, sy] = t.scale;
    let (a, b, c, d) = (cos * sx, sin * sx, -sin * sy, cos * sy);
    let ox = rect[0] + t.origin[0] * rect[2];
    let oy = rect[1] + t.origin[1] * rect[3];
    let e = ox + t.translate[0] - (a * ox + c * oy);
    let f = oy + t.translate[1] - (b * ox + d * oy);
    let [pa, pb, pc, pd, pe, pf, _, _] = p.transforms[parent as usize];
    p.transforms.push([
        pa * a + pc * b,
        pb * a + pd * b,
        pa * c + pc * d,
        pb * c + pd * d,
        pa * e + pc * f + pe,
        pb * e + pd * f + pf,
        0.0,
        0.0,
    ]);
    p.transforms.len() as u32 - 1
}

/// Corner radii (top left, top right, bottom right, bottom left) on a
/// `w`×`h` box, all scaled down by one factor when two on a side would
/// overlap, as CSS scales them.
fn radii(r: [f32; 4], w: f32, h: f32) -> [f32; 4] {
    let [tl, tr, br, bl] = r.map(|k| k.max(0.0));
    let fit = |side: f32, sum: f32| if sum > side { side / sum } else { 1.0 };
    let f = fit(w, tl + tr)
        .min(fit(w, bl + br))
        .min(fit(h, tl + bl))
        .min(fit(h, tr + br));
    [tl * f, tr * f, br * f, bl * f]
}

/// The font a text draws in: the body's mixed font (the body face, then
/// the faces it falls back to for what it lacks: `og_ui::Fonts::link`),
/// else the body alone.
fn text_font(fonts: &og_ui::Fonts) -> &og_ui::UiFont {
    fonts.body.metrics.mixed.as_deref().unwrap_or(&fonts.body)
}

/// The paint font slot of face `k` of a text's mixed font: the body's 0,
/// a fallback face's from 2 (1 is the icons').
fn slot_of(k: u8) -> u32 {
    match k {
        0 => 0,
        k => u32::from(k) + 1,
    }
}

/// Lays `text` out as glyph quads from its leaf's top left (a quad's x the
/// pen, y the baseline, w the size; `b` the font slot of the face that
/// has the character, [`slot_of`]), when it, the width or the run changed
/// since `g` was laid out (a change of faces clears `g.key`: `present`).
fn lay_out(
    g: &mut Glyphs,
    text: &str,
    eff: &Inherit,
    width: f32,
    m: &og_text::text::FontMetrics,
    missing: &mut Vec<char>,
    breaks: &mut Vec<(std::ops::Range<usize>, f32)>,
) {
    let t = &eff.text;
    let size = eff.size;
    let tracking = t.tracking(size);
    let line = t.line_height(size);
    let bold = t.weight >= 600;
    let key = [
        width.to_bits(),
        size.to_bits(),
        tracking.to_bits(),
        line.to_bits(),
        u32::from(t.align) | u32::from(t.nowrap) << 2 | u32::from(bold) << 3,
    ];
    if g.key == key && *g.text == *text {
        return;
    }
    g.key = key;
    if *g.text != *text {
        g.text.clear();
        g.text.push_str(text);
    }
    g.quads.clear();
    g.lines.clear();
    g.line_text.clear();
    let glyph = (m.ascender - m.descender) * size;
    let weight = if bold { size * 0.03 } else { 0.0 };
    let track = if size > 0.0 { tracking / size } else { 0.0 };
    let features = og_text::shape::Features::NONE;
    let mut place =
        |k: usize, l: &str, w: f32, quads: &mut Vec<PaintQuad>, lines: &mut Vec<[f32; 3]>| {
            let top = k as f32 * line + (line - glyph) / 2.0;
            let dx = match t.align {
                1 => (width - w) / 2.0,
                2 => width - w,
                _ => 0.0,
            };
            let baseline = top + m.ascender * size;
            lines.push([dx, baseline, w]);
            let mut pen = 0.0f32;
            let mut prev: Option<char> = None;
            for ch in l.chars() {
                if !m.has(ch) {
                    if !ch.is_whitespace() && !ch.is_control() && !missing.contains(&ch) {
                        missing.push(ch);
                    }
                    prev = None;
                    continue;
                }
                if let Some(p) = prev {
                    pen += m.kern(p, ch);
                }
                if !ch.is_whitespace() {
                    // The face that has it, drawn at its own scale.
                    let k = m.face_of(ch.encode_utf8(&mut [0; 4]));
                    let scale = m.part(usize::from(k)).map_or(1.0, |f| f.scale);
                    quads.push(quad(
                        PaintKind::Glyph,
                        [dx + pen * size, baseline, size * scale, 0.0],
                        [0.0; 4],
                        [0.0; 4],
                        [0.0; 4],
                        [weight, 0.0, 0.0, 0.0],
                        ch as u32,
                        slot_of(k),
                    ));
                }
                pen += m.advance(ch) + track;
                prev = Some(ch);
            }
        };
    // Broken as the measure broke it (text.rs), by the same shaped layout
    // at the same width, so a text never breaks inside the box it sized.
    let wrap = (!t.nowrap && size > 0.0).then_some((width + 0.5) / size);
    if og_text::text::paragraph::lines_into(m, text, track, features, wrap, breaks) {
        for (k, (r, w)) in breaks.iter().enumerate() {
            let l = text[r.clone()].trim_end_matches(['\n', '\r']);
            place(k, l, w * size, &mut g.quads, &mut g.lines);
            g.line_text.push_str(l);
            g.line_text.push('\n');
        }
        return;
    }
    let one = og_text::text::widest_line(text, m, size, track, features);
    if t.nowrap || (one <= width + 0.5 && !text.contains('\n')) {
        for (k, l) in text.split('\n').enumerate() {
            let w = og_text::text::widest_line(l, m, size, track, features);
            place(k, l, w, &mut g.quads, &mut g.lines);
            g.line_text.push_str(l);
            g.line_text.push('\n');
        }
        return;
    }
    let lines = og_text::text::break_lines_with_overflow(
        text,
        m,
        size,
        track,
        features,
        Some(width + 0.5),
        false,
    );
    for (k, l) in lines.iter().enumerate() {
        place(k, &l.text, l.width, &mut g.quads, &mut g.lines);
        g.line_text.push_str(&l.text);
        g.line_text.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::look::Layer;

    /// An overlay is its anchored children's containing block, even
    /// inside a fixed box: they fill it, not the fixed box.
    #[test]
    fn an_anchored_child_fills_its_overlay_not_a_fixed_ancestor() {
        let doc = UiDocument::parse(
            "o.svelte",
            "<meta name=\"component\" content=\"o\">\n\
             <div id=\"Bar\"><div id=\"Face\" layout=\"overlay\"><div id=\"Art\" anchor=\"fill\"></div></div></div>\n\
             <style>\n#Bar { position: fixed; left: 0; right: 0; bottom: 0; display: flex; justify-content: center; }\n\
             #Face { width: 64px; height: 64px; }\n</style>",
        )
        .unwrap();
        let look = Rc::new(Look::load(
            &[Layer {
                tokens: crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut tree = Tree::new(look, [Arc::new(doc)]);
        tree.resize(1280.0, 720.0);
        tree.mount("o", &[]).unwrap();
        tree.frame(&og_ui::Fonts::default(), &IconAtlas::default());
        let face = tree.find("Face").unwrap();
        let art = tree.find("Art").unwrap();
        assert_eq!(art, face, "the art covers its face");
    }

    /// A text paints over the panel it sits on: og-render draws a level's
    /// texts before its shapes, so the text takes a higher level.
    #[test]
    fn a_text_paints_over_its_panel() {
        let doc = UiDocument::parse(
            "p.svelte",
            "<meta name=\"component\" content=\"p\">\n\
             <section id=\"P\"><span id=\"T\">06:54</span></section>\n\
             <style>\n#P { background: #102030; padding: 8px; }\n</style>",
        )
        .unwrap();
        let look = Rc::new(Look::load(
            &[Layer {
                tokens: crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut tree = Tree::new(look, [Arc::new(doc)]);
        tree.resize(200.0, 200.0);
        tree.mount("p", &[]).unwrap();
        let fonts = og_ui::Fonts {
            body: og_ui::UiFont {
                handle: Some(og_draw::FontHandle::from_index(0)),
                metrics: og_text::text::FontMetrics::fixed(0.5),
            },
            ..Default::default()
        };
        tree.frame(&fonts, &IconAtlas::default());
        let at = |glyph: bool| {
            tree.drawn
                .quads
                .iter()
                .position(|q| (q.kind == PaintKind::Glyph as u32) == glyph)
                .unwrap()
        };
        assert!(at(true) > at(false), "the text over its panel");
    }

    /// A tracked word in a box its measure sized paints on one line: the
    /// measure takes tracking in em as og-text does, and paint breaks
    /// lines from the same shaped layout ("Leaderb/oard" before).
    #[test]
    fn a_tracked_word_paints_on_the_one_line_its_box_was_sized_for() {
        let doc = UiDocument::parse(
            "w.svelte",
            "<meta name=\"component\" content=\"w\">\n\
             <div id=\"R\"><span id=\"T\">Leaderboard</span><span id=\"U\">Achievement</span></div>\n\
             <style>\n#R { display: flex; flex-wrap: wrap; width: 400px; }\n\
             #T, #U { font-size: 13px; letter-spacing: 0.14em; }\n</style>",
        )
        .unwrap();
        let look = Rc::new(Look::load(
            &[Layer {
                tokens: crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut tree = Tree::new(look, [Arc::new(doc)]);
        tree.resize(800.0, 200.0);
        tree.mount("w", &[]).unwrap();
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../templates/fonts/latin/NotoSans-Latin.ttf"
        );
        let file = og_text::shape::FontFile::new(Arc::new(std::fs::read(path).unwrap())).unwrap();
        let atlas = og_draw::MsdfFont {
            atlas: og_draw::TextureHandle::from_slot(1).unwrap(),
            distance_range: 4.0,
            foreground: false,
            silhouette: false,
            line_height: 1.2,
            glyphs: Vec::new(),
            kerning: Vec::new(),
        };
        let fonts = og_ui::Fonts {
            body: og_ui::UiFont {
                handle: Some(og_draw::FontHandle::from_index(0)),
                metrics: og_text::text::FontMetrics::from_msdf(&atlas).with_file(Some(file)),
            },
            ..Default::default()
        };
        tree.frame(&fonts, &IconAtlas::default());
        let (_, ty, tw, th) = tree.find("T").unwrap();
        let (_, uy, uw, _) = tree.find("U").unwrap();
        // Untracked, a 13px word is about 6.5px a letter: 0.14em adds
        // 1.82px a letter, not the 23.7px a px-as-em measure gave.
        assert!(tw < 13.0 * 11.0 * (0.62 + 0.14), "measured {tw}");
        assert!(tw + uw < 400.0 && ty == uy, "both words share the row");
        let baselines: Vec<f32> = tree
            .drawn
            .quads
            .iter()
            .filter(|q| q.kind == PaintKind::Glyph as u32)
            .map(|q| q.rect[1])
            .collect();
        assert_eq!(baselines.len(), 22, "every letter drawn");
        assert!(
            baselines
                .iter()
                .all(|&b| (b - baselines[0]).abs() < 0.01 && b < th + ty),
            "one line: {baselines:?}"
        );
    }

    /// A box paints its outer shadows under it, then its fill and gradient,
    /// its inset shadows, its border and its outline, with CSS's radii
    /// (scaled down together when they overlap) and a corner gradient
    /// aimed by the box's aspect.
    #[test]
    fn a_box_paints_shadows_fill_border_and_outline_in_order() {
        let doc = UiDocument::parse(
            "b.svelte",
            "<meta name=\"component\" content=\"b\">\n\
             <div id=\"B\"></div>\n\
             <style>\n#B { width: 200px; height: 100px; background: linear-gradient(to top right, #000, #fff), #102030; \
             border: 1px solid #ff0000; border-width: 1px 2px 3px 4px; border-radius: 80px 80px 80px 0; \
             box-shadow: 0 4px 8px #000, inset 0 0 2px #fff; outline: 2px solid #00ff00; outline-offset: 3px; }\n</style>",
        )
        .unwrap();
        let look = Rc::new(Look::load(
            &[Layer {
                tokens: crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut tree = Tree::new(look, [Arc::new(doc)]);
        tree.resize(400.0, 300.0);
        tree.mount("b", &[]).unwrap();
        tree.frame(&og_ui::Fonts::default(), &IconAtlas::default());
        let q = &tree.drawn.quads;
        let kinds: Vec<u32> = q.iter().map(|q| q.kind).collect();
        use PaintKind as K;
        assert_eq!(
            kinds,
            [
                K::Shadow,
                K::Solid,
                K::Gradient,
                K::InsetShadow,
                K::Border,
                K::Border
            ]
            .map(|k| k as u32)
        );
        // 80 + 80 on the 100px sides: all four scale by 100 / 160.
        assert_eq!(q[1].radius, [50.0, 50.0, 50.0, 0.0]);
        assert_eq!(q[0].p, [0.0, 4.0, 8.0, 0.0], "offset, blur, spread");
        assert_eq!(q[4].p, [1.0, 2.0, 3.0, 4.0], "the border's widths");
        let (x, y, w, h) = tree.find("B").unwrap();
        assert_eq!(
            q[5].rect,
            [x - 5.0, y - 5.0, w + 10.0, h + 10.0],
            "outline 3px out, 2px wide"
        );
        assert_eq!(q[5].p, [2.0; 4]);
        assert_eq!(
            q[5].radius,
            [55.0, 55.0, 55.0, 0.0],
            "a square corner stays square"
        );
        // To the top right of a 2:1 box: perpendicular to the other diagonal.
        let angle = tree.drawn.gradients[0].angle;
        assert!(
            (angle - og_core::math::atan2(1.0, 2.0)).abs() < 1e-3,
            "{angle}"
        );
    }

    /// A transform moves a box and everything under it at paint, about its
    /// origin, composed down the tree; its layout stays. A text's shadow
    /// paints under its glyphs and its underline after them.
    #[test]
    fn a_transform_composes_down_the_tree_and_text_paints_its_shadow_and_lines() {
        let doc = UiDocument::parse(
            "x.svelte",
            "<meta name=\"component\" content=\"x\">\n\
             <div id=\"P\"><div id=\"C\"><span id=\"T\">ab</span></div></div>\n\
             <style>\n#P { width: 100px; height: 40px; background: #102030; rotate: 90deg; }\n\
             #C { width: 20px; height: 10px; background: #fff; translate: 30px 0; }\n\
             #T { text-shadow: 1px 2px 0 #000; text-decoration: underline; }\n</style>",
        )
        .unwrap();
        let look = Rc::new(Look::load(
            &[Layer {
                tokens: crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut tree = Tree::new(look, [Arc::new(doc)]);
        tree.resize(400.0, 300.0);
        tree.mount("x", &[]).unwrap();
        let fonts = og_ui::Fonts {
            body: og_ui::UiFont {
                handle: Some(og_draw::FontHandle::from_index(0)),
                metrics: og_text::text::FontMetrics::fixed(0.5),
            },
            ..Default::default()
        };
        tree.frame(&fonts, &IconAtlas::default());
        let (px, py, pw, ph) = tree.find("P").unwrap();
        let (cx, cy, _, _) = tree.find("C").unwrap();
        assert_eq!((cx, cy), (px, py), "layout does not move");
        let d = &tree.drawn;
        let at = |q: &PaintQuad, x: f32, y: f32| {
            let [a, b, c, dd, e, f, _, _] = d.transforms[q.xf as usize];
            [a * x + c * y + e, b * x + dd * y + f]
        };
        let near =
            |u: [f32; 2], v: [f32; 2]| (u[0] - v[0]).abs() < 1e-3 && (u[1] - v[1]).abs() < 1e-3;
        // P turns a quarter clockwise about its centre: its top left goes
        // to the top right of the turned box.
        let (mx, my) = (px + pw / 2.0, py + ph / 2.0);
        let pq = &d.quads[0];
        assert!(pq.xf != 0);
        assert!(
            near(at(pq, px, py), [mx + ph / 2.0, my - pw / 2.0]),
            "{:?}",
            at(pq, px, py)
        );
        // C moves 30px along P's turned x axis: down the screen.
        let cq = &d.quads[1];
        assert!(cq.xf != pq.xf);
        assert!(
            near(at(cq, cx, cy), [mx + ph / 2.0, my - pw / 2.0 + 30.0]),
            "{:?}",
            at(cq, cx, cy)
        );
        let kinds: Vec<u32> = d.quads[2..].iter().map(|q| q.kind).collect();
        let g = PaintKind::Glyph as u32;
        assert_eq!(
            kinds,
            [g, g, g, g, PaintKind::Solid as u32],
            "shadow, glyphs, underline"
        );
        assert_eq!(
            d.quads[2].rect[0] - d.quads[4].rect[0],
            1.0,
            "the shadow's offset"
        );
        assert!(
            d.quads[2..].iter().all(|q| q.xf == cq.xf),
            "the text rides its parents"
        );
    }

    /// A fill-tinted image takes the box's fill colour and replaces the
    /// fill under it; an image in its own colours sits over the fill.
    #[test]
    fn a_fill_tinted_image_takes_the_fill_in_its_place() {
        for tint in [crate::style::Tint::Fill, crate::style::Tint::Own] {
            let doc = UiDocument::parse(
                "n.svelte",
                "<meta name=\"component\" content=\"n\">\n\
                 <div id=\"B\"></div>\n\
                 <style>\n#B { width: 40px; height: 20px; background: #ff0000; }\n</style>",
            )
            .unwrap();
            let look = Rc::new(Look::load(
                &[Layer {
                    tokens: crate::look::default_tokens(),
                    surfaces: crate::look::DEFAULT_SURFACES,
                }],
                "dark",
            ));
            let mut tree = Tree::new(look, [Arc::new(doc)]);
            tree.resize(100.0, 100.0);
            tree.mount("n", &[]).unwrap();
            for el in &mut tree.els {
                if el.record.skin.fill[3] > 0.0 {
                    el.record.skin.image = crate::style::Slice {
                        texture: 3,
                        tint,
                        ..Default::default()
                    };
                }
            }
            tree.frame(&og_ui::Fonts::default(), &IconAtlas::default());
            let q = &tree.drawn.quads;
            let kinds: Vec<u32> = q.iter().map(|q| q.kind).collect();
            let image = q
                .iter()
                .find(|q| q.kind == PaintKind::Image as u32)
                .unwrap();
            if tint == crate::style::Tint::Fill {
                assert_eq!(kinds, [PaintKind::Image as u32], "no fill under it");
                assert_eq!(image.color, [1.0, 0.0, 0.0, 1.0], "the fill's colour");
            } else {
                assert_eq!(kinds, [PaintKind::Solid as u32, PaintKind::Image as u32]);
                assert_eq!(image.color, [1.0; 4], "its own colours");
            }
        }
    }

    /// A text's outline paints under its glyphs, each glyph thickened by
    /// half the stroke, in the stroke's colour.
    #[test]
    fn a_text_outline_paints_under_its_glyphs() {
        let doc = UiDocument::parse(
            "o.svelte",
            "<meta name=\"component\" content=\"o\">\n\
             <span id=\"T\">ab</span>\n\
             <style>\n#T { -webkit-text-stroke: 3px #ff0000; paint-order: stroke fill; }\n</style>",
        )
        .unwrap();
        let look = Rc::new(Look::load(
            &[Layer {
                tokens: crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut tree = Tree::new(look, [Arc::new(doc)]);
        tree.resize(200.0, 100.0);
        tree.mount("o", &[]).unwrap();
        let fonts = og_ui::Fonts {
            body: og_ui::UiFont {
                handle: Some(og_draw::FontHandle::from_index(0)),
                metrics: og_text::text::FontMetrics::fixed(0.5),
            },
            ..Default::default()
        };
        tree.frame(&fonts, &IconAtlas::default());
        let q: Vec<&PaintQuad> = tree
            .drawn
            .quads
            .iter()
            .filter(|q| q.kind == PaintKind::Glyph as u32)
            .collect();
        assert_eq!(q.len(), 4, "two outline glyphs, then two fill glyphs");
        assert_eq!(q[0].color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(q[0].rect, q[2].rect, "the outline sits on its glyph");
        assert_eq!(q[0].p[0] - q[2].p[0], 1.5, "half the stroke outside");
        assert_ne!(q[2].color, q[0].color);
    }

    /// An `<img src={…}>` paints its source's texture over its box, once
    /// the host resolves it, and nothing for a source it does not know.
    #[test]
    fn an_image_paints_the_texture_its_source_names() {
        let doc = UiDocument::parse(
            "i.svelte",
            "<meta name=\"component\" content=\"i\">\n\
             <img id=\"I\" src=\"7\" alt=\"\">\n\
             <style>\n#I { width: 30px; height: 20px; }\n</style>",
        )
        .unwrap();
        let look = Rc::new(Look::load(
            &[Layer {
                tokens: crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut tree = Tree::new(look, [Arc::new(doc)]);
        tree.resize(100.0, 100.0);
        tree.mount("i", &[]).unwrap();
        let images = |t: &Tree| {
            t.drawn
                .quads
                .iter()
                .filter(|q| q.kind == PaintKind::Image as u32)
                .map(|q| (q.a, q.rect))
                .collect::<Vec<_>>()
        };
        tree.frame(&og_ui::Fonts::default(), &IconAtlas::default());
        assert!(images(&tree).is_empty(), "no resolver yet");
        let handle = og_draw::TextureHandle::from_index(3);
        tree.set_textures(Box::new(move |h| if h == 7.0 { handle.slot() } else { 0 }));
        tree.frame(&og_ui::Fonts::default(), &IconAtlas::default());
        let (x, y, w, h) = tree.find("I").unwrap();
        assert_eq!(images(&tree), [(handle.to_bits(), [x, y, w, h])]);
    }

    /// A `display: none` element paints nothing, nor does what is under
    /// it: its zero box sits at its parent's corner, where its text would
    /// pile on its siblings' (the quest steps a compact HUD hides).
    #[test]
    fn a_hidden_element_paints_nothing() {
        let doc = UiDocument::parse(
            "h.svelte",
            "<meta name=\"component\" content=\"h\">\n\
             <section id=\"P\"><span id=\"A\">shown</span><div id=\"G\"><span id=\"B\">gone</span></div></section>\n\
             <style>\n#G { display: none; }\n</style>",
        )
        .unwrap();
        let look = Rc::new(Look::load(
            &[Layer {
                tokens: crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut tree = Tree::new(look, [Arc::new(doc)]);
        tree.resize(200.0, 200.0);
        tree.mount("h", &[]).unwrap();
        let fonts = og_ui::Fonts {
            body: og_ui::UiFont {
                handle: Some(og_draw::FontHandle::from_index(0)),
                metrics: og_text::text::FontMetrics::fixed(0.5),
            },
            ..Default::default()
        };
        tree.frame(&fonts, &IconAtlas::default());
        let glyphs = tree
            .drawn
            .quads
            .iter()
            .filter(|q| q.kind == PaintKind::Glyph as u32)
            .count();
        assert_eq!(glyphs, "shown".len(), "only the shown text");
    }

    /// An inset with `auto` sides places the box by the others: an
    /// anchor's `inset: 4px auto auto 4px` sits 4px in from the top left.
    #[test]
    fn an_auto_inset_side_leaves_the_box_to_the_others() {
        let doc = UiDocument::parse(
            "i.svelte",
            "<meta name=\"component\" content=\"i\">\n\
             <div id=\"P\"><div id=\"B\"></div></div>\n\
             <style>\n#P { width: 100px; height: 100px; }\n\
             #B { position: absolute; inset: 4px auto auto 6px; width: 10px; height: 10px; }\n</style>",
        )
        .unwrap();
        let look = Rc::new(Look::load(
            &[Layer {
                tokens: crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut tree = Tree::new(look, [Arc::new(doc)]);
        tree.resize(200.0, 200.0);
        tree.mount("i", &[]).unwrap();
        tree.frame(&og_ui::Fonts::default(), &IconAtlas::default());
        assert_eq!(tree.find("B"), Some((6.0, 4.0, 10.0, 10.0)));
    }

    /// An `auto` margin takes the free space, as flex places it: a row's
    /// last item pushed to its end, a `center` layout centred.
    #[test]
    fn an_auto_margin_takes_the_free_space() {
        let doc = UiDocument::parse(
            "m.svelte",
            "<meta name=\"component\" content=\"m\">\n\
             <div id=\"R\" layout=\"row\"><div id=\"A\"></div><div id=\"B\"></div></div>\n\
             <style>\n#R { width: 200px; }\n#A, #B { width: 20px; height: 10px; }\n\
             #B { margin-left: auto; }\n</style>",
        )
        .unwrap();
        let look = Rc::new(Look::load(
            &[Layer {
                tokens: crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut tree = Tree::new(look, [Arc::new(doc)]);
        tree.resize(400.0, 400.0);
        tree.mount("m", &[]).unwrap();
        tree.frame(&og_ui::Fonts::default(), &IconAtlas::default());
        assert_eq!(tree.find("B").unwrap().0, 180.0, "pushed to the row's end");
    }

    /// Each row of an `{#each}` holds its own state: one row's `state:done`
    /// is not another's.
    /// A press and release over one button clicks it, with its row's key
    /// and the handler its onclick names; a disabled one, or a release
    /// over another, clicks nothing.
    #[test]
    fn a_release_over_the_pressed_button_clicks_it() {
        let doc = UiDocument::parse(
            "b.svelte",
            "<meta name=\"component\" content=\"b\">\n\
             <div class=\"col\">{#each rows as r (r.id)}<button id=\"B\" class=\"b\" disabled={r.off} onclick={use}>go</button>{/each}</div>\n\
             <style>\n.col { display: flex; flex-direction: column; }\n.b { width: 100px; height: 30px; }\n</style>",
        )
        .unwrap();
        let look = Rc::new(Look::load(
            &[Layer {
                tokens: crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut tree = Tree::new(look, [Arc::new(doc)]);
        tree.resize(400.0, 400.0);
        let rows = serde_json::json!([
            {"id": "a", "off": false},
            {"id": "b", "off": false},
            {"id": "c", "off": true},
        ]);
        tree.mount("b", &[("rows", queue::signal(rows))]).unwrap();
        tree.frame(&og_ui::Fonts::default(), &IconAtlas::default());
        let centres: Vec<(f32, f32)> = tree
            .dump()
            .iter()
            .filter(|(id, ..)| *id == "B")
            .map(|(_, [x, y, w, h], ..)| (x + w / 2.0, y + h / 2.0))
            .collect();
        assert_eq!(centres.len(), 3);
        assert!(centres[1].1 > centres[0].1, "the rows stack: {centres:?}");
        let click = |tree: &mut Tree, from: (f32, f32), to: (f32, f32)| {
            let mut out = Vec::new();
            tree.pointer(from.0, from.1, true, false, &mut out);
            tree.pointer(to.0, to.1, false, true, &mut out);
            out
        };
        let out = click(&mut tree, centres[1], centres[1]);
        let seen: Vec<_> = out
            .iter()
            .map(|e| (e.kind, &*e.name, &*e.row_key))
            .collect();
        assert_eq!(seen, [("click", "B", "b"), ("emit", "use", "b")]);
        assert!(
            click(&mut tree, centres[2], centres[2]).is_empty(),
            "disabled"
        );
        assert!(
            click(&mut tree, centres[0], centres[1]).is_empty(),
            "moved off"
        );
    }

    #[test]
    fn each_row_holds_its_own_state() {
        let doc = UiDocument::parse(
            "r.svelte",
            "<meta name=\"component\" content=\"r\">\n\
             <ul>{#each rows as r (r.id)}<li class=\"row\" state:done={r.done}>x</li>{/each}</ul>\n\
             <style>\n.row { opacity: 0.25; }\n.row:state(done) { opacity: 1; }\n</style>",
        )
        .unwrap();
        let look = Rc::new(Look::load(
            &[Layer {
                tokens: crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut tree = Tree::new(look, [Arc::new(doc)]);
        tree.resize(400.0, 400.0);
        let rows = serde_json::json!([
            {"id": "a", "done": true},
            {"id": "b", "done": false},
            {"id": "c", "done": true},
        ]);
        tree.mount("r", &[("rows", queue::signal(rows))]).unwrap();
        tree.frame(&og_ui::Fonts::default(), &IconAtlas::default());
        let opacities: Vec<f32> = tree
            .els
            .iter()
            .filter(|el| {
                let c = &tree.docs[tree.insts[el.inst as usize].doc as usize];
                el.style != NONE
                    && c.doc.nodes[c.style.nodes[el.style as usize].node as usize]
                        .selector_attrs
                        .iter()
                        .any(|(k, _)| k == "state:done")
            })
            .map(|el| el.record.skin.opacity)
            .collect();
        assert_eq!(opacities, [1.0, 0.25, 1.0]);
    }

    /// A condition that reads both a store and the size class holds again
    /// when only the size class moves (a quest's current step on a phone).
    #[test]
    fn a_store_condition_on_the_size_class_follows_a_resize() {
        let doc = UiDocument::parse(
            "c.svelte",
            "<meta name=\"component\" content=\"c\">\n\
             <div><span id=\"S\" state:current={on}>x</span></div>\n\
             <style>\n#S { opacity: 0.25; }\n#S:state(current):size(compact) { opacity: 1; }\n</style>",
        )
        .unwrap();
        let look = Rc::new(Look::load(
            &[Layer {
                tokens: crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut tree = Tree::new(look, [Arc::new(doc)]);
        tree.resize(1280.0, 720.0);
        tree.mount("c", &[("on", queue::signal(true.into()))])
            .unwrap();
        let (fonts, icons) = (og_ui::Fonts::default(), IconAtlas::default());
        let opacity = |t: &Tree| {
            let e = t.els.iter().position(|el| {
                let c = &t.docs[t.insts[el.inst as usize].doc as usize];
                el.style != NONE
                    && c.doc.nodes[c.style.nodes[el.style as usize].node as usize].id == "S"
            });
            t.els[e.unwrap()].record.skin.opacity
        };
        tree.frame(&fonts, &icons);
        assert_eq!(opacity(&tree), 0.25, "wide");
        tree.resize(844.0, 390.0);
        tree.frame(&fonts, &icons);
        assert_eq!(opacity(&tree), 1.0, "a phone on its side is compact");
    }

    /// `t()` reads the host's string tables; a key they lack shows as is.
    #[test]
    fn a_translated_text_shows_its_message() {
        let doc = UiDocument::parse(
            "s.svelte",
            "<meta name=\"component\" content=\"s\">\n\
             <div><span id=\"A\">{t(\"quests.main\")}</span><span id=\"B\">{t(\"quests.none\")}</span></div>",
        )
        .unwrap();
        let look = Rc::new(Look::load(
            &[Layer {
                tokens: crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut tree = Tree::new(look, [Arc::new(doc)]);
        tree.set_strings(
            "en",
            Box::new(|k| (k == "quests.main").then(|| "Main quest".to_string())),
        );
        tree.resize(200.0, 200.0);
        tree.mount("s", &[]).unwrap();
        tree.frame(&og_ui::Fonts::default(), &IconAtlas::default());
        assert_eq!(tree.text("A"), Some("Main quest"));
        assert_eq!(tree.text("B"), Some("quests.none"));
    }

    /// An icon no atlas names draws from the engine's built-in set.
    #[test]
    fn a_built_in_icon_draws() {
        let doc = UiDocument::parse(
            "i.svelte",
            "<meta name=\"component\" content=\"i\">\n\
             <div><og-icon id=\"I\" icon=\"clock\"></og-icon></div>\n\
             <style>\n#I { width: 16px; height: 16px; }\n</style>",
        )
        .unwrap();
        let look = Rc::new(Look::load(
            &[Layer {
                tokens: crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut tree = Tree::new(look, [Arc::new(doc)]);
        tree.resize(200.0, 200.0);
        tree.mount("i", &[]).unwrap();
        tree.frame(&og_ui::Fonts::default(), &IconAtlas::default());
        let icon = tree
            .drawn
            .quads
            .iter()
            .find(|q| q.kind == PaintKind::Icon as u32);
        let (first, count) = crate::icons::icons().find("clock").unwrap();
        assert_eq!(
            icon.map(|q| (q.a, q.b, q.rect[2])),
            Some((first, count, 16.0)),
            "the clock's strokes in its 16 px box"
        );
        assert!(count > 4, "a circle and two hands: {count} segments");
    }

    /// A scroller's `:state(more)` holds while its content runs past its
    /// end, and lets go when the content fits again.
    #[test]
    fn a_scroller_has_more_while_its_content_runs_past_its_end() {
        let doc = UiDocument::parse(
            "s.svelte",
            "<meta name=\"component\" content=\"s\">\n\
             <div id=\"G\" layout=\"stack\">{#if long}<div id=\"Tall\"></div>{/if}<div id=\"F\"></div></div>\n\
             <style>\n#G { height: 40px; overflow-y: auto; }\n#Tall { height: 100px; flex-shrink: 0; }\n\
             #F { height: 4px; opacity: 0.25; }\n#G:state(more) #F { opacity: 1; }\n</style>",
        )
        .unwrap();
        let look = Rc::new(Look::load(
            &[Layer {
                tokens: crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut tree = Tree::new(look, [Arc::new(doc)]);
        tree.resize(200.0, 200.0);
        let long = queue::signal(true.into());
        tree.mount("s", &[("long", long)]).unwrap();
        let fonts = og_ui::Fonts::default();
        let icons = IconAtlas::default();
        let opacity = |t: &Tree| {
            let e = t.els.iter().position(|el| {
                let c = &t.docs[t.insts[el.inst as usize].doc as usize];
                el.style != NONE
                    && c.doc.nodes[c.style.nodes[el.style as usize].node as usize].id == "F"
            });
            t.els[e.unwrap()].record.skin.opacity
        };
        tree.frame(&fonts, &icons);
        assert_eq!(opacity(&tree), 1.0, "100px in a 40px scroller has more");
        queue::set(long, false.into());
        tree.frame(&fonts, &icons);
        assert_eq!(opacity(&tree), 0.25, "4px fits: no more");
        assert_eq!(
            tree.frame(&fonts, &icons),
            Frame::default(),
            "and it settles"
        );
    }
}
