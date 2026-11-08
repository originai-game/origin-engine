use super::*;
use crate::bind::Expr;
use crate::doc::topology::{Instance, Materialization, Topology};
use crate::reactive::{
    SourceId,
    bindings::{BindingContext, BindingExpression, BindingId, Bindings},
    graph::MountedNode,
};

mod content;
mod pointer;
mod queries;
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
    /// Records an immediate build's evaluated input (container-query replay).
    pub(super) fn remember_expression(&mut self, expression: &Expr, value: &Value) {
        if !self.expressions.iter().any(|(e, _)| e == expression) {
            self.expressions.push((expression.clone(), value.clone()));
        }
    }

    pub(super) fn remember_property(&mut self, property: &PropValue, ty: Ty, value: &Option<V>) {
        if !self
            .properties
            .iter()
            .any(|(p, t, _)| p == property && *t == ty)
        {
            self.properties.push((property.clone(), ty, value.clone()));
        }
    }

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
    /// While documents use generated content: the counter run as it
    /// entered this element, what its subtree did to the run, and whether
    /// any generated text in it read the run.
    run_in: Option<Arc<crate::doc::generated::Run>>,
    run_events: Arc<Vec<crate::doc::generated::Event>>,
    run_reads: bool,
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct Registration {
    document: String,
    inputs: BTreeMap<String, SourceId>,
    /// Content passed by reference: a document, or `document#snippet`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    content: BTreeMap<String, String>,
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
    /// `use:` lifecycle events (made, params changed, gone) for the next
    /// build's answer.
    uses: Vec<DocEvent>,
    collections: BTreeMap<SourceId, BTreeSet<MountedNode>>,
    dirty: BTreeSet<MountedNode>,
    inherited: BTreeMap<(MountedNode, String), BTreeSet<MountedNode>>,
    by_key: BTreeMap<u64, MountedNode>,
    interaction: [u64; 4],
    /// Popover elements by their popover key, for `:popover-open`.
    popover_nodes: BTreeMap<u64, MountedNode>,
    /// The `:target` element last applied, and the fragment it was for.
    target: (Option<String>, Vec<MountedNode>),
    viewport: Option<crate::doc::media::Context>,
    /// The safe area and the small and large viewports last build: a
    /// change re-resolves what read the viewport (`env()`, `svh`, `lvh`).
    surface: [f32; 8],
    theme: Option<crate::Theme>,
    /// The surface's direction last build: a document's root takes it.
    direction: Option<og_i18n::Direction>,
    theme_epoch: u64,
    media: [BTreeSet<MountedNode>; crate::doc::media::AXES],
    materializing: Option<Materialization>,
    built_tick: Option<u64>,
    /// Elements whose render nodes must be written again this frame.
    emit: BTreeSet<MountedNode>,
    /// Elements built every frame ([`ElementInstance::volatile`]).
    volatile: BTreeSet<MountedNode>,
    /// Elements resolved again every frame: their SMIL animations play.
    animated: BTreeSet<MountedNode>,
    /// The elements not built because they were hidden: shown again, the
    /// nearest ancestor that holds its node builds them.
    unbuilt_hidden: BTreeSet<MountedNode>,
    /// The elements whose frame is a Rive artboard: their players go on
    /// every tick ([`RetainedHost::advance_rive`]), without building them.
    rive: BTreeSet<MountedNode>,
    /// The elements with a trigger on each action: built in a frame the
    /// action fires, not every frame.
    on_action: BTreeMap<String, BTreeSet<MountedNode>>,
    /// The actions fired this frame whose listeners are marked, and the
    /// elements built this frame (kept while any element listens): an
    /// action a trigger fires while building reaches the listeners not yet
    /// built ([`RetainedHost::emit_fired`]).
    fired: usize,
    built_now: BTreeSet<MountedNode>,
    /// This frame's layer context of each mount emitted (whether its layer
    /// takes UI actions, the layer's serial): an element written in place
    /// after the layers' frames closed is written in its own.
    contexts: BTreeMap<SourceId, (bool, u32)>,
    /// The element that owns each render key (its nodes', its links').
    owner_of: std::collections::HashMap<u64, MountedNode>,
    /// The widget memories mounted elements keep while they are not built.
    memories: std::collections::HashMap<u64, u32>,
    /// This frame's and last frame's interaction keys (hover, press,
    /// focus, activation).
    input: Vec<u64>,
    last_input: Vec<u64>,
    /// The frame-wide inputs changed this frame: every node is written.
    restyled: bool,
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
    /// The input pass's memory ([`pointer`]).
    pointer: pointer::PointerMemory,
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
    /// Render key `key` belongs to a mounted element.
    pub(crate) fn owns(&self, key: u64) -> bool {
        self.owner_of.contains_key(&key)
    }

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
                lexical: Arc::default(),
            };
            let compiled =
                content::mount_content(ui, &registration.content, &context).map(|lexical| {
                    BindingContext {
                        lexical: Arc::new(lexical),
                        ..context
                    }
                });
            let compiled = compiled.and_then(|context| {
                self.compile(
                    ui,
                    (document.clone(), document.root),
                    context,
                    None,
                    root_path(&mount, &document),
                    env,
                )
            });
            match compiled {
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
            self.use_events(&node, "destroy", None);
        }
        self.elements.retain(|node, _| &node.mount != mount);
        self.emit.retain(|node| &node.mount != mount);
        self.volatile.retain(|node| &node.mount != mount);
        self.animated.retain(|node| &node.mount != mount);
        self.on_action.retain(|_, listeners| {
            listeners.retain(|node| &node.mount != mount);
            !listeners.is_empty()
        });
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
        if matches!(node.kind, NodeKind::If(_) | NodeKind::Key(_)) {
            return self.compile_branch(ui, (doc, index), context, parent, id, env);
        }
        match &node.kind {
            NodeKind::Use(component) => {
                let component = component.clone();
                return self.compile_component(
                    ui,
                    (doc, index),
                    &component,
                    context,
                    parent,
                    id,
                    env,
                );
            }
            NodeKind::Render { name, optional } => {
                let (name, optional) = (name.clone(), *optional);
                return self.compile_render(
                    ui,
                    (doc, index),
                    &name,
                    optional,
                    context,
                    parent,
                    id,
                    env,
                );
            }
            // A definition renders nothing where it is written.
            NodeKind::Snippet(_) => {
                return self.compile_block(
                    ui,
                    (doc, index),
                    context,
                    parent,
                    id,
                    &Expr::Lit(false.into()),
                    env,
                );
            }
            _ => {}
        }
        let NodeKind::Element(_) = node.kind else {
            return Err(format!(
                "{}:{}: structural mount compilation is not connected yet",
                doc.source, node.line
            ));
        };
        let path = parent.map_or_else(|| id.clone(), |p| format!("{}/{id}", self.content_path(p)));
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
            // A slot, and a player's picture and time, change without a binding.
            volatile: matches!(node.kind, NodeKind::Element("slot" | "video" | "audio")),
            materialization,
            run_in: None,
            run_events: Default::default(),
            run_reads: false,
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
        // Its box pseudo-elements' rules match against it: their
        // conditions are its inputs.
        let expressions = node
            .rules
            .iter()
            .chain(
                node.pseudo
                    .iter()
                    .flat_map(|&(_, at)| doc.nodes[at].rules.iter()),
            )
            .filter_map(|rule| rule.condition.as_ref())
            .chain(
                node.span_rules
                    .iter()
                    .filter_map(|rule| rule.condition.as_ref()),
            )
            .chain(node.groups.iter().filter_map(|group| group.bind.as_ref()))
            .chain(
                node.inline
                    .iter()
                    .map(|run| &run.text)
                    .filter(|text| !matches!(text, Expr::Lit(_))),
            )
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
            }))
            .chain(node.uses.iter().filter_map(|(_, params)| params.as_ref()))
            // `style:prop={…}`'s values (`style:--name`'s are selector facts).
            .chain(
                node.custom
                    .iter()
                    .map(|(_, e)| e)
                    .filter(|e| !node.selector_attrs.iter().any(|(_, s)| s == *e)),
            );
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
        if !self.restoring {
            self.use_events(&identity, "create", None);
        }
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
        for trigger in &doc.nodes[index].triggers {
            if let When::Action(action) = &trigger.when {
                self.on_action
                    .entry(action.clone())
                    .or_default()
                    .insert(identity.clone());
            }
        }
        // An exclusive `<details name>` closes as another of its name opens.
        if let Some(PropValue::Lit(Lit::Str(name))) = doc.nodes[index]
            .props
            .iter()
            .rev()
            .find(|p| p.name() == "exclusive")
            .map(|p| &p.value)
        {
            self.on_action
                .entry(super::nodes::details_action(name))
                .or_default()
                .insert(identity.clone());
        }
        // `<tabs>` steps on the bumpers (`tabPrev`, `tabNext`).
        if matches!(doc.nodes[index].kind, NodeKind::Element("tabs")) {
            for action in ["tabPrev", "tabNext"] {
                self.on_action
                    .entry(action.into())
                    .or_default()
                    .insert(identity.clone());
            }
        }
        if let Some(parent) = parent {
            self.emit.insert(parent.clone());
        }
        Ok(Child::Element(identity))
    }

    /// Queues `node`'s `use:` events of `phase` (`create`, `update`,
    /// `destroy`): an update only for the action whose params are
    /// `changed`'s expression, with its new value.
    fn use_events(&mut self, node: &MountedNode, phase: &str, changed: Option<(&Expr, Value)>) {
        let Some(element) = self.elements.get(node) else {
            return;
        };
        for (name, params) in &element.document.nodes[element.index].uses {
            let value = match (&changed, params) {
                (Some((e, v)), Some(p)) if *e == p => v.clone(),
                (Some(_), _) => continue,
                (None, _) if phase == "destroy" => Value::Null,
                (None, Some(Expr::Lit(v))) => v.clone(),
                (None, Some(p)) => element.cached.expression(p),
                (None, None) => Value::Null,
            };
            self.uses.push(DocEvent {
                mount: element.context.node.mount.id.clone(),
                path: element.path.clone(),
                row_key: element.context.row_key.clone(),
                kind: "use".into(),
                name: name.clone(),
                value: json!({ "phase": phase, "params": value }),
            });
        }
    }

    fn sync_facts(
        &mut self,
        ui: &mut Ui,
        element: &ElementInstance,
        env: &dyn Env,
    ) -> Result<(), String> {
        use crate::bind::InteractionState::*;
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
        let num = |name| facts.get(name).and_then(Value::as_f64);
        // A field's text its `type` or its constraints refuse is invalid.
        let field_invalid = facts.get("input").and_then(Value::as_str).is_some_and(|v| {
            let ty = facts
                .get("type")
                .and_then(Value::as_str)
                .and_then(crate::InputType::from_name)
                .unwrap_or_default();
            ty.mismatch(v)
                || crate::input_type::Constraints {
                    required: flag("required"),
                    min_chars: num("minChars").unwrap_or(0.0) as u32,
                    pattern: facts
                        .get("pattern")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    min: num("min"),
                    max: num("max"),
                    step: num("stepBy"),
                }
                .fails(ty, v)
        });
        self.topology.set_text(
            identity,
            facts
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        )?;
        let kind = match node.kind {
            NodeKind::Element(kind) => kind,
            _ => "",
        };
        // HTML's form-associated elements: they are enabled or disabled,
        // valid or invalid; text fields are read-write unless readonly.
        let form = matches!(
            kind,
            "button" | "field" | "slider" | "toggle" | "radio" | "select" | "color"
        );
        let disabled = flag("disabled");
        let invalid = flag("invalid") || field_invalid;
        let range = match (
            num("value").or_else(|| {
                facts
                    .get("input")
                    .and_then(Value::as_str)
                    .and_then(|v| v.trim().parse().ok())
            }),
            num("min"),
            num("max"),
        ) {
            (Some(v), min, max)
                if kind == "slider" || (kind == "field" && (min.is_some() || max.is_some())) =>
            {
                Some(min.is_none_or(|m| v >= m) && max.is_none_or(|m| v <= m))
            }
            _ => None,
        };
        let editable = kind == "field" && !flag("readonly") && !disabled;
        let open = flag("open");
        if facts.contains_key("popover") {
            let key = super::popover::popover_key(&build.mount, &node.id);
            self.popover_nodes.insert(key, identity.clone());
        }
        let authored = |name: &str| {
            node.selector_attrs
                .iter()
                .any(|(a, e)| a == name && !matches!(e, crate::bind::Expr::Lit(Value::Bool(false))))
        };
        for (state, enabled) in [
            (Disabled, disabled),
            (Enabled, form && !disabled),
            (Checked, flag("checked")),
            (Invalid, invalid),
            (Valid, form && !invalid),
            // After the player changed it: a field with text it refuses.
            (
                UserInvalid,
                invalid
                    && facts
                        .get("input")
                        .and_then(Value::as_str)
                        .is_some_and(|v| !v.is_empty()),
            ),
            (Required, flag("required")),
            (Optional, form && !flag("required")),
            (ReadWrite, editable),
            (ReadOnly, !editable),
            (InRange, range == Some(true)),
            (OutOfRange, range == Some(false)),
            (Indeterminate, flag("indeterminate")),
            (Default, authored("checked") || authored("selected")),
            (Open, open),
            (Modal, open && flag("modal")),
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
            // The states HTML always had here; the rest only when a rule of
            // the element's document reads them.
            if !matches!(state, Disabled | Checked | Invalid | PlaceholderShown)
                && !self
                    .profile(ui, &element.document.name)
                    .reads
                    .contains(&state)
            {
                continue;
            }
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
            || self
                .direction
                .replace(ui.direction)
                .is_some_and(|old| old != ui.direction)
        {
            self.theme_epoch += 1;
        }
        let scrolled = std::mem::take(&mut ui.docs.scroll_changed);
        self.rescroll(&scrolled);
        let viewport = ui.media_context();
        let [small, large] = ui.viewport.extremes();
        let sa = ui.viewport.safe_area;
        let surface = [
            sa.top, sa.right, sa.bottom, sa.left, small[0], small[1], large[0], large[1],
        ];
        let moved = std::mem::replace(&mut self.surface, surface) != surface;
        if let Some(old) = self.viewport.replace(viewport) {
            let mut changes = viewport.changes(&old);
            // The safe area and the small and large viewports read as the
            // viewport's axes do.
            changes[0] |= moved;
            changes[1] |= moved;
            for (axis, changed) in changes.into_iter().enumerate() {
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
                    let value = self.bindings.value(id).map_err(Clone::clone)?.clone();
                    if let Err(error) = self.refresh_branch(ui, &node, &value, env) {
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
            let mut changed_uses = Vec::new();
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
                            .uses
                            .iter()
                            .any(|(_, params)| params.as_ref() == Some(e))
                        {
                            changed_uses.push((node.clone(), e.clone(), value.clone()));
                        }
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
            for (node, params, value) in changed_uses {
                self.use_events(&node, "update", Some((&params, value)));
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
        use crate::bind::InteractionState::{
            Active, Focus, FocusVisible, FocusWithin, Hover, PopoverOpen, Target,
        };
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
        let shown: Vec<(MountedNode, bool)> = self
            .popover_nodes
            .iter()
            .filter(|(_, node)| self.topology.node(node).is_some())
            .map(|(key, node)| {
                (
                    node.clone(),
                    ui.docs.popovers.get(key).is_some_and(|l| l.shown),
                )
            })
            .collect();
        self.popover_nodes
            .retain(|_, node| self.topology.node(node).is_some());
        for (node, open) in shown {
            if self
                .topology
                .node(&node)
                .unwrap()
                .states
                .contains(&PopoverOpen)
                != open
            {
                self.topology.set_state(&node, PopoverOpen, open)?;
                ui.reactive.graph.interaction_changed(&node, PopoverOpen);
            }
        }
        if self.target.0 != ui.docs.target {
            let old = std::mem::take(&mut self.target.1);
            let id = ui.docs.target.clone();
            let new: Vec<MountedNode> = match &id {
                Some(id) => self
                    .elements
                    .iter()
                    .filter(|(_, e)| e.id == *id || e.document.nodes[e.index].id == *id)
                    .map(|(node, _)| node.clone())
                    .collect(),
                None => Vec::new(),
            };
            for (nodes, enabled) in [(&old, false), (&new, true)] {
                for node in nodes {
                    if self.topology.node(node).is_some() {
                        self.topology.set_state(node, Target, enabled)?;
                        ui.reactive.graph.interaction_changed(node, Target);
                    }
                }
            }
            self.target = (id, new);
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
        let mut values = ui.resolve(
            &build,
            &e.document.nodes[e.index],
            &e.path,
            &mut probe,
            &build.inherited,
            &mut |selector| {
                crate::doc::matching::matches(topology, &e.document, selector, identity, &mut work)
            },
        );
        values.ordinal = ui.ordinal_of(
            &e.document.nodes[e.index],
            &values,
            |filter, item_values| {
                crate::doc::lists::ordinal(topology, identity, filter, item_values)
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
        for (axis, read) in reads.axes().into_iter().enumerate() {
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
        // A world layer (`<anchor>`, `<panel world>`) stands outside the
        // tree, under no parent, and stays among the layers as it is.
        let outside = element.parent_slot == NONE;
        // A subtree whose generated text reads the counter run is held
        // only while the run enters it as it did when it was built.
        let uses_run = ui.docs.uses_run;
        let run_moved = uses_run
            && element.run_reads
            && element
                .run_in
                .as_ref()
                .is_none_or(|entered| !entered.same_state(&ui.docs.run));
        if !in_place
            && !element.volatile
            && !run_moved
            && !self.emit.contains(identity)
            && let Some(slot) = element.slot
            && ui.holds(slot, identity.key, if outside { NONE } else { parent_slot })
        {
            if !outside {
                ui.reattach(slot);
            }
            if uses_run {
                ui.docs.run.replay(&element.run_events);
                ui.docs.run.reads += u64::from(element.run_reads);
            }
            return Ok(());
        }
        // Written in place, outside the tree walk: the run as it entered.
        let saved_run = (uses_run && in_place)
            .then(|| element.run_in.as_deref().cloned())
            .flatten()
            .map(|entered| std::mem::replace(&mut ui.docs.run, entered));
        let run_mark = ui.docs.run.mark();
        let run_reads = ui.docs.run.reads;
        let run_in = uses_run.then(|| ui.docs.run.snapshot());
        self.emit.remove(identity);
        self.reconciled += 1;
        if !self.on_action.is_empty() {
            self.built_now.insert(identity.clone());
        }
        let e = self.elements[identity].clone();
        if e.slot.is_some() {
            // Built before and not since retired: not appearing.
            ui.docs.seen.insert(identity.key);
        }
        if in_place {
            ui.in_place = Some((identity.key, parent_slot));
        }
        ui.docs.keeps_hidden = e
            .slot
            .is_some_and(|slot| ui.holds(slot, identity.key, parent_slot));
        ui.owning.push(Default::default());
        ui.docs.capture.push(Default::default());
        let resolved = e.resolved.as_ref().expect("resolved mount");
        let mut b = e.build(env, resolved.values.cascade.computed.clone());
        let mut result = Ok(());
        let mut children_built = false;
        ui.run_enter(&resolved.values);
        ui.build_resolved_node(&mut b, e.index, resolved.clone(), |ui, b, virtual_props| {
            children_built = true;
            result = if let Some(props) = virtual_props {
                self.emit_virtual(ui, &e, b, props, env, events, writes)
            } else {
                e.children
                    .iter()
                    .try_for_each(|child| self.emit(ui, child, env, events, writes))
            };
        });
        ui.run_exit();
        let run_events = if uses_run {
            ui.docs.run.since(run_mark)
        } else {
            Default::default()
        };
        let reads_run = ui.docs.run.reads != run_reads;
        if let Some(saved) = saved_run {
            ui.docs.run = saved;
        }
        ui.in_place = None;
        let owned = ui.owning.pop().expect("pushed");
        let record = ui.docs.capture.pop().expect("pushed");
        events.extend(b.events);
        writes.extend(
            b.writes
                .into_iter()
                .map(|(path, value)| (e.context.clone(), path, value)),
        );
        let slot = owned.slots.iter().copied().find(|&i| {
            ui.nodes[i].key == identity.key
                && (ui.nodes[i].parent == parent_slot || ui.nodes[i].parent == NONE)
        });
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
        if slot.is_none() && resolved.values.has("visible") && !resolved.values.flag("visible") {
            self.unbuilt_hidden.insert(identity.clone());
        } else {
            self.unbuilt_hidden.remove(identity);
        }
        if resolved.values.str("rive").is_some() {
            self.rive.insert(identity.clone());
        } else {
            self.rive.remove(identity);
        }
        // A slot, and a viewport whose camera-controls turn it by the tick.
        let node = &e.document.nodes[e.index];
        let volatile = owned.moved
            || matches!(node.kind, NodeKind::Element("slot"))
            || (node.kind == NodeKind::Element("viewport")
                && node.props.iter().any(|p| p.name() == "cameraControls"));
        if volatile {
            self.volatile.insert(identity.clone());
        } else {
            self.volatile.remove(identity);
        }
        if owned.restyle {
            self.animated.insert(identity.clone());
        } else {
            self.animated.remove(identity);
        }
        let element = self.elements.get_mut(identity).unwrap();
        element.slot = slot;
        // The parent its node stands under: the open element, or none for
        // a world layer.
        element.parent_slot = slot.map_or(parent_slot, |i| ui.nodes[i].parent);
        element.owned = owned.slots;
        element.keys = keys;
        element.record = record;
        element.memories = owned.memories;
        element.volatile = volatile;
        element.materialization = materialization;
        element.run_in = run_in;
        element.run_events = run_events;
        element.run_reads = reads_run;
        if slot.is_none() || !children_built {
            // Not shown (`visible` false), or showing no content (a closed
            // dialog or disclosure): its descendants' nodes go too, and are
            // built afresh when shown.
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
        // An element without the slot it was built in (offscreen in a
        // virtual list, in a branch not shown, retired) is written whole
        // when it is built again: marking it does nothing, so the mark goes.
        // One materialized but without a node (never built while hidden,
        // now shown) is built by its nearest ancestor that holds its node.
        let holds = |host: &Self, identity: &MountedNode| {
            host.elements.get(identity).is_some_and(|e| {
                e.slot
                    .is_some_and(|slot| ui.holds(slot, identity.key, e.parent_slot))
            })
        };
        let visible = |host: &Self, identity: &MountedNode| {
            host.topology.node(identity).map(|n| n.materialization)
                == Some(Materialization::Visible)
        };
        let unheld: Vec<MountedNode> = self
            .emit
            .iter()
            .filter(|identity| !holds(self, identity))
            .cloned()
            .collect();
        for identity in unheld {
            self.emit.remove(&identity);
            let shown = self.elements.get(&identity).is_some_and(|e| {
                e.resolved
                    .as_ref()
                    .is_some_and(|r| !r.values.has("visible") || r.values.flag("visible"))
            });
            if !self.unbuilt_hidden.contains(&identity) || !shown || !visible(self, &identity) {
                continue;
            }
            let mut at = self.topology.parent(&identity).cloned();
            while let Some(parent) = at {
                if holds(self, &parent) {
                    self.emit.insert(parent);
                    break;
                }
                if !visible(self, &parent) {
                    break;
                }
                at = self.topology.parent(&parent).cloned();
            }
        }
        let mut pending: Vec<(usize, MountedNode)> = self
            .emit
            .iter()
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
            let outer = (ui.docs.accepts, ui.docs.building);
            let (accepts, building) = self.contexts.get(&identity.mount).copied().unwrap_or(outer);
            (ui.docs.accepts, ui.docs.building) = (accepts, building);
            // A world layer's own arm opens its layer again.
            if parent != NONE {
                ui.stack.push(parent);
            }
            let result = self.emit_element(ui, &identity, true, env, events, writes);
            if parent != NONE {
                ui.stack.pop();
            }
            (ui.docs.accepts, ui.docs.building) = outer;
            result?;
        }
        Ok(())
    }

    /// The path element `p`'s children are built under, so their keys
    /// are their nodes' own: a modal dialog's panel and a popover's layer
    /// stand in the top layer, named apart from the element (a spotlight's
    /// callout layer takes the element's own key and path).
    fn content_path(&self, p: &MountedNode) -> String {
        let e = &self.elements[p];
        let n = &e.document.nodes[e.index];
        let set = |name: &str| {
            n.props.iter().any(|s| {
                schema::PROPS[s.spec].name == name
                    && !matches!(
                        s.value,
                        crate::doc::PropValue::Lit(crate::doc::Lit::Bool(false))
                    )
            })
        };
        let local = e.path.rsplit('/').next().unwrap_or(&e.path);
        match n.kind {
            NodeKind::Element("dialog") if set("modal") => format!("{local}/Panel"),
            NodeKind::Element(_) if set("popover") => {
                format!("{}~popover", e.path.replace('/', "_"))
            }
            _ => e.path.clone(),
        }
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
        self.animated.remove(identity);
        self.emit.remove(identity);
        self.scrollers.remove(identity);
    }

    /// The document cascade's winning declarations of the element the
    /// render key `key` belongs to (`file:line` each), and its bindings, as
    /// the UI debugger shows them (`Ui::debug_element`): read from what its
    /// last resolve kept, for that one element.
    pub(crate) fn debug_styles(
        &self,
        key: u64,
    ) -> Option<(Vec<crate::UiDebugStyle>, crate::ui::DebugBindings)> {
        let e = self.elements.get(self.owner_of.get(&key)?)?;
        let resolved = e.resolved.as_ref()?;
        let source = &e.document.source;
        let styles = resolved
            .values
            .cascade
            .values
            .iter()
            .map(|(name, value, line)| crate::UiDebugStyle {
                name: name.clone(),
                value: value.trim().to_owned(),
                source: format!("{source}:{line}"),
            })
            .collect();
        let bindings = e
            .record
            .built
            .iter()
            .filter(|b| b.path == e.path)
            .flat_map(|b| &b.bindings)
            .map(|(p, x, v)| {
                let v = v.as_str().map_or_else(|| v.to_string(), str::to_owned);
                (p.clone(), x.clone(), v)
            })
            .collect();
        Some((styles, bindings))
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
        self.restyled = self.signature.as_ref() != Some(&signature);
        if self.restyled {
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
        // On-screen controls follow their further touches (og.Pointers):
        // those a touch holds, and every one when a touch goes down.
        input.extend(s.touch_owners.keys());
        if ui.input.pointers.pressed != 0 {
            input.extend(ui.docs.zones.keys());
        }
        if n.back {
            // Back belongs to the topmost scope (a modal, an open list),
            // else to what holds the focus (a dialog in place).
            input.push(s.scope);
            if s.focus != 0
                && let Some(mut i) = ui.nodes.iter().position(|node| node.key == s.focus)
            {
                while ui.nodes[i].parent != NONE {
                    i = ui.nodes[i].parent;
                    input.push(ui.nodes[i].key);
                }
            }
        }
        input.retain(|&key| key != 0);
        for key in input.iter().chain(&self.input) {
            if let Some(owner) = self.owner_of.get(key) {
                self.emit.insert(owner.clone());
            }
        }
        self.last_input = std::mem::replace(&mut self.input, input);
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
        // Computed values that follow a timeline that moved.
        for key in std::mem::take(&mut ui.kept.timeline_restyle) {
            if let Some(identity) = self.by_key.get(&key) {
                self.dirty.insert(identity.clone());
                self.emit.insert(identity.clone());
            }
        }
        self.dirty.extend(self.animated.iter().cloned());
        self.emit.extend(self.animated.iter().cloned());
        self.built_now.clear();
        self.fired = 0;
        self.mark_fired(ui);
    }

    /// Marks the listeners of the actions fired since the last mark.
    fn mark_fired(&mut self, ui: &Ui) {
        // Whether a listener's layer takes input is the build's question
        // (`docs.accepts`): every listener of a fired action is built.
        let fired = &ui.actions.fired;
        for action in &fired[self.fired.min(fired.len())..] {
            for listener in self.on_action.get(action).into_iter().flatten() {
                if !self.built_now.contains(listener) {
                    self.emit.insert(listener.clone());
                }
            }
        }
        self.fired = fired.len();
    }

    /// Goes on one tick with each Rive frame not built this frame: its
    /// player advances and its draw is replaced on its node, which paints
    /// again on its own; nothing is resolved or built.
    fn advance_rive(&mut self, ui: &mut Ui) {
        let elements = &self.elements;
        self.rive.retain(|identity| elements.contains_key(identity));
        self.unbuilt_hidden
            .retain(|identity| elements.contains_key(identity));
        for identity in &self.rive {
            let e = &self.elements[identity];
            let Some(slot) = e.slot else { continue };
            if !ui.holds(slot, identity.key, e.parent_slot) {
                continue;
            }
            let Some(resolved) = e.resolved.as_ref() else {
                continue;
            };
            let Some(board) = resolved.values.str("rive") else {
                continue;
            };
            let Some(index) = e
                .record
                .render_fields
                .iter()
                .find(|(k, _)| *k == resolved.key)
                .map(|(_, fields)| fields.index)
            else {
                continue;
            };
            let built = ui
                .state
                .rive
                .get(&ui.nodes[index].key)
                .is_some_and(|s| s.built == ui.state.tick);
            if !built && !ui.nodes[index].hidden {
                ui.play_rive(index, board, &resolved.path);
            }
        }
    }

    /// Builds the listeners of actions that triggers fired while this
    /// frame was built and that are not built yet, as every listener after
    /// the trigger in the frame reacts.
    fn emit_fired(
        &mut self,
        ui: &mut Ui,
        env: &dyn Env,
        events: &mut Vec<DocEvent>,
        writes: &mut Vec<(BindingContext, String, Value)>,
    ) -> Result<(), String> {
        while ui.actions.fired.len() > self.fired {
            self.mark_fired(ui);
            self.emit_in_place(ui, env, events, writes)?;
        }
        Ok(())
    }
}

/// A mount's root path: `scope:id`, and for a screen, whose layer is
/// `scope:id`, its root's id under it (`scope:id/Menu`), as a pushed
/// screen's root is `<screen>/<root id>`.
fn root_path(mount: &SourceId, doc: &UiDocument) -> String {
    let base = format!("{}:{}", mount.scope, mount.id);
    if doc.kind != crate::doc::DocKind::Screen {
        return base;
    }
    match doc.nodes.get(doc.root) {
        Some(n) if !n.id.is_empty() => format!("{base}/{}", n.id),
        _ => format!("{base}/Root"),
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
            recording: Default::default(),
        }
    }
}

impl Ui {
    pub fn mount_document(
        &mut self,
        mount: SourceId,
        document: &str,
        inputs: BTreeMap<String, SourceId>,
        content: BTreeMap<String, String>,
        env: &dyn Env,
    ) -> Result<(), String> {
        mount.validate()?;
        let doc = self
            .docs
            .docs
            .get(document)
            .cloned()
            .ok_or_else(|| format!("missing document '{document}'"))?;
        // Every mount reads the engine's `$ui` unless it binds its own.
        let mut inputs = inputs;
        if !inputs.contains_key("ui") {
            self.publish_ui_store();
            inputs.insert("ui".into(), crate::doc::UI_STORE.clone());
        }
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
            content: content.clone(),
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
            lexical: Arc::default(),
        };
        let lexical = match content::mount_content(self, &content, &context) {
            Ok(lexical) => lexical,
            Err(error) => {
                self.docs.retained = host;
                return Err(error);
            }
        };
        let context = BindingContext {
            lexical: Arc::new(lexical),
            ..context
        };
        let root = host.compile(
            self,
            (doc.clone(), doc.root),
            context,
            None,
            root_path(&mount, &doc),
            env,
        );
        let result = match root {
            Ok(root) => {
                host.registrations.insert(mount.clone(), registration);
                host.roots.insert(mount.clone(), root);
                self.docs.retained = host;
                self.sync_mount_layer(&mount, Some(document));
                return Ok(());
            }
            Err(error) => {
                host.remove_mount(self, &mount);
                Err(error)
            }
        };
        self.docs.retained = host;
        result
    }

    /// Unmounts a mount. A mounted screen leaves the stack with it; when its
    /// screen has a view transition, the mount stays until the cover has
    /// closed over it.
    pub fn unmount_document(&mut self, mount: &SourceId) {
        if let Some(k) = self.mount_layer(mount) {
            if self.layer_leaves(k) {
                return;
            }
            self.docs.layers.layers.remove(k);
        }
        self.finish_unmount(mount);
    }

    pub(crate) fn finish_unmount(&mut self, mount: &SourceId) {
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
            host.contexts.clear();
            let mut events = Vec::new();
            let mut writes = Vec::new();
            for (mount, root) in host
                .roots
                .iter()
                .map(|(m, r)| (m.clone(), r.clone()))
                .collect::<Vec<_>>()
            {
                // A mounted screen is a layer: its root is emitted inside the
                // layer's frame, and released while a layer above hides it.
                let layer = self.mount_layer(&mount);
                let frame = layer.map(|k| {
                    let (frame, lead) = self.open_layer_frame(k);
                    events.extend(lead);
                    let shown = frame.is_some();
                    let was = std::mem::replace(&mut self.docs.layers.layers[k].shown, shown);
                    if was && !shown {
                        host.release(self, &root);
                    }
                    frame
                });
                if matches!(frame, Some(None)) {
                    continue;
                }
                host.contexts
                    .insert(mount.clone(), (self.docs.accepts, self.docs.building));
                let emitted = host.emit(self, &root, env, &mut events, &mut writes);
                if let Some(Some(frame)) = frame {
                    let doc = &self.docs.docs[&host.registrations[&mount].document];
                    let path = root_path(&mount, doc);
                    events.extend(self.close_layer_frame(frame, &path));
                }
                if let Err(error) = emitted {
                    result = Err(error);
                    break;
                }
            }
            if result.is_ok()
                && let Err(error) = host
                    .emit_in_place(self, env, &mut events, &mut writes)
                    .and_then(|()| host.emit_fired(self, env, &mut events, &mut writes))
            {
                result = Err(error);
            }
            if result.is_ok() {
                host.advance_rive(self);
            }
            events.append(&mut host.uses);
            host.input_events(self, env, &mut events);
            self.docs.retained = std::mem::take(&mut host);
            self.keep_world_anchors();
            host = std::mem::take(&mut self.docs.retained);
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
            Default::default(),
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
        // Every cell animates its opacity: settled, none is visited.
        doc.push_str("</row><style>text { transition: opacity 200ms; }</style>");
        ui.define_document(UiDocument::parse("grid.svelte", &doc).unwrap());
        ui.mount_document(
            SourceId::new("test", "grid"),
            "grid",
            inputs,
            Default::default(),
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
        assert_eq!(
            (
                idle.bindings,
                idle.reconciled,
                idle.laid_out,
                idle.painted,
                idle.animated
            ),
            (0, 0, 0, 0, 0),
            "{idle:?}"
        );
        ui.reactive
            .execute(Command::Patch {
                source: SourceId::new("test", "c2"),
                key: "r19".into(),
                fields: json!({"text": "Value 7"}).as_object().unwrap().clone(),
            })
            .unwrap();
        let changed = tick(&mut ui);
        // Motion visits only the label written, not its moved siblings.
        assert_eq!(
            (changed.bindings, changed.reconciled, changed.animated),
            (1, 1, 1),
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
        assert_eq!(
            (idle.bindings, idle.reconciled, idle.animated),
            (0, 0, 0),
            "{idle:?}"
        );
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
            Default::default(),
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
            Default::default(),
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
            <column><label><input type="checkbox" switch id="Choice" checked={$checked}> Choose</label></column>
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
            Default::default(),
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
            Default::default(),
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
            Default::default(),
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
            Default::default(),
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
            Default::default(),
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

    #[test]
    fn script_props_derived_const_await_and_key_blocks_follow_svelte() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "badge.svelte",
                r#"
            <script lang="ts">
                // Data only: props with defaults and pure derived values.
                let { label = 'none', count = 2 }: { label?: string; count?: number } = $props();
                const doubled = $derived(count * 2);
                const shown = $derived.by(() => label + ':' + doubled);
            </script>
            <meta name="component" content="Badge">
            <text id="Badge">{shown}</text>
        "#,
            )
            .unwrap(),
        );
        ui.define_document(
            UiDocument::parse(
                "shop.svelte",
                r#"
            <meta name="component" content="shop">
            <column id="Root">
                <Badge id="Badge" label={$name}/>
                {#await $load}<text id="Wait">wait</text>
                {:then v}{@const n = v.count}<text id="Done">{n + 1}</text>
                {:catch e}<text id="Fail">{e}</text>{/await}
                {#key $round}<text id="Round">{$round}</text>{/key}
            </column>
        "#,
            )
            .unwrap(),
        );
        let store = |name: &str| SourceId::new("test", name);
        for (name, value) in [
            ("name", json!("gold")),
            ("load", json!({ "status": "pending" })),
            ("round", json!(1)),
        ] {
            ui.reactive
                .execute(Command::Signal {
                    source: store(name),
                    initial: value,
                    readable: false,
                })
                .unwrap();
        }
        ui.mount_document(
            store("screen"),
            "shop",
            ["name", "load", "round"]
                .into_iter()
                .map(|n| (n.to_string(), store(n)))
                .collect(),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let frame = |ui: &mut Ui, set: Option<(&str, Value)>| {
            if let Some((name, value)) = set {
                ui.reactive
                    .execute(Command::Set {
                        source: store(name),
                        value,
                        force: false,
                    })
                    .unwrap();
            }
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            ui.probe_elements()
                .into_iter()
                .map(|e| (e.path, e.text))
                .collect::<Vec<_>>()
        };
        let text = |els: &[(String, String)], id: &str| {
            els.iter()
                .find(|(p, _)| p.rsplit('/').next().unwrap().starts_with(id))
                .map(|(p, t)| (p.clone(), t.clone()))
        };
        let first = frame(&mut ui, None);
        // The default count (2) doubled, beside the label passed in.
        assert_eq!(text(&first, "Badge").unwrap().1, "gold:4");
        assert!(text(&first, "Wait").is_some() && text(&first, "Done").is_none());
        let (round_path, round) = text(&first, "Round").unwrap();
        assert_eq!(round, "1");
        let done = frame(
            &mut ui,
            Some((
                "load",
                json!({ "status": "fulfilled", "value": { "count": 3 } }),
            )),
        );
        assert_eq!(text(&done, "Done").unwrap().1, "4");
        assert!(text(&done, "Wait").is_none() && text(&done, "Fail").is_none());
        let failed = frame(
            &mut ui,
            Some(("load", json!({ "status": "rejected", "reason": "offline" }))),
        );
        assert_eq!(text(&failed, "Fail").unwrap().1, "offline");
        // A new key value makes a new element; the same value keeps it.
        let next = frame(&mut ui, Some(("round", json!(2))));
        let (next_path, next_text) = text(&next, "Round").unwrap();
        assert_eq!(next_text, "2");
        assert_ne!(next_path, round_path);
        let again = frame(&mut ui, Some(("name", json!("silver"))));
        assert_eq!(text(&again, "Round").unwrap().0, next_path);
        assert_eq!(text(&again, "Badge").unwrap().1, "silver:4");
    }

    #[test]
    fn directives_map_onto_presence_inline_style_and_use_events() {
        let mut ui = Ui::new(UiDesc::default());
        let doc = UiDocument::parse(
            "fx.svelte",
            r#"
            <meta name="component" content="fx">
            <svelte:window bind:innerWidth={vw}/>
            <column id="Root">
                {#if $show}
                <div id="Box" use:tooltip={{ text: 'hi' }} use:track={$n}
                     style="width: 40px; height: 12px" style:width={$w + 'px'}
                     transition:fly={{ y: 20, duration: 200, easing: quintOut }}></div>
                {/if}
                <p bind:this={note}>{vw}</p>
                <svelte:element this={$level}>{note}</svelte:element>
            </column>
            <style>#Box { width: 10px; height: 30px; } #note { width: 50px; }</style>
        "#,
        )
        .unwrap();
        let boxed = doc.nodes.iter().find(|n| n.id == "Box").unwrap();
        let presence = |name: &str| {
            boxed
                .props
                .iter()
                .find(|p| p.name() == name)
                .map(|p| match &p.value {
                    PropValue::Lit(crate::doc::Lit::Presence(s)) => s.clone(),
                    other => panic!("{other:?}"),
                })
        };
        let enter = presence("enter").unwrap();
        assert_eq!(enter, presence("exit").unwrap());
        assert!(enter.values.contains(&(crate::element::Prop::Y, 20.0)));
        assert!(enter.values.contains(&(crate::element::Prop::Opacity, 0.0)));
        assert_eq!(
            enter.transition,
            crate::motion::Transition::Tween {
                duration_ms: 200.0,
                ease: crate::motion::Ease::CubicBezier([0.22, 1.0, 0.36, 1.0])
            }
        );
        ui.define_document(doc);
        let store = |name: &str| SourceId::new("test", name);
        for (name, value) in [
            ("show", json!(true)),
            ("n", json!(1)),
            ("w", json!(70)),
            ("level", json!("h2")),
        ] {
            ui.reactive
                .execute(Command::Signal {
                    source: store(name),
                    initial: value,
                    readable: false,
                })
                .unwrap();
        }
        ui.mount_document(
            store("screen"),
            "fx",
            ["show", "n", "w", "level"]
                .into_iter()
                .map(|n| (n.to_string(), store(n)))
                .collect(),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let frame = |ui: &mut Ui, set: Option<(&str, Value)>| {
            if let Some((name, value)) = set {
                ui.reactive
                    .execute(Command::Set {
                        source: store(name),
                        value,
                        force: false,
                    })
                    .unwrap();
            }
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            let uses: Vec<_> = ui
                .take_doc_events()
                .into_iter()
                .filter(|e| e.kind == "use")
                .map(|e| (e.name, e.value["phase"].clone(), e.value["params"].clone()))
                .collect();
            (ui.probe_elements(), uses)
        };
        let (els, uses) = frame(&mut ui, None);
        let find = |id: &str| els.iter().find(|e| e.path.ends_with(id)).unwrap().clone();
        // The bound style:width beats the attribute's, which beats the rule.
        assert_eq!((find("/Box").width, find("/Box").height), (70.0, 12.0));
        assert_eq!(find("/note").text, "400");
        assert_eq!(find("/note").width, 50.0);
        assert!(els.iter().any(|e| e.text == "note" && e.role == "heading"));
        assert_eq!(
            uses,
            vec![
                ("tooltip".into(), json!("create"), json!({ "text": "hi" })),
                ("track".into(), json!("create"), json!(1)),
            ]
        );
        let (els, uses) = frame(&mut ui, Some(("n", json!(2))));
        assert_eq!(uses, vec![("track".into(), json!("update"), json!(2))]);
        assert_eq!(
            els.iter().find(|e| e.path.ends_with("/Box")).unwrap().width,
            70.0
        );
        let (_, uses) = frame(&mut ui, Some(("show", json!(false))));
        assert_eq!(
            uses,
            vec![
                ("tooltip".into(), json!("destroy"), Value::Null),
                ("track".into(), json!("destroy"), Value::Null),
            ]
        );
    }

    #[test]
    fn blur_and_draw_transitions_move_a_filter_and_a_stroke() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "fx2.svelte",
                r##"
            <meta name="component" content="fx2">
            <column id="Root">
                {#if $show}
                <div id="Soft" in:blur={{ amount: 8, duration: 400 }}></div>
                <svg id="Pic" viewBox="0 0 100 100">
                    <path id="Line" d="M10 10 L90 10" stroke="#ffffff" stroke-width="4"
                          in:draw={{ duration: 400, easing: linear }}/>
                </svg>
                {/if}
            </column>
            <style>#Soft { width: 40px; height: 20px; background: #ff0000; }
            #Pic { width: 100px; height: 100px; }</style>
        "##,
            )
            .unwrap(),
        );
        let show = SourceId::new("test", "show");
        ui.reactive
            .execute(Command::Signal {
                source: show.clone(),
                initial: json!(false),
                readable: false,
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "screen"),
            "fx2",
            [("show".to_string(), show.clone())].into_iter().collect(),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        // The blur layer's radius and the line's dash and offset.
        let frame = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            let mut blur = None;
            let mut dash = None;
            for item in &ui.frame().items {
                match &item.cmd {
                    crate::DrawCmd::BeginLayer {
                        material: og_render::UiMaterial::Builtin(og_render::UiBuiltin::Blur),
                        params,
                        ..
                    } => blur = Some(params[0][0]),
                    crate::DrawCmd::Shape { shape, .. } => {
                        if let og_render::UiMode::Stroke(s) = shape.mode {
                            dash = Some((s.dash, s.dash_offset));
                        }
                    }
                    _ => {}
                }
            }
            (blur, dash)
        };
        frame(&mut ui);
        ui.reactive
            .execute(Command::Set {
                source: show,
                value: json!(true),
                force: false,
            })
            .unwrap();
        let mut seen = Vec::new();
        for _ in 0..40 {
            seen.push(frame(&mut ui));
        }
        // Halfway: blurred by under its 8 px (a 16 px radius), and the
        // 80-unit line half hidden by one dash and one gap of its length.
        let (blur, dash) = seen[12];
        let r = blur.expect("a blur layer while it enters");
        assert!(r > 0.0 && r < 16.0, "{r}");
        let ([d, g], offset) = dash.expect("the line's stroke");
        assert!((d - 80.0).abs() < 0.5 && (g - 80.0).abs() < 0.5, "{d} {g}");
        assert!(offset > 20.0 && offset < 60.0, "{offset}");
        // Settled: no layer, a solid stroke.
        assert_eq!(seen[39], (None, Some(([0.0; 2], 0.0))));
    }
}
