//! The accessibility tree: every element's role, name, value and states,
//! with its bounds, built with the frame from the same nodes the UI draws
//! and takes input on, so it cannot drift from what is on screen.
//!
//! The probe's `dump_ui` carries it on each element (`role`, `name`,
//! `value`, `states`) and [`crate::Ui::a11y`] answers it as a tree of the
//! elements that have a role (layout containers and spacers have none), for
//! a shell to mirror into the page for screen readers.
//!
//! Focus order is the tree's order: keyboard Tab and the pads walk the
//! elements marked `focusable` in the order they appear here (the
//! drawing order: the page, then each layer over it), within the topmost
//! dialog while one is open. [`A11yTree::focus_order`] lists them.

#[cfg(not(target_arch = "wasm32"))]
pub mod native_a11y;
use crate::layout::{Node, Widget};

/// Semantic information from render nodes, shared by navigation and diagnostics.
pub(crate) fn node_info(
    nodes: &[Node],
    kinds: &[(&'static str, bool)],
    i: usize,
) -> (&'static str, String, String, String, Vec<String>) {
    let n = &nodes[i];
    let (kind, anonymous) = kinds.get(i).copied().unwrap_or(("", true));
    let role = role_of(kind, n);
    let none = || ("", String::new(), String::new(), String::new(), Vec::new());
    if role.is_empty() {
        return none();
    }
    let textbox = role == "textbox";
    let mut name = n.a11y.name.clone().unwrap_or_default();
    let mut description = n.a11y.description.clone().unwrap_or_default();
    if name.is_empty() && !textbox {
        name = n.probe_text.clone();
    }
    if name.is_empty() && role == "image" && !n.nav {
        return none();
    }
    if name.is_empty() && (n.el.interactive || n.nav) && !textbox {
        name = text_within(nodes, i);
    }
    let labelled = role != "text" && role != "heading";
    if name.is_empty()
        && labelled
        && n.parent != crate::layout::NONE
        && nodes[n.parent].layout.used.direction == crate::layout::Direction::Row
    {
        let siblings = &nodes[n.parent].children;
        if let Some(at) = siblings.iter().position(|&c| c == i)
            && at > 0
            && kinds[siblings[at - 1]].0 == "label"
        {
            name = nodes[siblings[at - 1]].probe_text.clone();
        }
    }
    if name.is_empty() && !description.is_empty() {
        name = std::mem::take(&mut description);
    }
    if name.is_empty() && !anonymous && labelled {
        name = n.id.clone();
    }
    (role, name, description, value_of(n), states_of(n))
}

/// What a widget tells the tree beyond its kind (set while it is built).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct A11y {
    /// A role other than its kind's (a list row is an `option`).
    pub role: Option<&'static str>,
    /// A name other than its text (an icon button's icon, a dialog's title).
    pub name: Option<String>,
    /// Read after the name (a text field's placeholder: its name is its
    /// label); it names the control when nothing else does.
    pub description: Option<String>,
    pub value: Option<String>,
    pub checked: Option<bool>,
    pub selected: Option<bool>,
    pub expanded: Option<bool>,
    pub busy: bool,
    /// A range control's bounds and value as numbers (a slider's, a
    /// bounded stepper's, a meter's): `aria-valuemin`, `-max` and `-now`.
    pub range: Option<A11yRange>,
    /// A heading's level, 1 to 6 (`aria-level`; `<h1>`…`<h6>`).
    pub level: Option<u8>,
    /// `aria-live` written on it, over its role's (`status` is polite,
    /// `alert` assertive).
    pub live: Option<Live>,
    /// `aria-atomic` written on it, over its role's.
    pub atomic: Option<bool>,
    /// A notification (a toast): its region announces as it appears.
    pub notify: bool,
    /// A toggle button's state (`aria-pressed`).
    pub pressed: Option<bool>,
    /// The current item of a set (`aria-current`).
    pub current: bool,
}

/// How a live region's changes are announced (`aria-live`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Live {
    #[default]
    Off,
    Polite,
    Assertive,
}

impl Live {
    pub fn parse(word: &str) -> Option<Self> {
        match word {
            "off" => Some(Live::Off),
            "polite" => Some(Live::Polite),
            "assertive" => Some(Live::Assertive),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Live::Off => "",
            Live::Polite => "polite",
            Live::Assertive => "assertive",
        }
    }
}

/// A node's live region setting and whether it is atomic: `aria-live` and
/// `aria-atomic` written on it, else its role's (WAI-ARIA 1.2: `status`
/// and `log` are polite, `alert` assertive; `status` and `alert` atomic).
pub(crate) fn live_of(role: &str, live: Option<Live>, atomic: Option<bool>) -> (Live, bool) {
    let (implied, whole) = match role {
        "status" => (Live::Polite, true),
        "alert" => (Live::Assertive, true),
        "log" => (Live::Polite, false),
        _ => (Live::Off, false),
    };
    (live.unwrap_or(implied), atomic.unwrap_or(whole))
}

/// What a live region says when its content changed: a screen reader
/// speaks it after what it is reading (polite) or at once (assertive).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Announcement {
    pub text: String,
    pub assertive: bool,
}

/// Announcements a host has not taken yet are kept up to this many; older
/// ones are dropped first (a host that never takes them holds no more).
pub const ANNOUNCEMENTS: usize = 16;

/// Appends `new` to `queue`, dropping the oldest past [`ANNOUNCEMENTS`].
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn queue_announcements(
    queue: &mut std::collections::VecDeque<Announcement>,
    new: impl IntoIterator<Item = Announcement>,
) {
    for a in new {
        if queue.len() == ANNOUNCEMENTS {
            queue.pop_front();
        }
        queue.push_back(a);
    }
}

/// The tree a host last sent, as announcing reads it: each node's parent,
/// live setting, hidden state, own words and children.
pub(crate) trait LiveTree {
    type Id: Copy + Eq;
    /// The node above it; `None` at the top.
    fn up(&self, id: Self::Id) -> Option<Self::Id>;
    fn live(&self, id: Self::Id) -> (Live, bool);
    /// It announces as it appears (an alert, a toast); other regions
    /// announce only what changes in them after.
    fn notifies(&self, id: Self::Id) -> bool;
    fn hidden(&self, id: Self::Id) -> bool;
    /// Its name and value, as a screen reader says them ([`words`]).
    fn text(&self, id: Self::Id) -> String;
    /// Its value alone.
    fn value(&self, id: Self::Id) -> String;
    fn kids(&self, id: Self::Id) -> Vec<Self::Id>;
}

/// The announcements of one update (ARIA's default `aria-relevant`,
/// `additions text`): each node in `changed` whose words differ from
/// `before` (`None`: it entered) and that sits in a live region, itself or
/// under one, says its words; an atomic region says its whole text
/// instead, read from `tree`. A node with children says its value alone
/// (its name labels them: a container's id, an `aria-label`). A region
/// that enters with its content says nothing unless it notifies (ARIA
/// announces a region's changes, not its arrival). Removals
/// and emptied text say nothing, nor does a region with a hidden node on
/// the way up. One announcement per region, in the order its first change
/// comes. Reads the changed nodes, their ancestors and an atomic region's
/// nodes: nothing else.
pub(crate) fn announce<T: LiveTree>(
    tree: &T,
    changed: impl IntoIterator<Item = (T::Id, Option<String>)>,
) -> Vec<Announcement> {
    let say = |n: T::Id| {
        if tree.kids(n).is_empty() {
            tree.text(n)
        } else {
            tree.value(n)
        }
    };
    let changed: Vec<(T::Id, Option<String>)> = changed.into_iter().collect();
    let entered = |n: T::Id| changed.iter().any(|(id, was)| *id == n && was.is_none());
    let mut regions: Vec<(T::Id, Live, bool, Vec<String>)> = Vec::new();
    for (id, before) in changed.iter().cloned() {
        if before.as_deref() == Some(tree.text(id).as_str()) {
            continue;
        }
        let text = say(id);
        if text.is_empty() {
            continue;
        }
        // One walk up: the nearest live region, and no hidden node on the
        // way to the top (a hidden region, or one under a hidden node).
        let mut at = Some(id);
        let mut region = None;
        let mut hidden = false;
        while let Some(n) = at {
            if tree.hidden(n) {
                hidden = true;
                break;
            }
            if region.is_none() {
                let (live, atomic) = tree.live(n);
                if live != Live::Off {
                    region = Some((n, live, atomic));
                }
            }
            at = tree.up(n);
        }
        let Some((r, live, atomic)) = region.filter(|_| !hidden) else {
            continue;
        };
        if entered(r) && !tree.notifies(r) {
            continue;
        }
        match regions.iter_mut().find(|(id, ..)| *id == r) {
            Some((.., texts)) => {
                if !texts.contains(&text) {
                    texts.push(text);
                }
            }
            None => regions.push((r, live, atomic, vec![text])),
        }
    }
    regions
        .into_iter()
        .map(|(r, live, atomic, texts)| {
            let text = if atomic {
                let mut words: Vec<String> = Vec::new();
                let mut stack = vec![r];
                while let Some(n) = stack.pop() {
                    let t = say(n);
                    if !t.is_empty() && words.last() != Some(&t) {
                        words.push(t);
                    }
                    let mut kids = tree.kids(n);
                    kids.reverse();
                    stack.extend(kids);
                }
                words.join(" ")
            } else {
                texts.join(" ")
            };
            Announcement {
                text,
                assertive: live == Live::Assertive,
            }
        })
        .collect()
}

/// A range's bounds and value (an unbounded stepper has a value alone).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct A11yRange {
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub now: f32,
}

/// One node of the accessibility tree (IDL `ui.A11yNode`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct A11yNode {
    /// Its path, as `touch_move_to` and `og.ui.focus` name it.
    pub path: String,
    /// The path of the nearest node above it in the tree; empty at the top.
    pub parent: String,
    pub role: String,
    pub name: String,
    /// Read after the name: a text field's placeholder, a hint. Empty for
    /// none.
    pub description: String,
    pub value: String,
    pub states: Vec<String>,
    /// A range's smallest value (a slider's, a bounded stepper's, 0 for a
    /// progress or a meter); `None` for none (a stepper without bounds).
    pub value_min: Option<f32>,
    /// A range's largest value; `None` for none.
    pub value_max: Option<f32>,
    /// A range's value as a number (the value text says it as people read
    /// it); `None` for a control that is no range.
    pub value_now: Option<f32>,
    /// A heading's level, 1 to 6; `None` for none.
    pub level: Option<u8>,
    /// A live region's setting, `polite` or `assertive` (`aria-live`, or
    /// its role's: `status`, `log`, `alert`); empty for none.
    pub live: String,
    /// A live region that says its whole text when any of it changes
    /// (`aria-atomic`, or its role's: `status`, `alert`).
    pub atomic: bool,
    /// A notification that announces as it appears (an alert, a toast);
    /// another live region announces only what changes in it after.
    pub notify: bool,
    /// Its bounds in CSS pixels (the whole element, also where a scroll
    /// container cuts it; `offscreen` then says so).
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// The accessibility tree of the last frame (IDL `ui.A11yTree`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct A11yTree {
    /// The nodes with a role, in tree order (the drawing order).
    pub nodes: Vec<A11yNode>,
    /// The paths keyboard Tab and the pads visit, in order.
    pub focus_order: Vec<String>,
    /// The path keyboard and pad 0's focus is on; empty for none.
    pub focused: String,
}

impl A11yTree {
    /// A digest of the whole tree (every node's path, role, name, value,
    /// states and bounds rounded to the CSS pixel, the focus order and the
    /// focused path): a host mirroring the tree rebuilds its copy only when
    /// this changes.
    pub fn digest(&self) -> u64 {
        let mut h = og_core::hash::StableHasher::new();
        for n in &self.nodes {
            for s in [
                &n.path,
                &n.parent,
                &n.role,
                &n.name,
                &n.description,
                &n.value,
                &n.live,
            ] {
                h.write_str(s);
            }
            h.write_u64(u64::from(n.atomic) | u64::from(n.notify) << 1);
            for v in [n.value_min, n.value_max, n.value_now] {
                h.write_u64(v.map_or(u64::MAX, |v| u64::from(v.to_bits())));
            }
            h.write_u64(n.level.map_or(0, u64::from));
            h.write_u64(n.states.len() as u64);
            for s in &n.states {
                h.write_str(s);
            }
            for v in [n.x, n.y, n.width, n.height] {
                h.write_u64(v.round() as i64 as u64);
            }
        }
        h.write_u64(self.focus_order.len() as u64);
        for p in &self.focus_order {
            h.write_str(p);
        }
        h.write_str(&self.focused);
        h.finish()
    }

    /// What this tree's live regions announce against `before`, the tree a
    /// host last sent (a page's mirror, which is sent whole when it
    /// changed): as the native bridge announces (`native_a11y::Bridge`),
    /// with each node's words compared by path. Call it only when the tree
    /// changed and the change is news ([`crate::Ui::semantic_news`]).
    pub fn announcements(&self, before: &A11yTree) -> Vec<Announcement> {
        struct Tree<'a> {
            pub nodes: &'a [A11yNode],
            at: std::collections::HashMap<&'a str, usize>,
            kids: Vec<Vec<usize>>,
        }
        impl LiveTree for Tree<'_> {
            type Id = usize;
            fn up(&self, id: usize) -> Option<usize> {
                self.at.get(self.nodes[id].parent.as_str()).copied()
            }
            fn live(&self, id: usize) -> (Live, bool) {
                let n = &self.nodes[id];
                (Live::parse(&n.live).unwrap_or_default(), n.atomic)
            }
            fn notifies(&self, id: usize) -> bool {
                self.nodes[id].notify
            }
            fn hidden(&self, id: usize) -> bool {
                self.nodes[id].states.iter().any(|s| s == "hidden")
            }
            fn text(&self, id: usize) -> String {
                words(&self.nodes[id].name, &self.nodes[id].value)
            }
            fn value(&self, id: usize) -> String {
                self.nodes[id].value.trim().to_owned()
            }
            fn kids(&self, id: usize) -> Vec<usize> {
                self.kids[id].clone()
            }
        }
        let at: std::collections::HashMap<&str, usize> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.path.as_str(), i))
            .collect();
        let mut kids = vec![Vec::new(); self.nodes.len()];
        for (i, n) in self.nodes.iter().enumerate() {
            if let Some(&p) = at.get(n.parent.as_str()) {
                kids[p].push(i);
            }
        }
        let old: std::collections::HashMap<&str, String> = before
            .nodes
            .iter()
            .map(|n| (n.path.as_str(), words(&n.name, &n.value)))
            .collect();
        let tree = Tree {
            nodes: &self.nodes,
            at,
            kids,
        };
        let changed = self.nodes.iter().enumerate().filter_map(|(i, n)| {
            let was = old.get(n.path.as_str()).cloned();
            (was.as_deref() != Some(tree.text(i).as_str())).then_some((i, was))
        });
        let changed: Vec<_> = changed.collect();
        announce(&tree, changed)
    }
}

/// A node's name and value as a screen reader says them, one after the
/// other (a value that repeats the name is said once).
pub(crate) fn words(name: &str, value: &str) -> String {
    match (name.trim(), value.trim()) {
        (n, "") => n.to_owned(),
        ("", v) => v.to_owned(),
        (n, v) if n == v => n.to_owned(),
        (n, v) => format!("{n} {v}"),
    }
}

/// The role an element of `kind` has by default.
pub(crate) fn role_of(kind: &str, n: &Node) -> &'static str {
    if let Some(r) = n.a11y.role {
        return r;
    }
    match kind {
        "label" if n.heading => "heading",
        "label" | "richtext" => "text",
        "button" => "button",
        "link" => "link",
        "checkbox" => "checkbox",
        "radio" => "radio",
        "toggle" => "switch",
        "slider" => "slider",
        "field" | "textarea" => "textbox",
        "select" => "combobox",
        "option" => "option",
        "tab" => "tab",
        "progress" => "progressbar",
        "meter" => "meter",
        "badge" => "status",
        "toast" => "status",
        "divider" => "separator",
        "tooltip" => "tooltip",
        "image" | "icon" => "image",
        "joystick" | "dpad" => "group",
        // A container keys and pads reach (a shop card) is a button unless
        // its widget says otherwise.
        _ if n.nav => "button",
        _ if n.scrolls() => "group",
        // A live region is a record even without a role of its own (an
        // `aria-live` column).
        _ if n.a11y.live.is_some_and(|l| l != Live::Off) => "group",
        _ => "",
    }
}

/// The text of a node's content and its descendants', for a control
/// named by what it shows (a card: "Iron sword 300").
pub(crate) fn text_within(nodes: &[Node], i: usize) -> String {
    let mut out = String::new();
    let mut stack = vec![i];
    while let Some(k) = stack.pop() {
        let n = &nodes[k];
        if n.is_hidden() {
            continue;
        }
        if !n.probe_text.is_empty() {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&n.probe_text);
        }
        stack.extend(n.children.iter().rev());
    }
    out
}

/// A node's value as text.
pub(crate) fn value_of(n: &Node) -> String {
    if let Some(v) = &n.a11y.value {
        return v.clone();
    }
    match n.widget {
        Widget::Progress { value } => format!("{:.0}%", value.clamp(0.0, 1.0) * 100.0),
        _ => String::new(),
    }
}

/// A node's range: its smallest and largest value and its value, as
/// numbers (a slider's, a stepper's, a meter's; a progress's percent 0 to
/// 100).
pub(crate) fn range_of(n: &Node) -> Option<A11yRange> {
    if let Some(r) = n.a11y.range {
        return Some(r);
    }
    match n.widget {
        Widget::Progress { value } => Some(A11yRange {
            min: Some(0.0),
            max: Some(100.0),
            now: (value.clamp(0.0, 1.0) * 100.0).round(),
        }),
        _ => None,
    }
}

/// A node's states, besides `focused` and `focusable` (the frame's).
pub(crate) fn states_of(n: &Node) -> Vec<String> {
    let mut s = Vec::new();
    if n.disabled {
        s.push("disabled".to_owned());
    }
    if n.a11y.checked == Some(true) {
        s.push("checked".to_owned());
    }
    if n.a11y.selected == Some(true) {
        s.push("selected".to_owned());
    }
    match n.a11y.expanded {
        Some(true) => s.push("expanded".to_owned()),
        // Expandable and closed (a combobox, a tree item, an accordion
        // header): a reader says "collapsed", not nothing.
        Some(false) => s.push("collapsed".to_owned()),
        None => {}
    }
    if matches!(n.widget, Widget::Button { pressed: true, .. }) || n.a11y.pressed == Some(true) {
        s.push("pressed".to_owned());
    }
    if n.a11y.current {
        s.push("current".to_owned());
    }
    if n.a11y.busy {
        s.push("busy".to_owned());
    }
    s
}

/// Every role an element can take ([`Semantics::role`]): the widgets' own
/// and the composite roles components built from containers need (a menu
/// and its items, a tree, a table's rows and cells, a tab panel, a toolbar).
pub const ROLES: &[&str] = &[
    "button",
    "link",
    "text",
    "heading",
    "image",
    "checkbox",
    "radio",
    "switch",
    "slider",
    "spinbutton",
    "textbox",
    "combobox",
    "listbox",
    "option",
    "tab",
    "tablist",
    "tabpanel",
    "radiogroup",
    "progressbar",
    "meter",
    "timer",
    "status",
    "alert",
    "log",
    "dialog",
    "alertdialog",
    "tooltip",
    "separator",
    "group",
    "region",
    "navigation",
    "toolbar",
    "menu",
    "menubar",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "list",
    "listitem",
    "tree",
    "treeitem",
    "table",
    "grid",
    "row",
    "rowheader",
    "columnheader",
    "cell",
    "gridcell",
    "figure",
    "scrollbar",
];

/// What a component tells the accessibility tree about an element it built
/// from containers (`Ui::semantics`, IDL `ui.SemanticsDesc`): a role other
/// than its kind's, a name, a value and states. `None` leaves each as the
/// element's kind gives it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Semantics {
    pub role: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub value: Option<String>,
    pub checked: Option<bool>,
    pub selected: Option<bool>,
    pub expanded: Option<bool>,
    pub busy: bool,
    /// Shown but inert (a disabled menu item): `disabled` in the tree and
    /// the `interaction` state group's `disabled` state.
    pub disabled: bool,
    pub value_min: Option<f32>,
    pub value_max: Option<f32>,
    pub value_now: Option<f32>,
}

impl crate::Ui {
    /// Gives the element built just before (a container just closed, or a
    /// widget) a role, a name, a value and states for the accessibility
    /// tree (docs/ui.md §2: a component's accessibility role). Answers
    /// false, with a frame warning, when there is no element before it or
    /// the role is none of [`ROLES`].
    pub fn semantics(&mut self, s: &Semantics) -> bool {
        let i = self.tree.last_closed;
        if i >= self.tree.nodes.len() {
            self.out.warnings.push(
                "semantics with no element before it: call ui.semantics() right after the element"
                    .into(),
            );
            return false;
        }
        let role = match s.role.as_deref() {
            None | Some("") => None,
            Some(r) => match ROLES.iter().find(|&&k| k == r) {
                Some(&k) => Some(k),
                None => {
                    self.out.warnings.push(format!(
                        "semantics: '{r}' is not a role; take one of {}",
                        ROLES.join(", ")
                    ));
                    return false;
                }
            },
        };
        self.tree.nodes[i].disabled |= s.disabled;
        let a = &mut self.tree.nodes[i].a11y;
        if role.is_some() {
            a.role = role;
        }
        if let Some(n) = &s.name {
            a.name = Some(n.clone());
        }
        if let Some(d) = &s.description {
            a.description = Some(d.clone());
        }
        if let Some(v) = &s.value {
            a.value = Some(v.clone());
        }
        a.checked = s.checked.or(a.checked);
        a.selected = s.selected.or(a.selected);
        a.expanded = s.expanded.or(a.expanded);
        a.busy |= s.busy;
        if let Some(now) = s.value_now {
            a.range = Some(A11yRange {
                min: s.value_min,
                max: s.value_max,
                now,
            });
        }
        true
    }
}
