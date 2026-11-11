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
    pub(crate) retained: super::retained::RetainedHost,
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
    pub(crate) popovers: BTreeMap<u64, super::popover::PopoverLink>,
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
    pub(super) render_fields: BTreeMap<u64, super::fields::RenderFields>,
    /// Events queued for code (screens built at the end of a tick reach
    /// code the next).
    pub(crate) queued: Vec<DocEvent>,
    /// Layer operations triggers asked for this tick, done after the build.
    pub(crate) ops: Vec<crate::layers::LayerOp>,
    /// Plays a document or the probe started: `(path, timeline)` → play.
    pub(crate) plays: BTreeMap<(String, String), u32>,
    /// The tick the layers were last built.
    pub(crate) built_tick: u64,
    pub(crate) layers: crate::layers::LayerStack,
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
    pub(crate) queried: Vec<super::containers::Queried>,
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

impl Ui {
    /// Registers a document (its timelines as `<name>/<timeline>`); one of
    /// the same name is replaced, the UI's state kept (a hot reload).
    pub fn define_document(&mut self, doc: UiDocument) {
        for (n, t) in &doc.timelines {
            self.define_timeline(&format!("{}/{n}", doc.name), t.clone());
        }
        self.docs.host.uses_run |= doc.nodes.iter().any(|n| {
            n.rules.iter().any(|rule| {
                rule.declarations
                    .iter()
                    .any(|d| crate::doc::generated::longhand(&d.name))
            })
        });
        let prefix = format!("{}/", doc.name);
        self.docs
            .host
            .keyframes
            .retain(|name, _| !name.starts_with(&prefix));
        for (n, k) in &doc.sheet.keyframes {
            self.docs
                .host
                .keyframes
                .insert(format!("{prefix}{n}"), Arc::new(k.clone()));
        }
        self.docs
            .host
            .structure
            .merge(&crate::doc::matching::Profile::of_document(&doc));
        for selector in doc
            .nodes
            .iter()
            .flat_map(|n| &n.rules)
            .filter_map(|r| r.structure.as_ref())
            .filter(|s| s.global)
        {
            self.docs.host.global_structure.add(selector);
        }
        self.docs.host.wide_keywords |= doc.nodes.iter().any(|n| {
            n.rules.iter().flat_map(|r| &r.declarations).any(|d| {
                d.value.binding().is_some()
                    || super::cascade::CARRIED_BY_DEFAULT.contains(&d.name.as_str())
                    || !d.name.starts_with("--")
                        && matches!(
                            crate::doc::stylesheet::wide_keyword(&d.source),
                            Some("inherit" | "unset" | "revert" | "revert-layer")
                        )
            })
        });
        let mut retained = std::mem::take(&mut self.docs.host.retained);
        retained.document_replaced(self, &doc.name);
        self.docs.host.retained = retained;
        self.docs.host.docs.insert(doc.name.clone(), Arc::new(doc));
    }

    /// Registers a Rive artboard by its id (`ui/chest.riv#Chest`, as og-cook
    /// converted it, docs/ui.md §3.3): `<og-rive src="id">` plays it.
    /// One of the same id is replaced (a hot
    /// reload): an element playing the old one goes on from where it was
    /// when it fits the new one, else starts it again.
    pub fn define_rive(&mut self, id: &str, artboard: Arc<crate::rive::Artboard>) {
        self.docs.host.rive.insert(id.to_owned(), artboard);
    }

    /// Binds the texture asset `path` a document names
    /// ([`UiDocument::cover_image`]) to its loaded handle.
    pub fn define_texture(&mut self, path: &str, texture: og_draw::TextureHandle) {
        self.docs.host.textures.insert(path.to_owned(), texture);
    }

    /// Binds the UI material asset `path` a document's `-og-material`
    /// names ([`UiDocument::material_assets`]) to its loaded handle and
    /// its parameters' defaults.
    pub fn define_material(
        &mut self,
        path: &str,
        handle: og_draw::UiMaterialHandle,
        defaults: [[f32; 4]; og_draw::UI_MATERIAL_PARAMS],
    ) {
        self.docs
            .host
            .materials
            .insert(path.to_owned(), (handle, defaults));
    }

    /// A document by name.
    pub fn document(&self, name: &str) -> Option<&UiDocument> {
        self.docs.host.docs.get(name).map(|d| &**d)
    }

    /// Every document's name.
    pub fn documents(&self) -> Vec<String> {
        self.docs.host.docs.keys().cloned().collect()
    }

    /// Mounts a component here: `props` is its viewmodel (writes of its
    /// two-way bindings land in it), `id` its root's id.
    pub fn component(
        &mut self,
        id: &str,
        name: &str,
        props: &mut Value,
        env: &dyn Env,
    ) -> DocResponse {
        let Some(doc) = self.docs.host.docs.get(name).cloned() else {
            self.out.warnings.push(format!(
                "Ui::component: no document named '{name}' (load its .ogt with og.load.document)"
            ));
            return DocResponse::default();
        };
        let mount = format!("{}/{id}", self.tree.path_of(self.tree.parent()));
        // An immediate document reads `$ui` from its viewmodel.
        if let Some(vm) = props.as_object_mut() {
            vm.insert("$ui".into(), self.ui_store_value());
        }
        let game = self.game_input();
        let accepts = std::mem::replace(&mut self.docs.host.accepts, game);
        let r = self.mount_doc(doc, mount, id, props, env, Arc::default());
        self.docs.host.accepts = accepts;
        r
    }

    /// Builds `doc` here with root id `id`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn mount_doc(
        &mut self,
        doc: Arc<UiDocument>,
        mount: String,
        id: &str,
        vm: &mut Value,
        env: &dyn Env,
        inherited: Arc<super::super::variables::Computed>,
    ) -> DocResponse {
        let root = doc.root;
        let mut b = Build {
            doc: doc.clone(),
            mount: mount.clone(),
            vm,
            env,
            locals: Vec::new(),
            row_key: String::new(),
            inherited,
            events: Vec::new(),
            writes: Vec::new(),
            cached: None,
            recording: Default::default(),
        };
        let path = self.build_node(&mut b, root, Some(id));
        let (events, writes) = (b.events, b.writes);
        for (p, v) in writes {
            set_path(vm, &p, v);
        }
        DocResponse {
            events,
            path: path.unwrap_or_default(),
        }
    }

    /// Opens a document's slot so code fills it (its children follow until
    /// [`Ui::close`]): a full path, or `<mount>/<slot id>`. False when no
    /// such slot was built this tick.
    pub fn open_slot(&mut self, path: &str) -> bool {
        match self.docs.host.slots.get(path) {
            Some(&i) if i < self.tree.nodes.len() => {
                self.tree.stack.push(i);
                true
            }
            _ => {
                self.out.warnings.push(format!(
                    "Ui::open_slot: no slot '{path}' this tick (mount the document first; slots: {})",
                    self.docs.host.slots.keys().cloned().collect::<Vec<_>>().join(", ")
                ));
                false
            }
        }
    }

    /// Events of documents built where code did not see them (screens
    /// built at the end of a tick), taken.
    /// The play, pause, seek, gain and loop requests documents' `<video>`
    /// and `<audio>` made this frame, for the host to apply to their
    /// players at the next tick.
    pub fn take_media_ops(&mut self) -> Vec<crate::bind::MediaOp> {
        std::mem::take(&mut self.docs.host.media_ops)
    }

    pub fn take_doc_events(&mut self) -> Vec<DocEvent> {
        std::mem::take(&mut self.docs.host.queued)
    }

    /// What documents built this tick (the probe's `dump_ui`): the
    /// immediate documents' elements, then the mounted ones' in tree order.
    pub fn doc_built(&self) -> Vec<Built> {
        let mut out = self.docs.host.built.clone();
        out.extend(
            self.docs
                .host
                .retained
                .records()
                .flat_map(|r| r.built.iter().cloned()),
        );
        out
    }

    /// The art slots documents declared this tick, by path.
    pub fn art_slots(&self) -> Vec<(String, crate::slot::SlotSpec)> {
        let mut out = self.docs.host.arts.clone();
        out.extend(
            self.docs
                .host
                .retained
                .records()
                .flat_map(|r| r.arts.iter().cloned()),
        );
        out
    }

    /// The probe's edits in effect.
    pub fn doc_edits(&self) -> &[Edit] {
        &self.docs.host.edits
    }

    /// `ui:set`: sets a prop of the element at `path` (presentation only:
    /// it applies over the document's own line until cleared or saved).
    pub fn set_doc_prop(&mut self, path: &str, prop: &str, value: &str) -> Result<(), String> {
        let (spec, v) = parse_probe_value(prop, value)?;
        self.docs
            .host
            .edits
            .retain(|e| !(e.path == path && e.prop == prop));
        self.docs.host.edits.push(Edit {
            path: path.to_string(),
            prop: prop.to_string(),
            spec,
            value: v,
            source: value.to_string(),
        });
        Ok(())
    }

    /// Drops the probe's edits of `path` (all of them: empty).
    pub fn clear_doc_edits(&mut self, path: &str) {
        self.docs
            .host
            .edits
            .retain(|e| !path.is_empty() && e.path != path);
    }

    /// The document element built at `path` last tick: its document's name
    /// and node.
    pub fn doc_element(&self, path: &str) -> Option<(&str, usize)> {
        self.docs
            .host
            .built
            .iter()
            .find(|b| b.path == path)
            .map(|b| (b.doc.as_str(), b.node))
    }

    /// Writes the probe's edits into their documents' text (`ui:save`):
    /// each edited element's line of that prop is replaced, or one added.
    /// Answers the documents changed, each with its source and new text;
    /// the edits written are dropped (the documents now say it).
    pub fn save_doc_edits(&mut self) -> Result<Vec<(String, String, String)>, String> {
        let edits = self.docs.host.edits.clone();
        let mut changed: BTreeMap<String, UiDocument> = BTreeMap::new();
        let mut kept = Vec::new();
        for e in edits {
            let Some((doc, node)) = self.doc_element(&e.path).map(|(d, n)| (d.to_string(), n))
            else {
                kept.push(e);
                continue;
            };
            let d = match changed.get_mut(&doc) {
                Some(d) => d,
                None => {
                    let d = (**self.docs.host.docs.get(&doc).ok_or("no document")?).clone();
                    changed.entry(doc.clone()).or_insert(d)
                }
            };
            d.set_prop(node, &e.prop, &e.source)
                .map_err(|why| format!("{}: {}.{}: {why}", d.source, e.path, e.prop))?;
        }
        self.docs.host.edits = kept;
        let mut out = Vec::new();
        for (name, d) in changed {
            out.push((name, d.source.clone(), d.text.clone()));
            self.define_document(d);
        }
        Ok(out)
    }

    /// Plays a document timeline on the element at `path` (`ui:play`): a
    /// name of the element's document, or `<doc>/<name>`.
    pub fn play_doc_timeline(
        &mut self,
        path: &str,
        timeline: &str,
        from_ms: f32,
        paused: bool,
    ) -> Result<u32, String> {
        let full = self.timeline_name(path, timeline)?;
        let id = self.play_timeline(
            path,
            &full,
            crate::motion::PlayDesc {
                from_ms,
                hold: true,
                ..Default::default()
            },
        );
        if paused {
            self.pause(id);
        }
        self.docs
            .host
            .plays
            .insert((path.to_string(), timeline.to_string()), id);
        Ok(id)
    }

    /// `ui:seek`: a timeline playing on the element to `ms` (started there,
    /// paused, when it is not playing).
    pub fn seek_doc_timeline(
        &mut self,
        path: &str,
        timeline: &str,
        ms: f32,
    ) -> Result<u32, String> {
        if let Some(&id) = self
            .docs
            .host
            .plays
            .get(&(path.to_string(), timeline.to_string()))
            && self.seek(id, ms)
        {
            self.pause(id);
            return Ok(id);
        }
        self.play_doc_timeline(path, timeline, ms, true)
    }

    fn timeline_name(&self, path: &str, timeline: &str) -> Result<String, String> {
        if timeline.contains('/') {
            return if self.timeline(timeline).is_some() {
                Ok(timeline.to_string())
            } else {
                Err(format!("no timeline '{timeline}'"))
            };
        }
        let doc = self
            .doc_element(path)
            .map(|(d, _)| d.to_string())
            .or_else(|| {
                // A path below a mount: the nearest built ancestor's document.
                self.docs
                    .host
                    .built
                    .iter()
                    .filter(|b| path.starts_with(&format!("{}/", b.path)))
                    .max_by_key(|b| b.path.len())
                    .map(|b| b.doc.clone())
            });
        match doc {
            Some(d) if self.timeline(&format!("{d}/{timeline}")).is_some() => {
                Ok(format!("{d}/{timeline}"))
            }
            _ if self.timeline(timeline).is_some() => Ok(timeline.to_string()),
            Some(d) => Err(format!(
                "the document '{d}' declares no timeline '{timeline}' ({})",
                self.document(&d)
                    .map(|x| x
                        .timelines
                        .iter()
                        .map(|t| t.0.clone())
                        .collect::<Vec<_>>()
                        .join(", "))
                    .unwrap_or_default()
            )),
            None => Err(format!(
                "no document element at '{path}' and no timeline '{timeline}'"
            )),
        }
    }

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
        self.sync_scroll_states();
    }
}

impl Ui {
    /// A document's `-og-material` as the layer it draws: a built-in's, or
    /// a defined asset's with its defaults under the numbers written.
    pub(crate) fn css_material(
        &self,
        m: crate::dress::DressMaterial,
    ) -> Result<crate::paint::Layer, String> {
        let Some(path) = m.asset else {
            return Ok(m.layer);
        };
        let (handle, mut params) = *self.docs.host.materials.get(&path).ok_or_else(|| {
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

    /// Plays the Rive artboard `id` in element `node` (its key `key`) this
    /// tick: the element's player goes on one tick, a restored one is
    /// rebuilt from its bytes, a new one starts; the element draws it over
    /// its background ([`crate::paint::ElementDraw::rive`]).
    pub(crate) fn play_rive(&mut self, node: usize, id: &str, path: &str) {
        let key = self.tree.nodes[node].key;
        let Some(board) = self.docs.host.rive.get(id).cloned() else {
            self.out.warnings.push(format!(
                "{path}: og-rive src=\"{id}\" names no Rive artboard the UI has; og.load.document loads those its document names"
            ));
            return;
        };
        let tick = self.interaction.state.tick;
        let dt = 1.0 / self.env.ticks_per_second.max(1) as f32;
        let player = match self.interaction.state.rive.remove(&key) {
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
                    self.out.warnings.push(format!(
                        "{path}: the saved state of Rive artboard {id} does not fit it ({e}); it is kept and not drawn"
                    ));
                    self.interaction.state.rive.insert(
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
        self.tree.draws.insert(
            node,
            crate::paint::ElementDraw {
                rive: Some(crate::paint::RiveDraw(player.clone())),
                ..crate::paint::ElementDraw::default()
            },
        );
        self.tree.dressable.remove(&node);
        self.interaction.state.rive.insert(
            key,
            crate::ui::RiveSlot {
                board: id.to_owned(),
                player: Some(player),
                saved: Vec::new(),
                built: tick,
            },
        );
    }

    pub(crate) fn note_built(&mut self, built: Built) {
        match self.docs.host.capture.last_mut() {
            Some(record) => record.built.push(built),
            None => self.docs.host.built.push(built),
        }
    }

    pub(crate) fn note_render_fields(&mut self, key: u64, fields: super::fields::RenderFields) {
        match self.docs.host.capture.last_mut() {
            Some(record) => record.render_fields.push((key, fields)),
            None => {
                self.docs.host.render_fields.insert(key, fields);
            }
        }
    }

    pub(crate) fn note_art(&mut self, path: String, spec: crate::slot::SlotSpec) {
        match self.docs.host.capture.last_mut() {
            Some(record) => record.arts.push((path, spec)),
            None => self.docs.host.arts.push((path, spec)),
        }
    }

    /// The art slot registered last.
    pub(crate) fn last_art(&self) -> &crate::slot::SlotSpec {
        &self.last_art_entry().1
    }

    pub(crate) fn last_art_path(&self) -> String {
        self.last_art_entry().0.clone()
    }

    fn last_art_entry(&self) -> &(String, crate::slot::SlotSpec) {
        match self.docs.host.capture.last() {
            Some(record) => record.arts.last(),
            None => self.docs.host.arts.last(),
        }
        .expect("an art slot was registered")
    }

    /// The art slot `path`'s spec, where it was registered.
    pub(crate) fn art_slot_mut(&mut self, path: &str) -> Option<&mut crate::slot::SlotSpec> {
        let key = crate::id::key_of_path(path);
        let docs = &mut self.docs.host;
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

    /// The render fields element `key` was emitted with.
    #[cfg(test)]
    pub(crate) fn render_fields(&self, key: u64) -> Option<&super::fields::RenderFields> {
        self.docs.host.render_fields.get(&key).or_else(|| {
            self.docs
                .host
                .retained
                .record(key)
                .and_then(|r| r.render_fields.iter().find(|(k, _)| *k == key))
                .map(|(_, fields)| fields)
        })
    }
}
