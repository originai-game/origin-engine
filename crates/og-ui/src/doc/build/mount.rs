use super::*;
use crate::doc::parse_probe_value;
use crate::reactive::{SourceId, session};

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

    /// Gives the texture asset `path` its alpha channel on the CPU, which
    /// a float's `shape-outside: url(<path>)` takes its contour from
    /// ([`UiDocument::shape_images`]); without it the float has no shape.
    pub fn define_texture_alpha(&mut self, path: &str, alpha: crate::ImageAlpha) {
        self.docs
            .host
            .alphas
            .insert(path.to_owned(), Arc::new(alpha));
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

    pub(crate) fn note_built(&mut self, built: Built) {
        match self.host.capture.last_mut() {
            Some(record) => record.built.push(built),
            None => self.host.built.push(built),
        }
    }

    pub(crate) fn note_render_fields(&mut self, key: u64, fields: super::fields::RenderFields) {
        match self.host.capture.last_mut() {
            Some(record) => record.render_fields.push((key, fields)),
            None => {
                self.host.render_fields.insert(key, fields);
            }
        }
    }

    pub(crate) fn note_art(&mut self, path: String, spec: crate::slot::SlotSpec) {
        match self.host.capture.last_mut() {
            Some(record) => record.arts.push((path, spec)),
            None => self.host.arts.push((path, spec)),
        }
    }

    /// The art slot registered last.
    pub(crate) fn last_art(&self) -> &crate::slot::SlotSpec {
        &self.last_art_entry().1
    }

    /// The render fields element `key` was emitted with.
    #[cfg(test)]
    pub(crate) fn render_fields(&self, key: u64) -> Option<&super::fields::RenderFields> {
        self.host.render_fields.get(&key).or_else(|| {
            self.host
                .retained
                .record(key)
                .and_then(|r| r.render_fields.iter().find(|(k, _)| *k == key))
                .map(|(_, fields)| fields)
        })
    }
}

impl Ui {
    /// A same-process hot reload keeps live subscribers and named callback
    /// registrations. Snapshot restore replaces values, not executable code.
    pub fn inherit_store_registrations(&mut self, previous: &Self) {
        self.docs.reactive = previous.docs.reactive.clone();
        self.docs.reactive.graph.clear_bindings();
        self.inherit_document_registrations(previous);
    }

    /// Apply the host's binary source/computation operations to this UI.
    pub fn apply_reactive(&mut self, bytes: &[u8]) -> Vec<u8> {
        use session::Command;
        session::Session::apply_with(bytes, |command| {
            match command {
                Command::MountInputs => return Ok(self.docs.document_mount_inputs()),
                Command::Mount {
                    mount,
                    document,
                    inputs,
                    content,
                } => {
                    self.register_document_mount(mount, document, inputs, content)?;
                }
                Command::Unmount { mount } => {
                    mount.validate()?;
                    self.docs.reactive.graph.check_idle(&mount)?;
                    self.unmount_document(&mount);
                }
                Command::Registered { sources, mounts } => {
                    self.docs.registered_document_mounts(mounts)?;
                    self.docs.reactive.execute(Command::Registered {
                        sources,
                        mounts: Vec::new(),
                    })?;
                }
                Command::Dispose { scope } => {
                    self.docs
                        .reactive
                        .graph
                        .check_idle(&SourceId::new(&scope, "dispose"))?;
                    self.unmount_scope(&scope);
                    return self.docs.reactive.execute(Command::Dispose { scope });
                }
                command => return self.docs.reactive.execute(command),
            }
            Ok(Value::Null)
        })
    }
}
