use super::*;

impl RetainedHost {
    /// Marks what this frame's interaction, clock and frame-wide inputs
    /// require to be built again, before anything is emitted.
    pub(crate) fn mark_frame(&mut self, ui: &mut Ui) {
        let signature = Signature {
            viewport: *ui.env.viewport(),
            unit: ui.env.unit(),
            direction: ui.env.direction(),
            locale: ui.env.locale().clone(),
            style: ui.env.style_name().clone(),
            style_epoch: ui.tree.invalidation().epoch,
            players: ui.interaction.players(),
            nav_on: ui.interaction.nav_on(),
        };
        self.restyled = self.signature.as_ref() != Some(&signature);
        if self.restyled {
            self.signature = Some(signature);
            self.emit.extend(self.elements.keys().cloned());
        }
        let s = &ui.interaction.state();
        let n = &ui.interaction.nav();
        // What a press holds reads the pointer each frame: written while
        // held and the frame it is let go.
        let mut held = Vec::new();
        if ui.interaction.input().down {
            held.push(s.active);
        }
        // On-screen controls follow their further touches (og.Pointers):
        // those a touch holds, and every one when a touch goes down.
        held.extend(s.touch_owners.keys());
        if ui.interaction.input().pointers.pressed != 0 {
            held.extend(ui.docs.state().zones.keys());
        }
        // Keys, typing and pads reach the focused controls as they build
        // (a field's edits, a list's arrows, a range's steps).
        let i = ui.interaction.input();
        let idle_pad = crate::interaction::input::GamepadFrame::default();
        if !i.keys.is_empty()
            || !i.text.is_empty()
            || i.composition.is_some()
            || !i.keys_held.is_empty()
            || !i.keys_pressed.is_empty()
            || !i.actions.is_empty()
            || i.gamepad != idle_pad
            || i.extra_pads.iter().any(|p| *p != idle_pad)
        {
            held.push(s.focus);
            held.extend(s.pad_focus);
        }
        // A shown popover closes on a press outside it or a key (light
        // dismiss, Escape): it builds on every tick that has one.
        if i.pressed
            || i.secondary_pressed
            || i.pointers.pressed != 0
            || !i.keys.is_empty()
            || n.back
        {
            for (pk, node) in &self.popover_nodes {
                if ui.docs.state().popovers.get(pk).is_some_and(|l| l.shown) {
                    self.emit.insert(node.clone());
                }
            }
        }
        // A pointer moving or wheeling over the hovered control reaches
        // it as it builds (a range's nearer knob, a picker's option).
        let pointer = [i.pointer.x, i.pointer.y];
        if pointer != self.last_pointer || i.wheel != og_core::glam::Vec2::ZERO {
            held.push(s.hot);
            self.last_pointer = pointer;
        }
        let mut input = vec![
            s.hot,
            s.focus,
            n.activate,
            n.tapped,
            n.double_tapped,
            n.long_pressed,
            n.spin.0,
        ];
        input.extend(s.pad_focus);
        input.extend(n.pad_activate);
        if n.back {
            // Back belongs to the topmost scope (a modal, an open list),
            // else to what holds the focus (a dialog in place).
            input.push(s.scope);
            if s.focus != 0
                && let Some(mut i) = ui.tree.nodes().iter().position(|node| node.key == s.focus)
            {
                while ui.tree.nodes()[i].parent != NONE {
                    i = ui.tree.nodes()[i].parent;
                    input.push(ui.tree.nodes()[i].key);
                }
            }
        }
        // A forced state is drawn by its element's widget: it builds again.
        input.extend(self.forced_keys(ui));
        for keys in [&mut input, &mut held] {
            keys.retain(|&key| key != 0);
            keys.sort_unstable();
            keys.dedup();
        }
        // An element is written again as its interaction changes, not
        // while it lasts: a focused or hovered element that stays so is
        // kept. One that counts while it lasts (a tooltip's delay, an
        // interest delay) asks to be built again itself (`Owned::moved`).
        let mut changed: Vec<u64> = held.iter().chain(&self.held).copied().collect();
        changed.extend(sorted_difference(&input, &self.input));
        changed.extend(sorted_difference(&self.input, &input));
        // The focused text control's caret blinks.
        let caret = if s.edit_key != 0 && s.edit_key == s.focus {
            (s.focus, ui.interaction.caret_shown(&ui.env))
        } else {
            (0, false)
        };
        if caret != self.caret {
            changed.extend([caret.0, self.caret.0]);
            self.caret = caret;
        }
        for key in &changed {
            if let Some(owner) = self.owner_of.get(key) {
                self.emit.insert(owner.clone());
            }
        }
        self.last_input = std::mem::replace(&mut self.input, input);
        self.last_held = std::mem::replace(&mut self.held, held);
        for (identity, offset) in &self.scrollers {
            if ui
                .interaction
                .state()
                .scroll
                .get(&identity.key)
                .copied()
                .unwrap_or(0.0)
                != *offset
            {
                self.emit.insert(identity.clone());
            }
        }
        // Nodes whose laid-out size moved: their widgets read last frame's
        // geometry (a narrow button, a list measuring its rows).
        for slot in std::mem::take(&mut ui.layout.cache_mut().resized) {
            if let Some(owner) = self.owner_of.get(&ui.tree.nodes()[slot].key) {
                self.emit.insert(owner.clone());
            }
        }
        // A scroll container whose markers changed or moved.
        let worked = self.reconciled > 0 || ui.frame().stats.laid_out > 0;
        for key in ui.scroll_groups_moved(worked) {
            if let Some(owner) = self.owner_of.get(&key) {
                self.emit.insert(owner.clone());
            }
        }
        // A text whose `::first-line` the last layout broke elsewhere.
        for (&key, &built) in &ui.docs.state().first_lines {
            if ui.style.first_line_chars(key, &ui.tree) != built
                && let Some(owner) = self.owner_of.get(&key)
            {
                self.emit.insert(owner.clone());
            }
        }
        self.emit.extend(self.volatile.iter().cloned());
        // Computed values that follow a timeline that moved.
        for key in std::mem::take(&mut ui.style.motion_mut().timeline_restyle) {
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
    pub(crate) fn mark_fired(&mut self, ui: &Ui) {
        // Whether a listener's layer takes input is the build's question
        // (`docs.accepts`): every listener of a fired action is built.
        let fired = &ui.interaction.actions().fired;
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
    pub(crate) fn advance_rive(&mut self, ui: &mut Ui) {
        let elements = &self.elements;
        self.rive.retain(|identity| elements.contains_key(identity));
        self.unbuilt_hidden
            .retain(|identity| elements.contains_key(identity));
        for identity in &self.rive {
            let e = &self.elements[identity];
            let Some(slot) = e.slot else { continue };
            if !ui.tree.holds(slot, identity.key, e.parent_slot) {
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
                .interaction
                .state()
                .rive
                .get(&ui.tree.nodes()[index].key)
                .is_some_and(|s| s.built == ui.interaction.state().tick);
            if !built && !ui.tree.nodes()[index].hidden {
                ui.docs.play_rive(
                    index,
                    board,
                    &resolved.path,
                    &ui.env,
                    &mut ui.out,
                    &mut ui.interaction,
                    &mut ui.tree,
                );
            }
        }
    }

    /// Builds the listeners of actions that triggers fired while this
    /// frame was built and that are not built yet, as every listener after
    /// the trigger in the frame reacts.
    pub(crate) fn emit_fired(
        &mut self,
        ui: &mut Ui,
        env: &dyn Env,
        events: &mut Vec<DocEvent>,
        writes: &mut Vec<(BindingContext, String, Value)>,
    ) -> Result<(), String> {
        while ui.interaction.actions().fired.len() > self.fired {
            self.mark_fired(ui);
            self.emit_in_place(ui, env, events, writes)?;
        }
        Ok(())
    }
}

/// The keys of sorted `a` that sorted `b` lacks.
fn sorted_difference<'a>(a: &'a [u64], b: &'a [u64]) -> impl Iterator<Item = u64> + 'a {
    a.iter()
        .copied()
        .filter(move |key| b.binary_search(key).is_err())
}
