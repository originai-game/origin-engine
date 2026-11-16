//! HTML's customizable select (`appearance: base-select`): the select is
//! built from its own children rather than the native dropdown. The
//! select's box is the trigger, a press on it opens and closes the picker.
//! It shows its first child `<button>` when it has one, else the chosen
//! option's label. `::picker(select)` is an auto popover in the top layer,
//! anchored to the select, holding the rest of its children: options,
//! `<optgroup>`s and `<hr>`s. A press on an enabled option chooses it,
//! writes the select's value (its binding, or its own value when it has
//! none), fires its change and closes the picker; a press outside or Back
//! closes it too. The chosen option matches `:checked` and a disabled one
//! `:disabled`. `<selectedcontent>` in the button shows the chosen
//! option's content, and the select matches `:open` while its picker is.
//!
//! Core keeps the value, the open state and the presses; what each part
//! looks like is the document's.

use super::nodes::Inner;
use super::*;
use crate::bind::InteractionState;
use serde_json::json;

/// A customizable select being built: its chosen option, its disabled
/// options, and the option pressed among those built again.
#[derive(Clone, Debug, Default)]
pub(crate) struct SelectBuild {
    chosen: Option<usize>,
    disabled: Vec<usize>,
    pressed: Option<usize>,
}

/// Option `n`'s place among its select's options, through its
/// `<optgroup>`; `None` for an element that is no select's option.
pub(crate) fn option_index(doc: &crate::doc::UiDocument, n: &DocNode) -> Option<usize> {
    let tag = |i: usize| doc.nodes[i].selector_tag.as_deref();
    let mut select = n.parent;
    if select != NONE && tag(select) == Some("optgroup") {
        select = doc.nodes[select].parent;
    }
    if select == NONE || tag(select) != Some("select") {
        return None;
    }
    let mut k = 0;
    for &c in &doc.nodes[select].children {
        let options: Vec<usize> = match tag(c) {
            Some("option") => vec![c],
            Some("optgroup") => doc.nodes[c]
                .children
                .iter()
                .copied()
                .filter(|&o| tag(o) == Some("option"))
                .collect(),
            _ => Vec::new(),
        };
        for o in options {
            if std::ptr::eq(&doc.nodes[o], n) {
                return Some(k);
            }
            k += 1;
        }
    }
    None
}

/// Whether a select resolved as `r` is a customizable one.
pub(crate) fn is_base_select(kind: Kind, r: &Resolved) -> bool {
    kind == Kind::Select && r.str("appearance") == Some("baseSelect")
}

/// A select's option values, each its label when it has no value.
fn values_of(r: &Resolved) -> (Vec<String>, Vec<usize>) {
    let strs = |name: &str| match r.get(name) {
        Some(V::Strs(s)) => s.clone(),
        _ => Vec::new(),
    };
    let items = strs("items");
    let mut values = strs("values");
    values.resize(items.len(), String::new());
    for (v, item) in values.iter_mut().zip(&items) {
        if v.is_empty() {
            v.clone_from(item);
        }
    }
    let disabled = strs("disabledItems")
        .iter()
        .filter_map(|k| k.parse().ok())
        .collect();
    (values, disabled)
}

impl Ui {
    /// Opens customizable select `n` (render key `key`) as its trigger, a
    /// box the player presses, and starts noting its options.
    pub(crate) fn open_base_select(&mut self, n: &DocNode, id: &str, key: u64, r: &Resolved) {
        let (values, disabled) = values_of(r);
        let own = self.docs.state().memory.values.get(&key).cloned();
        let now = match own {
            Some(v) if !nodes::bound(n, "current") => Some(v),
            _ => r.str("current").map(str::to_string),
        };
        // HTML: with none selected, the first enabled option is.
        let chosen = now
            .and_then(|v| values.iter().position(|x| *x == v))
            .or_else(|| (0..values.len()).find(|k| !disabled.contains(k)));
        self.docs.state_mut().select_keys.push(key);
        let open = self.docs.state().memory.open.get(&key).copied() == Some(true);
        self.docs.state_mut().selects.push(SelectBuild {
            chosen,
            disabled,
            pressed: None,
        });
        // Open, it builds every frame, as a popover does: its options sit
        // in the picker's layer, which the retained tree does not hold, so
        // a press on one reaches it only through its select.
        if open && let Some(owned) = self.tree.owning_mut().last_mut() {
            owned.moved = true;
        }
        let enabled = !r.flag("disabled");
        self.open(
            id,
            Element {
                direction: Direction::Row,
                interactive: enabled,
                focusable: enabled,
                ..Element::default()
            },
        );
    }

    /// The trigger's content: the select's first child when it is a
    /// `<button>`, else the chosen option's label in the select's text
    /// style. Where the picker's children start.
    pub(crate) fn base_select_trigger(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        r: &Resolved,
        build_inner: &mut dyn FnMut(&mut Ui, &mut Build<'_>, Inner<'_>),
    ) -> usize {
        let button = n
            .children
            .first()
            .is_some_and(|&c| b.doc.nodes[c].selector_tag.as_deref() == Some("button"));
        if button {
            build_inner(self, b, Inner::Range(0..1));
            return 1;
        }
        let strs = |name: &str| match r.get(name) {
            Some(V::Strs(s)) => s.clone(),
            _ => Vec::new(),
        };
        let chosen = self.docs.state().selects.last().and_then(|s| s.chosen);
        let label = chosen
            .and_then(|k| strs("items").get(k).cloned())
            .unwrap_or_default();
        let desc = r.label_desc(self.theme(), self.fonts(), r.own_flow().rtl);
        self.label("Value", &label, desc);
        0
    }

    /// `<selectedcontent>` `key`'s content: its copy of the chosen
    /// option's. Built again on its own (a choice marks it edited), it
    /// shows what its select last chose.
    pub(crate) fn selected_content(
        &mut self,
        b: &mut Build<'_>,
        key: u64,
        build_inner: &mut dyn FnMut(&mut Ui, &mut Build<'_>, Inner<'_>),
    ) {
        let state = self.docs.state_mut();
        let chosen = match (state.selects.last(), state.select_keys.last()) {
            (Some(s), Some(&select)) => {
                state.shown_by.insert(key, select);
                s.chosen
            }
            _ => state
                .shown_by
                .get(&key)
                .and_then(|select| state.select_chosen.get(select))
                .copied()
                .flatten(),
        };
        if let Some(k) = chosen {
            build_inner(self, b, Inner::Range(k..k + 1));
        }
    }

    /// Option `key`, the `index`th of the select being built, closed
    /// with `resp`.
    pub(crate) fn note_option(&mut self, key: u64, index: usize, resp: crate::ui::Response) {
        let state = self.docs.state_mut();
        if let Some(s) = state.selects.last_mut() {
            if resp.clicked {
                s.pressed = Some(index);
            }
            if let Some(select) = state.select_keys.last() {
                state.option_of.insert(key, (*select, index));
            }
        }
    }

    /// After the select's box closed with `resp`: its open state, its
    /// picker with the children from `start` on, and the option chosen.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn close_base_select(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        key: u64,
        path: &str,
        r: &Resolved,
        resp: &mut crate::ui::Response,
        start: usize,
        build_inner: &mut dyn FnMut(&mut Ui, &mut Build<'_>, Inner<'_>),
    ) {
        let at = self.tree.last_closed();
        let was_open = self.docs.state().memory.open.get(&key).copied() == Some(true);
        let mut open = was_open;
        if resp.clicked && !r.flag("disabled") {
            open = !open;
        }
        let picker = self.style.pseudo_node(b, r, path, "picker");
        let content = Element {
            direction: Direction::Column,
            ..Element::default()
        };
        let out = self.floating(&format!("{key}.picker"), open, at, content, |ui| {
            let layer = ui.tree.parent();
            build_inner(ui, b, Inner::Range(start..usize::MAX));
            if let Some(p) = &picker {
                ui.style_pseudo_box(b, layer, p);
                ui.place_popover(&p.1.values, layer, None);
            }
        });
        let s = self.docs.state_mut().selects.pop().unwrap_or_default();
        if out.dismissed {
            open = false;
        }
        self.docs.state_mut().select_keys.pop();
        let mut chosen = s.chosen;
        let picked = s.pressed.filter(|k| !s.disabled.contains(k));
        if let Some(k) = picked {
            open = false;
            let (values, _) = values_of(r);
            if Some(k) != chosen
                && let Some(v) = values.get(k)
            {
                chosen = Some(k);
                if !nodes::bound(n, "current") {
                    let state = self.docs.state_mut();
                    state.memory.values.insert(key, v.clone());
                    state.edited.insert(key);
                }
                self.docs.write_back(b, n, "current", json!(v));
                resp.changed = true;
            }
        }
        let state = self.docs.state_mut();
        if state.select_chosen.insert(key, chosen) != Some(chosen) {
            // Its `<selectedcontent>`s show the new choice.
            let shown: Vec<u64> = state
                .shown_by
                .iter()
                .filter(|(_, s)| **s == key)
                .map(|(k, _)| *k)
                .collect();
            state.edited.extend(shown);
        }
        let options: Vec<(u64, usize)> = state
            .option_of
            .iter()
            .filter(|(_, o)| o.0 == key)
            .map(|(option, o)| (*option, o.1))
            .collect();
        for (option, k) in options {
            let states = &mut state.selector_states;
            states.push((option, InteractionState::Checked, Some(k) == chosen));
            states.push((option, InteractionState::Disabled, s.disabled.contains(&k)));
        }
        if open != was_open {
            let state = self.docs.state_mut();
            state.memory.open.insert(key, open);
            state.edited.insert(key);
        }
        self.tree.semantics_mut(at).expanded = Some(open);
    }
}
