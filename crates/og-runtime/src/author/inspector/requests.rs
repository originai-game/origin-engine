//! The panel's requests applied: edits, saves and what follows an edit.

use super::*;

pub(super) fn apply(eng: &Engine, request: Request) {
    let set = |target: &str, text: &str| -> Result<(), String> {
        crate::author::probe::with_probe(eng, false, |p, w| p.tune(w, target, text))?
    };
    let note = |status: String| {
        if let Ok(mut rs) = eng.render.try_borrow_mut()
            && let Some(ins) = rs.inspector.as_mut()
        {
            ins.status = status;
        }
    };
    match request {
        Request::Tune { edit, merge } => match set(&edit.target, &edit.after) {
            Ok(()) => after_edit(eng, edit, merge),
            Err(e) => note(e),
        },
        Request::Undo | Request::Redo => {
            let step = {
                let Ok(mut rs) = eng.render.try_borrow_mut() else {
                    return;
                };
                let Some(ins) = rs.inspector.as_mut() else {
                    return;
                };
                if matches!(request, Request::Undo) {
                    ins.history.undo()
                } else {
                    ins.history.redo()
                }
            };
            if let Some((target, text)) = step {
                match set(&target, &text) {
                    Ok(()) => {
                        if let Ok(mut rs) = eng.render.try_borrow_mut()
                            && let Some(ins) = rs.inspector.as_mut()
                        {
                            ins.dirty.insert(target);
                        }
                    }
                    Err(e) => note(e),
                }
            }
        }
        Request::Prefab {
            entity,
            target,
            fields,
        } => {
            let edit = og_probe::PrefabEdit {
                entity,
                target: target.clone(),
            };
            let done = crate::author::probe::with_probe(eng, false, |p, w| {
                let id = p.names().id_of(w, entity);
                p.prefab_edit(w, &edit).map(|_| id)
            });
            match done.and_then(|r| r) {
                Ok(id) => {
                    let what = target.unwrap_or_else(|| "every override".into());
                    let Ok(mut rs) = eng.render.try_borrow_mut() else {
                        return;
                    };
                    let Some(ins) = rs.inspector.as_mut() else {
                        return;
                    };
                    let covers = |t: &str| {
                        t.strip_prefix(&format!("{id}.")).is_some_and(|rest| {
                            fields.iter().any(|f| panel::prefab_field(rest) == *f)
                        })
                    };
                    // A memory-only revert does not change the scene's TS
                    // overrides. Keep the source discrepancy visible.
                    ins.watch.retain(|w| !covers(&w.target));
                    ins.prefab = None;
                    ins.status =
                        format!("Reverted {what} in memory; use og set to persist TS source");
                }
                Err(e) => note(e),
            }
        }
        Request::Pause => {
            eng.probe.borrow_mut().pause();
        }
        Request::Resume => {
            eng.probe.borrow_mut().resume();
        }
        Request::Step => {
            if let Err(e) = crate::engine::advance_probe(eng, 1) {
                note(e);
            }
        }
        Request::Close => {
            if let Ok(mut rs) = eng.render.try_borrow_mut()
                && let Some(ins) = rs.inspector.as_mut()
            {
                ins.open = false;
                ins.panel = None;
                ins.editing = false;
                ins.draft = None;
                ins.tool.drag = None;
                ins.tool.hover = None;
            }
        }
    }
}

/// An edit that took: remembered for undo, marked unsaved and watched.
pub(super) fn after_edit(eng: &Engine, edit: Edit, merge: bool) {
    let Ok(world) = eng.world() else {
        return;
    };
    let tick = world.time().tick;
    let layouts = eng.state.borrow().layouts.clone();
    let (entity, component, field) = {
        let probe = eng.probe.borrow();
        split_target(world, probe.names(), &edit.target)
    };
    let shown = panel::read_parts(world, &layouts, entity, &component, &field);
    let Ok(mut rs) = eng.render.try_borrow_mut() else {
        return;
    };
    let Some(ins) = rs.inspector.as_mut() else {
        return;
    };
    ins.dirty.insert(edit.target.clone());
    ins.flash.remove(&edit.target);
    ins.watch.retain(|w| w.target != edit.target);
    if let Some(shown) = shown {
        ins.watch.push(Watch {
            target: edit.target.clone(),
            entity,
            component,
            field,
            shown,
            tick,
        });
    }
    ins.status.clear();
    if merge {
        ins.history.merge(edit);
    } else {
        ins.history.push(edit);
    }
}

/// A `set` target's entity (none for a resource), component and field.
pub(super) fn split_target(
    world: &World,
    names: &og_probe::Names,
    target: &str,
) -> (Option<Entity>, String, String) {
    let (head, field) = target.rsplit_once('.').unwrap_or((target, ""));
    if world
        .id_by_name(head)
        .is_ok_and(|id| world.has_resource_id(id))
    {
        return (None, head.to_string(), field.to_string());
    }
    let (entity, component) = match head.strip_prefix('#') {
        Some(rest) => {
            let mut p = rest.splitn(3, '.');
            match (p.next(), p.next(), p.next()) {
                (Some(i), Some(g), Some(c)) => (format!("#{i}.{g}"), c.to_string()),
                _ => (head.to_string(), String::new()),
            }
        }
        None => head
            .split_once('.')
            .map(|(e, c)| (e.to_string(), c.to_string()))
            .unwrap_or_default(),
    };
    (names.resolve(world, &entity), component, field.to_string())
}
