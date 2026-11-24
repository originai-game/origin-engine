//! Named presentation sources. Registration is idempotent across TS reloads;
//! writes are explicit, so consumers never need to compare whole viewmodels.

pub mod bindings;
pub mod graph;
pub mod session;
pub mod wire;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// A source's identity is independent of registration order and host handles.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourceId {
    pub scope: String,
    pub id: String,
}

impl SourceId {
    pub fn new(scope: &str, id: &str) -> Self {
        Self {
            scope: scope.into(),
            id: id.into(),
        }
    }

    fn error(&self, reason: &str) -> String {
        format!(
            "og.ui scope '{}' source '{}': {reason}",
            self.scope, self.id
        )
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.scope.is_empty() || self.id.is_empty() {
            return Err(self.error("scope and source ID must be nonempty"));
        }
        Ok(())
    }
}

/// A keyed collection's committed edit. Measurement/virtualization consumers
/// keep only keys and extents; the collection remains the owner of row data.
/// Positions are zero-based in this edit's pre/post order, not identities.
/// Notifications retain removed keys and old ranks until drained.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CollectionEdit {
    Insert {
        key: String,
        before: Option<String>,
        index: usize,
    },
    Remove {
        key: String,
        index: usize,
    },
    Move {
        key: String,
        before: Option<String>,
        from: usize,
        to: usize,
    },
    Patch {
        key: String,
        fields: Vec<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SourceChange {
    Signal,
    Collection(CollectionEdit),
}

/// Ordered notifications, separate from saved values. Consumers drain them
/// once at the tick boundary; restoring a snapshot requests a fresh build.
#[derive(Clone, Debug, PartialEq)]
pub struct Change {
    pub source: SourceId,
    pub revision: u64,
    pub change: SourceChange,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Source {
    Signal {
        value: Value,
        revision: u64,
        readable: bool,
        value_type: ValueType,
    },
    Collection(KeyedCollection),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum ValueType {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

impl ValueType {
    fn of(value: &Value) -> Self {
        match value {
            Value::Null => Self::Null,
            Value::Bool(_) => Self::Bool,
            Value::Number(_) => Self::Number,
            Value::String(_) => Self::String,
            Value::Array(_) => Self::Array,
            Value::Object(_) => Self::Object,
        }
    }
}

impl Source {
    fn same_type(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Signal {
                    readable: a,
                    value_type: x,
                    ..
                },
                Self::Signal {
                    readable: b,
                    value_type: y,
                    ..
                },
            ) => a == b && x == y,
            (Self::Collection(a), Self::Collection(b)) => a.key_field == b.key_field,
            _ => false,
        }
    }
}

/// Keys, order and records owned by one source. Keys cannot change in a patch.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyedCollection {
    key_field: String,
    keys: Vec<String>,
    rows: BTreeMap<String, Map<String, Value>>,
    revision: u64,
}

impl KeyedCollection {
    /// The field of each row that holds its key.
    pub fn key_field(&self) -> &str {
        &self.key_field
    }
    pub fn keys(&self) -> &[String] {
        &self.keys
    }
    pub fn row(&self, key: &str) -> Option<&Map<String, Value>> {
        self.rows.get(key)
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }

    fn destination(&self, source: &SourceId, before: Option<&str>) -> Result<usize, String> {
        match before {
            None => Ok(self.keys.len()),
            Some(key) => self
                .keys
                .iter()
                .position(|k| k == key)
                .ok_or_else(|| source.error(&format!("destination key '{key}' does not exist"))),
        }
    }
}

/// Engine-owned source values, included in the UI checkpoint. A declaration
/// with the same identity and kind keeps its value; an incompatible declaration
/// is refused without changing the old source.
#[derive(Clone, Debug, Default)]
pub struct Sources {
    sources: BTreeMap<SourceId, Source>,
    changes: Vec<Change>,
}

impl Sources {
    /// The sources declared in `scope`.
    pub fn in_scope<'a>(&'a self, scope: &'a str) -> impl Iterator<Item = &'a SourceId> {
        self.sources.keys().filter(move |id| id.scope == scope)
    }

    pub fn signal(&mut self, source: SourceId, initial: Value) -> Result<(), String> {
        self.declare_signal(source, initial, false)
    }

    fn declare_signal(
        &mut self,
        source: SourceId,
        initial: Value,
        readable: bool,
    ) -> Result<(), String> {
        source.validate()?;
        match self.sources.get(&source) {
            Some(Source::Signal {
                value_type,
                readable: previous,
                ..
            }) if *previous == readable && *value_type == ValueType::of(&initial) => Ok(()),
            Some(Source::Signal { .. }) => {
                Err(source.error("already registered with a different store type"))
            }
            Some(_) => Err(source.error("already registered as a collection")),
            None => {
                self.sources.insert(
                    source,
                    Source::Signal {
                        value_type: ValueType::of(&initial),
                        value: initial,
                        revision: 0,
                        readable,
                    },
                );
                Ok(())
            }
        }
    }

    pub fn collection(&mut self, source: SourceId, key_field: &str) -> Result<(), String> {
        source.validate()?;
        let key_field = if key_field.is_empty() {
            "id"
        } else {
            key_field
        };
        match self.sources.get(&source) {
            Some(Source::Collection(c)) if c.key_field == key_field => Ok(()),
            Some(_) => Err(source.error("already registered with a different kind or key field")),
            None => {
                self.sources.insert(
                    source,
                    Source::Collection(KeyedCollection {
                        key_field: key_field.into(),
                        keys: Vec::new(),
                        rows: BTreeMap::new(),
                        revision: 0,
                    }),
                );
                Ok(())
            }
        }
    }

    pub fn read(&self, source: &SourceId) -> Result<&Value, String> {
        match self.sources.get(source) {
            Some(Source::Signal { value, .. }) => Ok(value),
            _ => Err(source.error("signal does not exist")),
        }
    }

    pub fn keyed(&self, source: &SourceId) -> Result<&KeyedCollection, String> {
        match self.sources.get(source) {
            Some(Source::Collection(c)) => Ok(c),
            _ => Err(source.error("collection does not exist")),
        }
    }

    fn keyed_mut(&mut self, source: &SourceId) -> Result<&mut KeyedCollection, String> {
        match self.sources.get_mut(source) {
            Some(Source::Collection(c)) => Ok(c),
            _ => Err(source.error("collection does not exist")),
        }
    }

    pub fn set(&mut self, source: &SourceId, next: Value) -> Result<(), String> {
        self.set_value(source, next, false)
    }

    fn set_value(&mut self, source: &SourceId, next: Value, force: bool) -> Result<(), String> {
        let Some(Source::Signal {
            value, revision, ..
        }) = self.sources.get_mut(source)
        else {
            return Err(source.error("signal does not exist"));
        };
        if force || *value != next {
            *value = next;
            *revision += 1;
            self.changes.push(Change {
                source: source.clone(),
                revision: *revision,
                change: SourceChange::Signal,
            });
        }
        Ok(())
    }

    pub fn insert(
        &mut self,
        source: &SourceId,
        row: Map<String, Value>,
        before: Option<&str>,
    ) -> Result<(), String> {
        let c = self.keyed_mut(source)?;
        let key = row
            .get(&c.key_field)
            .and_then(Value::as_str)
            .filter(|k| !k.is_empty())
            .ok_or_else(|| {
                source.error(&format!(
                    "row field '{}' must be a nonempty string key",
                    c.key_field
                ))
            })?
            .to_owned();
        if c.rows.contains_key(&key) {
            return Err(source.error(&format!("duplicate key '{key}'")));
        }
        let index = c.destination(source, before)?;
        c.keys.insert(index, key.clone());
        c.rows.insert(key.clone(), row);
        self.note_collection(
            source,
            CollectionEdit::Insert {
                key,
                before: before.map(str::to_owned),
                index,
            },
        );
        Ok(())
    }

    pub fn remove(&mut self, source: &SourceId, key: &str) -> Result<(), String> {
        let c = self.keyed_mut(source)?;
        let index = c.destination(source, Some(key))?;
        c.keys.remove(index);
        c.rows.remove(key);
        self.note_collection(
            source,
            CollectionEdit::Remove {
                key: key.into(),
                index,
            },
        );
        Ok(())
    }

    pub fn move_before(
        &mut self,
        source: &SourceId,
        key: &str,
        before: Option<&str>,
    ) -> Result<(), String> {
        let c = self.keyed_mut(source)?;
        let from = c.destination(source, Some(key))?;
        let to = c.destination(source, before)?;
        if from == to || from + 1 == to {
            return Ok(());
        }
        let key = c.keys.remove(from);
        let to = if to > from { to - 1 } else { to };
        c.keys.insert(to, key.clone());
        self.note_collection(
            source,
            CollectionEdit::Move {
                key,
                before: before.map(str::to_owned),
                from,
                to,
            },
        );
        Ok(())
    }

    pub fn patch(
        &mut self,
        source: &SourceId,
        key: &str,
        fields: Map<String, Value>,
    ) -> Result<(), String> {
        let c = self.keyed_mut(source)?;
        if fields
            .get(&c.key_field)
            .is_some_and(|v| v.as_str() != Some(key))
        {
            return Err(source.error(&format!("patch cannot change key '{key}'")));
        }
        let row = c
            .rows
            .get_mut(key)
            .ok_or_else(|| source.error(&format!("key '{key}' does not exist")))?;
        let mut changed = Vec::new();
        for (field, value) in fields {
            if row.get(&field) != Some(&value) {
                changed.push(field.clone());
                row.insert(field, value);
            }
        }
        if !changed.is_empty() {
            self.note_collection(
                source,
                CollectionEdit::Patch {
                    key: key.into(),
                    fields: changed,
                },
            );
        }
        Ok(())
    }

    fn note_collection(&mut self, source: &SourceId, edit: CollectionEdit) {
        // Only called after a successful collection operation.
        if let Some(Source::Collection(c)) = self.sources.get_mut(source) {
            c.revision += 1;
            self.changes.push(Change {
                source: source.clone(),
                revision: c.revision,
                change: SourceChange::Collection(edit),
            });
        }
    }

    pub fn take_changes(&mut self) -> Vec<Change> {
        std::mem::take(&mut self.changes)
    }

    pub fn save(&self) -> Value {
        serde_json::json!(self.sources.iter().collect::<Vec<_>>())
    }

    /// Validate a checkpoint before replacing live values. Pending change
    /// notifications are derived; the restoring UI invalidates its consumers.
    pub fn restore(&mut self, value: Value) -> Result<(), String> {
        let entries: Vec<(SourceId, Source)> = serde_json::from_value(value)
            .map_err(|e| format!("og.ui sources: invalid checkpoint: {e}"))?;
        let mut restored = Self::default();
        for (id, source) in entries {
            id.validate()?;
            if restored.sources.contains_key(&id) {
                return Err(id.error("duplicate source in checkpoint"));
            }
            if let Source::Collection(c) = &source {
                let mut seen = std::collections::BTreeSet::new();
                for key in &c.keys {
                    if key.is_empty()
                        || !seen.insert(key)
                        || c.rows
                            .get(key)
                            .and_then(|row| row.get(&c.key_field))
                            .and_then(Value::as_str)
                            != Some(key)
                    {
                        return Err(id.error("invalid collection keys in checkpoint"));
                    }
                }
                if c.key_field.is_empty() || c.keys.len() != c.rows.len() {
                    return Err(id.error("invalid collection rows in checkpoint"));
                }
            }
            restored.sources.insert(id, source);
        }
        *self = restored;
        Ok(())
    }
}
