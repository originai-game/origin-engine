//! A pending restore applied between ticks: whole worlds, sessions, rewinds and selections.

use super::*;

/// The probe's end-of-tick hook: a restore asked for during the tick.
pub(crate) fn after_tick(eng: &Engine, world: &mut World) {
    let Some(pending) = eng.state.borrow_mut().pending_restore.take() else {
        return;
    };
    if let Err(e) = apply(eng, world, &pending) {
        let tick = world.time().tick;
        eng.state.borrow_mut().log.push(serde_json::json!({
            "tick": tick, "warning": true, "text": format!("world.restore: {e}"),
        }));
    }
}

/// The probe's start-of-tick hook: after a restore, the tick that follows
/// delivers og.WorldRestored (`from`, the tick the world was at, and
/// `tick`, the saved one), so systems can reset what lives in the session
/// (request numbers, settings read from the store). Sent here rather than
/// when the restore applies, so the world between the two hashes as the
/// save did.
pub(crate) fn before_tick(eng: &Engine, world: &mut World) {
    let Some((from, to)) = eng.state.borrow_mut().restored.take() else {
        return;
    };
    let Some(layout) = eng.state.borrow().layouts.get(RESTORED).cloned() else {
        return;
    };
    let sent = world.id_by_name(RESTORED).and_then(|id| {
        let v = serde_json::json!({ "from": from, "tick": to });
        let bytes = layout
            .encode(None, Some(&v))
            .map_err(|e| og_core::Error::new(og_core::ErrorKind::Invalid, e))?;
        world.send_bytes(id, &bytes)
    });
    if let Err(e) = sent {
        eng.state.borrow_mut().log.push(serde_json::json!({
            "tick": to, "warning": true, "text": format!("{RESTORED}: {e}"),
        }));
    }
}

/// The event a restore delivers the next tick.
pub(super) const RESTORED: &str = "og.WorldRestored";

/// Restores a checked save, keeping the session, and brings the runtime's
/// own view of the world (names, views, the platform clock, the renderer's
/// history) along.
pub(super) fn apply(eng: &Engine, world: &mut World, pending: &Pending) -> Result<(), String> {
    apply_keeping(eng, world, pending, &[])
}

/// A peer's world snapshot (og.net's lockstep start or late join), restored
/// as a save is, keeping the session and the resources `keep` names too.
pub(crate) fn restore_session(
    eng: &Engine,
    world: &mut World,
    snapshot: &[u8],
    keep: &[&str],
) -> Result<(), String> {
    crate::engine::lazy::before_restore(eng, world, snapshot);
    world
        .check_snapshot(snapshot, false)
        .map_err(|e| refusal(e, false))?;
    let pending = Pending {
        snapshot: snapshot.to_vec(),
        migrate: false,
        session: false,
        ui: None,
    };
    apply_keeping(eng, world, &pending, keep)
}

/// Rollback's rewind (og.net, G2-7): the world back to `snapshot`, which
/// is this process's own snapshot of tick `tick` from lockstep's ring
/// (og-net's `Gate::Rewind`), never a save and never a peer's bytes. So
/// nothing is checked against a game's saves or migrated (the bytes are
/// this build's, taken from this world a few ticks ago), no og.WorldRestored
/// is sent (the game did not load anything: the past runs again), and the
/// platform's timers and the renderer's history keep their ticks (the
/// ticks run forward again to where they were). The session (input, the
/// view, og.Lockstep and the player's bindings) is kept, as for a restore;
/// names, views and queries follow the world. Only og.net calls it.
pub(crate) fn rewind(
    eng: &Engine,
    world: &mut World,
    snapshot: &[u8],
    tick: u64,
    keep: &[&str],
) -> Result<(), String> {
    let kept: Vec<(og_ecs::SchemaId, Vec<u8>)> = SESSION
        .iter()
        .chain(keep.iter())
        .filter_map(|name| {
            let id = world.id_by_name(name).ok()?;
            Some((id, world.resource_bytes(id).ok()?))
        })
        .collect();
    let actions = eng.state.borrow().actions.clone();
    let kept_actions = actions.as_ref().map(|a| og_input::actions::lock(a).clone());
    world
        .restore(snapshot)
        .map_err(|e| e.reason().to_string())?;
    debug_assert_eq!(
        world.time().tick,
        tick,
        "a rewind's bytes are the ring's snapshot of its tick"
    );
    for (id, bytes) in kept {
        let _ = world.insert_resource_bytes(id, &bytes);
    }
    og_audio::restored(world).map_err(|e| e.reason().to_string())?;
    if let (Some(a), Some(map)) = (actions, kept_actions) {
        og_input::actions::lock(&a).keep_player(&map);
    }
    // The renderer snaps to the corrected world: nothing to blend from.
    if let Ok(mut rs) = eng.render.try_borrow_mut() {
        rs.interp.forget();
        rs.motion.forget();
    }
    {
        let mut st = eng.state.borrow_mut();
        st.names.clear();
        st.name_of.clear();
        st.name_ops.clear();
        for (e, name) in world.names() {
            st.names.insert(name.to_string(), e.to_bits());
            st.name_of.insert(e.to_bits(), name.to_string());
        }
    }
    crate::engine::invalidate(true);
    crate::engine::clear_world_queries();
    Ok(())
}

/// Restores a selection: the entities carrying its tags are replaced by
/// its own, its resources set, the rest of the world (the session too)
/// left as it is. References to a saved entity in an Entity field follow
/// it to its new id. The tick after it delivers og.WorldRestored, `from`
/// and `tick` both the current tick (og.Time is not in a selection).
pub(super) fn apply_selection(
    eng: &Engine,
    world: &mut World,
    pending: &Pending,
) -> Result<(), String> {
    let entity_fields = |ty: &str| -> Vec<usize> {
        eng.state
            .borrow()
            .layouts
            .get(ty)
            .map(|l| {
                l.fields
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| f.kind == crate::host::codec::Kind::Entity)
                    .map(|(i, _)| i)
                    .collect()
            })
            .unwrap_or_default()
    };
    let lenient = |ty: &str| SESSION.contains(&ty);
    crate::engine::lazy::before_restore(eng, world, &pending.snapshot);
    world
        .restore_selection(
            &pending.snapshot,
            pending.migrate,
            &lenient,
            &engine_owned,
            &entity_fields,
        )
        .map_err(|e| refusal(e, pending.migrate))?;
    let tick = world.time().tick;
    refresh(eng, world, tick, tick);
    Ok(())
}

/// The runtime's own view of the world after a restore: names, views, the
/// platform clock, the renderer's history.
pub(super) fn refresh(eng: &Engine, world: &World, from: u64, to: u64) {
    if let Some(link) = og_audio::link(eng.id) {
        link.clear_feedback();
    }
    {
        let mut st = eng.state.borrow_mut();
        st.names.clear();
        st.name_of.clear();
        st.name_ops.clear();
        for (e, name) in world.names() {
            st.names.insert(name.to_string(), e.to_bits());
            st.name_of.insert(e.to_bits(), name.to_string());
        }
    }
    eng.state.borrow_mut().restored = Some((from, to));
    crate::host::platform::shift_ticks(eng.id, from, to);
    if let Ok(mut rs) = eng.render.try_borrow_mut() {
        // A cut: the presentation state (the renderer's history: exposure,
        // TAA; the particles' ticks) starts afresh.
        crate::present::presentation::restore(&mut rs, 0);
    }
    crate::engine::invalidate(true);
    crate::engine::clear_world_queries();
}

pub(super) fn apply_keeping(
    eng: &Engine,
    world: &mut World,
    pending: &Pending,
    keep: &[&str],
) -> Result<(), String> {
    if is_selection(&pending.snapshot) {
        return apply_selection(eng, world, pending);
    }
    if let (Some(saved), Some(ui)) = (&pending.ui, &eng.state.borrow().ui) {
        saved.validate_for(ui)?;
    }
    let from = world.time().tick;
    // Nothing for a frame to blend from until the next tick.
    if let Ok(mut rs) = eng.render.try_borrow_mut() {
        rs.interp.forget();
        rs.motion.forget();
    }
    // With RestoreDesc.session the save's input too: only the audio device stays the host's.
    let kept: Vec<(og_ecs::SchemaId, Vec<u8>)> = SESSION
        .iter()
        .filter(|name| !pending.session || **name == "og.Audio")
        .chain(keep.iter())
        .filter_map(|name| {
            let id = world.id_by_name(name).ok()?;
            Some((id, world.resource_bytes(id).ok()?))
        })
        .collect();
    // The player's bindings and dead zones outlive a load (the actions'
    // state is the save's, as og.Input's edges come from what follows).
    let actions = eng.state.borrow().actions.clone();
    let kept_actions = actions.as_ref().map(|a| og_input::actions::lock(a).clone());
    crate::engine::lazy::before_restore(eng, world, &pending.snapshot);
    if pending.migrate {
        world
            .restore_migrating(&pending.snapshot)
            .map_err(|e| refusal(e, true))?;
    } else {
        world
            .restore(&pending.snapshot)
            .map_err(|e| refusal(e, false))?;
    }
    for (id, bytes) in kept {
        let _ = world.insert_resource_bytes(id, &bytes);
    }
    og_audio::restored(world).map_err(|e| e.reason().to_string())?;
    if let (Some(a), Some(map)) = (actions, kept_actions) {
        og_input::actions::lock(&a).keep_player(&map);
    }
    let to = world.time().tick;
    refresh(eng, world, from, to);
    if let (Some(saved), Some(ui)) = (&pending.ui, &eng.state.borrow().ui) {
        saved.restore(ui)?;
    }
    Ok(())
}
