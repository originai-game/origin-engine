use super::*;

impl RetainedHost {
    /// Marks what this frame's interaction, clock and frame-wide inputs
    /// require to be built again, before anything is emitted.
    pub(super) fn mark_frame(&mut self, ui: &mut Ui) {
        let signature = Signature {
            viewport: ui.env.viewport,
            unit: ui.env.unit,
            direction: ui.env.direction,
            locale: ui.env.locale.clone(),
            style: ui.env.style_name.clone(),
            style_epoch: ui.invalidation.epoch,
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
            input.extend(ui.docs.host.zones.keys());
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
        // A forced state is drawn by its element's widget: it builds again.
        input.extend(self.forced_keys(ui));
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
        for key in std::mem::take(&mut ui.style.motion.timeline_restyle) {
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
    pub(super) fn advance_rive(&mut self, ui: &mut Ui) {
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
    pub(super) fn emit_fired(
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
