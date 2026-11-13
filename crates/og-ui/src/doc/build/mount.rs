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
        let name = doc.name.clone();
        self.docs.library.define(doc);
        let mut retained = std::mem::take(&mut self.docs.state_mut().memory.retained);
        retained.document_replaced(self, &name);
        self.docs.state_mut().memory.retained = retained;
    }

    /// Registers a Rive artboard by its id (`ui/chest.riv#Chest`, as og-cook
    /// converted it, docs/ui.md §3.3): `<og-rive src="id">` plays it.
    /// One of the same id is replaced (a hot
    /// reload): an element playing the old one goes on from where it was
    /// when it fits the new one, else starts it again.
    pub fn define_rive(&mut self, id: &str, artboard: Arc<crate::rive::Artboard>) {
        self.docs.library.rive.insert(id.to_owned(), artboard);
    }

    /// Binds the texture asset `path` a document names
    /// ([`UiDocument::cover_image`]) to its loaded handle.
    pub fn define_texture(&mut self, path: &str, texture: og_draw::TextureHandle) {
        self.docs.library.textures.insert(path.to_owned(), texture);
    }

    /// Gives the texture asset `path` its alpha channel on the CPU, which
    /// a float's `shape-outside: url(<path>)` takes its contour from
    /// ([`UiDocument::shape_images`]); without it the float has no shape.
    pub fn define_texture_alpha(&mut self, path: &str, alpha: crate::ImageAlpha) {
        self.docs
            .library
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
            .library
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
        let Some(doc) = self.docs.library.documents.get(name).cloned() else {
            self.out.warnings_mut().push(format!(
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
        let accepts = std::mem::replace(&mut self.docs.state_mut().accepts, game);
        let r = self.mount_doc(doc, mount, id, props, env, Arc::default());
        self.docs.state_mut().accepts = accepts;
        r
    }

    /// Events of documents built where code did not see them (screens
    /// built at the end of a tick), taken.
    /// The play, pause, seek, gain and loop requests documents' `<video>`
    /// and `<audio>` made this frame, for the host to apply to their
    /// players at the next tick.
    pub fn take_media_ops(&mut self) -> Vec<crate::bind::MediaOp> {
        std::mem::take(&mut self.docs.state_mut().media_ops)
    }

    pub fn take_doc_events(&mut self) -> Vec<DocEvent> {
        std::mem::take(&mut self.docs.state_mut().queued)
    }

    /// What documents built this tick (the probe's `dump_ui`): the
    /// immediate documents' elements, then the mounted ones' in tree order.
    pub fn doc_built(&self) -> Vec<Built> {
        let mut out = self.docs.state().built.clone();
        out.extend(
            self.docs
                .state()
                .memory
                .retained
                .records()
                .flat_map(|r| r.built.iter().cloned()),
        );
        out
    }

    /// The art slots documents declared this tick, by path.
    pub fn art_slots(&self) -> Vec<(String, crate::slot::SlotSpec)> {
        let mut out = self.docs.state().arts.clone();
        out.extend(
            self.docs
                .state()
                .memory
                .retained
                .records()
                .flat_map(|r| r.arts.iter().cloned()),
        );
        out
    }

    /// The probe's edits in effect.
    pub fn doc_edits(&self) -> &[Edit] {
        &self.docs.state().edits
    }

    /// `ui:set`: sets a prop of the element at `path` (presentation only:
    /// it applies over the document's own line until cleared or saved).
    pub fn set_doc_prop(&mut self, path: &str, prop: &str, value: &str) -> Result<(), String> {
        let (spec, v) = parse_probe_value(prop, value)?;
        self.docs
            .state_mut()
            .edits
            .retain(|e| !(e.path == path && e.prop == prop));
        self.docs.state_mut().edits.push(Edit {
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
            .state_mut()
            .edits
            .retain(|e| !path.is_empty() && e.path != path);
    }

    /// The document element built at `path` last tick: its document's name
    /// and node.
    pub fn doc_element(&self, path: &str) -> Option<(&str, usize)> {
        self.docs
            .state()
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
        let edits = self.docs.state().edits.clone();
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
                    let d = (**self.docs.library.documents.get(&doc).ok_or("no document")?).clone();
                    changed.entry(doc.clone()).or_insert(d)
                }
            };
            d.set_prop(node, &e.prop, &e.source)
                .map_err(|why| format!("{}: {}.{}: {why}", d.source, e.path, e.prop))?;
        }
        self.docs.state_mut().edits = kept;
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
            .state_mut()
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
            .state()
            .plays
            .get(&(path.to_string(), timeline.to_string()))
            && self.seek(id, ms)
        {
            self.pause(id);
            return Ok(id);
        }
        self.play_doc_timeline(path, timeline, ms, true)
    }
}

impl Ui {
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

    pub(crate) fn timeline_name(&self, path: &str, timeline: &str) -> Result<String, String> {
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
                    .state()
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
}

impl crate::ui::Docs {
    pub(crate) fn note_built(&mut self, built: Built) {
        match self.state_mut().capture.last_mut() {
            Some(record) => record.built.push(built),
            None => self.state_mut().built.push(built),
        }
    }

    pub(crate) fn note_render_fields(&mut self, key: u64, fields: super::fields::RenderFields) {
        match self.state_mut().capture.last_mut() {
            Some(record) => record.render_fields.push((key, fields)),
            None => {
                self.state_mut().render_fields.insert(key, fields);
            }
        }
    }

    pub(crate) fn note_art(&mut self, path: String, spec: crate::slot::SlotSpec) {
        match self.state_mut().capture.last_mut() {
            Some(record) => record.arts.push((path, spec)),
            None => self.state_mut().arts.push((path, spec)),
        }
    }

    /// The art slot registered last.
    pub(crate) fn last_art(&self) -> &crate::slot::SlotSpec {
        &self.last_art_entry().1
    }

    /// The render fields element `key` was emitted with.
    #[cfg(test)]
    pub(crate) fn render_fields(&self, key: u64) -> Option<&super::fields::RenderFields> {
        self.state().render_fields.get(&key).or_else(|| {
            self.state()
                .memory
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
        self.docs.state_mut().memory.reactive = previous.docs.state().memory.reactive.clone();
        self.docs.state_mut().memory.reactive.graph.clear_bindings();
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
                    self.docs.state().memory.reactive.graph.check_idle(&mount)?;
                    self.unmount_document(&mount);
                }
                Command::Registered { sources, mounts } => {
                    self.docs.registered_document_mounts(mounts)?;
                    self.docs
                        .state_mut()
                        .memory
                        .reactive
                        .execute(Command::Registered {
                            sources,
                            mounts: Vec::new(),
                        })?;
                }
                Command::Dispose { scope } => {
                    self.docs
                        .state()
                        .memory
                        .reactive
                        .graph
                        .check_idle(&SourceId::new(&scope, "dispose"))?;
                    self.unmount_scope(&scope);
                    return self
                        .docs
                        .state_mut()
                        .memory
                        .reactive
                        .execute(Command::Dispose { scope });
                }
                command => return self.docs.state_mut().memory.reactive.execute(command),
            }
            Ok(Value::Null)
        })
    }
}
