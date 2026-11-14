use super::*;
use crate::doc::schema::Kind;

impl RetainedHost {
    pub(crate) fn compile(
        &mut self,
        ui: &mut Ui,
        source: (Arc<UiDocument>, usize),
        mut context: BindingContext,
        parent: Option<&MountedNode>,
        id: String,
        env: &dyn Env,
    ) -> Result<Child, String> {
        let (doc, index) = source;
        let node = &doc.nodes[index];
        if matches!(node.kind, NodeKind::For { .. }) {
            return self.compile_each(ui, (doc, index), context, parent, id, env);
        }
        if matches!(node.kind, NodeKind::If(_) | NodeKind::Key(_)) {
            return self.compile_branch(ui, (doc, index), context, parent, id, env);
        }
        match &node.kind {
            NodeKind::Use(component) => {
                let component = component.clone();
                return self.compile_component(
                    ui,
                    (doc, index),
                    &component,
                    context,
                    parent,
                    id,
                    env,
                );
            }
            NodeKind::Render { name, optional } => {
                let (name, optional) = (name.clone(), *optional);
                return self.compile_render(
                    ui,
                    (doc, index),
                    &name,
                    optional,
                    context,
                    parent,
                    id,
                    env,
                );
            }
            // A definition renders nothing where it is written.
            NodeKind::Snippet(_) => {
                return self.compile_block(
                    ui,
                    (doc, index),
                    context,
                    parent,
                    id,
                    &Expr::Lit(false.into()),
                    env,
                );
            }
            _ => {}
        }
        let NodeKind::Element(_) = node.kind else {
            return Err(format!(
                "{}:{}: structural mount compilation is not connected yet",
                doc.source, node.line
            ));
        };
        let path = parent.map_or_else(|| id.clone(), |p| format!("{}/{id}", self.content_path(p)));
        context.node.key = crate::id::key_of_path(&path);
        Arc::make_mut(&mut context.selector_nodes).insert(index, context.node.clone());
        let identity = context.node.clone();
        let materialization = parent.map_or(Materialization::Visible, |parent| {
            let element = &self.elements[parent];
            if matches!(
                element.document.nodes[element.index].kind,
                NodeKind::Element(Kind::List | Kind::Cells)
            ) {
                Materialization::Deferred
            } else {
                self.topology.node(parent).unwrap().materialization
            }
        });
        self.by_key.insert(identity.key, identity.clone());
        self.topology.insert(
            parent,
            self.topology.child_count(parent),
            Instance {
                identity: identity.clone(),
                document: doc.clone(),
                document_node: index,
                attributes: BTreeMap::new(),
                direct_text: String::new(),
                states: BTreeSet::new(),
                materialization,
            },
        )?;
        // What its id has before its own name: the snippets it is rendered
        // from, after the item node a virtual list writes around its row.
        let scope = parent.and_then(|parent| {
            let (before, _) = id.rsplit_once('/')?;
            let list = &self.elements[parent];
            if matches!(
                list.document.nodes[list.index].kind,
                NodeKind::Element(Kind::List | Kind::Cells)
            ) {
                before.split_once('/').map(|(_, scope)| scope.to_owned())
            } else {
                Some(before.to_owned())
            }
        });
        let mut element = ElementInstance {
            document: doc.clone(),
            index,
            context,
            path,
            id: id.rsplit('/').next().unwrap().to_owned(),
            scope,
            cached: CachedInputs::default(),
            resolved: None,
            style_dirty: true,
            theme_epoch: self.theme_epoch,
            children: Vec::new(),
            bindings: Vec::new(),
            slot: None,
            parent_slot: NONE,
            owned: Vec::new(),
            keys: Vec::new(),
            record: Default::default(),
            memories: Vec::new(),
            // A player's picture and time change without a binding.
            volatile: matches!(node.kind, NodeKind::Element(Kind::Video | Kind::Audio)),
            materialization,
            run_in: None,
            run_events: Default::default(),
            run_reads: false,
        };
        let properties = node
            .props
            .iter()
            .chain(node.at.iter().flat_map(|(_, props)| props));
        for p in properties {
            let expression = match &p.value {
                PropValue::Lit(_) => continue,
                PropValue::Bind(e, format) => BindingExpression::Expression(
                    format.map_or_else(|| e.clone(), |f| Expr::Format(f, Box::new(e.clone()))),
                ),
                PropValue::Model(path) => BindingExpression::Expression(Expr::Path(path.clone())),
                PropValue::Template(template) => BindingExpression::Template(template.clone()),
            };
            self.bind(
                ui,
                &mut element,
                Input::Property(p.value.clone(), PROPS[p.spec].ty),
                expression,
                env,
            )?;
        }
        for (_, expression) in &node.selector_attrs {
            self.bind(
                ui,
                &mut element,
                Input::Expression(expression.clone()),
                BindingExpression::Expression(expression.clone()),
                env,
            )?;
        }
        self.sync_facts(ui, &element, env)?;
        // Its box pseudo-elements' rules match against it: their
        // conditions are its inputs.
        let expressions = node
            .rules
            .iter()
            .chain(
                node.pseudo
                    .iter()
                    .flat_map(|&(_, at)| doc.nodes[at].rules.iter()),
            )
            .filter_map(|rule| rule.condition.as_ref())
            .chain(
                node.span_rules
                    .iter()
                    .filter_map(|rule| rule.condition.as_ref()),
            )
            .chain(node.groups.iter().filter_map(|group| group.bind.as_ref()))
            .chain(
                node.inline
                    .iter()
                    .map(|run| &run.text)
                    .filter(|text| !matches!(text, Expr::Lit(_))),
            )
            .chain(node.triggers.iter().flat_map(|trigger| {
                let when = match &trigger.when {
                    When::Change(expression) | When::Submit(Some(expression)) => Some(expression),
                    _ => None,
                };
                let action = match &trigger.act {
                    Act::Set(_, expression) => Some(expression),
                    _ => None,
                };
                when.into_iter().chain(action)
            }))
            .chain(node.uses.iter().filter_map(|(_, params)| params.as_ref()))
            .chain(
                node.rules
                    .iter()
                    .flat_map(|rule| rule.declarations.iter().filter_map(|d| d.value.binding())),
            )
            .chain(
                node.custom
                    .iter()
                    .map(|(_, e)| e)
                    .filter(|e| !node.selector_attrs.iter().any(|(_, s)| s == *e)),
            );
        for e in expressions {
            self.bind(
                ui,
                &mut element,
                Input::Expression(e.clone()),
                BindingExpression::Expression(e.clone()),
                env,
            )?;
        }
        let context = element.context.clone();
        self.elements.insert(identity.clone(), element);
        if !self.restoring {
            self.use_events(&identity, "create", None);
        }
        let mut children = Vec::new();
        for &child in &node.children {
            let n = &doc.nodes[child];
            let id = if n.id.is_empty() {
                format!("node{child}")
            } else {
                n.id.clone()
            };
            children.push(self.compile(
                ui,
                (doc.clone(), child),
                context.clone(),
                Some(&identity),
                id,
                env,
            )?);
        }
        self.elements.get_mut(&identity).unwrap().children = children;
        self.dirty.insert(identity.clone());
        if self.elements[&identity].volatile {
            self.volatile.insert(identity.clone());
        }
        for trigger in &doc.nodes[index].triggers {
            if let When::Action(action) = &trigger.when {
                self.on_action
                    .entry(action.clone())
                    .or_default()
                    .insert(identity.clone());
            }
        }
        // An exclusive `<details name>` closes as another of its name opens.
        if let Some(PropValue::Lit(Lit::Str(name))) = doc.nodes[index]
            .props
            .iter()
            .rev()
            .find(|p| p.name() == "exclusive")
            .map(|p| &p.value)
        {
            self.on_action
                .entry(super::nodes::details_action(name))
                .or_default()
                .insert(identity.clone());
        }
        // `<tabs>` steps on the bumpers (`tabPrev`, `tabNext`).
        if matches!(doc.nodes[index].kind, NodeKind::Element(Kind::Tabs)) {
            for action in ["tabPrev", "tabNext"] {
                self.on_action
                    .entry(action.into())
                    .or_default()
                    .insert(identity.clone());
            }
        }
        if let Some(parent) = parent {
            self.emit.insert(parent.clone());
        }
        Ok(Child::Element(identity))
    }
}
