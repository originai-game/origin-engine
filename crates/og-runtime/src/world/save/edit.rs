//! Saves opened for the game's migrations (World.openSave, SaveEdit).

use super::*;

/// An open save: its engine, the snapshot opened for editing, the tick it
/// was taken at, and the game's save version it is at.
pub(crate) struct Open {
    pub(super) engine: u32,
    pub(super) edit: SnapshotEdit,
    pub(super) tick: u64,
    pub(super) version: u32,
    pub(super) ui: Option<crate::present::presentation::UiCheckpoint>,
}

/// `World.openSave`: the save with the engine's migrations applied, open
/// for the game's (a handle for the `og_save_edit_*` calls).
pub fn og_world_open_save(world: u32, save: String) -> Result<serde_json::Value, String> {
    let eng = engine(world)?;
    let current = eng.state.borrow().save_version;
    let s = upgraded(&save, current)?;
    let edit = match SnapshotEdit::from_bytes(&s.snapshot) {
        Ok(e) => e,
        Err(e) => return Err(format!("the save cannot be opened: {}", e.reason())),
    };
    let open = Open {
        engine: world,
        edit,
        tick: s.tick,
        version: s.version,
        ui: s.ui,
    };
    let handle = HANDLES.with(|h| {
        let o = &mut h.borrow_mut().saves;
        match o.iter().position(Option::is_none) {
            Some(i) => {
                o[i] = Some(open);
                i
            }
            None => {
                o.push(Some(open));
                o.len() - 1
            }
        }
    });
    Ok(json!(handle + 1))
}

pub(super) fn with_open<T>(
    handle: u32,
    f: impl FnOnce(&mut Open) -> Result<T, String>,
) -> Result<T, String> {
    HANDLES.with(|h| {
        let o = &mut h.borrow_mut().saves;
        let open = (handle as usize)
            .checked_sub(1)
            .and_then(|i| o.get_mut(i))
            .and_then(Option::as_mut)
            .ok_or("this save is closed: open it again with world.openSave")?;
        f(open)
    })
}

pub(super) fn reason(e: og_core::Error) -> String {
    let r = e.reason().to_string();
    r.strip_prefix("snapshot: ").unwrap_or(&r).to_string()
}

/// `SaveEdit.version`.
pub fn og_save_edit_get_version(edit: u32) -> u32 {
    with_open(edit, |o| Ok(o.version)).unwrap_or(0)
}

/// `SaveEdit.types`: every type the save holds, with its saved version
/// and fields.
pub fn og_save_edit_types(edit: u32) -> Result<Vec<crate::host::wire::SaveType>, String> {
    with_open(edit, |o| {
        Ok(o.edit
            .schemas()
            .iter()
            .map(|t| SaveType {
                name: t.name.clone(),
                version: t.version,
                fields: t.fields.iter().map(|f| f.name.clone()).collect(),
            })
            .collect::<Vec<_>>())
    })
}

/// The saved type's layout, for reading its records.
pub(super) fn saved_layout(
    edit: &SnapshotEdit,
    ty: &str,
) -> Result<crate::host::codec::Layout, String> {
    let schema = edit
        .schema(ty)
        .ok_or_else(|| format!("the save holds no type '{ty}'"))?;
    Ok(crate::host::codec::Layout::from_schema(
        schema,
        crate::host::codec::RecordKind::Component,
    ))
}

/// `SaveEdit.records`: a type's values as the save holds them, components
/// by entity in storage order, then its resource (entity null).
pub fn og_save_edit_records(
    edit: u32,
    ty: String,
) -> Result<Vec<crate::host::wire::SaveRecord>, String> {
    with_open(edit, |o| {
        let layout = saved_layout(&o.edit, &ty)?;
        let mut out: Vec<SaveRecord> = o
            .edit
            .components(&ty)
            .iter()
            .map(|(e, b)| SaveRecord {
                entity: Some(e.to_bits()),
                value: layout.decode(b),
            })
            .collect();
        if let Some(b) = o.edit.resource(&ty) {
            out.push(SaveRecord {
                entity: None,
                value: layout.decode(b),
            });
        }
        Ok(out)
    })
}

/// This build's layout and schema of `ty`, for writing into a save.
pub(super) fn current(
    engine_id: u32,
    ty: &str,
) -> Result<(crate::host::codec::Layout, og_core::reflect::Schema), String> {
    let eng = engine(engine_id)?;
    let layout = eng.state.borrow().layouts.get(ty).cloned().ok_or_else(|| {
        format!(
            "'{ty}' is not a type of this build: a migration writes the types the game declares now"
        )
    })?;
    let w = eng.world()?;
    let schema = w
        .id_by_name(ty)
        .and_then(|id| w.schema(id).cloned())
        .map_err(|e| e.reason().to_string())?;
    Ok((layout, schema))
}

/// `SaveEdit.setRecords`: replaces a type's values with `records`, written
/// as this build's `into` (default the type itself): each record's fields
/// by name, those it leaves out at their defaults. The type's old values
/// go; its queued events too. A type the save does not hold is added.
pub fn og_save_edit_set_records(edit: u32, ty: String, desc: Option<String>) -> Result<(), String> {
    with_open(edit, |o| {
        let d: crate::host::wire::SetRecordsDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
        let into = if d.into.is_empty() {
            ty.clone()
        } else {
            d.into
        };
        // A type the save does not hold is added (written as `into`).
        let remove: &[&str] = if o.edit.schema(&ty).is_some() {
            &[ty.as_str()]
        } else {
            &[]
        };
        let (layout, schema) = current(o.engine, &into)?;
        let mut values = TypeValues {
            schema,
            components: Vec::new(),
            resource: None,
        };
        for r in &d.records {
            let bytes = layout
                .encode(None, Some(&r.value))
                .map_err(|e| format!("a record for {into}: {e}"))?;
            match r.entity {
                Some(e) => values
                    .components
                    .push((og_ecs::Entity::from_bits(e), bytes)),
                None if values.resource.is_none() => values.resource = Some(bytes),
                None => return Err(format!("two {into} resources (records without an entity)")),
            }
        }
        o.edit.replace_types(remove, vec![values]).map_err(reason)
    })
}

/// `SaveEdit.renameType`.
pub fn og_save_edit_rename_type(edit: u32, from: String, to: String) -> Result<(), String> {
    with_open(edit, |o| {
        let mut schema = o
            .edit
            .schema(&from)
            .cloned()
            .ok_or_else(|| format!("the save holds no type '{from}'"))?;
        schema.name = to.clone();
        let values = TypeValues {
            schema,
            components: o.edit.components(&from),
            resource: o.edit.resource(&from).map(<[u8]>::to_vec),
        };
        o.edit
            .replace_types(&[from.as_str()], vec![values])
            .map_err(reason)
    })
}

/// `SaveEdit.renameField`.
pub fn og_save_edit_rename_field(
    edit: u32,
    ty: String,
    from: String,
    to: String,
) -> Result<(), String> {
    with_open(edit, |o| {
        let mut schema = o
            .edit
            .schema(&ty)
            .cloned()
            .ok_or_else(|| format!("the save holds no type '{ty}'"))?;
        if schema.field_index(&to).is_some() {
            return Err(format!("{ty} already has a field '{to}'"));
        }
        let i = schema
            .field_index(&from)
            .ok_or_else(|| format!("{ty} has no field '{from}' in the save"))?;
        schema.fields[i].name = to;
        o.edit
            .map_type(&ty, schema, &mut |v| Ok(v.to_vec()))
            .map_err(reason)
    })
}

/// `SaveEdit.dropType`.
pub fn og_save_edit_drop_type(edit: u32, ty: String) -> Result<(), String> {
    with_open(edit, |o| {
        o.edit
            .replace_types(&[ty.as_str()], Vec::new())
            .map_err(reason)
    })
}

/// `SaveEdit.spawn`: an entity added to the save, its components written
/// as this build's types.
pub fn og_save_edit_spawn(edit: u32, desc: Option<String>) -> Result<u64, String> {
    with_open(edit, |o| {
        let d: crate::host::wire::SaveSpawnDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
        let mut components = Vec::new();
        for (ty, value) in d.components.as_object().into_iter().flatten() {
            let (layout, schema) = current(o.engine, ty)?;
            let bytes = layout
                .encode(None, Some(value))
                .map_err(|e| format!("SaveSpawnDesc.components.{ty}: {e}"))?;
            components.push((schema, bytes));
        }
        let name = Some(d.name.as_str()).filter(|n| !n.is_empty());
        o.edit
            .spawn(name, components)
            .map(|e| e.to_bits())
            .map_err(reason)
    })
}

/// `SaveEdit.text`: the save as text at the game's current save version
/// (what world.restore takes after the migrations ran).
pub fn og_save_edit_text(edit: u32) -> Result<String, String> {
    with_open(edit, |o| {
        let version = engine(o.engine)?.state.borrow().save_version;
        Ok(encode(&o.edit.to_bytes(), o.tick, version, o.ui.as_ref()))
    })
}

/// `SaveEdit.close`.
pub fn og_save_edit_close(edit: u32) {
    // Taken out of the table before it is dropped.
    let slot = HANDLES.with(|h| {
        (edit as usize)
            .checked_sub(1)
            .and_then(|i| h.borrow_mut().saves.get_mut(i).map(Option::take))
    });
    drop(slot);
}
