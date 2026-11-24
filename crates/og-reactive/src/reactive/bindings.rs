//! Document reads reference named stores and keyed rows, never copied viewmodels.

use super::{
    SourceId,
    graph::{Dependency, MountedNode},
    session::Session,
};
use crate::bind::{Env, Expr, InteractionState, Scope, Template, get_path};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// The keyed list an `{#each}` walks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ListRef {
    /// A keyed collection store.
    Collection(SourceId),
    /// An array held by a value store; an empty key names the string item itself.
    Value { source: SourceId, key: String },
    /// Immutable string-list component defaults.
    Strings(Arc<Vec<String>>),
    /// A list held in a row (`{#each r.cells as c (c.id)}`): the field
    /// path of `row` holding it, and the path in each item of its key.
    Nested {
        row: Arc<RowRef>,
        path: String,
        key: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowRef {
    pub list: ListRef,
    pub key: String,
    /// The name reads the row's position in its list (an `{#each}` index
    /// alias), not the row.
    pub position: bool,
}

/// A nested list's item keys, in order: each a nonempty string, none twice.
pub fn nested_keys(list: &Value, key: &str) -> Result<Vec<String>, String> {
    let items = match list {
        Value::Null => return Ok(Vec::new()),
        Value::Array(items) => items,
        _ => return Err("each reads a list".into()),
    };
    let mut keys = Vec::with_capacity(items.len());
    for item in items {
        match get_path(item, key).as_deref().and_then(Value::as_str) {
            Some(k) if !k.is_empty() && !keys.iter().any(|old| old == k) => keys.push(k.into()),
            Some(k) if !k.is_empty() => return Err(format!("each key '{k}' is in its list twice")),
            _ => return Err(format!("each item's '{key}' must be a nonempty string")),
        }
    }
    Ok(keys)
}

impl RowRef {
    /// The collection row field a nested row's list is held in.
    fn held_in(&self) -> Option<Dependency> {
        if let ListRef::Value { source, .. } = &self.list {
            return Some(Dependency::Value(source.clone()));
        }
        let ListRef::Nested { row, path, .. } = &self.list else {
            return None;
        };
        row.held_in().or_else(|| {
            let ListRef::Collection(source) = &row.list else {
                unreachable!("a nested row held in no collection")
            };
            let field = path.split('.').next().unwrap_or_default();
            Some(Dependency::Field(
                source.clone(),
                row.key.clone(),
                field.into(),
            ))
        })
    }

    /// The collection row this row is, or is held in, and the field path
    /// from that row to `rest` of this one.
    fn locate(
        &self,
        session: &Session,
        rest: &str,
    ) -> Result<(&SourceId, Option<&str>, String), String> {
        match &self.list {
            ListRef::Collection(source) => Ok((source, Some(&self.key), rest.to_owned())),
            ListRef::Value { source, key } => {
                let at = nested_keys(session.read_value(source)?, key)?
                    .iter()
                    .position(|key| *key == self.key)
                    .ok_or_else(|| source.error(&format!("item '{}' was removed", self.key)))?;
                Ok((
                    source,
                    None,
                    if rest.is_empty() {
                        at.to_string()
                    } else {
                        format!("{at}.{rest}")
                    },
                ))
            }
            ListRef::Strings(_) => Err("literal string-list items are read only".into()),
            ListRef::Nested { row, path, key } => {
                let (source, root, path) = row.locate(session, path)?;
                let (field, tail) = path.split_once('.').unwrap_or((&path, ""));
                let list = if let Some(root) = root {
                    session
                        .sources
                        .keyed(source)?
                        .row(root)
                        .ok_or_else(|| source.error(&format!("row '{root}' was removed")))?
                        .get(field)
                        .and_then(|value| get_path(value, tail))
                } else {
                    get_path(session.read_value(source)?, &path)
                };
                let at = nested_keys(list.as_deref().unwrap_or(&Value::Null), key)?
                    .iter()
                    .position(|key| *key == self.key)
                    .ok_or_else(|| {
                        source.error(&format!("item '{}' of '{path}' was removed", self.key))
                    })?;
                Ok((
                    source,
                    root,
                    if rest.is_empty() {
                        format!("{path}.{at}")
                    } else {
                        format!("{path}.{at}.{rest}")
                    },
                ))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BindingId(u64);

/// Shared identities, not copies of the stores or collection rows.
#[derive(Clone, Debug)]
pub struct BindingContext {
    pub node: MountedNode,
    pub inputs: Arc<BTreeMap<String, SourceId>>,
    pub rows: Arc<BTreeMap<String, RowRef>>,
    pub row_key: String,
    /// Current-document indices resolved through this instance's lexical ancestry.
    pub selector_nodes: Arc<BTreeMap<usize, MountedNode>>,
    /// A component instance's props and the content passed to it.
    pub lexical: Arc<Lexical>,
}

/// What a component instance, or content passed to one, reads beyond its
/// mount's stores (Svelte's props and snippets).
#[derive(Clone, Debug, Default)]
pub struct Lexical {
    /// Its props by name: each an expression of the instance's user, read
    /// in the user's context, so a prop's reads are the user's reads.
    pub props: BTreeMap<String, PropRef>,
    /// The snippets passed to it (and to a mount), by name: what
    /// `{@render name()}` renders.
    pub content: BTreeMap<String, ContentRef>,
}

/// A component prop: its user's expression and context; `model`, a
/// `bind:` prop's writable path in that context.
#[derive(Clone, Debug)]
pub struct PropRef {
    pub expression: BindingExpression,
    pub model: Option<String>,
    pub context: Arc<BindingContext>,
}

/// A snippet's document, opaque to the bindings: the documents that render
/// it know its type.
pub trait SnippetDocument: std::any::Any + std::fmt::Debug + Send + Sync {}

impl<T: std::any::Any + std::fmt::Debug + Send + Sync> SnippetDocument for T {}

/// Content passed by reference: a snippet (or a component's children)
/// rendered in the context it was written in, or a whole document.
#[derive(Clone, Debug)]
pub enum ContentRef {
    Snippet {
        document: Arc<dyn SnippetDocument>,
        index: usize,
        context: Arc<BindingContext>,
    },
    Document(String),
}

impl BindingContext {
    pub fn write_model(
        &self,
        session: &mut Session,
        path: &str,
        value: Value,
    ) -> Result<(), String> {
        let (root, rest) = path.split_once('.').unwrap_or((path, ""));
        if let Some(prop) = self.lexical.props.get(root) {
            let Some(model) = &prop.model else {
                return Err(format!(
                    "prop '{root}' is not bound two-way; its user writes bind:{root}={{…}}"
                ));
            };
            let path = if rest.is_empty() {
                model.clone()
            } else {
                format!("{model}.{rest}")
            };
            return prop.context.write_model(session, &path, value);
        }
        if let Some(row) = self.rows.get(root) {
            if row.position {
                return Err(format!(
                    "model '{path}' names a row's position, which is read only"
                ));
            }
            if rest.is_empty() {
                return Err(format!("model '{path}' must name a row field"));
            }
            let (source, key, field_path) = row.locate(session, rest)?;
            let Some(key) = key else {
                return session.write_model(source, &field_path, value);
            };
            let (source, key) = (source.clone(), key.to_owned());
            let (field, tail) = field_path.split_once('.').unwrap_or((&field_path, ""));
            let record = session
                .sources
                .keyed(&source)?
                .row(&key)
                .ok_or_else(|| source.error(&format!("row '{key}' was removed")))?;
            let mut next = record.get(field).cloned().unwrap_or(Value::Null);
            if !crate::bind::set_path(&mut next, tail, value) {
                return Err(format!("model cannot write path '{path}'"));
            }
            session.execute(super::session::Command::Patch {
                source,
                key,
                fields: [(field.to_owned(), next)].into_iter().collect(),
            })?;
            return Ok(());
        }
        let name = root
            .strip_prefix('$')
            .ok_or_else(|| format!("model '{path}' must name a store or keyed row field"))?;
        let source = self
            .inputs
            .get(name)
            .ok_or_else(|| format!("mount has no input store '{name}'"))?;
        session.write_model(source, rest, value)
    }
}

#[derive(Clone, Debug)]
struct Binding {
    context: BindingContext,
    expression: BindingExpression,
    value: Result<Value, String>,
}

#[derive(Clone, Debug)]
pub enum BindingExpression {
    Expression(Expr),
    Template(Template),
}

impl From<Expr> for BindingExpression {
    fn from(e: Expr) -> BindingExpression {
        BindingExpression::Expression(e)
    }
}

/// Compiled expressions and their last results. The source graph supplies the
/// work queue; refreshing an idle set never iterates its bindings or nodes.
#[derive(Clone, Debug, Default)]
pub struct Bindings {
    next_id: u64,
    entries: BTreeMap<BindingId, Binding>,
}

#[derive(Debug, Default)]
pub struct BindingChanges {
    pub evaluated: usize,
    pub changed: Vec<BindingId>,
}

impl Bindings {
    pub fn insert(
        &mut self,
        session: &mut Session,
        context: BindingContext,
        expression: Expr,
        host: &dyn Env,
        interaction: impl Fn(&MountedNode, InteractionState) -> bool,
    ) -> BindingId {
        self.insert_expression(
            session,
            context,
            BindingExpression::Expression(expression),
            host,
            interaction,
        )
    }

    pub fn insert_expression(
        &mut self,
        session: &mut Session,
        context: BindingContext,
        expression: BindingExpression,
        host: &dyn Env,
        interaction: impl Fn(&MountedNode, InteractionState) -> bool,
    ) -> BindingId {
        let id = BindingId(self.next_id);
        self.next_id += 1;
        let evaluation = SourceEnv {
            session,
            inputs: &context.inputs,
            rows: &context.rows,
            selector_nodes: &context.selector_nodes,
            lexical: &context.lexical,
            host,
            node: &context.node,
            interaction: &interaction,
        }
        .evaluate_binding(&expression);
        session.graph.bind(id.0, evaluation.reads);
        self.entries.insert(
            id,
            Binding {
                context,
                expression,
                value: evaluation.value,
            },
        );
        id
    }

    pub fn value(&self, id: BindingId) -> Result<&Value, &String> {
        self.entries[&id].value.as_ref()
    }

    pub fn node(&self, id: BindingId) -> &MountedNode {
        &self.entries[&id].context.node
    }

    pub fn remove(&mut self, session: &mut Session, id: BindingId) {
        session.graph.unbind(id.0);
        self.entries.remove(&id);
    }

    pub fn refresh(
        &mut self,
        session: &mut Session,
        host: &dyn Env,
        interaction: impl Fn(&MountedNode, InteractionState) -> bool,
    ) -> BindingChanges {
        let pending = session.graph.take_bindings();
        let mut changes = BindingChanges {
            evaluated: pending.len(),
            ..Default::default()
        };
        for raw in pending {
            let id = BindingId(raw);
            let binding = self.entries.get_mut(&id).expect("subscribed binding");
            let context = &binding.context;
            let evaluation = SourceEnv {
                session,
                inputs: &context.inputs,
                rows: &context.rows,
                selector_nodes: &context.selector_nodes,
                lexical: &context.lexical,
                host,
                node: &context.node,
                interaction: &interaction,
            }
            .evaluate_binding(&binding.expression);
            session.graph.bind(raw, evaluation.reads);
            if binding.value != evaluation.value {
                binding.value = evaluation.value;
                changes.changed.push(id);
            }
        }
        changes
    }
}

pub struct Evaluation {
    pub value: Result<Value, String>,
    pub reads: BTreeSet<Dependency>,
}

pub struct SourceEnv<'a> {
    pub session: &'a Session,
    pub inputs: &'a BTreeMap<String, SourceId>,
    pub rows: &'a BTreeMap<String, RowRef>,
    pub selector_nodes: &'a BTreeMap<usize, MountedNode>,
    pub lexical: &'a Lexical,
    pub host: &'a dyn Env,
    pub node: &'a MountedNode,
    pub interaction: &'a dyn Fn(&MountedNode, InteractionState) -> bool,
}

impl SourceEnv<'_> {
    pub fn evaluate(&self, expression: &Expr) -> Evaluation {
        self.evaluate_with(|scope| expression.eval(scope))
    }

    fn evaluate_binding(&self, expression: &BindingExpression) -> Evaluation {
        self.evaluate_with(|scope| match expression {
            BindingExpression::Expression(expression) => expression.eval(scope),
            BindingExpression::Template(template) => Value::String(template.format(scope)),
        })
    }

    fn evaluate_with(&self, evaluate: impl FnOnce(&Scope<'_>) -> Value) -> Evaluation {
        let tracked = Tracked {
            source: self,
            reads: RefCell::new(BTreeSet::new()),
            errors: RefCell::new(Vec::new()),
        };
        let value = evaluate(&Scope {
            vm: &Value::Null,
            locals: Vec::new(),
            env: &tracked,
        });
        let errors = tracked.errors.into_inner();
        Evaluation {
            value: if errors.is_empty() {
                Ok(value)
            } else {
                Err(errors.join("; "))
            },
            reads: tracked.reads.into_inner(),
        }
    }
}

struct Tracked<'a, 'b> {
    source: &'a SourceEnv<'b>,
    reads: RefCell<BTreeSet<Dependency>>,
    errors: RefCell<Vec<String>>,
}

impl Tracked<'_, '_> {
    fn lookup(&self, path: &str) -> Result<Value, String> {
        let (root, rest) = path.split_once('.').unwrap_or((path, ""));
        if let Some(prop) = self.source.lexical.props.get(root) {
            let user = &prop.context;
            let evaluation = SourceEnv {
                session: self.source.session,
                inputs: &user.inputs,
                rows: &user.rows,
                selector_nodes: &user.selector_nodes,
                lexical: &user.lexical,
                host: self.source.host,
                node: &user.node,
                interaction: self.source.interaction,
            }
            .evaluate_binding(&prop.expression);
            self.reads.borrow_mut().extend(evaluation.reads);
            let value = evaluation.value?;
            return Ok(get_path(&value, rest).map_or(Value::Null, |value| value.into_owned()));
        }
        if let Some(row) = self.source.rows.get(root) {
            let session = self.source.session;
            if let ListRef::Strings(items) = &row.list {
                let at = items
                    .iter()
                    .position(|key| *key == row.key)
                    .ok_or_else(|| format!("item '{}' was removed", row.key))?;
                return Ok(if row.position {
                    Value::from(at)
                } else {
                    get_path(&Value::String(row.key.clone()), rest)
                        .map_or(Value::Null, |value| value.into_owned())
                });
            }
            // A nested row's reads are those of the collection row holding
            // it: its list changing is what moves, changes or retires it.
            if let Some(held) = row.held_in() {
                self.reads.borrow_mut().insert(held);
            }
            if row.position {
                return match &row.list {
                    // A position changes when the order does, not when a row does.
                    ListRef::Collection(source) => {
                        self.reads
                            .borrow_mut()
                            .insert(Dependency::Keys(source.clone()));
                        session
                            .sources
                            .keyed(source)?
                            .keys()
                            .iter()
                            .position(|key| *key == row.key)
                            .map(Value::from)
                            .ok_or_else(|| source.error(&format!("row '{}' was removed", row.key)))
                    }
                    ListRef::Value { .. } | ListRef::Nested { .. } => {
                        let (_, _, path) = row.locate(session, "")?;
                        let at = path
                            .rsplit('.')
                            .next()
                            .and_then(|at| at.parse::<usize>().ok());
                        Ok(Value::from(at.expect("an item's path ends in its index")))
                    }
                    ListRef::Strings(_) => unreachable!("literal strings handled above"),
                };
            }
            let (source, key, path) = row.locate(session, rest)?;
            let Some(key) = key else {
                self.reads
                    .borrow_mut()
                    .insert(Dependency::Value(source.clone()));
                return Ok(get_path(session.read_value(source)?, &path)
                    .map_or(Value::Null, |value| value.into_owned()));
            };
            let (field, tail) = path.split_once('.').unwrap_or((&path, ""));
            self.reads.borrow_mut().insert(if field.is_empty() {
                Dependency::Row(source.clone(), key.into())
            } else {
                Dependency::Field(source.clone(), key.into(), field.into())
            });
            let record = session
                .sources
                .keyed(source)?
                .row(key)
                .ok_or_else(|| source.error(&format!("row '{key}' was removed")))?;
            if field.is_empty() {
                return Ok(Value::Object(record.clone()));
            }
            return Ok(record
                .get(field)
                .and_then(|value| get_path(value, tail))
                .map_or(Value::Null, |value| value.into_owned()));
        }
        let name = root.strip_prefix('$').ok_or_else(|| {
            format!("'{root}' is not a row variable; read a named store as '${root}'")
        })?;
        let source = self
            .source
            .inputs
            .get(name)
            .ok_or_else(|| format!("mount has no input store '{name}'"))?;
        // A keyed collection read as a value (`$items.length`, or `$items`
        // handed to a component prop whose document reads `items.length`):
        // its rows in order, its length from its keys alone.
        if let Ok(collection) = self.source.session.sources.keyed(source) {
            let mut reads = self.reads.borrow_mut();
            reads.insert(Dependency::Keys(source.clone()));
            if rest == "length" {
                return Ok(Value::from(collection.keys().len()));
            }
            let rows: Vec<Value> = collection
                .keys()
                .iter()
                .map(|key| {
                    reads.insert(Dependency::Row(source.clone(), key.clone()));
                    collection
                        .row(key)
                        .map_or(Value::Null, |row| Value::Object(row.clone()))
                })
                .collect();
            let list = Value::Array(rows);
            return Ok(get_path(&list, rest).map_or(Value::Null, |value| value.into_owned()));
        }
        self.reads
            .borrow_mut()
            .insert(Dependency::Value(source.clone()));
        let value = self.source.session.read_value(source)?;
        Ok(get_path(value, rest).map_or(Value::Null, |value| value.into_owned()))
    }
}

impl Env for Tracked<'_, '_> {
    fn lookup_path(&self, path: &str) -> Option<Value> {
        match self.lookup(path) {
            Ok(value) => Some(value),
            Err(error) => {
                self.errors.borrow_mut().push(error);
                None
            }
        }
    }

    fn interaction(&self, state: InteractionState) -> bool {
        self.reads
            .borrow_mut()
            .insert(Dependency::Interaction(self.source.node.clone(), state));
        (self.source.interaction)(self.source.node, state)
    }

    fn interaction_at(&self, index: usize, state: InteractionState) -> bool {
        let Some(node) = self.source.selector_nodes.get(&index) else {
            self.errors
                .borrow_mut()
                .push(self.source.node.mount.error(&format!(
                    "selector document node {index} has no mounted identity"
                )));
            return false;
        };
        self.reads
            .borrow_mut()
            .insert(Dependency::Interaction(node.clone(), state));
        (self.source.interaction)(node, state)
    }

    fn language(&self) -> &str {
        self.reads.borrow_mut().insert(Dependency::Locale);
        self.source.host.language()
    }
    fn translate(&self, key: &str) -> Option<String> {
        self.reads.borrow_mut().insert(Dependency::Locale);
        self.source.host.translate(key)
    }
    fn texture(&self, handle: f64) -> Option<og_draw::TextureHandle> {
        self.source.host.texture(handle)
    }
}
