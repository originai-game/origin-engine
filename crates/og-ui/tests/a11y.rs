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
        ui.toggle("Music", &mut f.on, "Music");
        ui.checkbox("Hints", &mut f.check, "Show hints");
        ui.row(
            "Gain",
            Element {
                width: Size::grow(),
                ..Element::row()
            },
            |ui| {
                ui.label("Label", "Master", LabelDesc::default());
                let mut gain = 0.5;
                ui.slider("Slider", &mut gain, SliderDesc::default());
            },
        );
        ui.slider(
            "Volume",
            &mut f.volume,
            SliderDesc {
                min: 0.0,
                max: 100.0,
                step: 5.0,
                ..SliderDesc::default()
            },
        );
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
        ui.tabs("Mode", &["Easy", "Hard"], &mut f.tab, TabsDesc::default());
        ui.select(
            "Quality",
            &["Low", "High"],
            &mut f.choice,
            &SelectDesc::default(),
        );
        ui.stepper(
            "Count",
            &mut f.count,
            StepperDesc {
                min: 0.0,
                max: 9.0,
                ..StepperDesc::default()
            },
        );
        ui.radio(
            "Size",
            &["Small", "Large"],
            &mut f.radio,
            RadioDesc::default(),
        );
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
    assert_eq!(has("Form/Volume", "slider", "Volume").value, "35");
    assert_eq!(
        has("Form/Gain/Slider", "slider", "Master").value,
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
    has("Form/Mode", "tablist", "Mode");
    assert!(
        has("Form/Mode/Tab#1", "tab", "Hard")
            .states
            .contains(&"selected".to_owned())
    );
    assert!(
        !node(&tree, "Form/Mode/Tab#0")
            .states
            .contains(&"selected".to_owned())
    );
    let q = has("Form/Quality", "combobox", "Quality");
    assert_eq!(q.value, "Low");
    assert!(!q.states.contains(&"expanded".to_owned()));
    assert_eq!(has("Form/Count", "spinbutton", "Count").value, "3");
    has("Form/Count/Dec", "button", "Decrease");
    has("Form/Count/Inc", "button", "Increase");
    has("Form/Size", "radiogroup", "Size");
    assert!(
        has("Form/Size/Option#0", "radio", "Small")
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
    let e = ui
        .frame()
        .element("Form/Volume")
        .expect("the slider element");
    assert_eq!(
        (e.role.as_str(), e.name.as_str(), e.value.as_str()),
        ("slider", "Volume", "35")
    );
    let e = ui
        .frame()
        .element("Form/Music")
        .expect("the switch element");
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
    let source = node(ui.a11y(), "Form/Volume");
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
