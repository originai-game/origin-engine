use super::*;

pub(crate) trait RetainedRefreshRetainedHost {
    fn sync_facts(
        &mut self,
        ui: &mut Ui,
        element: &ElementInstance,
        env: &dyn Env,
    ) -> Result<(), String>;

    fn refresh(&mut self, ui: &mut Ui, env: &dyn Env) -> Result<(), String>;

    /// Materialized nodes in invalidated style intervals resolve before they
    /// draw; deferred rows keep their interval until they materialize.
    fn resolve_invalidated_styles(&mut self);

    /// The mounted elements whose interaction state the probe forced.
    fn forced_keys(&self, ui: &Ui) -> Vec<u64>;

    fn sync_interaction(&mut self, ui: &mut Ui) -> Result<(), String>;

    fn resolve(&mut self, ui: &mut Ui, identity: &MountedNode, env: &dyn Env)
    -> Result<(), String>;
}

impl RetainedRefreshRetainedHost for RetainedHost {
    fn sync_facts(
        &mut self,
        ui: &mut Ui,
        element: &ElementInstance,
        env: &dyn Env,
    ) -> Result<(), String> {
        use crate::bind::InteractionState::*;
        let identity = &element.context.node;
        let build = element.build(env, Arc::default());
        let node = &element.document.nodes[element.index];
        let mut classes = BTreeSet::new();
        let mut has_classes = false;
        for (name, expression) in &node.selector_attrs {
            let value = build.eval(expression);
            if name == "class" {
                has_classes = true;
                if let Some(value) = value.as_str() {
                    classes.extend(value.split_whitespace().map(str::to_owned));
                }
            } else if let Some(class) = name.strip_prefix("class:") {
                has_classes = true;
                if truthy(&value) {
                    classes.insert(class.to_owned());
                }
                self.topology.set_attribute(identity, name, Some(value))?;
            } else {
                self.topology.set_attribute(identity, name, Some(value))?;
            }
        }
        if has_classes {
            self.topology.set_attribute(
                identity,
                "class",
                Some(json!(classes.into_iter().collect::<Vec<_>>().join(" "))),
            )?;
        }
        let mut facts = BTreeMap::new();
        for prop in &node.props {
            let spec = &PROPS[prop.spec];
            let value = match build.value(ui, spec.ty, &prop.value) {
                Some(V::Bool(v)) => Some(json!(v)),
                Some(V::Num(v)) => Some(json!(v)),
                Some(V::Str(v)) => Some(json!(v)),
                Some(V::Strs(v)) => Some(json!(v)),
                _ => None,
            };
            if let Some(value) = value {
                facts.insert(spec.name, value);
            }
        }
        let flag = |name| facts.get(name).is_some_and(truthy);
        let num = |name| facts.get(name).and_then(Value::as_f64);
        // A field's text its `type` or its constraints refuse is invalid.
        let field_invalid = facts.get("input").and_then(Value::as_str).is_some_and(|v| {
            let ty = facts
                .get("type")
                .and_then(Value::as_str)
                .and_then(crate::InputType::from_name)
                .unwrap_or_default();
            ty.mismatch(v)
                || crate::input_type::Constraints {
                    required: flag("required"),
                    min_chars: num("minChars").unwrap_or(0.0) as u32,
                    pattern: facts
                        .get("pattern")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    min: num("min"),
                    max: num("max"),
                    step: num("stepBy"),
                }
                .fails(ty, v)
        });
        self.topology.set_text(
            identity,
            facts
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        )?;
        let kind = match node.kind {
            NodeKind::Element(kind) => kind,
            _ => "",
        };
        // HTML's form-associated elements: they are enabled or disabled,
        // valid or invalid; text fields are read-write unless readonly.
        let form = matches!(
            kind,
            "button" | "field" | "slider" | "toggle" | "radio" | "select" | "color"
        );
        let disabled = flag("disabled");
        let invalid = flag("invalid") || field_invalid;
        let range = match (
            num("value").or_else(|| {
                facts
                    .get("input")
                    .and_then(Value::as_str)
                    .and_then(|v| v.trim().parse().ok())
            }),
            num("min"),
            num("max"),
        ) {
            (Some(v), min, max)
                if kind == "slider" || (kind == "field" && (min.is_some() || max.is_some())) =>
            {
                Some(min.is_none_or(|m| v >= m) && max.is_none_or(|m| v <= m))
            }
            _ => None,
        };
        let editable = kind == "field" && !flag("readonly") && !disabled;
        let open = flag("open");
        if facts.contains_key("popover") {
            let key = super::popover::popover_key(&build.mount, &node.id);
            self.popover_nodes.insert(key, identity.clone());
        }
        let authored = |name: &str| {
            node.selector_attrs
                .iter()
                .any(|(a, e)| a == name && !build.eval(e).is_null())
        };
        for (state, enabled) in [
            (Disabled, disabled),
            (Enabled, form && !disabled),
            (Checked, flag("checked")),
            (Invalid, invalid),
            (Valid, form && !invalid),
            // After the player changed it: a field with text it refuses.
            (
                UserInvalid,
                invalid
                    && facts
                        .get("input")
                        .and_then(Value::as_str)
                        .is_some_and(|v| !v.is_empty()),
            ),
            (Required, flag("required")),
            (Optional, form && !flag("required")),
            (ReadWrite, editable),
            (ReadOnly, !editable),
            (InRange, range == Some(true)),
            (OutOfRange, range == Some(false)),
            (Indeterminate, flag("indeterminate")),
            (Default, authored("checked") || authored("selected")),
            (Open, open),
            (Modal, open && flag("modal")),
            (
                PlaceholderShown,
                facts
                    .get("placeholder")
                    .and_then(Value::as_str)
                    .is_some_and(|v| !v.is_empty())
                    && facts
                        .get("input")
                        .and_then(Value::as_str)
                        .is_none_or(str::is_empty),
            ),
        ] {
            // The states HTML always had here; the rest only when a rule of
            // the element's document reads them.
            if !matches!(state, Disabled | Checked | Invalid | PlaceholderShown)
                && !self
                    .profile(ui, &element.document.name)
                    .reads
                    .contains(&state)
            {
                continue;
            }
            if self
                .topology
                .node(identity)
                .unwrap()
                .states
                .contains(&state)
                != enabled
            {
                self.topology.set_state(identity, state, enabled)?;
                ui.docs
                    .state_mut()
                    .memory
                    .reactive
                    .graph
                    .interaction_changed(identity, state);
            }
        }
        Ok(())
    }

    fn refresh(&mut self, ui: &mut Ui, env: &dyn Env) -> Result<(), String> {
        self.restore_instances(ui, env)?;
        self.refresh_collections(ui, env)?;
        self.sync_interaction(ui)?;
        let presentation_changed =
            std::mem::replace(&mut self.presentation_epoch, ui.tree.invalidation().epoch)
                != ui.tree.invalidation().epoch;
        if self
            .theme
            .replace(*ui.theme())
            .is_some_and(|old| old != *ui.theme())
            || self
                .direction
                .replace(ui.env.direction())
                .is_some_and(|old| old != ui.env.direction())
            || presentation_changed
        {
            self.theme_epoch += 1;
        }
        let scrolled = std::mem::take(&mut ui.docs.host_mut().scroll_changed);
        self.rescroll(&scrolled);
        let viewport = ui.media_context();
        let [small, large] = ui.env.viewport().extremes();
        let sa = ui.env.viewport().env_insets();
        let hud = ui.env.viewport().hud_insets(ui.env.hud_max_aspect());
        let surface = [
            sa.top, sa.right, sa.bottom, sa.left, small[0], small[1], large[0], large[1], hud.left,
            hud.right,
        ];
        let moved = std::mem::replace(&mut self.surface, surface) != surface;
        if let Some(old) = self.viewport.replace(viewport) {
            let mut changes = viewport.changes(&old);
            // The safe area and the small and large viewports read as the
            // viewport's axes do.
            changes[0] |= moved;
            changes[1] |= moved;
            for (axis, changed) in changes.into_iter().enumerate() {
                if changed {
                    self.dirty.extend(self.media[axis].iter().cloned());
                }
            }
        }
        self.evaluated = 0;
        self.resolved = 0;
        self.css = CssWork::default();
        loop {
            let topology = &self.topology;
            let changes = self.bindings.refresh(
                &mut ui.docs.state_mut().memory.reactive,
                env,
                |node, state| {
                    topology
                        .node(node)
                        .is_some_and(|n| n.states.contains(&state))
                },
            );
            self.evaluated += changes.evaluated;
            self.pending_bindings.extend(changes.changed);
            if changes.evaluated == 0 && self.pending_bindings.is_empty() {
                break;
            }
            let pending: Vec<_> = self.pending_bindings.iter().copied().collect();
            for &id in &pending {
                if let Some((node, Input::Structure)) = self.inputs.get(&id) {
                    let node = node.clone();
                    let value = self.bindings.value(id).map_err(Clone::clone)?.clone();
                    if let Err(error) = self.refresh_branch(ui, &node, &value, env) {
                        self.rebuild_mount(ui, &node.mount);
                        return Err(error);
                    }
                }
            }
            for &id in &pending {
                if self.inputs.contains_key(&id) {
                    self.bindings.value(id).map_err(Clone::clone)?;
                }
            }
            let mut facts_changed = BTreeSet::new();
            let mut changed_uses = Vec::new();
            for id in pending {
                self.pending_bindings.remove(&id);
                let Some((node, input)) = self.inputs.get(&id) else {
                    continue;
                };
                if matches!(input, Input::Structure) {
                    continue;
                }
                let value = self.bindings.value(id).map_err(Clone::clone)?.clone();
                let element = self.elements.get_mut(node).unwrap();
                match input {
                    Input::Expression(e) => {
                        if element.document.nodes[element.index]
                            .uses
                            .iter()
                            .any(|(_, params)| params.as_ref() == Some(e))
                        {
                            changed_uses.push((node.clone(), e.clone(), value.clone()));
                        }
                        if element.document.nodes[element.index]
                            .selector_attrs
                            .iter()
                            .any(|(_, expression)| expression == e)
                        {
                            facts_changed.insert(node.clone());
                        }
                        element
                            .cached
                            .expressions
                            .iter_mut()
                            .find(|(old, _)| old == e)
                            .unwrap()
                            .1 = value
                    }
                    Input::Property(p, ty) => {
                        facts_changed.insert(node.clone());
                        element
                            .cached
                            .properties
                            .iter_mut()
                            .find(|(old, t, _)| old == p && t == ty)
                            .unwrap()
                            .2 = coerce(ui, *ty, &value)
                    }
                    Input::Structure => unreachable!(),
                }
                self.dirty.insert(node.clone());
            }
            for (node, params, value) in changed_uses {
                self.use_events(&node, "update", Some((&params, value)));
            }
            for node in facts_changed {
                let element = self.elements.remove(&node).unwrap();
                let result = self.sync_facts(ui, &element, env);
                self.elements.insert(node, element);
                result?;
            }
        }
        self.consume_selector_changes(ui)?;
        self.resolve_invalidated_styles();
        while let Some(node) = self.dirty.pop_first() {
            self.elements.get_mut(&node).unwrap().style_dirty = true;
            if self.topology.node(&node).unwrap().materialization != Materialization::Deferred {
                self.resolve(ui, &node, env)?;
            }
        }
        Ok(())
    }

    fn resolve_invalidated_styles(&mut self) {
        for (parent, range) in self.topology.take_style_invalidations() {
            let virtual_rows = parent.as_ref().and_then(|parent| {
                let element = self.elements.get(parent)?;
                matches!(
                    element.document.nodes[element.index].kind,
                    NodeKind::Element("list" | "cells")
                )
                .then_some(element.children.first())
                .flatten()
            });
            let stale: Vec<MountedNode> = match virtual_rows {
                Some(Child::Each(each)) => {
                    let each = &self.each[each];
                    each.materialized
                        .iter()
                        .filter_map(|key| each.rows.get(key))
                        .flatten()
                        .filter_map(|child| match child {
                            Child::Element(identity) => Some(identity.clone()),
                            _ => None,
                        })
                        .filter(|identity| {
                            self.topology
                                .position(identity)
                                .is_some_and(|at| range.contains(&at))
                        })
                        .collect()
                }
                _ => self
                    .topology
                    .children(parent.as_ref(), range)
                    .filter(|identity| {
                        self.topology
                            .node(identity)
                            .is_some_and(|node| node.materialization != Materialization::Deferred)
                    })
                    .cloned()
                    .collect(),
            };
            self.dirty.extend(stale);
        }
    }

    fn forced_keys(&self, ui: &Ui) -> Vec<u64> {
        if ui.interaction.state().motion.forced.is_empty() {
            return Vec::new();
        }
        self.by_key
            .keys()
            .copied()
            .filter(|&k| ui.interaction.forced_interaction(k).is_some())
            .collect()
    }

    fn sync_interaction(&mut self, ui: &mut Ui) -> Result<(), String> {
        use crate::bind::InteractionState::{
            Active, Focus, FocusVisible, FocusWithin, Hover, PopoverOpen, Target,
        };
        let mut next: [Vec<u64>; 4] = [
            vec![ui.interaction.state().hot],
            vec![if ui.interaction.input().down {
                ui.interaction.state().active
            } else {
                0
            }],
            vec![ui.interaction.state().focus],
            vec![if ui.interaction.state().focus_visible {
                ui.interaction.state().focus
            } else {
                0
            }],
        ];
        for key in self.forced_keys(ui) {
            match ui.interaction.forced_interaction(key) {
                Some("hover") => next[0].push(key),
                Some("pressed") => next[0..2].iter_mut().for_each(|s| s.push(key)),
                Some("focused") => next[2..4].iter_mut().for_each(|s| s.push(key)),
                _ => {}
            }
        }
        for keys in &mut next {
            keys.retain(|&k| k != 0);
            keys.sort_unstable();
            keys.dedup();
        }
        let previous = std::mem::replace(&mut self.interaction, next.clone());
        for (i, state) in [Hover, Active, Focus, FocusVisible].into_iter().enumerate() {
            let off = previous[i].iter().filter(|k| !next[i].contains(k));
            let on = next[i].iter().filter(|k| !previous[i].contains(k));
            for (key, enabled) in off.map(|k| (k, false)).chain(on.map(|k| (k, true))) {
                if let Some(node) = self.by_key.get(key) {
                    self.topology.set_state(node, state, enabled)?;
                    ui.docs
                        .state_mut()
                        .memory
                        .reactive
                        .graph
                        .interaction_changed(node, state);
                }
            }
        }
        let shown: Vec<(MountedNode, bool)> = self
            .popover_nodes
            .iter()
            .filter(|(_, node)| self.topology.node(node).is_some())
            .map(|(key, node)| {
                (
                    node.clone(),
                    ui.docs.state().popovers.get(key).is_some_and(|l| l.shown),
                )
            })
            .collect();
        self.popover_nodes
            .retain(|_, node| self.topology.node(node).is_some());
        for (node, open) in shown {
            if self
                .topology
                .node(&node)
                .unwrap()
                .states
                .contains(&PopoverOpen)
                != open
            {
                self.topology.set_state(&node, PopoverOpen, open)?;
                ui.docs
                    .state_mut()
                    .memory
                    .reactive
                    .graph
                    .interaction_changed(&node, PopoverOpen);
            }
        }
        if self.target.0 != ui.docs.state().target {
            let old = std::mem::take(&mut self.target.1);
            let id = ui.docs.state().target.clone();
            let new: Vec<MountedNode> = match &id {
                Some(id) => self
                    .elements
                    .iter()
                    .filter(|(_, e)| e.id == *id || e.document.nodes[e.index].id == *id)
                    .map(|(node, _)| node.clone())
                    .collect(),
                None => Vec::new(),
            };
            for (nodes, enabled) in [(&old, false), (&new, true)] {
                for node in nodes {
                    if self.topology.node(node).is_some() {
                        self.topology.set_state(node, Target, enabled)?;
                        ui.docs
                            .state_mut()
                            .memory
                            .reactive
                            .graph
                            .interaction_changed(node, Target);
                    }
                }
            }
            self.target = (id, new);
        }
        if previous[2] != next[2] {
            let off = previous[2].iter().filter(|k| !next[2].contains(k));
            let on = next[2].iter().filter(|k| !previous[2].contains(k));
            for (key, enabled) in off.map(|k| (*k, false)).chain(on.map(|k| (*k, true))) {
                let mut ancestor = self.by_key.get(&key).cloned();
                while let Some(node) = ancestor {
                    self.topology.set_state(&node, FocusWithin, enabled)?;
                    ui.docs
                        .state_mut()
                        .memory
                        .reactive
                        .graph
                        .interaction_changed(&node, FocusWithin);
                    ancestor = self.topology.parent(&node).cloned();
                }
            }
        }
        Ok(())
    }

    fn resolve(
        &mut self,
        ui: &mut Ui,
        identity: &MountedNode,
        env: &dyn Env,
    ) -> Result<(), String> {
        let parent = self.topology.parent(identity).cloned();
        if let Some(parent) = &parent
            && (self.dirty.remove(parent)
                || self.elements[parent].resolved.is_none()
                || self.elements[parent].style_dirty
                || self.elements[parent].theme_epoch != self.theme_epoch
                || self.topology.style_is_stale(parent) == Some(true))
        {
            self.resolve(ui, parent, env)?;
        }
        let structurally_stale = ui.docs.host().structure.ancestors
            && self.topology.style_is_stale(identity) == Some(true);
        let inherited = parent.as_ref().map_or_else(Arc::default, |p| {
            self.elements[p]
                .resolved
                .as_ref()
                .unwrap()
                .values
                .cascade
                .computed
                .clone()
        });
        let e = self.elements.get_mut(identity).unwrap();
        for id in &e.bindings {
            if let Some((_, Input::Property(property, Ty::Paint))) = self.inputs.get(id) {
                e.cached
                    .properties
                    .iter_mut()
                    .find(|(p, ty, _)| p == property && *ty == Ty::Paint)
                    .unwrap()
                    .2 = coerce(
                    ui,
                    Ty::Paint,
                    self.bindings.value(*id).map_err(Clone::clone)?,
                );
            }
        }
        let build = e.build(env, inherited);
        let mut probe = Built {
            path: e.path.clone(),
            doc: e.document.name.clone(),
            node: e.index,
            line: e.document.nodes[e.index].line,
            mount: identity.mount.id.clone(),
            ..Default::default()
        };
        let topology = &self.topology;
        let mut work = crate::doc::matching::MatchWork::default();
        let mut values = ui.resolve(
            &build,
            &e.document.nodes[e.index],
            &e.path,
            &mut probe,
            &build.inherited,
            &mut |selector| {
                crate::doc::matching::matches(
                    topology,
                    Some(&e.document.name),
                    selector,
                    identity,
                    &mut work,
                )
            },
        );
        values.ordinal = ui.docs.ordinal_of(
            &e.document.nodes[e.index],
            &values,
            |filter, item_values| {
                crate::doc::lists::ordinal(topology, identity, filter, item_values)
            },
        );
        self.css.selectors += work.selectors;
        self.css.candidates += work.candidates;
        ui.out
            .warnings_mut()
            .extend(values.cascade.diagnostics.iter().map(|error| {
                format!(
                    "{}:{} ({}): {error}",
                    e.document.source, e.document.nodes[e.index].line, e.path
                )
            }));
        let text = values.str("text").unwrap_or_default().to_owned();
        let previous = e.resolved.as_ref().map(|r| &r.values.cascade);
        let mut reads = values.cascade.viewport_reads;
        if e.document.nodes[e.index].kind == NodeKind::Element("field")
            && values.str("type") == Some("number")
        {
            reads.width = true;
            reads.coarse_pointer = true;
        }
        for (axis, read) in reads.axes().into_iter().enumerate() {
            if read {
                self.media[axis].insert(identity.clone());
            } else {
                self.media[axis].remove(identity);
            }
        }
        if let Some(parent) = &parent {
            if let Some(previous) = previous {
                for name in &previous.inherited_reads {
                    if let Some(readers) = self.inherited.get_mut(&(parent.clone(), name.clone())) {
                        readers.remove(identity);
                    }
                }
            }
            for name in &values.cascade.inherited_reads {
                self.inherited
                    .entry((parent.clone(), name.clone()))
                    .or_default()
                    .insert(identity.clone());
            }
        }
        if let Some(previous) = previous {
            let names: BTreeSet<_> = previous
                .computed
                .dependency_names()
                .chain(values.cascade.computed.dependency_names())
                .collect();
            for name in names {
                if previous.computed.dependency(name) != values.cascade.computed.dependency(name)
                    && let Some(readers) = self.inherited.get(&(identity.clone(), name.clone()))
                {
                    self.dirty.extend(readers.iter().cloned());
                }
            }
        }
        let resolved = ResolvedNode {
            id: e.id.clone(),
            path: e.path.clone(),
            key: identity.key,
            values,
            probe,
        };
        self.elements.get_mut(identity).unwrap().resolved = Some(resolved);
        self.emit.insert(identity.clone());
        self.elements.get_mut(identity).unwrap().style_dirty = false;
        self.elements.get_mut(identity).unwrap().theme_epoch = self.theme_epoch;
        self.topology.set_text(identity, text)?;
        self.topology.style_resolved(identity)?;
        if structurally_stale {
            // Descendant and child selectors can read this element's position.
            self.topology
                .invalidate_styles(Some(identity), 0..usize::MAX)?;
        }
        self.resolved += 1;
        Ok(())
    }
}
