//! Dependency tracking is separate from executing game callbacks: hosts release
//! their UI lock before entering JS, then commit the tracked reads afterward.

use super::{CollectionEdit, SourceId};
use std::collections::{BTreeMap, BTreeSet};

/// Mounted identity includes the keyed row in the core node key. Selector
/// inputs cannot alias another mount or another row using the same template.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MountedNode {
    pub mount: SourceId,
    pub key: u64,
}

/// Collection order, rows and fields are separate subscriptions. A row patch
/// must not wake the list or a binding that reads another row.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dependency {
    Value(SourceId),
    Keys(SourceId),
    Row(SourceId, String),
    Field(SourceId, String, String),
    Interaction(MountedNode, crate::bind::InteractionState),
    CustomProperty(MountedNode, String),
    Locale,
}

impl Dependency {
    fn source(&self) -> Option<&SourceId> {
        match self {
            Self::Value(id) | Self::Keys(id) | Self::Row(id, _) | Self::Field(id, _, _) => Some(id),
            Self::Interaction(node, _) | Self::CustomProperty(node, _) => Some(&node.mount),
            Self::Locale => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Computation {
    Input,
    Memo,
    Effect,
}

#[derive(Clone, Debug)]
struct Node {
    kind: Computation,
    dirty: bool,
    enabled: bool,
    reads: BTreeSet<Dependency>,
    declared: Option<BTreeSet<SourceId>>,
}

#[derive(Clone, Debug)]
struct Evaluation {
    id: SourceId,
    reads: BTreeSet<Dependency>,
    tracking: bool,
    error: Option<String>,
}

/// A synchronous dependency graph. Values and callback registration IDs are
/// owned by the host's sources; no function or mutable JS closure is stored here.
#[derive(Clone, Debug, Default)]
pub struct Graph {
    nodes: BTreeMap<SourceId, Node>,
    observers: BTreeMap<Dependency, BTreeSet<SourceId>>,
    binding_reads: BTreeMap<u64, BTreeSet<Dependency>>,
    binding_observers: BTreeMap<Dependency, BTreeSet<u64>>,
    pending_bindings: BTreeSet<u64>,
    active: Vec<Evaluation>,
    batch_depth: usize,
    pending_memos: BTreeSet<SourceId>,
    pending_effects: BTreeSet<SourceId>,
}

impl Graph {
    pub fn check_registration(&self, id: &SourceId, kind: Computation) -> Result<(), String> {
        id.validate()?;
        if !self.active.is_empty() {
            return Err(id.error("cannot register a source while evaluating a computation"));
        }
        if let Some(old) = self.nodes.get(id)
            && old.kind != kind
        {
            return Err(id.error("already registered with a different computation kind"));
        }
        Ok(())
    }

    /// Re-registering a computation keeps its identity and subscriptions but
    /// reevaluates the new function. Inputs keep their value without notifying.
    pub fn register(&mut self, id: SourceId, kind: Computation) -> Result<(), String> {
        self.check_registration(&id, kind)?;
        if !self.nodes.contains_key(&id) {
            self.nodes.insert(
                id.clone(),
                Node {
                    kind,
                    dirty: false,
                    enabled: true,
                    reads: BTreeSet::new(),
                    declared: None,
                },
            );
        }
        self.nodes
            .get_mut(&id)
            .expect("registered computation")
            .enabled = true;
        if kind != Computation::Input {
            self.dirty(&id);
        }
        Ok(())
    }

    fn dirty(&mut self, id: &SourceId) {
        if let Some(node) = self.nodes.get_mut(id)
            && node.enabled
        {
            node.dirty = true;
            match node.kind {
                Computation::Memo => {
                    self.pending_memos.insert(id.clone());
                }
                Computation::Effect => {
                    self.pending_effects.insert(id.clone());
                }
                Computation::Input => {}
            }
        }
    }

    /// A setter must call this before modifying a source, even inside untrack.
    pub fn can_write(&self, id: &SourceId) -> Result<(), String> {
        if let Some(owner) = self.active.last() {
            return Err(id.error(&format!(
                "reentrant write while evaluating '{}/{}'",
                owner.id.scope, owner.id.id
            )));
        }
        if !self.nodes.contains_key(id) {
            return Err(id.error("source is not registered"));
        }
        Ok(())
    }

    /// Notify only when the source's value changed. Memo equality is decided by
    /// its typed value store, before finish; unchanged memos stop propagation.
    pub fn changed(&mut self, id: &SourceId) -> Result<(), String> {
        self.can_write(id)?;
        self.notify(&Dependency::Value(id.clone()));
        Ok(())
    }

    pub fn collection_changed(
        &mut self,
        id: &SourceId,
        edit: &CollectionEdit,
    ) -> Result<(), String> {
        self.changed(id)?;
        match edit {
            CollectionEdit::Move { .. } => self.notify(&Dependency::Keys(id.clone())),
            CollectionEdit::Insert { key, .. } | CollectionEdit::Remove { key, .. } => {
                self.notify(&Dependency::Keys(id.clone()));
                self.notify(&Dependency::Row(id.clone(), key.clone()));
                let fields: Vec<_> = self
                    .observers
                    .range(Dependency::Field(id.clone(), key.clone(), String::new())..)
                    .take_while(|(dependency, _)| {
                        matches!(dependency,
                        Dependency::Field(source, row, _) if source == id && row == key)
                    })
                    .map(|(dependency, _)| dependency.clone())
                    .chain(
                        self.binding_observers
                            .range(Dependency::Field(id.clone(), key.clone(), String::new())..)
                            .take_while(|(dependency, _)| {
                                matches!(dependency,
                                Dependency::Field(source, row, _) if source == id && row == key)
                            })
                            .map(|(dependency, _)| dependency.clone()),
                    )
                    .collect();
                for field in fields {
                    self.notify(&field);
                }
            }
            CollectionEdit::Patch { key, fields } => {
                self.notify(&Dependency::Row(id.clone(), key.clone()));
                for field in fields {
                    self.notify(&Dependency::Field(id.clone(), key.clone(), field.clone()));
                }
            }
        }
        Ok(())
    }

    fn notify(&mut self, dependency: &Dependency) {
        let observers = self.observers.get(dependency).cloned().unwrap_or_default();
        for observer in observers {
            self.dirty(&observer);
        }
        if let Some(bindings) = self.binding_observers.get(dependency) {
            self.pending_bindings.extend(bindings);
        }
    }

    /// Internal selector inputs are not writable or addressable game stores.
    pub fn interaction_changed(
        &mut self,
        node: &MountedNode,
        state: crate::bind::InteractionState,
    ) {
        self.notify(&Dependency::Interaction(node.clone(), state));
    }

    /// Notify after a computed value changes, including invalid/missing values
    /// becoming valid. The identity is the instance supplying the inherited map.
    pub fn custom_property_changed(&mut self, node: &MountedNode, name: &str) {
        self.notify(&Dependency::CustomProperty(node.clone(), name.into()));
    }

    pub fn locale_changed(&mut self) {
        self.notify(&Dependency::Locale);
    }

    /// Native document bindings share source invalidation with JS computations,
    /// but are drained by the renderer only after the store graph settles.
    pub fn bind(&mut self, binding: u64, reads: BTreeSet<Dependency>) {
        self.unbind(binding);
        for dependency in &reads {
            self.binding_observers
                .entry(dependency.clone())
                .or_default()
                .insert(binding);
        }
        self.binding_reads.insert(binding, reads);
    }

    pub fn unbind(&mut self, binding: u64) {
        if let Some(reads) = self.binding_reads.remove(&binding) {
            for dependency in reads {
                if let Some(observers) = self.binding_observers.get_mut(&dependency) {
                    observers.remove(&binding);
                    if observers.is_empty() {
                        self.binding_observers.remove(&dependency);
                    }
                }
            }
        }
        self.pending_bindings.remove(&binding);
    }

    pub fn clear_bindings(&mut self) {
        self.binding_reads.clear();
        self.binding_observers.clear();
        self.pending_bindings.clear();
    }

    pub fn take_bindings(&mut self) -> BTreeSet<u64> {
        if self.batch_depth != 0 || self.next().is_some() || !self.active.is_empty() {
            return BTreeSet::new();
        }
        std::mem::take(&mut self.pending_bindings)
    }

    /// Register a getter read in the innermost computation. The host first
    /// refreshes the requested memo using evaluation_needed, including in batch.
    pub fn read(&mut self, id: &SourceId) -> Result<(), String> {
        self.read_dependency(Dependency::Value(id.clone()))
    }

    pub fn read_dependency(&mut self, dependency: Dependency) -> Result<(), String> {
        let Some(id) = dependency.source() else {
            return Err("host dependencies are subscribed by native document bindings".into());
        };
        if !self.nodes.contains_key(id) {
            return Err(id.error("source is not registered"));
        }
        if self.active.iter().any(|e| &e.id == id) {
            return Err(id.error("reactive cycle: a computation read an active computation"));
        }
        if let Some(owner) = self.active.last()
            && self.nodes[&owner.id]
                .declared
                .as_ref()
                .is_some_and(|inputs| !inputs.contains(id))
        {
            return Err(owner
                .id
                .error(&format!("undeclared read of '{}/{}'", id.scope, id.id)));
        }
        if let Some(owner) = self.active.last_mut()
            && owner.tracking
        {
            owner.reads.insert(dependency);
        }
        Ok(())
    }

    /// The first stale dependency to refresh before a read, or this computation
    /// itself. Known dependencies are checked before callers observe the value.
    pub fn evaluation_needed(&self, id: &SourceId) -> Result<Option<SourceId>, String> {
        if self.pending_memos.is_empty()
            && self.pending_effects.is_empty()
            && self.active.is_empty()
            && self.nodes.contains_key(id)
        {
            return Ok(None);
        }
        self.needed(id, &mut BTreeSet::new())
    }

    fn needed(
        &self,
        id: &SourceId,
        visiting: &mut BTreeSet<SourceId>,
    ) -> Result<Option<SourceId>, String> {
        if !visiting.insert(id.clone()) || self.active.iter().any(|e| &e.id == id) {
            return Err(id.error("reactive cycle"));
        }
        let node = self
            .nodes
            .get(id)
            .ok_or_else(|| id.error("source is not registered"))?;
        for dependency in &node.reads {
            if let Some(source) = dependency.source()
                && let Some(next) = self.needed(source, visiting)?
            {
                return Ok(Some(next));
            }
        }
        visiting.remove(id);
        Ok(node.dirty.then(|| id.clone()))
    }

    /// Drain memos before effects, in stable ID order. Effects therefore see a
    /// settled diamond rather than one old and one new branch.
    pub fn next(&self) -> Option<SourceId> {
        if self.batch_depth != 0 || !self.active.is_empty() {
            return None;
        }
        self.pending_memos
            .first()
            .or_else(|| self.pending_effects.first())
            .cloned()
    }

    pub fn begin(&mut self, id: &SourceId) -> Result<(), String> {
        let node = self
            .nodes
            .get(id)
            .ok_or_else(|| id.error("computation is not registered"))?;
        if node.kind == Computation::Input || self.active.iter().any(|e| &e.id == id) {
            return Err(id.error("cannot evaluate an input or an active computation"));
        }
        self.active.push(Evaluation {
            id: id.clone(),
            reads: BTreeSet::new(),
            tracking: true,
            error: None,
        });
        Ok(())
    }

    /// A refused getter or setter invalidates the active result even when the
    /// user callback catches the error or continues with its previous value.
    pub fn fail(&mut self, reason: &str) {
        for evaluation in &mut self.active {
            if evaluation.error.is_none() {
                evaluation.error = Some(reason.to_owned());
            }
        }
    }

    /// On failure keep the previous dependencies as well as reads made before
    /// the error, so repairing either input can retry the computation.
    pub fn finish(&mut self, id: &SourceId, result: Result<bool, String>) -> Result<(), String> {
        if self.active.last().is_none_or(|e| &e.id != id) {
            return Err(id.error("evaluation finished out of order"));
        }
        let mut evaluation = self.active.pop().expect("checked active evaluation");
        let result = evaluation.error.take().map_or(result, Err);
        let node = self
            .nodes
            .get_mut(id)
            .expect("registered active computation");
        let old = std::mem::take(&mut node.reads);
        if result.is_err() {
            evaluation.reads.extend(old.iter().cloned());
        }
        node.reads = evaluation.reads.clone();
        node.dirty = false;
        for dependency in old.difference(&evaluation.reads) {
            self.unsubscribe(dependency, id);
        }
        for dependency in &evaluation.reads {
            self.observers
                .entry(dependency.clone())
                .or_default()
                .insert(id.clone());
        }
        self.pending_memos.remove(id);
        self.pending_effects.remove(id);
        match result {
            Ok(true) => {
                self.notify(&Dependency::Value(id.clone()));
                Ok(())
            }
            Ok(false) => Ok(()),
            Err(reason) => Err(id.error(&reason)),
        }
    }

    /// Returns the old tracking state, to restore even when user code fails.
    pub fn track(&mut self, enabled: bool) -> bool {
        self.active
            .last_mut()
            .map(|e| std::mem::replace(&mut e.tracking, enabled))
            .unwrap_or(true)
    }

    pub fn begin_batch(&mut self) {
        self.batch_depth += 1;
    }

    pub fn check_idle(&self, source: &SourceId) -> Result<(), String> {
        source.validate()?;
        if self.active.is_empty() {
            Ok(())
        } else {
            Err(source.error("cannot change mounts while evaluating a computation"))
        }
    }

    pub fn invalidate_all(&mut self) {
        self.active.clear();
        self.batch_depth = 0;
        self.pending_bindings.extend(self.binding_reads.keys());
        let derived: Vec<_> = self
            .nodes
            .iter()
            .filter(|(_, n)| n.kind != Computation::Input)
            .map(|(id, _)| id.clone())
            .collect();
        for id in derived {
            self.dirty(&id);
        }
    }

    pub fn restore_sources<'a>(
        &mut self,
        inputs: impl Iterator<Item = &'a SourceId>,
        computed: impl Iterator<Item = &'a SourceId>,
    ) -> Result<(), String> {
        self.check_idle(&SourceId::new("registration", "restore"))?;
        let expected: BTreeMap<_, _> = inputs
            .map(|id| (id.clone(), Computation::Input))
            .chain(computed.map(|id| (id.clone(), Computation::Memo)))
            .collect();
        for (id, kind) in &expected {
            if let Some(node) = self.nodes.get(id) {
                if node.kind != *kind {
                    return Err(id.error("checkpoint has a different store kind"));
                }
            } else {
                self.register(id.clone(), *kind)?;
                if *kind == Computation::Memo {
                    self.suspend(id)?;
                }
            }
        }
        let removed: Vec<_> = self
            .nodes
            .iter()
            .filter(|(id, node)| node.kind != Computation::Effect && !expected.contains_key(*id))
            .map(|(id, _)| id.clone())
            .collect();
        for id in removed {
            self.forget(&id)?;
        }
        self.invalidate_all();
        Ok(())
    }

    pub fn declare_reads(&mut self, id: &SourceId, inputs: Option<Vec<SourceId>>) {
        self.nodes
            .get_mut(id)
            .expect("registered computation")
            .declared = inputs.map(|v| v.into_iter().collect());
    }

    pub fn suspend(&mut self, id: &SourceId) -> Result<(), String> {
        self.can_write(id)?;
        let reads = std::mem::take(
            &mut self
                .nodes
                .get_mut(id)
                .expect("registered computation")
                .reads,
        );
        for dependency in reads {
            self.unsubscribe(&dependency, id);
        }
        let node = self.nodes.get_mut(id).expect("registered computation");
        node.dirty = false;
        node.enabled = false;
        self.pending_memos.remove(id);
        self.pending_effects.remove(id);
        Ok(())
    }

    pub fn forget(&mut self, id: &SourceId) -> Result<(), String> {
        self.suspend(id)?;
        let dependencies: Vec<_> = self
            .observers
            .keys()
            .chain(self.binding_observers.keys())
            .filter(|dependency| dependency.source() == Some(id))
            .cloned()
            .collect();
        for dependency in dependencies {
            self.notify(&dependency);
            if let Some(observers) = self.observers.remove(&dependency) {
                for observer in observers {
                    if let Some(node) = self.nodes.get_mut(&observer) {
                        node.reads.remove(&dependency);
                    }
                }
            }
        }
        self.nodes.remove(id);
        Ok(())
    }

    fn unsubscribe(&mut self, dependency: &Dependency, id: &SourceId) {
        if let Some(observers) = self.observers.get_mut(dependency) {
            observers.remove(id);
            if observers.is_empty() {
                self.observers.remove(dependency);
            }
        }
    }

    pub fn dispose(&mut self, scope: &str) -> Result<(), String> {
        if let Some(active) = self.active.last() {
            return Err(active.id.error("cannot dispose a scope during evaluation"));
        }
        let removed: Vec<_> = self
            .observers
            .keys()
            .chain(self.binding_observers.keys())
            .filter(|d| d.source().is_some_and(|source| source.scope == scope))
            .cloned()
            .collect();
        for dependency in &removed {
            self.notify(dependency);
        }
        self.nodes.retain(|id, _| id.scope != scope);
        for node in self.nodes.values_mut() {
            node.reads.retain(|dependency| {
                dependency
                    .source()
                    .is_none_or(|source| source.scope != scope)
            });
        }
        self.observers.retain(|dependency, observers| {
            observers.retain(|id| id.scope != scope);
            dependency
                .source()
                .is_none_or(|source| source.scope != scope)
                && !observers.is_empty()
        });
        self.pending_memos.retain(|id| id.scope != scope);
        self.pending_effects.retain(|id| id.scope != scope);
        Ok(())
    }

    pub fn end_batch(&mut self) -> Result<(), String> {
        if self.batch_depth == 0 {
            return Err("og.ui: batch ended without begin".into());
        }
        self.batch_depth -= 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(name: &str) -> SourceId {
        SourceId::new("hud", name)
    }
    fn evaluate(g: &mut Graph, name: &str, reads: &[&str], changed: bool) {
        g.begin(&id(name)).unwrap();
        for name in reads {
            g.read(&id(name)).unwrap();
        }
        g.finish(&id(name), Ok(changed)).unwrap();
    }

    #[test]
    fn diamond_batches_settle_before_effects_and_equal_memos_stop() {
        let mut g = Graph::default();
        g.register(id("count"), Computation::Input).unwrap();
        for name in ["left", "right", "sum"] {
            g.register(id(name), Computation::Memo).unwrap();
        }
        g.register(id("effect"), Computation::Effect).unwrap();
        evaluate(&mut g, "left", &["count"], true);
        evaluate(&mut g, "right", &["count"], true);
        evaluate(&mut g, "sum", &["left", "right"], true);
        evaluate(&mut g, "effect", &["sum"], false);
        assert_eq!(g.next(), None);
        g.begin_batch();
        g.changed(&id("count")).unwrap();
        assert_eq!(g.next(), None);
        assert_eq!(g.evaluation_needed(&id("sum")).unwrap(), Some(id("left")));
        g.end_batch().unwrap();
        assert_eq!(g.next(), Some(id("left")));
        evaluate(&mut g, "left", &["count"], false);
        assert_eq!(g.next(), Some(id("right")));
        evaluate(&mut g, "right", &["count"], false);
        assert_eq!(g.next(), None);
        g.changed(&id("count")).unwrap();
        evaluate(&mut g, "left", &["count"], true);
        evaluate(&mut g, "right", &["count"], true);
        assert_eq!(g.next(), Some(id("sum")));
        evaluate(&mut g, "sum", &["left", "right"], true);
        assert_eq!(g.next(), Some(id("effect")));
    }

    #[test]
    fn branches_untrack_cycles_and_reentrant_writes() {
        let mut g = Graph::default();
        for name in ["condition", "a", "b"] {
            g.register(id(name), Computation::Input).unwrap();
        }
        g.register(id("choice"), Computation::Memo).unwrap();
        evaluate(&mut g, "choice", &["condition", "a"], true);
        g.changed(&id("condition")).unwrap();
        g.begin(&id("choice")).unwrap();
        g.read(&id("condition")).unwrap();
        g.read(&id("b")).unwrap();
        let tracking = g.track(false);
        g.read(&id("a")).unwrap();
        g.track(tracking);
        assert!(g.read(&id("choice")).unwrap_err().contains("cycle"));
        assert!(g.can_write(&id("a")).unwrap_err().contains("hud"));
        g.finish(&id("choice"), Ok(true)).unwrap();
        g.changed(&id("a")).unwrap();
        assert_eq!(g.next(), None);
        g.changed(&id("b")).unwrap();
        assert_eq!(g.next(), Some(id("choice")));
    }

    #[test]
    fn document_bindings_follow_fields_and_replace_branch_dependencies() {
        let mut g = Graph::default();
        for source in ["rows", "a", "b"] {
            g.register(id(source), Computation::Input).unwrap();
        }
        let field = |key: &str, name: &str| Dependency::Field(id("rows"), key.into(), name.into());
        g.bind(1, BTreeSet::from([Dependency::Keys(id("rows"))]));
        g.bind(2, BTreeSet::from([field("left", "label")]));
        g.bind(3, BTreeSet::from([field("right", "label")]));
        g.bind(4, BTreeSet::from([field("left", "price")]));
        assert!(g.take_bindings().is_empty());
        g.collection_changed(
            &id("rows"),
            &CollectionEdit::Patch {
                key: "left".into(),
                fields: vec!["label".into()],
            },
        )
        .unwrap();
        assert_eq!(g.take_bindings(), BTreeSet::from([2]));
        g.collection_changed(
            &id("rows"),
            &CollectionEdit::Move {
                key: "left".into(),
                before: Some("right".into()),
                from: 1,
                to: 0,
            },
        )
        .unwrap();
        assert_eq!(g.take_bindings(), BTreeSet::from([1]));
        g.collection_changed(
            &id("rows"),
            &CollectionEdit::Remove {
                key: "left".into(),
                index: 0,
            },
        )
        .unwrap();
        assert_eq!(g.take_bindings(), BTreeSet::from([1, 2, 4]));
        g.bind(2, BTreeSet::from([Dependency::Value(id("a"))]));
        g.changed(&id("a")).unwrap();
        assert_eq!(g.take_bindings(), BTreeSet::from([2]));
        g.bind(2, BTreeSet::from([Dependency::Value(id("b"))]));
        g.changed(&id("a")).unwrap();
        assert!(g.take_bindings().is_empty());
        g.begin_batch();
        g.changed(&id("b")).unwrap();
        assert!(g.take_bindings().is_empty());
        g.end_batch().unwrap();
        assert_eq!(g.take_bindings(), BTreeSet::from([2]));
        g.unbind(2);
        g.changed(&id("b")).unwrap();
        assert!(g.take_bindings().is_empty());
        g.forget(&id("rows")).unwrap();
        assert_eq!(g.take_bindings(), BTreeSet::from([1, 3, 4]));
    }

    #[test]
    fn inherited_property_changes_target_the_supplier_instance_and_property() {
        let mut graph = Graph::default();
        let parents = [
            MountedNode {
                mount: id("first"),
                key: 7,
            },
            MountedNode {
                mount: id("second"),
                key: 7,
            },
            MountedNode {
                mount: id("first"),
                key: 11,
            },
        ];
        let read = |parent: &MountedNode, name: &str| {
            BTreeSet::from([Dependency::CustomProperty(parent.clone(), name.into())])
        };
        for (binding, parent) in parents.iter().enumerate() {
            graph.bind(binding as u64, read(parent, "--ink"));
        }
        graph.bind(3, read(&parents[0], "--gap"));
        assert!(graph.take_bindings().is_empty());
        graph.custom_property_changed(&parents[0], "--ink");
        assert_eq!(graph.take_bindings(), BTreeSet::from([0]));
        graph.custom_property_changed(&parents[0], "--gap");
        assert_eq!(graph.take_bindings(), BTreeSet::from([3]));
        // A fallback may stop reading one name and start reading another.
        graph.bind(0, read(&parents[0], "--fallback"));
        graph.custom_property_changed(&parents[0], "--ink");
        assert!(graph.take_bindings().is_empty());
        graph.custom_property_changed(&parents[0], "--fallback");
        assert_eq!(graph.take_bindings(), BTreeSet::from([0]));
        graph.unbind(0);
        graph.custom_property_changed(&parents[0], "--fallback");
        assert!(graph.take_bindings().is_empty());
        graph.custom_property_changed(&parents[1], "--ink");
        assert_eq!(graph.take_bindings(), BTreeSet::from([1]));
        graph.custom_property_changed(&parents[2], "--ink");
        assert_eq!(graph.take_bindings(), BTreeSet::from([2]));
    }
}
