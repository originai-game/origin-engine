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

/// What the block holds.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Store {
    /// Persisted cells' kept entities, by cell (bits).
    pub kept: BTreeMap<u64, Vec<Row>>,
    /// Loads in progress, by cell (bits).
    pub loading: BTreeMap<u64, Loading>,
}

pub(crate) type Shared = Arc<Mutex<Store>>;

pub(crate) fn lock(s: &Shared) -> std::sync::MutexGuard<'_, Store> {
    s.lock().unwrap_or_else(|p| p.into_inner())
}

/// The block registered with a world.
pub(crate) struct Block(pub Shared);

fn put_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn put_u64(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn put_bytes(out: &mut Vec<u8>, b: &[u8]) {
    put_u32(out, b.len() as u32);
    out.extend_from_slice(b);
}

impl Store {
    fn save(&self, out: &mut Vec<u8>) {
        put_u32(out, VERSION);
        put_u32(out, self.kept.len() as u32);
        for (cell, rows) in &self.kept {
            put_u64(out, *cell);
            put_u32(out, rows.len() as u32);
            for r in rows {
                put_u64(out, r.id);
                put_bytes(out, r.name.as_bytes());
                put_u32(out, r.components.len() as u32);
                for (ty, hash, bytes) in &r.components {
                    put_bytes(out, ty.as_bytes());
                    put_u64(out, *hash);
                    put_bytes(out, bytes);
                }
            }
        }
        put_u32(out, self.loading.len() as u32);
        for (cell, l) in &self.loading {
            put_u64(out, *cell);
            out.push(l.source as u8);
            put_u32(out, l.spawned.len() as u32);
            for e in &l.spawned {
                put_u64(out, *e);
            }
        }
    }

    fn read(bytes: &[u8]) -> Result<Store> {
        let mut r = Reader { bytes, at: 0 };
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
                let name = r.str()?;
                let k = r.u32()?;
                let mut components = Vec::with_capacity(k.min(256) as usize);
                for _ in 0..k {
                    let ty = r.str()?;
                    let hash = r.u64()?;
                    components.push((ty, hash, r.bytes()?.to_vec()));
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
        if r.at != bytes.len() {
            return Err(bad("has bytes after its end"));
        }
        Ok(s)
    }
}

fn bad(why: &str) -> Error {
    Error::new(ErrorKind::Parse, format!("{BLOCK} {why}"))
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let out = self
            .bytes
            .get(self.at..self.at + n)
            .ok_or_else(|| bad("ends early"))?;
        self.at += n;
        Ok(out)
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().expect("4")))
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().expect("8")))
    }
    fn bytes(&mut self) -> Result<&'a [u8]> {
        let n = self.u32()? as usize;
        self.take(n)
    }
    fn str(&mut self) -> Result<String> {
        std::str::from_utf8(self.bytes()?)
            .map(str::to_string)
            .map_err(|_| bad("holds a name that is not UTF-8"))
    }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_store_round_trips_byte_for_byte_and_refuses_damage() {
        let mut s = Store::default();
        s.kept.insert(
            7,
            vec![
                Row {
                    id: 1 << 32 | 9,
                    name: "well".into(),
                    components: vec![("og.Transform".into(), 42, vec![1, 2, 3])],
                },
                Row {
                    id: 1 << 32 | 10,
                    name: String::new(),
                    components: Vec::new(),
                },
            ],
        );
        s.loading.insert(
            3,
            Loading {
                source: Source::Kept,
                spawned: vec![5, 6],
            },
        );
        let mut out = Vec::new();
        s.save(&mut out);
        assert_eq!(Store::read(&out).unwrap(), s);
        let mut again = Vec::new();
        Store::read(&out).unwrap().save(&mut again);
        assert_eq!(again, out);
        assert!(Store::read(&out[..out.len() - 1]).is_err());
        let mut long = out.clone();
        long.push(0);
        assert!(Store::read(&long).is_err());
        let mut empty = Vec::new();
        Store::default().save(&mut empty);
        assert_eq!(Store::read(&empty).unwrap(), Store::default());
    }
}
