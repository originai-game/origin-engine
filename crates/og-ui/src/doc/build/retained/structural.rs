use super::*;
use crate::reactive::{
    CollectionEdit, SourceChange,
    bindings::{RowRef, SourceEnv},
};

/// A `{#key}` value's hash, carried in its children's ids.
fn key_hash(value: &Value) -> u64 {
    crate::id::key_of_path(&value.to_string())
}

pub(crate) trait RetainedStructuralRetainedHost {
    fn compile_branch(
        &mut self,
        ui: &mut Ui,
        source: (Arc<UiDocument>, usize),
        context: BindingContext,
        parent: Option<&MountedNode>,
        id: String,
        env: &dyn Env,
    ) -> Result<Child, String>;

    /// A block of `source`'s children, shown while `expression` holds: an
    /// `{#if}`, a rendered snippet (always), or a snippet's definition and
    /// an optional render of no snippet (never).
    #[allow(clippy::too_many_arguments)]
    fn compile_block(
        &mut self,
        ui: &mut Ui,
        source: (Arc<UiDocument>, usize),
        context: BindingContext,
        parent: Option<&MountedNode>,
        id: String,
        expression: &Expr,
        env: &dyn Env,
    ) -> Result<Child, String>;

    fn compile_branch_children(
        &mut self,
        ui: &mut Ui,
        branch: &BranchInstance,
        env: &dyn Env,
    ) -> Result<Vec<Child>, String>;

    fn refresh_branch(
        &mut self,
        ui: &mut Ui,
        id: &MountedNode,
        value: &Value,
        env: &dyn Env,
    ) -> Result<(), String>;

    fn set_branch(
        &mut self,
        ui: &mut Ui,
        id: &MountedNode,
        enabled: bool,
        env: &dyn Env,
    ) -> Result<(), String>;

    fn block_start(&self, children: &[Child], target: &MountedNode, at: usize) -> Option<usize>;

    fn compile_each(
        &mut self,
        ui: &mut Ui,
        source: (Arc<UiDocument>, usize),
        context: BindingContext,
        parent: Option<&MountedNode>,
        id: String,
        env: &dyn Env,
    ) -> Result<Child, String>;

    fn compile_row(
        &mut self,
        ui: &mut Ui,
        each: &EachInstance,
        key: Option<&str>,
        env: &dyn Env,
    ) -> Result<Vec<Child>, String>;

    fn plan_len(&self, child: &Child) -> usize;

    fn logical_roots(&self, child: &Child, out: &mut Vec<MountedNode>);

    fn each_start(&self, id: &MountedNode) -> usize;

    fn place_row(&mut self, children: &[Child], start: usize) -> Result<(), String>;

    fn remove_child(&mut self, ui: &mut Ui, child: Child);

    fn refresh_collections(&mut self, ui: &mut Ui, env: &dyn Env) -> Result<(), String>;

    fn edit_each(
        &mut self,
        ui: &mut Ui,
        each: &mut EachInstance,
        edit: &CollectionEdit,
        start: usize,
        env: &dyn Env,
    ) -> Result<(), String>;
}

impl RetainedStructuralRetainedHost for RetainedHost {
    fn compile_branch(
        &mut self,
        ui: &mut Ui,
        source: (Arc<UiDocument>, usize),
        context: BindingContext,
        parent: Option<&MountedNode>,
        id: String,
        env: &dyn Env,
    ) -> Result<Child, String> {
        let (NodeKind::If(expression) | NodeKind::Key(expression)) = &source.0.nodes[source.1].kind
        else {
            unreachable!()
        };
        let expression = expression.clone();
        self.compile_block(ui, source, context, parent, id, &expression, env)
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_block(
        &mut self,
        ui: &mut Ui,
        source: (Arc<UiDocument>, usize),
        mut context: BindingContext,
        parent: Option<&MountedNode>,
        id: String,
        expression: &Expr,
        env: &dyn Env,
    ) -> Result<Child, String> {
        let (doc, index) = source;
        let parent = parent.ok_or("a block requires an element parent")?.clone();
        context.node.key = crate::id::child_key(parent.key, &format!("$if:{id}"));
        let identity = context.node.clone();
        let topology = &self.topology;
        let binding = self.bindings.insert(
            ui.docs.reactive_mut(),
            context.clone(),
            expression.clone(),
            env,
            |node, state| {
                topology
                    .node(node)
                    .is_some_and(|node| node.states.contains(&state))
            },
        );
        self.inputs
            .insert(binding, (identity.clone(), Input::Structure));
        let value = self.bindings.value(binding).map_err(Clone::clone)?;
        let keyed = matches!(doc.nodes[index].kind, NodeKind::Key(_)).then(|| key_hash(value));
        let enabled = keyed.is_some() || truthy(value);
        let mut branch = BranchInstance {
            document: doc,
            index,
            context,
            parent,
            suffix: id
                .rsplit('/')
                .next()
                .unwrap()
                .find('#')
                .map_or_else(String::new, |at| {
                    id.rsplit('/').next().unwrap()[at..].to_owned()
                }),
            render_prefix: id
                .rsplit_once('/')
                .map_or_else(String::new, |(prefix, _)| format!("{prefix}/")),
            binding,
            enabled,
            keyed,
            children: Vec::new(),
        };
        if enabled {
            branch.children = self.compile_branch_children(ui, &branch, env)?;
        }
        self.branches.insert(identity.clone(), branch);
        Ok(Child::Branch(identity))
    }

    fn compile_branch_children(
        &mut self,
        ui: &mut Ui,
        branch: &BranchInstance,
        env: &dyn Env,
    ) -> Result<Vec<Child>, String> {
        let mut children = Vec::new();
        for &child in &branch.document.nodes[branch.index].children {
            let n = &branch.document.nodes[child];
            let id = format!(
                "{}{}{}{}",
                branch.render_prefix,
                if n.id.is_empty() {
                    format!("node{child}")
                } else {
                    n.id.clone()
                },
                branch
                    .keyed
                    .map_or_else(String::new, |k| format!("~{:08x}", k as u32)),
                branch.suffix
            );
            children.push(self.compile(
                ui,
                (branch.document.clone(), child),
                branch.context.clone(),
                Some(&branch.parent),
                id,
                env,
            )?);
        }
        Ok(children)
    }

    fn refresh_branch(
        &mut self,
        ui: &mut Ui,
        id: &MountedNode,
        value: &Value,
        env: &dyn Env,
    ) -> Result<(), String> {
        if let Some(old) = self.branches[id].keyed {
            let key = key_hash(value);
            if old == key {
                return Ok(());
            }
            // A new value: the old content goes (its exits play) and new
            // elements are made (their entrances play), as Svelte's {#key}.
            self.set_branch(ui, id, false, env)?;
            self.branches.get_mut(id).unwrap().keyed = Some(key);
            return self.set_branch(ui, id, true, env);
        }
        self.set_branch(ui, id, truthy(value), env)
    }

    fn set_branch(
        &mut self,
        ui: &mut Ui,
        id: &MountedNode,
        enabled: bool,
        env: &dyn Env,
    ) -> Result<(), String> {
        if self.branches[id].enabled == enabled {
            return Ok(());
        }
        let parent = self.branches[id].parent.clone();
        let start = self
            .block_start(&self.elements[&parent].children, id, 0)
            .expect("mounted structural child");
        self.emit.insert(parent.clone());
        let mut branch = self.branches.remove(id).unwrap();
        let result = if enabled {
            self.compile_branch_children(ui, &branch, env)
                .and_then(|children| {
                    self.place_row(&children, start)?;
                    branch.children = children;
                    Ok(())
                })
        } else {
            for child in std::mem::take(&mut branch.children) {
                self.remove_child(ui, child);
            }
            Ok(())
        };
        branch.enabled = enabled;
        self.branches.insert(id.clone(), branch);
        result
    }

    fn block_start(
        &self,
        children: &[Child],
        target: &MountedNode,
        mut at: usize,
    ) -> Option<usize> {
        for child in children {
            match child {
                Child::Each(id) => {
                    if id == target {
                        return Some(at);
                    }
                    let each = &self.each[id];
                    let mut row_at = at;
                    for key in &each.order {
                        if let Some(at) = self.block_start(&each.rows[key], target, row_at) {
                            return Some(at);
                        }
                        row_at += each.rows[key]
                            .iter()
                            .map(|child| self.plan_len(child))
                            .sum::<usize>();
                    }
                    if let Some(at) = self.block_start(&each.empty, target, row_at) {
                        return Some(at);
                    }
                }
                Child::Branch(id) => {
                    if id == target {
                        return Some(at);
                    }
                    if let Some(at) = self.block_start(&self.branches[id].children, target, at) {
                        return Some(at);
                    }
                }
                Child::Element(_) => {}
            }
            at += self.plan_len(child);
        }
        None
    }

    fn compile_each(
        &mut self,
        ui: &mut Ui,
        source: (Arc<UiDocument>, usize),
        mut context: BindingContext,
        parent: Option<&MountedNode>,
        id: String,
        env: &dyn Env,
    ) -> Result<Child, String> {
        let (doc, index) = source;
        let parent = parent.ok_or("each requires an element parent")?.clone();
        let NodeKind::For {
            list: Expr::Path(path),
            ..
        } = &doc.nodes[index].kind
        else {
            return Err(format!(
                "{}:{}: each must read a named keyed collection",
                doc.source, doc.nodes[index].line
            ));
        };
        let source = super::content::collection(&context, path)?;
        let collection = ui.docs.reactive().sources.keyed(&source)?;
        let order = collection.keys().to_vec();
        let revision = collection.revision();
        context.node.key = crate::id::child_key(parent.key, &format!("$each:{id}"));
        let identity = context.node.clone();
        let mut each = EachInstance {
            document: doc,
            index,
            context,
            parent,
            source: source.clone(),
            revision,
            order,
            rows: BTreeMap::new(),
            empty: Vec::new(),
            materialized: BTreeSet::new(),
        };
        for key in each.order.clone() {
            let children = self.compile_row(ui, &each, Some(&key), env)?;
            each.rows.insert(key, children);
        }
        if each.order.is_empty() {
            each.empty = self.compile_row(ui, &each, None, env)?;
        }
        self.collections
            .entry(source)
            .or_default()
            .insert(identity.clone());
        self.each.insert(identity.clone(), each);
        Ok(Child::Each(identity))
    }

    fn compile_row(
        &mut self,
        ui: &mut Ui,
        each: &EachInstance,
        key: Option<&str>,
        env: &dyn Env,
    ) -> Result<Vec<Child>, String> {
        let node = &each.document.nodes[each.index];
        let NodeKind::For {
            var,
            key: expression,
            empty,
            ..
        } = &node.kind
        else {
            unreachable!()
        };
        let mut context = each.context.clone();
        if let Some(key) = key {
            context.row_key = key.into();
            Arc::make_mut(&mut context.rows).insert(
                var.clone(),
                RowRef {
                    source: each.source.clone(),
                    key: key.into(),
                },
            );
            let result = SourceEnv {
                session: ui.docs.reactive(),
                inputs: &context.inputs,
                rows: &context.rows,
                selector_nodes: &context.selector_nodes,
                lexical: &context.lexical,
                host: env,
                node: &context.node,
                interaction: &|_, _| false,
            }
            .evaluate(expression)
            .value?;
            if result.as_str() != Some(key) {
                return Err(format!(
                    "{}:{}: each key must match its collection's stable key '{key}'",
                    each.document.source, node.line
                ));
            }
        }
        let mut children = Vec::new();
        for &child in if key.is_some() { &node.children } else { empty } {
            let n = &each.document.nodes[child];
            let mut id = if n.id.is_empty() {
                format!("node{child}")
            } else {
                n.id.clone()
            };
            if let Some(key) = key {
                let escaped = key
                    .replace('%', "%25")
                    .replace('/', "%2F")
                    .replace('#', "%23");
                id.push('#');
                id.push_str(&escaped);
                let parent = &self.elements[&each.parent];
                if let NodeKind::Element(kind @ ("list" | "cells")) =
                    parent.document.nodes[parent.index].kind
                {
                    id = format!("{}/{id}", crate::virtualize::item_id(key, kind == "cells"));
                }
            }
            children.push(self.compile(
                ui,
                (each.document.clone(), child),
                context.clone(),
                Some(&each.parent),
                id,
                env,
            )?);
        }
        Ok(children)
    }

    fn plan_len(&self, child: &Child) -> usize {
        match child {
            Child::Element(_) => 1,
            Child::Each(id) => self.each[id]
                .rows
                .values()
                .flatten()
                .chain(&self.each[id].empty)
                .map(|child| self.plan_len(child))
                .sum(),
            Child::Branch(id) => self.branches[id]
                .children
                .iter()
                .map(|child| self.plan_len(child))
                .sum(),
        }
    }

    fn logical_roots(&self, child: &Child, out: &mut Vec<MountedNode>) {
        match child {
            Child::Element(id) => out.push(id.clone()),
            Child::Branch(id) => {
                for child in &self.branches[id].children {
                    self.logical_roots(child, out);
                }
            }
            Child::Each(id) => {
                let each = &self.each[id];
                for key in &each.order {
                    for child in &each.rows[key] {
                        self.logical_roots(child, out);
                    }
                }
                if each.order.is_empty() {
                    for child in &each.empty {
                        self.logical_roots(child, out);
                    }
                }
            }
        }
    }

    fn each_start(&self, id: &MountedNode) -> usize {
        let parent = &self.each[id].parent;
        self.block_start(&self.elements[parent].children, id, 0)
            .expect("mounted each position")
    }

    fn place_row(&mut self, children: &[Child], start: usize) -> Result<(), String> {
        let mut roots = Vec::new();
        for child in children {
            self.logical_roots(child, &mut roots);
        }
        let forward = roots
            .first()
            .and_then(|id| self.topology.position(id))
            .is_some_and(|at| at < start);
        let indexes: Box<dyn Iterator<Item = usize>> = if forward {
            Box::new((0..roots.len()).rev())
        } else {
            Box::new(0..roots.len())
        };
        for i in indexes {
            self.topology.move_to(&roots[i], start + i)?;
        }
        Ok(())
    }

    fn remove_child(&mut self, ui: &mut Ui, child: Child) {
        match child {
            Child::Branch(id) => {
                let branch = self.branches.remove(&id).expect("mounted branch");
                for child in branch.children {
                    self.remove_child(ui, child);
                }
                self.bindings.remove(ui.docs.reactive_mut(), branch.binding);
                self.inputs.remove(&branch.binding);
                self.pending_bindings.remove(&branch.binding);
            }
            Child::Element(id) => {
                self.use_events(&id, "destroy", None);
                let parent = self.topology.parent(&id).cloned();
                if let Some(parent) = &parent {
                    self.emit.insert(parent.clone());
                }
                self.release_element(ui, &id);
                self.emit.remove(&id);
                self.volatile.remove(&id);
                self.on_action.retain(|_, listeners| {
                    listeners.remove(&id);
                    !listeners.is_empty()
                });
                let element = self.elements.remove(&id).expect("mounted element");
                for child in element.children {
                    self.remove_child(ui, child);
                }
                for binding in element.bindings {
                    self.bindings.remove(ui.docs.reactive_mut(), binding);
                    self.inputs.remove(&binding);
                    self.pending_bindings.remove(&binding);
                }
                self.topology.remove(&id).expect("mounted element topology");
                self.by_key.remove(&id.key);
                ui.interaction.state_mut().virtual_lists.remove(&id.key);
                self.dirty.remove(&id);
                for readers in &mut self.media {
                    readers.remove(&id);
                }
                if let Some(resolved) = element.resolved {
                    for name in resolved.values.cascade.inherited_reads {
                        if let Some(parent) = &parent
                            && let Some(readers) = self.inherited.get_mut(&(parent.clone(), name))
                        {
                            readers.remove(&id);
                        }
                    }
                }
            }
            Child::Each(id) => {
                let each = self.each.remove(&id).expect("mounted each");
                for child in each.rows.into_values().flatten().chain(each.empty) {
                    self.remove_child(ui, child);
                }
                if let Some(readers) = self.collections.get_mut(&each.source) {
                    readers.remove(&id);
                }
            }
        }
    }

    fn refresh_collections(&mut self, ui: &mut Ui, env: &dyn Env) -> Result<(), String> {
        let mut error = None;
        for change in ui.docs.reactive_mut().sources.take_changes() {
            let SourceChange::Collection(edit) = change.change else {
                continue;
            };
            if let CollectionEdit::Patch { key, .. } = &edit {
                if let Some(readers) = self.collections.get(&change.source) {
                    for id in readers {
                        if let Some(list) = ui
                            .interaction
                            .state_mut()
                            .virtual_lists
                            .get_mut(&self.each[id].parent.key)
                        {
                            list.invalidate(std::slice::from_ref(key));
                        }
                    }
                }
                continue;
            }
            let readers = self
                .collections
                .get(&change.source)
                .cloned()
                .unwrap_or_default();
            for id in readers {
                if self
                    .each
                    .get(&id)
                    .is_none_or(|each| each.revision >= change.revision)
                {
                    continue;
                }
                let start = self.each_start(&id);
                self.emit.insert(self.each[&id].parent.clone());
                let mut each = self.each.remove(&id).unwrap();
                let result = self.edit_each(ui, &mut each, &edit, start, env);
                each.revision = change.revision;
                self.each.insert(id.clone(), each);
                if let Err(reason) = result {
                    self.rebuild_mount(ui, &id.mount);
                    error.get_or_insert(reason);
                }
            }
        }
        error.map_or(Ok(()), Err)
    }

    fn edit_each(
        &mut self,
        ui: &mut Ui,
        each: &mut EachInstance,
        edit: &CollectionEdit,
        start: usize,
        env: &dyn Env,
    ) -> Result<(), String> {
        match edit {
            CollectionEdit::Insert { key, before, .. } => {
                if each.rows.contains_key(key)
                    || ui
                        .docs
                        .reactive()
                        .sources
                        .keyed(&each.source)?
                        .row(key)
                        .is_none()
                {
                    return Ok(());
                }
                if each.order.is_empty() {
                    for child in std::mem::take(&mut each.empty) {
                        self.remove_child(ui, child);
                    }
                }
                let at = before
                    .as_ref()
                    .and_then(|before| each.order.iter().position(|key| key == before))
                    .unwrap_or(each.order.len());
                let offset: usize = each.order[..at]
                    .iter()
                    .flat_map(|key| &each.rows[key])
                    .map(|child| self.plan_len(child))
                    .sum();
                let children = self.compile_row(ui, each, Some(key), env)?;
                self.place_row(&children, start + offset)?;
                each.order.insert(at, key.clone());
                each.rows.insert(key.clone(), children);
            }
            CollectionEdit::Remove { key, .. } => {
                if let Some(children) = each.rows.remove(key) {
                    for child in children {
                        self.remove_child(ui, child);
                    }
                    each.order.retain(|old| old != key);
                    if each.order.is_empty() {
                        each.empty = self.compile_row(ui, each, None, env)?;
                        self.place_row(&each.empty, start)?;
                    }
                }
            }
            CollectionEdit::Move { key, before, .. } => {
                if !each.rows.contains_key(key) {
                    return Ok(());
                }
                each.order.retain(|old| old != key);
                let at = before
                    .as_ref()
                    .and_then(|before| each.order.iter().position(|key| key == before))
                    .unwrap_or(each.order.len());
                let offset: usize = each.order[..at]
                    .iter()
                    .flat_map(|key| &each.rows[key])
                    .map(|child| self.plan_len(child))
                    .sum();
                self.place_row(&each.rows[key], start + offset)?;
                each.order.insert(at, key.clone());
            }
            CollectionEdit::Patch { .. } => {}
        }
        Ok(())
    }
}
