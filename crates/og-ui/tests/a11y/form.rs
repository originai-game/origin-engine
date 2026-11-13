//! The form `a11y` reads the tree of, shared with the wire test
//! (og-tests-ui includes this file from its `a11y`).

use og_ui::*;

#[derive(Default)]
pub(crate) struct Form {
    pub(crate) on: bool,
    pub(crate) check: bool,
    pub(crate) volume: f32,
    pub(crate) name: String,
    pub(crate) nick: String,
    pub(crate) tab: usize,
    pub(crate) choice: usize,
    pub(crate) count: f32,
    pub(crate) radio: usize,
    pub(crate) dialog: bool,
}

pub(crate) fn segment(label: &str) -> SegmentItem {
    SegmentItem {
        label: label.into(),
        ..SegmentItem::default()
    }
}

pub(crate) fn option(label: &str) -> DropdownOption {
    DropdownOption {
        label: label.into(),
        ..DropdownOption::default()
    }
}

pub(crate) fn radio(label: &str) -> RadioOption {
    RadioOption {
        label: label.into(),
        ..RadioOption::default()
    }
}

pub(crate) fn form(ui: &mut Ui, f: &mut Form) {
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
        let dialog = ui.open_modal(
            "Confirm",
            true,
            &ModalDesc {
                title: "Delete this save?".into(),
                ..ModalDesc::default()
            },
        );
        if dialog.visible {
            ui.button("Cancel", "Cancel", ButtonDesc::default());
            ui.button("Delete", "Delete", ButtonDesc::default());
        }
        ui.close_modal();
    }
}

pub(crate) fn tick(ui: &mut Ui, f: &mut Form, input: InputFrame) {
    ui.begin(input, Viewport::new(800.0, 1000.0));
    form(ui, f);
    ui.end();
}

pub(crate) fn node<'a>(tree: &'a A11yTree, path: &str) -> &'a A11yNode {
    tree.nodes
        .iter()
        .find(|n| n.path == path)
        .unwrap_or_else(|| panic!("no node {path} in {:#?}", tree.nodes))
}
