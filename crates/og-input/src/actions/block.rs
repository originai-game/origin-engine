use super::*;

// ── the state block ─────────────────────────────────────────────────────

fn rt_json(rt: &Runtime) -> Value {
    let s = rt.state;
    json!({
        "down": s.down, "pressed": s.pressed, "released": s.released, "held": s.held,
        "value": s.value, "vx": s.vector.x, "vy": s.vector.y, "device": s.device.index(),
        "rawDown": rt.raw_down, "since": rt.since, "toggled": rt.toggled, "suppressed": rt.suppressed,
    })
}

fn rt_of(v: &Value) -> Result<Runtime, String> {
    Ok(Runtime {
        state: ActionState {
            down: bool_of(v, "down", false)?,
            pressed: bool_of(v, "pressed", false)?,
            released: bool_of(v, "released", false)?,
            held: bool_of(v, "held", false)?,
            value: f32_of(v, "value", 0.0)?,
            vector: Vec2::new(f32_of(v, "vx", 0.0)?, f32_of(v, "vy", 0.0)?),
            device: Device::from_index(v["device"].as_u64().unwrap_or(0) as u32)
                .unwrap_or_default(),
        },
        raw_down: bool_of(v, "rawDown", false)?,
        since: v["since"].as_u64().unwrap_or(0),
        toggled: bool_of(v, "toggled", false)?,
        suppressed: bool_of(v, "suppressed", false)?,
    })
}

fn list_json(list: &[Binding]) -> Value {
    Value::Array(list.iter().map(Binding::to_json).collect())
}

fn list_of(v: &Value) -> Result<Vec<Binding>, String> {
    v.as_array()
        .ok_or("a list of bindings")?
        .iter()
        .map(Binding::from_json)
        .collect()
}

impl ActionMap {
    /// The map as the state block saves it: the actions with the player's
    /// bindings and each one's state, the contexts, dead zones, the device
    /// used last and a capture. Equal maps, equal bytes.
    pub fn save_state(&self) -> Vec<u8> {
        let v = json!({
            "actions": self.actions.iter().map(|a| json!({
                "desc": a.desc.to_json(),
                "current": a.current.as_deref().map(list_json),
                "toggle": a.toggle,
                "rt": rt_json(&a.rt),
            })).collect::<Vec<_>>(),
            "pending": self.pending.iter().map(|(k, v)| (k.clone(), list_json(v))).collect::<Map<_, _>>(),
            "pendingToggles": self.pending_toggles.iter().map(|(k, v)| (k.clone(), json!(v))).collect::<Map<_, _>>(),
            "actionMaps": self.contexts,
            "deadzones": self.deadzones.iter().map(|d| json!([d.stick, d.trigger])).collect::<Vec<_>>(),
            "device": self.device.index(),
            "tick": self.tick,
            "deflected": self.deflected,
            "capture": self.capture.as_ref().map(|c| json!({ "desc": c.desc.to_json(), "start": c.start, "disarmed": c.disarmed })),
            "captured": self.captured.to_json(),
        });
        let mut v = v;
        // Only when set: a world whose player set none saves the bytes it
        // always did (and hashes the same).
        if !self.senses.is_empty() {
            v["senses"] = self
                .senses
                .iter()
                .map(|(k, s)| (k.clone(), senses_json(s)))
                .collect::<Map<_, _>>()
                .into();
        }
        v.to_string().into_bytes()
    }

    /// A map from [`ActionMap::save_state`]'s bytes (the session's
    /// surface and queued rumbles are not saved: `self`'s stay).
    pub fn restore_state(&self, bytes: &[u8]) -> Result<ActionMap, String> {
        let v: Value =
            serde_json::from_slice(bytes).map_err(|e| format!("og.input.actions: {e}"))?;
        let mut out = ActionMap {
            rumble: self.rumble.clone(),
            surface: self.surface,
            zones: self.zones.clone(),
            ..ActionMap::default()
        };
        for a in v["actions"].as_array().into_iter().flatten() {
            out.actions.push(Action {
                desc: ActionDesc::from_json(&a["desc"])?,
                current: match &a["current"] {
                    Value::Null => None,
                    c => Some(list_of(c)?),
                },
                toggle: a["toggle"].as_bool(),
                rt: rt_of(&a["rt"])?,
            });
        }
        for (k, l) in v["pending"].as_object().into_iter().flatten() {
            out.pending.insert(k.clone(), list_of(l)?);
        }
        for (k, t) in v["pendingToggles"].as_object().into_iter().flatten() {
            out.pending_toggles
                .insert(k.clone(), t.as_bool().unwrap_or(false));
        }
        out.contexts = v["actionMaps"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|c| c.as_str().map(str::to_string))
            .collect();
        for (p, d) in v["deadzones"]
            .as_array()
            .into_iter()
            .flatten()
            .enumerate()
            .take(MAX_GAMEPADS)
        {
            out.deadzones[p] = PadDeadzone {
                stick: d[0].as_f64().unwrap_or(0.15) as f32,
                trigger: d[1].as_f64().unwrap_or(0.05) as f32,
            };
        }
        out.device =
            Device::from_index(v["device"].as_u64().unwrap_or(0) as u32).unwrap_or_default();
        out.tick = v["tick"].as_u64().unwrap_or(0);
        out.deflected = v["deflected"].as_u64().unwrap_or(0);
        if let Some(c) = v["capture"].as_object() {
            out.capture = Some(Capture {
                desc: CaptureDesc::from_json(&c["desc"])?,
                start: c["start"].as_u64().unwrap_or(0),
                disarmed: c["disarmed"].as_u64().unwrap_or(0),
            });
        }
        out.captured = CaptureState::from_json(&v["captured"])?;
        for (k, l) in v["senses"].as_object().into_iter().flatten() {
            out.senses.insert(k.clone(), senses_of(l)?);
        }
        Ok(out)
    }
}

/// The map as world state: a state block over a [`SharedActions`].
pub struct ActionsBlock(pub SharedActions);

impl og_ecs::StateBlock for ActionsBlock {
    fn save(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&lock(&self.0).save_state());
    }

    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn std::any::Any + Send>> {
        let map = lock(&self.0)
            .restore_state(bytes)
            .map_err(|e| og_core::Error::new(og_core::ErrorKind::Parse, e))?;
        Ok(Box::new(map))
    }

    fn apply(&mut self, decoded: Option<Box<dyn std::any::Any + Send>>) {
        let mut map = lock(&self.0);
        let next = match decoded.and_then(|d| d.downcast::<ActionMap>().ok()) {
            Some(m) => *m,
            None => ActionMap {
                rumble: map.rumble.clone(),
                surface: map.surface,
                zones: map.zones.clone(),
                ..ActionMap::default()
            },
        };
        *map = next;
    }

    /// Bindings and their state: nothing in the world.
    fn shift(&mut self, _: &og_ecs::Shift) {}
}
