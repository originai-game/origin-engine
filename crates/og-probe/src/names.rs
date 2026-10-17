//! Probe entity ids: an entity's name, or `#<index>.<generation>` for an
//! unnamed one (IDL `probe`: "Probe entity ids are entity names").
//!
//! Names live in the world (`World::set_name`, og-ecs §Names): the host names
//! entities as it spawns them, names are in snapshots and the world hash, a
//! restore brings them back, and despawning takes the name with it. This is
//! the probe's read side.

use og_ecs::{Entity, World};

/// Resolves probe ids against the world's names.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Names;

impl Names {
    /// The live entity a probe id names.
    pub fn resolve(&self, world: &World, id: &str) -> Option<Entity> {
        let entity = match id.strip_prefix('#') {
            Some(rest) => {
                let (index, generation) = rest.split_once('.')?;
                let index: u32 = index.parse().ok()?;
                let generation: u32 = generation.parse().ok()?;
                Entity::from_bits((u64::from(generation) << 32) | u64::from(index))
            }
            None => world.find(id)?,
        };
        world.is_alive(entity).then_some(entity)
    }

    /// The name of an entity, if it has one.
    pub fn name_of<'w>(&self, world: &'w World, entity: Entity) -> Option<&'w str> {
        world.name(entity)
    }

    /// The probe id of an entity: its name, or `#<index>.<generation>`.
    pub fn id_of(&self, world: &World, entity: Entity) -> String {
        match world.name(entity) {
            Some(name) => name.to_owned(),
            None => format!("#{}.{}", entity.index(), entity.generation()),
        }
    }

    /// Named live entities, in entity order.
    pub fn live(&self, world: &World) -> Vec<(String, Entity)> {
        let mut out: Vec<(String, Entity)> = world
            .names()
            .into_iter()
            .filter(|(e, _)| world.is_alive(*e))
            .map(|(e, n)| (n.to_owned(), e))
            .collect();
        out.sort_by_key(|(_, e)| (e.index(), e.generation()));
        out
    }
}
