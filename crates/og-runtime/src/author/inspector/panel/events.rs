//! The panel's events, as the inspector's requests and state: a click
//! names the element clicked and the row it is in, a control's `changed`
//! names the prop it changed (a text box's `input`, a slider's `value`, a
//! checkbox's `checked`, tabs' and a select's `current`) and its row.

use super::super::gizmo::{Mode, Space};
use super::values::{Field, Shown, TABS};
use super::*;
use og_ui::DocEvent;
use serde_json::Value;

/// Does what this frame's events ask; `paused` is the clock as shown.
pub(super) fn apply(ins: &mut Inspector, shown: &Shown, paused: bool, events: &[DocEvent]) {
    for e in events {
        match e.kind.as_str() {
            "click" => click(ins, shown, paused, &e.name, &e.row_key),
            "changed" => changed(ins, shown, &e.name, &e.row_key, &e.value),
            _ => {}
        }
    }
}

fn click(ins: &mut Inspector, shown: &Shown, paused: bool, name: &str, key: &str) {
    let request = match name {
        "Key" | "Close" => Request::Close,
        "Save" => Request::Save,
        "Clock" if paused => Request::Resume,
        "Clock" => Request::Pause,
        "Step" => Request::Step,
        "Undo" => Request::Undo,
        "Redo" => Request::Redo,
        "ApplyAll" | "RevertAll" | "Apply" | "Revert" => {
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
                apply: name.starts_with("Apply"),
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

/// A part's element path ends in its key, escaped as og-ui keys a row.
fn part_path(key: &str) -> String {
    let escaped = key
        .replace('%', "%25")
        .replace('/', "%2F")
        .replace('#', "%23");
    format!("/Part#{escaped}")
}

/// Writes the text typed into a part once the part no longer has the
/// keyboard (Enter, or a click elsewhere): the field's `set`, or why it
/// was refused.
pub(super) fn commit_draft(ins: &mut Inspector, shown: &Shown, focus: &str) {
    let Some((key, _)) = &ins.draft else {
        return;
    };
    let at = part_path(key);
    if focus.ends_with(&at) || focus.contains(&format!("{at}/")) {
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
