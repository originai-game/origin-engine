//! og.net: inputs, commands, the session's shape, its status and hashes.

use super::*;

pub fn og_net_submit(world: u32, input: Option<String>) -> Result<(), String> {
    let d = parse(input.as_deref())?;
    let i: crate::host::wire::InputDesc = crate::host::wire::value(&d, "input")?;
    // Only the fields given are written; the rest keep og.PlayerInput's defaults.
    let mut fields = serde_json::Map::new();
    for (k, v) in [
        ("buttons", json!(i.buttons)),
        ("x", json!(i.x)),
        ("y", json!(i.y)),
        ("aimX", json!(i.aim_x)),
        ("aimY", json!(i.aim_y)),
    ] {
        if d.get(k).is_none_or(Value::is_null) {
            continue;
        }
        if !v.as_f64().is_some_and(f64::is_finite) {
            return Err(format!("{k} must be a finite number, got {}", d[k]));
        }
        fields.insert(k.into(), v);
    }
    if let Some(k) = d.as_object().and_then(|o| {
        o.keys()
            .find(|k| !matches!(k.as_str(), "buttons" | "x" | "y" | "aimX" | "aimY"))
    }) {
        return Err(format!(
            "og.net.submit takes buttons, x, y, aimX and aimY; not '{k}'"
        ));
    }
    if let Some(input) = with(world, |n| {
        (!n.shape.input.is_empty()).then(|| n.shape.input.clone())
    }) {
        return Err(format!(
            "og.net.submit gives og.PlayerInput; og.net.shape named '{input}': og.net.submitInput(world, {input}, {{ ... }})"
        ));
    }
    let layout = match engine_of(world)
        .ok()
        .and_then(|e| e.state.borrow().layouts.get(PLAYER_INPUT).cloned())
    {
        Some(l) => l,
        None => return Err("og.PlayerInput is not registered in this world".into()),
    };
    let bytes = layout.encode(None, Some(&Value::Object(fields)))?;
    with(world, |n| n.ls.set_local(bytes));
    Ok(())
}

/// The zero value of the input record `name` (its defaults), packed.
pub(super) fn state_zero(engine: u32, name: &str) -> Vec<u8> {
    engine_of(engine)
        .ok()
        .and_then(|e| e.state.borrow().layouts.get(name).cloned())
        .and_then(|l| l.encode(None, None).ok())
        .unwrap_or_default()
}

/// A token's name: `{ kind, name }` (a token as JSON) or the name itself.
pub(super) fn token_name(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Object(o) => o.get("name").and_then(Value::as_str).map(str::to_string),
        _ => None,
    }
}

/// A game's own component that og.net.shape may name: registered, not the
/// engine's, with a u32 `player` field.
pub(super) fn shape_layout(
    eng: &Engine,
    name: &str,
    what: &str,
) -> Result<crate::host::codec::Layout, String> {
    let layout = eng
        .state
        .borrow()
        .layouts
        .get(name)
        .cloned()
        .filter(|l| l.kind == crate::host::codec::RecordKind::Component)
        .ok_or_else(|| format!("og.net.shape's {what}: '{name}' is not a registered component"))?;
    if name.starts_with("og.") {
        return Err(format!(
            "og.net.shape's {what}: '{name}' is the engine's; name a component of the game's own (defineComponent)"
        ));
    }
    match layout.field("player") {
        Some((_, f)) if f.kind == crate::host::codec::Kind::U32 => Ok(layout),
        _ => Err(format!(
            "og.net.shape's {what}: '{name}' needs a `player: u32()` field (the seat it carries)"
        )),
    }
}

/// FNV-1a over the shape's names and layouts: equal on two peers exactly
/// when their inputs are read the same way.
pub(super) fn fingerprint(parts: &[&crate::host::codec::Layout], max: u32) -> u64 {
    let mut h = og_core::hash::StableHasher::new();
    let mut eat = |b: &[u8]| h.write(b);
    for l in parts {
        eat(l.schema.as_bytes());
        for f in l.fields.iter() {
            eat(f.name.as_bytes());
            eat(format!("{:?}", f.ty).as_bytes());
        }
        eat(b";");
    }
    eat(&max.to_le_bytes());
    h.finish().max(1)
}

pub fn og_net_shape(world: u32, desc: Option<String>) -> Result<(), String> {
    let d: crate::host::wire::NetShapeDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
    let eng = engine_of(world)?;
    let w = match eng.world() {
        Ok(w) => w,
        Err(_) => {
            return Err(
                "og.net.shape is called outside a system (setup or start), before the room starts"
                    .into(),
            );
        }
    };
    if with(world, |n| n.ls.running()) {
        return Err("og.net.shape: lockstep runs; set the shape before the room starts".into());
    }
    let name = |k: &str, v: &Option<Value>| -> Result<Option<String>, String> {
        match v {
            None => Ok(None),
            Some(v) => token_name(v)
                .map(Some)
                .ok_or_else(|| format!("og.net.shape's {k} is a component token, got {v}")),
        }
    };
    let (input, command) = match (name("input", &d.input), name("command", &d.command)) {
        (Ok(i), Ok(c)) => (i.unwrap_or_default(), c.unwrap_or_default()),
        (Err(e), _) | (_, Err(e)) => return Err(e),
    };
    let input = if input == PLAYER_INPUT {
        String::new()
    } else {
        input
    };
    let max_commands = d.max_commands.clamp(1, 256);
    let input_layout = if input.is_empty() {
        eng.state.borrow().layouts.get(PLAYER_INPUT).cloned()
    } else {
        {
            let l = shape_layout(&eng, &input, "input")?;
            Some(l)
        }
    };
    let Some(input_layout) = input_layout else {
        return Err("og.PlayerInput is not registered in this world".into());
    };
    let command_layout = if command.is_empty() {
        None
    } else {
        {
            let l = shape_layout(&eng, &command, "command")?;
            Some(l)
        }
    };
    if !input.is_empty()
        && let Err(e) = w.grant(NET_SYSTEM, &[input.as_str()])
    {
        return Err(e.reason().into());
    }
    let shape = Shape {
        fingerprint: if input.is_empty() && command.is_empty() {
            0
        } else {
            let mut parts = vec![&input_layout];
            parts.extend(command_layout.as_ref());
            fingerprint(&parts, max_commands)
        },
        state_bytes: if command.is_empty() {
            0
        } else {
            input_layout.size()
        },
        command_bytes: command_layout.as_ref().map_or(0, |l| l.size()),
        input,
        command,
        max_commands,
    };
    with(world, |n| {
        if n.shape.input != shape.input {
            n.ls.set_local(Vec::new());
        }
        n.shape = shape;
        n.given.clear();
    });
    Ok(())
}

pub fn og_net_submit_input(world: u32, component: String, value: String) -> Result<(), String> {
    let (shape, eng) = {
        let e = engine_of(world)?;
        (with(world, |n| n.shape.clone()), e)
    };
    if shape.input.is_empty() {
        return Err("og.net.submitInput needs og.net.shape's input; og.PlayerInput's is given with og.net.submit".into());
    }
    let name =
        token_name(&serde_json::from_str(&component).unwrap_or(Value::String(component.clone())))
            .unwrap_or(component);
    if name != shape.input {
        return Err(format!(
            "og.net.submitInput takes '{}', og.net.shape's input; not '{name}'",
            shape.input
        ));
    }
    let v = parse(Some(&value))?;
    if v.get("player").is_some() || v.get("tick").is_some() {
        return Err("og.net.submitInput: `player` and `tick` are the engine's".into());
    }
    let Some(layout) = eng.state.borrow().layouts.get(&name).cloned() else {
        return Err(format!("'{name}' is not registered in this world"));
    };
    match layout.encode(None, Some(&v)) {
        Ok(bytes) => {
            with(world, |n| n.ls.set_local(bytes));
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub fn og_net_command(world: u32, component: String, value: String) -> Result<(), String> {
    let eng = engine_of(world)?;
    let shape = with(world, |n| n.shape.clone());
    let name =
        token_name(&serde_json::from_str(&component).unwrap_or(Value::String(component.clone())))
            .unwrap_or(component);
    if shape.command.is_empty() {
        return Err(
            "og.net.command needs a command component: og.net.shape(world, { command: MyCommand })"
                .into(),
        );
    }
    if name != shape.command {
        return Err(format!(
            "og.net.command takes '{}', og.net.shape's command; not '{name}'",
            shape.command
        ));
    }
    let v = parse(Some(&value))?;
    if v.get("player").is_some() {
        return Err("og.net.command: `player` is the engine's (the seat that gave it)".into());
    }
    let Some(layout) = eng.state.borrow().layouts.get(&name).cloned() else {
        return Err(format!("'{name}' is not registered in this world"));
    };
    let bytes = layout.encode(None, Some(&v))?;
    let tick = tick_of(world);
    with(world, |n| {
        // A re-simulated tick's commands went out on its first run.
        if n.ls.resimulating(tick) {
            return Ok(());
        }
        if n.given.len() >= shape.max_commands as usize {
            return Err(format!(
                "og.net.command: {} commands this tick already (og.net.shape's maxCommands)",
                shape.max_commands
            ));
        }
        n.given.push(bytes);
        Ok(())
    })
}

pub fn og_net_commands(world: u32, component: String) -> Result<serde_json::Value, String> {
    let (shape, eng) = {
        let e = engine_of(world)?;
        (with(world, |n| n.shape.clone()), e)
    };
    let name =
        token_name(&serde_json::from_str(&component).unwrap_or(Value::String(component.clone())))
            .unwrap_or(component);
    if shape.command.is_empty() {
        return Ok(json!([]));
    }
    if name != shape.command {
        return Err(format!(
            "og.net.commands reads '{}', og.net.shape's command; not '{name}'",
            shape.command
        ));
    }
    let Some(layout) = eng.state.borrow().layouts.get(&name).cloned() else {
        return Err(format!("'{name}' is not registered in this world"));
    };
    let tick = tick_of(world);
    let size = shape.command_bytes.max(1);
    let by_seat: Vec<(u32, Vec<Vec<u8>>)> = with(world, |n| {
        if !n.ls.running() {
            return if n.local_commands.0 == tick {
                vec![(0, n.local_commands.1.clone())]
            } else {
                Vec::new()
            };
        }
        (0..n.ls.players().unwrap_or(0))
            .map(|seat| {
                let cmds =
                    n.ls.input(tick, seat)
                        .and_then(|b| b.get(shape.state_bytes..))
                        .unwrap_or_default();
                let list = cmds
                    .chunks_exact(size)
                    .take(shape.max_commands as usize)
                    .map(<[u8]>::to_vec)
                    .collect();
                (seat, list)
            })
            .collect()
    });
    let mut out = Vec::new();
    for (seat, list) in by_seat {
        for c in list {
            {
                let b = layout.encode(Some(&c), Some(&json!({ "player": seat })))?;
                out.push(layout.decode(&b))
            }
        }
    }
    Ok(Value::Array(out))
}

pub fn og_net_status(world: u32) -> LockstepStatus {
    with(world, |n| {
        let s = n.ls.status();
        // og-net's status, read field by field into the IDL's type: an
        // absent or `null` value is the field's default.
        let seats = s["seats"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|h| LockstepSeat {
                        seat: read(&h[0]),
                        peer: read(&h[1]),
                        since: read(&h[2]),
                        until: read(&h[3]),
                        left: read(&h[4]),
                    })
                    .collect()
            })
            .unwrap_or_default();
        LockstepStatus {
            running: read(&s["running"]),
            peer: read(&s["peer"]),
            peers: read(&s["peers"]),
            start_tick: read(&s["startTick"]),
            players: read(&s["players"]),
            seats,
            spectators: read(&s["spectators"]),
            waiting_for: read(&s["waitingFor"]),
            waits: read(&s["waits"]),
            desync: read(&s["desync"]["tick"]),
            rollback: read(&s["rollback"]),
            max_prediction_ticks: read(&s["maxPredictionTicks"]),
            confirmed_tick: read(&s["confirmedTick"]),
            final_tick: read(&s["finalTick"]),
            acknowledged: read(&s["acknowledged"]),
            predicted_ticks: read(&s["predictedTicks"]),
            rollbacks: read(&s["rollbacks"]),
            resimulated_ticks: read(&s["resimulatedTicks"]),
            snapshot_bytes: read(&s["snapshotBytes"]),
            stalled: read(&s["stalled"]),
            drops: read(&s["drops"]),
        }
    })
}

/// `v` read as `T`; `T`'s default when it is absent, `null` or of
/// another type.
pub(super) fn read<T: wire::FromJson + Default>(v: &Value) -> T {
    wire::value(v, "").unwrap_or_default()
}

pub fn og_net_resimulating(world: u32) -> bool {
    with(world, |n| n.ls.resimulating(tick_of(world)))
}

pub fn og_net_final_hash(world: u32, tick: f64) -> Option<String> {
    with(world, |n| {
        let h = (tick >= 0.0 && tick.fract() == 0.0)
            .then(|| n.ls.final_hash(tick as u64))
            .flatten();
        h.map(|h| format!("{h:016x}"))
    })
}

/// The world's snapshot bytes (og-ecs's format), mid-tick or between
/// ticks: what og.net hashes, and `og replay` (og-host-node) hashes and diffs.
pub fn world_snapshot_bytes(world: u32) -> Result<Vec<u8>, String> {
    if let Some(f) = running(world) {
        // SAFETY: the frame is published while its system runs.
        return Ok(unsafe { f.ctx() }.snapshot());
    }
    engine_of(world).and_then(|e| e.world().map(|w| w.snapshot()))
}

/// The hash as text, or `error: <reason>`; the host writes it as a JSON
/// string.
pub fn og_net_hash(world: u32) -> String {
    match world_snapshot_bytes(world).and_then(|bytes| shared_hash(&bytes)) {
        Ok(h) => format!("{h:016x}"),
        Err(e) => format!("error: {e}"),
    }
}
