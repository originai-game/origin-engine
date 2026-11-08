//! og.ui's controls (`api/ui-controls.ogt`): the switch, the checkbox, the
//! radio group, the segmented control, the slider, the number stepper and
//! the dropdown, each og-ui's native control (docs/ui.md §2), their descs
//! read from the JSON the API passes.

use super::*;
use og_ui::{
    ControlSize, DropdownDesc, DropdownOption, LabelSide, NumberStepperDesc, SwitchResult,
    ToggleSwitchDesc, ValueFormat,
};

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

fn control_size(v: &Value) -> ControlSize {
    match v.as_str() {
        Some("small") => ControlSize::Small,
        Some("large") => ControlSize::Large,
        _ => ControlSize::Medium,
    }
}

fn format(v: &Value) -> ValueFormat {
    ValueFormat {
        decimals: v["decimals"].as_i64().and_then(|d| usize::try_from(d).ok()),
        unit: text(&v["unit"]),
        percent: flag(&v["percent"]),
        labels: items(&v["labels"]).iter().map(text).collect(),
    }
}

fn switch(r: SwitchResult) -> String {
    json!({ "response": response(r.response), "on": r.on, "changed": r.changed }).to_string()
}

pub fn og_ui_toggle_switch(id: String, on: bool, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let desc = ToggleSwitchDesc {
        label: text(&d["label"]),
        description: text(&d["description"]),
        disabled: flag(&d["disabled"]),
        side: d["side"]
            .as_str()
            .and_then(LabelSide::from_name)
            .unwrap_or(LabelSide::Start),
        size: control_size(&d["size"]),
        width: size(&d["width"]),
        on_label: text(&d["onLabel"]),
        off_label: text(&d["offLabel"]),
    };
    let r = with_ui("og.ui.toggleSwitch", |ui| {
        ui.toggle_switch(id.as_str(), on, &desc)
    });
    switch(r.unwrap_or(SwitchResult {
        on,
        ..SwitchResult::default()
    }))
}

pub fn og_ui_number_stepper(id: String, value: f64, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let desc = NumberStepperDesc {
        min: num(&d["min"]),
        max: num(&d["max"]),
        step: num(&d["step"]).unwrap_or(1.0),
        large_step: num(&d["largeStep"]).unwrap_or(0.0),
        label: text(&d["label"]),
        format: format(&d["format"]),
        wrap: flag(&d["wrap"]),
        disabled: flag(&d["disabled"]),
        width: size(&d["width"]),
    };
    let v = value as f32;
    let r = with_ui("og.ui.numberStepper", |ui| {
        ui.number_stepper(id.as_str(), v, &desc)
    })
    .unwrap_or(og_ui::NumberResult {
        value: v,
        changed: false,
    });
    json!({ "value": r.value, "changed": r.changed }).to_string()
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
