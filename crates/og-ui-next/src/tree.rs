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
use og_ui::bind::{Env, Expr, Scope, Screen};
use og_ui::doc::{Lit, NodeKind, PropValue, UiDocument, schema::Kind};
use og_ui::{DrawCmd, DrawItem, IconAtlas, IconCell, Rect};
use rustc_hash::FxHashMap;
use serde_json::Value;
use std::cell::Cell;
use std::rc::Rc;

const NONE: u32 = u32::MAX;

/// The UI's box and input, which `:size()` and `:touch` read.
#[derive(Default)]
pub struct Surface {
    pub size: Cell<(f32, f32)>,
    pub touch: Cell<bool>,
}

impl Env for Surface {
    fn screen(&self, class: Screen) -> bool {
        let (w, h) = self.size.get();
        class.holds(w, h, self.touch.get())
    }
}

/// A document compiled against the look.
struct Compiled {
    doc: UiDocument,
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

/// One instance of a document: the conditions its elements hold.
struct Inst {
    doc: u32,
    holds: Vec<u64>,
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
    /// The element its arena node is a child of (a fixed one: none).
    arena_parent: u32,
    /// The element it inherits from.
    parent: u32,
    inst: u32,
    style: u32,
    kind: Kind,
    leaf: Option<Id>,
    /// Its text, a text leaf's or an icon's name.
    text: Box<str>,
    hidden: u16,
    dead: bool,
    queued: bool,
    record: Record,
    eff: Inherit,
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
    library: FxHashMap<Box<str>, UiDocument>,
    insts: Vec<Inst>,
    els: Vec<El>,
    ifs: Vec<IfBlock>,
    eaches: Vec<EachBlock>,
    arena: Arena,
    root: Inherit,
    ops: Vec<Op>,
    dirty: Vec<u32>,
    paint: bool,
    pub items: Vec<DrawItem>,
    /// Bumped each time `items` change.
    pub generation: u64,
    abs: Vec<(f32, f32, bool)>,
}

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

/// The signals of `scope` an expression reads, by their root name.
fn reads(e: &Expr, scope: &[(Box<str>, u32)]) -> Reads {
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
    out
}

/// `e` over `reads`, as a derived signal.
fn derive(e: Expr, reads: Reads, env: Rc<Surface>) -> u32 {
    queue::derive(move || {
        let mut vm = serde_json::Map::new();
        for (n, s) in &reads {
            vm.insert(n.to_string(), queue::read(*s));
        }
        let vm = Value::Object(vm);
        e.eval(&Scope {
            vm: &vm,
            locals: Vec::new(),
            env: &*env,
        })
    })
}

/// What a fixed condition reads: the surface, and the instance's
/// scrollers with more below.
struct Fixed<'a> {
    surface: &'a Surface,
    more: &'a [u32],
}

impl og_ui::bind::Env for Fixed<'_> {
    fn screen(&self, class: Screen) -> bool {
        self.surface.screen(class)
    }
    fn interaction_at(&self, node: usize, state: og_ui::bind::InteractionState) -> bool {
        state == og_ui::bind::InteractionState::More && self.more.contains(&(node as u32))
    }
}

fn eval_static(e: &Expr, env: &Fixed) -> bool {
    og_ui::bind::truthy(&e.eval(&Scope {
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
            state: og_ui::bind::InteractionState::More,
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
    /// An empty tree over `look`, with the documents it may use.
    pub fn new(look: Rc<Look>, library: impl IntoIterator<Item = UiDocument>) -> Tree {
        let mut root = look.root.clone();
        root.text.set = 0;
        Tree {
            id: NEXT_TREE.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            env: Rc::new(Surface::default()),
            root: Inherit {
                size: root.font_size,
                ink: root.ink,
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
            insts: Vec::new(),
            els: Vec::new(),
            ifs: Vec::new(),
            eaches: Vec::new(),
            arena: Arena::new(),
            ops: Vec::new(),
            dirty: Vec::new(),
            paint: true,
            items: Vec::new(),
            generation: 0,
            abs: Vec::new(),
        }
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
        let style = DocStyle::compile(&doc, &self.look.tokens, &self.look.surfaces);
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
        self.instance(name, NONE, 0, scope, &mut Vec::new())
            .ok_or_else(|| format!("og-ui-next: no document {name}"))?;
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
        let conds = self.docs[d as usize].style.conditions.len();
        self.insts.push(Inst {
            doc: d,
            holds: vec![0; conds.div_ceil(64)],
            more: Vec::new(),
        });
        // The document's own derived values, in order.
        let derived = self.docs[d as usize].doc.derived.clone();
        for (n, e) in derived {
            let r = reads(&e, &scope);
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
                let r = reads(&e, scope);
                queue::bind(self.id, Target::Shown { node: b }, e, r, self.env.clone());
            }
            NodeKind::For {
                var,
                index,
                list,
                key,
                ..
            } => {
                let r = reads(&list, scope);
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
                    Expr::Path("keys".into()),
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
                            let r = reads(e, scope);
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
            let mut row_scope = scope.clone();
            let root = queue::owned(|| {
                let (var2, key2, env) = (var.to_string(), key.clone(), self.env.clone());
                let row = queue::derive(move || {
                    let Value::Array(items) = queue::read(list) else {
                        return Value::Null;
                    };
                    items
                        .into_iter()
                        .find(|item| {
                            key2.eval(&Scope {
                                vm: &Value::Null,
                                locals: vec![(var2.as_str(), item.clone())],
                                env: &*env,
                            }) == k
                        })
                        .unwrap_or(Value::Null)
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
        let record = self.fold(inst, style);
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
            _ => None,
        };
        let leaf = (text.is_some() && kind != Kind::Icon).then(|| self.arena.text(node, "", 16.0));
        let children = n.children.clone();
        self.els.push(El {
            node,
            arena_parent: arena_parent_el,
            parent,
            inst,
            style,
            kind,
            leaf,
            text: "".into(),
            hidden: 0,
            dead: false,
            queued: false,
            record,
            eff: self.root,
        });
        for &b in chain.iter() {
            self.ifs[b as usize].tops.push(e);
        }
        self.inherit(e);
        match text {
            Some(PropValue::Lit(l)) => self.set_text(e, &queue::text_of(&lit(&l))),
            Some(PropValue::Bind(x, _)) => {
                let r = reads(&x, scope);
                queue::bind(self.id, Target::Text { node: e }, x, r, self.env.clone());
            }
            _ => {}
        }
        // Its conditions that read stores, each a binding.
        if style != NONE {
            let c = &self.docs[d as usize];
            let [at, len] = c.style.nodes[style as usize].spans;
            let conds: Vec<(u32, Expr)> = c.style.spans[at as usize..(at + len) as usize]
                .iter()
                .filter(|s| s.condition != u32::MAX && !c.fixed[s.condition as usize])
                .map(|s| {
                    (
                        s.condition,
                        c.style.conditions[s.condition as usize].clone(),
                    )
                })
                .collect();
            for (cond, x) in conds {
                let r = reads(&x, scope);
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

    fn fold(&self, inst: u32, style: u32) -> Record {
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
                ins.holds[(cond / 64) as usize] & (1 << (cond % 64)) != 0
            }
        })
    }

    fn set_text(&mut self, e: u32, text: &str) {
        let el = &mut self.els[e as usize];
        if &*el.text == text {
            return;
        }
        el.text = text.into();
        if let Some(leaf) = el.leaf {
            self.arena.set_text(leaf, text);
        }
        self.paint = true;
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
            ink: if r.inked { r.ink } else { from.ink },
            text: r.text.over(&from.text),
            opacity: from.opacity * r.opacity,
        };
        let mut layout = r.layout_in([0.0; 4]);
        if el.hidden > 0 {
            let mut s = taffy::Style::from(&layout);
            s.display = taffy::Display::None;
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
        for c in e + 1..self.els.len() as u32 {
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
        if moved || self.els.is_empty() {
            for e in 0..self.els.len() as u32 {
                self.queue(e);
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
                    let inst = self.els[node as usize].inst as usize;
                    let w = &mut self.insts[inst].holds[(cond / 64) as usize];
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
                if el.dead {
                    continue;
                }
                let r = self.fold(el.inst, el.style);
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

    /// Lays out what is dirty and paints again when anything changed:
    /// whether it laid out, and whether it painted.
    pub fn present(&mut self, fonts: &og_ui::Fonts, icons: &IconAtlas) -> (bool, bool) {
        let (w, h) = self.env.size.get();
        let m = &fonts.body.metrics;
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
        let painted = laid_out || self.paint;
        if painted {
            self.repaint(fonts, icons);
            self.paint = false;
        }
        (laid_out, painted)
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

    fn repaint(&mut self, fonts: &og_ui::Fonts, icons: &IconAtlas) {
        let (vw, vh) = self.env.size.get();
        let clip = Rect::new(0.0, 0.0, vw, vh);
        self.items.clear();
        self.abs.clear();
        let font = fonts.body.handle;
        let m = &fonts.body.metrics;
        for e in 0..self.els.len() {
            let el = &self.els[e];
            let b = self.arena.box_of(el.node);
            let (px, py, pv) = if el.arena_parent == NONE {
                (0.0, 0.0, true)
            } else {
                self.abs[el.arena_parent as usize]
            };
            let visible = pv && !el.dead && el.hidden == 0 && el.eff.opacity > 0.0;
            let (x, y) = (px + b.x, py + b.y);
            self.abs.push((x, y, visible));
            if !visible {
                continue;
            }
            let r = &el.record;
            let alpha = el.eff.opacity;
            if r.fill[3] > 0.0 {
                let mut c = r.fill;
                c[3] *= alpha;
                self.items.push(item(DrawCmd::Quad {
                    rect: Rect::new(x, y, b.w, b.h),
                    radius: r.radius.min(b.w.min(b.h) / 2.0),
                    color: c,
                    clip,
                }));
            }
            let mut ink = el.eff.ink;
            ink[3] *= alpha;
            if el.kind == Kind::Icon && !el.text.is_empty() {
                let size = b.w.min(b.h).max(1.0);
                match icons.find_drawn(&el.text) {
                    Some((IconCell::Glyph(f, ch), _)) => {
                        let (asc, desc) = icons
                            .font
                            .as_ref()
                            .map_or((0.8, -0.2), |f| (f.metrics.ascender, f.metrics.descender));
                        let top = y + (b.h - (asc - desc) * size) / 2.0;
                        self.items.push(item(text_cmd(
                            0,
                            x + (b.w - size) / 2.0,
                            top + asc * size,
                            size,
                            ch.to_string(),
                            ink,
                            f,
                            clip,
                            0.0,
                        )));
                    }
                    Some((IconCell::Image(t, uv), pictures)) => {
                        self.items.push(item(DrawCmd::Image {
                            rect: Rect::new(
                                x + (b.w - size) / 2.0,
                                y + (b.h - size) / 2.0,
                                size,
                                size,
                            ),
                            texture: Some(t),
                            uv,
                            color: if pictures {
                                [1.0, 1.0, 1.0, alpha]
                            } else {
                                ink
                            },
                            clip,
                        }))
                    }
                    _ => {}
                }
            }
            if let (Some(leaf), Some(font)) = (el.leaf, font)
                && !el.text.is_empty()
            {
                let lb = self.arena.box_of(leaf);
                let (lx, ly) = (x + lb.x, y + lb.y);
                let t = &el.eff.text;
                let size = el.eff.size;
                let tracking = t.tracking(size);
                let line = t.line_height(size);
                let lines = if t.nowrap {
                    vec![og_text::text::Line {
                        width: og_text::text::widest_line(
                            &el.text,
                            m,
                            size,
                            tracking,
                            og_text::shape::Features::NONE,
                        ),
                        text: el.text.to_string(),
                    }]
                } else {
                    og_text::text::break_lines(
                        &el.text,
                        m,
                        size,
                        tracking,
                        og_text::shape::Features::NONE,
                        Some(lb.w + 0.5),
                    )
                };
                let glyph = (m.ascender - m.descender) * size;
                let weight = if t.weight >= 600 { size * 0.03 } else { 0.0 };
                for (k, l) in lines.into_iter().enumerate() {
                    let top = ly + k as f32 * line + (line - glyph) / 2.0;
                    let dx = match t.align {
                        1 => (lb.w - l.width) / 2.0,
                        2 => lb.w - l.width,
                        _ => 0.0,
                    };
                    self.items.push(item(text_cmd(
                        e as u64 + 1,
                        lx + dx,
                        top + m.ascender * size,
                        size,
                        l.text,
                        ink,
                        font,
                        clip,
                        if size > 0.0 { tracking / size } else { 0.0 },
                    )));
                    if let Some(DrawItem {
                        cmd:
                            DrawCmd::Text {
                                weight: w, bounds, ..
                            },
                        ..
                    }) = self.items.last_mut()
                    {
                        *w = weight;
                        *bounds = Rect::new(lx + dx, top, l.width, line);
                    }
                }
            }
        }
        self.generation += 1;
    }

    /// An element's box by its document id (`#Time`), for checks.
    pub fn find(&self, id: &str) -> Option<(f32, f32, f32, f32)> {
        let e = self.els.iter().position(|el| {
            let ins = &self.insts[el.inst as usize];
            let c = &self.docs[ins.doc as usize];
            el.style != NONE && c.doc.nodes[c.style.nodes[el.style as usize].node as usize].id == id
        })?;
        let (x, y, _) = *self.abs.get(e)?;
        let b = self.arena.box_of(self.els[e].node);
        Some((x, y, b.w, b.h))
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

fn item(cmd: DrawCmd) -> DrawItem {
    DrawItem {
        level: og_ui::LAYER_BASE,
        cmd,
        transform: og_ui::Affine::IDENTITY,
    }
}

#[allow(clippy::too_many_arguments)]
fn text_cmd(
    source: u64,
    x: f32,
    baseline: f32,
    size: f32,
    text: String,
    color: Rgba,
    font: og_draw::FontHandle,
    clip: Rect,
    tracking: f32,
) -> DrawCmd {
    DrawCmd::Text {
        source,
        x,
        baseline,
        size,
        text,
        color,
        font,
        bounds: Rect::new(x, baseline - size, 0.0, size),
        clip,
        tracking,
        weight: 0.0,
        glow: None,
        placements: Vec::new(),
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
                tokens: &crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut tree = Tree::new(look, [doc]);
        tree.resize(1280.0, 720.0);
        tree.mount("o", &[]).unwrap();
        tree.frame(&og_ui::Fonts::default(), &IconAtlas::default());
        let face = tree.find("Face").unwrap();
        let art = tree.find("Art").unwrap();
        assert_eq!(art, face, "the art covers its face");
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
                tokens: &crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut tree = Tree::new(look, [doc]);
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
            t.els[e.unwrap()].record.opacity
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
