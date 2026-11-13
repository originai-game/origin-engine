//! The retained host's state: the mounted elements, their structural
//! instances and registrations, and what the input pass remembers.

use super::*;
use crate::bind::Expr;
use crate::doc::topology::{Materialization, Topology};
use crate::reactive::{
    SourceId,
    bindings::{BindingContext, BindingId, Bindings, ListRef},
    graph::MountedNode,
};

#[derive(Clone, Debug, Default)]
pub struct CachedInputs {
    pub expressions: Vec<(Expr, Value)>,
    pub properties: Vec<(PropValue, Ty, Option<V>)>,
}

impl CachedInputs {
    /// Records an immediate build's evaluated input (container-query replay).
    pub(crate) fn remember_expression(&mut self, expression: &Expr, value: &Value) {
        if !self.expressions.iter().any(|(e, _)| e == expression) {
            self.expressions.push((expression.clone(), value.clone()));
        }
    }

    pub(crate) fn remember_property(&mut self, property: &PropValue, ty: Ty, value: &Option<V>) {
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
pub enum Input {
    Expression(Expr),
    Property(PropValue, Ty),
    Structure,
}

#[derive(Clone)]
pub struct ElementInstance {
    pub document: Arc<UiDocument>,
    pub index: usize,
    pub context: BindingContext,
    pub path: String,
    pub id: String,
    pub cached: CachedInputs,
    pub resolved: Option<ResolvedNode>,
    pub style_dirty: bool,
    pub theme_epoch: u64,
    pub children: Vec<Child>,
    pub bindings: Vec<BindingId>,
    /// Its render node and the parent it was written under, while it has one.
    pub slot: Option<usize>,
    pub parent_slot: usize,
    /// Its own node and its widget's internal nodes: what it owns in the
    /// render tree and retires when it stops writing them.
    pub owned: Vec<usize>,
    /// The render keys of those nodes and of its links, for input routing.
    pub keys: Vec<u64>,
    /// Its probe record, render fields and art slots.
    pub record: super::host::DocRecord,
    pub memories: Vec<u64>,
    /// Built again every frame: it plays Rive, hosts a code slot, answers
    /// named actions, or its widget memories are in motion.
    pub volatile: bool,
    pub materialization: Materialization,
    /// While documents use generated content: the counter run as it
    /// entered this element, what its subtree did to the run, and whether
    /// any generated text in it read the run.
    pub run_in: Option<Arc<crate::doc::generated::Run>>,
    pub run_events: Arc<Vec<crate::doc::generated::Event>>,
    pub run_reads: bool,
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Registration {
    pub document: String,
    pub inputs: BTreeMap<String, SourceId>,
    /// Content passed by reference: a document, or `document#snippet`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub content: BTreeMap<String, String>,
}

#[derive(Clone, Default)]
pub struct RetainedHost {
    pub topology: Topology,
    pub bindings: Bindings,
    pub inputs: BTreeMap<BindingId, (MountedNode, Input)>,
    pub pending_bindings: BTreeSet<BindingId>,
    pub elements: BTreeMap<MountedNode, ElementInstance>,
    pub roots: BTreeMap<SourceId, Child>,
    pub registrations: BTreeMap<SourceId, Registration>,
    pub managed: Option<BTreeMap<SourceId, Registration>>,
    pub pending_mounts: BTreeSet<SourceId>,
    pub restoring: bool,
    pub each: BTreeMap<MountedNode, EachInstance>,
    pub branches: BTreeMap<MountedNode, BranchInstance>,
    /// `use:` lifecycle events (made, params changed, gone) for the next
    /// build's answer.
    pub uses: Vec<DocEvent>,
    pub collections: BTreeMap<SourceId, BTreeSet<MountedNode>>,
    pub dirty: BTreeSet<MountedNode>,
    pub inherited: BTreeMap<(MountedNode, String), BTreeSet<MountedNode>>,
    pub by_key: BTreeMap<u64, MountedNode>,
    /// The elements matching `:hover`, `:active`, `:focus` and
    /// `:focus-visible` last frame: the pointer's, the focus's and any the
    /// probe forced.
    pub interaction: [Vec<u64>; 4],
    /// Popover elements by their popover key, for `:popover-open`.
    pub popover_nodes: BTreeMap<u64, MountedNode>,
    /// The `:target` element last applied, and the fragment it was for.
    pub target: (Option<String>, Vec<MountedNode>),
    pub viewport: Option<crate::doc::media::Context>,
    /// The safe area, the small and large viewports and the HUD band's
    /// sides last build: a change re-resolves what read the viewport
    /// (`env()`, `svh`, `lvh`).
    pub surface: [f32; 10],
    pub theme: Option<crate::Theme>,
    pub presentation_epoch: u64,
    /// The surface's direction last build: a document's root takes it.
    pub direction: Option<og_i18n::Direction>,
    pub theme_epoch: u64,
    pub media: [BTreeSet<MountedNode>; crate::doc::media::AXES],
    pub materializing: Option<Materialization>,
    pub built_tick: Option<u64>,
    /// Elements whose render nodes must be written again this frame.
    pub emit: BTreeSet<MountedNode>,
    /// Elements built every frame ([`ElementInstance::volatile`]).
    pub volatile: BTreeSet<MountedNode>,
    /// Elements resolved again every frame: their SMIL animations play.
    pub animated: BTreeSet<MountedNode>,
    /// The elements not built because they were hidden: shown again, the
    /// nearest ancestor that holds its node builds them.
    pub unbuilt_hidden: BTreeSet<MountedNode>,
    /// The elements whose frame is a Rive artboard: their players go on
    /// every tick ([`RetainedHost::advance_rive`]), without building them.
    pub rive: BTreeSet<MountedNode>,
    /// The elements with a trigger on each action: built in a frame the
    /// action fires, not every frame.
    pub on_action: BTreeMap<String, BTreeSet<MountedNode>>,
    /// The actions fired this frame whose listeners are marked, and the
    /// elements built this frame (kept while any element listens): an
    /// action a trigger fires while building reaches the listeners not yet
    /// built ([`RetainedHost::emit_fired`]).
    pub fired: usize,
    pub built_now: BTreeSet<MountedNode>,
    /// This frame's layer context of each mount emitted (whether its layer
    /// takes UI actions, the layer's serial): an element written in place
    /// after the layers' frames closed is written in its own.
    pub contexts: BTreeMap<SourceId, (bool, u32)>,
    /// The element that owns each render key (its nodes', its links').
    pub owner_of: std::collections::HashMap<u64, MountedNode>,
    /// The widget memories mounted elements keep while they are not built.
    pub memories: std::collections::HashMap<u64, u32>,
    /// This frame's and last frame's interaction keys (hover, press,
    /// focus, activation).
    pub input: Vec<u64>,
    pub last_input: Vec<u64>,
    /// The frame-wide inputs changed this frame: every node is written.
    pub restyled: bool,
    /// What every element's nodes depend on beyond its own inputs.
    pub signature: Option<Signature>,
    /// Virtual containers and the scroll offset they were emitted at.
    pub scrollers: BTreeMap<MountedNode, f32>,
    pub evaluated: usize,
    pub resolved: usize,
    /// Elements whose render nodes were written this frame.
    pub reconciled: usize,
    /// Structural-selector work since the last refresh.
    pub css: CssWork,
    pub profiles: BTreeMap<String, crate::doc::matching::Profile>,
    /// The input pass's memory ([`pointer`]).
    pub pointer: PointerMemory,
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
pub struct Signature {
    pub viewport: crate::Viewport,
    pub unit: f32,
    pub direction: og_i18n::Direction,
    pub locale: String,
    pub style: String,
    pub style_epoch: u64,
    pub players: u32,
    pub nav_on: bool,
}

impl std::fmt::Debug for RetainedHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetainedHost")
            .field("roots", &self.roots)
            .field("elements", &self.elements.len())
            .finish()
    }
}

#[derive(Clone, Debug)]
pub enum Child {
    Element(MountedNode),
    Each(MountedNode),
    Branch(MountedNode),
}

#[derive(Clone)]
pub struct BranchInstance {
    pub document: Arc<UiDocument>,
    pub index: usize,
    pub context: BindingContext,
    pub parent: MountedNode,
    pub suffix: String,
    pub render_prefix: String,
    pub binding: BindingId,
    pub enabled: bool,
    /// A `{#key}` block's value's hash: its children are new elements,
    /// their ids carrying it, each time the value changes.
    pub keyed: Option<u64>,
    pub children: Vec<Child>,
}

#[derive(Clone)]
pub struct EachInstance {
    pub document: Arc<UiDocument>,
    pub index: usize,
    pub context: BindingContext,
    pub parent: MountedNode,
    pub list: ListRef,
    /// A list held in a row, read as a value: its items' keys are read
    /// from it again when it changes. A collection reports its own edits.
    pub binding: Option<BindingId>,
    pub revision: u64,
    pub order: Vec<String>,
    pub rows: BTreeMap<String, Vec<Child>>,
    pub empty: Vec<Child>,
    pub materialized: BTreeSet<String>,
}

/// What the input pass remembers between ticks.
#[derive(Clone, Copy, Debug, Default)]
pub struct PointerMemory {
    /// The element a press is held on, whether it was dragging, and the
    /// drop target it was over.
    pub drag: Option<(u64, bool, u64)>,
    /// The element the last press went down on (`pointerup` goes to it,
    /// as captured pointers do).
    pub pressed: u64,
    pub pointer: [f32; 2],
    /// Two touches pinching: the element they went down on, their spread
    /// then, and the scale last sent.
    pub pinch: Option<(u64, f32, f32)>,
}

impl RetainedHost {
    /// Render key `key` belongs to a mounted element.
    pub fn owns(&self, key: u64) -> bool {
        self.owner_of.contains_key(&key)
    }

    pub fn deferred_focus_path(&self, key: u64) -> Option<&str> {
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

    /// The document cascade's winning declarations of the element the
    /// render key `key` belongs to (`file:line` each), and its bindings, as
    /// the UI debugger shows them (`Ui::debug_element`): read from what its
    /// last resolve kept, for that one element.
    pub fn debug_styles(
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

    /// A widget memory a mounted element keeps while it is not built.
    pub fn keeps_memory(&self, key: u64) -> bool {
        self.memories.contains_key(&key)
    }
}

impl RetainedHost {
    pub fn record_mut(&mut self, key: u64) -> Option<&mut super::host::DocRecord> {
        let identity = self.by_key.get(&key)?;
        Some(&mut self.elements.get_mut(identity)?.record)
    }
}

impl RetainedHost {
    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
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

    /// The layout keys of mounted query containers.
    pub fn containers(&self) -> impl Iterator<Item = u64> + '_ {
        self.elements
            .values()
            .filter_map(|e| e.resolved.as_ref()?.values.container)
    }

    /// The containers whose scroll state mounted elements read.
    pub(crate) fn scroll_reads(&self) -> impl Iterator<Item = u64> + '_ {
        self.elements
            .values()
            .filter_map(|e| e.resolved.as_ref())
            .flat_map(|r| r.values.cascade.scroll_reads.iter().copied())
    }
}

impl super::host::DocMemory {
    pub(crate) fn document_registration_state(&self) -> Value {
        json!(
            self.retained
                .managed
                .as_ref()
                .map(|m| m.iter().collect::<Vec<_>>())
        )
    }

    pub(crate) fn restore_document_mounts(
        &self,
        value: &Value,
        managed: &Value,
    ) -> Result<RetainedHost, String> {
        let mut restored = RetainedHost::restore(value)?;
        let saved = if managed.is_null() {
            None
        } else {
            Some(RetainedHost::restore(managed)?.registrations)
        };
        if let Some(current) = &self.retained.managed {
            if let Some(saved) = &saved {
                for id in saved.keys().filter(|id| !current.contains_key(*id)) {
                    restored.registrations.remove(id);
                }
            }
            for (id, declaration) in current {
                // Active mounts use current code. A newly declared mount takes
                // its default; a saved runtime unmount remains unmounted.
                if restored.registrations.contains_key(id)
                    || saved.as_ref().is_none_or(|m| !m.contains_key(id))
                {
                    restored
                        .registrations
                        .insert(id.clone(), declaration.clone());
                }
            }
            restored.managed = Some(current.clone());
        } else {
            restored.managed = saved;
        }
        restored.pending_mounts = restored.registrations.keys().cloned().collect();
        restored.restoring = !restored.pending_mounts.is_empty();
        Ok(restored)
    }
}

impl ElementInstance {
    pub fn build<'a>(
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
