//! Where a world server keeps its shared objects across restarts
//! (docs/design.md §2.12): [`Store`] is what the server needs of one, and
//! [`FileStore`] keeps them in one file on the server's disk. Another
//! backend (a database, a provider's storage) implements the same trait.
//!
//! The file is an append log of records, each its length (u32 LE) and its
//! bytes: a put (kind 1: world, then the object as it crosses) or a removal
//! (kind 2: world, key). Loading replays it and stops at the first record
//! that is cut short or does not read (a write the process died in), then
//! writes the objects alone to a new file that replaces the old, so the log
//! never grows past one record per change since the last start. The server
//! appends each tick's changes and syncs them once a tick, so a crash loses
//! at most the last tick's.

use crate::protocol::SharedObject;
use crate::shard::Change;
use crate::wire::{Reader, WireError, WireResult, Writer};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// What a world server needs of a store.
pub trait Store: Send {
    /// The objects kept, by world.
    fn load(&mut self) -> io::Result<Vec<(String, SharedObject)>>;
    /// Keep these changes, in order, before returning.
    fn apply(&mut self, changes: &[Change]) -> io::Result<()>;
}

/// Shared objects in one file (`og world-server --store FILE`).
pub struct FileStore {
    path: PathBuf,
    file: Option<File>,
}

const PUT: u8 = 1;
const REMOVE: u8 = 2;

impl FileStore {
    pub fn new(path: impl Into<PathBuf>) -> FileStore {
        FileStore {
            path: path.into(),
            file: None,
        }
    }

    fn record(c: &Change, w: &mut Writer) {
        let mut body = Writer::new();
        match c {
            Change::Put { world, object } => {
                body.u8(PUT);
                body.str(world);
                object.write(&mut body);
            }
            Change::Remove { world, key } => {
                body.u8(REMOVE);
                body.str(world);
                body.str(key);
            }
        }
        w.u32(body.len() as u32);
        w.buf.extend_from_slice(&body.buf);
    }

    fn replay(bytes: &[u8]) -> BTreeMap<(String, String), SharedObject> {
        let mut objects = BTreeMap::new();
        let mut at = 0;
        while bytes.len() - at >= 4 {
            let len = u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
            let Some(body) = bytes.get(at + 4..at + 4 + len) else {
                break;
            };
            let read = |r: &mut Reader| -> WireResult<Change> {
                let kind = r.u8()?;
                let world = r.str(crate::protocol::MAX_NAME)?.to_string();
                let c = match kind {
                    PUT => Change::Put {
                        world,
                        object: SharedObject::read(r)?,
                    },
                    REMOVE => Change::Remove {
                        world,
                        key: r.str(crate::protocol::MAX_KEY)?.to_string(),
                    },
                    _ => return Err(WireError("an unknown store record")),
                };
                if !r.done() {
                    return Err(WireError("bytes after a store record"));
                }
                Ok(c)
            };
            match read(&mut Reader::new(body)) {
                Ok(Change::Put { world, object }) => {
                    objects.insert((world, object.key.clone()), object);
                }
                Ok(Change::Remove { world, key }) => {
                    objects.remove(&(world, key));
                }
                Err(_) => break,
            }
            at += 4 + len;
        }
        objects
    }

    fn sibling(path: &Path, ext: &str) -> PathBuf {
        let mut p = path.as_os_str().to_owned();
        p.push(ext);
        PathBuf::from(p)
    }
}

impl Store for FileStore {
    fn load(&mut self) -> io::Result<Vec<(String, SharedObject)>> {
        let bytes = match std::fs::read(&self.path) {
            Ok(b) => b,
            Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e),
        };
        let objects = FileStore::replay(&bytes);
        // Compact: the objects alone, to a new file that replaces the old.
        let mut w = Writer::new();
        for ((world, _), object) in &objects {
            FileStore::record(
                &Change::Put {
                    world: world.clone(),
                    object: object.clone(),
                },
                &mut w,
            );
        }
        let tmp = FileStore::sibling(&self.path, ".new");
        {
            let mut f = File::create(&tmp)?;
            f.write_all(&w.buf)?;
            f.sync_all()?;
        }
        std::fs::rename(&tmp, &self.path)?;
        self.file = Some(OpenOptions::new().append(true).open(&self.path)?);
        Ok(objects
            .into_iter()
            .map(|((world, _), o)| (world, o))
            .collect())
    }

    fn apply(&mut self, changes: &[Change]) -> io::Result<()> {
        if changes.is_empty() {
            return Ok(());
        }
        if self.file.is_none() {
            self.file = Some(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.path)?,
            );
        }
        let mut w = Writer::new();
        for c in changes {
            FileStore::record(c, &mut w);
        }
        let f = self.file.as_mut().unwrap();
        f.write_all(&w.buf)?;
        f.sync_data()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Place;

    fn door(version: u32, open: u8) -> SharedObject {
        SharedObject {
            key: "door".into(),
            version,
            place: Place::from_world(128.0, [10.0, 0.0, -5.0], 0.5),
            data: vec![open],
        }
    }

    /// What a store kept comes back after a restart, the last change of
    /// each object winning; a record cut short by a crash is dropped and
    /// the rest kept; a removal stays removed.
    #[test]
    fn a_file_store_keeps_the_last_state_across_restarts_and_a_torn_write() {
        let dir = std::env::temp_dir().join(format!("og-store-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("objects.log");
        let _ = std::fs::remove_file(&path);
        let put = |world: &str, o: SharedObject| Change::Put {
            world: world.into(),
            object: o,
        };
        let mut s = FileStore::new(&path);
        assert!(s.load().unwrap().is_empty());
        s.apply(&[put("main", door(1, 0)), put("main", door(2, 1))])
            .unwrap();
        s.apply(&[
            put("other", door(1, 0)),
            Change::Remove {
                world: "other".into(),
                key: "door".into(),
            },
        ])
        .unwrap();
        drop(s);
        // A crash mid-write: half a record at the end.
        let mut torn = Writer::new();
        FileStore::record(&put("main", door(3, 0)), &mut torn);
        let mut f = OpenOptions::new().append(true).open(&path).unwrap();
        f.write_all(&torn.buf[..torn.buf.len() - 3]).unwrap();
        drop(f);
        let mut s = FileStore::new(&path);
        assert_eq!(s.load().unwrap(), vec![("main".to_string(), door(2, 1))]);
        // Compacted, and appending again after it reads back.
        assert_eq!(std::fs::metadata(&path).unwrap().len(), {
            let mut w = Writer::new();
            FileStore::record(&put("main", door(2, 1)), &mut w);
            w.buf.len() as u64
        });
        s.apply(&[put("main", door(3, 0))]).unwrap();
        drop(s);
        assert_eq!(
            FileStore::new(&path).load().unwrap(),
            vec![("main".to_string(), door(3, 0))]
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
