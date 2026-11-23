//! The arena as an AccessKit tree (port of rune's `a11y/tree.rs`, MIT,
//! `third_party/rune-a11y`). What an element is for assistive technology
//! (its role, name, states and range) is the [`Semantics`] the op applier
//! writes from a node's tag, its a11y props and its listeners
//! (CONVENTIONS.md §Accessibility); its box and text are the arena's.
//!
//! An update that read nothing changed costs one stamp comparison. Else one
//! walk over the arena finds the tree's nodes and a fingerprint of all each
//! shows (its semantics, its role, its box, its children, the text it is
//! named by); only the nodes whose fingerprint moved are built and sent.
//! A node the walk no longer finds leaves with its parent's new children,
//! as AccessKit removes it. Node ids are arena ids: the arena's root is the
//! tree's root, a window the size of the viewport.

pub use accesskit;
use accesskit::{
    Action, Affine, AriaCurrent, Live, Node, NodeId, Rect, Role, Toggled, TreeId, TreeInfo,
    TreeUpdate,
};
use rustc_hash::FxHashMap;
use std::hash::{Hash, Hasher};

use crate::arena::{Arena, Id};

/// The tree's root: the arena's root element, the viewport.
pub const ROOT: NodeId = NodeId(0);

/// Text a node is named by from its content stops here, in chars.
const NAME_MAX: usize = 400;

/// What an element is, by the tag it was created with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Kind {
    #[default]
    View,
    Text,
    Image,
    Icon,
    Scroll,
    Input,
    /// A `<Line>` (tag 6).
    Line,
    /// A `<Particles>` (tag 7): decoration, an image when labelled.
    Particles,
    /// A `Show`/`For` anchor: never a node.
    Placeholder,
}

impl Kind {
    /// A `create` op's tag code (CONVENTIONS.md §Ops: 0 View, 1 Text,
    /// 2 Image, 3 Scroll, 4 Input, 5 Icon, 6 Line, 7 Particles).
    pub fn of(tag: u8) -> Kind {
        match tag {
            1 => Kind::Text,
            2 => Kind::Image,
            3 => Kind::Scroll,
            4 => Kind::Input,
            5 => Kind::Icon,
            6 => Kind::Line,
            7.. => Kind::Particles,
            0 => Kind::View,
        }
    }
}

/// One element's semantics, parsed once from its props.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sem {
    pub kind: Kind,
    /// `role`: `Some(None)` for `none`/`presentation` (left out, its
    /// children join its parent).
    pub role: Option<Option<Role>>,
    pub label: Option<Box<str>>,
    pub description: Option<Box<str>>,
    /// An input's text (`value`).
    pub value: Option<Box<str>>,
    /// A range's value as words (`valueText`).
    pub value_text: Option<Box<str>>,
    pub checked: Option<Toggled>,
    pub pressed: Option<Toggled>,
    pub selected: Option<bool>,
    pub expanded: Option<bool>,
    pub busy: bool,
    /// Its value fails the game's checks (`invalid`).
    pub invalid: bool,
    pub hidden: bool,
    pub modal: bool,
    pub current: bool,
    /// On its way out (an exit animation plays): no longer in the tree.
    pub exiting: bool,
    /// An anchored node's placed top left on the surface, and whether its
    /// target is off the screen (`away`): then no longer in the tree.
    pub at: Option<[f32; 2]>,
    pub away: bool,
    pub disabled: bool,
    /// `tabIndex`.
    pub tab_index: Option<i32>,
    /// It listens to `press` or `tap`.
    pub press: bool,
    pub level: Option<u8>,
    pub live: Option<Live>,
    /// `valueMin`, `valueMax`, `valueNow`.
    pub range: [Option<f64>; 3],
    /// A hash of all the above, kept as it changes.
    print: u64,
}

impl Sem {
    /// The keyboard reaches it.
    pub fn focusable(&self) -> bool {
        match self.tab_index {
            Some(t) => t >= 0,
            None => self.press || self.kind == Kind::Input,
        }
    }

    fn reprint(&mut self) {
        let mut h = rustc_hash::FxHasher::default();
        let s = &*self;
        (
            s.kind,
            s.role,
            &s.label,
            &s.description,
            &s.value,
            &s.value_text,
        )
            .hash(&mut h);
        (
            s.checked, s.pressed, s.selected, s.expanded, s.level, s.live,
        )
            .hash(&mut h);
        (
            s.busy,
            s.invalid,
            s.hidden,
            s.exiting,
            s.modal,
            s.current,
            s.disabled,
            s.tab_index,
            s.press,
        )
            .hash(&mut h);
        s.range.map(|v| v.map(f64::to_bits)).hash(&mut h);
        self.print = h.finish();
    }
}

static PLAIN: Sem = Sem {
    kind: Kind::View,
    role: None,
    label: None,
    description: None,
    value: None,
    value_text: None,
    checked: None,
    pressed: None,
    selected: None,
    expanded: None,
    busy: false,
    invalid: false,
    hidden: false,
    modal: false,
    current: false,
    exiting: false,
    at: None,
    away: false,
    disabled: false,
    tab_index: None,
    press: false,
    level: None,
    live: None,
    range: [None; 3],
    print: 0,
};

/// Every element's semantics by arena id, as the op applier writes them.
#[derive(Clone, Debug, Default)]
pub struct Semantics {
    nodes: Vec<Sem>,
    /// Counts changes, for the tree's stamp.
    rev: u64,
    /// How many elements are modal: the walk looks for the topmost only
    /// when one is.
    modals: u32,
    /// The box each transformed node is drawn in on screen (x, y, w, h),
    /// where it is not its layout box: a node scaling in is where it shows.
    drawn: FxHashMap<Id, [f32; 4]>,
}

impl Semantics {
    pub fn get(&self, id: Id) -> &Sem {
        self.nodes.get(id as usize).unwrap_or(&PLAIN)
    }

    /// The boxes the transformed nodes are drawn in after a paint (a
    /// motion's scale, a translate): a node's bounds are where it is drawn,
    /// so a reader's ring lands on it. Changes the stamp only when they did.
    pub fn set_drawn(&mut self, drawn: FxHashMap<Id, [f32; 4]>) {
        if drawn != self.drawn {
            self.drawn = drawn;
            self.rev += 1;
        }
    }

    fn at(&mut self, id: Id) -> &mut Sem {
        let i = id as usize;
        if i >= self.nodes.len() {
            self.nodes.resize(i + 1, Sem::default());
        }
        &mut self.nodes[i]
    }

    /// `create`, `createText` (a [`Kind::Text`]) and `placeholder`.
    pub fn create(&mut self, id: Id, kind: Kind) {
        if self.get(id).modal {
            self.modals -= 1;
        }
        let s = self.at(id);
        *s = Sem {
            kind,
            ..Sem::default()
        };
        s.reprint();
        self.rev += 1;
    }

    /// `release`: the element is gone.
    pub fn release(&mut self, id: Id) {
        if (id as usize) < self.nodes.len() {
            self.create(id, Kind::View);
        }
    }

    /// Whether it is on its way out (removed or released while its exit
    /// plays): the tree leaves it out at once.
    pub fn exiting(&mut self, id: Id, on: bool) {
        if self.get(id).exiting != on {
            let s = self.at(id);
            s.exiting = on;
            s.reprint();
            self.rev += 1;
        }
    }

    /// Where an anchored node was placed (`Some`), off the screen (`None`
    /// while anchored), or back in flow: [`Tree::place_anchors`].
    pub fn place(&mut self, id: Id, at: Option<[f32; 2]>) {
        let s = self.at(id);
        let away = at.is_none();
        if s.away != away || (at.is_some() && s.at != at) {
            s.away = away;
            if at.is_some() {
                s.at = at;
            }
            self.rev += 1;
        }
    }

    /// The node comes back into flow: no longer placed.
    pub fn unplace(&mut self, id: Id) {
        let s = self.at(id);
        if s.at.is_some() || s.away {
            (s.at, s.away) = (None, false);
            self.rev += 1;
        }
    }

    /// Whether it listens to `press` or `tap`.
    pub fn pressable(&mut self, id: Id, on: bool) {
        if self.get(id).press != on {
            let s = self.at(id);
            s.press = on;
            s.reprint();
            self.rev += 1;
        }
    }

    /// `setProp` with its value as JSON text (`null` or empty unsets it;
    /// unquoted text is a string). Answers
    /// whether the name is one of §Accessibility's; others are left to
    /// their owners.
    pub fn prop(&mut self, id: Id, name: &str, value: &str) -> bool {
        let v = Scalar::parse(value);
        let was = self.get(id).clone();
        let s = self.at(id);
        match name {
            "role" => s.role = v.text().map(role_of),
            "label" => s.label = v.text().map(Into::into),
            "description" => s.description = v.text().map(Into::into),
            "value" => s.value = v.words().map(Into::into),
            "valueText" => s.value_text = v.text().map(Into::into),
            "checked" => s.checked = v.toggled(),
            "pressed" => s.pressed = v.toggled(),
            "selected" => s.selected = v.flag(),
            "expanded" => s.expanded = v.flag(),
            "busy" => s.busy = v.flag() == Some(true),
            "invalid" => s.invalid = v.flag() == Some(true),
            "hidden" => s.hidden = v.flag() == Some(true),
            "modal" => s.modal = v.flag() == Some(true),
            "current" => s.current = v.flag() == Some(true),
            "disabled" => s.disabled = v.flag() == Some(true),
            "tabIndex" => s.tab_index = v.number().map(|n| n as i32),
            "level" => {
                s.level = v
                    .number()
                    .filter(|n| (1.0..=6.0).contains(n))
                    .map(|n| n as u8)
            }
            "live" => {
                s.live = match v.text() {
                    Some("polite") => Some(Live::Polite),
                    Some("assertive") => Some(Live::Assertive),
                    Some("off") => Some(Live::Off),
                    _ => None,
                }
            }
            "valueMin" => s.range[0] = v.number(),
            "valueMax" => s.range[1] = v.number(),
            "valueNow" => s.range[2] = v.number(),
            _ => return false,
        }
        s.reprint();
        let modal = s.modal;
        if *s != was {
            self.rev += 1;
            match (was.modal, modal) {
                (false, true) => self.modals += 1,
                (true, false) => self.modals -= 1,
                _ => {}
            }
        }
        true
    }
}

/// A prop's JSON scalar.
enum Scalar<'a> {
    Null,
    Bool(bool),
    Number(f64),
    Text(std::borrow::Cow<'a, str>),
}

impl<'a> Scalar<'a> {
    fn parse(json: &'a str) -> Scalar<'a> {
        let t = json.trim();
        match t {
            "null" | "" => Scalar::Null,
            "true" => Scalar::Bool(true),
            "false" => Scalar::Bool(false),
            _ if t.starts_with('"') => match serde_json::from_str::<std::borrow::Cow<'a, str>>(t) {
                Ok(s) => Scalar::Text(s),
                Err(_) => Scalar::Null,
            },
            _ => t.parse().map_or(Scalar::Text(t.into()), Scalar::Number),
        }
    }

    fn text(&self) -> Option<&str> {
        match self {
            Scalar::Text(s) => Some(s),
            _ => None,
        }
    }

    /// A string, or a number or bool as its words (an input's value).
    fn words(&self) -> Option<String> {
        match self {
            Scalar::Text(s) => Some(s.to_string()),
            Scalar::Number(n) => Some(n.to_string()),
            Scalar::Bool(b) => Some(b.to_string()),
            Scalar::Null => None,
        }
    }

    fn flag(&self) -> Option<bool> {
        match self {
            Scalar::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn number(&self) -> Option<f64> {
        match self {
            Scalar::Number(n) if n.is_finite() => Some(*n),
            _ => None,
        }
    }

    fn toggled(&self) -> Option<Toggled> {
        match self {
            Scalar::Bool(b) => Some(Toggled::from(*b)),
            Scalar::Text(s) if s == "mixed" => Some(Toggled::Mixed),
            _ => None,
        }
    }
}

/// An ARIA role name's AccessKit role; `None` for `none` and
/// `presentation`, a group for one it does not know.
fn role_of(name: &str) -> Option<Role> {
    Some(match name {
        "none" | "presentation" => return None,
        "button" => Role::Button,
        "link" => Role::Link,
        "heading" => Role::Heading,
        "image" | "img" => Role::Image,
        "text" => Role::Label,
        "checkbox" => Role::CheckBox,
        "radio" => Role::RadioButton,
        "switch" => Role::Switch,
        "slider" => Role::Slider,
        "spinbutton" => Role::SpinButton,
        "textbox" => Role::TextInput,
        "combobox" => Role::ComboBox,
        "listbox" => Role::ListBox,
        "option" => Role::ListBoxOption,
        "tab" => Role::Tab,
        "tablist" => Role::TabList,
        "tabpanel" => Role::TabPanel,
        "radiogroup" => Role::RadioGroup,
        "progressbar" => Role::ProgressIndicator,
        "meter" => Role::Meter,
        "timer" => Role::Timer,
        "status" => Role::Status,
        "alert" => Role::Alert,
        "log" => Role::Log,
        "dialog" => Role::Dialog,
        "alertdialog" => Role::AlertDialog,
        "tooltip" => Role::Tooltip,
        "separator" => Role::Splitter,
        "list" => Role::List,
        "listitem" => Role::ListItem,
        "menu" => Role::Menu,
        "menubar" => Role::MenuBar,
        "menuitem" => Role::MenuItem,
        "menuitemcheckbox" => Role::MenuItemCheckBox,
        "menuitemradio" => Role::MenuItemRadio,
        "toolbar" => Role::Toolbar,
        "grid" => Role::Grid,
        "table" => Role::Table,
        "row" => Role::Row,
        "columnheader" => Role::ColumnHeader,
        "rowheader" => Role::RowHeader,
        "gridcell" | "cell" => Role::GridCell,
        "tree" => Role::Tree,
        "treeitem" => Role::TreeItem,
        "searchbox" => Role::SearchInput,
        "scrollbar" => Role::ScrollBar,
        "article" => Role::Article,
        "banner" => Role::Banner,
        "complementary" => Role::Complementary,
        "contentinfo" => Role::ContentInfo,
        "form" => Role::Form,
        "search" => Role::Search,
        "region" => Role::Region,
        "navigation" => Role::Navigation,
        "main" => Role::Main,
        _ => Role::Group,
    })
}

/// A node whose content is one leaf a reader hears as one thing (ARIA's
/// children presentational): named or valued by the text inside it, which
/// is not a node of its own.
fn leaf(role: Role) -> bool {
    matches!(
        role,
        Role::Label
            | Role::Heading
            | Role::Button
            | Role::CheckBox
            | Role::RadioButton
            | Role::Switch
            | Role::Tab
            | Role::ListBoxOption
            | Role::Link
            | Role::MenuItem
            | Role::MenuItemCheckBox
            | Role::MenuItemRadio
            | Role::Image
            | Role::ProgressIndicator
            | Role::Meter
            | Role::Slider
            | Role::SpinButton
            | Role::TextInput
            | Role::Splitter
    )
}

/// Element `id`'s role in the tree, or `None` when it joins its parent:
/// its `role`, else its tag's (rune's `roles::map` and `group`).
fn role(s: &Sem, run: bool) -> Option<Role> {
    if let Some(r) = s.role {
        return r;
    }
    if run {
        return Some(Role::Label);
    }
    match s.kind {
        Kind::Input => Some(Role::TextInput),
        Kind::Text => Some(Role::Label),
        Kind::Scroll => Some(Role::ScrollView),
        Kind::Image | Kind::Icon | Kind::Line | Kind::Particles => {
            s.label.is_some().then_some(Role::Image)
        }
        Kind::Placeholder => None,
        Kind::View if s.modal => Some(Role::Dialog),
        Kind::View if s.press => Some(Role::Button),
        Kind::View => (s.focusable() || s.label.is_some()).then_some(Role::Group),
    }
}

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
            if w.slot != 0 && leaf(up.role) {
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
        if let Some(t) = arena.text_of(s.id) {
            push_words(&mut w, t);
        } else if s.words {
            content(arena, s.id, &mut w);
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
fn content(arena: &Arena, id: Id, out: &mut String) {
    for &c in arena.children(id) {
        if out.chars().count() >= NAME_MAX || !arena.shown(c) {
            continue;
        }
        match arena.text_of(c) {
            Some(t) => push_words(out, t),
            None => content(arena, c, out),
        }
    }
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

/// The arena node a host's request names: the path the web mirror sends
/// back is the node's id.
pub fn target(path: &str) -> Option<Id> {
    path.parse().ok()
}

/// An update as the web shell's mirror takes it (`web/a11y.ts`
/// `A11yUpdate`): `full` (it holds the whole tree), `focus` (the focused
/// node's path, empty for none) and the changed nodes, each with its path
/// (its id as text), ARIA role, name, description, value, states, range,
/// live setting, box in CSS px and its children's paths. The root, path
/// `"0"`, is the canvas; a node no `children` reaches any more is gone.
pub fn web(u: &TreeUpdate) -> serde_json::Value {
    use serde_json::json;
    let nodes: Vec<serde_json::Value> = u
        .nodes
        .iter()
        .map(|(id, n)| {
            let b = n.bounds().unwrap_or_default();
            let f = facts(n);
            json!({
                "path": id.0.to_string(),
                "role": f.role,
                "name": f.name,
                "description": n.description().unwrap_or(""),
                "value": f.value,
                "states": f.states,
                "valueMin": n.min_numeric_value(),
                "valueMax": n.max_numeric_value(),
                "valueNow": n.numeric_value(),
                "level": n.level(),
                "live": match n.live() {
                    Some(Live::Polite) => "polite",
                    Some(Live::Assertive) => "assertive",
                    _ => "",
                },
                "atomic": n.is_live_atomic(),
                "x": b.x0, "y": b.y0, "width": b.x1 - b.x0, "height": b.y1 - b.y0,
                "children": n.children().iter().map(|c| c.0.to_string()).collect::<Vec<_>>(),
            })
        })
        .collect();
    json!({
        "full": u.tree.is_some(),
        "focus": if u.focus == ROOT { String::new() } else { u.focus.0.to_string() },
        "nodes": nodes,
    })
}

/// What a node is, as the web mirror and the probe say it: its ARIA role,
/// its name, its value and its states.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Facts {
    pub role: &'static str,
    pub name: String,
    pub value: String,
    /// `disabled`, `checked`, `mixed`, `pressed`, `selected`, `expanded`,
    /// `collapsed`, `busy`, `modal`, `current`, `focusable`, and `focused`
    /// on the focused node ([`snapshot`]).
    pub states: Vec<&'static str>,
}

/// A built node's [`Facts`].
pub fn facts(n: &Node) -> Facts {
    let role = n.role();
    let mut states = Vec::new();
    match n.toggled() {
        Some(t) if role == Role::Button => {
            if t != Toggled::False {
                states.push("pressed");
            }
        }
        Some(Toggled::True) => states.push("checked"),
        Some(Toggled::Mixed) => states.push("mixed"),
        _ => {}
    }
    let flags = [
        (n.is_disabled(), "disabled"),
        (n.is_selected() == Some(true), "selected"),
        (n.is_expanded() == Some(true), "expanded"),
        (n.is_expanded() == Some(false), "collapsed"),
        (n.is_busy(), "busy"),
        (n.invalid().is_some(), "invalid"),
        (n.is_modal(), "modal"),
        (n.aria_current().is_some(), "current"),
        (n.supports_action(Action::Focus), "focusable"),
    ];
    states.extend(flags.iter().filter(|f| f.0).map(|f| f.1));
    let label = role == Role::Label;
    Facts {
        role: aria(role),
        name: n
            .label()
            .or(if label { n.value() } else { None })
            .unwrap_or("")
            .to_string(),
        value: if label { None } else { n.value() }
            .unwrap_or("")
            .to_string(),
        states,
    }
}

/// The whole tree now, for a reader that asks once (the probe, `og ui
/// check`): each node's facts by arena id, and the focus order, the
/// focusable nodes in tree order. Builds the tree whole; nothing is kept.
pub fn snapshot(
    arena: &Arena,
    sem: &Semantics,
    focus: Option<Id>,
    viewport: Viewport,
) -> (FxHashMap<Id, Facts>, Vec<Id>) {
    let whole = Tree::whole(arena, sem, focus, viewport);
    let mut order = Vec::new();
    let mut by = FxHashMap::default();
    for (id, n) in whole.nodes.iter().filter(|(id, _)| *id != ROOT) {
        let mut f = facts(n);
        if *id == whole.focus {
            f.states.push("focused");
        }
        if f.states.contains(&"focusable") {
            order.push(id.0 as Id);
        }
        by.insert(id.0 as Id, f);
    }
    (by, order)
}

/// The ARIA role a node mirrors as ([`role_of`] backwards).
fn aria(role: Role) -> &'static str {
    match role {
        Role::Window => "",
        Role::Button => "button",
        Role::Link => "link",
        Role::Heading => "heading",
        Role::Image => "img",
        Role::Label => "text",
        Role::CheckBox => "checkbox",
        Role::RadioButton => "radio",
        Role::Switch => "switch",
        Role::Slider => "slider",
        Role::SpinButton => "spinbutton",
        Role::TextInput => "textbox",
        Role::ComboBox => "combobox",
        Role::ListBox => "listbox",
        Role::ListBoxOption => "option",
        Role::Tab => "tab",
        Role::TabList => "tablist",
        Role::TabPanel => "tabpanel",
        Role::RadioGroup => "radiogroup",
        Role::ProgressIndicator => "progressbar",
        Role::Meter => "meter",
        Role::Timer => "timer",
        Role::Status => "status",
        Role::Alert => "alert",
        Role::Log => "log",
        Role::Dialog => "dialog",
        Role::AlertDialog => "alertdialog",
        Role::Tooltip => "tooltip",
        Role::Splitter => "separator",
        Role::List => "list",
        Role::ListItem => "listitem",
        Role::Menu => "menu",
        Role::MenuBar => "menubar",
        Role::MenuItem => "menuitem",
        Role::MenuItemCheckBox => "menuitemcheckbox",
        Role::MenuItemRadio => "menuitemradio",
        Role::Toolbar => "toolbar",
        Role::Grid => "grid",
        Role::Row => "row",
        Role::GridCell => "gridcell",
        Role::Region => "region",
        Role::Navigation => "navigation",
        Role::Main => "main",
        _ => "group",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::Run;
    use rune_taffy::CompactStyle;

    fn measure(_: Id, text: &str, run: &Run, _: crate::arena::Avail) -> (f32, f32) {
        (text.chars().count() as f32 * run.size * 0.5, run.size)
    }

    const VP: Viewport = Viewport {
        width: 1280.0,
        height: 720.0,
        scale: 2.0,
    };

    struct Ui {
        arena: Arena,
        sem: Semantics,
    }

    impl Ui {
        fn new() -> Ui {
            let mut sem = Semantics::default();
            sem.create(0, Kind::View);
            Ui {
                arena: Arena::new(),
                sem,
            }
        }

        fn view(&mut self, parent: Id) -> Id {
            let id = self.arena.element(parent, CompactStyle::default());
            self.sem.create(id, Kind::View);
            id
        }

        fn text(&mut self, parent: Id, text: &str) -> Id {
            let id = self.arena.text(parent, text, 20.0);
            self.sem.create(id, Kind::Text);
            id
        }

        fn lay_out(&mut self) {
            self.arena.layout(VP.width, VP.height, &mut measure);
        }

        fn whole(&self, focus: Option<Id>) -> TreeUpdate {
            Tree::whole(&self.arena, &self.sem, focus, VP)
        }
    }

    fn node(u: &TreeUpdate, id: Id) -> Option<&Node> {
        u.nodes
            .iter()
            .find(|(n, _)| *n == NodeId(id as u64))
            .map(|(_, n)| n)
    }

    fn kids(u: &TreeUpdate, id: Id) -> Vec<u64> {
        node(u, id)
            .unwrap()
            .children()
            .iter()
            .map(|c| c.0)
            .collect()
    }

    /// What an update sent, applied over what was sent before, is the
    /// whole tree now.
    fn check(sent: &mut FxHashMap<u64, Node>, update: &TreeUpdate, whole: &TreeUpdate) {
        for (id, n) in &update.nodes {
            sent.insert(id.0, n.clone());
        }
        let mut reach = vec![ROOT.0];
        let mut seen = Vec::new();
        while let Some(id) = reach.pop() {
            seen.push(id);
            reach.extend(sent[&id].children().iter().map(|c| c.0));
        }
        sent.retain(|id, _| seen.contains(id));
        assert_eq!(sent.len(), whole.nodes.len(), "the whole tree's nodes");
        for (id, n) in &whole.nodes {
            assert_eq!(sent.get(&id.0), Some(n), "node {id:?}");
        }
    }

    #[test]
    fn roles_come_from_tags_and_props_and_a_buttons_text_names_it() {
        let mut ui = Ui::new();
        let bar = ui.view(0);
        let play = ui.view(bar);
        ui.sem.pressable(play, true);
        ui.text(play, "Play");
        ui.text(play, "now");
        let title = ui.text(bar, "Clear");
        ui.sem.prop(title, "role", "\"heading\"");
        ui.sem.prop(title, "level", "1");
        let hp = ui.view(bar);
        assert!(ui.sem.prop(hp, "role", "\"progressbar\""));
        ui.sem.prop(hp, "valueNow", "40");
        ui.sem.prop(hp, "valueMax", "100");
        ui.sem.prop(hp, "label", "Health"); // unquoted text is a string
        assert!(!ui.sem.prop(hp, "src", "\"x.png\""), "not an a11y prop");
        let gone = ui.view(bar);
        ui.sem.prop(gone, "hidden", "true");
        ui.sem.pressable(gone, true);
        let none = ui.arena.element(
            bar,
            CompactStyle::from(taffy::Style {
                display: taffy::Display::None,
                ..taffy::Style::DEFAULT
            }),
        );
        ui.sem.create(none, Kind::View);
        ui.sem.pressable(none, true);
        let blank = ui.text(bar, "  ");
        ui.lay_out();
        let t = ui.whole(Some(play));
        // The bar is a plain box: left out, its children the root's.
        assert_eq!(
            kids(&t, 0),
            [play, title, hp].map(|i| i as u64),
            "no bar, hidden, display: none or blank text"
        );
        assert!(node(&t, bar).is_none() && node(&t, blank).is_none());
        let b = node(&t, play).unwrap();
        assert_eq!(
            (b.role(), b.label(), b.children().len()),
            (Role::Button, Some("Play now"), 0),
            "a button's texts are its name, not nodes"
        );
        assert!(b.supports_action(Action::Click) && b.supports_action(Action::Focus));
        let h = node(&t, title).unwrap();
        assert_eq!(
            (h.role(), h.level(), h.label()),
            (Role::Heading, Some(1), Some("Clear"))
        );
        let p = node(&t, hp).unwrap();
        assert_eq!(
            (
                p.role(),
                p.numeric_value(),
                p.max_numeric_value(),
                p.label()
            ),
            (
                Role::ProgressIndicator,
                Some(40.0),
                Some(100.0),
                Some("Health")
            )
        );
        assert_eq!(t.focus, NodeId(play as u64));
        let root = node(&t, 0).unwrap();
        assert_eq!(root.role(), Role::Window);
        assert_eq!(root.bounds().map(|r| r.x1), Some(1280.0));
    }

    #[test]
    fn an_update_sends_only_what_changed_and_nothing_when_nothing_did() {
        let mut ui = Ui::new();
        let list = ui.view(0);
        ui.sem.prop(list, "role", "\"list\"");
        let rows: Vec<Id> = (0..3)
            .map(|k| {
                let r = ui.view(list);
                ui.sem.prop(r, "role", "\"listitem\"");
                ui.text(r, ["Sword", "Bow", "Axe"][k]);
                r
            })
            .collect();
        let clock = ui.text(0, "06:54");
        ui.lay_out();
        // A host makes a session's client by `Default` (og-host-node).
        let mut tree = Tree::default();
        let mut sent = FxHashMap::default();
        let first = tree.update(&ui.arena, &ui.sem, None, VP).unwrap();
        assert!(
            first.tree.is_some(),
            "the first update holds the tree's info"
        );
        check(&mut sent, &first, &ui.whole(None));
        assert!(
            tree.update(&ui.arena, &ui.sem, None, VP).is_none(),
            "nothing changed"
        );

        // A text of the same size: only its node.
        ui.arena.set_text(clock, "07:15");
        ui.lay_out();
        let u = tree.update(&ui.arena, &ui.sem, None, VP).unwrap();
        assert_eq!(
            u.nodes.iter().map(|n| n.0.0).collect::<Vec<_>>(),
            [clock as u64]
        );
        check(&mut sent, &u, &ui.whole(None));

        // A state on one row: that row alone.
        ui.sem.prop(rows[1], "selected", "true");
        let u = tree.update(&ui.arena, &ui.sem, None, VP).unwrap();
        assert_eq!(
            u.nodes.iter().map(|n| n.0.0).collect::<Vec<_>>(),
            [rows[1] as u64]
        );
        assert_eq!(u.nodes[0].1.is_selected(), Some(true));
        check(&mut sent, &u, &ui.whole(None));

        // Focus alone moves: no node, the new focus.
        ui.sem.prop(rows[2], "tabIndex", "0");
        tree.update(&ui.arena, &ui.sem, None, VP).unwrap();
        let u = tree.update(&ui.arena, &ui.sem, Some(rows[2]), VP).unwrap();
        assert!(u.nodes.is_empty());
        assert_eq!(u.focus, NodeId(rows[2] as u64));

        // A row taken out: its parent's new children, and the rows below
        // it, which moved up.
        ui.arena.detach(rows[0]);
        ui.lay_out();
        let u = tree.update(&ui.arena, &ui.sem, Some(rows[2]), VP).unwrap();
        let ids: Vec<u64> = u.nodes.iter().map(|n| n.0.0).collect();
        assert!(ids.contains(&(list as u64)) && !ids.contains(&(rows[0] as u64)));
        check(&mut sent, &u, &ui.whole(Some(rows[2])));
    }

    #[test]
    fn behind_an_open_modal_the_tree_holds_only_the_modal() {
        let mut ui = Ui::new();
        let hud = ui.view(0);
        let go = ui.view(hud);
        ui.sem.pressable(go, true);
        ui.text(go, "Go");
        let layer = ui.view(0);
        let dialog = ui.view(layer);
        ui.sem.prop(dialog, "modal", "true");
        ui.sem.prop(dialog, "label", "\"Quit?\"");
        let yes = ui.view(dialog);
        ui.sem.pressable(yes, true);
        ui.text(yes, "Yes");
        ui.lay_out();
        let t = ui.whole(None);
        assert_eq!(kids(&t, 0), [dialog as u64], "the page behind it is gone");
        assert_eq!(node(&t, dialog).unwrap().role(), Role::Dialog);
        assert!(node(&t, dialog).unwrap().is_modal());
        assert_eq!(kids(&t, dialog), [yes as u64]);
        // Closed, the page is back.
        let mut tree = Tree::new();
        let mut sent = FxHashMap::default();
        check(
            &mut sent,
            &tree.update(&ui.arena, &ui.sem, None, VP).unwrap(),
            &t,
        );
        ui.sem.prop(dialog, "modal", "false");
        ui.sem.prop(dialog, "hidden", "true");
        let u = tree.update(&ui.arena, &ui.sem, None, VP).unwrap();
        let t = ui.whole(None);
        assert_eq!(kids(&t, 0), [go as u64]);
        check(&mut sent, &u, &t);
    }
}
