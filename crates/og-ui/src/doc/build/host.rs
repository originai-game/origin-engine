//! The documents' state the stages read: the documents a UI knows, what
//! their mounts keep between ticks, and what a build tells code and the probe.

use super::*;

/// A text field's constraint validity as last built ([`Act::Submit`]).
#[derive(Clone, Debug)]
pub(crate) struct FieldValidity {
    pub(crate) mount: String,
    pub(crate) doc: String,
    pub(crate) node: usize,
    pub(crate) row_key: String,
    pub(crate) invalid: bool,
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
pub(crate) struct DocRecord {
    pub built: Vec<Built>,
    pub render_fields: Vec<(u64, super::fields::RenderFields)>,
    pub arts: Vec<(String, crate::slot::SlotSpec)>,
}

/// The documents a UI knows and what their mounts keep between ticks.
/// A view transition's covers still opening after their layer left: the
/// covers, their colour and image, and the tick it left.
pub(crate) type Revealing = (
    Vec<crate::motion::css::CssAnimation>,
    Option<crate::Srgba>,
    Option<String>,
    u64,
);

#[derive(Clone, Debug, Default)]
pub struct DocHost {
    /// The records of the retained elements being emitted, innermost last.
    pub(crate) capture: Vec<DocRecord>,
    pub(crate) retained: super::instances::RetainedHost,
    /// By name.
    pub(crate) docs: BTreeMap<String, Arc<UiDocument>>,
    pub(crate) edits: Vec<Edit>,
    /// A trigger's last value (`on change`), by element key and trigger.
    pub(crate) last: BTreeMap<(u64, usize), Value>,
    /// The states triggers set (`on click state selection on`), by element key and group.
    pub(crate) states: BTreeMap<(u64, String), String>,
    /// Document elements seen last tick (for `on appear`).
    pub(crate) seen: BTreeSet<u64>,
    pub(crate) seen_now: BTreeSet<u64>,
    /// Immediately built elements with `use:` actions, by key: who they
    /// are and their actions' names, for their `destroy` once not built.
    pub(crate) uses: BTreeMap<u64, (DocEvent, Vec<String>)>,
    /// Text fields' constraint validity as last built, by key: what a
    /// form's submit reads. Kept while their element is.
    pub(crate) validity: BTreeMap<u64, FieldValidity>,
    /// What media elements asked of their players this frame, taken by
    /// the host ([`Ui::take_media_ops`]).
    pub(crate) media_ops: Vec<crate::bind::MediaOp>,
    /// What each media element last sent, the time it last read and the
    /// frames a seek it sent is still on its way, by key.
    pub(crate) media_sent: BTreeMap<u64, (f32, bool, f64, u8)>,
    /// `data-og-instant` elements whose first entrance has landed: a
    /// later one plays (presentation, outside `world_hash`).
    pub(crate) entered: BTreeSet<u64>,
    /// What happened to the element being built beyond its response
    /// (`ontoggle`, `onclose`), taken by its triggers.
    pub(crate) happened: Vec<crate::doc::When>,
    /// The popovers documents name, by their key in their mount.
    pub(crate) popovers: BTreeMap<u64, PopoverLink>,
    /// Some defined document writes a CSS-wide keyword on a property, so
    /// elements carry their winners to their children's `inherit`.
    pub(crate) wide_keywords: bool,
    /// The id the last `#fragment` link named: CSS `:target`.
    pub(crate) target: Option<String>,
    /// The `$ui` store's value as last published.
    pub(crate) ui_store: Option<Value>,
    /// This tick's builds, slots and art slots.
    pub(crate) built: Vec<Built>,
    pub(crate) slots: BTreeMap<String, usize>,
    pub(crate) arts: Vec<(String, crate::slot::SlotSpec)>,
    pub(crate) render_fields: BTreeMap<u64, super::fields::RenderFields>,
    /// Events queued for code (screens built at the end of a tick reach
    /// code the next).
    pub(crate) queued: Vec<DocEvent>,
    /// Layer operations triggers asked for this tick, done after the build.
    pub(crate) ops: Vec<crate::layer_stack::LayerOp>,
    /// Plays a document or the probe started: `(path, timeline)` → play.
    pub(crate) plays: BTreeMap<(String, String), u32>,
    /// The tick the layers were last built.
    pub(crate) built_tick: u64,
    pub(crate) layers: crate::layer_stack::LayerStack,
    /// The mount being built is the top layer that takes input (its `on action` triggers fire).
    pub(crate) accepts: bool,
    /// The serial of the layer being built (0 outside one): what its `pop` pops.
    pub(crate) building: u32,
    /// The retained element being built holds its node: hidden
    /// (`display: none`, `hidden`), it keeps the node and its subtree, as
    /// the web keeps a hidden element's DOM, and showing it again is a field
    /// change. One never built is not built while hidden.
    pub(crate) keeps_hidden: bool,
    /// How deep the element images being built are nested (an `feImage`
    /// of an element whose filters draw element images): past 4, no more.
    pub(crate) element_images: u32,
    /// View-transition covers still opening after their layer left: the
    /// covers, their colour and image and the tick it left.
    pub(crate) revealing: Vec<Revealing>,
    /// The textures documents name by asset path (a view transition's
    /// cover image), by path.
    pub(crate) textures: BTreeMap<String, og_draw::TextureHandle>,
    /// The alpha of the textures a `shape-outside: url()` names, by path
    /// ([`Ui::define_texture_alpha`]).
    pub(crate) alphas: BTreeMap<String, Arc<crate::layout::float::ImageAlpha>>,
    /// The on-screen controls documents lay out (`<joystick action>`,
    /// `<dpad action>`, `<button data-og-touch>`), by render key, their
    /// rects filled from the frame's hits; `zones_changed`: og-input's
    /// copy is stale ([`Ui::take_control_zones`]).
    pub(crate) zones: BTreeMap<u64, og_input::ControlZone>,
    pub(crate) zones_changed: bool,
    /// The Rive artboards elements can play, by id ([`Ui::define_rive`]).
    pub(crate) rive: BTreeMap<String, Arc<crate::rive::Artboard>>,
    /// The `type ui-material` assets documents name (`-og-material:
    /// "<path>" …`): each one's handle and its parameters' defaults.
    pub(crate) materials: BTreeMap<
        String,
        (
            og_draw::UiMaterialHandle,
            [[f32; 4]; og_draw::UI_MATERIAL_PARAMS],
        ),
    >,
    /// CSS `@keyframes` of every defined document, as `<doc>/<name>`.
    pub(crate) keyframes: BTreeMap<String, Arc<crate::motion::css::Keyframes>>,
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
    pub(crate) scroll_states: BTreeMap<u64, (og_core::glam::Vec2, u16)>,
    pub(crate) scroll_now: BTreeSet<u64>,
    pub(crate) scroll_changed: BTreeSet<u64>,
    /// The document-order counter and quote run of the tick being built,
    /// kept only while some defined document uses generated content.
    pub(crate) run: crate::doc::generated::Run,
    pub(crate) uses_run: bool,
    /// The `::placeholder` color of the text control being built.
    pub(crate) placeholder_ink: Option<Srgba>,
    /// `::selection` styles by render key.
    pub(crate) selection: BTreeMap<u64, Arc<crate::doc::generated::Selection>>,
    /// What structural selectors of every defined document read.
    pub(crate) structure: crate::doc::matching::Profile,
    /// Global rules can read facts in a different document from their subject.
    pub(crate) global_structure: crate::doc::matching::Profile,
    /// The instantiated sibling levels of the screen being built.
    pub(crate) levels: Vec<super::structure::Level>,
    /// A component's root continues its instance's place among siblings.
    pub(crate) component_root: bool,
}

/// What the controls naming a popover last did to it, by its key
/// ([`popover_key`]): presentation, rebuilt from the controls as they build.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PopoverLink {
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
pub(crate) enum Mode {
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
    pub(crate) fn css_material(
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

    pub(crate) fn last_art_entry(&self) -> &(String, crate::slot::SlotSpec) {
        match self.host.capture.last() {
            Some(record) => record.arts.last(),
            None => self.host.arts.last(),
        }
        .expect("an art slot was registered")
    }

    /// The art slot `path`'s spec, where it was registered.
    pub(crate) fn art_slot_mut(&mut self, path: &str) -> Option<&mut crate::slot::SlotSpec> {
        let key = crate::id::key_of_path(path);
        let docs = &mut self.host;
        if let Some((_, spec)) = docs
            .capture
            .iter_mut()
            .rev()
            .flat_map(|r| r.arts.iter_mut())
            .find(|(p, _)| p == path)
        {
            return Some(spec);
        }
        if let Some((_, spec)) = docs.arts.iter_mut().find(|(p, _)| p == path) {
            return Some(spec);
        }
        docs.retained
            .record_mut(key)
            .and_then(|r| r.arts.iter_mut().find(|(p, _)| p == path))
            .map(|(_, spec)| spec)
    }
}

impl Ui {
    /// The tick's document bookkeeping starts (from `begin`).
    pub(crate) fn docs_begin(&mut self) {
        let d = &mut self.docs.host;
        d.run.clear();
        d.seen = std::mem::take(&mut d.seen_now);
        let gone: Vec<u64> = d
            .uses
            .keys()
            .filter(|k| !d.seen.contains(k))
            .copied()
            .collect();
        for key in gone {
            let (event, names) = d.uses.remove(&key).unwrap();
            for (k, name) in names.into_iter().enumerate() {
                d.last.remove(&(key, usize::MAX - k));
                d.queued.push(DocEvent {
                    kind: "use".into(),
                    name,
                    value: json!({ "phase": "destroy", "params": Value::Null }),
                    ..event.clone()
                });
            }
        }
        d.built.clear();
        d.slots.clear();
        d.arts.clear();
        d.render_fields.clear();
        d.retained.evaluated = 0;
        d.retained.resolved = 0;
        d.retained.reconciled = 0;
        d.queried.clear();
        // Last tick's containers, immediate and mounted, stay watched.
        let mut live = std::mem::take(&mut d.containers_now);
        live.extend(d.retained.containers());
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
    pub(crate) fn play_rive(
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
    fn requery(
        &self,
        host: &mut RetainedHost,
        ui: &mut Ui,
        changed: &BTreeSet<u64>,
    ) -> Result<Vec<usize>, String>;
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
