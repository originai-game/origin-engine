//! The panel's toolbar, footer and gizmo bar.

use super::*;

pub(super) fn toolbar(ui: &mut Ui, s: &mut State, cx: &Ctx) {
    let t = *ui.theme();
    let p = t.palette;
    let dirty = s.ins.dirty.len();
    let row = |gap| Element {
        direction: Direction::Row,
        width: Size::grow(),
        cross: Cross::Center,
        gap,
        ..Element::default()
    };
    ui.row("Bar", row(t.space.xs), |ui| {
        ui.label(
            "Title",
            "Inspector",
            LabelDesc {
                step: Some(t.type_ramp.base),
                color: Some(p.ink),
                heading: true,
                nowrap: true,
                ..LabelDesc::default()
            },
        );
        let key = s.ins.key_label();
        if ui.button("Key", &key, small(ButtonVariant::Quiet)).clicked {
            s.ins.requests.push(Request::Close);
        }
        ui.tooltip(&if s.ins.needs_ctrl {
            format!(
                "{key} shows and hides the inspector (the game binds {}, so it takes Ctrl with it)",
                s.ins.key
            )
        } else {
            format!("{key} shows and hides the inspector")
        });
        ui.spacer("Gap", None);
        let save = if dirty > 0 {
            format!("Save ({dirty})")
        } else {
            "Save".to_string()
        };
        let r = ui.button(
            "Save",
            &save,
            ButtonDesc {
                disabled: dirty == 0,
                ..small(if dirty > 0 {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Secondary
                })
            },
        );
        if r.clicked {
            s.ins.requests.push(Request::Save);
        }
        ui.tooltip("Write the edited fields into the scenes' text (save_scene)");
        if ui
            .button("Close", "Close", small(ButtonVariant::Quiet))
            .clicked
        {
            s.ins.requests.push(Request::Close);
        }
    });
    ui.row("Tools", row(t.space.xs), |ui| {
        let (text, req) = if cx.paused {
            ("Play", Request::Resume)
        } else {
            ("Pause", Request::Pause)
        };
        if ui
            .button("Clock", text, small(ButtonVariant::Secondary))
            .clicked
        {
            s.ins.requests.push(req);
        }
        let step = ui.button(
            "Step",
            "Step",
            ButtonDesc {
                disabled: !cx.paused,
                ..small(ButtonVariant::Secondary)
            },
        );
        if step.clicked {
            s.ins.requests.push(Request::Step);
        }
        ui.tooltip("Advance one tick while paused");
        let pick = ui.button(
            "Pick",
            if s.ins.picking { "Picking…" } else { "Pick" },
            small(if s.ins.picking {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Secondary
            }),
        );
        if pick.clicked {
            s.ins.picking = !s.ins.picking;
        }
        ui.tooltip("Select what is under the next click in the game");
        ui.spacer("Gap", None);
        let undo = ui.button(
            "Undo",
            "Undo",
            ButtonDesc {
                disabled: !s.ins.history.can_undo(),
                ..small(ButtonVariant::Secondary)
            },
        );
        if undo.clicked {
            s.ins.requests.push(Request::Undo);
        }
        ui.tooltip("Undo the last edit (Ctrl+Z)");
        let redo = ui.button(
            "Redo",
            "Redo",
            ButtonDesc {
                disabled: !s.ins.history.can_redo(),
                ..small(ButtonVariant::Secondary)
            },
        );
        if redo.clicked {
            s.ins.requests.push(Request::Redo);
        }
        ui.tooltip("Redo (Ctrl+Shift+Z)");
    });
}

pub(super) fn footer(ui: &mut Ui, s: &mut State, cx: &Ctx) {
    let t = *ui.theme();
    let p = t.palette;
    if s.ins.tab == 0 {
        gizmo_bar(ui, s);
    }
    ui.row(
        "Status",
        Element {
            direction: Direction::Row,
            width: Size::grow(),
            cross: Cross::Center,
            gap: t.space.sm,
            ..Element::default()
        },
        |ui| {
            let clock = if cx.paused {
                format!("Tick {} · paused", cx.tick)
            } else {
                format!("Tick {}", cx.tick)
            };
            label(ui, "Clock", &clock, p.ink_faint, true, Size::fit());
            let text = if s.ins.status.is_empty() {
                String::new()
            } else {
                s.ins.status.clone()
            };
            label(ui, "Note", &text, p.ink_muted, true, Size::grow());
        },
    );
}

/// The gizmo's mode (W, E, R) and space.
pub(super) fn gizmo_bar(ui: &mut Ui, s: &mut State) {
    use super::super::gizmo::{Mode, Space};
    let t = *ui.theme();
    ui.row(
        "Gizmo",
        Element {
            direction: Direction::Row,
            width: Size::grow(),
            cross: Cross::Center,
            gap: t.space.xs,
            ..Element::default()
        },
        |ui| {
            for (id, text, mode, tip) in [
                ("Move", "Move", Mode::Translate, "Move along an axis or a plane (W); Ctrl snaps to 0.5"),
                ("Rotate", "Rotate", Mode::Rotate, "Turn about an axis or the view (E); Ctrl snaps to 15°"),
                ("Scale", "Scale", Mode::Scale, "Scale along the entity's own axes, or all from the centre (R); Ctrl snaps to 0.1"),
            ] {
                let on = s.ins.tool.mode == mode;
                let r = ui.button(
                    id,
                    text,
                    small(if on {
                        ButtonVariant::Primary
                    } else {
                        ButtonVariant::Secondary
                    }),
                );
                if r.clicked {
                    s.ins.tool.mode = mode;
                }
                ui.tooltip(tip);
            }
            ui.spacer("Gap", None);
            let (text, next) = match s.ins.tool.space {
                Space::World => ("World", Space::Local),
                Space::Local => ("Local", Space::World),
            };
            if ui
                .button("Space", text, small(ButtonVariant::Quiet))
                .clicked
            {
                s.ins.tool.space = next;
            }
            ui.tooltip("Move and rotate along the world's axes or the entity's own");
        },
    );
}
