//! A module state block's per-entity form: what a comparison of two worlds
//! reads of a block whose saved bytes hold entity ids (og-ecs's
//! `StateBlock::form`).
//!
//! A block's bytes are its module's own format, keyed by entity id and by
//! the module's internal handles. Two worlds that hold the same game with
//! different ids (a restart that despawned everything and spawned it
//! again) save different bytes for the same state. The block's form names
//! that state by entity instead: one [`Record`] per entity and thing it
//! holds (`body`, `collider`, `layer 0`), each with its fields by name, an
//! entity reference as [`Field::Entity`] so the comparison reads it through
//! its pairing of the two worlds' entities, a tick on the world's clock as
//! [`Field::Tick`] or [`Field::Until`] so it compares relative to each
//! world's current tick, and the state that is no entity's as
//! [`Records::global`]. A module's own registry handle (a nav grid, a
//! behaviour) is given as the thing it names, never its handle's number.

use serde::Serialize;
use serde_json::Value;

/// How a block's saved bytes compare between worlds.
#[derive(Clone, Debug, PartialEq)]
pub enum BlockForm {
    /// The block holds no entity id: compared byte for byte.
    Global,
    /// The block's state by entity, and what is no entity's.
    Entities(Records),
}

/// A block's state: per entity, and the rest.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Records {
    /// What is no entity's (a gravity, a queue's length), by name.
    pub global: Vec<(String, Field)>,
    /// One record per entity and thing it holds, in the block's order.
    pub entities: Vec<Record>,
}

/// One thing a block holds for one entity.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    /// The entity's id bits.
    pub entity: u64,
    /// What the record is (`body`, `collider`, `layer`, `path`).
    pub kind: String,
    /// Which one of the entity's records of this kind: [`Value::Null`] for
    /// its only one, an index (`layer` 2), or an entity (a contact with it).
    pub key: Field,
    /// Its state by field name.
    pub fields: Vec<(String, Field)>,
}

/// One field of a record.
#[derive(Clone, Debug, PartialEq)]
pub enum Field {
    /// An entity's id bits (0: none).
    Entity(u64),
    /// A list of entities' id bits.
    Entities(Vec<u64>),
    /// A tick in the past on og.Time's clock (`state_since`, last seen),
    /// compared by how long ago it was in each world's own snapshot: a
    /// restart does not turn the clock back. 0 is never.
    Tick(u64),
    /// A deadline tick on og.Time's clock (a cooldown's end), compared by
    /// the ticks left until it in each world's snapshot, none once passed.
    /// 0 is none.
    Until(u64),
    /// Any other value, as JSON (numbers exact, as the module's).
    Value(Value),
}

impl Field {
    /// `value` serialized as JSON.
    pub fn of(value: &impl Serialize) -> Field {
        Field::Value(serde_json::to_value(value).unwrap_or(Value::Null))
    }
}

/// The members of `value` serialized (a struct or a map with text keys) as
/// fields, those named in `entities` as entity ids: a number (or `null`,
/// none) is one, a list of numbers is a list. A value that is not an object
/// is one field named `value`.
pub fn members(value: &impl Serialize, entities: &[&str]) -> Vec<(String, Field)> {
    let as_entity = |v: &Value| -> Option<Field> {
        match v {
            Value::Null => Some(Field::Entity(0)),
            Value::Number(n) => n.as_u64().map(Field::Entity),
            Value::Array(list) => list
                .iter()
                .map(Value::as_u64)
                .collect::<Option<Vec<u64>>>()
                .map(Field::Entities),
            _ => None,
        }
    };
    match serde_json::to_value(value).unwrap_or(Value::Null) {
        Value::Object(map) => map
            .into_iter()
            .map(|(k, v)| {
                let f = if entities.contains(&k.as_str()) {
                    as_entity(&v).unwrap_or(Field::Value(v))
                } else {
                    Field::Value(v)
                };
                (k, f)
            })
            .collect(),
        other => vec![("value".to_owned(), Field::Value(other))],
    }
}

impl Record {
    /// `entity`'s only record of `kind`, with no fields yet.
    pub fn new(entity: u64, kind: &str) -> Record {
        Record {
            entity,
            kind: kind.to_owned(),
            key: Field::Value(Value::Null),
            fields: Vec::new(),
        }
    }

    /// Which of the entity's records of this kind it is.
    pub fn key(mut self, key: Field) -> Record {
        self.key = key;
        self
    }

    /// Adds one field.
    pub fn field(mut self, name: &str, field: Field) -> Record {
        self.fields.push((name.to_owned(), field));
        self
    }

    /// Adds `value`'s members ([`members`]), those named in `entities` as entities.
    pub fn members(mut self, value: &impl Serialize, entities: &[&str]) -> Record {
        self.fields.extend(members(value, entities));
        self
    }
}
