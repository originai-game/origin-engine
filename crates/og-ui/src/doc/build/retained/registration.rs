use super::*;

impl Ui {
    /// Register once over the host protocol; the next frame compiles with its
    /// real host environment, after source callbacks have settled.
    pub(crate) fn register_document_mount(
        &mut self,
        mount: SourceId,
        document: String,
        inputs: BTreeMap<String, SourceId>,
        content: BTreeMap<String, String>,
    ) -> Result<(), String> {
        mount.validate()?;
        self.docs.reactive.graph.check_idle(&mount)?;
        if !self.docs.host.docs.contains_key(&document) {
            return Err(format!("missing document '{document}'"));
        }
        super::content::check_instances(&self.docs.host.docs, &document)?;
        for reference in content.values() {
            let name = reference
                .split_once('#')
                .map_or(reference.as_str(), |(d, _)| d);
            if !self.docs.host.docs.contains_key(name) {
                return Err(format!("mount content names a missing document '{name}'"));
            }
        }
        for (name, source) in &inputs {
            if name.is_empty() {
                return Err("document input names must be nonempty".into());
            }
            source.validate()?;
        }
        // Every mount reads the engine's `$ui` unless it binds its own, as
        // `Ui::mount_document`'s do.
        let mut inputs = inputs;
        if !inputs.contains_key("ui") {
            self.publish_ui_store();
            inputs.insert("ui".into(), crate::doc::UI_STORE.clone());
        }
        let registration = Registration {
            document,
            inputs,
            content,
        };
        let document = registration.document.clone();
        if self.docs.host.retained.registrations.get(&mount) == Some(&registration) {
            self.sync_mount_layer(&mount, Some(&document));
            return Ok(());
        }
        let mut host = std::mem::take(&mut self.docs.host.retained);
        host.remove_mount(self, &mount);
        host.registrations.insert(mount.clone(), registration);
        host.pending_mounts.insert(mount.clone());
        self.docs.host.retained = host;
        self.sync_mount_layer(&mount, Some(&document));
        Ok(())
    }

    pub(crate) fn unmount_scope(&mut self, scope: &str) {
        let mounts: Vec<_> = self
            .docs
            .host
            .retained
            .registrations
            .keys()
            .filter(|mount| mount.scope == scope)
            .cloned()
            .collect();
        // Disposal is immediate: a leaving cover would read removed stores.
        for mount in mounts {
            if let Some(k) = self.docs.mount_layer(&mount) {
                self.docs.host.layers.layers.remove(k);
            }
            self.finish_unmount(&mount);
        }
        if let Some(managed) = &mut self.docs.host.retained.managed {
            managed.retain(|id, _| id.scope != scope);
        }
    }

    pub(crate) fn inherit_document_registrations(&mut self, previous: &Self) {
        let registrations = previous.docs.host.retained.registrations.clone();
        self.docs.host.retained = RetainedHost {
            pending_mounts: registrations.keys().cloned().collect(),
            registrations,
            managed: previous.docs.host.retained.managed.clone(),
            ..Default::default()
        };
    }
}

impl crate::ui::Docs {
    pub(crate) fn document_mount_inputs(&self) -> Value {
        json!(self.host.retained.registrations.iter().map(|(mount, registration)| {
            json!({ "mount": mount, "inputs": registration.inputs.values().collect::<Vec<_>>() })
        }).collect::<Vec<_>>())
    }

    pub(crate) fn registered_document_mounts(
        &mut self,
        mounts: Vec<SourceId>,
    ) -> Result<(), String> {
        self.reactive
            .graph
            .check_idle(&SourceId::new("registration", "complete"))?;
        let mut managed = BTreeMap::new();
        for mount in mounts {
            let declaration = self
                .host
                .retained
                .registrations
                .get(&mount)
                .ok_or_else(|| {
                    format!("unregistered document mount '{}:{}'", mount.scope, mount.id)
                })?;
            managed.insert(mount, declaration.clone());
        }
        self.host.retained.managed = Some(managed);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactive::wire;

    #[test]
    fn binary_mounts_render_update_restore_and_dispose_without_element_calls() {
        let mut ui = Ui::default();
        ui.define_document(
            UiDocument::parse(
                "wire.svelte",
                r#"
            <meta name="component" content="wire">
            <column><text id="Title">{$title}</text></column>
        "#,
            )
            .unwrap(),
        );
        let send = |ui: &mut Ui, command: Value| {
            let reply = wire::decode(&ui.apply_reactive(&wire::encode(&json!([command])).unwrap()))
                .unwrap();
            assert_eq!(reply[0]["ok"], true, "{reply}");
        };
        let tick = |ui: &mut Ui| {
            ui.begin(
                crate::InputFrame::default(),
                crate::Viewport::new(640.0, 480.0),
            );
            ui.end();
            ui.probe_elements()
                .into_iter()
                .filter(|e| e.kind == "label")
                .map(|e| e.text)
                .collect::<Vec<_>>()
        };
        let source = SourceId::new("test", "title");
        let mount = json!({"op":"mount", "mount":SourceId::new("test", "view"),
            "document":"wire", "inputs":{"title":source}});
        send(
            &mut ui,
            json!({"op":"signal", "source":source, "initial":"First"}),
        );
        send(&mut ui, mount.clone());
        assert_eq!(tick(&mut ui), ["First"]);
        let saved = ui.save_state();
        let identity = ui.mounted_topology().child_at(None, 0).unwrap().clone();
        send(&mut ui, mount);
        send(
            &mut ui,
            json!({"op":"set", "source":source, "value":"Second"}),
        );
        assert_eq!(tick(&mut ui), ["Second"]);
        assert_eq!(ui.mounted_topology().child_at(None, 0), Some(&identity));
        ui.restore_state(&saved).unwrap();
        assert_eq!(tick(&mut ui), ["First"]);
        send(&mut ui, json!({"op":"dispose", "scope":"test"}));
        assert!(tick(&mut ui).is_empty());
    }

    /// A mount registered over the host protocol (TypeScript's
    /// `scope.mount`) reads the engine's `$ui` as `Ui::mount_document`'s
    /// do: an action glyph follows the device through it.
    #[test]
    fn binary_mounts_read_the_engine_s_ui_store() {
        let mut ui = Ui::default();
        ui.define_document(
            UiDocument::parse(
                "device.svelte",
                r#"
            <meta name="component" content="device">
            <column><text id="Device">{$ui.device}</text></column>
        "#,
            )
            .unwrap(),
        );
        let command = json!({"op":"mount", "mount":SourceId::new("test", "view"),
            "document":"device", "inputs":{}});
        let reply =
            wire::decode(&ui.apply_reactive(&wire::encode(&json!([command])).unwrap())).unwrap();
        assert_eq!(reply[0]["ok"], true, "{reply}");
        ui.begin(
            crate::InputFrame::default(),
            crate::Viewport::new(640.0, 480.0),
        );
        let warnings = ui.end().warnings.clone();
        let text: Vec<_> = ui
            .probe_elements()
            .into_iter()
            .filter(|e| e.kind == "label")
            .map(|e| e.text)
            .collect();
        assert_eq!(text, ["keyboardMouse"]);
        assert!(warnings.is_empty(), "{warnings:?}");
    }
}
