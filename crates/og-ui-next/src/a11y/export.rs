//! Web mirror JSON serialisation and probe accessibility snapshots.

use super::*;

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
        Role::ScrollBar => "scrollbar",
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
