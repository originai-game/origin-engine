//! Schema access, RuntimeProbe members and world dumps.

use super::*;

impl Probe {
    // ---- schema-driven reads and writes -----------------------------------

    /// The (component, schema, field index) a `Component.field` names.
    fn field_ref(world: &World, parameter: &str) -> Result<(SchemaId, Schema, usize), String> {
        let (component, field) = parameter
            .rsplit_once('.')
            .ok_or_else(|| format!("'{parameter}' is not `Component.field`"))?;
        let id = world
            .id_by_name(component)
            .map_err(|_| format!("no component '{component}' is registered"))?;
        let schema = world.schema(id).map_err(|e| e.reason().to_owned())?.clone();
        let index = schema
            .field_index(field)
            .ok_or_else(|| format!("'{component}' has no field '{field}'"))?;
        Ok((id, schema, index))
    }

    /// The field a `set:` target names (`Resource.field`,
    /// `<entity>.Component.field`), when it names one: a string field reads
    /// its value as text, not numbers.
    pub(super) fn set_target_field(world: &World, target: &str) -> Option<og_core::FieldSchema> {
        if let Some((id, field)) = Self::resource_target(world, target) {
            let schema = world.schema(id).ok()?;
            return schema.fields.get(schema.field_index(&field)?).cloned();
        }
        let parameter = match target.strip_prefix('#') {
            Some(rest) => rest.splitn(3, '.').nth(2)?,
            None => target.split_once('.')?.1,
        };
        let (_, schema, index) = Self::field_ref(world, parameter).ok()?;
        schema.fields.into_iter().nth(index)
    }

    /// Write one field of an entity's component through the schema. With
    /// `tunable_only`, a field without the `tunable` flag is refused.
    pub(super) fn write_field(
        &self,
        world: &mut World,
        e: Entity,
        parameter: &str,
        value: &Value,
        tunable_only: bool,
    ) -> Result<(), String> {
        let (id, schema, index) = Self::field_ref(world, parameter)?;
        let field = &schema.fields[index];
        if tunable_only && !field.tunable {
            return Err(format!("'{parameter}' is not tunable"));
        }
        let mut bytes = world.get_bytes(e, id).map_err(|e| e.reason().to_owned())?;
        let packed = value::json_field(field, value).map_err(|r| format!("{parameter}: {r}"))?;
        bytes[Layout::of(&schema).range(index)].copy_from_slice(&packed);
        world
            .insert_bytes(e, id, &bytes)
            .map_err(|e| e.reason().to_owned())
    }

    /// A Tune write, as the command channel's `set` does it: `target` is
    /// `<entity>.<Component>.<field>` or `<Resource>.<field>` and `text`
    /// its value as a command writes it (`1,2.5,0`, `true`, a variant
    /// name). The field is written through its schema and remembered as
    /// tuned, so `save_scene` writes it back.
    pub fn tune(&mut self, world: &mut World, target: &str, text: &str) -> Result<(), String> {
        let usage = "set:<entity>.<component>.<field>=<value> or set:<Resource>.<field>=<value>";
        // `<Resource>.<field>` names the resource, as a game's write would:
        // a lazy one is registered first ([`super::RegisterFn`]).
        if let (Some(register), Some((name, _))) = (&self.host.register, target.rsplit_once('.'))
            && world.id_by_name(name).is_err()
        {
            register(world, name);
        }
        let value = match Self::set_target_field(world, target) {
            Some(field) => value::command_value(&field, text)?,
            None => value::text_json(text)?,
        };
        if let Some((resource, field)) = Self::resource_target(world, target) {
            Self::write_resource_field(world, resource, &field, &value)?;
            self.tuned.insert(target.to_owned());
            return Ok(());
        }
        let (entity, parameter) = match target.strip_prefix('#') {
            Some(rest) => {
                let mut parts = rest.splitn(3, '.');
                match (parts.next(), parts.next(), parts.next()) {
                    (Some(i), Some(g), Some(p)) => (format!("#{i}.{g}"), p.to_owned()),
                    _ => return Err(format!("write `{usage}`")),
                }
            }
            None => {
                let (entity, parameter) = target
                    .split_once('.')
                    .ok_or_else(|| format!("write `{usage}`"))?;
                (entity.to_owned(), parameter.to_owned())
            }
        };
        let e = self.entity_of(world, &entity)?;
        self.write_field(world, e, &parameter, &value, false)?;
        self.tuned.insert(format!("{entity}.{parameter}"));
        Ok(())
    }

    /// `Resource.field` when the part before the last dot names a resource
    /// the world holds (resource names may themselves hold dots: `og.Camera`).
    pub(super) fn resource_target(world: &World, target: &str) -> Option<(SchemaId, String)> {
        let (name, field) = target.rsplit_once('.')?;
        let id = world.id_by_name(name).ok()?;
        world.has_resource_id(id).then(|| (id, field.to_owned()))
    }

    /// Write one field of a resource through its schema.
    pub(super) fn write_resource_field(
        world: &mut World,
        id: SchemaId,
        field: &str,
        value: &Value,
    ) -> Result<(), String> {
        let schema = world.schema(id).map_err(|e| e.reason().to_owned())?.clone();
        let index = schema
            .field_index(field)
            .ok_or_else(|| format!("'{}' has no field '{field}'", schema.name))?;
        let mut bytes = world
            .resource_bytes(id)
            .map_err(|e| e.reason().to_owned())?;
        let packed = value::json_field(&schema.fields[index], value)
            .map_err(|r| format!("{}.{field}: {r}", schema.name))?;
        bytes[Layout::of(&schema).range(index)].copy_from_slice(&packed);
        world
            .insert_resource_bytes(id, &bytes)
            .map_err(|e| e.reason().to_owned())
    }

    /// What `get` reads: a resource or one of its fields, an entity (its
    /// dump), one of its components or one field.
    pub(super) fn read_target(&self, world: &World, target: &str) -> Result<Value, String> {
        let resource = |name: &str| {
            world
                .id_by_name(name)
                .ok()
                .filter(|&id| world.has_resource_id(id))
        };
        let resource_json = |id: SchemaId| -> Result<(String, Map<String, Value>), String> {
            let schema = world.schema(id).map_err(|e| e.reason().to_owned())?;
            let bytes = world
                .resource_bytes(id)
                .map_err(|e| e.reason().to_owned())?;
            let mut fields = value::component_json(schema, &bytes);
            if schema.name == <og_core::Time as og_core::Reflect>::NAME {
                fields = time_json(world);
            }
            if let Some(derive) = &self.host.derive {
                derive(world, None, &schema.name, &mut fields);
            }
            Ok((schema.name.clone(), fields))
        };
        if let Some(id) = resource(target) {
            return Ok(Value::Object(resource_json(id)?.1));
        }
        if let Some((id, field)) = Self::resource_target(world, target) {
            let (name, mut fields) = resource_json(id)?;
            return fields
                .remove(&field)
                .ok_or_else(|| format!("'{name}' has no field '{field}'"));
        }
        // An entity: `#i.g` or a name, then `.Component[.field]`, where a
        // component's name may hold dots (`og.Transform`).
        let (entity, rest) = match target.strip_prefix('#') {
            Some(r) => {
                let mut parts = r.splitn(3, '.');
                match (parts.next(), parts.next(), parts.next()) {
                    (Some(i), Some(g), rest) => (format!("#{i}.{g}"), rest.unwrap_or("")),
                    _ => return Err(format!("no resource or live entity '{target}'")),
                }
            }
            None => match target.split_once('.') {
                Some((e, rest)) => (e.to_owned(), rest),
                None => (target.to_owned(), ""),
            },
        };
        let e = self.names.resolve(world, &entity).ok_or_else(|| {
            format!("no resource or live entity '{entity}' (a resource name, an entity name, or #<index>.<generation>)")
        })?;
        if rest.is_empty() {
            return Ok(self.dump_entity(world, e));
        }
        let read = |name: &str| -> Option<Map<String, Value>> {
            let id = world.id_by_name(name).ok()?;
            let schema = world.schema(id).ok()?;
            let bytes = world.get_bytes(e, id).ok()?;
            let mut fields = value::component_json(schema, &bytes);
            if let Some(derive) = &self.host.derive {
                derive(world, Some(e), &schema.name, &mut fields);
            }
            Some(fields)
        };
        if let Some(fields) = read(rest) {
            return Ok(Value::Object(fields));
        }
        let (component, field) = rest
            .rsplit_once('.')
            .ok_or_else(|| format!("'{entity}' has no component '{rest}'"))?;
        let mut fields =
            read(component).ok_or_else(|| format!("'{entity}' has no component '{component}'"))?;
        fields
            .remove(field)
            .ok_or_else(|| format!("'{component}' has no field '{field}'"))
    }

    /// Each component of an entity with its schema and packed value.
    fn components(world: &World, e: Entity) -> Vec<(Schema, Vec<u8>)> {
        world
            .components_of(e)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|id| {
                let schema = world.schema(id).ok()?.clone();
                let bytes = world.get_bytes(e, id).ok()?;
                Some((schema, bytes))
            })
            .collect()
    }

    // ---- RuntimeProbe protocol 5 ------------------------------------------

    /// `hello`: who is speaking.
    pub fn hello(&self, world: &World) -> Value {
        json!({
            "protocol": PROBE_VERSION,
            "wire": PROBE_WIRE_VERSION,
            "substrate": "origin-engine",
            "engine": self.host.engine,
            "ticksPerSecond": tick_rate(world),
            "tier": self.host.capture.as_ref().map_or("none", |c| c.tier()),
            "build": BUILD,
            "members": served_members(),
        })
    }

    /// `tick`: fixed steps since load.
    pub fn tick(&self, world: &World) -> u64 {
        world.time().tick
    }

    /// `state`: the fixed head, then the game's facts.
    pub fn state(&self, world: &World) -> Value {
        let seed = world.resource::<Rng>().map_or(0, |r| r.seed());
        let entities: Vec<Value> = self
            .names
            .live(world)
            .into_iter()
            .map(|(id, _)| json!({ "id": id }))
            .collect();
        json!({
            "tick": world.time().tick,
            "paused": self.clock.paused,
            "seed": seed,
            "timings": {
                "step": self.stats.step_ms,
                "render": self.stats.render_ms,
                "frameMs": self.stats.step_ms + self.stats.render_ms,
                "systems": self.stats.systems_ms.iter().map(|(name, ms)| json!({"name": name, "ms": ms})).collect::<Vec<_>>(),
                "triangles": self.stats.triangles,
                "passes": self.stats.passes_ms.iter().map(|(name, ms)| json!({"name": name, "ms": ms})).collect::<Vec<_>>(),
                "gpuPasses": self.stats.gpu_passes_ms.iter().map(|(name, ms)| json!({"name": name, "ms": ms})).collect::<Vec<_>>(),
            },
            "entities": entities,
            "facts": self.host.facts.as_ref().map_or(Value::Null, |f| f(world)),
        })
    }

    /// `advance`: take whole fixed steps regardless of the wall clock,
    /// running the command steps that come due on the way. `Err` is a system
    /// that failed; the world stops at that tick. A screenshot waiting for
    /// its frame (which cannot land during the call) holds the steps, not
    /// the ticks: they run late, once it has landed.
    pub fn advance(&mut self, world: &mut World, steps: u64) -> og_core::Result<u64> {
        for _ in 0..steps {
            if self.schedule.shot.is_none() {
                self.pump(world);
            }
            if self.clock.quit || self.clock.held(world) {
                break;
            }
            self.tick_once(world)?;
        }
        Ok(world.time().tick)
    }

    /// `pause`: hold the clock.
    pub fn pause(&mut self) -> bool {
        self.clock.paused = true;
        true
    }

    /// `resume`: release the clock.
    pub fn resume(&mut self) -> bool {
        self.clock.paused = false;
        self.clock.script_paused = false;
        true
    }

    /// `input`: one event through the input layer's own path.
    pub fn input(&mut self, world: &mut World, event: &InjectedInput) -> bool {
        self.input_at(world, event, 0.0)
    }

    /// `input` with the event's time within the tick that reads it (IDL
    /// `KeyInput.at`, `PointerInput.at`: -1 to 1 ticks from its start), from
    /// the host's event timestamp.
    pub fn input_at(&mut self, world: &mut World, event: &InjectedInput, at: f32) -> bool {
        self.inject.stamp = at;
        let accepted = self.inject.deliver(world, &mut *self.host.input, event);
        self.inject.stamp = 0.0;
        self.timeline
            .input(world.time().tick, &event.to_json(), accepted);
        accepted
    }

    /// `accepted`: events the input layer accepted since load.
    pub fn accepted(&self) -> u32 {
        self.host.input.accepted()
    }

    /// `pick_ui`: the UI element under a point (CSS px of the play
    /// surface): the topmost one that shows something or takes input, with
    /// its rectangle, the document line that built it and the style rules
    /// that dress it ([`crate::hooks::pick_ui`]); `None` when there is none
    /// or no UI.
    pub fn pick_ui(&self, world: &World, x: f32, y: f32) -> Option<Value> {
        let elements = self.host.ui.as_ref()?.elements(world);
        let e = crate::hooks::pick_ui(&elements, x, y)?;
        let d = &e.detail;
        let field = |k: &str| d.get(k).cloned().unwrap_or(Value::Null);
        let rules = d.get("rules").cloned().unwrap_or_else(|| json!([]));
        Some(json!({
            "id": e.id,
            "path": e.path,
            "kind": e.kind,
            "text": e.text,
            "rect": { "x": e.x, "y": e.y, "width": e.width, "height": e.height },
            "layer": field("layer"),
            "source": field("source"),
            "part": field("part"),
            "rule": rules.as_array().and_then(|r| r.last()).cloned().unwrap_or(Value::Null),
            "rules": rules,
        }))
    }

    /// `pick`: the scene entity under a point, with the scene line that
    /// declares it as its `source` and, outside a publish build, the UI
    /// element over the point as `ui` ([`Probe::pick_ui`]); `None` without
    /// a renderer that picks, or nothing there.
    pub fn pick(&mut self, world: &World, x: f32, y: f32) -> Option<Value> {
        let hit = match self.host.capture.as_mut().and_then(|c| c.pick(world, x, y)) {
            Some(hit) => hit,
            None => self.host.scene_view.as_mut()?.pick(world, x, y)?,
        };
        let source = hit.source.or_else(|| {
            self.host
                .scene_writer
                .as_ref()
                .and_then(|w| w.source(world, hit.entity))
        });
        let mut out = json!({
            "entity": self.names.id_of(world, hit.entity),
            "world": hit.world,
            "source": source,
        });
        if !cfg!(feature = "publish")
            && let Some(ui) = self.pick_ui(world, x, y)
        {
            out["ui"] = ui;
        }
        Some(out)
    }

    /// `entity`: the entity's tunable fields as `Component.field` keys.
    pub fn entity(&self, world: &World, id: &str) -> Option<Map<String, Value>> {
        let e = self.names.resolve(world, id)?;
        let mut out = Map::new();
        for (schema, bytes) in Self::components(world, e) {
            let layout = Layout::of(&schema);
            for (i, f) in schema.fields.iter().enumerate().filter(|(_, f)| f.tunable) {
                out.insert(
                    format!("{}.{}", schema.name, f.name),
                    value::field_json(f.ty, &bytes[layout.range(i)]),
                );
            }
        }
        Some(out)
    }

    /// `bounds`: the ranges of the entity's tunable fields, from the schema.
    pub fn bounds(&self, world: &World, id: &str) -> Option<Map<String, Value>> {
        let e = self.names.resolve(world, id)?;
        let mut out = Map::new();
        for (schema, _) in Self::components(world, e) {
            for f in schema.fields.iter().filter(|f| f.tunable) {
                let path = format!("{}.{}", schema.name, f.name);
                let mut tunable = Map::new();
                tunable.insert("path".into(), Value::String(path.clone()));
                if let Some(label) = &f.label {
                    tunable.insert("label".into(), Value::String(label.clone()));
                }
                for (key, bound) in [("min", f.min), ("max", f.max)] {
                    if let Some(n) = bound.and_then(serde_json::Number::from_f64) {
                        tunable.insert(key.into(), Value::Number(n));
                    }
                }
                out.insert(path, Value::Object(tunable));
            }
        }
        Some(out)
    }

    /// `patch`: move one tunable field for this run only. `false` for an
    /// unknown entity or field, a field that is not tunable, or a value of
    /// another kind or outside the schema's range.
    pub fn patch(&mut self, world: &mut World, id: &str, parameter: &str, value: &Value) -> bool {
        let written = match self.names.resolve(world, id) {
            Some(e) => self.write_field(world, e, parameter, value, true).is_ok(),
            None => false,
        };
        if written {
            self.tuned.insert(format!("{id}.{parameter}"));
        }
        written
    }

    /// `run`: a program of members, stopping at the first refusal.
    pub fn run(&mut self, world: &mut World, program: &[Value]) -> og_core::Result<Value> {
        let mut results = Vec::new();
        if program.len() > PROBE_PROGRAM_MAX_OPERATIONS {
            return Ok(json!({"results": results, "refused": 0, "tick": self.tick(world)}));
        }
        let watched: Vec<String> = program
            .iter()
            .filter(|o| o.get("op").and_then(Value::as_str) == Some("expect"))
            .filter_map(|o| o.get("what").and_then(Value::as_str))
            .filter_map(|w| w.strip_prefix("event:"))
            .map(|w| w.split(':').next().unwrap_or("").to_owned())
            .collect();
        self.checks.watch_events(watched.iter().map(String::as_str));
        for (index, operation) in program.iter().enumerate() {
            let op = operation.get("op").and_then(Value::as_str).unwrap_or("");
            let answer: Option<Value> = match op {
                "advance" => match operation
                    .get("steps")
                    .and_then(Value::as_u64)
                    .filter(|s| (1..=PLAYTEST_MAX_TICKS).contains(s))
                {
                    Some(steps) => Some(json!(self.advance(world, steps)?)),
                    None => None,
                },
                "input" => Some(json!(operation.get("event").is_some_and(|v| {
                    InjectedInput::from_json(v)
                        .is_some_and(|e| self.input_at(world, &e, stamp_of(v)))
                }))),
                "pause" => Some(json!(self.pause())),
                "resume" => Some(json!(self.resume())),
                "read" => Some(json!({"tick": self.tick(world), "state": self.state(world)})),
                "pick" => {
                    let x = operation
                        .get("x")
                        .and_then(Value::as_f64)
                        .filter(|v| v.is_finite());
                    let y = operation
                        .get("y")
                        .and_then(Value::as_f64)
                        .filter(|v| v.is_finite());
                    match (x, y) {
                        (Some(x), Some(y)) => {
                            Some(self.pick(world, x as f32, y as f32).unwrap_or(Value::Null))
                        }
                        _ => None,
                    }
                }
                "entity" => operation
                    .get("id")
                    .and_then(Value::as_str)
                    .and_then(|id| self.entity(world, id))
                    .map(Value::Object),
                // An expectation answers its result, failed or not: the
                // program goes on and the caller reads `status`.
                "expect" => operation
                    .get("what")
                    .and_then(Value::as_str)
                    .and_then(|what| self.expect(world, what).ok())
                    .map(|r| r.to_json()),
                "describe" => Some(self.describe(world)),
                _ => None,
            };
            match answer {
                None | Some(Value::Bool(false)) => {
                    return Ok(
                        json!({"results": results, "refused": index, "tick": self.tick(world)}),
                    );
                }
                Some(value) => results.push(value),
            }
        }
        Ok(json!({"results": results, "refused": Value::Null, "tick": self.tick(world)}))
    }

    // ---- the engine's wire members ----------------------------------------

    /// `capture`: the frame at the current tick; `None` without a renderer.
    pub fn capture(
        &mut self,
        world: &World,
        width: u32,
        height: u32,
        fit: u32,
        region: Option<[f32; 4]>,
    ) -> Option<Value> {
        let frame = self.host.capture.as_mut()?.capture(world, width, height)?;
        // The rendered size answers as is: no decode, no encode.
        if region.is_none() && crate::frame::Rgba::fit_scale(frame.width, frame.height, fit) >= 1.0
        {
            return Some(json!({
                "tick": world.time().tick,
                "width": frame.width,
                "height": frame.height,
                "scale": self.host.surface_scale(world, frame.width),
                "region": Value::Null,
                "png": base64::encode(&frame.png),
            }));
        }
        let image = crate::frame::decode_png(&frame.png).ok()?;
        let mut scale = self.host.surface_scale(world, image.width);
        let image = match region {
            Some([x, y, w, h]) => {
                let r =
                    crate::frame::PixelRect::clipped(x, y, w, h, scale, image.width, image.height)?;
                image.crop(r)
            }
            None => image,
        };
        let fitted = image.fit(fit);
        scale *= fitted.width as f32 / image.width as f32;
        Some(json!({
            "tick": world.time().tick,
            "width": fitted.width,
            "height": fitted.height,
            "scale": scale,
            "region": region.map(|[x, y, w, h]| json!({ "x": x, "y": y, "width": w, "height": h })),
            "png": base64::encode(&fitted.png()),
        }))
    }
}

impl state::Host {
    /// Frame pixels per CSS pixel of the play surface for a frame
    /// `width` pixels wide: 1 without a UI module that knows the surface.
    pub(crate) fn surface_scale(&self, world: &World, width: u32) -> f32 {
        match self.ui.as_ref().and_then(|u| u.surface(world)) {
            Some((w, _, _)) if w > 0.0 => width as f32 / w,
            _ => 1.0,
        }
    }

    fn save(&mut self, world: &World) -> Saved {
        let bytes = world.snapshot();
        Saved {
            hash: world.world_hash(),
            world: bytes,
            tick: world.time().tick,
            render: self.capture.as_mut().map_or(0, |c| c.save_state()),
        }
    }

    fn load(&mut self, world: &mut World, saved: &Saved) -> og_core::Result<()> {
        if let Some(c) = self.capture.as_ref() {
            c.validate_restore(saved.render)?;
        }
        world.restore(&saved.world)?;
        if let Some(c) = self.capture.as_mut() {
            c.restore_state(saved.render);
        }
        Ok(())
    }
}

impl Probe {
    /// `sweep`: per value, restore the snapshot taken at the call, patch,
    /// advance (no command steps run) and capture; the world is restored
    /// afterwards. `None` for an unknown entity or parameter, a refused
    /// value, or no renderer. `Err` is a system that failed.
    ///
    /// Where frames land on a later call (the web), `None` also means "ask
    /// again": each call with the same description on the same world goes
    /// on from the frames it has, and the last answers them all.
    pub fn sweep(&mut self, world: &mut World, desc: &SweepDesc) -> og_core::Result<Option<Value>> {
        if !self.host.capture.as_ref().is_some_and(|c| c.has_renderer()) {
            return Ok(None);
        }
        let Some(e) = self.names.resolve(world, &desc.id) else {
            return Ok(None);
        };
        let tunable = Self::field_ref(world, &desc.parameter).is_ok_and(|(id, schema, i)| {
            schema.fields[i].tunable && world.get_bytes(e, id).is_ok()
        });
        if !tunable {
            return Ok(None);
        }
        let deferred = self.host.capture.as_ref().is_some_and(|c| c.deferred());
        // A screenshot's frame in flight goes first: they may share a size.
        if deferred
            && self
                .schedule
                .shot_waits(world, &mut self.host, &mut self.out)
        {
            return Ok(None);
        }
        let saved = self.host.save(world);
        let result = if deferred {
            self.sweep_deferred(world, e, desc, &saved)
        } else {
            self.sweep_values(world, e, desc, &saved)
        };
        self.host
            .load(world, &saved)
            .map_err(|err| fault("sweep could not restore the world", err))?;
        result
    }

    fn sweep_values(
        &mut self,
        world: &mut World,
        e: Entity,
        desc: &SweepDesc,
        saved: &Saved,
    ) -> og_core::Result<Option<Value>> {
        let mut frames = Vec::with_capacity(desc.values.len());
        for value in &desc.values {
            self.host.load(world, saved)?;
            if self
                .write_field(world, e, &desc.parameter, value, true)
                .is_err()
            {
                return Ok(None);
            }
            for _ in 0..desc.steps {
                self.tick_once(world)?;
            }
            let capture = self.host.capture.as_mut().expect("checked");
            let Some(frame) = capture.capture(world, desc.width, desc.height) else {
                return Ok(None);
            };
            frames.push(json!({
                "value": value,
                "tick": world.time().tick,
                "png": base64::encode(&frame.png),
            }));
        }
        Ok(Some(json!({ "frames": frames })))
    }

    fn sweep_deferred(
        &mut self,
        world: &mut World,
        e: Entity,
        desc: &SweepDesc,
        saved: &Saved,
    ) -> og_core::Result<Option<Value>> {
        let from = (saved.tick, saved.hash);
        let mut job = match self.saves.sweeping.take() {
            Some(job) if job.desc == *desc && job.from == from => job,
            _ => SweepJob {
                desc: desc.clone(),
                from,
                frames: Vec::with_capacity(desc.values.len()),
                waiting: None,
                polls: 0,
            },
        };
        loop {
            let Some(value) = desc.values.get(job.frames.len()) else {
                return Ok(Some(json!({ "frames": job.frames })));
            };
            match &job.waiting {
                Some(at) => self.host.load(world, at)?,
                None => {
                    self.host.load(world, saved)?;
                    if self
                        .write_field(world, e, &desc.parameter, value, true)
                        .is_err()
                    {
                        return Ok(None);
                    }
                    for _ in 0..desc.steps {
                        self.tick_once(world)?;
                    }
                    job.waiting = Some(self.host.save(world));
                    job.polls = 0;
                }
            }
            let capture = self.host.capture.as_mut().expect("checked");
            match capture.capture(world, desc.width, desc.height) {
                Some(frame) => {
                    job.frames.push(json!({
                        "value": value,
                        "tick": world.time().tick,
                        "png": base64::encode(&frame.png),
                    }));
                    job.waiting = None;
                }
                None if job.polls < MAX_CAPTURE_POLLS => {
                    job.polls += 1;
                    self.saves.sweeping = Some(job);
                    return Ok(None);
                }
                None => return Ok(None),
            }
        }
    }

    /// `reload`: snapshot, swap the bundle through the [`Reloader`],
    /// re-register the old world's types by name in the new one, restore.
    /// `None` (the old bundle and world keep running, the reason in the log)
    /// without a reloader, when the bundle fails to load, or when a type
    /// changed layout (field-by-field migration needs og-ecs support).
    pub fn reload(&mut self, world: &mut World) -> Option<Value> {
        let tick = world.time().tick;
        let Some(reloader) = self.host.reloader.as_mut() else {
            self.out
                .note(tick, true, "reload: this host has no bundle loader".into());
            return None;
        };
        let saved = world.snapshot();
        let rebuilt = match reloader.rebuild(world) {
            Ok(rebuilt) => rebuilt,
            Err(why) => {
                self.out.note(
                    tick,
                    true,
                    format!(
                        "reload: the new bundle failed to load ({why}); the old one keeps running"
                    ),
                );
                return None;
            }
        };
        let mut next = rebuilt.world;
        if let Err(err) = self.host.input.attach(&mut next) {
            self.out
                .note(tick, true, format!("reload: input: {}", err.reason()));
            return None;
        }
        // Types by name, fields by name (og-ecs `restore_migrating`): added
        // fields start at their defaults, removed fields and types are dropped.
        let report = match next.restore_migrating(&saved) {
            Ok(r) => r,
            Err(err) => {
                self.out.note(
                    tick,
                    true,
                    format!(
                        "reload: restore into the new bundle failed: {}; the old one keeps running",
                        err.reason()
                    ),
                );
                return None;
            }
        };
        let warnings = report.warnings.clone();
        let migrated: Vec<String> = report
            .migrated
            .iter()
            .chain(report.added.iter())
            .cloned()
            .collect();
        let dropped: Vec<String> = report
            .dropped
            .iter()
            .chain(report.removed.iter())
            .cloned()
            .collect();
        *world = next;
        Some(json!({
            "tick": world.time().tick,
            "ms": rebuilt.ms,
            "migrated": migrated,
            "dropped": dropped,
            "warnings": warnings,
        }))
    }

    /// `observe`: the game's reflected state as values, by record: each
    /// resource's fields, and each component's fields on up to
    /// [`OBSERVE_ROWS`] entities by probe id. `records` names the records
    /// (engine ones too); none is every record the game declared (a name
    /// without the `og.` prefix). Events hold no state and are left out.
    pub fn observe(&self, world: &World, records: Option<&[String]>, with_types: bool) -> Value {
        let mut resources = Map::new();
        let mut components = Map::new();
        let mut types = Map::new();
        for i in 0..world.type_count() {
            let id = og_ecs::SchemaId::from_index(i as u32);
            let Ok(schema) = world.schema(id) else {
                continue;
            };
            let wanted = match records {
                Some(r) => r.contains(&schema.name),
                None => !schema.name.starts_with("og."),
            };
            if !wanted {
                continue;
            }
            if with_types {
                let fields: Vec<Value> = schema
                    .fields
                    .iter()
                    .map(|f| {
                        json!({ "name": f.name, "type": f.ty.name(), "min": f.min, "max": f.max, "variants": f.variants })
                    })
                    .collect();
                types.insert(schema.name.clone(), json!({ "fields": fields }));
            }
            if world.has_resource_id(id) {
                if let Some(v) = resource_json(world, &schema.name) {
                    resources.insert(schema.name.clone(), v);
                }
                continue;
            }
            let sizes: Vec<usize> = schema.fields.iter().map(|f| f.ty.size()).collect();
            let mut rows = Map::new();
            world.scan_component(id, None, None, None, |entities, cols, _| {
                for (row, &e) in entities.iter().enumerate() {
                    if rows.len() >= OBSERVE_ROWS {
                        return;
                    }
                    let mut bytes = Vec::new();
                    for (k, size) in sizes.iter().enumerate() {
                        bytes.extend_from_slice(&cols[k][row * size..(row + 1) * size]);
                    }
                    rows.insert(
                        self.names.id_of(world, e),
                        Value::Object(value::component_json(schema, &bytes)),
                    );
                }
            });
            if !rows.is_empty() {
                components.insert(schema.name.clone(), Value::Object(rows));
            }
        }
        // A record's kind: a resource is present, a component has rows; the
        // host names the others (`Probe::observe`'s caller knows its layouts).
        for (name, t) in types.iter_mut() {
            if resources.contains_key(name) {
                t["kind"] = json!("resource");
            } else if components.contains_key(name) {
                t["kind"] = json!("component");
            }
        }
        let mut out =
            json!({ "tick": world.time().tick, "resources": resources, "components": components });
        if with_types {
            out["types"] = Value::Object(types);
        }
        out
    }

    /// `snapshot`: the whole world, kept in this process.
    pub fn snapshot(&mut self, world: &World) -> Value {
        let saved = self.host.save(world);
        let id = format!("s{}", self.saves.next_snapshot);
        self.saves.next_snapshot += 1;
        let presentation = self
            .host
            .capture
            .as_ref()
            .and_then(|c| c.presentation_hash())
            .map_or_else(String::new, |h| format!("{h:016x}"));
        let info = json!({
            "id": id,
            "tick": saved.tick,
            "hash": format!("{:016x}", saved.hash),
            "bytes": saved.world.len(),
            "presentation": presentation,
        });
        self.saves.snapshots.push_back((id, saved));
        while self.saves.snapshots.len() > MAX_SNAPSHOTS {
            self.saves.snapshots.pop_front();
        }
        info
    }

    /// `restore`: a snapshot this process took; `false` for an unknown id
    /// (or one dropped beyond [`MAX_SNAPSHOTS`]) or a world it no longer fits.
    /// A `quit` that ran is undone with it: the restored world is from
    /// before it, and its ticks play again (`og run --forks`).
    pub fn restore(&mut self, world: &mut World, id: &str) -> bool {
        let Some(saved) = self
            .saves
            .snapshots
            .iter()
            .find(|(sid, _)| sid == id)
            .map(|(_, s)| s.clone())
        else {
            return false;
        };
        let ok = self.host.load(world, &saved).is_ok();
        if ok {
            self.clock.quit = false;
        }
        ok
    }

    /// `dump`: a world, entity or UI dump; `None` for an unknown entity.
    pub fn dump(&self, world: &World, kind: DumpKind, id: &str) -> Option<Value> {
        match kind {
            DumpKind::World => Some(self.dump_world(world)),
            DumpKind::Ui => Some(self.dump_ui(world)),
            DumpKind::Desync => match self
                .commands
                .registered
                .get("dump_desync")
                .map(|r| &r.handler)
            {
                Some(Handler::Dump(handler, _)) => {
                    let mut v = handler.clone()(world, &[]).ok()?;
                    v["kind"] = json!("desync");
                    Some(v)
                }
                _ => None,
            },
            DumpKind::Entity => {
                let e = self.names.resolve(world, id)?;
                Some(self.dump_entity(world, e))
            }
        }
    }

    /// `commands`: schedule a script; `None` for a malformed one (the parse
    /// error is in the log).
    pub fn commands(&mut self, world: &World, script: &str) -> Option<Value> {
        match self.schedule(world, script) {
            Ok(report) => Some(report.to_json()),
            Err(err) => {
                self.out
                    .note(world.time().tick, true, format!("commands: {err}"));
                None
            }
        }
    }

    /// `save_scene`: every scene the world was assembled from rewritten
    /// from the world (the lines whose values changed, and the fields
    /// `set` and `patch` tuned), the boot scene's first and its fields at
    /// the top level; with `spawned`, the entities the game spawned under
    /// the boot scene's marked section; with `write`, over their source
    /// files. `Err` without a scene writer or a scene.
    pub fn save_scene(
        &mut self,
        world: &World,
        write: bool,
        spawned: bool,
    ) -> Result<Value, String> {
        let tuned: Vec<String> = self.tuned.iter().cloned().collect();
        let writer = self
            .host
            .scene_writer
            .as_mut()
            .ok_or("this host has no scene write-back")?;
        let saved = writer.save(world, &tuned, write, spawned)?;
        let first = saved
            .files
            .first()
            .cloned()
            .ok_or("no scene to write back")?;
        Ok(json!({
            "tick": world.time().tick,
            "source": first.source,
            "written": first.written,
            "lines": first.lines,
            "skipped": saved.skipped,
            "text": first.text,
            "spawned": saved.spawned,
            "scenes": saved.files.iter().map(|f| json!({
                "source": f.source, "written": f.written, "lines": f.lines, "text": f.text,
            })).collect::<Vec<_>>(),
        }))
    }

    /// `apply_prefab` and `revert_prefab`: the host's prefab write-back,
    /// noted in the log. `Err` without a scene writer, or for an entity
    /// not placed from a prefab.
    pub fn prefab_edit(
        &mut self,
        world: &mut World,
        edit: &crate::PrefabEdit,
    ) -> Result<crate::SavedScene, String> {
        let writer = self
            .host
            .scene_writer
            .as_mut()
            .ok_or("this host has no scene write-back")?;
        writer.prefab(world, edit)
    }

    // ---- dumps ------------------------------------------------------------

    /// The world dump: archetype tables (non-empty, in creation order) with
    /// their component sets and entity counts, systems in run order, resources.
    pub fn dump_world(&self, world: &World) -> Value {
        let archetypes: Vec<Value> = world
            .query(&QueryDesc::default())
            .map(|q| {
                q.map(|view| {
                    let components: Vec<String> = view
                        .types()
                        .iter()
                        .map(|id| schema_name(world, *id))
                        .collect();
                    json!({"components": components, "count": view.len()})
                })
                .collect()
            })
            .unwrap_or_default();
        // og-ecs reports every system: its declared access, what its
        // queries matched when it last ran and, when the host gave the world
        // a clock (World::set_clock), the milliseconds of that run (null
        // without one) and of every timed run summed, with their count.
        let systems: Vec<Value> = world
            .system_info()
            .into_iter()
            .map(|s| json!({"name": s.name, "matched": s.matched, "reads": s.reads, "writes": s.writes, "ms": s.ms, "totalMs": s.total_ms, "runs": s.runs}))
            .collect();
        let resources: Vec<String> = world
            .schemas()
            .into_iter()
            .filter(|s| {
                world
                    .id_by_name(&s.name)
                    .is_ok_and(|id| world.resource_bytes(id).is_ok())
            })
            .map(|s| s.name)
            .collect();
        json!({
            "kind": "world",
            "tick": world.time().tick,
            "hash": format!("{:016x}", world.world_hash()),
            "entities": world.entity_count(),
            "archetypes": archetypes,
            "systems": systems,
            "resources": resources,
        })
    }

    /// One entity: every component with every field, by component name
    /// (its transform is `components["og.Transform"]`).
    pub fn dump_entity(&self, world: &World, e: Entity) -> Value {
        let components: Map<String, Value> = Self::components(world, e)
            .into_iter()
            .map(|(schema, bytes)| {
                let fields = value::component_json(&schema, &bytes);
                (schema.name, Value::Object(fields))
            })
            .collect();
        let mut out = json!({
            "kind": "entity",
            "id": self.names.id_of(world, e),
            "name": self.names.name_of(world, e),
            "components": components,
        });
        if let Some(r) = self
            .host
            .capture
            .as_ref()
            .and_then(|c| c.entity_render(world, e))
        {
            out["render"] = r;
        }
        if let Some(p) = self.entity_prefab(world, e) {
            out["prefab"] = p;
        }
        out
    }

    /// Where an entity placed from a prefab came from, as `dump_entity`'s
    /// `prefab`: `{prefab, node, root, overrides}` (the root by its probe
    /// id, the overrides its instance's lines in the scene's text as
    /// `Component.field`); none for another entity or a host without
    /// scene write-back.
    pub fn entity_prefab(&self, world: &World, e: Entity) -> Option<Value> {
        let mut p = self.host.scene_writer.as_ref()?.entity_prefab(world, e)?;
        if let Some(bits) = p["root"].as_f64() {
            p["root"] = json!(self.names.id_of(world, Entity::from_bits(bits as u64)));
        }
        Some(p)
    }

    /// The UI tree in draw order; empty without a UI module.
    pub fn dump_ui(&self, world: &World) -> Value {
        let elements: Vec<Value> = self
            .host
            .ui
            .as_ref()
            .map(|ui| ui.elements(world).iter().map(|e| e.to_json()).collect())
            .unwrap_or_default();
        let surface = self
            .host
            .ui
            .as_ref()
            .and_then(|ui| ui.surface(world))
            .map_or(
                Value::Null,
                |(w, h, dpr)| json!({ "width": w, "height": h, "dpr": dpr }),
            );
        // The style's name ("" is the engine's default style): a
        // check that the game dressed its UI reads it here.
        let style = self.host.ui.as_ref().and_then(|ui| ui.style(world));
        let mut dump = json!({"kind": "ui", "tick": world.time().tick, "surface": surface, "style": style, "elements": elements});
        if let (Some(extra), Value::Object(o)) = (
            self.host.ui.as_ref().and_then(|ui| ui.extra(world)),
            &mut dump,
        ) {
            for (k, v) in extra {
                o.entry(k).or_insert(v);
            }
        }
        dump
    }

    /// `dump_ui:layers`: the layer stack alone (docs/ui.md §6).
    pub fn dump_layers(&self, world: &World) -> Value {
        let layers = self
            .host
            .ui
            .as_ref()
            .and_then(|ui| ui.extra(world))
            .and_then(|mut e| e.remove("layers"))
            .unwrap_or_else(|| json!([]));
        json!({"kind": "layers", "tick": world.time().tick, "layers": layers})
    }

    /// `dump_ui:motion`: the motion alone (docs/ui.md §6): each element in
    /// a visual state or animating, with its states, what is moving, what
    /// plays and whether it is leaving.
    pub fn dump_motion(&self, world: &World) -> Value {
        let motion = self
            .host
            .ui
            .as_ref()
            .and_then(|ui| ui.extra(world))
            .and_then(|mut e| e.remove("motion"))
            .unwrap_or_else(|| json!([]));
        json!({"kind": "motion", "tick": world.time().tick, "motion": motion})
    }
}
