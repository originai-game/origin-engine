use super::*;

/// `og_engine_write_resource`.
pub fn write_resource(world: u32, resource: &str, value: &str) -> String {
    match parse(Some(value)).and_then(|v| write_resource_value(world, resource, &v)) {
        Ok(()) => super::ok_void(),
        Err(e) => refuse(e),
    }
}

/// Writes the fields `value` names into a resource (the others keep
/// theirs): through the running system's context, which refuses a
/// resource it did not declare among its writes, or straight into the
/// world outside systems.
pub(crate) fn write_resource_value(
    world: u32,
    resource: &str,
    value: &Value,
) -> Result<(), String> {
    if let Some(f) = running(world) {
        return (|| {
            let eng = engine(world)?;
            let (layout, id) = {
                let st = eng.state.borrow();
                let layout = st
                    .layouts
                    .get(resource)
                    .filter(|l| l.kind == RecordKind::Resource)
                    .ok_or_else(|| format!("'{resource}' is not a registered resource"))?
                    .clone();
                let id = st
                    .names_by_id
                    .iter()
                    .position(|n| n == resource)
                    .map(|i| SchemaId::from_index(i as u32))
                    .ok_or_else(|| format!("'{resource}' is not registered with this world"))?;
                (layout, id)
            };
            if let Some(field) = layout
                .fields
                .iter()
                .find(|fl| fl.readonly && value.get(&fl.name).is_some())
            {
                return Err(format!("{resource}.{} is read only", field.name));
            }
            let declared = eng.state.borrow().systems[f.system]
                .writes
                .iter()
                .any(|w| w == resource);
            if !declared {
                return Err(super::api::undeclared(&eng, f.system, "write", resource));
            }
            // SAFETY: the frame is published while its system runs.
            let ctx = unsafe { f.ctx() };
            let current = ctx.resource_bytes(id).map_err(err)?;
            let bytes = layout.encode(Some(&current), Some(value))?;
            ctx.set_resource_bytes(id, &bytes).map_err(err)
        })();
    }
    (|| {
        let eng = engine(world)?;
        let st = eng.state.borrow();
        let layout = st
            .layouts
            .get(resource)
            .filter(|l| l.kind == RecordKind::Resource)
            .ok_or_else(|| format!("'{resource}' is not a registered resource"))?
            .clone();
        drop(st);
        if let Some(f) = layout
            .fields
            .iter()
            .find(|f| f.readonly && value.get(&f.name).is_some())
        {
            return Err(format!("{resource}.{} is read only", f.name));
        }
        let w = eng.world()?;
        crate::lazy::ensure(&eng, w, resource)?;
        let id = w.id_by_name(resource).map_err(err)?;
        let current = w.resource_bytes(id).map_err(err)?;
        let bytes = layout.encode(Some(&current), Some(value))?;
        w.insert_resource_bytes(id, &bytes).map_err(err)
    })()
}

// ── resources as bytes (the runtime's typed handles) ────────────────────────

thread_local! {
    /// Moves whenever a world's layouts may have changed (an engine
    /// assembled, a reload swapped in): what a resource's bytes were made
    /// with travels beside them, so a handle never reads or writes them
    /// with a layout that no longer holds.
    static LAYOUT_EPOCH: Cell<u32> = const { Cell::new(1) };
}

pub(crate) fn bump_layout_epoch() {
    LAYOUT_EPOCH.with(|e| e.set(e.get().wrapping_add(1).max(1)));
}

fn layout_epoch() -> u32 {
    LAYOUT_EPOCH.with(Cell::get)
}

/// Resources whose handle is computed rather than their packed record.
fn computed_resource(name: &str) -> bool {
    name == Time::NAME || name == Rng::NAME
}

fn kind_name(f: &super::codec::FieldLayout) -> &'static str {
    use super::codec::Kind;
    match f.kind {
        Kind::F32 => "f32",
        Kind::F64 => "f64",
        Kind::U32 => "u32",
        Kind::I32 => "i32",
        Kind::Bool => "bool",
        Kind::Vector2 => "Vector2",
        Kind::Vector3 => "Vector3",
        Kind::Quaternion => "Quaternion",
        Kind::Color => "Color",
        Kind::Matrix4 => "Matrix4",
        Kind::Entity | Kind::Geometry | Kind::Material | Kind::Asset | Kind::Sound => "handle",
        Kind::Str => "str",
        Kind::Enum => "enum",
        Kind::Ease => "ease",
    }
}

/// `og_engine_resource_layout`.
pub fn resource_layout(world: u32, resource: &str) -> String {
    let answer = (|| {
        if computed_resource(resource) {
            return Err(format!("'{resource}' is read through its JSON value"));
        }
        let eng = engine(world)?;
        let st = eng.state.borrow();
        let layout = st
            .layouts
            .get(resource)
            .filter(|l| l.kind == RecordKind::Resource)
            .ok_or_else(|| format!("'{resource}' is not a registered resource"))?;
        let mut offset = 0;
        let fields: Vec<Value> = layout
            .fields
            .iter()
            .map(|f| {
                let size = f.ty.size();
                let mut v = json!({
                    "name": f.name, "kind": kind_name(f), "offset": offset, "size": size,
                    "readonly": f.readonly,
                });
                // A list: its element's kind and its capacity in elements;
                // the count lane comes first, each lane the element's size.
                if let Some((_, capacity)) = f.ty.list_of() {
                    v["kind"] = json!("list");
                    v["element"] = json!(kind_name(f));
                    v["capacity"] = json!(capacity);
                }
                if !f.variants.is_empty() {
                    v["variants"] = json!(f.variants);
                }
                offset += size;
                v
            })
            .collect();
        Ok(json!({ "epoch": layout_epoch(), "size": offset, "fields": fields }))
    })();
    match answer {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

/// The id of resource `name` in the world `eng` runs (through the running
/// system's context while one runs). Outside systems a lazy resource
/// (super::super::lazy) is registered by its first write.
fn resource_id(eng: &Engine, name: &str) -> Result<SchemaId, String> {
    if let Some(f) = running(eng.id) {
        // SAFETY: the frame is published while its system runs.
        return unsafe { f.ctx() }.id_by_name(name).map_err(err);
    }
    let world = eng.world()?;
    crate::lazy::ensure(eng, world, name)?;
    world.id_by_name(name).map_err(err)
}

/// A lazy resource no game has named yet, read outside systems: its
/// default, without registering it (reading leaves the hash alone).
fn unregistered_default(eng: &Engine, name: &str) -> Option<Vec<u8>> {
    if running(eng.id).is_some() || !crate::lazy::is_deferred(eng, name) {
        return None;
    }
    if eng.world().ok()?.id_by_name(name).is_ok() {
        return None;
    }
    let st = eng.state.borrow();
    st.layouts.get(name)?.encode(None, None).ok()
}

/// `og_engine_read_resource`: the packed bytes and the layout epoch, or
/// nothing when refused.
pub fn read_resource(world: u32, resource: &str) -> Vec<u8> {
    let read = || -> Result<Vec<u8>, String> {
        if computed_resource(resource) {
            return Err(String::new());
        }
        let eng = engine(world)?;
        if let Some(mut bytes) = unregistered_default(&eng, resource) {
            bytes.extend_from_slice(&layout_epoch().to_le_bytes());
            return Ok(bytes);
        }
        let id = resource_id(&eng, resource)?;
        let mut bytes = match running(world) {
            // SAFETY: the frame is published while its system runs. The
            // context refuses a resource its system did not declare.
            Some(f) => unsafe { f.ctx() }.resource_bytes(id).map_err(err)?,
            None => eng.world()?.resource_bytes(id).map_err(err)?,
        };
        bytes.extend_from_slice(&layout_epoch().to_le_bytes());
        Ok(bytes)
    };
    read().unwrap_or_default()
}

/// `og_engine_write_resource_bytes`.
pub fn write_resource_bytes(world: u32, resource: &str, bytes: &[u8]) -> String {
    let write = || -> Result<(), String> {
        let Some((record, epoch)) = bytes.split_last_chunk::<4>() else {
            return Err("the bytes carry no layout epoch".into());
        };
        if u32::from_le_bytes(*epoch) != layout_epoch() {
            return Err(format!(
                "'{resource}' was read with a layout that no longer holds (a reload or a new engine): take world.resource({resource}) again"
            ));
        }
        let eng = engine(world)?;
        let id = resource_id(&eng, resource)?;
        // Read-only fields keep the engine's value whatever the bytes say.
        let readonly: Vec<(usize, usize)> = {
            let st = eng.state.borrow();
            let layout = st
                .layouts
                .get(resource)
                .filter(|l| l.kind == RecordKind::Resource && !computed_resource(resource))
                .ok_or_else(|| format!("'{resource}' is not a registered resource"))?;
            if layout.size() != record.len() {
                return Err(format!(
                    "'{resource}' is {} bytes packed, got {}",
                    layout.size(),
                    record.len()
                ));
            }
            let mut at = 0;
            let mut out = Vec::new();
            for f in layout.fields.iter() {
                let n = f.ty.size();
                if f.readonly {
                    out.push((at, n));
                }
                at += n;
            }
            out
        };
        let patched = |current: Vec<u8>| -> Vec<u8> {
            let mut b = record.to_vec();
            for (at, n) in &readonly {
                b[*at..at + n].copy_from_slice(&current[*at..at + n]);
            }
            b
        };
        if let Some(f) = running(world) {
            let declared = eng.state.borrow().systems[f.system]
                .writes
                .iter()
                .any(|w| w == resource);
            if !declared {
                return Err(super::api::undeclared(&eng, f.system, "write", resource));
            }
            // SAFETY: the frame is published while its system runs.
            let ctx = unsafe { f.ctx() };
            let b = if readonly.is_empty() {
                record.to_vec()
            } else {
                patched(ctx.resource_bytes(id).map_err(err)?)
            };
            return ctx.set_resource_bytes(id, &b).map_err(err);
        }
        let w = eng.world()?;
        let b = if readonly.is_empty() {
            record.to_vec()
        } else {
            patched(w.resource_bytes(id).map_err(err)?)
        };
        w.insert_resource_bytes(id, &b).map_err(err)
    };
    match write() {
        Ok(()) => super::ok_void(),
        Err(e) => refuse(e),
    }
}

// ── views ───────────────────────────────────────────────────────────────────

/// The columns of `component` in view `view`, in field order, or why not.
/// A running system's view borrows them here, the first time they are
/// asked for, and holds them until the system returns.
pub fn view_columns(view: u32, component: &str) -> Result<Vec<ColumnPtr>, String> {
    HANDLES.with(|t| {
        let mut t = t.borrow_mut();
        let stale = "stale view: views last for the system call (or, outside systems, until the next structural change) they were handed out in";
        let (engine_id, rec) = t.views.get_mut(view).ok_or(stale)?;
        let cols: Vec<ColumnPtr> = rec
            .columns
            .iter()
            .filter(|(c, _)| c == component)
            .map(|(_, p)| *p)
            .collect();
        if !cols.is_empty() {
            return Ok(cols);
        }
        let wanted: Vec<QueryColumn> = rec
            .lazy
            .as_ref()
            .map(|(_, specs)| {
                specs
                    .iter()
                    .filter(|c| c.component == component)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        if wanted.is_empty() {
            return Err(if rec.types.iter().any(|t| t == component) {
                format!("'{component}' is not among the reads or writes this view was made with")
            } else {
                format!("this archetype has no '{component}'")
            });
        }
        let arch = rec.lazy.as_ref().map_or(0, |(k, _)| *k);
        let frame = running(*engine_id).ok_or(stale)?;
        // SAFETY: the frame is published while its system runs; the
        // archetypes cannot move until it returns (structural changes wait).
        let ctx = unsafe { frame.ctx() };
        let a = ctx
            .query(0)
            .map_err(err)?
            .nth(arch)
            .ok_or(stale)?;
        let mut out = Vec::with_capacity(wanted.len());
        let mut held = frame.held.borrow_mut();
        for c in &wanted {
            let (ptr, guard) = if c.writable {
                let mut b = a.write_bytes(c.field).map_err(|e| taken(e, component))?;
                let p = ColumnPtr {
                    ptr: b.as_mut_ptr(),
                    len: b.len(),
                    writable: true,
                };
                // SAFETY: the borrow lives in the frame, which `run_system`
                // clears before the world's borrow for the call ends.
                (p, Held::Write(unsafe { std::mem::transmute::<std::cell::RefMut<'_, [u8]>, std::cell::RefMut<'static, [u8]>>(b) }))
            } else {
                let b = a.read_bytes(c.field).map_err(|e| taken(e, component))?;
                let p = ColumnPtr {
                    ptr: b.as_ptr() as *mut u8,
                    len: b.len(),
                    writable: false,
                };
                // SAFETY: as above.
                (p, Held::Read(unsafe { std::mem::transmute::<std::cell::Ref<'_, [u8]>, std::cell::Ref<'static, [u8]>>(b) }))
            };
            held.push(guard);
            out.push(ptr);
            rec.columns.push((component.to_string(), ptr));
        }
        Ok(out)
    })
}

/// A view's column the running code already borrows otherwise (an engine
/// call in progress holds it): the reason, with what to do.
fn taken(e: Error, component: &str) -> String {
    if e.kind() == ErrorKind::Access {
        format!(
            "the column view of '{component}' is refused: the engine holds that column for a call in progress; take the view outside that call"
        )
    } else {
        err(e)
    }
}

/// The entities of view `view` as (index, generation) pairs; empty for a
/// stale view.
pub fn view_entities(view: u32) -> Vec<u32> {
    HANDLES.with(|t| {
        let t = t.borrow();
        let Some((_, rec)) = t.views.get(view) else {
            return Vec::new();
        };
        // SAFETY: the record is live, so the archetype's entity list has not
        // moved since the view was made.
        let ents = unsafe { std::slice::from_raw_parts(rec.entities, rec.count as usize) };
        let mut out = Vec::with_capacity(ents.len() * 2);
        for e in ents {
            out.push(e.index());
            out.push(e.generation());
        }
        out
    })
}

/// Builds view records for a world query outside systems.
pub(crate) fn world_query_views(eng: &Engine, desc: &QueryDesc) -> Result<Vec<u32>, String> {
    let world = eng.world()?;
    let tick = world.time().tick as u32;
    let st = eng.state.borrow();
    let mut out = Vec::new();
    for a in world.query(desc).map_err(err)? {
        let mut cols = Vec::new();
        for name in desc.write.iter() {
            let layout = st
                .layouts
                .get(name)
                .ok_or_else(|| format!("'{name}' is not a registered component"))?;
            let id = world.id_by_name(name).map_err(err)?;
            for f in layout.fields.iter() {
                let field = world.field_by_id(id, &f.name).map_err(err)?;
                let mut b = a.write_bytes(field).map_err(err)?;
                cols.push((
                    name.clone(),
                    ColumnPtr {
                        ptr: b.as_mut_ptr(),
                        len: b.len(),
                        writable: true,
                    },
                ));
            }
        }
        let types = a
            .types()
            .iter()
            .filter_map(|id: &SchemaId| st.names_by_id.get(id.index() as usize).cloned())
            .collect();
        let rec = ViewRecord {
            count: a.len() as u32,
            tick,
            entities: a.entities().as_ptr(),
            columns: cols,
            types,
            rows: a.filtered().then(|| a.rows()),
            lazy: None,
        };
        out.push(HANDLES.with(|t| t.borrow_mut().views.push((eng.id, rec))));
    }
    Ok(out)
}
