//! The documents' state the stages read: the documents a UI knows, what
//! their mounts keep between ticks, and what a build tells code and the probe.

use super::*;

/// A text field's constraint validity as last built ([`Act::Submit`]).
#[derive(Clone, Debug)]
pub struct FieldValidity {
    pub mount: String,
    pub doc: String,
    pub node: usize,
    pub row_key: String,
    pub invalid: bool,
}

/// Something a document did that code may want: a click, a changed
/// control, a named event (`emit`, a timeline's callback), an action, a
/// two-way write (`set`), a Back the layer turned into an event.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DocEvent {
    /// The document's mount: the screen's name, or a component instance's path.
    pub mount: String,
    /// The element's path.
    pub path: String,
    /// Stable key of the innermost each row; empty outside a row.
    pub row_key: String,
    /// `click`, `changed`, `select`, `emit`, `action`, `set`, `back`.
    pub kind: String,
    /// The event's or the action's name, or the path a `set` wrote.
    pub name: String,
    pub value: Value,
}

impl DocEvent {
    pub fn to_json(&self) -> Value {
        json!({ "mount": self.mount, "path": self.path, "rowKey": self.row_key, "kind": self.kind, "name": self.name, "value": self.value })
    }
}

/// What a mount answers: its events this tick, and whether its viewmodel changed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DocResponse {
    pub events: Vec<DocEvent>,
    /// Its root's path (empty: it was not built).
    pub path: String,
}

/// A probe edit of a prop (`ui:set`): presentation only, marked in the dump.
#[derive(Clone, Debug, PartialEq)]
pub struct Edit {
    pub path: String,
    pub prop: String,
    pub spec: usize,
    pub value: PropValue,
    /// The value as given.
    pub source: String,
}

/// One element a document built this tick, for the probe.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Built {
    pub path: String,
    /// The document's name and the node's line.
    pub doc: String,
    pub node: usize,
    pub line: usize,
    /// The mount it belongs to (a screen's name is its layer).
    pub mount: String,
    /// `(prop, expression as written, value now)`.
    pub bindings: Vec<(String, String, Value)>,
    /// Its probe edits: `(prop, value as given)`.
    pub edits: Vec<(String, String)>,
    /// The state groups it declares, with the state each is in by its binding.
    pub states: Vec<(String, String)>,
}

/// What one retained element registered as it was emitted: its probe
/// record, render fields and art slots. It lives with the element and is
/// replaced only when the element is emitted again.
#[derive(Clone, Debug, Default)]
pub struct DocRecord {
    pub built: Vec<Built>,
    pub render_fields: Vec<(u64, super::fields::RenderFields)>,
    pub arts: Vec<(String, crate::slot::SlotSpec)>,
}

/// The documents a UI knows and what their mounts keep between ticks.
/// A view transition's covers still opening after their layer left: the
/// covers, their colour and image, and the tick it left.
pub type Revealing = (
    Vec<crate::motion::css::CssAnimation>,
    Option<crate::Srgba>,
    Option<String>,
    u64,
);

#[derive(Clone, Debug, Default)]
pub struct DocHost {
    /// By name.
    pub docs: BTreeMap<String, Arc<UiDocument>>,
    /// The states triggers set (`on click state selection on`), by element key and group.
    pub states: BTreeMap<(u64, String), String>,
    /// `data-og-instant` elements whose first entrance has landed: a
    /// later one plays (presentation, outside `world_hash`).
    pub(crate) entered: BTreeSet<u64>,
    /// Some defined document writes a CSS-wide keyword on a property, so
    /// elements carry their winners to their children's `inherit`.
    pub wide_keywords: bool,
    /// The `$ui` store's value as last published.
    pub(crate) ui_store: Option<Value>,
    pub layers: crate::layer_stack::LayerStack,
    /// The textures documents name by asset path (a view transition's
    /// cover image), by path.
    pub textures: BTreeMap<String, og_draw::TextureHandle>,
    /// The alpha of the textures a `shape-outside: url()` names, by path
    /// ([`Ui::define_texture_alpha`]).
    pub alphas: BTreeMap<String, Arc<crate::layout::float::ImageAlpha>>,
    /// The Rive artboards elements can play, by id ([`Ui::define_rive`]).
    pub rive: BTreeMap<String, Arc<crate::rive::Artboard>>,
    /// The `type ui-material` assets documents name (`-og-material:
    /// "<path>" …`): each one's handle and its parameters' defaults.
    pub materials: BTreeMap<
        String,
        (
            og_draw::UiMaterialHandle,
            [[f32; 4]; og_draw::UI_MATERIAL_PARAMS],
        ),
    >,
    /// CSS `@keyframes` of every defined document, as `<doc>/<name>`.
    pub keyframes: BTreeMap<String, Arc<crate::motion::css::Keyframes>>,
    /// Query containers' settled content boxes, by layout key.
    pub(crate) container_sizes: crate::doc::container::Sizes,
    /// This tick's immediate-built elements under a query container, in
    /// build order, with what settlement needs to resolve them again.
    pub(crate) queried: Vec<Queried>,
    /// Query containers watched for settlement, and those built this tick.
    pub(crate) containers: BTreeSet<u64>,
    pub(crate) containers_now: BTreeSet<u64>,
    /// `scroll-state()` containers read: their offset and flags
    /// (`media::scroll`) at this tick's start, those immediate builds read
    /// this tick, and those whose flags changed at its start.
    pub scroll_states: BTreeMap<u64, (og_core::glam::Vec2, u16)>,
    pub scroll_now: BTreeSet<u64>,
    pub scroll_changed: BTreeSet<u64>,
    pub uses_run: bool,
    /// What structural selectors of every defined document read.
    pub structure: crate::doc::matching::Profile,
    /// Global rules can read facts in a different document from their subject.
    pub global_structure: crate::doc::matching::Profile,
    /// The instantiated sibling levels of the screen being built.
    pub levels: Vec<super::structure::Level>,
    /// A component's root continues its instance's place among siblings.
    pub component_root: bool,
}

/// What the documents keep between ticks that a snapshot carries: their
/// mounts, the reactive session, the elements seen and the triggers' last
/// values. The documents hold it in their state ([`DocumentState`]); the
/// stages read and write it in [`Ui::save_state`] and [`Ui::restore_state`],
/// with or without the documents.
#[derive(Clone, Debug, Default)]
pub struct DocMemory {
    pub retained: super::instances::RetainedHost,
    pub reactive: crate::reactive::session::Session,
    /// Document elements seen last tick (for `on appear`).
    pub seen: BTreeSet<u64>,
    pub seen_now: BTreeSet<u64>,
    /// A trigger's last value (`on change`), by element key and trigger.
    pub last: BTreeMap<(u64, usize), Value>,
}

impl DocMemory {
    /// The tick starts: what was seen is last tick's, and the mounts count afresh.
    pub fn begin(&mut self) {
        self.seen = std::mem::take(&mut self.seen_now);
        self.retained.evaluated = 0;
        self.retained.resolved = 0;
        self.retained.reconciled = 0;
    }

    /// Restores what a snapshot carried ([`Ui::restore_state`]).
    pub fn restore(&mut self, v: &Value) -> Result<(), String> {
        let retained = self.restore_document_mounts(&v["mounts"], &v["mountRegistrations"])?;
        if let Some(stores) = v.get("stores") {
            self.reactive.restore(stores)?;
        } else {
            self.reactive = Default::default();
        }
        self.reactive.graph.clear_bindings();
        self.retained = retained;
        let key = |v: &Value| v.as_str().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        self.seen_now = v["seen"]
            .as_array()
            .map(|a| a.iter().map(key).collect())
            .unwrap_or_default();
        self.seen = self.seen_now.clone();
        self.last = v["last"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|s| {
                        (
                            (key(&s[0]), s[1].as_u64().unwrap_or(0) as usize),
                            s[2].clone(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(())
    }

    /// The art slot `path`'s spec among the mounted elements' records.
    pub fn record_art_slot_mut(&mut self, path: &str) -> Option<&mut crate::slot::SlotSpec> {
        self.retained
            .record_mut(crate::id::key_of_path(path))
            .and_then(|r| r.arts.iter_mut().find(|(p, _)| p == path))
            .map(|(_, spec)| spec)
    }
}

/// The documents' state, held by the stages for them ([`crate::ui::Docs`]):
/// what a build keeps as it goes, what it tells code and the probe this
/// tick, and what its mounts keep between ticks. The documents define it;
/// the stages reach it through these operations, and a UI that has not
/// built a document holds [`NoDocuments`].
pub trait DocumentState: std::any::Any + std::fmt::Debug + Send + Sync {
    /// A copy, for a cloned UI.
    fn clone_state(&self) -> Box<dyn DocumentState>;
    /// What a snapshot carries.
    fn memory(&self) -> &DocMemory;
    fn memory_mut(&mut self) -> &mut DocMemory;
    /// The tick's document bookkeeping starts.
    fn begin_tick(&mut self);
    /// Restores what a snapshot carried.
    fn restore(&mut self, v: &Value) -> Result<(), String>;
    /// The probe's edits of props (`ui:set`).
    fn edits(&self) -> &[Edit];
    /// The render fields element `key` registered this tick.
    fn render_fields(&self, key: u64) -> Option<&super::fields::RenderFields>;
    /// Element `key`'s `::selection` style.
    fn selection(&self, key: u64) -> Option<&Arc<crate::doc::generated::Selection>>;
    /// The `::placeholder` color of the text control being built.
    fn placeholder_ink(&self) -> Option<Srgba>;
    /// The art slot registered last, by the element being emitted or this tick.
    fn last_art_entry(&self) -> Option<&(String, crate::slot::SlotSpec)>;
    /// The art slot `path`'s spec, where it was registered.
    fn art_slot_mut(&mut self, path: &str) -> Option<&mut crate::slot::SlotSpec>;
    /// Keeps the widget memory of elements `built` this frame or kept by
    /// their mounts, and forgets the rest.
    fn retain_element_memory(&mut self, built: &dyn Fn(u64) -> bool);
}

impl Clone for Box<dyn DocumentState> {
    fn clone(&self) -> Self {
        self.clone_state()
    }
}

/// The documents' state of a UI that has not built a document: what a
/// snapshot restored or the stages published into it, and a view of it the
/// documents read as their own state ([`NoDocuments::view`]).
#[derive(Debug, Default)]
pub struct NoDocuments {
    pub memory: DocMemory,
    view: std::sync::OnceLock<Box<dyn std::any::Any + Send + Sync>>,
}

impl NoDocuments {
    /// The documents' state made from this memory by `make`, made once
    /// until the memory changes.
    pub fn view<T: std::any::Any + Send + Sync>(&self, make: impl FnOnce(&DocMemory) -> T) -> &T {
        let view = self.view.get_or_init(|| Box::new(make(&self.memory)));
        (**view)
            .downcast_ref()
            .expect("one kind of documents' state")
    }
}

impl Clone for NoDocuments {
    fn clone(&self) -> Self {
        Self {
            memory: self.memory.clone(),
            view: Default::default(),
        }
    }
}

impl DocumentState for NoDocuments {
    fn clone_state(&self) -> Box<dyn DocumentState> {
        Box::new(self.clone())
    }
    fn memory(&self) -> &DocMemory {
        &self.memory
    }
    fn memory_mut(&mut self) -> &mut DocMemory {
        self.view.take();
        &mut self.memory
    }
    fn begin_tick(&mut self) {
        self.memory_mut().begin();
    }
    fn restore(&mut self, v: &Value) -> Result<(), String> {
        self.memory_mut().restore(v)
    }
    fn edits(&self) -> &[Edit] {
        &[]
    }
    fn render_fields(&self, _: u64) -> Option<&super::fields::RenderFields> {
        None
    }
    fn selection(&self, _: u64) -> Option<&Arc<crate::doc::generated::Selection>> {
        None
    }
    fn placeholder_ink(&self) -> Option<Srgba> {
        None
    }
    fn last_art_entry(&self) -> Option<&(String, crate::slot::SlotSpec)> {
        None
    }
    fn art_slot_mut(&mut self, path: &str) -> Option<&mut crate::slot::SlotSpec> {
        self.memory_mut().record_art_slot_mut(path)
    }
    fn retain_element_memory(&mut self, _: &dyn Fn(u64) -> bool) {}
}

/// What the controls naming a popover last did to it, by its key
/// ([`popover_key`]): presentation, rebuilt from the controls as they build.
#[derive(Clone, Copy, Debug, Default)]
pub struct PopoverLink {
    /// The control it is placed against (its node index and key).
    pub invoker: (usize, u64),
    /// An `interestfor` control holds the player's interest.
    pub interest: bool,
    /// Its layer's key, as last built (hovering it keeps a hint shown).
    pub layer: u64,
    /// It showed when last built (`ontoggle` fires as that changes).
    pub shown: bool,
    /// `auto`, `manual` or `hint`, as last built.
    pub mode: Mode,
    /// Opened by `contextmenu`: the point it opened at, its anchor.
    pub point: Option<[f32; 2]>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Auto,
    Manual,
    Hint,
}

/// What an immediate build evaluated for one element under a container:
/// its expression and property inputs, and its structural selector results.
#[derive(Clone, Debug, Default)]
pub(crate) struct Replay {
    pub(crate) cached: super::instances::CachedInputs,
    pub(crate) decisions: Vec<(usize, bool)>,
}

/// One immediate-built element under a query container, in build order.
#[derive(Clone, Debug)]
pub(crate) struct Queried {
    pub(crate) doc: Arc<UiDocument>,
    pub(crate) node: usize,
    pub(crate) path: String,
    pub(crate) key: u64,
    pub(crate) inherited: Arc<crate::doc::variables::Computed>,
    pub(crate) computed: Arc<crate::doc::variables::Computed>,
    pub(crate) reads: BTreeSet<u64>,
    pub(crate) replay: Arc<Replay>,
}

impl Ui {
    /// A document by name.
    pub fn document(&self, name: &str) -> Option<&UiDocument> {
        self.docs.host.docs.get(name).map(|d| &**d)
    }

    /// Every document's name.
    pub fn documents(&self) -> Vec<String> {
        self.docs.host.docs.keys().cloned().collect()
    }
}

impl crate::ui::Docs {
    /// A document's `-og-material` as the layer it draws: a built-in's, or
    /// a defined asset's with its defaults under the numbers written.
    pub fn css_material(
        &self,
        m: crate::dress::DressMaterial,
    ) -> Result<crate::paint::Layer, String> {
        let Some(path) = m.asset else {
            return Ok(m.layer);
        };
        let (handle, mut params) = *self.host.materials.get(&path).ok_or_else(|| {
            format!("-og-material: \"{path}\" is not a loaded UI material (a `type ui-material` asset under assets/)")
        })?;
        for k in 0..m.params {
            params[k / 4][k % 4] = m.layer.params[k / 4][k % 4];
        }
        Ok(crate::paint::Layer {
            material: crate::paint::Material::Custom(handle),
            params,
            filter: None,
        })
    }

    pub(crate) fn last_art_path(&self) -> String {
        self.last_art_entry().0.clone()
    }

    pub fn last_art_entry(&self) -> &(String, crate::slot::SlotSpec) {
        self.state
            .last_art_entry()
            .expect("an art slot was registered")
    }

    /// The art slot `path`'s spec, where it was registered.
    pub(crate) fn art_slot_mut(&mut self, path: &str) -> Option<&mut crate::slot::SlotSpec> {
        self.state.art_slot_mut(path)
    }
}

impl Ui {
    /// The tick's document bookkeeping starts (from `begin`).
    pub(crate) fn docs_begin(&mut self) {
        self.docs.state.begin_tick();
        let d = &mut self.docs.host;
        d.queried.clear();
        // Last tick's containers, immediate and mounted, stay watched.
        let mut live = std::mem::take(&mut d.containers_now);
        live.extend(self.docs.state.memory().retained.containers());
        self.watch_document_containers(live);
        self.docs.sync_scroll_states(&self.interaction, &self.tree);
    }
}

impl crate::ui::Docs {
    /// Plays the Rive artboard `id` in element `node` (its key `key`) this
    /// tick: the element's player goes on one tick, a restored one is
    /// rebuilt from its bytes, a new one starts; the element draws it over
    /// its background ([`crate::paint::ElementDraw::rive`]).
    #[allow(clippy::too_many_arguments)]
    pub fn play_rive(
        &mut self,
        node: usize,
        id: &str,
        path: &str,
        env: &crate::ui::Env,
        out: &mut crate::ui::Out,
        interaction: &mut crate::ui::Interaction,
        tree: &mut crate::ui::Tree,
    ) {
        let key = tree.nodes[node].key;
        let Some(board) = self.host.rive.get(id).cloned() else {
            out.warnings.push(format!(
                "{path}: og-rive src=\"{id}\" names no Rive artboard the UI has; og.load.document loads those its document names"
            ));
            return;
        };
        let tick = interaction.state.tick;
        let dt = 1.0 / env.ticks_per_second.max(1) as f32;
        let player = match interaction.state.rive.remove(&key) {
            Some(crate::ui::RiveSlot {
                board: b,
                player: Some(mut p),
                ..
            }) if b == id => {
                if !Arc::ptr_eq(&p.artboard, &board) {
                    // A hot reload: on from where it was, when that fits.
                    let mut q = crate::rive::Player::new(board.clone(), None);
                    p = if q.restore(&p.save()).is_ok() {
                        q
                    } else {
                        crate::rive::Player::new(board, None)
                    };
                }
                p.advance(dt);
                p
            }
            Some(crate::ui::RiveSlot {
                board: b,
                player: None,
                saved,
                ..
            }) if b == id => {
                let mut p = crate::rive::Player::new(board, None);
                if let Err(e) = p.restore(&saved) {
                    out.warnings.push(format!(
                        "{path}: the saved state of Rive artboard {id} does not fit it ({e}); it is kept and not drawn"
                    ));
                    interaction.state.rive.insert(
                        key,
                        crate::ui::RiveSlot {
                            board: b,
                            player: None,
                            saved,
                            built: tick,
                        },
                    );
                    return;
                }
                p.advance(dt);
                p
            }
            _ => crate::rive::Player::new(board, None),
        };
        tree.draws.insert(
            node,
            crate::paint::ElementDraw {
                rive: Some(crate::paint::RiveDraw(player.clone())),
                ..crate::paint::ElementDraw::default()
            },
        );
        tree.dressable.remove(&node);
        interaction.state.rive.insert(
            key,
            crate::ui::RiveSlot {
                board: id.to_owned(),
                player: Some(player),
                saved: Vec::new(),
                built: tick,
            },
        );
    }
}

/// What the stages ask of the documents (docs/ui.md §7): the document
/// build installs it ([`install_documents`]) as the first document is
/// defined. A UI that has defined no document has no screen or mount to
/// build or restyle, so until then the stages go on without it.
pub trait Documents: Sync {
    /// Builds the layer stack and the mounted documents once a tick, where
    /// no host built them before the frame ends ([`Ui::build_layers`]).
    fn build(&self, ui: &mut Ui);
    /// Restyles the mounted elements that read the query containers in
    /// `changed`, answering their layout nodes (settlement).
    fn requery(&self, ui: &mut Ui, changed: &BTreeSet<u64>) -> Result<Vec<usize>, String>;
}

static DOCUMENTS: std::sync::OnceLock<&'static dyn Documents> = std::sync::OnceLock::new();

/// Installs the documents the stages call; the first one installed stays.
pub fn install_documents(documents: &'static dyn Documents) {
    DOCUMENTS.get_or_init(|| documents);
}

/// The installed documents, if a document was ever defined.
pub(crate) fn documents() -> Option<&'static dyn Documents> {
    DOCUMENTS.get().copied()
}
