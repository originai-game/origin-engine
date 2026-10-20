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
//! - the probe's elements and the tree disagree;
//! - Tab visits controls in another order than `focus_order`, skips one or
//!   reaches one the tree does not list; an open dialog's tree lets focus
//!   order leave it.

mod support;

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

#[test]
fn tab_walks_the_focus_order_and_a_dialog_keeps_it() {
    let mut ui = ui_with_font(test_font_metrics());
    let mut f = Form::default();
    for _ in 0..2 {
        tick(&mut ui, &mut f, InputFrame::default());
    }
    let order = ui.a11y().focus_order.clone();
    assert!(
        order.len() >= 12,
        "every control is in the focus order: {order:?}"
    );
    for path in &order {
        assert!(
            node(ui.a11y(), path)
                .states
                .contains(&"focusable".to_owned()),
            "{path} is marked focusable"
        );
    }
    let tab = InputFrame {
        keys: vec![UiKey::Tab],
        ..InputFrame::default()
    };
    let mut walked = Vec::new();
    for _ in 0..order.len() + 1 {
        tick(&mut ui, &mut f, tab.clone());
        walked.push(ui.a11y().focused.clone());
        let focused: Vec<&A11yNode> = ui
            .a11y()
            .nodes
            .iter()
            .filter(|n| n.states.contains(&"focused".to_owned()))
            .collect();
        assert_eq!(focused.len(), 1, "one node is focused: {focused:?}");
    }
    assert_eq!(
        walked[..order.len()],
        order[..],
        "Tab visits the focus order"
    );
    assert_eq!(walked[order.len()], order[0], "and wraps to its start");
    // An open dialog: its focus order is its own controls, and it is a
    // dialog named by its title.
    f.dialog = true;
    for _ in 0..30 {
        tick(&mut ui, &mut f, InputFrame::default());
    }
    let tree = ui.a11y();
    let d = node(tree, "Confirm/Panel");
    assert_eq!(
        (d.role.as_str(), d.name.as_str()),
        ("dialog", "Delete this save?")
    );
    assert_eq!(
        tree.focus_order,
        vec![
            "Confirm/Panel/Cancel".to_owned(),
            "Confirm/Panel/Delete".to_owned()
        ],
        "focus stays in the dialog"
    );
    assert!(
        !node(tree, "Form/Save")
            .states
            .contains(&"focusable".to_owned()),
        "the page under the dialog is not focusable"
    );
}

fn asked(act: og_input::A11yAct, path: &str) -> InputFrame {
    InputFrame {
        a11y: vec![(act, path.to_owned())],
        ..InputFrame::default()
    }
}

/// A screen reader's requests (`InputFrame::a11y`, og-input's
/// `og.A11yAction`) act on the control the tree names: Focus moves focus
/// there with the ring, Activate presses it as Enter does (a switch
/// toggles, a tab is chosen), Increment and Decrement step a slider and a
/// stepper's spinbutton, a
/// text field takes the focus to edit; a path outside an open dialog, or
/// naming no control, does nothing.
///
/// Ways it could fail: the path resolves to another key than the frame
/// gave the control (layers, nested rows, links); the request acts a tick
/// late or twice; it reaches the page under a dialog.
#[test]
fn a_screen_readers_requests_reach_the_control_the_tree_names() {
    use og_input::A11yAct;
    let mut ui = ui_with_font(test_font_metrics());
    let mut f = Form::default();
    for _ in 0..2 {
        tick(&mut ui, &mut f, InputFrame::default());
    }
    tick(&mut ui, &mut f, asked(A11yAct::Focus, "Form/Hints"));
    assert_eq!(ui.a11y().focused, "Form/Hints");
    assert!(!f.check, "focus alone does not check it");
    tick(&mut ui, &mut f, asked(A11yAct::Activate, "Form/Music"));
    assert!(f.on, "activating the switch toggles it");
    assert_eq!(ui.a11y().focused, "Form/Music");
    tick(&mut ui, &mut f, InputFrame::default());
    assert!(f.on, "once");
    tick(&mut ui, &mut f, asked(A11yAct::Activate, "Form/Mode/Tab#1"));
    assert_eq!(f.tab, 1, "activating a tab chooses it");
    tick(&mut ui, &mut f, asked(A11yAct::Increment, "Form/Volume"));
    assert_eq!(f.volume, 5.0, "increment steps the slider by its step");
    tick(&mut ui, &mut f, asked(A11yAct::Increment, "Form/Volume"));
    tick(&mut ui, &mut f, asked(A11yAct::Decrement, "Form/Volume"));
    assert_eq!(f.volume, 5.0);
    // A stepper's spinbutton adjusts as a whole (the mirror's native
    // control, VoiceOver's swipe up and down), within its range.
    tick(&mut ui, &mut f, asked(A11yAct::Increment, "Form/Count"));
    assert_eq!(f.count, 1.0, "increment steps the stepper");
    let twice = InputFrame {
        a11y: vec![
            (A11yAct::Increment, "Form/Count".to_owned()),
            (A11yAct::Increment, "Form/Count".to_owned()),
        ],
        ..InputFrame::default()
    };
    tick(&mut ui, &mut f, twice);
    assert_eq!(f.count, 3.0, "two requests in a tick, two steps");
    let node = ui
        .a11y()
        .nodes
        .iter()
        .find(|n| n.path == "Form/Count")
        .cloned();
    assert_eq!(
        node.map(|n| (n.role, n.value)),
        Some(("spinbutton".into(), "3".into()))
    );
    for _ in 0..5 {
        tick(&mut ui, &mut f, asked(A11yAct::Decrement, "Form/Count"));
    }
    assert_eq!(f.count, 0.0, "not past its minimum");
    tick(&mut ui, &mut f, asked(A11yAct::Activate, "Form/Name"));
    assert_eq!(ui.a11y().focused, "Form/Name", "a field takes the focus");
    tick(
        &mut ui,
        &mut f,
        asked(A11yAct::Activate, "Form/Nothing here"),
    );
    assert_eq!(
        ui.a11y().focused,
        "Form/Name",
        "an unknown path does nothing"
    );
    // A dialog's scope: the page under it is out of reach.
    f.dialog = true;
    for _ in 0..30 {
        tick(&mut ui, &mut f, InputFrame::default());
    }
    tick(&mut ui, &mut f, asked(A11yAct::Activate, "Form/Music"));
    assert!(f.on, "the switch under the dialog did not toggle");
    tick(&mut ui, &mut f, asked(A11yAct::Increment, "Form/Count"));
    assert_eq!(f.count, 0.0, "nor the stepper");
    tick(
        &mut ui,
        &mut f,
        asked(A11yAct::Focus, "Confirm/Panel/Delete"),
    );
    assert_eq!(ui.a11y().focused, "Confirm/Panel/Delete");
    // The tree's digest moves with what a mirror shows.
    let before = ui.a11y().digest();
    tick(&mut ui, &mut f, InputFrame::default());
    assert_eq!(ui.a11y().digest(), before, "an unchanged frame keeps it");
    tick(
        &mut ui,
        &mut f,
        asked(A11yAct::Focus, "Confirm/Panel/Cancel"),
    );
    assert_ne!(ui.a11y().digest(), before, "focus moved: it changes");
}

/// The page's preferences arrive with the viewport: reduced motion and
/// reduced transparency turn the theme's flags on (reduced transparency
/// also flattens the scrim and the translucent fills, as the game's own
/// flag does), more contrast takes the high-contrast variant,
/// over whatever the game configured, and leave again when the page
/// drops them.
#[test]
fn the_pages_motion_and_contrast_preferences_reach_the_theme() {
    let mut ui = ui_with_font(test_font_metrics());
    let mut f = Form::default();
    let configured = *ui.theme();
    assert!(!configured.reduced_motion && configured.contrast == Contrast::Standard);
    let at = |reduced_motion, high_contrast| Viewport {
        reduced_motion,
        high_contrast,
        reduced_transparency: reduced_motion,
        ..Viewport::new(800.0, 1000.0)
    };
    for (reduced, high) in [(true, false), (false, true), (true, true), (false, false)] {
        ui.begin(InputFrame::default(), at(reduced, high));
        form(&mut ui, &mut f);
        ui.end();
        let t = ui.theme();
        assert_eq!(t.reduced_motion, reduced, "reduced motion {reduced}");
        assert_eq!(t.reduced_transparency, reduced, "reduced transparency");
        assert_eq!(t.contrast == Contrast::High, high, "high contrast {high}");
        if high {
            assert_eq!(
                t.palette.fg_muted, t.palette.fg,
                "high contrast inks muted text as fg"
            );
        }
        // Reduced transparency flattens the scrim and the translucent fills.
        let p = t.palette;
        let flat = [
            p.scrim,
            p.fill,
            p.fill_strong,
            p.accent_soft,
            p.material_thick,
        ];
        assert_eq!(
            flat.iter().all(|c| c.alpha == 1.0),
            reduced,
            "flat under reduced transparency only: {flat:?}"
        );
        if reduced && !high {
            let c = configured.palette;
            assert_eq!(
                p.fill,
                c.fill.over(c.surface),
                "a fill as it looks on a surface"
            );
            assert_eq!(p.scrim, c.scrim.over(c.bg));
            assert_eq!(p.fg, c.fg, "opaque inks unchanged");
        }
    }
    assert_eq!(*ui.theme(), configured, "the configured theme is back");
    // A game's own reduced transparency flattens the same way.
    ui.set_theme(Theme {
        reduced_transparency: true,
        ..configured
    });
    ui.begin(InputFrame::default(), at(false, false));
    form(&mut ui, &mut f);
    ui.end();
    assert_eq!(ui.theme().palette.scrim.alpha, 1.0);
    assert_eq!(*ui.theme(), configured.to_reduced_transparency());
}
