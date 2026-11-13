//! The inspector's probe commands.

use super::*;

/// The gizmo as `dump_inspector` reports it: its mode and space, the
/// handle under the pointer or dragged, and each handle's grab point on
/// the surface (CSS px), what a script presses to drag it.
pub(super) fn gizmo_json(ins: &Inspector) -> serde_json::Value {
    let t = &ins.tool;
    let handles: Vec<serde_json::Value> = match (t.view, t.on) {
        (Some(view), Some(on)) => on
            .gizmo
            .handles()
            .into_iter()
            .filter_map(|h| {
                let p = view.project(on.gizmo.grab_point(&view, h))?;
                Some(serde_json::json!({ "handle": h.name(), "x": p.x, "y": p.y }))
            })
            .collect(),
        _ => Vec::new(),
    };
    serde_json::json!({
        "mode": t.mode.name(),
        "space": t.space.name(),
        "shown": t.on.is_some(),
        "hover": t.hover.map(|h| h.name()),
        "dragging": t.drag.as_ref().map(|d| d.drag.handle.name()),
        "handles": handles,
    })
}

/// The command channel's `inspect` and `dump_inspector`.
pub(crate) fn register(probe: &mut og_probe::Probe, engine_id: u32) {
    let _ = probe.register_dump(
        "dump_inspector",
        og_probe::CommandDesc {
            usage: "dump_inspector[:path]".into(),
            doc: "the inspector's panel as data: whether it is open, the selected entity, the unsaved fields, its status line and every element of its last frame with its rectangle in CSS px (what touch_move and type reach)".into(),
        },
        0,
        move |world, _args| {
            let eng = crate::engine::engine(engine_id)?;
            let rs = eng.render.try_borrow().map_err(|_| "the renderer is busy")?;
            let Some(ins) = rs.inspector.as_ref() else {
                return Ok(serde_json::json!({ "open": false, "elements": [] }));
            };
            let selected = ins
                .selected
                .filter(|e| world.is_alive(*e))
                .map(|e| og_probe::Names.id_of(world, e));
            let elements: Vec<serde_json::Value> = if ins.open {
                ins.ui
                    .as_ref()
                    .map(|u| u.probe_elements().iter().map(|e| e.to_json()).collect())
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            Ok(serde_json::json!({
                "open": ins.open,
                "key": ins.key_label(),
                "selected": selected,
                "ui": {
                    "selected": ins.ui_selected,
                    "bounds": ins.ui_bounds,
                    "element": ins.ui_view.element.as_ref().map(|e| serde_json::json!({
                        "path": e.path, "kind": e.kind,
                        "box": [e.border_box.x, e.border_box.y, e.border_box.width, e.border_box.height],
                        "content": [e.content.x, e.content.y, e.content.width, e.content.height],
                        "styles": e.styles.iter().map(|s| serde_json::json!({ "name": s.name, "value": s.value, "source": s.source })).collect::<Vec<_>>(),
                        "bindings": e.bindings.iter().map(|(p, x, v)| serde_json::json!({ "prop": p, "expr": x, "value": v })).collect::<Vec<_>>(),
                        "boundaries": e.boundaries,
                    })),
                },
                "dirty": ins.dirty.iter().collect::<Vec<_>>(),
                "changedByGame": ins.flash.keys().collect::<Vec<_>>(),
                "status": ins.status,
                "gizmo": gizmo_json(ins),
                "elements": elements,
            }))
        },
    );
    let _ = probe.register_command(
        "inspect",
        og_probe::CommandDesc {
            usage: "inspect[:on|off|<entity>|ui[:<path>]|key:<code>|mode:translate|rotate|scale|space:world|local]".into(),
            doc: "the inspector (development builds): open it (on, or no argument), close it (off), open it on an entity (its name or #<index>.<generation>), open its UI view (ui) on a UI element (ui:<path>, as dump_ui names it), set the key that toggles it (key:F2; F1 by default, with Ctrl when the game binds it), or the transform gizmo's mode (mode:rotate; W, E and R) and axes (space:local)".into(),
        },
        move |world, args| {
            let eng = crate::engine::engine(engine_id)?;
            let arg = args.first().map(String::as_str).unwrap_or("on");
            let selected = match arg {
                "on" | "off" => None,
                "key" | "mode" | "space" | "ui" => None,
                id => Some(
                    og_probe::Names
                        .resolve(world, id)
                        .ok_or_else(|| format!("inspect:{id}: no live entity of that id"))?,
                ),
            };
            let rate = tick_rate(world);
            let mut rs = eng.render.borrow_mut();
            let ins = of(&mut rs, rate);
            match arg {
                "on" => ins.open = true,
                "off" => ins.open = false,
                "ui" => {
                    ins.open = true;
                    ins.tab = ui_debug::TAB;
                    if args.len() > 1 {
                        ins.ui_selected = Some(args[1..].join(":"));
                    }
                }
                "key" => {
                    let code = args.get(1).map(String::as_str).unwrap_or("");
                    if !og_input::KEY_CODES.contains(&code) {
                        return Err(format!(
                            "inspect:key:{code}: not a KeyboardEvent.code (F1, F2, Backquote)"
                        ));
                    }
                    ins.key = code.to_string();
                }
                "mode" => {
                    let m = args.get(1).map(String::as_str).unwrap_or("");
                    ins.tool.mode = match m {
                        "translate" => gizmo::Mode::Translate,
                        "rotate" => gizmo::Mode::Rotate,
                        "scale" => gizmo::Mode::Scale,
                        _ => return Err(format!("inspect:mode:{m}: translate, rotate or scale")),
                    };
                }
                "space" => {
                    let m = args.get(1).map(String::as_str).unwrap_or("");
                    ins.tool.space = match m {
                        "world" => gizmo::Space::World,
                        "local" => gizmo::Space::Local,
                        _ => return Err(format!("inspect:space:{m}: world or local")),
                    };
                }
                _ => {
                    ins.open = true;
                    ins.selected = selected;
                    ins.tab = 0;
                }
            }
            Ok(())
        },
    );
}
