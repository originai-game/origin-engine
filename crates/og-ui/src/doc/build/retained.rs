use super::*;
use crate::bind::Expr;
use crate::doc::topology::{Instance, Materialization, Topology};
use crate::reactive::{
    SourceId,
    bindings::{BindingContext, BindingExpression, BindingId, Bindings},
    graph::MountedNode,
};

mod registration;
mod selection;
mod structural;
mod virtualized;
use structural::{BranchInstance, Child, EachInstance};

#[derive(Clone, Debug, Default)]
pub(super) struct CachedInputs {
    expressions: Vec<(Expr, Value)>,
    properties: Vec<(PropValue, Ty, Option<V>)>,
}

impl CachedInputs {
    pub fn expression(&self, expression: &Expr) -> Value {
        self.expressions
            .iter()
            .find(|(e, _)| e == expression)
            .expect("compiled expression input")
            .1
            .clone()
    }

    pub fn property(&self, property: &PropValue, ty: Ty) -> Option<V> {
        self.properties
            .iter()
            .find(|(p, t, _)| p == property && *t == ty)
            .expect("compiled property input")
            .2
            .clone()
    }
}

#[derive(Clone, Debug)]
enum Input {
    Expression(Expr),
    Property(PropValue, Ty),
    Structure,
}

#[derive(Clone)]
struct ElementInstance {
    document: Arc<UiDocument>,
    index: usize,
    context: BindingContext,
    path: String,
    id: String,
    cached: CachedInputs,
    resolved: Option<ResolvedNode>,
    style_dirty: bool,
    theme_epoch: u64,
    children: Vec<Child>,
    bindings: Vec<BindingId>,
    /// Its render node and the parent it was written under, while it has one.
    slot: Option<usize>,
    parent_slot: usize,
    /// Its own node and its widget's internal nodes: what it owns in the
    /// render tree and retires when it stops writing them.
    owned: Vec<usize>,
    /// The render keys of those nodes and of its links, for input routing.
    keys: Vec<u64>,
    /// Its probe record, render fields and art slots.
    record: super::mount::DocRecord,
    memories: Vec<u64>,
    /// Built again every frame: it plays Rive, hosts a code slot, answers
    /// named actions, or its widget memories are in motion.
    volatile: bool,
    materialization: Materialization,
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct Registration {
    document: String,
    inputs: BTreeMap<String, SourceId>,
}

#[derive(Clone, Default)]
pub struct RetainedHost {
    pub topology: Topology,
    bindings: Bindings,
    inputs: BTreeMap<BindingId, (MountedNode, Input)>,
    pending_bindings: BTreeSet<BindingId>,
    elements: BTreeMap<MountedNode, ElementInstance>,
    roots: BTreeMap<SourceId, Child>,
    registrations: BTreeMap<SourceId, Registration>,
    managed: Option<BTreeMap<SourceId, Registration>>,
    pending_mounts: BTreeSet<SourceId>,
    restoring: bool,
    each: BTreeMap<MountedNode, EachInstance>,
    branches: BTreeMap<MountedNode, BranchInstance>,
    collections: BTreeMap<SourceId, BTreeSet<MountedNode>>,
    dirty: BTreeSet<MountedNode>,
    inherited: BTreeMap<(MountedNode, String), BTreeSet<MountedNode>>,
    by_key: BTreeMap<u64, MountedNode>,
    interaction: [u64; 4],
    viewport: Option<crate::doc::media::Context>,
    theme: Option<crate::Theme>,
    theme_epoch: u64,
    media: [BTreeSet<MountedNode>; 3],
    materializing: Option<Materialization>,
    built_tick: Option<u64>,
    /// Elements whose render nodes must be written again this frame.
    emit: BTreeSet<MountedNode>,
    /// Elements built every frame ([`ElementInstance::volatile`]).
    volatile: BTreeSet<MountedNode>,
    /// The element that owns each render key (its nodes', its links').
    owner_of: std::collections::HashMap<u64, MountedNode>,
    /// The widget memories mounted elements keep while they are not built.
    memories: std::collections::HashMap<u64, u32>,
    /// Last frame's interaction keys (hover, press, focus, activation).
    input: Vec<u64>,
    /// What every element's nodes depend on beyond its own inputs.
    signature: Option<Signature>,
    /// Virtual containers and the scroll offset they were emitted at.
    scrollers: BTreeMap<MountedNode, f32>,
    pub evaluated: usize,
    pub resolved: usize,
    /// Elements whose render nodes were written this frame.
    pub reconciled: usize,
    /// Structural-selector work since the last refresh.
    pub css: CssWork,
    profiles: BTreeMap<String, crate::doc::matching::Profile>,
}

/// Structural-selector work of one refresh, for bounded-cost checks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CssWork {
    /// Sibling edits and selector-fact changes consumed.
    pub changes: usize,
    /// Sibling intervals invalidated lazily (each costs O(log n)).
    pub intervals: usize,
    /// Structural selectors evaluated, and elements visited for them.
    pub selectors: usize,
    pub candidates: usize,
}

/// The frame-wide inputs of every element's emission: a change emits all.
#[derive(Clone, Debug, PartialEq)]
struct Signature {
    viewport: crate::Viewport,
    unit: f32,
    direction: og_i18n::Direction,
    locale: String,
    style: String,
    dress: usize,
    players: u32,
    nav_on: bool,
}

impl std::fmt::Debug for RetainedHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetainedHost")
            .field("roots", &self.roots)
            .field("elements", &self.elements.len())
            .finish()
    }
}

impl RetainedHost {
    pub(crate) fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }

    pub(crate) fn deferred_focus_path(&self, key: u64) -> Option<&str> {
        let identity = self.by_key.get(&key)?;
        let node = self.topology.node(identity)?;
        if node.materialization != Materialization::Deferred
            || node
                .states
                .contains(&crate::bind::InteractionState::Disabled)
        {
            return None;
        }
        let element = &self.elements[identity];
        if element.resolved.as_ref().is_some_and(|resolved| {
            resolved.values.has("visible") && !resolved.values.flag("visible")
        }) {
            return None;
        }
        Some(&element.path)
    }

    pub(crate) fn save(&self) -> Value {
        serde_json::to_value(self.registrations.iter().collect::<Vec<_>>())
            .expect("serializable mount registrations")
    }

    pub(crate) fn restore(value: &Value) -> Result<Self, String> {
        let registrations: Vec<(SourceId, Registration)> = if value.is_null() {
            Vec::new()
        } else {
            serde_json::from_value(value.clone())
                .map_err(|error| format!("invalid UI mounts: {error}"))?
        };
        let mut out = Self::default();
        for (id, registration) in registrations {
            id.validate()?;
            if out.registrations.insert(id, registration).is_some() {
                return Err("duplicate UI mount in snapshot".into());
            }
        }
        out.restoring = !out.registrations.is_empty();
        out.pending_mounts.extend(out.registrations.keys().cloned());
        Ok(out)
    }

    fn restore_instances(&mut self, ui: &mut Ui, env: &dyn Env) -> Result<(), String> {
        while let Some(mount) = self.pending_mounts.first().cloned() {
            let registration = self.registrations[&mount].clone();
            let document = ui
                .docs
                .docs
                .get(&registration.document)
                .cloned()
                .ok_or_else(|| format!("missing document '{}'", registration.document))?;
            let context = BindingContext {
                node: MountedNode {
                    mount: mount.clone(),
                    key: 0,
                },
                inputs: Arc::new(registration.inputs.clone()),
                rows: Arc::default(),
                row_key: String::new(),
                selector_nodes: Arc::default(),
            };
            match self.compile(
                ui,
                (document.clone(), document.root),
                context,
                None,
                format!("{}:{}", mount.scope, mount.id),
                env,
            ) {
                Ok(root) => {
                    self.pending_mounts.remove(&mount);
                    self.roots.insert(mount, root);
                }
                Err(error) => {
                    self.remove_mount(ui, &mount);
                    self.pending_mounts.insert(mount.clone());
                    self.registrations.insert(mount, registration);
                    return Err(error);
                }
            }
        }
        if self.restoring {
            self.topology.take_changes();
            self.topology.take_fact_changes();
        }
        self.restoring = false;
        Ok(())
    }

    pub(super) fn document_replaced(&mut self, ui: &mut Ui, document: &str) {
        self.profiles.remove(document);
        let mounts: Vec<_> = self
            .registrations
            .iter()
            .filter(|(_, registration)| registration.document == document)
            .map(|(mount, _)| mount.clone())
            .collect();
        for mount in mounts {
            self.rebuild_mount(ui, &mount);
        }
    }

    fn rebuild_mount(&mut self, ui: &mut Ui, mount: &SourceId) {
        let registration = self.registrations[mount].clone();
        self.remove_mount(ui, mount);
        self.pending_mounts.insert(mount.clone());
        self.registrations.insert(mount.clone(), registration);
    }

    fn bind(
        &mut self,
        ui: &mut Ui,
        element: &mut ElementInstance,
        input: Input,
        expression: BindingExpression,
        env: &dyn Env,
    ) -> Result<(), String> {
        let already_bound = match &input {
            Input::Expression(e) => element.cached.expressions.iter().any(|(old, _)| old == e),
            Input::Property(p, ty) => element
                .cached
                .properties
                .iter()
                .any(|(old, t, _)| old == p && t == ty),
            Input::Structure => false,
        };
        if already_bound {
            return Ok(());
        }
        let topology = &self.topology;
        let id = self.bindings.insert_expression(
            &mut ui.reactive,
            element.context.clone(),
            expression,
            env,
            |node, state| {
                topology
                    .node(node)
                    .is_some_and(|n| n.states.contains(&state))
            },
        );
        self.inputs
            .insert(id, (element.context.node.clone(), input.clone()));
        element.bindings.push(id);
        let value = self.bindings.value(id).map_err(Clone::clone)?.clone();
        match &input {
            Input::Expression(e) => element.cached.expressions.push((e.clone(), value)),
            Input::Property(p, ty) => {
                element
                    .cached
                    .properties
                    .push((p.clone(), *ty, coerce(ui, *ty, &value)))
            }
            Input::Structure => unreachable!("structural bindings are owned by blocks"),
        }
        Ok(())
    }

    fn remove_mount(&mut self, ui: &mut Ui, mount: &SourceId) {
        let bindings: Vec<_> = self
            .inputs
            .iter()
            .filter(|(_, (node, _))| &node.mount == mount)
            .map(|(id, _)| *id)
            .collect();
        for id in bindings {
            self.bindings.remove(&mut ui.reactive, id);
            self.inputs.remove(&id);
            self.pending_bindings.remove(&id);
        }
        let roots: Vec<_> = self
            .topology
            .children(None, 0..usize::MAX)
            .filter(|n| &n.mount == mount)
            .cloned()
            .collect();
        for root in roots {
            self.topology.remove(&root).expect("live mount root");
        }
        let removed: Vec<_> = self
            .elements
            .keys()
            .filter(|node| &node.mount == mount)
            .cloned()
            .collect();
        for node in removed {
            ui.state.virtual_lists.remove(&node.key);
            self.release_element(ui, &node);
        }
        self.elements.retain(|node, _| &node.mount != mount);
        self.emit.retain(|node| &node.mount != mount);
        self.volatile.retain(|node| &node.mount != mount);
        self.scrollers.retain(|node, _| &node.mount != mount);
        self.each.retain(|node, _| &node.mount != mount);
        self.branches.retain(|node, _| &node.mount != mount);
        self.collections.retain(|_, readers| {
            readers.retain(|node| &node.mount != mount);
            !readers.is_empty()
        });
        self.by_key.retain(|_, node| &node.mount != mount);
        self.dirty.retain(|node| &node.mount != mount);
        self.inherited.retain(|(node, _), _| &node.mount != mount);
        for readers in &mut self.media {
            readers.retain(|node| &node.mount != mount);
        }
        self.roots.remove(mount);
        self.registrations.remove(mount);
        self.pending_mounts.remove(mount);
    }

    fn compile(
        &mut self,
        ui: &mut Ui,
        source: (Arc<UiDocument>, usize),
        mut context: BindingContext,
        parent: Option<&MountedNode>,
        id: String,
        env: &dyn Env,
    ) -> Result<Child, String> {
        let (doc, index) = source;
        let node = &doc.nodes[index];
        if matches!(node.kind, NodeKind::For { .. }) {
            return self.compile_each(ui, (doc, index), context, parent, id, env);
        }
        if matches!(node.kind, NodeKind::If(_)) {
            return self.compile_branch(ui, (doc, index), context, parent, id, env);
        }
        let NodeKind::Element(_) = node.kind else {
            return Err(format!(
                "{}:{}: structural mount compilation is not connected yet",
                doc.source, node.line
            ));
        };
        let path = parent.map_or_else(|| id.clone(), |p| format!("{}/{id}", self.elements[p].path));
        context.node.key = crate::id::key_of_path(&path);
        Arc::make_mut(&mut context.selector_nodes).insert(index, context.node.clone());
        let identity = context.node.clone();
        let materialization = parent.map_or(Materialization::Visible, |parent| {
            let element = &self.elements[parent];
            if matches!(
                element.document.nodes[element.index].kind,
                NodeKind::Element("list" | "cells")
            ) {
                Materialization::Deferred
            } else {
                self.topology.node(parent).unwrap().materialization
            }
        });
        self.by_key.insert(identity.key, identity.clone());
        self.topology.insert(
            parent,
            self.topology.child_count(parent),
            Instance {
                identity: identity.clone(),
                document: doc.clone(),
                document_node: index,
                attributes: BTreeMap::new(),
                direct_text: String::new(),
                states: BTreeSet::new(),
                materialization,
            },
        )?;
        let mut element = ElementInstance {
            document: doc.clone(),
            index,
            context,
            path,
            id: id.rsplit('/').next().unwrap().to_owned(),
            cached: CachedInputs::default(),
            resolved: None,
            style_dirty: true,
            theme_epoch: self.theme_epoch,
            children: Vec::new(),
            bindings: Vec::new(),
            slot: None,
            parent_slot: NONE,
            owned: Vec::new(),
            keys: Vec::new(),
            record: Default::default(),
            memories: Vec::new(),
            volatile: matches!(node.kind, NodeKind::Element("slot"))
                || node
                    .triggers
                    .iter()
                    .any(|trigger| matches!(trigger.when, When::Action(_))),
            materialization,
        };
        let properties = node
            .props
            .iter()
            .chain(node.at.iter().flat_map(|(_, props)| props));
        for p in properties {
            let expression = match &p.value {
                PropValue::Lit(_) => continue,
                PropValue::Bind(e, format) => BindingExpression::Expression(
                    format.map_or_else(|| e.clone(), |f| Expr::Format(f, Box::new(e.clone()))),
                ),
                PropValue::Model(path) => BindingExpression::Expression(Expr::Path(path.clone())),
                PropValue::Template(template) => BindingExpression::Template(template.clone()),
            };
            self.bind(
                ui,
                &mut element,
                Input::Property(p.value.clone(), PROPS[p.spec].ty),
                expression,
                env,
            )?;
        }
        for (_, expression) in &node.selector_attrs {
            self.bind(
                ui,
                &mut element,
                Input::Expression(expression.clone()),
                BindingExpression::Expression(expression.clone()),
                env,
            )?;
        }
        self.sync_facts(ui, &element, env)?;
        let expressions = node
            .rules
            .iter()
            .filter_map(|rule| rule.condition.as_ref())
            .chain(
                node.span_rules
                    .iter()
                    .filter_map(|rule| rule.condition.as_ref()),
            )
            .chain(node.groups.iter().filter_map(|group| group.bind.as_ref()))
            .chain(node.triggers.iter().flat_map(|trigger| {
                let when = match &trigger.when {
                    When::Change(expression) => Some(expression),
                    _ => None,
                };
                let action = match &trigger.act {
                    Act::Set(_, expression) => Some(expression),
                    _ => None,
                };
                when.into_iter().chain(action)
            }));
        for e in expressions {
            self.bind(
                ui,
                &mut element,
                Input::Expression(e.clone()),
                BindingExpression::Expression(e.clone()),
                env,
            )?;
        }
        let context = element.context.clone();
        self.elements.insert(identity.clone(), element);
        let mut children = Vec::new();
        for &child in &node.children {
            let n = &doc.nodes[child];
            let id = if n.id.is_empty() {
                format!("node{child}")
            } else {
                n.id.clone()
            };
            children.push(self.compile(
                ui,
                (doc.clone(), child),
                context.clone(),
                Some(&identity),
                id,
                env,
            )?);
        }
        self.elements.get_mut(&identity).unwrap().children = children;
        self.dirty.insert(identity.clone());
        if self.elements[&identity].volatile {
            self.volatile.insert(identity.clone());
        }
        if let Some(parent) = parent {
            self.emit.insert(parent.clone());
        }
        Ok(Child::Element(identity))
    }

    fn sync_facts(
        &mut self,
        ui: &mut Ui,
        element: &ElementInstance,
        env: &dyn Env,
    ) -> Result<(), String> {
        use crate::bind::InteractionState::{Checked, Disabled, Invalid, PlaceholderShown};
        let identity = &element.context.node;
        let build = element.build(env, Arc::default());
        let node = &element.document.nodes[element.index];
        let mut classes = BTreeSet::new();
        let mut has_classes = false;
        for (name, expression) in &node.selector_attrs {
            let value = build.eval(expression);
            if name == "class" {
                has_classes = true;
                if let Some(value) = value.as_str() {
                    classes.extend(value.split_whitespace().map(str::to_owned));
                }
            } else if let Some(class) = name.strip_prefix("class:") {
                has_classes = true;
                if truthy(&value) {
                    classes.insert(class.to_owned());
                }
                self.topology.set_attribute(identity, name, Some(value))?;
            } else {
                self.topology.set_attribute(identity, name, Some(value))?;
            }
        }
        if has_classes {
            self.topology.set_attribute(
                identity,
                "class",
                Some(json!(classes.into_iter().collect::<Vec<_>>().join(" "))),
            )?;
        }
        let mut facts = BTreeMap::new();
        for prop in &node.props {
            let spec = &PROPS[prop.spec];
            let value = match build.value(ui, spec.ty, &prop.value) {
                Some(V::Bool(v)) => Some(json!(v)),
                Some(V::Num(v)) => Some(json!(v)),
                Some(V::Str(v)) => Some(json!(v)),
                Some(V::Strs(v)) => Some(json!(v)),
                _ => None,
            };
            if let Some(value) = value {
                facts.insert(spec.name, value);
            }
        }
        let flag = |name| facts.get(name).is_some_and(truthy);
        self.topology.set_text(
            identity,
            facts
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        )?;
        for (state, enabled) in [
            (Disabled, flag("disabled")),
            (Checked, flag("checked")),
            // A field's text its `type` does not take is invalid too.
            (
                Invalid,
                flag("invalid")
                    || facts
                        .get("type")
                        .and_then(Value::as_str)
                        .and_then(crate::InputType::from_name)
                        .zip(facts.get("input").and_then(Value::as_str))
                        .is_some_and(|(t, v)| t.mismatch(v)),
            ),
            (
                PlaceholderShown,
                facts
                    .get("placeholder")
                    .and_then(Value::as_str)
                    .is_some_and(|v| !v.is_empty())
                    && facts
                        .get("input")
                        .and_then(Value::as_str)
                        .is_none_or(str::is_empty),
            ),
        ] {
            if self
                .topology
                .node(identity)
                .unwrap()
                .states
                .contains(&state)
                != enabled
            {
                self.topology.set_state(identity, state, enabled)?;
                ui.reactive.graph.interaction_changed(identity, state);
            }
        }
        Ok(())
    }

    fn refresh(&mut self, ui: &mut Ui, env: &dyn Env) -> Result<(), String> {
        self.restore_instances(ui, env)?;
        self.refresh_collections(ui, env)?;
        self.sync_interaction(ui)?;
        if self
            .theme
            .replace(*ui.theme())
            .is_some_and(|old| old != *ui.theme())
        {
            self.theme_epoch += 1;
        }
        let viewport = crate::doc::media::Context {
            width: ui.viewport.width,
            height: ui.viewport.height,
            reduced_motion: ui.theme().reduced_motion,
        };
        if let Some(old) = self.viewport.replace(viewport) {
            for (axis, changed) in [
                old.width != viewport.width,
                old.height != viewport.height,
                old.reduced_motion != viewport.reduced_motion,
            ]
            .into_iter()
            .enumerate()
            {
                if changed {
                    self.dirty.extend(self.media[axis].iter().cloned());
                }
            }
        }
        self.evaluated = 0;
        self.resolved = 0;
        self.css = CssWork::default();
        loop {
            let topology = &self.topology;
            let changes = self.bindings.refresh(&mut ui.reactive, env, |node, state| {
                topology
                    .node(node)
                    .is_some_and(|n| n.states.contains(&state))
            });
            self.evaluated += changes.evaluated;
            self.pending_bindings.extend(changes.changed);
            if changes.evaluated == 0 && self.pending_bindings.is_empty() {
                break;
            }
            let pending: Vec<_> = self.pending_bindings.iter().copied().collect();
            for &id in &pending {
                if let Some((node, Input::Structure)) = self.inputs.get(&id) {
                    let node = node.clone();
                    let enabled = truthy(self.bindings.value(id).map_err(Clone::clone)?);
                    if let Err(error) = self.refresh_branch(ui, &node, enabled, env) {
                        self.rebuild_mount(ui, &node.mount);
                        return Err(error);
                    }
                }
            }
            for &id in &pending {
                if self.inputs.contains_key(&id) {
                    self.bindings.value(id).map_err(Clone::clone)?;
                }
            }
            let mut facts_changed = BTreeSet::new();
            for id in pending {
                self.pending_bindings.remove(&id);
                let Some((node, input)) = self.inputs.get(&id) else {
                    continue;
                };
                if matches!(input, Input::Structure) {
                    continue;
                }
                let value = self.bindings.value(id).map_err(Clone::clone)?.clone();
                let element = self.elements.get_mut(node).unwrap();
                match input {
                    Input::Expression(e) => {
                        if element.document.nodes[element.index]
                            .selector_attrs
                            .iter()
                            .any(|(_, expression)| expression == e)
                        {
                            facts_changed.insert(node.clone());
                        }
                        element
                            .cached
                            .expressions
                            .iter_mut()
                            .find(|(old, _)| old == e)
                            .unwrap()
                            .1 = value
                    }
                    Input::Property(p, ty) => {
                        facts_changed.insert(node.clone());
                        element
                            .cached
                            .properties
                            .iter_mut()
                            .find(|(old, t, _)| old == p && t == ty)
                            .unwrap()
                            .2 = coerce(ui, *ty, &value)
                    }
                    Input::Structure => unreachable!(),
                }
                self.dirty.insert(node.clone());
            }
            for node in facts_changed {
                let element = self.elements.remove(&node).unwrap();
                let result = self.sync_facts(ui, &element, env);
                self.elements.insert(node, element);
                result?;
            }
        }
        self.consume_selector_changes(ui)?;
        self.resolve_invalidated_styles();
        while let Some(node) = self.dirty.pop_first() {
            self.elements.get_mut(&node).unwrap().style_dirty = true;
            if self.topology.node(&node).unwrap().materialization != Materialization::Deferred {
                self.resolve(ui, &node, env)?;
            }
        }
        Ok(())
    }

    /// Materialized nodes in invalidated style intervals resolve before they
    /// draw; deferred rows keep their interval until they materialize.
    fn resolve_invalidated_styles(&mut self) {
        for (parent, range) in self.topology.take_style_invalidations() {
            let virtual_rows = parent.as_ref().and_then(|parent| {
                let element = self.elements.get(parent)?;
                matches!(
                    element.document.nodes[element.index].kind,
                    NodeKind::Element("list" | "cells")
                )
                .then_some(element.children.first())
                .flatten()
            });
            let stale: Vec<MountedNode> = match virtual_rows {
                Some(Child::Each(each)) => {
                    let each = &self.each[each];
                    each.materialized
                        .iter()
                        .filter_map(|key| each.rows.get(key))
                        .flatten()
                        .filter_map(|child| match child {
                            Child::Element(identity) => Some(identity.clone()),
                            _ => None,
                        })
                        .filter(|identity| {
                            self.topology
                                .position(identity)
                                .is_some_and(|at| range.contains(&at))
                        })
                        .collect()
                }
                _ => self
                    .topology
                    .children(parent.as_ref(), range)
                    .filter(|identity| {
                        self.topology
                            .node(identity)
                            .is_some_and(|node| node.materialization != Materialization::Deferred)
                    })
                    .cloned()
                    .collect(),
            };
            self.dirty.extend(stale);
        }
    }

    fn sync_interaction(&mut self, ui: &mut Ui) -> Result<(), String> {
        use crate::bind::InteractionState::{Active, Focus, FocusVisible, FocusWithin, Hover};
        let next = [
            ui.state.hot,
            if ui.input.down { ui.state.active } else { 0 },
            ui.state.focus,
            if ui.state.focus_visible {
                ui.state.focus
            } else {
                0
            },
        ];
        let previous = std::mem::replace(&mut self.interaction, next);
        for (i, state) in [Hover, Active, Focus, FocusVisible].into_iter().enumerate() {
            if previous[i] == next[i] {
                continue;
            }
            for (key, enabled) in [(previous[i], false), (next[i], true)] {
                if let Some(node) = self.by_key.get(&key) {
                    self.topology.set_state(node, state, enabled)?;
                    ui.reactive.graph.interaction_changed(node, state);
                }
            }
        }
        if previous[2] != next[2] {
            for (key, enabled) in [(previous[2], false), (next[2], true)] {
                let mut ancestor = self.by_key.get(&key).cloned();
                while let Some(node) = ancestor {
                    self.topology.set_state(&node, FocusWithin, enabled)?;
                    ui.reactive.graph.interaction_changed(&node, FocusWithin);
                    ancestor = self.topology.parent(&node).cloned();
                }
            }
        }
        Ok(())
    }

    fn resolve(
        &mut self,
        ui: &mut Ui,
        identity: &MountedNode,
        env: &dyn Env,
    ) -> Result<(), String> {
        let parent = self.topology.parent(identity).cloned();
        if let Some(parent) = &parent
            && (self.dirty.remove(parent)
                || self.elements[parent].resolved.is_none()
                || self.elements[parent].style_dirty
                || self.elements[parent].theme_epoch != self.theme_epoch
                || self.topology.style_is_stale(parent) == Some(true))
        {
            self.resolve(ui, parent, env)?;
        }
        let structurally_stale =
            ui.docs.structure.ancestors && self.topology.style_is_stale(identity) == Some(true);
        let inherited = parent.as_ref().map_or_else(Arc::default, |p| {
            self.elements[p]
                .resolved
                .as_ref()
                .unwrap()
                .values
                .cascade
                .computed
                .clone()
        });
        let e = self.elements.get_mut(identity).unwrap();
        for id in &e.bindings {
            if let Some((_, Input::Property(property, Ty::Paint))) = self.inputs.get(id) {
                e.cached
                    .properties
                    .iter_mut()
                    .find(|(p, ty, _)| p == property && *ty == Ty::Paint)
                    .unwrap()
                    .2 = coerce(
                    ui,
                    Ty::Paint,
                    self.bindings.value(*id).map_err(Clone::clone)?,
                );
            }
        }
        let build = e.build(env, inherited);
        let mut probe = Built {
            path: e.path.clone(),
            doc: e.document.name.clone(),
            node: e.index,
            line: e.document.nodes[e.index].line,
            mount: identity.mount.id.clone(),
            ..Default::default()
        };
        let topology = &self.topology;
        let mut work = crate::doc::matching::MatchWork::default();
        let values = ui.resolve(
            &build,
            &e.document.nodes[e.index],
            &e.path,
            &mut probe,
            &build.inherited,
            &mut |selector| {
                crate::doc::matching::matches(topology, &e.document, selector, identity, &mut work)
            },
        );
        self.css.selectors += work.selectors;
        self.css.candidates += work.candidates;
        ui.warnings
            .extend(values.cascade.diagnostics.iter().map(|error| {
                format!(
                    "{}:{} ({}): {error}",
                    e.document.source, e.document.nodes[e.index].line, e.path
                )
            }));
        let text = values.str("text").unwrap_or_default().to_owned();
        let previous = e.resolved.as_ref().map(|r| &r.values.cascade);
        let reads = values.cascade.viewport_reads;
        for (axis, read) in [reads.width, reads.height, reads.reduced_motion]
            .into_iter()
            .enumerate()
        {
            if read {
                self.media[axis].insert(identity.clone());
            } else {
                self.media[axis].remove(identity);
            }
        }
        if let Some(parent) = &parent {
            if let Some(previous) = previous {
                for name in &previous.inherited_reads {
                    if let Some(readers) = self.inherited.get_mut(&(parent.clone(), name.clone())) {
                        readers.remove(identity);
                    }
                }
            }
            for name in &values.cascade.inherited_reads {
                self.inherited
                    .entry((parent.clone(), name.clone()))
                    .or_default()
                    .insert(identity.clone());
            }
        }
        if let Some(previous) = previous {
            let names: BTreeSet<_> = previous
                .computed
                .keys()
                .chain(values.cascade.computed.keys())
                .collect();
            for name in names {
                if previous.computed.get(name) != values.cascade.computed.get(name)
                    && let Some(readers) = self.inherited.get(&(identity.clone(), name.clone()))
                {
                    self.dirty.extend(readers.iter().cloned());
                }
            }
        }
        let resolved = ResolvedNode {
            id: e.id.clone(),
            path: e.path.clone(),
            key: identity.key,
            values,
            probe,
        };
        self.elements.get_mut(identity).unwrap().resolved = Some(resolved);
        self.emit.insert(identity.clone());
        self.elements.get_mut(identity).unwrap().style_dirty = false;
        self.elements.get_mut(identity).unwrap().theme_epoch = self.theme_epoch;
        self.topology.set_text(identity, text)?;
        self.topology.style_resolved(identity)?;
        if structurally_stale {
            // Descendant and child selectors can read this element's position.
            self.topology
                .invalidate_styles(Some(identity), 0..usize::MAX)?;
        }
        self.resolved += 1;
        Ok(())
    }

    fn emit(
        &mut self,
        ui: &mut Ui,
        child: &Child,
        env: &dyn Env,
        events: &mut Vec<DocEvent>,
        writes: &mut Vec<(BindingContext, String, Value)>,
    ) -> Result<(), String> {
        match child {
            Child::Element(identity) => self.emit_element(ui, identity, false, env, events, writes),
            Child::Branch(identity) => {
                for child in self.branches[identity].children.clone() {
                    self.emit(ui, &child, env, events, writes)?;
                }
                Ok(())
            }
            Child::Each(identity) => {
                let each = &self.each[identity];
                let children: Vec<_> = if each.order.is_empty() {
                    each.empty.clone()
                } else {
                    each.order
                        .iter()
                        .flat_map(|key| each.rows[key].iter().cloned())
                        .collect()
                };
                for child in children {
                    self.emit(ui, &child, env, events, writes)?;
                }
                Ok(())
            }
        }
    }

    /// Writes element `identity`'s render node under the open element, or,
    /// when nothing it is built from changed, puts the node it already has
    /// back without visiting its subtree. `in_place`: the node is written
    /// again into the slot its parent already lists (its parent is not
    /// built this frame).
    fn emit_element(
        &mut self,
        ui: &mut Ui,
        identity: &MountedNode,
        in_place: bool,
        env: &dyn Env,
        events: &mut Vec<DocEvent>,
        writes: &mut Vec<(BindingContext, String, Value)>,
    ) -> Result<(), String> {
        let materialization = if in_place {
            self.elements[identity].materialization
        } else {
            self.materializing.unwrap_or(Materialization::Visible)
        };
        if self.topology.node(identity).unwrap().materialization != materialization {
            self.topology
                .set_materialization(identity, materialization)?;
        }
        if self.dirty.remove(identity)
            || self.elements[identity].style_dirty
            || self.elements[identity].theme_epoch != self.theme_epoch
            || self.topology.style_is_stale(identity) == Some(true)
        {
            self.resolve(ui, identity, env)?;
        }
        let parent_slot = ui.parent();
        let element = &self.elements[identity];
        if !in_place
            && !element.volatile
            && !self.emit.contains(identity)
            && let Some(slot) = element.slot
            && ui.holds(slot, identity.key, parent_slot)
        {
            ui.reattach(slot);
            return Ok(());
        }
        self.emit.remove(identity);
        self.reconciled += 1;
        let e = self.elements[identity].clone();
        if e.slot.is_some() {
            // Built before and not since retired: not appearing.
            ui.docs.seen.insert(identity.key);
        }
        if in_place {
            ui.in_place = Some((identity.key, parent_slot));
        }
        ui.owning.push(Default::default());
        ui.docs.capture.push(Default::default());
        let resolved = e.resolved.as_ref().expect("resolved mount");
        let mut b = e.build(env, resolved.values.cascade.computed.clone());
        let mut result = Ok(());
        ui.build_resolved_node(&mut b, e.index, resolved.clone(), |ui, b, virtual_props| {
            result = if let Some(props) = virtual_props {
                self.emit_virtual(ui, &e, b, props, env, events, writes)
            } else {
                e.children
                    .iter()
                    .try_for_each(|child| self.emit(ui, child, env, events, writes))
            };
        });
        ui.in_place = None;
        let owned = ui.owning.pop().expect("pushed");
        let record = ui.docs.capture.pop().expect("pushed");
        events.extend(b.events);
        writes.extend(
            b.writes
                .into_iter()
                .map(|(path, value)| (e.context.clone(), path, value)),
        );
        let slot = owned
            .slots
            .iter()
            .copied()
            .find(|&i| ui.nodes[i].key == identity.key && ui.nodes[i].parent == parent_slot);
        let mut retired: Vec<usize> = e.owned.clone();
        let mut kept = owned.slots.clone();
        kept.sort_unstable();
        retired.retain(|i| kept.binary_search(i).is_err());
        ui.retire_owned(&retired);
        for key in &e.keys {
            self.owner_of.remove(key);
        }
        let mut keys = Vec::with_capacity(owned.slots.len());
        for &i in &owned.slots {
            let node = &ui.nodes[i];
            keys.push(node.key);
            keys.extend(node.links.iter().map(|l| crate::id::child_key(node.key, l)));
        }
        for key in &keys {
            self.owner_of.insert(*key, identity.clone());
        }
        self.forget_memories(&e.memories);
        for &key in &owned.memories {
            *self.memories.entry(key).or_default() += 1;
        }
        let volatile = owned.moved
            || resolved.values.str("rive").is_some()
            || matches!(e.document.nodes[e.index].kind, NodeKind::Element("slot"))
            || e.document.nodes[e.index]
                .triggers
                .iter()
                .any(|trigger| matches!(trigger.when, When::Action(_)));
        if volatile {
            self.volatile.insert(identity.clone());
        } else {
            self.volatile.remove(identity);
        }
        let element = self.elements.get_mut(identity).unwrap();
        element.slot = slot;
        element.parent_slot = parent_slot;
        element.owned = owned.slots;
        element.keys = keys;
        element.record = record;
        element.memories = owned.memories;
        element.volatile = volatile;
        element.materialization = materialization;
        if slot.is_none() {
            // Not shown (`visible` false): its descendants' nodes go too.
            for child in e.children.clone() {
                self.release(ui, &child);
            }
        }
        result
    }

    /// Writes again, into the slots they hold, the elements whose parents
    /// were not built this frame: their ancestors' nodes stay as they are.
    fn emit_in_place(
        &mut self,
        ui: &mut Ui,
        env: &dyn Env,
        events: &mut Vec<DocEvent>,
        writes: &mut Vec<(BindingContext, String, Value)>,
    ) -> Result<(), String> {
        let mut pending: Vec<(usize, MountedNode)> = self
            .emit
            .iter()
            .filter(|identity| {
                self.elements.get(*identity).is_some_and(|e| {
                    e.slot
                        .is_some_and(|slot| ui.holds(slot, identity.key, e.parent_slot))
                })
            })
            .map(|identity| {
                let mut depth = 0;
                let mut at = self.topology.parent(identity);
                while let Some(parent) = at {
                    depth += 1;
                    at = self.topology.parent(parent);
                }
                (depth, identity.clone())
            })
            .collect();
        pending.sort();
        for (_, identity) in pending {
            if !self.emit.contains(&identity) {
                continue;
            }
            let element = &self.elements[&identity];
            let Some(slot) = element.slot else { continue };
            let parent = element.parent_slot;
            if !ui.holds(slot, identity.key, parent) {
                continue;
            }
            ui.stack.push(parent);
            let result = self.emit_element(ui, &identity, true, env, events, writes);
            ui.stack.pop();
            result?;
        }
        Ok(())
    }

    fn forget_memories(&mut self, keys: &[u64]) {
        for key in keys {
            if let Some(count) = self.memories.get_mut(key) {
                *count -= 1;
                if *count == 0 {
                    self.memories.remove(key);
                }
            }
        }
    }

    /// Retires the render nodes of `child` and everything under it: it left
    /// the render tree (removed, hidden, scrolled out of a virtual list).
    fn release(&mut self, ui: &mut Ui, child: &Child) {
        match child {
            Child::Element(identity) => {
                if self.elements[identity].slot.is_none()
                    && self.elements[identity].owned.is_empty()
                {
                    return;
                }
                self.release_element(ui, identity);
                for child in self.elements[identity].children.clone() {
                    self.release(ui, &child);
                }
            }
            Child::Branch(identity) => {
                for child in self.branches[identity].children.clone() {
                    self.release(ui, &child);
                }
            }
            Child::Each(identity) => {
                let each = &self.each[identity];
                let children: Vec<_> = each
                    .rows
                    .values()
                    .flatten()
                    .chain(&each.empty)
                    .cloned()
                    .collect();
                for child in children {
                    self.release(ui, &child);
                }
            }
        }
    }

    /// Retires one element's own render nodes (not its descendants').
    fn release_element(&mut self, ui: &mut Ui, identity: &MountedNode) {
        let Some(element) = self.elements.get_mut(identity) else {
            return;
        };
        let owned = std::mem::take(&mut element.owned);
        let keys = std::mem::take(&mut element.keys);
        let memories = std::mem::take(&mut element.memories);
        element.slot = None;
        element.record = Default::default();
        ui.retire_owned(&owned);
        for key in keys {
            self.owner_of.remove(&key);
        }
        self.forget_memories(&memories);
        self.volatile.remove(identity);
        self.scrollers.remove(identity);
    }

    /// The retained elements' records, mounted ones in tree order.
    pub(crate) fn records(&self) -> impl Iterator<Item = &super::mount::DocRecord> {
        let mut order = Vec::new();
        for root in self.roots.values() {
            self.collect_rendered(root, &mut order);
        }
        order
            .into_iter()
            .map(|identity| &self.elements[identity].record)
    }

    fn collect_rendered<'a>(&'a self, child: &'a Child, out: &mut Vec<&'a MountedNode>) {
        match child {
            Child::Element(identity) => {
                let element = &self.elements[identity];
                if element.slot.is_none() {
                    return;
                }
                out.push(identity);
                for child in &element.children {
                    self.collect_rendered(child, out);
                }
            }
            Child::Branch(identity) => {
                for child in &self.branches[identity].children {
                    self.collect_rendered(child, out);
                }
            }
            Child::Each(identity) => {
                let each = &self.each[identity];
                for key in &each.order {
                    for child in &each.rows[key] {
                        self.collect_rendered(child, out);
                    }
                }
                for child in &each.empty {
                    self.collect_rendered(child, out);
                }
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn record(&self, key: u64) -> Option<&super::mount::DocRecord> {
        let identity = self.by_key.get(&key)?;
        Some(&self.elements.get(identity)?.record)
    }

    pub(crate) fn record_mut(&mut self, key: u64) -> Option<&mut super::mount::DocRecord> {
        let identity = self.by_key.get(&key)?;
        Some(&mut self.elements.get_mut(identity)?.record)
    }

    /// A widget memory a mounted element keeps while it is not built.
    pub(crate) fn keeps_memory(&self, key: u64) -> bool {
        self.memories.contains_key(&key)
    }

    /// Marks what this frame's interaction, clock and frame-wide inputs
    /// require to be built again, before anything is emitted.
    fn mark_frame(&mut self, ui: &mut Ui) {
        let signature = Signature {
            viewport: ui.viewport,
            unit: ui.unit,
            direction: ui.direction,
            locale: ui.locale.clone(),
            style: ui.style_name.clone(),
            dress: std::sync::Arc::as_ptr(&ui.dress) as usize,
            players: ui.players,
            nav_on: ui.nav_on,
        };
        if self.signature.as_ref() != Some(&signature) {
            self.signature = Some(signature);
            self.emit.extend(self.elements.keys().cloned());
        }
        let s = &ui.state;
        let n = &ui.nav;
        let mut input = vec![
            s.hot,
            if ui.input.down { s.active } else { 0 },
            s.focus,
            n.activate,
            n.tapped,
            n.double_tapped,
            n.long_pressed,
            n.spin.0,
        ];
        input.extend(s.pad_focus);
        input.extend(n.pad_activate);
        input.retain(|&key| key != 0);
        for key in input.iter().chain(&self.input) {
            if let Some(owner) = self.owner_of.get(key) {
                self.emit.insert(owner.clone());
            }
        }
        self.input = input;
        for (identity, offset) in &self.scrollers {
            if ui.state.scroll.get(&identity.key).copied().unwrap_or(0.0) != *offset {
                self.emit.insert(identity.clone());
            }
        }
        // Nodes whose laid-out size moved: their widgets read last frame's
        // geometry (a narrow button, a list measuring its rows).
        for slot in std::mem::take(&mut ui.kept.resized) {
            if let Some(owner) = self.owner_of.get(&ui.nodes[slot].key) {
                self.emit.insert(owner.clone());
            }
        }
        self.emit.extend(self.volatile.iter().cloned());
    }
}

impl ElementInstance {
    fn build<'a>(
        &'a self,
        env: &'a dyn Env,
        inherited: Arc<super::super::variables::Computed>,
    ) -> Build<'a> {
        Build {
            doc: self.document.clone(),
            mount: self.context.node.mount.id.clone(),
            vm: &Value::Null,
            env,
            locals: Vec::new(),
            row_key: self.context.row_key.clone(),
            inherited,
            events: Vec::new(),
            writes: Vec::new(),
            cached: Some(&self.cached),
        }
    }
}

impl Ui {
    pub fn mount_document(
        &mut self,
        mount: SourceId,
        document: &str,
        inputs: BTreeMap<String, SourceId>,
        env: &dyn Env,
    ) -> Result<(), String> {
        mount.validate()?;
        let doc = self
            .docs
            .docs
            .get(document)
            .cloned()
            .ok_or_else(|| format!("missing document '{document}'"))?;
        let mut host = std::mem::take(&mut self.docs.retained);
        if host.roots.contains_key(&mount) {
            self.docs.retained = host;
            return Err(format!(
                "mount '{}:{}' already exists",
                mount.scope, mount.id
            ));
        }
        let registration = Registration {
            document: document.into(),
            inputs: inputs.clone(),
        };
        let context = BindingContext {
            node: MountedNode {
                mount: mount.clone(),
                key: 0,
            },
            inputs: Arc::new(inputs),
            rows: Arc::default(),
            row_key: String::new(),
            selector_nodes: Arc::default(),
        };
        let root = host.compile(
            self,
            (doc.clone(), doc.root),
            context,
            None,
            format!("{}:{}", mount.scope, mount.id),
            env,
        );
        let result = match root {
            Ok(root) => {
                host.registrations.insert(mount.clone(), registration);
                host.roots.insert(mount, root);
                Ok(())
            }
            Err(error) => {
                host.remove_mount(self, &mount);
                Err(error)
            }
        };
        self.docs.retained = host;
        result
    }

    pub fn unmount_document(&mut self, mount: &SourceId) {
        let mut host = std::mem::take(&mut self.docs.retained);
        host.remove_mount(self, mount);
        self.docs.retained = host;
    }

    pub fn build_retained_documents(&mut self, env: &dyn Env) -> Result<(), String> {
        let mut host = std::mem::take(&mut self.docs.retained);
        if host.built_tick == Some(self.tick()) {
            self.docs.retained = host;
            return Ok(());
        }
        let mut result = host.refresh(self, env);
        if result.is_ok() {
            host.mark_frame(self);
            host.reconciled = 0;
            let mut events = Vec::new();
            let mut writes = Vec::new();
            for root in host.roots.values().cloned().collect::<Vec<_>>() {
                if let Err(error) = host.emit(self, &root, env, &mut events, &mut writes) {
                    result = Err(error);
                    break;
                }
            }
            if result.is_ok()
                && let Err(error) = host.emit_in_place(self, env, &mut events, &mut writes)
            {
                result = Err(error);
            }
            self.docs.queued.extend(events);
            for (context, path, value) in writes {
                if let Err(error) = context.write_model(&mut self.reactive, &path, value) {
                    self.docs.retained = host;
                    return Err(error);
                }
            }
            if result.is_ok() {
                host.built_tick = Some(self.tick());
            }
        }
        self.docs.retained = host;
        result
    }

    pub fn mounted_topology(&self) -> &Topology {
        &self.docs.retained.topology
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactive::session::Command;
    use crate::{InputFrame, UiDesc, Viewport};

    #[test]
    fn retained_actions_report_the_innermost_collection_key_after_restore() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "actions.svelte",
                r#"
            <meta name="component" content="actions">
            <column>
                {#each $outer as z (z.id)}
                    <column id="Group">
                        {#each $inner as a (a.id)}
                            <button id="Inner" onclick={pick}>inner</button>
                        {/each}
                        <button id="Outer" onclick={pick}>outer</button>
                    </column>
                {/each}
                <button id="Outside" onclick={pick}>outside</button>
            </column>
        "#,
            )
            .unwrap(),
        );
        let mut inputs = BTreeMap::new();
        for (name, key) in [("outer", "group"), ("inner", "entry/#%")] {
            let source = SourceId::new("test", name);
            ui.reactive
                .execute(Command::Collection {
                    source: source.clone(),
                    key: "id".into(),
                    items: vec![json!({"id":key}).as_object().unwrap().clone()],
                })
                .unwrap();
            inputs.insert(name.into(), source);
        }
        ui.mount_document(
            SourceId::new("test", "actions"),
            "actions",
            inputs,
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui, input| {
            ui.begin(input, Viewport::new(400.0, 500.0));
            ui.end();
        };
        tick(&mut ui, InputFrame::default());
        let snapshot = ui.save_state();
        for restored in [false, true] {
            if restored {
                ui.restore_state(&snapshot).unwrap();
                tick(&mut ui, InputFrame::default());
            }
            for (label, expected) in [("inner", "entry/#%"), ("outer", "group"), ("outside", "")] {
                let path = ui
                    .probe_elements()
                    .into_iter()
                    .find(|node| node.text == label)
                    .unwrap()
                    .path;
                ui.begin(InputFrame::default(), Viewport::new(400.0, 500.0));
                ui.set_focus(&path);
                ui.end();
                assert_eq!(ui.focus_path(0), path);
                ui.take_doc_events();
                tick(
                    &mut ui,
                    InputFrame {
                        keys: vec![crate::UiKey::Enter],
                        keys_pressed: vec!["Enter"],
                        ..Default::default()
                    },
                );
                let events: Vec<_> = ui
                    .take_doc_events()
                    .into_iter()
                    .filter(|event| event.name == "pick")
                    .collect();
                assert_eq!(events.len(), 1, "{label}, restored={restored}");
                assert_eq!(events[0].row_key, expected);
                assert_eq!(events[0].path, path);
            }
        }
    }

    /// A mounted grid of labels: idle ticks evaluate and write nothing,
    /// one patched row field evaluates its one binding and writes its one
    /// element, and that label shows the new text.
    #[test]
    fn mounted_frames_write_only_what_changed() {
        let mut ui = Ui::new(UiDesc::default());
        let mut doc =
            String::from("<meta name=\"component\" content=\"grid\">\n<row id=\"Screen\">");
        let mut inputs = BTreeMap::new();
        for column in 0..4 {
            let source = SourceId::new("test", &format!("c{column}"));
            ui.reactive
                .execute(Command::Collection {
                    source: source.clone(),
                    key: "id".into(),
                    items: (0..25)
                        .map(|row| {
                            json!({"id": format!("r{row}"), "text": format!("Cell {column}:{row}")})
                                .as_object()
                                .unwrap()
                                .clone()
                        })
                        .collect(),
                })
                .unwrap();
            inputs.insert(format!("c{column}"), source);
            doc.push_str(&format!(
                "<column id=\"Column{column}\" width=\"128\" height=\"900\">{{#each $c{column} as cell (cell.id)}}<text id=\"Cell{column}\">{{cell.text}}</text>{{/each}}</column>"
            ));
        }
        doc.push_str("</row>");
        ui.define_document(UiDocument::parse("grid.svelte", &doc).unwrap());
        ui.mount_document(
            SourceId::new("test", "grid"),
            "grid",
            inputs,
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), Viewport::new(1280.0, 960.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end().stats
        };
        // The first frames lay everything out and settle sizes.
        for _ in 0..3 {
            tick(&mut ui);
        }
        let idle = tick(&mut ui);
        assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
        ui.reactive
            .execute(Command::Patch {
                source: SourceId::new("test", "c2"),
                key: "r19".into(),
                fields: json!({"text": "Value 7"}).as_object().unwrap().clone(),
            })
            .unwrap();
        let changed = tick(&mut ui);
        assert_eq!(
            (changed.bindings, changed.reconciled),
            (1, 1),
            "{changed:?}"
        );
        let text = |ui: &Ui| {
            ui.probe_elements()
                .into_iter()
                .find(|e| e.path == "test:grid/Column2/Cell2#r19")
                .unwrap()
                .text
        };
        assert_eq!(text(&ui), "Value 7");
        // Its size moved: written once more for widgets that read their
        // last layout, then idle again.
        tick(&mut ui);
        let idle = tick(&mut ui);
        assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
        assert_eq!(text(&ui), "Value 7");
    }

    #[test]
    fn virtual_mounts_defer_offscreen_styles_and_keep_far_row_identity() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("virtual.svelte", r#"
            <meta name="component" content="virtual">
            <list id="Rows" row-height="40" height="400">
                {#each $items as item (item.id)}<button id="Pick" height="40">{item.label}</button>{/each}
            </list>
        "#).unwrap());
        let source = SourceId::new("test", "items");
        ui.reactive
            .execute(Command::Collection {
                source: source.clone(),
                key: "id".into(),
                items: (0..1000)
                    .map(|i| {
                        json!({"id":format!("r{i}"),"label":format!("Row {i}")})
                            .as_object()
                            .unwrap()
                            .clone()
                    })
                    .collect(),
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "virtual"),
            "virtual",
            BTreeMap::from([("items".into(), source.clone())]),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui, wheel: f32, focus: Option<&str>| {
            ui.begin(
                InputFrame {
                    pointer_inside: true,
                    pointer: og_core::glam::Vec2::new(200.0, 100.0),
                    wheel: og_core::glam::Vec2::new(0.0, wheel),
                    ..Default::default()
                },
                Viewport::new(2560.0, 1440.0),
            );
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            if let Some(path) = focus {
                ui.set_focus(path);
            }
            ui.end();
        };
        tick(&mut ui, 0.0, None);
        let list = ui.mounted_topology().child_at(None, 0).unwrap().clone();
        let far = ui
            .mounted_topology()
            .child_at(Some(&list), 999)
            .unwrap()
            .clone();
        assert_eq!(
            ui.mounted_topology().node(&far).unwrap().direct_text,
            "Row 999"
        );
        assert_eq!(
            ui.mounted_topology().node(&far).unwrap().materialization,
            Materialization::Deferred
        );
        assert!(ui.docs.retained.resolved < 20);
        tick(&mut ui, 0.0, None);
        assert_eq!(ui.docs.retained.resolved, 0);
        assert_eq!(ui.docs.retained.evaluated, 0);
        ui.docs
            .retained
            .topology
            .invalidate_styles(Some(&list), 0..1000)
            .unwrap();
        tick(&mut ui, 0.0, None);
        assert!(ui.docs.retained.resolved < 20);
        assert_eq!(ui.docs.retained.topology.style_is_stale(&far), Some(true));
        tick(&mut ui, 100000.0, None);
        assert!(ui.probe_elements().iter().any(|n| n.text == "Row 999"));
        let path = "test:virtual/Row#r999/Pick#r999";
        tick(&mut ui, 0.0, Some(path));
        assert_eq!(ui.focused_key(), far.key);
        assert_eq!(ui.focus_path(0), path);
        assert_eq!(ui.docs.retained.topology.style_is_stale(&far), Some(false));
        tick(&mut ui, -100000.0, None);
        assert!(ui.probe_elements().iter().any(|n| n.text == "Row 0"));
        assert_eq!(ui.mounted_topology().child_at(Some(&list), 999), Some(&far));
        assert_eq!(
            ui.mounted_topology().node(&far).unwrap().materialization,
            Materialization::Deferred
        );
        let snapshot = ui.save_state();
        ui.restore_state(&snapshot).unwrap();
        tick(&mut ui, 0.0, None);
        assert_eq!(ui.focused_key(), far.key);
        assert_eq!(ui.focus_path(0), path);
        tick(&mut ui, 100000.0, None);
        assert_eq!(ui.mounted_topology().child_at(Some(&list), 999), Some(&far));
        assert_eq!(ui.focused_key(), far.key);
        assert_eq!(ui.focus_path(0), path);
        ui.reactive
            .execute(Command::Move {
                source: source.clone(),
                key: "r999".into(),
                before: Some("r0".into()),
            })
            .unwrap();
        tick(&mut ui, 0.0, None);
        assert_eq!(ui.mounted_topology().child_at(Some(&list), 0), Some(&far));
        assert_eq!(ui.focused_key(), far.key);
        tick(&mut ui, -100000.0, None);
        assert!(ui.probe_elements().iter().any(|n| n.text == "Row 999"));
        assert_eq!(ui.focus_path(0), path);
        ui.reactive
            .execute(Command::Remove {
                source,
                key: "r999".into(),
            })
            .unwrap();
        tick(&mut ui, 0.0, None);
        assert!(ui.mounted_topology().node(&far).is_none());
        assert_ne!(ui.focused_key(), far.key);
    }

    #[test]
    fn mounted_each_edits_keep_multiple_siblings_and_bound_row_identity() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "each.svelte",
                r#"
            <meta name="component" content="each">
            <column id="Root">
                <text id="Before">before</text>
                {#each $items as item (item.id)}
                    <text id="Name">{item.name}</text><text id="Count">{item.count}</text>
                    {#if item.count > 20}<text id="Extra">extra</text>{/if}
                {:else}<text id="Empty">empty</text>{/each}
                <text id="After">after</text>
            </column>
        "#,
            )
            .unwrap(),
        );
        let source = SourceId::new("test", "items");
        ui.reactive
            .execute(Command::Collection {
                source: source.clone(),
                key: "id".into(),
                items: vec![
                    json!({"id":"a","name":"alpha","count":7})
                        .as_object()
                        .unwrap()
                        .clone(),
                    json!({"id":"b","name":"beta","count":19})
                        .as_object()
                        .unwrap()
                        .clone(),
                ],
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "screen"),
            "each",
            BTreeMap::from([("items".into(), source.clone())]),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), Viewport::new(400.0, 500.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            let t = ui.mounted_topology();
            let root = t.child_at(None, 0).unwrap();
            t.children(Some(root), 0..usize::MAX)
                .map(|id| t.node(id).unwrap().direct_text.to_owned())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            tick(&mut ui),
            ["before", "alpha", "7", "beta", "19", "after"]
        );
        let root = ui.mounted_topology().child_at(None, 0).unwrap().clone();
        let alpha = ui
            .mounted_topology()
            .child_at(Some(&root), 1)
            .unwrap()
            .clone();
        ui.reactive
            .execute(Command::Move {
                source: source.clone(),
                key: "a".into(),
                before: None,
            })
            .unwrap();
        assert_eq!(
            tick(&mut ui),
            ["before", "beta", "19", "alpha", "7", "after"]
        );
        assert_eq!(ui.docs.retained.evaluated, 0);
        assert_eq!(ui.docs.retained.resolved, 0);
        assert_eq!(ui.mounted_topology().child_at(Some(&root), 3), Some(&alpha));
        ui.reactive
            .execute(Command::Patch {
                source: source.clone(),
                key: "a".into(),
                fields: json!({"count":23}).as_object().unwrap().clone(),
            })
            .unwrap();
        assert_eq!(
            tick(&mut ui),
            ["before", "beta", "19", "alpha", "23", "extra", "after"]
        );
        assert_eq!(ui.docs.retained.evaluated, 2);
        assert_eq!(ui.docs.retained.resolved, 2);
        for (count, expected) in [
            (7, vec!["before", "beta", "19", "alpha", "7", "after"]),
            (
                23,
                vec!["before", "beta", "19", "alpha", "23", "extra", "after"],
            ),
        ] {
            ui.reactive
                .execute(Command::Patch {
                    source: source.clone(),
                    key: "a".into(),
                    fields: json!({"count":count}).as_object().unwrap().clone(),
                })
                .unwrap();
            assert_eq!(tick(&mut ui), expected);
            assert_eq!(ui.mounted_topology().child_at(Some(&root), 3), Some(&alpha));
        }
        ui.reactive
            .execute(Command::Insert {
                source: source.clone(),
                row: json!({"id":"c","name":"gamma","count":31})
                    .as_object()
                    .unwrap()
                    .clone(),
                before: Some("b".into()),
            })
            .unwrap();
        assert_eq!(
            tick(&mut ui),
            [
                "before", "gamma", "31", "extra", "beta", "19", "alpha", "23", "extra", "after"
            ]
        );
        for key in ["b", "a", "c"] {
            ui.reactive
                .execute(Command::Remove {
                    source: source.clone(),
                    key: key.into(),
                })
                .unwrap();
        }
        assert_eq!(tick(&mut ui), ["before", "empty", "after"]);
        assert!(ui.mounted_topology().node(&alpha).is_none());
        ui.unmount_document(&SourceId::new("test", "screen"));
        ui.reactive
            .execute(Command::Insert {
                source,
                row: json!({"id":"d","name":"unmounted","count":44})
                    .as_object()
                    .unwrap()
                    .clone(),
                before: None,
            })
            .unwrap();
        ui.begin(InputFrame::default(), Viewport::new(400.0, 500.0));
        ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
        ui.end();
        assert_eq!(ui.mounted_topology().child_count(None), 0);
        assert!(
            ui.probe_elements()
                .iter()
                .all(|node| node.text != "unmounted")
        );
    }

    #[test]
    fn authored_control_state_restyles_subscribers_in_the_same_frame() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "states.svelte",
                r#"
            <meta name="component" content="states">
            <column><toggle id="Choice" checked={$checked}>Choose</toggle></column>
            <style>#Choice { width: 70px; } #Choice:checked { width: 140px; }</style>
        "#,
            )
            .unwrap(),
        );
        let source = SourceId::new("test", "checked");
        ui.reactive
            .execute(Command::Signal {
                source: source.clone(),
                initial: json!(true),
                readable: false,
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "state"),
            "states",
            BTreeMap::from([("checked".into(), source.clone())]),
            &crate::bind::NoEnv,
        )
        .unwrap();
        for (checked, width) in [(true, 140.0), (false, 70.0), (true, 140.0)] {
            ui.reactive
                .execute(Command::Set {
                    source: source.clone(),
                    value: json!(checked),
                    force: false,
                })
                .unwrap();
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            let elements = ui.probe_elements();
            assert_eq!(
                elements
                    .iter()
                    .find(|n| n.path == "test:state/Choice")
                    .unwrap()
                    .width,
                width
            );
            let t = ui.mounted_topology();
            let root = t.child_at(None, 0).unwrap();
            let choice = t.node(t.child_at(Some(root), 0).unwrap()).unwrap();
            assert_eq!(choice.attributes["checked"], json!(checked));
            assert_eq!(
                choice
                    .states
                    .contains(&crate::bind::InteractionState::Checked),
                checked
            );
        }
    }

    #[test]
    fn inherited_cascade_replaces_removed_overrides_without_rereading_text() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "cascade.svelte",
                r#"
            <meta name="component" content="cascade">
            <column id="Root" class={$wide ? 'wide' : ''}>
                <text id="Label">{$label}</text><text id="Shadow">fixed</text>
            </column>
            <style>
                #Root { --measure: 30px; }
                #Root.wide { --measure: 120px; }
                #Label { width: var(--measure); }
                #Shadow { --measure: 17px; width: var(--measure); }
            </style>
        "#,
            )
            .unwrap(),
        );
        let wide = SourceId::new("test", "wide");
        let label = SourceId::new("test", "label");
        for (source, value) in [(wide.clone(), json!(false)), (label.clone(), json!("kept"))] {
            ui.reactive
                .execute(Command::Signal {
                    source,
                    initial: value,
                    readable: false,
                })
                .unwrap();
        }
        ui.mount_document(
            SourceId::new("test", "screen"),
            "cascade",
            BTreeMap::from([("wide".into(), wide.clone()), ("label".into(), label)]),
            &crate::bind::NoEnv,
        )
        .unwrap();
        for (enabled, width) in [(false, 30.0), (true, 120.0), (false, 30.0)] {
            ui.reactive
                .execute(Command::Set {
                    source: wide.clone(),
                    value: json!(enabled),
                    force: false,
                })
                .unwrap();
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            let elements = ui.probe_elements();
            let label = elements
                .iter()
                .find(|e| e.path == "test:screen/Label")
                .unwrap();
            assert_eq!(label.width, width);
            assert_eq!(label.text, "kept");
            assert_eq!(
                elements
                    .iter()
                    .find(|e| e.path == "test:screen/Shadow")
                    .unwrap()
                    .width,
                17.0
            );
            let topology = ui.mounted_topology();
            let root = topology.node(topology.child_at(None, 0).unwrap()).unwrap();
            assert_eq!(
                root.attributes["class"],
                json!(if enabled { "wide" } else { "" })
            );
            if ui.tick() > 1 {
                assert_eq!(ui.docs.retained.evaluated, 2); // Class value and matching predicate.
                assert_eq!(ui.docs.retained.resolved, 2);
            }
        }
    }

    #[test]
    fn mounted_store_updates_resolve_one_element_and_emit_current_text() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("retained.svelte", r#"
            <meta name="component" content="retained">
            <column id="Root"><text id="Name">{$label}</text><text id="Other">unchanged</text></column>
        "#).unwrap());
        let source = SourceId::new("test", "label");
        ui.reactive
            .execute(Command::Signal {
                source: source.clone(),
                initial: json!("first"),
                readable: false,
            })
            .unwrap();
        let mount = SourceId::new("test", "screen");
        ui.mount_document(
            mount.clone(),
            "retained",
            BTreeMap::from([("label".into(), source.clone())]),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.end();
        };
        tick(&mut ui);
        assert_eq!(
            ui.probe_elements()
                .iter()
                .find(|n| n.path == "test:screen/Name")
                .unwrap()
                .text,
            "first"
        );
        let root = ui.mounted_topology().child_at(None, 0).unwrap().clone();
        let label = ui
            .mounted_topology()
            .child_at(Some(&root), 0)
            .unwrap()
            .clone();
        assert_eq!(
            ui.mounted_topology().node(&label).unwrap().direct_text,
            "first"
        );
        ui.docs.retained.topology.take_changes();
        tick(&mut ui);
        assert_eq!(ui.docs.retained.evaluated, 0);
        assert_eq!(ui.docs.retained.resolved, 0);
        let snapshot = ui.save_state();
        ui.reactive
            .execute(Command::Set {
                source: source.clone(),
                value: json!("changed"),
                force: false,
            })
            .unwrap();
        tick(&mut ui);
        assert_eq!(ui.docs.retained.evaluated, 1);
        assert_eq!(ui.docs.retained.resolved, 1);
        assert_eq!(
            ui.probe_elements()
                .iter()
                .find(|n| n.path == "test:screen/Name")
                .unwrap()
                .text,
            "changed"
        );
        assert_eq!(ui.mounted_topology().child_at(Some(&root), 0), Some(&label));
        assert!(ui.docs.retained.topology.take_changes().is_empty());
        ui.restore_state(&snapshot).unwrap();
        assert_eq!(ui.save_state(), snapshot);
        tick(&mut ui);
        assert_eq!(ui.mounted_topology().child_at(Some(&root), 0), Some(&label));
        assert_eq!(
            ui.mounted_topology().node(&label).unwrap().direct_text,
            "first"
        );
        assert!(ui.docs.retained.topology.take_changes().is_empty());
        ui.reactive
            .execute(Command::Set {
                source: source.clone(),
                value: json!("after restore"),
                force: false,
            })
            .unwrap();
        tick(&mut ui);
        assert_eq!(
            ui.mounted_topology().node(&label).unwrap().direct_text,
            "after restore"
        );
        assert_eq!(ui.docs.retained.evaluated, 1);
        assert_eq!(ui.docs.retained.resolved, 1);
        ui.define_document(UiDocument::parse("retained.svelte", r#"
            <meta name="component" content="retained">
            <column id="Root"><text id="Name">new {$label}</text><text id="Replacement">added</text></column>
        "#).unwrap());
        tick(&mut ui);
        assert_eq!(ui.mounted_topology().child_at(Some(&root), 0), Some(&label));
        assert_eq!(
            ui.mounted_topology().node(&label).unwrap().direct_text,
            "new after restore"
        );
        assert!(ui.probe_elements().iter().any(|n| n.id == "Replacement"));
        assert!(!ui.probe_elements().iter().any(|n| n.id == "Other"));
        ui.reactive
            .execute(Command::Set {
                source,
                value: json!("reloaded"),
                force: false,
            })
            .unwrap();
        tick(&mut ui);
        assert_eq!(
            ui.mounted_topology().node(&label).unwrap().direct_text,
            "new reloaded"
        );
    }

    #[test]
    fn a_retained_bound_palette_token_follows_theme_without_rereading_its_store() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "paint.svelte",
                r#"
            <meta name="component" content="paint">
            <column><column id="Patch" width="40" height="30" background={$paint} /></column>
        "#,
            )
            .unwrap(),
        );
        let source = SourceId::new("test", "paint");
        ui.reactive
            .execute(Command::Signal {
                source: source.clone(),
                initial: json!("panel"),
                readable: false,
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "paint"),
            "paint",
            BTreeMap::from([("paint".into(), source)]),
            &crate::bind::NoEnv,
        )
        .unwrap();
        for color in [Srgba::rgb(210, 30, 40), Srgba::rgb(30, 200, 70)] {
            let mut theme = *ui.theme();
            theme.palette.panel = color;
            ui.set_theme(theme);
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.end();
            assert!(ui.frame().items.iter().any(|item| matches!(&item.cmd,
                crate::DrawCmd::Quad { color: actual, .. } if *actual == color.linear()
            )));
            assert_eq!(ui.docs.retained.evaluated, 0);
        }
    }

    #[test]
    fn a_failed_refresh_keeps_other_pending_updates_and_can_rebuild_a_failed_branch() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "failure.svelte",
                r#"
            <meta name="component" content="failure">
            <column>
                <text id="Risk">{$bad ? $missing : "safe"}</text>
                {#if $show}<text id="Later">{$later}</text>{/if}
                <text id="Other">{$label}</text>
            </column>
        "#,
            )
            .unwrap(),
        );
        let source = |name| SourceId::new("test", name);
        for (name, initial) in [
            ("bad", json!(false)),
            ("show", json!(false)),
            ("label", json!("old")),
        ] {
            ui.reactive
                .execute(Command::Signal {
                    source: source(name),
                    initial,
                    readable: false,
                })
                .unwrap();
        }
        ui.mount_document(
            source("failure"),
            "failure",
            ["bad", "show", "label", "later"]
                .into_iter()
                .map(|name| (name.to_owned(), source(name)))
                .collect(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            let result = ui.build_retained_documents(&crate::bind::NoEnv);
            ui.end();
            result
        };
        tick(&mut ui).unwrap();
        for (name, value) in [("bad", json!(true)), ("label", json!("new"))] {
            ui.reactive
                .execute(Command::Set {
                    source: source(name),
                    value,
                    force: false,
                })
                .unwrap();
        }
        assert!(tick(&mut ui).unwrap_err().contains("missing"));
        assert!(tick(&mut ui).unwrap_err().contains("missing"));
        ui.reactive
            .execute(Command::Set {
                source: source("bad"),
                value: json!(false),
                force: false,
            })
            .unwrap();
        tick(&mut ui).unwrap();
        assert!(
            ui.probe_elements()
                .iter()
                .any(|node| node.id == "Other" && node.text == "new")
        );
        ui.reactive
            .execute(Command::Set {
                source: source("show"),
                value: json!(true),
                force: false,
            })
            .unwrap();
        assert!(tick(&mut ui).unwrap_err().contains("later"));
        ui.reactive
            .execute(Command::Signal {
                source: source("later"),
                initial: json!("recovered"),
                readable: false,
            })
            .unwrap();
        tick(&mut ui).unwrap();
        assert!(
            ui.probe_elements()
                .iter()
                .any(|node| node.id == "Later" && node.text == "recovered")
        );
        assert!(
            ui.probe_elements()
                .iter()
                .any(|node| node.id == "Other" && node.text == "new")
        );
    }
}
