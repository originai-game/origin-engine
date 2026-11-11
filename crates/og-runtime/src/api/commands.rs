//! Command registration and probe binding calls.

use super::*;

// ── commands ────────────────────────────────────────────────────────────────

pub fn og_commands_register(name: String, handler: u32, desc: Option<String>) -> String {
    if let Some(why) = rprobe::reserved(&name) {
        return refuse(why);
    }
    let valid = og_probe::command::parse(&format!("0:{name}"))
        .is_ok_and(|s| s.len() == 1 && s[0].action == name && s[0].args.is_empty());
    if !valid {
        return refuse(format!(
            "'{name}' is not an action name (snake_case, such as `spawn_wave`)"
        ));
    }
    let desc: wire::CommandDesc = match wire::arg(desc.as_deref(), "desc") {
        Ok(d) => d,
        Err(e) => return refuse(e),
    };
    let decl = CommandDecl {
        handler,
        usage: desc.usage,
        doc: desc.doc,
    };
    let taken = DECL.with(|d| {
        let mut d = d.borrow_mut();
        if d.commands.contains_key(&name) {
            return true;
        }
        d.commands.insert(name.clone(), decl.clone());
        false
    });
    if taken {
        return refuse(format!("'{name}' is already registered"));
    }
    // Engines already running take it too.
    for id in super::engine::engine_ids() {
        if let Ok(eng) = engine(id) {
            if eng.state.borrow().commands.contains_key(&name) {
                continue;
            }
            eng.state
                .borrow_mut()
                .commands
                .insert(name.clone(), decl.clone());
            if let Ok(mut probe) = eng.probe.try_borrow_mut() {
                let _ = rprobe::register_one(&mut probe, id, &name, &decl.usage, &decl.doc);
            }
        }
    }
    ok_void()
}

pub fn og_commands_parse(script: String) -> String {
    match og_probe::command::parse(&script) {
        Ok(steps) => ok(Value::Array(steps.iter().map(|s| s.to_json()).collect())),
        Err(e) => refuse(e.to_string()),
    }
}

pub fn og_commands_schedule(world: u32, script: String) -> String {
    result((|| {
        let eng = engine(world)?;
        with_probe(&eng, false, |p, w| p.schedule(w, &script))?
            .map(|r| r.to_json())
            .map_err(|e| e.to_string())
    })())
}

pub fn og_commands_help(world: u32) -> String {
    let list = engine(world)
        .ok()
        .and_then(|eng| {
            eng.probe
                .try_borrow()
                .ok()
                .map(|p| p.help().iter().map(|c| c.to_json()).collect::<Vec<_>>())
        })
        .unwrap_or_else(|| {
            og_probe::command::CORE_ACTIONS
                .iter()
                .map(|(n, u, d)| json!({ "name": n, "usage": u, "doc": d, "core": true }))
                .collect()
        });
    Value::Array(list).to_string()
}

// ── probe ───────────────────────────────────────────────────────────────────

/// One probe member by name through og-probe's JSON entry, so the typed
/// binding symbols answer exactly what the WebSocket and the request ring do.
fn member(probe: u32, method: &str, params: Value) -> Value {
    let frame =
        json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }).to_string();
    let answer = super::engine::call_rpc(probe, &frame);
    let v: Value = serde_json::from_str(&answer).unwrap_or(Value::Null);
    v.get("result").cloned().unwrap_or(Value::Null)
}

pub fn og_runtime_probe_get_version(_probe: u32) -> u32 {
    og_probe::PROBE_VERSION
}

pub fn og_runtime_probe_tick(probe: u32) -> u32 {
    og_world_get_tick(probe)
}

pub fn og_runtime_probe_state(probe: u32) -> String {
    member(probe, "state", json!({})).to_string()
}

pub fn og_runtime_probe_advance(probe: u32, steps: u32) -> u32 {
    let v = member(
        probe,
        "advance",
        json!({ "steps": steps.min(og_probe::PLAYTEST_MAX_TICKS as u32) }),
    );
    v.as_u64()
        .map_or_else(|| og_world_get_tick(probe), |t| t as u32)
}

pub fn og_runtime_probe_pause(probe: u32) -> bool {
    member(probe, "pause", json!({})).as_bool().unwrap_or(false)
}

pub fn og_runtime_probe_resume(probe: u32) -> bool {
    member(probe, "resume", json!({}))
        .as_bool()
        .unwrap_or(false)
}

pub fn og_runtime_probe_input(probe: u32, event: String) -> bool {
    let Ok(ev) = parse(Some(&event)) else {
        return false;
    };
    member(probe, "input", json!({ "event": ev }))
        .as_bool()
        .unwrap_or(false)
}

pub fn og_runtime_probe_accepted(probe: u32) -> u32 {
    member(probe, "accepted", json!({})).as_u64().unwrap_or(0) as u32
}

pub fn og_runtime_probe_pick(probe: u32, x: f64, y: f64) -> String {
    member(probe, "pick", json!({ "x": x, "y": y })).to_string()
}

pub fn og_engine_probe_pick_ui(probe: u32, x: f64, y: f64) -> String {
    member(probe, "pick_ui", json!({ "x": x, "y": y })).to_string()
}

pub fn og_runtime_probe_entity(probe: u32, id: String) -> String {
    member(probe, "entity", json!({ "id": id })).to_string()
}

pub fn og_runtime_probe_bounds(probe: u32, id: String) -> String {
    member(probe, "bounds", json!({ "id": id })).to_string()
}

pub fn og_runtime_probe_patch(probe: u32, id: String, parameter: String, value: String) -> bool {
    let Ok(value) = parse(Some(&value)) else {
        return false;
    };
    member(
        probe,
        "patch",
        json!({ "id": id, "parameter": parameter, "value": value }),
    )
    .as_bool()
    .unwrap_or(false)
}

pub fn og_runtime_probe_run(probe: u32, program: String) -> String {
    let program = parse(Some(&program)).unwrap_or(json!([]));
    member(probe, "run", json!({ "program": program })).to_string()
}

pub fn og_engine_probe_hello(probe: u32) -> String {
    member(probe, "hello", json!({})).to_string()
}

pub fn og_engine_probe_capture(
    probe: u32,
    width: Option<u32>,
    height: Option<u32>,
    fit: Option<u32>,
    region: Option<String>,
) -> String {
    let mut params = json!({ "width": width.unwrap_or(0), "height": height.unwrap_or(0) });
    if let Some(fit) = fit {
        params["fit"] = json!(fit);
    }
    if let Some(region) = parse(region.as_deref()).ok().filter(|r| !r.is_null()) {
        params["region"] = region;
    }
    member(probe, "capture", params).to_string()
}

pub fn og_engine_probe_marks(probe: u32, desc: Option<String>) -> String {
    member(probe, "marks", parse(desc.as_deref()).unwrap_or(json!({}))).to_string()
}

pub fn og_engine_probe_timeline(probe: u32) -> String {
    member(probe, "timeline", json!({})).to_string()
}

pub fn og_engine_probe_sweep(probe: u32, desc: Option<String>) -> String {
    member(probe, "sweep", parse(desc.as_deref()).unwrap_or(json!({}))).to_string()
}

pub fn og_engine_probe_reload(probe: u32) -> String {
    member(probe, "reload", json!({})).to_string()
}

pub fn og_engine_probe_snapshot(probe: u32) -> String {
    member(probe, "snapshot", json!({})).to_string()
}

pub fn og_engine_probe_restore(probe: u32, id: String) -> bool {
    member(probe, "restore", json!({ "id": id }))
        .as_bool()
        .unwrap_or(false)
}

pub fn og_engine_probe_dump(probe: u32, desc: Option<String>) -> String {
    let mut d = parse(desc.as_deref()).unwrap_or(json!({}));
    if d["what"].is_null() {
        d["what"] = json!("world");
    }
    member(probe, "dump", d).to_string()
}

pub fn og_engine_probe_commands(probe: u32, script: String) -> String {
    member(probe, "commands", json!({ "script": script })).to_string()
}

pub fn og_engine_probe_listen(probe: u32, seconds: Option<f64>) -> String {
    member(
        probe,
        "listen",
        json!({ "seconds": seconds.unwrap_or(1.0) }),
    )
    .to_string()
}

pub fn og_engine_probe_memory(probe: u32) -> String {
    member(probe, "memory", json!({})).to_string()
}

pub fn og_engine_probe_describe(probe: u32) -> String {
    member(probe, "describe", json!({})).to_string()
}

/// A malformed expectation answers null (the call's error names why).
pub fn og_engine_probe_expect(probe: u32, what: String) -> String {
    member(probe, "expect", json!({ "what": what })).to_string()
}

pub fn og_engine_probe_expectations(probe: u32) -> String {
    member(probe, "expectations", json!({})).to_string()
}

pub fn og_engine_probe_assertions(probe: u32, take: Option<bool>) -> String {
    member(
        probe,
        "assertions",
        json!({ "take": take.unwrap_or(false) }),
    )
    .to_string()
}

pub fn og_engine_probe_observe(probe: u32, records: Option<String>, types: Option<bool>) -> String {
    let records = wire::arg::<Option<Vec<String>>>(records.as_deref(), "records")
        .ok()
        .flatten()
        .filter(|a| !a.is_empty());
    let mut out = member(
        probe,
        "observe",
        json!({ "records": records, "types": types.unwrap_or(false) }),
    );
    // Each record's kind, from the layouts the game declared.
    if let (Some(types), Ok(eng)) = (out["types"].as_object_mut(), crate::engine::engine(probe)) {
        let st = eng.state.borrow();
        for (name, t) in types.iter_mut() {
            if let Some(l) = st.layouts.get(name) {
                t["kind"] = json!(match l.kind {
                    crate::codec::RecordKind::Component => "component",
                    crate::codec::RecordKind::Resource => "resource",
                    crate::codec::RecordKind::Event => "event",
                });
            }
        }
    }
    out.to_string()
}

pub fn og_engine_probe_assertion_events(probe: u32) -> String {
    member(probe, "assertion_events", json!({})).to_string()
}

pub fn og_engine_probe_save_scene(
    probe: u32,
    write: Option<bool>,
    spawned: Option<bool>,
) -> String {
    member(
        probe,
        "save_scene",
        json!({ "write": write.unwrap_or(false), "spawned": spawned.unwrap_or(false) }),
    )
    .to_string()
}
