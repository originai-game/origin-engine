//! Components and content in mounted documents (docs/ui.md §2.1, Svelte 5
//! props and snippets): a component instance compiles its document's root
//! in a context whose props are its user's expressions, read in the user's
//! context; `{@render name()}` renders the content passed to the instance
//! (or to the mount), else the document's own snippet, in the context it
//! was written in.

use super::*;
use crate::doc::Lit;
use crate::reactive::bindings::{BindingExpression, ContentRef, Lexical, PropRef};

pub(crate) trait RetainedContentRetainedHost {
    /// `<Card title={item.name}>{#snippet body()}…{/snippet}<p>…</p></Card>`:
    /// the component's root, its props `title` and its content `body` and
    /// `children` (the instance's other children).
    #[allow(clippy::too_many_arguments)]
    fn compile_component(
        &mut self,
        ui: &mut Ui,
        source: (Arc<UiDocument>, usize),
        component: &str,
        context: BindingContext,
        parent: Option<&MountedNode>,
        id: String,
        env: &dyn Env,
    ) -> Result<Child, String>;

    /// `{@render name()}`: the content passed under `name`, else the
    /// document's snippet `name`; with `name?.()` nothing when there is
    /// neither.
    #[allow(clippy::too_many_arguments)]
    fn compile_render(
        &mut self,
        ui: &mut Ui,
        source: (Arc<UiDocument>, usize),
        name: &str,
        optional: bool,
        context: BindingContext,
        parent: Option<&MountedNode>,
        id: String,
        env: &dyn Env,
    ) -> Result<Child, String>;
}

impl RetainedContentRetainedHost for RetainedHost {
    #[allow(clippy::too_many_arguments)]
    fn compile_component(
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
        let instance = ui.docs.host().docs.get(component).cloned().ok_or_else(|| {
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
        // A prop its user leaves out reads its default (`$props()`).
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

    #[allow(clippy::too_many_arguments)]
    fn compile_render(
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
                    .host()
                    .docs
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
/// declares with no default (`let { orientation = "vertical" } = $props()`)
/// is refused: the prop would be unbound, and the document built nothing.
/// Content props are passed as snippets and children, not arguments.
fn unpassed(doc: &UiDocument, index: usize, instance: &UiDocument) -> Result<(), String> {
    let node = &doc.nodes[index];
    let missing = instance.props.iter().find(|p| {
        p.default.is_none()
            && p.ty != "content"
            && !node.args.iter().any(|(name, ..)| *name == p.name)
    });
    match missing {
        Some(p) => Err(format!(
            "{}:{}: <{}> leaves out its prop '{}'; pass it, or give it a default in the component (`let {{ {} = … }} = $props()`)",
            doc.source, node.line, instance.name, p.name, p.name
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

/// The content a mount is given, by name: `document` (its root) or
/// `document#snippet` (one of its snippets), rendered with the mount's
/// stores.
pub(super) fn mount_content(
    ui: &Ui,
    content: &BTreeMap<String, String>,
    context: &BindingContext,
) -> Result<Lexical, String> {
    let mut lexical = Lexical::default();
    let written = Arc::new(context.clone());
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
            .host()
            .docs
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

/// The keyed collection an `{#each}` reads: a `$name` input, or a
/// component prop its user bound to one (`<List items={$inventory}>`).
pub(super) fn collection(context: &BindingContext, path: &str) -> Result<SourceId, String> {
    if let Some(name) = path.strip_prefix('$') {
        return context
            .inputs
            .get(name)
            .cloned()
            .ok_or_else(|| format!("mount has no collection '{name}'"));
    }
    match context.lexical.props.get(path) {
        Some(PropRef {
            expression: BindingExpression::Expression(Expr::Path(inner)),
            context: user,
            ..
        }) => collection(user, inner),
        _ => Err(format!(
            "each reads a named collection as $name, or a prop bound to one; '{path}' is neither"
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
            .reactive_mut()
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
            .reactive_mut()
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
            .reactive_mut()
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
            .reactive_mut()
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
}
