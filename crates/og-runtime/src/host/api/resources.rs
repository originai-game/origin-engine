//! World's resources and events.

use super::*;

pub(super) fn time_json(t: Time, scale: og_core::TimeScale, rate: u32) -> Value {
    let mut v =
        json!({ "tick": t.tick, "dt": f64::from(t.dt), "elapsed": t.elapsed, "rate": rate });
    if let (Value::Object(o), Value::Object(s)) =
        (&mut v, crate::present::feel::time_scale_json(scale, t))
    {
        o.extend(s);
    }
    v
}

pub(super) fn rng_json(r: &Rng) -> Value {
    json!({ "seed": r.seed(), "state": r.save() })
}

/// A resource's current value as JSON, outside or inside a system.
pub(crate) fn resource_value(eng: &Engine, name: &str) -> Result<Value, String> {
    let rate = eng.state.borrow().rate;
    if let Some(f) = running(eng.id) {
        // SAFETY: the frame is published while its system runs.
        let ctx = unsafe { f.ctx() };
        let st = eng.state.borrow();
        let s = &st.systems[f.system];
        let declared = s.reads.iter().chain(&s.writes).any(|r| r == name);
        if name == Time::NAME {
            return Ok(time_json(ctx.time(), ctx.time_scale(), rate));
        }
        if !declared {
            let system = f.system;
            drop(st);
            return Err(undeclared(eng, system, "read", name));
        }
        if name == Rng::NAME {
            return ctx.resource::<Rng>().map(|r| rng_json(&r)).map_err(err);
        }
        drop(st);
        let (layout, id) = running_record(eng, name, RecordKind::Resource)?;
        let bytes = ctx.resource_bytes(id).map_err(err)?;
        return Ok(layout.decode(&bytes));
    }
    let w = eng.world()?;
    if name == Time::NAME {
        return Ok(time_json(w.time(), w.time_scale(), rate));
    }
    if name == Rng::NAME {
        return w.resource::<Rng>().map(|r| rng_json(&r)).map_err(err);
    }
    let layout = eng
        .state
        .borrow()
        .layouts
        .get(name)
        .filter(|l| l.kind == RecordKind::Resource)
        .cloned()
        .ok_or_else(|| format!("'{name}' is not a registered resource"))?;
    // A lazy resource no game has named yet (og.TimeGroups): its default,
    // read without registering it, so reading leaves the hash alone.
    let Ok(id) = w.id_by_name(name) else {
        if crate::engine::lazy::is_deferred(eng, name) {
            return Ok(layout.decode(&layout.encode(None, None)?));
        }
        return Err(format!("no type named '{name}'"));
    };
    let bytes = w.resource_bytes(id).map_err(err)?;
    Ok(layout.decode(&bytes))
}

pub fn og_world_resource(world: u32, res: String) -> Result<serde_json::Value, String> {
    engine(world).and_then(|eng| resource_value(&eng, &res))
}

pub(super) fn event_layout(eng: &Engine, name: &str) -> Result<(Layout, og_ecs::SchemaId), String> {
    crate::engine::lazy::ensure(eng, eng.world()?, name)?;
    let st = eng.state.borrow();
    match st.layouts.get(name) {
        Some(l) if l.kind == RecordKind::Event => {
            let l = l.clone();
            drop(st);
            let id = eng.world()?.id_by_name(name).map_err(err)?;
            Ok((l, id))
        }
        _ => Err(st
            .warnings
            .iter()
            .find(|w| w.starts_with(&format!("{name} ")))
            .cloned()
            .unwrap_or_else(|| format!("'{name}' is not a registered event"))),
    }
}

/// `world.events`: inside a system, the events delivered this tick (sent
/// during the previous one), which the system must declare among its reads;
/// between ticks, the events sent since the last tick, which the next tick
/// delivers. The IDL answers a list, so a refusal is an empty list.
pub fn og_world_events(world: u32, event: String) -> Vec<Value> {
    let list = (|| -> Result<Vec<Value>, String> {
        let eng = engine(world)?;
        let bytes = if let Some(f) = running(world) {
            if crate::host::platform::is_platform_event(&event)
                && crate::host::platform::withheld(world)
            {
                return Ok(Vec::new());
            }
            let (layout, id) = running_record(&eng, &event, RecordKind::Event)?;
            // SAFETY: the frame is published while its system runs.
            let all = unsafe { f.ctx() }
                .events_bytes(id)
                .map_err(|_| undeclared(&eng, f.system, "read", &event))?;
            (layout, all)
        } else {
            let (layout, id) = event_layout(&eng, &event)?;
            let all = eng.world()?.pending_events_bytes(id).map_err(err)?;
            (layout, all)
        };
        Ok(bytes.1.iter().map(|b| bytes.0.decode(b)).collect())
    })();
    list.unwrap_or_default()
}

pub fn og_world_emit(world: u32, event: String, payload: String) -> Result<(), String> {
    if let Some(f) = running(world) {
        return (|| {
            let eng = engine(world)?;
            let (layout, id) = running_record(&eng, &event, RecordKind::Event)?;
            let declared = eng.state.borrow().systems[f.system].writes.contains(&event);
            if !declared {
                return Err(undeclared(&eng, f.system, "write", &event));
            }
            let bytes = layout.encode(None, Some(&parse(Some(&payload))?))?;
            // SAFETY: the frame is published while its system runs.
            unsafe { f.ctx() }
                .send_bytes(id, &bytes)
                .map_err(|_| undeclared(&eng, f.system, "write", &event))
        })();
    }
    (|| {
        let eng = engine(world)?;
        let (layout, id) = event_layout(&eng, &event)?;
        let bytes = layout.encode(None, Some(&parse(Some(&payload))?))?;
        eng.world()?.send_bytes(id, &bytes).map_err(err)
    })()
}
