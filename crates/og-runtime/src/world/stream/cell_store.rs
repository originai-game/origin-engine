//! The `og.cells` state block (G3-3): what the world's cells keep outside
//! their entities, so a snapshot, a save, the hash and a rollback carry it.
//!
//! - **Persisted cells** (`StreamCell.persist`): when one unloads, its
//!   members and their descendants are kept here as rows, each an entity's
//!   saved id, name and components (type name, layout hash, packed bytes),
//!   parents before their children. When it loads again the rows spawn
//!   instead of its sub-scene, entity fields pointing into the rows
//!   remapped to the new ids, and the rows are dropped once the last one
//!   spawned (the entities are in the world again).
//! - **Loads in progress** (`StreamCell.budget`): a cell that spawns over
//!   several ticks keeps the ids it spawned so far, in spawn order, so the
//!   next tick (or the tick after a restore) goes on from there.
//!
//! The block is added with the cell's records (og-runtime's `lazy`), so a
//! world without cells keeps its hash, and a restore of a save that has
//! cells registers them, and with them the block, first. Its bytes are a
//! function of its state: cells by entity, rows in order.

use og_core::block_form::{BlockForm, Field, Record, Records};
use og_core::bytes::{ByteReader, ByteWriter};
use og_core::{Error, ErrorKind, Result};
use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// The block's name in snapshots.
pub(crate) const BLOCK: &str = "og.cells";

/// The layout of the block's bytes.
const VERSION: u32 = 1;

/// One kept entity.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Row {
    /// Its id when it was kept (what other rows' entity fields hold).
    pub id: u64,
    /// Its name ("" for none).
    pub name: String,
    /// Its components: type name, the type's layout hash, packed bytes.
    pub components: Vec<(String, u64, Vec<u8>)>,
}

/// Where a load in progress spawns from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Source {
    /// The cell's sub-scene, as authored.
    Scene = 0,
    /// The cell's kept rows.
    Kept = 1,
}

/// A cell's load in progress.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Loading {
    pub source: Source,
    /// The entities spawned so far, in spawn order (bits).
    pub spawned: Vec<u64>,
}

/// Failed spawns retry on fixed ticks, at most eight times per scene.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Retry {
    pub scene: u64,
    pub attempts: u32,
    pub next: u64,
}

/// What the block holds.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Store {
    /// Persisted cells' kept entities, by cell (bits).
    pub kept: BTreeMap<u64, Vec<Row>>,
    /// Loads in progress, by cell (bits).
    pub loading: BTreeMap<u64, Loading>,
    pub retries: BTreeMap<u64, Retry>,
}

pub(crate) type Shared = Arc<Mutex<Store>>;

pub(crate) fn lock(s: &Shared) -> std::sync::MutexGuard<'_, Store> {
    s.lock().unwrap_or_else(|p| p.into_inner())
}

/// The block registered with a world.
pub(crate) struct Block(pub Shared);

impl Store {
    fn save(&self, out: &mut Vec<u8>) {
        let mut w = ByteWriter::new(out);
        w.u32(VERSION);
        w.u32(self.kept.len() as u32);
        for (cell, rows) in &self.kept {
            w.u64(*cell);
            w.u32(rows.len() as u32);
            for r in rows {
                w.u64(r.id);
                w.str(&r.name);
                w.u32(r.components.len() as u32);
                for (ty, hash, bytes) in &r.components {
                    w.str(ty);
                    w.u64(*hash);
                    w.sized(bytes);
                }
            }
        }
        w.u32(self.loading.len() as u32);
        for (cell, l) in &self.loading {
            w.u64(*cell);
            w.u8(l.source as u8);
            w.u32(l.spawned.len() as u32);
            for e in &l.spawned {
                w.u64(*e);
            }
        }
        // Keep the bytes (and hashes) of error-free worlds unchanged.
        if !self.retries.is_empty() {
            w.u32(0x52545259); // RTRY: optional extension of version 1.
            w.u32(self.retries.len() as u32);
            for (cell, retry) in &self.retries {
                w.u64(*cell);
                w.u64(retry.scene);
                w.u32(retry.attempts);
                w.u64(retry.next);
            }
        }
    }

    fn read(bytes: &[u8]) -> Result<Store> {
        let mut r = ByteReader::new(bytes, BLOCK);
        let version = r.u32()?;
        if version != VERSION {
            return Err(bad(&format!("has version {version}, not {VERSION}")));
        }
        let mut s = Store::default();
        for _ in 0..r.u32()? {
            let cell = r.u64()?;
            let n = r.u32()?;
            let mut rows = Vec::with_capacity(n.min(1 << 16) as usize);
            for _ in 0..n {
                let id = r.u64()?;
                let name = r.string()?;
                let k = r.u32()?;
                let mut components = Vec::with_capacity(k.min(256) as usize);
                for _ in 0..k {
                    let ty = r.string()?;
                    let hash = r.u64()?;
                    components.push((ty, hash, r.sized()?.to_vec()));
                }
                rows.push(Row {
                    id,
                    name,
                    components,
                });
            }
            s.kept.insert(cell, rows);
        }
        for _ in 0..r.u32()? {
            let cell = r.u64()?;
            let source = match r.take(1)?[0] {
                0 => Source::Scene,
                1 => Source::Kept,
                x => return Err(bad(&format!("names load source {x}"))),
            };
            let n = r.u32()?;
            let mut spawned = Vec::with_capacity(n.min(1 << 16) as usize);
            for _ in 0..n {
                spawned.push(r.u64()?);
            }
            s.loading.insert(cell, Loading { source, spawned });
        }
        if !r.is_done() {
            if r.u32()? != 0x52545259 {
                return Err(bad("has an unknown extension"));
            }
            for _ in 0..r.u32()? {
                let cell = r.u64()?;
                s.retries.insert(
                    cell,
                    Retry {
                        scene: r.u64()?,
                        attempts: r.u32()?,
                        next: r.u64()?,
                    },
                );
            }
        }
        if !r.is_done() {
            return Err(bad("has bytes after its end"));
        }
        Ok(s)
    }
}

fn bad(why: &str) -> Error {
    Error::new(ErrorKind::Parse, format!("{BLOCK} {why}"))
}

impl og_ecs::StateBlock for Block {
    fn save(&self, out: &mut Vec<u8>) {
        lock(&self.0).save(out);
    }

    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>> {
        Ok(Box::new(Store::read(bytes)?))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let next = decoded
            .and_then(|d| d.downcast::<Store>().ok())
            .map(|b| *b)
            .unwrap_or_default();
        *lock(&self.0) = next;
    }

    /// Per cell: each kept row (its saved id an entity; its components'
    /// packed bytes as they are), its load in progress and its retry.
    fn form(&self, bytes: &[u8]) -> Result<BlockForm> {
        let s = Store::read(bytes)?;
        let mut out = Records::default();
        for (&cell, rows) in &s.kept {
            for (i, r) in rows.iter().enumerate() {
                out.entities.push(
                    Record::new(cell, "kept")
                        .key(Field::of(&i))
                        .field("id", Field::Entity(r.id))
                        .field("name", Field::of(&r.name))
                        .field("components", Field::of(&r.components)),
                );
            }
        }
        for (&cell, l) in &s.loading {
            out.entities.push(
                Record::new(cell, "loading")
                    .field("source", Field::of(&(l.source as u8)))
                    .field("spawned", Field::Entities(l.spawned.clone())),
            );
        }
        for (&cell, r) in &s.retries {
            out.entities.push(
                Record::new(cell, "retry")
                    .field("scene", Field::of(&r.scene))
                    .field("attempts", Field::of(&r.attempts))
                    .field("next", Field::of(&r.next)),
            );
        }
        Ok(BlockForm::Entities(out))
    }

    /// R12's origin shift: the kept entities' places move as they would
    /// in the world (a row with og.Parent is a child; its parent's frame
    /// moved with the parent). A load in progress keeps ids only.
    fn shift(&mut self, shift: &og_ecs::Shift) {
        let mut s = lock(&self.0);
        for row in s.kept.values_mut().flatten() {
            let child = row.components.iter().any(|c| c.0 == "og.Parent");
            for (name, hash, bytes) in &mut row.components {
                shift.kept_value(name, *hash, bytes, child);
            }
        }
    }
}
