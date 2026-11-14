//! Components and content in mounted documents (docs/ui.md §2.1, Svelte 5
//! props and snippets): a component instance compiles its document's root
//! in a context whose props are its user's expressions, read in the user's
//! context; `{@render name()}` renders the content passed to the instance
//! (or to the mount), else the document's own snippet, in the context it
//! was written in.

use super::*;
use crate::doc::Lit;
use crate::reactive::bindings::{BindingExpression, ContentRef, Lexical, ListRef, PropRef};

impl RetainedHost {
    /// `<Card title={item.name}>{#snippet body()}…{/snippet}<p>…</p></Card>`:
    /// the component's root, its props `title` and its content `body` and
    /// `children` (the instance's other children).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compile_component(
        &mut self,
        ui: &mut Ui,
        source: (Arc<UiDocument>, usize),
        component: &str,
        context: BindingContext,
        parent: Option<&MountedNode>,
        id: String,
        env: &dyn Env,
    ) -> Result<Child, String> {
        let (doc, index) = source;
        let node = &doc.nodes[index];
        let instance = ui
            .docs
            .library
            .documents
            .get(component)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "{}:{}: no component '{component}' (load its document)",
                    doc.source, node.line
                )
            })?;
        unpassed(&doc, index, &instance)?;
        let user = Arc::new(context.clone());
        let mut lexical = Lexical::default();
        for (name, value, _) in &node.args {
            let (expression, model) = match value {
                PropValue::Lit(lit) => (Expr::Lit(literal(lit)).into(), None),
                PropValue::Bind(e, format) => (
                    format
                        .map_or_else(|| e.clone(), |f| Expr::Format(f, Box::new(e.clone())))
                        .into(),
                    None,
                ),
                PropValue::Model(path) => (Expr::Path(path.clone()).into(), Some(path.clone())),
                PropValue::Template(t) => (BindingExpression::Template(t.clone()), None),
            };
            lexical.props.insert(
                name.clone(),
                PropRef {
                    expression,
                    model,
                    context: user.clone(),
                },
            );
        }
        // Its style props (`--name={value}`), and those of the instance it
        // is the root of: custom properties its root inherits.
        for (name, expression) in &node.custom {
            lexical.props.insert(
                name.clone(),
                PropRef {
                    expression: expression.clone().into(),
                    model: None,
                    context: user.clone(),
                },
            );
        }
        if index == doc.root {
            for (name, prop) in style_props(&context) {
                lexical
                    .props
                    .entry(name.clone())
                    .or_insert_with(|| prop.clone());
            }
        }
        // A prop its user leaves out reads its default (`data-default`).
        for prop in &instance.props {
            if let Some(default) = &prop.default
                && !lexical.props.contains_key(&prop.name)
            {
                lexical.props.insert(
                    prop.name.clone(),
                    PropRef {
                        expression: Expr::Lit(default.clone()).into(),
                        model: None,
                        context: user.clone(),
                    },
                );
            }
        }
        let mut children = false;
        for &c in &node.children {
            match &doc.nodes[c].kind {
                NodeKind::Snippet(name) => {
                    lexical.content.insert(
                        name.clone(),
                        ContentRef::Snippet {
                            document: doc.clone(),
                            index: c,
                            context: user.clone(),
                        },
                    );
                }
                _ => children = true,
            }
        }
        if children && !lexical.content.contains_key("children") {
            // Svelte's implicit `children` snippet: the instance's own
            // children (its snippet definitions among them render nothing).
            lexical.content.insert(
                "children".into(),
                ContentRef::Snippet {
                    document: doc.clone(),
                    index,
                    context: user,
                },
            );
        }
        let inner = BindingContext {
            node: context.node.clone(),
            inputs: context.inputs.clone(),
            rows: Arc::default(),
            row_key: context.row_key.clone(),
            selector_nodes: Arc::default(),
            lexical: Arc::new(lexical),
        };
        let root = instance.root;
        self.compile(ui, (instance, root), inner, parent, id, env)
    }

    /// `{@render name()}`: the content passed under `name`, else the
    /// document's snippet `name`; with `name?.()` nothing when there is
    /// neither.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compile_render(
        &mut self,
        ui: &mut Ui,
        source: (Arc<UiDocument>, usize),
        name: &str,
        optional: bool,
        context: BindingContext,
        parent: Option<&MountedNode>,
        id: String,
        env: &dyn Env,
    ) -> Result<Child, String> {
        let (doc, index) = source;
        let always = Expr::Lit(true.into());
        // Rendered content is written in another document (or another place
        // of this one): its elements' ids sit under the snippet's name, so
        // they never meet the ids of the element they render into, and the
        // block's own id (`@`) never meets theirs.
        let id = match id.rsplit_once('/') {
            Some((prefix, last)) => format!("{prefix}/{name}/@{last}"),
            None => format!("{name}/@{id}"),
        };
        match context.lexical.content.get(name).cloned() {
            Some(ContentRef::Snippet {
                document,
                index,
                context: written,
            }) => {
                let document = (document as Arc<dyn std::any::Any + Send + Sync>)
                    .downcast::<UiDocument>()
                    .map_err(|_| format!("content '{name}' is not a document"))?;
                return self.compile_block(
                    ui,
                    (document, index),
                    (*written).clone(),
                    parent,
                    id,
                    &always,
                    env,
                );
            }
            Some(ContentRef::Document(name)) => {
                let document = ui
                    .docs
                    .library
                    .documents
                    .get(&name)
                    .cloned()
                    .ok_or_else(|| format!("content '{name}' names no loaded document"))?;
                let inner = BindingContext {
                    rows: Arc::default(),
                    selector_nodes: Arc::default(),
                    lexical: Arc::default(),
                    ..context
                };
                let root = document.root;
                let id = format!("{id}/{}", root_id(&document));
                return self.compile(ui, (document, root), inner, parent, id, env);
            }
            None => {}
        }
        if let Some(snippet) = doc
            .nodes
            .iter()
            .position(|n| n.kind == NodeKind::Snippet(name.to_string()))
        {
            return self.compile_block(ui, (doc, snippet), context, parent, id, &always, env);
        }
        if optional {
            let never = Expr::Lit(false.into());
            return self.compile_block(ui, (doc, index), context, parent, id, &never, env);
        }
        Err(format!(
            "{}:{}: {{@render {name}()}} has no content '{name}' and the document no such snippet; pass it, or write {{@render {name}?.()}}",
            doc.source, doc.nodes[index].line
        ))
    }
}

/// An instance at `doc.nodes[index]` that leaves out a prop `instance`
/// declares `data-required` is refused: the prop would be unbound, and the
/// document built nothing. Every other prop has a default.
/// Content props are passed as snippets and children, not arguments.
/// The style props of the instance `context` is inside: its props named as
/// custom properties.
pub(crate) fn style_props(context: &BindingContext) -> impl Iterator<Item = (&String, &PropRef)> {
    context
        .lexical
        .props
        .iter()
        .filter(|(name, _)| name.starts_with("--"))
}

fn unpassed(doc: &UiDocument, index: usize, instance: &UiDocument) -> Result<(), String> {
    let node = &doc.nodes[index];
    let missing = instance.props.iter().find(|p| {
        p.default.is_none()
            && p.ty != "content"
            && !node.args.iter().any(|(name, ..)| *name == p.name)
    });
    match missing {
        Some(p) => Err(format!(
            "{}:{}: <{}> leaves out its prop '{}', which it requires (data-required); pass it",
            doc.source, node.line, instance.name, p.name
        )),
        None => Ok(()),
    }
}

/// Every component instance `document` builds, through the components it
/// uses, passes each prop with no default: checked when the document is
/// mounted, so a mount that would build nothing returns the error.
/// Components not loaded yet are checked when they compile.
pub(crate) fn check_instances(
    docs: &BTreeMap<String, Arc<UiDocument>>,
    document: &str,
) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    let mut queue = vec![document.to_string()];
    while let Some(name) = queue.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let Some(doc) = docs.get(&name) else {
            continue;
        };
        for (index, node) in doc.nodes.iter().enumerate() {
            if let NodeKind::Use(component) = &node.kind
                && let Some(instance) = docs.get(component)
            {
                unpassed(doc, index, instance)?;
                queue.push(component.clone());
            }
        }
    }
    Ok(())
}

/// What a mount's root reads beyond its stores (`$name`). The names its
/// document reads bare ([`UiDocument::bare_names`]: its props) are each
/// the mount's store of that name, as an instance's are its user's
/// attributes; a prop the mount leaves out reads its default. The
/// content it is given, by name, is `document` (its
/// root) or `document#snippet` (one of its snippets), rendered with the
/// mount's stores.
pub(super) fn mount_lexical(
    ui: &Ui,
    document: &UiDocument,
    content: &BTreeMap<String, String>,
    context: &BindingContext,
) -> Result<Lexical, String> {
    let mut lexical = Lexical::default();
    let written = Arc::new(context.clone());
    for name in document.bare_names() {
        let store = format!("${name}");
        let default = document.props.iter().find(|p| p.name == name);
        let (expression, model) = if context.inputs.contains_key(&name) {
            (Expr::Path(store.clone()), Some(store))
        } else if let Some(default) = default.and_then(|p| p.default.as_ref()) {
            (Expr::Lit(default.clone()), None)
        } else {
            continue;
        };
        lexical.props.insert(
            name,
            PropRef {
                expression: expression.into(),
                model,
                context: written.clone(),
            },
        );
    }
    for (name, reference) in content {
        if !crate::doc::syntax::identifier(name) {
            return Err(format!("mount content name '{name}' is not an identifier"));
        }
        let (document, snippet) = match reference.split_once('#') {
            Some((d, s)) => (d, Some(s)),
            None => (reference.as_str(), None),
        };
        let doc = ui
            .docs
            .library
            .documents
            .get(document)
            .cloned()
            .ok_or_else(|| format!("mount content '{name}': no document '{document}'"))?;
        let value = match snippet {
            None => ContentRef::Document(document.into()),
            Some(s) => {
                let index = doc
                    .nodes
                    .iter()
                    .position(|n| n.kind == NodeKind::Snippet(s.into()))
                    .ok_or_else(|| {
                        format!("mount content '{name}': '{document}' has no snippet '{s}'")
                    })?;
                ContentRef::Snippet {
                    document: doc,
                    index,
                    context: written.clone(),
                }
            }
        };
        lexical.content.insert(name.clone(), value);
    }
    Ok(lexical)
}

/// The keyed list an `{#each}` reads: a `$name` collection, a list held in
/// a row of an enclosing each (`r.cells`), a component prop its user
/// bound to one (`<List items={$inventory}>`), or a list prop left out,
/// which has no rows. `key` is the path in each item of a held list's
/// key, which a collection does not need.
pub(super) fn collection(
    context: &BindingContext,
    path: &str,
    key: Option<&str>,
) -> Result<ListRef, String> {
    if let Some(name) = path.strip_prefix('$') {
        return context
            .inputs
            .get(name)
            .cloned()
            .map(ListRef::Collection)
            .ok_or_else(|| format!("mount has no collection '{name}'"));
    }
    let (root, rest) = path.split_once('.').unwrap_or((path, ""));
    if let Some(PropRef {
        expression: BindingExpression::Expression(Expr::Path(inner)),
        context: user,
        ..
    }) = context.lexical.props.get(root)
    {
        return if rest.is_empty() {
            collection(user, inner, key)
        } else {
            collection(user, &format!("{inner}.{rest}"), key)
        };
    }
    if let Some(PropRef {
        expression: BindingExpression::Expression(Expr::Lit(value)),
        ..
    }) = context.lexical.props.get(root)
        && rest.is_empty()
    {
        return match value {
            Value::Array(items) if items.is_empty() => {
                Ok(ListRef::Collection(crate::doc::NO_ROWS.clone()))
            }
            _ => Err(format!(
                "each over '{path}' reads a literal list; its default is [] (none), and its user binds it to a collection"
            )),
        };
    }
    match context.rows.get(root) {
        Some(row) if !row.position && !rest.is_empty() => Ok(ListRef::Nested {
            row: Arc::new(row.clone()),
            path: rest.into(),
            key: key
                .ok_or_else(|| {
                    format!("each over '{path}' is keyed by a field of its items, as (item.id)")
                })?
                .into(),
        }),
        _ => Err(format!(
            "each reads a named collection as $name, a list held in a row, or a prop bound to one; '{path}' is none of these"
        )),
    }
}

fn root_id(doc: &UiDocument) -> String {
    match doc.nodes.get(doc.root) {
        Some(n) if !n.id.is_empty() => n.id.clone(),
        _ => doc.name.clone(),
    }
}

fn literal(lit: &Lit) -> Value {
    match lit {
        Lit::Num(x) => json!(x),
        Lit::Bool(x) => json!(x),
        Lit::Str(s) => json!(s),
        Lit::Strs(s) => json!(s),
        _ => Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactive::session::Command;
    use crate::{InputFrame, UiDesc, Viewport};

    const CARD: &str = r#"<meta name="component" content="Card">
<article id="Card"><h3 id="Title">{title}</h3>{@render body?.()}{@render children?.()}</article>"#;
    const LIST: &str = r#"<meta name="component" content="List">
<ul id="List">{#each items as it (it.id)}<li id="Row">{it.name}</li>{/each}</ul>"#;
    const ASIDE: &str = r#"<meta name="component" content="aside">
<column>{#snippet note()}<p id="Note">note {$footer}</p>{/snippet}</column>"#;
    const PAGE: &str = r#"<meta name="component" content="page">
<column id="Page">
  {#each $items as item (item.id)}
    <Card id="C" title={item.name}>
      {#snippet body()}<p id="Desc">{item.desc}</p>{/snippet}
      <p id="Extra">by {$footer}</p>
    </Card>
  {/each}
  <List id="L" items={$items}/>
  {@render side?.()}
  {@render missing?.()}
</column>"#;

    fn ui() -> Ui {
        let mut ui = Ui::new(UiDesc::default());
        for (name, text) in [
            ("Card", CARD),
            ("List", LIST),
            ("aside", ASIDE),
            ("page", PAGE),
        ] {
            ui.define_document(UiDocument::parse(&format!("{name}.svelte"), text).unwrap());
        }
        let rows = [("a", "Axe", "sharp"), ("b", "Bow", "far")];
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Collection {
                source: SourceId::new("test", "items"),
                key: "id".into(),
                items: rows
                    .iter()
                    .map(|(id, name, desc)| {
                        json!({"id": id, "name": name, "desc": desc})
                            .as_object()
                            .unwrap()
                            .clone()
                    })
                    .collect(),
            })
            .unwrap();
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Signal {
                source: SourceId::new("test", "footer"),
                initial: json!("Ann"),
                readable: false,
            })
            .unwrap();
        ui
    }

    fn inputs() -> BTreeMap<String, SourceId> {
        BTreeMap::from([
            ("items".into(), SourceId::new("test", "items")),
            ("footer".into(), SourceId::new("test", "footer")),
        ])
    }

    fn step(ui: &mut Ui) -> crate::LayoutStats {
        ui.begin(InputFrame::default(), Viewport::new(640.0, 800.0));
        ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
        ui.end().stats
    }

    fn texts(ui: &Ui) -> Vec<(String, String)> {
        ui.probe_elements()
            .into_iter()
            .filter(|e| e.kind == "label")
            .map(|e| (e.path, e.text))
            .collect()
    }

    /// A component in a keyed each takes its props from the row, renders
    /// the snippet passed to it and its children in the row's context, and
    /// iterates a collection its user passed as a prop; mount content
    /// renders another document's snippet. A patched row field evaluates
    /// one binding.
    ///
    /// Ways it could fail: a prop read as a store (`$title`) or copied
    /// once, a snippet rendered in the component's context (no `item`),
    /// the implicit children dropped, `{#each}` refusing a prop, or a
    /// patch re-evaluating every prop of every instance.
    #[test]
    fn components_take_props_snippets_and_children_from_their_user() {
        let mut ui = ui();
        let content = BTreeMap::from([("side".to_string(), "aside#note".to_string())]);
        ui.mount_document(
            SourceId::new("test", "page"),
            "page",
            inputs(),
            content,
            &crate::bind::NoEnv,
        )
        .unwrap();
        for _ in 0..3 {
            step(&mut ui);
        }
        let all = texts(&ui);
        let of = |id: &str| -> Vec<String> {
            all.iter()
                .filter(|(p, _)| {
                    p.split('/')
                        .any(|s| s == id || s.starts_with(&format!("{id}#")))
                })
                .map(|(_, t)| t.clone())
                .collect()
        };
        assert_eq!(of("Title"), ["Axe", "Bow"], "{all:?}");
        assert_eq!(of("Desc"), ["sharp", "far"], "{all:?}");
        assert_eq!(of("Extra"), ["by Ann", "by Ann"], "{all:?}");
        assert_eq!(of("Row"), ["Axe", "Bow"], "{all:?}");
        assert_eq!(of("Note"), ["note Ann"], "{all:?}");
        let idle = step(&mut ui);
        assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Patch {
                source: SourceId::new("test", "items"),
                key: "b".into(),
                fields: json!({"name": "Bolt"}).as_object().unwrap().clone(),
            })
            .unwrap();
        let changed = step(&mut ui);
        // The card's title and the list row read the patched field.
        assert_eq!(changed.bindings, 2, "{changed:?}");
        let all = texts(&ui);
        assert_eq!(
            all.iter().filter(|(_, t)| t == "Bolt").count(),
            2,
            "{all:?}"
        );
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Set {
                source: SourceId::new("test", "footer"),
                value: json!("Bo"),
                force: false,
            })
            .unwrap();
        let changed = step(&mut ui);
        // Two children of the cards and the passed note.
        assert_eq!(changed.bindings, 3, "{changed:?}");
    }

    #[test]
    fn a_render_without_content_or_snippet_is_refused() {
        let mut ui = ui();
        ui.define_document(
            UiDocument::parse(
                "bad.svelte",
                r#"<meta name="component" content="bad"><column>{@render body()}</column>"#,
            )
            .unwrap(),
        );
        let error = ui
            .mount_document(
                SourceId::new("test", "bad"),
                "bad",
                inputs(),
                Default::default(),
                &crate::bind::NoEnv,
            )
            .unwrap_err();
        assert!(error.contains("has no content 'body'"), "{error}");
        let error = ui
            .mount_document(
                SourceId::new("test", "page"),
                "page",
                inputs(),
                BTreeMap::from([("side".into(), "aside#nowhere".into())]),
                &crate::bind::NoEnv,
            )
            .unwrap_err();
        assert!(error.contains("no snippet 'nowhere'"), "{error}");
    }

    /// A mount's root reads its document's declared props bare: each is
    /// the mount's store of that name and follows it, and one the mount
    /// leaves out reads its default.
    ///
    /// Ways it could fail: a prop read only as `$name` (the bare name is
    /// unknown), the default shown over a store the mount has, or the
    /// store read once.
    #[test]
    fn a_mount_s_root_reads_its_declared_props_bare() {
        let mut ui = ui();
        ui.define_document(
            UiDocument::parse(
                "sign.svelte",
                r#"<meta name="component" content="sign">
<meta name="prop:footer" content="string" data-default="nobody">
<meta name="prop:title" content="string" data-default="Untitled">
<column id="Sign"><p id="Title">{title}</p><p id="By">{footer}</p></column>"#,
            )
            .unwrap(),
        );
        ui.mount_document(
            SourceId::new("test", "sign"),
            "sign",
            inputs(),
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        step(&mut ui);
        let shown = |ui: &Ui| -> Vec<String> { texts(ui).into_iter().map(|(_, t)| t).collect() };
        assert_eq!(shown(&ui), ["Untitled", "Ann"]);
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Set {
                source: SourceId::new("test", "footer"),
                value: json!("Bea"),
                force: false,
            })
            .unwrap();
        step(&mut ui);
        assert_eq!(shown(&ui), ["Untitled", "Bea"]);
    }

    /// An element a snippet renders keeps its own content: its node is the
    /// one its path names, under the snippet's name, so it is held from
    /// frame to frame with what stands under it.
    ///
    /// Ways it could fail: the node keyed without the snippet's name (its
    /// element never finds it, and releases its children as unbuilt), or
    /// the name written as a node of its own.
    #[test]
    fn rendered_content_keeps_its_nested_elements() {
        let mut ui = ui();
        ui.define_document(
            UiDocument::parse(
                "nest.svelte",
                r#"<meta name="component" content="nest">
<column id="Page">{@render card()}</column>
{#snippet card()}<row id="Foot"><p id="Step">step {$footer}</p></row>{/snippet}"#,
            )
            .unwrap(),
        );
        ui.mount_document(
            SourceId::new("test", "nest"),
            "nest",
            inputs(),
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        for _ in 0..2 {
            step(&mut ui);
            assert_eq!(
                texts(&ui),
                [(
                    "test:nest/card/Foot/Step".to_string(),
                    "step Ann".to_string()
                )]
            );
        }
        let idle = step(&mut ui);
        assert_eq!(idle.reconciled, 0, "{idle:?}");
    }
}
