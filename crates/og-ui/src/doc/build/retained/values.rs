//! A mount fed by values instead of its user's stores: a screen pushed
//! with a viewmodel, a document shown with one of its fixtures. The mount
//! holds a store, in a scope of its own that goes with it, for each field
//! of the values and for each name its document reads that they leave
//! out: a prop's default, else nothing (null, or no rows where the
//! document iterates it). A list of rows, each an object with its own key
//! (the field the document's `{#each}` keys it by, else `id`), is a
//! collection keyed by it, anything else a writable store, which the
//! document's two-way bindings write and [`Ui::mount_values`] reads back.

use super::*;
use crate::reactive::session::Command;

/// The scope of the stores `mount` holds.
fn held_scope(mount: &SourceId) -> String {
    format!("og.values/{}:{}", mount.scope, mount.id)
}

/// A list a collection keyed by `key` can hold: rows, each an object
/// whose `key` is a nonempty string no other has.
fn rows(items: &[Value], key: &str) -> Option<Vec<serde_json::Map<String, Value>>> {
    let mut keys = BTreeSet::new();
    items
        .iter()
        .map(|item| {
            let row = item.as_object()?;
            let key = row.get(key)?.as_str().filter(|key| !key.is_empty())?;
            keys.insert(key).then(|| row.clone())
        })
        .collect()
}

/// What a mount of `document` reads by name: the names it reads bare (its
/// props), and the stores it and the components it uses read (`$name`).
/// Each has the field its rows
/// are keyed by where an `{#each}` iterates it.
fn names(
    documents: &BTreeMap<String, Arc<UiDocument>>,
    document: &str,
) -> BTreeMap<String, Option<String>> {
    let mut out: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut queue = vec![document.to_owned()];
    while let Some(name) = queue.pop() {
        let Some(doc) = documents.get(&name).filter(|_| seen.insert(name.clone())) else {
            continue;
        };
        let root = name == document;
        let props = doc.bare_names().into_iter().filter(|_| root).map(|prop| {
            let key = doc.each_key(&prop);
            (prop, key)
        });
        let stores = doc.stores().into_iter().map(|store| {
            let key = doc.each_key(&format!("${store}"));
            (store, key)
        });
        for (name, key) in props.chain(stores) {
            let held = out.entry(name).or_default();
            if held.is_none() {
                *held = key.map(str::to_owned);
            }
        }
        queue.extend(doc.nodes.iter().filter_map(|node| match &node.kind {
            NodeKind::Use(component) => Some(component.clone()),
            _ => None,
        }));
    }
    out
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
        let mut fields = match values {
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
        let documents = &self.docs.library.documents;
        let doc = documents
            .get(document)
            .ok_or_else(|| format!("missing document '{document}'"))?;
        let names = names(documents, document);
        for (name, key) in &names {
            if fields.contains_key(name) {
                continue;
            }
            let prop = doc.props.iter().find(|p| p.name == *name);
            let iterated = key.is_some() || prop.is_some_and(|p| p.ty == "list");
            let unset = if iterated { json!([]) } else { Value::Null };
            let value = prop.and_then(|p| p.default.clone()).unwrap_or(unset);
            fields.insert(name.clone(), value);
        }
        let scope = held_scope(mount);
        let mut inputs = BTreeMap::new();
        for (name, value) in fields {
            let source = SourceId::new(&scope, &name);
            let key = names.get(&name).cloned().flatten().unwrap_or("id".into());
            let command = match value.as_array().map(|items| rows(items, &key)) {
                Some(Some(items)) => Command::Collection {
                    source: source.clone(),
                    key,
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

    /// Makes `mount`, mounted with values, hold `values` (an object of some
    /// of its fields): a list of rows replaces its collection's rows by
    /// key, so a row that stays keeps its elements, and any other value is
    /// set where it changed. A field it does not hold is refused.
    pub fn set_mount_values(&mut self, mount: &SourceId, values: &Value) -> Result<(), String> {
        let Value::Object(fields) = values else {
            return Err("mount values are an object of named fields".into());
        };
        let scope = held_scope(mount);
        let registration = self
            .docs
            .state()
            .memory
            .retained
            .registrations
            .get(mount)
            .ok_or_else(|| format!("mount '{}:{}' is not mounted", mount.scope, mount.id))?;
        let mut held = Vec::new();
        for (name, value) in fields {
            let source = registration
                .inputs
                .get(name)
                .filter(|source| source.scope == scope)
                .ok_or_else(|| format!("mount '{}' holds no value '{name}'", mount.id))?;
            held.push((source.clone(), value));
        }
        for (source, value) in held {
            let sources = &self.docs.state().memory.reactive.sources;
            if let Ok(collection) = sources.keyed(&source) {
                let key = collection.key_field().to_owned();
                let rows = value
                    .as_array()
                    .and_then(|items| rows(items, &key))
                    .ok_or_else(|| {
                        format!(
                            "the value '{}' is a list of rows, each with its own string '{key}'",
                            source.id
                        )
                    })?;
                self.set_rows(&source, &rows);
            } else if sources.read(&source).ok() != Some(value) {
                self.docs.execute(Command::Set {
                    source,
                    value: value.clone(),
                    force: false,
                })?;
            }
        }
        Ok(())
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
