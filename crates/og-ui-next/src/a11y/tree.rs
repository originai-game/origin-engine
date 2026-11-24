//! Incremental AccessKit tree builder and diff emitter.

use super::*;

/// The surface a tree is built for: its size in CSS px and its scale to
/// physical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
    pub scale: f32,
}

/// What an update read last time: when none of it moved, nothing it builds
/// can have.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Stamp {
    arena: u64,
    sem: u64,
    focus: Option<Id>,
    viewport: [u32; 3],
}

/// A node the walk found.
#[derive(Clone, Copy, Debug)]
struct Slot {
    id: Id,
    /// Index of the nearest ancestor slot (the root's: itself).
    parent: usize,
    role: Role,
    /// Its box in the viewport, CSS px.
    rect: [f32; 4],
    /// Its own fingerprint, then its children's and content's mixed in.
    print: u64,
    /// The words of the text inside a leaf.
    words: bool,
}

/// An element's place while the tree walks.
#[derive(Clone, Copy, Debug)]
struct Walk {
    id: Id,
    /// The slot it joins.
    slot: usize,
    /// Its parent's origin in the viewport.
    origin: [f32; 2],
    /// Inside the topmost modal (or no modal is open).
    in_modal: bool,
}

/// The tree an assistive client was last sent: each node's fingerprint and
/// the update that last saw it. A new one ([`Tree::new`], or
/// `Default`) sends the whole tree first.
#[derive(Clone, Debug)]
pub struct Tree {
    sent: FxHashMap<u64, (u64, u32)>,
    pass: u32,
    stamp: Option<Stamp>,
    /// The focus last sent (its id).
    focus: u64,
    /// The next update holds the whole tree.
    full: bool,
    slots: Vec<Slot>,
    stack: Vec<Walk>,
}

impl Default for Tree {
    fn default() -> Tree {
        Tree::new()
    }
}

impl Tree {
    /// A tree whose first update holds all of it.
    pub fn new() -> Tree {
        Tree {
            sent: FxHashMap::default(),
            pass: 0,
            stamp: None,
            focus: 0,
            full: true,
            slots: Vec::new(),
            stack: Vec::new(),
        }
    }

    /// The client asked again: the next update holds the whole tree.
    pub fn reset(&mut self) {
        self.full = true;
        self.stamp = None;
    }

    /// The update to send, after the arena laid out, when something the
    /// tree shows changed since the last one. `focus`: the element the
    /// keyboard's focus is on.
    pub fn update(
        &mut self,
        arena: &Arena,
        sem: &Semantics,
        focus: Option<Id>,
        viewport: Viewport,
    ) -> Option<TreeUpdate> {
        // Boxes come from the layout: the frame after it sends the update.
        if !arena.settled() {
            return None;
        }
        let stamp = Stamp {
            arena: arena.rev(),
            sem: sem.rev,
            focus,
            viewport: [viewport.width, viewport.height, viewport.scale].map(f32::to_bits),
        };
        if !self.full && self.stamp == Some(stamp) {
            return None;
        }
        self.stamp = Some(stamp);
        let focused = self.walk(arena, sem, focus, viewport);
        let full = std::mem::take(&mut self.full);
        let sent = &self.sent;
        let changed: Vec<bool> = self
            .slots
            .iter()
            .map(|s| full || sent.get(&(s.id as u64)).is_none_or(|&(p, _)| p != s.print))
            .collect();
        let nodes = build(&self.slots, &changed, arena, sem, viewport);
        self.pass = self.pass.wrapping_add(1);
        let pass = self.pass;
        for s in &self.slots {
            self.sent.insert(s.id as u64, (s.print, pass));
        }
        self.sent.retain(|_, s| s.1 == pass);
        if !full && nodes.is_empty() && focused.0 == self.focus {
            return None;
        }
        self.focus = focused.0;
        Some(TreeUpdate {
            nodes,
            tree: full.then(info),
            tree_id: TreeId::ROOT,
            focus: focused,
        })
    }

    /// The whole tree now, as a client that attached would get it.
    pub fn whole(
        arena: &Arena,
        sem: &Semantics,
        focus: Option<Id>,
        viewport: Viewport,
    ) -> TreeUpdate {
        let mut t = Tree::new();
        let focused = t.walk(arena, sem, focus, viewport);
        let nodes = build(&t.slots, &vec![true; t.slots.len()], arena, sem, viewport);
        TreeUpdate {
            nodes,
            tree: Some(info()),
            tree_id: TreeId::ROOT,
            focus: focused,
        }
    }

    /// One pass over the arena: which elements are nodes, where they are,
    /// and their fingerprints, into `slots`. Answers the focused node.
    fn walk(&mut self, arena: &Arena, sem: &Semantics, focus: Option<Id>, v: Viewport) -> NodeId {
        // Behind an open modal the tree holds only the topmost one and what
        // contains it.
        let modal = (sem.modals > 0)
            .then(|| topmost_modal(arena, sem, &mut self.stack))
            .flatten();
        let holds: Vec<Id> =
            std::iter::successors(modal.and_then(|m| arena.parent(m)), |&p| arena.parent(p))
                .collect();
        self.slots.clear();
        let vp = [v.width, v.height, v.scale].map(|f| f.to_bits() as u64);
        self.slots.push(Slot {
            id: 0,
            parent: 0,
            role: Role::Window,
            rect: [0.0, 0.0, v.width, v.height],
            print: vp.into_iter().fold(sem.get(0).print, mix),
            words: false,
        });
        let mut focused = ROOT;
        self.stack.clear();
        // Before the layout gives the elements their boxes, the root alone.
        if !arena.settled() {
            return focused;
        }
        for &c in arena.children(0).iter().rev() {
            self.stack.push(Walk {
                id: c,
                slot: 0,
                origin: [0.0; 2],
                in_modal: modal.is_none(),
            });
        }
        while let Some(w) = self.stack.pop() {
            let s = sem.get(w.id);
            if s.hidden || s.exiting || s.away || !arena.shown(w.id) {
                continue;
            }
            let in_modal = w.in_modal || modal == Some(w.id);
            if !in_modal && !holds.contains(&w.id) {
                continue;
            }
            let b = arena.box_of(w.id);
            let rect = match s.at {
                Some([x, y]) => [x, y, b.w, b.h],
                None => [w.origin[0] + b.x, w.origin[1] + b.y, b.w, b.h],
            };
            // Where it is drawn; its children lie out from its layout box.
            let shown = sem.drawn.get(&w.id).copied().unwrap_or(rect);
            let run = arena.text_of(w.id);
            let up = self.slots[w.slot];
            let mut slot = w.slot;
            // A pressable span in a paragraph is a link of its own, named
            // by its runs (the paragraph draws them, so they take no box).
            let link = w.slot != 0 && leaf(up.role) && s.kind == Kind::Text && s.press;
            if link {
                let mut h = rustc_hash::FxHasher::default();
                (s.print, Role::Link as u8).hash(&mut h);
                shown.map(|f| (f * 32.0).round() as i32).hash(&mut h);
                self.slots.push(Slot {
                    id: w.id,
                    parent: w.slot,
                    role: Role::Link,
                    rect: shown,
                    print: mix(h.finish(), runs_print(arena, w.id)),
                    words: true,
                });
                if focus == Some(w.id) {
                    focused = NodeId(w.id as u64);
                }
                continue;
            } else if w.slot != 0 && leaf(up.role) {
                // Inside a leaf: its text is the leaf's content.
                if let Some(t) = run.filter(|t| !t.trim().is_empty()) {
                    let up = &mut self.slots[w.slot];
                    up.print = mix(up.print, hash_str(t));
                    up.words = true;
                }
                if focus == Some(w.id) {
                    focused = NodeId(up.id as u64);
                }
            } else if let Some(r) = role(s, run.is_some()) {
                let mut h = rustc_hash::FxHasher::default();
                (s.print, r as u8, run).hash(&mut h);
                shown.map(|f| (f * 32.0).round() as i32).hash(&mut h);
                self.slots.push(Slot {
                    id: w.id,
                    parent: w.slot,
                    role: r,
                    rect: shown,
                    print: h.finish(),
                    words: run.is_some_and(|t| !t.trim().is_empty()),
                });
                slot = self.slots.len() - 1;
                if focus == Some(w.id) {
                    focused = NodeId(w.id as u64);
                }
            }
            for &c in arena.children(w.id).iter().rev() {
                self.stack.push(Walk {
                    id: c,
                    slot,
                    origin: [rect[0], rect[1]],
                    in_modal,
                });
            }
        }
        // A text that says nothing is no node.
        let empty = |s: &Slot| s.role == Role::Label && !s.words && sem.get(s.id).label.is_none();
        if self.slots.iter().any(empty) {
            let mut keep = vec![0usize; self.slots.len()];
            let mut out = 0;
            for k in 0..self.slots.len() {
                if k > 0 && empty(&self.slots[k]) {
                    keep[k] = usize::MAX;
                    if NodeId(self.slots[k].id as u64) == focused {
                        focused = ROOT;
                    }
                    continue;
                }
                let mut s = self.slots[k];
                s.parent = keep[s.parent];
                keep[k] = out;
                self.slots[out] = s;
                out += 1;
            }
            self.slots.truncate(out);
        }
        // Each node's children are part of what it shows.
        let mut kids = vec![0u64; self.slots.len()];
        for s in self.slots.iter().skip(1) {
            kids[s.parent] = mix(kids[s.parent], s.id as u64 + 1);
        }
        for (s, k) in self.slots.iter_mut().zip(kids) {
            s.print = mix(s.print, k);
        }
        focused
    }
}

/// The last shown modal in tree order (the one drawn over the others).
fn topmost_modal(arena: &Arena, sem: &Semantics, stack: &mut Vec<Walk>) -> Option<Id> {
    let mut found = None;
    stack.clear();
    stack.push(Walk {
        id: 0,
        slot: 0,
        origin: [0.0; 2],
        in_modal: false,
    });
    while let Some(w) = stack.pop() {
        let s = sem.get(w.id);
        if s.hidden || s.exiting || s.away || !arena.shown(w.id) {
            continue;
        }
        if s.modal {
            found = Some(w.id);
        }
        for &c in arena.children(w.id).iter().rev() {
            stack.push(Walk { id: c, ..w });
        }
    }
    found
}

/// The nodes of `slots` whose `changed` flag is set.
fn build(
    slots: &[Slot],
    changed: &[bool],
    arena: &Arena,
    sem: &Semantics,
    v: Viewport,
) -> Vec<(NodeId, Node)> {
    let mut children: Vec<Vec<NodeId>> = vec![Vec::new(); slots.len()];
    for s in slots.iter().skip(1) {
        if changed[s.parent] {
            children[s.parent].push(NodeId(s.id as u64));
        }
    }
    let mut out = Vec::with_capacity(changed.iter().filter(|c| **c).count());
    for (k, s) in slots.iter().enumerate().filter(|(k, _)| changed[*k]) {
        let mut node = Node::new(s.role);
        node.set_children(std::mem::take(&mut children[k]));
        if k == 0 {
            if v.scale != 1.0 {
                node.set_transform(Affine::scale(f64::from(v.scale)));
            }
            node.set_bounds(rect(s.rect));
            out.push((ROOT, node));
            continue;
        }
        describe(&mut node, s, arena, sem.get(s.id));
        out.push((NodeId(s.id as u64), node));
    }
    out
}

/// A node's name, value, states, actions and bounds.
fn describe(node: &mut Node, s: &Slot, arena: &Arena, m: &Sem) {
    let role = s.role;
    node.set_bounds(rect(s.rect));
    let words = || {
        let mut w = String::new();
        if let Some(t) = arena
            .text_of(s.id)
            .filter(|_| m.kind != Kind::Text || !m.press)
        {
            push_words(&mut w, t);
        } else if s.words {
            content(arena, s.id, &mut w, m.kind == Kind::Text);
        }
        w
    };
    match &m.label {
        Some(l) => node.set_label(&**l),
        None if role == Role::Label => node.set_value(words()),
        None if leaf(role) && s.words => node.set_label(words()),
        None => {}
    }
    if let Some(d) = &m.description {
        node.set_description(&**d);
    }
    if role == Role::TextInput {
        node.set_value(m.value.as_deref().unwrap_or(""));
    } else if let Some(t) = &m.value_text {
        node.set_value(&**t);
    }
    if let Some(t) = m.checked.or(m.pressed) {
        node.set_toggled(t);
    }
    if let Some(b) = m.selected {
        node.set_selected(b);
    }
    if let Some(b) = m.expanded {
        node.set_expanded(b);
    }
    if let Some(l) = m.level {
        node.set_level(l as usize);
    }
    let live = m.live.or(match role {
        Role::Status | Role::Log => Some(Live::Polite),
        Role::Alert => Some(Live::Assertive),
        _ => None,
    });
    if let Some(l) = live.filter(|l| *l != Live::Off) {
        node.set_live(l);
        if matches!(role, Role::Status | Role::Alert) {
            node.set_live_atomic();
        }
    }
    let [min, max, now] = m.range;
    if let Some(n) = now {
        node.set_numeric_value(n);
    }
    if let Some(n) = min {
        node.set_min_numeric_value(n);
    }
    if let Some(n) = max {
        node.set_max_numeric_value(n);
    }
    if m.busy {
        node.set_busy();
    }
    if m.invalid {
        node.set_invalid(accesskit::Invalid::True);
    }
    if m.modal {
        node.set_modal();
    }
    if m.current {
        node.set_aria_current(AriaCurrent::True);
    }
    if m.disabled {
        node.set_disabled();
        return;
    }
    if m.focusable() {
        node.add_action(Action::Focus);
    }
    if m.press {
        node.add_action(Action::Click);
    }
    if matches!(role, Role::Slider | Role::SpinButton) {
        node.add_action(Action::Increment);
        node.add_action(Action::Decrement);
    }
}

/// The text inside `id`, in tree order, its runs' words one space apart.
/// The words of `id`'s content: its shown texts, or with `runs` (a span
/// of a paragraph, which draws them) all of them.
fn content(arena: &Arena, id: Id, out: &mut String, runs: bool) {
    for &c in arena.children(id) {
        if out.chars().count() >= NAME_MAX || !(runs || arena.shown(c)) {
            continue;
        }
        match arena.text_of(c) {
            Some(t) => push_words(out, t),
            None => content(arena, c, out, runs),
        }
    }
}

/// A print of a span's runs' texts, so a change of them sends it again.
fn runs_print(arena: &Arena, id: Id) -> u64 {
    let mut w = String::new();
    content(arena, id, &mut w, true);
    hash_str(&w)
}

fn push_words(out: &mut String, t: &str) {
    for word in t.split_whitespace() {
        if out.chars().count() >= NAME_MAX {
            return;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
    }
}

fn info() -> TreeInfo {
    TreeInfo {
        root: ROOT,
        toolkit_name: Some("og-ui".into()),
        toolkit_version: Some(env!("CARGO_PKG_VERSION").into()),
    }
}

fn hash_str(s: &str) -> u64 {
    let mut h = rustc_hash::FxHasher::default();
    s.hash(&mut h);
    h.finish()
}

/// Folds `v` into `h`, order-sensitively.
fn mix(h: u64, v: u64) -> u64 {
    (h.rotate_left(5) ^ v).wrapping_mul(0x517c_c1b7_2722_0a95)
}

/// A box in CSS px, to a 32nd of one (what the fingerprint reads).
fn rect(r: [f32; 4]) -> Rect {
    let round = |v: f32| (f64::from(v) * 32.0).round() / 32.0;
    let (x, y) = (round(r[0]), round(r[1]));
    Rect {
        x0: x,
        y0: y,
        x1: x + round(r[2].max(0.0)),
        y1: y + round(r[3].max(0.0)),
    }
}
