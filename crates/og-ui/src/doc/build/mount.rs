use super::*;

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
    /// View-transition covers still opening after their layer left: the
    /// covers, their colour and the tick it left.
    pub(crate) revealing: Vec<(
        Vec<crate::motion::css::CssAnimation>,
        Option<crate::Srgba>,
        u64,
    )>,
    /// The Rive artboards elements can play, by id ([`Ui::define_rive`]).
    pub(crate) rive: BTreeMap<String, Arc<crate::rive::Artboard>>,
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
    /// What structural selectors of every defined document read.
    pub(crate) structure: crate::doc::matching::Profile,
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
        let prefix = format!("{}/", doc.name);
        self.docs
            .keyframes
            .retain(|name, _| !name.starts_with(&prefix));
        for (n, k) in &doc.keyframes {
            self.docs
                .keyframes
                .insert(format!("{prefix}{n}"), Arc::new(k.clone()));
        }
        self.docs
            .structure
            .merge(&crate::doc::matching::Profile::of_document(&doc));
        let mut retained = std::mem::take(&mut self.docs.retained);
        retained.document_replaced(self, &doc.name);
        self.docs.retained = retained;
        self.docs.docs.insert(doc.name.clone(), Arc::new(doc));
    }

    /// Registers a Rive artboard by its id (`ui/chest.riv#Chest`, as og-cook
    /// converted it, docs/ui.md §3.3): an element whose `-og-frame` is
    /// `rive("<id>")` plays it. One of the same id is replaced (a hot
    /// reload): an element playing the old one goes on from where it was
    /// when it fits the new one, else starts it again.
    pub fn define_rive(&mut self, id: &str, artboard: Arc<crate::rive::Artboard>) {
        self.docs.rive.insert(id.to_owned(), artboard);
    }

    /// Plays the Rive artboard `id` in element `node` (its key `key`) this
    /// tick: the element's player goes on one tick, a restored one is
    /// rebuilt from its bytes, a new one starts; the element draws it over
    /// its background ([`crate::paint::ElementDraw::rive`]).
    pub(crate) fn play_rive(&mut self, node: usize, id: &str, path: &str) {
        let key = self.nodes[node].key;
        let Some(board) = self.docs.rive.get(id).cloned() else {
            self.warnings.push(format!(
                "{path}: -og-frame: rive(\"{id}\") names no Rive artboard the UI has; og.load.document loads those its document names"
            ));
            return;
        };
        let tick = self.state.tick;
        let dt = 1.0 / self.ticks_per_second.max(1) as f32;
        let player = match self.state.rive.remove(&key) {
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
                    self.warnings.push(format!(
                        "{path}: the saved state of Rive artboard {id} does not fit it ({e}); it is kept and not drawn"
                    ));
                    self.state.rive.insert(
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
        self.draws.insert(
            node,
            crate::paint::ElementDraw {
                rive: Some(crate::paint::RiveDraw(player.clone())),
                ..crate::paint::ElementDraw::default()
            },
        );
        self.dressable.remove(&node);
        self.state.rive.insert(
            key,
            crate::ui::RiveSlot {
                board: id.to_owned(),
                player: Some(player),
                saved: Vec::new(),
                built: tick,
            },
        );
    }

    /// A document by name.
    pub fn document(&self, name: &str) -> Option<&UiDocument> {
        self.docs.docs.get(name).map(|d| &**d)
    }

    /// Every document's name.
    pub fn documents(&self) -> Vec<String> {
        self.docs.docs.keys().cloned().collect()
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
        let Some(doc) = self.docs.docs.get(name).cloned() else {
            self.warnings.push(format!(
                "og.ui.component: no document named '{name}' (load its .ogt with og.load.document)"
            ));
            return DocResponse::default();
        };
        let mount = format!("{}/{id}", self.path_of(self.parent()));
        let game = self.game_input();
        let accepts = std::mem::replace(&mut self.docs.accepts, game);
        let r = self.mount_doc(doc, mount, id, props, env, Arc::default());
        self.docs.accepts = accepts;
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
        match self.docs.slots.get(path) {
            Some(&i) if i < self.nodes.len() => {
                self.stack.push(i);
                true
            }
            _ => {
                self.warnings.push(format!(
                    "og.ui.openSlot: no slot '{path}' this tick (mount the document first; slots: {})",
                    self.docs.slots.keys().cloned().collect::<Vec<_>>().join(", ")
                ));
                false
            }
        }
    }

    /// Events of documents built where code did not see them (screens
    /// built at the end of a tick), taken.
    pub fn take_doc_events(&mut self) -> Vec<DocEvent> {
        std::mem::take(&mut self.docs.queued)
    }

    /// What documents built this tick (the probe's `dump_ui`): the
    /// immediate documents' elements, then the mounted ones' in tree order.
    pub fn doc_built(&self) -> Vec<Built> {
        let mut out = self.docs.built.clone();
        out.extend(
            self.docs
                .retained
                .records()
                .flat_map(|r| r.built.iter().cloned()),
        );
        out
    }

    /// The art slots documents declared this tick, by path.
    pub fn art_slots(&self) -> Vec<(String, crate::slot::SlotSpec)> {
        let mut out = self.docs.arts.clone();
        out.extend(
            self.docs
                .retained
                .records()
                .flat_map(|r| r.arts.iter().cloned()),
        );
        out
    }

    pub(crate) fn note_built(&mut self, built: Built) {
        match self.docs.capture.last_mut() {
            Some(record) => record.built.push(built),
            None => self.docs.built.push(built),
        }
    }

    pub(crate) fn note_render_fields(&mut self, key: u64, fields: super::fields::RenderFields) {
        match self.docs.capture.last_mut() {
            Some(record) => record.render_fields.push((key, fields)),
            None => {
                self.docs.render_fields.insert(key, fields);
            }
        }
    }

    pub(crate) fn note_art(&mut self, path: String, spec: crate::slot::SlotSpec) {
        match self.docs.capture.last_mut() {
            Some(record) => record.arts.push((path, spec)),
            None => self.docs.arts.push((path, spec)),
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
        match self.docs.capture.last() {
            Some(record) => record.arts.last(),
            None => self.docs.arts.last(),
        }
        .expect("an art slot was registered")
    }

    /// The art slot `path`'s spec, where it was registered.
    pub(crate) fn art_slot_mut(&mut self, path: &str) -> Option<&mut crate::slot::SlotSpec> {
        let key = crate::id::key_of_path(path);
        let docs = &mut self.docs;
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
        self.docs.render_fields.get(&key).or_else(|| {
            self.docs
                .retained
                .record(key)
                .and_then(|r| r.render_fields.iter().find(|(k, _)| *k == key))
                .map(|(_, fields)| fields)
        })
    }

    /// The probe's edits in effect.
    pub fn doc_edits(&self) -> &[Edit] {
        &self.docs.edits
    }

    /// `ui:set`: sets a prop of the element at `path` (presentation only:
    /// it applies over the document's own line until cleared or saved).
    pub fn set_doc_prop(&mut self, path: &str, prop: &str, value: &str) -> Result<(), String> {
        let (spec, v) = parse_probe_value(prop, value)?;
        self.docs
            .edits
            .retain(|e| !(e.path == path && e.prop == prop));
        self.docs.edits.push(Edit {
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
            .edits
            .retain(|e| !path.is_empty() && e.path != path);
    }

    /// The document element built at `path` last tick: its document's name
    /// and node.
    pub fn doc_element(&self, path: &str) -> Option<(&str, usize)> {
        self.docs
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
        let edits = self.docs.edits.clone();
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
                    let d = (**self.docs.docs.get(&doc).ok_or("no document")?).clone();
                    changed.entry(doc.clone()).or_insert(d)
                }
            };
            d.set_prop(node, &e.prop, &e.source)
                .map_err(|why| format!("{}: {}.{}: {why}", d.source, e.path, e.prop))?;
        }
        self.docs.edits = kept;
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
        let d = &mut self.docs;
        d.seen = std::mem::take(&mut d.seen_now);
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
    }
}
