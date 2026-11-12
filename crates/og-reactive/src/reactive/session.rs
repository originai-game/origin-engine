//! The host-independent operations behind the function-call authoring API.

use super::{
    SourceChange, SourceId, Sources, ValueType,
    graph::{Computation, Dependency, Graph},
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
pub enum Command {
    MountInputs,
    Mount {
        mount: SourceId,
        document: String,
        inputs: BTreeMap<String, SourceId>,
        /// Content passed by reference: `document` or `document#snippet`.
        #[serde(default)]
        content: BTreeMap<String, String>,
    },
    Unmount {
        mount: SourceId,
    },
    Signal {
        source: SourceId,
        initial: Value,
        #[serde(default)]
        readable: bool,
    },
    Collection {
        source: SourceId,
        #[serde(default)]
        key: String,
        #[serde(default)]
        items: Vec<Map<String, Value>>,
    },
    Memo {
        source: SourceId,
        inputs: Option<Vec<SourceId>>,
        initial: Option<Value>,
        #[serde(default)]
        inactive: bool,
    },
    Effect {
        source: SourceId,
    },
    Read {
        source: SourceId,
    },
    Set {
        source: SourceId,
        value: Value,
        #[serde(default)]
        force: bool,
    },
    Insert {
        source: SourceId,
        row: Map<String, Value>,
        before: Option<String>,
    },
    Remove {
        source: SourceId,
        key: String,
    },
    Move {
        source: SourceId,
        key: String,
        before: Option<String>,
    },
    Patch {
        source: SourceId,
        key: String,
        fields: Map<String, Value>,
    },
    Keys {
        source: SourceId,
    },
    Row {
        source: SourceId,
        key: String,
    },
    Field {
        source: SourceId,
        key: String,
        field: String,
    },
    Begin {
        source: SourceId,
    },
    Finish {
        source: SourceId,
        value: Value,
        error: Option<String>,
    },
    Next,
    BatchBegin,
    BatchEnd,
    Track {
        enabled: bool,
    },
    Fail {
        reason: String,
    },
    Dispose {
        scope: String,
    },
    CheckIdle {
        source: SourceId,
    },
    Suspend {
        source: SourceId,
    },
    Forget {
        source: SourceId,
    },
    RemoveSource {
        source: SourceId,
    },
    Registered {
        sources: Vec<SourceId>,
        #[serde(default)]
        mounts: Vec<SourceId>,
    },
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Registration {
    sources: BTreeSet<SourceId>,
}

#[derive(Clone, Debug, Default)]
pub struct Session {
    pub sources: Sources,
    defaults: Sources,
    pub graph: Graph,
    computed: BTreeMap<SourceId, Option<Value>>,
    registration: Option<Registration>,
}

impl Session {
    /// Borrow a scalar store or settled memo; collections are read by key.
    pub fn read_value(&self, source: &SourceId) -> Result<&Value, String> {
        if self.graph.evaluation_needed(source)?.is_some() {
            return Err(source.error("store callback must settle before document evaluation"));
        }
        if let Some(value) = self.computed.get(source) {
            return value
                .as_ref()
                .ok_or_else(|| source.error("memo has no successful value"));
        }
        self.sources.read(source)
    }

    /// A document model can write a writable store, never a readable producer
    /// or a derived value. The generic Set command also serves producers.
    pub fn write_model(
        &mut self,
        source: &SourceId,
        path: &str,
        value: Value,
    ) -> Result<(), String> {
        let Some(super::Source::Signal {
            value: current,
            readable: false,
            ..
        }) = self.sources.sources.get(source)
        else {
            return Err(source.error("model requires a writable store"));
        };
        let mut next = current.clone();
        if !crate::bind::set_path(&mut next, path, value) {
            return Err(source.error(&format!("model cannot write path '{path}'")));
        }
        self.execute(Command::Set {
            source: source.clone(),
            value: next,
            force: false,
        })?;
        Ok(())
    }

    pub fn save(&self) -> Value {
        json!({
            "sources": self.sources.save(),
            "computed": self.computed.iter().collect::<Vec<_>>(),
            "registration": self.registration,
        })
    }

    pub fn restore(&mut self, value: &Value) -> Result<(), String> {
        let mut sources = Sources::default();
        sources.restore(value["sources"].clone())?;
        let computed: Vec<(SourceId, Option<Value>)> =
            serde_json::from_value(value["computed"].clone())
                .map_err(|e| format!("og.ui computed checkpoint: {e}"))?;
        let saved_registration: Option<Registration> =
            serde_json::from_value(value["registration"].clone())
                .map_err(|e| format!("og.ui registration checkpoint: {e}"))?;
        let mut restored_computed = BTreeMap::new();
        for (id, value) in computed {
            id.validate()?;
            if sources.sources.contains_key(&id)
                || restored_computed.insert(id.clone(), value).is_some()
            {
                return Err(id.error("duplicate or incompatible computed store in checkpoint"));
            }
        }
        if let Some(current) = &self.registration {
            if let Some(saved) = &saved_registration {
                for removed in saved.sources.difference(&current.sources) {
                    sources.sources.remove(removed);
                    restored_computed.remove(removed);
                }
            }
            for added in &current.sources {
                if !sources.sources.contains_key(added) && !restored_computed.contains_key(added) {
                    if let Some(source) = self.defaults.sources.get(added) {
                        sources.sources.insert(added.clone(), source.clone());
                    }
                    if let Some(value) = self.computed.get(added) {
                        restored_computed.insert(added.clone(), value.clone());
                    }
                }
            }
        }
        let mut graph = self.graph.clone();
        for (id, restored) in &sources.sources {
            if let Some(current) = self.sources.sources.get(id)
                && !current.same_type(restored)
            {
                return Err(id.error("checkpoint has a different store type"));
            }
        }
        for (id, restored) in &restored_computed {
            if let (Some(Some(current)), Some(restored)) = (self.computed.get(id), restored)
                && ValueType::of(current) != ValueType::of(restored)
            {
                return Err(id.error("checkpoint has a different derived store type"));
            }
        }
        graph.restore_sources(sources.sources.keys(), restored_computed.keys())?;
        self.sources = sources;
        self.computed = restored_computed;
        self.graph = graph;
        Ok(())
    }

    /// Executes one operation. The caller runs returned computation IDs outside
    /// the engine lock, then supplies their result with Finish. The graph never
    /// calls into JS while borrowed and never stores a JS function.
    pub fn execute(&mut self, command: Command) -> Result<Value, String> {
        let before = self.sources.changes.len();
        let result = self.run(command).and_then(|value| {
            for change in self.sources.changes.get(before..).unwrap_or_default() {
                match &change.change {
                    SourceChange::Signal => self.graph.changed(&change.source)?,
                    SourceChange::Collection(edit) => {
                        self.graph.collection_changed(&change.source, edit)?
                    }
                }
            }
            Ok(value)
        });
        if let Err(reason) = &result {
            self.graph.fail(reason);
        }
        result
    }

    pub(super) fn value(&self, source: &SourceId) -> Result<Value, String> {
        if let Some(value) = self.computed.get(source) {
            return value
                .clone()
                .ok_or_else(|| source.error("memo has no successful value"));
        }
        if let Ok(collection) = self.sources.keyed(source) {
            return Ok(Value::Array(
                collection
                    .keys()
                    .iter()
                    .map(|key| json!(collection.row(key)))
                    .collect(),
            ));
        }
        self.sources.read(source).cloned()
    }

    fn run(&mut self, command: Command) -> Result<Value, String> {
        match command {
            Command::Signal {
                source,
                initial,
                readable,
            } => {
                self.graph.check_registration(&source, Computation::Input)?;
                self.sources
                    .declare_signal(source.clone(), initial.clone(), readable)?;
                self.defaults
                    .declare_signal(source.clone(), initial, readable)?;
                self.graph.register(source.clone(), Computation::Input)?;
            }
            Command::Collection { source, key, items } => {
                // Validate a new collection as a unit; duplicate initial keys
                // must not leave a partially registered source behind.
                let mut fresh = Sources::default();
                if !self.sources.sources.contains_key(&source) {
                    fresh.collection(source.clone(), &key)?;
                    for row in items {
                        fresh.insert(&source, row, None)?;
                    }
                }
                self.graph.register(source.clone(), Computation::Input)?;
                self.sources.collection(source.clone(), &key)?;
                if let Some(value) = fresh.sources.remove(&source) {
                    self.defaults.sources.insert(source.clone(), value.clone());
                    self.sources.sources.insert(source, value);
                }
            }
            Command::Memo {
                source,
                inputs,
                initial,
                inactive,
            } => {
                if let (Some(Some(current)), Some(initial)) = (self.computed.get(&source), &initial)
                    && ValueType::of(current) != ValueType::of(initial)
                {
                    return Err(
                        source.error("already registered with a different derived store type")
                    );
                }
                self.graph.register(source.clone(), Computation::Memo)?;
                self.graph.declare_reads(&source, inputs);
                if inactive {
                    self.graph.suspend(&source)?;
                }
                self.computed.entry(source).or_insert(initial);
            }
            Command::Effect { source } => {
                self.graph.register(source.clone(), Computation::Effect)?;
            }
            Command::Read { source } => {
                self.graph.read(&source)?;
                if let Some(next) = self.graph.evaluation_needed(&source)? {
                    return Ok(json!({"evaluate": next}));
                }
                let value = self.value(&source)?;
                return Ok(json!({"value": value}));
            }
            Command::Set {
                source,
                value,
                force,
            } => {
                self.graph.can_write(&source)?;
                self.sources.set_value(&source, value, force)?;
            }
            Command::Insert {
                source,
                row,
                before,
            } => {
                self.graph.can_write(&source)?;
                self.sources.insert(&source, row, before.as_deref())?;
            }
            Command::Remove { source, key } => {
                self.graph.can_write(&source)?;
                self.sources.remove(&source, &key)?;
            }
            Command::Move {
                source,
                key,
                before,
            } => {
                self.graph.can_write(&source)?;
                self.sources.move_before(&source, &key, before.as_deref())?;
            }
            Command::Patch {
                source,
                key,
                fields,
            } => {
                self.graph.can_write(&source)?;
                self.sources.patch(&source, &key, fields)?;
            }
            Command::Keys { source } => {
                self.graph
                    .read_dependency(Dependency::Keys(source.clone()))?;
                return Ok(json!(self.sources.keyed(&source)?.keys()));
            }
            Command::Row { source, key } => {
                self.graph
                    .read_dependency(Dependency::Row(source.clone(), key.clone()))?;
                return self
                    .sources
                    .keyed(&source)?
                    .row(&key)
                    .map(|r| json!(r))
                    .ok_or_else(|| source.error(&format!("key '{key}' does not exist")));
            }
            Command::Field { source, key, field } => {
                self.graph.read_dependency(Dependency::Field(
                    source.clone(),
                    key.clone(),
                    field.clone(),
                ))?;
                return Ok(self
                    .sources
                    .keyed(&source)?
                    .row(&key)
                    .and_then(|row| row.get(&field))
                    .cloned()
                    .unwrap_or(Value::Null));
            }
            Command::Begin { source } => {
                self.graph.begin(&source)?;
            }
            Command::Finish {
                source,
                value,
                error,
            } => {
                let error = error.or_else(|| {
                    self.computed
                        .get(&source)
                        .and_then(Option::as_ref)
                        .filter(|current| ValueType::of(current) != ValueType::of(&value))
                        .map(|_| source.error("computed value has a different derived store type"))
                });
                let changed = self.computed.get(&source).is_some_and(|v| {
                    value.is_object() || value.is_array() || v.as_ref() != Some(&value)
                });
                self.graph.finish(&source, error.map_or(Ok(changed), Err))?;
                if let Some(cached) = self.computed.get_mut(&source) {
                    *cached = Some(value);
                }
            }
            Command::MountInputs | Command::Mount { .. } | Command::Unmount { .. } => {
                return Err("document operations require a UI host".into());
            }
            Command::Next => return Ok(json!(self.graph.next())),
            Command::BatchBegin => self.graph.begin_batch(),
            Command::BatchEnd => self.graph.end_batch()?,
            Command::Track { enabled } => return Ok(json!(self.graph.track(enabled))),
            Command::Fail { reason } => self.graph.fail(&reason),
            Command::Dispose { scope } => {
                self.graph.dispose(&scope)?;
                self.sources.sources.retain(|id, _| id.scope != scope);
                self.defaults.sources.retain(|id, _| id.scope != scope);
                if let Some(registration) = &mut self.registration {
                    registration.sources.retain(|id| id.scope != scope);
                }
                self.sources
                    .changes
                    .retain(|change| change.source.scope != scope);
                self.computed.retain(|id, _| id.scope != scope);
            }
            Command::CheckIdle { source } => self.graph.check_idle(&source)?,
            Command::Suspend { source } => self.graph.suspend(&source)?,
            Command::Forget { source } => {
                self.graph.forget(&source)?;
                self.computed.remove(&source);
            }
            Command::RemoveSource { source } => {
                self.graph.forget(&source)?;
                self.sources.sources.remove(&source);
                self.defaults.sources.remove(&source);
                self.sources
                    .changes
                    .retain(|change| change.source != source);
                self.computed.remove(&source);
            }
            Command::Registered { sources, .. } => {
                self.graph
                    .check_idle(&SourceId::new("registration", "complete"))?;
                self.registration = Some(Registration {
                    sources: sources.into_iter().collect(),
                });
            }
        }
        Ok(Value::Null)
    }

    /// Each result corresponds to its operation; a refused operation changes
    /// nothing and does not discard successful earlier operations in the batch.
    pub fn apply(&mut self, bytes: &[u8]) -> Vec<u8> {
        Self::apply_with(bytes, |command| self.execute(command))
    }

    pub fn apply_with(
        bytes: &[u8],
        mut execute: impl FnMut(Command) -> Result<Value, String>,
    ) -> Vec<u8> {
        let decoded = super::wire::decode(bytes).and_then(|v| {
            serde_json::from_value::<Vec<Command>>(v).map_err(|e| format!("UI patch batch: {e}"))
        });
        let result = match decoded {
            Ok(commands) => Value::Array(
                commands
                    .into_iter()
                    .map(|command| match execute(command) {
                        Ok(value) => json!({"ok": true, "value": value}),
                        Err(reason) => json!({"ok": false, "reason": reason}),
                    })
                    .collect(),
            ),
            Err(reason) => json!({"ok": false, "reason": reason}),
        };
        super::wire::encode(&result).unwrap_or_else(|reason| {
            super::wire::encode(&json!({"ok": false, "reason": reason}))
                .expect("bounded error reply")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(name: &str) -> SourceId {
        SourceId::new("hud", name)
    }

    fn command(session: &mut Session, value: Value) -> Result<Value, String> {
        session.execute(serde_json::from_value(value).unwrap())
    }

    fn read(session: &mut Session, name: &str) -> Value {
        command(session, json!({"op": "read", "source": id(name)})).unwrap()
    }

    fn finish(session: &mut Session, name: &str, value: Value) -> Result<Value, String> {
        command(
            session,
            json!({"op": "finish", "source": id(name), "value": value, "error": null}),
        )
    }

    #[test]
    fn collection_bindings_invalidate_only_the_order_row_or_field_they_read() {
        let mut session = Session::default();
        command(
            &mut session,
            json!({"op": "collection", "source": id("items"), "items": [
                {"id": "a", "count": 3, "name": "Amber"}, {"id": "b", "count": 7, "name": "Blue"}
            ]}),
        )
        .unwrap();
        let bindings = [
            ("order", json!({"op": "keys", "source": id("items")})),
            (
                "row",
                json!({"op": "row", "source": id("items"), "key": "a"}),
            ),
            (
                "count",
                json!({"op": "field", "source": id("items"), "key": "a", "field": "count"}),
            ),
            (
                "name",
                json!({"op": "field", "source": id("items"), "key": "a", "field": "name"}),
            ),
            (
                "other",
                json!({"op": "field", "source": id("items"), "key": "b", "field": "count"}),
            ),
        ];
        for (name, read) in &bindings {
            command(&mut session, json!({"op": "effect", "source": id(name)})).unwrap();
            command(&mut session, json!({"op": "begin", "source": id(name)})).unwrap();
            command(&mut session, read.clone()).unwrap();
            finish(&mut session, name, Value::Null).unwrap();
        }
        let drain = |session: &mut Session| {
            let mut ran = Vec::new();
            while let Some(source) = session.graph.next() {
                let (_, read) = bindings
                    .iter()
                    .find(|(name, _)| *name == source.id)
                    .unwrap();
                session.graph.begin(&source).unwrap();
                let value = command(session, read.clone());
                // Removing a row makes the row read fail; field reads yield
                // null. Both must wake so the template can remove stale text.
                let _ = finish(session, &source.id, value.unwrap_or(Value::Null));
                ran.push(source.id);
            }
            ran
        };
        command(
            &mut session,
            json!({"op": "patch", "source": id("items"), "key": "a", "fields": {"count": 13}}),
        )
        .unwrap();
        assert_eq!(drain(&mut session), ["count", "row"]);
        command(
            &mut session,
            json!({"op": "move", "source": id("items"), "key": "a"}),
        )
        .unwrap();
        assert_eq!(drain(&mut session), ["order"]);
        command(
            &mut session,
            json!({"op": "patch", "source": id("items"), "key": "b", "fields": {"count": 19}}),
        )
        .unwrap();
        assert_eq!(drain(&mut session), ["other"]);
        command(
            &mut session,
            json!({"op": "remove", "source": id("items"), "key": "a"}),
        )
        .unwrap();
        assert_eq!(drain(&mut session), ["count", "name", "order", "row"]);
        command(
            &mut session,
            json!({"op": "insert", "source": id("items"), "row": {"id": "a", "count": 23}}),
        )
        .unwrap();
        assert_eq!(drain(&mut session), ["count", "name", "order", "row"]);
        command(
            &mut session,
            json!({"op": "patch", "source": id("items"), "key": "a", "fields": {"count": 23}}),
        )
        .unwrap();
        assert!(drain(&mut session).is_empty());
    }

    #[test]
    fn binary_batches_keep_successes_and_report_duplicate_keys_without_partial_edits() {
        let mut session = Session::default();
        let commands = json!([
            {"op": "collection", "source": id("items")},
            {"op": "insert", "source": id("items"), "row": {"id": "b", "amount": 7}},
            {"op": "insert", "source": id("items"), "row": {"id": "a", "amount": 13}, "before": "b"},
            {"op": "insert", "source": id("items"), "row": {"id": "a", "amount": 99}},
            {"op": "patch", "source": id("items"), "key": "b", "fields": {"amount": 17}},
            {"op": "keys", "source": id("items")},
            {"op": "row", "source": id("items"), "key": "a"},
            {"op": "row", "source": id("items"), "key": "b"}
        ]);
        let bytes = super::super::wire::encode(&commands).unwrap();
        let result = super::super::wire::decode(&session.apply(&bytes)).unwrap();
        assert_eq!(result[3]["ok"], false);
        let reason = result[3]["reason"].as_str().unwrap();
        assert!(
            reason.contains("hud")
                && reason.contains("items")
                && reason.contains("duplicate key 'a'")
        );
        assert_eq!(result[5]["value"], json!(["a", "b"]));
        assert_eq!(result[6]["value"]["amount"].as_f64(), Some(13.0));
        assert_eq!(result[7]["value"]["amount"].as_f64(), Some(17.0));
    }

    #[test]
    fn a_refused_reentrant_write_cannot_commit_a_memo_or_escape_invalidation() {
        let mut session = Session::default();
        command(
            &mut session,
            json!({"op": "signal", "source": id("count"), "initial": 3}),
        )
        .unwrap();
        command(&mut session, json!({"op": "memo", "source": id("double")})).unwrap();
        assert_eq!(
            read(&mut session, "double"),
            json!({"evaluate": id("double")})
        );
        command(&mut session, json!({"op": "begin", "source": id("double")})).unwrap();
        assert_eq!(read(&mut session, "count"), json!({"value": 3}));
        finish(&mut session, "double", json!(6)).unwrap();

        command(
            &mut session,
            json!({"op": "set", "source": id("count"), "value": 5}),
        )
        .unwrap();
        command(&mut session, json!({"op": "begin", "source": id("double")})).unwrap();
        read(&mut session, "count");
        let refused = command(
            &mut session,
            json!({"op": "set", "source": id("count"), "value": 99}),
        );
        assert!(refused.unwrap_err().contains("reentrant write"));
        assert!(finish(&mut session, "double", json!(198)).is_err());
        assert_eq!(read(&mut session, "count"), json!({"value": 5}));
        assert_eq!(read(&mut session, "double"), json!({"value": 6}));

        command(
            &mut session,
            json!({"op": "set", "source": id("count"), "value": 7}),
        )
        .unwrap();
        assert_eq!(
            read(&mut session, "double"),
            json!({"evaluate": id("double")})
        );
        command(&mut session, json!({"op": "begin", "source": id("double")})).unwrap();
        read(&mut session, "count");
        finish(&mut session, "double", json!(14)).unwrap();
        assert_eq!(read(&mut session, "double"), json!({"value": 14}));
        assert_eq!(session.execute(Command::Next).unwrap(), Value::Null);
    }
}
