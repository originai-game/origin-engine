//! Probe entity ids: an entity's name, or `#<index>.<generation>` for an
//! unnamed one (IDL `probe`: "Probe entity ids are entity names").

use og_ecs::{Entity, World};
use std::collections::BTreeMap;

/// The names the probe and the command channel address entities by.
///
/// Names belong to the host's spawn path (`SpawnDesc.name`, scene entity
/// names); the host records them here. A name maps to one entity; naming
/// another entity with it moves it. A dead entity's name resolves to nothing,
/// and resolves again if a restore brings the entity back.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Names {
    by_name: BTreeMap<String, Entity>,
    /// Naming order, which `state().entities` reports.
    order: Vec<String>,
}

impl Names {
    /// Name an entity. Refused (`false`) for an empty name, one starting
    /// with `#`, or one holding `:`, `;`, `=` or whitespace, which the
    /// command grammar could not address.
    pub fn set(&mut self, entity: Entity, name: &str) -> bool {
        let addressable = !name.is_empty()
            && !name.starts_with('#')
            && !name
                .chars()
                .any(|c| c == ':' || c == ';' || c == '=' || c.is_whitespace());
        if !addressable {
            return false;
        }
        self.by_name.retain(|n, e| *e != entity || n == name);
        self.order.retain(|n| self.by_name.contains_key(n));
        if self.by_name.insert(name.to_owned(), entity).is_none() {
            self.order.push(name.to_owned());
        }
        true
    }

    /// Forget a name.
    pub fn remove(&mut self, name: &str) {
        self.by_name.remove(name);
        self.order.retain(|n| n != name);
    }

    /// The live entity a probe id names.
    pub fn resolve(&self, world: &World, id: &str) -> Option<Entity> {
        let entity = match id.strip_prefix('#') {
            Some(rest) => {
                let (index, generation) = rest.split_once('.')?;
                let index: u32 = index.parse().ok()?;
                let generation: u32 = generation.parse().ok()?;
                Entity::from_bits((u64::from(generation) << 32) | u64::from(index))
            }
            None => *self.by_name.get(id)?,
        };
        world.is_alive(entity).then_some(entity)
    }

    /// The name of an entity, if it has one.
    pub fn name_of(&self, entity: Entity) -> Option<&str> {
        self.by_name
            .iter()
            .find(|(_, e)| **e == entity)
            .map(|(n, _)| n.as_str())
    }

    /// The probe id of an entity: its name, or `#<index>.<generation>`.
    pub fn id_of(&self, entity: Entity) -> String {
        match self.name_of(entity) {
            Some(name) => name.to_owned(),
            None => format!("#{}.{}", entity.index(), entity.generation()),
        }
    }

    /// Named live entities, in naming order.
    pub fn live(&self, world: &World) -> Vec<(String, Entity)> {
        self.order
            .iter()
            .filter_map(|n| {
                let e = self.by_name[n];
                world.is_alive(e).then(|| (n.clone(), e))
            })
            .collect()
    }
}
