//! The tick: whether the session lets it run, and what follows it.

use super::*;

/// og.Lockstep's settings as the world holds them.
pub(super) fn config(eng: &Engine, world: &World) -> Config {
    let layout = eng.state.borrow().layouts.get(LOCKSTEP).cloned();
    let v = layout.and_then(|l| {
        let id = world.id_by_name(LOCKSTEP).ok()?;
        l.read::<record::Lockstep>(&world.resource_bytes(id).ok()?)
            .ok()
    });
    let Some(v) = v else {
        return Config::default();
    };
    Config {
        enabled: v.enabled,
        players: v.players.clamp(1, 16),
        input_delay: v.input_delay.clamp(1, 30),
        hash_every: v.hash_every,
        rollback: v.mode == NetMode::Rollback,
        max_prediction: v.max_prediction.clamp(1, 30),
        stall_timeout: v.stall_timeout.min(600_000),
        rejoin: v.rejoin,
        budget: with(eng.id, |n| n.cost.budget(f64::from(world.time().dt))),
        state_bytes: with(eng.id, |n| n.shape.state_bytes as u32),
        shape: with(eng.id, |n| n.shape.fingerprint),
    }
}

/// The probe's tick gate: whether the next tick may run.
/// Whether lockstep (or rollback) runs on this engine now: its world is
/// shared with the room's other peers.
pub(crate) fn session_running(engine: u32) -> bool {
    crate::engine::engine(engine)
        .is_ok_and(|eng| eng.modules.net.try_borrow().is_ok_and(|n| n.ls.running()))
}

pub(crate) fn gate(engine: u32, world: &mut World) -> bool {
    if !with(engine, |n| n.ls.me().is_some()) {
        return true;
    }
    let Ok(eng) = engine_of(engine) else {
        return true;
    };
    let cfg = config(&eng, world);
    if cfg.enabled {
        // Before the start's snapshot, so every peer's world holds them.
        for name in SEAT_EVENTS {
            if let Err(e) = crate::engine::lazy::ensure(&eng, world, name) {
                eng.state
                    .borrow_mut()
                    .log
                    .push(json!({ "tick": world.time().tick, "warning": true, "text": e }));
            }
        }
    }
    // A run that could not stay in step is refused before it starts: the
    // room stays a lobby, and the game hears why once.
    if with(engine, |n| n.ls.would_start(&cfg))
        && let Some(why) = crate::world::stream::cells::session_refusal(&eng, world)
    {
        let text = format!("lockstep: not started: {why}");
        let mut st = eng.state.borrow_mut();
        if st.ui_warned.insert(text.clone()) {
            st.log
                .push(json!({ "tick": world.time().tick, "warning": true, "text": text }));
        }
        return true;
    }
    loop {
        let tick = world.time().tick;
        let gate = with(engine, |n| {
            n.ls.set_clock(crate::author::probe::now_ms() as u64);
            let g = n.ls.gate(&cfg, tick, &mut || world.snapshot());
            let players = n.ls.players().unwrap_or(0);
            for (peer, seat, ms) in n.ls.take_stalls() {
                n.pending.push((
                    PEER_STALLED,
                    json!({ "peer": peer, "seat": seat.unwrap_or(players), "silentMs": ms.min(u64::from(u32::MAX)) }),
                ));
            }
            flush(engine, n, tick);
            g
        });
        match gate {
            Gate::Go => {
                with(engine, |n| {
                    n.cost.started = Some(crate::author::probe::now_ms())
                });
                return true;
            }
            Gate::Wait => return false,
            Gate::Rewind { snapshot, tick: to } => {
                let t0 = crate::author::probe::now_ms();
                if let Err(e) = crate::world::save::rewind(&eng, world, &snapshot, to, &[LOCKSTEP])
                {
                    eng.state.borrow_mut().log.push(json!({
                        "tick": tick, "warning": true,
                        "text": format!("rollback: the rewind to tick {to} failed: {e}; this peer leaves the room"),
                    }));
                    with(engine, |n| n.outbox.push(protocol::leave()));
                    return true;
                }
                if let Some(link) = og_audio::link(engine) {
                    let tick_ms = (f64::from(world.time().dt) * 1000.0).max(1.0);
                    link.rewound(to, tick, (LATE_PLAY_MS as f64 / tick_ms).ceil() as u64);
                }
                let ms = crate::author::probe::now_ms() - t0;
                with(engine, |n| Cost::note(&mut n.cost.rewind_ms, ms));
            }
            Gate::Restore { snapshot, .. } => {
                if let Err(e) =
                    crate::world::save::restore_session(&eng, world, &snapshot, &[LOCKSTEP])
                {
                    eng.state.borrow_mut().log.push(json!({
                        "tick": tick, "warning": true,
                        "text": format!("lockstep: the peer's snapshot was refused: {e}; this peer leaves lockstep"),
                    }));
                    with(engine, |n| n.outbox.push(protocol::leave()));
                    return true;
                }
            }
        }
    }
}

/// After every tick: this peer's input for the tick inputDelay on, and the
/// world's hash.
pub(crate) fn after_tick(engine: u32, world: &mut World) {
    if !with(engine, |n| n.ls.running()) {
        return;
    }
    let tick = world.time().tick;
    with(engine, |n| {
        let ran = crate::author::probe::now_ms();
        // This tick's commands go with this peer's input inputDelay on,
        // after its state (whole, so the commands start where every peer
        // reads them).
        let given = std::mem::take(&mut n.given);
        if !given.is_empty() {
            if n.ls.local().len() != n.shape.state_bytes {
                let zero = state_zero(engine, n.shape.input());
                n.ls.set_local(zero);
            }
            for c in &given {
                n.ls.push_commands(c);
            }
        }
        n.ls.after_tick(tick, &mut || {
            let bytes = world.snapshot();
            (shared_hash(&bytes).unwrap_or(0), bytes)
        });
        // A re-simulated tick pays for its snapshot and hash too, so the
        // budget counts them.
        let kept = crate::author::probe::now_ms();
        Cost::note(&mut n.cost.keep_ms, kept - ran);
        if let Some(t0) = n.cost.started.take() {
            Cost::note(&mut n.cost.tick_ms, kept - t0);
        }
        flush(engine, n, tick);
    });
}

pub(super) fn send(
    ctx: &og_ecs::SystemContext<'_>,
    layouts: &BTreeMap<String, crate::host::codec::Layout>,
    name: &str,
    v: &Value,
) -> og_core::Result<()> {
    let Some(layout) = layouts.get(name) else {
        return Ok(());
    };
    let bytes = layout
        .encode(None, Some(v))
        .map_err(|e| og_core::Error::new(og_core::ErrorKind::Invalid, e))?;
    ctx.send_bytes(ctx.id_by_name(name)?, &bytes)
}
