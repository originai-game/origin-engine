//! Runtime author provenance and in-memory prefab reverts. Source persistence
//! belongs to the tool's checked OXC transaction, never a runtime text writer.
use crate::engine::{Engine, engine};
use crate::world::assets::{self, Payload};
use og_ecs::{Entity, World};
use og_probe::{PrefabEdit, SceneAuthor};
use serde_json::{Value, json};

pub(crate) struct RuntimeAuthor {
    pub engine: u32,
}

fn placement(eng: &Engine, entity: Entity) -> Option<(usize, usize)> {
    let state = eng.state.borrow();
    let (id, row) = state.scene_instances.iter().find_map(|s| {
        s.entities
            .iter()
            .position(|e| *e == entity.to_bits())
            .map(|row| (&s.id, row))
    })?;
    eng.assets
        .borrow()
        .entries
        .iter()
        .enumerate()
        .find_map(|(i, e)| {
            matches!(&e.payload, Payload::Scene(s) if s.id == *id).then_some((i, row))
        })
}

fn link(eng: &Engine, world: &World, entity: Entity) -> Option<(usize, usize, Entity)> {
    let layout = eng
        .state
        .borrow()
        .layouts
        .get(crate::world::prefab::LINK)?
        .clone();
    let sid = world.id_by_name(crate::world::prefab::LINK).ok()?;
    let bytes = world.get_bytes(entity, sid).ok()?;
    let value = layout.decode_kept(bytes);
    Some((
        assets::index_of(value["prefab"].as_f64()?)?,
        value["node"].as_f64()? as usize,
        Entity::from_bits(value["root"].as_f64()? as u64),
    ))
}

fn node_path(scene: &og_bundle::runtime::SceneDesc, node: usize) -> Option<String> {
    let mut names = Vec::new();
    let mut at = Some(node);
    while let Some(i) = at {
        let e = scene.entities.get(i)?;
        if e.parent.is_some() {
            names.push(e.name.clone());
        }
        at = e.parent;
    }
    names.reverse();
    Some(names.join("/"))
}

impl SceneAuthor for RuntimeAuthor {
    fn authored_target(&self, _: &World, entity: Entity, parameter: &str) -> Option<Value> {
        let eng = engine(self.engine).ok()?;
        let (index, row) = placement(&eng, entity)?;
        let assets = eng.assets.borrow();
        let Payload::Scene(scene) = &assets.entries[index].payload else {
            return None;
        };
        let (component, field) = parameter.rsplit_once('.')?;
        let state = eng.state.borrow();
        // Use the same registered-name resolution as scene spawning. An
        // author's `Transform` and the runtime's `og.Transform` are one type.
        let authored_component = scene.sources.get(row)?.keys().find_map(|key| {
            let (name, f) = key.rsplit_once('.')?;
            let layout = state
                .layouts
                .get(name)
                .or_else(|| state.layouts.get(&format!("og.{name}")))?;
            (f == field && layout.schema == component).then_some(name)
        })?;
        let mut names = Vec::new();
        let mut at = Some(row);
        while let Some(i) = at {
            let node = scene.entities.get(i)?;
            names.push(node.name.as_str());
            at = node.parent;
        }
        names.reverse();
        Some(json!({
            "scene": scene.id,
            "entity": names.join("/"),
            "component": authored_component,
            "field": field,
        }))
    }

    fn source(&self, _: &World, entity: Entity) -> Option<String> {
        let eng = engine(self.engine).ok()?;
        if let Some((index, row)) = placement(&eng, entity) {
            let assets = eng.assets.borrow();
            let Payload::Scene(scene) = &assets.entries[index].payload else {
                return None;
            };
            let file = scene.origins.get(row).filter(|f| !f.is_empty())?;
            return Some(match scene.entities[row].line {
                Some(line) => format!("{file}:{line}"),
                None => file.clone(),
            });
        }
        eng.state
            .borrow()
            .game_spawned
            .get(&entity.to_bits())?
            .as_deref()
            .map(Into::into)
    }

    fn prefab_id(&self, handle: u64) -> Option<String> {
        let eng = engine(self.engine).ok()?;
        let index = assets::index_of(handle as f64)?;
        eng.assets
            .borrow()
            .entries
            .get(index)
            .map(|e| og_bundle::runtime::asset_id(&e.path).into())
    }

    fn entity_prefab(&self, world: &World, entity: Entity) -> Option<Value> {
        let eng = engine(self.engine).ok()?;
        let (index, node, root) = link(&eng, world, entity)?;
        let assets = eng.assets.borrow();
        let Payload::Scene(prefab) = &assets.entries.get(index)?.payload else {
            return None;
        };
        let overrides = placement(&eng, entity)
            .and_then(|(i, row)| {
                let Payload::Scene(scene) = &assets.entries[i].payload else {
                    return None;
                };
                scene.overrides.get(&row).cloned()
            })
            .unwrap_or_default();
        Some(
            json!({"prefab":prefab.id,"node":node_path(prefab,node)?,"root":root.to_bits(),"overrides":overrides}),
        )
    }

    fn prefab(&mut self, world: &mut World, edit: &PrefabEdit) -> Result<(), String> {
        let eng = engine(self.engine)?;
        let (index, node, _) =
            link(&eng, world, edit.entity).ok_or("entity was not placed from a prefab")?;
        let (tables, bindings) = {
            let assets = eng.assets.borrow();
            let Payload::Scene(scene) = &assets.entries[index].payload else {
                return Err("prefab is not loaded".into());
            };
            (
                scene
                    .components
                    .iter()
                    .filter_map(|t| {
                        t.get(node)
                            .map(|row| (t.name.clone(), t.schema.clone(), row.to_vec()))
                    })
                    .collect::<Vec<_>>(),
                scene.bindings.get(&node).cloned().unwrap_or_default(),
            )
        };
        let layouts = eng.state.borrow().layouts.clone();
        for (name, schema, row) in tables {
            let Some(layout) = layouts.get(&name) else {
                continue;
            };
            let mut value = assets::migrate(&schema, &row, layout, &mut Vec::new());
            if let Some(fields) = bindings.get(&name) {
                assets::bind_fields(&eng, world, layout, fields, &mut value)?;
            }
            let selected = |field: &str| {
                edit.target.as_ref().is_none_or(|t| {
                    t.strip_prefix("og.").unwrap_or(t) == name.strip_prefix("og.").unwrap_or(&name)
                        || t.strip_prefix("og.").unwrap_or(t)
                            == format!("{}.{}", name.strip_prefix("og.").unwrap_or(&name), field)
                })
            };
            value
                .as_object_mut()
                .expect("record object")
                .retain(|field, _| selected(field));
            if value.as_object().is_none_or(|v| v.is_empty()) {
                continue;
            }
            let sid = world
                .id_by_name(&layout.schema)
                .map_err(|e| e.reason().to_string())?;
            let now = world
                .get_bytes(edit.entity, sid)
                .map_err(|e| e.reason().to_string())?;
            let bytes = layout.encode(Some(&now), Some(&value))?;
            world
                .insert_bytes(edit.entity, sid, &bytes)
                .map_err(|e| e.reason().to_string())?;
        }
        Ok(())
    }
}
