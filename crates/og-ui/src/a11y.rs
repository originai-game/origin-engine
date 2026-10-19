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

use crate::layout::{Node, Widget};

/// What a widget tells the tree beyond its kind (set while it is built).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct A11y {
    /// A role other than its kind's (a list row is an `option`).
    pub role: Option<&'static str>,
    /// A name other than its text (an icon button's icon, a dialog's title).
    pub name: Option<String>,
    pub value: Option<String>,
    pub checked: Option<bool>,
    pub selected: Option<bool>,
    pub expanded: Option<bool>,
    pub busy: bool,
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
    pub value: String,
    pub states: Vec<String>,
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

/// The role an element of `kind` has by default.
pub(crate) fn role_of(kind: &str, n: &Node) -> &'static str {
    if let Some(r) = n.a11y.role {
        return r;
    }
    match kind {
        "label" if n.heading => "heading",
        "label" | "richtext" => "text",
        "button" | "action" => "button",
        "link" => "link",
        "checkbox" => "checkbox",
        "radio" => "radio",
        "toggle" => "switch",
        "slider" => "slider",
        "field" => "textbox",
        "select" => "combobox",
        "option" => "option",
        "tab" => "tab",
        "chip" => "button",
        "progress" => "progressbar",
        "skeleton" | "spinner" => "progressbar",
        "meter" => "meter",
        "cooldown" => "timer",
        "badge" => "status",
        "toast" => "status",
        "divider" => "separator",
        "tooltip" => "tooltip",
        "image" | "icon" | "portrait" | "map" => "image",
        "joystick" | "dpad" | "dialogue" => "group",
        // A container keys and pads reach (a shop card) is a button unless
        // its widget says otherwise.
        _ if n.nav => "button",
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
        if n.hidden {
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

/// A node's states, besides `focused` and `focusable` (the frame's).
pub(crate) fn states_of(n: &Node) -> Vec<String> {
    let mut s = Vec::new();
    if n.disabled {
        s.push("disabled".to_owned());
    }
    let checked = n.a11y.checked.or(match n.widget {
        Widget::Toggle { on } => Some(on),
        _ => None,
    });
    if checked == Some(true) {
        s.push("checked".to_owned());
    }
    if n.a11y.selected == Some(true) {
        s.push("selected".to_owned());
    }
    if n.a11y.expanded == Some(true) {
        s.push("expanded".to_owned());
    }
    if let Widget::Button { pressed: true, .. } = n.widget {
        s.push("pressed".to_owned());
    }
    if n.a11y.busy {
        s.push("busy".to_owned());
    }
    s
}
