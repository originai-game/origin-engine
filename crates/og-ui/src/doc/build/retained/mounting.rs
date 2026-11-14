use super::*;

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
        if !self.docs.library.documents.contains_key(document) {
            return Err(format!("missing document '{document}'"));
        }
        content::check_instances(&self.docs.library.documents, document)?;
        // Every mount reads the engine's `$ui` unless it binds its own.
        let mut inputs = inputs;
        if !inputs.contains_key("ui") {
            self.publish_ui_store();
            inputs.insert("ui".into(), crate::doc::UI_STORE.clone());
        }
        let mut host = std::mem::take(&mut self.docs.state_mut().memory.retained);
        if host.roots.contains_key(&mount) {
            self.docs.state_mut().memory.retained = host;
            return Err(format!(
                "mount '{}:{}' already exists",
                mount.scope, mount.id
            ));
        }
        let registration = Registration {
            document: document.into(),
            inputs,
            content,
        };
        let result = match host.compile_mount(self, &mount, &registration, env) {
            Ok(root) => {
                host.registrations.insert(mount.clone(), registration);
                host.roots.insert(mount.clone(), root);
                self.docs.state_mut().memory.retained = host;
                self.sync_mount_layer(&mount, Some(document));
                return Ok(());
            }
            Err(error) => {
                host.remove_mount(self, &mount);
                Err(error)
            }
        };
        self.docs.state_mut().memory.retained = host;
        result
    }

    /// Unmounts a mount. A mounted screen leaves the stack with it; when its
    /// screen has a view transition, the mount stays until the cover has
    /// closed over it.
    pub fn unmount_document(&mut self, mount: &SourceId) {
        if let Some(k) = self.docs.mount_layer(mount) {
            if self.layer_leaves(k) {
                return;
            }
            self.docs.layers.layers.remove(k);
        }
        self.finish_unmount(mount);
    }

    pub fn build_retained_documents(&mut self, env: &dyn Env) -> Result<(), String> {
        let mut host = std::mem::take(&mut self.docs.state_mut().memory.retained);
        if host.built_tick == Some(self.tick()) {
            self.docs.state_mut().memory.retained = host;
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
                let layer = self.docs.mount_layer(&mount);
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
                host.contexts.insert(
                    mount.clone(),
                    (self.docs.state().accepts, self.docs.state().building),
                );
                let emitted = host.emit(self, &root, env, &mut events, &mut writes);
                if let Some(Some(frame)) = frame {
                    let doc = &self.docs.library.documents[&host.registrations[&mount].document];
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
            self.docs.state_mut().memory.retained = std::mem::take(&mut host);
            self.keep_world_anchors();
            host = std::mem::take(&mut self.docs.state_mut().memory.retained);
            self.docs.state_mut().queued.extend(events);
            for (context, path, value) in writes {
                if let Err(error) =
                    context.write_model(&mut self.docs.state_mut().memory.reactive, &path, value)
                {
                    self.docs.state_mut().memory.retained = host;
                    return Err(error);
                }
            }
            if result.is_ok() {
                host.built_tick = Some(self.tick());
            }
        }
        self.docs.state_mut().memory.retained = host;
        result
    }

    pub fn mounted_topology(&self) -> &Topology {
        &self.docs.state().memory.retained.topology
    }
}
