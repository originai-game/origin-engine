//! Native accessibility data, not an OS adapter. The window-owning process
//! applies the updates; the engine queues requests until the UI's next tick.
use crate::Ui;
pub use accesskit;
use accesskit::{
    Action, ActionData, ActionRequest, Node, NodeId, Role, TreeId, TreeInfo, TreeUpdate,
};
use std::collections::BTreeMap;

const ROOT: NodeId = NodeId(0);

/// One socket session's incremental tree. Path ids are never reused within
/// a session, including after removal, so a delayed action cannot hit a new control.
#[derive(Clone, Default)]
pub struct Bridge {
    ids: BTreeMap<String, NodeId>,
    /// The node ids of the tree last sent, by element key, the key of each
    /// and the parent each was sent under.
    by_key: std::collections::HashMap<u64, NodeId>,
    key_of: std::collections::HashMap<NodeId, u64>,
    parent_of: std::collections::HashMap<NodeId, NodeId>,
    previous: BTreeMap<NodeId, Node>,
    focus: Option<NodeId>,
    /// The UI's semantic frame and the scale it last read at.
    frame: Option<u64>,
    scale: [f64; 2],
    read: usize,
}

impl Bridge {
    /// Bounds are multiplied into the shared frame's physical pixel space.
    /// A fresh bridge produces a full tree; unchanged frames produce
    /// nothing. Afterwards it reads only the nodes the UI says may have
    /// changed since its last update (`Ui::semantic_changes`), adds and
    /// drops the ones that entered and left, and reads again only the
    /// children of the records whose children changed
    /// (`Ui::semantic_shapes`).
    pub fn update(&mut self, ui: &Ui, scale: [f64; 2]) -> Option<TreeUpdate> {
        let since = self.frame.filter(|_| self.scale == scale);
        let (changes, frame) = ui.semantic_changes(since);
        let shapes = ui.semantic_shapes(since);
        let initial = self.focus.is_none();
        self.read = 0;
        let patched = changes
            .zip(shapes)
            .and_then(|(keys, shapes)| self.patched(ui, &keys, &shapes));
        let (changed, focus) = match patched {
            Some(patched) => patched,
            None => self.full(ui, scale),
        };
        self.frame = Some(frame);
        self.scale = scale;
        #[cfg(debug_assertions)]
        if crate::ui::bounded::checking() {
            let mut twin = self.clone();
            let (_, twin_focus) = twin.full(ui, scale);
            let differ: Vec<_> = twin
                .previous
                .iter()
                .filter(|(id, n)| self.previous.get(id) != Some(n))
                .map(|(id, n)| {
                    (
                        id,
                        n.role(),
                        n.label().map(str::to_owned),
                        self.previous.get(id),
                    )
                })
                .collect();
            assert!(
                differ.is_empty() && twin_focus == focus,
                "the screen reader's incremental tree differs from the whole tree: {differ:?}, focus {twin_focus:?} vs {focus:?}"
            );
        }
        if changed.is_empty() && self.focus == Some(focus) {
            return None;
        }
        self.focus = Some(focus);
        Some(TreeUpdate {
            nodes: changed,
            tree: initial.then(|| TreeInfo::new(ROOT)),
            tree_id: TreeId::ROOT,
            focus,
        })
    }

    /// The whole tree read again: what differs from the tree last sent,
    /// and the focus.
    fn full(&mut self, ui: &Ui, scale: [f64; 2]) -> (Vec<(NodeId, Node)>, NodeId) {
        let tree = ui.a11y();
        self.read = tree.nodes.len();
        for n in &tree.nodes {
            let next = NodeId(self.ids.len() as u64 + 1);
            self.ids.entry(n.path.clone()).or_insert(next);
        }
        let mut nodes = BTreeMap::from([(ROOT, Node::new(Role::Pane))]);
        self.by_key.clear();
        self.key_of.clear();
        self.parent_of.clear();
        for n in &tree.nodes {
            let id = self.ids[&n.path];
            nodes.insert(id, access_node(ui, n, scale));
            let key = crate::id::key_of_path(&n.path);
            self.by_key.insert(key, id);
            self.key_of.insert(id, key);
        }
        for n in &tree.nodes {
            let parent = self
                .ids
                .get(&n.parent)
                .copied()
                .filter(|id| nodes.contains_key(id))
                .unwrap_or(ROOT);
            let id = self.ids[&n.path];
            nodes.get_mut(&parent).expect("tree parent").push_child(id);
            self.parent_of.insert(id, parent);
        }
        let focus = self
            .ids
            .get(&tree.focused)
            .copied()
            .filter(|id| nodes.contains_key(id))
            .unwrap_or(ROOT);
        let changed = nodes
            .iter()
            .filter(|(id, node)| self.previous.get(id) != Some(node))
            .map(|(id, node)| (*id, node.clone()))
            .collect::<Vec<_>>();
        self.previous = nodes;
        (changed, focus)
    }

    /// The nodes `keys` name read again, those that entered added and
    /// those that left dropped with what was under them, and the children
    /// of the records whose children changed (`shapes`, and the parents of
    /// what entered or left) read again; `None` where that cannot follow
    /// (the whole tree is read then). Nothing is kept until it all reads.
    fn patched(
        &mut self,
        ui: &Ui,
        keys: &[u64],
        shapes: &[u64],
    ) -> Option<(Vec<(NodeId, Node)>, NodeId)> {
        let reach = if keys.is_empty() {
            Vec::new()
        } else {
            ui.a11y_reach()
        };
        // What this update writes (`None`: dropped), over `previous`.
        let mut staged: BTreeMap<NodeId, Option<Node>> = BTreeMap::new();
        let mut by_key = Vec::new();
        let mut parents = Vec::new();
        let mut reshape = std::collections::BTreeSet::new();
        for &key in shapes {
            if let Some(owner) = ui.a11y_owner(key, true) {
                reshape.insert(owner);
            }
        }
        let current = |staged: &BTreeMap<NodeId, Option<Node>>, id: NodeId| -> Option<Node> {
            match staged.get(&id) {
                Some(n) => n.clone(),
                None => self.previous.get(&id).cloned(),
            }
        };
        for &key in keys {
            self.read += 1;
            let record = ui.a11y_node(key, &reach);
            match (&record, self.by_key.get(&key).copied()) {
                (Some(n), Some(id)) if self.ids.get(&n.path) == Some(&id) => {
                    let mut node = access_node(ui, n, self.scale);
                    for &child in current(&staged, id)?.children() {
                        node.push_child(child);
                    }
                    staged.insert(id, Some(node));
                }
                (Some(n), None) => {
                    let next = NodeId(self.ids.len() as u64 + 1);
                    let id = *self.ids.entry(n.path.clone()).or_insert(next);
                    if current(&staged, id).is_some() {
                        return None;
                    }
                    staged.insert(id, Some(access_node(ui, n, self.scale)));
                    by_key.push((key, Some(id)));
                    reshape.insert(key);
                    reshape.insert(ui.a11y_owner(key, false)?);
                }
                (None, Some(id)) => {
                    let parent = self.parent_of.get(&id).copied().unwrap_or(ROOT);
                    reshape.insert(if parent == ROOT {
                        0
                    } else {
                        *self.key_of.get(&parent)?
                    });
                    let mut stack = vec![id];
                    while let Some(gone) = stack.pop() {
                        if let Some(old) = current(&staged, gone) {
                            stack.extend_from_slice(old.children());
                        }
                        staged.insert(gone, None);
                        if let Some(&k) = self.key_of.get(&gone) {
                            by_key.push((k, None));
                        }
                    }
                }
                (None, None) => {}
                _ => return None,
            }
        }
        let known = |by_key: &[(u64, Option<NodeId>)], key: u64| -> Option<NodeId> {
            match by_key.iter().rev().find(|(k, _)| *k == key) {
                Some(&(_, id)) => id,
                None => self.by_key.get(&key).copied(),
            }
        };
        for owner in reshape {
            let id = if owner == 0 {
                ROOT
            } else {
                known(&by_key, owner)?
            };
            let Some(mut node) = current(&staged, id) else {
                continue;
            };
            let kids = ui.a11y_children(owner)?;
            self.read += kids.len();
            let mut ids = Vec::with_capacity(kids.len());
            for (key, path) in kids {
                let child = known(&by_key, key)?;
                if self.ids.get(&path) != Some(&child) || current(&staged, child).is_none() {
                    return None;
                }
                ids.push(child);
                parents.push((child, id));
            }
            // As `full` builds it: a node without children has no
            // children property at all, not an empty one.
            node.clear_children();
            for child in ids {
                node.push_child(child);
            }
            staged.insert(id, Some(node));
        }
        for (key, id) in by_key {
            match id {
                Some(id) => {
                    self.by_key.insert(key, id);
                    self.key_of.insert(id, key);
                }
                None => {
                    if let Some(id) = self.by_key.remove(&key) {
                        self.key_of.remove(&id);
                        self.parent_of.remove(&id);
                    }
                }
            }
        }
        self.parent_of.extend(parents);
        let mut changed = Vec::new();
        for (id, node) in staged {
            match node {
                Some(node) => {
                    if self.previous.get(&id) != Some(&node) {
                        self.previous.insert(id, node.clone());
                        changed.push((id, node));
                    }
                }
                None => {
                    self.previous.remove(&id);
                }
            }
        }
        let focus = self
            .by_key
            .get(&ui.state.focus)
            .copied()
            .filter(|id| self.previous.contains_key(id))
            .unwrap_or(ROOT);
        Some((changed, focus))
    }

    /// How many accessibility records the last update read (tests and
    /// profiling): the changed nodes, or the whole tree.
    #[doc(hidden)]
    pub fn read(&self) -> usize {
        self.read
    }

    /// Validate a request against the last tree actually sent, including
    /// supported actions and deleted ids. The UI rechecks scope on the tick.
    pub fn action(&self, request: ActionRequest) -> Option<Request> {
        if request.target_tree != TreeId::ROOT {
            return None;
        }
        let node = self.previous.get(&request.target_node)?;
        if !node.supports_action(request.action) {
            return None;
        }
        let path = self
            .ids
            .iter()
            .find(|(_, id)| **id == request.target_node)?
            .0
            .clone();
        match (&request.action, &request.data) {
            (Action::SetValue, Some(ActionData::NumericValue(v))) if v.is_finite() => {}
            (Action::SetValue, Some(ActionData::Value(_))) => {}
            (Action::SetValue, _) => return None,
            _ => {}
        }
        Some(Request {
            path,
            action: request.action,
            data: request.data,
        })
    }
}

/// A validated request addressed by the UI's stable path.
#[derive(Clone, Debug)]
pub struct Request {
    path: String,
    action: Action,
    data: Option<ActionData>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Input {
    pending: Vec<Request>,
    pub values: BTreeMap<u64, ActionData>,
}

impl Ui {
    /// Called between ticks by the native host, never from its socket thread.
    pub fn queue_accessibility(&mut self, request: Request) {
        self.native_input.pending.push(request);
    }

    pub(crate) fn apply_native_accessibility(&mut self) {
        self.native_input.values.clear();
        for request in std::mem::take(&mut self.native_input.pending) {
            let key = crate::id::key_of_path(&request.path);
            let Some(node) = self.a11y().nodes.iter().find(|n| n.path == request.path) else {
                continue;
            };
            if node.states.iter().any(|s| s == "disabled" || s == "hidden") {
                continue;
            }
            let act = match request.action {
                Action::Focus | Action::SetValue => og_input::A11yAct::Focus,
                Action::Click => og_input::A11yAct::Activate,
                Action::Increment => og_input::A11yAct::Increment,
                Action::Decrement => og_input::A11yAct::Decrement,
                Action::ScrollUp
                | Action::ScrollDown
                | Action::ScrollLeft
                | Action::ScrollRight => {
                    if self.state.scope != 0
                        && self.hit(key).is_none_or(|h| h.scope != self.state.scope)
                    {
                        continue;
                    }
                    if let Some(scroll) = self.scroll_state(&request.path) {
                        let delta = match request.action {
                            Action::ScrollUp => og_core::math::Vec2::new(0.0, -scroll.view_height),
                            Action::ScrollDown => og_core::math::Vec2::new(0.0, scroll.view_height),
                            Action::ScrollLeft => og_core::math::Vec2::new(-scroll.view_width, 0.0),
                            _ => og_core::math::Vec2::new(scroll.view_width, 0.0),
                        };
                        self.scroll_by(key, delta);
                    }
                    continue;
                }
                Action::ScrollIntoView => {
                    // The focus scope still gates offscreen controls.
                    if self.focusables().iter().any(|h| h.key == key) {
                        self.scroll_into_view_path(&request.path, false);
                    }
                    continue;
                }
                _ => continue,
            };
            self.apply_a11y(&[(act, request.path)]);
            if request.action == Action::SetValue
                && self.state.focus == key
                && let Some(data) = request.data
            {
                self.native_input.values.insert(key, data);
            }
        }
    }

    pub(crate) fn accessibility_number(
        &mut self,
        key: u64,
        value: &mut f32,
        min: f32,
        max: f32,
        step: f32,
    ) -> bool {
        let number = match self.native_input.values.remove(&key) {
            Some(ActionData::NumericValue(v)) => v,
            Some(ActionData::Value(v)) => match v.parse::<f64>() {
                Ok(v) => v,
                Err(_) => return false,
            },
            _ => return false,
        };
        if !number.is_finite() {
            return false;
        }
        let mut next = (number as f32).clamp(min, max);
        if step > 0.0 {
            let origin = if min == f32::MIN { 0.0 } else { min };
            next = (origin + ((next - origin) / step).round() * step).clamp(min, max);
        }
        let changed = next != *value;
        *value = next;
        changed
    }
}

/// One accessibility record as an AccessKit node, without its children.
fn access_node(ui: &Ui, n: &crate::a11y::A11yNode, scale: [f64; 2]) -> Node {
    let mut node = Node::new(role(&n.role));
    node.set_label(n.name.clone());
    if let Some(level) = n.level {
        node.set_level(usize::from(level));
    }
    node.set_description(n.description.clone());
    node.set_value(n.value.clone());
    node.set_bounds(accesskit::Rect::new(
        f64::from(n.x) * scale[0],
        f64::from(n.y) * scale[1],
        f64::from(n.x + n.width) * scale[0],
        f64::from(n.y + n.height) * scale[1],
    ));
    let state = |s: &str| n.states.iter().any(|v| v == s);
    if state("disabled") {
        node.set_disabled();
    }
    if state("hidden") {
        node.set_hidden();
    }
    if state("busy") {
        node.set_busy();
    }
    if state("expanded") || state("collapsed") {
        node.set_expanded(state("expanded"));
    }
    if matches!(
        n.role.as_str(),
        "checkbox" | "radio" | "switch" | "menuitemcheckbox" | "menuitemradio"
    ) {
        node.set_toggled(if state("checked") {
            accesskit::Toggled::True
        } else {
            accesskit::Toggled::False
        });
    }
    if matches!(n.role.as_str(), "option" | "tab" | "treeitem" | "row") {
        node.set_selected(state("selected"));
    }
    if let Some(v) = n.value_min {
        node.set_min_numeric_value(f64::from(v));
    }
    if let Some(v) = n.value_max {
        node.set_max_numeric_value(f64::from(v));
    }
    if let Some(v) = n.value_now {
        node.set_numeric_value(f64::from(v));
    }
    if !state("disabled") && !state("hidden") {
        node.add_action(Action::ScrollIntoView);
        if let Some(scroll) = ui.scroll_state(&n.path) {
            node.set_scroll_x(f64::from(scroll.offset_x) * scale[0]);
            node.set_scroll_y(f64::from(scroll.offset_y) * scale[1]);
            if scroll.content_height > scroll.view_height {
                node.add_action(Action::ScrollUp);
                node.add_action(Action::ScrollDown);
            }
            if scroll.content_width > scroll.view_width {
                node.add_action(Action::ScrollLeft);
                node.add_action(Action::ScrollRight);
            }
        }
        // `focusable`: in the focus order (`A11yTree::focus_order`).
        if state("focusable") {
            node.add_action(Action::Focus);
            node.add_action(Action::Click);
        }
        if matches!(n.role.as_str(), "textbox" | "slider" | "spinbutton") {
            node.add_action(Action::SetValue);
        }
        if matches!(n.role.as_str(), "slider" | "spinbutton") {
            node.add_action(Action::Increment);
            node.add_action(Action::Decrement);
        }
    }
    node
}

fn role(name: &str) -> Role {
    match name {
        "button" => Role::Button,
        "link" => Role::Link,
        "text" => Role::Label,
        "heading" => Role::Heading,
        "image" => Role::Image,
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
        "group" => Role::Group,
        "region" => Role::Region,
        "navigation" => Role::Navigation,
        "main" => Role::Main,
        "banner" => Role::Banner,
        "contentinfo" => Role::ContentInfo,
        "complementary" => Role::Complementary,
        "article" => Role::Article,
        "toolbar" => Role::Toolbar,
        "menu" => Role::Menu,
        "menubar" => Role::MenuBar,
        "menuitem" => Role::MenuItem,
        "menuitemcheckbox" => Role::MenuItemCheckBox,
        "menuitemradio" => Role::MenuItemRadio,
        "list" => Role::List,
        "listitem" => Role::ListItem,
        "tree" => Role::Tree,
        "treeitem" => Role::TreeItem,
        "table" => Role::Table,
        "grid" => Role::Grid,
        "row" => Role::Row,
        "rowheader" => Role::RowHeader,
        "columnheader" => Role::ColumnHeader,
        "cell" => Role::Cell,
        "gridcell" => Role::GridCell,
        "figure" => Role::Figure,
        "scrollbar" => Role::ScrollBar,
        _ => Role::Unknown,
    }
}
