//! The accessibility tree (`Ui::a11y`, and the probe's `dump_ui` fields):
//! every widget's role, name, value, states and bounds, and the focus
//! order keyboard Tab walks.
//!
//! Ways it could fail, each covered below:
//! - a widget has no role, or the wrong one (a switch read as a button);
//! - a control has no name (an icon button, a stepper's − and +, a field
//!   that shows only its text, a slider beside its row's label), a text
//!   field is named by its placeholder or its text when a label names it
//!   (the placeholder is its description), or a value or state goes stale (a checked
//!   box not `checked`, a slider's number missing, the chosen tab not
//!   `selected`);
//! - a range carries no bounds: a slider's, a stepper's spinbutton's, a
//!   progress's or a meter's `value_min`, `value_max` or `value_now`
//!   missing or wrong, a stepper without bounds given some, a control
//!   that is no range given a value number →
//!   `ranges_carry_their_bounds_and_value`
//! - the probe's elements and the tree disagree;
//! - Tab visits controls in another order than `focus_order`, skips one or
//!   reaches one the tree does not list; an open dialog's tree lets focus
//!   order leave it.

use crate::support;

use og_ui::*;
use support::*;

#[derive(Default)]
struct Form {
    on: bool,
    check: bool,
    volume: f32,
    name: String,
    nick: String,
    tab: usize,
    choice: usize,
    count: f32,
    radio: usize,
    dialog: bool,
}

fn segment(label: &str) -> SegmentItem {
    SegmentItem {
        label: label.into(),
        ..SegmentItem::default()
    }
}

fn option(label: &str) -> DropdownOption {
    DropdownOption {
        label: label.into(),
        ..DropdownOption::default()
    }
}

fn radio(label: &str) -> RadioOption {
    RadioOption {
        label: label.into(),
        ..RadioOption::default()
    }
}

fn form(ui: &mut Ui, f: &mut Form) {
    ui.column("Form", Element::column(), |ui| {
        ui.label(
            "Title",
            "Settings",
            LabelDesc {
                heading: true,
                ..LabelDesc::default()
            },
        );
        ui.label("Note", "Changes save at once.", LabelDesc::default());
        ui.button("Save", "Save", ButtonDesc::default());
        f.on = ui
            .toggle_switch(
                "Music",
                f.on,
                &ToggleSwitchDesc {
                    label: "Music".into(),
                    ..ToggleSwitchDesc::default()
                },
            )
            .on;
        f.check = ui
            .check_field(
                "Hints",
                f.check,
                &CheckFieldDesc {
                    label: "Show hints".into(),
                    ..CheckFieldDesc::default()
                },
            )
            .on;
        ui.row(
            "Gain",
            Element {
                width: Size::grow(),
                ..Element::row()
            },
            |ui| {
                ui.label("Label", "Master", LabelDesc::default());
                ui.value_slider("Slider", 0.5, &ValueSliderDesc::default());
            },
        );
        f.volume = ui
            .value_slider(
                "Volume",
                f.volume,
                &ValueSliderDesc {
                    min: 0.0,
                    max: 100.0,
                    step: 5.0,
                    ..ValueSliderDesc::default()
                },
            )
            .value;
        ui.text_field(
            "Name",
            &mut f.name,
            &FieldDesc {
                placeholder: "Your name".into(),
                ..FieldDesc::default()
            },
        );
        // A settings row: the label names the field, the placeholder
        // describes it (lantern-vale FINDINGS #23 heard "Your name").
        ui.row(
            "Player",
            Element {
                width: Size::grow(),
                ..Element::row()
            },
            |ui| {
                ui.label("Label", "Lamplighter", LabelDesc::default());
                ui.text_field(
                    "Field",
                    &mut f.nick,
                    &FieldDesc {
                        placeholder: "Your name".into(),
                        ..FieldDesc::default()
                    },
                );
                let mut note = String::new();
                ui.text_area(
                    "Note",
                    &mut note,
                    &TextAreaDesc {
                        placeholder: "A note".into(),
                        ..TextAreaDesc::default()
                    },
                );
            },
        );
        f.tab = ui
            .segmented(
                "Mode",
                &SegmentedDesc {
                    items: ["Easy", "Hard"].map(segment).to_vec(),
                    selected: f.tab as i32,
                    ..SegmentedDesc::default()
                },
            )
            .selected as usize;
        f.choice = ui
            .dropdown(
                "Quality",
                &DropdownDesc {
                    options: ["Low", "High"].map(option).to_vec(),
                    selected: f.choice as i32,
                    ..DropdownDesc::default()
                },
            )
            .selected
            .max(0) as usize;
        f.count = ui
            .number_stepper(
                "Count",
                f.count,
                &NumberStepperDesc {
                    min: Some(0.0),
                    max: Some(9.0),
                    ..NumberStepperDesc::default()
                },
            )
            .value;
        f.radio = ui
            .radio_group(
                "Size",
                &RadioGroupDesc {
                    options: ["Small", "Large"].map(radio).to_vec(),
                    selected: f.radio as i32,
                    ..RadioGroupDesc::default()
                },
            )
            .selected
            .max(0) as usize;
        ui.progress("Load", 0.4, ProgressDesc::default());
        ui.rich_text(
            "Help",
            "Read the [link=rules]rules[/link].",
            RichTextDesc::default(),
        );
    });
    if f.dialog {
        ui.modal(
            "Confirm",
            true,
            &ModalDesc {
                title: "Delete this save?".into(),
                ..ModalDesc::default()
            },
            |ui| {
                ui.button("Cancel", "Cancel", ButtonDesc::default());
                ui.button("Delete", "Delete", ButtonDesc::default());
            },
        );
    }
}

fn tick(ui: &mut Ui, f: &mut Form, input: InputFrame) {
    ui.begin(input, Viewport::new(800.0, 1000.0));
    form(ui, f);
    ui.end();
}

fn node<'a>(tree: &'a A11yTree, path: &str) -> &'a A11yNode {
    tree.nodes
        .iter()
        .find(|n| n.path == path)
        .unwrap_or_else(|| panic!("no node {path} in {:#?}", tree.nodes))
}

#[test]
fn every_widget_has_a_role_a_name_and_its_state() {
    let mut ui = ui_with_font(test_font_metrics());
    let mut f = Form {
        on: true,
        volume: 35.0,
        name: "Ada".into(),
        nick: "Mei".into(),
        tab: 1,
        count: 3.0,
        ..Form::default()
    };
    for _ in 0..2 {
        tick(&mut ui, &mut f, InputFrame::default());
    }
    let tree = ui.a11y().clone();
    let has = |path: &str, role: &str, name: &str| {
        let n = node(&tree, path);
        assert_eq!(
            (n.role.as_str(), n.name.as_str()),
            (role, name),
            "{path}'s role and name"
        );
        n
    };
    has("Form/Title", "heading", "Settings");
    has("Form/Note", "text", "Changes save at once.");
    has("Form/Save", "button", "Save");
    assert!(
        has("Form/Music", "switch", "Music")
            .states
            .contains(&"checked".to_owned())
    );
    assert!(
        !has("Form/Hints", "checkbox", "Show hints")
            .states
            .contains(&"checked".to_owned())
    );
    assert_eq!(has("Form/Volume/Track", "slider", "Volume").value, "35");
    assert_eq!(
        has("Form/Gain/Slider/Track", "slider", "Master").value,
        "0.50",
        "a control without text takes the label before it"
    );
    // A field alone is named by its placeholder; one after its row's label
    // by the label, the placeholder its description, whatever it holds.
    let alone = has("Form/Name", "textbox", "Your name");
    assert_eq!(
        (alone.value.as_str(), alone.description.as_str()),
        ("Ada", "")
    );
    let labelled = has("Form/Player/Field", "textbox", "Lamplighter");
    assert_eq!(
        (labelled.value.as_str(), labelled.description.as_str()),
        ("Mei", "Your name"),
        "the label names it, the placeholder describes it"
    );
    // An area after a field is not labelled by it: its placeholder names it.
    let area = has("Form/Player/Note", "textbox", "A note");
    assert_eq!(area.description, "");
    has("Form/Mode", "radiogroup", "Mode");
    assert!(
        has("Form/Mode/Strip/Track/Item#1", "radio", "Hard")
            .states
            .contains(&"checked".to_owned())
    );
    assert!(
        !node(&tree, "Form/Mode/Strip/Track/Item#0")
            .states
            .contains(&"checked".to_owned())
    );
    let q = has("Form/Quality/Trigger", "combobox", "Quality");
    assert_eq!(q.value, "Low");
    assert!(!q.states.contains(&"expanded".to_owned()));
    assert_eq!(
        has("Form/Count/Frame/Value", "spinbutton", "Count").value,
        "3"
    );
    has("Form/Count/Frame/Dec", "button", "Decrease");
    has("Form/Count/Frame/Inc", "button", "Increase");
    has("Form/Size", "radiogroup", "Size");
    assert!(
        has("Form/Size/Options/Option#0", "radio", "Small")
            .states
            .contains(&"checked".to_owned())
    );
    assert_eq!(has("Form/Load", "progressbar", "Load").value, "40%");
    has("Form/Help", "text", "Read the rules.");
    let link = has("Form/Help/rules", "link", "rules");
    assert_eq!(link.parent, "Form/Help", "a link hangs under its paragraph");
    // Every control keys reach is named; every node has bounds.
    for n in &tree.nodes {
        assert!(n.width > 0.0 && n.height > 0.0, "{} has bounds", n.path);
        if n.states.contains(&"focusable".to_owned()) {
            assert!(!n.name.is_empty(), "{} ({}) is named", n.path, n.role);
        }
    }
    // The probe reads the same: dump_ui's elements carry role, name, value
    // and states.
    let e = ui.element("Form/Volume/Track").expect("the slider element");
    assert_eq!(
        (e.role.as_str(), e.name.as_str(), e.value.as_str()),
        ("slider", "Volume", "35")
    );
    let e = ui.element("Form/Music").expect("the switch element");
    assert!(e.states.contains(&"checked".to_owned()) && e.states.contains(&"focusable".to_owned()));
    // A layout column has no role and is not in the tree.
    assert!(tree.nodes.iter().all(|n| n.path != "Form"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_tree_is_incremental_and_actions_round_trip_on_the_tick() {
    use native_a11y::{
        Bridge,
        accesskit::{Action, ActionData, ActionRequest, Role, TreeId},
    };
    use og_render::shared_frames::{Msg, recv, send};
    let mut ui = ui_with_font(test_font_metrics());
    let mut f = Form {
        volume: 35.0,
        name: "Ada".into(),
        ..Default::default()
    };
    tick(&mut ui, &mut f, InputFrame::default());
    let mut bridge = Bridge::default();
    let update = bridge.update(&ui, [2.0, 3.0]).unwrap();
    let (volume, slider) = update
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::Slider && n.numeric_value() == Some(35.0))
        .unwrap();
    let volume = *volume;
    assert_eq!(slider.min_numeric_value(), Some(0.0));
    assert_eq!(slider.max_numeric_value(), Some(100.0));
    let source = node(ui.a11y(), "Form/Volume/Track");
    assert_eq!(slider.bounds().unwrap().x0, f64::from(source.x) * 2.0);
    assert_eq!(
        slider.bounds().unwrap().y1,
        f64::from(source.y + source.height) * 3.0
    );
    let name = update
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::TextInput && n.value() == Some("Ada"))
        .unwrap()
        .0;
    let check = update
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::CheckBox && n.label() == Some("Show hints"))
        .unwrap()
        .0;
    let message = Msg::AccessibilityUpdate {
        version: 1,
        generation: 2,
        seq: 19,
        update: serde_json::to_value(&update).unwrap(),
    };
    let mut bytes = Vec::new();
    send(&mut bytes, &message).unwrap();
    assert_eq!(recv(&mut bytes.as_slice()).unwrap(), message);
    assert!(bridge.update(&ui, [2.0, 3.0]).is_none());
    let request = |target_node, action, data| ActionRequest {
        action,
        target_node,
        target_tree: TreeId::ROOT,
        data,
    };
    let dispatch = |ui: &mut Ui, request: ActionRequest| {
        let msg = Msg::AccessibilityAction {
            version: 1,
            request: serde_json::to_value(request).unwrap(),
        };
        let mut bytes = Vec::new();
        send(&mut bytes, &msg).unwrap();
        let Msg::AccessibilityAction { request, .. } = recv(&mut bytes.as_slice()).unwrap() else {
            panic!("action")
        };
        ui.queue_accessibility(
            bridge
                .action(serde_json::from_value(request).unwrap())
                .unwrap(),
        );
    };
    dispatch(
        &mut ui,
        request(
            volume,
            Action::SetValue,
            Some(ActionData::NumericValue(73.0)),
        ),
    );
    assert_eq!(f.volume, 35.0, "socket receipt does not change simulation");
    tick(&mut ui, &mut f, InputFrame::default());
    assert_eq!(f.volume, 75.0, "slider quantizes to its step");
    dispatch(
        &mut ui,
        request(
            name,
            Action::SetValue,
            Some(ActionData::Value("Grace".into())),
        ),
    );
    tick(&mut ui, &mut f, InputFrame::default());
    assert_eq!(f.name, "Grace");
    dispatch(&mut ui, request(check, Action::Click, None));
    tick(&mut ui, &mut f, InputFrame::default());
    assert!(f.check);
    let diff = bridge.update(&ui, [2.0, 3.0]).unwrap();
    assert!(diff.tree.is_none());
    assert!(diff.nodes.len() < update.nodes.len());
    assert_eq!(diff.focus, check);
    assert!(
        diff.nodes
            .iter()
            .any(|(id, n)| *id == volume && n.numeric_value() == Some(75.0))
    );
    ui.begin(InputFrame::default(), Viewport::new(800.0, 1000.0));
    ui.end();
    bridge.update(&ui, [2.0, 3.0]).unwrap();
    assert!(
        bridge.action(request(check, Action::Click, None)).is_none(),
        "deleted ids reject delayed actions"
    );
}

/// A screen reader reads only what changed: an idle tick reads no record
/// and sends nothing, and one bound label reads a handful of records and
/// sends that label, however long the list beside it is. Rows that enter,
/// leave or move in a small group read that group's children and the rows
/// themselves, not the 300 rows beside it. Fails if the bridge falls back
/// to the whole tree on every change, or if a change is missed
/// (`OG_UI_CHECK_BOUNDED=1` also compares every update with the whole
/// tree).
#[test]
fn native_tree_reads_only_the_changed_nodes() {
    use og_ui::reactive::{SourceId, wire};
    use serde_json::{Value, json};
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(
        UiDocument::parse(
            "ui/hud.svelte",
            r#"<meta name="component" content="hud">
<column id="Hud">
  <h2 id="Title">Hud</h2>
  <text id="Score">{$score}</text>
  {#each $rows as row (row.id)}<button id="Row">{row.id}</button>{/each}
  <scroll id="Small" height="160">{#each $small as item (item.id)}<button id="Item">{item.id}</button>{/each}</scroll>
</column>"#,
        )
        .unwrap(),
    );
    let apply = |ui: &mut Ui, command: Value| {
        let reply =
            wire::decode(&ui.apply_reactive(&wire::encode(&json!([command])).unwrap())).unwrap();
        assert_eq!(reply[0]["ok"], true, "{reply}");
    };
    let rows: Vec<Value> = (0..300).map(|i| json!({ "id": format!("r{i}") })).collect();
    let mut inputs = std::collections::BTreeMap::new();
    let (score, list) = (
        SourceId::new("test", "score"),
        SourceId::new("test", "rows"),
    );
    apply(
        &mut ui,
        json!({"op": "signal", "source": score, "initial": "10"}),
    );
    apply(
        &mut ui,
        json!({"op": "collection", "source": list, "key": "id", "items": rows}),
    );
    let small = SourceId::new("test", "small");
    apply(
        &mut ui,
        json!({"op": "collection", "source": small, "key": "id",
               "items": [{"id": "a"}, {"id": "b"}, {"id": "c"}]}),
    );
    inputs.insert("score".to_string(), score);
    inputs.insert("rows".to_string(), list);
    inputs.insert("small".to_string(), small.clone());
    ui.mount_document(
        SourceId::new("test", "hud"),
        "hud",
        inputs,
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let step = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 900.0));
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
    };
    step(&mut ui);
    step(&mut ui);
    let mut bridge = native_a11y::Bridge::default();
    let first = bridge.update(&ui, [1.0, 1.0]).expect("the whole tree");
    assert!(bridge.read() > 300, "the first update reads the whole tree");
    let heading = first
        .nodes
        .iter()
        .find(|(_, n)| n.label() == Some("Hud"))
        .expect("the heading");
    assert_eq!(
        (heading.1.role(), heading.1.level()),
        (native_a11y::accesskit::Role::Heading, Some(2)),
        "an <h2> reaches the screen reader as a level 2 heading"
    );
    step(&mut ui);
    assert!(bridge.update(&ui, [1.0, 1.0]).is_none());
    assert_eq!(bridge.read(), 0, "an idle tick reads nothing");
    apply(
        &mut ui,
        json!({"op": "set", "source": SourceId::new("test", "score"), "value": "11"}),
    );
    step(&mut ui);
    let update = bridge.update(&ui, [1.0, 1.0]).expect("the label changed");
    let labels: Vec<_> = update.nodes.iter().map(|(_, n)| n.label()).collect();
    assert_eq!(labels, [Some("11")], "only the label is sent");
    assert!(bridge.read() < 8, "read {} records", bridge.read());
    step(&mut ui);
    assert!(bridge.update(&ui, [1.0, 1.0]).is_none());
    // In the small group: an item enters, one moves, one leaves. The
    // group's children are sent again, and only the items that moved with
    // them; the order is the whole tree's.
    let names = |ui: &Ui| -> Vec<String> {
        ui.a11y()
            .nodes
            .iter()
            .filter(|n| n.path.contains("Item"))
            .map(|n| n.name.clone())
            .collect()
    };
    for (command, expect) in [
        (
            json!({"op": "insert", "source": small, "row": {"id": "n"}, "before": "b"}),
            ["a", "n", "b", "c"].as_slice(),
        ),
        (
            json!({"op": "move", "source": small, "key": "c", "before": "a"}),
            ["c", "a", "n", "b"].as_slice(),
        ),
        (
            json!({"op": "remove", "source": small, "key": "a"}),
            ["c", "n", "b"].as_slice(),
        ),
    ] {
        apply(&mut ui, command);
        step(&mut ui);
        let update = bridge.update(&ui, [1.0, 1.0]).expect("the group changed");
        assert!(bridge.read() < 30, "read {} records", bridge.read());
        assert!(update.nodes.len() < 8, "sent {} nodes", update.nodes.len());
        let group = update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == native_a11y::accesskit::Role::Group)
            .expect("the group's children are sent again");
        assert_eq!(group.1.children().len(), expect.len());
        assert_eq!(names(&ui), expect);
    }
}
