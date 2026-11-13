//! Input the inspector takes before the game sees it.

use super::*;

/// The probe's input filter: true takes the event from the game.
pub(crate) fn filter(engine_id: u32, world: &World, event: &InjectedInput) -> bool {
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
    let rate = tick_rate(world);
    let ins = of(&mut rs, rate);
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
            // A gizmo drag has the pointer until it ends.
            if let Some(d) = ins.tool.drag.as_mut() {
                d.at = Vec2::new(*x, *y);
                if *action == PointerAction::Up {
                    d.released = true;
                }
                // Its own UI saw the press: it sees the release too.
                ins.input.apply(event);
                return true;
            }
            let over = ins.over_panel(*x, *y);
            let mut take = over || ins.captured;
            if !over && !ins.captured {
                ins.tool.hover = ins.tool.hit(*x, *y);
            }
            match action {
                PointerAction::Down => {
                    if over {
                        ins.captured = true;
                    } else if !ins.picking
                        && matches!(button, PointerButton::Left | PointerButton::None)
                        && ins.tool.begin(*x, *y)
                    {
                        // A field being edited is left as a click
                        // elsewhere leaves it.
                        ins.input.apply(event);
                        return true;
                    } else if ins.picking {
                        if ins.tab == ui_debug::TAB {
                            ins.ui_pick = Some((*x, *y));
                        } else {
                            ins.pick_at = Some((*x, *y));
                        }
                        ins.picking = false;
                        ins.captured = true;
                        take = true;
                    }
                }
                PointerAction::Up => ins.captured = false,
                _ => {}
            }
            // Its own UI sees every move (to leave a hover) and the presses
            // it takes; a press outside also reaches it, which ends a
            // field's edit as a click elsewhere does.
            ins.input.apply(event);
            take
        }
        InjectedInput::Wheel { x, y, .. } => {
            let over = ins.over_panel(*x, *y);
            if over {
                ins.input.apply(event);
            }
            over
        }
        InjectedInput::Key { .. } | InjectedInput::Text { .. } => {
            // Keys are the panel's while a field of it is edited, or while
            // the pointer rests on it (Ctrl+Z, Escape).
            let pointer = ins.input.pointer;
            let over = ins.over_panel(pointer.x, pointer.y);
            // W, E and R pick the gizmo's mode: over the panel, or anywhere
            // when the game binds none of them.
            if let InjectedInput::Key { key, action, .. } = event
                && !ins.editing
                && let Some(mode) = gizmo::Mode::of_key(key)
                && (over || !needs_ctrl(key))
            {
                if *action == KeyAction::Down {
                    ins.tool.mode = mode;
                }
                return true;
            }
            let take = ins.editing || over;
            if take {
                match event {
                    InjectedInput::Text { text, composing } => ins
                        .typed
                        .extend(og_input::TextInput::split(text, *composing)),
                    _ => {
                        ins.input.apply(event);
                    }
                }
            }
            take
        }
        _ => false,
    }
}
