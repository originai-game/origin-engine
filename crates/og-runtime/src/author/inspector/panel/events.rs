//! The panel's events, as the inspector's requests and state: a click
//! names the control clicked and the row it is in, a control's `changed`
//! names the prop it changed (a text box's `input`, a slider's `value`, a
//! checkbox's `checked`, tabs' and a select's `current`) and its row. A
//! field part's text is its `input` until its `commit` (Enter, or the
//! keyboard leaving it) or `cancel` (Escape).

use super::super::gizmo::{Mode, Space};
use super::values::{Field, Shown, TABS};
use super::*;
use serde::Deserialize;
use serde_json::Value;

/// One event of the TSX panel (`og_engine_inspector_event`).
#[derive(Deserialize)]
pub(in super::super) struct Event {
    kind: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    row: String,
    #[serde(default)]
    value: Value,
}

/// Does what one event asks; `paused` is the clock as shown.
pub(super) fn apply(ins: &mut Inspector, shown: &Shown, paused: bool, e: &Event) {
    match e.kind.as_str() {
        "click" => click(ins, shown, paused, &e.name, &e.row),
        "changed" => changed(ins, shown, &e.name, &e.row, &e.value),
        "commit" => commit(ins, shown, &e.row),
        "cancel" => ins.draft = None,
        "focus" => ins.editing = true,
        "blur" => ins.editing = false,
        "panel" => ins.panel_node = e.value.as_u64().map(|n| n as u32),
        _ => {}
    }
}

fn click(ins: &mut Inspector, shown: &Shown, paused: bool, name: &str, key: &str) {
    let request = match name {
        "Key" | "Close" => Request::Close,
        "Clock" if paused => Request::Resume,
        "Clock" => Request::Pause,
        "Step" => Request::Step,
        "Undo" => Request::Undo,
        "Redo" => Request::Redo,
        "RevertAll" | "Revert" => {
            let Some((entity, fields)) = &shown.prefab else {
                return;
            };
            let all = name.ends_with("All");
            if !all && !fields.iter().any(|f| f == key) {
                return;
            }
            Request::Prefab {
                entity: *entity,
                target: (!all).then(|| key.to_string()),
                fields: if all {
                    fields.clone()
                } else {
                    vec![key.to_string()]
                },
            }
        }
        _ => {
            state(ins, shown, name, key);
            return;
        }
    };
    ins.requests.push(request);
}

/// A click that changes only what the panel shows.
fn state(ins: &mut Inspector, shown: &Shown, name: &str, key: &str) {
    match name {
        "Pick" => ins.picking = !ins.picking,
        "Fold" => {
            if let Some(e) = shown.entities.get(key) {
                let bits = e.to_bits();
                if !ins.collapsed.remove(&bits) {
                    ins.collapsed.insert(bits);
                }
            }
        }
        "Label" => ins.selected = shown.entities.get(key).copied().or(ins.selected),
        "Section" => {
            if !ins.folded.remove(key) {
                ins.folded.insert(key.to_string());
            }
        }
        "Link" => {
            if let Some(f) = shown.fields.get(key) {
                ins.pending_select = f.shown.first().cloned();
            }
        }
        "Move" => ins.tool.mode = Mode::Translate,
        "Rotate" => ins.tool.mode = Mode::Rotate,
        "Scale" => ins.tool.mode = Mode::Scale,
        "Space" => {
            ins.tool.space = match ins.tool.space {
                Space::World => Space::Local,
                Space::Local => Space::World,
            }
        }
        _ => {}
    }
}

fn changed(ins: &mut Inspector, shown: &Shown, prop: &str, key: &str, value: &Value) {
    if key.is_empty() {
        match prop {
            "current" => {
                if let Some(tab) = TABS.iter().position(|t| Some(*t) == value.as_str()) {
                    ins.tab = tab;
                }
            }
            "input" => ins.filter = value.as_str().unwrap_or_default().to_string(),
            "checked" => ins.ui_bounds = value.as_bool().unwrap_or(false),
            _ => {}
        }
        return;
    }
    let Some(f) = shown.fields.get(key) else {
        // Typing in a part keeps its text until the part is left.
        if prop == "input" {
            ins.draft = Some((key.to_string(), value.as_str().unwrap_or_default().into()));
        }
        return;
    };
    let after = match (prop, f.editor) {
        ("value", Editor::Number { int }) => {
            let Some(v) = value.as_f64() else {
                return;
            };
            let after = model::number_text(if int { v.round() } else { v });
            // A slider's steps fold into one undo step, as a drag's do.
            return tune(ins, key, f, after, true);
        }
        ("checked", Editor::Bool) => value.as_bool().unwrap_or(false).to_string(),
        ("current", Editor::Enum) => match value.as_str() {
            Some(v) if f.variants.iter().any(|n| n == v) => v.to_string(),
            _ => return,
        },
        _ => return,
    };
    tune(ins, key, f, after, false);
}

fn tune(ins: &mut Inspector, target: &str, f: &Field, after: String, merge: bool) {
    ins.requests.push(Request::Tune {
        edit: Edit {
            target: target.to_string(),
            before: f.current.clone(),
            after,
        },
        merge,
    });
}

/// Writes the text typed into a part when it is committed: the field's
/// `set`, or why it was refused.
fn commit(ins: &mut Inspector, shown: &Shown, key: &str) {
    if ins.draft.as_ref().is_none_or(|(k, _)| k != key) {
        return;
    }
    let Some((key, text)) = ins.draft.take() else {
        return;
    };
    let Some((target, part)) = key
        .rsplit_once('#')
        .and_then(|(t, p)| Some((t, p.parse::<usize>().ok()?)))
    else {
        return;
    };
    let Some(f) = shown.fields.get(target) else {
        return;
    };
    if f.shown.get(part).is_some_and(|was| text.trim() == was) {
        return;
    }
    match model::command(f.editor, &f.shown, part, &text) {
        Ok(after) => tune(ins, target, f, after, false),
        Err(e) => ins.status = format!("{target}: {e}"),
    }
}
