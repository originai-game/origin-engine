//! og.ui's dropdown (`api/ui-controls.ogt`), og-ui's native control
//! (docs/ui.md §2), its desc read from the JSON the API passes.

use super::*;
use og_ui::{DropdownDesc, DropdownOption};

fn text(v: &Value) -> String {
    v.as_str().unwrap_or_default().to_owned()
}

fn flag(v: &Value) -> bool {
    v.as_bool().unwrap_or(false)
}

fn num(v: &Value) -> Option<f32> {
    v.as_f64().map(|n| n as f32)
}

fn items(v: &Value) -> &[Value] {
    v.as_array().map_or(&[], Vec::as_slice)
}

pub fn og_ui_dropdown(id: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let desc = DropdownDesc {
        options: items(&d["options"])
            .iter()
            .map(|o| DropdownOption {
                label: text(&o["label"]),
                description: text(&o["description"]),
                icon: text(&o["icon"]),
                disabled: flag(&o["disabled"]),
                group: text(&o["group"]),
            })
            .collect(),
        selected: d["selected"].as_i64().map_or(-1, |n| n as i32),
        label: text(&d["label"]),
        placeholder: text(&d["placeholder"]),
        helper: text(&d["helper"]),
        error: text(&d["error"]),
        disabled: flag(&d["disabled"]),
        width: size(&d["width"]),
        searchable: d["searchable"].as_bool(),
        search_placeholder: text(&d["searchPlaceholder"]),
        max_height: num(&d["maxHeight"]).unwrap_or(320.0),
    };
    let r = with_ui("og.ui.dropdown", |ui| ui.dropdown(id.as_str(), &desc)).unwrap_or(
        og_ui::DropdownResult {
            selected: desc.selected,
            ..og_ui::DropdownResult::default()
        },
    );
    json!({ "selected": r.selected, "changed": r.changed, "open": r.open }).to_string()
}
