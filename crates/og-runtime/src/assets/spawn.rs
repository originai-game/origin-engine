//! Spawning a cooked scene: its rows migrated to the world's layouts, its parts placed.

use super::*;

/// One field of a packed row as the JSON the codec reads, shaped for the
/// target field's kind.
fn field_json(ty: FieldType, bytes: &[u8], target: &crate::codec::FieldLayout) -> Value {
    if ty.is_string() {
        return json!(og_core::unpack_str(bytes));
    }
    if let Some((element, _)) = ty.list_of() {
        // A list moves by its elements; the codec refuses more than fit.
        let one = element.field_type();
        return Value::Array(
            og_core::list_elements(ty, bytes)
                .map(|e| field_json(one, e, target))
                .collect(),
        );
    }
    let lanes: Vec<f64> = (0..ty.lanes())
        .map(|i| {
            let at = i * ty.scalar().size();
            let b = &bytes[at..at + ty.scalar().size()];
            match &*ty.name() {
                "f64" => f64::from_le_bytes(b.try_into().unwrap_or([0; 8])),
                "i32" => i32::from_le_bytes(b.try_into().unwrap_or([0; 4])) as f64,
                "u32" | "enum" => u32::from_le_bytes(b.try_into().unwrap_or([0; 4])) as f64,
                "i64" => i64::from_le_bytes(b.try_into().unwrap_or([0; 8])) as f64,
                "u64" => u64::from_le_bytes(b.try_into().unwrap_or([0; 8])) as f64,
                _ if b.len() == 1 => f64::from(b[0]),
                _ => f64::from(f32::from_le_bytes(b.try_into().unwrap_or([0; 4]))),
            }
        })
        .collect();
    use crate::codec::Kind;
    match target.kind {
        Kind::Vector2 | Kind::Vector3 | Kind::Quaternion | Kind::Color => {
            let keys: &[&str] = match target.kind {
                Kind::Vector2 => &["x", "y"],
                Kind::Vector3 => &["x", "y", "z"],
                Kind::Quaternion => &["x", "y", "z", "w"],
                _ => &["r", "g", "b"],
            };
            let mut m = Map::new();
            for (i, k) in keys.iter().enumerate() {
                m.insert((*k).into(), json!(lanes.get(i).copied().unwrap_or(0.0)));
            }
            Value::Object(m)
        }
        Kind::Matrix4 => json!({ "elements": lanes }),
        Kind::Bool => json!(lanes.first().copied().unwrap_or(0.0) != 0.0),
        _ => json!(lanes.first().copied().unwrap_or(0.0)),
    }
}

/// A scene table's row as a value of the registered layout, migrated by
/// field name: missing fields keep their defaults, extra ones are reported.
pub(crate) fn migrate(
    schema: &Schema,
    row: &[u8],
    layout: &Layout,
    dropped: &mut Vec<String>,
) -> Value {
    let packed = og_core::reflect::Layout::of(schema);
    let mut out = Map::new();
    for (i, f) in schema.fields.iter().enumerate() {
        let Some((_, target)) = layout.field(&f.name) else {
            let name = format!("{}.{}", layout.schema, f.name);
            if !dropped.contains(&name) {
                dropped.push(name);
            }
            continue;
        };
        let range = packed.range(i);
        // A string moves by its text and a list by its elements; the codec
        // refuses one over the capacity.
        let strings = f.ty.is_string() && target.ty.is_string();
        let lists = f.ty.is_list() && target.ty.is_list();
        if !strings && !lists && f.ty.lanes() != target.ty.lanes() {
            dropped.push(format!(
                "{}.{} ({} lanes in the scene, {} registered)",
                layout.schema,
                f.name,
                f.ty.lanes(),
                target.ty.lanes()
            ));
            continue;
        }
        out.insert(f.name.clone(), field_json(f.ty, &row[range], target));
    }
    Value::Object(out)
}

/// The registered component a scene table names: exact, else the engine's
/// `og.<name>`.
fn resolve<'a>(layouts: &'a BTreeMap<String, Layout>, name: &str) -> Option<&'a Layout> {
    layouts
        .get(name)
        .or_else(|| layouts.get(&format!("og.{name}")))
        .filter(|l| l.kind == crate::codec::RecordKind::Component)
}

/// Where a scene's entities go ([`spawn_placed`]).
#[derive(Clone, Copy, Debug)]
pub(crate) enum Placement {
    /// Its top-level entities under this root (at the top level when `None`).
    Under(Option<Entity>),
    /// A world cell's sub-scene (G3-3, [`crate::stream::cells`]): its
    /// top-level entities are roots placed at the cell's world pose after
    /// their own transforms, and every entity it spawns carries an
    /// `og.CellMember` naming the cell.
    Cell {
        cell: Entity,
        pose: og_core::math::Pose,
    },
}

/// Spawns scene entity rows under `root` (top level when `None`). The whole
/// scene is refused before anything spawns when a name is taken.
pub(crate) fn spawn_scene(
    eng: &Engine,
    scene: &SceneDesc,
    root: Option<Entity>,
) -> Result<Vec<Entity>, String> {
    spawn_placed(eng, scene, Placement::Under(root))
}

/// The names `scene`'s entities take, each free: not an entity's, not
/// held for a scene a system is placing (`world.instantiate` there), not
/// `root` (the name its root is about to take, or none) and not twice in
/// the scene. A prefab's own blob (og.load.prefab) spawns unnamed, since it
/// is placed many times: it takes none.
pub(crate) fn names_free(
    eng: &Engine,
    scene: &SceneDesc,
    root: &str,
) -> Result<Vec<String>, String> {
    if scene.prefabs.first() == Some(&scene.id) {
        return Ok(Vec::new());
    }
    let st = eng.state.borrow();
    let mut seen = std::collections::BTreeSet::new();
    for e in &scene.entities {
        if e.name.is_empty() {
            continue;
        }
        if !seen.insert(e.name.clone()) {
            return Err(format!(
                "{}: two entities are named '{}'; rename one in the scene",
                scene.id, e.name
            ));
        }
        if st.names.contains_key(&e.name) || st.held_names.contains(&e.name) || e.name == root {
            // A root does not scope names: world.find has one entity per
            // name, so a second instance clashes wherever it is placed.
            return Err(format!(
                "{}: an entity named '{}' already exists, and names are unique in the world, under any root; despawn the one that has it, rename it in the scene, or make what is placed more than once a prefab (og.load.prefab spawns its entities unnamed)",
                scene.id, e.name
            ));
        }
    }
    Ok(seen.into_iter().collect())
}

/// [`spawn_scene`] with a [`Placement`]. An entity with a sub-scene
/// (`scene "<id>"`) gets it spawned under it, or, when it is a world cell
/// (`og.StreamCell`), its `scene` set to the sub-scene's handle for the
/// cell to load.
pub(crate) fn spawn_placed(
    eng: &Engine,
    scene: &SceneDesc,
    placement: Placement,
) -> Result<Vec<Entity>, String> {
    let mut spawned = Vec::with_capacity(scene.entities.len());
    spawn_part(eng, scene, placement, &mut spawned, scene.entities.len())?;
    Ok(spawned)
}

/// [`spawn_placed`] in parts (a world cell's `budget`, G3-3): spawns the
/// scene's entities from `spawned.len()` up to `end`, pushing each onto
/// `spawned` (the entities earlier parts spawned, in order: parents and
/// prefab roots are found there), and once the last one is in, what
/// follows the entities (sub-scenes, resources, the instance record).
/// The name check runs with the first part. On `Err`, `spawned` holds
/// what spawned before the refusal.
pub(crate) fn spawn_part(
    eng: &Engine,
    scene: &SceneDesc,
    placement: Placement,
    spawned: &mut Vec<Entity>,
    end: usize,
) -> Result<(), String> {
    let end = end.min(scene.entities.len());
    let root = match placement {
        Placement::Under(root) => root,
        Placement::Cell { .. } => None,
    };
    // A prefab's own blob (og.load.prefab) spawns unnamed: it is placed
    // many times.
    let prefab = scene.prefabs.first() == Some(&scene.id);
    if spawned.is_empty() {
        names_free(eng, scene, "")?;
        crate::timeline::register_scene_paths(eng.id, &scene.paths);
    }
    let layouts = eng.state.borrow().layouts.clone();
    let transform = resolve(&layouts, "Transform").cloned();
    let parent_layout = resolve(&layouts, "Parent").cloned();
    let model_layout = resolve(&layouts, "Model").cloned();
    let mut dropped = Vec::new();
    let mut tables = Vec::new();
    for t in &scene.components {
        match resolve(&layouts, &t.name) {
            Some(l) => {
                // A lazy engine record (og.StreamCell, ...) is registered
                // once a scene names it.
                crate::lazy::ensure(eng, eng.world()?, &l.schema)?;
                tables.push((t, l.clone()))
            }
            None => dropped.push(format!(
                "{} (no registered component of that name; declare it with defineComponent before the scene loads)",
                t.name
            )),
        }
    }
    // The prefabs its entities came from, each loaded once (the links'
    // asset handles), and og.PrefabLink registered: before anything spawns.
    let mut prefab_assets = Vec::with_capacity(scene.prefabs.len());
    for id in &scene.prefabs {
        prefab_assets.push(if *id == scene.id {
            load_scene(eng, id)?
        } else {
            crate::prefab::load(eng, id).map_err(|r| format!("{}: {r}", scene.id))?
        });
    }
    let link = if scene.prefabs.is_empty() {
        None
    } else {
        let id = crate::prefab::register(eng, eng.world()?)?;
        let layout = eng.state.borrow().layouts.get(crate::prefab::LINK).cloned();
        layout.map(|l| (id, l))
    };
    // A cell's members: og.CellMember, registered with the cell.
    let member = match placement {
        Placement::Cell { cell, .. } => {
            let name = crate::stream::cells::MEMBER;
            crate::lazy::ensure(eng, eng.world()?, name)?;
            let layout = layouts
                .get(name)
                .ok_or_else(|| format!("{name} has no layout"))?;
            let id = eng
                .world()?
                .id_by_name(name)
                .map_err(|e| e.reason().to_string())?;
            let bytes = layout.encode(None, Some(&json!({ "cell": cell.to_bits() as f64 })))?;
            Some((id, bytes))
        }
        Placement::Under(_) => None,
    };
    let cell_layout = resolve(&layouts, crate::stream::cells::CELL).cloned();
    before_structural(eng);
    for i in spawned.len()..end {
        let e = &scene.entities[i];
        let world = eng.world()?;
        let mut components = Vec::new();
        // Its og.PrefabLink, spawned with it (an instance's root gets its
        // own once its id is known, below).
        let origin = e.prefab.zip(link.as_ref());
        if let Some((o, (id, layout))) = origin
            && o.root != i
        {
            let root = spawned.get(o.root).map_or(0, |r| r.to_bits());
            let value = crate::prefab::link_value(prefab_assets[o.prefab], o.node, root);
            components.push((*id, layout.encode(None, Some(&value))?));
        }
        let mut has_transform = false;
        // A `Model` table beside a `model` line: its fields, with the
        // line's asset (`model_table`).
        let mut model_table = None;
        // A cell's top-level entity: placed at the cell's pose after its own.
        let place = match placement {
            Placement::Cell { pose, .. } if e.parent.is_none() => Some(pose),
            _ => None,
        };
        for (t, layout) in &tables {
            if let Some(row) = t.get(i) {
                let mut value = migrate(&t.schema, row, layout, &mut dropped);
                if let (Some(pose), Some(tl)) = (place, &transform)
                    && tl.schema == layout.schema
                {
                    value = crate::stream::cells::placed(pose, &value);
                }
                if e.instance.is_some()
                    && model_layout
                        .as_ref()
                        .is_some_and(|ml| ml.schema == layout.schema)
                {
                    model_table = Some(value);
                    continue;
                }
                let id = world
                    .id_by_name(&layout.schema)
                    .map_err(|e| e.reason().to_string())?;
                components.push((id, layout.encode(None, Some(&value))?));
                has_transform |= transform
                    .as_ref()
                    .is_some_and(|tl| tl.schema == layout.schema);
            }
        }
        if !has_transform && let Some(tl) = &transform {
            let id = world
                .id_by_name(&tl.schema)
                .map_err(|e| e.reason().to_string())?;
            let value = place.map(|pose| crate::stream::cells::placed(pose, &json!({})));
            components.push((id, tl.encode(None, value.as_ref())?));
        }
        if let Some((id, bytes)) = &member {
            components.push((*id, bytes.clone()));
        }
        let parent = e.parent.and_then(|p| spawned.get(p).copied()).or(root);
        if let (Some(p), Some(pl)) = (parent, &parent_layout) {
            let id = world
                .id_by_name(&pl.schema)
                .map_err(|e| e.reason().to_string())?;
            components.push((
                id,
                pl.encode(None, Some(&json!({ "entity": p.to_bits() as f64 })))?,
            ));
        }
        if let Some(model) = &e.instance {
            let index = load_gltf(eng, model)
                .map_err(|r| format!("{}: entity '{}': {r}", scene.id, e.name))?;
            match &model_layout {
                Some(ml) => {
                    let world = eng.world()?;
                    let id = world
                        .id_by_name(&ml.schema)
                        .map_err(|e| e.reason().to_string())?;
                    // The scene's own Model fields, else the load's shadow
                    // flags (GltfDesc).
                    let value = match model_table.take() {
                        Some(mut v) => {
                            v["asset"] = json!(handle(index) as f64);
                            v
                        }
                        None => eng
                            .assets
                            .borrow()
                            .model_shadows(index)
                            .model_fields(handle(index)),
                    };
                    components.push((id, ml.encode(None, Some(&value))?));
                }
                None => dropped.push(format!("{}: og.Model is not registered", e.name)),
            }
        }
        let world = eng.world()?;
        let entity = world
            .spawn_bytes(components)
            .map_err(|e| e.reason().to_string())?;
        if let Some((o, (id, layout))) = origin
            && o.root == i
        {
            let value =
                crate::prefab::link_value(prefab_assets[o.prefab], o.node, entity.to_bits());
            eng.world()?
                .insert_bytes(entity, *id, &layout.encode(None, Some(&value))?)
                .map_err(|e| e.reason().to_string())?;
        }
        if !e.name.is_empty() && !prefab {
            crate::api::set_name(eng, entity, &e.name);
        }
        spawned.push(entity);
        if let Some(model) = &e.instance {
            let index = load_gltf(eng, model)?;
            spawn_lights(eng, index, entity)
                .map_err(|r| format!("{}: entity '{}': {r}", scene.id, e.name))?;
        }
    }
    if spawned.len() < scene.entities.len() {
        return Ok(());
    }
    // Sub-scenes, once the scene's own entities are in: a cell's handle
    // set for it to load; any other spawned under its entity now.
    for (i, e) in scene.entities.iter().enumerate() {
        let Some(sub) = &e.subscene else { continue };
        let entity = spawned[i];
        let is_cell = tables.iter().any(|(t, l)| {
            t.get(i).is_some() && cell_layout.as_ref().is_some_and(|cl| cl.schema == l.schema)
        });
        let index =
            load_scene(eng, sub).map_err(|r| format!("{}: sub-scene {sub}: {r}", scene.id))?;
        if is_cell {
            crate::stream::cells::set_scene(eng, entity, handle(index))?;
            continue;
        }
        // Whether its pack has landed or not: only a cell loads a cell
        // group's sub-scene.
        if let Some(group) = eng.assets.borrow().stream.group_of_scene(index) {
            return Err(format!(
                "{}: entity '{}': sub-scene {sub} is in cell group '{group}', which only a cell loads; give the entity an og.StreamCell",
                scene.id, e.name
            ));
        }
        let desc = match &eng.assets.borrow().entries[index].payload {
            Payload::Scene(s) => s.clone(),
            _ => return Err(format!("{}: sub-scene {sub} is not a scene", scene.id)),
        };
        spawn_placed(eng, &desc, Placement::Under(Some(entity)))?;
    }
    apply_resources(eng, scene, &layouts, &mut dropped)?;
    if !dropped.is_empty() {
        eng.assets.borrow_mut().warnings.push(format!(
            "{}: dropped {}",
            scene.id,
            dropped.join("; ")
        ));
    }
    crate::baked::scene_spawned(eng, scene, spawned);
    eng.state
        .borrow_mut()
        .scene_instances
        .push(crate::engine::SceneInstance {
            id: scene.id.clone(),
            root: match placement {
                Placement::Under(root) => root.map(Entity::to_bits),
                Placement::Cell { cell, .. } => Some(cell.to_bits()),
            },
            entities: spawned.iter().map(|e| e.to_bits()).collect(),
            lines: scene.entities.iter().map(|e| e.line).collect(),
        });
    Ok(())
}

/// A scene's `resource` blocks: each field it lists set on the world's
/// resource of that name (the others keep their values), migrated by field
/// name to the registered layout.
fn apply_resources(
    eng: &Engine,
    scene: &SceneDesc,
    layouts: &BTreeMap<String, Layout>,
    dropped: &mut Vec<String>,
) -> Result<(), String> {
    for r in &scene.resources {
        let Some(layout) = layouts
            .get(&r.name)
            .or_else(|| layouts.get(&format!("og.{}", r.name)))
            .filter(|l| l.kind == crate::codec::RecordKind::Resource)
        else {
            dropped.push(format!(
                "resource {} (no registered resource of that name; list it in GameDesc.resources)",
                r.name
            ));
            continue;
        };
        let value = migrate(&r.schema, &r.bytes, layout, dropped);
        let world = eng.world()?;
        // A lazy resource (og.Lighting2D) a scene sets is registered then.
        crate::lazy::ensure(eng, world, &layout.schema)?;
        let id = world
            .id_by_name(&layout.schema)
            .map_err(|e| e.reason().to_string())?;
        let base = world.resource_bytes(id).ok();
        let bytes = layout.encode(base.as_deref(), Some(&value))?;
        world
            .insert_resource_bytes(id, &bytes)
            .map_err(|e| e.reason().to_string())?;
    }
    Ok(())
}
