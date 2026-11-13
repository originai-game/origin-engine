use super::*;
use crate::doc::schema::Kind;

impl RetainedHost {
    pub(crate) fn emit(
        &mut self,
        ui: &mut Ui,
        child: &Child,
        env: &dyn Env,
        events: &mut Vec<DocEvent>,
        writes: &mut Vec<(BindingContext, String, Value)>,
    ) -> Result<(), String> {
        match child {
            Child::Element(identity) => self.emit_element(ui, identity, false, env, events, writes),
            Child::Branch(identity) => {
                for child in self.branches[identity].children.clone() {
                    self.emit(ui, &child, env, events, writes)?;
                }
                Ok(())
            }
            Child::Each(identity) => {
                let each = &self.each[identity];
                let children: Vec<_> = if each.order.is_empty() {
                    each.empty.clone()
                } else {
                    each.order
                        .iter()
                        .flat_map(|key| each.rows[key].iter().cloned())
                        .collect()
                };
                for child in children {
                    self.emit(ui, &child, env, events, writes)?;
                }
                Ok(())
            }
        }
    }

    /// Writes element `identity`'s render node under the open element, or,
    /// when nothing it is built from changed, puts the node it already has
    /// back without visiting its subtree. `in_place`: the node is written
    /// again into the slot its parent already lists (its parent is not
    /// built this frame).
    pub(crate) fn emit_element(
        &mut self,
        ui: &mut Ui,
        identity: &MountedNode,
        in_place: bool,
        env: &dyn Env,
        events: &mut Vec<DocEvent>,
        writes: &mut Vec<(BindingContext, String, Value)>,
    ) -> Result<(), String> {
        let materialization = if in_place {
            self.elements[identity].materialization
        } else {
            self.materializing.unwrap_or(Materialization::Visible)
        };
        if self.topology.node(identity).unwrap().materialization != materialization {
            self.topology
                .set_materialization(identity, materialization)?;
        }
        if self.dirty.remove(identity)
            || self.elements[identity].style_dirty
            || self.elements[identity].theme_epoch != self.theme_epoch
            || self.topology.style_is_stale(identity) == Some(true)
        {
            self.resolve(ui, identity, env)?;
        }
        let parent_slot = ui.tree.content_parent();
        let element = &self.elements[identity];
        // A world layer (`<anchor>`, `<panel world>`) stands outside the
        // tree, under no parent, and stays among the layers as it is.
        let outside = element.parent_slot == NONE;
        // A subtree whose generated text reads the counter run is held
        // only while the run enters it as it did when it was built.
        let uses_run = ui.docs.library.uses_run;
        let run_moved = uses_run
            && element.run_reads
            && element
                .run_in
                .as_ref()
                .is_none_or(|entered| !entered.same_state(&ui.docs.state().run));
        if !in_place
            && !element.volatile
            && !run_moved
            && !self.emit.contains(identity)
            && let Some(slot) = element.slot
            && ui
                .tree
                .holds(slot, identity.key, if outside { NONE } else { parent_slot })
        {
            if !outside {
                ui.tree.reattach(slot);
            }
            if uses_run {
                ui.docs.state_mut().run.replay(&element.run_events);
                ui.docs.state_mut().run.reads += u64::from(element.run_reads);
            }
            return Ok(());
        }
        // Written in place, outside the tree walk: the run as it entered.
        let saved_run = (uses_run && in_place)
            .then(|| element.run_in.as_deref().cloned())
            .flatten()
            .map(|entered| std::mem::replace(&mut ui.docs.state_mut().run, entered));
        let run_mark = ui.docs.state().run.mark();
        let run_reads = ui.docs.state().run.reads;
        let run_in = uses_run.then(|| ui.docs.state().run.snapshot());
        self.emit.remove(identity);
        self.reconciled += 1;
        if !self.on_action.is_empty() {
            self.built_now.insert(identity.clone());
        }
        let e = self.elements[identity].clone();
        if e.slot.is_some() {
            // Built before and not since retired: not appearing.
            ui.docs.state_mut().memory.seen.insert(identity.key);
        }
        if in_place {
            *ui.tree.in_place_mut() = Some((identity.key, parent_slot));
        }
        ui.docs.state_mut().keeps_hidden = e
            .slot
            .is_some_and(|slot| ui.tree.holds(slot, identity.key, parent_slot));
        ui.tree.owning_mut().push(Default::default());
        ui.docs.state_mut().capture.push(Default::default());
        let resolved = e.resolved.as_ref().expect("resolved mount");
        let mut b = e.build(env, resolved.values.cascade.computed.clone());
        let mut result = Ok(());
        let mut children_built = false;
        ui.docs.run_enter(&resolved.values);
        ui.build_resolved_node(&mut b, e.index, resolved.clone(), |ui, b, virtual_props| {
            children_built = true;
            result = if let Some(props) = virtual_props {
                self.emit_virtual(ui, &e, b, props, env, events, writes)
            } else {
                e.children
                    .iter()
                    .try_for_each(|child| self.emit(ui, child, env, events, writes))
            };
        });
        ui.docs.run_exit();
        let run_events = if uses_run {
            ui.docs.state().run.since(run_mark)
        } else {
            Default::default()
        };
        let reads_run = ui.docs.state().run.reads != run_reads;
        if let Some(saved) = saved_run {
            ui.docs.state_mut().run = saved;
        }
        *ui.tree.in_place_mut() = None;
        let owned = ui.tree.owning_mut().pop().expect("pushed");
        let record = ui.docs.state_mut().capture.pop().expect("pushed");
        events.extend(b.events);
        writes.extend(
            b.writes
                .into_iter()
                .map(|(path, value)| (e.context.clone(), path, value)),
        );
        let slot = owned.slots.iter().copied().find(|&i| {
            ui.tree.nodes()[i].key == identity.key
                && (ui.tree.nodes()[i].parent == parent_slot || ui.tree.nodes()[i].parent == NONE)
        });
        let mut retired: Vec<usize> = e.owned.clone();
        let mut kept = owned.slots.clone();
        kept.sort_unstable();
        retired.retain(|i| kept.binary_search(i).is_err());
        ui.tree.retire_owned(&retired);
        for key in &e.keys {
            self.owner_of.remove(key);
        }
        let mut keys = Vec::with_capacity(owned.slots.len());
        for &i in &owned.slots {
            let node = &ui.tree.nodes()[i];
            keys.push(node.key);
            keys.extend(
                node.layout
                    .links
                    .iter()
                    .map(|l| crate::id::child_key(node.key, l)),
            );
        }
        for key in &keys {
            self.owner_of.insert(*key, identity.clone());
        }
        self.forget_memories(&e.memories);
        for &key in &owned.memories {
            *self.memories.entry(key).or_default() += 1;
        }
        if slot.is_none() && resolved.values.has("visible") && !resolved.values.flag("visible") {
            self.unbuilt_hidden.insert(identity.clone());
        } else {
            self.unbuilt_hidden.remove(identity);
        }
        if resolved.values.str("rive").is_some() {
            self.rive.insert(identity.clone());
        } else {
            self.rive.remove(identity);
        }
        // A viewport whose camera-controls turn it by the tick.
        let node = &e.document.nodes[e.index];
        let volatile = owned.moved
            || (node.kind == NodeKind::Element(Kind::Viewport)
                && node.props.iter().any(|p| p.name() == "cameraControls"));
        if volatile {
            self.volatile.insert(identity.clone());
        } else {
            self.volatile.remove(identity);
        }
        if owned.restyle {
            self.animated.insert(identity.clone());
        } else {
            self.animated.remove(identity);
        }
        let element = self.elements.get_mut(identity).unwrap();
        element.slot = slot;
        // The parent its node stands under: the open element, or none for
        // a world layer.
        element.parent_slot = slot.map_or(parent_slot, |i| ui.tree.nodes()[i].parent);
        element.owned = owned.slots;
        element.keys = keys;
        element.record = record;
        element.memories = owned.memories;
        element.volatile = volatile;
        element.materialization = materialization;
        element.run_in = run_in;
        element.run_events = run_events;
        element.run_reads = reads_run;
        if slot.is_none() || !children_built {
            // Not shown (`visible` false), or showing no content (a closed
            // dialog or disclosure): its descendants' nodes go too, and are
            // built afresh when shown.
            for child in e.children.clone() {
                self.release(ui, &child);
            }
        }
        result
    }

    /// Writes again, into the slots they hold, the elements whose parents
    /// were not built this frame: their ancestors' nodes stay as they are.
    pub(crate) fn emit_in_place(
        &mut self,
        ui: &mut Ui,
        env: &dyn Env,
        events: &mut Vec<DocEvent>,
        writes: &mut Vec<(BindingContext, String, Value)>,
    ) -> Result<(), String> {
        // An element without the slot it was built in (offscreen in a
        // virtual list, in a branch not shown, retired) is written whole
        // when it is built again: marking it does nothing, so the mark goes.
        // One materialized but without a node (never built while hidden,
        // now shown) is built by its nearest ancestor that holds its node.
        let holds = |host: &Self, identity: &MountedNode| {
            host.elements.get(identity).is_some_and(|e| {
                e.slot
                    .is_some_and(|slot| ui.tree.holds(slot, identity.key, e.parent_slot))
            })
        };
        let visible = |host: &Self, identity: &MountedNode| {
            host.topology.node(identity).map(|n| n.materialization)
                == Some(Materialization::Visible)
        };
        let unheld: Vec<MountedNode> = self
            .emit
            .iter()
            .filter(|identity| !holds(self, identity))
            .cloned()
            .collect();
        for identity in unheld {
            self.emit.remove(&identity);
            let shown = self.elements.get(&identity).is_some_and(|e| {
                e.resolved
                    .as_ref()
                    .is_some_and(|r| !r.values.has("visible") || r.values.flag("visible"))
            });
            if !self.unbuilt_hidden.contains(&identity) || !shown || !visible(self, &identity) {
                continue;
            }
            let mut at = self.topology.parent(&identity).cloned();
            while let Some(parent) = at {
                if holds(self, &parent) {
                    self.emit.insert(parent);
                    break;
                }
                if !visible(self, &parent) {
                    break;
                }
                at = self.topology.parent(&parent).cloned();
            }
        }
        let mut pending: Vec<(usize, MountedNode)> = self
            .emit
            .iter()
            .map(|identity| {
                let mut depth = 0;
                let mut at = self.topology.parent(identity);
                while let Some(parent) = at {
                    depth += 1;
                    at = self.topology.parent(parent);
                }
                (depth, identity.clone())
            })
            .collect();
        pending.sort();
        for (_, identity) in pending {
            if !self.emit.contains(&identity) {
                continue;
            }
            let element = &self.elements[&identity];
            let Some(slot) = element.slot else { continue };
            let parent = element.parent_slot;
            if !ui.tree.holds(slot, identity.key, parent) {
                continue;
            }
            let outer = (ui.docs.state().accepts, ui.docs.state().building);
            let (accepts, building) = self.contexts.get(&identity.mount).copied().unwrap_or(outer);
            (ui.docs.state_mut().accepts, ui.docs.state_mut().building) = (accepts, building);
            // A world layer's own arm opens its layer again.
            if parent != NONE {
                let owner = ui.tree.nodes()[parent]
                    .part_box
                    .as_ref()
                    .filter(|part| ui.tree.nodes()[part.owner].content_slot == parent)
                    .map_or(parent, |part| part.owner);
                ui.tree.stack_mut().push(owner);
            }
            let result = self.emit_element(ui, &identity, true, env, events, writes);
            if parent != NONE {
                ui.tree.stack_mut().pop();
            }
            (ui.docs.state_mut().accepts, ui.docs.state_mut().building) = outer;
            result?;
        }
        Ok(())
    }

    /// The path element `p`'s children are built under, so their keys
    /// are their nodes' own: a modal dialog's panel and a popover's layer
    /// stand in the top layer, named apart from the element (a spotlight's
    /// callout layer takes the element's own key and path).
    pub(crate) fn content_path(&self, p: &MountedNode) -> String {
        let e = &self.elements[p];
        let n = &e.document.nodes[e.index];
        let set = |name: &str| {
            n.props.iter().any(|s| {
                schema::PROPS[s.spec].name == name
                    && !matches!(
                        s.value,
                        crate::doc::PropValue::Lit(crate::doc::Lit::Bool(false))
                    )
            })
        };
        let local = e.path.rsplit('/').next().unwrap_or(&e.path);
        match n.kind {
            NodeKind::Element(Kind::Dialog) if set("modal") => format!("{local}/Panel"),
            NodeKind::Element(_) if set("popover") => {
                format!("{}~popover", e.path.replace('/', "_"))
            }
            _ => e.path.clone(),
        }
    }

    pub(crate) fn forget_memories(&mut self, keys: &[u64]) {
        for key in keys {
            if let Some(count) = self.memories.get_mut(key) {
                *count -= 1;
                if *count == 0 {
                    self.memories.remove(key);
                }
            }
        }
    }

    /// Retires the render nodes of `child` and everything under it: it left
    /// the render tree (removed, hidden, scrolled out of a virtual list).
    pub(crate) fn release(&mut self, ui: &mut Ui, child: &Child) {
        match child {
            Child::Element(identity) => {
                if self.elements[identity].slot.is_none()
                    && self.elements[identity].owned.is_empty()
                {
                    return;
                }
                self.release_element(ui, identity);
                for child in self.elements[identity].children.clone() {
                    self.release(ui, &child);
                }
            }
            Child::Branch(identity) => {
                for child in self.branches[identity].children.clone() {
                    self.release(ui, &child);
                }
            }
            Child::Each(identity) => {
                let each = &self.each[identity];
                let children: Vec<_> = each
                    .rows
                    .values()
                    .flatten()
                    .chain(&each.empty)
                    .cloned()
                    .collect();
                for child in children {
                    self.release(ui, &child);
                }
            }
        }
    }

    /// Retires one element's own render nodes (not its descendants').
    pub(crate) fn release_element(&mut self, ui: &mut Ui, identity: &MountedNode) {
        let Some(element) = self.elements.get_mut(identity) else {
            return;
        };
        let owned = std::mem::take(&mut element.owned);
        let keys = std::mem::take(&mut element.keys);
        let memories = std::mem::take(&mut element.memories);
        element.slot = None;
        element.record = Default::default();
        ui.tree.retire_owned(&owned);
        for key in keys {
            self.owner_of.remove(&key);
        }
        self.forget_memories(&memories);
        self.volatile.remove(identity);
        self.animated.remove(identity);
        self.emit.remove(identity);
        self.scrollers.remove(identity);
    }
}
