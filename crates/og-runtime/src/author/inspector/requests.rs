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
            apply,
        } => {
            let edit = og_probe::PrefabEdit {
                entity,
                target: target.clone(),
                apply,
                base: false,
                write: WRITES_FILES,
            };
            let done = crate::author::probe::with_probe(eng, false, |p, w| {
                let id = p.names().id_of(w, entity);
                p.prefab_edit(w, &edit).map(|saved| (id, saved))
            });
            match done.and_then(|r| r) {
                Ok((id, saved)) => {
                    let written = saved.files.iter().filter(|f| !f.written.is_empty()).count();
                    let what = target.unwrap_or_else(|| "every override".into());
                    let verb = if apply { "Applied" } else { "Reverted" };
                    let Ok(mut rs) = eng.render.try_borrow_mut() else {
                        return;
                    };
                    let Some(ins) = rs.inspector.as_mut() else {
                        return;
                    };
                    // The fields it covered are the prefab's again: no
                    // longer unsaved here.
                    let covers = |t: &str| {
                        t.strip_prefix(&format!("{id}.")).is_some_and(|rest| {
                            fields.iter().any(|f| panel::prefab_field(rest) == *f)
                        })
                    };
                    ins.dirty.retain(|t| !covers(t));
                    ins.watch.retain(|w| !covers(&w.target));
                    ins.prefab = None;
                    ins.status = if written > 0 {
                        format!("{verb} {what}: {written} files written")
                    } else {
                        hand_over(ins, &saved.files);
                        format!("{verb} {what}: the texts go to the page")
                    };
                }
                Err(e) => note(e),
            }
        }
        Request::Save => {
            let saved = crate::author::probe::with_probe(eng, false, |p, w| {
                p.save_scene(w, WRITES_FILES, false)
            });
            match saved.and_then(|r| r) {
                Ok(v) => {
                    let files = v["scenes"].as_array().map_or(0, Vec::len);
                    // `written` is the path written to (empty when the
                    // host writes no files), `lines` the changed lines.
                    let written = v["scenes"].as_array().map_or(0, |s| {
                        s.iter()
                            .filter(|f| f["written"].as_str().is_some_and(|w| !w.is_empty()))
                            .count()
                    });
                    let lines: usize = v["scenes"].as_array().map_or(0, |s| {
                        s.iter()
                            .map(|f| f["lines"].as_array().map_or(0, Vec::len))
                            .sum()
                    });
                    let skipped = v["skipped"].as_array().map_or(0, Vec::len);
                    let mut msg = if written > 0 {
                        format!("Saved {lines} lines to {written} of {files} scenes")
                    } else {
                        format!("{lines} lines ready; the page saves them")
                    };
                    if skipped > 0 {
                        msg.push_str(&format!(" ({skipped} not in scene text)"));
                    }
                    if let Ok(mut rs) = eng.render.try_borrow_mut()
                        && let Some(ins) = rs.inspector.as_mut()
                    {
                        if written > 0 {
                            ins.dirty.clear();
                        } else {
                            // A host that writes no files (the web page)
                            // hands the texts on: the frame's `saved`.
                            let files: Vec<og_probe::SavedFile> = v["scenes"]
                                .as_array()
                                .into_iter()
                                .flatten()
                                .map(|f| og_probe::SavedFile {
                                    source: f["source"].as_str().unwrap_or("").into(),
                                    written: String::new(),
                                    text: f["text"].as_str().unwrap_or("").into(),
                                    lines: f["lines"]
                                        .as_array()
                                        .map(|l| {
                                            l.iter()
                                                .filter_map(|n| n.as_u64())
                                                .map(|n| n as usize)
                                                .collect()
                                        })
                                        .unwrap_or_default(),
                                })
                                .collect();
                            if hand_over(ins, &files) {
                                ins.dirty.clear();
                            }
                        }
                        ins.prefab = None;
                        ins.status = msg;
                    }
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
                ins.captured = false;
                ins.tool.drag = None;
                ins.tool.hover = None;
            }
        }
    }
}

/// Keeps the changed scene texts of a save on a host that wrote none, for
/// the next frame's answer; true when there was one.
pub(super) fn hand_over(ins: &mut Inspector, files: &[og_probe::SavedFile]) -> bool {
    let before = ins.saved.len();
    for f in files
        .iter()
        .filter(|f| !f.lines.is_empty() && !f.source.is_empty())
    {
        ins.saved.retain(|(s, _)| *s != f.source);
        ins.saved.push((f.source.clone(), f.text.clone()));
    }
    ins.saved.len() > before || files.iter().any(|f| !f.lines.is_empty())
}

/// The scene texts the inspector saved since the last frame on a host
/// that writes no files, as the frame answer's `saved` (the web shell
/// sends them to the Stage or downloads them); none when there are none.
pub(crate) fn take_saved(rs: &mut RenderState) -> Option<serde_json::Value> {
    saved_of(rs.inspector.as_mut()?)
}

pub(super) fn saved_of(ins: &mut Inspector) -> Option<serde_json::Value> {
    if ins.saved.is_empty() {
        return None;
    }
    let scenes: Vec<serde_json::Value> = std::mem::take(&mut ins.saved)
        .into_iter()
        .map(|(source, text)| serde_json::json!({ "source": source, "text": text }))
        .collect();
    Some(serde_json::json!({ "scenes": scenes }))
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
