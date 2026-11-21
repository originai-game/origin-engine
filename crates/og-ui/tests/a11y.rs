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

#[path = "a11y/form.rs"]
mod form;
use form::*;

#[test]
fn every_widget_has_a_role_a_name_and_its_state() {
    let mut ui = ui_with_font(test_font_metrics());
    mount_form(
        &mut ui,
        &Form {
            on: true,
            volume: 35.0,
            name: "Ada",
            nick: "Mei",
            tab: 1,
            count: "3",
            ..Form::default()
        },
    );
    for _ in 0..2 {
        tick(&mut ui, InputFrame::default());
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
    let area = has("Form/Player/Memo", "textbox", "A note");
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
    // A document's number is <input type="number">, an editable field
    // with its buttons (the stepper widget is engine-internal): read by
    // its name and value, not its role.
    let count = node(&tree, "Form/Count");
    assert_eq!((count.name.as_str(), count.value.as_str()), ("Count", "3"));
    has("Form/Count/Decrement", "button", "Decrease");
    has("Form/Count/Increment", "button", "Increase");
    has("Form/Size", "radiogroup", "Size");
    assert!(
        has("Form/Size/Small/Options/Option#0", "radio", "Small")
            .states
            .contains(&"checked".to_owned())
    );
    assert_eq!(has("Form/Load", "progressbar", "Load").value, "40%");
    has("Form/Help", "text", "Read the rules.");
    let link = has("Form/Help/link0", "link", "rules");
    assert_eq!(
        link.parent, "test:form/Form/Help",
        "a link hangs under its paragraph"
    );
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
        .element("test:form/Form/Volume/Track")
        .expect("the slider element");
    assert_eq!(
        (e.role.as_str(), e.name.as_str(), e.value.as_str()),
        ("slider", "Volume", "35")
    );
    let e = ui
        .element("test:form/Form/Music")
        .expect("the switch element");
    assert!(e.states.contains(&"checked".to_owned()) && e.states.contains(&"focusable".to_owned()));
    // A layout column has no role and is not in the tree.
    assert!(tree.nodes.iter().all(|n| n.path != "test:form/Form"));
}

/// The hidden state silences a live region, its own or an ancestor's
/// (`A11yTree::announcements`): a region
/// that `visible false` takes out of the tree says nothing because its
/// nodes left, but a host's tree can also carry `hidden` on a node.
#[test]
fn hidden_live_regions_say_nothing() {
    let node = |path: &str, parent: &str, name: &str, live: &str, hidden: bool| A11yNode {
        path: path.into(),
        parent: parent.into(),
        role: if live.is_empty() { "text" } else { "group" }.into(),
        name: name.into(),
        live: live.into(),
        notify: false,
        states: if hidden {
            vec!["hidden".into()]
        } else {
            Vec::new()
        },
        ..Default::default()
    };
    let tree = |text: &str, region_hidden: bool, top_hidden: bool| A11yTree {
        nodes: vec![
            node("Top", "", "", "", top_hidden),
            node("Top/R", "Top", "", "polite", region_hidden),
            node("Top/R/T", "Top/R", text, "", false),
        ],
        ..Default::default()
    };
    let said = |after: &A11yTree, before: &A11yTree| -> Vec<String> {
        after
            .announcements(before)
            .into_iter()
            .map(|a| a.text)
            .collect()
    };
    assert_eq!(
        said(&tree("b", false, false), &tree("a", false, false)),
        ["b"]
    );
    assert!(
        said(&tree("b", true, false), &tree("a", true, false)).is_empty(),
        "hidden region"
    );
    assert!(
        said(&tree("b", false, true), &tree("a", false, true)).is_empty(),
        "hidden ancestor"
    );
}
