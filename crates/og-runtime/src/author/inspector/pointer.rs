//! Input the inspector takes before the game sees it. The panel is the
//! game's UI: what lands on it goes on to the UI tree.

use super::*;

/// The probe's input filter: true takes the event from the game.
pub(crate) fn filter(engine_id: u32, _world: &World, event: &InjectedInput) -> bool {
    let Ok(eng) = crate::engine::engine(engine_id) else {
        return false;
    };
    let needs_ctrl = |key: &str| {
        eng.state
            .try_borrow()
            .ok()
            .and_then(|st| st.actions.clone())
            .is_some_and(|a| model::game_binds(&og_input::actions::lock(&a), key))
    };
    let Ok(mut rs) = eng.render.try_borrow_mut() else {
        return false;
    };
    let ins = of(&mut rs);
    if let InjectedInput::Key {
        key,
        action,
        modifiers,
    } = event
    {
        ins.tool.ctrl = match key.as_str() {
            "ControlLeft" | "ControlRight" => *action == KeyAction::Down,
            _ => modifiers.control,
        };
    }
    match event {
        InjectedInput::Key {
            key,
            action,
            modifiers,
        } if *key == ins.key => {
            if *action == KeyAction::Up {
                return std::mem::take(&mut ins.toggle_held);
            }
            ins.needs_ctrl = needs_ctrl(key);
            if ins.needs_ctrl && !modifiers.control {
                return false;
            }
            ins.open = !ins.open;
            ins.toggle_held = true;
            true
        }
        _ if !ins.open => false,
        InjectedInput::Pointer {
            action,
            x,
            y,
            id,
            button,
            ..
        } => {
            if *id != 0 {
                return false;
            }
            ins.pointer = Vec2::new(*x, *y);
            // A gizmo drag has the pointer until it ends.
            if let Some(d) = ins.tool.drag.as_mut() {
                d.at = ins.pointer;
                if *action == PointerAction::Up {
                    d.released = true;
                }
                return true;
            }
            if ins.over_panel(*x, *y) {
                ins.tool.hover = None;
                return false;
            }
            ins.tool.hover = ins.tool.hit(*x, *y);
            if *action != PointerAction::Down {
                return false;
            }
            if ins.picking {
                if ins.tab == ui_debug::TAB {
                    ins.ui_pick = Some((*x, *y));
                } else {
                    ins.pick_at = Some((*x, *y));
                }
                ins.picking = false;
                return true;
            }
            matches!(button, PointerButton::Left | PointerButton::None) && ins.tool.begin(*x, *y)
        }
        InjectedInput::Key {
            key,
            action,
            modifiers,
        } if !ins.editing => {
            let over = ins.over_panel(ins.pointer.x, ins.pointer.y);
            // Ctrl+Z and Ctrl+Y over the panel undo and redo its edits.
            let shortcut = modifiers.control || modifiers.platform;
            if shortcut && over && matches!(key.as_str(), "KeyZ" | "KeyY") {
                if *action == KeyAction::Down {
                    ins.requests.push(if key == "KeyY" || modifiers.shift {
                        Request::Redo
                    } else {
                        Request::Undo
                    });
                }
                return true;
            }
            // W, E and R pick the gizmo's mode: over the panel, or anywhere
            // when the game binds none of them.
            match gizmo::Mode::of_key(key) {
                Some(mode) if over || !needs_ctrl(key) => {
                    if *action == KeyAction::Down {
                        ins.tool.mode = mode;
                    }
                    true
                }
                _ => false,
            }
        }
        _ => false,
    }
}
