//! A mount fed by values instead of its user's stores: a screen pushed
//! with a viewmodel, a document shown with one of its fixtures. The mount
//! holds a store for each field, in a scope of its own that goes with it:
//! a list of rows, each an object with its own `id`, is a collection keyed
//! by it, anything else a writable store, which the document's two-way
//! bindings write and [`Ui::mount_values`] reads back.

use super::*;
use crate::reactive::session::Command;

/// The scope of the stores `mount` holds.
fn held_scope(mount: &SourceId) -> String {
    format!("og.values/{}:{}", mount.scope, mount.id)
}

/// A list a collection can hold: rows, each an object with an `id` (a
/// nonempty string) no other has.
fn rows(items: &[Value]) -> Option<Vec<serde_json::Map<String, Value>>> {
    let mut ids = BTreeSet::new();
    items
        .iter()
        .map(|item| {
            let row = item.as_object()?;
            let id = row.get("id")?.as_str().filter(|id| !id.is_empty())?;
            ids.insert(id).then(|| row.clone())
        })
        .collect()
}

impl Ui {
    /// Mounts `document` as `mount`, reading `values`' fields as its
    /// stores (an object; null is none). It compiles at the next build.
    pub fn mount_with_values(
        &mut self,
        mount: SourceId,
        document: &str,
        values: &Value,
    ) -> Result<(), String> {
        let inputs = self.hold_values(&mount, document, values)?;
        let holds = !inputs.is_empty();
        let mounted =
            self.register_document_mount(mount.clone(), document.into(), inputs, BTreeMap::new());
        if mounted.is_err() && holds {
            self.drop_values(&mount);
        }
        mounted
    }

    /// Declares the stores `mount` holds for `values`; answers them by
    /// field, as the mount's inputs.
    pub(crate) fn hold_values(
        &mut self,
        mount: &SourceId,
        document: &str,
        values: &Value,
    ) -> Result<BTreeMap<String, SourceId>, String> {
        mount.validate()?;
        let fields = match values {
            Value::Object(fields) => fields.clone(),
            Value::Null => Default::default(),
            _ => {
                return Err(format!(
                    "the values of '{document}' are an object of named fields"
                ));
            }
        };
        if self.mounted(mount) {
            return Err(format!(
                "mount '{}:{}' already exists",
                mount.scope, mount.id
            ));
        }
        let scope = held_scope(mount);
        let mut inputs = BTreeMap::new();
        for (name, value) in fields {
            let source = SourceId::new(&scope, &name);
            let command = match value.as_array().map(|items| rows(items)) {
                Some(Some(items)) => Command::Collection {
                    source: source.clone(),
                    key: String::new(),
                    items,
                },
                _ => Command::Signal {
                    source: source.clone(),
                    initial: value,
                    readable: false,
                },
            };
            if let Err(error) = self.docs.execute(command) {
                if !inputs.is_empty() {
                    self.drop_values(mount);
                }
                return Err(format!("the value '{name}' of '{document}': {error}"));
            }
            inputs.insert(name, source);
        }
        Ok(inputs)
    }

    /// Removes the stores `mount` holds.
    fn drop_values(&mut self, mount: &SourceId) {
        let scope = held_scope(mount);
        if let Err(error) = self.docs.execute(Command::Dispose { scope }) {
            self.out.warnings_mut().push(error);
        }
    }

    /// The values `mount` holds now, by field: what it was mounted with
    /// and what its document's two-way bindings wrote since. None when it
    /// is not mounted.
    pub fn mount_values(&self, mount: &SourceId) -> Option<Value> {
        let memory = &self.docs.state().memory;
        let registration = memory.retained.registrations.get(mount)?;
        let scope = held_scope(mount);
        let sources = &memory.reactive.sources;
        let fields = registration
            .inputs
            .iter()
            .filter(|(_, source)| source.scope == scope)
            .map(|(name, source)| {
                let value = match sources.keyed(source) {
                    Ok(collection) => collection
                        .keys()
                        .iter()
                        .map(|key| json!(collection.row(key)))
                        .collect(),
                    Err(_) => sources.read(source).cloned().unwrap_or(Value::Null),
                };
                (name.clone(), value)
            })
            .collect();
        Some(Value::Object(fields))
    }

    fn mounted(&self, mount: &SourceId) -> bool {
        self.docs
            .state()
            .memory
            .retained
            .registrations
            .contains_key(mount)
    }

    /// Removes `mount`, and the stores it holds with it.
    pub(crate) fn finish_unmount(&mut self, mount: &SourceId) {
        let mut host = std::mem::take(&mut self.docs.state_mut().memory.retained);
        let scope = held_scope(mount);
        let holds = host
            .registrations
            .get(mount)
            .is_some_and(|r| r.inputs.values().any(|source| source.scope == scope));
        host.remove_mount(self, mount);
        self.docs.state_mut().memory.retained = host;
        if holds {
            self.drop_values(mount);
        }
    }
}
