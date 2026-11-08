//! Document reads reference named stores and keyed rows, never copied viewmodels.

use super::{
    SourceId,
    graph::{Dependency, MountedNode},
    session::Session,
};
use crate::bind::{Env, Expr, InteractionState, Scope, get_path};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowRef {
    pub source: SourceId,
    pub key: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BindingId(u64);

/// Shared identities, not copies of the stores or collection rows.
#[derive(Clone, Debug)]
pub struct BindingContext {
    pub node: MountedNode,
    pub inputs: Arc<BTreeMap<String, SourceId>>,
    pub rows: Arc<BTreeMap<String, RowRef>>,
    /// Current-document indices resolved through this instance's lexical ancestry.
    pub selector_nodes: Arc<BTreeMap<usize, MountedNode>>,
}

impl BindingContext {
    pub fn write_model(
        &self,
        session: &mut Session,
        path: &str,
        value: Value,
    ) -> Result<(), String> {
        let (root, rest) = path.split_once('.').unwrap_or((path, ""));
        if let Some(row) = self.rows.get(root) {
            let (field, tail) = rest.split_once('.').unwrap_or((rest, ""));
            if field.is_empty() {
                return Err(format!("model '{path}' must name a row field"));
            }
            let record = session
                .sources
                .keyed(&row.source)?
                .row(&row.key)
                .ok_or_else(|| row.source.error(&format!("row '{}' was removed", row.key)))?;
            let mut next = record.get(field).cloned().unwrap_or(Value::Null);
            if !crate::bind::set_path(&mut next, tail, value) {
                return Err(format!("model cannot write path '{path}'"));
            }
            session.execute(super::session::Command::Patch {
                source: row.source.clone(),
                key: row.key.clone(),
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
    expression: Expr,
    value: Result<Value, String>,
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
        let id = BindingId(self.next_id);
        self.next_id += 1;
        let evaluation = SourceEnv {
            session,
            inputs: &context.inputs,
            rows: &context.rows,
            selector_nodes: &context.selector_nodes,
            host,
            node: &context.node,
            interaction: &interaction,
        }
        .evaluate(&expression);
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
                host,
                node: &context.node,
                interaction: &interaction,
            }
            .evaluate(&binding.expression);
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
    pub host: &'a dyn Env,
    pub node: &'a MountedNode,
    pub interaction: &'a dyn Fn(&MountedNode, InteractionState) -> bool,
}

impl SourceEnv<'_> {
    pub fn evaluate(&self, expression: &Expr) -> Evaluation {
        let tracked = Tracked {
            source: self,
            reads: RefCell::new(BTreeSet::new()),
            errors: RefCell::new(Vec::new()),
        };
        let value = expression.eval(&Scope {
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
        if let Some(row) = self.source.rows.get(root) {
            let field = rest.split('.').next().unwrap_or_default();
            self.reads.borrow_mut().insert(if field.is_empty() {
                Dependency::Row(row.source.clone(), row.key.clone())
            } else {
                Dependency::Field(row.source.clone(), row.key.clone(), field.into())
            });
            let record = self
                .source
                .session
                .sources
                .keyed(&row.source)?
                .row(&row.key)
                .ok_or_else(|| row.source.error(&format!("row '{}' was removed", row.key)))?;
            if rest.is_empty() {
                return Ok(Value::Object(record.clone()));
            }
            let (field, tail) = rest.split_once('.').unwrap_or((rest, ""));
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
    fn texture(&self, handle: f64) -> Option<og_render::TextureHandle> {
        self.source.host.texture(handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactive::session::Command;
    use serde_json::json;

    #[test]
    fn models_write_writable_inputs_and_keyed_rows_but_not_readable_or_derived_inputs() {
        let mut session = Session::default();
        let id = |name| SourceId::new("settings", name);
        for (name, readable) in [("editable", false), ("external", true)] {
            session
                .execute(Command::Signal {
                    source: id(name),
                    initial: json!({"level": 3}),
                    readable,
                })
                .unwrap();
        }
        session
            .execute(Command::Memo {
                source: id("derived"),
                inputs: Some(Vec::new()),
                initial: Some(json!(5)),
                inactive: true,
            })
            .unwrap();
        session
            .execute(Command::Collection {
                source: id("items"),
                key: "id".into(),
                items: [json!({"id":"a", "count":7}), json!({"id":"b", "count":11})]
                    .into_iter()
                    .map(|v| v.as_object().unwrap().clone())
                    .collect(),
            })
            .unwrap();
        let context = BindingContext {
            node: MountedNode {
                mount: id("screen"),
                key: 1,
            },
            selector_nodes: Arc::default(),
            inputs: Arc::new(
                ["editable", "external", "derived"]
                    .into_iter()
                    .map(|name| (name.into(), id(name)))
                    .collect(),
            ),
            rows: Arc::new(BTreeMap::from([(
                "item".into(),
                RowRef {
                    source: id("items"),
                    key: "a".into(),
                },
            )])),
        };
        context
            .write_model(&mut session, "$editable.level", json!(9))
            .unwrap();
        assert_eq!(
            session.read_value(&id("editable")).unwrap(),
            &json!({"level":9})
        );
        assert!(
            context
                .write_model(&mut session, "$external.level", json!(9))
                .is_err()
        );
        assert!(
            context
                .write_model(&mut session, "$derived", json!(9))
                .is_err()
        );
        assert_eq!(
            session.read_value(&id("external")).unwrap(),
            &json!({"level":3})
        );
        // Producer writes remain legal even though documents cannot write them.
        session
            .execute(Command::Set {
                source: id("external"),
                value: json!({"level":4}),
                force: false,
            })
            .unwrap();
        assert_eq!(
            session.read_value(&id("external")).unwrap(),
            &json!({"level":4})
        );
        session
            .execute(Command::Move {
                source: id("items"),
                key: "a".into(),
                before: None,
            })
            .unwrap();
        context
            .write_model(&mut session, "item.count", json!(17))
            .unwrap();
        let items = session.sources.keyed(&id("items")).unwrap();
        assert_eq!(items.keys(), &["b", "a"]);
        assert_eq!(items.row("a").unwrap()["count"], json!(17));
        assert_eq!(items.row("b").unwrap()["count"], json!(11));
        assert!(
            context
                .write_model(&mut session, "item.id", json!("c"))
                .is_err()
        );
        assert!(
            session
                .sources
                .keyed(&id("items"))
                .unwrap()
                .row("c")
                .is_none()
        );
    }

    #[test]
    fn queued_bindings_do_no_idle_work_and_update_only_the_changed_row_field() {
        let mut session = Session::default();
        let source = SourceId::new("inventory", "items");
        session
            .execute(Command::Collection {
                source: source.clone(),
                key: "id".into(),
                items: (0..1000)
                    .map(|i| {
                        json!({"id": i.to_string(), "label": format!("Item {i}"), "price": i + 7})
                            .as_object()
                            .unwrap()
                            .clone()
                    })
                    .collect(),
            })
            .unwrap();
        let mut bindings = Bindings::default();
        let mut ids = Vec::new();
        for key in 0..1000 {
            ids.push(bindings.insert(
                &mut session,
                BindingContext {
                    node: MountedNode {
                        mount: SourceId::new("inventory", "screen"),
                        key,
                    },
                    selector_nodes: Arc::default(),
                    inputs: Arc::default(),
                    rows: Arc::new(BTreeMap::from([(
                        "item".into(),
                        RowRef {
                            source: source.clone(),
                            key: key.to_string(),
                        },
                    )])),
                },
                Expr::parse("item.label").unwrap(),
                &crate::bind::NoEnv,
                |_, _| false,
            ));
        }
        let idle = bindings.refresh(&mut session, &crate::bind::NoEnv, |_, _| {
            panic!("idle visited a binding")
        });
        assert_eq!(idle.evaluated, 0);
        assert!(idle.changed.is_empty());
        session
            .execute(Command::Patch {
                source: source.clone(),
                key: "999".into(),
                fields: json!({"price": 2}).as_object().unwrap().clone(),
            })
            .unwrap();
        assert_eq!(
            bindings
                .refresh(&mut session, &crate::bind::NoEnv, |_, _| panic!(
                    "unread field"
                ))
                .evaluated,
            0
        );
        session
            .execute(Command::Patch {
                source: source.clone(),
                key: "999".into(),
                fields: json!({"label": "Last row"}).as_object().unwrap().clone(),
            })
            .unwrap();
        let changed = bindings.refresh(&mut session, &crate::bind::NoEnv, |_, _| {
            panic!("label binding does not read interaction")
        });
        assert_eq!(changed.evaluated, 1);
        assert_eq!(changed.changed, vec![ids[999]]);
        assert_eq!(bindings.value(ids[999]).unwrap(), &json!("Last row"));
        assert_eq!(bindings.value(ids[0]).unwrap(), &json!("Item 0"));
        session
            .execute(Command::Move {
                source: source.clone(),
                key: "999".into(),
                before: Some("0".into()),
            })
            .unwrap();
        assert_eq!(
            bindings
                .refresh(&mut session, &crate::bind::NoEnv, |_, _| panic!(
                    "key moved, value did not"
                ))
                .evaluated,
            0
        );
        session
            .execute(Command::Remove {
                source: source.clone(),
                key: "999".into(),
            })
            .unwrap();
        let removed = bindings.refresh(&mut session, &crate::bind::NoEnv, |_, _| false);
        assert_eq!(removed.evaluated, 1);
        assert_eq!(removed.changed, vec![ids[999]]);
        assert!(
            bindings
                .value(ids[999])
                .unwrap_err()
                .contains("row '999' was removed")
        );
        session
            .execute(Command::Insert {
                source,
                before: None,
                row: json!({"id": "999", "label": "Restored", "price": 17})
                    .as_object()
                    .unwrap()
                    .clone(),
            })
            .unwrap();
        assert_eq!(
            bindings
                .refresh(&mut session, &crate::bind::NoEnv, |_, _| false)
                .changed,
            vec![ids[999]]
        );
        assert_eq!(bindings.value(ids[999]).unwrap(), &json!("Restored"));
    }

    #[test]
    fn selectors_are_scoped_to_mount_and_key_and_unmount_removes_subscriptions() {
        let mut session = Session::default();
        let mut bindings = Bindings::default();
        let nodes = [
            MountedNode {
                mount: SourceId::new("hud", "first"),
                key: 17,
            },
            MountedNode {
                mount: SourceId::new("hud", "second"),
                key: 17,
            },
            MountedNode {
                mount: SourceId::new("hud", "first"),
                key: 32,
            },
        ];
        let ids: Vec<_> = nodes
            .iter()
            .map(|node| {
                bindings.insert(
                    &mut session,
                    BindingContext {
                        node: node.clone(),
                        inputs: Arc::default(),
                        rows: Arc::default(),
                        selector_nodes: Arc::default(),
                    },
                    Expr::Interaction(InteractionState::Hover),
                    &crate::bind::NoEnv,
                    |_, _| false,
                )
            })
            .collect();
        session
            .graph
            .interaction_changed(&nodes[0], InteractionState::FocusVisible);
        assert_eq!(
            bindings
                .refresh(&mut session, &crate::bind::NoEnv, |_, _| panic!(
                    "unread selector"
                ))
                .evaluated,
            0
        );
        session
            .graph
            .interaction_changed(&nodes[0], InteractionState::Hover);
        let changed = bindings.refresh(&mut session, &crate::bind::NoEnv, |node, state| {
            assert_eq!(node, &nodes[0]);
            state == InteractionState::Hover
        });
        assert_eq!(changed.evaluated, 1);
        assert_eq!(changed.changed, vec![ids[0]]);
        assert_eq!(bindings.value(ids[0]).unwrap(), &json!(true));
        assert_eq!(bindings.value(ids[1]).unwrap(), &json!(false));
        assert_eq!(bindings.value(ids[2]).unwrap(), &json!(false));
        bindings.remove(&mut session, ids[0]);
        session
            .graph
            .interaction_changed(&nodes[0], InteractionState::Hover);
        assert_eq!(
            bindings
                .refresh(&mut session, &crate::bind::NoEnv, |_, _| panic!(
                    "removed node"
                ))
                .evaluated,
            0
        );
    }

    #[test]
    fn ancestor_selectors_track_mapped_instances_and_refuse_missing_identities() {
        let mut session = Session::default();
        let mut bindings = Bindings::default();
        let ancestors = [
            MountedNode {
                mount: SourceId::new("hud", "first"),
                key: 17,
            },
            MountedNode {
                mount: SourceId::new("hud", "second"),
                key: 17,
            },
            MountedNode {
                mount: SourceId::new("hud", "first"),
                key: 32,
            },
        ];
        let mut ids = Vec::new();
        for ancestor in &ancestors {
            let target = MountedNode {
                mount: ancestor.mount.clone(),
                key: ancestor.key + 100,
            };
            let binding = bindings.insert(
                &mut session,
                BindingContext {
                    node: target,
                    inputs: Arc::default(),
                    rows: Arc::default(),
                    selector_nodes: Arc::new(BTreeMap::from([(7, ancestor.clone())])),
                },
                Expr::InteractionAt {
                    node: 7,
                    state: InteractionState::PlaceholderShown,
                },
                &crate::bind::NoEnv,
                |node, _| node.key >= 100,
            );
            assert_eq!(bindings.value(binding).unwrap(), &json!(false));
            ids.push(binding);
        }
        session
            .graph
            .interaction_changed(&ancestors[0], InteractionState::PlaceholderShown);
        let changed = bindings.refresh(&mut session, &crate::bind::NoEnv, |node, state| {
            assert_eq!(node, &ancestors[0]);
            assert_eq!(state, InteractionState::PlaceholderShown);
            true
        });
        assert_eq!(changed.evaluated, 1);
        assert_eq!(changed.changed, vec![ids[0]]);
        assert_eq!(bindings.value(ids[0]).unwrap(), &json!(true));
        assert_eq!(bindings.value(ids[1]).unwrap(), &json!(false));
        assert_eq!(bindings.value(ids[2]).unwrap(), &json!(false));
        let missing = bindings.insert(
            &mut session,
            BindingContext {
                node: ancestors[0].clone(),
                inputs: Arc::default(),
                rows: Arc::default(),
                selector_nodes: Arc::default(),
            },
            Expr::InteractionAt {
                node: 7,
                state: InteractionState::Hover,
            },
            &crate::bind::NoEnv,
            |_, _| panic!("missing ancestor must not query target state"),
        );
        let error = bindings.value(missing).unwrap_err();
        assert!(error.contains("hud") && error.contains("first") && error.contains("node 7"));
    }

    #[test]
    fn locale_changes_refresh_translations_but_not_literal_bindings() {
        struct Messages(&'static str);
        impl Env for Messages {
            fn language(&self) -> &str {
                self.0
            }
            fn translate(&self, key: &str) -> Option<String> {
                assert_eq!(key, "greeting");
                Some(if self.0 == "fr" { "Bonjour" } else { "Hello" }.into())
            }
        }
        let mut ui = crate::Ui::new(Default::default());
        let mut bindings = Bindings::default();
        let context = BindingContext {
            node: MountedNode {
                mount: SourceId::new("hud", "main"),
                key: 1,
            },
            selector_nodes: Arc::default(),
            inputs: Arc::default(),
            rows: Arc::default(),
        };
        let translated = bindings.insert(
            &mut ui.reactive,
            context.clone(),
            Expr::Translate(Box::new(Expr::Lit(json!("greeting"))), Vec::new()),
            &Messages("en"),
            |_, _| false,
        );
        let literal = bindings.insert(
            &mut ui.reactive,
            context,
            Expr::Lit(json!("unchanged")),
            &Messages("en"),
            |_, _| false,
        );
        assert_eq!(bindings.value(translated).unwrap(), &json!("Hello"));
        ui.set_locale("fr");
        let changed = bindings.refresh(&mut ui.reactive, &Messages("fr"), |_, _| false);
        assert_eq!(changed.evaluated, 1);
        assert_eq!(changed.changed, vec![translated]);
        assert_eq!(bindings.value(translated).unwrap(), &json!("Bonjour"));
        assert_eq!(bindings.value(literal).unwrap(), &json!("unchanged"));
        ui.set_locale("fr");
        assert_eq!(
            bindings
                .refresh(&mut ui.reactive, &Messages("fr"), |_, _| panic!(
                    "locale unchanged"
                ))
                .evaluated,
            0
        );
    }

    #[test]
    fn expressions_track_only_the_selected_branch_and_row_field() {
        let mut session = Session::default();
        let selected = SourceId::new("inventory", "selected");
        let items = SourceId::new("inventory", "items");
        session
            .execute(Command::Signal {
                source: selected.clone(),
                initial: json!(true),
                readable: false,
            })
            .unwrap();
        session
            .execute(Command::Collection {
                source: items.clone(),
                key: "id".into(),
                items: vec![
                    json!({"id":"shield", "label":"Oak", "price":17})
                        .as_object()
                        .unwrap()
                        .clone(),
                ],
            })
            .unwrap();
        let inputs = BTreeMap::from([("selected".into(), selected.clone())]);
        let rows = BTreeMap::from([(
            "item".into(),
            RowRef {
                source: items.clone(),
                key: "shield".into(),
            },
        )]);
        let expression = Expr::parse("$selected ? item.label : item.price").unwrap();
        let node = MountedNode {
            mount: SourceId::new("inventory", "screen"),
            key: 41,
        };
        let run = |session: &Session| {
            SourceEnv {
                session,
                inputs: &inputs,
                rows: &rows,
                selector_nodes: &BTreeMap::new(),
                host: &crate::bind::NoEnv,
                node: &node,
                interaction: &|_, state| state == InteractionState::Hover,
            }
            .evaluate(&expression)
        };
        let first = run(&session);
        assert_eq!(first.value.unwrap(), json!("Oak"));
        assert_eq!(
            first.reads,
            BTreeSet::from([
                Dependency::Value(selected.clone()),
                Dependency::Field(items.clone(), "shield".into(), "label".into()),
            ])
        );
        session
            .execute(Command::Set {
                source: selected.clone(),
                value: json!(false),
                force: false,
            })
            .unwrap();
        let second = run(&session);
        assert_eq!(second.value.unwrap(), json!(17));
        assert_eq!(
            second.reads,
            BTreeSet::from([
                Dependency::Value(selected),
                Dependency::Field(items, "shield".into(), "price".into()),
            ])
        );
    }
}
