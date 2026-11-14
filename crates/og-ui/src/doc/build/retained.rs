use super::*;
use crate::bind::Expr;
use crate::doc::topology::{Instance, Materialization, Topology};
use crate::reactive::{
    SourceId,
    bindings::{BindingContext, BindingExpression},
    graph::MountedNode,
};

pub(crate) mod compile;
pub(crate) mod content;
pub(crate) mod emit;
pub(crate) mod frame;
pub(crate) mod mounting;
pub(crate) mod pointer;
pub(crate) mod queries;
pub(crate) mod records;
pub(crate) mod refresh;
pub(crate) mod registration;
pub(crate) mod selection;
pub(crate) mod structural;
pub(crate) mod values;
pub(crate) mod virtualized;
pub(crate) use super::instances::*;

impl RetainedHost {
    pub(crate) fn restore_instances(&mut self, ui: &mut Ui, env: &dyn Env) -> Result<(), String> {
        while let Some(mount) = self.pending_mounts.first().cloned() {
            let registration = self.registrations[&mount].clone();
            match self.compile_mount(ui, &mount, &registration, env) {
                Ok(root) => {
                    self.pending_mounts.remove(&mount);
                    self.roots.insert(mount, root);
                }
                Err(error) => {
                    self.remove_mount(ui, &mount);
                    self.pending_mounts.insert(mount.clone());
                    self.registrations.insert(mount, registration);
                    return Err(error);
                }
            }
        }
        if self.restoring {
            self.topology.take_changes();
            self.topology.take_fact_changes();
        }
        self.restoring = false;
        Ok(())
    }

    /// Compiles `mount`'s root as `registration` declares it: its document,
    /// read through its stores and the content it is given.
    pub(crate) fn compile_mount(
        &mut self,
        ui: &mut Ui,
        mount: &SourceId,
        registration: &Registration,
        env: &dyn Env,
    ) -> Result<Child, String> {
        let document = ui
            .docs
            .library
            .documents
            .get(&registration.document)
            .cloned()
            .ok_or_else(|| format!("missing document '{}'", registration.document))?;
        let context = BindingContext {
            node: MountedNode {
                mount: mount.clone(),
                key: 0,
            },
            inputs: Arc::new(registration.inputs.clone()),
            rows: Arc::default(),
            row_key: String::new(),
            selector_nodes: Arc::default(),
            lexical: Arc::default(),
        };
        let lexical = content::mount_lexical(ui, &document, &registration.content, &context)?;
        let context = BindingContext {
            lexical: Arc::new(lexical),
            ..context
        };
        let path = root_path(mount, &document);
        self.compile(
            ui,
            (document.clone(), document.root),
            context,
            None,
            path,
            env,
        )
    }

    pub(crate) fn document_replaced(&mut self, ui: &mut Ui, document: &str) {
        self.profiles.remove(document);
        let mounts: Vec<_> = self
            .registrations
            .iter()
            .filter(|(_, registration)| registration.document == document)
            .map(|(mount, _)| mount.clone())
            .collect();
        for mount in mounts {
            self.rebuild_mount(ui, &mount);
        }
    }

    pub(crate) fn rebuild_mount(&mut self, ui: &mut Ui, mount: &SourceId) {
        let registration = self.registrations[mount].clone();
        self.remove_mount(ui, mount);
        self.pending_mounts.insert(mount.clone());
        self.registrations.insert(mount.clone(), registration);
    }

    pub(crate) fn bind(
        &mut self,
        ui: &mut Ui,
        element: &mut ElementInstance,
        input: Input,
        expression: BindingExpression,
        env: &dyn Env,
    ) -> Result<(), String> {
        let already_bound = match &input {
            Input::Expression(e) => element.cached.expressions.iter().any(|(old, _)| old == e),
            Input::Property(p, ty) => element
                .cached
                .properties
                .iter()
                .any(|(old, t, _)| old == p && t == ty),
            Input::Structure => false,
        };
        if already_bound {
            return Ok(());
        }
        let topology = &self.topology;
        let id = self.bindings.insert_expression(
            &mut ui.docs.state_mut().memory.reactive,
            element.context.clone(),
            expression,
            env,
            |node, state| {
                topology
                    .node(node)
                    .is_some_and(|n| n.states.contains(&state))
            },
        );
        self.inputs
            .insert(id, (element.context.node.clone(), input.clone()));
        element.bindings.push(id);
        let value = self.bindings.value(id).map_err(Clone::clone)?.clone();
        match &input {
            Input::Expression(e) => element.cached.expressions.push((e.clone(), value)),
            Input::Property(p, ty) => {
                element
                    .cached
                    .properties
                    .push((p.clone(), *ty, coerce(ui, *ty, &value)))
            }
            Input::Structure => unreachable!("structural bindings are owned by blocks"),
        }
        Ok(())
    }

    pub(crate) fn remove_mount(&mut self, ui: &mut Ui, mount: &SourceId) {
        let bindings: Vec<_> = self
            .inputs
            .iter()
            .filter(|(_, (node, _))| &node.mount == mount)
            .map(|(id, _)| *id)
            .collect();
        for id in bindings {
            self.bindings
                .remove(&mut ui.docs.state_mut().memory.reactive, id);
            self.inputs.remove(&id);
            self.pending_bindings.remove(&id);
        }
        let roots: Vec<_> = self
            .topology
            .children(None, 0..usize::MAX)
            .filter(|n| &n.mount == mount)
            .cloned()
            .collect();
        for root in roots {
            self.topology.remove(&root).expect("live mount root");
        }
        let removed: Vec<_> = self
            .elements
            .keys()
            .filter(|node| &node.mount == mount)
            .cloned()
            .collect();
        for node in removed {
            ui.interaction.state_mut().virtual_lists.remove(&node.key);
            self.release_element(ui, &node);
            self.use_events(&node, "destroy", None);
        }
        self.elements.retain(|node, _| &node.mount != mount);
        self.emit.retain(|node| &node.mount != mount);
        self.volatile.retain(|node| &node.mount != mount);
        self.animated.retain(|node| &node.mount != mount);
        self.on_action.retain(|_, listeners| {
            listeners.retain(|node| &node.mount != mount);
            !listeners.is_empty()
        });
        self.scrollers.retain(|node, _| &node.mount != mount);
        self.each.retain(|node, _| &node.mount != mount);
        self.branches.retain(|node, _| &node.mount != mount);
        self.collections.retain(|_, readers| {
            readers.retain(|node| &node.mount != mount);
            !readers.is_empty()
        });
        self.by_key.retain(|_, node| &node.mount != mount);
        self.dirty.retain(|node| &node.mount != mount);
        self.inherited.retain(|(node, _), _| &node.mount != mount);
        for readers in &mut self.media {
            readers.retain(|node| &node.mount != mount);
        }
        self.roots.remove(mount);
        self.registrations.remove(mount);
        self.pending_mounts.remove(mount);
    }

    /// Queues `node`'s `use:` events of `phase` (`create`, `update`,
    /// `destroy`): an update only for the action whose params are
    /// `changed`'s expression, with its new value.
    pub(crate) fn use_events(
        &mut self,
        node: &MountedNode,
        phase: &str,
        changed: Option<(&Expr, Value)>,
    ) {
        let Some(element) = self.elements.get(node) else {
            return;
        };
        for (name, params) in &element.document.nodes[element.index].uses {
            let value = match (&changed, params) {
                (Some((e, v)), Some(p)) if *e == p => v.clone(),
                (Some(_), _) => continue,
                (None, _) if phase == "destroy" => Value::Null,
                (None, Some(Expr::Lit(v))) => v.clone(),
                (None, Some(p)) => element.cached.expression(p),
                (None, None) => Value::Null,
            };
            self.uses.push(DocEvent {
                mount: element.context.node.mount.id.clone(),
                path: element.path.clone(),
                row_key: element.context.row_key.clone(),
                kind: "use".into(),
                name: name.clone(),
                value: json!({ "phase": phase, "params": value }),
            });
        }
    }
}

/// A mount's root path: its name ([`mount_path`]: `scope:id`, a pushed
/// screen's name), and for a screen, whose layer has that name, its
/// root's id under it (`scope:id/Menu`, `pause/Menu`).
fn root_path(mount: &SourceId, doc: &UiDocument) -> String {
    let base = crate::layer_stack::mount_path(mount);
    if doc.kind != crate::doc::DocKind::Screen {
        return base;
    }
    match doc.nodes.get(doc.root) {
        Some(n) if !n.id.is_empty() => format!("{base}/{}", n.id),
        _ => format!("{base}/Root"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactive::session::Command;
    use crate::{InputFrame, UiDesc, Viewport};

    /// A keyed collection read as a value: `$rows.length` in the document
    /// that mounts it, and `items.length` in a component it is handed to as
    /// a prop. Both count the rows and follow an insert and a removal; the
    /// mount builds, rather than failing its reads as a missing signal.
    #[test]
    fn a_collection_store_reads_its_length_directly_and_through_a_component_prop() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "counted.svelte",
                r#"
            <meta name="component" content="Counted">
            <meta name="prop:items" content="list">
            <column>
                <text id="Inner">{items.length}</text>
                {#if items.length == 0}<text id="Empty">none</text>{/if}
            </column>
        "#,
            )
            .unwrap(),
        );
        ui.define_document(
            UiDocument::parse(
                "page.svelte",
                r#"
            <meta name="component" content="page">
            <column>
                <text id="Outer">{$rows.length}</text>
                <Counted items={$rows} />
            </column>
        "#,
            )
            .unwrap(),
        );
        let rows = SourceId::new("test", "rows");
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Collection {
                source: rows.clone(),
                key: "id".into(),
                items: ["a", "b", "c"]
                    .map(|id| json!({ "id": id }).as_object().unwrap().clone())
                    .to_vec(),
            })
            .unwrap();
        let inputs = BTreeMap::from([("rows".to_string(), rows.clone())]);
        ui.mount_document(
            SourceId::new("test", "page"),
            "page",
            inputs,
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let texts = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            let mut texts: Vec<String> = ui
                .probe_elements()
                .into_iter()
                .filter(|n| {
                    ["Outer", "Inner", "Empty"]
                        .iter()
                        .any(|id| n.path.ends_with(id))
                })
                .map(|n| format!("{}={}", n.path.rsplit('/').next().unwrap(), n.text))
                .collect();
            texts.sort();
            texts
        };
        assert_eq!(texts(&mut ui), ["Inner=3", "Outer=3"]);
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Insert {
                source: rows.clone(),
                row: json!({ "id": "d" }).as_object().unwrap().clone(),
                before: None,
            })
            .unwrap();
        assert_eq!(texts(&mut ui), ["Inner=4", "Outer=4"]);
        for key in ["a", "b", "c", "d"] {
            ui.docs
                .state_mut()
                .memory
                .reactive
                .execute(Command::Remove {
                    source: rows.clone(),
                    key: key.into(),
                })
                .unwrap();
        }
        assert_eq!(texts(&mut ui), ["Empty=none", "Inner=0", "Outer=0"]);
    }

    /// A mount whose document uses a component without a prop that
    /// component declares with no default returns the error, naming the
    /// instance and the prop, instead of returning ok and building nothing
    /// (the bound name was unbound, and the deferred compile's error went
    /// only to the warnings). A prop with a `data-default` may be left
    /// out and reads its default.
    ///
    /// Ways it could fail: only the page's own instances checked (a
    /// component inside a component still mounts nothing), the check run
    /// only on the immediate `mount_document` (the host's `scope.mount`
    /// registers and still returns ok), or a defaulted prop refused.
    #[test]
    fn a_mount_refuses_an_instance_that_leaves_out_a_declared_prop() {
        let mut ui = Ui::new(UiDesc::default());
        for (file, text) in [
            (
                "card.svelte",
                r#"<meta name="component" content="Card">
            <meta name="prop:title" content="string">
            <meta name="prop:orientation" content="string">
            <column class:h={orientation == "horizontal"}><text id="Title">{title}</text></column>"#,
            ),
            (
                "tag.svelte",
                r#"<meta name="component" content="Tag">
            <meta name="prop:label" content="string" data-default="plain">
            <text id="Tag">{label}</text>"#,
            ),
            (
                "row.svelte",
                r#"<meta name="component" content="Row">
            <row><Card title="hi" /><Tag /></row>"#,
            ),
            (
                "page.svelte",
                r#"<meta name="component" content="page">
            <column><Row /></column>"#,
            ),
            (
                "fixed.svelte",
                r#"<meta name="component" content="fixed">
            <column><Card title="hi" orientation="vertical" /><Tag /></column>"#,
            ),
        ] {
            ui.define_document(UiDocument::parse(file, text).unwrap());
        }
        let mount = |ui: &mut Ui, document: &str| {
            ui.register_document_mount(
                SourceId::new("test", document),
                document.into(),
                BTreeMap::new(),
                BTreeMap::new(),
            )
        };
        let error = mount(&mut ui, "page").unwrap_err();
        assert!(
            error.starts_with("row.svelte:2: <Card> leaves out its prop 'orientation'"),
            "{error}"
        );
        assert!(ui.docs.state().memory.retained.registrations.is_empty());

        mount(&mut ui, "fixed").unwrap();
        ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
        ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
        ui.end();
        let mut texts: Vec<String> = ui
            .probe_elements()
            .into_iter()
            .filter(|n| !n.text.is_empty())
            .map(|n| n.text)
            .collect();
        texts.sort();
        assert_eq!(texts, ["hi", "plain"]);
    }

    #[test]
    fn retained_actions_report_the_innermost_collection_key_after_restore() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "actions.svelte",
                r#"
            <meta name="component" content="actions">
            <column>
                {#each $outer as z (z.id)}
                    <column id="Group">
                        {#each $inner as a (a.id)}
                            <button id="Inner" onclick={pick}>inner</button>
                        {/each}
                        <button id="Outer" onclick={pick}>outer</button>
                    </column>
                {/each}
                <button id="Outside" onclick={pick}>outside</button>
            </column>
        "#,
            )
            .unwrap(),
        );
        let mut inputs = BTreeMap::new();
        for (name, key) in [("outer", "group"), ("inner", "entry/#%")] {
            let source = SourceId::new("test", name);
            ui.docs
                .state_mut()
                .memory
                .reactive
                .execute(Command::Collection {
                    source: source.clone(),
                    key: "id".into(),
                    items: vec![json!({"id":key}).as_object().unwrap().clone()],
                })
                .unwrap();
            inputs.insert(name.into(), source);
        }
        ui.mount_document(
            SourceId::new("test", "actions"),
            "actions",
            inputs,
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui, input| {
            ui.begin(input, Viewport::new(400.0, 500.0));
            ui.end();
        };
        tick(&mut ui, InputFrame::default());
        let snapshot = ui.save_state();
        for restored in [false, true] {
            if restored {
                ui.restore_state(&snapshot).unwrap();
                tick(&mut ui, InputFrame::default());
            }
            for (label, expected) in [("inner", "entry/#%"), ("outer", "group"), ("outside", "")] {
                let path = ui
                    .probe_elements()
                    .into_iter()
                    .find(|node| node.text == label)
                    .unwrap()
                    .path;
                ui.begin(InputFrame::default(), Viewport::new(400.0, 500.0));
                ui.set_focus(&path);
                ui.end();
                assert_eq!(ui.focus_path(0), path);
                ui.take_doc_events();
                tick(
                    &mut ui,
                    InputFrame {
                        keys: vec![crate::UiKey::Enter],
                        keys_pressed: vec!["Enter"],
                        ..Default::default()
                    },
                );
                let events: Vec<_> = ui
                    .take_doc_events()
                    .into_iter()
                    .filter(|event| event.name == "pick")
                    .collect();
                assert_eq!(events.len(), 1, "{label}, restored={restored}");
                assert_eq!(events[0].row_key, expected);
                assert_eq!(events[0].path, path);
            }
        }
    }

    /// A mounted grid of labels: idle ticks evaluate and write nothing,
    /// one patched row field evaluates its one binding and writes its one
    /// element, and that label shows the new text.
    #[test]
    fn mounted_frames_write_only_what_changed() {
        let mut ui = Ui::new(UiDesc::default());
        let mut doc =
            String::from("<meta name=\"component\" content=\"grid\">\n<row id=\"Screen\">");
        let mut inputs = BTreeMap::new();
        for column in 0..4 {
            let source = SourceId::new("test", &format!("c{column}"));
            ui.docs
                .state_mut()
                .memory
                .reactive
                .execute(Command::Collection {
                    source: source.clone(),
                    key: "id".into(),
                    items: (0..25)
                        .map(|row| {
                            json!({"id": format!("r{row}"), "text": format!("Cell {column}:{row}")})
                                .as_object()
                                .unwrap()
                                .clone()
                        })
                        .collect(),
                })
                .unwrap();
            inputs.insert(format!("c{column}"), source);
            doc.push_str(&format!(
                "<column id=\"Column{column}\" width=\"128\" height=\"900\">{{#each $c{column} as cell (cell.id)}}<text id=\"Cell{column}\">{{cell.text}}</text>{{/each}}</column>"
            ));
        }
        // Every cell animates its opacity: settled, none is visited.
        doc.push_str("</row><style>text { transition: opacity 200ms; }</style>");
        ui.define_document(UiDocument::parse("grid.svelte", &doc).unwrap());
        ui.mount_document(
            SourceId::new("test", "grid"),
            "grid",
            inputs,
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), Viewport::new(1280.0, 960.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end().stats
        };
        // The first frames lay everything out and settle sizes.
        for _ in 0..3 {
            tick(&mut ui);
        }
        let idle = tick(&mut ui);
        assert_eq!(
            (
                idle.bindings,
                idle.reconciled,
                idle.laid_out,
                idle.painted,
                idle.animated
            ),
            (0, 0, 0, 0, 0),
            "{idle:?}"
        );
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Patch {
                source: SourceId::new("test", "c2"),
                key: "r19".into(),
                fields: json!({"text": "Value 7"}).as_object().unwrap().clone(),
            })
            .unwrap();
        let changed = tick(&mut ui);
        // Motion visits only the label written, not its moved siblings.
        assert_eq!(
            (changed.bindings, changed.reconciled, changed.animated),
            (1, 1, 1),
            "{changed:?}"
        );
        let text = |ui: &Ui| {
            ui.probe_elements()
                .into_iter()
                .find(|e| e.path == "test:grid/Column2/Cell2#r19")
                .unwrap()
                .text
        };
        assert_eq!(text(&ui), "Value 7");
        // Its size moved: written once more for widgets that read their
        // last layout, then idle again.
        tick(&mut ui);
        let idle = tick(&mut ui);
        assert_eq!(
            (idle.bindings, idle.reconciled, idle.animated),
            (0, 0, 0),
            "{idle:?}"
        );
        assert_eq!(text(&ui), "Value 7");
    }

    #[test]
    fn virtual_mounts_defer_offscreen_styles_and_keep_far_row_identity() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("virtual.svelte", r#"
            <meta name="component" content="virtual">
            <list id="Rows" row-height="40" height="400">
                {#each $items as item (item.id)}<button id="Pick" height="40">{item.label}</button>{/each}
            </list>
        "#).unwrap());
        let source = SourceId::new("test", "items");
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Collection {
                source: source.clone(),
                key: "id".into(),
                items: (0..1000)
                    .map(|i| {
                        json!({"id":format!("r{i}"),"label":format!("Row {i}")})
                            .as_object()
                            .unwrap()
                            .clone()
                    })
                    .collect(),
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "virtual"),
            "virtual",
            BTreeMap::from([("items".into(), source.clone())]),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui, wheel: f32, focus: Option<&str>| {
            ui.begin(
                InputFrame {
                    pointer_inside: true,
                    pointer: og_core::glam::Vec2::new(200.0, 100.0),
                    wheel: og_core::glam::Vec2::new(0.0, wheel),
                    ..Default::default()
                },
                Viewport::new(2560.0, 1440.0),
            );
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            if let Some(path) = focus {
                ui.set_focus(path);
            }
            ui.end();
        };
        tick(&mut ui, 0.0, None);
        let list = ui.mounted_topology().child_at(None, 0).unwrap().clone();
        let far = ui
            .mounted_topology()
            .child_at(Some(&list), 999)
            .unwrap()
            .clone();
        assert_eq!(
            ui.mounted_topology().node(&far).unwrap().direct_text,
            "Row 999"
        );
        assert_eq!(
            ui.mounted_topology().node(&far).unwrap().materialization,
            Materialization::Deferred
        );
        assert!(ui.docs.state().memory.retained.resolved < 20);
        tick(&mut ui, 0.0, None);
        assert_eq!(ui.docs.state().memory.retained.resolved, 0);
        assert_eq!(ui.docs.state().memory.retained.evaluated, 0);
        ui.docs
            .state_mut()
            .memory
            .retained
            .topology
            .invalidate_styles(Some(&list), 0..1000)
            .unwrap();
        tick(&mut ui, 0.0, None);
        assert!(ui.docs.state().memory.retained.resolved < 20);
        assert_eq!(
            ui.docs
                .state()
                .memory
                .retained
                .topology
                .style_is_stale(&far),
            Some(true)
        );
        tick(&mut ui, 100000.0, None);
        assert!(ui.probe_elements().iter().any(|n| n.text == "Row 999"));
        let path = "test:virtual/Row#r999/Pick#r999";
        tick(&mut ui, 0.0, Some(path));
        assert_eq!(ui.focused_key(), far.key);
        assert_eq!(ui.focus_path(0), path);
        assert_eq!(
            ui.docs
                .state()
                .memory
                .retained
                .topology
                .style_is_stale(&far),
            Some(false)
        );
        tick(&mut ui, -100000.0, None);
        assert!(ui.probe_elements().iter().any(|n| n.text == "Row 0"));
        assert_eq!(ui.mounted_topology().child_at(Some(&list), 999), Some(&far));
        assert_eq!(
            ui.mounted_topology().node(&far).unwrap().materialization,
            Materialization::Deferred
        );
        let snapshot = ui.save_state();
        ui.restore_state(&snapshot).unwrap();
        tick(&mut ui, 0.0, None);
        assert_eq!(ui.focused_key(), far.key);
        assert_eq!(ui.focus_path(0), path);
        tick(&mut ui, 100000.0, None);
        assert_eq!(ui.mounted_topology().child_at(Some(&list), 999), Some(&far));
        assert_eq!(ui.focused_key(), far.key);
        assert_eq!(ui.focus_path(0), path);
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Move {
                source: source.clone(),
                key: "r999".into(),
                before: Some("r0".into()),
            })
            .unwrap();
        tick(&mut ui, 0.0, None);
        assert_eq!(ui.mounted_topology().child_at(Some(&list), 0), Some(&far));
        assert_eq!(ui.focused_key(), far.key);
        tick(&mut ui, -100000.0, None);
        assert!(ui.probe_elements().iter().any(|n| n.text == "Row 999"));
        assert_eq!(ui.focus_path(0), path);
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Remove {
                source,
                key: "r999".into(),
            })
            .unwrap();
        tick(&mut ui, 0.0, None);
        assert!(ui.mounted_topology().node(&far).is_none());
        assert_ne!(ui.focused_key(), far.key);
    }

    #[test]
    fn mounted_each_edits_keep_multiple_siblings_and_bound_row_identity() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "each.svelte",
                r#"
            <meta name="component" content="each">
            <column id="Root">
                <text id="Before">before</text>
                {#each $items as item (item.id)}
                    <text id="Name">{item.name}</text><text id="Count">{item.count}</text>
                    {#if item.count > 20}<text id="Extra">extra</text>{/if}
                {:else}<text id="Empty">empty</text>{/each}
                <text id="After">after</text>
            </column>
        "#,
            )
            .unwrap(),
        );
        let source = SourceId::new("test", "items");
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Collection {
                source: source.clone(),
                key: "id".into(),
                items: vec![
                    json!({"id":"a","name":"alpha","count":7})
                        .as_object()
                        .unwrap()
                        .clone(),
                    json!({"id":"b","name":"beta","count":19})
                        .as_object()
                        .unwrap()
                        .clone(),
                ],
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "screen"),
            "each",
            BTreeMap::from([("items".into(), source.clone())]),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), Viewport::new(400.0, 500.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            let t = ui.mounted_topology();
            let root = t.child_at(None, 0).unwrap();
            t.children(Some(root), 0..usize::MAX)
                .map(|id| t.node(id).unwrap().direct_text.to_owned())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            tick(&mut ui),
            ["before", "alpha", "7", "beta", "19", "after"]
        );
        let root = ui.mounted_topology().child_at(None, 0).unwrap().clone();
        let alpha = ui
            .mounted_topology()
            .child_at(Some(&root), 1)
            .unwrap()
            .clone();
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Move {
                source: source.clone(),
                key: "a".into(),
                before: None,
            })
            .unwrap();
        assert_eq!(
            tick(&mut ui),
            ["before", "beta", "19", "alpha", "7", "after"]
        );
        assert_eq!(ui.docs.state().memory.retained.evaluated, 0);
        assert_eq!(ui.docs.state().memory.retained.resolved, 0);
        assert_eq!(ui.mounted_topology().child_at(Some(&root), 3), Some(&alpha));
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Patch {
                source: source.clone(),
                key: "a".into(),
                fields: json!({"count":23}).as_object().unwrap().clone(),
            })
            .unwrap();
        assert_eq!(
            tick(&mut ui),
            ["before", "beta", "19", "alpha", "23", "extra", "after"]
        );
        assert_eq!(ui.docs.state().memory.retained.evaluated, 2);
        assert_eq!(ui.docs.state().memory.retained.resolved, 2);
        for (count, expected) in [
            (7, vec!["before", "beta", "19", "alpha", "7", "after"]),
            (
                23,
                vec!["before", "beta", "19", "alpha", "23", "extra", "after"],
            ),
        ] {
            ui.docs
                .state_mut()
                .memory
                .reactive
                .execute(Command::Patch {
                    source: source.clone(),
                    key: "a".into(),
                    fields: json!({"count":count}).as_object().unwrap().clone(),
                })
                .unwrap();
            assert_eq!(tick(&mut ui), expected);
            assert_eq!(ui.mounted_topology().child_at(Some(&root), 3), Some(&alpha));
        }
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Insert {
                source: source.clone(),
                row: json!({"id":"c","name":"gamma","count":31})
                    .as_object()
                    .unwrap()
                    .clone(),
                before: Some("b".into()),
            })
            .unwrap();
        assert_eq!(
            tick(&mut ui),
            [
                "before", "gamma", "31", "extra", "beta", "19", "alpha", "23", "extra", "after"
            ]
        );
        for key in ["b", "a", "c"] {
            ui.docs
                .state_mut()
                .memory
                .reactive
                .execute(Command::Remove {
                    source: source.clone(),
                    key: key.into(),
                })
                .unwrap();
        }
        assert_eq!(tick(&mut ui), ["before", "empty", "after"]);
        assert!(ui.mounted_topology().node(&alpha).is_none());
        ui.unmount_document(&SourceId::new("test", "screen"));
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Insert {
                source,
                row: json!({"id":"d","name":"unmounted","count":44})
                    .as_object()
                    .unwrap()
                    .clone(),
                before: None,
            })
            .unwrap();
        ui.begin(InputFrame::default(), Viewport::new(400.0, 500.0));
        ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
        ui.end();
        assert_eq!(ui.mounted_topology().child_count(None), 0);
        assert!(
            ui.probe_elements()
                .iter()
                .all(|node| node.text != "unmounted")
        );
    }

    #[test]
    fn authored_control_state_restyles_subscribers_in_the_same_frame() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "states.svelte",
                r#"
            <meta name="component" content="states">
            <column><label><input type="checkbox" switch id="Choice" checked={$checked}> Choose</label></column>
            <style>#Choice { width: 70px; } #Choice:checked { width: 140px; }</style>
        "#,
            )
            .unwrap(),
        );
        let source = SourceId::new("test", "checked");
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Signal {
                source: source.clone(),
                initial: json!(true),
                readable: false,
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "state"),
            "states",
            BTreeMap::from([("checked".into(), source.clone())]),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        for (checked, width) in [(true, 140.0), (false, 70.0), (true, 140.0)] {
            ui.docs
                .state_mut()
                .memory
                .reactive
                .execute(Command::Set {
                    source: source.clone(),
                    value: json!(checked),
                    force: false,
                })
                .unwrap();
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            let elements = ui.probe_elements();
            assert_eq!(
                elements
                    .iter()
                    .find(|n| n.path == "test:state/Choice")
                    .unwrap()
                    .width,
                width
            );
            let t = ui.mounted_topology();
            let root = t.child_at(None, 0).unwrap();
            let choice = t.node(t.child_at(Some(root), 0).unwrap()).unwrap();
            assert_eq!(
                choice.attributes["checked"],
                if checked { json!("") } else { Value::Null }
            );
            assert_eq!(
                choice
                    .states
                    .contains(&crate::bind::InteractionState::Checked),
                checked
            );
        }
    }

    #[test]
    fn inherited_cascade_replaces_removed_overrides_without_rereading_text() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "cascade.svelte",
                r#"
            <meta name="component" content="cascade">
            <column id="Root" class={$wide ? 'wide' : ''}>
                <text id="Label">{$label}</text><text id="Shadow">fixed</text>
            </column>
            <style>
                #Root { --measure: 30px; }
                #Root.wide { --measure: 120px; }
                #Label { width: var(--measure); }
                #Shadow { --measure: 17px; width: var(--measure); }
            </style>
        "#,
            )
            .unwrap(),
        );
        let wide = SourceId::new("test", "wide");
        let label = SourceId::new("test", "label");
        for (source, value) in [(wide.clone(), json!(false)), (label.clone(), json!("kept"))] {
            ui.docs
                .state_mut()
                .memory
                .reactive
                .execute(Command::Signal {
                    source,
                    initial: value,
                    readable: false,
                })
                .unwrap();
        }
        ui.mount_document(
            SourceId::new("test", "screen"),
            "cascade",
            BTreeMap::from([("wide".into(), wide.clone()), ("label".into(), label)]),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        for (enabled, width) in [(false, 30.0), (true, 120.0), (false, 30.0)] {
            ui.docs
                .state_mut()
                .memory
                .reactive
                .execute(Command::Set {
                    source: wide.clone(),
                    value: json!(enabled),
                    force: false,
                })
                .unwrap();
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            let elements = ui.probe_elements();
            let label = elements
                .iter()
                .find(|e| e.path == "test:screen/Label")
                .unwrap();
            assert_eq!(label.width, width);
            assert_eq!(label.text, "kept");
            assert_eq!(
                elements
                    .iter()
                    .find(|e| e.path == "test:screen/Shadow")
                    .unwrap()
                    .width,
                17.0
            );
            let topology = ui.mounted_topology();
            let root = topology.node(topology.child_at(None, 0).unwrap()).unwrap();
            assert_eq!(
                root.attributes["class"],
                json!(if enabled { "wide" } else { "" })
            );
            if ui.tick() > 1 {
                assert_eq!(ui.docs.state().memory.retained.evaluated, 2); // Class value and matching predicate.
                assert_eq!(ui.docs.state().memory.retained.resolved, 2);
            }
        }
    }

    #[test]
    fn mounted_store_updates_resolve_one_element_and_emit_current_text() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("retained.svelte", r#"
            <meta name="component" content="retained">
            <column id="Root"><text id="Name">{$label}</text><text id="Other">unchanged</text></column>
        "#).unwrap());
        let source = SourceId::new("test", "label");
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Signal {
                source: source.clone(),
                initial: json!("first"),
                readable: false,
            })
            .unwrap();
        let mount = SourceId::new("test", "screen");
        ui.mount_document(
            mount.clone(),
            "retained",
            BTreeMap::from([("label".into(), source.clone())]),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.end();
        };
        tick(&mut ui);
        assert_eq!(
            ui.probe_elements()
                .iter()
                .find(|n| n.path == "test:screen/Name")
                .unwrap()
                .text,
            "first"
        );
        let root = ui.mounted_topology().child_at(None, 0).unwrap().clone();
        let label = ui
            .mounted_topology()
            .child_at(Some(&root), 0)
            .unwrap()
            .clone();
        assert_eq!(
            ui.mounted_topology().node(&label).unwrap().direct_text,
            "first"
        );
        ui.docs.state_mut().memory.retained.topology.take_changes();
        tick(&mut ui);
        assert_eq!(ui.docs.state().memory.retained.evaluated, 0);
        assert_eq!(ui.docs.state().memory.retained.resolved, 0);
        let snapshot = ui.save_state();
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Set {
                source: source.clone(),
                value: json!("changed"),
                force: false,
            })
            .unwrap();
        tick(&mut ui);
        assert_eq!(ui.docs.state().memory.retained.evaluated, 1);
        assert_eq!(ui.docs.state().memory.retained.resolved, 1);
        assert_eq!(
            ui.probe_elements()
                .iter()
                .find(|n| n.path == "test:screen/Name")
                .unwrap()
                .text,
            "changed"
        );
        assert_eq!(ui.mounted_topology().child_at(Some(&root), 0), Some(&label));
        assert!(
            ui.docs
                .state_mut()
                .memory
                .retained
                .topology
                .take_changes()
                .is_empty()
        );
        ui.restore_state(&snapshot).unwrap();
        assert_eq!(ui.save_state(), snapshot);
        tick(&mut ui);
        assert_eq!(ui.mounted_topology().child_at(Some(&root), 0), Some(&label));
        assert_eq!(
            ui.mounted_topology().node(&label).unwrap().direct_text,
            "first"
        );
        assert!(
            ui.docs
                .state_mut()
                .memory
                .retained
                .topology
                .take_changes()
                .is_empty()
        );
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Set {
                source: source.clone(),
                value: json!("after restore"),
                force: false,
            })
            .unwrap();
        tick(&mut ui);
        assert_eq!(
            ui.mounted_topology().node(&label).unwrap().direct_text,
            "after restore"
        );
        assert_eq!(ui.docs.state().memory.retained.evaluated, 1);
        assert_eq!(ui.docs.state().memory.retained.resolved, 1);
        ui.define_document(UiDocument::parse("retained.svelte", r#"
            <meta name="component" content="retained">
            <column id="Root"><text id="Name">new {$label}</text><text id="Replacement">added</text></column>
        "#).unwrap());
        tick(&mut ui);
        assert_eq!(ui.mounted_topology().child_at(Some(&root), 0), Some(&label));
        assert_eq!(
            ui.mounted_topology().node(&label).unwrap().direct_text,
            "new after restore"
        );
        assert!(ui.probe_elements().iter().any(|n| n.id == "Replacement"));
        assert!(!ui.probe_elements().iter().any(|n| n.id == "Other"));
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Set {
                source,
                value: json!("reloaded"),
                force: false,
            })
            .unwrap();
        tick(&mut ui);
        assert_eq!(
            ui.mounted_topology().node(&label).unwrap().direct_text,
            "new reloaded"
        );
    }

    #[test]
    fn a_retained_bound_palette_token_follows_theme_without_rereading_its_store() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "paint.svelte",
                r#"
            <meta name="component" content="paint">
            <column><column id="Patch" width="40" height="30" background={$paint} /></column>
        "#,
            )
            .unwrap(),
        );
        let source = SourceId::new("test", "paint");
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Signal {
                source: source.clone(),
                initial: json!("panel"),
                readable: false,
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "paint"),
            "paint",
            BTreeMap::from([("paint".into(), source)]),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        for color in [Srgba::rgb(210, 30, 40), Srgba::rgb(30, 200, 70)] {
            let mut theme = *ui.theme();
            theme.palette.panel = color;
            ui.set_theme(theme);
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.end();
            assert!(ui.frame().items.iter().any(|item| matches!(&item.cmd,
                crate::DrawCmd::Quad { color: actual, .. } if *actual == color.linear()
            )));
            assert_eq!(ui.docs.state().memory.retained.evaluated, 0);
        }
    }

    #[test]
    fn a_failed_refresh_keeps_other_pending_updates_and_can_rebuild_a_failed_branch() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "failure.svelte",
                r#"
            <meta name="component" content="failure">
            <column>
                <text id="Risk">{$bad ? $missing : "safe"}</text>
                {#if $show}<text id="Later">{$later}</text>{/if}
                <text id="Other">{$label}</text>
            </column>
        "#,
            )
            .unwrap(),
        );
        let source = |name| SourceId::new("test", name);
        for (name, initial) in [
            ("bad", json!(false)),
            ("show", json!(false)),
            ("label", json!("old")),
        ] {
            ui.docs
                .state_mut()
                .memory
                .reactive
                .execute(Command::Signal {
                    source: source(name),
                    initial,
                    readable: false,
                })
                .unwrap();
        }
        ui.mount_document(
            source("failure"),
            "failure",
            ["bad", "show", "label", "later"]
                .into_iter()
                .map(|name| (name.to_owned(), source(name)))
                .collect(),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            let result = ui.build_retained_documents(&crate::bind::NoEnv);
            ui.end();
            result
        };
        tick(&mut ui).unwrap();
        for (name, value) in [("bad", json!(true)), ("label", json!("new"))] {
            ui.docs
                .state_mut()
                .memory
                .reactive
                .execute(Command::Set {
                    source: source(name),
                    value,
                    force: false,
                })
                .unwrap();
        }
        assert!(tick(&mut ui).unwrap_err().contains("missing"));
        assert!(tick(&mut ui).unwrap_err().contains("missing"));
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Set {
                source: source("bad"),
                value: json!(false),
                force: false,
            })
            .unwrap();
        tick(&mut ui).unwrap();
        assert!(
            ui.probe_elements()
                .iter()
                .any(|node| node.id == "Other" && node.text == "new")
        );
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Set {
                source: source("show"),
                value: json!(true),
                force: false,
            })
            .unwrap();
        assert!(tick(&mut ui).unwrap_err().contains("later"));
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Signal {
                source: source("later"),
                initial: json!("recovered"),
                readable: false,
            })
            .unwrap();
        tick(&mut ui).unwrap();
        assert!(
            ui.probe_elements()
                .iter()
                .any(|node| node.id == "Later" && node.text == "recovered")
        );
        assert!(
            ui.probe_elements()
                .iter()
                .any(|node| node.id == "Other" && node.text == "new")
        );
    }

    #[test]
    fn script_props_derived_const_await_and_key_blocks_follow_svelte() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "badge.svelte",
                r#"
            <script lang="ts">
                // Data only: pure derived values.
                const doubled = $derived(count * 2);
                const shown = $derived.by(() => label + ':' + doubled);
            </script>
            <meta name="component" content="Badge">
            <meta name="prop:label" content="string" data-default="none">
            <meta name="prop:count" content="number" data-default="2">
            <text id="Badge">{shown}</text>
        "#,
            )
            .unwrap(),
        );
        ui.define_document(
            UiDocument::parse(
                "shop.svelte",
                r#"
            <meta name="component" content="shop">
            <column id="Root">
                <Badge id="Badge" label={$name}/>
                {#await $load}<text id="Wait">wait</text>
                {:then v}{@const n = v.count}<text id="Done">{n + 1}</text>
                {:catch e}<text id="Fail">{e}</text>{/await}
                {#key $round}<text id="Round">{$round}</text>{/key}
            </column>
        "#,
            )
            .unwrap(),
        );
        let store = |name: &str| SourceId::new("test", name);
        for (name, value) in [
            ("name", json!("gold")),
            ("load", json!({ "status": "pending" })),
            ("round", json!(1)),
        ] {
            ui.docs
                .state_mut()
                .memory
                .reactive
                .execute(Command::Signal {
                    source: store(name),
                    initial: value,
                    readable: false,
                })
                .unwrap();
        }
        ui.mount_document(
            store("screen"),
            "shop",
            ["name", "load", "round"]
                .into_iter()
                .map(|n| (n.to_string(), store(n)))
                .collect(),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let frame = |ui: &mut Ui, set: Option<(&str, Value)>| {
            if let Some((name, value)) = set {
                ui.docs
                    .state_mut()
                    .memory
                    .reactive
                    .execute(Command::Set {
                        source: store(name),
                        value,
                        force: false,
                    })
                    .unwrap();
            }
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            ui.probe_elements()
                .into_iter()
                .map(|e| (e.path, e.text))
                .collect::<Vec<_>>()
        };
        let text = |els: &[(String, String)], id: &str| {
            els.iter()
                .find(|(p, _)| p.rsplit('/').next().unwrap().starts_with(id))
                .map(|(p, t)| (p.clone(), t.clone()))
        };
        let first = frame(&mut ui, None);
        // The default count (2) doubled, beside the label passed in.
        assert_eq!(text(&first, "Badge").unwrap().1, "gold:4");
        assert!(text(&first, "Wait").is_some() && text(&first, "Done").is_none());
        let (round_path, round) = text(&first, "Round").unwrap();
        assert_eq!(round, "1");
        let done = frame(
            &mut ui,
            Some((
                "load",
                json!({ "status": "fulfilled", "value": { "count": 3 } }),
            )),
        );
        assert_eq!(text(&done, "Done").unwrap().1, "4");
        assert!(text(&done, "Wait").is_none() && text(&done, "Fail").is_none());
        let failed = frame(
            &mut ui,
            Some(("load", json!({ "status": "rejected", "reason": "offline" }))),
        );
        assert_eq!(text(&failed, "Fail").unwrap().1, "offline");
        // A new key value makes a new element; the same value keeps it.
        let next = frame(&mut ui, Some(("round", json!(2))));
        let (next_path, next_text) = text(&next, "Round").unwrap();
        assert_eq!(next_text, "2");
        assert_ne!(next_path, round_path);
        let again = frame(&mut ui, Some(("name", json!("silver"))));
        assert_eq!(text(&again, "Round").unwrap().0, next_path);
        assert_eq!(text(&again, "Badge").unwrap().1, "silver:4");
    }

    #[test]
    fn directives_map_onto_presence_inline_style_and_use_events() {
        let mut ui = Ui::new(UiDesc::default());
        let doc = UiDocument::parse(
            "fx.svelte",
            r#"
            <meta name="component" content="fx">
            <svelte:window bind:innerWidth={vw}/>
            <column id="Root">
                {#if $show}
                <div id="Box" use:tooltip={{ text: 'hi' }} use:track={$n}
                     style="width: 40px; height: 12px" style:width={$w + 'px'}
                     transition:fly={{ y: 20, duration: 200, easing: quintOut }}></div>
                {/if}
                <p bind:this={note}>{vw}</p>
                <svelte:element this={$level}>{note}</svelte:element>
            </column>
            <style>#Box { width: 10px; height: 30px; } #note { width: 50px; }</style>
        "#,
        )
        .unwrap();
        let boxed = doc.nodes.iter().find(|n| n.id == "Box").unwrap();
        let enter = boxed.enter.clone().unwrap();
        assert_eq!(Some(&enter), boxed.exit.as_ref());
        assert!(enter.values.contains(&(crate::element::Prop::Y, 20.0)));
        assert!(enter.values.contains(&(crate::element::Prop::Opacity, 0.0)));
        assert_eq!(
            enter.transition,
            crate::motion::Transition::Tween {
                duration_ms: 200.0,
                ease: crate::motion::Ease::CubicBezier([0.22, 1.0, 0.36, 1.0])
            }
        );
        ui.define_document(doc);
        let store = |name: &str| SourceId::new("test", name);
        for (name, value) in [
            ("show", json!(true)),
            ("n", json!(1)),
            ("w", json!(70)),
            ("level", json!("h2")),
        ] {
            ui.docs
                .state_mut()
                .memory
                .reactive
                .execute(Command::Signal {
                    source: store(name),
                    initial: value,
                    readable: false,
                })
                .unwrap();
        }
        ui.mount_document(
            store("screen"),
            "fx",
            ["show", "n", "w", "level"]
                .into_iter()
                .map(|n| (n.to_string(), store(n)))
                .collect(),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let frame = |ui: &mut Ui, set: Option<(&str, Value)>| {
            if let Some((name, value)) = set {
                ui.docs
                    .state_mut()
                    .memory
                    .reactive
                    .execute(Command::Set {
                        source: store(name),
                        value,
                        force: false,
                    })
                    .unwrap();
            }
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            let uses: Vec<_> = ui
                .take_doc_events()
                .into_iter()
                .filter(|e| e.kind == "use")
                .map(|e| (e.name, e.value["phase"].clone(), e.value["params"].clone()))
                .collect();
            (ui.probe_elements(), uses)
        };
        let (els, uses) = frame(&mut ui, None);
        let find = |id: &str| els.iter().find(|e| e.path.ends_with(id)).unwrap().clone();
        // The bound style:width beats the attribute's, which beats the rule.
        assert_eq!((find("/Box").width, find("/Box").height), (70.0, 12.0));
        assert_eq!(find("/note").text, "400");
        assert_eq!(find("/note").width, 50.0);
        assert!(els.iter().any(|e| e.text == "note" && e.role == "heading"));
        assert_eq!(
            uses,
            vec![
                ("tooltip".into(), json!("create"), json!({ "text": "hi" })),
                ("track".into(), json!("create"), json!(1)),
            ]
        );
        let (els, uses) = frame(&mut ui, Some(("n", json!(2))));
        assert_eq!(uses, vec![("track".into(), json!("update"), json!(2))]);
        assert_eq!(
            els.iter().find(|e| e.path.ends_with("/Box")).unwrap().width,
            70.0
        );
        let (_, uses) = frame(&mut ui, Some(("show", json!(false))));
        assert_eq!(
            uses,
            vec![
                ("tooltip".into(), json!("destroy"), Value::Null),
                ("track".into(), json!("destroy"), Value::Null),
            ]
        );
    }

    #[test]
    fn blur_and_draw_transitions_move_a_filter_and_a_stroke() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "fx2.svelte",
                r##"
            <meta name="component" content="fx2">
            <column id="Root">
                {#if $show}
                <div id="Soft" in:blur={{ amount: 8, duration: 400 }}></div>
                <svg id="Pic" viewBox="0 0 100 100">
                    <path id="Line" d="M10 10 L90 10" stroke="#ffffff" stroke-width="4"
                          in:draw={{ duration: 400, easing: linear }}/>
                </svg>
                {/if}
            </column>
            <style>#Soft { width: 40px; height: 20px; background: #ff0000; }
            #Pic { width: 100px; height: 100px; }</style>
        "##,
            )
            .unwrap(),
        );
        let show = SourceId::new("test", "show");
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Signal {
                source: show.clone(),
                initial: json!(false),
                readable: false,
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "screen"),
            "fx2",
            [("show".to_string(), show.clone())].into_iter().collect(),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        // The blur layer's radius and the line's dash and offset.
        let frame = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            let mut blur = None;
            let mut dash = None;
            for item in &ui.frame().items {
                match &item.cmd {
                    crate::DrawCmd::BeginLayer {
                        material: og_draw::UiMaterial::Builtin(og_draw::UiBuiltin::Blur),
                        params,
                        ..
                    } => blur = Some(params[0][0]),
                    crate::DrawCmd::Shape { shape, .. } => {
                        if let og_draw::UiMode::Stroke(s) = shape.mode {
                            dash = Some((s.dash, s.dash_offset));
                        }
                    }
                    _ => {}
                }
            }
            (blur, dash)
        };
        frame(&mut ui);
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Set {
                source: show,
                value: json!(true),
                force: false,
            })
            .unwrap();
        let mut seen = Vec::new();
        for _ in 0..40 {
            seen.push(frame(&mut ui));
        }
        // Halfway: blurred by under its 8 px (a 16 px radius), and the
        // 80-unit line half hidden by one dash and one gap of its length.
        let (blur, dash) = seen[12];
        let r = blur.expect("a blur layer while it enters");
        assert!(r > 0.0 && r < 16.0, "{r}");
        let ([d, g], offset) = dash.expect("the line's stroke");
        assert!((d - 80.0).abs() < 0.5 && (g - 80.0).abs() < 0.5, "{d} {g}");
        assert!(offset > 20.0 && offset < 60.0, "{offset}");
        // Settled: no layer, a solid stroke.
        assert_eq!(seen[39], (None, Some(([0.0; 2], 0.0))));
    }

    /// A rule's ancestor compound reads its state on the element it
    /// matches, not on the rule's subject: `.card:hover > text` follows
    /// the card's hover, for each state a selector names.
    ///
    /// Ways it could fail: the state read on the subject (the title's own
    /// state restyles it), a state's name bound to another state, the
    /// title not resolved again when the card's state changes.
    #[test]
    fn ancestor_selectors_read_the_matched_element_s_state() {
        use crate::bind::InteractionState;
        for (name, state) in [
            ("hover", InteractionState::Hover),
            ("active", InteractionState::Active),
            ("focus", InteractionState::Focus),
            ("focus-visible", InteractionState::FocusVisible),
            ("focus-within", InteractionState::FocusWithin),
            ("disabled", InteractionState::Disabled),
            ("checked", InteractionState::Checked),
            ("selected", InteractionState::Selected),
            ("invalid", InteractionState::Invalid),
            ("placeholder-shown", InteractionState::PlaceholderShown),
        ] {
            let source = format!(
                r#"<meta name="component" content="states">
<column id="Card" class="card"><text id="Title">Title</text></column>
<style>text {{ width: 22px; }} .card:{name} > text:not(:where(.excluded)) {{ width: 44px; }}</style>"#
            );
            let mut ui = Ui::new(UiDesc::default());
            ui.define_document(UiDocument::parse("states.svelte", &source).unwrap());
            ui.mount_document(
                SourceId::new("test", "states"),
                "states",
                BTreeMap::new(),
                Default::default(),
                &crate::bind::NoEnv,
            )
            .unwrap();
            // The element at `path` enters or leaves the state.
            let set = |ui: &mut Ui, path: &str, enabled: bool| {
                let memory = &mut ui.docs.state_mut().memory;
                let node = memory.retained.by_key[&crate::id::key_of_path(path)].clone();
                memory
                    .retained
                    .topology
                    .set_state(&node, state, enabled)
                    .unwrap();
                memory.reactive.graph.interaction_changed(&node, state);
            };
            let title = |ui: &mut Ui| {
                ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
                ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
                ui.end();
                let elements = ui.probe_elements();
                let title = elements.iter().find(|e| e.path == "test:states/Title");
                title.unwrap().width
            };
            assert_eq!(title(&mut ui), 22.0, "{name}: on neither");
            set(&mut ui, "test:states", true);
            assert_eq!(title(&mut ui), 44.0, "{name}: on the card");
            set(&mut ui, "test:states", false);
            set(&mut ui, "test:states/Title", true);
            assert_eq!(title(&mut ui), 22.0, "{name}: on the title");
        }
    }

    /// A rule that sets a prop by a binding (`top: bind(y)`) places its
    /// element by the bound value, and follows it.
    ///
    /// Ways it could fail: the rule's prop left out of the element's
    /// inputs (resolving it finds no value), or read once.
    #[test]
    fn a_rule_s_bound_prop_follows_its_binding() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "placed.svelte",
                r#"<meta name="component" content="placed">
<meta name="prop:y" content="number">
<div id="Box"><div id="Card"></div></div>
<style>
#Box { width: 200px; height: 200px; }
#Card { position: absolute; top: bind(y); width: 10px; height: 10px; }
</style>"#,
            )
            .unwrap(),
        );
        let y = SourceId::new("test", "y");
        ui.docs
            .execute(Command::Signal {
                source: y.clone(),
                initial: json!(40),
                readable: false,
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "placed"),
            "placed",
            BTreeMap::from([("y".into(), y.clone())]),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let top = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), Viewport::new(640.0, 480.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            let elements = ui.probe_elements();
            let of = |id: &str| {
                elements
                    .iter()
                    .find(|e| e.path == id || e.path.ends_with(&format!("/{id}")))
                    .unwrap_or_else(|| panic!("no {id}"))
                    .y
            };
            of("Card") - of("test:placed")
        };
        assert_eq!(top(&mut ui), 40.0);
        ui.docs
            .execute(Command::Set {
                source: y,
                value: json!(64),
                force: false,
            })
            .unwrap();
        assert_eq!(top(&mut ui), 64.0);
    }
}
