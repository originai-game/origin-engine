//! Each element's accessibility semantics (role, name, value, states,
//! range, live setting), read from the frame's nodes: the probe's
//! `dump_ui` carries them on each element, and navigation reads roles. The
//! page's mirror and native screen readers read og-ui-next's tree.

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
        name = spoken(n).to_string();
    }
    if name.is_empty() && role == "text" && n.a11y.spoken.is_some() {
        return none();
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
            name = spoken(&nodes[siblings[at - 1]]).to_string();
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
    /// What its text reads as where that differs from what it draws: its
    /// generated content's alternative text (CSS Generated Content 3
    /// §1.2). It stands in for the text wherever the text names a node;
    /// a text that reads as nothing is left out of the tree.
    pub spoken: Option<String>,
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

/// A range's bounds and value (an unbounded stepper has a value alone).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct A11yRange {
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub now: f32,
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

/// What a node's text reads as: its alternative text, else its text.
fn spoken(n: &Node) -> &str {
    n.a11y.spoken.as_deref().unwrap_or(&n.probe_text)
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
        let text = spoken(n);
        if !text.is_empty() {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(text);
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
        Widget::Progress { value, .. } => format!("{:.0}%", value.clamp(0.0, 1.0) * 100.0),
        _ => String::new(),
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
