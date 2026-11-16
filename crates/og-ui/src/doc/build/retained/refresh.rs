use super::*;
use crate::doc::schema::Kind;

impl RetainedHost {
    pub(crate) fn sync_facts(
        &mut self,
        ui: &mut Ui,
        element: &ElementInstance,
        env: &dyn Env,
    ) -> Result<(), String> {
        use crate::bind::InteractionState::*;
        let identity = &element.context.node;
        let build = element.build(env, Arc::default());
        let node = &element.document.nodes[element.index];
        for (name, value) in crate::doc::build::structure::selector_facts(node, |e| build.eval(e)) {
            self.topology.set_attribute(identity, &name, Some(value))?;
        }
        for (name, expression) in super::content::derived(node) {
            self.topology
                .set_attribute(identity, name, Some(build.eval(&expression)))?;
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
        let own = &ui.docs.state().memory;
        if let Some(value) = own.values.get(&identity.key) {
            facts.insert("input", json!(value));
        }
        if let Some(open) = own.open.get(&identity.key) {
            facts.insert("open", json!(open));
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
            NodeKind::Element(kind) => Some(kind),
            _ => None,
        };
        // HTML's form-associated elements: they are enabled or disabled,
        // valid or invalid; text fields are read-write unless readonly.
        let form = matches!(
            kind,
            Some(
                Kind::Button
                    | Kind::Field
                    | Kind::Slider
                    | Kind::Toggle
                    | Kind::Radio
                    | Kind::Select
                    | Kind::Color
            )
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
                if kind == Some(Kind::Slider)
                    || (kind == Some(Kind::Field) && (min.is_some() || max.is_some())) =>
            {
                Some(min.is_none_or(|m| v >= m) && max.is_none_or(|m| v <= m))
            }
            _ => None,
        };
        let editable = kind == Some(Kind::Field) && !flag("readonly") && !disabled;
        let changed = facts
            .get("input")
            .and_then(Value::as_str)
            .is_some_and(|v| !v.is_empty());
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
            // After the player changed it: a field with text it refuses,
            // or one with text it takes.
            (UserInvalid, invalid && changed),
            (UserValid, form && !invalid && changed),
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

    pub(crate) fn refresh(&mut self, ui: &mut Ui, env: &dyn Env) -> Result<(), String> {
        self.restore_instances(ui, env)?;
        self.refresh_collections(ui, env)?;
        self.sync_interaction(ui)?;
        for key in std::mem::take(&mut ui.docs.state_mut().edited) {
            let Some(node) = self.by_key.get(&key).cloned() else {
                continue;
            };
            let Some(element) = self.elements.remove(&node) else {
                continue;
            };
            let result = self.sync_facts(ui, &element, env);
            self.elements.insert(node.clone(), element);
            result?;
            self.dirty.insert(node);
        }
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
        let scrolled = std::mem::take(&mut ui.docs.containers.scroll_changed);
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
            let (topology, pseudo) = (&self.topology, &self.pseudo_nodes);
            let changes = self.bindings.refresh(
                &mut ui.docs.state_mut().memory.reactive,
                env,
                |node, state| in_state(topology, pseudo, node, state),
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
                    let result = if self.each.contains_key(&node) {
                        self.refresh_held(ui, &node, &value, env)
                    } else {
                        self.refresh_branch(ui, &node, &value, env)
                    };
                    if let Err(error) = result {
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
                        let written = &element.document.nodes[element.index];
                        if written
                            .selector_attrs
                            .iter()
                            .any(|(_, expression)| expression == e)
                            || super::content::derived(written).iter().any(|(_, d)| d == e)
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
                // A control document's root reads its `:host` rules'
                // conditions for its host.
                let root = &self.elements[node];
                if root.document.control.is_some()
                    && root.index == root.document.root
                    && let Some(host) = self.topology.parent(node)
                {
                    self.dirty.insert(host.clone());
                }
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

    /// Materialized nodes in invalidated style intervals resolve before they
    /// draw; deferred rows keep their interval until they materialize.
    pub(crate) fn resolve_invalidated_styles(&mut self) {
        for (parent, range) in self.topology.take_style_invalidations() {
            let virtual_rows = parent.as_ref().and_then(|parent| {
                let element = self.elements.get(parent)?;
                matches!(
                    element.document.nodes[element.index].kind,
                    NodeKind::Element(Kind::List | Kind::Cells)
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
            // An element resolved since the interval was invalidated (a new
            // child, resolved after its parent) already reads its position.
            let stale = stale
                .into_iter()
                .filter(|identity| self.topology.style_is_stale(identity) != Some(false));
            self.dirty.extend(stale);
        }
    }

    /// The mounted elements whose interaction state the probe forced.
    pub(crate) fn forced_keys(&self, ui: &Ui) -> Vec<u64> {
        if ui.interaction.state().motion.forced.is_empty() {
            return Vec::new();
        }
        self.by_key
            .keys()
            .copied()
            .filter(|&k| ui.interaction.forced_interaction(k).is_some())
            .collect()
    }

    pub(crate) fn sync_interaction(&mut self, ui: &mut Ui) -> Result<(), String> {
        use crate::bind::InteractionState::{
            Active, ActiveViewTransition, Focus, FocusVisible, FocusWithin, Hover, PopoverOpen,
            Target,
        };
        // The boxes styled as pseudo-elements since the last refresh.
        for (styled, pseudo) in std::mem::take(&mut ui.docs.state_mut().pseudo_boxes) {
            if self.pseudo_nodes.contains_key(&pseudo)
                || ui.docs.state().instances.contains_key(&pseudo)
            {
                self.pseudo_boxes.insert(styled, pseudo);
            }
        }
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
            let changed: Vec<(u64, bool)> = off
                .map(|k| (*k, false))
                .chain(on.map(|k| (*k, true)))
                .collect();
            for (key, enabled) in changed {
                self.set_key_state(ui, key, state, enabled)?;
            }
        }
        for (key, state, enabled) in std::mem::take(&mut ui.docs.state_mut().selector_states) {
            self.set_key_state(ui, key, state, enabled)?;
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
        // A layer's screen root is the document element of its view
        // transition: it matches from the frame the transition starts to
        // the frame it ends.
        let transitioning: Vec<MountedNode> = ui
            .docs
            .layers
            .layers
            .iter()
            .filter(|l| l.since != 0)
            .filter_map(|l| match self.roots.get(&l.mount) {
                Some(Child::Element(node)) => Some(node.clone()),
                _ => None,
            })
            .collect();
        if transitioning != self.transitioning {
            let old = std::mem::replace(&mut self.transitioning, transitioning.clone());
            let off = old.iter().filter(|n| !transitioning.contains(n));
            let on = transitioning.iter().filter(|n| !old.contains(n));
            for (node, enabled) in off.map(|n| (n, false)).chain(on.map(|n| (n, true))) {
                if self.topology.node(node).is_some() {
                    self.topology
                        .set_state(node, ActiveViewTransition, enabled)?;
                    ui.docs
                        .state_mut()
                        .memory
                        .reactive
                        .graph
                        .interaction_changed(node, ActiveViewTransition);
                }
            }
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

    /// An inherited value `node` hands down without declaring it (a
    /// custom property, ` text color`) is read through it: `node` reads it
    /// from its parent too, and so on up to the element that declares it,
    /// so a change there restyles what reads it below.
    fn read_through(&mut self, mut node: MountedNode, name: &str) {
        while let Some(parent) = self.topology.parent(&node) {
            // Read through already: a restyle mostly finds its reads kept.
            if self
                .inherited
                .get(parent)
                .and_then(|names| names.get(name))
                .is_some_and(|readers| readers.contains(&node))
            {
                break;
            }
            let computed = |n: &MountedNode| {
                self.elements
                    .get(n)
                    .and_then(|e| e.resolved.as_ref())
                    .map(|r| &r.values.cascade.computed)
            };
            let passes = match (computed(&node), computed(parent)) {
                (Some(own), Some(above)) => own.dependency(name) == above.dependency(name),
                _ => false,
            };
            if !passes {
                break;
            }
            let parent = parent.clone();
            self.inherited
                .entry(parent.clone())
                .or_default()
                .entry(name.to_owned())
                .or_default()
                .insert(node);
            node = parent;
        }
    }

    /// Whether `identity` is an inline element of a text.
    pub(crate) fn is_inline(&self, identity: &MountedNode) -> bool {
        self.elements.get(identity).is_some_and(|e| {
            e.document.nodes[e.index].kind == NodeKind::Element(crate::doc::schema::Kind::Inline)
        })
    }

    /// Text `identity`'s inline elements, resolved where they are stale,
    /// by document node, for its build to lower ([`Build::inline`]).
    pub(crate) fn resolve_inline(
        &mut self,
        ui: &mut Ui,
        identity: &MountedNode,
        env: &dyn Env,
    ) -> Result<Vec<(usize, Arc<ResolvedNode>)>, String> {
        let mut out = Vec::new();
        let mut stack: Vec<MountedNode> = self.elements[identity]
            .children
            .iter()
            .rev()
            .filter_map(|c| match c {
                Child::Element(e) => Some(e.clone()),
                _ => None,
            })
            .collect();
        while let Some(child) = stack.pop() {
            if !self.is_inline(&child) {
                continue;
            }
            let e = &self.elements[&child];
            if self.dirty.remove(&child)
                || e.resolved.is_none()
                || e.style_dirty
                || e.theme_epoch != self.theme_epoch
                || self.topology.style_is_stale(&child) == Some(true)
            {
                self.resolve(ui, &child, env)?;
            }
            let e = &self.elements[&child];
            out.push((e.index, e.resolved.clone().expect("resolved")));
            stack.extend(e.children.iter().rev().filter_map(|c| match c {
                Child::Element(e) => Some(e.clone()),
                _ => None,
            }));
        }
        // Resolving them marked the text, which is building now.
        if !out.is_empty() {
            self.emit.remove(identity);
        }
        Ok(out)
    }

    pub(crate) fn resolve(
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
        let structurally_stale = ui.docs.library.structure.ancestors
            && self.topology.style_is_stale(identity) == Some(true);
        // A frame's document inherits nothing from the page around it, as
        // an iframe's document does not.
        let in_frame = |p: &MountedNode| {
            let (e, frame) = (&self.elements[identity], &self.elements[p]);
            e.index == e.document.root && frame.document.nodes[frame.index].frame.is_some()
        };
        let inherited = match &parent {
            Some(p) if in_frame(p) => Default::default(),
            Some(p) => self.elements[p]
                .resolved
                .as_ref()
                .unwrap()
                .values
                .cascade
                .computed
                .clone(),
            // An element image's root inherits what the filtered element's
            // build gave it.
            None => self
                .images
                .get(identity)
                .map(|image| image.inherited.clone())
                .unwrap_or_default(),
        };
        // Content rendered at a `{@render}` site of another document's
        // element takes that document's `::slotted()` rules too.
        let slotted = parent.as_ref().and_then(|parent| {
            let holder = &self.elements[parent].document;
            let passed = self.topology.writer(identity) != Some(&holder.name);
            (passed && !holder.slotted_rules.is_empty()).then(|| holder.clone())
        });
        // Rules of another document that reach this element (docs/ui.md
        // §2.4). A control's host takes its control document's `:host`
        // rules that hold, under its own document's rules (depth -1). A
        // part of a control document takes the `::part()` rules written
        // for its host, over its own document's (depth +1).
        let mut reaching: Vec<crate::doc::StyleRule> = Vec::new();
        let own = &self.elements[identity];
        if let Some(root) = own.control.as_ref().and_then(|r| self.elements.get(r)) {
            reaching.extend(
                root.document
                    .host_rules
                    .iter()
                    .filter(|rule| {
                        rule.condition
                            .as_ref()
                            .is_none_or(|c| truthy(&root.cached.expression(c)))
                    })
                    .map(|rule| crate::doc::StyleRule {
                        condition: None,
                        ..rule.clone()
                    }),
            );
        }
        // A part is reached from its host's document, and from each
        // document further out that its host exports it to (`exportparts`,
        // CSS Shadow Parts §4.2), one boundary deeper each time.
        if own.document.control.is_some()
            && let Some(part) = own.document.nodes[own.index].part_name()
        {
            let mut names: Vec<String> = part.split_ascii_whitespace().map(str::to_owned).collect();
            let mut inside = own;
            let mut depth = 1;
            while !names.is_empty()
                && inside.document.control.is_some()
                && let Some(host) = inside.context.selector_nodes.get(&crate::doc::HOST_NODE)
                && let (Some(element), Some(facts)) =
                    (self.elements.get(host), self.topology.node(host))
            {
                let node = &element.document.nodes[element.index];
                let tag = node.selector_tag.as_deref().unwrap_or_default();
                for name in &names {
                    reaching.extend(ui.control_part_rules(
                        &element.document,
                        tag,
                        name,
                        facts.attributes,
                        facts.states,
                        depth,
                    ));
                }
                names = node
                    .exported_parts()
                    .into_iter()
                    .filter(|(inner, _)| names.contains(inner))
                    .map(|(_, outer)| outer)
                    .collect();
                inside = element;
                depth += 1;
            }
        }
        let e = self.elements.get_mut(identity).unwrap();
        // An instance's root inherits the instance's style props as if a
        // box around it set them: its own rules and `style:` win. Null (or
        // a list) leaves the property as inherited.
        let mut inherited = inherited;
        if e.index == e.document.root {
            for (name, _) in super::content::style_props(&e.context) {
                let text = match e.cached.expression(&Expr::Path(name.clone())) {
                    Value::String(s) => s,
                    Value::Number(x) => x.to_string(),
                    Value::Bool(x) => x.to_string(),
                    _ => continue,
                };
                Arc::make_mut(&mut inherited).insert(name.clone(), Some(text));
            }
        }
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
        // An element styled once keeps no cascade; one styled again keeps
        // its last ones.
        // A new theme (or direction) drops them: the texts kept do not
        // name the theme they read.
        let mut kept = e
            .cascades
            .take()
            .filter(|_| e.theme_epoch == self.theme_epoch);
        if e.resolved.is_some() {
            kept.get_or_insert_default();
        }
        let build = e.build(env, inherited);
        let siblings = match self.topology.position(identity) {
            Some(at) if ui.docs.library.uses_siblings => [
                at as u32 + 1,
                self.topology.child_count(parent.as_ref()) as u32,
            ],
            _ => [0; 2],
        };
        let mut probe = Built {
            path: e.path.clone(),
            doc: e.document.name.clone(),
            node: e.index,
            line: e.document.nodes[e.index].line,
            mount: identity.mount.id.clone(),
            siblings,
            ..Default::default()
        };
        let topology = &self.topology;
        let mut work = crate::doc::matching::MatchWork::default();
        let mut matcher =
            crate::doc::matching::Matcher::new(topology, Some(&e.document.name), identity);
        // The node with the rules of the document it is rendered in: a
        // rule list of its own, so no kept cascade is its.
        // On a plain element they rank under its own document's rules
        // (their depth, -1). A component instance is its user's content,
        // and its component's rules for its root are the inner ones, as
        // a browser ranks `:host` rules under `::slotted()`.
        let instance = topology.writer(identity) != Some(&e.document.name);
        let passed = (slotted.is_some() || !reaching.is_empty()).then(|| {
            let mut node = e.document.nodes[e.index].clone();
            if let Some(holder) = &slotted {
                node.rules
                    .extend(holder.slotted_rules.iter().cloned().map(|rule| {
                        crate::doc::StyleRule {
                            depth: if instance { 1 } else { rule.depth },
                            ..rule
                        }
                    }));
            }
            node.rules.append(&mut reaching);
            node
        });
        let mut values = ui.resolve(
            &build,
            passed.as_ref().unwrap_or(&e.document.nodes[e.index]),
            &e.path,
            &mut probe,
            &build.inherited,
            &mut |selector| matcher.matches(selector, &mut work),
            match passed {
                Some(_) => None,
                None => kept.as_mut().map(Arc::make_mut),
            },
            None,
        );
        e.cascades = kept;
        values.ordinal = ui.docs.ordinal_of(
            &e.document.nodes[e.index],
            &values,
            |filter, item_values| {
                crate::doc::lists::ordinal(topology, identity, filter, item_values)
            },
        );
        self.css.selectors += work.selectors;
        self.css.candidates += work.candidates;
        let bursts = ui.burst_errors(&values);
        let said: Vec<String> = values
            .cascade
            .diagnostics
            .iter()
            .chain(&bursts)
            .map(|error| {
                format!(
                    "{}:{} ({}): {error}",
                    e.document.source, e.document.nodes[e.index].line, e.path
                )
            })
            .collect();
        if said.is_empty() {
            self.diagnostics.remove(identity);
        } else {
            self.diagnostics.insert(identity.clone(), said);
        }
        let text = values.str("text").unwrap_or_default().to_owned();
        let previous = e.resolved.as_ref().map(|r| &r.values.cascade);
        let mut reads = values.cascade.viewport_reads;
        if e.document.nodes[e.index].kind == NodeKind::Element(Kind::Field)
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
            // Only the reads that changed move: a restyle mostly reads what
            // it read before.
            let empty = BTreeSet::new();
            let before = previous.map_or(&empty, |p| &p.inherited_reads);
            let now = &values.cascade.inherited_reads;
            if let Some(names) = self.inherited.get_mut(parent) {
                for name in before.difference(now) {
                    if let Some(readers) = names.get_mut(name) {
                        readers.remove(identity);
                    }
                }
            }
            for name in now.difference(before) {
                self.inherited
                    .entry(parent.clone())
                    .or_default()
                    .entry(name.clone())
                    .or_default()
                    .insert(identity.clone());
            }
        }
        let added: Vec<String> = match &parent {
            Some(_) => values
                .cascade
                .inherited_reads
                .iter()
                .filter(|name| previous.is_none_or(|p| !p.inherited_reads.contains(*name)))
                .cloned()
                .collect(),
            None => Vec::new(),
        };
        // The inherited values it hands down that changed (`None`: its
        // first style, every one).
        let changed: Option<Vec<String>> = previous.map(|previous| {
            previous
                .computed
                .changed(&values.cascade.computed)
                .into_iter()
                .cloned()
                .collect()
        });
        for name in changed.iter().flatten() {
            if let Some(readers) = self.inherited.get(identity).and_then(|n| n.get(name)) {
                self.dirty.extend(readers.iter().cloned());
            }
        }
        // Whether it declares what it declared: then a value it hands down
        // starts to pass through it only as the value changes.
        let declares_as_before = previous.is_some_and(|p| {
            let c = &values.cascade;
            fn names(d: &[(String, String)]) -> impl Iterator<Item = &String> {
                d.iter().map(|(name, _)| name)
            }
            p.local_names == c.local_names
                && p.values
                    .iter()
                    .map(|v| &v.0)
                    .eq(c.values.iter().map(|v| &v.0))
                && names(&p.text).eq(names(&c.text))
                && names(&p.svg).eq(names(&c.svg))
                && names(&p.container).eq(names(&c.container))
        });
        let resolved = ResolvedNode {
            id: e.id.clone(),
            path: e.path.clone(),
            key: identity.key,
            values,
            probe,
        };
        self.elements.get_mut(identity).unwrap().resolved = Some(Arc::new(resolved));
        if let Some(parent) = &parent {
            for name in &added {
                self.read_through(parent.clone(), name);
            }
        }
        // What its children read through it, now that it may hand down
        // what it declared before.
        let read: Vec<String> = self.inherited.get(identity).map_or_else(Vec::new, |names| {
            let reads = |name: &String| names.get(name).is_some_and(|r| !r.is_empty());
            match changed {
                Some(changed) if declares_as_before => {
                    changed.into_iter().filter(|name| reads(name)).collect()
                }
                _ => names
                    .iter()
                    .filter(|(_, readers)| !readers.is_empty())
                    .map(|(name, _)| name.clone())
                    .collect(),
            }
        });
        for name in read {
            self.read_through(identity.clone(), &name);
        }
        // An inline element is built by its text: the text builds again.
        let mut built = identity.clone();
        while self.is_inline(&built)
            && let Some(parent) = self.topology.parent(&built)
        {
            built = parent.clone();
        }
        self.emit.insert(built);
        self.elements.get_mut(identity).unwrap().style_dirty = false;
        self.elements.get_mut(identity).unwrap().theme_epoch = self.theme_epoch;
        self.topology.set_text(identity, text)?;
        self.topology.style_resolved(identity)?;
        // The parts of its control document, and those of nested controls
        // that may be exported to it, match `::part()` rules by its facts:
        // they are styled again after it.
        if let Some(root) = self.elements[identity].control.clone() {
            let mut pending = vec![root];
            while let Some(at) = pending.pop() {
                let Some(element) = self.elements.get(&at) else {
                    continue;
                };
                if element.document.control.is_some()
                    && element.document.nodes[element.index].part_name().is_some()
                {
                    self.dirty.insert(at.clone());
                }
                pending.extend(self.topology.children(Some(&at), 0..usize::MAX).cloned());
            }
        }
        if structurally_stale {
            // Descendant and child selectors can read this element's position.
            self.topology
                .invalidate_styles(Some(identity), 0..usize::MAX)?;
        }
        self.resolved += 1;
        Ok(())
    }
}
