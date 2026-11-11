// Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate.
//! The typed wire types, generated from the IDL: every struct, enum and
//! union a binding takes or answers, with the IDL's defaults, a reader
//! (`FromJson`, src/wire.rs) that refuses a value of the wrong type and a
//! writer (`ToJson`) of the IDL's fields. See
//! og-idl's `gen/wire.rs` for the rules. The IDL's prose is not repeated:
//! each type names its wiki page.

#![allow(clippy::derivable_impls, clippy::approx_constant, clippy::excessive_precision, clippy::large_enum_variant, clippy::unreadable_literal, clippy::type_complexity)]
// Which imports are used depends on what the IDL declares.
#![allow(unused_imports)]

use super::{FromJson, OneOf, ToJson, at, field, object, text, unknown_variant, variant};
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// `Color`: packages/knowledge/wiki/math.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: Default::default(),
            g: Default::default(),
            b: Default::default(),
        }
    }
}

impl FromJson for Color {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            r: field(o, "r", d.r)?,
            g: field(o, "g", d.g)?,
            b: field(o, "b", d.b)?,
        })
    }
}

impl ToJson for Color {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("r".into(), self.r.to_json());
        o.insert("g".into(), self.g.to_json());
        o.insert("b".into(), self.b.to_json());
        Value::Object(o)
    }
}

/// `Ray`: packages/knowledge/wiki/math.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Default for Ray {
    fn default() -> Self {
        Self {
            origin: Default::default(),
            direction: Default::default(),
        }
    }
}

impl FromJson for Ray {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            origin: field(o, "origin", d.origin)?,
            direction: field(o, "direction", d.direction)?,
        })
    }
}

impl ToJson for Ray {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("origin".into(), self.origin.to_json());
        o.insert("direction".into(), self.direction.to_json());
        Value::Object(o)
    }
}

/// `SpawnDesc`: packages/knowledge/wiki/world.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SpawnDesc {
    pub name: String,
    pub parent: Option<u64>,
    pub position: Vec3,
    pub quaternion: Quat,
    pub scale: Vec3,
    pub bare: bool,
    pub source: String,
}

impl Default for SpawnDesc {
    fn default() -> Self {
        Self {
            name: Default::default(),
            parent: Default::default(),
            position: Default::default(),
            quaternion: Default::default(),
            scale: Vec3::new(1.0_f32, 1.0_f32, 1.0_f32),
            bare: Default::default(),
            source: Default::default(),
        }
    }
}

impl FromJson for SpawnDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            parent: field(o, "parent", d.parent)?,
            position: field(o, "position", d.position)?,
            quaternion: field(o, "quaternion", d.quaternion)?,
            scale: field(o, "scale", d.scale)?,
            bare: field(o, "bare", d.bare)?,
            source: field(o, "source", d.source)?,
        })
    }
}

impl ToJson for SpawnDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("parent".into(), self.parent.to_json());
        o.insert("position".into(), self.position.to_json());
        o.insert("quaternion".into(), self.quaternion.to_json());
        o.insert("scale".into(), self.scale.to_json());
        o.insert("bare".into(), self.bare.to_json());
        o.insert("source".into(), self.source.to_json());
        Value::Object(o)
    }
}

/// `RestoreDesc`: packages/knowledge/wiki/world.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RestoreDesc {
    pub migrate: bool,
    pub session: bool,
}

impl Default for RestoreDesc {
    fn default() -> Self {
        Self {
            migrate: Default::default(),
            session: Default::default(),
        }
    }
}

impl FromJson for RestoreDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            migrate: field(o, "migrate", d.migrate)?,
            session: field(o, "session", d.session)?,
        })
    }
}

impl ToJson for RestoreDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("migrate".into(), self.migrate.to_json());
        o.insert("session".into(), self.session.to_json());
        Value::Object(o)
    }
}

/// `SnapshotDesc`: packages/knowledge/wiki/world.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotDesc {
    pub with: Vec<Value>,
    pub resources: Vec<Value>,
}

impl Default for SnapshotDesc {
    fn default() -> Self {
        Self {
            with: Default::default(),
            resources: Default::default(),
        }
    }
}

impl FromJson for SnapshotDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            with: field(o, "with", d.with)?,
            resources: field(o, "resources", d.resources)?,
        })
    }
}

impl ToJson for SnapshotDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("with".into(), self.with.to_json());
        o.insert("resources".into(), self.resources.to_json());
        Value::Object(o)
    }
}

/// `SaveType`: packages/knowledge/wiki/world.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SaveType {
    pub name: String,
    pub version: u32,
    pub fields: Vec<String>,
}

impl Default for SaveType {
    fn default() -> Self {
        Self {
            name: Default::default(),
            version: Default::default(),
            fields: Default::default(),
        }
    }
}

impl FromJson for SaveType {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            version: field(o, "version", d.version)?,
            fields: field(o, "fields", d.fields)?,
        })
    }
}

impl ToJson for SaveType {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("version".into(), self.version.to_json());
        o.insert("fields".into(), self.fields.to_json());
        Value::Object(o)
    }
}

/// `SaveRecord`: packages/knowledge/wiki/world.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SaveRecord {
    pub entity: Option<u64>,
    pub value: Value,
}

impl Default for SaveRecord {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            value: Default::default(),
        }
    }
}

impl FromJson for SaveRecord {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            value: field(o, "value", d.value)?,
        })
    }
}

impl ToJson for SaveRecord {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("value".into(), self.value.to_json());
        Value::Object(o)
    }
}

/// `SetRecordsDesc`: packages/knowledge/wiki/world.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SetRecordsDesc {
    pub into: String,
    pub records: Vec<SaveRecord>,
}

impl Default for SetRecordsDesc {
    fn default() -> Self {
        Self {
            into: Default::default(),
            records: Default::default(),
        }
    }
}

impl FromJson for SetRecordsDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            into: field(o, "into", d.into)?,
            records: field(o, "records", d.records)?,
        })
    }
}

impl ToJson for SetRecordsDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("into".into(), self.into.to_json());
        o.insert("records".into(), self.records.to_json());
        Value::Object(o)
    }
}

/// `SaveSpawnDesc`: packages/knowledge/wiki/world.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SaveSpawnDesc {
    pub name: String,
    pub components: Value,
}

impl Default for SaveSpawnDesc {
    fn default() -> Self {
        Self {
            name: Default::default(),
            components: Default::default(),
        }
    }
}

impl FromJson for SaveSpawnDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            components: field(o, "components", d.components)?,
        })
    }
}

impl ToJson for SaveSpawnDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("components".into(), self.components.to_json());
        Value::Object(o)
    }
}

/// `SaveMigration`: packages/knowledge/wiki/world.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SaveMigration {
    pub from: u32,
    pub migrate: u32,
}

impl Default for SaveMigration {
    fn default() -> Self {
        Self {
            from: Default::default(),
            migrate: Default::default(),
        }
    }
}

impl FromJson for SaveMigration {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            from: field(o, "from", d.from)?,
            migrate: field(o, "migrate", d.migrate)?,
        })
    }
}

impl ToJson for SaveMigration {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("from".into(), self.from.to_json());
        o.insert("migrate".into(), self.migrate.to_json());
        Value::Object(o)
    }
}

/// `QueryDesc`: packages/knowledge/wiki/world.md.
#[derive(Clone, Debug, PartialEq)]
pub struct QueryDesc {
    pub with: Vec<Value>,
    pub without: Vec<Value>,
    pub changed: Vec<Value>,
    pub added: Vec<Value>,
    pub removed: Vec<Value>,
}

impl Default for QueryDesc {
    fn default() -> Self {
        Self {
            with: Default::default(),
            without: Default::default(),
            changed: Default::default(),
            added: Default::default(),
            removed: Default::default(),
        }
    }
}

impl FromJson for QueryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            with: field(o, "with", d.with)?,
            without: field(o, "without", d.without)?,
            changed: field(o, "changed", d.changed)?,
            added: field(o, "added", d.added)?,
            removed: field(o, "removed", d.removed)?,
        })
    }
}

impl ToJson for QueryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("with".into(), self.with.to_json());
        o.insert("without".into(), self.without.to_json());
        o.insert("changed".into(), self.changed.to_json());
        o.insert("added".into(), self.added.to_json());
        o.insert("removed".into(), self.removed.to_json());
        Value::Object(o)
    }
}

/// `SpatialDesc`: packages/knowledge/wiki/world.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SpatialDesc {
    pub with: Vec<Value>,
    pub without: Vec<Value>,
    pub cell: f32,
}

impl Default for SpatialDesc {
    fn default() -> Self {
        Self {
            with: Default::default(),
            without: Default::default(),
            cell: Default::default(),
        }
    }
}

impl FromJson for SpatialDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            with: field(o, "with", d.with)?,
            without: field(o, "without", d.without)?,
            cell: field(o, "cell", d.cell)?,
        })
    }
}

impl ToJson for SpatialDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("with".into(), self.with.to_json());
        o.insert("without".into(), self.without.to_json());
        o.insert("cell".into(), self.cell.to_json());
        Value::Object(o)
    }
}

/// `Neighbour`: packages/knowledge/wiki/world.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Neighbour {
    pub entity: u64,
    pub distance: f64,
}

impl Default for Neighbour {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            distance: Default::default(),
        }
    }
}

impl FromJson for Neighbour {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            distance: field(o, "distance", d.distance)?,
        })
    }
}

impl ToJson for Neighbour {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("distance".into(), self.distance.to_json());
        Value::Object(o)
    }
}

/// `StateWhen`: packages/knowledge/wiki/system.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StateWhen {
    #[default]
    During,
    Enter,
    Exit,
}

impl StateWhen {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["during", "enter", "exit"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::During, Self::Enter, Self::Exit];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("StateWhen", s, Self::NAMES)),
        }
    }
}

impl FromJson for StateWhen {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "StateWhen", Self::NAMES)?)
    }
}

impl ToJson for StateWhen {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `StateDesc`: packages/knowledge/wiki/system.md.
#[derive(Clone, Debug, PartialEq)]
pub struct StateDesc {
    pub name: String,
    pub values: Vec<String>,
    pub initial: Option<String>,
}

impl Default for StateDesc {
    fn default() -> Self {
        Self {
            name: Default::default(),
            values: Default::default(),
            initial: Default::default(),
        }
    }
}

impl FromJson for StateDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            values: field(o, "values", d.values)?,
            initial: field(o, "initial", d.initial)?,
        })
    }
}

impl ToJson for StateDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("values".into(), self.values.to_json());
        if let Some(x) = &self.initial {
            o.insert("initial".into(), x.to_json());
        }
        Value::Object(o)
    }
}

/// `SystemDesc`: packages/knowledge/wiki/system.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SystemDesc {
    pub reads: Vec<OneOf<Value, Value>>,
    pub writes: Vec<OneOf<Value, Value>>,
    pub without: Vec<Value>,
    pub after: Vec<String>,
    pub before: Vec<String>,
    pub changed: Vec<Value>,
    pub added: Vec<Value>,
    pub removed: Vec<Value>,
    pub state: String,
    pub value: String,
    pub when: StateWhen,
    pub run_if: Option<u32>,
}

impl Default for SystemDesc {
    fn default() -> Self {
        Self {
            reads: Default::default(),
            writes: Default::default(),
            without: Default::default(),
            after: Default::default(),
            before: Default::default(),
            changed: Default::default(),
            added: Default::default(),
            removed: Default::default(),
            state: Default::default(),
            value: Default::default(),
            when: Default::default(),
            run_if: Default::default(),
        }
    }
}

impl FromJson for SystemDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            reads: field(o, "reads", d.reads)?,
            writes: field(o, "writes", d.writes)?,
            without: field(o, "without", d.without)?,
            after: field(o, "after", d.after)?,
            before: field(o, "before", d.before)?,
            changed: field(o, "changed", d.changed)?,
            added: field(o, "added", d.added)?,
            removed: field(o, "removed", d.removed)?,
            state: field(o, "state", d.state)?,
            value: field(o, "value", d.value)?,
            when: field(o, "when", d.when)?,
            run_if: field(o, "runIf", d.run_if)?,
        })
    }
}

impl ToJson for SystemDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("reads".into(), self.reads.to_json());
        o.insert("writes".into(), self.writes.to_json());
        o.insert("without".into(), self.without.to_json());
        o.insert("after".into(), self.after.to_json());
        o.insert("before".into(), self.before.to_json());
        o.insert("changed".into(), self.changed.to_json());
        o.insert("added".into(), self.added.to_json());
        o.insert("removed".into(), self.removed.to_json());
        o.insert("state".into(), self.state.to_json());
        o.insert("value".into(), self.value.to_json());
        o.insert("when".into(), self.when.to_json());
        o.insert("runIf".into(), self.run_if.to_json());
        Value::Object(o)
    }
}

/// `HookEvent`: packages/knowledge/wiki/system.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HookEvent {
    #[default]
    Add,
    Remove,
}

impl HookEvent {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["add", "remove"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Add, Self::Remove];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("HookEvent", s, Self::NAMES)),
        }
    }
}

impl FromJson for HookEvent {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "HookEvent", Self::NAMES)?)
    }
}

impl ToJson for HookEvent {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `HookDesc`: packages/knowledge/wiki/system.md.
#[derive(Clone, Debug, PartialEq)]
pub struct HookDesc {
    pub on: HookEvent,
    pub reads: Vec<OneOf<Value, Value>>,
    pub writes: Vec<OneOf<Value, Value>>,
}

impl Default for HookDesc {
    fn default() -> Self {
        Self {
            on: Default::default(),
            reads: Default::default(),
            writes: Default::default(),
        }
    }
}

impl FromJson for HookDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            on: field(o, "on", d.on)?,
            reads: field(o, "reads", d.reads)?,
            writes: field(o, "writes", d.writes)?,
        })
    }
}

impl ToJson for HookDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("on".into(), self.on.to_json());
        o.insert("reads".into(), self.reads.to_json());
        o.insert("writes".into(), self.writes.to_json());
        Value::Object(o)
    }
}

/// `GameDesc`: packages/knowledge/wiki/game.md.
#[derive(Clone, Debug, PartialEq)]
pub struct GameDesc {
    pub tick: u32,
    pub seed: u32,
    pub scenes: BTreeMap<String, String>,
    pub scene: String,
    pub systems: Vec<OneOf<u32, i32>>,
    pub hooks: Vec<OneOf<u32, i32>>,
    pub states: Vec<StateDesc>,
    pub resources: Vec<Value>,
    pub commands: BTreeMap<String, u32>,
    pub command_help: BTreeMap<String, CommandDesc>,
    pub max_catch_up: u32,
    pub facts: Option<u32>,
    pub start: Option<u32>,
    pub ui: Option<u32>,
    pub render: RenderOptions,
    pub save_version: u32,
    pub migrations: Vec<OneOf<SaveMigration, u32>>,
    pub explore: ExploreDesc,
    pub look: String,
}

impl Default for GameDesc {
    fn default() -> Self {
        Self {
            tick: 60,
            seed: Default::default(),
            scenes: Default::default(),
            scene: String::from("main"),
            systems: Default::default(),
            hooks: Default::default(),
            states: Default::default(),
            resources: Default::default(),
            commands: Default::default(),
            command_help: Default::default(),
            max_catch_up: 4,
            facts: Default::default(),
            start: Default::default(),
            ui: Default::default(),
            render: Default::default(),
            save_version: 1,
            migrations: Default::default(),
            explore: Default::default(),
            look: Default::default(),
        }
    }
}

impl FromJson for GameDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tick: field(o, "tick", d.tick)?,
            seed: field(o, "seed", d.seed)?,
            scenes: field(o, "scenes", d.scenes)?,
            scene: field(o, "scene", d.scene)?,
            systems: field(o, "systems", d.systems)?,
            hooks: field(o, "hooks", d.hooks)?,
            states: field(o, "states", d.states)?,
            resources: field(o, "resources", d.resources)?,
            commands: field(o, "commands", d.commands)?,
            command_help: field(o, "commandHelp", d.command_help)?,
            max_catch_up: field(o, "maxCatchUp", d.max_catch_up)?,
            facts: field(o, "facts", d.facts)?,
            start: field(o, "start", d.start)?,
            ui: field(o, "ui", d.ui)?,
            render: field(o, "render", d.render)?,
            save_version: field(o, "saveVersion", d.save_version)?,
            migrations: field(o, "migrations", d.migrations)?,
            explore: field(o, "explore", d.explore)?,
            look: field(o, "look", d.look)?,
        })
    }
}

impl ToJson for GameDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tick".into(), self.tick.to_json());
        o.insert("seed".into(), self.seed.to_json());
        o.insert("scenes".into(), self.scenes.to_json());
        o.insert("scene".into(), self.scene.to_json());
        o.insert("systems".into(), self.systems.to_json());
        o.insert("hooks".into(), self.hooks.to_json());
        o.insert("states".into(), self.states.to_json());
        o.insert("resources".into(), self.resources.to_json());
        o.insert("commands".into(), self.commands.to_json());
        o.insert("commandHelp".into(), self.command_help.to_json());
        o.insert("maxCatchUp".into(), self.max_catch_up.to_json());
        o.insert("facts".into(), self.facts.to_json());
        o.insert("start".into(), self.start.to_json());
        o.insert("ui".into(), self.ui.to_json());
        o.insert("render".into(), self.render.to_json());
        o.insert("saveVersion".into(), self.save_version.to_json());
        o.insert("migrations".into(), self.migrations.to_json());
        o.insert("explore".into(), self.explore.to_json());
        o.insert("look".into(), self.look.to_json());
        Value::Object(o)
    }
}

/// `ExploreDesc`: packages/knowledge/wiki/game.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ExploreDesc {
    pub cells: Vec<String>,
    pub ignore: Vec<String>,
    pub actions: Vec<String>,
    pub intents: Vec<ExploreIntent>,
}

impl Default for ExploreDesc {
    fn default() -> Self {
        Self {
            cells: Default::default(),
            ignore: Default::default(),
            actions: Default::default(),
            intents: Default::default(),
        }
    }
}

impl FromJson for ExploreDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            cells: field(o, "cells", d.cells)?,
            ignore: field(o, "ignore", d.ignore)?,
            actions: field(o, "actions", d.actions)?,
            intents: field(o, "intents", d.intents)?,
        })
    }
}

impl ToJson for ExploreDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("cells".into(), self.cells.to_json());
        o.insert("ignore".into(), self.ignore.to_json());
        o.insert("actions".into(), self.actions.to_json());
        o.insert("intents".into(), self.intents.to_json());
        Value::Object(o)
    }
}

/// `ExploreIntent`: packages/knowledge/wiki/game.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ExploreIntent {
    pub event: String,
    pub values: Vec<Value>,
}

impl Default for ExploreIntent {
    fn default() -> Self {
        Self {
            event: Default::default(),
            values: Default::default(),
        }
    }
}

impl FromJson for ExploreIntent {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            event: field(o, "event", d.event)?,
            values: field(o, "values", d.values)?,
        })
    }
}

impl ToJson for ExploreIntent {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("event".into(), self.event.to_json());
        o.insert("values".into(), self.values.to_json());
        Value::Object(o)
    }
}

/// `NoiseKind`: packages/knowledge/wiki/procgen.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NoiseKind {
    #[default]
    Perlin,
    Simplex,
}

impl NoiseKind {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["perlin", "simplex"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Perlin, Self::Simplex];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("NoiseKind", s, Self::NAMES)),
        }
    }
}

impl FromJson for NoiseKind {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "NoiseKind", Self::NAMES)?)
    }
}

impl ToJson for NoiseKind {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `FbmDesc`: packages/knowledge/wiki/procgen.md.
#[derive(Clone, Debug, PartialEq)]
pub struct FbmDesc {
    pub seed: u32,
    pub kind: NoiseKind,
    pub octaves: u32,
    pub frequency: f32,
    pub lacunarity: f32,
    pub gain: f32,
    pub three_d: bool,
}

impl Default for FbmDesc {
    fn default() -> Self {
        Self {
            seed: Default::default(),
            kind: Default::default(),
            octaves: 1,
            frequency: 1.0_f32,
            lacunarity: 2.0_f32,
            gain: 0.5_f32,
            three_d: Default::default(),
        }
    }
}

impl FromJson for FbmDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            seed: field(o, "seed", d.seed)?,
            kind: field(o, "kind", d.kind)?,
            octaves: field(o, "octaves", d.octaves)?,
            frequency: field(o, "frequency", d.frequency)?,
            lacunarity: field(o, "lacunarity", d.lacunarity)?,
            gain: field(o, "gain", d.gain)?,
            three_d: field(o, "threeD", d.three_d)?,
        })
    }
}

impl ToJson for FbmDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("seed".into(), self.seed.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("octaves".into(), self.octaves.to_json());
        o.insert("frequency".into(), self.frequency.to_json());
        o.insert("lacunarity".into(), self.lacunarity.to_json());
        o.insert("gain".into(), self.gain.to_json());
        o.insert("threeD".into(), self.three_d.to_json());
        Value::Object(o)
    }
}

/// `NoiseGridDesc`: packages/knowledge/wiki/procgen.md.
#[derive(Clone, Debug, PartialEq)]
pub struct NoiseGridDesc {
    pub noise: FbmDesc,
    pub width: u32,
    pub height: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub step: f32,
}

impl Default for NoiseGridDesc {
    fn default() -> Self {
        Self {
            noise: Default::default(),
            width: 64,
            height: 64,
            x: Default::default(),
            y: Default::default(),
            z: Default::default(),
            step: 1.0_f32,
        }
    }
}

impl FromJson for NoiseGridDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            noise: field(o, "noise", d.noise)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            z: field(o, "z", d.z)?,
            step: field(o, "step", d.step)?,
        })
    }
}

impl ToJson for NoiseGridDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("noise".into(), self.noise.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("z".into(), self.z.to_json());
        o.insert("step".into(), self.step.to_json());
        Value::Object(o)
    }
}

/// `PoissonDesc`: packages/knowledge/wiki/procgen.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PoissonDesc {
    pub seed: u32,
    pub width: f32,
    pub height: f32,
    pub radius: f32,
    pub attempts: u32,
    pub max_points: u32,
}

impl Default for PoissonDesc {
    fn default() -> Self {
        Self {
            seed: Default::default(),
            width: Default::default(),
            height: Default::default(),
            radius: 1.0_f32,
            attempts: 30,
            max_points: 4096,
        }
    }
}

impl FromJson for PoissonDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            seed: field(o, "seed", d.seed)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            radius: field(o, "radius", d.radius)?,
            attempts: field(o, "attempts", d.attempts)?,
            max_points: field(o, "maxPoints", d.max_points)?,
        })
    }
}

impl ToJson for PoissonDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("seed".into(), self.seed.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("radius".into(), self.radius.to_json());
        o.insert("attempts".into(), self.attempts.to_json());
        o.insert("maxPoints".into(), self.max_points.to_json());
        Value::Object(o)
    }
}

/// `CellRect`: packages/knowledge/wiki/procgen.md.
#[derive(Clone, Debug, PartialEq)]
pub struct CellRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Default for CellRect {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            width: Default::default(),
            height: Default::default(),
        }
    }
}

impl FromJson for CellRect {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
        })
    }
}

impl ToJson for CellRect {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        Value::Object(o)
    }
}

/// `BspDesc`: packages/knowledge/wiki/procgen.md.
#[derive(Clone, Debug, PartialEq)]
pub struct BspDesc {
    pub seed: u32,
    pub width: u32,
    pub height: u32,
    pub min_room: u32,
    pub max_room: u32,
    pub depth: u32,
    pub padding: u32,
    pub corridor: u32,
}

impl Default for BspDesc {
    fn default() -> Self {
        Self {
            seed: Default::default(),
            width: 48,
            height: 32,
            min_room: 4,
            max_room: 12,
            depth: 5,
            padding: 1,
            corridor: 1,
        }
    }
}

impl FromJson for BspDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            seed: field(o, "seed", d.seed)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            min_room: field(o, "minRoom", d.min_room)?,
            max_room: field(o, "maxRoom", d.max_room)?,
            depth: field(o, "depth", d.depth)?,
            padding: field(o, "padding", d.padding)?,
            corridor: field(o, "corridor", d.corridor)?,
        })
    }
}

impl ToJson for BspDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("seed".into(), self.seed.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("minRoom".into(), self.min_room.to_json());
        o.insert("maxRoom".into(), self.max_room.to_json());
        o.insert("depth".into(), self.depth.to_json());
        o.insert("padding".into(), self.padding.to_json());
        o.insert("corridor".into(), self.corridor.to_json());
        Value::Object(o)
    }
}

/// `BspResult`: packages/knowledge/wiki/procgen.md.
#[derive(Clone, Debug, PartialEq)]
pub struct BspResult {
    pub rooms: Vec<CellRect>,
    pub corridors: Vec<CellRect>,
    pub cells: Vec<u32>,
}

impl Default for BspResult {
    fn default() -> Self {
        Self {
            rooms: Default::default(),
            corridors: Default::default(),
            cells: Default::default(),
        }
    }
}

impl FromJson for BspResult {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            rooms: field(o, "rooms", d.rooms)?,
            corridors: field(o, "corridors", d.corridors)?,
            cells: field(o, "cells", d.cells)?,
        })
    }
}

impl ToJson for BspResult {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("rooms".into(), self.rooms.to_json());
        o.insert("corridors".into(), self.corridors.to_json());
        o.insert("cells".into(), self.cells.to_json());
        Value::Object(o)
    }
}

/// `WfcTile`: packages/knowledge/wiki/procgen.md.
#[derive(Clone, Debug, PartialEq)]
pub struct WfcTile {
    pub weight: f32,
    pub up: u32,
    pub right: u32,
    pub down: u32,
    pub left: u32,
    pub rotations: bool,
}

impl Default for WfcTile {
    fn default() -> Self {
        Self {
            weight: 1.0_f32,
            up: Default::default(),
            right: Default::default(),
            down: Default::default(),
            left: Default::default(),
            rotations: Default::default(),
        }
    }
}

impl FromJson for WfcTile {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            weight: field(o, "weight", d.weight)?,
            up: field(o, "up", d.up)?,
            right: field(o, "right", d.right)?,
            down: field(o, "down", d.down)?,
            left: field(o, "left", d.left)?,
            rotations: field(o, "rotations", d.rotations)?,
        })
    }
}

impl ToJson for WfcTile {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("weight".into(), self.weight.to_json());
        o.insert("up".into(), self.up.to_json());
        o.insert("right".into(), self.right.to_json());
        o.insert("down".into(), self.down.to_json());
        o.insert("left".into(), self.left.to_json());
        o.insert("rotations".into(), self.rotations.to_json());
        Value::Object(o)
    }
}

/// `WfcCell`: packages/knowledge/wiki/procgen.md.
#[derive(Clone, Debug, PartialEq)]
pub struct WfcCell {
    pub x: u32,
    pub y: u32,
    pub tile: u32,
    pub rotation: u32,
}

impl Default for WfcCell {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            tile: Default::default(),
            rotation: Default::default(),
        }
    }
}

impl FromJson for WfcCell {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            tile: field(o, "tile", d.tile)?,
            rotation: field(o, "rotation", d.rotation)?,
        })
    }
}

impl ToJson for WfcCell {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("tile".into(), self.tile.to_json());
        o.insert("rotation".into(), self.rotation.to_json());
        Value::Object(o)
    }
}

/// `WfcDesc`: packages/knowledge/wiki/procgen.md.
#[derive(Clone, Debug, PartialEq)]
pub struct WfcDesc {
    pub seed: u32,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<WfcTile>,
    pub fixed: Vec<WfcCell>,
    pub wrap: bool,
    pub attempts: u32,
}

impl Default for WfcDesc {
    fn default() -> Self {
        Self {
            seed: Default::default(),
            width: 16,
            height: 16,
            tiles: Default::default(),
            fixed: Default::default(),
            wrap: Default::default(),
            attempts: 10,
        }
    }
}

impl FromJson for WfcDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            seed: field(o, "seed", d.seed)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            tiles: field(o, "tiles", d.tiles)?,
            fixed: field(o, "fixed", d.fixed)?,
            wrap: field(o, "wrap", d.wrap)?,
            attempts: field(o, "attempts", d.attempts)?,
        })
    }
}

impl ToJson for WfcDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("seed".into(), self.seed.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("tiles".into(), self.tiles.to_json());
        o.insert("fixed".into(), self.fixed.to_json());
        o.insert("wrap".into(), self.wrap.to_json());
        o.insert("attempts".into(), self.attempts.to_json());
        Value::Object(o)
    }
}

/// `WfcResult`: packages/knowledge/wiki/procgen.md.
#[derive(Clone, Debug, PartialEq)]
pub struct WfcResult {
    pub tiles: Vec<u32>,
    pub rotations: Vec<u32>,
    pub attempt: u32,
}

impl Default for WfcResult {
    fn default() -> Self {
        Self {
            tiles: Default::default(),
            rotations: Default::default(),
            attempt: Default::default(),
        }
    }
}

impl FromJson for WfcResult {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tiles: field(o, "tiles", d.tiles)?,
            rotations: field(o, "rotations", d.rotations)?,
            attempt: field(o, "attempt", d.attempt)?,
        })
    }
}

impl ToJson for WfcResult {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tiles".into(), self.tiles.to_json());
        o.insert("rotations".into(), self.rotations.to_json());
        o.insert("attempt".into(), self.attempt.to_json());
        Value::Object(o)
    }
}

/// `RecordedIntent`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RecordedIntent {
    pub tick: f64,
    pub name: String,
    pub layout_hash: String,
    pub payload: Vec<u32>,
}

impl Default for RecordedIntent {
    fn default() -> Self {
        Self {
            tick: Default::default(),
            name: Default::default(),
            layout_hash: Default::default(),
            payload: Default::default(),
        }
    }
}

impl FromJson for RecordedIntent {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tick: field(o, "tick", d.tick)?,
            name: field(o, "name", d.name)?,
            layout_hash: field(o, "layoutHash", d.layout_hash)?,
            payload: field(o, "payload", d.payload)?,
        })
    }
}

impl ToJson for RecordedIntent {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tick".into(), self.tick.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("layoutHash".into(), self.layout_hash.to_json());
        o.insert("payload".into(), self.payload.to_json());
        Value::Object(o)
    }
}

/// `InputDevice`: packages/knowledge/wiki/input.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InputDevice {
    #[default]
    KeyboardMouse,
    Xbox,
    Playstation,
    Switch,
    Gamepad,
    Touch,
}

impl InputDevice {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["keyboardMouse", "xbox", "playstation", "switch", "gamepad", "touch"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::KeyboardMouse, Self::Xbox, Self::Playstation, Self::Switch, Self::Gamepad, Self::Touch];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("InputDevice", s, Self::NAMES)),
        }
    }
}

impl FromJson for InputDevice {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "InputDevice", Self::NAMES)?)
    }
}

impl ToJson for InputDevice {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `ActionKind`: packages/knowledge/wiki/input.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ActionKind {
    #[default]
    Button,
    Axis,
    Axis2,
}

impl ActionKind {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["button", "axis", "axis2"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Button, Self::Axis, Self::Axis2];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("ActionKind", s, Self::NAMES)),
        }
    }
}

impl FromJson for ActionKind {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "ActionKind", Self::NAMES)?)
    }
}

impl ToJson for ActionKind {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `TouchZone`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TouchZone {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for TouchZone {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            width: Default::default(),
            height: Default::default(),
        }
    }
}

impl FromJson for TouchZone {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
        })
    }
}

impl ToJson for TouchZone {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        Value::Object(o)
    }
}

/// `InputBinding`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct InputBinding {
    pub control: String,
    pub negative: String,
    pub positive: String,
    pub up: String,
    pub down: String,
    pub left: String,
    pub right: String,
    pub scale: f32,
    pub invert_y: bool,
    pub deadzone: f32,
    pub zone: TouchZone,
    pub radius: f32,
}

impl Default for InputBinding {
    fn default() -> Self {
        Self {
            control: Default::default(),
            negative: Default::default(),
            positive: Default::default(),
            up: Default::default(),
            down: Default::default(),
            left: Default::default(),
            right: Default::default(),
            scale: 1.0_f32,
            invert_y: Default::default(),
            deadzone: Default::default(),
            zone: Default::default(),
            radius: Default::default(),
        }
    }
}

impl FromJson for InputBinding {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            control: field(o, "control", d.control)?,
            negative: field(o, "negative", d.negative)?,
            positive: field(o, "positive", d.positive)?,
            up: field(o, "up", d.up)?,
            down: field(o, "down", d.down)?,
            left: field(o, "left", d.left)?,
            right: field(o, "right", d.right)?,
            scale: field(o, "scale", d.scale)?,
            invert_y: field(o, "invertY", d.invert_y)?,
            deadzone: field(o, "deadzone", d.deadzone)?,
            zone: field(o, "zone", d.zone)?,
            radius: field(o, "radius", d.radius)?,
        })
    }
}

impl ToJson for InputBinding {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("control".into(), self.control.to_json());
        o.insert("negative".into(), self.negative.to_json());
        o.insert("positive".into(), self.positive.to_json());
        o.insert("up".into(), self.up.to_json());
        o.insert("down".into(), self.down.to_json());
        o.insert("left".into(), self.left.to_json());
        o.insert("right".into(), self.right.to_json());
        o.insert("scale".into(), self.scale.to_json());
        o.insert("invertY".into(), self.invert_y.to_json());
        o.insert("deadzone".into(), self.deadzone.to_json());
        o.insert("zone".into(), self.zone.to_json());
        o.insert("radius".into(), self.radius.to_json());
        Value::Object(o)
    }
}

/// `ActionDesc`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionDesc {
    pub name: String,
    pub kind: ActionKind,
    pub action_map: String,
    pub label: String,
    pub bindings: Vec<InputBinding>,
    pub threshold: f32,
    pub deadzone: f32,
    pub hold_ms: f32,
    pub locked: bool,
    pub toggle: bool,
}

impl Default for ActionDesc {
    fn default() -> Self {
        Self {
            name: Default::default(),
            kind: Default::default(),
            action_map: String::from("gameplay"),
            label: Default::default(),
            bindings: Default::default(),
            threshold: 0.5_f32,
            deadzone: Default::default(),
            hold_ms: Default::default(),
            locked: Default::default(),
            toggle: Default::default(),
        }
    }
}

impl FromJson for ActionDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            kind: field(o, "kind", d.kind)?,
            action_map: field(o, "actionMap", d.action_map)?,
            label: field(o, "label", d.label)?,
            bindings: field(o, "bindings", d.bindings)?,
            threshold: field(o, "threshold", d.threshold)?,
            deadzone: field(o, "deadzone", d.deadzone)?,
            hold_ms: field(o, "holdMs", d.hold_ms)?,
            locked: field(o, "locked", d.locked)?,
            toggle: field(o, "toggle", d.toggle)?,
        })
    }
}

impl ToJson for ActionDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("actionMap".into(), self.action_map.to_json());
        o.insert("label".into(), self.label.to_json());
        o.insert("bindings".into(), self.bindings.to_json());
        o.insert("threshold".into(), self.threshold.to_json());
        o.insert("deadzone".into(), self.deadzone.to_json());
        o.insert("holdMs".into(), self.hold_ms.to_json());
        o.insert("locked".into(), self.locked.to_json());
        o.insert("toggle".into(), self.toggle.to_json());
        Value::Object(o)
    }
}

/// `ActionState`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionState {
    pub down: bool,
    pub pressed: bool,
    pub released: bool,
    pub held: bool,
    pub value: f32,
    pub vector: Vec2,
    pub device: InputDevice,
}

impl Default for ActionState {
    fn default() -> Self {
        Self {
            down: Default::default(),
            pressed: Default::default(),
            released: Default::default(),
            held: Default::default(),
            value: Default::default(),
            vector: Default::default(),
            device: Default::default(),
        }
    }
}

impl FromJson for ActionState {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            down: field(o, "down", d.down)?,
            pressed: field(o, "pressed", d.pressed)?,
            released: field(o, "released", d.released)?,
            held: field(o, "held", d.held)?,
            value: field(o, "value", d.value)?,
            vector: field(o, "vector", d.vector)?,
            device: field(o, "device", d.device)?,
        })
    }
}

impl ToJson for ActionState {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("down".into(), self.down.to_json());
        o.insert("pressed".into(), self.pressed.to_json());
        o.insert("released".into(), self.released.to_json());
        o.insert("held".into(), self.held.to_json());
        o.insert("value".into(), self.value.to_json());
        o.insert("vector".into(), self.vector.to_json());
        o.insert("device".into(), self.device.to_json());
        Value::Object(o)
    }
}

/// `ActionInfo`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionInfo {
    pub name: String,
    pub kind: ActionKind,
    pub action_map: String,
    pub label: String,
    pub bindings: Vec<InputBinding>,
    pub defaults: Vec<InputBinding>,
    pub locked: bool,
    pub toggle: bool,
    pub rebound: bool,
}

impl Default for ActionInfo {
    fn default() -> Self {
        Self {
            name: Default::default(),
            kind: Default::default(),
            action_map: Default::default(),
            label: Default::default(),
            bindings: Default::default(),
            defaults: Default::default(),
            locked: Default::default(),
            toggle: Default::default(),
            rebound: Default::default(),
        }
    }
}

impl FromJson for ActionInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            kind: field(o, "kind", d.kind)?,
            action_map: field(o, "actionMap", d.action_map)?,
            label: field(o, "label", d.label)?,
            bindings: field(o, "bindings", d.bindings)?,
            defaults: field(o, "defaults", d.defaults)?,
            locked: field(o, "locked", d.locked)?,
            toggle: field(o, "toggle", d.toggle)?,
            rebound: field(o, "rebound", d.rebound)?,
        })
    }
}

impl ToJson for ActionInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("actionMap".into(), self.action_map.to_json());
        o.insert("label".into(), self.label.to_json());
        o.insert("bindings".into(), self.bindings.to_json());
        o.insert("defaults".into(), self.defaults.to_json());
        o.insert("locked".into(), self.locked.to_json());
        o.insert("toggle".into(), self.toggle.to_json());
        o.insert("rebound".into(), self.rebound.to_json());
        Value::Object(o)
    }
}

/// `BindingConflict`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct BindingConflict {
    pub action: String,
    pub slot: u32,
    pub control: String,
    pub action_map: String,
}

impl Default for BindingConflict {
    fn default() -> Self {
        Self {
            action: Default::default(),
            slot: Default::default(),
            control: Default::default(),
            action_map: Default::default(),
        }
    }
}

impl FromJson for BindingConflict {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            action: field(o, "action", d.action)?,
            slot: field(o, "slot", d.slot)?,
            control: field(o, "control", d.control)?,
            action_map: field(o, "actionMap", d.action_map)?,
        })
    }
}

impl ToJson for BindingConflict {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("action".into(), self.action.to_json());
        o.insert("slot".into(), self.slot.to_json());
        o.insert("control".into(), self.control.to_json());
        o.insert("actionMap".into(), self.action_map.to_json());
        Value::Object(o)
    }
}

/// `ConflictPolicy`: packages/knowledge/wiki/input.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ConflictPolicy {
    #[default]
    Refuse,
    Swap,
    Replace,
    Allow,
}

impl ConflictPolicy {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["refuse", "swap", "replace", "allow"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Refuse, Self::Swap, Self::Replace, Self::Allow];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("ConflictPolicy", s, Self::NAMES)),
        }
    }
}

impl FromJson for ConflictPolicy {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "ConflictPolicy", Self::NAMES)?)
    }
}

impl ToJson for ConflictPolicy {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `RebindDesc`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RebindDesc {
    pub action: String,
    pub slot: u32,
    pub part: String,
    pub binding: InputBinding,
    pub policy: ConflictPolicy,
}

impl Default for RebindDesc {
    fn default() -> Self {
        Self {
            action: Default::default(),
            slot: Default::default(),
            part: Default::default(),
            binding: Default::default(),
            policy: Default::default(),
        }
    }
}

impl FromJson for RebindDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            action: field(o, "action", d.action)?,
            slot: field(o, "slot", d.slot)?,
            part: field(o, "part", d.part)?,
            binding: field(o, "binding", d.binding)?,
            policy: field(o, "policy", d.policy)?,
        })
    }
}

impl ToJson for RebindDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("action".into(), self.action.to_json());
        o.insert("slot".into(), self.slot.to_json());
        o.insert("part".into(), self.part.to_json());
        o.insert("binding".into(), self.binding.to_json());
        o.insert("policy".into(), self.policy.to_json());
        Value::Object(o)
    }
}

/// `RebindAnswer`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RebindAnswer {
    pub applied: bool,
    pub conflicts: Vec<BindingConflict>,
}

impl Default for RebindAnswer {
    fn default() -> Self {
        Self {
            applied: Default::default(),
            conflicts: Default::default(),
        }
    }
}

impl FromJson for RebindAnswer {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            applied: field(o, "applied", d.applied)?,
            conflicts: field(o, "conflicts", d.conflicts)?,
        })
    }
}

impl ToJson for RebindAnswer {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("applied".into(), self.applied.to_json());
        o.insert("conflicts".into(), self.conflicts.to_json());
        Value::Object(o)
    }
}

/// `CaptureAccept`: packages/knowledge/wiki/input.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CaptureAccept {
    #[default]
    Any,
    KeyboardMouse,
    Gamepad,
}

impl CaptureAccept {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["any", "keyboardMouse", "gamepad"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Any, Self::KeyboardMouse, Self::Gamepad];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("CaptureAccept", s, Self::NAMES)),
        }
    }
}

impl FromJson for CaptureAccept {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "CaptureAccept", Self::NAMES)?)
    }
}

impl ToJson for CaptureAccept {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `CaptureDesc`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct CaptureDesc {
    pub action: String,
    pub slot: u32,
    pub part: String,
    pub accept: CaptureAccept,
    pub cancel: Vec<String>,
    pub timeout_ms: f32,
}

impl Default for CaptureDesc {
    fn default() -> Self {
        Self {
            action: Default::default(),
            slot: Default::default(),
            part: Default::default(),
            accept: Default::default(),
            cancel: Default::default(),
            timeout_ms: Default::default(),
        }
    }
}

impl FromJson for CaptureDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            action: field(o, "action", d.action)?,
            slot: field(o, "slot", d.slot)?,
            part: field(o, "part", d.part)?,
            accept: field(o, "accept", d.accept)?,
            cancel: field(o, "cancel", d.cancel)?,
            timeout_ms: field(o, "timeoutMs", d.timeout_ms)?,
        })
    }
}

impl ToJson for CaptureDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("action".into(), self.action.to_json());
        o.insert("slot".into(), self.slot.to_json());
        o.insert("part".into(), self.part.to_json());
        o.insert("accept".into(), self.accept.to_json());
        o.insert("cancel".into(), self.cancel.to_json());
        o.insert("timeoutMs".into(), self.timeout_ms.to_json());
        Value::Object(o)
    }
}

/// `CaptureState`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct CaptureState {
    pub active: bool,
    pub done: bool,
    pub cancelled: bool,
    pub action: String,
    pub slot: u32,
    pub part: String,
    pub control: String,
    pub binding: InputBinding,
    pub conflicts: Vec<BindingConflict>,
}

impl Default for CaptureState {
    fn default() -> Self {
        Self {
            active: Default::default(),
            done: Default::default(),
            cancelled: Default::default(),
            action: Default::default(),
            slot: Default::default(),
            part: Default::default(),
            control: Default::default(),
            binding: Default::default(),
            conflicts: Default::default(),
        }
    }
}

impl FromJson for CaptureState {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            active: field(o, "active", d.active)?,
            done: field(o, "done", d.done)?,
            cancelled: field(o, "cancelled", d.cancelled)?,
            action: field(o, "action", d.action)?,
            slot: field(o, "slot", d.slot)?,
            part: field(o, "part", d.part)?,
            control: field(o, "control", d.control)?,
            binding: field(o, "binding", d.binding)?,
            conflicts: field(o, "conflicts", d.conflicts)?,
        })
    }
}

impl ToJson for CaptureState {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("active".into(), self.active.to_json());
        o.insert("done".into(), self.done.to_json());
        o.insert("cancelled".into(), self.cancelled.to_json());
        o.insert("action".into(), self.action.to_json());
        o.insert("slot".into(), self.slot.to_json());
        o.insert("part".into(), self.part.to_json());
        o.insert("control".into(), self.control.to_json());
        o.insert("binding".into(), self.binding.to_json());
        o.insert("conflicts".into(), self.conflicts.to_json());
        Value::Object(o)
    }
}

/// `InputGlyph`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct InputGlyph {
    pub icon: String,
    pub text: String,
    pub device: InputDevice,
    pub control: String,
}

impl Default for InputGlyph {
    fn default() -> Self {
        Self {
            icon: Default::default(),
            text: Default::default(),
            device: Default::default(),
            control: Default::default(),
        }
    }
}

impl FromJson for InputGlyph {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            icon: field(o, "icon", d.icon)?,
            text: field(o, "text", d.text)?,
            device: field(o, "device", d.device)?,
            control: field(o, "control", d.control)?,
        })
    }
}

impl ToJson for InputGlyph {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("icon".into(), self.icon.to_json());
        o.insert("text".into(), self.text.to_json());
        o.insert("device".into(), self.device.to_json());
        o.insert("control".into(), self.control.to_json());
        Value::Object(o)
    }
}

/// `SensitivityDesc`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SensitivityDesc {
    pub action: String,
    pub device: InputDevice,
    pub scale: f32,
    pub invert_x: bool,
    pub invert_y: bool,
}

impl Default for SensitivityDesc {
    fn default() -> Self {
        Self {
            action: Default::default(),
            device: Default::default(),
            scale: 1.0_f32,
            invert_x: Default::default(),
            invert_y: Default::default(),
        }
    }
}

impl FromJson for SensitivityDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            action: field(o, "action", d.action)?,
            device: field(o, "device", d.device)?,
            scale: field(o, "scale", d.scale)?,
            invert_x: field(o, "invertX", d.invert_x)?,
            invert_y: field(o, "invertY", d.invert_y)?,
        })
    }
}

impl ToJson for SensitivityDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("action".into(), self.action.to_json());
        o.insert("device".into(), self.device.to_json());
        o.insert("scale".into(), self.scale.to_json());
        o.insert("invertX".into(), self.invert_x.to_json());
        o.insert("invertY".into(), self.invert_y.to_json());
        Value::Object(o)
    }
}

/// `KeyLabel`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyLabel {
    pub code: String,
    pub label: String,
}

impl Default for KeyLabel {
    fn default() -> Self {
        Self {
            code: Default::default(),
            label: Default::default(),
        }
    }
}

impl FromJson for KeyLabel {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            code: field(o, "code", d.code)?,
            label: field(o, "label", d.label)?,
        })
    }
}

impl ToJson for KeyLabel {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("code".into(), self.code.to_json());
        o.insert("label".into(), self.label.to_json());
        Value::Object(o)
    }
}

/// `MouseDesc`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct MouseDesc {
    pub raw: bool,
}

impl Default for MouseDesc {
    fn default() -> Self {
        Self {
            raw: true,
        }
    }
}

impl FromJson for MouseDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            raw: field(o, "raw", d.raw)?,
        })
    }
}

impl ToJson for MouseDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("raw".into(), self.raw.to_json());
        Value::Object(o)
    }
}

/// `MouseInfo`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct MouseInfo {
    pub raw: bool,
    pub raw_active: bool,
    pub raw_unsupported: bool,
}

impl Default for MouseInfo {
    fn default() -> Self {
        Self {
            raw: Default::default(),
            raw_active: Default::default(),
            raw_unsupported: Default::default(),
        }
    }
}

impl FromJson for MouseInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            raw: field(o, "raw", d.raw)?,
            raw_active: field(o, "rawActive", d.raw_active)?,
            raw_unsupported: field(o, "rawUnsupported", d.raw_unsupported)?,
        })
    }
}

impl ToJson for MouseInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("raw".into(), self.raw.to_json());
        o.insert("rawActive".into(), self.raw_active.to_json());
        o.insert("rawUnsupported".into(), self.raw_unsupported.to_json());
        Value::Object(o)
    }
}

/// `DeadzoneDesc`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DeadzoneDesc {
    pub pad: i32,
    pub stick: f32,
    pub trigger: f32,
}

impl Default for DeadzoneDesc {
    fn default() -> Self {
        Self {
            pad: -1,
            stick: 0.15_f32,
            trigger: 0.05_f32,
        }
    }
}

impl FromJson for DeadzoneDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            pad: field(o, "pad", d.pad)?,
            stick: field(o, "stick", d.stick)?,
            trigger: field(o, "trigger", d.trigger)?,
        })
    }
}

impl ToJson for DeadzoneDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("pad".into(), self.pad.to_json());
        o.insert("stick".into(), self.stick.to_json());
        o.insert("trigger".into(), self.trigger.to_json());
        Value::Object(o)
    }
}

/// `RumbleDesc`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RumbleDesc {
    pub pad: i32,
    pub low: f32,
    pub high: f32,
    pub ms: f32,
}

impl Default for RumbleDesc {
    fn default() -> Self {
        Self {
            pad: -1,
            low: Default::default(),
            high: Default::default(),
            ms: 200.0_f32,
        }
    }
}

impl FromJson for RumbleDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            pad: field(o, "pad", d.pad)?,
            low: field(o, "low", d.low)?,
            high: field(o, "high", d.high)?,
            ms: field(o, "ms", d.ms)?,
        })
    }
}

impl ToJson for RumbleDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("pad".into(), self.pad.to_json());
        o.insert("low".into(), self.low.to_json());
        o.insert("high".into(), self.high.to_json());
        o.insert("ms".into(), self.ms.to_json());
        Value::Object(o)
    }
}

/// `GamepadButton`: packages/knowledge/wiki/input.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GamepadButton {
    #[default]
    A,
    B,
    X,
    Y,
    LeftBumper,
    RightBumper,
    LeftTrigger,
    RightTrigger,
    Back,
    Start,
    LeftStick,
    RightStick,
    Up,
    Down,
    Left,
    Right,
    Home,
}

impl GamepadButton {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["a", "b", "x", "y", "leftBumper", "rightBumper", "leftTrigger", "rightTrigger", "back", "start", "leftStick", "rightStick", "up", "down", "left", "right", "home"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::A, Self::B, Self::X, Self::Y, Self::LeftBumper, Self::RightBumper, Self::LeftTrigger, Self::RightTrigger, Self::Back, Self::Start, Self::LeftStick, Self::RightStick, Self::Up, Self::Down, Self::Left, Self::Right, Self::Home];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("GamepadButton", s, Self::NAMES)),
        }
    }
}

impl FromJson for GamepadButton {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "GamepadButton", Self::NAMES)?)
    }
}

impl ToJson for GamepadButton {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `GamepadAxis`: packages/knowledge/wiki/input.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GamepadAxis {
    #[default]
    LeftX,
    LeftY,
    RightX,
    RightY,
    LeftTrigger,
    RightTrigger,
}

impl GamepadAxis {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["leftX", "leftY", "rightX", "rightY", "leftTrigger", "rightTrigger"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::LeftX, Self::LeftY, Self::RightX, Self::RightY, Self::LeftTrigger, Self::RightTrigger];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("GamepadAxis", s, Self::NAMES)),
        }
    }
}

impl FromJson for GamepadAxis {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "GamepadAxis", Self::NAMES)?)
    }
}

impl ToJson for GamepadAxis {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `KeyAction`: packages/knowledge/wiki/input.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KeyAction {
    #[default]
    Press,
    Down,
    Up,
}

impl KeyAction {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["press", "down", "up"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Press, Self::Down, Self::Up];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("KeyAction", s, Self::NAMES)),
        }
    }
}

impl FromJson for KeyAction {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "KeyAction", Self::NAMES)?)
    }
}

impl ToJson for KeyAction {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `PointerAction`: packages/knowledge/wiki/input.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PointerAction {
    #[default]
    Click,
    Move,
    Down,
    Up,
}

impl PointerAction {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["click", "move", "down", "up"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Click, Self::Move, Self::Down, Self::Up];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("PointerAction", s, Self::NAMES)),
        }
    }
}

impl FromJson for PointerAction {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "PointerAction", Self::NAMES)?)
    }
}

impl ToJson for PointerAction {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `PointerButton`: packages/knowledge/wiki/input.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PointerButton {
    #[default]
    None,
    Left,
    Middle,
    Right,
}

impl PointerButton {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["none", "left", "middle", "right"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::None, Self::Left, Self::Middle, Self::Right];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("PointerButton", s, Self::NAMES)),
        }
    }
}

impl FromJson for PointerButton {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "PointerButton", Self::NAMES)?)
    }
}

impl ToJson for PointerButton {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `Modifiers`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Modifiers {
    pub alt: bool,
    pub control: bool,
    pub platform: bool,
    pub shift: bool,
}

impl Default for Modifiers {
    fn default() -> Self {
        Self {
            alt: Default::default(),
            control: Default::default(),
            platform: Default::default(),
            shift: Default::default(),
        }
    }
}

impl FromJson for Modifiers {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            alt: field(o, "alt", d.alt)?,
            control: field(o, "control", d.control)?,
            platform: field(o, "platform", d.platform)?,
            shift: field(o, "shift", d.shift)?,
        })
    }
}

impl ToJson for Modifiers {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("alt".into(), self.alt.to_json());
        o.insert("control".into(), self.control.to_json());
        o.insert("platform".into(), self.platform.to_json());
        o.insert("shift".into(), self.shift.to_json());
        Value::Object(o)
    }
}

/// `KeyInput`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyInput {
    pub key: String,
    pub action: KeyAction,
    pub modifiers: Option<Modifiers>,
    pub at: Option<f32>,
}

impl Default for KeyInput {
    fn default() -> Self {
        Self {
            key: Default::default(),
            action: Default::default(),
            modifiers: Default::default(),
            at: Default::default(),
        }
    }
}

impl FromJson for KeyInput {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            key: field(o, "key", d.key)?,
            action: field(o, "action", d.action)?,
            modifiers: field(o, "modifiers", d.modifiers)?,
            at: field(o, "at", d.at)?,
        })
    }
}

impl ToJson for KeyInput {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("key".into(), self.key.to_json());
        o.insert("action".into(), self.action.to_json());
        if let Some(x) = &self.modifiers {
            o.insert("modifiers".into(), x.to_json());
        }
        if let Some(x) = &self.at {
            o.insert("at".into(), x.to_json());
        }
        Value::Object(o)
    }
}

/// `PointerInput`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PointerInput {
    pub action: PointerAction,
    pub x: f32,
    pub y: f32,
    pub button: PointerButton,
    pub id: Option<u32>,
    pub at: Option<f32>,
    pub movement_x: Option<f32>,
    pub movement_y: Option<f32>,
}

impl Default for PointerInput {
    fn default() -> Self {
        Self {
            action: Default::default(),
            x: Default::default(),
            y: Default::default(),
            button: Default::default(),
            id: Default::default(),
            at: Default::default(),
            movement_x: Default::default(),
            movement_y: Default::default(),
        }
    }
}

impl FromJson for PointerInput {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            action: field(o, "action", d.action)?,
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            button: field(o, "button", d.button)?,
            id: field(o, "id", d.id)?,
            at: field(o, "at", d.at)?,
            movement_x: field(o, "movementX", d.movement_x)?,
            movement_y: field(o, "movementY", d.movement_y)?,
        })
    }
}

impl ToJson for PointerInput {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("action".into(), self.action.to_json());
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("button".into(), self.button.to_json());
        if let Some(x) = &self.id {
            o.insert("id".into(), x.to_json());
        }
        if let Some(x) = &self.at {
            o.insert("at".into(), x.to_json());
        }
        if let Some(x) = &self.movement_x {
            o.insert("movementX".into(), x.to_json());
        }
        if let Some(x) = &self.movement_y {
            o.insert("movementY".into(), x.to_json());
        }
        Value::Object(o)
    }
}

/// `PointerLockReport`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PointerLockReport {
    pub locked: bool,
}

impl Default for PointerLockReport {
    fn default() -> Self {
        Self {
            locked: Default::default(),
        }
    }
}

impl FromJson for PointerLockReport {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            locked: field(o, "locked", d.locked)?,
        })
    }
}

impl ToJson for PointerLockReport {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("locked".into(), self.locked.to_json());
        Value::Object(o)
    }
}

/// `GamepadInput`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct GamepadInput {
    pub pad: Option<u32>,
    pub connected: Option<bool>,
    pub buttons: Option<Vec<GamepadButton>>,
    pub axes: Option<Vec<f32>>,
    pub triggers: Option<Vec<f32>>,
    pub device: Option<InputDevice>,
}

impl Default for GamepadInput {
    fn default() -> Self {
        Self {
            pad: Default::default(),
            connected: Default::default(),
            buttons: Default::default(),
            axes: Default::default(),
            triggers: Default::default(),
            device: Default::default(),
        }
    }
}

impl FromJson for GamepadInput {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            pad: field(o, "pad", d.pad)?,
            connected: field(o, "connected", d.connected)?,
            buttons: field(o, "buttons", d.buttons)?,
            axes: field(o, "axes", d.axes)?,
            triggers: field(o, "triggers", d.triggers)?,
            device: field(o, "device", d.device)?,
        })
    }
}

impl ToJson for GamepadInput {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        if let Some(x) = &self.pad {
            o.insert("pad".into(), x.to_json());
        }
        if let Some(x) = &self.connected {
            o.insert("connected".into(), x.to_json());
        }
        if let Some(x) = &self.buttons {
            o.insert("buttons".into(), x.to_json());
        }
        if let Some(x) = &self.axes {
            o.insert("axes".into(), x.to_json());
        }
        if let Some(x) = &self.triggers {
            o.insert("triggers".into(), x.to_json());
        }
        if let Some(x) = &self.device {
            o.insert("device".into(), x.to_json());
        }
        Value::Object(o)
    }
}

/// `WheelInput`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct WheelInput {
    pub x: f32,
    pub y: f32,
    pub delta_x: f32,
    pub delta_y: f32,
}

impl Default for WheelInput {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            delta_x: Default::default(),
            delta_y: Default::default(),
        }
    }
}

impl FromJson for WheelInput {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            delta_x: field(o, "deltaX", d.delta_x)?,
            delta_y: field(o, "deltaY", d.delta_y)?,
        })
    }
}

impl ToJson for WheelInput {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("deltaX".into(), self.delta_x.to_json());
        o.insert("deltaY".into(), self.delta_y.to_json());
        Value::Object(o)
    }
}

/// `TextEntry`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TextEntry {
    pub text: String,
    pub composing: Option<bool>,
}

impl Default for TextEntry {
    fn default() -> Self {
        Self {
            text: Default::default(),
            composing: Default::default(),
        }
    }
}

impl FromJson for TextEntry {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            text: field(o, "text", d.text)?,
            composing: field(o, "composing", d.composing)?,
        })
    }
}

impl ToJson for TextEntry {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("text".into(), self.text.to_json());
        if let Some(x) = &self.composing {
            o.insert("composing".into(), x.to_json());
        }
        Value::Object(o)
    }
}

/// `A11yActionKind`: packages/knowledge/wiki/input.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum A11yActionKind {
    #[default]
    Focus,
    Activate,
    Increment,
    Decrement,
}

impl A11yActionKind {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["focus", "activate", "increment", "decrement"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Focus, Self::Activate, Self::Increment, Self::Decrement];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("A11yActionKind", s, Self::NAMES)),
        }
    }
}

impl FromJson for A11yActionKind {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "A11yActionKind", Self::NAMES)?)
    }
}

impl ToJson for A11yActionKind {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `A11yRequest`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub struct A11yRequest {
    pub action: A11yActionKind,
    pub path: String,
}

impl Default for A11yRequest {
    fn default() -> Self {
        Self {
            action: Default::default(),
            path: Default::default(),
        }
    }
}

impl FromJson for A11yRequest {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            action: field(o, "action", d.action)?,
            path: field(o, "path", d.path)?,
        })
    }
}

impl ToJson for A11yRequest {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("action".into(), self.action.to_json());
        o.insert("path".into(), self.path.to_json());
        Value::Object(o)
    }
}

/// `InjectedInput`: packages/knowledge/wiki/input.md.
#[derive(Clone, Debug, PartialEq)]
pub enum InjectedInput {
    Key(KeyInput),
    Pointer(PointerInput),
    Wheel(WheelInput),
    Gamepad(GamepadInput),
    Text(TextEntry),
    A11y(A11yRequest),
    PointerLock(PointerLockReport),
}

impl Default for InjectedInput {
    fn default() -> Self {
        Self::Key(Default::default())
    }
}

impl InjectedInput {
    /// The tag key and the cases' tags, in order.
    pub const TAG: &'static str = "kind";
    pub const TAGS: &'static [&'static str] = &["key", "pointer", "wheel", "gamepad", "text", "a11y", "pointerLock"];
    /// This case's tag.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Key(_) => "key",
            Self::Pointer(_) => "pointer",
            Self::Wheel(_) => "wheel",
            Self::Gamepad(_) => "gamepad",
            Self::Text(_) => "text",
            Self::A11y(_) => "a11y",
            Self::PointerLock(_) => "pointerLock",
        }
    }
}

impl FromJson for InjectedInput {
    fn from_json(v: &Value) -> Result<Self, String> {
        let o = object(v)?;
        let tag = match o.get(Self::TAG) {
            Some(t) => text(t).map_err(|e| at(Self::TAG, e))?,
            None => return Err(at(Self::TAG, format!("missing: one of {}", Self::TAGS.join(", ")))),
        };
        Ok(match tag {
            "key" => Self::Key(KeyInput::from_json(v)?),
            "pointer" => Self::Pointer(PointerInput::from_json(v)?),
            "wheel" => Self::Wheel(WheelInput::from_json(v)?),
            "gamepad" => Self::Gamepad(GamepadInput::from_json(v)?),
            "text" => Self::Text(TextEntry::from_json(v)?),
            "a11y" => Self::A11y(A11yRequest::from_json(v)?),
            "pointerLock" => Self::PointerLock(PointerLockReport::from_json(v)?),
            t => return Err(at(Self::TAG, unknown_variant("InjectedInput", t, Self::TAGS))),
        })
    }
}

impl ToJson for InjectedInput {
    fn to_json(&self) -> Value {
        let mut v = match self {
            Self::Key(p) => p.to_json(),
            Self::Pointer(p) => p.to_json(),
            Self::Wheel(p) => p.to_json(),
            Self::Gamepad(p) => p.to_json(),
            Self::Text(p) => p.to_json(),
            Self::A11y(p) => p.to_json(),
            Self::PointerLock(p) => p.to_json(),
        };
        if let Value::Object(o) = &mut v {
            o.insert(Self::TAG.into(), Value::String(self.tag().into()));
        }
        v
    }
}

/// `Side`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Side {
    #[default]
    Front,
    Back,
    Double,
}

impl Side {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["front", "back", "double"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Front, Self::Back, Self::Double];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("Side", s, Self::NAMES)),
        }
    }
}

impl FromJson for Side {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "Side", Self::NAMES)?)
    }
}

impl ToJson for Side {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `RenderTier`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RenderTier {
    #[default]
    Webgpu,
    Vulkan,
    VulkanCpu,
    Metal,
    None,
}

impl RenderTier {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["webgpu", "vulkan", "vulkan-cpu", "metal", "none"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Webgpu, Self::Vulkan, Self::VulkanCpu, Self::Metal, Self::None];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("RenderTier", s, Self::NAMES)),
        }
    }
}

impl FromJson for RenderTier {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "RenderTier", Self::NAMES)?)
    }
}

impl ToJson for RenderTier {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `Antialias`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Antialias {
    #[default]
    Tier,
    Off,
    Msaa4,
    Taa,
    Fxaa,
    Smaa,
    SmaaT2x,
}

impl Antialias {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["tier", "off", "msaa4", "taa", "fxaa", "smaa", "smaaT2x"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Tier, Self::Off, Self::Msaa4, Self::Taa, Self::Fxaa, Self::Smaa, Self::SmaaT2x];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("Antialias", s, Self::NAMES)),
        }
    }
}

impl FromJson for Antialias {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "Antialias", Self::NAMES)?)
    }
}

impl ToJson for Antialias {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `PointShadows`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PointShadows {
    #[default]
    Auto,
    Cube,
    Paraboloid,
}

impl PointShadows {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["auto", "cube", "paraboloid"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Auto, Self::Cube, Self::Paraboloid];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("PointShadows", s, Self::NAMES)),
        }
    }
}

impl FromJson for PointShadows {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "PointShadows", Self::NAMES)?)
    }
}

impl ToJson for PointShadows {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `TaaHistory`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TaaHistory {
    #[default]
    Bilinear,
    CatmullRom,
}

impl TaaHistory {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["bilinear", "catmullRom"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Bilinear, Self::CatmullRom];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("TaaHistory", s, Self::NAMES)),
        }
    }
}

impl FromJson for TaaHistory {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "TaaHistory", Self::NAMES)?)
    }
}

impl ToJson for TaaHistory {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `DebugView`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DebugView {
    #[default]
    None,
    Physics,
    Nav,
    Bounds,
    Lod,
    Cascades,
    Ao,
    Albedo,
    IblDiffuse,
    IblSpecular,
    Direct,
    AoDepth,
    AoRaw,
    Terrain,
    TerrainLayers,
    TerrainMip,
    TerrainProjection,
    TerrainDensity,
}

impl DebugView {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["none", "physics", "nav", "bounds", "lod", "cascades", "ao", "albedo", "iblDiffuse", "iblSpecular", "direct", "aoDepth", "aoRaw", "terrain", "terrainLayers", "terrainMip", "terrainProjection", "terrainDensity"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::None, Self::Physics, Self::Nav, Self::Bounds, Self::Lod, Self::Cascades, Self::Ao, Self::Albedo, Self::IblDiffuse, Self::IblSpecular, Self::Direct, Self::AoDepth, Self::AoRaw, Self::Terrain, Self::TerrainLayers, Self::TerrainMip, Self::TerrainProjection, Self::TerrainDensity];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("DebugView", s, Self::NAMES)),
        }
    }
}

impl FromJson for DebugView {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "DebugView", Self::NAMES)?)
    }
}

impl ToJson for DebugView {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `Transparency`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Transparency {
    #[default]
    Weighted,
    LinkedList,
}

impl Transparency {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["weighted", "linkedList"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Weighted, Self::LinkedList];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("Transparency", s, Self::NAMES)),
        }
    }
}

impl FromJson for Transparency {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "Transparency", Self::NAMES)?)
    }
}

impl ToJson for Transparency {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `LitFeature`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LitFeature {
    #[default]
    SurfaceWeather,
    Gi,
    AreaLights,
    Decals,
    Caustics,
}

impl LitFeature {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["surfaceWeather", "gi", "areaLights", "decals", "caustics"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::SurfaceWeather, Self::Gi, Self::AreaLights, Self::Decals, Self::Caustics];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("LitFeature", s, Self::NAMES)),
        }
    }
}

impl FromJson for LitFeature {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "LitFeature", Self::NAMES)?)
    }
}

impl ToJson for LitFeature {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `RenderOptions`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderOptions {
    pub antialias: Antialias,
    pub hdr: bool,
    pub paper_white_nits: f32,
    pub peak_nits: f32,
    pub shadow_map_size: u32,
    pub point_shadows: PointShadows,
    pub taa_history: TaaHistory,
    pub taa_sharpen: f32,
    pub planar_reflections: u32,
    pub transparency: Transparency,
    pub transparency_layers: u32,
    pub without: Vec<LitFeature>,
    pub debug_view: DebugView,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            antialias: Default::default(),
            hdr: Default::default(),
            paper_white_nits: Default::default(),
            peak_nits: Default::default(),
            shadow_map_size: 2048,
            point_shadows: Default::default(),
            taa_history: Default::default(),
            taa_sharpen: Default::default(),
            planar_reflections: 1,
            transparency: Default::default(),
            transparency_layers: 0,
            without: Default::default(),
            debug_view: Default::default(),
        }
    }
}

impl FromJson for RenderOptions {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            antialias: field(o, "antialias", d.antialias)?,
            hdr: field(o, "hdr", d.hdr)?,
            paper_white_nits: field(o, "paperWhiteNits", d.paper_white_nits)?,
            peak_nits: field(o, "peakNits", d.peak_nits)?,
            shadow_map_size: field(o, "shadowMapSize", d.shadow_map_size)?,
            point_shadows: field(o, "pointShadows", d.point_shadows)?,
            taa_history: field(o, "taaHistory", d.taa_history)?,
            taa_sharpen: field(o, "taaSharpen", d.taa_sharpen)?,
            planar_reflections: field(o, "planarReflections", d.planar_reflections)?,
            transparency: field(o, "transparency", d.transparency)?,
            transparency_layers: field(o, "transparencyLayers", d.transparency_layers)?,
            without: field(o, "without", d.without)?,
            debug_view: field(o, "debugView", d.debug_view)?,
        })
    }
}

impl ToJson for RenderOptions {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("antialias".into(), self.antialias.to_json());
        o.insert("hdr".into(), self.hdr.to_json());
        o.insert("paperWhiteNits".into(), self.paper_white_nits.to_json());
        o.insert("peakNits".into(), self.peak_nits.to_json());
        o.insert("shadowMapSize".into(), self.shadow_map_size.to_json());
        o.insert("pointShadows".into(), self.point_shadows.to_json());
        o.insert("taaHistory".into(), self.taa_history.to_json());
        o.insert("taaSharpen".into(), self.taa_sharpen.to_json());
        o.insert("planarReflections".into(), self.planar_reflections.to_json());
        o.insert("transparency".into(), self.transparency.to_json());
        o.insert("transparencyLayers".into(), self.transparency_layers.to_json());
        o.insert("without".into(), self.without.to_json());
        o.insert("debugView".into(), self.debug_view.to_json());
        Value::Object(o)
    }
}

/// `FoliageDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct FoliageDesc {
    pub translucency: f32,
    pub dome_normals: f32,
}

impl Default for FoliageDesc {
    fn default() -> Self {
        Self {
            translucency: Default::default(),
            dome_normals: Default::default(),
        }
    }
}

impl FromJson for FoliageDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            translucency: field(o, "translucency", d.translucency)?,
            dome_normals: field(o, "domeNormals", d.dome_normals)?,
        })
    }
}

impl ToJson for FoliageDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("translucency".into(), self.translucency.to_json());
        o.insert("domeNormals".into(), self.dome_normals.to_json());
        Value::Object(o)
    }
}

/// `StandardMaterialDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct StandardMaterialDesc {
    pub foliage: FoliageDesc,
    pub color: Color,
    pub map: Option<u64>,
    pub metalness: f32,
    pub metalness_map: Option<u64>,
    pub roughness: f32,
    pub roughness_map: Option<u64>,
    pub normal_map: Option<u64>,
    pub ao_map: Option<u64>,
    pub emissive: Color,
    pub emissive_intensity: f32,
    pub emissive_map: Option<u64>,
    pub opacity: f32,
    pub transparent: bool,
    pub alpha_test: f32,
    pub side: Side,
    pub flat_shading: bool,
    pub alpha_to_coverage: bool,
    pub env_map_intensity: f32,
    pub normal_scale: Vec2,
    pub ao_map_intensity: f32,
    pub unlit: bool,
    pub toon: bool,
    pub gradient_map: Option<u64>,
    pub rim_color: Color,
    pub rim_intensity: f32,
    pub rim_power: f32,
    pub flash_color: Color,
    pub flash: f32,
    pub dissolve: f32,
    pub dissolve_map: Option<u64>,
    pub dissolve_edge_color: Color,
    pub dissolve_edge_width: f32,
    pub dissolve_scale: f32,
    pub outline_color: Color,
    pub outline_width: f32,
    pub styled: bool,
    pub clearcoat: f32,
    pub clearcoat_roughness: f32,
    pub clearcoat_map: Option<u64>,
    pub clearcoat_roughness_map: Option<u64>,
    pub clearcoat_normal_map: Option<u64>,
    pub clearcoat_normal_scale: Vec2,
    pub sheen: f32,
    pub sheen_color: Color,
    pub sheen_roughness: f32,
    pub sheen_color_map: Option<u64>,
    pub sheen_roughness_map: Option<u64>,
    pub transmission: f32,
    pub transmission_map: Option<u64>,
    pub thickness: f32,
    pub thickness_map: Option<u64>,
    pub attenuation_distance: f32,
    pub attenuation_color: Color,
    pub ior: f32,
    pub iridescence: f32,
    pub iridescence_ior: f32,
    pub iridescence_thickness_range: Vec2,
    pub iridescence_map: Option<u64>,
    pub iridescence_thickness_map: Option<u64>,
    pub anisotropy: f32,
    pub anisotropy_rotation: f32,
    pub anisotropy_map: Option<u64>,
    pub map_array: Option<u64>,
    pub map_layer: u32,
    pub normal_map_array: Option<u64>,
    pub normal_map_layer: u32,
    pub map_offset: Vec2,
    pub map_repeat: Vec2,
    pub map_rotation: f32,
    pub metalness_map_offset: Vec2,
    pub metalness_map_repeat: Vec2,
    pub metalness_map_rotation: f32,
    pub normal_map_offset: Vec2,
    pub normal_map_repeat: Vec2,
    pub normal_map_rotation: f32,
    pub emissive_map_offset: Vec2,
    pub emissive_map_repeat: Vec2,
    pub emissive_map_rotation: f32,
    pub ao_map_offset: Vec2,
    pub ao_map_repeat: Vec2,
    pub ao_map_rotation: f32,
    pub wireframe: bool,
    pub wireframe_angle: f32,
    pub wireframe_color: Color,
    pub paint_detail: f32,
    pub weathering: f32,
}

impl Default for StandardMaterialDesc {
    fn default() -> Self {
        Self {
            foliage: Default::default(),
            color: Color { r: 1.0_f32, g: 1.0_f32, b: 1.0_f32 },
            map: Default::default(),
            metalness: Default::default(),
            metalness_map: Default::default(),
            roughness: 1.0_f32,
            roughness_map: Default::default(),
            normal_map: Default::default(),
            ao_map: Default::default(),
            emissive: Default::default(),
            emissive_intensity: 1.0_f32,
            emissive_map: Default::default(),
            opacity: 1.0_f32,
            transparent: Default::default(),
            alpha_test: Default::default(),
            side: Default::default(),
            flat_shading: Default::default(),
            alpha_to_coverage: Default::default(),
            env_map_intensity: 1.0_f32,
            normal_scale: Vec2::new(1.0_f32, 1.0_f32),
            ao_map_intensity: 1.0_f32,
            unlit: Default::default(),
            toon: Default::default(),
            gradient_map: Default::default(),
            rim_color: Color { r: 1.0_f32, g: 1.0_f32, b: 1.0_f32 },
            rim_intensity: Default::default(),
            rim_power: 3.0_f32,
            flash_color: Color { r: 1.0_f32, g: 1.0_f32, b: 1.0_f32 },
            flash: Default::default(),
            dissolve: Default::default(),
            dissolve_map: Default::default(),
            dissolve_edge_color: Color { r: 4.0_f32, g: 1.5_f32, b: 0.375_f32 },
            dissolve_edge_width: 0.0625_f32,
            dissolve_scale: 4.0_f32,
            outline_color: Default::default(),
            outline_width: Default::default(),
            styled: true,
            clearcoat: Default::default(),
            clearcoat_roughness: Default::default(),
            clearcoat_map: Default::default(),
            clearcoat_roughness_map: Default::default(),
            clearcoat_normal_map: Default::default(),
            clearcoat_normal_scale: Vec2::new(1.0_f32, 1.0_f32),
            sheen: Default::default(),
            sheen_color: Default::default(),
            sheen_roughness: 1.0_f32,
            sheen_color_map: Default::default(),
            sheen_roughness_map: Default::default(),
            transmission: Default::default(),
            transmission_map: Default::default(),
            thickness: Default::default(),
            thickness_map: Default::default(),
            attenuation_distance: Default::default(),
            attenuation_color: Color { r: 1.0_f32, g: 1.0_f32, b: 1.0_f32 },
            ior: 1.5_f32,
            iridescence: Default::default(),
            iridescence_ior: 1.3_f32,
            iridescence_thickness_range: Vec2::new(100.0_f32, 400.0_f32),
            iridescence_map: Default::default(),
            iridescence_thickness_map: Default::default(),
            anisotropy: Default::default(),
            anisotropy_rotation: Default::default(),
            anisotropy_map: Default::default(),
            map_array: Default::default(),
            map_layer: Default::default(),
            normal_map_array: Default::default(),
            normal_map_layer: Default::default(),
            map_offset: Default::default(),
            map_repeat: Vec2::new(1.0_f32, 1.0_f32),
            map_rotation: Default::default(),
            metalness_map_offset: Default::default(),
            metalness_map_repeat: Vec2::new(1.0_f32, 1.0_f32),
            metalness_map_rotation: Default::default(),
            normal_map_offset: Default::default(),
            normal_map_repeat: Vec2::new(1.0_f32, 1.0_f32),
            normal_map_rotation: Default::default(),
            emissive_map_offset: Default::default(),
            emissive_map_repeat: Vec2::new(1.0_f32, 1.0_f32),
            emissive_map_rotation: Default::default(),
            ao_map_offset: Default::default(),
            ao_map_repeat: Vec2::new(1.0_f32, 1.0_f32),
            ao_map_rotation: Default::default(),
            wireframe: Default::default(),
            wireframe_angle: Default::default(),
            wireframe_color: Default::default(),
            paint_detail: Default::default(),
            weathering: 1.0_f32,
        }
    }
}

impl FromJson for StandardMaterialDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            foliage: field(o, "foliage", d.foliage)?,
            color: field(o, "color", d.color)?,
            map: field(o, "map", d.map)?,
            metalness: field(o, "metalness", d.metalness)?,
            metalness_map: field(o, "metalnessMap", d.metalness_map)?,
            roughness: field(o, "roughness", d.roughness)?,
            roughness_map: field(o, "roughnessMap", d.roughness_map)?,
            normal_map: field(o, "normalMap", d.normal_map)?,
            ao_map: field(o, "aoMap", d.ao_map)?,
            emissive: field(o, "emissive", d.emissive)?,
            emissive_intensity: field(o, "emissiveIntensity", d.emissive_intensity)?,
            emissive_map: field(o, "emissiveMap", d.emissive_map)?,
            opacity: field(o, "opacity", d.opacity)?,
            transparent: field(o, "transparent", d.transparent)?,
            alpha_test: field(o, "alphaTest", d.alpha_test)?,
            side: field(o, "side", d.side)?,
            flat_shading: field(o, "flatShading", d.flat_shading)?,
            alpha_to_coverage: field(o, "alphaToCoverage", d.alpha_to_coverage)?,
            env_map_intensity: field(o, "envMapIntensity", d.env_map_intensity)?,
            normal_scale: field(o, "normalScale", d.normal_scale)?,
            ao_map_intensity: field(o, "aoMapIntensity", d.ao_map_intensity)?,
            unlit: field(o, "unlit", d.unlit)?,
            toon: field(o, "toon", d.toon)?,
            gradient_map: field(o, "gradientMap", d.gradient_map)?,
            rim_color: field(o, "rimColor", d.rim_color)?,
            rim_intensity: field(o, "rimIntensity", d.rim_intensity)?,
            rim_power: field(o, "rimPower", d.rim_power)?,
            flash_color: field(o, "flashColor", d.flash_color)?,
            flash: field(o, "flash", d.flash)?,
            dissolve: field(o, "dissolve", d.dissolve)?,
            dissolve_map: field(o, "dissolveMap", d.dissolve_map)?,
            dissolve_edge_color: field(o, "dissolveEdgeColor", d.dissolve_edge_color)?,
            dissolve_edge_width: field(o, "dissolveEdgeWidth", d.dissolve_edge_width)?,
            dissolve_scale: field(o, "dissolveScale", d.dissolve_scale)?,
            outline_color: field(o, "outlineColor", d.outline_color)?,
            outline_width: field(o, "outlineWidth", d.outline_width)?,
            styled: field(o, "styled", d.styled)?,
            clearcoat: field(o, "clearcoat", d.clearcoat)?,
            clearcoat_roughness: field(o, "clearcoatRoughness", d.clearcoat_roughness)?,
            clearcoat_map: field(o, "clearcoatMap", d.clearcoat_map)?,
            clearcoat_roughness_map: field(o, "clearcoatRoughnessMap", d.clearcoat_roughness_map)?,
            clearcoat_normal_map: field(o, "clearcoatNormalMap", d.clearcoat_normal_map)?,
            clearcoat_normal_scale: field(o, "clearcoatNormalScale", d.clearcoat_normal_scale)?,
            sheen: field(o, "sheen", d.sheen)?,
            sheen_color: field(o, "sheenColor", d.sheen_color)?,
            sheen_roughness: field(o, "sheenRoughness", d.sheen_roughness)?,
            sheen_color_map: field(o, "sheenColorMap", d.sheen_color_map)?,
            sheen_roughness_map: field(o, "sheenRoughnessMap", d.sheen_roughness_map)?,
            transmission: field(o, "transmission", d.transmission)?,
            transmission_map: field(o, "transmissionMap", d.transmission_map)?,
            thickness: field(o, "thickness", d.thickness)?,
            thickness_map: field(o, "thicknessMap", d.thickness_map)?,
            attenuation_distance: field(o, "attenuationDistance", d.attenuation_distance)?,
            attenuation_color: field(o, "attenuationColor", d.attenuation_color)?,
            ior: field(o, "ior", d.ior)?,
            iridescence: field(o, "iridescence", d.iridescence)?,
            iridescence_ior: field(o, "iridescenceIOR", d.iridescence_ior)?,
            iridescence_thickness_range: field(o, "iridescenceThicknessRange", d.iridescence_thickness_range)?,
            iridescence_map: field(o, "iridescenceMap", d.iridescence_map)?,
            iridescence_thickness_map: field(o, "iridescenceThicknessMap", d.iridescence_thickness_map)?,
            anisotropy: field(o, "anisotropy", d.anisotropy)?,
            anisotropy_rotation: field(o, "anisotropyRotation", d.anisotropy_rotation)?,
            anisotropy_map: field(o, "anisotropyMap", d.anisotropy_map)?,
            map_array: field(o, "mapArray", d.map_array)?,
            map_layer: field(o, "mapLayer", d.map_layer)?,
            normal_map_array: field(o, "normalMapArray", d.normal_map_array)?,
            normal_map_layer: field(o, "normalMapLayer", d.normal_map_layer)?,
            map_offset: field(o, "mapOffset", d.map_offset)?,
            map_repeat: field(o, "mapRepeat", d.map_repeat)?,
            map_rotation: field(o, "mapRotation", d.map_rotation)?,
            metalness_map_offset: field(o, "metalnessMapOffset", d.metalness_map_offset)?,
            metalness_map_repeat: field(o, "metalnessMapRepeat", d.metalness_map_repeat)?,
            metalness_map_rotation: field(o, "metalnessMapRotation", d.metalness_map_rotation)?,
            normal_map_offset: field(o, "normalMapOffset", d.normal_map_offset)?,
            normal_map_repeat: field(o, "normalMapRepeat", d.normal_map_repeat)?,
            normal_map_rotation: field(o, "normalMapRotation", d.normal_map_rotation)?,
            emissive_map_offset: field(o, "emissiveMapOffset", d.emissive_map_offset)?,
            emissive_map_repeat: field(o, "emissiveMapRepeat", d.emissive_map_repeat)?,
            emissive_map_rotation: field(o, "emissiveMapRotation", d.emissive_map_rotation)?,
            ao_map_offset: field(o, "aoMapOffset", d.ao_map_offset)?,
            ao_map_repeat: field(o, "aoMapRepeat", d.ao_map_repeat)?,
            ao_map_rotation: field(o, "aoMapRotation", d.ao_map_rotation)?,
            wireframe: field(o, "wireframe", d.wireframe)?,
            wireframe_angle: field(o, "wireframeAngle", d.wireframe_angle)?,
            wireframe_color: field(o, "wireframeColor", d.wireframe_color)?,
            paint_detail: field(o, "paintDetail", d.paint_detail)?,
            weathering: field(o, "weathering", d.weathering)?,
        })
    }
}

impl ToJson for StandardMaterialDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("foliage".into(), self.foliage.to_json());
        o.insert("color".into(), self.color.to_json());
        o.insert("map".into(), self.map.to_json());
        o.insert("metalness".into(), self.metalness.to_json());
        o.insert("metalnessMap".into(), self.metalness_map.to_json());
        o.insert("roughness".into(), self.roughness.to_json());
        o.insert("roughnessMap".into(), self.roughness_map.to_json());
        o.insert("normalMap".into(), self.normal_map.to_json());
        o.insert("aoMap".into(), self.ao_map.to_json());
        o.insert("emissive".into(), self.emissive.to_json());
        o.insert("emissiveIntensity".into(), self.emissive_intensity.to_json());
        o.insert("emissiveMap".into(), self.emissive_map.to_json());
        o.insert("opacity".into(), self.opacity.to_json());
        o.insert("transparent".into(), self.transparent.to_json());
        o.insert("alphaTest".into(), self.alpha_test.to_json());
        o.insert("side".into(), self.side.to_json());
        o.insert("flatShading".into(), self.flat_shading.to_json());
        o.insert("alphaToCoverage".into(), self.alpha_to_coverage.to_json());
        o.insert("envMapIntensity".into(), self.env_map_intensity.to_json());
        o.insert("normalScale".into(), self.normal_scale.to_json());
        o.insert("aoMapIntensity".into(), self.ao_map_intensity.to_json());
        o.insert("unlit".into(), self.unlit.to_json());
        o.insert("toon".into(), self.toon.to_json());
        o.insert("gradientMap".into(), self.gradient_map.to_json());
        o.insert("rimColor".into(), self.rim_color.to_json());
        o.insert("rimIntensity".into(), self.rim_intensity.to_json());
        o.insert("rimPower".into(), self.rim_power.to_json());
        o.insert("flashColor".into(), self.flash_color.to_json());
        o.insert("flash".into(), self.flash.to_json());
        o.insert("dissolve".into(), self.dissolve.to_json());
        o.insert("dissolveMap".into(), self.dissolve_map.to_json());
        o.insert("dissolveEdgeColor".into(), self.dissolve_edge_color.to_json());
        o.insert("dissolveEdgeWidth".into(), self.dissolve_edge_width.to_json());
        o.insert("dissolveScale".into(), self.dissolve_scale.to_json());
        o.insert("outlineColor".into(), self.outline_color.to_json());
        o.insert("outlineWidth".into(), self.outline_width.to_json());
        o.insert("styled".into(), self.styled.to_json());
        o.insert("clearcoat".into(), self.clearcoat.to_json());
        o.insert("clearcoatRoughness".into(), self.clearcoat_roughness.to_json());
        o.insert("clearcoatMap".into(), self.clearcoat_map.to_json());
        o.insert("clearcoatRoughnessMap".into(), self.clearcoat_roughness_map.to_json());
        o.insert("clearcoatNormalMap".into(), self.clearcoat_normal_map.to_json());
        o.insert("clearcoatNormalScale".into(), self.clearcoat_normal_scale.to_json());
        o.insert("sheen".into(), self.sheen.to_json());
        o.insert("sheenColor".into(), self.sheen_color.to_json());
        o.insert("sheenRoughness".into(), self.sheen_roughness.to_json());
        o.insert("sheenColorMap".into(), self.sheen_color_map.to_json());
        o.insert("sheenRoughnessMap".into(), self.sheen_roughness_map.to_json());
        o.insert("transmission".into(), self.transmission.to_json());
        o.insert("transmissionMap".into(), self.transmission_map.to_json());
        o.insert("thickness".into(), self.thickness.to_json());
        o.insert("thicknessMap".into(), self.thickness_map.to_json());
        o.insert("attenuationDistance".into(), self.attenuation_distance.to_json());
        o.insert("attenuationColor".into(), self.attenuation_color.to_json());
        o.insert("ior".into(), self.ior.to_json());
        o.insert("iridescence".into(), self.iridescence.to_json());
        o.insert("iridescenceIOR".into(), self.iridescence_ior.to_json());
        o.insert("iridescenceThicknessRange".into(), self.iridescence_thickness_range.to_json());
        o.insert("iridescenceMap".into(), self.iridescence_map.to_json());
        o.insert("iridescenceThicknessMap".into(), self.iridescence_thickness_map.to_json());
        o.insert("anisotropy".into(), self.anisotropy.to_json());
        o.insert("anisotropyRotation".into(), self.anisotropy_rotation.to_json());
        o.insert("anisotropyMap".into(), self.anisotropy_map.to_json());
        o.insert("mapArray".into(), self.map_array.to_json());
        o.insert("mapLayer".into(), self.map_layer.to_json());
        o.insert("normalMapArray".into(), self.normal_map_array.to_json());
        o.insert("normalMapLayer".into(), self.normal_map_layer.to_json());
        o.insert("mapOffset".into(), self.map_offset.to_json());
        o.insert("mapRepeat".into(), self.map_repeat.to_json());
        o.insert("mapRotation".into(), self.map_rotation.to_json());
        o.insert("metalnessMapOffset".into(), self.metalness_map_offset.to_json());
        o.insert("metalnessMapRepeat".into(), self.metalness_map_repeat.to_json());
        o.insert("metalnessMapRotation".into(), self.metalness_map_rotation.to_json());
        o.insert("normalMapOffset".into(), self.normal_map_offset.to_json());
        o.insert("normalMapRepeat".into(), self.normal_map_repeat.to_json());
        o.insert("normalMapRotation".into(), self.normal_map_rotation.to_json());
        o.insert("emissiveMapOffset".into(), self.emissive_map_offset.to_json());
        o.insert("emissiveMapRepeat".into(), self.emissive_map_repeat.to_json());
        o.insert("emissiveMapRotation".into(), self.emissive_map_rotation.to_json());
        o.insert("aoMapOffset".into(), self.ao_map_offset.to_json());
        o.insert("aoMapRepeat".into(), self.ao_map_repeat.to_json());
        o.insert("aoMapRotation".into(), self.ao_map_rotation.to_json());
        o.insert("wireframe".into(), self.wireframe.to_json());
        o.insert("wireframeAngle".into(), self.wireframe_angle.to_json());
        o.insert("wireframeColor".into(), self.wireframe_color.to_json());
        o.insert("paintDetail".into(), self.paint_detail.to_json());
        o.insert("weathering".into(), self.weathering.to_json());
        Value::Object(o)
    }
}

/// `GraphicsQuality`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GraphicsQuality {
    #[default]
    Medium,
    Low,
    High,
}

impl GraphicsQuality {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["medium", "low", "high"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Medium, Self::Low, Self::High];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("GraphicsQuality", s, Self::NAMES)),
        }
    }
}

impl FromJson for GraphicsQuality {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "GraphicsQuality", Self::NAMES)?)
    }
}

impl ToJson for GraphicsQuality {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `SpriteBlend`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SpriteBlend {
    #[default]
    Alpha,
    Additive,
}

impl SpriteBlend {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["alpha", "additive"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Alpha, Self::Additive];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("SpriteBlend", s, Self::NAMES)),
        }
    }
}

impl FromJson for SpriteBlend {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "SpriteBlend", Self::NAMES)?)
    }
}

impl ToJson for SpriteBlend {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `SpriteFacing`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SpriteFacing {
    #[default]
    None,
    Camera,
    CameraY,
}

impl SpriteFacing {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["none", "camera", "cameraY"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::None, Self::Camera, Self::CameraY];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("SpriteFacing", s, Self::NAMES)),
        }
    }
}

impl FromJson for SpriteFacing {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "SpriteFacing", Self::NAMES)?)
    }
}

impl ToJson for SpriteFacing {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `StripBlend`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StripBlend {
    #[default]
    Additive,
    Alpha,
}

impl StripBlend {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["additive", "alpha"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Additive, Self::Alpha];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("StripBlend", s, Self::NAMES)),
        }
    }
}

impl FromJson for StripBlend {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "StripBlend", Self::NAMES)?)
    }
}

impl ToJson for StripBlend {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `LookInfo`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct LookInfo {
    pub look: u64,
    pub path: String,
    pub name: String,
    pub extends: Vec<String>,
    pub mood: Option<String>,
    pub resources: Vec<String>,
    pub style: Option<u64>,
    pub antialias: Option<Antialias>,
}

impl Default for LookInfo {
    fn default() -> Self {
        Self {
            look: Default::default(),
            path: Default::default(),
            name: Default::default(),
            extends: Default::default(),
            mood: Default::default(),
            resources: Default::default(),
            style: Default::default(),
            antialias: Default::default(),
        }
    }
}

impl FromJson for LookInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            look: field(o, "look", d.look)?,
            path: field(o, "path", d.path)?,
            name: field(o, "name", d.name)?,
            extends: field(o, "extends", d.extends)?,
            mood: field(o, "mood", d.mood)?,
            resources: field(o, "resources", d.resources)?,
            style: field(o, "style", d.style)?,
            antialias: field(o, "antialias", d.antialias)?,
        })
    }
}

impl ToJson for LookInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("look".into(), self.look.to_json());
        o.insert("path".into(), self.path.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("extends".into(), self.extends.to_json());
        o.insert("mood".into(), self.mood.to_json());
        o.insert("resources".into(), self.resources.to_json());
        o.insert("style".into(), self.style.to_json());
        o.insert("antialias".into(), self.antialias.to_json());
        Value::Object(o)
    }
}

/// `RibbonPoint`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RibbonPoint {
    pub position: Vec3,
    pub width: f32,
    pub color: Color,
    pub opacity: f32,
}

impl Default for RibbonPoint {
    fn default() -> Self {
        Self {
            position: Default::default(),
            width: 0.1_f32,
            color: Color { r: 1.0_f32, g: 1.0_f32, b: 1.0_f32 },
            opacity: 1.0_f32,
        }
    }
}

impl FromJson for RibbonPoint {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            position: field(o, "position", d.position)?,
            width: field(o, "width", d.width)?,
            color: field(o, "color", d.color)?,
            opacity: field(o, "opacity", d.opacity)?,
        })
    }
}

impl ToJson for RibbonPoint {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("position".into(), self.position.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("color".into(), self.color.to_json());
        o.insert("opacity".into(), self.opacity.to_json());
        Value::Object(o)
    }
}

/// `RibbonDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RibbonDesc {
    pub points: Vec<RibbonPoint>,
    pub texture: Option<u64>,
    pub uv_tile: f32,
    pub blend: StripBlend,
}

impl Default for RibbonDesc {
    fn default() -> Self {
        Self {
            points: Default::default(),
            texture: Default::default(),
            uv_tile: Default::default(),
            blend: Default::default(),
        }
    }
}

impl FromJson for RibbonDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            points: field(o, "points", d.points)?,
            texture: field(o, "texture", d.texture)?,
            uv_tile: field(o, "uvTile", d.uv_tile)?,
            blend: field(o, "blend", d.blend)?,
        })
    }
}

impl ToJson for RibbonDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("points".into(), self.points.to_json());
        o.insert("texture".into(), self.texture.to_json());
        o.insert("uvTile".into(), self.uv_tile.to_json());
        o.insert("blend".into(), self.blend.to_json());
        Value::Object(o)
    }
}

/// `TextDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TextDesc {
    pub font: Option<u64>,
    pub text: String,
    pub position: Vec3,
    pub size: f32,
    pub rotation: f32,
    pub color: Color,
    pub opacity: f32,
    pub layer: i32,
    pub align: TextAlign,
    pub outline: f32,
    pub outline_color: Color,
    pub facing: SpriteFacing,
    pub screen: bool,
    pub clip: Option<[f32; 4]>,
}

impl Default for TextDesc {
    fn default() -> Self {
        Self {
            font: Default::default(),
            text: Default::default(),
            position: Default::default(),
            size: 1.0_f32,
            rotation: Default::default(),
            color: Color { r: 1.0_f32, g: 1.0_f32, b: 1.0_f32 },
            opacity: 1.0_f32,
            layer: Default::default(),
            align: Default::default(),
            outline: Default::default(),
            outline_color: Default::default(),
            facing: Default::default(),
            screen: Default::default(),
            clip: Default::default(),
        }
    }
}

impl FromJson for TextDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            font: field(o, "font", d.font)?,
            text: field(o, "text", d.text)?,
            position: field(o, "position", d.position)?,
            size: field(o, "size", d.size)?,
            rotation: field(o, "rotation", d.rotation)?,
            color: field(o, "color", d.color)?,
            opacity: field(o, "opacity", d.opacity)?,
            layer: field(o, "layer", d.layer)?,
            align: field(o, "align", d.align)?,
            outline: field(o, "outline", d.outline)?,
            outline_color: field(o, "outlineColor", d.outline_color)?,
            facing: field(o, "facing", d.facing)?,
            screen: field(o, "screen", d.screen)?,
            clip: field(o, "clip", d.clip)?,
        })
    }
}

impl ToJson for TextDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("font".into(), self.font.to_json());
        o.insert("text".into(), self.text.to_json());
        o.insert("position".into(), self.position.to_json());
        o.insert("size".into(), self.size.to_json());
        o.insert("rotation".into(), self.rotation.to_json());
        o.insert("color".into(), self.color.to_json());
        o.insert("opacity".into(), self.opacity.to_json());
        o.insert("layer".into(), self.layer.to_json());
        o.insert("align".into(), self.align.to_json());
        o.insert("outline".into(), self.outline.to_json());
        o.insert("outlineColor".into(), self.outline_color.to_json());
        o.insert("facing".into(), self.facing.to_json());
        o.insert("screen".into(), self.screen.to_json());
        o.insert("clip".into(), self.clip.to_json());
        Value::Object(o)
    }
}

/// `TextMeasureDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TextMeasureDesc {
    pub font: Option<u64>,
    pub text: String,
    pub size: f32,
}

impl Default for TextMeasureDesc {
    fn default() -> Self {
        Self {
            font: Default::default(),
            text: Default::default(),
            size: 1.0_f32,
        }
    }
}

impl FromJson for TextMeasureDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            font: field(o, "font", d.font)?,
            text: field(o, "text", d.text)?,
            size: field(o, "size", d.size)?,
        })
    }
}

impl ToJson for TextMeasureDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("font".into(), self.font.to_json());
        o.insert("text".into(), self.text.to_json());
        o.insert("size".into(), self.size.to_json());
        Value::Object(o)
    }
}

/// `TextMetrics`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TextMetrics {
    pub width: f32,
    pub height: f32,
    pub lines: u32,
}

impl Default for TextMetrics {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
            lines: Default::default(),
        }
    }
}

impl FromJson for TextMetrics {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            lines: field(o, "lines", d.lines)?,
        })
    }
}

impl ToJson for TextMetrics {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("lines".into(), self.lines.to_json());
        Value::Object(o)
    }
}

/// `SpriteDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteDesc {
    pub texture: Option<u64>,
    pub array: Option<u64>,
    pub array_layer: u32,
    pub position: Vec3,
    pub width: f32,
    pub height: f32,
    pub anchor: Vec2,
    pub rotation: f32,
    pub color: Color,
    pub opacity: f32,
    pub uv: Option<[f32; 4]>,
    pub layer: i32,
    pub flash: f32,
    pub flash_color: Color,
    pub glow: f32,
    pub blend: SpriteBlend,
    pub cast_shadow: bool,
    pub drop_shadow: f32,
    pub drop_shadow_height: f32,
    pub tone_mapped: bool,
    pub facing: SpriteFacing,
    pub flip_x: bool,
    pub flip_y: bool,
    pub sort_y: bool,
    pub slice: Option<[f32; 4]>,
    pub slice_scale: f32,
    pub screen: bool,
    pub clip: Option<[f32; 4]>,
}

impl Default for SpriteDesc {
    fn default() -> Self {
        Self {
            texture: Default::default(),
            array: Default::default(),
            array_layer: Default::default(),
            position: Default::default(),
            width: 1.0_f32,
            height: 1.0_f32,
            anchor: Vec2::new(0.5_f32, 0.5_f32),
            rotation: Default::default(),
            color: Color { r: 1.0_f32, g: 1.0_f32, b: 1.0_f32 },
            opacity: 1.0_f32,
            uv: Default::default(),
            layer: Default::default(),
            flash: Default::default(),
            flash_color: Color { r: 1.0_f32, g: 1.0_f32, b: 1.0_f32 },
            glow: Default::default(),
            blend: Default::default(),
            cast_shadow: Default::default(),
            drop_shadow: Default::default(),
            drop_shadow_height: Default::default(),
            tone_mapped: true,
            facing: Default::default(),
            flip_x: Default::default(),
            flip_y: Default::default(),
            sort_y: Default::default(),
            slice: Default::default(),
            slice_scale: 0.0625_f32,
            screen: Default::default(),
            clip: Default::default(),
        }
    }
}

impl FromJson for SpriteDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            texture: field(o, "texture", d.texture)?,
            array: field(o, "array", d.array)?,
            array_layer: field(o, "arrayLayer", d.array_layer)?,
            position: field(o, "position", d.position)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            anchor: field(o, "anchor", d.anchor)?,
            rotation: field(o, "rotation", d.rotation)?,
            color: field(o, "color", d.color)?,
            opacity: field(o, "opacity", d.opacity)?,
            uv: field(o, "uv", d.uv)?,
            layer: field(o, "layer", d.layer)?,
            flash: field(o, "flash", d.flash)?,
            flash_color: field(o, "flashColor", d.flash_color)?,
            glow: field(o, "glow", d.glow)?,
            blend: field(o, "blend", d.blend)?,
            cast_shadow: field(o, "castShadow", d.cast_shadow)?,
            drop_shadow: field(o, "dropShadow", d.drop_shadow)?,
            drop_shadow_height: field(o, "dropShadowHeight", d.drop_shadow_height)?,
            tone_mapped: field(o, "toneMapped", d.tone_mapped)?,
            facing: field(o, "facing", d.facing)?,
            flip_x: field(o, "flipX", d.flip_x)?,
            flip_y: field(o, "flipY", d.flip_y)?,
            sort_y: field(o, "sortY", d.sort_y)?,
            slice: field(o, "slice", d.slice)?,
            slice_scale: field(o, "sliceScale", d.slice_scale)?,
            screen: field(o, "screen", d.screen)?,
            clip: field(o, "clip", d.clip)?,
        })
    }
}

impl ToJson for SpriteDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("texture".into(), self.texture.to_json());
        o.insert("array".into(), self.array.to_json());
        o.insert("arrayLayer".into(), self.array_layer.to_json());
        o.insert("position".into(), self.position.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("anchor".into(), self.anchor.to_json());
        o.insert("rotation".into(), self.rotation.to_json());
        o.insert("color".into(), self.color.to_json());
        o.insert("opacity".into(), self.opacity.to_json());
        o.insert("uv".into(), self.uv.to_json());
        o.insert("layer".into(), self.layer.to_json());
        o.insert("flash".into(), self.flash.to_json());
        o.insert("flashColor".into(), self.flash_color.to_json());
        o.insert("glow".into(), self.glow.to_json());
        o.insert("blend".into(), self.blend.to_json());
        o.insert("castShadow".into(), self.cast_shadow.to_json());
        o.insert("dropShadow".into(), self.drop_shadow.to_json());
        o.insert("dropShadowHeight".into(), self.drop_shadow_height.to_json());
        o.insert("toneMapped".into(), self.tone_mapped.to_json());
        o.insert("facing".into(), self.facing.to_json());
        o.insert("flipX".into(), self.flip_x.to_json());
        o.insert("flipY".into(), self.flip_y.to_json());
        o.insert("sortY".into(), self.sort_y.to_json());
        o.insert("slice".into(), self.slice.to_json());
        o.insert("sliceScale".into(), self.slice_scale.to_json());
        o.insert("screen".into(), self.screen.to_json());
        o.insert("clip".into(), self.clip.to_json());
        Value::Object(o)
    }
}

/// `FloatingTextDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct FloatingTextDesc {
    pub text: String,
    pub position: Vec3,
    pub font: Option<u64>,
    pub size: f32,
    pub color: Color,
    pub outline: f32,
    pub outline_color: Color,
    pub rise: f32,
    pub spread: f32,
    pub lifetime: f32,
    pub fade: f32,
    pub pop: f32,
    pub crit: bool,
    pub crit_scale: f32,
    pub layer: i32,
    pub facing: SpriteFacing,
}

impl Default for FloatingTextDesc {
    fn default() -> Self {
        Self {
            text: Default::default(),
            position: Default::default(),
            font: Default::default(),
            size: 0.5_f32,
            color: Color { r: 1.0_f32, g: 1.0_f32, b: 1.0_f32 },
            outline: 0.6_f32,
            outline_color: Color { r: 0.02_f32, g: 0.02_f32, b: 0.03_f32 },
            rise: 1.0_f32,
            spread: 0.3_f32,
            lifetime: 0.8_f32,
            fade: 0.4_f32,
            pop: 0.4_f32,
            crit: Default::default(),
            crit_scale: 1.6_f32,
            layer: 100,
            facing: Default::default(),
        }
    }
}

impl FromJson for FloatingTextDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            text: field(o, "text", d.text)?,
            position: field(o, "position", d.position)?,
            font: field(o, "font", d.font)?,
            size: field(o, "size", d.size)?,
            color: field(o, "color", d.color)?,
            outline: field(o, "outline", d.outline)?,
            outline_color: field(o, "outlineColor", d.outline_color)?,
            rise: field(o, "rise", d.rise)?,
            spread: field(o, "spread", d.spread)?,
            lifetime: field(o, "lifetime", d.lifetime)?,
            fade: field(o, "fade", d.fade)?,
            pop: field(o, "pop", d.pop)?,
            crit: field(o, "crit", d.crit)?,
            crit_scale: field(o, "critScale", d.crit_scale)?,
            layer: field(o, "layer", d.layer)?,
            facing: field(o, "facing", d.facing)?,
        })
    }
}

impl ToJson for FloatingTextDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("text".into(), self.text.to_json());
        o.insert("position".into(), self.position.to_json());
        o.insert("font".into(), self.font.to_json());
        o.insert("size".into(), self.size.to_json());
        o.insert("color".into(), self.color.to_json());
        o.insert("outline".into(), self.outline.to_json());
        o.insert("outlineColor".into(), self.outline_color.to_json());
        o.insert("rise".into(), self.rise.to_json());
        o.insert("spread".into(), self.spread.to_json());
        o.insert("lifetime".into(), self.lifetime.to_json());
        o.insert("fade".into(), self.fade.to_json());
        o.insert("pop".into(), self.pop.to_json());
        o.insert("crit".into(), self.crit.to_json());
        o.insert("critScale".into(), self.crit_scale.to_json());
        o.insert("layer".into(), self.layer.to_json());
        o.insert("facing".into(), self.facing.to_json());
        Value::Object(o)
    }
}

/// `BoxGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct BoxGeometryDesc {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub uv_repeat: Vec2,
}

impl Default for BoxGeometryDesc {
    fn default() -> Self {
        Self {
            width: 1.0_f32,
            height: 1.0_f32,
            depth: 1.0_f32,
            uv_repeat: Vec2::new(1.0_f32, 1.0_f32),
        }
    }
}

impl FromJson for BoxGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            depth: field(o, "depth", d.depth)?,
            uv_repeat: field(o, "uvRepeat", d.uv_repeat)?,
        })
    }
}

impl ToJson for BoxGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("depth".into(), self.depth.to_json());
        o.insert("uvRepeat".into(), self.uv_repeat.to_json());
        Value::Object(o)
    }
}

/// `PlaneGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PlaneGeometryDesc {
    pub width: f32,
    pub height: f32,
    pub uv_repeat: Vec2,
}

impl Default for PlaneGeometryDesc {
    fn default() -> Self {
        Self {
            width: 1.0_f32,
            height: 1.0_f32,
            uv_repeat: Vec2::new(1.0_f32, 1.0_f32),
        }
    }
}

impl FromJson for PlaneGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            uv_repeat: field(o, "uvRepeat", d.uv_repeat)?,
        })
    }
}

impl ToJson for PlaneGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("uvRepeat".into(), self.uv_repeat.to_json());
        Value::Object(o)
    }
}

/// `SphereGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SphereGeometryDesc {
    pub radius: f32,
    pub width_segments: u32,
    pub height_segments: u32,
    pub uv_repeat: Vec2,
}

impl Default for SphereGeometryDesc {
    fn default() -> Self {
        Self {
            radius: 1.0_f32,
            width_segments: 32,
            height_segments: 16,
            uv_repeat: Vec2::new(1.0_f32, 1.0_f32),
        }
    }
}

impl FromJson for SphereGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            radius: field(o, "radius", d.radius)?,
            width_segments: field(o, "widthSegments", d.width_segments)?,
            height_segments: field(o, "heightSegments", d.height_segments)?,
            uv_repeat: field(o, "uvRepeat", d.uv_repeat)?,
        })
    }
}

impl ToJson for SphereGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("radius".into(), self.radius.to_json());
        o.insert("widthSegments".into(), self.width_segments.to_json());
        o.insert("heightSegments".into(), self.height_segments.to_json());
        o.insert("uvRepeat".into(), self.uv_repeat.to_json());
        Value::Object(o)
    }
}

/// `RenderCapabilities`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderCapabilities {
    pub tier: RenderTier,
    pub compute: bool,
    pub antialias: Antialias,
    pub hdr_supported: bool,
    pub hdr: bool,
    pub max_lights: u32,
    pub max_shadowed_lights: u32,
    pub max_msaa_samples: u32,
    pub point_shadows: PointShadows,
    pub width: u32,
    pub height: u32,
    pub stylise: Vec<String>,
    pub ssr: bool,
    pub reflection_probes: u32,
    pub render_target_depth: bool,
    pub max_render_target_size: u32,
    pub max_custom_posts: u32,
    pub volumetric_fog: VolumetricFog,
    pub volumetric_lights: u32,
    pub max_light_cookies: u32,
    pub gpu_timestamps: bool,
    pub sprite_anisotropy: u32,
    pub planar_reflections: u32,
    pub without: Vec<LitFeature>,
    pub upscalers: Vec<String>,
    pub transparency: Transparency,
    pub gpu_crowds: bool,
    pub exr_decode: bool,
    pub visibility_buffer: bool,
    pub visibility_buffer_refused: Option<String>,
    pub visual_simulation: bool,
    pub world3d: bool,
}

impl Default for RenderCapabilities {
    fn default() -> Self {
        Self {
            tier: Default::default(),
            compute: Default::default(),
            antialias: Default::default(),
            hdr_supported: Default::default(),
            hdr: Default::default(),
            max_lights: Default::default(),
            max_shadowed_lights: Default::default(),
            max_msaa_samples: Default::default(),
            point_shadows: Default::default(),
            width: Default::default(),
            height: Default::default(),
            stylise: Default::default(),
            ssr: Default::default(),
            reflection_probes: Default::default(),
            render_target_depth: Default::default(),
            max_render_target_size: Default::default(),
            max_custom_posts: Default::default(),
            volumetric_fog: Default::default(),
            volumetric_lights: Default::default(),
            max_light_cookies: Default::default(),
            gpu_timestamps: Default::default(),
            sprite_anisotropy: Default::default(),
            planar_reflections: Default::default(),
            without: Default::default(),
            upscalers: Default::default(),
            transparency: Default::default(),
            gpu_crowds: Default::default(),
            exr_decode: Default::default(),
            visibility_buffer: Default::default(),
            visibility_buffer_refused: Default::default(),
            visual_simulation: Default::default(),
            world3d: Default::default(),
        }
    }
}

impl FromJson for RenderCapabilities {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tier: field(o, "tier", d.tier)?,
            compute: field(o, "compute", d.compute)?,
            antialias: field(o, "antialias", d.antialias)?,
            hdr_supported: field(o, "hdrSupported", d.hdr_supported)?,
            hdr: field(o, "hdr", d.hdr)?,
            max_lights: field(o, "maxLights", d.max_lights)?,
            max_shadowed_lights: field(o, "maxShadowedLights", d.max_shadowed_lights)?,
            max_msaa_samples: field(o, "maxMsaaSamples", d.max_msaa_samples)?,
            point_shadows: field(o, "pointShadows", d.point_shadows)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            stylise: field(o, "stylise", d.stylise)?,
            ssr: field(o, "ssr", d.ssr)?,
            reflection_probes: field(o, "reflectionProbes", d.reflection_probes)?,
            render_target_depth: field(o, "renderTargetDepth", d.render_target_depth)?,
            max_render_target_size: field(o, "maxRenderTargetSize", d.max_render_target_size)?,
            max_custom_posts: field(o, "maxCustomPosts", d.max_custom_posts)?,
            volumetric_fog: field(o, "volumetricFog", d.volumetric_fog)?,
            volumetric_lights: field(o, "volumetricLights", d.volumetric_lights)?,
            max_light_cookies: field(o, "maxLightCookies", d.max_light_cookies)?,
            gpu_timestamps: field(o, "gpuTimestamps", d.gpu_timestamps)?,
            sprite_anisotropy: field(o, "spriteAnisotropy", d.sprite_anisotropy)?,
            planar_reflections: field(o, "planarReflections", d.planar_reflections)?,
            without: field(o, "without", d.without)?,
            upscalers: field(o, "upscalers", d.upscalers)?,
            transparency: field(o, "transparency", d.transparency)?,
            gpu_crowds: field(o, "gpuCrowds", d.gpu_crowds)?,
            exr_decode: field(o, "exrDecode", d.exr_decode)?,
            visibility_buffer: field(o, "visibilityBuffer", d.visibility_buffer)?,
            visibility_buffer_refused: field(o, "visibilityBufferRefused", d.visibility_buffer_refused)?,
            visual_simulation: field(o, "visualSimulation", d.visual_simulation)?,
            world3d: field(o, "world3d", d.world3d)?,
        })
    }
}

impl ToJson for RenderCapabilities {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tier".into(), self.tier.to_json());
        o.insert("compute".into(), self.compute.to_json());
        o.insert("antialias".into(), self.antialias.to_json());
        o.insert("hdrSupported".into(), self.hdr_supported.to_json());
        o.insert("hdr".into(), self.hdr.to_json());
        o.insert("maxLights".into(), self.max_lights.to_json());
        o.insert("maxShadowedLights".into(), self.max_shadowed_lights.to_json());
        o.insert("maxMsaaSamples".into(), self.max_msaa_samples.to_json());
        o.insert("pointShadows".into(), self.point_shadows.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("stylise".into(), self.stylise.to_json());
        o.insert("ssr".into(), self.ssr.to_json());
        o.insert("reflectionProbes".into(), self.reflection_probes.to_json());
        o.insert("renderTargetDepth".into(), self.render_target_depth.to_json());
        o.insert("maxRenderTargetSize".into(), self.max_render_target_size.to_json());
        o.insert("maxCustomPosts".into(), self.max_custom_posts.to_json());
        o.insert("volumetricFog".into(), self.volumetric_fog.to_json());
        o.insert("volumetricLights".into(), self.volumetric_lights.to_json());
        o.insert("maxLightCookies".into(), self.max_light_cookies.to_json());
        o.insert("gpuTimestamps".into(), self.gpu_timestamps.to_json());
        o.insert("spriteAnisotropy".into(), self.sprite_anisotropy.to_json());
        o.insert("planarReflections".into(), self.planar_reflections.to_json());
        o.insert("without".into(), self.without.to_json());
        o.insert("upscalers".into(), self.upscalers.to_json());
        o.insert("transparency".into(), self.transparency.to_json());
        o.insert("gpuCrowds".into(), self.gpu_crowds.to_json());
        o.insert("exrDecode".into(), self.exr_decode.to_json());
        o.insert("visibilityBuffer".into(), self.visibility_buffer.to_json());
        o.insert("visibilityBufferRefused".into(), self.visibility_buffer_refused.to_json());
        o.insert("visualSimulation".into(), self.visual_simulation.to_json());
        o.insert("world3d".into(), self.world3d.to_json());
        Value::Object(o)
    }
}

/// `FrameCap`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FrameCap {
    #[default]
    Auto,
    Display,
    Fps60,
    Fps30,
    Fps120,
    Fps144,
}

impl FrameCap {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["auto", "display", "fps60", "fps30", "fps120", "fps144"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Auto, Self::Display, Self::Fps60, Self::Fps30, Self::Fps120, Self::Fps144];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("FrameCap", s, Self::NAMES)),
        }
    }
}

impl FromJson for FrameCap {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "FrameCap", Self::NAMES)?)
    }
}

impl ToJson for FrameCap {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `PowerPreference`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PowerPreference {
    #[default]
    HighPerformance,
    LowPower,
}

impl PowerPreference {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["highPerformance", "lowPower"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::HighPerformance, Self::LowPower];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("PowerPreference", s, Self::NAMES)),
        }
    }
}

impl FromJson for PowerPreference {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "PowerPreference", Self::NAMES)?)
    }
}

impl ToJson for PowerPreference {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `DisplaySettings`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DisplaySettings {
    pub quality: Option<GraphicsQuality>,
    pub frame_cap: FrameCap,
    pub battery_saver: bool,
    pub render_scale: f32,
    pub auto_quality: bool,
    pub auto_min_scale: f32,
    pub cloth_subdivisions: u32,
    pub auto_preset_steps: u32,
    pub fullscreen: bool,
    pub power_preference: PowerPreference,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            quality: Default::default(),
            frame_cap: Default::default(),
            battery_saver: Default::default(),
            render_scale: 1.0_f32,
            auto_quality: true,
            auto_min_scale: 0.5_f32,
            cloth_subdivisions: 0,
            auto_preset_steps: 1,
            fullscreen: Default::default(),
            power_preference: Default::default(),
        }
    }
}

impl FromJson for DisplaySettings {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            quality: field(o, "quality", d.quality)?,
            frame_cap: field(o, "frameCap", d.frame_cap)?,
            battery_saver: field(o, "batterySaver", d.battery_saver)?,
            render_scale: field(o, "renderScale", d.render_scale)?,
            auto_quality: field(o, "autoQuality", d.auto_quality)?,
            auto_min_scale: field(o, "autoMinScale", d.auto_min_scale)?,
            cloth_subdivisions: field(o, "clothSubdivisions", d.cloth_subdivisions)?,
            auto_preset_steps: field(o, "autoPresetSteps", d.auto_preset_steps)?,
            fullscreen: field(o, "fullscreen", d.fullscreen)?,
            power_preference: field(o, "powerPreference", d.power_preference)?,
        })
    }
}

impl ToJson for DisplaySettings {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("quality".into(), self.quality.to_json());
        o.insert("frameCap".into(), self.frame_cap.to_json());
        o.insert("batterySaver".into(), self.battery_saver.to_json());
        o.insert("renderScale".into(), self.render_scale.to_json());
        o.insert("autoQuality".into(), self.auto_quality.to_json());
        o.insert("autoMinScale".into(), self.auto_min_scale.to_json());
        o.insert("clothSubdivisions".into(), self.cloth_subdivisions.to_json());
        o.insert("autoPresetSteps".into(), self.auto_preset_steps.to_json());
        o.insert("fullscreen".into(), self.fullscreen.to_json());
        o.insert("powerPreference".into(), self.power_preference.to_json());
        Value::Object(o)
    }
}

/// `GpuMemory`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct GpuMemory {
    pub textures: f64,
    pub meshes: f64,
    pub render_targets: f64,
    pub buffers: f64,
    pub total: f64,
}

impl Default for GpuMemory {
    fn default() -> Self {
        Self {
            textures: Default::default(),
            meshes: Default::default(),
            render_targets: Default::default(),
            buffers: Default::default(),
            total: Default::default(),
        }
    }
}

impl FromJson for GpuMemory {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            textures: field(o, "textures", d.textures)?,
            meshes: field(o, "meshes", d.meshes)?,
            render_targets: field(o, "renderTargets", d.render_targets)?,
            buffers: field(o, "buffers", d.buffers)?,
            total: field(o, "total", d.total)?,
        })
    }
}

impl ToJson for GpuMemory {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("textures".into(), self.textures.to_json());
        o.insert("meshes".into(), self.meshes.to_json());
        o.insert("renderTargets".into(), self.render_targets.to_json());
        o.insert("buffers".into(), self.buffers.to_json());
        o.insert("total".into(), self.total.to_json());
        Value::Object(o)
    }
}

/// `DisplayInfo`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DisplayInfo {
    pub settings: DisplaySettings,
    pub presenting: bool,
    pub max_fps: u32,
    pub display_hz: f32,
    pub render_scale: f32,
    pub preset_steps: u32,
    pub scene_width: u32,
    pub scene_height: u32,
    pub frame_ms: f32,
    pub device_losses: u32,
    pub gpu: Option<GpuMemory>,
    pub vsync: bool,
    pub fullscreen: bool,
    pub power_preference: PowerPreference,
    pub restart_needed: bool,
}

impl Default for DisplayInfo {
    fn default() -> Self {
        Self {
            settings: Default::default(),
            presenting: Default::default(),
            max_fps: Default::default(),
            display_hz: Default::default(),
            render_scale: Default::default(),
            preset_steps: Default::default(),
            scene_width: Default::default(),
            scene_height: Default::default(),
            frame_ms: Default::default(),
            device_losses: Default::default(),
            gpu: Default::default(),
            vsync: Default::default(),
            fullscreen: Default::default(),
            power_preference: Default::default(),
            restart_needed: Default::default(),
        }
    }
}

impl FromJson for DisplayInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            settings: field(o, "settings", d.settings)?,
            presenting: field(o, "presenting", d.presenting)?,
            max_fps: field(o, "maxFps", d.max_fps)?,
            display_hz: field(o, "displayHz", d.display_hz)?,
            render_scale: field(o, "renderScale", d.render_scale)?,
            preset_steps: field(o, "presetSteps", d.preset_steps)?,
            scene_width: field(o, "sceneWidth", d.scene_width)?,
            scene_height: field(o, "sceneHeight", d.scene_height)?,
            frame_ms: field(o, "frameMs", d.frame_ms)?,
            device_losses: field(o, "deviceLosses", d.device_losses)?,
            gpu: field(o, "gpu", d.gpu)?,
            vsync: field(o, "vsync", d.vsync)?,
            fullscreen: field(o, "fullscreen", d.fullscreen)?,
            power_preference: field(o, "powerPreference", d.power_preference)?,
            restart_needed: field(o, "restartNeeded", d.restart_needed)?,
        })
    }
}

impl ToJson for DisplayInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("settings".into(), self.settings.to_json());
        o.insert("presenting".into(), self.presenting.to_json());
        o.insert("maxFps".into(), self.max_fps.to_json());
        o.insert("displayHz".into(), self.display_hz.to_json());
        o.insert("renderScale".into(), self.render_scale.to_json());
        o.insert("presetSteps".into(), self.preset_steps.to_json());
        o.insert("sceneWidth".into(), self.scene_width.to_json());
        o.insert("sceneHeight".into(), self.scene_height.to_json());
        o.insert("frameMs".into(), self.frame_ms.to_json());
        o.insert("deviceLosses".into(), self.device_losses.to_json());
        o.insert("gpu".into(), self.gpu.to_json());
        o.insert("vsync".into(), self.vsync.to_json());
        o.insert("fullscreen".into(), self.fullscreen.to_json());
        o.insert("powerPreference".into(), self.power_preference.to_json());
        o.insert("restartNeeded".into(), self.restart_needed.to_json());
        Value::Object(o)
    }
}

/// `BufferGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct BufferGeometryDesc {
    pub positions: Vec<f32>,
    pub normals: Option<Vec<f32>>,
    pub uvs: Option<Vec<f32>>,
    pub colors: Option<Vec<f32>>,
    pub tangents: Option<Vec<f32>>,
    pub indices: Option<Vec<u32>>,
    pub dynamic: bool,
    pub physics: bool,
}

impl Default for BufferGeometryDesc {
    fn default() -> Self {
        Self {
            positions: Default::default(),
            normals: Default::default(),
            uvs: Default::default(),
            colors: Default::default(),
            tangents: Default::default(),
            indices: Default::default(),
            dynamic: Default::default(),
            physics: true,
        }
    }
}

impl FromJson for BufferGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            positions: field(o, "positions", d.positions)?,
            normals: field(o, "normals", d.normals)?,
            uvs: field(o, "uvs", d.uvs)?,
            colors: field(o, "colors", d.colors)?,
            tangents: field(o, "tangents", d.tangents)?,
            indices: field(o, "indices", d.indices)?,
            dynamic: field(o, "dynamic", d.dynamic)?,
            physics: field(o, "physics", d.physics)?,
        })
    }
}

impl ToJson for BufferGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("positions".into(), self.positions.to_json());
        o.insert("normals".into(), self.normals.to_json());
        o.insert("uvs".into(), self.uvs.to_json());
        o.insert("colors".into(), self.colors.to_json());
        o.insert("tangents".into(), self.tangents.to_json());
        o.insert("indices".into(), self.indices.to_json());
        o.insert("dynamic".into(), self.dynamic.to_json());
        o.insert("physics".into(), self.physics.to_json());
        Value::Object(o)
    }
}

/// `GeometryInfo`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryInfo {
    pub vertices: u32,
    pub triangles: u32,
    pub min: Vec3,
    pub max: Vec3,
    pub dynamic: bool,
}

impl Default for GeometryInfo {
    fn default() -> Self {
        Self {
            vertices: Default::default(),
            triangles: Default::default(),
            min: Default::default(),
            max: Default::default(),
            dynamic: Default::default(),
        }
    }
}

impl FromJson for GeometryInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            vertices: field(o, "vertices", d.vertices)?,
            triangles: field(o, "triangles", d.triangles)?,
            min: field(o, "min", d.min)?,
            max: field(o, "max", d.max)?,
            dynamic: field(o, "dynamic", d.dynamic)?,
        })
    }
}

impl ToJson for GeometryInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("vertices".into(), self.vertices.to_json());
        o.insert("triangles".into(), self.triangles.to_json());
        o.insert("min".into(), self.min.to_json());
        o.insert("max".into(), self.max.to_json());
        o.insert("dynamic".into(), self.dynamic.to_json());
        Value::Object(o)
    }
}

/// `CylinderGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct CylinderGeometryDesc {
    pub radius_top: f32,
    pub radius_bottom: f32,
    pub height: f32,
    pub radial_segments: u32,
    pub height_segments: u32,
    pub open_ended: bool,
    pub theta_start: f32,
    pub theta_length: f32,
    pub uv_repeat: Vec2,
}

impl Default for CylinderGeometryDesc {
    fn default() -> Self {
        Self {
            radius_top: 1.0_f32,
            radius_bottom: 1.0_f32,
            height: 1.0_f32,
            radial_segments: 32,
            height_segments: 1,
            open_ended: Default::default(),
            theta_start: Default::default(),
            theta_length: 6.283185307179586_f32,
            uv_repeat: Vec2::new(1.0_f32, 1.0_f32),
        }
    }
}

impl FromJson for CylinderGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            radius_top: field(o, "radiusTop", d.radius_top)?,
            radius_bottom: field(o, "radiusBottom", d.radius_bottom)?,
            height: field(o, "height", d.height)?,
            radial_segments: field(o, "radialSegments", d.radial_segments)?,
            height_segments: field(o, "heightSegments", d.height_segments)?,
            open_ended: field(o, "openEnded", d.open_ended)?,
            theta_start: field(o, "thetaStart", d.theta_start)?,
            theta_length: field(o, "thetaLength", d.theta_length)?,
            uv_repeat: field(o, "uvRepeat", d.uv_repeat)?,
        })
    }
}

impl ToJson for CylinderGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("radiusTop".into(), self.radius_top.to_json());
        o.insert("radiusBottom".into(), self.radius_bottom.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("radialSegments".into(), self.radial_segments.to_json());
        o.insert("heightSegments".into(), self.height_segments.to_json());
        o.insert("openEnded".into(), self.open_ended.to_json());
        o.insert("thetaStart".into(), self.theta_start.to_json());
        o.insert("thetaLength".into(), self.theta_length.to_json());
        o.insert("uvRepeat".into(), self.uv_repeat.to_json());
        Value::Object(o)
    }
}

/// `ConeGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ConeGeometryDesc {
    pub radius: f32,
    pub height: f32,
    pub radial_segments: u32,
    pub height_segments: u32,
    pub open_ended: bool,
    pub uv_repeat: Vec2,
}

impl Default for ConeGeometryDesc {
    fn default() -> Self {
        Self {
            radius: 1.0_f32,
            height: 1.0_f32,
            radial_segments: 32,
            height_segments: 1,
            open_ended: Default::default(),
            uv_repeat: Vec2::new(1.0_f32, 1.0_f32),
        }
    }
}

impl FromJson for ConeGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            radius: field(o, "radius", d.radius)?,
            height: field(o, "height", d.height)?,
            radial_segments: field(o, "radialSegments", d.radial_segments)?,
            height_segments: field(o, "heightSegments", d.height_segments)?,
            open_ended: field(o, "openEnded", d.open_ended)?,
            uv_repeat: field(o, "uvRepeat", d.uv_repeat)?,
        })
    }
}

impl ToJson for ConeGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("radius".into(), self.radius.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("radialSegments".into(), self.radial_segments.to_json());
        o.insert("heightSegments".into(), self.height_segments.to_json());
        o.insert("openEnded".into(), self.open_ended.to_json());
        o.insert("uvRepeat".into(), self.uv_repeat.to_json());
        Value::Object(o)
    }
}

/// `CapsuleGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct CapsuleGeometryDesc {
    pub radius: f32,
    pub height: f32,
    pub cap_segments: u32,
    pub radial_segments: u32,
    pub height_segments: u32,
    pub uv_repeat: Vec2,
}

impl Default for CapsuleGeometryDesc {
    fn default() -> Self {
        Self {
            radius: 1.0_f32,
            height: 1.0_f32,
            cap_segments: 4,
            radial_segments: 8,
            height_segments: 1,
            uv_repeat: Vec2::new(1.0_f32, 1.0_f32),
        }
    }
}

impl FromJson for CapsuleGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            radius: field(o, "radius", d.radius)?,
            height: field(o, "height", d.height)?,
            cap_segments: field(o, "capSegments", d.cap_segments)?,
            radial_segments: field(o, "radialSegments", d.radial_segments)?,
            height_segments: field(o, "heightSegments", d.height_segments)?,
            uv_repeat: field(o, "uvRepeat", d.uv_repeat)?,
        })
    }
}

impl ToJson for CapsuleGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("radius".into(), self.radius.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("capSegments".into(), self.cap_segments.to_json());
        o.insert("radialSegments".into(), self.radial_segments.to_json());
        o.insert("heightSegments".into(), self.height_segments.to_json());
        o.insert("uvRepeat".into(), self.uv_repeat.to_json());
        Value::Object(o)
    }
}

/// `TorusGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TorusGeometryDesc {
    pub radius: f32,
    pub tube: f32,
    pub radial_segments: u32,
    pub tubular_segments: u32,
    pub arc: f32,
    pub uv_repeat: Vec2,
}

impl Default for TorusGeometryDesc {
    fn default() -> Self {
        Self {
            radius: 1.0_f32,
            tube: 0.4_f32,
            radial_segments: 12,
            tubular_segments: 48,
            arc: 6.283185307179586_f32,
            uv_repeat: Vec2::new(1.0_f32, 1.0_f32),
        }
    }
}

impl FromJson for TorusGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            radius: field(o, "radius", d.radius)?,
            tube: field(o, "tube", d.tube)?,
            radial_segments: field(o, "radialSegments", d.radial_segments)?,
            tubular_segments: field(o, "tubularSegments", d.tubular_segments)?,
            arc: field(o, "arc", d.arc)?,
            uv_repeat: field(o, "uvRepeat", d.uv_repeat)?,
        })
    }
}

impl ToJson for TorusGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("radius".into(), self.radius.to_json());
        o.insert("tube".into(), self.tube.to_json());
        o.insert("radialSegments".into(), self.radial_segments.to_json());
        o.insert("tubularSegments".into(), self.tubular_segments.to_json());
        o.insert("arc".into(), self.arc.to_json());
        o.insert("uvRepeat".into(), self.uv_repeat.to_json());
        Value::Object(o)
    }
}

/// `RingGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RingGeometryDesc {
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub theta_segments: u32,
    pub phi_segments: u32,
    pub theta_start: f32,
    pub theta_length: f32,
    pub uv_repeat: Vec2,
}

impl Default for RingGeometryDesc {
    fn default() -> Self {
        Self {
            inner_radius: 0.5_f32,
            outer_radius: 1.0_f32,
            theta_segments: 32,
            phi_segments: 1,
            theta_start: Default::default(),
            theta_length: 6.283185307179586_f32,
            uv_repeat: Vec2::new(1.0_f32, 1.0_f32),
        }
    }
}

impl FromJson for RingGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            inner_radius: field(o, "innerRadius", d.inner_radius)?,
            outer_radius: field(o, "outerRadius", d.outer_radius)?,
            theta_segments: field(o, "thetaSegments", d.theta_segments)?,
            phi_segments: field(o, "phiSegments", d.phi_segments)?,
            theta_start: field(o, "thetaStart", d.theta_start)?,
            theta_length: field(o, "thetaLength", d.theta_length)?,
            uv_repeat: field(o, "uvRepeat", d.uv_repeat)?,
        })
    }
}

impl ToJson for RingGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("innerRadius".into(), self.inner_radius.to_json());
        o.insert("outerRadius".into(), self.outer_radius.to_json());
        o.insert("thetaSegments".into(), self.theta_segments.to_json());
        o.insert("phiSegments".into(), self.phi_segments.to_json());
        o.insert("thetaStart".into(), self.theta_start.to_json());
        o.insert("thetaLength".into(), self.theta_length.to_json());
        o.insert("uvRepeat".into(), self.uv_repeat.to_json());
        Value::Object(o)
    }
}

/// `CircleGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct CircleGeometryDesc {
    pub radius: f32,
    pub segments: u32,
    pub theta_start: f32,
    pub theta_length: f32,
    pub uv_repeat: Vec2,
}

impl Default for CircleGeometryDesc {
    fn default() -> Self {
        Self {
            radius: 1.0_f32,
            segments: 32,
            theta_start: Default::default(),
            theta_length: 6.283185307179586_f32,
            uv_repeat: Vec2::new(1.0_f32, 1.0_f32),
        }
    }
}

impl FromJson for CircleGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            radius: field(o, "radius", d.radius)?,
            segments: field(o, "segments", d.segments)?,
            theta_start: field(o, "thetaStart", d.theta_start)?,
            theta_length: field(o, "thetaLength", d.theta_length)?,
            uv_repeat: field(o, "uvRepeat", d.uv_repeat)?,
        })
    }
}

impl ToJson for CircleGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("radius".into(), self.radius.to_json());
        o.insert("segments".into(), self.segments.to_json());
        o.insert("thetaStart".into(), self.theta_start.to_json());
        o.insert("thetaLength".into(), self.theta_length.to_json());
        o.insert("uvRepeat".into(), self.uv_repeat.to_json());
        Value::Object(o)
    }
}

/// `TubeGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TubeGeometryDesc {
    pub points: Vec<Vec3>,
    pub path: u64,
    pub tubular_segments: u32,
    pub radius: f32,
    pub radial_segments: u32,
    pub closed: bool,
    pub uv_repeat: Vec2,
}

impl Default for TubeGeometryDesc {
    fn default() -> Self {
        Self {
            points: Default::default(),
            path: Default::default(),
            tubular_segments: 64,
            radius: 1.0_f32,
            radial_segments: 8,
            closed: Default::default(),
            uv_repeat: Vec2::new(1.0_f32, 1.0_f32),
        }
    }
}

impl FromJson for TubeGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            points: field(o, "points", d.points)?,
            path: field(o, "path", d.path)?,
            tubular_segments: field(o, "tubularSegments", d.tubular_segments)?,
            radius: field(o, "radius", d.radius)?,
            radial_segments: field(o, "radialSegments", d.radial_segments)?,
            closed: field(o, "closed", d.closed)?,
            uv_repeat: field(o, "uvRepeat", d.uv_repeat)?,
        })
    }
}

impl ToJson for TubeGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("points".into(), self.points.to_json());
        o.insert("path".into(), self.path.to_json());
        o.insert("tubularSegments".into(), self.tubular_segments.to_json());
        o.insert("radius".into(), self.radius.to_json());
        o.insert("radialSegments".into(), self.radial_segments.to_json());
        o.insert("closed".into(), self.closed.to_json());
        o.insert("uvRepeat".into(), self.uv_repeat.to_json());
        Value::Object(o)
    }
}

/// `ShapePoints`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapePoints {
    pub shape: Vec<Vec2>,
    pub holes: Vec<Vec<Vec2>>,
}

impl Default for ShapePoints {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            holes: Default::default(),
        }
    }
}

impl FromJson for ShapePoints {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            shape: field(o, "shape", d.shape)?,
            holes: field(o, "holes", d.holes)?,
        })
    }
}

impl ToJson for ShapePoints {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("shape".into(), self.shape.to_json());
        o.insert("holes".into(), self.holes.to_json());
        Value::Object(o)
    }
}

/// `ShapeGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeGeometryDesc {
    pub shapes: Option<OneOf<u32, OneOf<Vec<u32>, OneOf<Vec<Vec2>, Vec<ShapePoints>>>>>,
    pub curve_segments: u32,
    pub uv_repeat: Vec2,
    pub physics: bool,
    pub dynamic: bool,
}

impl Default for ShapeGeometryDesc {
    fn default() -> Self {
        Self {
            shapes: Default::default(),
            curve_segments: 12,
            uv_repeat: Vec2::new(1.0_f32, 1.0_f32),
            physics: true,
            dynamic: Default::default(),
        }
    }
}

impl FromJson for ShapeGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            shapes: field(o, "shapes", d.shapes)?,
            curve_segments: field(o, "curveSegments", d.curve_segments)?,
            uv_repeat: field(o, "uvRepeat", d.uv_repeat)?,
            physics: field(o, "physics", d.physics)?,
            dynamic: field(o, "dynamic", d.dynamic)?,
        })
    }
}

impl ToJson for ShapeGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("shapes".into(), self.shapes.to_json());
        o.insert("curveSegments".into(), self.curve_segments.to_json());
        o.insert("uvRepeat".into(), self.uv_repeat.to_json());
        o.insert("physics".into(), self.physics.to_json());
        o.insert("dynamic".into(), self.dynamic.to_json());
        Value::Object(o)
    }
}

/// `ExtrudeGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ExtrudeGeometryDesc {
    pub shapes: Option<OneOf<u32, OneOf<Vec<u32>, OneOf<Vec<Vec2>, Vec<ShapePoints>>>>>,
    pub depth: f32,
    pub steps: u32,
    pub bevel_enabled: bool,
    pub bevel_thickness: f32,
    pub bevel_size: Option<f32>,
    pub bevel_offset: f32,
    pub bevel_segments: u32,
    pub curve_segments: u32,
    pub extrude_path: u64,
    pub uv_repeat: Vec2,
    pub physics: bool,
    pub dynamic: bool,
}

impl Default for ExtrudeGeometryDesc {
    fn default() -> Self {
        Self {
            shapes: Default::default(),
            depth: 1.0_f32,
            steps: 1,
            bevel_enabled: true,
            bevel_thickness: 0.2_f32,
            bevel_size: Default::default(),
            bevel_offset: Default::default(),
            bevel_segments: 3,
            curve_segments: 12,
            extrude_path: Default::default(),
            uv_repeat: Vec2::new(1.0_f32, 1.0_f32),
            physics: true,
            dynamic: Default::default(),
        }
    }
}

impl FromJson for ExtrudeGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            shapes: field(o, "shapes", d.shapes)?,
            depth: field(o, "depth", d.depth)?,
            steps: field(o, "steps", d.steps)?,
            bevel_enabled: field(o, "bevelEnabled", d.bevel_enabled)?,
            bevel_thickness: field(o, "bevelThickness", d.bevel_thickness)?,
            bevel_size: field(o, "bevelSize", d.bevel_size)?,
            bevel_offset: field(o, "bevelOffset", d.bevel_offset)?,
            bevel_segments: field(o, "bevelSegments", d.bevel_segments)?,
            curve_segments: field(o, "curveSegments", d.curve_segments)?,
            extrude_path: field(o, "extrudePath", d.extrude_path)?,
            uv_repeat: field(o, "uvRepeat", d.uv_repeat)?,
            physics: field(o, "physics", d.physics)?,
            dynamic: field(o, "dynamic", d.dynamic)?,
        })
    }
}

impl ToJson for ExtrudeGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("shapes".into(), self.shapes.to_json());
        o.insert("depth".into(), self.depth.to_json());
        o.insert("steps".into(), self.steps.to_json());
        o.insert("bevelEnabled".into(), self.bevel_enabled.to_json());
        o.insert("bevelThickness".into(), self.bevel_thickness.to_json());
        o.insert("bevelSize".into(), self.bevel_size.to_json());
        o.insert("bevelOffset".into(), self.bevel_offset.to_json());
        o.insert("bevelSegments".into(), self.bevel_segments.to_json());
        o.insert("curveSegments".into(), self.curve_segments.to_json());
        o.insert("extrudePath".into(), self.extrude_path.to_json());
        o.insert("uvRepeat".into(), self.uv_repeat.to_json());
        o.insert("physics".into(), self.physics.to_json());
        o.insert("dynamic".into(), self.dynamic.to_json());
        Value::Object(o)
    }
}

/// `LatheGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct LatheGeometryDesc {
    pub points: Vec<Vec2>,
    pub segments: u32,
    pub phi_start: f32,
    pub phi_length: f32,
    pub uv_repeat: Vec2,
    pub physics: bool,
    pub dynamic: bool,
}

impl Default for LatheGeometryDesc {
    fn default() -> Self {
        Self {
            points: Default::default(),
            segments: 12,
            phi_start: Default::default(),
            phi_length: 6.283185307179586_f32,
            uv_repeat: Vec2::new(1.0_f32, 1.0_f32),
            physics: true,
            dynamic: Default::default(),
        }
    }
}

impl FromJson for LatheGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            points: field(o, "points", d.points)?,
            segments: field(o, "segments", d.segments)?,
            phi_start: field(o, "phiStart", d.phi_start)?,
            phi_length: field(o, "phiLength", d.phi_length)?,
            uv_repeat: field(o, "uvRepeat", d.uv_repeat)?,
            physics: field(o, "physics", d.physics)?,
            dynamic: field(o, "dynamic", d.dynamic)?,
        })
    }
}

impl ToJson for LatheGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("points".into(), self.points.to_json());
        o.insert("segments".into(), self.segments.to_json());
        o.insert("phiStart".into(), self.phi_start.to_json());
        o.insert("phiLength".into(), self.phi_length.to_json());
        o.insert("uvRepeat".into(), self.uv_repeat.to_json());
        o.insert("physics".into(), self.physics.to_json());
        o.insert("dynamic".into(), self.dynamic.to_json());
        Value::Object(o)
    }
}

/// `PolyhedronGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PolyhedronGeometryDesc {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub radius: f32,
    pub detail: u32,
    pub uv_repeat: Vec2,
    pub physics: bool,
    pub dynamic: bool,
}

impl Default for PolyhedronGeometryDesc {
    fn default() -> Self {
        Self {
            vertices: Default::default(),
            indices: Default::default(),
            radius: 1.0_f32,
            detail: Default::default(),
            uv_repeat: Vec2::new(1.0_f32, 1.0_f32),
            physics: true,
            dynamic: Default::default(),
        }
    }
}

impl FromJson for PolyhedronGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            vertices: field(o, "vertices", d.vertices)?,
            indices: field(o, "indices", d.indices)?,
            radius: field(o, "radius", d.radius)?,
            detail: field(o, "detail", d.detail)?,
            uv_repeat: field(o, "uvRepeat", d.uv_repeat)?,
            physics: field(o, "physics", d.physics)?,
            dynamic: field(o, "dynamic", d.dynamic)?,
        })
    }
}

impl ToJson for PolyhedronGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("vertices".into(), self.vertices.to_json());
        o.insert("indices".into(), self.indices.to_json());
        o.insert("radius".into(), self.radius.to_json());
        o.insert("detail".into(), self.detail.to_json());
        o.insert("uvRepeat".into(), self.uv_repeat.to_json());
        o.insert("physics".into(), self.physics.to_json());
        o.insert("dynamic".into(), self.dynamic.to_json());
        Value::Object(o)
    }
}

/// `SolidGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SolidGeometryDesc {
    pub radius: f32,
    pub detail: u32,
    pub uv_repeat: Vec2,
    pub physics: bool,
    pub dynamic: bool,
}

impl Default for SolidGeometryDesc {
    fn default() -> Self {
        Self {
            radius: 1.0_f32,
            detail: Default::default(),
            uv_repeat: Vec2::new(1.0_f32, 1.0_f32),
            physics: true,
            dynamic: Default::default(),
        }
    }
}

impl FromJson for SolidGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            radius: field(o, "radius", d.radius)?,
            detail: field(o, "detail", d.detail)?,
            uv_repeat: field(o, "uvRepeat", d.uv_repeat)?,
            physics: field(o, "physics", d.physics)?,
            dynamic: field(o, "dynamic", d.dynamic)?,
        })
    }
}

impl ToJson for SolidGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("radius".into(), self.radius.to_json());
        o.insert("detail".into(), self.detail.to_json());
        o.insert("uvRepeat".into(), self.uv_repeat.to_json());
        o.insert("physics".into(), self.physics.to_json());
        o.insert("dynamic".into(), self.dynamic.to_json());
        Value::Object(o)
    }
}

/// `SweepKey`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SweepKey {
    pub t: f32,
    pub value: f32,
}

impl Default for SweepKey {
    fn default() -> Self {
        Self {
            t: Default::default(),
            value: Default::default(),
        }
    }
}

impl FromJson for SweepKey {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            t: field(o, "t", d.t)?,
            value: field(o, "value", d.value)?,
        })
    }
}

impl ToJson for SweepKey {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("t".into(), self.t.to_json());
        o.insert("value".into(), self.value.to_json());
        Value::Object(o)
    }
}

/// `SweepGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SweepGeometryDesc {
    pub section: Option<OneOf<u32, OneOf<Vec<Vec2>, ShapePoints>>>,
    pub path: u64,
    pub steps: u32,
    pub scale: Vec<SweepKey>,
    pub twist: Vec<SweepKey>,
    pub caps: bool,
    pub curve_segments: u32,
    pub uv_repeat: Vec2,
    pub physics: bool,
    pub dynamic: bool,
}

impl Default for SweepGeometryDesc {
    fn default() -> Self {
        Self {
            section: Default::default(),
            path: Default::default(),
            steps: 64,
            scale: Default::default(),
            twist: Default::default(),
            caps: true,
            curve_segments: 12,
            uv_repeat: Vec2::new(1.0_f32, 1.0_f32),
            physics: true,
            dynamic: Default::default(),
        }
    }
}

impl FromJson for SweepGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            section: field(o, "section", d.section)?,
            path: field(o, "path", d.path)?,
            steps: field(o, "steps", d.steps)?,
            scale: field(o, "scale", d.scale)?,
            twist: field(o, "twist", d.twist)?,
            caps: field(o, "caps", d.caps)?,
            curve_segments: field(o, "curveSegments", d.curve_segments)?,
            uv_repeat: field(o, "uvRepeat", d.uv_repeat)?,
            physics: field(o, "physics", d.physics)?,
            dynamic: field(o, "dynamic", d.dynamic)?,
        })
    }
}

impl ToJson for SweepGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("section".into(), self.section.to_json());
        o.insert("path".into(), self.path.to_json());
        o.insert("steps".into(), self.steps.to_json());
        o.insert("scale".into(), self.scale.to_json());
        o.insert("twist".into(), self.twist.to_json());
        o.insert("caps".into(), self.caps.to_json());
        o.insert("curveSegments".into(), self.curve_segments.to_json());
        o.insert("uvRepeat".into(), self.uv_repeat.to_json());
        o.insert("physics".into(), self.physics.to_json());
        o.insert("dynamic".into(), self.dynamic.to_json());
        Value::Object(o)
    }
}

/// `MirrorAxis`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MirrorAxis {
    #[default]
    None,
    X,
    Y,
    Z,
}

impl MirrorAxis {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["none", "x", "y", "z"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::None, Self::X, Self::Y, Self::Z];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("MirrorAxis", s, Self::NAMES)),
        }
    }
}

impl FromJson for MirrorAxis {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "MirrorAxis", Self::NAMES)?)
    }
}

impl ToJson for MirrorAxis {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `GeometryPart`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryPart {
    pub geometry: u64,
    pub position: Vec3,
    pub quaternion: Quat,
    pub scale: Vec3,
    pub mirror: MirrorAxis,
}

impl Default for GeometryPart {
    fn default() -> Self {
        Self {
            geometry: Default::default(),
            position: Default::default(),
            quaternion: Default::default(),
            scale: Vec3::new(1.0_f32, 1.0_f32, 1.0_f32),
            mirror: Default::default(),
        }
    }
}

impl FromJson for GeometryPart {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            geometry: field(o, "geometry", d.geometry)?,
            position: field(o, "position", d.position)?,
            quaternion: field(o, "quaternion", d.quaternion)?,
            scale: field(o, "scale", d.scale)?,
            mirror: field(o, "mirror", d.mirror)?,
        })
    }
}

impl ToJson for GeometryPart {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("geometry".into(), self.geometry.to_json());
        o.insert("position".into(), self.position.to_json());
        o.insert("quaternion".into(), self.quaternion.to_json());
        o.insert("scale".into(), self.scale.to_json());
        o.insert("mirror".into(), self.mirror.to_json());
        Value::Object(o)
    }
}

/// `CsgOp`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CsgOp {
    #[default]
    Union,
    Subtract,
    Intersect,
}

impl CsgOp {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["union", "subtract", "intersect"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Union, Self::Subtract, Self::Intersect];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("CsgOp", s, Self::NAMES)),
        }
    }
}

impl FromJson for CsgOp {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "CsgOp", Self::NAMES)?)
    }
}

impl ToJson for CsgOp {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `CsgGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct CsgGeometryDesc {
    pub op: CsgOp,
    pub a: GeometryPart,
    pub b: GeometryPart,
    pub uv_repeat: Vec2,
    pub physics: bool,
    pub dynamic: bool,
}

impl Default for CsgGeometryDesc {
    fn default() -> Self {
        Self {
            op: Default::default(),
            a: Default::default(),
            b: Default::default(),
            uv_repeat: Vec2::new(1.0_f32, 1.0_f32),
            physics: true,
            dynamic: Default::default(),
        }
    }
}

impl FromJson for CsgGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            op: field(o, "op", d.op)?,
            a: field(o, "a", d.a)?,
            b: field(o, "b", d.b)?,
            uv_repeat: field(o, "uvRepeat", d.uv_repeat)?,
            physics: field(o, "physics", d.physics)?,
            dynamic: field(o, "dynamic", d.dynamic)?,
        })
    }
}

impl ToJson for CsgGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("op".into(), self.op.to_json());
        o.insert("a".into(), self.a.to_json());
        o.insert("b".into(), self.b.to_json());
        o.insert("uvRepeat".into(), self.uv_repeat.to_json());
        o.insert("physics".into(), self.physics.to_json());
        o.insert("dynamic".into(), self.dynamic.to_json());
        Value::Object(o)
    }
}

/// `MergeGeometriesDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct MergeGeometriesDesc {
    pub parts: Vec<GeometryPart>,
    pub uv_repeat: Vec2,
    pub physics: bool,
    pub dynamic: bool,
}

impl Default for MergeGeometriesDesc {
    fn default() -> Self {
        Self {
            parts: Default::default(),
            uv_repeat: Vec2::new(1.0_f32, 1.0_f32),
            physics: true,
            dynamic: Default::default(),
        }
    }
}

impl FromJson for MergeGeometriesDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            parts: field(o, "parts", d.parts)?,
            uv_repeat: field(o, "uvRepeat", d.uv_repeat)?,
            physics: field(o, "physics", d.physics)?,
            dynamic: field(o, "dynamic", d.dynamic)?,
        })
    }
}

impl ToJson for MergeGeometriesDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("parts".into(), self.parts.to_json());
        o.insert("uvRepeat".into(), self.uv_repeat.to_json());
        o.insert("physics".into(), self.physics.to_json());
        o.insert("dynamic".into(), self.dynamic.to_json());
        Value::Object(o)
    }
}

/// `LineMode`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LineMode {
    #[default]
    Strip,
    Segments,
    Loop,
}

impl LineMode {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["strip", "segments", "loop"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Strip, Self::Segments, Self::Loop];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("LineMode", s, Self::NAMES)),
        }
    }
}

impl FromJson for LineMode {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "LineMode", Self::NAMES)?)
    }
}

impl ToJson for LineMode {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `LinesDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct LinesDesc {
    pub points: Vec<Vec3>,
    pub colors: Vec<Color>,
    pub color: Color,
    pub opacity: f32,
    pub mode: LineMode,
    pub width: f32,
    pub world_units: bool,
    pub depth_test: bool,
    pub screen: bool,
    pub dashed: bool,
    pub dash_size: f32,
    pub gap_size: f32,
}

impl Default for LinesDesc {
    fn default() -> Self {
        Self {
            points: Default::default(),
            colors: Default::default(),
            color: Color { r: 1.0_f32, g: 1.0_f32, b: 1.0_f32 },
            opacity: 1.0_f32,
            mode: Default::default(),
            width: 1.0_f32,
            world_units: Default::default(),
            depth_test: true,
            screen: Default::default(),
            dashed: Default::default(),
            dash_size: 3.0_f32,
            gap_size: 1.0_f32,
        }
    }
}

impl FromJson for LinesDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            points: field(o, "points", d.points)?,
            colors: field(o, "colors", d.colors)?,
            color: field(o, "color", d.color)?,
            opacity: field(o, "opacity", d.opacity)?,
            mode: field(o, "mode", d.mode)?,
            width: field(o, "width", d.width)?,
            world_units: field(o, "worldUnits", d.world_units)?,
            depth_test: field(o, "depthTest", d.depth_test)?,
            screen: field(o, "screen", d.screen)?,
            dashed: field(o, "dashed", d.dashed)?,
            dash_size: field(o, "dashSize", d.dash_size)?,
            gap_size: field(o, "gapSize", d.gap_size)?,
        })
    }
}

impl ToJson for LinesDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("points".into(), self.points.to_json());
        o.insert("colors".into(), self.colors.to_json());
        o.insert("color".into(), self.color.to_json());
        o.insert("opacity".into(), self.opacity.to_json());
        o.insert("mode".into(), self.mode.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("worldUnits".into(), self.world_units.to_json());
        o.insert("depthTest".into(), self.depth_test.to_json());
        o.insert("screen".into(), self.screen.to_json());
        o.insert("dashed".into(), self.dashed.to_json());
        o.insert("dashSize".into(), self.dash_size.to_json());
        o.insert("gapSize".into(), self.gap_size.to_json());
        Value::Object(o)
    }
}

/// `PointsDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PointsDesc {
    pub points: Vec<Vec3>,
    pub colors: Vec<Color>,
    pub color: Color,
    pub opacity: f32,
    pub size: f32,
    pub size_attenuation: bool,
    pub depth_test: bool,
    pub screen: bool,
}

impl Default for PointsDesc {
    fn default() -> Self {
        Self {
            points: Default::default(),
            colors: Default::default(),
            color: Color { r: 1.0_f32, g: 1.0_f32, b: 1.0_f32 },
            opacity: 1.0_f32,
            size: 1.0_f32,
            size_attenuation: true,
            depth_test: true,
            screen: Default::default(),
        }
    }
}

impl FromJson for PointsDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            points: field(o, "points", d.points)?,
            colors: field(o, "colors", d.colors)?,
            color: field(o, "color", d.color)?,
            opacity: field(o, "opacity", d.opacity)?,
            size: field(o, "size", d.size)?,
            size_attenuation: field(o, "sizeAttenuation", d.size_attenuation)?,
            depth_test: field(o, "depthTest", d.depth_test)?,
            screen: field(o, "screen", d.screen)?,
        })
    }
}

impl ToJson for PointsDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("points".into(), self.points.to_json());
        o.insert("colors".into(), self.colors.to_json());
        o.insert("color".into(), self.color.to_json());
        o.insert("opacity".into(), self.opacity.to_json());
        o.insert("size".into(), self.size.to_json());
        o.insert("sizeAttenuation".into(), self.size_attenuation.to_json());
        o.insert("depthTest".into(), self.depth_test.to_json());
        o.insert("screen".into(), self.screen.to_json());
        Value::Object(o)
    }
}

/// `RaycastGeometryDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RaycastGeometryDesc {
    pub max_distance: f32,
    pub first: bool,
    pub layers: u32,
    pub invisible: bool,
}

impl Default for RaycastGeometryDesc {
    fn default() -> Self {
        Self {
            max_distance: Default::default(),
            first: Default::default(),
            layers: 1,
            invisible: Default::default(),
        }
    }
}

impl FromJson for RaycastGeometryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            max_distance: field(o, "maxDistance", d.max_distance)?,
            first: field(o, "first", d.first)?,
            layers: field(o, "layers", d.layers)?,
            invisible: field(o, "invisible", d.invisible)?,
        })
    }
}

impl ToJson for RaycastGeometryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("maxDistance".into(), self.max_distance.to_json());
        o.insert("first".into(), self.first.to_json());
        o.insert("layers".into(), self.layers.to_json());
        o.insert("invisible".into(), self.invisible.to_json());
        Value::Object(o)
    }
}

/// `GeometryHit`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryHit {
    pub entity: u64,
    pub geometry: u64,
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub face_index: u32,
}

impl Default for GeometryHit {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            geometry: Default::default(),
            distance: Default::default(),
            point: Default::default(),
            normal: Default::default(),
            uv: Default::default(),
            face_index: Default::default(),
        }
    }
}

impl FromJson for GeometryHit {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            geometry: field(o, "geometry", d.geometry)?,
            distance: field(o, "distance", d.distance)?,
            point: field(o, "point", d.point)?,
            normal: field(o, "normal", d.normal)?,
            uv: field(o, "uv", d.uv)?,
            face_index: field(o, "faceIndex", d.face_index)?,
        })
    }
}

impl ToJson for GeometryHit {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("geometry".into(), self.geometry.to_json());
        o.insert("distance".into(), self.distance.to_json());
        o.insert("point".into(), self.point.to_json());
        o.insert("normal".into(), self.normal.to_json());
        o.insert("uv".into(), self.uv.to_json());
        o.insert("faceIndex".into(), self.face_index.to_json());
        Value::Object(o)
    }
}

/// `RenderTargetFormat`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RenderTargetFormat {
    #[default]
    Ldr,
    Hdr,
}

impl RenderTargetFormat {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["ldr", "hdr"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Ldr, Self::Hdr];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("RenderTargetFormat", s, Self::NAMES)),
        }
    }
}

impl FromJson for RenderTargetFormat {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "RenderTargetFormat", Self::NAMES)?)
    }
}

impl ToJson for RenderTargetFormat {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `RenderTargetDesc`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderTargetDesc {
    pub width: u32,
    pub height: u32,
    pub format: RenderTargetFormat,
    pub depth: bool,
}

impl Default for RenderTargetDesc {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            format: Default::default(),
            depth: Default::default(),
        }
    }
}

impl FromJson for RenderTargetDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            format: field(o, "format", d.format)?,
            depth: field(o, "depth", d.depth)?,
        })
    }
}

impl ToJson for RenderTargetDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("format".into(), self.format.to_json());
        o.insert("depth".into(), self.depth.to_json());
        Value::Object(o)
    }
}

/// `RenderTargetInfo`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderTargetInfo {
    pub target: u64,
    pub texture: u64,
    pub depth_texture: u64,
    pub width: u32,
    pub height: u32,
}

impl Default for RenderTargetInfo {
    fn default() -> Self {
        Self {
            target: Default::default(),
            texture: Default::default(),
            depth_texture: Default::default(),
            width: Default::default(),
            height: Default::default(),
        }
    }
}

impl FromJson for RenderTargetInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            target: field(o, "target", d.target)?,
            texture: field(o, "texture", d.texture)?,
            depth_texture: field(o, "depthTexture", d.depth_texture)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
        })
    }
}

impl ToJson for RenderTargetInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("target".into(), self.target.to_json());
        o.insert("texture".into(), self.texture.to_json());
        o.insert("depthTexture".into(), self.depth_texture.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        Value::Object(o)
    }
}

/// `PostError`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PostError {
    pub post: u64,
    pub message: String,
}

impl Default for PostError {
    fn default() -> Self {
        Self {
            post: Default::default(),
            message: Default::default(),
        }
    }
}

impl FromJson for PostError {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            post: field(o, "post", d.post)?,
            message: field(o, "message", d.message)?,
        })
    }
}

impl ToJson for PostError {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("post".into(), self.post.to_json());
        o.insert("message".into(), self.message.to_json());
        Value::Object(o)
    }
}

/// `VolumetricFog`: packages/knowledge/wiki/render.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VolumetricFog {
    #[default]
    Froxel,
    Raymarch,
}

impl VolumetricFog {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["froxel", "raymarch"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Froxel, Self::Raymarch];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("VolumetricFog", s, Self::NAMES)),
        }
    }
}

impl FromJson for VolumetricFog {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "VolumetricFog", Self::NAMES)?)
    }
}

impl ToJson for VolumetricFog {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `BakedScene`: packages/knowledge/wiki/render.md.
#[derive(Clone, Debug, PartialEq)]
pub struct BakedScene {
    pub scene: String,
    pub baked: bool,
    pub reason: String,
    pub models: u32,
    pub pages: u32,
    pub texels: u32,
    pub volumes: u32,
    pub probes: u32,
}

impl Default for BakedScene {
    fn default() -> Self {
        Self {
            scene: Default::default(),
            baked: Default::default(),
            reason: Default::default(),
            models: Default::default(),
            pages: Default::default(),
            texels: Default::default(),
            volumes: Default::default(),
            probes: Default::default(),
        }
    }
}

impl FromJson for BakedScene {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            scene: field(o, "scene", d.scene)?,
            baked: field(o, "baked", d.baked)?,
            reason: field(o, "reason", d.reason)?,
            models: field(o, "models", d.models)?,
            pages: field(o, "pages", d.pages)?,
            texels: field(o, "texels", d.texels)?,
            volumes: field(o, "volumes", d.volumes)?,
            probes: field(o, "probes", d.probes)?,
        })
    }
}

impl ToJson for BakedScene {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("scene".into(), self.scene.to_json());
        o.insert("baked".into(), self.baked.to_json());
        o.insert("reason".into(), self.reason.to_json());
        o.insert("models".into(), self.models.to_json());
        o.insert("pages".into(), self.pages.to_json());
        o.insert("texels".into(), self.texels.to_json());
        o.insert("volumes".into(), self.volumes.to_json());
        o.insert("probes".into(), self.probes.to_json());
        Value::Object(o)
    }
}

/// `LoadStatus`: packages/knowledge/wiki/load.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LoadStatus {
    #[default]
    Pending,
    Ready,
    Failed,
}

impl LoadStatus {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["pending", "ready", "failed"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Pending, Self::Ready, Self::Failed];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("LoadStatus", s, Self::NAMES)),
        }
    }
}

impl FromJson for LoadStatus {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "LoadStatus", Self::NAMES)?)
    }
}

impl ToJson for LoadStatus {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `GltfDesc`: packages/knowledge/wiki/load.md.
#[derive(Clone, Debug, PartialEq)]
pub struct GltfDesc {
    pub scene: u32,
    pub cast_shadow: bool,
    pub receive_shadow: bool,
}

impl Default for GltfDesc {
    fn default() -> Self {
        Self {
            scene: Default::default(),
            cast_shadow: true,
            receive_shadow: true,
        }
    }
}

impl FromJson for GltfDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            scene: field(o, "scene", d.scene)?,
            cast_shadow: field(o, "castShadow", d.cast_shadow)?,
            receive_shadow: field(o, "receiveShadow", d.receive_shadow)?,
        })
    }
}

impl ToJson for GltfDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("scene".into(), self.scene.to_json());
        o.insert("castShadow".into(), self.cast_shadow.to_json());
        o.insert("receiveShadow".into(), self.receive_shadow.to_json());
        Value::Object(o)
    }
}

/// `GltfPart`: packages/knowledge/wiki/load.md.
#[derive(Clone, Debug, PartialEq)]
pub struct GltfPart {
    pub geometry: u64,
    pub material: u64,
    pub transform: Mat4,
}

impl Default for GltfPart {
    fn default() -> Self {
        Self {
            geometry: Default::default(),
            material: Default::default(),
            transform: Default::default(),
        }
    }
}

impl FromJson for GltfPart {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            geometry: field(o, "geometry", d.geometry)?,
            material: field(o, "material", d.material)?,
            transform: field(o, "transform", d.transform)?,
        })
    }
}

impl ToJson for GltfPart {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("geometry".into(), self.geometry.to_json());
        o.insert("material".into(), self.material.to_json());
        o.insert("transform".into(), self.transform.to_json());
        Value::Object(o)
    }
}

/// `GltfInfo`: packages/knowledge/wiki/load.md.
#[derive(Clone, Debug, PartialEq)]
pub struct GltfInfo {
    pub scenes: u32,
    pub geometries: Vec<u64>,
    pub parts: Vec<GltfPart>,
    pub materials: Vec<u64>,
    pub variants: Vec<String>,
    pub lights: u32,
    pub animations: Vec<String>,
}

impl Default for GltfInfo {
    fn default() -> Self {
        Self {
            scenes: Default::default(),
            geometries: Default::default(),
            parts: Default::default(),
            materials: Default::default(),
            variants: Default::default(),
            lights: Default::default(),
            animations: Default::default(),
        }
    }
}

impl FromJson for GltfInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            scenes: field(o, "scenes", d.scenes)?,
            geometries: field(o, "geometries", d.geometries)?,
            parts: field(o, "parts", d.parts)?,
            materials: field(o, "materials", d.materials)?,
            variants: field(o, "variants", d.variants)?,
            lights: field(o, "lights", d.lights)?,
            animations: field(o, "animations", d.animations)?,
        })
    }
}

impl ToJson for GltfInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("scenes".into(), self.scenes.to_json());
        o.insert("geometries".into(), self.geometries.to_json());
        o.insert("parts".into(), self.parts.to_json());
        o.insert("materials".into(), self.materials.to_json());
        o.insert("variants".into(), self.variants.to_json());
        o.insert("lights".into(), self.lights.to_json());
        o.insert("animations".into(), self.animations.to_json());
        Value::Object(o)
    }
}

/// `BundleInfo`: packages/knowledge/wiki/load.md.
#[derive(Clone, Debug, PartialEq)]
pub struct BundleInfo {
    pub path: String,
    pub assets: u32,
    pub outputs: u32,
}

impl Default for BundleInfo {
    fn default() -> Self {
        Self {
            path: Default::default(),
            assets: Default::default(),
            outputs: Default::default(),
        }
    }
}

impl FromJson for BundleInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            path: field(o, "path", d.path)?,
            assets: field(o, "assets", d.assets)?,
            outputs: field(o, "outputs", d.outputs)?,
        })
    }
}

impl ToJson for BundleInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("path".into(), self.path.to_json());
        o.insert("assets".into(), self.assets.to_json());
        o.insert("outputs".into(), self.outputs.to_json());
        Value::Object(o)
    }
}

/// `FontInfo`: packages/knowledge/wiki/load.md.
#[derive(Clone, Debug, PartialEq)]
pub struct FontInfo {
    pub glyphs: u32,
    pub pages: u32,
    pub em_px: u32,
    pub px_range: f32,
    pub ascender: f32,
    pub descender: f32,
    pub line_height: f32,
    pub icons: u32,
}

impl Default for FontInfo {
    fn default() -> Self {
        Self {
            glyphs: Default::default(),
            pages: Default::default(),
            em_px: Default::default(),
            px_range: Default::default(),
            ascender: Default::default(),
            descender: Default::default(),
            line_height: Default::default(),
            icons: Default::default(),
        }
    }
}

impl FromJson for FontInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            glyphs: field(o, "glyphs", d.glyphs)?,
            pages: field(o, "pages", d.pages)?,
            em_px: field(o, "emPx", d.em_px)?,
            px_range: field(o, "pxRange", d.px_range)?,
            ascender: field(o, "ascender", d.ascender)?,
            descender: field(o, "descender", d.descender)?,
            line_height: field(o, "lineHeight", d.line_height)?,
            icons: field(o, "icons", d.icons)?,
        })
    }
}

impl ToJson for FontInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("glyphs".into(), self.glyphs.to_json());
        o.insert("pages".into(), self.pages.to_json());
        o.insert("emPx".into(), self.em_px.to_json());
        o.insert("pxRange".into(), self.px_range.to_json());
        o.insert("ascender".into(), self.ascender.to_json());
        o.insert("descender".into(), self.descender.to_json());
        o.insert("lineHeight".into(), self.line_height.to_json());
        o.insert("icons".into(), self.icons.to_json());
        Value::Object(o)
    }
}

/// `SoundInfo`: packages/knowledge/wiki/load.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SoundInfo {
    pub rate: u32,
    pub channels: u32,
    pub frames: u32,
    pub seconds: f32,
    pub chunks: u32,
}

impl Default for SoundInfo {
    fn default() -> Self {
        Self {
            rate: Default::default(),
            channels: Default::default(),
            frames: Default::default(),
            seconds: Default::default(),
            chunks: Default::default(),
        }
    }
}

impl FromJson for SoundInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            rate: field(o, "rate", d.rate)?,
            channels: field(o, "channels", d.channels)?,
            frames: field(o, "frames", d.frames)?,
            seconds: field(o, "seconds", d.seconds)?,
            chunks: field(o, "chunks", d.chunks)?,
        })
    }
}

impl ToJson for SoundInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("rate".into(), self.rate.to_json());
        o.insert("channels".into(), self.channels.to_json());
        o.insert("frames".into(), self.frames.to_json());
        o.insert("seconds".into(), self.seconds.to_json());
        o.insert("chunks".into(), self.chunks.to_json());
        Value::Object(o)
    }
}

/// `TextureArrayInfo`: packages/knowledge/wiki/load.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TextureArrayInfo {
    pub array: u64,
    pub width: u32,
    pub height: u32,
    pub layers: u32,
    pub levels: u32,
    pub format: String,
    pub bytes: f64,
}

impl Default for TextureArrayInfo {
    fn default() -> Self {
        Self {
            array: Default::default(),
            width: Default::default(),
            height: Default::default(),
            layers: Default::default(),
            levels: Default::default(),
            format: Default::default(),
            bytes: Default::default(),
        }
    }
}

impl FromJson for TextureArrayInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            array: field(o, "array", d.array)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            layers: field(o, "layers", d.layers)?,
            levels: field(o, "levels", d.levels)?,
            format: field(o, "format", d.format)?,
            bytes: field(o, "bytes", d.bytes)?,
        })
    }
}

impl ToJson for TextureArrayInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("array".into(), self.array.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("layers".into(), self.layers.to_json());
        o.insert("levels".into(), self.levels.to_json());
        o.insert("format".into(), self.format.to_json());
        o.insert("bytes".into(), self.bytes.to_json());
        Value::Object(o)
    }
}

/// `LoadBudget`: packages/knowledge/wiki/load.md.
#[derive(Clone, Debug, PartialEq)]
pub struct LoadBudget {
    pub bytes: f64,
    pub triangles: u32,
    pub texture_size: u32,
    pub frame_ms: f32,
}

impl Default for LoadBudget {
    fn default() -> Self {
        Self {
            bytes: 33554432.0_f64,
            triangles: 500000,
            texture_size: 4096,
            frame_ms: 2.0_f32,
        }
    }
}

impl FromJson for LoadBudget {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            bytes: field(o, "bytes", d.bytes)?,
            triangles: field(o, "triangles", d.triangles)?,
            texture_size: field(o, "textureSize", d.texture_size)?,
            frame_ms: field(o, "frameMs", d.frame_ms)?,
        })
    }
}

impl ToJson for LoadBudget {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("bytes".into(), self.bytes.to_json());
        o.insert("triangles".into(), self.triangles.to_json());
        o.insert("textureSize".into(), self.texture_size.to_json());
        o.insert("frameMs".into(), self.frame_ms.to_json());
        Value::Object(o)
    }
}

/// `ImageKind`: packages/knowledge/wiki/load.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ImageKind {
    #[default]
    Color,
    Linear,
    Pixel,
}

impl ImageKind {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["color", "linear", "pixel"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Color, Self::Linear, Self::Pixel];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("ImageKind", s, Self::NAMES)),
        }
    }
}

impl FromJson for ImageKind {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "ImageKind", Self::NAMES)?)
    }
}

impl ToJson for ImageKind {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `GltfDataDesc`: packages/knowledge/wiki/load.md.
#[derive(Clone, Debug, PartialEq)]
pub struct GltfDataDesc {
    pub budget: LoadBudget,
}

impl Default for GltfDataDesc {
    fn default() -> Self {
        Self {
            budget: Default::default(),
        }
    }
}

impl FromJson for GltfDataDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            budget: field(o, "budget", d.budget)?,
        })
    }
}

impl ToJson for GltfDataDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("budget".into(), self.budget.to_json());
        Value::Object(o)
    }
}

/// `ImageDataDesc`: packages/knowledge/wiki/load.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ImageDataDesc {
    pub kind: ImageKind,
    pub budget: LoadBudget,
}

impl Default for ImageDataDesc {
    fn default() -> Self {
        Self {
            kind: Default::default(),
            budget: Default::default(),
        }
    }
}

impl FromJson for ImageDataDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            kind: field(o, "kind", d.kind)?,
            budget: field(o, "budget", d.budget)?,
        })
    }
}

impl ToJson for ImageDataDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("kind".into(), self.kind.to_json());
        o.insert("budget".into(), self.budget.to_json());
        Value::Object(o)
    }
}

/// `EnvironmentDataDesc`: packages/knowledge/wiki/load.md.
#[derive(Clone, Debug, PartialEq)]
pub struct EnvironmentDataDesc {
    pub size: u32,
    pub intensity: f32,
    pub budget: LoadBudget,
}

impl Default for EnvironmentDataDesc {
    fn default() -> Self {
        Self {
            size: 128,
            intensity: 1.0_f32,
            budget: Default::default(),
        }
    }
}

impl FromJson for EnvironmentDataDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            size: field(o, "size", d.size)?,
            intensity: field(o, "intensity", d.intensity)?,
            budget: field(o, "budget", d.budget)?,
        })
    }
}

impl ToJson for EnvironmentDataDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("size".into(), self.size.to_json());
        o.insert("intensity".into(), self.intensity.to_json());
        o.insert("budget".into(), self.budget.to_json());
        Value::Object(o)
    }
}

/// `EnvironmentInfo`: packages/knowledge/wiki/load.md.
#[derive(Clone, Debug, PartialEq)]
pub struct EnvironmentInfo {
    pub map: u64,
    pub width: u32,
    pub height: u32,
    pub size: u32,
}

impl Default for EnvironmentInfo {
    fn default() -> Self {
        Self {
            map: Default::default(),
            width: Default::default(),
            height: Default::default(),
            size: Default::default(),
        }
    }
}

impl FromJson for EnvironmentInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            map: field(o, "map", d.map)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            size: field(o, "size", d.size)?,
        })
    }
}

impl ToJson for EnvironmentInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("map".into(), self.map.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("size".into(), self.size.to_json());
        Value::Object(o)
    }
}

/// `ImageInfo`: packages/knowledge/wiki/load.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ImageInfo {
    pub texture: u64,
    pub width: u32,
    pub height: u32,
}

impl Default for ImageInfo {
    fn default() -> Self {
        Self {
            texture: Default::default(),
            width: Default::default(),
            height: Default::default(),
        }
    }
}

impl FromJson for ImageInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            texture: field(o, "texture", d.texture)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
        })
    }
}

impl ToJson for ImageInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("texture".into(), self.texture.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        Value::Object(o)
    }
}

/// `LoadCompletion`: packages/knowledge/wiki/load.md.
#[derive(Clone, Debug, PartialEq)]
pub struct LoadCompletion {
    pub request: u32,
    pub tick: f64,
    pub ok: bool,
}

impl Default for LoadCompletion {
    fn default() -> Self {
        Self {
            request: Default::default(),
            tick: Default::default(),
            ok: Default::default(),
        }
    }
}

impl FromJson for LoadCompletion {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            request: field(o, "request", d.request)?,
            tick: field(o, "tick", d.tick)?,
            ok: field(o, "ok", d.ok)?,
        })
    }
}

impl ToJson for LoadCompletion {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("request".into(), self.request.to_json());
        o.insert("tick".into(), self.tick.to_json());
        o.insert("ok".into(), self.ok.to_json());
        Value::Object(o)
    }
}

/// `QueryFilter`: packages/knowledge/wiki/physics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct QueryFilter {
    pub mask: u32,
    pub include_sensors: bool,
    pub exclude: Option<u64>,
}

impl Default for QueryFilter {
    fn default() -> Self {
        Self {
            mask: Default::default(),
            include_sensors: Default::default(),
            exclude: Default::default(),
        }
    }
}

impl FromJson for QueryFilter {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            mask: field(o, "mask", d.mask)?,
            include_sensors: field(o, "includeSensors", d.include_sensors)?,
            exclude: field(o, "exclude", d.exclude)?,
        })
    }
}

impl ToJson for QueryFilter {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("mask".into(), self.mask.to_json());
        o.insert("includeSensors".into(), self.include_sensors.to_json());
        o.insert("exclude".into(), self.exclude.to_json());
        Value::Object(o)
    }
}

/// `RaycastDesc`: packages/knowledge/wiki/physics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RaycastDesc {
    pub origin: Vec3,
    pub direction: Vec3,
    pub max_distance: f32,
    pub filter: QueryFilter,
}

impl Default for RaycastDesc {
    fn default() -> Self {
        Self {
            origin: Default::default(),
            direction: Vec3::new(0.0_f32, -1.0_f32, 0.0_f32),
            max_distance: Default::default(),
            filter: Default::default(),
        }
    }
}

impl FromJson for RaycastDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            origin: field(o, "origin", d.origin)?,
            direction: field(o, "direction", d.direction)?,
            max_distance: field(o, "maxDistance", d.max_distance)?,
            filter: field(o, "filter", d.filter)?,
        })
    }
}

impl ToJson for RaycastDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("origin".into(), self.origin.to_json());
        o.insert("direction".into(), self.direction.to_json());
        o.insert("maxDistance".into(), self.max_distance.to_json());
        o.insert("filter".into(), self.filter.to_json());
        Value::Object(o)
    }
}

/// `RaycastHit`: packages/knowledge/wiki/physics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RaycastHit {
    pub entity: u64,
    pub body: u64,
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
}

impl Default for RaycastHit {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            body: Default::default(),
            distance: Default::default(),
            point: Default::default(),
            normal: Default::default(),
        }
    }
}

impl FromJson for RaycastHit {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            body: field(o, "body", d.body)?,
            distance: field(o, "distance", d.distance)?,
            point: field(o, "point", d.point)?,
            normal: field(o, "normal", d.normal)?,
        })
    }
}

impl ToJson for RaycastHit {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("body".into(), self.body.to_json());
        o.insert("distance".into(), self.distance.to_json());
        o.insert("point".into(), self.point.to_json());
        o.insert("normal".into(), self.normal.to_json());
        Value::Object(o)
    }
}

/// `RaycastAllDesc`: packages/knowledge/wiki/physics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RaycastAllDesc {
    pub origin: Vec3,
    pub direction: Vec3,
    pub max_distance: f32,
    pub filter: QueryFilter,
    pub max_hits: u32,
}

impl Default for RaycastAllDesc {
    fn default() -> Self {
        Self {
            origin: Default::default(),
            direction: Vec3::new(0.0_f32, -1.0_f32, 0.0_f32),
            max_distance: Default::default(),
            filter: Default::default(),
            max_hits: Default::default(),
        }
    }
}

impl FromJson for RaycastAllDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            origin: field(o, "origin", d.origin)?,
            direction: field(o, "direction", d.direction)?,
            max_distance: field(o, "maxDistance", d.max_distance)?,
            filter: field(o, "filter", d.filter)?,
            max_hits: field(o, "maxHits", d.max_hits)?,
        })
    }
}

impl ToJson for RaycastAllDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("origin".into(), self.origin.to_json());
        o.insert("direction".into(), self.direction.to_json());
        o.insert("maxDistance".into(), self.max_distance.to_json());
        o.insert("filter".into(), self.filter.to_json());
        o.insert("maxHits".into(), self.max_hits.to_json());
        Value::Object(o)
    }
}

/// `ProjectPointDesc`: packages/knowledge/wiki/physics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectPointDesc {
    pub point: Vec3,
    pub max_distance: f32,
    pub solid: bool,
    pub filter: QueryFilter,
}

impl Default for ProjectPointDesc {
    fn default() -> Self {
        Self {
            point: Default::default(),
            max_distance: Default::default(),
            solid: true,
            filter: Default::default(),
        }
    }
}

impl FromJson for ProjectPointDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            point: field(o, "point", d.point)?,
            max_distance: field(o, "maxDistance", d.max_distance)?,
            solid: field(o, "solid", d.solid)?,
            filter: field(o, "filter", d.filter)?,
        })
    }
}

impl ToJson for ProjectPointDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("point".into(), self.point.to_json());
        o.insert("maxDistance".into(), self.max_distance.to_json());
        o.insert("solid".into(), self.solid.to_json());
        o.insert("filter".into(), self.filter.to_json());
        Value::Object(o)
    }
}

/// `PointProjection`: packages/knowledge/wiki/physics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PointProjection {
    pub entity: u64,
    pub body: u64,
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub inside: bool,
}

impl Default for PointProjection {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            body: Default::default(),
            point: Default::default(),
            normal: Default::default(),
            distance: Default::default(),
            inside: Default::default(),
        }
    }
}

impl FromJson for PointProjection {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            body: field(o, "body", d.body)?,
            point: field(o, "point", d.point)?,
            normal: field(o, "normal", d.normal)?,
            distance: field(o, "distance", d.distance)?,
            inside: field(o, "inside", d.inside)?,
        })
    }
}

impl ToJson for PointProjection {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("body".into(), self.body.to_json());
        o.insert("point".into(), self.point.to_json());
        o.insert("normal".into(), self.normal.to_json());
        o.insert("distance".into(), self.distance.to_json());
        o.insert("inside".into(), self.inside.to_json());
        Value::Object(o)
    }
}

/// `ShapeDesc`: packages/knowledge/wiki/physics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeDesc {
    pub shape: u32,
    pub size: Vec3,
    pub radius: f32,
    pub height: f32,
    pub mesh: u64,
}

impl Default for ShapeDesc {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            size: Vec3::new(1.0_f32, 1.0_f32, 1.0_f32),
            radius: 0.5_f32,
            height: 1.0_f32,
            mesh: Default::default(),
        }
    }
}

impl FromJson for ShapeDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            shape: field(o, "shape", d.shape)?,
            size: field(o, "size", d.size)?,
            radius: field(o, "radius", d.radius)?,
            height: field(o, "height", d.height)?,
            mesh: field(o, "mesh", d.mesh)?,
        })
    }
}

impl ToJson for ShapeDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("shape".into(), self.shape.to_json());
        o.insert("size".into(), self.size.to_json());
        o.insert("radius".into(), self.radius.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("mesh".into(), self.mesh.to_json());
        Value::Object(o)
    }
}

/// `ShapeCastDesc`: packages/knowledge/wiki/physics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeCastDesc {
    pub shape: ShapeDesc,
    pub position: Vec3,
    pub rotation: Quat,
    pub direction: Vec3,
    pub max_distance: f32,
    pub filter: QueryFilter,
}

impl Default for ShapeCastDesc {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            position: Default::default(),
            rotation: Default::default(),
            direction: Vec3::new(0.0_f32, -1.0_f32, 0.0_f32),
            max_distance: Default::default(),
            filter: Default::default(),
        }
    }
}

impl FromJson for ShapeCastDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            shape: field(o, "shape", d.shape)?,
            position: field(o, "position", d.position)?,
            rotation: field(o, "rotation", d.rotation)?,
            direction: field(o, "direction", d.direction)?,
            max_distance: field(o, "maxDistance", d.max_distance)?,
            filter: field(o, "filter", d.filter)?,
        })
    }
}

impl ToJson for ShapeCastDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("shape".into(), self.shape.to_json());
        o.insert("position".into(), self.position.to_json());
        o.insert("rotation".into(), self.rotation.to_json());
        o.insert("direction".into(), self.direction.to_json());
        o.insert("maxDistance".into(), self.max_distance.to_json());
        o.insert("filter".into(), self.filter.to_json());
        Value::Object(o)
    }
}

/// `ShapeCastHit`: packages/knowledge/wiki/physics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeCastHit {
    pub entity: u64,
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
}

impl Default for ShapeCastHit {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            distance: Default::default(),
            point: Default::default(),
            normal: Default::default(),
        }
    }
}

impl FromJson for ShapeCastHit {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            distance: field(o, "distance", d.distance)?,
            point: field(o, "point", d.point)?,
            normal: field(o, "normal", d.normal)?,
        })
    }
}

impl ToJson for ShapeCastHit {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("distance".into(), self.distance.to_json());
        o.insert("point".into(), self.point.to_json());
        o.insert("normal".into(), self.normal.to_json());
        Value::Object(o)
    }
}

/// `OverlapDesc`: packages/knowledge/wiki/physics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlapDesc {
    pub shape: ShapeDesc,
    pub position: Vec3,
    pub rotation: Quat,
    pub filter: QueryFilter,
}

impl Default for OverlapDesc {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            position: Default::default(),
            rotation: Default::default(),
            filter: Default::default(),
        }
    }
}

impl FromJson for OverlapDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            shape: field(o, "shape", d.shape)?,
            position: field(o, "position", d.position)?,
            rotation: field(o, "rotation", d.rotation)?,
            filter: field(o, "filter", d.filter)?,
        })
    }
}

impl ToJson for OverlapDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("shape".into(), self.shape.to_json());
        o.insert("position".into(), self.position.to_json());
        o.insert("rotation".into(), self.rotation.to_json());
        o.insert("filter".into(), self.filter.to_json());
        Value::Object(o)
    }
}

/// `MeshDesc`: packages/knowledge/wiki/physics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshDesc {
    pub positions: Vec<Vec3>,
    pub indices: Vec<u32>,
}

impl Default for MeshDesc {
    fn default() -> Self {
        Self {
            positions: Default::default(),
            indices: Default::default(),
        }
    }
}

impl FromJson for MeshDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            positions: field(o, "positions", d.positions)?,
            indices: field(o, "indices", d.indices)?,
        })
    }
}

impl ToJson for MeshDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("positions".into(), self.positions.to_json());
        o.insert("indices".into(), self.indices.to_json());
        Value::Object(o)
    }
}

/// `WheelPose`: packages/knowledge/wiki/physics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct WheelPose {
    pub position: Vec3,
    pub rotation: Quat,
    pub contact: bool,
    pub slip: f32,
    pub side_speed: f32,
    pub entity: u64,
}

impl Default for WheelPose {
    fn default() -> Self {
        Self {
            position: Default::default(),
            rotation: Default::default(),
            contact: Default::default(),
            slip: Default::default(),
            side_speed: Default::default(),
            entity: Default::default(),
        }
    }
}

impl FromJson for WheelPose {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            position: field(o, "position", d.position)?,
            rotation: field(o, "rotation", d.rotation)?,
            contact: field(o, "contact", d.contact)?,
            slip: field(o, "slip", d.slip)?,
            side_speed: field(o, "sideSpeed", d.side_speed)?,
            entity: field(o, "entity", d.entity)?,
        })
    }
}

impl ToJson for WheelPose {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("position".into(), self.position.to_json());
        o.insert("rotation".into(), self.rotation.to_json());
        o.insert("contact".into(), self.contact.to_json());
        o.insert("slip".into(), self.slip.to_json());
        o.insert("sideSpeed".into(), self.side_speed.to_json());
        o.insert("entity".into(), self.entity.to_json());
        Value::Object(o)
    }
}

/// `RagdollBoneDesc`: packages/knowledge/wiki/physics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RagdollBoneDesc {
    pub parent: i32,
    pub position: Vec3,
    pub rotation: Quat,
    pub length: f32,
    pub axis: Vec3,
    pub joint: i32,
}

impl Default for RagdollBoneDesc {
    fn default() -> Self {
        Self {
            parent: -1,
            position: Default::default(),
            rotation: Default::default(),
            length: 0.25_f32,
            axis: Vec3::new(0.0_f32, 1.0_f32, 0.0_f32),
            joint: -1,
        }
    }
}

impl FromJson for RagdollBoneDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            parent: field(o, "parent", d.parent)?,
            position: field(o, "position", d.position)?,
            rotation: field(o, "rotation", d.rotation)?,
            length: field(o, "length", d.length)?,
            axis: field(o, "axis", d.axis)?,
            joint: field(o, "joint", d.joint)?,
        })
    }
}

impl ToJson for RagdollBoneDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("parent".into(), self.parent.to_json());
        o.insert("position".into(), self.position.to_json());
        o.insert("rotation".into(), self.rotation.to_json());
        o.insert("length".into(), self.length.to_json());
        o.insert("axis".into(), self.axis.to_json());
        o.insert("joint".into(), self.joint.to_json());
        Value::Object(o)
    }
}

/// `RagdollRigDesc`: packages/knowledge/wiki/physics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RagdollRigDesc {
    pub bones: Vec<RagdollBoneDesc>,
    pub skeleton: u64,
}

impl Default for RagdollRigDesc {
    fn default() -> Self {
        Self {
            bones: Default::default(),
            skeleton: Default::default(),
        }
    }
}

impl FromJson for RagdollRigDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            bones: field(o, "bones", d.bones)?,
            skeleton: field(o, "skeleton", d.skeleton)?,
        })
    }
}

impl ToJson for RagdollRigDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("bones".into(), self.bones.to_json());
        o.insert("skeleton".into(), self.skeleton.to_json());
        Value::Object(o)
    }
}

/// `BonePose`: packages/knowledge/wiki/physics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct BonePose {
    pub position: Vec3,
    pub rotation: Quat,
}

impl Default for BonePose {
    fn default() -> Self {
        Self {
            position: Default::default(),
            rotation: Default::default(),
        }
    }
}

impl FromJson for BonePose {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            position: field(o, "position", d.position)?,
            rotation: field(o, "rotation", d.rotation)?,
        })
    }
}

impl ToJson for BonePose {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("position".into(), self.position.to_json());
        o.insert("rotation".into(), self.rotation.to_json());
        Value::Object(o)
    }
}

/// `SoftBodyPoints`: packages/knowledge/wiki/physics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SoftBodyPoints {
    pub points: Vec<Vec3>,
    pub indices: Vec<u32>,
}

impl Default for SoftBodyPoints {
    fn default() -> Self {
        Self {
            points: Default::default(),
            indices: Default::default(),
        }
    }
}

impl FromJson for SoftBodyPoints {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            points: field(o, "points", d.points)?,
            indices: field(o, "indices", d.indices)?,
        })
    }
}

impl ToJson for SoftBodyPoints {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("points".into(), self.points.to_json());
        o.insert("indices".into(), self.indices.to_json());
        Value::Object(o)
    }
}

/// `PhysicsPhase`: packages/knowledge/wiki/physics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PhysicsPhase {
    pub name: String,
    pub ms: f64,
    pub split: bool,
}

impl Default for PhysicsPhase {
    fn default() -> Self {
        Self {
            name: Default::default(),
            ms: Default::default(),
            split: Default::default(),
        }
    }
}

impl FromJson for PhysicsPhase {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            ms: field(o, "ms", d.ms)?,
            split: field(o, "split", d.split)?,
        })
    }
}

impl ToJson for PhysicsPhase {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("ms".into(), self.ms.to_json());
        o.insert("split".into(), self.split.to_json());
        Value::Object(o)
    }
}

/// `PhysicsStats`: packages/knowledge/wiki/physics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PhysicsStats {
    pub tick: f64,
    pub bodies: u32,
    pub threads: u32,
    pub pool: bool,
    pub ms: f64,
    pub phases: Vec<PhysicsPhase>,
}

impl Default for PhysicsStats {
    fn default() -> Self {
        Self {
            tick: Default::default(),
            bodies: Default::default(),
            threads: Default::default(),
            pool: Default::default(),
            ms: Default::default(),
            phases: Default::default(),
        }
    }
}

impl FromJson for PhysicsStats {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tick: field(o, "tick", d.tick)?,
            bodies: field(o, "bodies", d.bodies)?,
            threads: field(o, "threads", d.threads)?,
            pool: field(o, "pool", d.pool)?,
            ms: field(o, "ms", d.ms)?,
            phases: field(o, "phases", d.phases)?,
        })
    }
}

impl ToJson for PhysicsStats {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tick".into(), self.tick.to_json());
        o.insert("bodies".into(), self.bodies.to_json());
        o.insert("threads".into(), self.threads.to_json());
        o.insert("pool".into(), self.pool.to_json());
        o.insert("ms".into(), self.ms.to_json());
        o.insert("phases".into(), self.phases.to_json());
        Value::Object(o)
    }
}

/// `ClipInfo`: packages/knowledge/wiki/anim.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ClipInfo {
    pub clip: u64,
    pub name: String,
    pub duration: f32,
    pub markers: Vec<String>,
}

impl Default for ClipInfo {
    fn default() -> Self {
        Self {
            clip: Default::default(),
            name: Default::default(),
            duration: Default::default(),
            markers: Default::default(),
        }
    }
}

impl FromJson for ClipInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            clip: field(o, "clip", d.clip)?,
            name: field(o, "name", d.name)?,
            duration: field(o, "duration", d.duration)?,
            markers: field(o, "markers", d.markers)?,
        })
    }
}

impl ToJson for ClipInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("clip".into(), self.clip.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("duration".into(), self.duration.to_json());
        o.insert("markers".into(), self.markers.to_json());
        Value::Object(o)
    }
}

/// `Animation`: packages/knowledge/wiki/anim.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Animation {
    pub skeleton: u64,
    pub clips: Vec<ClipInfo>,
    pub nodes: Vec<String>,
    pub skins: Vec<u32>,
    pub morph_targets: u32,
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            skeleton: Default::default(),
            clips: Default::default(),
            nodes: Default::default(),
            skins: Default::default(),
            morph_targets: Default::default(),
        }
    }
}

impl FromJson for Animation {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            skeleton: field(o, "skeleton", d.skeleton)?,
            clips: field(o, "clips", d.clips)?,
            nodes: field(o, "nodes", d.nodes)?,
            skins: field(o, "skins", d.skins)?,
            morph_targets: field(o, "morphTargets", d.morph_targets)?,
        })
    }
}

impl ToJson for Animation {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("skeleton".into(), self.skeleton.to_json());
        o.insert("clips".into(), self.clips.to_json());
        o.insert("nodes".into(), self.nodes.to_json());
        o.insert("skins".into(), self.skins.to_json());
        o.insert("morphTargets".into(), self.morph_targets.to_json());
        Value::Object(o)
    }
}

/// `GraphDesc`: packages/knowledge/wiki/anim.md.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphDesc {
    pub text: String,
    pub clips: Vec<ClipInfo>,
    pub sprite: bool,
}

impl Default for GraphDesc {
    fn default() -> Self {
        Self {
            text: Default::default(),
            clips: Default::default(),
            sprite: Default::default(),
        }
    }
}

impl FromJson for GraphDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            text: field(o, "text", d.text)?,
            clips: field(o, "clips", d.clips)?,
            sprite: field(o, "sprite", d.sprite)?,
        })
    }
}

impl ToJson for GraphDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("text".into(), self.text.to_json());
        o.insert("clips".into(), self.clips.to_json());
        o.insert("sprite".into(), self.sprite.to_json());
        Value::Object(o)
    }
}

/// `HumanBone`: packages/knowledge/wiki/anim.md.
#[derive(Clone, Debug, PartialEq)]
pub struct HumanBone {
    pub bone: String,
    pub node: String,
}

impl Default for HumanBone {
    fn default() -> Self {
        Self {
            bone: Default::default(),
            node: Default::default(),
        }
    }
}

impl FromJson for HumanBone {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            bone: field(o, "bone", d.bone)?,
            node: field(o, "node", d.node)?,
        })
    }
}

impl ToJson for HumanBone {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("bone".into(), self.bone.to_json());
        o.insert("node".into(), self.node.to_json());
        Value::Object(o)
    }
}

/// `Humanoid`: packages/knowledge/wiki/anim.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Humanoid {
    pub scheme: String,
    pub bones: Vec<HumanBone>,
}

impl Default for Humanoid {
    fn default() -> Self {
        Self {
            scheme: Default::default(),
            bones: Default::default(),
        }
    }
}

impl FromJson for Humanoid {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            scheme: field(o, "scheme", d.scheme)?,
            bones: field(o, "bones", d.bones)?,
        })
    }
}

impl ToJson for Humanoid {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("scheme".into(), self.scheme.to_json());
        o.insert("bones".into(), self.bones.to_json());
        Value::Object(o)
    }
}

/// `HumanoidDesc`: packages/knowledge/wiki/anim.md.
#[derive(Clone, Debug, PartialEq)]
pub struct HumanoidDesc {
    pub skeleton: u64,
    pub rig: u64,
}

impl Default for HumanoidDesc {
    fn default() -> Self {
        Self {
            skeleton: Default::default(),
            rig: Default::default(),
        }
    }
}

impl FromJson for HumanoidDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            skeleton: field(o, "skeleton", d.skeleton)?,
            rig: field(o, "rig", d.rig)?,
        })
    }
}

impl ToJson for HumanoidDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("skeleton".into(), self.skeleton.to_json());
        o.insert("rig".into(), self.rig.to_json());
        Value::Object(o)
    }
}

/// `RetargetDesc`: packages/knowledge/wiki/anim.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RetargetDesc {
    pub clip: u64,
    pub source: u64,
    pub target: u64,
    pub source_rig: u64,
    pub target_rig: u64,
    pub foot_ik: bool,
}

impl Default for RetargetDesc {
    fn default() -> Self {
        Self {
            clip: Default::default(),
            source: Default::default(),
            target: Default::default(),
            source_rig: Default::default(),
            target_rig: Default::default(),
            foot_ik: true,
        }
    }
}

impl FromJson for RetargetDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            clip: field(o, "clip", d.clip)?,
            source: field(o, "source", d.source)?,
            target: field(o, "target", d.target)?,
            source_rig: field(o, "sourceRig", d.source_rig)?,
            target_rig: field(o, "targetRig", d.target_rig)?,
            foot_ik: field(o, "footIk", d.foot_ik)?,
        })
    }
}

impl ToJson for RetargetDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("clip".into(), self.clip.to_json());
        o.insert("source".into(), self.source.to_json());
        o.insert("target".into(), self.target.to_json());
        o.insert("sourceRig".into(), self.source_rig.to_json());
        o.insert("targetRig".into(), self.target_rig.to_json());
        o.insert("footIk".into(), self.foot_ik.to_json());
        Value::Object(o)
    }
}

/// `MirrorDesc`: packages/knowledge/wiki/anim.md.
#[derive(Clone, Debug, PartialEq)]
pub struct MirrorDesc {
    pub clip: u64,
    pub skeleton: u64,
}

impl Default for MirrorDesc {
    fn default() -> Self {
        Self {
            clip: Default::default(),
            skeleton: Default::default(),
        }
    }
}

impl FromJson for MirrorDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            clip: field(o, "clip", d.clip)?,
            skeleton: field(o, "skeleton", d.skeleton)?,
        })
    }
}

impl ToJson for MirrorDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("clip".into(), self.clip.to_json());
        o.insert("skeleton".into(), self.skeleton.to_json());
        Value::Object(o)
    }
}

/// `FlipMode`: packages/knowledge/wiki/anim.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FlipMode {
    #[default]
    Loop,
    PingPong,
    Once,
}

impl FlipMode {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["loop", "pingPong", "once"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Loop, Self::PingPong, Self::Once];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("FlipMode", s, Self::NAMES)),
        }
    }
}

impl FromJson for FlipMode {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "FlipMode", Self::NAMES)?)
    }
}

impl ToJson for FlipMode {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `FlipbookClipInfo`: packages/knowledge/wiki/anim.md.
#[derive(Clone, Debug, PartialEq)]
pub struct FlipbookClipInfo {
    pub clip: u32,
    pub name: String,
    pub frames: u32,
    pub duration: f32,
    pub mode: FlipMode,
    pub events: Vec<String>,
}

impl Default for FlipbookClipInfo {
    fn default() -> Self {
        Self {
            clip: Default::default(),
            name: Default::default(),
            frames: Default::default(),
            duration: Default::default(),
            mode: Default::default(),
            events: Default::default(),
        }
    }
}

impl FromJson for FlipbookClipInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            clip: field(o, "clip", d.clip)?,
            name: field(o, "name", d.name)?,
            frames: field(o, "frames", d.frames)?,
            duration: field(o, "duration", d.duration)?,
            mode: field(o, "mode", d.mode)?,
            events: field(o, "events", d.events)?,
        })
    }
}

impl ToJson for FlipbookClipInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("clip".into(), self.clip.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("frames".into(), self.frames.to_json());
        o.insert("duration".into(), self.duration.to_json());
        o.insert("mode".into(), self.mode.to_json());
        o.insert("events".into(), self.events.to_json());
        Value::Object(o)
    }
}

/// `FlipbookInfo`: packages/knowledge/wiki/anim.md.
#[derive(Clone, Debug, PartialEq)]
pub struct FlipbookInfo {
    pub sheet: u64,
    pub texture: u64,
    pub array: u64,
    pub clips: Vec<FlipbookClipInfo>,
    pub frames: u32,
    pub frame_width: u32,
    pub frame_height: u32,
}

impl Default for FlipbookInfo {
    fn default() -> Self {
        Self {
            sheet: Default::default(),
            texture: Default::default(),
            array: Default::default(),
            clips: Default::default(),
            frames: Default::default(),
            frame_width: Default::default(),
            frame_height: Default::default(),
        }
    }
}

impl FromJson for FlipbookInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            sheet: field(o, "sheet", d.sheet)?,
            texture: field(o, "texture", d.texture)?,
            array: field(o, "array", d.array)?,
            clips: field(o, "clips", d.clips)?,
            frames: field(o, "frames", d.frames)?,
            frame_width: field(o, "frameWidth", d.frame_width)?,
            frame_height: field(o, "frameHeight", d.frame_height)?,
        })
    }
}

impl ToJson for FlipbookInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("sheet".into(), self.sheet.to_json());
        o.insert("texture".into(), self.texture.to_json());
        o.insert("array".into(), self.array.to_json());
        o.insert("clips".into(), self.clips.to_json());
        o.insert("frames".into(), self.frames.to_json());
        o.insert("frameWidth".into(), self.frame_width.to_json());
        o.insert("frameHeight".into(), self.frame_height.to_json());
        Value::Object(o)
    }
}

/// `Rig2DClipInfo`: packages/knowledge/wiki/anim.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Rig2DClipInfo {
    pub clip: u32,
    pub name: String,
    pub duration: f32,
    pub r#loop: bool,
    pub events: Vec<String>,
}

impl Default for Rig2DClipInfo {
    fn default() -> Self {
        Self {
            clip: Default::default(),
            name: Default::default(),
            duration: Default::default(),
            r#loop: Default::default(),
            events: Default::default(),
        }
    }
}

impl FromJson for Rig2DClipInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            clip: field(o, "clip", d.clip)?,
            name: field(o, "name", d.name)?,
            duration: field(o, "duration", d.duration)?,
            r#loop: field(o, "loop", d.r#loop)?,
            events: field(o, "events", d.events)?,
        })
    }
}

impl ToJson for Rig2DClipInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("clip".into(), self.clip.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("duration".into(), self.duration.to_json());
        o.insert("loop".into(), self.r#loop.to_json());
        o.insert("events".into(), self.events.to_json());
        Value::Object(o)
    }
}

/// `Rig2DInfo`: packages/knowledge/wiki/anim.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Rig2DInfo {
    pub rig: u64,
    pub texture: u64,
    pub unit: f32,
    pub bones: Vec<String>,
    pub slots: Vec<String>,
    pub constraints: Vec<String>,
    pub skins: Vec<String>,
    pub clips: Vec<Rig2DClipInfo>,
}

impl Default for Rig2DInfo {
    fn default() -> Self {
        Self {
            rig: Default::default(),
            texture: Default::default(),
            unit: Default::default(),
            bones: Default::default(),
            slots: Default::default(),
            constraints: Default::default(),
            skins: Default::default(),
            clips: Default::default(),
        }
    }
}

impl FromJson for Rig2DInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            rig: field(o, "rig", d.rig)?,
            texture: field(o, "texture", d.texture)?,
            unit: field(o, "unit", d.unit)?,
            bones: field(o, "bones", d.bones)?,
            slots: field(o, "slots", d.slots)?,
            constraints: field(o, "constraints", d.constraints)?,
            skins: field(o, "skins", d.skins)?,
            clips: field(o, "clips", d.clips)?,
        })
    }
}

impl ToJson for Rig2DInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("rig".into(), self.rig.to_json());
        o.insert("texture".into(), self.texture.to_json());
        o.insert("unit".into(), self.unit.to_json());
        o.insert("bones".into(), self.bones.to_json());
        o.insert("slots".into(), self.slots.to_json());
        o.insert("constraints".into(), self.constraints.to_json());
        o.insert("skins".into(), self.skins.to_json());
        o.insert("clips".into(), self.clips.to_json());
        Value::Object(o)
    }
}

/// `Skeleton2DBox`: packages/knowledge/wiki/anim.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Skeleton2DBox {
    pub slot: String,
    pub name: String,
    pub points: Vec<Vec2>,
}

impl Default for Skeleton2DBox {
    fn default() -> Self {
        Self {
            slot: Default::default(),
            name: Default::default(),
            points: Default::default(),
        }
    }
}

impl FromJson for Skeleton2DBox {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            slot: field(o, "slot", d.slot)?,
            name: field(o, "name", d.name)?,
            points: field(o, "points", d.points)?,
        })
    }
}

impl ToJson for Skeleton2DBox {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("slot".into(), self.slot.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("points".into(), self.points.to_json());
        Value::Object(o)
    }
}

/// `TweenDesc`: packages/knowledge/wiki/tween.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TweenDesc {
    pub entity: u64,
    pub component: Option<Value>,
    pub field: String,
    pub to: Option<OneOf<f32, Vec<f32>>>,
    pub from: Option<OneOf<f32, Vec<f32>>>,
    pub relative: bool,
    pub duration: f32,
    pub delay: f32,
    pub ease: String,
    pub repeat: i32,
    pub yoyo: bool,
    pub unscaled: bool,
    pub tag: u32,
}

impl Default for TweenDesc {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            component: Default::default(),
            field: Default::default(),
            to: Default::default(),
            from: Default::default(),
            relative: Default::default(),
            duration: 0.3_f32,
            delay: Default::default(),
            ease: Default::default(),
            repeat: Default::default(),
            yoyo: Default::default(),
            unscaled: Default::default(),
            tag: Default::default(),
        }
    }
}

impl FromJson for TweenDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            component: field(o, "component", d.component)?,
            field: field(o, "field", d.field)?,
            to: field(o, "to", d.to)?,
            from: field(o, "from", d.from)?,
            relative: field(o, "relative", d.relative)?,
            duration: field(o, "duration", d.duration)?,
            delay: field(o, "delay", d.delay)?,
            ease: field(o, "ease", d.ease)?,
            repeat: field(o, "repeat", d.repeat)?,
            yoyo: field(o, "yoyo", d.yoyo)?,
            unscaled: field(o, "unscaled", d.unscaled)?,
            tag: field(o, "tag", d.tag)?,
        })
    }
}

impl ToJson for TweenDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("component".into(), self.component.to_json());
        o.insert("field".into(), self.field.to_json());
        o.insert("to".into(), self.to.to_json());
        o.insert("from".into(), self.from.to_json());
        o.insert("relative".into(), self.relative.to_json());
        o.insert("duration".into(), self.duration.to_json());
        o.insert("delay".into(), self.delay.to_json());
        o.insert("ease".into(), self.ease.to_json());
        o.insert("repeat".into(), self.repeat.to_json());
        o.insert("yoyo".into(), self.yoyo.to_json());
        o.insert("unscaled".into(), self.unscaled.to_json());
        o.insert("tag".into(), self.tag.to_json());
        Value::Object(o)
    }
}

/// `TweenFromDesc`: packages/knowledge/wiki/tween.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TweenFromDesc {
    pub entity: u64,
    pub component: Option<Value>,
    pub field: String,
    pub from: Option<OneOf<f32, Vec<f32>>>,
    pub duration: f32,
    pub delay: f32,
    pub ease: String,
    pub repeat: i32,
    pub yoyo: bool,
    pub unscaled: bool,
    pub tag: u32,
}

impl Default for TweenFromDesc {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            component: Default::default(),
            field: Default::default(),
            from: Default::default(),
            duration: 0.3_f32,
            delay: Default::default(),
            ease: Default::default(),
            repeat: Default::default(),
            yoyo: Default::default(),
            unscaled: Default::default(),
            tag: Default::default(),
        }
    }
}

impl FromJson for TweenFromDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            component: field(o, "component", d.component)?,
            field: field(o, "field", d.field)?,
            from: field(o, "from", d.from)?,
            duration: field(o, "duration", d.duration)?,
            delay: field(o, "delay", d.delay)?,
            ease: field(o, "ease", d.ease)?,
            repeat: field(o, "repeat", d.repeat)?,
            yoyo: field(o, "yoyo", d.yoyo)?,
            unscaled: field(o, "unscaled", d.unscaled)?,
            tag: field(o, "tag", d.tag)?,
        })
    }
}

impl ToJson for TweenFromDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("component".into(), self.component.to_json());
        o.insert("field".into(), self.field.to_json());
        o.insert("from".into(), self.from.to_json());
        o.insert("duration".into(), self.duration.to_json());
        o.insert("delay".into(), self.delay.to_json());
        o.insert("ease".into(), self.ease.to_json());
        o.insert("repeat".into(), self.repeat.to_json());
        o.insert("yoyo".into(), self.yoyo.to_json());
        o.insert("unscaled".into(), self.unscaled.to_json());
        o.insert("tag".into(), self.tag.to_json());
        Value::Object(o)
    }
}

/// `TweenGroupDesc`: packages/knowledge/wiki/tween.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TweenGroupDesc {
    pub repeat: i32,
    pub unscaled: bool,
    pub tag: u32,
}

impl Default for TweenGroupDesc {
    fn default() -> Self {
        Self {
            repeat: Default::default(),
            unscaled: Default::default(),
            tag: Default::default(),
        }
    }
}

impl FromJson for TweenGroupDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            repeat: field(o, "repeat", d.repeat)?,
            unscaled: field(o, "unscaled", d.unscaled)?,
            tag: field(o, "tag", d.tag)?,
        })
    }
}

impl ToJson for TweenGroupDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("repeat".into(), self.repeat.to_json());
        o.insert("unscaled".into(), self.unscaled.to_json());
        o.insert("tag".into(), self.tag.to_json());
        Value::Object(o)
    }
}

/// `TweenDefaults`: packages/knowledge/wiki/tween.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TweenDefaults {
    pub component: Option<Value>,
    pub duration: f32,
    pub ease: String,
}

impl Default for TweenDefaults {
    fn default() -> Self {
        Self {
            component: Default::default(),
            duration: 0.3_f32,
            ease: Default::default(),
        }
    }
}

impl FromJson for TweenDefaults {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            component: field(o, "component", d.component)?,
            duration: field(o, "duration", d.duration)?,
            ease: field(o, "ease", d.ease)?,
        })
    }
}

impl ToJson for TweenDefaults {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("component".into(), self.component.to_json());
        o.insert("duration".into(), self.duration.to_json());
        o.insert("ease".into(), self.ease.to_json());
        Value::Object(o)
    }
}

/// `TimelineDesc`: packages/knowledge/wiki/tween.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineDesc {
    pub defaults: TweenDefaults,
    pub repeat: i32,
    pub yoyo: bool,
    pub repeat_delay: f32,
    pub paused: bool,
    pub time_scale: f32,
    pub auto_kill: bool,
    pub unscaled: bool,
    pub tag: u32,
    pub on_start: Option<String>,
    pub on_update: Option<String>,
    pub on_complete: Option<String>,
    pub on_repeat: Option<String>,
}

impl Default for TimelineDesc {
    fn default() -> Self {
        Self {
            defaults: Default::default(),
            repeat: Default::default(),
            yoyo: Default::default(),
            repeat_delay: Default::default(),
            paused: Default::default(),
            time_scale: 1.0_f32,
            auto_kill: Default::default(),
            unscaled: Default::default(),
            tag: Default::default(),
            on_start: Default::default(),
            on_update: Default::default(),
            on_complete: Default::default(),
            on_repeat: Default::default(),
        }
    }
}

impl FromJson for TimelineDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            defaults: field(o, "defaults", d.defaults)?,
            repeat: field(o, "repeat", d.repeat)?,
            yoyo: field(o, "yoyo", d.yoyo)?,
            repeat_delay: field(o, "repeatDelay", d.repeat_delay)?,
            paused: field(o, "paused", d.paused)?,
            time_scale: field(o, "timeScale", d.time_scale)?,
            auto_kill: field(o, "autoKill", d.auto_kill)?,
            unscaled: field(o, "unscaled", d.unscaled)?,
            tag: field(o, "tag", d.tag)?,
            on_start: field(o, "onStart", d.on_start)?,
            on_update: field(o, "onUpdate", d.on_update)?,
            on_complete: field(o, "onComplete", d.on_complete)?,
            on_repeat: field(o, "onRepeat", d.on_repeat)?,
        })
    }
}

impl ToJson for TimelineDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("defaults".into(), self.defaults.to_json());
        o.insert("repeat".into(), self.repeat.to_json());
        o.insert("yoyo".into(), self.yoyo.to_json());
        o.insert("repeatDelay".into(), self.repeat_delay.to_json());
        o.insert("paused".into(), self.paused.to_json());
        o.insert("timeScale".into(), self.time_scale.to_json());
        o.insert("autoKill".into(), self.auto_kill.to_json());
        o.insert("unscaled".into(), self.unscaled.to_json());
        o.insert("tag".into(), self.tag.to_json());
        o.insert("onStart".into(), self.on_start.to_json());
        o.insert("onUpdate".into(), self.on_update.to_json());
        o.insert("onComplete".into(), self.on_complete.to_json());
        o.insert("onRepeat".into(), self.on_repeat.to_json());
        Value::Object(o)
    }
}

/// `StaggerDesc`: packages/knowledge/wiki/tween.md.
#[derive(Clone, Debug, PartialEq)]
pub struct StaggerDesc {
    pub each: Option<f32>,
    pub amount: Option<f32>,
    pub from: Option<OneOf<String, u32>>,
    pub grid: Option<Vec<u32>>,
    pub ease: String,
}

impl Default for StaggerDesc {
    fn default() -> Self {
        Self {
            each: Default::default(),
            amount: Default::default(),
            from: Default::default(),
            grid: Default::default(),
            ease: String::from("none"),
        }
    }
}

impl FromJson for StaggerDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            each: field(o, "each", d.each)?,
            amount: field(o, "amount", d.amount)?,
            from: field(o, "from", d.from)?,
            grid: field(o, "grid", d.grid)?,
            ease: field(o, "ease", d.ease)?,
        })
    }
}

impl ToJson for StaggerDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("each".into(), self.each.to_json());
        o.insert("amount".into(), self.amount.to_json());
        o.insert("from".into(), self.from.to_json());
        o.insert("grid".into(), self.grid.to_json());
        o.insert("ease".into(), self.ease.to_json());
        Value::Object(o)
    }
}

/// `TweenKeyframe`: packages/knowledge/wiki/tween.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TweenKeyframe {
    pub to: Option<OneOf<f32, Vec<f32>>>,
    pub duration: Option<f32>,
    pub ease: Option<String>,
    pub delay: f32,
}

impl Default for TweenKeyframe {
    fn default() -> Self {
        Self {
            to: Default::default(),
            duration: Default::default(),
            ease: Default::default(),
            delay: Default::default(),
        }
    }
}

impl FromJson for TweenKeyframe {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            to: field(o, "to", d.to)?,
            duration: field(o, "duration", d.duration)?,
            ease: field(o, "ease", d.ease)?,
            delay: field(o, "delay", d.delay)?,
        })
    }
}

impl ToJson for TweenKeyframe {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("to".into(), self.to.to_json());
        o.insert("duration".into(), self.duration.to_json());
        o.insert("ease".into(), self.ease.to_json());
        o.insert("delay".into(), self.delay.to_json());
        Value::Object(o)
    }
}

/// `TimelineTweenVars`: packages/knowledge/wiki/tween.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineTweenVars {
    pub component: Option<Value>,
    pub field: String,
    pub to: Option<OneOf<f32, Vec<f32>>>,
    pub from: Option<OneOf<f32, Vec<f32>>>,
    pub relative: bool,
    pub duration: Option<f32>,
    pub delay: f32,
    pub ease: Option<String>,
    pub repeat: i32,
    pub yoyo: bool,
    pub repeat_delay: f32,
    pub stagger: Option<OneOf<f32, StaggerDesc>>,
    pub keyframes: Option<Vec<TweenKeyframe>>,
    pub on_start: Option<String>,
    pub on_update: Option<String>,
    pub on_complete: Option<String>,
    pub on_repeat: Option<String>,
}

impl Default for TimelineTweenVars {
    fn default() -> Self {
        Self {
            component: Default::default(),
            field: Default::default(),
            to: Default::default(),
            from: Default::default(),
            relative: Default::default(),
            duration: Default::default(),
            delay: Default::default(),
            ease: Default::default(),
            repeat: Default::default(),
            yoyo: Default::default(),
            repeat_delay: Default::default(),
            stagger: Default::default(),
            keyframes: Default::default(),
            on_start: Default::default(),
            on_update: Default::default(),
            on_complete: Default::default(),
            on_repeat: Default::default(),
        }
    }
}

impl FromJson for TimelineTweenVars {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            component: field(o, "component", d.component)?,
            field: field(o, "field", d.field)?,
            to: field(o, "to", d.to)?,
            from: field(o, "from", d.from)?,
            relative: field(o, "relative", d.relative)?,
            duration: field(o, "duration", d.duration)?,
            delay: field(o, "delay", d.delay)?,
            ease: field(o, "ease", d.ease)?,
            repeat: field(o, "repeat", d.repeat)?,
            yoyo: field(o, "yoyo", d.yoyo)?,
            repeat_delay: field(o, "repeatDelay", d.repeat_delay)?,
            stagger: field(o, "stagger", d.stagger)?,
            keyframes: field(o, "keyframes", d.keyframes)?,
            on_start: field(o, "onStart", d.on_start)?,
            on_update: field(o, "onUpdate", d.on_update)?,
            on_complete: field(o, "onComplete", d.on_complete)?,
            on_repeat: field(o, "onRepeat", d.on_repeat)?,
        })
    }
}

impl ToJson for TimelineTweenVars {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("component".into(), self.component.to_json());
        o.insert("field".into(), self.field.to_json());
        o.insert("to".into(), self.to.to_json());
        o.insert("from".into(), self.from.to_json());
        o.insert("relative".into(), self.relative.to_json());
        o.insert("duration".into(), self.duration.to_json());
        o.insert("delay".into(), self.delay.to_json());
        o.insert("ease".into(), self.ease.to_json());
        o.insert("repeat".into(), self.repeat.to_json());
        o.insert("yoyo".into(), self.yoyo.to_json());
        o.insert("repeatDelay".into(), self.repeat_delay.to_json());
        o.insert("stagger".into(), self.stagger.to_json());
        o.insert("keyframes".into(), self.keyframes.to_json());
        o.insert("onStart".into(), self.on_start.to_json());
        o.insert("onUpdate".into(), self.on_update.to_json());
        o.insert("onComplete".into(), self.on_complete.to_json());
        o.insert("onRepeat".into(), self.on_repeat.to_json());
        Value::Object(o)
    }
}

/// `SquashDesc`: packages/knowledge/wiki/tween.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SquashDesc {
    pub entity: u64,
    pub impact: Value,
    pub rest: Vec3,
    pub duration: f32,
    pub ease: String,
}

impl Default for SquashDesc {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            impact: Default::default(),
            rest: Vec3::new(1.0_f32, 1.0_f32, 1.0_f32),
            duration: 0.18_f32,
            ease: String::from("back.out"),
        }
    }
}

impl FromJson for SquashDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            impact: field(o, "impact", d.impact)?,
            rest: field(o, "rest", d.rest)?,
            duration: field(o, "duration", d.duration)?,
            ease: field(o, "ease", d.ease)?,
        })
    }
}

impl ToJson for SquashDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("impact".into(), self.impact.to_json());
        o.insert("rest".into(), self.rest.to_json());
        o.insert("duration".into(), self.duration.to_json());
        o.insert("ease".into(), self.ease.to_json());
        Value::Object(o)
    }
}

/// `FlashDesc`: packages/knowledge/wiki/tween.md.
#[derive(Clone, Debug, PartialEq)]
pub struct FlashDesc {
    pub entity: u64,
    pub color: Color,
    pub amount: f32,
    pub duration: f32,
    pub ease: String,
}

impl Default for FlashDesc {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            color: Color { r: 1.0_f32, g: 1.0_f32, b: 1.0_f32 },
            amount: 1.0_f32,
            duration: 0.12_f32,
            ease: String::from("power1.in"),
        }
    }
}

impl FromJson for FlashDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            color: field(o, "color", d.color)?,
            amount: field(o, "amount", d.amount)?,
            duration: field(o, "duration", d.duration)?,
            ease: field(o, "ease", d.ease)?,
        })
    }
}

impl ToJson for FlashDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("color".into(), self.color.to_json());
        o.insert("amount".into(), self.amount.to_json());
        o.insert("duration".into(), self.duration.to_json());
        o.insert("ease".into(), self.ease.to_json());
        Value::Object(o)
    }
}

/// `TileOrientation`: packages/knowledge/wiki/tilemap.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TileOrientation {
    #[default]
    Orthogonal,
    Isometric,
    Staggered,
    Hex,
    HexFlat,
}

impl TileOrientation {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["orthogonal", "isometric", "staggered", "hex", "hexFlat"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Orthogonal, Self::Isometric, Self::Staggered, Self::Hex, Self::HexFlat];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("TileOrientation", s, Self::NAMES)),
        }
    }
}

impl FromJson for TileOrientation {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "TileOrientation", Self::NAMES)?)
    }
}

impl ToJson for TileOrientation {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `TileLayerInfo`: packages/knowledge/wiki/tilemap.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TileLayerInfo {
    pub name: String,
    pub index: u32,
    pub z: i32,
    pub parallax: Vec2,
    pub collision: bool,
    pub collider_layers: u32,
    pub collider_mask: u32,
    pub cell_colliders: bool,
    pub terrain: String,
    pub visible: bool,
    pub lit: bool,
}

impl Default for TileLayerInfo {
    fn default() -> Self {
        Self {
            name: Default::default(),
            index: Default::default(),
            z: Default::default(),
            parallax: Default::default(),
            collision: Default::default(),
            collider_layers: Default::default(),
            collider_mask: Default::default(),
            cell_colliders: Default::default(),
            terrain: Default::default(),
            visible: Default::default(),
            lit: Default::default(),
        }
    }
}

impl FromJson for TileLayerInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            index: field(o, "index", d.index)?,
            z: field(o, "z", d.z)?,
            parallax: field(o, "parallax", d.parallax)?,
            collision: field(o, "collision", d.collision)?,
            collider_layers: field(o, "colliderLayers", d.collider_layers)?,
            collider_mask: field(o, "colliderMask", d.collider_mask)?,
            cell_colliders: field(o, "cellColliders", d.cell_colliders)?,
            terrain: field(o, "terrain", d.terrain)?,
            visible: field(o, "visible", d.visible)?,
            lit: field(o, "lit", d.lit)?,
        })
    }
}

impl ToJson for TileLayerInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("index".into(), self.index.to_json());
        o.insert("z".into(), self.z.to_json());
        o.insert("parallax".into(), self.parallax.to_json());
        o.insert("collision".into(), self.collision.to_json());
        o.insert("colliderLayers".into(), self.collider_layers.to_json());
        o.insert("colliderMask".into(), self.collider_mask.to_json());
        o.insert("cellColliders".into(), self.cell_colliders.to_json());
        o.insert("terrain".into(), self.terrain.to_json());
        o.insert("visible".into(), self.visible.to_json());
        o.insert("lit".into(), self.lit.to_json());
        Value::Object(o)
    }
}

/// `TileObject`: packages/knowledge/wiki/tilemap.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TileObject {
    pub name: String,
    pub kind: String,
    pub position: Vec2,
    pub size: Vec2,
    pub center: Vec2,
    pub properties: BTreeMap<String, String>,
}

impl Default for TileObject {
    fn default() -> Self {
        Self {
            name: Default::default(),
            kind: Default::default(),
            position: Default::default(),
            size: Default::default(),
            center: Default::default(),
            properties: Default::default(),
        }
    }
}

impl FromJson for TileObject {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            kind: field(o, "kind", d.kind)?,
            position: field(o, "position", d.position)?,
            size: field(o, "size", d.size)?,
            center: field(o, "center", d.center)?,
            properties: field(o, "properties", d.properties)?,
        })
    }
}

impl ToJson for TileObject {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("position".into(), self.position.to_json());
        o.insert("size".into(), self.size.to_json());
        o.insert("center".into(), self.center.to_json());
        o.insert("properties".into(), self.properties.to_json());
        Value::Object(o)
    }
}

/// `TilemapInfo`: packages/knowledge/wiki/tilemap.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TilemapInfo {
    pub map: u64,
    pub width: u32,
    pub height: u32,
    pub tile_width: f32,
    pub tile_height: f32,
    pub layers: Vec<TileLayerInfo>,
    pub objects: Vec<TileObject>,
    pub orientation: TileOrientation,
}

impl Default for TilemapInfo {
    fn default() -> Self {
        Self {
            map: Default::default(),
            width: Default::default(),
            height: Default::default(),
            tile_width: Default::default(),
            tile_height: Default::default(),
            layers: Default::default(),
            objects: Default::default(),
            orientation: Default::default(),
        }
    }
}

impl FromJson for TilemapInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            map: field(o, "map", d.map)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            tile_width: field(o, "tileWidth", d.tile_width)?,
            tile_height: field(o, "tileHeight", d.tile_height)?,
            layers: field(o, "layers", d.layers)?,
            objects: field(o, "objects", d.objects)?,
            orientation: field(o, "orientation", d.orientation)?,
        })
    }
}

impl ToJson for TilemapInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("map".into(), self.map.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("tileWidth".into(), self.tile_width.to_json());
        o.insert("tileHeight".into(), self.tile_height.to_json());
        o.insert("layers".into(), self.layers.to_json());
        o.insert("objects".into(), self.objects.to_json());
        o.insert("orientation".into(), self.orientation.to_json());
        Value::Object(o)
    }
}

/// `TileLayerDesc`: packages/knowledge/wiki/tilemap.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TileLayerDesc {
    pub name: String,
    pub tileset: u32,
    pub terrain: String,
    pub z: i32,
    pub collision: bool,
    pub collider_layers: u32,
    pub collider_mask: u32,
    pub cell_colliders: bool,
    pub visible: bool,
}

impl Default for TileLayerDesc {
    fn default() -> Self {
        Self {
            name: Default::default(),
            tileset: Default::default(),
            terrain: Default::default(),
            z: Default::default(),
            collision: Default::default(),
            collider_layers: Default::default(),
            collider_mask: Default::default(),
            cell_colliders: Default::default(),
            visible: true,
        }
    }
}

impl FromJson for TileLayerDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            tileset: field(o, "tileset", d.tileset)?,
            terrain: field(o, "terrain", d.terrain)?,
            z: field(o, "z", d.z)?,
            collision: field(o, "collision", d.collision)?,
            collider_layers: field(o, "colliderLayers", d.collider_layers)?,
            collider_mask: field(o, "colliderMask", d.collider_mask)?,
            cell_colliders: field(o, "cellColliders", d.cell_colliders)?,
            visible: field(o, "visible", d.visible)?,
        })
    }
}

impl ToJson for TileLayerDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("tileset".into(), self.tileset.to_json());
        o.insert("terrain".into(), self.terrain.to_json());
        o.insert("z".into(), self.z.to_json());
        o.insert("collision".into(), self.collision.to_json());
        o.insert("colliderLayers".into(), self.collider_layers.to_json());
        o.insert("colliderMask".into(), self.collider_mask.to_json());
        o.insert("cellColliders".into(), self.cell_colliders.to_json());
        o.insert("visible".into(), self.visible.to_json());
        Value::Object(o)
    }
}

/// `TilemapDesc`: packages/knowledge/wiki/tilemap.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TilemapDesc {
    pub width: u32,
    pub height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub unit: f32,
    pub orientation: TileOrientation,
    pub tilesets: Vec<String>,
    pub layers: Vec<TileLayerDesc>,
}

impl Default for TilemapDesc {
    fn default() -> Self {
        Self {
            width: 16,
            height: 16,
            tile_width: 16,
            tile_height: 16,
            unit: 1.0_f32,
            orientation: Default::default(),
            tilesets: Default::default(),
            layers: Default::default(),
        }
    }
}

impl FromJson for TilemapDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            tile_width: field(o, "tileWidth", d.tile_width)?,
            tile_height: field(o, "tileHeight", d.tile_height)?,
            unit: field(o, "unit", d.unit)?,
            orientation: field(o, "orientation", d.orientation)?,
            tilesets: field(o, "tilesets", d.tilesets)?,
            layers: field(o, "layers", d.layers)?,
        })
    }
}

impl ToJson for TilemapDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("tileWidth".into(), self.tile_width.to_json());
        o.insert("tileHeight".into(), self.tile_height.to_json());
        o.insert("unit".into(), self.unit.to_json());
        o.insert("orientation".into(), self.orientation.to_json());
        o.insert("tilesets".into(), self.tilesets.to_json());
        o.insert("layers".into(), self.layers.to_json());
        Value::Object(o)
    }
}

/// `ResizeDesc`: packages/knowledge/wiki/tilemap.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ResizeDesc {
    pub map: u64,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
}

impl Default for ResizeDesc {
    fn default() -> Self {
        Self {
            map: Default::default(),
            width: 16,
            height: 16,
            x: Default::default(),
            y: Default::default(),
        }
    }
}

impl FromJson for ResizeDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            map: field(o, "map", d.map)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
        })
    }
}

impl ToJson for ResizeDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("map".into(), self.map.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        Value::Object(o)
    }
}

/// `SetTileDesc`: packages/knowledge/wiki/tilemap.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SetTileDesc {
    pub entity: u64,
    pub layer: String,
    pub x: u32,
    pub y: u32,
    pub tile: i32,
}

impl Default for SetTileDesc {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            layer: Default::default(),
            x: Default::default(),
            y: Default::default(),
            tile: -1,
        }
    }
}

impl FromJson for SetTileDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            layer: field(o, "layer", d.layer)?,
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            tile: field(o, "tile", d.tile)?,
        })
    }
}

impl ToJson for SetTileDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("layer".into(), self.layer.to_json());
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("tile".into(), self.tile.to_json());
        Value::Object(o)
    }
}

/// `TileCell`: packages/knowledge/wiki/tilemap.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TileCell {
    pub x: u32,
    pub y: u32,
    pub inside: bool,
}

impl Default for TileCell {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            inside: Default::default(),
        }
    }
}

impl FromJson for TileCell {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            inside: field(o, "inside", d.inside)?,
        })
    }
}

impl ToJson for TileCell {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("inside".into(), self.inside.to_json());
        Value::Object(o)
    }
}

/// `FillDesc`: packages/knowledge/wiki/tilemap.md.
#[derive(Clone, Debug, PartialEq)]
pub struct FillDesc {
    pub entity: u64,
    pub layer: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub tiles: Vec<i32>,
}

impl Default for FillDesc {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            layer: Default::default(),
            x: Default::default(),
            y: Default::default(),
            width: Default::default(),
            tiles: Default::default(),
        }
    }
}

impl FromJson for FillDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            layer: field(o, "layer", d.layer)?,
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            width: field(o, "width", d.width)?,
            tiles: field(o, "tiles", d.tiles)?,
        })
    }
}

impl ToJson for FillDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("layer".into(), self.layer.to_json());
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("tiles".into(), self.tiles.to_json());
        Value::Object(o)
    }
}

/// `SoundDesc`: packages/knowledge/wiki/audio.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SoundDesc {
    pub samples: Vec<f32>,
    pub rate: u32,
    pub channels: u32,
}

impl Default for SoundDesc {
    fn default() -> Self {
        Self {
            samples: Default::default(),
            rate: 48000,
            channels: 1,
        }
    }
}

impl FromJson for SoundDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            samples: field(o, "samples", d.samples)?,
            rate: field(o, "rate", d.rate)?,
            channels: field(o, "channels", d.channels)?,
        })
    }
}

impl ToJson for SoundDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("samples".into(), self.samples.to_json());
        o.insert("rate".into(), self.rate.to_json());
        o.insert("channels".into(), self.channels.to_json());
        Value::Object(o)
    }
}

/// `AudioFeedbackDesc`: packages/knowledge/wiki/audio.md.
#[derive(Clone, Debug, PartialEq)]
pub struct AudioFeedbackDesc {
    pub volume: f32,
    pub playback_rate: f32,
}

impl Default for AudioFeedbackDesc {
    fn default() -> Self {
        Self {
            volume: 1.0_f32,
            playback_rate: 1.0_f32,
        }
    }
}

impl FromJson for AudioFeedbackDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            volume: field(o, "volume", d.volume)?,
            playback_rate: field(o, "playbackRate", d.playback_rate)?,
        })
    }
}

impl ToJson for AudioFeedbackDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("volume".into(), self.volume.to_json());
        o.insert("playbackRate".into(), self.playback_rate.to_json());
        Value::Object(o)
    }
}

/// `NavAlgorithm`: packages/knowledge/wiki/nav.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NavAlgorithm {
    #[default]
    Jps,
    Astar,
}

impl NavAlgorithm {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["jps", "astar"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Jps, Self::Astar];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("NavAlgorithm", s, Self::NAMES)),
        }
    }
}

impl FromJson for NavAlgorithm {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "NavAlgorithm", Self::NAMES)?)
    }
}

impl ToJson for NavAlgorithm {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `NavLinkKind`: packages/knowledge/wiki/nav.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NavLinkKind {
    #[default]
    Jump,
    Ladder,
    Door,
    Teleport,
}

impl NavLinkKind {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["jump", "ladder", "door", "teleport"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Jump, Self::Ladder, Self::Door, Self::Teleport];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("NavLinkKind", s, Self::NAMES)),
        }
    }
}

impl FromJson for NavLinkKind {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "NavLinkKind", Self::NAMES)?)
    }
}

impl ToJson for NavLinkKind {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `NavLinkDesc`: packages/knowledge/wiki/nav.md.
#[derive(Clone, Debug, PartialEq)]
pub struct NavLinkDesc {
    pub start: Vec3,
    pub end: Vec3,
    pub kind: NavLinkKind,
    pub bidirectional: bool,
    pub cost: f32,
    pub open: bool,
    pub duration: f32,
    pub game: bool,
}

impl Default for NavLinkDesc {
    fn default() -> Self {
        Self {
            start: Default::default(),
            end: Default::default(),
            kind: Default::default(),
            bidirectional: true,
            cost: Default::default(),
            open: true,
            duration: Default::default(),
            game: Default::default(),
        }
    }
}

impl FromJson for NavLinkDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            start: field(o, "start", d.start)?,
            end: field(o, "end", d.end)?,
            kind: field(o, "kind", d.kind)?,
            bidirectional: field(o, "bidirectional", d.bidirectional)?,
            cost: field(o, "cost", d.cost)?,
            open: field(o, "open", d.open)?,
            duration: field(o, "duration", d.duration)?,
            game: field(o, "game", d.game)?,
        })
    }
}

impl ToJson for NavLinkDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("start".into(), self.start.to_json());
        o.insert("end".into(), self.end.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("bidirectional".into(), self.bidirectional.to_json());
        o.insert("cost".into(), self.cost.to_json());
        o.insert("open".into(), self.open.to_json());
        o.insert("duration".into(), self.duration.to_json());
        o.insert("game".into(), self.game.to_json());
        Value::Object(o)
    }
}

/// `NavShapeKind`: packages/knowledge/wiki/nav.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NavShapeKind {
    #[default]
    Box,
    OrientedBox,
    Cylinder,
    Convex,
}

impl NavShapeKind {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["box", "orientedBox", "cylinder", "convex"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Box, Self::OrientedBox, Self::Cylinder, Self::Convex];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("NavShapeKind", s, Self::NAMES)),
        }
    }
}

impl FromJson for NavShapeKind {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "NavShapeKind", Self::NAMES)?)
    }
}

impl ToJson for NavShapeKind {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `NavShape`: packages/knowledge/wiki/nav.md.
#[derive(Clone, Debug, PartialEq)]
pub struct NavShape {
    pub kind: NavShapeKind,
    pub min: Vec3,
    pub max: Vec3,
    pub center: Vec3,
    pub size: Vec3,
    pub yaw: f32,
    pub radius: f32,
    pub height: f32,
    pub points: Vec<Vec3>,
}

impl Default for NavShape {
    fn default() -> Self {
        Self {
            kind: Default::default(),
            min: Default::default(),
            max: Default::default(),
            center: Default::default(),
            size: Default::default(),
            yaw: Default::default(),
            radius: Default::default(),
            height: Default::default(),
            points: Default::default(),
        }
    }
}

impl FromJson for NavShape {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            kind: field(o, "kind", d.kind)?,
            min: field(o, "min", d.min)?,
            max: field(o, "max", d.max)?,
            center: field(o, "center", d.center)?,
            size: field(o, "size", d.size)?,
            yaw: field(o, "yaw", d.yaw)?,
            radius: field(o, "radius", d.radius)?,
            height: field(o, "height", d.height)?,
            points: field(o, "points", d.points)?,
        })
    }
}

impl ToJson for NavShape {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("kind".into(), self.kind.to_json());
        o.insert("min".into(), self.min.to_json());
        o.insert("max".into(), self.max.to_json());
        o.insert("center".into(), self.center.to_json());
        o.insert("size".into(), self.size.to_json());
        o.insert("yaw".into(), self.yaw.to_json());
        o.insert("radius".into(), self.radius.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("points".into(), self.points.to_json());
        Value::Object(o)
    }
}

/// `NavAreaDesc`: packages/knowledge/wiki/nav.md.
#[derive(Clone, Debug, PartialEq)]
pub struct NavAreaDesc {
    pub shape: NavShape,
    pub area: u32,
}

impl Default for NavAreaDesc {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            area: Default::default(),
        }
    }
}

impl FromJson for NavAreaDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            shape: field(o, "shape", d.shape)?,
            area: field(o, "area", d.area)?,
        })
    }
}

impl ToJson for NavAreaDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("shape".into(), self.shape.to_json());
        o.insert("area".into(), self.area.to_json());
        Value::Object(o)
    }
}

/// `NavGridDesc`: packages/knowledge/wiki/nav.md.
#[derive(Clone, Debug, PartialEq)]
pub struct NavGridDesc {
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
    pub origin: Vec2,
    pub costs: Vec<u32>,
}

impl Default for NavGridDesc {
    fn default() -> Self {
        Self {
            width: 32,
            height: 32,
            cell_size: 1.0_f32,
            origin: Default::default(),
            costs: Default::default(),
        }
    }
}

impl FromJson for NavGridDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            cell_size: field(o, "cellSize", d.cell_size)?,
            origin: field(o, "origin", d.origin)?,
            costs: field(o, "costs", d.costs)?,
        })
    }
}

impl ToJson for NavGridDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("cellSize".into(), self.cell_size.to_json());
        o.insert("origin".into(), self.origin.to_json());
        o.insert("costs".into(), self.costs.to_json());
        Value::Object(o)
    }
}

/// `TilemapGridDesc`: packages/knowledge/wiki/nav.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TilemapGridDesc {
    pub entity: u64,
    pub layers: Vec<String>,
    pub inflate: u32,
}

impl Default for TilemapGridDesc {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            layers: Default::default(),
            inflate: Default::default(),
        }
    }
}

impl FromJson for TilemapGridDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            layers: field(o, "layers", d.layers)?,
            inflate: field(o, "inflate", d.inflate)?,
        })
    }
}

impl ToJson for TilemapGridDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("layers".into(), self.layers.to_json());
        o.insert("inflate".into(), self.inflate.to_json());
        Value::Object(o)
    }
}

/// `NavCell`: packages/knowledge/wiki/nav.md.
#[derive(Clone, Debug, PartialEq)]
pub struct NavCell {
    pub x: u32,
    pub y: u32,
    pub inside: bool,
}

impl Default for NavCell {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            inside: Default::default(),
        }
    }
}

impl FromJson for NavCell {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            inside: field(o, "inside", d.inside)?,
        })
    }
}

impl ToJson for NavCell {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("inside".into(), self.inside.to_json());
        Value::Object(o)
    }
}

/// `PathDesc`: packages/knowledge/wiki/nav.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PathDesc {
    pub nav: u64,
    pub from: Vec3,
    pub to: Vec3,
    pub algorithm: NavAlgorithm,
    pub smooth: bool,
    pub radius: f32,
    pub area_costs: Vec<f32>,
    pub excluded: Vec<u32>,
}

impl Default for PathDesc {
    fn default() -> Self {
        Self {
            nav: Default::default(),
            from: Default::default(),
            to: Default::default(),
            algorithm: Default::default(),
            smooth: true,
            radius: Default::default(),
            area_costs: Default::default(),
            excluded: Default::default(),
        }
    }
}

impl FromJson for PathDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            nav: field(o, "nav", d.nav)?,
            from: field(o, "from", d.from)?,
            to: field(o, "to", d.to)?,
            algorithm: field(o, "algorithm", d.algorithm)?,
            smooth: field(o, "smooth", d.smooth)?,
            radius: field(o, "radius", d.radius)?,
            area_costs: field(o, "areaCosts", d.area_costs)?,
            excluded: field(o, "excluded", d.excluded)?,
        })
    }
}

impl ToJson for PathDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("nav".into(), self.nav.to_json());
        o.insert("from".into(), self.from.to_json());
        o.insert("to".into(), self.to.to_json());
        o.insert("algorithm".into(), self.algorithm.to_json());
        o.insert("smooth".into(), self.smooth.to_json());
        o.insert("radius".into(), self.radius.to_json());
        o.insert("areaCosts".into(), self.area_costs.to_json());
        o.insert("excluded".into(), self.excluded.to_json());
        Value::Object(o)
    }
}

/// `NavPath`: packages/knowledge/wiki/nav.md.
#[derive(Clone, Debug, PartialEq)]
pub struct NavPath {
    pub points: Vec<Vec3>,
    pub length: f32,
    pub complete: bool,
    pub expanded: u32,
    pub algorithm: String,
    pub links: Vec<u32>,
}

impl Default for NavPath {
    fn default() -> Self {
        Self {
            points: Default::default(),
            length: Default::default(),
            complete: Default::default(),
            expanded: Default::default(),
            algorithm: Default::default(),
            links: Default::default(),
        }
    }
}

impl FromJson for NavPath {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            points: field(o, "points", d.points)?,
            length: field(o, "length", d.length)?,
            complete: field(o, "complete", d.complete)?,
            expanded: field(o, "expanded", d.expanded)?,
            algorithm: field(o, "algorithm", d.algorithm)?,
            links: field(o, "links", d.links)?,
        })
    }
}

impl ToJson for NavPath {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("points".into(), self.points.to_json());
        o.insert("length".into(), self.length.to_json());
        o.insert("complete".into(), self.complete.to_json());
        o.insert("expanded".into(), self.expanded.to_json());
        o.insert("algorithm".into(), self.algorithm.to_json());
        o.insert("links".into(), self.links.to_json());
        Value::Object(o)
    }
}

/// `FlowDesc`: packages/knowledge/wiki/nav.md.
#[derive(Clone, Debug, PartialEq)]
pub struct FlowDesc {
    pub grid: u64,
    pub targets: Vec<Vec3>,
}

impl Default for FlowDesc {
    fn default() -> Self {
        Self {
            grid: Default::default(),
            targets: Default::default(),
        }
    }
}

impl FromJson for FlowDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            grid: field(o, "grid", d.grid)?,
            targets: field(o, "targets", d.targets)?,
        })
    }
}

impl ToJson for FlowDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("grid".into(), self.grid.to_json());
        o.insert("targets".into(), self.targets.to_json());
        Value::Object(o)
    }
}

/// `NavBox`: packages/knowledge/wiki/nav.md.
#[derive(Clone, Debug, PartialEq)]
pub struct NavBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl Default for NavBox {
    fn default() -> Self {
        Self {
            min: Default::default(),
            max: Default::default(),
        }
    }
}

impl FromJson for NavBox {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            min: field(o, "min", d.min)?,
            max: field(o, "max", d.max)?,
        })
    }
}

impl ToJson for NavBox {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("min".into(), self.min.to_json());
        o.insert("max".into(), self.max.to_json());
        Value::Object(o)
    }
}

/// `NavMeshDesc`: packages/knowledge/wiki/nav.md.
#[derive(Clone, Debug, PartialEq)]
pub struct NavMeshDesc {
    pub min: Vec3,
    pub max: Vec3,
    pub blocked: Vec<NavBox>,
    pub cell_size: f32,
    pub agent_radius: f32,
    pub agent_height: f32,
    pub tile_size: u32,
}

impl Default for NavMeshDesc {
    fn default() -> Self {
        Self {
            min: Default::default(),
            max: Vec3::new(16.0_f32, 1.0_f32, 16.0_f32),
            blocked: Default::default(),
            cell_size: 0.25_f32,
            agent_radius: 0.4_f32,
            agent_height: 1.8_f32,
            tile_size: 32,
        }
    }
}

impl FromJson for NavMeshDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            min: field(o, "min", d.min)?,
            max: field(o, "max", d.max)?,
            blocked: field(o, "blocked", d.blocked)?,
            cell_size: field(o, "cellSize", d.cell_size)?,
            agent_radius: field(o, "agentRadius", d.agent_radius)?,
            agent_height: field(o, "agentHeight", d.agent_height)?,
            tile_size: field(o, "tileSize", d.tile_size)?,
        })
    }
}

impl ToJson for NavMeshDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("min".into(), self.min.to_json());
        o.insert("max".into(), self.max.to_json());
        o.insert("blocked".into(), self.blocked.to_json());
        o.insert("cellSize".into(), self.cell_size.to_json());
        o.insert("agentRadius".into(), self.agent_radius.to_json());
        o.insert("agentHeight".into(), self.agent_height.to_json());
        o.insert("tileSize".into(), self.tile_size.to_json());
        Value::Object(o)
    }
}

/// `NavMeshInfo`: packages/knowledge/wiki/nav.md.
#[derive(Clone, Debug, PartialEq)]
pub struct NavMeshInfo {
    pub mesh: u64,
    pub polygons: u32,
    pub tiles: u32,
    pub min: Vec3,
    pub max: Vec3,
    pub cell_size: f32,
    pub profile: String,
}

impl Default for NavMeshInfo {
    fn default() -> Self {
        Self {
            mesh: Default::default(),
            polygons: Default::default(),
            tiles: Default::default(),
            min: Default::default(),
            max: Default::default(),
            cell_size: Default::default(),
            profile: Default::default(),
        }
    }
}

impl FromJson for NavMeshInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            mesh: field(o, "mesh", d.mesh)?,
            polygons: field(o, "polygons", d.polygons)?,
            tiles: field(o, "tiles", d.tiles)?,
            min: field(o, "min", d.min)?,
            max: field(o, "max", d.max)?,
            cell_size: field(o, "cellSize", d.cell_size)?,
            profile: field(o, "profile", d.profile)?,
        })
    }
}

impl ToJson for NavMeshInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("mesh".into(), self.mesh.to_json());
        o.insert("polygons".into(), self.polygons.to_json());
        o.insert("tiles".into(), self.tiles.to_json());
        o.insert("min".into(), self.min.to_json());
        o.insert("max".into(), self.max.to_json());
        o.insert("cellSize".into(), self.cell_size.to_json());
        o.insert("profile".into(), self.profile.to_json());
        Value::Object(o)
    }
}

/// `NavRebuild`: packages/knowledge/wiki/nav.md.
#[derive(Clone, Debug, PartialEq)]
pub struct NavRebuild {
    pub tiles: Vec<u32>,
    pub polygons: u32,
}

impl Default for NavRebuild {
    fn default() -> Self {
        Self {
            tiles: Default::default(),
            polygons: Default::default(),
        }
    }
}

impl FromJson for NavRebuild {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tiles: field(o, "tiles", d.tiles)?,
            polygons: field(o, "polygons", d.polygons)?,
        })
    }
}

impl ToJson for NavRebuild {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tiles".into(), self.tiles.to_json());
        o.insert("polygons".into(), self.polygons.to_json());
        Value::Object(o)
    }
}

/// `BehaviourInfo`: packages/knowledge/wiki/ai.md.
#[derive(Clone, Debug, PartialEq)]
pub struct BehaviourInfo {
    pub behaviour: u64,
    pub name: String,
    pub kind: String,
    pub states: Vec<String>,
    pub nodes: Vec<String>,
    pub keys: Vec<String>,
    pub subtrees: Vec<String>,
}

impl Default for BehaviourInfo {
    fn default() -> Self {
        Self {
            behaviour: Default::default(),
            name: Default::default(),
            kind: Default::default(),
            states: Default::default(),
            nodes: Default::default(),
            keys: Default::default(),
            subtrees: Default::default(),
        }
    }
}

impl FromJson for BehaviourInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            behaviour: field(o, "behaviour", d.behaviour)?,
            name: field(o, "name", d.name)?,
            kind: field(o, "kind", d.kind)?,
            states: field(o, "states", d.states)?,
            nodes: field(o, "nodes", d.nodes)?,
            keys: field(o, "keys", d.keys)?,
            subtrees: field(o, "subtrees", d.subtrees)?,
        })
    }
}

impl ToJson for BehaviourInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("behaviour".into(), self.behaviour.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("states".into(), self.states.to_json());
        o.insert("nodes".into(), self.nodes.to_json());
        o.insert("keys".into(), self.keys.to_json());
        o.insert("subtrees".into(), self.subtrees.to_json());
        Value::Object(o)
    }
}

/// `NoiseDesc`: packages/knowledge/wiki/ai.md.
#[derive(Clone, Debug, PartialEq)]
pub struct NoiseDesc {
    pub position: Vec3,
    pub loudness: f32,
    pub source: u64,
}

impl Default for NoiseDesc {
    fn default() -> Self {
        Self {
            position: Default::default(),
            loudness: 1.0_f32,
            source: Default::default(),
        }
    }
}

impl FromJson for NoiseDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            position: field(o, "position", d.position)?,
            loudness: field(o, "loudness", d.loudness)?,
            source: field(o, "source", d.source)?,
        })
    }
}

impl ToJson for NoiseDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("position".into(), self.position.to_json());
        o.insert("loudness".into(), self.loudness.to_json());
        o.insert("source".into(), self.source.to_json());
        Value::Object(o)
    }
}

/// `AiDescription`: packages/knowledge/wiki/ai.md.
#[derive(Clone, Debug, PartialEq)]
pub struct AiDescription {
    pub behaviour: String,
    pub kind: String,
    pub state: String,
    pub node: String,
    pub path: String,
    pub status: String,
    pub blackboard: Value,
    pub sees: bool,
    pub target: u64,
}

impl Default for AiDescription {
    fn default() -> Self {
        Self {
            behaviour: Default::default(),
            kind: Default::default(),
            state: Default::default(),
            node: Default::default(),
            path: Default::default(),
            status: Default::default(),
            blackboard: Default::default(),
            sees: Default::default(),
            target: Default::default(),
        }
    }
}

impl FromJson for AiDescription {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            behaviour: field(o, "behaviour", d.behaviour)?,
            kind: field(o, "kind", d.kind)?,
            state: field(o, "state", d.state)?,
            node: field(o, "node", d.node)?,
            path: field(o, "path", d.path)?,
            status: field(o, "status", d.status)?,
            blackboard: field(o, "blackboard", d.blackboard)?,
            sees: field(o, "sees", d.sees)?,
            target: field(o, "target", d.target)?,
        })
    }
}

impl ToJson for AiDescription {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("behaviour".into(), self.behaviour.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("state".into(), self.state.to_json());
        o.insert("node".into(), self.node.to_json());
        o.insert("path".into(), self.path.to_json());
        o.insert("status".into(), self.status.to_json());
        o.insert("blackboard".into(), self.blackboard.to_json());
        o.insert("sees".into(), self.sees.to_json());
        o.insert("target".into(), self.target.to_json());
        Value::Object(o)
    }
}

/// `StorySaliency`: packages/knowledge/wiki/story.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StorySaliency {
    #[default]
    First,
    Best,
    BestLeastRecentlyViewed,
    RandomBestLeastRecentlyViewed,
    ConditionalFirstWeighted,
}

impl StorySaliency {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["first", "best", "bestLeastRecentlyViewed", "randomBestLeastRecentlyViewed", "conditionalFirstWeighted"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::First, Self::Best, Self::BestLeastRecentlyViewed, Self::RandomBestLeastRecentlyViewed, Self::ConditionalFirstWeighted];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("StorySaliency", s, Self::NAMES)),
        }
    }
}

impl FromJson for StorySaliency {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "StorySaliency", Self::NAMES)?)
    }
}

impl ToJson for StorySaliency {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `StoryAsset`: packages/knowledge/wiki/story.md.
#[derive(Clone, Debug, PartialEq)]
pub struct StoryAsset {
    pub kind: String,
    pub name: String,
    pub nodes: Vec<String>,
    pub speakers: Vec<String>,
    pub variables: Vec<String>,
}

impl Default for StoryAsset {
    fn default() -> Self {
        Self {
            kind: Default::default(),
            name: Default::default(),
            nodes: Default::default(),
            speakers: Default::default(),
            variables: Default::default(),
        }
    }
}

impl FromJson for StoryAsset {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            kind: field(o, "kind", d.kind)?,
            name: field(o, "name", d.name)?,
            nodes: field(o, "nodes", d.nodes)?,
            speakers: field(o, "speakers", d.speakers)?,
            variables: field(o, "variables", d.variables)?,
        })
    }
}

impl ToJson for StoryAsset {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("kind".into(), self.kind.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("nodes".into(), self.nodes.to_json());
        o.insert("speakers".into(), self.speakers.to_json());
        o.insert("variables".into(), self.variables.to_json());
        Value::Object(o)
    }
}

/// `StartDesc`: packages/knowledge/wiki/story.md.
#[derive(Clone, Debug, PartialEq)]
pub struct StartDesc {
    pub dialogue: String,
    pub node: String,
    pub cast: Value,
    pub vars: Value,
}

impl Default for StartDesc {
    fn default() -> Self {
        Self {
            dialogue: Default::default(),
            node: Default::default(),
            cast: Default::default(),
            vars: Default::default(),
        }
    }
}

impl FromJson for StartDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            dialogue: field(o, "dialogue", d.dialogue)?,
            node: field(o, "node", d.node)?,
            cast: field(o, "cast", d.cast)?,
            vars: field(o, "vars", d.vars)?,
        })
    }
}

impl ToJson for StartDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("dialogue".into(), self.dialogue.to_json());
        o.insert("node".into(), self.node.to_json());
        o.insert("cast".into(), self.cast.to_json());
        o.insert("vars".into(), self.vars.to_json());
        Value::Object(o)
    }
}

/// `SpeakerView`: packages/knowledge/wiki/story.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SpeakerView {
    pub id: String,
    pub name: String,
    pub entity: u64,
    pub portrait: String,
    pub expression: String,
    pub side: String,
}

impl Default for SpeakerView {
    fn default() -> Self {
        Self {
            id: Default::default(),
            name: Default::default(),
            entity: Default::default(),
            portrait: Default::default(),
            expression: Default::default(),
            side: Default::default(),
        }
    }
}

impl FromJson for SpeakerView {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            id: field(o, "id", d.id)?,
            name: field(o, "name", d.name)?,
            entity: field(o, "entity", d.entity)?,
            portrait: field(o, "portrait", d.portrait)?,
            expression: field(o, "expression", d.expression)?,
            side: field(o, "side", d.side)?,
        })
    }
}

impl ToJson for SpeakerView {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("id".into(), self.id.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("entity".into(), self.entity.to_json());
        o.insert("portrait".into(), self.portrait.to_json());
        o.insert("expression".into(), self.expression.to_json());
        o.insert("side".into(), self.side.to_json());
        Value::Object(o)
    }
}

/// `CheckView`: packages/knowledge/wiki/story.md.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckView {
    pub stat: String,
    pub target: f32,
    pub chance: f32,
    pub kind: String,
}

impl Default for CheckView {
    fn default() -> Self {
        Self {
            stat: Default::default(),
            target: Default::default(),
            chance: Default::default(),
            kind: Default::default(),
        }
    }
}

impl FromJson for CheckView {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            stat: field(o, "stat", d.stat)?,
            target: field(o, "target", d.target)?,
            chance: field(o, "chance", d.chance)?,
            kind: field(o, "kind", d.kind)?,
        })
    }
}

impl ToJson for CheckView {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("stat".into(), self.stat.to_json());
        o.insert("target".into(), self.target.to_json());
        o.insert("chance".into(), self.chance.to_json());
        o.insert("kind".into(), self.kind.to_json());
        Value::Object(o)
    }
}

/// `ChoiceView`: packages/knowledge/wiki/story.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ChoiceView {
    pub index: u32,
    pub id: String,
    pub text: String,
    pub spans: Vec<TextSpan>,
    pub key: String,
    pub args: Value,
    pub enabled: bool,
    pub reason: String,
    pub reason_key: String,
    pub reason_args: Value,
    pub visited: bool,
    pub kind: String,
    pub check: Option<CheckView>,
}

impl Default for ChoiceView {
    fn default() -> Self {
        Self {
            index: Default::default(),
            id: Default::default(),
            text: Default::default(),
            spans: Default::default(),
            key: Default::default(),
            args: Default::default(),
            enabled: Default::default(),
            reason: Default::default(),
            reason_key: Default::default(),
            reason_args: Default::default(),
            visited: Default::default(),
            kind: Default::default(),
            check: Default::default(),
        }
    }
}

impl FromJson for ChoiceView {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            index: field(o, "index", d.index)?,
            id: field(o, "id", d.id)?,
            text: field(o, "text", d.text)?,
            spans: field(o, "spans", d.spans)?,
            key: field(o, "key", d.key)?,
            args: field(o, "args", d.args)?,
            enabled: field(o, "enabled", d.enabled)?,
            reason: field(o, "reason", d.reason)?,
            reason_key: field(o, "reasonKey", d.reason_key)?,
            reason_args: field(o, "reasonArgs", d.reason_args)?,
            visited: field(o, "visited", d.visited)?,
            kind: field(o, "kind", d.kind)?,
            check: field(o, "check", d.check)?,
        })
    }
}

impl ToJson for ChoiceView {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("index".into(), self.index.to_json());
        o.insert("id".into(), self.id.to_json());
        o.insert("text".into(), self.text.to_json());
        o.insert("spans".into(), self.spans.to_json());
        o.insert("key".into(), self.key.to_json());
        o.insert("args".into(), self.args.to_json());
        o.insert("enabled".into(), self.enabled.to_json());
        o.insert("reason".into(), self.reason.to_json());
        o.insert("reasonKey".into(), self.reason_key.to_json());
        o.insert("reasonArgs".into(), self.reason_args.to_json());
        o.insert("visited".into(), self.visited.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("check".into(), self.check.to_json());
        Value::Object(o)
    }
}

/// `TimerView`: packages/knowledge/wiki/story.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TimerView {
    pub ticks_left: u32,
    pub ticks_total: u32,
    pub default_choice: u32,
}

impl Default for TimerView {
    fn default() -> Self {
        Self {
            ticks_left: Default::default(),
            ticks_total: Default::default(),
            default_choice: Default::default(),
        }
    }
}

impl FromJson for TimerView {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            ticks_left: field(o, "ticksLeft", d.ticks_left)?,
            ticks_total: field(o, "ticksTotal", d.ticks_total)?,
            default_choice: field(o, "defaultChoice", d.default_choice)?,
        })
    }
}

impl ToJson for TimerView {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("ticksLeft".into(), self.ticks_left.to_json());
        o.insert("ticksTotal".into(), self.ticks_total.to_json());
        o.insert("defaultChoice".into(), self.default_choice.to_json());
        Value::Object(o)
    }
}

/// `RollView`: packages/knowledge/wiki/story.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RollView {
    pub stat: String,
    pub target: f32,
    pub roll: u32,
    pub passed: bool,
}

impl Default for RollView {
    fn default() -> Self {
        Self {
            stat: Default::default(),
            target: Default::default(),
            roll: Default::default(),
            passed: Default::default(),
        }
    }
}

impl FromJson for RollView {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            stat: field(o, "stat", d.stat)?,
            target: field(o, "target", d.target)?,
            roll: field(o, "roll", d.roll)?,
            passed: field(o, "passed", d.passed)?,
        })
    }
}

impl ToJson for RollView {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("stat".into(), self.stat.to_json());
        o.insert("target".into(), self.target.to_json());
        o.insert("roll".into(), self.roll.to_json());
        o.insert("passed".into(), self.passed.to_json());
        Value::Object(o)
    }
}

/// `DialogueView`: packages/knowledge/wiki/story.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DialogueView {
    pub conversation: u32,
    pub node: String,
    pub seq: u32,
    pub seen: bool,
    pub key: String,
    pub args: Value,
    pub text: String,
    pub spans: Vec<TextSpan>,
    pub voice: String,
    pub voice_ticks: u32,
    pub voice_left: u32,
    pub speaker: SpeakerView,
    pub shot: String,
    pub choices: Vec<ChoiceView>,
    pub timer: Option<TimerView>,
    pub last_check: Option<RollView>,
    pub ended: bool,
}

impl Default for DialogueView {
    fn default() -> Self {
        Self {
            conversation: Default::default(),
            node: Default::default(),
            seq: Default::default(),
            seen: Default::default(),
            key: Default::default(),
            args: Default::default(),
            text: Default::default(),
            spans: Default::default(),
            voice: Default::default(),
            voice_ticks: Default::default(),
            voice_left: Default::default(),
            speaker: Default::default(),
            shot: Default::default(),
            choices: Default::default(),
            timer: Default::default(),
            last_check: Default::default(),
            ended: Default::default(),
        }
    }
}

impl FromJson for DialogueView {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            conversation: field(o, "conversation", d.conversation)?,
            node: field(o, "node", d.node)?,
            seq: field(o, "seq", d.seq)?,
            seen: field(o, "seen", d.seen)?,
            key: field(o, "key", d.key)?,
            args: field(o, "args", d.args)?,
            text: field(o, "text", d.text)?,
            spans: field(o, "spans", d.spans)?,
            voice: field(o, "voice", d.voice)?,
            voice_ticks: field(o, "voiceTicks", d.voice_ticks)?,
            voice_left: field(o, "voiceLeft", d.voice_left)?,
            speaker: field(o, "speaker", d.speaker)?,
            shot: field(o, "shot", d.shot)?,
            choices: field(o, "choices", d.choices)?,
            timer: field(o, "timer", d.timer)?,
            last_check: field(o, "lastCheck", d.last_check)?,
            ended: field(o, "ended", d.ended)?,
        })
    }
}

impl ToJson for DialogueView {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("conversation".into(), self.conversation.to_json());
        o.insert("node".into(), self.node.to_json());
        o.insert("seq".into(), self.seq.to_json());
        o.insert("seen".into(), self.seen.to_json());
        o.insert("key".into(), self.key.to_json());
        o.insert("args".into(), self.args.to_json());
        o.insert("text".into(), self.text.to_json());
        o.insert("spans".into(), self.spans.to_json());
        o.insert("voice".into(), self.voice.to_json());
        o.insert("voiceTicks".into(), self.voice_ticks.to_json());
        o.insert("voiceLeft".into(), self.voice_left.to_json());
        o.insert("speaker".into(), self.speaker.to_json());
        o.insert("shot".into(), self.shot.to_json());
        o.insert("choices".into(), self.choices.to_json());
        o.insert("timer".into(), self.timer.to_json());
        o.insert("lastCheck".into(), self.last_check.to_json());
        o.insert("ended".into(), self.ended.to_json());
        Value::Object(o)
    }
}

/// `ChosenView`: packages/knowledge/wiki/story.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ChosenView {
    pub index: u32,
    pub text: String,
    pub spans: Vec<TextSpan>,
}

impl Default for ChosenView {
    fn default() -> Self {
        Self {
            index: Default::default(),
            text: Default::default(),
            spans: Default::default(),
        }
    }
}

impl FromJson for ChosenView {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            index: field(o, "index", d.index)?,
            text: field(o, "text", d.text)?,
            spans: field(o, "spans", d.spans)?,
        })
    }
}

impl ToJson for ChosenView {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("index".into(), self.index.to_json());
        o.insert("text".into(), self.text.to_json());
        o.insert("spans".into(), self.spans.to_json());
        Value::Object(o)
    }
}

/// `StoryLine`: packages/knowledge/wiki/story.md.
#[derive(Clone, Debug, PartialEq)]
pub struct StoryLine {
    pub seq: u32,
    pub node: String,
    pub speaker: SpeakerView,
    pub key: String,
    pub args: Value,
    pub text: String,
    pub spans: Vec<TextSpan>,
    pub voice: String,
    pub seen: bool,
    pub chosen: Option<ChosenView>,
}

impl Default for StoryLine {
    fn default() -> Self {
        Self {
            seq: Default::default(),
            node: Default::default(),
            speaker: Default::default(),
            key: Default::default(),
            args: Default::default(),
            text: Default::default(),
            spans: Default::default(),
            voice: Default::default(),
            seen: Default::default(),
            chosen: Default::default(),
        }
    }
}

impl FromJson for StoryLine {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            seq: field(o, "seq", d.seq)?,
            node: field(o, "node", d.node)?,
            speaker: field(o, "speaker", d.speaker)?,
            key: field(o, "key", d.key)?,
            args: field(o, "args", d.args)?,
            text: field(o, "text", d.text)?,
            spans: field(o, "spans", d.spans)?,
            voice: field(o, "voice", d.voice)?,
            seen: field(o, "seen", d.seen)?,
            chosen: field(o, "chosen", d.chosen)?,
        })
    }
}

impl ToJson for StoryLine {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("seq".into(), self.seq.to_json());
        o.insert("node".into(), self.node.to_json());
        o.insert("speaker".into(), self.speaker.to_json());
        o.insert("key".into(), self.key.to_json());
        o.insert("args".into(), self.args.to_json());
        o.insert("text".into(), self.text.to_json());
        o.insert("spans".into(), self.spans.to_json());
        o.insert("voice".into(), self.voice.to_json());
        o.insert("seen".into(), self.seen.to_json());
        o.insert("chosen".into(), self.chosen.to_json());
        Value::Object(o)
    }
}

/// `BarkDesc`: packages/knowledge/wiki/story.md.
#[derive(Clone, Debug, PartialEq)]
pub struct BarkDesc {
    pub entity: u64,
    pub topic: String,
    pub dialogue: String,
}

impl Default for BarkDesc {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            topic: Default::default(),
            dialogue: Default::default(),
        }
    }
}

impl FromJson for BarkDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            topic: field(o, "topic", d.topic)?,
            dialogue: field(o, "dialogue", d.dialogue)?,
        })
    }
}

impl ToJson for BarkDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("topic".into(), self.topic.to_json());
        o.insert("dialogue".into(), self.dialogue.to_json());
        Value::Object(o)
    }
}

/// `AmbientDesc`: packages/knowledge/wiki/story.md.
#[derive(Clone, Debug, PartialEq)]
pub struct AmbientDesc {
    pub entity: u64,
    pub topic: String,
    pub dialogue: String,
    pub every: u32,
    pub jitter: u32,
}

impl Default for AmbientDesc {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            topic: Default::default(),
            dialogue: Default::default(),
            every: 600,
            jitter: Default::default(),
        }
    }
}

impl FromJson for AmbientDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            topic: field(o, "topic", d.topic)?,
            dialogue: field(o, "dialogue", d.dialogue)?,
            every: field(o, "every", d.every)?,
            jitter: field(o, "jitter", d.jitter)?,
        })
    }
}

impl ToJson for AmbientDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("topic".into(), self.topic.to_json());
        o.insert("dialogue".into(), self.dialogue.to_json());
        o.insert("every".into(), self.every.to_json());
        o.insert("jitter".into(), self.jitter.to_json());
        Value::Object(o)
    }
}

/// `BarkView`: packages/knowledge/wiki/story.md.
#[derive(Clone, Debug, PartialEq)]
pub struct BarkView {
    pub entity: u64,
    pub dialogue: String,
    pub bark: String,
    pub seq: u32,
    pub key: String,
    pub args: Value,
    pub text: String,
    pub spans: Vec<TextSpan>,
    pub speaker: SpeakerView,
    pub voice: String,
    pub voice_left: u32,
    pub ticks_left: u32,
    pub priority: i32,
}

impl Default for BarkView {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            dialogue: Default::default(),
            bark: Default::default(),
            seq: Default::default(),
            key: Default::default(),
            args: Default::default(),
            text: Default::default(),
            spans: Default::default(),
            speaker: Default::default(),
            voice: Default::default(),
            voice_left: Default::default(),
            ticks_left: Default::default(),
            priority: Default::default(),
        }
    }
}

impl FromJson for BarkView {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            dialogue: field(o, "dialogue", d.dialogue)?,
            bark: field(o, "bark", d.bark)?,
            seq: field(o, "seq", d.seq)?,
            key: field(o, "key", d.key)?,
            args: field(o, "args", d.args)?,
            text: field(o, "text", d.text)?,
            spans: field(o, "spans", d.spans)?,
            speaker: field(o, "speaker", d.speaker)?,
            voice: field(o, "voice", d.voice)?,
            voice_left: field(o, "voiceLeft", d.voice_left)?,
            ticks_left: field(o, "ticksLeft", d.ticks_left)?,
            priority: field(o, "priority", d.priority)?,
        })
    }
}

impl ToJson for BarkView {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("dialogue".into(), self.dialogue.to_json());
        o.insert("bark".into(), self.bark.to_json());
        o.insert("seq".into(), self.seq.to_json());
        o.insert("key".into(), self.key.to_json());
        o.insert("args".into(), self.args.to_json());
        o.insert("text".into(), self.text.to_json());
        o.insert("spans".into(), self.spans.to_json());
        o.insert("speaker".into(), self.speaker.to_json());
        o.insert("voice".into(), self.voice.to_json());
        o.insert("voiceLeft".into(), self.voice_left.to_json());
        o.insert("ticksLeft".into(), self.ticks_left.to_json());
        o.insert("priority".into(), self.priority.to_json());
        Value::Object(o)
    }
}

/// `MarkerDesc`: packages/knowledge/wiki/story.md.
#[derive(Clone, Debug, PartialEq)]
pub struct MarkerDesc {
    pub entity: u64,
    pub position: Vec3,
}

impl Default for MarkerDesc {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            position: Default::default(),
        }
    }
}

impl FromJson for MarkerDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            position: field(o, "position", d.position)?,
        })
    }
}

impl ToJson for MarkerDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("position".into(), self.position.to_json());
        Value::Object(o)
    }
}

/// `MarkerView`: packages/knowledge/wiki/story.md.
#[derive(Clone, Debug, PartialEq)]
pub struct MarkerView {
    pub entity: u64,
    pub position: Vec3,
}

impl Default for MarkerView {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            position: Default::default(),
        }
    }
}

impl FromJson for MarkerView {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            position: field(o, "position", d.position)?,
        })
    }
}

impl ToJson for MarkerView {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("position".into(), self.position.to_json());
        Value::Object(o)
    }
}

/// `RewardView`: packages/knowledge/wiki/story.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RewardView {
    pub name: String,
    pub amount: f64,
    pub image: String,
    pub rarity: String,
}

impl Default for RewardView {
    fn default() -> Self {
        Self {
            name: Default::default(),
            amount: Default::default(),
            image: Default::default(),
            rarity: Default::default(),
        }
    }
}

impl FromJson for RewardView {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            amount: field(o, "amount", d.amount)?,
            image: field(o, "image", d.image)?,
            rarity: field(o, "rarity", d.rarity)?,
        })
    }
}

impl ToJson for RewardView {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("amount".into(), self.amount.to_json());
        o.insert("image".into(), self.image.to_json());
        o.insert("rarity".into(), self.rarity.to_json());
        Value::Object(o)
    }
}

/// `ObjectiveView`: packages/knowledge/wiki/story.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectiveView {
    pub id: String,
    pub text: String,
    pub count: u32,
    pub target: u32,
    pub done: bool,
    pub failed: bool,
    pub optional: bool,
    pub hidden: bool,
}

impl Default for ObjectiveView {
    fn default() -> Self {
        Self {
            id: Default::default(),
            text: Default::default(),
            count: Default::default(),
            target: Default::default(),
            done: Default::default(),
            failed: Default::default(),
            optional: Default::default(),
            hidden: Default::default(),
        }
    }
}

impl FromJson for ObjectiveView {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            id: field(o, "id", d.id)?,
            text: field(o, "text", d.text)?,
            count: field(o, "count", d.count)?,
            target: field(o, "target", d.target)?,
            done: field(o, "done", d.done)?,
            failed: field(o, "failed", d.failed)?,
            optional: field(o, "optional", d.optional)?,
            hidden: field(o, "hidden", d.hidden)?,
        })
    }
}

impl ToJson for ObjectiveView {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("id".into(), self.id.to_json());
        o.insert("text".into(), self.text.to_json());
        o.insert("count".into(), self.count.to_json());
        o.insert("target".into(), self.target.to_json());
        o.insert("done".into(), self.done.to_json());
        o.insert("failed".into(), self.failed.to_json());
        o.insert("optional".into(), self.optional.to_json());
        o.insert("hidden".into(), self.hidden.to_json());
        Value::Object(o)
    }
}

/// `QuestView`: packages/knowledge/wiki/story.md.
#[derive(Clone, Debug, PartialEq)]
pub struct QuestView {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub category: String,
    pub stage: String,
    pub status: String,
    pub tracked: bool,
    pub is_new: bool,
    pub updated: bool,
    pub updated_tick: u32,
    pub giver: String,
    pub region: String,
    pub marker: Option<MarkerView>,
    pub rewards: Vec<RewardView>,
    pub objectives: Vec<ObjectiveView>,
}

impl Default for QuestView {
    fn default() -> Self {
        Self {
            id: Default::default(),
            title: Default::default(),
            summary: Default::default(),
            category: Default::default(),
            stage: Default::default(),
            status: Default::default(),
            tracked: Default::default(),
            is_new: Default::default(),
            updated: Default::default(),
            updated_tick: Default::default(),
            giver: Default::default(),
            region: Default::default(),
            marker: Default::default(),
            rewards: Default::default(),
            objectives: Default::default(),
        }
    }
}

impl FromJson for QuestView {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            id: field(o, "id", d.id)?,
            title: field(o, "title", d.title)?,
            summary: field(o, "summary", d.summary)?,
            category: field(o, "category", d.category)?,
            stage: field(o, "stage", d.stage)?,
            status: field(o, "status", d.status)?,
            tracked: field(o, "tracked", d.tracked)?,
            is_new: field(o, "isNew", d.is_new)?,
            updated: field(o, "updated", d.updated)?,
            updated_tick: field(o, "updatedTick", d.updated_tick)?,
            giver: field(o, "giver", d.giver)?,
            region: field(o, "region", d.region)?,
            marker: field(o, "marker", d.marker)?,
            rewards: field(o, "rewards", d.rewards)?,
            objectives: field(o, "objectives", d.objectives)?,
        })
    }
}

impl ToJson for QuestView {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("id".into(), self.id.to_json());
        o.insert("title".into(), self.title.to_json());
        o.insert("summary".into(), self.summary.to_json());
        o.insert("category".into(), self.category.to_json());
        o.insert("stage".into(), self.stage.to_json());
        o.insert("status".into(), self.status.to_json());
        o.insert("tracked".into(), self.tracked.to_json());
        o.insert("isNew".into(), self.is_new.to_json());
        o.insert("updated".into(), self.updated.to_json());
        o.insert("updatedTick".into(), self.updated_tick.to_json());
        o.insert("giver".into(), self.giver.to_json());
        o.insert("region".into(), self.region.to_json());
        o.insert("marker".into(), self.marker.to_json());
        o.insert("rewards".into(), self.rewards.to_json());
        o.insert("objectives".into(), self.objectives.to_json());
        Value::Object(o)
    }
}

/// `PlatformOrder`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PlatformOrder {
    #[default]
    Desc,
    Asc,
}

impl PlatformOrder {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["desc", "asc"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Desc, Self::Asc];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("PlatformOrder", s, Self::NAMES)),
        }
    }
}

impl FromJson for PlatformOrder {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "PlatformOrder", Self::NAMES)?)
    }
}

impl ToJson for PlatformOrder {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `StoreSetDesc`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct StoreSetDesc {
    pub key: String,
    pub value: Value,
    pub revision: u32,
    pub create: bool,
}

impl Default for StoreSetDesc {
    fn default() -> Self {
        Self {
            key: Default::default(),
            value: Default::default(),
            revision: Default::default(),
            create: Default::default(),
        }
    }
}

impl FromJson for StoreSetDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            key: field(o, "key", d.key)?,
            value: field(o, "value", d.value)?,
            revision: field(o, "revision", d.revision)?,
            create: field(o, "create", d.create)?,
        })
    }
}

impl ToJson for StoreSetDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("key".into(), self.key.to_json());
        o.insert("value".into(), self.value.to_json());
        o.insert("revision".into(), self.revision.to_json());
        o.insert("create".into(), self.create.to_json());
        Value::Object(o)
    }
}

/// `LeaderboardSubmitDesc`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct LeaderboardSubmitDesc {
    pub board: String,
    pub score: Option<f64>,
    pub order: PlatformOrder,
}

impl Default for LeaderboardSubmitDesc {
    fn default() -> Self {
        Self {
            board: String::from("main"),
            score: Default::default(),
            order: Default::default(),
        }
    }
}

impl FromJson for LeaderboardSubmitDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            board: field(o, "board", d.board)?,
            score: field(o, "score", d.score)?,
            order: field(o, "order", d.order)?,
        })
    }
}

impl ToJson for LeaderboardSubmitDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("board".into(), self.board.to_json());
        o.insert("score".into(), self.score.to_json());
        o.insert("order".into(), self.order.to_json());
        Value::Object(o)
    }
}

/// `LeaderboardQueryDesc`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct LeaderboardQueryDesc {
    pub board: String,
    pub count: u32,
}

impl Default for LeaderboardQueryDesc {
    fn default() -> Self {
        Self {
            board: String::from("main"),
            count: 10,
        }
    }
}

impl FromJson for LeaderboardQueryDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            board: field(o, "board", d.board)?,
            count: field(o, "count", d.count)?,
        })
    }
}

impl ToJson for LeaderboardQueryDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("board".into(), self.board.to_json());
        o.insert("count".into(), self.count.to_json());
        Value::Object(o)
    }
}

/// `PointDesc`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PointDesc {
    pub reason: String,
    pub amount: u32,
}

impl Default for PointDesc {
    fn default() -> Self {
        Self {
            reason: Default::default(),
            amount: 1,
        }
    }
}

impl FromJson for PointDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            reason: field(o, "reason", d.reason)?,
            amount: field(o, "amount", d.amount)?,
        })
    }
}

impl ToJson for PointDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("reason".into(), self.reason.to_json());
        o.insert("amount".into(), self.amount.to_json());
        Value::Object(o)
    }
}

/// `AiMarkDesc`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct AiMarkDesc {
    pub kind: String,
    pub label: String,
    pub model: String,
}

impl Default for AiMarkDesc {
    fn default() -> Self {
        Self {
            kind: Default::default(),
            label: Default::default(),
            model: Default::default(),
        }
    }
}

impl FromJson for AiMarkDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            kind: field(o, "kind", d.kind)?,
            label: field(o, "label", d.label)?,
            model: field(o, "model", d.model)?,
        })
    }
}

impl ToJson for AiMarkDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("kind".into(), self.kind.to_json());
        o.insert("label".into(), self.label.to_json());
        o.insert("model".into(), self.model.to_json());
        Value::Object(o)
    }
}

/// `SaveSlotDesc`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SaveSlotDesc {
    pub slot: String,
    pub text: String,
    pub meta: Value,
}

impl Default for SaveSlotDesc {
    fn default() -> Self {
        Self {
            slot: Default::default(),
            text: Default::default(),
            meta: Default::default(),
        }
    }
}

impl FromJson for SaveSlotDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            slot: field(o, "slot", d.slot)?,
            text: field(o, "text", d.text)?,
            meta: field(o, "meta", d.meta)?,
        })
    }
}

impl ToJson for SaveSlotDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("slot".into(), self.slot.to_json());
        o.insert("text".into(), self.text.to_json());
        o.insert("meta".into(), self.meta.to_json());
        Value::Object(o)
    }
}

/// `LoadSlotDesc`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct LoadSlotDesc {
    pub slot: String,
    pub meta_only: bool,
}

impl Default for LoadSlotDesc {
    fn default() -> Self {
        Self {
            slot: Default::default(),
            meta_only: Default::default(),
        }
    }
}

impl FromJson for LoadSlotDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            slot: field(o, "slot", d.slot)?,
            meta_only: field(o, "metaOnly", d.meta_only)?,
        })
    }
}

impl ToJson for LoadSlotDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("slot".into(), self.slot.to_json());
        o.insert("metaOnly".into(), self.meta_only.to_json());
        Value::Object(o)
    }
}

/// `PlatformError`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PlatformError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
    pub revision: u32,
}

impl Default for PlatformError {
    fn default() -> Self {
        Self {
            code: Default::default(),
            message: Default::default(),
            retryable: Default::default(),
            revision: Default::default(),
        }
    }
}

impl FromJson for PlatformError {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            code: field(o, "code", d.code)?,
            message: field(o, "message", d.message)?,
            retryable: field(o, "retryable", d.retryable)?,
            revision: field(o, "revision", d.revision)?,
        })
    }
}

impl ToJson for PlatformError {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("code".into(), self.code.to_json());
        o.insert("message".into(), self.message.to_json());
        o.insert("retryable".into(), self.retryable.to_json());
        o.insert("revision".into(), self.revision.to_json());
        Value::Object(o)
    }
}

/// `PlatformResult`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PlatformResult {
    pub request: u32,
    pub method: String,
    pub ok: bool,
    pub data: Value,
    pub error: Option<PlatformError>,
    pub tick: u32,
    pub time: f64,
}

impl Default for PlatformResult {
    fn default() -> Self {
        Self {
            request: Default::default(),
            method: Default::default(),
            ok: Default::default(),
            data: Default::default(),
            error: Default::default(),
            tick: Default::default(),
            time: Default::default(),
        }
    }
}

impl FromJson for PlatformResult {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            request: field(o, "request", d.request)?,
            method: field(o, "method", d.method)?,
            ok: field(o, "ok", d.ok)?,
            data: field(o, "data", d.data)?,
            error: field(o, "error", d.error)?,
            tick: field(o, "tick", d.tick)?,
            time: field(o, "time", d.time)?,
        })
    }
}

impl ToJson for PlatformResult {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("request".into(), self.request.to_json());
        o.insert("method".into(), self.method.to_json());
        o.insert("ok".into(), self.ok.to_json());
        o.insert("data".into(), self.data.to_json());
        if let Some(x) = &self.error {
            o.insert("error".into(), x.to_json());
        }
        o.insert("tick".into(), self.tick.to_json());
        o.insert("time".into(), self.time.to_json());
        Value::Object(o)
    }
}

/// `PlatformInfo`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PlatformInfo {
    pub host: String,
    pub capabilities: Vec<String>,
    pub pending: u32,
    pub clock: String,
}

impl Default for PlatformInfo {
    fn default() -> Self {
        Self {
            host: Default::default(),
            capabilities: Default::default(),
            pending: Default::default(),
            clock: Default::default(),
        }
    }
}

impl FromJson for PlatformInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            host: field(o, "host", d.host)?,
            capabilities: field(o, "capabilities", d.capabilities)?,
            pending: field(o, "pending", d.pending)?,
            clock: field(o, "clock", d.clock)?,
        })
    }
}

impl ToJson for PlatformInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("host".into(), self.host.to_json());
        o.insert("capabilities".into(), self.capabilities.to_json());
        o.insert("pending".into(), self.pending.to_json());
        o.insert("clock".into(), self.clock.to_json());
        Value::Object(o)
    }
}

/// `AdKind`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AdKind {
    #[default]
    Rewarded,
    Interstitial,
}

impl AdKind {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["rewarded", "interstitial"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Rewarded, Self::Interstitial];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("AdKind", s, Self::NAMES)),
        }
    }
}

impl FromJson for AdKind {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "AdKind", Self::NAMES)?)
    }
}

impl ToJson for AdKind {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `PlatformNoticeData`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PlatformNoticeData {
    pub notice: u32,
    pub kind: String,
    pub data: Value,
    pub tick: u32,
    pub time: f64,
}

impl Default for PlatformNoticeData {
    fn default() -> Self {
        Self {
            notice: Default::default(),
            kind: Default::default(),
            data: Default::default(),
            tick: Default::default(),
            time: Default::default(),
        }
    }
}

impl FromJson for PlatformNoticeData {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            notice: field(o, "notice", d.notice)?,
            kind: field(o, "kind", d.kind)?,
            data: field(o, "data", d.data)?,
            tick: field(o, "tick", d.tick)?,
            time: field(o, "time", d.time)?,
        })
    }
}

impl ToJson for PlatformNoticeData {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("notice".into(), self.notice.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("data".into(), self.data.to_json());
        o.insert("tick".into(), self.tick.to_json());
        o.insert("time".into(), self.time.to_json());
        Value::Object(o)
    }
}

/// `ProductsDesc`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductsDesc {
    pub ids: Vec<String>,
}

impl Default for ProductsDesc {
    fn default() -> Self {
        Self {
            ids: Default::default(),
        }
    }
}

impl FromJson for ProductsDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            ids: field(o, "ids", d.ids)?,
        })
    }
}

impl ToJson for ProductsDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("ids".into(), self.ids.to_json());
        Value::Object(o)
    }
}

/// `PurchaseDesc`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PurchaseDesc {
    pub product: String,
    pub quantity: u32,
}

impl Default for PurchaseDesc {
    fn default() -> Self {
        Self {
            product: Default::default(),
            quantity: 1,
        }
    }
}

impl FromJson for PurchaseDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            product: field(o, "product", d.product)?,
            quantity: field(o, "quantity", d.quantity)?,
        })
    }
}

impl ToJson for PurchaseDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("product".into(), self.product.to_json());
        o.insert("quantity".into(), self.quantity.to_json());
        Value::Object(o)
    }
}

/// `AdDesc`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct AdDesc {
    pub kind: AdKind,
    pub placement: String,
}

impl Default for AdDesc {
    fn default() -> Self {
        Self {
            kind: Default::default(),
            placement: String::from("default"),
        }
    }
}

impl FromJson for AdDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            kind: field(o, "kind", d.kind)?,
            placement: field(o, "placement", d.placement)?,
        })
    }
}

impl ToJson for AdDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("kind".into(), self.kind.to_json());
        o.insert("placement".into(), self.placement.to_json());
        Value::Object(o)
    }
}

/// `AchievementProgressDesc`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct AchievementProgressDesc {
    pub id: String,
    pub percent: f32,
}

impl Default for AchievementProgressDesc {
    fn default() -> Self {
        Self {
            id: Default::default(),
            percent: Default::default(),
        }
    }
}

impl FromJson for AchievementProgressDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            id: field(o, "id", d.id)?,
            percent: field(o, "percent", d.percent)?,
        })
    }
}

impl ToJson for AchievementProgressDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("id".into(), self.id.to_json());
        o.insert("percent".into(), self.percent.to_json());
        Value::Object(o)
    }
}

/// `FriendsDesc`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct FriendsDesc {
    pub count: u32,
}

impl Default for FriendsDesc {
    fn default() -> Self {
        Self {
            count: 50,
        }
    }
}

impl FromJson for FriendsDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            count: field(o, "count", d.count)?,
        })
    }
}

impl ToJson for FriendsDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("count".into(), self.count.to_json());
        Value::Object(o)
    }
}

/// `InviteDesc`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct InviteDesc {
    pub to: String,
    pub message: String,
    pub payload: Value,
}

impl Default for InviteDesc {
    fn default() -> Self {
        Self {
            to: Default::default(),
            message: Default::default(),
            payload: Default::default(),
        }
    }
}

impl FromJson for InviteDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            to: field(o, "to", d.to)?,
            message: field(o, "message", d.message)?,
            payload: field(o, "payload", d.payload)?,
        })
    }
}

impl ToJson for InviteDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("to".into(), self.to.to_json());
        o.insert("message".into(), self.message.to_json());
        o.insert("payload".into(), self.payload.to_json());
        Value::Object(o)
    }
}

/// `ShareDesc`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ShareDesc {
    pub text: String,
    pub title: String,
    pub path: String,
    pub image: bool,
}

impl Default for ShareDesc {
    fn default() -> Self {
        Self {
            text: Default::default(),
            title: Default::default(),
            path: Default::default(),
            image: Default::default(),
        }
    }
}

impl FromJson for ShareDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            text: field(o, "text", d.text)?,
            title: field(o, "title", d.title)?,
            path: field(o, "path", d.path)?,
            image: field(o, "image", d.image)?,
        })
    }
}

impl ToJson for ShareDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("text".into(), self.text.to_json());
        o.insert("title".into(), self.title.to_json());
        o.insert("path".into(), self.path.to_json());
        o.insert("image".into(), self.image.to_json());
        Value::Object(o)
    }
}

/// `RoomDesc`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RoomDesc {
    pub name: String,
    pub capacity: u32,
}

impl Default for RoomDesc {
    fn default() -> Self {
        Self {
            name: Default::default(),
            capacity: 2,
        }
    }
}

impl FromJson for RoomDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            capacity: field(o, "capacity", d.capacity)?,
        })
    }
}

impl ToJson for RoomDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("capacity".into(), self.capacity.to_json());
        Value::Object(o)
    }
}

/// `QuickPlayDesc`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct QuickPlayDesc {
    pub mode: String,
    pub players: u32,
    pub timeout_ms: u32,
    pub name: String,
}

impl Default for QuickPlayDesc {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            players: 2,
            timeout_ms: 30000,
            name: Default::default(),
        }
    }
}

impl FromJson for QuickPlayDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            mode: field(o, "mode", d.mode)?,
            players: field(o, "players", d.players)?,
            timeout_ms: field(o, "timeoutMs", d.timeout_ms)?,
            name: field(o, "name", d.name)?,
        })
    }
}

impl ToJson for QuickPlayDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("mode".into(), self.mode.to_json());
        o.insert("players".into(), self.players.to_json());
        o.insert("timeoutMs".into(), self.timeout_ms.to_json());
        o.insert("name".into(), self.name.to_json());
        Value::Object(o)
    }
}

/// `RoomPeer`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RoomPeer {
    pub peer: u32,
    pub name: String,
    pub local: bool,
}

impl Default for RoomPeer {
    fn default() -> Self {
        Self {
            peer: Default::default(),
            name: Default::default(),
            local: Default::default(),
        }
    }
}

impl FromJson for RoomPeer {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            peer: field(o, "peer", d.peer)?,
            name: field(o, "name", d.name)?,
            local: field(o, "local", d.local)?,
        })
    }
}

impl ToJson for RoomPeer {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("peer".into(), self.peer.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("local".into(), self.local.to_json());
        Value::Object(o)
    }
}

/// `RoomInfo`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RoomInfo {
    pub state: String,
    pub room: String,
    pub host: String,
    pub peer: u32,
    pub peers: Vec<RoomPeer>,
    pub link: String,
}

impl Default for RoomInfo {
    fn default() -> Self {
        Self {
            state: Default::default(),
            room: Default::default(),
            host: Default::default(),
            peer: Default::default(),
            peers: Default::default(),
            link: Default::default(),
        }
    }
}

impl FromJson for RoomInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            state: field(o, "state", d.state)?,
            room: field(o, "room", d.room)?,
            host: field(o, "host", d.host)?,
            peer: field(o, "peer", d.peer)?,
            peers: field(o, "peers", d.peers)?,
            link: field(o, "link", d.link)?,
        })
    }
}

impl ToJson for RoomInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("state".into(), self.state.to_json());
        o.insert("room".into(), self.room.to_json());
        o.insert("host".into(), self.host.to_json());
        o.insert("peer".into(), self.peer.to_json());
        o.insert("peers".into(), self.peers.to_json());
        o.insert("link".into(), self.link.to_json());
        Value::Object(o)
    }
}

/// `RoomMessageData`: packages/knowledge/wiki/platform.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RoomMessageData {
    pub message: u32,
    pub from: u32,
    pub tick: u32,
    pub data: Value,
    pub bytes: Option<Value>,
}

impl Default for RoomMessageData {
    fn default() -> Self {
        Self {
            message: Default::default(),
            from: Default::default(),
            tick: Default::default(),
            data: Default::default(),
            bytes: Default::default(),
        }
    }
}

impl FromJson for RoomMessageData {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            message: field(o, "message", d.message)?,
            from: field(o, "from", d.from)?,
            tick: field(o, "tick", d.tick)?,
            data: field(o, "data", d.data)?,
            bytes: field(o, "bytes", d.bytes)?,
        })
    }
}

impl ToJson for RoomMessageData {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("message".into(), self.message.to_json());
        o.insert("from".into(), self.from.to_json());
        o.insert("tick".into(), self.tick.to_json());
        o.insert("data".into(), self.data.to_json());
        o.insert("bytes".into(), self.bytes.to_json());
        Value::Object(o)
    }
}

/// `DateStyle`: packages/knowledge/wiki/i18n.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DateStyle {
    #[default]
    Medium,
    Short,
    Long,
    Full,
    MonthYear,
    MonthName,
    Weekday,
    WeekdayShort,
}

impl DateStyle {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["medium", "short", "long", "full", "monthYear", "monthName", "weekday", "weekdayShort"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Medium, Self::Short, Self::Long, Self::Full, Self::MonthYear, Self::MonthName, Self::Weekday, Self::WeekdayShort];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("DateStyle", s, Self::NAMES)),
        }
    }
}

impl FromJson for DateStyle {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "DateStyle", Self::NAMES)?)
    }
}

impl ToJson for DateStyle {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `TimeStyle`: packages/knowledge/wiki/i18n.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TimeStyle {
    #[default]
    Short,
    Medium,
}

impl TimeStyle {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["short", "medium"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Short, Self::Medium];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("TimeStyle", s, Self::NAMES)),
        }
    }
}

impl FromJson for TimeStyle {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "TimeStyle", Self::NAMES)?)
    }
}

impl ToJson for TimeStyle {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `DateFormat`: packages/knowledge/wiki/i18n.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DateFormat {
    pub style: DateStyle,
    pub offset_minutes: i32,
}

impl Default for DateFormat {
    fn default() -> Self {
        Self {
            style: Default::default(),
            offset_minutes: Default::default(),
        }
    }
}

impl FromJson for DateFormat {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            style: field(o, "style", d.style)?,
            offset_minutes: field(o, "offsetMinutes", d.offset_minutes)?,
        })
    }
}

impl ToJson for DateFormat {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("style".into(), self.style.to_json());
        o.insert("offsetMinutes".into(), self.offset_minutes.to_json());
        Value::Object(o)
    }
}

/// `TimeFormat`: packages/knowledge/wiki/i18n.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TimeFormat {
    pub style: TimeStyle,
    pub offset_minutes: i32,
}

impl Default for TimeFormat {
    fn default() -> Self {
        Self {
            style: Default::default(),
            offset_minutes: Default::default(),
        }
    }
}

impl FromJson for TimeFormat {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            style: field(o, "style", d.style)?,
            offset_minutes: field(o, "offsetMinutes", d.offset_minutes)?,
        })
    }
}

impl ToJson for TimeFormat {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("style".into(), self.style.to_json());
        o.insert("offsetMinutes".into(), self.offset_minutes.to_json());
        Value::Object(o)
    }
}

/// `LocaleInfo`: packages/knowledge/wiki/i18n.md.
#[derive(Clone, Debug, PartialEq)]
pub struct LocaleInfo {
    pub requested: String,
    pub active: String,
    pub direction: String,
    pub locales: Vec<String>,
    pub fonts: Vec<String>,
}

impl Default for LocaleInfo {
    fn default() -> Self {
        Self {
            requested: Default::default(),
            active: Default::default(),
            direction: Default::default(),
            locales: Default::default(),
            fonts: Default::default(),
        }
    }
}

impl FromJson for LocaleInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            requested: field(o, "requested", d.requested)?,
            active: field(o, "active", d.active)?,
            direction: field(o, "direction", d.direction)?,
            locales: field(o, "locales", d.locales)?,
            fonts: field(o, "fonts", d.fonts)?,
        })
    }
}

impl ToJson for LocaleInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("requested".into(), self.requested.to_json());
        o.insert("active".into(), self.active.to_json());
        o.insert("direction".into(), self.direction.to_json());
        o.insert("locales".into(), self.locales.to_json());
        o.insert("fonts".into(), self.fonts.to_json());
        Value::Object(o)
    }
}

/// `DecimalFormat`: packages/knowledge/wiki/i18n.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DecimalFormat {
    pub decimals: Option<u32>,
    pub grouping: Option<bool>,
}

impl Default for DecimalFormat {
    fn default() -> Self {
        Self {
            decimals: Default::default(),
            grouping: Default::default(),
        }
    }
}

impl FromJson for DecimalFormat {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            decimals: field(o, "decimals", d.decimals)?,
            grouping: field(o, "grouping", d.grouping)?,
        })
    }
}

impl ToJson for DecimalFormat {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("decimals".into(), self.decimals.to_json());
        o.insert("grouping".into(), self.grouping.to_json());
        Value::Object(o)
    }
}

/// `DurationStyle`: packages/knowledge/wiki/i18n.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DurationStyle {
    #[default]
    Clock,
    Short,
}

impl DurationStyle {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["clock", "short"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Clock, Self::Short];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("DurationStyle", s, Self::NAMES)),
        }
    }
}

impl FromJson for DurationStyle {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "DurationStyle", Self::NAMES)?)
    }
}

impl ToJson for DurationStyle {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `TextAlign`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TextAlign {
    #[default]
    Start,
    Center,
    End,
}

impl TextAlign {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["start", "center", "end"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Start, Self::Center, Self::End];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("TextAlign", s, Self::NAMES)),
        }
    }
}

impl FromJson for TextAlign {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "TextAlign", Self::NAMES)?)
    }
}

impl ToJson for TextAlign {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `Scheme`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Scheme {
    #[default]
    Dark,
    Light,
}

impl Scheme {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["dark", "light"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Dark, Self::Light];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("Scheme", s, Self::NAMES)),
        }
    }
}

impl FromJson for Scheme {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "Scheme", Self::NAMES)?)
    }
}

impl ToJson for Scheme {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `DensityMode`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DensityMode {
    #[default]
    Auto,
    Mouse,
    Pad,
    Touch,
}

impl DensityMode {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["auto", "mouse", "pad", "touch"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Auto, Self::Mouse, Self::Pad, Self::Touch];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("DensityMode", s, Self::NAMES)),
        }
    }
}

impl FromJson for DensityMode {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "DensityMode", Self::NAMES)?)
    }
}

impl ToJson for DensityMode {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `ThemeColor`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ThemeColor {
    #[default]
    Backdrop,
    Well,
    Panel,
    PanelHover,
    PanelRaised,
    Line,
    LineStrong,
    Ink,
    InkMuted,
    InkFaint,
    InkInverse,
    Control,
    ControlStrong,
    Primary,
    PrimaryHover,
    PrimaryActive,
    OnPrimary,
    PrimarySoft,
    Highlight,
    Selection,
    OnSelection,
    Accent,
    SceneInk,
    ScenePlate,
    SceneShadow,
    Focus,
    Knob,
    Hud,
    Scrim,
    Shadow,
    Sheen,
    Success,
    Warn,
    Danger,
    Info,
    Health,
    Mana,
    Xp,
    Gold,
}

impl ThemeColor {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["backdrop", "well", "panel", "panelHover", "panelRaised", "line", "lineStrong", "ink", "inkMuted", "inkFaint", "inkInverse", "control", "controlStrong", "primary", "primaryHover", "primaryActive", "onPrimary", "primarySoft", "highlight", "selection", "onSelection", "accent", "sceneInk", "scenePlate", "sceneShadow", "focus", "knob", "hud", "scrim", "shadow", "sheen", "success", "warn", "danger", "info", "health", "mana", "xp", "gold"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Backdrop, Self::Well, Self::Panel, Self::PanelHover, Self::PanelRaised, Self::Line, Self::LineStrong, Self::Ink, Self::InkMuted, Self::InkFaint, Self::InkInverse, Self::Control, Self::ControlStrong, Self::Primary, Self::PrimaryHover, Self::PrimaryActive, Self::OnPrimary, Self::PrimarySoft, Self::Highlight, Self::Selection, Self::OnSelection, Self::Accent, Self::SceneInk, Self::ScenePlate, Self::SceneShadow, Self::Focus, Self::Knob, Self::Hud, Self::Scrim, Self::Shadow, Self::Sheen, Self::Success, Self::Warn, Self::Danger, Self::Info, Self::Health, Self::Mana, Self::Xp, Self::Gold];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("ThemeColor", s, Self::NAMES)),
        }
    }
}

impl FromJson for ThemeColor {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "ThemeColor", Self::NAMES)?)
    }
}

impl ToJson for ThemeColor {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `Insets`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Insets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Default for Insets {
    fn default() -> Self {
        Self {
            left: Default::default(),
            top: Default::default(),
            right: Default::default(),
            bottom: Default::default(),
        }
    }
}

impl FromJson for Insets {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            left: field(o, "left", d.left)?,
            top: field(o, "top", d.top)?,
            right: field(o, "right", d.right)?,
            bottom: field(o, "bottom", d.bottom)?,
        })
    }
}

impl ToJson for Insets {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("left".into(), self.left.to_json());
        o.insert("top".into(), self.top.to_json());
        o.insert("right".into(), self.right.to_json());
        o.insert("bottom".into(), self.bottom.to_json());
        Value::Object(o)
    }
}

/// `Paint`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Paint {
    pub token: Option<ThemeColor>,
    pub role: Option<String>,
    pub color: Option<Color>,
    pub alpha: f32,
}

impl Default for Paint {
    fn default() -> Self {
        Self {
            token: Default::default(),
            role: Default::default(),
            color: Default::default(),
            alpha: 1.0_f32,
        }
    }
}

impl FromJson for Paint {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            token: field(o, "token", d.token)?,
            role: field(o, "role", d.role)?,
            color: field(o, "color", d.color)?,
            alpha: field(o, "alpha", d.alpha)?,
        })
    }
}

impl ToJson for Paint {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("token".into(), self.token.to_json());
        o.insert("role".into(), self.role.to_json());
        o.insert("color".into(), self.color.to_json());
        o.insert("alpha".into(), self.alpha.to_json());
        Value::Object(o)
    }
}

/// `Viewport`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
    pub dpr: f32,
    pub safe_area: Insets,
    pub small_width: Option<f32>,
    pub small_height: Option<f32>,
    pub large_width: Option<f32>,
    pub large_height: Option<f32>,
    pub coarse_pointer: bool,
    pub reduced_motion: Option<bool>,
    pub high_contrast: Option<bool>,
    pub reduced_transparency: Option<bool>,
    pub mac_keys: Option<bool>,
    pub covered: Option<UiRect>,
    pub breakpoint: Breakpoint,
    pub hud: Option<UiRect>,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: 1280.0_f32,
            height: 720.0_f32,
            dpr: 1.0_f32,
            safe_area: Default::default(),
            small_width: Default::default(),
            small_height: Default::default(),
            large_width: Default::default(),
            large_height: Default::default(),
            coarse_pointer: Default::default(),
            reduced_motion: Default::default(),
            high_contrast: Default::default(),
            reduced_transparency: Default::default(),
            mac_keys: Default::default(),
            covered: Default::default(),
            breakpoint: Default::default(),
            hud: Default::default(),
        }
    }
}

impl FromJson for Viewport {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            dpr: field(o, "dpr", d.dpr)?,
            safe_area: field(o, "safeArea", d.safe_area)?,
            small_width: field(o, "smallWidth", d.small_width)?,
            small_height: field(o, "smallHeight", d.small_height)?,
            large_width: field(o, "largeWidth", d.large_width)?,
            large_height: field(o, "largeHeight", d.large_height)?,
            coarse_pointer: field(o, "coarsePointer", d.coarse_pointer)?,
            reduced_motion: field(o, "reducedMotion", d.reduced_motion)?,
            high_contrast: field(o, "highContrast", d.high_contrast)?,
            reduced_transparency: field(o, "reducedTransparency", d.reduced_transparency)?,
            mac_keys: field(o, "macKeys", d.mac_keys)?,
            covered: field(o, "covered", d.covered)?,
            breakpoint: field(o, "breakpoint", d.breakpoint)?,
            hud: field(o, "hud", d.hud)?,
        })
    }
}

impl ToJson for Viewport {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("dpr".into(), self.dpr.to_json());
        o.insert("safeArea".into(), self.safe_area.to_json());
        if let Some(x) = &self.small_width {
            o.insert("smallWidth".into(), x.to_json());
        }
        if let Some(x) = &self.small_height {
            o.insert("smallHeight".into(), x.to_json());
        }
        if let Some(x) = &self.large_width {
            o.insert("largeWidth".into(), x.to_json());
        }
        if let Some(x) = &self.large_height {
            o.insert("largeHeight".into(), x.to_json());
        }
        o.insert("coarsePointer".into(), self.coarse_pointer.to_json());
        if let Some(x) = &self.reduced_motion {
            o.insert("reducedMotion".into(), x.to_json());
        }
        if let Some(x) = &self.high_contrast {
            o.insert("highContrast".into(), x.to_json());
        }
        if let Some(x) = &self.reduced_transparency {
            o.insert("reducedTransparency".into(), x.to_json());
        }
        if let Some(x) = &self.mac_keys {
            o.insert("macKeys".into(), x.to_json());
        }
        if let Some(x) = &self.covered {
            o.insert("covered".into(), x.to_json());
        }
        o.insert("breakpoint".into(), self.breakpoint.to_json());
        if let Some(x) = &self.hud {
            o.insert("hud".into(), x.to_json());
        }
        Value::Object(o)
    }
}

/// `UiDesc`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiDesc {
    pub scheme: Scheme,
    pub density: DensityMode,
    pub font: Option<u64>,
    pub display_font: Option<u64>,
    pub mono_font: Option<u64>,
    pub rounded_font: Option<u64>,
    pub fallback_fonts: Vec<u64>,
    pub contrast: Contrast,
    pub primary_color: Option<Color>,
    pub reduced_motion: bool,
    pub text_scale: f32,
    pub unit_scale: f32,
    pub reduced_transparency: bool,
    pub color_vision: ColorVision,
    pub color_vision_filter: ColorVisionFilter,
    pub color_vision_strength: f32,
    pub icons: Option<IconAtlasDesc>,
    pub fallback_icons: Option<IconAtlasDesc>,
    pub images: Vec<UiInlineImage>,
    pub toast_anchor: Option<Anchor>,
    pub hud_max_aspect: f32,
    pub style: Option<u64>,
    pub sounds: Option<UiSounds>,
}

impl Default for UiDesc {
    fn default() -> Self {
        Self {
            scheme: Default::default(),
            density: Default::default(),
            font: Default::default(),
            display_font: Default::default(),
            mono_font: Default::default(),
            rounded_font: Default::default(),
            fallback_fonts: Default::default(),
            contrast: Default::default(),
            primary_color: Default::default(),
            reduced_motion: Default::default(),
            text_scale: 1.0_f32,
            unit_scale: 0.0_f32,
            reduced_transparency: Default::default(),
            color_vision: Default::default(),
            color_vision_filter: Default::default(),
            color_vision_strength: 1.0_f32,
            icons: Default::default(),
            fallback_icons: Default::default(),
            images: Default::default(),
            toast_anchor: Default::default(),
            hud_max_aspect: 2.3333333_f32,
            style: Default::default(),
            sounds: Default::default(),
        }
    }
}

impl FromJson for UiDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            scheme: field(o, "scheme", d.scheme)?,
            density: field(o, "density", d.density)?,
            font: field(o, "font", d.font)?,
            display_font: field(o, "displayFont", d.display_font)?,
            mono_font: field(o, "monoFont", d.mono_font)?,
            rounded_font: field(o, "roundedFont", d.rounded_font)?,
            fallback_fonts: field(o, "fallbackFonts", d.fallback_fonts)?,
            contrast: field(o, "contrast", d.contrast)?,
            primary_color: field(o, "primaryColor", d.primary_color)?,
            reduced_motion: field(o, "reducedMotion", d.reduced_motion)?,
            text_scale: field(o, "textScale", d.text_scale)?,
            unit_scale: field(o, "unitScale", d.unit_scale)?,
            reduced_transparency: field(o, "reducedTransparency", d.reduced_transparency)?,
            color_vision: field(o, "colorVision", d.color_vision)?,
            color_vision_filter: field(o, "colorVisionFilter", d.color_vision_filter)?,
            color_vision_strength: field(o, "colorVisionStrength", d.color_vision_strength)?,
            icons: field(o, "icons", d.icons)?,
            fallback_icons: field(o, "fallbackIcons", d.fallback_icons)?,
            images: field(o, "images", d.images)?,
            toast_anchor: field(o, "toastAnchor", d.toast_anchor)?,
            hud_max_aspect: field(o, "hudMaxAspect", d.hud_max_aspect)?,
            style: field(o, "style", d.style)?,
            sounds: field(o, "sounds", d.sounds)?,
        })
    }
}

impl ToJson for UiDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("scheme".into(), self.scheme.to_json());
        o.insert("density".into(), self.density.to_json());
        o.insert("font".into(), self.font.to_json());
        o.insert("displayFont".into(), self.display_font.to_json());
        o.insert("monoFont".into(), self.mono_font.to_json());
        o.insert("roundedFont".into(), self.rounded_font.to_json());
        o.insert("fallbackFonts".into(), self.fallback_fonts.to_json());
        o.insert("contrast".into(), self.contrast.to_json());
        o.insert("primaryColor".into(), self.primary_color.to_json());
        o.insert("reducedMotion".into(), self.reduced_motion.to_json());
        o.insert("textScale".into(), self.text_scale.to_json());
        o.insert("unitScale".into(), self.unit_scale.to_json());
        o.insert("reducedTransparency".into(), self.reduced_transparency.to_json());
        o.insert("colorVision".into(), self.color_vision.to_json());
        o.insert("colorVisionFilter".into(), self.color_vision_filter.to_json());
        o.insert("colorVisionStrength".into(), self.color_vision_strength.to_json());
        o.insert("icons".into(), self.icons.to_json());
        o.insert("fallbackIcons".into(), self.fallback_icons.to_json());
        o.insert("images".into(), self.images.to_json());
        o.insert("toastAnchor".into(), self.toast_anchor.to_json());
        o.insert("hudMaxAspect".into(), self.hud_max_aspect.to_json());
        o.insert("style".into(), self.style.to_json());
        o.insert("sounds".into(), self.sounds.to_json());
        Value::Object(o)
    }
}

/// `UiSounds`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiSounds {
    pub hover: Option<u64>,
    pub press: Option<u64>,
    pub open: Option<u64>,
    pub close: Option<u64>,
    pub success: Option<u64>,
    pub error: Option<u64>,
}

impl Default for UiSounds {
    fn default() -> Self {
        Self {
            hover: Default::default(),
            press: Default::default(),
            open: Default::default(),
            close: Default::default(),
            success: Default::default(),
            error: Default::default(),
        }
    }
}

impl FromJson for UiSounds {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            hover: field(o, "hover", d.hover)?,
            press: field(o, "press", d.press)?,
            open: field(o, "open", d.open)?,
            close: field(o, "close", d.close)?,
            success: field(o, "success", d.success)?,
            error: field(o, "error", d.error)?,
        })
    }
}

impl ToJson for UiSounds {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("hover".into(), self.hover.to_json());
        o.insert("press".into(), self.press.to_json());
        o.insert("open".into(), self.open.to_json());
        o.insert("close".into(), self.close.to_json());
        o.insert("success".into(), self.success.to_json());
        o.insert("error".into(), self.error.to_json());
        Value::Object(o)
    }
}

/// `TypeStep`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TypeStep {
    pub size: f32,
    pub line_height: f32,
    pub tracking_em: f32,
    pub cjk_line_height: f32,
    pub cjk_tracking_em: f32,
    pub weight: u32,
    pub uppercase: bool,
    pub features: Vec<String>,
}

impl Default for TypeStep {
    fn default() -> Self {
        Self {
            size: Default::default(),
            line_height: Default::default(),
            tracking_em: Default::default(),
            cjk_line_height: Default::default(),
            cjk_tracking_em: Default::default(),
            weight: Default::default(),
            uppercase: Default::default(),
            features: Default::default(),
        }
    }
}

impl FromJson for TypeStep {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            size: field(o, "size", d.size)?,
            line_height: field(o, "lineHeight", d.line_height)?,
            tracking_em: field(o, "trackingEm", d.tracking_em)?,
            cjk_line_height: field(o, "cjkLineHeight", d.cjk_line_height)?,
            cjk_tracking_em: field(o, "cjkTrackingEm", d.cjk_tracking_em)?,
            weight: field(o, "weight", d.weight)?,
            uppercase: field(o, "uppercase", d.uppercase)?,
            features: field(o, "features", d.features)?,
        })
    }
}

impl ToJson for TypeStep {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("size".into(), self.size.to_json());
        o.insert("lineHeight".into(), self.line_height.to_json());
        o.insert("trackingEm".into(), self.tracking_em.to_json());
        o.insert("cjkLineHeight".into(), self.cjk_line_height.to_json());
        o.insert("cjkTrackingEm".into(), self.cjk_tracking_em.to_json());
        o.insert("weight".into(), self.weight.to_json());
        o.insert("uppercase".into(), self.uppercase.to_json());
        o.insert("features".into(), self.features.to_json());
        Value::Object(o)
    }
}

/// `ControlSizes`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ControlSizes {
    pub control_sm: f32,
    pub control_md: f32,
    pub control_lg: f32,
    pub gutter: f32,
    pub header: f32,
    pub row: f32,
    pub section: f32,
}

impl Default for ControlSizes {
    fn default() -> Self {
        Self {
            control_sm: Default::default(),
            control_md: Default::default(),
            control_lg: Default::default(),
            gutter: Default::default(),
            header: Default::default(),
            row: Default::default(),
            section: Default::default(),
        }
    }
}

impl FromJson for ControlSizes {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            control_sm: field(o, "controlSm", d.control_sm)?,
            control_md: field(o, "controlMd", d.control_md)?,
            control_lg: field(o, "controlLg", d.control_lg)?,
            gutter: field(o, "gutter", d.gutter)?,
            header: field(o, "header", d.header)?,
            row: field(o, "row", d.row)?,
            section: field(o, "section", d.section)?,
        })
    }
}

impl ToJson for ControlSizes {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("controlSm".into(), self.control_sm.to_json());
        o.insert("controlMd".into(), self.control_md.to_json());
        o.insert("controlLg".into(), self.control_lg.to_json());
        o.insert("gutter".into(), self.gutter.to_json());
        o.insert("header".into(), self.header.to_json());
        o.insert("row".into(), self.row.to_json());
        o.insert("section".into(), self.section.to_json());
        Value::Object(o)
    }
}

/// `UiTheme`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiTheme {
    pub scheme: Scheme,
    pub palette: BTreeMap<String, String>,
    pub space: BTreeMap<String, f32>,
    pub radius: BTreeMap<String, f32>,
    pub type_ramp: BTreeMap<String, TypeStep>,
    pub sizes: ControlSizes,
    pub durations_ms: BTreeMap<String, f32>,
    pub style: String,
    pub style_version: u32,
    pub contrast: Contrast,
    pub reduced_motion: bool,
    pub text_scale: f32,
    pub unit: f32,
    pub color_vision: ColorVision,
    pub color_vision_filter: ColorVisionFilter,
    pub color_vision_strength: f32,
    pub reduced_transparency: bool,
    pub tokens: Value,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            scheme: Default::default(),
            palette: Default::default(),
            space: Default::default(),
            radius: Default::default(),
            type_ramp: Default::default(),
            sizes: Default::default(),
            durations_ms: Default::default(),
            style: Default::default(),
            style_version: Default::default(),
            contrast: Default::default(),
            reduced_motion: Default::default(),
            text_scale: Default::default(),
            unit: Default::default(),
            color_vision: Default::default(),
            color_vision_filter: Default::default(),
            color_vision_strength: Default::default(),
            reduced_transparency: Default::default(),
            tokens: Default::default(),
        }
    }
}

impl FromJson for UiTheme {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            scheme: field(o, "scheme", d.scheme)?,
            palette: field(o, "palette", d.palette)?,
            space: field(o, "space", d.space)?,
            radius: field(o, "radius", d.radius)?,
            type_ramp: field(o, "typeRamp", d.type_ramp)?,
            sizes: field(o, "sizes", d.sizes)?,
            durations_ms: field(o, "durationsMs", d.durations_ms)?,
            style: field(o, "style", d.style)?,
            style_version: field(o, "styleVersion", d.style_version)?,
            contrast: field(o, "contrast", d.contrast)?,
            reduced_motion: field(o, "reducedMotion", d.reduced_motion)?,
            text_scale: field(o, "textScale", d.text_scale)?,
            unit: field(o, "unit", d.unit)?,
            color_vision: field(o, "colorVision", d.color_vision)?,
            color_vision_filter: field(o, "colorVisionFilter", d.color_vision_filter)?,
            color_vision_strength: field(o, "colorVisionStrength", d.color_vision_strength)?,
            reduced_transparency: field(o, "reducedTransparency", d.reduced_transparency)?,
            tokens: field(o, "tokens", d.tokens)?,
        })
    }
}

impl ToJson for UiTheme {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("scheme".into(), self.scheme.to_json());
        o.insert("palette".into(), self.palette.to_json());
        o.insert("space".into(), self.space.to_json());
        o.insert("radius".into(), self.radius.to_json());
        o.insert("typeRamp".into(), self.type_ramp.to_json());
        o.insert("sizes".into(), self.sizes.to_json());
        o.insert("durationsMs".into(), self.durations_ms.to_json());
        o.insert("style".into(), self.style.to_json());
        o.insert("styleVersion".into(), self.style_version.to_json());
        o.insert("contrast".into(), self.contrast.to_json());
        o.insert("reducedMotion".into(), self.reduced_motion.to_json());
        o.insert("textScale".into(), self.text_scale.to_json());
        o.insert("unit".into(), self.unit.to_json());
        o.insert("colorVision".into(), self.color_vision.to_json());
        o.insert("colorVisionFilter".into(), self.color_vision_filter.to_json());
        o.insert("colorVisionStrength".into(), self.color_vision_strength.to_json());
        o.insert("reducedTransparency".into(), self.reduced_transparency.to_json());
        o.insert("tokens".into(), self.tokens.to_json());
        Value::Object(o)
    }
}

/// `TextSpan`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TextSpan {
    pub name: String,
    pub position: u32,
    pub length: u32,
    pub properties: BTreeMap<String, OneOf<String, OneOf<f64, bool>>>,
}

impl Default for TextSpan {
    fn default() -> Self {
        Self {
            name: Default::default(),
            position: Default::default(),
            length: Default::default(),
            properties: Default::default(),
        }
    }
}

impl FromJson for TextSpan {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            position: field(o, "position", d.position)?,
            length: field(o, "length", d.length)?,
            properties: field(o, "properties", d.properties)?,
        })
    }
}

impl ToJson for TextSpan {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("position".into(), self.position.to_json());
        o.insert("length".into(), self.length.to_json());
        o.insert("properties".into(), self.properties.to_json());
        Value::Object(o)
    }
}

/// `Anchor`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Anchor {
    #[default]
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl Anchor {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["topLeft", "top", "topRight", "left", "center", "right", "bottomLeft", "bottom", "bottomRight"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::TopLeft, Self::Top, Self::TopRight, Self::Left, Self::Center, Self::Right, Self::BottomLeft, Self::Bottom, Self::BottomRight];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("Anchor", s, Self::NAMES)),
        }
    }
}

impl FromJson for Anchor {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "Anchor", Self::NAMES)?)
    }
}

impl ToJson for Anchor {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `Tone`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Tone {
    #[default]
    Neutral,
    Success,
    Warn,
    Danger,
    Info,
    Primary,
    Inverse,
}

impl Tone {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["neutral", "success", "warn", "danger", "info", "primary", "inverse"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Neutral, Self::Success, Self::Warn, Self::Danger, Self::Info, Self::Primary, Self::Inverse];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("Tone", s, Self::NAMES)),
        }
    }
}

impl FromJson for Tone {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "Tone", Self::NAMES)?)
    }
}

impl ToJson for Tone {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `Breakpoint`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Breakpoint {
    #[default]
    Desktop,
    Tablet,
    PhoneLandscape,
    PhonePortrait,
}

impl Breakpoint {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["desktop", "tablet", "phoneLandscape", "phonePortrait"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Desktop, Self::Tablet, Self::PhoneLandscape, Self::PhonePortrait];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("Breakpoint", s, Self::NAMES)),
        }
    }
}

impl FromJson for Breakpoint {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "Breakpoint", Self::NAMES)?)
    }
}

impl ToJson for Breakpoint {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `ColorVision`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ColorVision {
    #[default]
    None,
    Protanopia,
    Deuteranopia,
    Tritanopia,
}

impl ColorVision {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["none", "protanopia", "deuteranopia", "tritanopia"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::None, Self::Protanopia, Self::Deuteranopia, Self::Tritanopia];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("ColorVision", s, Self::NAMES)),
        }
    }
}

impl FromJson for ColorVision {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "ColorVision", Self::NAMES)?)
    }
}

impl ToJson for ColorVision {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `ColorVisionFilter`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ColorVisionFilter {
    #[default]
    Correct,
    Simulate,
}

impl ColorVisionFilter {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["correct", "simulate"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Correct, Self::Simulate];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("ColorVisionFilter", s, Self::NAMES)),
        }
    }
}

impl FromJson for ColorVisionFilter {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "ColorVisionFilter", Self::NAMES)?)
    }
}

impl ToJson for ColorVisionFilter {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `Contrast`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Contrast {
    #[default]
    Standard,
    High,
}

impl Contrast {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["standard", "high"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Standard, Self::High];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("Contrast", s, Self::NAMES)),
        }
    }
}

impl FromJson for Contrast {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "Contrast", Self::NAMES)?)
    }
}

impl ToJson for Contrast {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `NavIntent`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NavIntent {
    #[default]
    Up,
    Down,
    Left,
    Right,
    Next,
    Prev,
    Activate,
    Back,
}

impl NavIntent {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["up", "down", "left", "right", "next", "prev", "activate", "back"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Up, Self::Down, Self::Left, Self::Right, Self::Next, Self::Prev, Self::Activate, Self::Back];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("NavIntent", s, Self::NAMES)),
        }
    }
}

impl FromJson for NavIntent {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "NavIntent", Self::NAMES)?)
    }
}

impl ToJson for NavIntent {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `UiRect`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for UiRect {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            width: Default::default(),
            height: Default::default(),
        }
    }
}

impl FromJson for UiRect {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
        })
    }
}

impl ToJson for UiRect {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        Value::Object(o)
    }
}

/// `IconAtlasDesc`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct IconAtlasDesc {
    pub font: Option<u64>,
    pub texture: Option<u64>,
    pub columns: u32,
    pub rows: u32,
    pub names: Vec<String>,
}

impl Default for IconAtlasDesc {
    fn default() -> Self {
        Self {
            font: Default::default(),
            texture: Default::default(),
            columns: 1,
            rows: 1,
            names: Default::default(),
        }
    }
}

impl FromJson for IconAtlasDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            font: field(o, "font", d.font)?,
            texture: field(o, "texture", d.texture)?,
            columns: field(o, "columns", d.columns)?,
            rows: field(o, "rows", d.rows)?,
            names: field(o, "names", d.names)?,
        })
    }
}

impl ToJson for IconAtlasDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("font".into(), self.font.to_json());
        o.insert("texture".into(), self.texture.to_json());
        o.insert("columns".into(), self.columns.to_json());
        o.insert("rows".into(), self.rows.to_json());
        o.insert("names".into(), self.names.to_json());
        Value::Object(o)
    }
}

/// `UiInlineImage`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiInlineImage {
    pub name: String,
    pub texture: Option<u64>,
    pub uv: Option<[f32; 4]>,
    pub aspect: f32,
}

impl Default for UiInlineImage {
    fn default() -> Self {
        Self {
            name: Default::default(),
            texture: Default::default(),
            uv: Default::default(),
            aspect: Default::default(),
        }
    }
}

impl FromJson for UiInlineImage {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            texture: field(o, "texture", d.texture)?,
            uv: field(o, "uv", d.uv)?,
            aspect: field(o, "aspect", d.aspect)?,
        })
    }
}

impl ToJson for UiInlineImage {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("texture".into(), self.texture.to_json());
        o.insert("uv".into(), self.uv.to_json());
        o.insert("aspect".into(), self.aspect.to_json());
        Value::Object(o)
    }
}

/// `ToastDesc`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ToastDesc {
    pub tone: Tone,
    pub duration_ms: f32,
}

impl Default for ToastDesc {
    fn default() -> Self {
        Self {
            tone: Default::default(),
            duration_ms: Default::default(),
        }
    }
}

impl FromJson for ToastDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tone: field(o, "tone", d.tone)?,
            duration_ms: field(o, "durationMs", d.duration_ms)?,
        })
    }
}

impl ToJson for ToastDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tone".into(), self.tone.to_json());
        o.insert("durationMs".into(), self.duration_ms.to_json());
        Value::Object(o)
    }
}

/// `UiPointer`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiPointer {
    pub x: f32,
    pub y: f32,
    pub inside: bool,
    pub down: bool,
    pub pressed: bool,
    pub released: bool,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl Default for UiPointer {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            inside: Default::default(),
            down: Default::default(),
            pressed: Default::default(),
            released: Default::default(),
            shift: Default::default(),
            ctrl: Default::default(),
            alt: Default::default(),
        }
    }
}

impl FromJson for UiPointer {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            inside: field(o, "inside", d.inside)?,
            down: field(o, "down", d.down)?,
            pressed: field(o, "pressed", d.pressed)?,
            released: field(o, "released", d.released)?,
            shift: field(o, "shift", d.shift)?,
            ctrl: field(o, "ctrl", d.ctrl)?,
            alt: field(o, "alt", d.alt)?,
        })
    }
}

impl ToJson for UiPointer {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("inside".into(), self.inside.to_json());
        o.insert("down".into(), self.down.to_json());
        o.insert("pressed".into(), self.pressed.to_json());
        o.insert("released".into(), self.released.to_json());
        o.insert("shift".into(), self.shift.to_json());
        o.insert("ctrl".into(), self.ctrl.to_json());
        o.insert("alt".into(), self.alt.to_json());
        Value::Object(o)
    }
}

/// `ScrollState`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollState {
    pub offset_x: f32,
    pub offset_y: f32,
    pub view_width: f32,
    pub view_height: f32,
    pub content_width: f32,
    pub content_height: f32,
}

impl Default for ScrollState {
    fn default() -> Self {
        Self {
            offset_x: Default::default(),
            offset_y: Default::default(),
            view_width: Default::default(),
            view_height: Default::default(),
            content_width: Default::default(),
            content_height: Default::default(),
        }
    }
}

impl FromJson for ScrollState {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            offset_x: field(o, "offsetX", d.offset_x)?,
            offset_y: field(o, "offsetY", d.offset_y)?,
            view_width: field(o, "viewWidth", d.view_width)?,
            view_height: field(o, "viewHeight", d.view_height)?,
            content_width: field(o, "contentWidth", d.content_width)?,
            content_height: field(o, "contentHeight", d.content_height)?,
        })
    }
}

impl ToJson for ScrollState {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("offsetX".into(), self.offset_x.to_json());
        o.insert("offsetY".into(), self.offset_y.to_json());
        o.insert("viewWidth".into(), self.view_width.to_json());
        o.insert("viewHeight".into(), self.view_height.to_json());
        o.insert("contentWidth".into(), self.content_width.to_json());
        o.insert("contentHeight".into(), self.content_height.to_json());
        Value::Object(o)
    }
}

/// `VirtualCursorDesc`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct VirtualCursorDesc {
    pub speed: f32,
    pub accel: f32,
    pub magnet: f32,
    pub scroll: f32,
}

impl Default for VirtualCursorDesc {
    fn default() -> Self {
        Self {
            speed: Default::default(),
            accel: Default::default(),
            magnet: Default::default(),
            scroll: Default::default(),
        }
    }
}

impl FromJson for VirtualCursorDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            speed: field(o, "speed", d.speed)?,
            accel: field(o, "accel", d.accel)?,
            magnet: field(o, "magnet", d.magnet)?,
            scroll: field(o, "scroll", d.scroll)?,
        })
    }
}

impl ToJson for VirtualCursorDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("speed".into(), self.speed.to_json());
        o.insert("accel".into(), self.accel.to_json());
        o.insert("magnet".into(), self.magnet.to_json());
        o.insert("scroll".into(), self.scroll.to_json());
        Value::Object(o)
    }
}

/// `VirtualCursor`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct VirtualCursor {
    pub x: f32,
    pub y: f32,
    pub shown: bool,
}

impl Default for VirtualCursor {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            shown: Default::default(),
        }
    }
}

impl FromJson for VirtualCursor {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            shown: field(o, "shown", d.shown)?,
        })
    }
}

impl ToJson for VirtualCursor {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("shown".into(), self.shown.to_json());
        Value::Object(o)
    }
}

/// `HudPlacement`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct HudPlacement {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub opacity: f32,
}

impl Default for HudPlacement {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            scale: Default::default(),
            opacity: Default::default(),
        }
    }
}

impl FromJson for HudPlacement {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            scale: field(o, "scale", d.scale)?,
            opacity: field(o, "opacity", d.opacity)?,
        })
    }
}

impl ToJson for HudPlacement {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("scale".into(), self.scale.to_json());
        o.insert("opacity".into(), self.opacity.to_json());
        Value::Object(o)
    }
}

/// `HudEdit`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct HudEdit {
    pub on: bool,
    pub player: u32,
    pub selected: String,
    pub placement: HudPlacement,
    pub keys: Vec<String>,
    pub changed: bool,
}

impl Default for HudEdit {
    fn default() -> Self {
        Self {
            on: Default::default(),
            player: Default::default(),
            selected: Default::default(),
            placement: Default::default(),
            keys: Default::default(),
            changed: Default::default(),
        }
    }
}

impl FromJson for HudEdit {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            on: field(o, "on", d.on)?,
            player: field(o, "player", d.player)?,
            selected: field(o, "selected", d.selected)?,
            placement: field(o, "placement", d.placement)?,
            keys: field(o, "keys", d.keys)?,
            changed: field(o, "changed", d.changed)?,
        })
    }
}

impl ToJson for HudEdit {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("on".into(), self.on.to_json());
        o.insert("player".into(), self.player.to_json());
        o.insert("selected".into(), self.selected.to_json());
        o.insert("placement".into(), self.placement.to_json());
        o.insert("keys".into(), self.keys.to_json());
        o.insert("changed".into(), self.changed.to_json());
        Value::Object(o)
    }
}

/// `A11yNode`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct A11yNode {
    pub path: String,
    pub parent: String,
    pub role: String,
    pub name: String,
    pub description: String,
    pub value: String,
    pub states: Vec<String>,
    pub value_min: Option<f32>,
    pub value_max: Option<f32>,
    pub value_now: Option<f32>,
    pub level: Option<u32>,
    pub live: String,
    pub atomic: bool,
    pub notify: bool,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for A11yNode {
    fn default() -> Self {
        Self {
            path: Default::default(),
            parent: Default::default(),
            role: Default::default(),
            name: Default::default(),
            description: Default::default(),
            value: Default::default(),
            states: Default::default(),
            value_min: Default::default(),
            value_max: Default::default(),
            value_now: Default::default(),
            level: Default::default(),
            live: Default::default(),
            atomic: Default::default(),
            notify: Default::default(),
            x: Default::default(),
            y: Default::default(),
            width: Default::default(),
            height: Default::default(),
        }
    }
}

impl FromJson for A11yNode {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            path: field(o, "path", d.path)?,
            parent: field(o, "parent", d.parent)?,
            role: field(o, "role", d.role)?,
            name: field(o, "name", d.name)?,
            description: field(o, "description", d.description)?,
            value: field(o, "value", d.value)?,
            states: field(o, "states", d.states)?,
            value_min: field(o, "valueMin", d.value_min)?,
            value_max: field(o, "valueMax", d.value_max)?,
            value_now: field(o, "valueNow", d.value_now)?,
            level: field(o, "level", d.level)?,
            live: field(o, "live", d.live)?,
            atomic: field(o, "atomic", d.atomic)?,
            notify: field(o, "notify", d.notify)?,
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
        })
    }
}

impl ToJson for A11yNode {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("path".into(), self.path.to_json());
        o.insert("parent".into(), self.parent.to_json());
        o.insert("role".into(), self.role.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("description".into(), self.description.to_json());
        o.insert("value".into(), self.value.to_json());
        o.insert("states".into(), self.states.to_json());
        o.insert("valueMin".into(), self.value_min.to_json());
        o.insert("valueMax".into(), self.value_max.to_json());
        o.insert("valueNow".into(), self.value_now.to_json());
        o.insert("level".into(), self.level.to_json());
        o.insert("live".into(), self.live.to_json());
        o.insert("atomic".into(), self.atomic.to_json());
        o.insert("notify".into(), self.notify.to_json());
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        Value::Object(o)
    }
}

/// `A11yTree`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct A11yTree {
    pub nodes: Vec<A11yNode>,
    pub focus_order: Vec<String>,
    pub focused: String,
}

impl Default for A11yTree {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            focus_order: Default::default(),
            focused: Default::default(),
        }
    }
}

impl FromJson for A11yTree {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            nodes: field(o, "nodes", d.nodes)?,
            focus_order: field(o, "focusOrder", d.focus_order)?,
            focused: field(o, "focused", d.focused)?,
        })
    }
}

impl ToJson for A11yTree {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("nodes".into(), self.nodes.to_json());
        o.insert("focusOrder".into(), self.focus_order.to_json());
        o.insert("focused".into(), self.focused.to_json());
        Value::Object(o)
    }
}

/// `UiProp`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum UiProp {
    #[default]
    X,
    Y,
    ScaleX,
    ScaleY,
    Scale,
    Rotate,
    SkewX,
    SkewY,
    Opacity,
    Radius,
    BackgroundR,
    BackgroundG,
    BackgroundB,
    BackgroundA,
    InkR,
    InkG,
    InkB,
    InkA,
    Z,
    Inert,
    Blur,
    Draw,
}

impl UiProp {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["x", "y", "scaleX", "scaleY", "scale", "rotate", "skewX", "skewY", "opacity", "radius", "backgroundR", "backgroundG", "backgroundB", "backgroundA", "inkR", "inkG", "inkB", "inkA", "z", "inert", "blur", "draw"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::X, Self::Y, Self::ScaleX, Self::ScaleY, Self::Scale, Self::Rotate, Self::SkewX, Self::SkewY, Self::Opacity, Self::Radius, Self::BackgroundR, Self::BackgroundG, Self::BackgroundB, Self::BackgroundA, Self::InkR, Self::InkG, Self::InkB, Self::InkA, Self::Z, Self::Inert, Self::Blur, Self::Draw];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("UiProp", s, Self::NAMES)),
        }
    }
}

impl FromJson for UiProp {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "UiProp", Self::NAMES)?)
    }
}

impl ToJson for UiProp {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `MotionToken`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MotionToken {
    #[default]
    Ui,
    Panel,
    Emphasis,
    Playful,
    Enter,
    Exit,
}

impl MotionToken {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["ui", "panel", "emphasis", "playful", "enter", "exit"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Ui, Self::Panel, Self::Emphasis, Self::Playful, Self::Enter, Self::Exit];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("MotionToken", s, Self::NAMES)),
        }
    }
}

impl FromJson for MotionToken {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "MotionToken", Self::NAMES)?)
    }
}

impl ToJson for MotionToken {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `Repeat`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Repeat {
    #[default]
    Once,
    Loop,
    PingPong,
}

impl Repeat {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["once", "loop", "pingPong"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Once, Self::Loop, Self::PingPong];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("Repeat", s, Self::NAMES)),
        }
    }
}

impl FromJson for Repeat {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "Repeat", Self::NAMES)?)
    }
}

impl ToJson for Repeat {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `SpringSpec`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SpringSpec {
    pub duration_ms: f32,
    pub bounce: f32,
}

impl Default for SpringSpec {
    fn default() -> Self {
        Self {
            duration_ms: 350.0_f32,
            bounce: Default::default(),
        }
    }
}

impl FromJson for SpringSpec {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            duration_ms: field(o, "durationMs", d.duration_ms)?,
            bounce: field(o, "bounce", d.bounce)?,
        })
    }
}

impl ToJson for SpringSpec {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("durationMs".into(), self.duration_ms.to_json());
        o.insert("bounce".into(), self.bounce.to_json());
        Value::Object(o)
    }
}

/// `TweenSpec`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TweenSpec {
    pub duration_ms: f32,
    pub ease: String,
    pub bezier: Vec<f32>,
}

impl Default for TweenSpec {
    fn default() -> Self {
        Self {
            duration_ms: 200.0_f32,
            ease: Default::default(),
            bezier: Default::default(),
        }
    }
}

impl FromJson for TweenSpec {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            duration_ms: field(o, "durationMs", d.duration_ms)?,
            ease: field(o, "ease", d.ease)?,
            bezier: field(o, "bezier", d.bezier)?,
        })
    }
}

impl ToJson for TweenSpec {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("durationMs".into(), self.duration_ms.to_json());
        o.insert("ease".into(), self.ease.to_json());
        o.insert("bezier".into(), self.bezier.to_json());
        Value::Object(o)
    }
}

/// `TokenRef`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TokenRef {
    pub token: MotionToken,
}

impl Default for TokenRef {
    fn default() -> Self {
        Self {
            token: Default::default(),
        }
    }
}

impl FromJson for TokenRef {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            token: field(o, "token", d.token)?,
        })
    }
}

impl ToJson for TokenRef {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("token".into(), self.token.to_json());
        Value::Object(o)
    }
}

/// `InstantSpec`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct InstantSpec {
}

impl Default for InstantSpec {
    fn default() -> Self {
        Self {}
    }
}

impl FromJson for InstantSpec {
    fn from_json(v: &Value) -> Result<Self, String> {
        object(v)?;
        Ok(Self {})
    }
}

impl ToJson for InstantSpec {
    fn to_json(&self) -> Value {
        Value::Object(Map::new())
    }
}

/// `Transition`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub enum Transition {
    Instant(InstantSpec),
    Spring(SpringSpec),
    Tween(TweenSpec),
    Token(TokenRef),
}

impl Default for Transition {
    fn default() -> Self {
        Self::Instant(Default::default())
    }
}

impl Transition {
    /// The tag key and the cases' tags, in order.
    pub const TAG: &'static str = "kind";
    pub const TAGS: &'static [&'static str] = &["instant", "spring", "tween", "token"];
    /// This case's tag.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Instant(_) => "instant",
            Self::Spring(_) => "spring",
            Self::Tween(_) => "tween",
            Self::Token(_) => "token",
        }
    }
}

impl FromJson for Transition {
    fn from_json(v: &Value) -> Result<Self, String> {
        let o = object(v)?;
        let tag = match o.get(Self::TAG) {
            Some(t) => text(t).map_err(|e| at(Self::TAG, e))?,
            None => return Err(at(Self::TAG, format!("missing: one of {}", Self::TAGS.join(", ")))),
        };
        Ok(match tag {
            "instant" => Self::Instant(InstantSpec::from_json(v)?),
            "spring" => Self::Spring(SpringSpec::from_json(v)?),
            "tween" => Self::Tween(TweenSpec::from_json(v)?),
            "token" => Self::Token(TokenRef::from_json(v)?),
            t => return Err(at(Self::TAG, unknown_variant("Transition", t, Self::TAGS))),
        })
    }
}

impl ToJson for Transition {
    fn to_json(&self) -> Value {
        let mut v = match self {
            Self::Instant(p) => p.to_json(),
            Self::Spring(p) => p.to_json(),
            Self::Tween(p) => p.to_json(),
            Self::Token(p) => p.to_json(),
        };
        if let Value::Object(o) = &mut v {
            o.insert(Self::TAG.into(), Value::String(self.tag().into()));
        }
        v
    }
}

/// `Cursor`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Cursor {
    #[default]
    Auto,
    Default,
    Pointer,
    Text,
    Grab,
    Grabbing,
    Move,
    NotAllowed,
    Crosshair,
    Help,
    Wait,
    Progress,
    ResizeEw,
    ResizeNs,
    ResizeNesw,
    ResizeNwse,
    ZoomIn,
    ZoomOut,
    None,
    ContextMenu,
    Cell,
    VerticalText,
    Alias,
    Copy,
    NoDrop,
    AllScroll,
    ResizeCol,
    ResizeRow,
    ResizeN,
    ResizeE,
    ResizeS,
    ResizeW,
    ResizeNe,
    ResizeNw,
    ResizeSe,
    ResizeSw,
}

impl Cursor {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["auto", "default", "pointer", "text", "grab", "grabbing", "move", "notAllowed", "crosshair", "help", "wait", "progress", "resizeEw", "resizeNs", "resizeNesw", "resizeNwse", "zoomIn", "zoomOut", "none", "contextMenu", "cell", "verticalText", "alias", "copy", "noDrop", "allScroll", "resizeCol", "resizeRow", "resizeN", "resizeE", "resizeS", "resizeW", "resizeNe", "resizeNw", "resizeSe", "resizeSw"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Auto, Self::Default, Self::Pointer, Self::Text, Self::Grab, Self::Grabbing, Self::Move, Self::NotAllowed, Self::Crosshair, Self::Help, Self::Wait, Self::Progress, Self::ResizeEw, Self::ResizeNs, Self::ResizeNesw, Self::ResizeNwse, Self::ZoomIn, Self::ZoomOut, Self::None, Self::ContextMenu, Self::Cell, Self::VerticalText, Self::Alias, Self::Copy, Self::NoDrop, Self::AllScroll, Self::ResizeCol, Self::ResizeRow, Self::ResizeN, Self::ResizeE, Self::ResizeS, Self::ResizeW, Self::ResizeNe, Self::ResizeNw, Self::ResizeSe, Self::ResizeSw];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("Cursor", s, Self::NAMES)),
        }
    }
}

impl FromJson for Cursor {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "Cursor", Self::NAMES)?)
    }
}

impl ToJson for Cursor {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `EaseKey`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct EaseKey {
    pub ease: String,
}

impl Default for EaseKey {
    fn default() -> Self {
        Self {
            ease: Default::default(),
        }
    }
}

impl FromJson for EaseKey {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            ease: field(o, "ease", d.ease)?,
        })
    }
}

impl ToJson for EaseKey {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("ease".into(), self.ease.to_json());
        Value::Object(o)
    }
}

/// `HoldKey`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct HoldKey {
}

impl Default for HoldKey {
    fn default() -> Self {
        Self {}
    }
}

impl FromJson for HoldKey {
    fn from_json(v: &Value) -> Result<Self, String> {
        object(v)?;
        Ok(Self {})
    }
}

impl ToJson for HoldKey {
    fn to_json(&self) -> Value {
        Value::Object(Map::new())
    }
}

/// `LinearKey`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct LinearKey {
}

impl Default for LinearKey {
    fn default() -> Self {
        Self {}
    }
}

impl FromJson for LinearKey {
    fn from_json(v: &Value) -> Result<Self, String> {
        object(v)?;
        Ok(Self {})
    }
}

impl ToJson for LinearKey {
    fn to_json(&self) -> Value {
        Value::Object(Map::new())
    }
}

/// `BezierKey`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct BezierKey {
    pub bezier: Vec<f32>,
}

impl Default for BezierKey {
    fn default() -> Self {
        Self {
            bezier: Default::default(),
        }
    }
}

impl FromJson for BezierKey {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            bezier: field(o, "bezier", d.bezier)?,
        })
    }
}

impl ToJson for BezierKey {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("bezier".into(), self.bezier.to_json());
        Value::Object(o)
    }
}

/// `KeyEase`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub enum KeyEase {
    Linear(LinearKey),
    Ease(EaseKey),
    Bezier(BezierKey),
    Spring(SpringSpec),
    Hold(HoldKey),
}

impl Default for KeyEase {
    fn default() -> Self {
        Self::Linear(Default::default())
    }
}

impl KeyEase {
    /// The tag key and the cases' tags, in order.
    pub const TAG: &'static str = "kind";
    pub const TAGS: &'static [&'static str] = &["linear", "ease", "bezier", "spring", "hold"];
    /// This case's tag.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Linear(_) => "linear",
            Self::Ease(_) => "ease",
            Self::Bezier(_) => "bezier",
            Self::Spring(_) => "spring",
            Self::Hold(_) => "hold",
        }
    }
}

impl FromJson for KeyEase {
    fn from_json(v: &Value) -> Result<Self, String> {
        let o = object(v)?;
        let tag = match o.get(Self::TAG) {
            Some(t) => text(t).map_err(|e| at(Self::TAG, e))?,
            None => return Err(at(Self::TAG, format!("missing: one of {}", Self::TAGS.join(", ")))),
        };
        Ok(match tag {
            "linear" => Self::Linear(LinearKey::from_json(v)?),
            "ease" => Self::Ease(EaseKey::from_json(v)?),
            "bezier" => Self::Bezier(BezierKey::from_json(v)?),
            "spring" => Self::Spring(SpringSpec::from_json(v)?),
            "hold" => Self::Hold(HoldKey::from_json(v)?),
            t => return Err(at(Self::TAG, unknown_variant("KeyEase", t, Self::TAGS))),
        })
    }
}

impl ToJson for KeyEase {
    fn to_json(&self) -> Value {
        let mut v = match self {
            Self::Linear(p) => p.to_json(),
            Self::Ease(p) => p.to_json(),
            Self::Bezier(p) => p.to_json(),
            Self::Spring(p) => p.to_json(),
            Self::Hold(p) => p.to_json(),
        };
        if let Value::Object(o) = &mut v {
            o.insert(Self::TAG.into(), Value::String(self.tag().into()));
        }
        v
    }
}

/// `TimelineKey`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineKey {
    pub at_ms: f32,
    pub value: f32,
    pub ease: Option<KeyEase>,
}

impl Default for TimelineKey {
    fn default() -> Self {
        Self {
            at_ms: Default::default(),
            value: Default::default(),
            ease: Default::default(),
        }
    }
}

impl FromJson for TimelineKey {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            at_ms: field(o, "atMs", d.at_ms)?,
            value: field(o, "value", d.value)?,
            ease: field(o, "ease", d.ease)?,
        })
    }
}

impl ToJson for TimelineKey {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("atMs".into(), self.at_ms.to_json());
        o.insert("value".into(), self.value.to_json());
        o.insert("ease".into(), self.ease.to_json());
        Value::Object(o)
    }
}

/// `TimelineTrack`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineTrack {
    pub target: String,
    pub prop: UiProp,
    pub keys: Vec<TimelineKey>,
}

impl Default for TimelineTrack {
    fn default() -> Self {
        Self {
            target: Default::default(),
            prop: Default::default(),
            keys: Default::default(),
        }
    }
}

impl FromJson for TimelineTrack {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            target: field(o, "target", d.target)?,
            prop: field(o, "prop", d.prop)?,
            keys: field(o, "keys", d.keys)?,
        })
    }
}

impl ToJson for TimelineTrack {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("target".into(), self.target.to_json());
        o.insert("prop".into(), self.prop.to_json());
        o.insert("keys".into(), self.keys.to_json());
        Value::Object(o)
    }
}

/// `SoundEvent`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SoundEvent {
    pub name: String,
    pub playback_rate: f32,
}

impl Default for SoundEvent {
    fn default() -> Self {
        Self {
            name: Default::default(),
            playback_rate: 1.0_f32,
        }
    }
}

impl FromJson for SoundEvent {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            playback_rate: field(o, "playbackRate", d.playback_rate)?,
        })
    }
}

impl ToJson for SoundEvent {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("playbackRate".into(), self.playback_rate.to_json());
        Value::Object(o)
    }
}

/// `HapticEvent`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct HapticEvent {
    pub name: String,
}

impl Default for HapticEvent {
    fn default() -> Self {
        Self {
            name: Default::default(),
        }
    }
}

impl FromJson for HapticEvent {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
        })
    }
}

impl ToJson for HapticEvent {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        Value::Object(o)
    }
}

/// `BurstEvent`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct BurstEvent {
    pub count: u32,
    pub radius: f32,
    pub color: Option<Color>,
}

impl Default for BurstEvent {
    fn default() -> Self {
        Self {
            count: Default::default(),
            radius: Default::default(),
            color: Default::default(),
        }
    }
}

impl FromJson for BurstEvent {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            count: field(o, "count", d.count)?,
            radius: field(o, "radius", d.radius)?,
            color: field(o, "color", d.color)?,
        })
    }
}

impl ToJson for BurstEvent {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("count".into(), self.count.to_json());
        o.insert("radius".into(), self.radius.to_json());
        o.insert("color".into(), self.color.to_json());
        Value::Object(o)
    }
}

/// `StateEvent`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct StateEvent {
    pub group: String,
    pub state: String,
}

impl Default for StateEvent {
    fn default() -> Self {
        Self {
            group: Default::default(),
            state: Default::default(),
        }
    }
}

impl FromJson for StateEvent {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            group: field(o, "group", d.group)?,
            state: field(o, "state", d.state)?,
        })
    }
}

impl ToJson for StateEvent {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("group".into(), self.group.to_json());
        o.insert("state".into(), self.state.to_json());
        Value::Object(o)
    }
}

/// `CallbackEvent`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct CallbackEvent {
    pub name: String,
}

impl Default for CallbackEvent {
    fn default() -> Self {
        Self {
            name: Default::default(),
        }
    }
}

impl FromJson for CallbackEvent {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
        })
    }
}

impl ToJson for CallbackEvent {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        Value::Object(o)
    }
}

/// `TimelineAction`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub enum TimelineAction {
    Sound(SoundEvent),
    Haptic(HapticEvent),
    Burst(BurstEvent),
    State(StateEvent),
    Callback(CallbackEvent),
}

impl Default for TimelineAction {
    fn default() -> Self {
        Self::Sound(Default::default())
    }
}

impl TimelineAction {
    /// The tag key and the cases' tags, in order.
    pub const TAG: &'static str = "kind";
    pub const TAGS: &'static [&'static str] = &["sound", "haptic", "burst", "state", "callback"];
    /// This case's tag.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Sound(_) => "sound",
            Self::Haptic(_) => "haptic",
            Self::Burst(_) => "burst",
            Self::State(_) => "state",
            Self::Callback(_) => "callback",
        }
    }
}

impl FromJson for TimelineAction {
    fn from_json(v: &Value) -> Result<Self, String> {
        let o = object(v)?;
        let tag = match o.get(Self::TAG) {
            Some(t) => text(t).map_err(|e| at(Self::TAG, e))?,
            None => return Err(at(Self::TAG, format!("missing: one of {}", Self::TAGS.join(", ")))),
        };
        Ok(match tag {
            "sound" => Self::Sound(SoundEvent::from_json(v)?),
            "haptic" => Self::Haptic(HapticEvent::from_json(v)?),
            "burst" => Self::Burst(BurstEvent::from_json(v)?),
            "state" => Self::State(StateEvent::from_json(v)?),
            "callback" => Self::Callback(CallbackEvent::from_json(v)?),
            t => return Err(at(Self::TAG, unknown_variant("TimelineAction", t, Self::TAGS))),
        })
    }
}

impl ToJson for TimelineAction {
    fn to_json(&self) -> Value {
        let mut v = match self {
            Self::Sound(p) => p.to_json(),
            Self::Haptic(p) => p.to_json(),
            Self::Burst(p) => p.to_json(),
            Self::State(p) => p.to_json(),
            Self::Callback(p) => p.to_json(),
        };
        if let Value::Object(o) = &mut v {
            o.insert(Self::TAG.into(), Value::String(self.tag().into()));
        }
        v
    }
}

/// `TimelineEvent`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineEvent {
    pub at_ms: f32,
    pub target: String,
    pub action: TimelineAction,
}

impl Default for TimelineEvent {
    fn default() -> Self {
        Self {
            at_ms: Default::default(),
            target: Default::default(),
            action: Default::default(),
        }
    }
}

impl FromJson for TimelineEvent {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            at_ms: field(o, "atMs", d.at_ms)?,
            target: field(o, "target", d.target)?,
            action: field(o, "action", d.action)?,
        })
    }
}

impl ToJson for TimelineEvent {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("atMs".into(), self.at_ms.to_json());
        o.insert("target".into(), self.target.to_json());
        o.insert("action".into(), self.action.to_json());
        Value::Object(o)
    }
}

/// `TimelineLabel`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineLabel {
    pub name: String,
    pub at_ms: f32,
}

impl Default for TimelineLabel {
    fn default() -> Self {
        Self {
            name: Default::default(),
            at_ms: Default::default(),
        }
    }
}

impl FromJson for TimelineLabel {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            at_ms: field(o, "atMs", d.at_ms)?,
        })
    }
}

impl ToJson for TimelineLabel {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("atMs".into(), self.at_ms.to_json());
        Value::Object(o)
    }
}

/// `UiTimeline`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiTimeline {
    pub tracks: Vec<TimelineTrack>,
    pub events: Vec<TimelineEvent>,
    pub labels: Vec<TimelineLabel>,
    pub duration_ms: f32,
}

impl Default for UiTimeline {
    fn default() -> Self {
        Self {
            tracks: Default::default(),
            events: Default::default(),
            labels: Default::default(),
            duration_ms: Default::default(),
        }
    }
}

impl FromJson for UiTimeline {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tracks: field(o, "tracks", d.tracks)?,
            events: field(o, "events", d.events)?,
            labels: field(o, "labels", d.labels)?,
            duration_ms: field(o, "durationMs", d.duration_ms)?,
        })
    }
}

impl ToJson for UiTimeline {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tracks".into(), self.tracks.to_json());
        o.insert("events".into(), self.events.to_json());
        o.insert("labels".into(), self.labels.to_json());
        o.insert("durationMs".into(), self.duration_ms.to_json());
        Value::Object(o)
    }
}

/// `ChoreoPlay`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ChoreoPlay {
    pub timeline: String,
    pub target: String,
    pub delay_ms: f32,
}

impl Default for ChoreoPlay {
    fn default() -> Self {
        Self {
            timeline: Default::default(),
            target: Default::default(),
            delay_ms: Default::default(),
        }
    }
}

impl FromJson for ChoreoPlay {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            timeline: field(o, "timeline", d.timeline)?,
            target: field(o, "target", d.target)?,
            delay_ms: field(o, "delayMs", d.delay_ms)?,
        })
    }
}

impl ToJson for ChoreoPlay {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("timeline".into(), self.timeline.to_json());
        o.insert("target".into(), self.target.to_json());
        o.insert("delayMs".into(), self.delay_ms.to_json());
        Value::Object(o)
    }
}

/// `ChoreoSteps`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ChoreoSteps {
    pub steps: Vec<Choreo>,
}

impl Default for ChoreoSteps {
    fn default() -> Self {
        Self {
            steps: Default::default(),
        }
    }
}

impl FromJson for ChoreoSteps {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            steps: field(o, "steps", d.steps)?,
        })
    }
}

impl ToJson for ChoreoSteps {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("steps".into(), self.steps.to_json());
        Value::Object(o)
    }
}

/// `ChoreoStagger`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ChoreoStagger {
    pub timeline: String,
    pub targets: Vec<String>,
    pub step_ms: f32,
    pub cap_ms: f32,
    pub from: String,
}

impl Default for ChoreoStagger {
    fn default() -> Self {
        Self {
            timeline: Default::default(),
            targets: Default::default(),
            step_ms: Default::default(),
            cap_ms: Default::default(),
            from: String::from("start"),
        }
    }
}

impl FromJson for ChoreoStagger {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            timeline: field(o, "timeline", d.timeline)?,
            targets: field(o, "targets", d.targets)?,
            step_ms: field(o, "stepMs", d.step_ms)?,
            cap_ms: field(o, "capMs", d.cap_ms)?,
            from: field(o, "from", d.from)?,
        })
    }
}

impl ToJson for ChoreoStagger {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("timeline".into(), self.timeline.to_json());
        o.insert("targets".into(), self.targets.to_json());
        o.insert("stepMs".into(), self.step_ms.to_json());
        o.insert("capMs".into(), self.cap_ms.to_json());
        o.insert("from".into(), self.from.to_json());
        Value::Object(o)
    }
}

/// `ChoreoCascade`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ChoreoCascade {
    pub steps: Vec<Choreo>,
    pub on: String,
}

impl Default for ChoreoCascade {
    fn default() -> Self {
        Self {
            steps: Default::default(),
            on: Default::default(),
        }
    }
}

impl FromJson for ChoreoCascade {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            steps: field(o, "steps", d.steps)?,
            on: field(o, "on", d.on)?,
        })
    }
}

impl ToJson for ChoreoCascade {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("steps".into(), self.steps.to_json());
        o.insert("on".into(), self.on.to_json());
        Value::Object(o)
    }
}

/// `Choreo`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub enum Choreo {
    Play(ChoreoPlay),
    Sequence(ChoreoSteps),
    Parallel(ChoreoSteps),
    Stagger(ChoreoStagger),
    Cascade(ChoreoCascade),
}

impl Default for Choreo {
    fn default() -> Self {
        Self::Play(Default::default())
    }
}

impl Choreo {
    /// The tag key and the cases' tags, in order.
    pub const TAG: &'static str = "kind";
    pub const TAGS: &'static [&'static str] = &["play", "sequence", "parallel", "stagger", "cascade"];
    /// This case's tag.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Play(_) => "play",
            Self::Sequence(_) => "sequence",
            Self::Parallel(_) => "parallel",
            Self::Stagger(_) => "stagger",
            Self::Cascade(_) => "cascade",
        }
    }
}

impl FromJson for Choreo {
    fn from_json(v: &Value) -> Result<Self, String> {
        let o = object(v)?;
        let tag = match o.get(Self::TAG) {
            Some(t) => text(t).map_err(|e| at(Self::TAG, e))?,
            None => return Err(at(Self::TAG, format!("missing: one of {}", Self::TAGS.join(", ")))),
        };
        Ok(match tag {
            "play" => Self::Play(ChoreoPlay::from_json(v)?),
            "sequence" => Self::Sequence(ChoreoSteps::from_json(v)?),
            "parallel" => Self::Parallel(ChoreoSteps::from_json(v)?),
            "stagger" => Self::Stagger(ChoreoStagger::from_json(v)?),
            "cascade" => Self::Cascade(ChoreoCascade::from_json(v)?),
            t => return Err(at(Self::TAG, unknown_variant("Choreo", t, Self::TAGS))),
        })
    }
}

impl ToJson for Choreo {
    fn to_json(&self) -> Value {
        let mut v = match self {
            Self::Play(p) => p.to_json(),
            Self::Sequence(p) => p.to_json(),
            Self::Parallel(p) => p.to_json(),
            Self::Stagger(p) => p.to_json(),
            Self::Cascade(p) => p.to_json(),
        };
        if let Value::Object(o) = &mut v {
            o.insert(Self::TAG.into(), Value::String(self.tag().into()));
        }
        v
    }
}

/// `PlayDesc`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PlayDesc {
    pub reverse: bool,
    pub repeat: Repeat,
    pub speed: f32,
    pub from_ms: f32,
    pub from_label: String,
    pub hold: bool,
    pub decorative: bool,
}

impl Default for PlayDesc {
    fn default() -> Self {
        Self {
            reverse: Default::default(),
            repeat: Default::default(),
            speed: Default::default(),
            from_ms: Default::default(),
            from_label: Default::default(),
            hold: Default::default(),
            decorative: Default::default(),
        }
    }
}

impl FromJson for PlayDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            reverse: field(o, "reverse", d.reverse)?,
            repeat: field(o, "repeat", d.repeat)?,
            speed: field(o, "speed", d.speed)?,
            from_ms: field(o, "fromMs", d.from_ms)?,
            from_label: field(o, "fromLabel", d.from_label)?,
            hold: field(o, "hold", d.hold)?,
            decorative: field(o, "decorative", d.decorative)?,
        })
    }
}

impl ToJson for PlayDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("reverse".into(), self.reverse.to_json());
        o.insert("repeat".into(), self.repeat.to_json());
        o.insert("speed".into(), self.speed.to_json());
        o.insert("fromMs".into(), self.from_ms.to_json());
        o.insert("fromLabel".into(), self.from_label.to_json());
        o.insert("hold".into(), self.hold.to_json());
        o.insert("decorative".into(), self.decorative.to_json());
        Value::Object(o)
    }
}

/// `PlayInfo`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PlayInfo {
    pub at_ms: f32,
    pub length_ms: f32,
    pub paused: bool,
    pub reverse: bool,
    pub ended: bool,
}

impl Default for PlayInfo {
    fn default() -> Self {
        Self {
            at_ms: Default::default(),
            length_ms: Default::default(),
            paused: Default::default(),
            reverse: Default::default(),
            ended: Default::default(),
        }
    }
}

impl FromJson for PlayInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            at_ms: field(o, "atMs", d.at_ms)?,
            length_ms: field(o, "lengthMs", d.length_ms)?,
            paused: field(o, "paused", d.paused)?,
            reverse: field(o, "reverse", d.reverse)?,
            ended: field(o, "ended", d.ended)?,
        })
    }
}

impl ToJson for PlayInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("atMs".into(), self.at_ms.to_json());
        o.insert("lengthMs".into(), self.length_ms.to_json());
        o.insert("paused".into(), self.paused.to_json());
        o.insert("reverse".into(), self.reverse.to_json());
        o.insert("ended".into(), self.ended.to_json());
        Value::Object(o)
    }
}

/// `GestureInfo`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct GestureInfo {
    pub held: bool,
    pub dragging: bool,
    pub offset_x: f32,
    pub offset_y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub scale: f32,
    pub rotation: f32,
    pub snapped: i32,
    pub dismissed: bool,
    pub over: String,
    pub dropped_on: String,
    pub hovering: bool,
    pub received: String,
    pub tapped: bool,
    pub double_tapped: bool,
    pub long_pressed: bool,
    pub long_press: f32,
}

impl Default for GestureInfo {
    fn default() -> Self {
        Self {
            held: Default::default(),
            dragging: Default::default(),
            offset_x: Default::default(),
            offset_y: Default::default(),
            velocity_x: Default::default(),
            velocity_y: Default::default(),
            scale: Default::default(),
            rotation: Default::default(),
            snapped: Default::default(),
            dismissed: Default::default(),
            over: Default::default(),
            dropped_on: Default::default(),
            hovering: Default::default(),
            received: Default::default(),
            tapped: Default::default(),
            double_tapped: Default::default(),
            long_pressed: Default::default(),
            long_press: Default::default(),
        }
    }
}

impl FromJson for GestureInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            held: field(o, "held", d.held)?,
            dragging: field(o, "dragging", d.dragging)?,
            offset_x: field(o, "offsetX", d.offset_x)?,
            offset_y: field(o, "offsetY", d.offset_y)?,
            velocity_x: field(o, "velocityX", d.velocity_x)?,
            velocity_y: field(o, "velocityY", d.velocity_y)?,
            scale: field(o, "scale", d.scale)?,
            rotation: field(o, "rotation", d.rotation)?,
            snapped: field(o, "snapped", d.snapped)?,
            dismissed: field(o, "dismissed", d.dismissed)?,
            over: field(o, "over", d.over)?,
            dropped_on: field(o, "droppedOn", d.dropped_on)?,
            hovering: field(o, "hovering", d.hovering)?,
            received: field(o, "received", d.received)?,
            tapped: field(o, "tapped", d.tapped)?,
            double_tapped: field(o, "doubleTapped", d.double_tapped)?,
            long_pressed: field(o, "longPressed", d.long_pressed)?,
            long_press: field(o, "longPress", d.long_press)?,
        })
    }
}

impl ToJson for GestureInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("held".into(), self.held.to_json());
        o.insert("dragging".into(), self.dragging.to_json());
        o.insert("offsetX".into(), self.offset_x.to_json());
        o.insert("offsetY".into(), self.offset_y.to_json());
        o.insert("velocityX".into(), self.velocity_x.to_json());
        o.insert("velocityY".into(), self.velocity_y.to_json());
        o.insert("scale".into(), self.scale.to_json());
        o.insert("rotation".into(), self.rotation.to_json());
        o.insert("snapped".into(), self.snapped.to_json());
        o.insert("dismissed".into(), self.dismissed.to_json());
        o.insert("over".into(), self.over.to_json());
        o.insert("droppedOn".into(), self.dropped_on.to_json());
        o.insert("hovering".into(), self.hovering.to_json());
        o.insert("received".into(), self.received.to_json());
        o.insert("tapped".into(), self.tapped.to_json());
        o.insert("doubleTapped".into(), self.double_tapped.to_json());
        o.insert("longPressed".into(), self.long_pressed.to_json());
        o.insert("longPress".into(), self.long_press.to_json());
        Value::Object(o)
    }
}

/// `UiMotionEvent`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiMotionEvent {
    pub kind: String,
    pub name: String,
    pub target: String,
    pub play: u32,
    pub index: u32,
    pub playback_rate: f32,
}

impl Default for UiMotionEvent {
    fn default() -> Self {
        Self {
            kind: Default::default(),
            name: Default::default(),
            target: Default::default(),
            play: Default::default(),
            index: Default::default(),
            playback_rate: Default::default(),
        }
    }
}

impl FromJson for UiMotionEvent {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            kind: field(o, "kind", d.kind)?,
            name: field(o, "name", d.name)?,
            target: field(o, "target", d.target)?,
            play: field(o, "play", d.play)?,
            index: field(o, "index", d.index)?,
            playback_rate: field(o, "playbackRate", d.playback_rate)?,
        })
    }
}

impl ToJson for UiMotionEvent {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("kind".into(), self.kind.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("target".into(), self.target.to_json());
        o.insert("play".into(), self.play.to_json());
        o.insert("index".into(), self.index.to_json());
        o.insert("playbackRate".into(), self.playback_rate.to_json());
        Value::Object(o)
    }
}

/// `UiOverlap`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiOverlap {
    pub path: String,
    pub other: String,
    pub rect: UiRect,
    pub in_flow: bool,
}

impl Default for UiOverlap {
    fn default() -> Self {
        Self {
            path: Default::default(),
            other: Default::default(),
            rect: Default::default(),
            in_flow: Default::default(),
        }
    }
}

impl FromJson for UiOverlap {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            path: field(o, "path", d.path)?,
            other: field(o, "other", d.other)?,
            rect: field(o, "rect", d.rect)?,
            in_flow: field(o, "inFlow", d.in_flow)?,
        })
    }
}

impl ToJson for UiOverlap {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("path".into(), self.path.to_json());
        o.insert("other".into(), self.other.to_json());
        o.insert("rect".into(), self.rect.to_json());
        o.insert("inFlow".into(), self.in_flow.to_json());
        Value::Object(o)
    }
}

/// `UiLayoutStats`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiLayoutStats {
    pub elements: u32,
    pub rebuilt: u32,
    pub laid_out: u32,
    pub kept_commands: u32,
}

impl Default for UiLayoutStats {
    fn default() -> Self {
        Self {
            elements: Default::default(),
            rebuilt: Default::default(),
            laid_out: Default::default(),
            kept_commands: Default::default(),
        }
    }
}

impl FromJson for UiLayoutStats {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            elements: field(o, "elements", d.elements)?,
            rebuilt: field(o, "rebuilt", d.rebuilt)?,
            laid_out: field(o, "laidOut", d.laid_out)?,
            kept_commands: field(o, "keptCommands", d.kept_commands)?,
        })
    }
}

impl ToJson for UiLayoutStats {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("elements".into(), self.elements.to_json());
        o.insert("rebuilt".into(), self.rebuilt.to_json());
        o.insert("laidOut".into(), self.laid_out.to_json());
        o.insert("keptCommands".into(), self.kept_commands.to_json());
        Value::Object(o)
    }
}

/// `WorldOcclusion`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WorldOcclusion {
    #[default]
    None,
    Fade,
    Hide,
}

impl WorldOcclusion {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["none", "fade", "hide"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::None, Self::Fade, Self::Hide];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("WorldOcclusion", s, Self::NAMES)),
        }
    }
}

impl FromJson for WorldOcclusion {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "WorldOcclusion", Self::NAMES)?)
    }
}

impl ToJson for WorldOcclusion {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `MassKind`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MassKind {
    #[default]
    Number,
    Text,
    Bar,
    Icon,
    Particle,
}

impl MassKind {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["number", "text", "bar", "icon", "particle"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Number, Self::Text, Self::Bar, Self::Icon, Self::Particle];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("MassKind", s, Self::NAMES)),
        }
    }
}

impl FromJson for MassKind {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "MassKind", Self::NAMES)?)
    }
}

impl ToJson for MassKind {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `MassSpace`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MassSpace {
    #[default]
    World,
    Screen,
}

impl MassSpace {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["world", "screen"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::World, Self::Screen];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("MassSpace", s, Self::NAMES)),
        }
    }
}

impl FromJson for MassSpace {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "MassSpace", Self::NAMES)?)
    }
}

impl ToJson for MassSpace {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `ParticleShape`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ParticleShape {
    #[default]
    Square,
    Circle,
    Diamond,
}

impl ParticleShape {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["square", "circle", "diamond"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Square, Self::Circle, Self::Diamond];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("ParticleShape", s, Self::NAMES)),
        }
    }
}

impl FromJson for ParticleShape {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "ParticleShape", Self::NAMES)?)
    }
}

impl ToJson for ParticleShape {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `MassKey`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct MassKey {
    pub t: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub scale: f32,
    pub alpha: f32,
    pub ease: String,
}

impl Default for MassKey {
    fn default() -> Self {
        Self {
            t: Default::default(),
            offset_x: Default::default(),
            offset_y: Default::default(),
            scale: 1.0_f32,
            alpha: 1.0_f32,
            ease: String::from("none"),
        }
    }
}

impl FromJson for MassKey {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            t: field(o, "t", d.t)?,
            offset_x: field(o, "offsetX", d.offset_x)?,
            offset_y: field(o, "offsetY", d.offset_y)?,
            scale: field(o, "scale", d.scale)?,
            alpha: field(o, "alpha", d.alpha)?,
            ease: field(o, "ease", d.ease)?,
        })
    }
}

impl ToJson for MassKey {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("t".into(), self.t.to_json());
        o.insert("offsetX".into(), self.offset_x.to_json());
        o.insert("offsetY".into(), self.offset_y.to_json());
        o.insert("scale".into(), self.scale.to_json());
        o.insert("alpha".into(), self.alpha.to_json());
        o.insert("ease".into(), self.ease.to_json());
        Value::Object(o)
    }
}

/// `MassTemplateDesc`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct MassTemplateDesc {
    pub kind: MassKind,
    pub space: MassSpace,
    pub font: Option<u64>,
    pub texture: Option<u64>,
    pub strings: Vec<String>,
    pub frames: Vec<f32>,
    pub decimals: u32,
    pub plus: bool,
    pub compact: bool,
    pub bar_back: Option<Paint>,
    pub bar_ghost: Option<Paint>,
    pub bar_radius: f32,
    pub particle_shape: ParticleShape,
    pub size: Vec2,
    pub pivot: Vec2,
    pub lifetime: u32,
    pub keys: Vec<MassKey>,
    pub drift: f32,
    pub gravity: f32,
    pub drag: bool,
    pub reference: f32,
    pub min_scale: f32,
    pub max_scale: f32,
    pub occlusion: WorldOcclusion,
    pub occluded_alpha: f32,
    pub outline: Option<Paint>,
    pub outline_width: f32,
}

impl Default for MassTemplateDesc {
    fn default() -> Self {
        Self {
            kind: Default::default(),
            space: Default::default(),
            font: Default::default(),
            texture: Default::default(),
            strings: Default::default(),
            frames: Default::default(),
            decimals: Default::default(),
            plus: Default::default(),
            compact: Default::default(),
            bar_back: Default::default(),
            bar_ghost: Default::default(),
            bar_radius: Default::default(),
            particle_shape: Default::default(),
            size: Default::default(),
            pivot: Vec2::new(0.5_f32, 0.5_f32),
            lifetime: Default::default(),
            keys: Default::default(),
            drift: Default::default(),
            gravity: Default::default(),
            drag: Default::default(),
            reference: Default::default(),
            min_scale: 0.25_f32,
            max_scale: 4.0_f32,
            occlusion: Default::default(),
            occluded_alpha: 0.25_f32,
            outline: Default::default(),
            outline_width: 2.0_f32,
        }
    }
}

impl FromJson for MassTemplateDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            kind: field(o, "kind", d.kind)?,
            space: field(o, "space", d.space)?,
            font: field(o, "font", d.font)?,
            texture: field(o, "texture", d.texture)?,
            strings: field(o, "strings", d.strings)?,
            frames: field(o, "frames", d.frames)?,
            decimals: field(o, "decimals", d.decimals)?,
            plus: field(o, "plus", d.plus)?,
            compact: field(o, "compact", d.compact)?,
            bar_back: field(o, "barBack", d.bar_back)?,
            bar_ghost: field(o, "barGhost", d.bar_ghost)?,
            bar_radius: field(o, "barRadius", d.bar_radius)?,
            particle_shape: field(o, "particleShape", d.particle_shape)?,
            size: field(o, "size", d.size)?,
            pivot: field(o, "pivot", d.pivot)?,
            lifetime: field(o, "lifetime", d.lifetime)?,
            keys: field(o, "keys", d.keys)?,
            drift: field(o, "drift", d.drift)?,
            gravity: field(o, "gravity", d.gravity)?,
            drag: field(o, "drag", d.drag)?,
            reference: field(o, "reference", d.reference)?,
            min_scale: field(o, "minScale", d.min_scale)?,
            max_scale: field(o, "maxScale", d.max_scale)?,
            occlusion: field(o, "occlusion", d.occlusion)?,
            occluded_alpha: field(o, "occludedAlpha", d.occluded_alpha)?,
            outline: field(o, "outline", d.outline)?,
            outline_width: field(o, "outlineWidth", d.outline_width)?,
        })
    }
}

impl ToJson for MassTemplateDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("kind".into(), self.kind.to_json());
        o.insert("space".into(), self.space.to_json());
        o.insert("font".into(), self.font.to_json());
        o.insert("texture".into(), self.texture.to_json());
        o.insert("strings".into(), self.strings.to_json());
        o.insert("frames".into(), self.frames.to_json());
        o.insert("decimals".into(), self.decimals.to_json());
        o.insert("plus".into(), self.plus.to_json());
        o.insert("compact".into(), self.compact.to_json());
        o.insert("barBack".into(), self.bar_back.to_json());
        o.insert("barGhost".into(), self.bar_ghost.to_json());
        o.insert("barRadius".into(), self.bar_radius.to_json());
        o.insert("particleShape".into(), self.particle_shape.to_json());
        o.insert("size".into(), self.size.to_json());
        o.insert("pivot".into(), self.pivot.to_json());
        o.insert("lifetime".into(), self.lifetime.to_json());
        o.insert("keys".into(), self.keys.to_json());
        o.insert("drift".into(), self.drift.to_json());
        o.insert("gravity".into(), self.gravity.to_json());
        o.insert("drag".into(), self.drag.to_json());
        o.insert("reference".into(), self.reference.to_json());
        o.insert("minScale".into(), self.min_scale.to_json());
        o.insert("maxScale".into(), self.max_scale.to_json());
        o.insert("occlusion".into(), self.occlusion.to_json());
        o.insert("occludedAlpha".into(), self.occluded_alpha.to_json());
        o.insert("outline".into(), self.outline.to_json());
        o.insert("outlineWidth".into(), self.outline_width.to_json());
        Value::Object(o)
    }
}

/// `MassInstance`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct MassInstance {
    pub position: Vec3,
    pub value: f64,
    pub color: Option<Paint>,
    pub scale: f32,
    pub seed: u32,
    pub ghost: f32,
    pub velocity: Vec2,
    pub spin: f32,
    pub rotation: f32,
}

impl Default for MassInstance {
    fn default() -> Self {
        Self {
            position: Default::default(),
            value: Default::default(),
            color: Default::default(),
            scale: 1.0_f32,
            seed: Default::default(),
            ghost: Default::default(),
            velocity: Default::default(),
            spin: Default::default(),
            rotation: Default::default(),
        }
    }
}

impl FromJson for MassInstance {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            position: field(o, "position", d.position)?,
            value: field(o, "value", d.value)?,
            color: field(o, "color", d.color)?,
            scale: field(o, "scale", d.scale)?,
            seed: field(o, "seed", d.seed)?,
            ghost: field(o, "ghost", d.ghost)?,
            velocity: field(o, "velocity", d.velocity)?,
            spin: field(o, "spin", d.spin)?,
            rotation: field(o, "rotation", d.rotation)?,
        })
    }
}

impl ToJson for MassInstance {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("position".into(), self.position.to_json());
        o.insert("value".into(), self.value.to_json());
        o.insert("color".into(), self.color.to_json());
        o.insert("scale".into(), self.scale.to_json());
        o.insert("seed".into(), self.seed.to_json());
        o.insert("ghost".into(), self.ghost.to_json());
        o.insert("velocity".into(), self.velocity.to_json());
        o.insert("spin".into(), self.spin.to_json());
        o.insert("rotation".into(), self.rotation.to_json());
        Value::Object(o)
    }
}

/// `UiDocEvent`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiDocEvent {
    pub mount: String,
    pub path: String,
    pub row_key: String,
    pub kind: String,
    pub name: String,
    pub value: Value,
}

impl Default for UiDocEvent {
    fn default() -> Self {
        Self {
            mount: Default::default(),
            path: Default::default(),
            row_key: Default::default(),
            kind: Default::default(),
            name: Default::default(),
            value: Default::default(),
        }
    }
}

impl FromJson for UiDocEvent {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            mount: field(o, "mount", d.mount)?,
            path: field(o, "path", d.path)?,
            row_key: field(o, "rowKey", d.row_key)?,
            kind: field(o, "kind", d.kind)?,
            name: field(o, "name", d.name)?,
            value: field(o, "value", d.value)?,
        })
    }
}

impl ToJson for UiDocEvent {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("mount".into(), self.mount.to_json());
        o.insert("path".into(), self.path.to_json());
        o.insert("rowKey".into(), self.row_key.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("value".into(), self.value.to_json());
        Value::Object(o)
    }
}

/// `UiLayer`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiLayer {
    pub serial: u32,
    pub screen: String,
    pub mount: String,
    pub kind: String,
    pub input: String,
    pub capture: bool,
    pub pause: bool,
    pub under: String,
    pub back: String,
    pub focus: String,
    pub remember: bool,
    pub top: bool,
    pub active: bool,
    pub hidden: bool,
    pub inert: bool,
}

impl Default for UiLayer {
    fn default() -> Self {
        Self {
            serial: Default::default(),
            screen: Default::default(),
            mount: Default::default(),
            kind: Default::default(),
            input: Default::default(),
            capture: Default::default(),
            pause: Default::default(),
            under: Default::default(),
            back: Default::default(),
            focus: Default::default(),
            remember: Default::default(),
            top: Default::default(),
            active: Default::default(),
            hidden: Default::default(),
            inert: Default::default(),
        }
    }
}

impl FromJson for UiLayer {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            serial: field(o, "serial", d.serial)?,
            screen: field(o, "screen", d.screen)?,
            mount: field(o, "mount", d.mount)?,
            kind: field(o, "kind", d.kind)?,
            input: field(o, "input", d.input)?,
            capture: field(o, "capture", d.capture)?,
            pause: field(o, "pause", d.pause)?,
            under: field(o, "under", d.under)?,
            back: field(o, "back", d.back)?,
            focus: field(o, "focus", d.focus)?,
            remember: field(o, "remember", d.remember)?,
            top: field(o, "top", d.top)?,
            active: field(o, "active", d.active)?,
            hidden: field(o, "hidden", d.hidden)?,
            inert: field(o, "inert", d.inert)?,
        })
    }
}

impl ToJson for UiLayer {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("serial".into(), self.serial.to_json());
        o.insert("screen".into(), self.screen.to_json());
        o.insert("mount".into(), self.mount.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("input".into(), self.input.to_json());
        o.insert("capture".into(), self.capture.to_json());
        o.insert("pause".into(), self.pause.to_json());
        o.insert("under".into(), self.under.to_json());
        o.insert("back".into(), self.back.to_json());
        o.insert("focus".into(), self.focus.to_json());
        o.insert("remember".into(), self.remember.to_json());
        o.insert("top".into(), self.top.to_json());
        o.insert("active".into(), self.active.to_json());
        o.insert("hidden".into(), self.hidden.to_json());
        o.insert("inert".into(), self.inert.to_json());
        Value::Object(o)
    }
}

/// `UiActionTrigger`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum UiActionTrigger {
    #[default]
    Press,
    Release,
    Hold,
}

impl UiActionTrigger {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["press", "release", "hold"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Press, Self::Release, Self::Hold];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("UiActionTrigger", s, Self::NAMES)),
        }
    }
}

impl FromJson for UiActionTrigger {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "UiActionTrigger", Self::NAMES)?)
    }
}

impl ToJson for UiActionTrigger {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `UiActionDesc`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiActionDesc {
    pub name: String,
    pub keys: Vec<String>,
    pub buttons: Vec<String>,
    pub trigger: UiActionTrigger,
    pub hold_ms: f32,
    pub toggle: bool,
    pub label: String,
    pub action: Option<String>,
}

impl Default for UiActionDesc {
    fn default() -> Self {
        Self {
            name: Default::default(),
            keys: Default::default(),
            buttons: Default::default(),
            trigger: Default::default(),
            hold_ms: Default::default(),
            toggle: Default::default(),
            label: Default::default(),
            action: Default::default(),
        }
    }
}

impl FromJson for UiActionDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            keys: field(o, "keys", d.keys)?,
            buttons: field(o, "buttons", d.buttons)?,
            trigger: field(o, "trigger", d.trigger)?,
            hold_ms: field(o, "holdMs", d.hold_ms)?,
            toggle: field(o, "toggle", d.toggle)?,
            label: field(o, "label", d.label)?,
            action: field(o, "action", d.action)?,
        })
    }
}

impl ToJson for UiActionDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("keys".into(), self.keys.to_json());
        o.insert("buttons".into(), self.buttons.to_json());
        o.insert("trigger".into(), self.trigger.to_json());
        o.insert("holdMs".into(), self.hold_ms.to_json());
        o.insert("toggle".into(), self.toggle.to_json());
        o.insert("label".into(), self.label.to_json());
        if let Some(x) = &self.action {
            o.insert("action".into(), x.to_json());
        }
        Value::Object(o)
    }
}

/// `UiDevice`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum UiDevice {
    #[default]
    KeyboardMouse,
    Xbox,
    Playstation,
    Switch,
    Gamepad,
    Touch,
}

impl UiDevice {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["keyboardMouse", "xbox", "playstation", "switch", "gamepad", "touch"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::KeyboardMouse, Self::Xbox, Self::Playstation, Self::Switch, Self::Gamepad, Self::Touch];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("UiDevice", s, Self::NAMES)),
        }
    }
}

impl FromJson for UiDevice {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "UiDevice", Self::NAMES)?)
    }
}

impl ToJson for UiDevice {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `UiGlyph`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiGlyph {
    pub icon: String,
    pub text: String,
    pub device: UiDevice,
}

impl Default for UiGlyph {
    fn default() -> Self {
        Self {
            icon: Default::default(),
            text: Default::default(),
            device: Default::default(),
        }
    }
}

impl FromJson for UiGlyph {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            icon: field(o, "icon", d.icon)?,
            text: field(o, "text", d.text)?,
            device: field(o, "device", d.device)?,
        })
    }
}

impl ToJson for UiGlyph {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("icon".into(), self.icon.to_json());
        o.insert("text".into(), self.text.to_json());
        o.insert("device".into(), self.device.to_json());
        Value::Object(o)
    }
}

/// `UiActionHint`: packages/knowledge/wiki/ui.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiActionHint {
    pub action: String,
    pub hint: String,
    pub icon: String,
    pub text: String,
    pub hold: bool,
}

impl Default for UiActionHint {
    fn default() -> Self {
        Self {
            action: Default::default(),
            hint: Default::default(),
            icon: Default::default(),
            text: Default::default(),
            hold: Default::default(),
        }
    }
}

impl FromJson for UiActionHint {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            action: field(o, "action", d.action)?,
            hint: field(o, "hint", d.hint)?,
            icon: field(o, "icon", d.icon)?,
            text: field(o, "text", d.text)?,
            hold: field(o, "hold", d.hold)?,
        })
    }
}

impl ToJson for UiActionHint {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("action".into(), self.action.to_json());
        o.insert("hint".into(), self.hint.to_json());
        o.insert("icon".into(), self.icon.to_json());
        o.insert("text".into(), self.text.to_json());
        o.insert("hold".into(), self.hold.to_json());
        Value::Object(o)
    }
}

/// `CommandDesc`: packages/knowledge/wiki/commands.md.
#[derive(Clone, Debug, PartialEq)]
pub struct CommandDesc {
    pub usage: String,
    pub doc: String,
}

impl Default for CommandDesc {
    fn default() -> Self {
        Self {
            usage: Default::default(),
            doc: Default::default(),
        }
    }
}

impl FromJson for CommandDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            usage: field(o, "usage", d.usage)?,
            doc: field(o, "doc", d.doc)?,
        })
    }
}

impl ToJson for CommandDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("usage".into(), self.usage.to_json());
        o.insert("doc".into(), self.doc.to_json());
        Value::Object(o)
    }
}

/// `CommandStep`: packages/knowledge/wiki/commands.md.
#[derive(Clone, Debug, PartialEq)]
pub struct CommandStep {
    pub at: f32,
    pub action: String,
    pub args: Vec<String>,
}

impl Default for CommandStep {
    fn default() -> Self {
        Self {
            at: Default::default(),
            action: Default::default(),
            args: Default::default(),
        }
    }
}

impl FromJson for CommandStep {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            at: field(o, "at", d.at)?,
            action: field(o, "action", d.action)?,
            args: field(o, "args", d.args)?,
        })
    }
}

impl ToJson for CommandStep {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("at".into(), self.at.to_json());
        o.insert("action".into(), self.action.to_json());
        o.insert("args".into(), self.args.to_json());
        Value::Object(o)
    }
}

/// `CommandInfo`: packages/knowledge/wiki/commands.md.
#[derive(Clone, Debug, PartialEq)]
pub struct CommandInfo {
    pub name: String,
    pub usage: String,
    pub doc: String,
    pub core: bool,
}

impl Default for CommandInfo {
    fn default() -> Self {
        Self {
            name: Default::default(),
            usage: Default::default(),
            doc: Default::default(),
            core: Default::default(),
        }
    }
}

impl FromJson for CommandInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            usage: field(o, "usage", d.usage)?,
            doc: field(o, "doc", d.doc)?,
            core: field(o, "core", d.core)?,
        })
    }
}

impl ToJson for CommandInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("usage".into(), self.usage.to_json());
        o.insert("doc".into(), self.doc.to_json());
        o.insert("core".into(), self.core.to_json());
        Value::Object(o)
    }
}

/// `CommandsReport`: packages/knowledge/wiki/commands.md.
#[derive(Clone, Debug, PartialEq)]
pub struct CommandsReport {
    pub scheduled: u32,
    pub until_tick: u32,
    pub warnings: Vec<String>,
}

impl Default for CommandsReport {
    fn default() -> Self {
        Self {
            scheduled: Default::default(),
            until_tick: Default::default(),
            warnings: Default::default(),
        }
    }
}

impl FromJson for CommandsReport {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            scheduled: field(o, "scheduled", d.scheduled)?,
            until_tick: field(o, "untilTick", d.until_tick)?,
            warnings: field(o, "warnings", d.warnings)?,
        })
    }
}

impl ToJson for CommandsReport {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("scheduled".into(), self.scheduled.to_json());
        o.insert("untilTick".into(), self.until_tick.to_json());
        o.insert("warnings".into(), self.warnings.to_json());
        Value::Object(o)
    }
}

/// `PickResult`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PickResult {
    pub entity: String,
    pub world: OneOf<[f32; 2], [f32; 3]>,
    pub source: Option<String>,
    pub ui: Option<UiPickElement>,
}

impl Default for PickResult {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            world: Default::default(),
            source: Default::default(),
            ui: Default::default(),
        }
    }
}

impl FromJson for PickResult {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entity: field(o, "entity", d.entity)?,
            world: field(o, "world", d.world)?,
            source: field(o, "source", d.source)?,
            ui: field(o, "ui", d.ui)?,
        })
    }
}

impl ToJson for PickResult {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entity".into(), self.entity.to_json());
        o.insert("world".into(), self.world.to_json());
        o.insert("source".into(), self.source.to_json());
        if let Some(x) = &self.ui {
            o.insert("ui".into(), x.to_json());
        }
        Value::Object(o)
    }
}

/// `UiPickRule`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiPickRule {
    pub selector: String,
    pub source: String,
}

impl Default for UiPickRule {
    fn default() -> Self {
        Self {
            selector: Default::default(),
            source: Default::default(),
        }
    }
}

impl FromJson for UiPickRule {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            selector: field(o, "selector", d.selector)?,
            source: field(o, "source", d.source)?,
        })
    }
}

impl ToJson for UiPickRule {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("selector".into(), self.selector.to_json());
        o.insert("source".into(), self.source.to_json());
        Value::Object(o)
    }
}

/// `UiPickElement`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiPickElement {
    pub id: String,
    pub path: String,
    pub kind: String,
    pub text: String,
    pub rect: UiRect,
    pub layer: Option<String>,
    pub source: Option<String>,
    pub part: Option<String>,
    pub rule: Option<UiPickRule>,
    pub rules: Vec<UiPickRule>,
}

impl Default for UiPickElement {
    fn default() -> Self {
        Self {
            id: Default::default(),
            path: Default::default(),
            kind: Default::default(),
            text: Default::default(),
            rect: Default::default(),
            layer: Default::default(),
            source: Default::default(),
            part: Default::default(),
            rule: Default::default(),
            rules: Default::default(),
        }
    }
}

impl FromJson for UiPickElement {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            id: field(o, "id", d.id)?,
            path: field(o, "path", d.path)?,
            kind: field(o, "kind", d.kind)?,
            text: field(o, "text", d.text)?,
            rect: field(o, "rect", d.rect)?,
            layer: field(o, "layer", d.layer)?,
            source: field(o, "source", d.source)?,
            part: field(o, "part", d.part)?,
            rule: field(o, "rule", d.rule)?,
            rules: field(o, "rules", d.rules)?,
        })
    }
}

impl ToJson for UiPickElement {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("id".into(), self.id.to_json());
        o.insert("path".into(), self.path.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("text".into(), self.text.to_json());
        o.insert("rect".into(), self.rect.to_json());
        o.insert("layer".into(), self.layer.to_json());
        o.insert("source".into(), self.source.to_json());
        o.insert("part".into(), self.part.to_json());
        o.insert("rule".into(), self.rule.to_json());
        o.insert("rules".into(), self.rules.to_json());
        Value::Object(o)
    }
}

/// `ProbeTunable`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ProbeTunable {
    pub path: String,
    pub label: Option<String>,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub step: Option<f32>,
}

impl Default for ProbeTunable {
    fn default() -> Self {
        Self {
            path: Default::default(),
            label: Default::default(),
            min: Default::default(),
            max: Default::default(),
            step: Default::default(),
        }
    }
}

impl FromJson for ProbeTunable {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            path: field(o, "path", d.path)?,
            label: field(o, "label", d.label)?,
            min: field(o, "min", d.min)?,
            max: field(o, "max", d.max)?,
            step: field(o, "step", d.step)?,
        })
    }
}

impl ToJson for ProbeTunable {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("path".into(), self.path.to_json());
        if let Some(x) = &self.label {
            o.insert("label".into(), x.to_json());
        }
        if let Some(x) = &self.min {
            o.insert("min".into(), x.to_json());
        }
        if let Some(x) = &self.max {
            o.insert("max".into(), x.to_json());
        }
        if let Some(x) = &self.step {
            o.insert("step".into(), x.to_json());
        }
        Value::Object(o)
    }
}

/// `ProbeEntityRef`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ProbeEntityRef {
    pub id: String,
    pub label: Option<String>,
}

impl Default for ProbeEntityRef {
    fn default() -> Self {
        Self {
            id: Default::default(),
            label: Default::default(),
        }
    }
}

impl FromJson for ProbeEntityRef {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            id: field(o, "id", d.id)?,
            label: field(o, "label", d.label)?,
        })
    }
}

impl ToJson for ProbeEntityRef {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("id".into(), self.id.to_json());
        if let Some(x) = &self.label {
            o.insert("label".into(), x.to_json());
        }
        Value::Object(o)
    }
}

/// `ProbeTimings`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ProbeTimings {
    pub step: f32,
    pub render: f32,
    pub frame_ms: f32,
    pub systems: Vec<ProbeTiming>,
    pub passes: Vec<ProbeTiming>,
    pub gpu_passes: Vec<ProbeTiming>,
    pub triangles: u32,
}

impl Default for ProbeTimings {
    fn default() -> Self {
        Self {
            step: Default::default(),
            render: Default::default(),
            frame_ms: Default::default(),
            systems: Default::default(),
            passes: Default::default(),
            gpu_passes: Default::default(),
            triangles: Default::default(),
        }
    }
}

impl FromJson for ProbeTimings {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            step: field(o, "step", d.step)?,
            render: field(o, "render", d.render)?,
            frame_ms: field(o, "frameMs", d.frame_ms)?,
            systems: field(o, "systems", d.systems)?,
            passes: field(o, "passes", d.passes)?,
            gpu_passes: field(o, "gpuPasses", d.gpu_passes)?,
            triangles: field(o, "triangles", d.triangles)?,
        })
    }
}

impl ToJson for ProbeTimings {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("step".into(), self.step.to_json());
        o.insert("render".into(), self.render.to_json());
        o.insert("frameMs".into(), self.frame_ms.to_json());
        o.insert("systems".into(), self.systems.to_json());
        o.insert("passes".into(), self.passes.to_json());
        o.insert("gpuPasses".into(), self.gpu_passes.to_json());
        o.insert("triangles".into(), self.triangles.to_json());
        Value::Object(o)
    }
}

/// `ProbeTiming`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ProbeTiming {
    pub name: String,
    pub ms: f32,
}

impl Default for ProbeTiming {
    fn default() -> Self {
        Self {
            name: Default::default(),
            ms: Default::default(),
        }
    }
}

impl FromJson for ProbeTiming {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            ms: field(o, "ms", d.ms)?,
        })
    }
}

impl ToJson for ProbeTiming {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("ms".into(), self.ms.to_json());
        Value::Object(o)
    }
}

/// `ProbeState`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ProbeState {
    pub tick: u32,
    pub paused: bool,
    pub seed: u32,
    pub timings: ProbeTimings,
    pub entities: Vec<ProbeEntityRef>,
    pub facts: Value,
}

impl Default for ProbeState {
    fn default() -> Self {
        Self {
            tick: Default::default(),
            paused: Default::default(),
            seed: Default::default(),
            timings: Default::default(),
            entities: Default::default(),
            facts: Default::default(),
        }
    }
}

impl FromJson for ProbeState {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tick: field(o, "tick", d.tick)?,
            paused: field(o, "paused", d.paused)?,
            seed: field(o, "seed", d.seed)?,
            timings: field(o, "timings", d.timings)?,
            entities: field(o, "entities", d.entities)?,
            facts: field(o, "facts", d.facts)?,
        })
    }
}

impl ToJson for ProbeState {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tick".into(), self.tick.to_json());
        o.insert("paused".into(), self.paused.to_json());
        o.insert("seed".into(), self.seed.to_json());
        o.insert("timings".into(), self.timings.to_json());
        o.insert("entities".into(), self.entities.to_json());
        o.insert("facts".into(), self.facts.to_json());
        Value::Object(o)
    }
}

/// `AdvanceOperation`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct AdvanceOperation {
    pub steps: u32,
}

impl Default for AdvanceOperation {
    fn default() -> Self {
        Self {
            steps: Default::default(),
        }
    }
}

impl FromJson for AdvanceOperation {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            steps: field(o, "steps", d.steps)?,
        })
    }
}

impl ToJson for AdvanceOperation {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("steps".into(), self.steps.to_json());
        Value::Object(o)
    }
}

/// `InputOperation`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct InputOperation {
    pub event: InjectedInput,
}

impl Default for InputOperation {
    fn default() -> Self {
        Self {
            event: Default::default(),
        }
    }
}

impl FromJson for InputOperation {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            event: field(o, "event", d.event)?,
        })
    }
}

impl ToJson for InputOperation {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("event".into(), self.event.to_json());
        Value::Object(o)
    }
}

/// `PickOperation`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PickOperation {
    pub x: f32,
    pub y: f32,
}

impl Default for PickOperation {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
        }
    }
}

impl FromJson for PickOperation {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
        })
    }
}

impl ToJson for PickOperation {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        Value::Object(o)
    }
}

/// `EntityOperation`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityOperation {
    pub id: String,
}

impl Default for EntityOperation {
    fn default() -> Self {
        Self {
            id: Default::default(),
        }
    }
}

impl FromJson for EntityOperation {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            id: field(o, "id", d.id)?,
        })
    }
}

impl ToJson for EntityOperation {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("id".into(), self.id.to_json());
        Value::Object(o)
    }
}

/// `ExpectOperation`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ExpectOperation {
    pub what: String,
}

impl Default for ExpectOperation {
    fn default() -> Self {
        Self {
            what: Default::default(),
        }
    }
}

impl FromJson for ExpectOperation {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            what: field(o, "what", d.what)?,
        })
    }
}

impl ToJson for ExpectOperation {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("what".into(), self.what.to_json());
        Value::Object(o)
    }
}

/// `ProbeOperation`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub enum ProbeOperation {
    Advance(AdvanceOperation),
    Input(InputOperation),
    Pause,
    Resume,
    Read,
    Pick(PickOperation),
    Entity(EntityOperation),
    Expect(ExpectOperation),
    Describe,
}

impl Default for ProbeOperation {
    fn default() -> Self {
        Self::Advance(Default::default())
    }
}

impl ProbeOperation {
    /// The tag key and the cases' tags, in order.
    pub const TAG: &'static str = "op";
    pub const TAGS: &'static [&'static str] = &["advance", "input", "pause", "resume", "read", "pick", "entity", "expect", "describe"];
    /// This case's tag.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Advance(_) => "advance",
            Self::Input(_) => "input",
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Read => "read",
            Self::Pick(_) => "pick",
            Self::Entity(_) => "entity",
            Self::Expect(_) => "expect",
            Self::Describe => "describe",
        }
    }
}

impl FromJson for ProbeOperation {
    fn from_json(v: &Value) -> Result<Self, String> {
        let o = object(v)?;
        let tag = match o.get(Self::TAG) {
            Some(t) => text(t).map_err(|e| at(Self::TAG, e))?,
            None => return Err(at(Self::TAG, format!("missing: one of {}", Self::TAGS.join(", ")))),
        };
        Ok(match tag {
            "advance" => Self::Advance(AdvanceOperation::from_json(v)?),
            "input" => Self::Input(InputOperation::from_json(v)?),
            "pause" => Self::Pause,
            "resume" => Self::Resume,
            "read" => Self::Read,
            "pick" => Self::Pick(PickOperation::from_json(v)?),
            "entity" => Self::Entity(EntityOperation::from_json(v)?),
            "expect" => Self::Expect(ExpectOperation::from_json(v)?),
            "describe" => Self::Describe,
            t => return Err(at(Self::TAG, unknown_variant("ProbeOperation", t, Self::TAGS))),
        })
    }
}

impl ToJson for ProbeOperation {
    fn to_json(&self) -> Value {
        let mut v = match self {
            Self::Advance(p) => p.to_json(),
            Self::Input(p) => p.to_json(),
            Self::Pause => Value::Object(Map::new()),
            Self::Resume => Value::Object(Map::new()),
            Self::Read => Value::Object(Map::new()),
            Self::Pick(p) => p.to_json(),
            Self::Entity(p) => p.to_json(),
            Self::Expect(p) => p.to_json(),
            Self::Describe => Value::Object(Map::new()),
        };
        if let Value::Object(o) = &mut v {
            o.insert(Self::TAG.into(), Value::String(self.tag().into()));
        }
        v
    }
}

/// `ProbeProgramResult`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ProbeProgramResult {
    pub results: Vec<Value>,
    pub refused: Option<u32>,
    pub tick: u32,
}

impl Default for ProbeProgramResult {
    fn default() -> Self {
        Self {
            results: Default::default(),
            refused: Default::default(),
            tick: Default::default(),
        }
    }
}

impl FromJson for ProbeProgramResult {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            results: field(o, "results", d.results)?,
            refused: field(o, "refused", d.refused)?,
            tick: field(o, "tick", d.tick)?,
        })
    }
}

impl ToJson for ProbeProgramResult {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("results".into(), self.results.to_json());
        o.insert("refused".into(), self.refused.to_json());
        o.insert("tick".into(), self.tick.to_json());
        Value::Object(o)
    }
}

/// `Hello`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Hello {
    pub protocol: u32,
    pub wire: u32,
    pub substrate: String,
    pub engine: String,
    pub ticks_per_second: u32,
    pub tier: RenderTier,
    pub build: BuildProfile,
    pub members: Vec<String>,
    pub modules: Vec<String>,
}

impl Default for Hello {
    fn default() -> Self {
        Self {
            protocol: Default::default(),
            wire: Default::default(),
            substrate: Default::default(),
            engine: Default::default(),
            ticks_per_second: Default::default(),
            tier: Default::default(),
            build: Default::default(),
            members: Default::default(),
            modules: Default::default(),
        }
    }
}

impl FromJson for Hello {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            protocol: field(o, "protocol", d.protocol)?,
            wire: field(o, "wire", d.wire)?,
            substrate: field(o, "substrate", d.substrate)?,
            engine: field(o, "engine", d.engine)?,
            ticks_per_second: field(o, "ticksPerSecond", d.ticks_per_second)?,
            tier: field(o, "tier", d.tier)?,
            build: field(o, "build", d.build)?,
            members: field(o, "members", d.members)?,
            modules: field(o, "modules", d.modules)?,
        })
    }
}

impl ToJson for Hello {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("protocol".into(), self.protocol.to_json());
        o.insert("wire".into(), self.wire.to_json());
        o.insert("substrate".into(), self.substrate.to_json());
        o.insert("engine".into(), self.engine.to_json());
        o.insert("ticksPerSecond".into(), self.ticks_per_second.to_json());
        o.insert("tier".into(), self.tier.to_json());
        o.insert("build".into(), self.build.to_json());
        o.insert("members".into(), self.members.to_json());
        o.insert("modules".into(), self.modules.to_json());
        Value::Object(o)
    }
}

/// `BuildProfile`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BuildProfile {
    #[default]
    Dev,
    Release,
    Publish,
}

impl BuildProfile {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["dev", "release", "publish"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Dev, Self::Release, Self::Publish];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("BuildProfile", s, Self::NAMES)),
        }
    }
}

impl FromJson for BuildProfile {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "BuildProfile", Self::NAMES)?)
    }
}

impl ToJson for BuildProfile {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `Capture`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Capture {
    pub tick: u32,
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    pub region: Option<ScreenBox>,
    pub png: String,
}

impl Default for Capture {
    fn default() -> Self {
        Self {
            tick: Default::default(),
            width: Default::default(),
            height: Default::default(),
            scale: 1.0_f32,
            region: Default::default(),
            png: Default::default(),
        }
    }
}

impl FromJson for Capture {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tick: field(o, "tick", d.tick)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            scale: field(o, "scale", d.scale)?,
            region: field(o, "region", d.region)?,
            png: field(o, "png", d.png)?,
        })
    }
}

impl ToJson for Capture {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tick".into(), self.tick.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("scale".into(), self.scale.to_json());
        o.insert("region".into(), self.region.to_json());
        o.insert("png".into(), self.png.to_json());
        Value::Object(o)
    }
}

/// `MarkRect`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct MarkRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Default for MarkRect {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            width: Default::default(),
            height: Default::default(),
        }
    }
}

impl FromJson for MarkRect {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
        })
    }
}

impl ToJson for MarkRect {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        Value::Object(o)
    }
}

/// `Mark`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Mark {
    pub n: u32,
    pub kind: String,
    pub id: String,
    pub path: String,
    pub what: String,
    pub text: String,
    pub rect: MarkRect,
    pub point: ScreenPoint,
    pub source: Option<String>,
    pub part: Option<String>,
    pub rule: Option<UiPickRule>,
    pub depth: Option<f32>,
}

impl Default for Mark {
    fn default() -> Self {
        Self {
            n: Default::default(),
            kind: Default::default(),
            id: Default::default(),
            path: Default::default(),
            what: Default::default(),
            text: Default::default(),
            rect: Default::default(),
            point: Default::default(),
            source: Default::default(),
            part: Default::default(),
            rule: Default::default(),
            depth: Default::default(),
        }
    }
}

impl FromJson for Mark {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            n: field(o, "n", d.n)?,
            kind: field(o, "kind", d.kind)?,
            id: field(o, "id", d.id)?,
            path: field(o, "path", d.path)?,
            what: field(o, "what", d.what)?,
            text: field(o, "text", d.text)?,
            rect: field(o, "rect", d.rect)?,
            point: field(o, "point", d.point)?,
            source: field(o, "source", d.source)?,
            part: field(o, "part", d.part)?,
            rule: field(o, "rule", d.rule)?,
            depth: field(o, "depth", d.depth)?,
        })
    }
}

impl ToJson for Mark {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("n".into(), self.n.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("id".into(), self.id.to_json());
        o.insert("path".into(), self.path.to_json());
        o.insert("what".into(), self.what.to_json());
        o.insert("text".into(), self.text.to_json());
        o.insert("rect".into(), self.rect.to_json());
        o.insert("point".into(), self.point.to_json());
        o.insert("source".into(), self.source.to_json());
        o.insert("part".into(), self.part.to_json());
        o.insert("rule".into(), self.rule.to_json());
        o.insert("depth".into(), self.depth.to_json());
        Value::Object(o)
    }
}

/// `MarksDesc`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct MarksDesc {
    pub width: u32,
    pub height: u32,
    pub fit: u32,
    pub ui: bool,
    pub entities: bool,
    pub max: u32,
}

impl Default for MarksDesc {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
            fit: 1280,
            ui: true,
            entities: true,
            max: 64,
        }
    }
}

impl FromJson for MarksDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            fit: field(o, "fit", d.fit)?,
            ui: field(o, "ui", d.ui)?,
            entities: field(o, "entities", d.entities)?,
            max: field(o, "max", d.max)?,
        })
    }
}

impl ToJson for MarksDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("fit".into(), self.fit.to_json());
        o.insert("ui".into(), self.ui.to_json());
        o.insert("entities".into(), self.entities.to_json());
        o.insert("max".into(), self.max.to_json());
        Value::Object(o)
    }
}

/// `MarkedCapture`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct MarkedCapture {
    pub tick: u32,
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    pub png: String,
    pub marks: Vec<Mark>,
    pub omitted: u32,
}

impl Default for MarkedCapture {
    fn default() -> Self {
        Self {
            tick: Default::default(),
            width: Default::default(),
            height: Default::default(),
            scale: Default::default(),
            png: Default::default(),
            marks: Default::default(),
            omitted: Default::default(),
        }
    }
}

impl FromJson for MarkedCapture {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tick: field(o, "tick", d.tick)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            scale: field(o, "scale", d.scale)?,
            png: field(o, "png", d.png)?,
            marks: field(o, "marks", d.marks)?,
            omitted: field(o, "omitted", d.omitted)?,
        })
    }
}

impl ToJson for MarkedCapture {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tick".into(), self.tick.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("scale".into(), self.scale.to_json());
        o.insert("png".into(), self.png.to_json());
        o.insert("marks".into(), self.marks.to_json());
        o.insert("omitted".into(), self.omitted.to_json());
        Value::Object(o)
    }
}

/// `RunEvent`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RunEvent {
    pub tick: u32,
    pub kind: String,
    pub what: String,
    pub detail: Value,
}

impl Default for RunEvent {
    fn default() -> Self {
        Self {
            tick: Default::default(),
            kind: Default::default(),
            what: Default::default(),
            detail: Default::default(),
        }
    }
}

impl FromJson for RunEvent {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tick: field(o, "tick", d.tick)?,
            kind: field(o, "kind", d.kind)?,
            what: field(o, "what", d.what)?,
            detail: field(o, "detail", d.detail)?,
        })
    }
}

impl ToJson for RunEvent {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tick".into(), self.tick.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("what".into(), self.what.to_json());
        o.insert("detail".into(), self.detail.to_json());
        Value::Object(o)
    }
}

/// `RunTimeline`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RunTimeline {
    pub tick: u32,
    pub screens: Vec<String>,
    pub events: Vec<RunEvent>,
    pub dropped: u32,
}

impl Default for RunTimeline {
    fn default() -> Self {
        Self {
            tick: Default::default(),
            screens: Default::default(),
            events: Default::default(),
            dropped: Default::default(),
        }
    }
}

impl FromJson for RunTimeline {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tick: field(o, "tick", d.tick)?,
            screens: field(o, "screens", d.screens)?,
            events: field(o, "events", d.events)?,
            dropped: field(o, "dropped", d.dropped)?,
        })
    }
}

impl ToJson for RunTimeline {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tick".into(), self.tick.to_json());
        o.insert("screens".into(), self.screens.to_json());
        o.insert("events".into(), self.events.to_json());
        o.insert("dropped".into(), self.dropped.to_json());
        Value::Object(o)
    }
}

/// `Heard`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Heard {
    pub tick: u32,
    pub rate: u32,
    pub frames: u32,
    pub peak: f32,
    pub rms: f32,
    pub wav: String,
}

impl Default for Heard {
    fn default() -> Self {
        Self {
            tick: Default::default(),
            rate: Default::default(),
            frames: Default::default(),
            peak: Default::default(),
            rms: Default::default(),
            wav: Default::default(),
        }
    }
}

impl FromJson for Heard {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tick: field(o, "tick", d.tick)?,
            rate: field(o, "rate", d.rate)?,
            frames: field(o, "frames", d.frames)?,
            peak: field(o, "peak", d.peak)?,
            rms: field(o, "rms", d.rms)?,
            wav: field(o, "wav", d.wav)?,
        })
    }
}

impl ToJson for Heard {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tick".into(), self.tick.to_json());
        o.insert("rate".into(), self.rate.to_json());
        o.insert("frames".into(), self.frames.to_json());
        o.insert("peak".into(), self.peak.to_json());
        o.insert("rms".into(), self.rms.to_json());
        o.insert("wav".into(), self.wav.to_json());
        Value::Object(o)
    }
}

/// `ProbeHeap`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ProbeHeap {
    pub live: f64,
    pub peak: f64,
    pub allocations: f64,
}

impl Default for ProbeHeap {
    fn default() -> Self {
        Self {
            live: Default::default(),
            peak: Default::default(),
            allocations: Default::default(),
        }
    }
}

impl FromJson for ProbeHeap {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            live: field(o, "live", d.live)?,
            peak: field(o, "peak", d.peak)?,
            allocations: field(o, "allocations", d.allocations)?,
        })
    }
}

impl ToJson for ProbeHeap {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("live".into(), self.live.to_json());
        o.insert("peak".into(), self.peak.to_json());
        o.insert("allocations".into(), self.allocations.to_json());
        Value::Object(o)
    }
}

/// `ProbeWorldMemory`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ProbeWorldMemory {
    pub entities: u32,
}

impl Default for ProbeWorldMemory {
    fn default() -> Self {
        Self {
            entities: Default::default(),
        }
    }
}

impl FromJson for ProbeWorldMemory {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            entities: field(o, "entities", d.entities)?,
        })
    }
}

impl ToJson for ProbeWorldMemory {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("entities".into(), self.entities.to_json());
        Value::Object(o)
    }
}

/// `ProbeStreamMemory`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ProbeStreamMemory {
    pub budget: f64,
    pub used: f64,
    pub peak: f64,
}

impl Default for ProbeStreamMemory {
    fn default() -> Self {
        Self {
            budget: Default::default(),
            used: Default::default(),
            peak: Default::default(),
        }
    }
}

impl FromJson for ProbeStreamMemory {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            budget: field(o, "budget", d.budget)?,
            used: field(o, "used", d.used)?,
            peak: field(o, "peak", d.peak)?,
        })
    }
}

impl ToJson for ProbeStreamMemory {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("budget".into(), self.budget.to_json());
        o.insert("used".into(), self.used.to_json());
        o.insert("peak".into(), self.peak.to_json());
        Value::Object(o)
    }
}

/// `ProbeMemory`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ProbeMemory {
    pub host: String,
    pub linear: Option<f64>,
    pub resident: Option<f64>,
    pub heap: Option<ProbeHeap>,
    pub world: ProbeWorldMemory,
    pub stream: Option<ProbeStreamMemory>,
    pub audio: Option<f64>,
    pub gpu: Option<GpuMemory>,
    pub device_losses: Option<u32>,
}

impl Default for ProbeMemory {
    fn default() -> Self {
        Self {
            host: Default::default(),
            linear: Default::default(),
            resident: Default::default(),
            heap: Default::default(),
            world: Default::default(),
            stream: Default::default(),
            audio: Default::default(),
            gpu: Default::default(),
            device_losses: Default::default(),
        }
    }
}

impl FromJson for ProbeMemory {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            host: field(o, "host", d.host)?,
            linear: field(o, "linear", d.linear)?,
            resident: field(o, "resident", d.resident)?,
            heap: field(o, "heap", d.heap)?,
            world: field(o, "world", d.world)?,
            stream: field(o, "stream", d.stream)?,
            audio: field(o, "audio", d.audio)?,
            gpu: field(o, "gpu", d.gpu)?,
            device_losses: field(o, "deviceLosses", d.device_losses)?,
        })
    }
}

impl ToJson for ProbeMemory {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("host".into(), self.host.to_json());
        o.insert("linear".into(), self.linear.to_json());
        o.insert("resident".into(), self.resident.to_json());
        o.insert("heap".into(), self.heap.to_json());
        o.insert("world".into(), self.world.to_json());
        o.insert("stream".into(), self.stream.to_json());
        o.insert("audio".into(), self.audio.to_json());
        o.insert("gpu".into(), self.gpu.to_json());
        o.insert("deviceLosses".into(), self.device_losses.to_json());
        Value::Object(o)
    }
}

/// `SweepDesc`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SweepDesc {
    pub id: String,
    pub parameter: String,
    pub values: Vec<Value>,
    pub steps: u32,
    pub width: u32,
    pub height: u32,
}

impl Default for SweepDesc {
    fn default() -> Self {
        Self {
            id: Default::default(),
            parameter: Default::default(),
            values: Default::default(),
            steps: Default::default(),
            width: Default::default(),
            height: Default::default(),
        }
    }
}

impl FromJson for SweepDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            id: field(o, "id", d.id)?,
            parameter: field(o, "parameter", d.parameter)?,
            values: field(o, "values", d.values)?,
            steps: field(o, "steps", d.steps)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
        })
    }
}

impl ToJson for SweepDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("id".into(), self.id.to_json());
        o.insert("parameter".into(), self.parameter.to_json());
        o.insert("values".into(), self.values.to_json());
        o.insert("steps".into(), self.steps.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        Value::Object(o)
    }
}

/// `SweepFrame`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SweepFrame {
    pub value: Value,
    pub tick: u32,
    pub png: String,
}

impl Default for SweepFrame {
    fn default() -> Self {
        Self {
            value: Default::default(),
            tick: Default::default(),
            png: Default::default(),
        }
    }
}

impl FromJson for SweepFrame {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            value: field(o, "value", d.value)?,
            tick: field(o, "tick", d.tick)?,
            png: field(o, "png", d.png)?,
        })
    }
}

impl ToJson for SweepFrame {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("value".into(), self.value.to_json());
        o.insert("tick".into(), self.tick.to_json());
        o.insert("png".into(), self.png.to_json());
        Value::Object(o)
    }
}

/// `SweepResult`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SweepResult {
    pub frames: Vec<SweepFrame>,
}

impl Default for SweepResult {
    fn default() -> Self {
        Self {
            frames: Default::default(),
        }
    }
}

impl FromJson for SweepResult {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            frames: field(o, "frames", d.frames)?,
        })
    }
}

impl ToJson for SweepResult {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("frames".into(), self.frames.to_json());
        Value::Object(o)
    }
}

/// `ReloadReport`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ReloadReport {
    pub tick: u32,
    pub ms: f32,
    pub migrated: Vec<String>,
    pub dropped: Vec<String>,
    pub warnings: Vec<String>,
}

impl Default for ReloadReport {
    fn default() -> Self {
        Self {
            tick: Default::default(),
            ms: Default::default(),
            migrated: Default::default(),
            dropped: Default::default(),
            warnings: Default::default(),
        }
    }
}

impl FromJson for ReloadReport {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tick: field(o, "tick", d.tick)?,
            ms: field(o, "ms", d.ms)?,
            migrated: field(o, "migrated", d.migrated)?,
            dropped: field(o, "dropped", d.dropped)?,
            warnings: field(o, "warnings", d.warnings)?,
        })
    }
}

impl ToJson for ReloadReport {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tick".into(), self.tick.to_json());
        o.insert("ms".into(), self.ms.to_json());
        o.insert("migrated".into(), self.migrated.to_json());
        o.insert("dropped".into(), self.dropped.to_json());
        o.insert("warnings".into(), self.warnings.to_json());
        Value::Object(o)
    }
}

/// `SnapshotInfo`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotInfo {
    pub id: String,
    pub tick: u32,
    pub hash: String,
    pub bytes: u32,
    pub presentation: String,
}

impl Default for SnapshotInfo {
    fn default() -> Self {
        Self {
            id: Default::default(),
            tick: Default::default(),
            hash: Default::default(),
            bytes: Default::default(),
            presentation: Default::default(),
        }
    }
}

impl FromJson for SnapshotInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            id: field(o, "id", d.id)?,
            tick: field(o, "tick", d.tick)?,
            hash: field(o, "hash", d.hash)?,
            bytes: field(o, "bytes", d.bytes)?,
            presentation: field(o, "presentation", d.presentation)?,
        })
    }
}

impl ToJson for SnapshotInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("id".into(), self.id.to_json());
        o.insert("tick".into(), self.tick.to_json());
        o.insert("hash".into(), self.hash.to_json());
        o.insert("bytes".into(), self.bytes.to_json());
        o.insert("presentation".into(), self.presentation.to_json());
        Value::Object(o)
    }
}

/// `DumpKind`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DumpKind {
    #[default]
    World,
    Entity,
    Ui,
    Desync,
}

impl DumpKind {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["world", "entity", "ui", "desync"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::World, Self::Entity, Self::Ui, Self::Desync];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("DumpKind", s, Self::NAMES)),
        }
    }
}

impl FromJson for DumpKind {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "DumpKind", Self::NAMES)?)
    }
}

impl ToJson for DumpKind {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `DumpDesc`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DumpDesc {
    pub what: DumpKind,
    pub id: String,
}

impl Default for DumpDesc {
    fn default() -> Self {
        Self {
            what: Default::default(),
            id: Default::default(),
        }
    }
}

impl FromJson for DumpDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            what: field(o, "what", d.what)?,
            id: field(o, "id", d.id)?,
        })
    }
}

impl ToJson for DumpDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("what".into(), self.what.to_json());
        o.insert("id".into(), self.id.to_json());
        Value::Object(o)
    }
}

/// `ArchetypeDump`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ArchetypeDump {
    pub components: Vec<String>,
    pub count: u32,
}

impl Default for ArchetypeDump {
    fn default() -> Self {
        Self {
            components: Default::default(),
            count: Default::default(),
        }
    }
}

impl FromJson for ArchetypeDump {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            components: field(o, "components", d.components)?,
            count: field(o, "count", d.count)?,
        })
    }
}

impl ToJson for ArchetypeDump {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("components".into(), self.components.to_json());
        o.insert("count".into(), self.count.to_json());
        Value::Object(o)
    }
}

/// `SystemDump`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SystemDump {
    pub name: String,
    pub matched: Option<u32>,
    pub reads: Option<Vec<String>>,
    pub writes: Option<Vec<String>>,
    pub ms: Option<f32>,
}

impl Default for SystemDump {
    fn default() -> Self {
        Self {
            name: Default::default(),
            matched: Default::default(),
            reads: Default::default(),
            writes: Default::default(),
            ms: Default::default(),
        }
    }
}

impl FromJson for SystemDump {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            matched: field(o, "matched", d.matched)?,
            reads: field(o, "reads", d.reads)?,
            writes: field(o, "writes", d.writes)?,
            ms: field(o, "ms", d.ms)?,
        })
    }
}

impl ToJson for SystemDump {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("matched".into(), self.matched.to_json());
        o.insert("reads".into(), self.reads.to_json());
        o.insert("writes".into(), self.writes.to_json());
        o.insert("ms".into(), self.ms.to_json());
        Value::Object(o)
    }
}

/// `WorldDump`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldDump {
    pub tick: u32,
    pub hash: String,
    pub entities: u32,
    pub archetypes: Vec<ArchetypeDump>,
    pub systems: Vec<SystemDump>,
    pub resources: Vec<String>,
}

impl Default for WorldDump {
    fn default() -> Self {
        Self {
            tick: Default::default(),
            hash: Default::default(),
            entities: Default::default(),
            archetypes: Default::default(),
            systems: Default::default(),
            resources: Default::default(),
        }
    }
}

impl FromJson for WorldDump {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tick: field(o, "tick", d.tick)?,
            hash: field(o, "hash", d.hash)?,
            entities: field(o, "entities", d.entities)?,
            archetypes: field(o, "archetypes", d.archetypes)?,
            systems: field(o, "systems", d.systems)?,
            resources: field(o, "resources", d.resources)?,
        })
    }
}

impl ToJson for WorldDump {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tick".into(), self.tick.to_json());
        o.insert("hash".into(), self.hash.to_json());
        o.insert("entities".into(), self.entities.to_json());
        o.insert("archetypes".into(), self.archetypes.to_json());
        o.insert("systems".into(), self.systems.to_json());
        o.insert("resources".into(), self.resources.to_json());
        Value::Object(o)
    }
}

/// `EntityDump`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityDump {
    pub id: String,
    pub name: Option<String>,
    pub components: BTreeMap<String, Value>,
    pub prefab: Option<EntityPrefab>,
}

impl Default for EntityDump {
    fn default() -> Self {
        Self {
            id: Default::default(),
            name: Default::default(),
            components: Default::default(),
            prefab: Default::default(),
        }
    }
}

impl FromJson for EntityDump {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            id: field(o, "id", d.id)?,
            name: field(o, "name", d.name)?,
            components: field(o, "components", d.components)?,
            prefab: field(o, "prefab", d.prefab)?,
        })
    }
}

impl ToJson for EntityDump {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("id".into(), self.id.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("components".into(), self.components.to_json());
        if let Some(x) = &self.prefab {
            o.insert("prefab".into(), x.to_json());
        }
        Value::Object(o)
    }
}

/// `EntityPrefab`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityPrefab {
    pub prefab: String,
    pub node: String,
    pub root: String,
    pub overrides: Vec<String>,
}

impl Default for EntityPrefab {
    fn default() -> Self {
        Self {
            prefab: Default::default(),
            node: Default::default(),
            root: Default::default(),
            overrides: Default::default(),
        }
    }
}

impl FromJson for EntityPrefab {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            prefab: field(o, "prefab", d.prefab)?,
            node: field(o, "node", d.node)?,
            root: field(o, "root", d.root)?,
            overrides: field(o, "overrides", d.overrides)?,
        })
    }
}

impl ToJson for EntityPrefab {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("prefab".into(), self.prefab.to_json());
        o.insert("node".into(), self.node.to_json());
        o.insert("root".into(), self.root.to_json());
        o.insert("overrides".into(), self.overrides.to_json());
        Value::Object(o)
    }
}

/// `DescribedPrefab`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DescribedPrefab {
    pub prefab: String,
    pub instances: Vec<String>,
}

impl Default for DescribedPrefab {
    fn default() -> Self {
        Self {
            prefab: Default::default(),
            instances: Default::default(),
        }
    }
}

impl FromJson for DescribedPrefab {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            prefab: field(o, "prefab", d.prefab)?,
            instances: field(o, "instances", d.instances)?,
        })
    }
}

impl ToJson for DescribedPrefab {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("prefab".into(), self.prefab.to_json());
        o.insert("instances".into(), self.instances.to_json());
        Value::Object(o)
    }
}

/// `UiElement`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiElement {
    pub id: String,
    pub parent: String,
    pub kind: String,
    pub disabled: bool,
    pub clipped: bool,
    pub text: String,
    pub role: String,
    pub name: String,
    pub value: String,
    pub states: Vec<String>,
    pub opaque: bool,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for UiElement {
    fn default() -> Self {
        Self {
            id: Default::default(),
            parent: Default::default(),
            kind: Default::default(),
            disabled: Default::default(),
            clipped: Default::default(),
            text: Default::default(),
            role: Default::default(),
            name: Default::default(),
            value: Default::default(),
            states: Default::default(),
            opaque: Default::default(),
            x: Default::default(),
            y: Default::default(),
            width: Default::default(),
            height: Default::default(),
        }
    }
}

impl FromJson for UiElement {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            id: field(o, "id", d.id)?,
            parent: field(o, "parent", d.parent)?,
            kind: field(o, "kind", d.kind)?,
            disabled: field(o, "disabled", d.disabled)?,
            clipped: field(o, "clipped", d.clipped)?,
            text: field(o, "text", d.text)?,
            role: field(o, "role", d.role)?,
            name: field(o, "name", d.name)?,
            value: field(o, "value", d.value)?,
            states: field(o, "states", d.states)?,
            opaque: field(o, "opaque", d.opaque)?,
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
        })
    }
}

impl ToJson for UiElement {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("id".into(), self.id.to_json());
        o.insert("parent".into(), self.parent.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("disabled".into(), self.disabled.to_json());
        o.insert("clipped".into(), self.clipped.to_json());
        o.insert("text".into(), self.text.to_json());
        o.insert("role".into(), self.role.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("value".into(), self.value.to_json());
        o.insert("states".into(), self.states.to_json());
        o.insert("opaque".into(), self.opaque.to_json());
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        Value::Object(o)
    }
}

/// `UiSurface`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiSurface {
    pub width: f32,
    pub height: f32,
    pub dpr: f32,
}

impl Default for UiSurface {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
            dpr: 1.0_f32,
        }
    }
}

impl FromJson for UiSurface {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            dpr: field(o, "dpr", d.dpr)?,
        })
    }
}

impl ToJson for UiSurface {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("dpr".into(), self.dpr.to_json());
        Value::Object(o)
    }
}

/// `UiDump`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct UiDump {
    pub tick: u32,
    pub surface: Option<UiSurface>,
    pub elements: Vec<UiElement>,
}

impl Default for UiDump {
    fn default() -> Self {
        Self {
            tick: Default::default(),
            surface: Default::default(),
            elements: Default::default(),
        }
    }
}

impl FromJson for UiDump {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tick: field(o, "tick", d.tick)?,
            surface: field(o, "surface", d.surface)?,
            elements: field(o, "elements", d.elements)?,
        })
    }
}

impl ToJson for UiDump {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tick".into(), self.tick.to_json());
        o.insert("surface".into(), self.surface.to_json());
        o.insert("elements".into(), self.elements.to_json());
        Value::Object(o)
    }
}

/// `DesyncDump`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DesyncDump {
    pub desync: bool,
    pub tick: Option<u32>,
    pub local: Value,
    pub remote: Value,
    pub diff: Value,
}

impl Default for DesyncDump {
    fn default() -> Self {
        Self {
            desync: Default::default(),
            tick: Default::default(),
            local: Default::default(),
            remote: Default::default(),
            diff: Default::default(),
        }
    }
}

impl FromJson for DesyncDump {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            desync: field(o, "desync", d.desync)?,
            tick: field(o, "tick", d.tick)?,
            local: field(o, "local", d.local)?,
            remote: field(o, "remote", d.remote)?,
            diff: field(o, "diff", d.diff)?,
        })
    }
}

impl ToJson for DesyncDump {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("desync".into(), self.desync.to_json());
        o.insert("tick".into(), self.tick.to_json());
        o.insert("local".into(), self.local.to_json());
        o.insert("remote".into(), self.remote.to_json());
        o.insert("diff".into(), self.diff.to_json());
        Value::Object(o)
    }
}

/// `DumpResult`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub enum DumpResult {
    World(WorldDump),
    Entity(EntityDump),
    Ui(UiDump),
    Desync(DesyncDump),
}

impl Default for DumpResult {
    fn default() -> Self {
        Self::World(Default::default())
    }
}

impl DumpResult {
    /// The tag key and the cases' tags, in order.
    pub const TAG: &'static str = "kind";
    pub const TAGS: &'static [&'static str] = &["world", "entity", "ui", "desync"];
    /// This case's tag.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::World(_) => "world",
            Self::Entity(_) => "entity",
            Self::Ui(_) => "ui",
            Self::Desync(_) => "desync",
        }
    }
}

impl FromJson for DumpResult {
    fn from_json(v: &Value) -> Result<Self, String> {
        let o = object(v)?;
        let tag = match o.get(Self::TAG) {
            Some(t) => text(t).map_err(|e| at(Self::TAG, e))?,
            None => return Err(at(Self::TAG, format!("missing: one of {}", Self::TAGS.join(", ")))),
        };
        Ok(match tag {
            "world" => Self::World(WorldDump::from_json(v)?),
            "entity" => Self::Entity(EntityDump::from_json(v)?),
            "ui" => Self::Ui(UiDump::from_json(v)?),
            "desync" => Self::Desync(DesyncDump::from_json(v)?),
            t => return Err(at(Self::TAG, unknown_variant("DumpResult", t, Self::TAGS))),
        })
    }
}

impl ToJson for DumpResult {
    fn to_json(&self) -> Value {
        let mut v = match self {
            Self::World(p) => p.to_json(),
            Self::Entity(p) => p.to_json(),
            Self::Ui(p) => p.to_json(),
            Self::Desync(p) => p.to_json(),
        };
        if let Value::Object(o) = &mut v {
            o.insert(Self::TAG.into(), Value::String(self.tag().into()));
        }
        v
    }
}

/// `ScreenBox`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ScreenBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for ScreenBox {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            width: Default::default(),
            height: Default::default(),
        }
    }
}

impl FromJson for ScreenBox {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
        })
    }
}

impl ToJson for ScreenBox {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        Value::Object(o)
    }
}

/// `ScreenPoint`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ScreenPoint {
    pub x: f32,
    pub y: f32,
}

impl Default for ScreenPoint {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
        }
    }
}

impl FromJson for ScreenPoint {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
        })
    }
}

impl ToJson for ScreenPoint {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        Value::Object(o)
    }
}

/// `DescribedEntity`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DescribedEntity {
    pub id: String,
    pub name: Option<String>,
    pub order: u32,
    pub archetype: Vec<String>,
    pub labels: Vec<String>,
    pub r#box: ScreenBox,
    pub drawn: ScreenBox,
    pub point: ScreenPoint,
    pub depth: f32,
    pub coverage: f32,
    pub occlusion: f32,
    pub views: Vec<String>,
}

impl Default for DescribedEntity {
    fn default() -> Self {
        Self {
            id: Default::default(),
            name: Default::default(),
            order: Default::default(),
            archetype: Default::default(),
            labels: Default::default(),
            r#box: Default::default(),
            drawn: Default::default(),
            point: Default::default(),
            depth: Default::default(),
            coverage: Default::default(),
            occlusion: Default::default(),
            views: Default::default(),
        }
    }
}

impl FromJson for DescribedEntity {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            id: field(o, "id", d.id)?,
            name: field(o, "name", d.name)?,
            order: field(o, "order", d.order)?,
            archetype: field(o, "archetype", d.archetype)?,
            labels: field(o, "labels", d.labels)?,
            r#box: field(o, "box", d.r#box)?,
            drawn: field(o, "drawn", d.drawn)?,
            point: field(o, "point", d.point)?,
            depth: field(o, "depth", d.depth)?,
            coverage: field(o, "coverage", d.coverage)?,
            occlusion: field(o, "occlusion", d.occlusion)?,
            views: field(o, "views", d.views)?,
        })
    }
}

impl ToJson for DescribedEntity {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("id".into(), self.id.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("order".into(), self.order.to_json());
        o.insert("archetype".into(), self.archetype.to_json());
        o.insert("labels".into(), self.labels.to_json());
        o.insert("box".into(), self.r#box.to_json());
        o.insert("drawn".into(), self.drawn.to_json());
        o.insert("point".into(), self.point.to_json());
        o.insert("depth".into(), self.depth.to_json());
        o.insert("coverage".into(), self.coverage.to_json());
        o.insert("occlusion".into(), self.occlusion.to_json());
        o.insert("views".into(), self.views.to_json());
        Value::Object(o)
    }
}

/// `DescribedView`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DescribedView {
    pub id: String,
    pub name: Option<String>,
    pub kind: String,
    pub size: PassSize,
    pub entities: Vec<DescribedEntity>,
}

impl Default for DescribedView {
    fn default() -> Self {
        Self {
            id: Default::default(),
            name: Default::default(),
            kind: Default::default(),
            size: Default::default(),
            entities: Default::default(),
        }
    }
}

impl FromJson for DescribedView {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            id: field(o, "id", d.id)?,
            name: field(o, "name", d.name)?,
            kind: field(o, "kind", d.kind)?,
            size: field(o, "size", d.size)?,
            entities: field(o, "entities", d.entities)?,
        })
    }
}

impl ToJson for DescribedView {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("id".into(), self.id.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("size".into(), self.size.to_json());
        o.insert("entities".into(), self.entities.to_json());
        Value::Object(o)
    }
}

/// `DescribedUi`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DescribedUi {
    pub id: String,
    pub parent: String,
    pub kind: String,
    pub role: String,
    pub text: String,
    pub value: String,
    pub states: Vec<String>,
    pub opaque: bool,
    pub r#box: ScreenBox,
}

impl Default for DescribedUi {
    fn default() -> Self {
        Self {
            id: Default::default(),
            parent: Default::default(),
            kind: Default::default(),
            role: Default::default(),
            text: Default::default(),
            value: Default::default(),
            states: Default::default(),
            opaque: Default::default(),
            r#box: Default::default(),
        }
    }
}

impl FromJson for DescribedUi {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            id: field(o, "id", d.id)?,
            parent: field(o, "parent", d.parent)?,
            kind: field(o, "kind", d.kind)?,
            role: field(o, "role", d.role)?,
            text: field(o, "text", d.text)?,
            value: field(o, "value", d.value)?,
            states: field(o, "states", d.states)?,
            opaque: field(o, "opaque", d.opaque)?,
            r#box: field(o, "box", d.r#box)?,
        })
    }
}

impl ToJson for DescribedUi {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("id".into(), self.id.to_json());
        o.insert("parent".into(), self.parent.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("role".into(), self.role.to_json());
        o.insert("text".into(), self.text.to_json());
        o.insert("value".into(), self.value.to_json());
        o.insert("states".into(), self.states.to_json());
        o.insert("opaque".into(), self.opaque.to_json());
        o.insert("box".into(), self.r#box.to_json());
        Value::Object(o)
    }
}

/// `DescribedCamera`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DescribedCamera {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
    pub projection: String,
    pub fov: Option<f32>,
    pub height: Option<f32>,
    pub near: f32,
    pub far: f32,
}

impl Default for DescribedCamera {
    fn default() -> Self {
        Self {
            position: Default::default(),
            target: Default::default(),
            up: Default::default(),
            projection: Default::default(),
            fov: Default::default(),
            height: Default::default(),
            near: Default::default(),
            far: Default::default(),
        }
    }
}

impl FromJson for DescribedCamera {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            position: field(o, "position", d.position)?,
            target: field(o, "target", d.target)?,
            up: field(o, "up", d.up)?,
            projection: field(o, "projection", d.projection)?,
            fov: field(o, "fov", d.fov)?,
            height: field(o, "height", d.height)?,
            near: field(o, "near", d.near)?,
            far: field(o, "far", d.far)?,
        })
    }
}

impl ToJson for DescribedCamera {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("position".into(), self.position.to_json());
        o.insert("target".into(), self.target.to_json());
        o.insert("up".into(), self.up.to_json());
        o.insert("projection".into(), self.projection.to_json());
        o.insert("fov".into(), self.fov.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("near".into(), self.near.to_json());
        o.insert("far".into(), self.far.to_json());
        Value::Object(o)
    }
}

/// `PassSize`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PassSize {
    pub width: u32,
    pub height: u32,
}

impl Default for PassSize {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
        }
    }
}

impl FromJson for PassSize {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
        })
    }
}

impl ToJson for PassSize {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        Value::Object(o)
    }
}

/// `Description`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Description {
    pub tick: u32,
    pub surface: Option<UiSurface>,
    pub pass: Option<PassSize>,
    pub camera: Option<DescribedCamera>,
    pub entities: Vec<DescribedEntity>,
    pub views: Vec<DescribedView>,
    pub ui: Vec<DescribedUi>,
    pub prefabs: Option<Vec<DescribedPrefab>>,
}

impl Default for Description {
    fn default() -> Self {
        Self {
            tick: Default::default(),
            surface: Default::default(),
            pass: Default::default(),
            camera: Default::default(),
            entities: Default::default(),
            views: Default::default(),
            ui: Default::default(),
            prefabs: Default::default(),
        }
    }
}

impl FromJson for Description {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tick: field(o, "tick", d.tick)?,
            surface: field(o, "surface", d.surface)?,
            pass: field(o, "pass", d.pass)?,
            camera: field(o, "camera", d.camera)?,
            entities: field(o, "entities", d.entities)?,
            views: field(o, "views", d.views)?,
            ui: field(o, "ui", d.ui)?,
            prefabs: field(o, "prefabs", d.prefabs)?,
        })
    }
}

impl ToJson for Description {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tick".into(), self.tick.to_json());
        o.insert("surface".into(), self.surface.to_json());
        o.insert("pass".into(), self.pass.to_json());
        o.insert("camera".into(), self.camera.to_json());
        o.insert("entities".into(), self.entities.to_json());
        o.insert("views".into(), self.views.to_json());
        o.insert("ui".into(), self.ui.to_json());
        if let Some(x) = &self.prefabs {
            o.insert("prefabs".into(), x.to_json());
        }
        Value::Object(o)
    }
}

/// `ExpectStatus`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ExpectStatus {
    #[default]
    Pass,
    Fail,
    Unchecked,
}

impl ExpectStatus {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["pass", "fail", "unchecked"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Pass, Self::Fail, Self::Unchecked];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("ExpectStatus", s, Self::NAMES)),
        }
    }
}

impl FromJson for ExpectStatus {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "ExpectStatus", Self::NAMES)?)
    }
}

impl ToJson for ExpectStatus {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `ExpectResult`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ExpectResult {
    pub tick: u32,
    pub at: Option<f32>,
    pub expect: String,
    pub kind: String,
    pub status: ExpectStatus,
    pub expected: Value,
    pub observed: Value,
    pub reason: String,
}

impl Default for ExpectResult {
    fn default() -> Self {
        Self {
            tick: Default::default(),
            at: Default::default(),
            expect: Default::default(),
            kind: Default::default(),
            status: Default::default(),
            expected: Default::default(),
            observed: Default::default(),
            reason: Default::default(),
        }
    }
}

impl FromJson for ExpectResult {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tick: field(o, "tick", d.tick)?,
            at: field(o, "at", d.at)?,
            expect: field(o, "expect", d.expect)?,
            kind: field(o, "kind", d.kind)?,
            status: field(o, "status", d.status)?,
            expected: field(o, "expected", d.expected)?,
            observed: field(o, "observed", d.observed)?,
            reason: field(o, "reason", d.reason)?,
        })
    }
}

impl ToJson for ExpectResult {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tick".into(), self.tick.to_json());
        o.insert("at".into(), self.at.to_json());
        o.insert("expect".into(), self.expect.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("status".into(), self.status.to_json());
        o.insert("expected".into(), self.expected.to_json());
        o.insert("observed".into(), self.observed.to_json());
        o.insert("reason".into(), self.reason.to_json());
        Value::Object(o)
    }
}

/// `ExpectReport`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ExpectReport {
    pub passed: u32,
    pub failed: u32,
    pub unchecked: u32,
    pub results: Vec<ExpectResult>,
}

impl Default for ExpectReport {
    fn default() -> Self {
        Self {
            passed: Default::default(),
            failed: Default::default(),
            unchecked: Default::default(),
            results: Default::default(),
        }
    }
}

impl FromJson for ExpectReport {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            passed: field(o, "passed", d.passed)?,
            failed: field(o, "failed", d.failed)?,
            unchecked: field(o, "unchecked", d.unchecked)?,
            results: field(o, "results", d.results)?,
        })
    }
}

impl ToJson for ExpectReport {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("passed".into(), self.passed.to_json());
        o.insert("failed".into(), self.failed.to_json());
        o.insert("unchecked".into(), self.unchecked.to_json());
        o.insert("results".into(), self.results.to_json());
        Value::Object(o)
    }
}

/// `SavedScene`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SavedScene {
    pub tick: u32,
    pub source: String,
    pub written: String,
    pub lines: Vec<u32>,
    pub skipped: Vec<String>,
    pub text: String,
    pub spawned: u32,
    pub scenes: Vec<SavedSceneFile>,
}

impl Default for SavedScene {
    fn default() -> Self {
        Self {
            tick: Default::default(),
            source: Default::default(),
            written: Default::default(),
            lines: Default::default(),
            skipped: Default::default(),
            text: Default::default(),
            spawned: Default::default(),
            scenes: Default::default(),
        }
    }
}

impl FromJson for SavedScene {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            tick: field(o, "tick", d.tick)?,
            source: field(o, "source", d.source)?,
            written: field(o, "written", d.written)?,
            lines: field(o, "lines", d.lines)?,
            skipped: field(o, "skipped", d.skipped)?,
            text: field(o, "text", d.text)?,
            spawned: field(o, "spawned", d.spawned)?,
            scenes: field(o, "scenes", d.scenes)?,
        })
    }
}

impl ToJson for SavedScene {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("tick".into(), self.tick.to_json());
        o.insert("source".into(), self.source.to_json());
        o.insert("written".into(), self.written.to_json());
        o.insert("lines".into(), self.lines.to_json());
        o.insert("skipped".into(), self.skipped.to_json());
        o.insert("text".into(), self.text.to_json());
        o.insert("spawned".into(), self.spawned.to_json());
        o.insert("scenes".into(), self.scenes.to_json());
        Value::Object(o)
    }
}

/// `SavedSceneFile`: packages/knowledge/wiki/probe.md.
#[derive(Clone, Debug, PartialEq)]
pub struct SavedSceneFile {
    pub source: String,
    pub written: String,
    pub lines: Vec<u32>,
    pub text: String,
}

impl Default for SavedSceneFile {
    fn default() -> Self {
        Self {
            source: Default::default(),
            written: Default::default(),
            lines: Default::default(),
            text: Default::default(),
        }
    }
}

impl FromJson for SavedSceneFile {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            source: field(o, "source", d.source)?,
            written: field(o, "written", d.written)?,
            lines: field(o, "lines", d.lines)?,
            text: field(o, "text", d.text)?,
        })
    }
}

impl ToJson for SavedSceneFile {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("source".into(), self.source.to_json());
        o.insert("written".into(), self.written.to_json());
        o.insert("lines".into(), self.lines.to_json());
        o.insert("text".into(), self.text.to_json());
        Value::Object(o)
    }
}

/// `TerrainBrushMode`: packages/knowledge/wiki/terrain.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TerrainBrushMode {
    #[default]
    Raise,
    Lower,
    Flatten,
    Smooth,
    Paint,
    Hole,
    Fill,
}

impl TerrainBrushMode {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["raise", "lower", "flatten", "smooth", "paint", "hole", "fill"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Raise, Self::Lower, Self::Flatten, Self::Smooth, Self::Paint, Self::Hole, Self::Fill];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("TerrainBrushMode", s, Self::NAMES)),
        }
    }
}

impl FromJson for TerrainBrushMode {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "TerrainBrushMode", Self::NAMES)?)
    }
}

impl ToJson for TerrainBrushMode {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `TerrainStampMode`: packages/knowledge/wiki/terrain.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TerrainStampMode {
    #[default]
    Carve,
    Hole,
}

impl TerrainStampMode {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["carve", "hole"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Carve, Self::Hole];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("TerrainStampMode", s, Self::NAMES)),
        }
    }
}

impl FromJson for TerrainStampMode {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "TerrainStampMode", Self::NAMES)?)
    }
}

impl ToJson for TerrainStampMode {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `TerrainInfo`: packages/knowledge/wiki/terrain.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainInfo {
    pub data: u64,
    pub geometry: u64,
    pub size: Vec2,
    pub columns: u32,
    pub rows: u32,
    pub min_height: f32,
    pub max_height: f32,
    pub layers: u32,
    pub collision: bool,
    pub cast_shadows: bool,
    pub holes: bool,
    pub sea_level: Option<f32>,
}

impl Default for TerrainInfo {
    fn default() -> Self {
        Self {
            data: Default::default(),
            geometry: Default::default(),
            size: Default::default(),
            columns: Default::default(),
            rows: Default::default(),
            min_height: Default::default(),
            max_height: Default::default(),
            layers: Default::default(),
            collision: Default::default(),
            cast_shadows: Default::default(),
            holes: Default::default(),
            sea_level: Default::default(),
        }
    }
}

impl FromJson for TerrainInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            data: field(o, "data", d.data)?,
            geometry: field(o, "geometry", d.geometry)?,
            size: field(o, "size", d.size)?,
            columns: field(o, "columns", d.columns)?,
            rows: field(o, "rows", d.rows)?,
            min_height: field(o, "minHeight", d.min_height)?,
            max_height: field(o, "maxHeight", d.max_height)?,
            layers: field(o, "layers", d.layers)?,
            collision: field(o, "collision", d.collision)?,
            cast_shadows: field(o, "castShadows", d.cast_shadows)?,
            holes: field(o, "holes", d.holes)?,
            sea_level: field(o, "seaLevel", d.sea_level)?,
        })
    }
}

impl ToJson for TerrainInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("data".into(), self.data.to_json());
        o.insert("geometry".into(), self.geometry.to_json());
        o.insert("size".into(), self.size.to_json());
        o.insert("columns".into(), self.columns.to_json());
        o.insert("rows".into(), self.rows.to_json());
        o.insert("minHeight".into(), self.min_height.to_json());
        o.insert("maxHeight".into(), self.max_height.to_json());
        o.insert("layers".into(), self.layers.to_json());
        o.insert("collision".into(), self.collision.to_json());
        o.insert("castShadows".into(), self.cast_shadows.to_json());
        o.insert("holes".into(), self.holes.to_json());
        o.insert("seaLevel".into(), self.sea_level.to_json());
        Value::Object(o)
    }
}

/// `TerrainSampleDesc`: packages/knowledge/wiki/terrain.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainSampleDesc {
    pub data: u64,
    pub origin: Vec3,
    pub quaternion: Quat,
    pub scale: Vec3,
    pub x: f32,
    pub z: f32,
}

impl Default for TerrainSampleDesc {
    fn default() -> Self {
        Self {
            data: Default::default(),
            origin: Default::default(),
            quaternion: Default::default(),
            scale: Vec3::new(1.0_f32, 1.0_f32, 1.0_f32),
            x: Default::default(),
            z: Default::default(),
        }
    }
}

impl FromJson for TerrainSampleDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            data: field(o, "data", d.data)?,
            origin: field(o, "origin", d.origin)?,
            quaternion: field(o, "quaternion", d.quaternion)?,
            scale: field(o, "scale", d.scale)?,
            x: field(o, "x", d.x)?,
            z: field(o, "z", d.z)?,
        })
    }
}

impl ToJson for TerrainSampleDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("data".into(), self.data.to_json());
        o.insert("origin".into(), self.origin.to_json());
        o.insert("quaternion".into(), self.quaternion.to_json());
        o.insert("scale".into(), self.scale.to_json());
        o.insert("x".into(), self.x.to_json());
        o.insert("z".into(), self.z.to_json());
        Value::Object(o)
    }
}

/// `TerrainSample`: packages/knowledge/wiki/terrain.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainSample {
    pub height: f32,
    pub normal: Vec3,
    pub slope: f32,
    pub inside: bool,
    pub hole: bool,
}

impl Default for TerrainSample {
    fn default() -> Self {
        Self {
            height: Default::default(),
            normal: Default::default(),
            slope: Default::default(),
            inside: Default::default(),
            hole: Default::default(),
        }
    }
}

impl FromJson for TerrainSample {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            height: field(o, "height", d.height)?,
            normal: field(o, "normal", d.normal)?,
            slope: field(o, "slope", d.slope)?,
            inside: field(o, "inside", d.inside)?,
            hole: field(o, "hole", d.hole)?,
        })
    }
}

impl ToJson for TerrainSample {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("height".into(), self.height.to_json());
        o.insert("normal".into(), self.normal.to_json());
        o.insert("slope".into(), self.slope.to_json());
        o.insert("inside".into(), self.inside.to_json());
        o.insert("hole".into(), self.hole.to_json());
        Value::Object(o)
    }
}

/// `TerrainBounds`: packages/knowledge/wiki/terrain.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainBounds {
    pub min: Vec3,
    pub max: Vec3,
}

impl Default for TerrainBounds {
    fn default() -> Self {
        Self {
            min: Default::default(),
            max: Default::default(),
        }
    }
}

impl FromJson for TerrainBounds {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            min: field(o, "min", d.min)?,
            max: field(o, "max", d.max)?,
        })
    }
}

impl ToJson for TerrainBounds {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("min".into(), self.min.to_json());
        o.insert("max".into(), self.max.to_json());
        Value::Object(o)
    }
}

/// `TerrainBrush`: packages/knowledge/wiki/terrain.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainBrush {
    pub mode: TerrainBrushMode,
    pub origin: Vec3,
    pub quaternion: Quat,
    pub scale: Vec3,
    pub center: Vec3,
    pub radius: f32,
    pub strength: f32,
    pub falloff: f32,
    pub layer: u32,
}

impl Default for TerrainBrush {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            origin: Default::default(),
            quaternion: Default::default(),
            scale: Vec3::new(1.0_f32, 1.0_f32, 1.0_f32),
            center: Default::default(),
            radius: 8.0_f32,
            strength: 1.0_f32,
            falloff: 0.5_f32,
            layer: Default::default(),
        }
    }
}

impl FromJson for TerrainBrush {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            mode: field(o, "mode", d.mode)?,
            origin: field(o, "origin", d.origin)?,
            quaternion: field(o, "quaternion", d.quaternion)?,
            scale: field(o, "scale", d.scale)?,
            center: field(o, "center", d.center)?,
            radius: field(o, "radius", d.radius)?,
            strength: field(o, "strength", d.strength)?,
            falloff: field(o, "falloff", d.falloff)?,
            layer: field(o, "layer", d.layer)?,
        })
    }
}

impl ToJson for TerrainBrush {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("mode".into(), self.mode.to_json());
        o.insert("origin".into(), self.origin.to_json());
        o.insert("quaternion".into(), self.quaternion.to_json());
        o.insert("scale".into(), self.scale.to_json());
        o.insert("center".into(), self.center.to_json());
        o.insert("radius".into(), self.radius.to_json());
        o.insert("strength".into(), self.strength.to_json());
        o.insert("falloff".into(), self.falloff.to_json());
        o.insert("layer".into(), self.layer.to_json());
        Value::Object(o)
    }
}

/// `TerrainEdit`: packages/knowledge/wiki/terrain.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainEdit {
    pub samples: u32,
    pub min_height: f32,
    pub max_height: f32,
    pub stamp: u32,
}

impl Default for TerrainEdit {
    fn default() -> Self {
        Self {
            samples: Default::default(),
            min_height: Default::default(),
            max_height: Default::default(),
            stamp: Default::default(),
        }
    }
}

impl FromJson for TerrainEdit {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            samples: field(o, "samples", d.samples)?,
            min_height: field(o, "minHeight", d.min_height)?,
            max_height: field(o, "maxHeight", d.max_height)?,
            stamp: field(o, "stamp", d.stamp)?,
        })
    }
}

impl ToJson for TerrainEdit {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("samples".into(), self.samples.to_json());
        o.insert("minHeight".into(), self.min_height.to_json());
        o.insert("maxHeight".into(), self.max_height.to_json());
        o.insert("stamp".into(), self.stamp.to_json());
        Value::Object(o)
    }
}

/// `TerrainStampDesc`: packages/knowledge/wiki/terrain.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainStampDesc {
    pub data: u64,
    pub origin: Vec3,
    pub quaternion: Quat,
    pub scale: Vec3,
    pub geometry: u64,
    pub position: Vec3,
    pub rotation: Quat,
    pub size: Vec3,
    pub mode: TerrainStampMode,
    pub depth: f32,
}

impl Default for TerrainStampDesc {
    fn default() -> Self {
        Self {
            data: Default::default(),
            origin: Default::default(),
            quaternion: Default::default(),
            scale: Vec3::new(1.0_f32, 1.0_f32, 1.0_f32),
            geometry: Default::default(),
            position: Default::default(),
            rotation: Default::default(),
            size: Vec3::new(1.0_f32, 1.0_f32, 1.0_f32),
            mode: Default::default(),
            depth: 0.25_f32,
        }
    }
}

impl FromJson for TerrainStampDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            data: field(o, "data", d.data)?,
            origin: field(o, "origin", d.origin)?,
            quaternion: field(o, "quaternion", d.quaternion)?,
            scale: field(o, "scale", d.scale)?,
            geometry: field(o, "geometry", d.geometry)?,
            position: field(o, "position", d.position)?,
            rotation: field(o, "rotation", d.rotation)?,
            size: field(o, "size", d.size)?,
            mode: field(o, "mode", d.mode)?,
            depth: field(o, "depth", d.depth)?,
        })
    }
}

impl ToJson for TerrainStampDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("data".into(), self.data.to_json());
        o.insert("origin".into(), self.origin.to_json());
        o.insert("quaternion".into(), self.quaternion.to_json());
        o.insert("scale".into(), self.scale.to_json());
        o.insert("geometry".into(), self.geometry.to_json());
        o.insert("position".into(), self.position.to_json());
        o.insert("rotation".into(), self.rotation.to_json());
        o.insert("size".into(), self.size.to_json());
        o.insert("mode".into(), self.mode.to_json());
        o.insert("depth".into(), self.depth.to_json());
        Value::Object(o)
    }
}

/// `TerrainLayerDesc`: packages/knowledge/wiki/terrain.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainLayerDesc {
    pub color: Color,
    pub roughness: f32,
    pub tile: f32,
    pub triplanar: bool,
    pub project_slope: f32,
    pub anti_tiling: f32,
    pub height_contrast: f32,
    pub macro_variation: f32,
    pub displacement: f32,
    pub cell_variation: f32,
    pub normal_strength: f32,
    pub slope_min: f32,
    pub slope_max: f32,
    pub slope_fade: f32,
    pub altitude_min: f32,
    pub altitude_max: f32,
    pub altitude_fade: f32,
}

impl Default for TerrainLayerDesc {
    fn default() -> Self {
        Self {
            color: Color { r: 1.0_f32, g: 1.0_f32, b: 1.0_f32 },
            roughness: 0.9_f32,
            tile: 4.0_f32,
            triplanar: Default::default(),
            project_slope: Default::default(),
            anti_tiling: Default::default(),
            height_contrast: Default::default(),
            macro_variation: Default::default(),
            displacement: Default::default(),
            cell_variation: Default::default(),
            normal_strength: 1.0_f32,
            slope_min: Default::default(),
            slope_max: 90.0_f32,
            slope_fade: 4.0_f32,
            altitude_min: -1000000.0_f32,
            altitude_max: 1000000.0_f32,
            altitude_fade: 4.0_f32,
        }
    }
}

impl FromJson for TerrainLayerDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            color: field(o, "color", d.color)?,
            roughness: field(o, "roughness", d.roughness)?,
            tile: field(o, "tile", d.tile)?,
            triplanar: field(o, "triplanar", d.triplanar)?,
            project_slope: field(o, "projectSlope", d.project_slope)?,
            anti_tiling: field(o, "antiTiling", d.anti_tiling)?,
            height_contrast: field(o, "heightContrast", d.height_contrast)?,
            macro_variation: field(o, "macroVariation", d.macro_variation)?,
            displacement: field(o, "displacement", d.displacement)?,
            cell_variation: field(o, "cellVariation", d.cell_variation)?,
            normal_strength: field(o, "normalStrength", d.normal_strength)?,
            slope_min: field(o, "slopeMin", d.slope_min)?,
            slope_max: field(o, "slopeMax", d.slope_max)?,
            slope_fade: field(o, "slopeFade", d.slope_fade)?,
            altitude_min: field(o, "altitudeMin", d.altitude_min)?,
            altitude_max: field(o, "altitudeMax", d.altitude_max)?,
            altitude_fade: field(o, "altitudeFade", d.altitude_fade)?,
        })
    }
}

impl ToJson for TerrainLayerDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("color".into(), self.color.to_json());
        o.insert("roughness".into(), self.roughness.to_json());
        o.insert("tile".into(), self.tile.to_json());
        o.insert("triplanar".into(), self.triplanar.to_json());
        o.insert("projectSlope".into(), self.project_slope.to_json());
        o.insert("antiTiling".into(), self.anti_tiling.to_json());
        o.insert("heightContrast".into(), self.height_contrast.to_json());
        o.insert("macroVariation".into(), self.macro_variation.to_json());
        o.insert("displacement".into(), self.displacement.to_json());
        o.insert("cellVariation".into(), self.cell_variation.to_json());
        o.insert("normalStrength".into(), self.normal_strength.to_json());
        o.insert("slopeMin".into(), self.slope_min.to_json());
        o.insert("slopeMax".into(), self.slope_max.to_json());
        o.insert("slopeFade".into(), self.slope_fade.to_json());
        o.insert("altitudeMin".into(), self.altitude_min.to_json());
        o.insert("altitudeMax".into(), self.altitude_max.to_json());
        o.insert("altitudeFade".into(), self.altitude_fade.to_json());
        Value::Object(o)
    }
}

/// `StreamState`: packages/knowledge/wiki/stream.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StreamState {
    #[default]
    Unloaded,
    Queued,
    Loading,
    Resident,
    Failed,
}

impl StreamState {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["unloaded", "queued", "loading", "resident", "failed"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Unloaded, Self::Queued, Self::Loading, Self::Resident, Self::Failed];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("StreamState", s, Self::NAMES)),
        }
    }
}

impl FromJson for StreamState {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "StreamState", Self::NAMES)?)
    }
}

impl ToJson for StreamState {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `StreamStatus`: packages/knowledge/wiki/stream.md.
#[derive(Clone, Debug, PartialEq)]
pub struct StreamStatus {
    pub name: String,
    pub scene: String,
    pub state: StreamState,
    pub wanted: bool,
    pub requested: bool,
    pub priority: i32,
    pub trigger: String,
    pub memory: f64,
    pub bytes: f64,
    pub assets: Vec<String>,
    pub reason: String,
    pub since: f64,
    pub loads: u32,
    pub kept: f64,
}

impl Default for StreamStatus {
    fn default() -> Self {
        Self {
            name: Default::default(),
            scene: Default::default(),
            state: Default::default(),
            wanted: Default::default(),
            requested: Default::default(),
            priority: Default::default(),
            trigger: Default::default(),
            memory: Default::default(),
            bytes: Default::default(),
            assets: Default::default(),
            reason: Default::default(),
            since: Default::default(),
            loads: Default::default(),
            kept: Default::default(),
        }
    }
}

impl FromJson for StreamStatus {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            name: field(o, "name", d.name)?,
            scene: field(o, "scene", d.scene)?,
            state: field(o, "state", d.state)?,
            wanted: field(o, "wanted", d.wanted)?,
            requested: field(o, "requested", d.requested)?,
            priority: field(o, "priority", d.priority)?,
            trigger: field(o, "trigger", d.trigger)?,
            memory: field(o, "memory", d.memory)?,
            bytes: field(o, "bytes", d.bytes)?,
            assets: field(o, "assets", d.assets)?,
            reason: field(o, "reason", d.reason)?,
            since: field(o, "since", d.since)?,
            loads: field(o, "loads", d.loads)?,
            kept: field(o, "kept", d.kept)?,
        })
    }
}

impl ToJson for StreamStatus {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("name".into(), self.name.to_json());
        o.insert("scene".into(), self.scene.to_json());
        o.insert("state".into(), self.state.to_json());
        o.insert("wanted".into(), self.wanted.to_json());
        o.insert("requested".into(), self.requested.to_json());
        o.insert("priority".into(), self.priority.to_json());
        o.insert("trigger".into(), self.trigger.to_json());
        o.insert("memory".into(), self.memory.to_json());
        o.insert("bytes".into(), self.bytes.to_json());
        o.insert("assets".into(), self.assets.to_json());
        o.insert("reason".into(), self.reason.to_json());
        o.insert("since".into(), self.since.to_json());
        o.insert("loads".into(), self.loads.to_json());
        o.insert("kept".into(), self.kept.to_json());
        Value::Object(o)
    }
}

/// `StreamBudget`: packages/knowledge/wiki/stream.md.
#[derive(Clone, Debug, PartialEq)]
pub struct StreamBudget {
    pub budget: f64,
    pub used: f64,
    pub kept: f64,
    pub peak: f64,
    pub groups: Vec<StreamStatus>,
}

impl Default for StreamBudget {
    fn default() -> Self {
        Self {
            budget: Default::default(),
            used: Default::default(),
            kept: Default::default(),
            peak: Default::default(),
            groups: Default::default(),
        }
    }
}

impl FromJson for StreamBudget {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            budget: field(o, "budget", d.budget)?,
            used: field(o, "used", d.used)?,
            kept: field(o, "kept", d.kept)?,
            peak: field(o, "peak", d.peak)?,
            groups: field(o, "groups", d.groups)?,
        })
    }
}

impl ToJson for StreamBudget {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("budget".into(), self.budget.to_json());
        o.insert("used".into(), self.used.to_json());
        o.insert("kept".into(), self.kept.to_json());
        o.insert("peak".into(), self.peak.to_json());
        o.insert("groups".into(), self.groups.to_json());
        Value::Object(o)
    }
}

/// `VideoInfo`: packages/knowledge/wiki/video.md.
#[derive(Clone, Debug, PartialEq)]
pub struct VideoInfo {
    pub video: u64,
    pub texture: u64,
    pub sound: u64,
    pub width: u32,
    pub height: u32,
    pub frames: u32,
    pub fps: u32,
    pub seconds: f32,
    pub format: String,
    pub bytes: f64,
    pub streamed: bool,
    pub codec: String,
    pub poster: u64,
}

impl Default for VideoInfo {
    fn default() -> Self {
        Self {
            video: Default::default(),
            texture: Default::default(),
            sound: Default::default(),
            width: Default::default(),
            height: Default::default(),
            frames: Default::default(),
            fps: Default::default(),
            seconds: Default::default(),
            format: Default::default(),
            bytes: Default::default(),
            streamed: Default::default(),
            codec: Default::default(),
            poster: Default::default(),
        }
    }
}

impl FromJson for VideoInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            video: field(o, "video", d.video)?,
            texture: field(o, "texture", d.texture)?,
            sound: field(o, "sound", d.sound)?,
            width: field(o, "width", d.width)?,
            height: field(o, "height", d.height)?,
            frames: field(o, "frames", d.frames)?,
            fps: field(o, "fps", d.fps)?,
            seconds: field(o, "seconds", d.seconds)?,
            format: field(o, "format", d.format)?,
            bytes: field(o, "bytes", d.bytes)?,
            streamed: field(o, "streamed", d.streamed)?,
            codec: field(o, "codec", d.codec)?,
            poster: field(o, "poster", d.poster)?,
        })
    }
}

impl ToJson for VideoInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("video".into(), self.video.to_json());
        o.insert("texture".into(), self.texture.to_json());
        o.insert("sound".into(), self.sound.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("height".into(), self.height.to_json());
        o.insert("frames".into(), self.frames.to_json());
        o.insert("fps".into(), self.fps.to_json());
        o.insert("seconds".into(), self.seconds.to_json());
        o.insert("format".into(), self.format.to_json());
        o.insert("bytes".into(), self.bytes.to_json());
        o.insert("streamed".into(), self.streamed.to_json());
        o.insert("codec".into(), self.codec.to_json());
        o.insert("poster".into(), self.poster.to_json());
        Value::Object(o)
    }
}

/// `VideoCapabilities`: packages/knowledge/wiki/video.md.
#[derive(Clone, Debug, PartialEq)]
pub struct VideoCapabilities {
    pub streamed: bool,
    pub decoder: String,
    pub hardware: bool,
    pub max_width: u32,
    pub max_height: u32,
}

impl Default for VideoCapabilities {
    fn default() -> Self {
        Self {
            streamed: Default::default(),
            decoder: Default::default(),
            hardware: Default::default(),
            max_width: Default::default(),
            max_height: Default::default(),
        }
    }
}

impl FromJson for VideoCapabilities {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            streamed: field(o, "streamed", d.streamed)?,
            decoder: field(o, "decoder", d.decoder)?,
            hardware: field(o, "hardware", d.hardware)?,
            max_width: field(o, "maxWidth", d.max_width)?,
            max_height: field(o, "maxHeight", d.max_height)?,
        })
    }
}

impl ToJson for VideoCapabilities {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("streamed".into(), self.streamed.to_json());
        o.insert("decoder".into(), self.decoder.to_json());
        o.insert("hardware".into(), self.hardware.to_json());
        o.insert("maxWidth".into(), self.max_width.to_json());
        o.insert("maxHeight".into(), self.max_height.to_json());
        Value::Object(o)
    }
}

/// `DebugDrawDesc`: packages/knowledge/wiki/debug.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DebugDrawDesc {
    pub color: Color,
    pub width: f32,
    pub duration: u32,
    pub rotation: Quat,
}

impl Default for DebugDrawDesc {
    fn default() -> Self {
        Self {
            color: Color { r: 1.0_f32, g: 1.0_f32, b: 0.0_f32 },
            width: 2.0_f32,
            duration: Default::default(),
            rotation: Default::default(),
        }
    }
}

impl FromJson for DebugDrawDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            color: field(o, "color", d.color)?,
            width: field(o, "width", d.width)?,
            duration: field(o, "duration", d.duration)?,
            rotation: field(o, "rotation", d.rotation)?,
        })
    }
}

impl ToJson for DebugDrawDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("color".into(), self.color.to_json());
        o.insert("width".into(), self.width.to_json());
        o.insert("duration".into(), self.duration.to_json());
        o.insert("rotation".into(), self.rotation.to_json());
        Value::Object(o)
    }
}

/// `AssertKind`: packages/knowledge/wiki/assert.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AssertKind {
    #[default]
    Always,
    Sometimes,
    Reachable,
    Unreachable,
    Each,
}

impl AssertKind {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["always", "sometimes", "reachable", "unreachable", "each"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Always, Self::Sometimes, Self::Reachable, Self::Unreachable, Self::Each];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("AssertKind", s, Self::NAMES)),
        }
    }
}

impl FromJson for AssertKind {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "AssertKind", Self::NAMES)?)
    }
}

impl ToJson for AssertKind {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `AssertRecord`: packages/knowledge/wiki/assert.md.
#[derive(Clone, Debug, PartialEq)]
pub struct AssertRecord {
    pub kind: AssertKind,
    pub name: String,
    pub location: String,
    pub condition: Option<bool>,
    pub key: String,
    pub details: Value,
    pub tick: u32,
    pub declared: bool,
}

impl Default for AssertRecord {
    fn default() -> Self {
        Self {
            kind: Default::default(),
            name: Default::default(),
            location: Default::default(),
            condition: Default::default(),
            key: Default::default(),
            details: Default::default(),
            tick: Default::default(),
            declared: Default::default(),
        }
    }
}

impl FromJson for AssertRecord {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            kind: field(o, "kind", d.kind)?,
            name: field(o, "name", d.name)?,
            location: field(o, "location", d.location)?,
            condition: field(o, "condition", d.condition)?,
            key: field(o, "key", d.key)?,
            details: field(o, "details", d.details)?,
            tick: field(o, "tick", d.tick)?,
            declared: field(o, "declared", d.declared)?,
        })
    }
}

impl ToJson for AssertRecord {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("kind".into(), self.kind.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("location".into(), self.location.to_json());
        o.insert("condition".into(), self.condition.to_json());
        o.insert("key".into(), self.key.to_json());
        o.insert("details".into(), self.details.to_json());
        o.insert("tick".into(), self.tick.to_json());
        o.insert("declared".into(), self.declared.to_json());
        Value::Object(o)
    }
}

/// `AssertOrigin`: packages/knowledge/wiki/assert.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AssertOrigin {
    #[default]
    Game,
    Default,
    Playtest,
}

impl AssertOrigin {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["game", "default", "playtest"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Game, Self::Default, Self::Playtest];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("AssertOrigin", s, Self::NAMES)),
        }
    }
}

impl FromJson for AssertOrigin {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "AssertOrigin", Self::NAMES)?)
    }
}

impl ToJson for AssertOrigin {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `AssertBucket`: packages/knowledge/wiki/assert.md.
#[derive(Clone, Debug, PartialEq)]
pub struct AssertBucket {
    pub key: String,
    pub fired: bool,
    pub first_tick: Option<u32>,
}

impl Default for AssertBucket {
    fn default() -> Self {
        Self {
            key: Default::default(),
            fired: Default::default(),
            first_tick: Default::default(),
        }
    }
}

impl FromJson for AssertBucket {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            key: field(o, "key", d.key)?,
            fired: field(o, "fired", d.fired)?,
            first_tick: field(o, "firstTick", d.first_tick)?,
        })
    }
}

impl ToJson for AssertBucket {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("key".into(), self.key.to_json());
        o.insert("fired".into(), self.fired.to_json());
        o.insert("firstTick".into(), self.first_tick.to_json());
        Value::Object(o)
    }
}

/// `AssertSite`: packages/knowledge/wiki/assert.md.
#[derive(Clone, Debug, PartialEq)]
pub struct AssertSite {
    pub kind: AssertKind,
    pub origin: AssertOrigin,
    pub name: String,
    pub location: String,
    pub ran: bool,
    pub fired: bool,
    pub first_tick: Option<u32>,
    pub failures: u32,
    pub buckets: Option<Vec<AssertBucket>>,
    pub buckets_dropped: Option<u32>,
}

impl Default for AssertSite {
    fn default() -> Self {
        Self {
            kind: Default::default(),
            origin: Default::default(),
            name: Default::default(),
            location: Default::default(),
            ran: Default::default(),
            fired: Default::default(),
            first_tick: Default::default(),
            failures: Default::default(),
            buckets: Default::default(),
            buckets_dropped: Default::default(),
        }
    }
}

impl FromJson for AssertSite {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            kind: field(o, "kind", d.kind)?,
            origin: field(o, "origin", d.origin)?,
            name: field(o, "name", d.name)?,
            location: field(o, "location", d.location)?,
            ran: field(o, "ran", d.ran)?,
            fired: field(o, "fired", d.fired)?,
            first_tick: field(o, "firstTick", d.first_tick)?,
            failures: field(o, "failures", d.failures)?,
            buckets: field(o, "buckets", d.buckets)?,
            buckets_dropped: field(o, "bucketsDropped", d.buckets_dropped)?,
        })
    }
}

impl ToJson for AssertSite {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("kind".into(), self.kind.to_json());
        o.insert("origin".into(), self.origin.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("location".into(), self.location.to_json());
        o.insert("ran".into(), self.ran.to_json());
        o.insert("fired".into(), self.fired.to_json());
        o.insert("firstTick".into(), self.first_tick.to_json());
        o.insert("failures".into(), self.failures.to_json());
        if let Some(x) = &self.buckets {
            o.insert("buckets".into(), x.to_json());
        }
        if let Some(x) = &self.buckets_dropped {
            o.insert("bucketsDropped".into(), x.to_json());
        }
        Value::Object(o)
    }
}

/// `AssertViolation`: packages/knowledge/wiki/assert.md.
#[derive(Clone, Debug, PartialEq)]
pub struct AssertViolation {
    pub kind: AssertKind,
    pub origin: AssertOrigin,
    pub name: String,
    pub location: String,
    pub tick: u32,
    pub entity: Option<String>,
    pub component: Option<String>,
    pub field: Option<String>,
    pub value: Value,
    pub expected: Value,
    pub message: String,
    pub details: Value,
    pub failures: u32,
    pub last_tick: u32,
}

impl Default for AssertViolation {
    fn default() -> Self {
        Self {
            kind: Default::default(),
            origin: Default::default(),
            name: Default::default(),
            location: Default::default(),
            tick: Default::default(),
            entity: Default::default(),
            component: Default::default(),
            field: Default::default(),
            value: Default::default(),
            expected: Default::default(),
            message: Default::default(),
            details: Default::default(),
            failures: Default::default(),
            last_tick: Default::default(),
        }
    }
}

impl FromJson for AssertViolation {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            kind: field(o, "kind", d.kind)?,
            origin: field(o, "origin", d.origin)?,
            name: field(o, "name", d.name)?,
            location: field(o, "location", d.location)?,
            tick: field(o, "tick", d.tick)?,
            entity: field(o, "entity", d.entity)?,
            component: field(o, "component", d.component)?,
            field: field(o, "field", d.field)?,
            value: field(o, "value", d.value)?,
            expected: field(o, "expected", d.expected)?,
            message: field(o, "message", d.message)?,
            details: field(o, "details", d.details)?,
            failures: field(o, "failures", d.failures)?,
            last_tick: field(o, "lastTick", d.last_tick)?,
        })
    }
}

impl ToJson for AssertViolation {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("kind".into(), self.kind.to_json());
        o.insert("origin".into(), self.origin.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("location".into(), self.location.to_json());
        o.insert("tick".into(), self.tick.to_json());
        o.insert("entity".into(), self.entity.to_json());
        o.insert("component".into(), self.component.to_json());
        o.insert("field".into(), self.field.to_json());
        o.insert("value".into(), self.value.to_json());
        o.insert("expected".into(), self.expected.to_json());
        o.insert("message".into(), self.message.to_json());
        o.insert("details".into(), self.details.to_json());
        o.insert("failures".into(), self.failures.to_json());
        o.insert("lastTick".into(), self.last_tick.to_json());
        Value::Object(o)
    }
}

/// `AssertUncovered`: packages/knowledge/wiki/assert.md.
#[derive(Clone, Debug, PartialEq)]
pub struct AssertUncovered {
    pub kind: AssertKind,
    pub name: String,
    pub location: String,
    pub key: Option<String>,
    pub ran: Option<bool>,
}

impl Default for AssertUncovered {
    fn default() -> Self {
        Self {
            kind: Default::default(),
            name: Default::default(),
            location: Default::default(),
            key: Default::default(),
            ran: Default::default(),
        }
    }
}

impl FromJson for AssertUncovered {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            kind: field(o, "kind", d.kind)?,
            name: field(o, "name", d.name)?,
            location: field(o, "location", d.location)?,
            key: field(o, "key", d.key)?,
            ran: field(o, "ran", d.ran)?,
        })
    }
}

impl ToJson for AssertUncovered {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("kind".into(), self.kind.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("location".into(), self.location.to_json());
        if let Some(x) = &self.key {
            o.insert("key".into(), x.to_json());
        }
        if let Some(x) = &self.ran {
            o.insert("ran".into(), x.to_json());
        }
        Value::Object(o)
    }
}

/// `AssertCost`: packages/knowledge/wiki/assert.md.
#[derive(Clone, Debug, PartialEq)]
pub struct AssertCost {
    pub fields: u32,
    pub ticks: u32,
    pub values: f64,
    pub total_ms: f64,
    pub mean_us: f64,
    pub max_us: f64,
}

impl Default for AssertCost {
    fn default() -> Self {
        Self {
            fields: Default::default(),
            ticks: Default::default(),
            values: Default::default(),
            total_ms: Default::default(),
            mean_us: Default::default(),
            max_us: Default::default(),
        }
    }
}

impl FromJson for AssertCost {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            fields: field(o, "fields", d.fields)?,
            ticks: field(o, "ticks", d.ticks)?,
            values: field(o, "values", d.values)?,
            total_ms: field(o, "totalMs", d.total_ms)?,
            mean_us: field(o, "meanUs", d.mean_us)?,
            max_us: field(o, "maxUs", d.max_us)?,
        })
    }
}

impl ToJson for AssertCost {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("fields".into(), self.fields.to_json());
        o.insert("ticks".into(), self.ticks.to_json());
        o.insert("values".into(), self.values.to_json());
        o.insert("totalMs".into(), self.total_ms.to_json());
        o.insert("meanUs".into(), self.mean_us.to_json());
        o.insert("maxUs".into(), self.max_us.to_json());
        Value::Object(o)
    }
}

/// `AssertEvent`: packages/knowledge/wiki/assert.md.
#[derive(Clone, Debug, PartialEq)]
pub struct AssertEvent {
    pub outcome: String,
    pub tick: u32,
    pub kind: AssertKind,
    pub origin: AssertOrigin,
    pub name: String,
    pub location: String,
    pub key: Option<String>,
    pub details: Value,
    pub entity: Option<String>,
    pub component: Option<String>,
    pub field: Option<String>,
    pub value: Option<Value>,
    pub message: Option<String>,
}

impl Default for AssertEvent {
    fn default() -> Self {
        Self {
            outcome: Default::default(),
            tick: Default::default(),
            kind: Default::default(),
            origin: Default::default(),
            name: Default::default(),
            location: Default::default(),
            key: Default::default(),
            details: Default::default(),
            entity: Default::default(),
            component: Default::default(),
            field: Default::default(),
            value: Default::default(),
            message: Default::default(),
        }
    }
}

impl FromJson for AssertEvent {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            outcome: field(o, "outcome", d.outcome)?,
            tick: field(o, "tick", d.tick)?,
            kind: field(o, "kind", d.kind)?,
            origin: field(o, "origin", d.origin)?,
            name: field(o, "name", d.name)?,
            location: field(o, "location", d.location)?,
            key: field(o, "key", d.key)?,
            details: field(o, "details", d.details)?,
            entity: field(o, "entity", d.entity)?,
            component: field(o, "component", d.component)?,
            field: field(o, "field", d.field)?,
            value: field(o, "value", d.value)?,
            message: field(o, "message", d.message)?,
        })
    }
}

impl ToJson for AssertEvent {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("outcome".into(), self.outcome.to_json());
        o.insert("tick".into(), self.tick.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("origin".into(), self.origin.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("location".into(), self.location.to_json());
        if let Some(x) = &self.key {
            o.insert("key".into(), x.to_json());
        }
        o.insert("details".into(), self.details.to_json());
        o.insert("entity".into(), self.entity.to_json());
        o.insert("component".into(), self.component.to_json());
        o.insert("field".into(), self.field.to_json());
        if let Some(x) = &self.value {
            o.insert("value".into(), x.to_json());
        }
        if let Some(x) = &self.message {
            o.insert("message".into(), x.to_json());
        }
        Value::Object(o)
    }
}

/// `AssertEvents`: packages/knowledge/wiki/assert.md.
#[derive(Clone, Debug, PartialEq)]
pub struct AssertEvents {
    pub epoch: u32,
    pub events: Vec<AssertEvent>,
    pub dropped: u32,
}

impl Default for AssertEvents {
    fn default() -> Self {
        Self {
            epoch: Default::default(),
            events: Default::default(),
            dropped: Default::default(),
        }
    }
}

impl FromJson for AssertEvents {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            epoch: field(o, "epoch", d.epoch)?,
            events: field(o, "events", d.events)?,
            dropped: field(o, "dropped", d.dropped)?,
        })
    }
}

impl ToJson for AssertEvents {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("epoch".into(), self.epoch.to_json());
        o.insert("events".into(), self.events.to_json());
        o.insert("dropped".into(), self.dropped.to_json());
        Value::Object(o)
    }
}

/// `AssertReport`: packages/knowledge/wiki/assert.md.
#[derive(Clone, Debug, PartialEq)]
pub struct AssertReport {
    pub epoch: u32,
    pub failures: u32,
    pub violations: Vec<AssertViolation>,
    pub uncovered: Vec<AssertUncovered>,
    pub sites: Vec<AssertSite>,
    pub sites_dropped: u32,
    pub defaults: AssertCost,
}

impl Default for AssertReport {
    fn default() -> Self {
        Self {
            epoch: Default::default(),
            failures: Default::default(),
            violations: Default::default(),
            uncovered: Default::default(),
            sites: Default::default(),
            sites_dropped: Default::default(),
            defaults: Default::default(),
        }
    }
}

impl FromJson for AssertReport {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            epoch: field(o, "epoch", d.epoch)?,
            failures: field(o, "failures", d.failures)?,
            violations: field(o, "violations", d.violations)?,
            uncovered: field(o, "uncovered", d.uncovered)?,
            sites: field(o, "sites", d.sites)?,
            sites_dropped: field(o, "sitesDropped", d.sites_dropped)?,
            defaults: field(o, "defaults", d.defaults)?,
        })
    }
}

impl ToJson for AssertReport {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("epoch".into(), self.epoch.to_json());
        o.insert("failures".into(), self.failures.to_json());
        o.insert("violations".into(), self.violations.to_json());
        o.insert("uncovered".into(), self.uncovered.to_json());
        o.insert("sites".into(), self.sites.to_json());
        o.insert("sitesDropped".into(), self.sites_dropped.to_json());
        o.insert("defaults".into(), self.defaults.to_json());
        Value::Object(o)
    }
}

/// `PathKind`: packages/knowledge/wiki/path.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PathKind {
    #[default]
    CatmullRom,
    Bezier,
    Linear,
}

impl PathKind {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["catmullRom", "bezier", "linear"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::CatmullRom, Self::Bezier, Self::Linear];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("PathKind", s, Self::NAMES)),
        }
    }
}

impl FromJson for PathKind {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "PathKind", Self::NAMES)?)
    }
}

impl ToJson for PathKind {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `CurveDesc`: packages/knowledge/wiki/path.md.
#[derive(Clone, Debug, PartialEq)]
pub struct CurveDesc {
    pub points: Vec<Vec3>,
    pub closed: bool,
    pub kind: PathKind,
    pub tension: f32,
}

impl Default for CurveDesc {
    fn default() -> Self {
        Self {
            points: Default::default(),
            closed: Default::default(),
            kind: Default::default(),
            tension: 0.5_f32,
        }
    }
}

impl FromJson for CurveDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            points: field(o, "points", d.points)?,
            closed: field(o, "closed", d.closed)?,
            kind: field(o, "kind", d.kind)?,
            tension: field(o, "tension", d.tension)?,
        })
    }
}

impl ToJson for CurveDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("points".into(), self.points.to_json());
        o.insert("closed".into(), self.closed.to_json());
        o.insert("kind".into(), self.kind.to_json());
        o.insert("tension".into(), self.tension.to_json());
        Value::Object(o)
    }
}

/// `PathSample`: packages/knowledge/wiki/path.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PathSample {
    pub position: Vec3,
    pub tangent: Vec3,
    pub up: Vec3,
}

impl Default for PathSample {
    fn default() -> Self {
        Self {
            position: Default::default(),
            tangent: Default::default(),
            up: Default::default(),
        }
    }
}

impl FromJson for PathSample {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            position: field(o, "position", d.position)?,
            tangent: field(o, "tangent", d.tangent)?,
            up: field(o, "up", d.up)?,
        })
    }
}

impl ToJson for PathSample {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("position".into(), self.position.to_json());
        o.insert("tangent".into(), self.tangent.to_json());
        o.insert("up".into(), self.up.to_json());
        Value::Object(o)
    }
}

/// `PathNearest`: packages/knowledge/wiki/path.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PathNearest {
    pub t: f32,
    pub position: Vec3,
    pub distance: f32,
}

impl Default for PathNearest {
    fn default() -> Self {
        Self {
            t: Default::default(),
            position: Default::default(),
            distance: Default::default(),
        }
    }
}

impl FromJson for PathNearest {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            t: field(o, "t", d.t)?,
            position: field(o, "position", d.position)?,
            distance: field(o, "distance", d.distance)?,
        })
    }
}

impl ToJson for PathNearest {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("t".into(), self.t.to_json());
        o.insert("position".into(), self.position.to_json());
        o.insert("distance".into(), self.distance.to_json());
        Value::Object(o)
    }
}

/// `TimelineTrackInfo`: packages/knowledge/wiki/timeline.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineTrackInfo {
    pub kind: String,
    pub target: String,
    pub keys: u32,
}

impl Default for TimelineTrackInfo {
    fn default() -> Self {
        Self {
            kind: Default::default(),
            target: Default::default(),
            keys: Default::default(),
        }
    }
}

impl FromJson for TimelineTrackInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            kind: field(o, "kind", d.kind)?,
            target: field(o, "target", d.target)?,
            keys: field(o, "keys", d.keys)?,
        })
    }
}

impl ToJson for TimelineTrackInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("kind".into(), self.kind.to_json());
        o.insert("target".into(), self.target.to_json());
        o.insert("keys".into(), self.keys.to_json());
        Value::Object(o)
    }
}

/// `TimelineInfo`: packages/knowledge/wiki/timeline.md.
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineInfo {
    pub asset: u64,
    pub name: String,
    pub duration: f32,
    pub tracks: Vec<TimelineTrackInfo>,
    pub paths: Vec<String>,
    pub markers: Vec<String>,
}

impl Default for TimelineInfo {
    fn default() -> Self {
        Self {
            asset: Default::default(),
            name: Default::default(),
            duration: Default::default(),
            tracks: Default::default(),
            paths: Default::default(),
            markers: Default::default(),
        }
    }
}

impl FromJson for TimelineInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            asset: field(o, "asset", d.asset)?,
            name: field(o, "name", d.name)?,
            duration: field(o, "duration", d.duration)?,
            tracks: field(o, "tracks", d.tracks)?,
            paths: field(o, "paths", d.paths)?,
            markers: field(o, "markers", d.markers)?,
        })
    }
}

impl ToJson for TimelineInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("asset".into(), self.asset.to_json());
        o.insert("name".into(), self.name.to_json());
        o.insert("duration".into(), self.duration.to_json());
        o.insert("tracks".into(), self.tracks.to_json());
        o.insert("paths".into(), self.paths.to_json());
        o.insert("markers".into(), self.markers.to_json());
        Value::Object(o)
    }
}

/// `InputDesc`: packages/knowledge/wiki/net.md.
#[derive(Clone, Debug, PartialEq)]
pub struct InputDesc {
    pub buttons: u32,
    pub x: f32,
    pub y: f32,
    pub aim_x: f32,
    pub aim_y: f32,
}

impl Default for InputDesc {
    fn default() -> Self {
        Self {
            buttons: Default::default(),
            x: Default::default(),
            y: Default::default(),
            aim_x: Default::default(),
            aim_y: Default::default(),
        }
    }
}

impl FromJson for InputDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            buttons: field(o, "buttons", d.buttons)?,
            x: field(o, "x", d.x)?,
            y: field(o, "y", d.y)?,
            aim_x: field(o, "aimX", d.aim_x)?,
            aim_y: field(o, "aimY", d.aim_y)?,
        })
    }
}

impl ToJson for InputDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("buttons".into(), self.buttons.to_json());
        o.insert("x".into(), self.x.to_json());
        o.insert("y".into(), self.y.to_json());
        o.insert("aimX".into(), self.aim_x.to_json());
        o.insert("aimY".into(), self.aim_y.to_json());
        Value::Object(o)
    }
}

/// `NetShapeDesc`: packages/knowledge/wiki/net.md.
#[derive(Clone, Debug, PartialEq)]
pub struct NetShapeDesc {
    pub input: Option<Value>,
    pub command: Option<Value>,
    pub max_commands: u32,
}

impl Default for NetShapeDesc {
    fn default() -> Self {
        Self {
            input: Default::default(),
            command: Default::default(),
            max_commands: 32,
        }
    }
}

impl FromJson for NetShapeDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            input: field(o, "input", d.input)?,
            command: field(o, "command", d.command)?,
            max_commands: field(o, "maxCommands", d.max_commands)?,
        })
    }
}

impl ToJson for NetShapeDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("input".into(), self.input.to_json());
        o.insert("command".into(), self.command.to_json());
        o.insert("maxCommands".into(), self.max_commands.to_json());
        Value::Object(o)
    }
}

/// `LockstepSeat`: packages/knowledge/wiki/net.md.
#[derive(Clone, Debug, PartialEq)]
pub struct LockstepSeat {
    pub seat: u32,
    pub peer: u32,
    pub since: f64,
    pub until: Option<f64>,
    pub left: bool,
}

impl Default for LockstepSeat {
    fn default() -> Self {
        Self {
            seat: Default::default(),
            peer: Default::default(),
            since: Default::default(),
            until: Default::default(),
            left: Default::default(),
        }
    }
}

impl FromJson for LockstepSeat {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            seat: field(o, "seat", d.seat)?,
            peer: field(o, "peer", d.peer)?,
            since: field(o, "since", d.since)?,
            until: field(o, "until", d.until)?,
            left: field(o, "left", d.left)?,
        })
    }
}

impl ToJson for LockstepSeat {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("seat".into(), self.seat.to_json());
        o.insert("peer".into(), self.peer.to_json());
        o.insert("since".into(), self.since.to_json());
        o.insert("until".into(), self.until.to_json());
        o.insert("left".into(), self.left.to_json());
        Value::Object(o)
    }
}

/// `LockstepStatus`: packages/knowledge/wiki/net.md.
#[derive(Clone, Debug, PartialEq)]
pub struct LockstepStatus {
    pub running: bool,
    pub peer: Option<u32>,
    pub peers: Vec<u32>,
    pub start_tick: Option<f64>,
    pub players: Option<u32>,
    pub seats: Vec<LockstepSeat>,
    pub spectators: Vec<u32>,
    pub waiting_for: Vec<u32>,
    pub waits: f64,
    pub desync: Option<u32>,
    pub rollback: bool,
    pub max_prediction: Option<u32>,
    pub confirmed_tick: Option<f64>,
    pub final_tick: Option<f64>,
    pub acknowledged: Option<f64>,
    pub predicted: u32,
    pub rollbacks: f64,
    pub resimulated_ticks: f64,
    pub stalled: Vec<u32>,
    pub drops: f64,
    pub snapshot_bytes: f64,
}

impl Default for LockstepStatus {
    fn default() -> Self {
        Self {
            running: Default::default(),
            peer: Default::default(),
            peers: Default::default(),
            start_tick: Default::default(),
            players: Default::default(),
            seats: Default::default(),
            spectators: Default::default(),
            waiting_for: Default::default(),
            waits: Default::default(),
            desync: Default::default(),
            rollback: Default::default(),
            max_prediction: Default::default(),
            confirmed_tick: Default::default(),
            final_tick: Default::default(),
            acknowledged: Default::default(),
            predicted: Default::default(),
            rollbacks: Default::default(),
            resimulated_ticks: Default::default(),
            stalled: Default::default(),
            drops: Default::default(),
            snapshot_bytes: Default::default(),
        }
    }
}

impl FromJson for LockstepStatus {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            running: field(o, "running", d.running)?,
            peer: field(o, "peer", d.peer)?,
            peers: field(o, "peers", d.peers)?,
            start_tick: field(o, "startTick", d.start_tick)?,
            players: field(o, "players", d.players)?,
            seats: field(o, "seats", d.seats)?,
            spectators: field(o, "spectators", d.spectators)?,
            waiting_for: field(o, "waitingFor", d.waiting_for)?,
            waits: field(o, "waits", d.waits)?,
            desync: field(o, "desync", d.desync)?,
            rollback: field(o, "rollback", d.rollback)?,
            max_prediction: field(o, "maxPrediction", d.max_prediction)?,
            confirmed_tick: field(o, "confirmedTick", d.confirmed_tick)?,
            final_tick: field(o, "finalTick", d.final_tick)?,
            acknowledged: field(o, "acknowledged", d.acknowledged)?,
            predicted: field(o, "predicted", d.predicted)?,
            rollbacks: field(o, "rollbacks", d.rollbacks)?,
            resimulated_ticks: field(o, "resimulatedTicks", d.resimulated_ticks)?,
            stalled: field(o, "stalled", d.stalled)?,
            drops: field(o, "drops", d.drops)?,
            snapshot_bytes: field(o, "snapshotBytes", d.snapshot_bytes)?,
        })
    }
}

impl ToJson for LockstepStatus {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("running".into(), self.running.to_json());
        o.insert("peer".into(), self.peer.to_json());
        o.insert("peers".into(), self.peers.to_json());
        o.insert("startTick".into(), self.start_tick.to_json());
        o.insert("players".into(), self.players.to_json());
        o.insert("seats".into(), self.seats.to_json());
        o.insert("spectators".into(), self.spectators.to_json());
        o.insert("waitingFor".into(), self.waiting_for.to_json());
        o.insert("waits".into(), self.waits.to_json());
        o.insert("desync".into(), self.desync.to_json());
        o.insert("rollback".into(), self.rollback.to_json());
        o.insert("maxPrediction".into(), self.max_prediction.to_json());
        o.insert("confirmedTick".into(), self.confirmed_tick.to_json());
        o.insert("finalTick".into(), self.final_tick.to_json());
        o.insert("acknowledged".into(), self.acknowledged.to_json());
        o.insert("predicted".into(), self.predicted.to_json());
        o.insert("rollbacks".into(), self.rollbacks.to_json());
        o.insert("resimulatedTicks".into(), self.resimulated_ticks.to_json());
        o.insert("stalled".into(), self.stalled.to_json());
        o.insert("drops".into(), self.drops.to_json());
        o.insert("snapshotBytes".into(), self.snapshot_bytes.to_json());
        Value::Object(o)
    }
}

/// `MotionGoalKind`: packages/knowledge/wiki/motion.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MotionGoalKind {
    #[default]
    None,
    Pose,
    Place,
    Face,
    WalkTo,
    Jump,
}

impl MotionGoalKind {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["none", "pose", "place", "face", "walkTo", "jump"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::None, Self::Pose, Self::Place, Self::Face, Self::WalkTo, Self::Jump];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("MotionGoalKind", s, Self::NAMES)),
        }
    }
}

impl FromJson for MotionGoalKind {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "MotionGoalKind", Self::NAMES)?)
    }
}

impl ToJson for MotionGoalKind {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `MotionBackend`: packages/knowledge/wiki/motion.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MotionBackend {
    #[default]
    None,
    Gpu,
    Cpu,
}

impl MotionBackend {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["none", "gpu", "cpu"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::None, Self::Gpu, Self::Cpu];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("MotionBackend", s, Self::NAMES)),
        }
    }
}

impl FromJson for MotionBackend {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "MotionBackend", Self::NAMES)?)
    }
}

impl ToJson for MotionBackend {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `PoseGoalDesc`: packages/knowledge/wiki/motion.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PoseGoalDesc {
    pub pose: String,
    pub duration: f32,
}

impl Default for PoseGoalDesc {
    fn default() -> Self {
        Self {
            pose: Default::default(),
            duration: 1.5_f32,
        }
    }
}

impl FromJson for PoseGoalDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            pose: field(o, "pose", d.pose)?,
            duration: field(o, "duration", d.duration)?,
        })
    }
}

impl ToJson for PoseGoalDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("pose".into(), self.pose.to_json());
        o.insert("duration".into(), self.duration.to_json());
        Value::Object(o)
    }
}

/// `PlaceGoalDesc`: packages/knowledge/wiki/motion.md.
#[derive(Clone, Debug, PartialEq)]
pub struct PlaceGoalDesc {
    pub pose: String,
    pub position: Vec3,
    pub heading: f32,
    pub duration: f32,
}

impl Default for PlaceGoalDesc {
    fn default() -> Self {
        Self {
            pose: Default::default(),
            position: Default::default(),
            heading: Default::default(),
            duration: 1.5_f32,
        }
    }
}

impl FromJson for PlaceGoalDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            pose: field(o, "pose", d.pose)?,
            position: field(o, "position", d.position)?,
            heading: field(o, "heading", d.heading)?,
            duration: field(o, "duration", d.duration)?,
        })
    }
}

impl ToJson for PlaceGoalDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("pose".into(), self.pose.to_json());
        o.insert("position".into(), self.position.to_json());
        o.insert("heading".into(), self.heading.to_json());
        o.insert("duration".into(), self.duration.to_json());
        Value::Object(o)
    }
}

/// `FaceGoalDesc`: packages/knowledge/wiki/motion.md.
#[derive(Clone, Debug, PartialEq)]
pub struct FaceGoalDesc {
    pub heading: f32,
    pub duration: f32,
}

impl Default for FaceGoalDesc {
    fn default() -> Self {
        Self {
            heading: Default::default(),
            duration: 0.8_f32,
        }
    }
}

impl FromJson for FaceGoalDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            heading: field(o, "heading", d.heading)?,
            duration: field(o, "duration", d.duration)?,
        })
    }
}

impl ToJson for FaceGoalDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("heading".into(), self.heading.to_json());
        o.insert("duration".into(), self.duration.to_json());
        Value::Object(o)
    }
}

/// `WalkGoalDesc`: packages/knowledge/wiki/motion.md.
#[derive(Clone, Debug, PartialEq)]
pub struct WalkGoalDesc {
    pub position: Vec3,
    pub speed: f32,
    pub pose: String,
    pub arrival_heading: f32,
    pub deceleration: f32,
    pub acceleration: f32,
}

impl Default for WalkGoalDesc {
    fn default() -> Self {
        Self {
            position: Default::default(),
            speed: 1.4_f32,
            pose: Default::default(),
            arrival_heading: Default::default(),
            deceleration: 3.0_f32,
            acceleration: 4.0_f32,
        }
    }
}

impl FromJson for WalkGoalDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            position: field(o, "position", d.position)?,
            speed: field(o, "speed", d.speed)?,
            pose: field(o, "pose", d.pose)?,
            arrival_heading: field(o, "arrivalHeading", d.arrival_heading)?,
            deceleration: field(o, "deceleration", d.deceleration)?,
            acceleration: field(o, "acceleration", d.acceleration)?,
        })
    }
}

impl ToJson for WalkGoalDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("position".into(), self.position.to_json());
        o.insert("speed".into(), self.speed.to_json());
        o.insert("pose".into(), self.pose.to_json());
        o.insert("arrivalHeading".into(), self.arrival_heading.to_json());
        o.insert("deceleration".into(), self.deceleration.to_json());
        o.insert("acceleration".into(), self.acceleration.to_json());
        Value::Object(o)
    }
}

/// `JumpGoalDesc`: packages/knowledge/wiki/motion.md.
#[derive(Clone, Debug, PartialEq)]
pub struct JumpGoalDesc {
    pub height: f32,
    pub distance: f32,
    pub heading: f32,
    pub prepare: f32,
    pub recover: f32,
    pub pose: String,
}

impl Default for JumpGoalDesc {
    fn default() -> Self {
        Self {
            height: 0.5_f32,
            distance: 0.0_f32,
            heading: Default::default(),
            prepare: 0.25_f32,
            recover: 0.35_f32,
            pose: Default::default(),
        }
    }
}

impl FromJson for JumpGoalDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            height: field(o, "height", d.height)?,
            distance: field(o, "distance", d.distance)?,
            heading: field(o, "heading", d.heading)?,
            prepare: field(o, "prepare", d.prepare)?,
            recover: field(o, "recover", d.recover)?,
            pose: field(o, "pose", d.pose)?,
        })
    }
}

impl ToJson for JumpGoalDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("height".into(), self.height.to_json());
        o.insert("distance".into(), self.distance.to_json());
        o.insert("heading".into(), self.heading.to_json());
        o.insert("prepare".into(), self.prepare.to_json());
        o.insert("recover".into(), self.recover.to_json());
        o.insert("pose".into(), self.pose.to_json());
        Value::Object(o)
    }
}

/// `LookAtDesc`: packages/knowledge/wiki/motion.md.
#[derive(Clone, Debug, PartialEq)]
pub struct LookAtDesc {
    pub target: Vec3,
    pub duration: f32,
    pub blend: f32,
    pub max_yaw: f32,
    pub max_pitch: f32,
    pub weight: f32,
}

impl Default for LookAtDesc {
    fn default() -> Self {
        Self {
            target: Default::default(),
            duration: 2.0_f32,
            blend: 0.2_f32,
            max_yaw: 1.92_f32,
            max_pitch: 0.6_f32,
            weight: 1.0_f32,
        }
    }
}

impl FromJson for LookAtDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            target: field(o, "target", d.target)?,
            duration: field(o, "duration", d.duration)?,
            blend: field(o, "blend", d.blend)?,
            max_yaw: field(o, "maxYaw", d.max_yaw)?,
            max_pitch: field(o, "maxPitch", d.max_pitch)?,
            weight: field(o, "weight", d.weight)?,
        })
    }
}

impl ToJson for LookAtDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("target".into(), self.target.to_json());
        o.insert("duration".into(), self.duration.to_json());
        o.insert("blend".into(), self.blend.to_json());
        o.insert("maxYaw".into(), self.max_yaw.to_json());
        o.insert("maxPitch".into(), self.max_pitch.to_json());
        o.insert("weight".into(), self.weight.to_json());
        Value::Object(o)
    }
}

/// `ReactionDesc`: packages/knowledge/wiki/motion.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ReactionDesc {
    pub action: String,
    pub direction: Vec3,
    pub strength: f32,
    pub duration: f32,
    pub blend: f32,
}

impl Default for ReactionDesc {
    fn default() -> Self {
        Self {
            action: Default::default(),
            direction: Default::default(),
            strength: 1.0_f32,
            duration: 1.0_f32,
            blend: 0.2_f32,
        }
    }
}

impl FromJson for ReactionDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            action: field(o, "action", d.action)?,
            direction: field(o, "direction", d.direction)?,
            strength: field(o, "strength", d.strength)?,
            duration: field(o, "duration", d.duration)?,
            blend: field(o, "blend", d.blend)?,
        })
    }
}

impl ToJson for ReactionDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("action".into(), self.action.to_json());
        o.insert("direction".into(), self.direction.to_json());
        o.insert("strength".into(), self.strength.to_json());
        o.insert("duration".into(), self.duration.to_json());
        o.insert("blend".into(), self.blend.to_json());
        Value::Object(o)
    }
}

/// `MotionStatus`: packages/knowledge/wiki/motion.md.
#[derive(Clone, Debug, PartialEq)]
pub struct MotionStatus {
    pub goal: MotionGoalKind,
    pub pose: String,
    pub target: Vec3,
    pub heading: f32,
    pub elapsed: f32,
    pub airborne: bool,
    pub queued: u32,
    pub completed: u32,
    pub looking: bool,
    pub reacting: bool,
}

impl Default for MotionStatus {
    fn default() -> Self {
        Self {
            goal: Default::default(),
            pose: Default::default(),
            target: Default::default(),
            heading: Default::default(),
            elapsed: Default::default(),
            airborne: Default::default(),
            queued: Default::default(),
            completed: Default::default(),
            looking: Default::default(),
            reacting: Default::default(),
        }
    }
}

impl FromJson for MotionStatus {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            goal: field(o, "goal", d.goal)?,
            pose: field(o, "pose", d.pose)?,
            target: field(o, "target", d.target)?,
            heading: field(o, "heading", d.heading)?,
            elapsed: field(o, "elapsed", d.elapsed)?,
            airborne: field(o, "airborne", d.airborne)?,
            queued: field(o, "queued", d.queued)?,
            completed: field(o, "completed", d.completed)?,
            looking: field(o, "looking", d.looking)?,
            reacting: field(o, "reacting", d.reacting)?,
        })
    }
}

impl ToJson for MotionStatus {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("goal".into(), self.goal.to_json());
        o.insert("pose".into(), self.pose.to_json());
        o.insert("target".into(), self.target.to_json());
        o.insert("heading".into(), self.heading.to_json());
        o.insert("elapsed".into(), self.elapsed.to_json());
        o.insert("airborne".into(), self.airborne.to_json());
        o.insert("queued".into(), self.queued.to_json());
        o.insert("completed".into(), self.completed.to_json());
        o.insert("looking".into(), self.looking.to_json());
        o.insert("reacting".into(), self.reacting.to_json());
        Value::Object(o)
    }
}

/// `MotionPresentationState`: packages/knowledge/wiki/motion.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MotionPresentationState {
    #[default]
    Loading,
    Warming,
    Ready,
    Failed,
}

impl MotionPresentationState {
    /// The variants' IDL names, in index order.
    pub const NAMES: &'static [&'static str] = &["loading", "warming", "ready", "failed"];
    /// Every variant, in index order.
    pub const ALL: &'static [Self] = &[Self::Loading, Self::Warming, Self::Ready, Self::Failed];
    /// The IDL name.
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
    /// The variant named `s`, or a refusal naming them all.
    pub fn parse(s: &str) -> Result<Self, String> {
        match Self::NAMES.iter().position(|n| *n == s) {
            Some(i) => Ok(Self::ALL[i]),
            None => Err(unknown_variant("MotionPresentationState", s, Self::NAMES)),
        }
    }
}

impl FromJson for MotionPresentationState {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::parse(variant(v, "MotionPresentationState", Self::NAMES)?)
    }
}

impl ToJson for MotionPresentationState {
    fn to_json(&self) -> Value {
        Value::String(self.as_str().into())
    }
}

/// `MotionPresentationStatus`: packages/knowledge/wiki/motion.md.
#[derive(Clone, Debug, PartialEq)]
pub struct MotionPresentationStatus {
    pub state: MotionPresentationState,
    pub reason: String,
}

impl Default for MotionPresentationStatus {
    fn default() -> Self {
        Self {
            state: Default::default(),
            reason: Default::default(),
        }
    }
}

impl FromJson for MotionPresentationStatus {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            state: field(o, "state", d.state)?,
            reason: field(o, "reason", d.reason)?,
        })
    }
}

impl ToJson for MotionPresentationStatus {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("state".into(), self.state.to_json());
        o.insert("reason".into(), self.reason.to_json());
        Value::Object(o)
    }
}

/// `MotionCapabilities`: packages/knowledge/wiki/motion.md.
#[derive(Clone, Debug, PartialEq)]
pub struct MotionCapabilities {
    pub backend: MotionBackend,
    pub reason: String,
    pub model_bytes: f64,
}

impl Default for MotionCapabilities {
    fn default() -> Self {
        Self {
            backend: Default::default(),
            reason: Default::default(),
            model_bytes: Default::default(),
        }
    }
}

impl FromJson for MotionCapabilities {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            backend: field(o, "backend", d.backend)?,
            reason: field(o, "reason", d.reason)?,
            model_bytes: field(o, "modelBytes", d.model_bytes)?,
        })
    }
}

impl ToJson for MotionCapabilities {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("backend".into(), self.backend.to_json());
        o.insert("reason".into(), self.reason.to_json());
        o.insert("modelBytes".into(), self.model_bytes.to_json());
        Value::Object(o)
    }
}

/// `DamageDesc`: packages/knowledge/wiki/destruction.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DamageDesc {
    pub point: Vec3,
    pub radius: f32,
    pub inner_radius: f32,
    pub amount: f32,
    pub impulse: Vec3,
}

impl Default for DamageDesc {
    fn default() -> Self {
        Self {
            point: Default::default(),
            radius: 0.4_f32,
            inner_radius: 0.1_f32,
            amount: 500.0_f32,
            impulse: Default::default(),
        }
    }
}

impl FromJson for DamageDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            point: field(o, "point", d.point)?,
            radius: field(o, "radius", d.radius)?,
            inner_radius: field(o, "innerRadius", d.inner_radius)?,
            amount: field(o, "amount", d.amount)?,
            impulse: field(o, "impulse", d.impulse)?,
        })
    }
}

impl ToJson for DamageDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("point".into(), self.point.to_json());
        o.insert("radius".into(), self.radius.to_json());
        o.insert("innerRadius".into(), self.inner_radius.to_json());
        o.insert("amount".into(), self.amount.to_json());
        o.insert("impulse".into(), self.impulse.to_json());
        Value::Object(o)
    }
}

/// `ExplosionDesc`: packages/knowledge/wiki/destruction.md.
#[derive(Clone, Debug, PartialEq)]
pub struct ExplosionDesc {
    pub point: Vec3,
    pub radius: f32,
    pub inner_radius: f32,
    pub damage: f32,
    pub impulse: f32,
}

impl Default for ExplosionDesc {
    fn default() -> Self {
        Self {
            point: Default::default(),
            radius: 3.0_f32,
            inner_radius: 0.5_f32,
            damage: 3000.0_f32,
            impulse: 2000.0_f32,
        }
    }
}

impl FromJson for ExplosionDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            point: field(o, "point", d.point)?,
            radius: field(o, "radius", d.radius)?,
            inner_radius: field(o, "innerRadius", d.inner_radius)?,
            damage: field(o, "damage", d.damage)?,
            impulse: field(o, "impulse", d.impulse)?,
        })
    }
}

impl ToJson for ExplosionDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("point".into(), self.point.to_json());
        o.insert("radius".into(), self.radius.to_json());
        o.insert("innerRadius".into(), self.inner_radius.to_json());
        o.insert("damage".into(), self.damage.to_json());
        o.insert("impulse".into(), self.impulse.to_json());
        Value::Object(o)
    }
}

/// `DestructibleInfo`: packages/knowledge/wiki/destruction.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DestructibleInfo {
    pub chunks: u32,
    pub children: u32,
    pub bonds: u32,
    pub intact: u32,
    pub islands: u32,
    pub anchored: u32,
    pub health: f32,
    pub debris: u32,
    pub crumbs: u32,
}

impl Default for DestructibleInfo {
    fn default() -> Self {
        Self {
            chunks: Default::default(),
            children: Default::default(),
            bonds: Default::default(),
            intact: Default::default(),
            islands: Default::default(),
            anchored: Default::default(),
            health: Default::default(),
            debris: Default::default(),
            crumbs: Default::default(),
        }
    }
}

impl FromJson for DestructibleInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            chunks: field(o, "chunks", d.chunks)?,
            children: field(o, "children", d.children)?,
            bonds: field(o, "bonds", d.bonds)?,
            intact: field(o, "intact", d.intact)?,
            islands: field(o, "islands", d.islands)?,
            anchored: field(o, "anchored", d.anchored)?,
            health: field(o, "health", d.health)?,
            debris: field(o, "debris", d.debris)?,
            crumbs: field(o, "crumbs", d.crumbs)?,
        })
    }
}

impl ToJson for DestructibleInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("chunks".into(), self.chunks.to_json());
        o.insert("children".into(), self.children.to_json());
        o.insert("bonds".into(), self.bonds.to_json());
        o.insert("intact".into(), self.intact.to_json());
        o.insert("islands".into(), self.islands.to_json());
        o.insert("anchored".into(), self.anchored.to_json());
        o.insert("health".into(), self.health.to_json());
        o.insert("debris".into(), self.debris.to_json());
        o.insert("crumbs".into(), self.crumbs.to_json());
        Value::Object(o)
    }
}

/// `DestructibleMaterial`: packages/knowledge/wiki/destruction.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DestructibleMaterial {
    pub toughness: f32,
    pub strength: f32,
    pub friction: f32,
    pub tension: f32,
    pub density: f32,
    pub impact_threshold: f32,
    pub impact_radius: f32,
}

impl Default for DestructibleMaterial {
    fn default() -> Self {
        Self {
            toughness: Default::default(),
            strength: Default::default(),
            friction: Default::default(),
            tension: Default::default(),
            density: Default::default(),
            impact_threshold: Default::default(),
            impact_radius: Default::default(),
        }
    }
}

impl FromJson for DestructibleMaterial {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            toughness: field(o, "toughness", d.toughness)?,
            strength: field(o, "strength", d.strength)?,
            friction: field(o, "friction", d.friction)?,
            tension: field(o, "tension", d.tension)?,
            density: field(o, "density", d.density)?,
            impact_threshold: field(o, "impactThreshold", d.impact_threshold)?,
            impact_radius: field(o, "impactRadius", d.impact_radius)?,
        })
    }
}

impl ToJson for DestructibleMaterial {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("toughness".into(), self.toughness.to_json());
        o.insert("strength".into(), self.strength.to_json());
        o.insert("friction".into(), self.friction.to_json());
        o.insert("tension".into(), self.tension.to_json());
        o.insert("density".into(), self.density.to_json());
        o.insert("impactThreshold".into(), self.impact_threshold.to_json());
        o.insert("impactRadius".into(), self.impact_radius.to_json());
        Value::Object(o)
    }
}

/// `DestructionCapabilities`: packages/knowledge/wiki/destruction.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DestructionCapabilities {
    pub visual_debris: bool,
    pub max_visual_debris: u32,
    pub visual_debris_contacts: String,
}

impl Default for DestructionCapabilities {
    fn default() -> Self {
        Self {
            visual_debris: Default::default(),
            max_visual_debris: Default::default(),
            visual_debris_contacts: Default::default(),
        }
    }
}

impl FromJson for DestructionCapabilities {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            visual_debris: field(o, "visualDebris", d.visual_debris)?,
            max_visual_debris: field(o, "maxVisualDebris", d.max_visual_debris)?,
            visual_debris_contacts: field(o, "visualDebrisContacts", d.visual_debris_contacts)?,
        })
    }
}

impl ToJson for DestructionCapabilities {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("visualDebris".into(), self.visual_debris.to_json());
        o.insert("maxVisualDebris".into(), self.max_visual_debris.to_json());
        o.insert("visualDebrisContacts".into(), self.visual_debris_contacts.to_json());
        Value::Object(o)
    }
}

/// `Carve2dDesc`: packages/knowledge/wiki/destruction.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Carve2dDesc {
    pub point: Vec3,
    pub radius: f32,
}

impl Default for Carve2dDesc {
    fn default() -> Self {
        Self {
            point: Default::default(),
            radius: 1.0_f32,
        }
    }
}

impl FromJson for Carve2dDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            point: field(o, "point", d.point)?,
            radius: field(o, "radius", d.radius)?,
        })
    }
}

impl ToJson for Carve2dDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("point".into(), self.point.to_json());
        o.insert("radius".into(), self.radius.to_json());
        Value::Object(o)
    }
}

/// `Fill2dDesc`: packages/knowledge/wiki/destruction.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Fill2dDesc {
    pub point: Vec3,
    pub radius: f32,
    pub bedrock: bool,
}

impl Default for Fill2dDesc {
    fn default() -> Self {
        Self {
            point: Default::default(),
            radius: 1.0_f32,
            bedrock: Default::default(),
        }
    }
}

impl FromJson for Fill2dDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            point: field(o, "point", d.point)?,
            radius: field(o, "radius", d.radius)?,
            bedrock: field(o, "bedrock", d.bedrock)?,
        })
    }
}

impl ToJson for Fill2dDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("point".into(), self.point.to_json());
        o.insert("radius".into(), self.radius.to_json());
        o.insert("bedrock".into(), self.bedrock.to_json());
        Value::Object(o)
    }
}

/// `Explosion2dDesc`: packages/knowledge/wiki/destruction.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Explosion2dDesc {
    pub point: Vec3,
    pub radius: f32,
    pub debris: f32,
    pub piece_cells: u32,
    pub speed: f32,
}

impl Default for Explosion2dDesc {
    fn default() -> Self {
        Self {
            point: Default::default(),
            radius: 1.5_f32,
            debris: 0.4_f32,
            piece_cells: 24,
            speed: 8.0_f32,
        }
    }
}

impl FromJson for Explosion2dDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            point: field(o, "point", d.point)?,
            radius: field(o, "radius", d.radius)?,
            debris: field(o, "debris", d.debris)?,
            piece_cells: field(o, "pieceCells", d.piece_cells)?,
            speed: field(o, "speed", d.speed)?,
        })
    }
}

impl ToJson for Explosion2dDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("point".into(), self.point.to_json());
        o.insert("radius".into(), self.radius.to_json());
        o.insert("debris".into(), self.debris.to_json());
        o.insert("pieceCells".into(), self.piece_cells.to_json());
        o.insert("speed".into(), self.speed.to_json());
        Value::Object(o)
    }
}

/// `Fracture2dDesc`: packages/knowledge/wiki/destruction.md.
#[derive(Clone, Debug, PartialEq)]
pub struct Fracture2dDesc {
    pub pieces: u32,
}

impl Default for Fracture2dDesc {
    fn default() -> Self {
        Self {
            pieces: 8,
        }
    }
}

impl FromJson for Fracture2dDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            pieces: field(o, "pieces", d.pieces)?,
        })
    }
}

impl ToJson for Fracture2dDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("pieces".into(), self.pieces.to_json());
        Value::Object(o)
    }
}

/// `VfxSpawnDesc`: packages/knowledge/wiki/vfx.md.
#[derive(Clone, Debug, PartialEq)]
pub struct VfxSpawnDesc {
    pub asset: u64,
    pub position: Vec3,
    pub rotation: Quat,
    pub parent: Option<u64>,
    pub scale: f32,
    pub tint: Color,
    pub seed: u32,
}

impl Default for VfxSpawnDesc {
    fn default() -> Self {
        Self {
            asset: Default::default(),
            position: Default::default(),
            rotation: Default::default(),
            parent: Default::default(),
            scale: 1.0_f32,
            tint: Color { r: 1.0_f32, g: 1.0_f32, b: 1.0_f32 },
            seed: Default::default(),
        }
    }
}

impl FromJson for VfxSpawnDesc {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            asset: field(o, "asset", d.asset)?,
            position: field(o, "position", d.position)?,
            rotation: field(o, "rotation", d.rotation)?,
            parent: field(o, "parent", d.parent)?,
            scale: field(o, "scale", d.scale)?,
            tint: field(o, "tint", d.tint)?,
            seed: field(o, "seed", d.seed)?,
        })
    }
}

impl ToJson for VfxSpawnDesc {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("asset".into(), self.asset.to_json());
        o.insert("position".into(), self.position.to_json());
        o.insert("rotation".into(), self.rotation.to_json());
        o.insert("parent".into(), self.parent.to_json());
        o.insert("scale".into(), self.scale.to_json());
        o.insert("tint".into(), self.tint.to_json());
        o.insert("seed".into(), self.seed.to_json());
        Value::Object(o)
    }
}

/// `VfxInfo`: packages/knowledge/wiki/vfx.md.
#[derive(Clone, Debug, PartialEq)]
pub struct VfxInfo {
    pub asset: u64,
    pub duration: f32,
    pub r#loop: bool,
    pub events: Vec<String>,
    pub impact: f32,
    pub elements: Vec<String>,
}

impl Default for VfxInfo {
    fn default() -> Self {
        Self {
            asset: Default::default(),
            duration: Default::default(),
            r#loop: Default::default(),
            events: Default::default(),
            impact: Default::default(),
            elements: Default::default(),
        }
    }
}

impl FromJson for VfxInfo {
    fn from_json(v: &Value) -> Result<Self, String> {
        Self::from_json_over(v, Self::default())
    }
    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {
        let o = object(v)?;
        Ok(Self {
            asset: field(o, "asset", d.asset)?,
            duration: field(o, "duration", d.duration)?,
            r#loop: field(o, "loop", d.r#loop)?,
            events: field(o, "events", d.events)?,
            impact: field(o, "impact", d.impact)?,
            elements: field(o, "elements", d.elements)?,
        })
    }
}

impl ToJson for VfxInfo {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("asset".into(), self.asset.to_json());
        o.insert("duration".into(), self.duration.to_json());
        o.insert("loop".into(), self.r#loop.to_json());
        o.insert("events".into(), self.events.to_json());
        o.insert("impact".into(), self.impact.to_json());
        o.insert("elements".into(), self.elements.to_json());
        Value::Object(o)
    }
}

