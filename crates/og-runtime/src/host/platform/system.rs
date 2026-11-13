//! The og.platform system and the probe's dump.

use super::*;

/// Registers og.PlatformAnswered and the og.platform system (front).
pub(crate) fn mount(world: &mut World, mount: crate::engine::Mount<'_>) -> Result<(), String> {
    let engine_id = mount.id;
    world
        .register::<PlatformAnswered>()
        .map_err(|e| e.reason().to_string())?;
    world
        .add_system(
            SystemDesc::new(PLATFORM_SYSTEM).write::<PlatformAnswered>(),
            move |ctx| {
                let tick = ctx.time().tick;
                let ms = tick_ms(engine_id);
                let arrived = with(engine_id, |p| {
                    let inbox = std::mem::take(&mut p.inbox);
                    let mut arrived = Vec::new();
                    for (id, v) in inbox {
                        if !p.log.contains_key(&id) {
                            continue;
                        }
                        let ok = v["ok"].as_bool() == Some(true);
                        if let Some(t) = time_of(&v) {
                            rebase(p, t, tick, v["clock"].as_str(), ms);
                        }
                        if let Some(c) = v["clock"].as_str() {
                            p.clock = Some(c.to_string());
                        }
                        // A slot call's store call: the slot call moves on,
                        // and only its own answer is sent.
                        let slot = p.sub.contains_key(&id);
                        let done = slot.then(|| advance_slot(p, id, &v, tick)).flatten();
                        let host_time = time_of(&v);
                        if let Some(r) = p.log.get_mut(&id) {
                            r.answer = Some((v, tick));
                        }
                        if let Some((outer, mut answer)) = done {
                            // Every answer carries the host's time (og-platform-sdk
                            // §1): a slot call's, that of the store answer that
                            // ended it, or the clock moved on to this tick.
                            if time_of(&answer).is_none()
                                && let Some(t) = host_time.or_else(|| {
                                    p.base
                                        .map(|(t, at)| t + tick.saturating_sub(at) as f64 * ms)
                                })
                            {
                                answer["time"] = json!(t);
                            }
                            let ok = answer["ok"].as_bool() == Some(true);
                            if let Some(r) = p.log.get_mut(&outer) {
                                r.answer = Some((answer, tick));
                            }
                            arrived.push((outer, ok));
                        }
                        if !slot {
                            arrived.push((id, ok));
                        }
                    }
                    arrived
                });
                // Notices: numbered, kept, and announced when the game named
                // og.PlatformNotice (registered with the world).
                let notices = with(engine_id, |p| {
                    let mut out = Vec::new();
                    for v in std::mem::take(&mut p.notices_in) {
                        if let Some(t) = time_of(&v) {
                            rebase(p, t, tick, v["clock"].as_str(), ms);
                        }
                        p.next_notice = p.next_notice.wrapping_add(1).max(1);
                        let n = p.next_notice;
                        let kind = v["kind"].as_str().unwrap_or("").to_string();
                        out.push((n, kind.clone()));
                        p.notices.insert(
                            n,
                            Notice {
                                kind,
                                data: v.get("data").cloned().unwrap_or(Value::Null),
                                time: time_of(&v).unwrap_or(0.0),
                                tick,
                            },
                        );
                        while p.notices.len() > KEEP_NOTICES {
                            p.notices.pop_first();
                        }
                    }
                    out
                });
                for (request, ok) in arrived {
                    ctx.send(PlatformAnswered { request, ok })?;
                }
                if !notices.is_empty()
                    && let Ok(id) = ctx.id_by_name(NOTICE)
                {
                    let layout = engine(engine_id)
                        .ok()
                        .and_then(|e| e.state.borrow().layouts.get(NOTICE).cloned());
                    if let Some(layout) = layout {
                        for (notice, kind) in notices {
                            let bytes = layout
                                .encode(None, Some(&json!({ "notice": notice, "kind": kind })))
                                .map_err(|e| og_core::Error::new(og_core::ErrorKind::Invalid, e))?;
                            ctx.send_bytes(id, &bytes)?;
                        }
                    }
                }
                Ok(())
            },
        )
        .map_err(|e| e.reason().to_string())
}

/// `dump_platform`'s JSON: the host, its capabilities and every call.
pub(crate) fn dump(engine: u32) -> Value {
    with(engine, |p| {
        json!({
            "host": p.host.clone().unwrap_or_else(|| "none".into()),
            "capabilities": p.capabilities,
            "calls": p.log.iter().map(|(id, r)| {
                let mut c = json!({ "request": id, "method": r.method, "args": r.args, "tick": r.tick, "oneway": r.oneway });
                match (&r.answer, r.oneway) {
                    (_, true) => c["state"] = json!("sent"),
                    (None, _) => c["state"] = json!("pending"),
                    (Some(_), _) => {
                        c["state"] = json!("answered");
                        c["result"] = result_of(*id, r).to_json();
                    }
                }
                c
            }).collect::<Vec<_>>(),
            "notices": p.notices.iter().map(|(n, x)| json!({
                "notice": n, "kind": x.kind, "data": x.data, "tick": x.tick, "time": x.time,
            })).collect::<Vec<_>>(),
        })
    })
}
