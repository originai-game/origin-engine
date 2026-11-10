use super::syntax::{Attr, Error, Node, identifier};
use super::*;
use std::collections::BTreeSet;

mod directives;
mod elements;
mod html;
mod joins;
mod nodes;
mod text;

pub(super) fn parse(
    source: &str,
    text: &str,
    imports: Vec<(String, String)>,
) -> Result<UiDocument, Vec<String>> {
    parse_with_pack(source, text, imports, None)
}

pub(crate) fn parse_with_pack(
    source: &str,
    text: &str,
    mut imports: Vec<(String, String)>,
    pack: Option<&crate::style_pack::StylePack>,
) -> Result<UiDocument, Vec<String>> {
    if let Some(pack) = pack {
        for (id, css) in &pack.sheet.imports {
            imports.retain(|(old, _)| old != id);
            imports.push((id.clone(), css.clone()));
        }
    }
    let run = || -> Result<UiDocument, Error> {
        let nodes = syntax::parse(text)?;
        let mut p = Parser {
            doc: UiDocument {
                kind: DocKind::Screen,
                name: String::new(),
                source: source.into(),
                text: text.into(),
                strings: Vec::new(),
                layer: LayerSpec::of(LayerKind::Menu),
                props: Vec::new(),
                parts: Vec::new(),
                actions: Vec::new(),
                timelines: Vec::new(),
                fixtures: Vec::new(),
                derived: Vec::new(),
                sheet: at_rules::Sheet {
                    imports,
                    ..Default::default()
                },
                part_rules: Vec::new(),
                nodes: Vec::new(),
                root: NONE,
                warnings: Vec::new(),
            },
            ids: BTreeSet::new(),
            selectors: Vec::new(),
            styles: Vec::new(),
            label_fors: Vec::new(),
            datalists: Vec::new(),
            list_fors: Vec::new(),
            aria_refs: Vec::new(),
            consts: Vec::new(),
            script_props: Vec::new(),
            inline_styles: Vec::new(),
            scoped: Vec::new(),
            forms: Vec::new(),
            buttons: Vec::new(),
            fieldsets: Vec::new(),
            novalidate: Vec::new(),
            aria_refs_for: Vec::new(),
            white_space: None,
        };
        let mut script = false;
        let mut window = None;
        for node in &nodes {
            match node {
                Node::Text(text, _) if text.trim().is_empty() => {}
                Node::Element { tag, attrs, at, .. } if tag == "meta" => p.meta(attrs, *at)?,
                Node::Element {
                    tag,
                    attrs,
                    children,
                    at,
                    ..
                } if tag == "svelte:window" => {
                    if window.is_some() || !children.is_empty() {
                        return Err((*at, "a document has one empty <svelte:window/>".into()));
                    }
                    window = Some(attrs.clone());
                }
                Node::Element { tag, children, .. } if tag == "style" => {
                    if let [Node::Text(css, at)] = children.as_slice() {
                        p.styles.push((css.clone(), *at));
                    }
                }
                Node::Snippet { .. } => {
                    p.node(node, NONE)?;
                }
                Node::Script(text, at) => {
                    if std::mem::replace(&mut script, true) {
                        return Err((*at, "a document has one <script>".into()));
                    }
                    let (props, derived) = super::script::read(text, *at)?;
                    if let Some((n, _)) = derived
                        .iter()
                        .find(|(n, _)| p.doc.derived.iter().any(|(m, _)| m == n))
                    {
                        return Err((*at, format!("{n} is declared twice")));
                    }
                    for prop in props {
                        // A `<meta name="prop:…">` before it states the type
                        // and its doc; the script adds the default.
                        if let Some(q) = p.doc.props.iter_mut().find(|q| q.name == prop.name) {
                            q.default = prop.default;
                        } else {
                            p.script_props.push(prop.name.clone());
                            p.doc.props.push(prop);
                        }
                    }
                    p.doc.derived.extend(derived);
                }
                _ => {
                    if p.doc.root != NONE {
                        return Err((0, "a document has one root element; wrap siblings in a column, row or panel".into()));
                    }
                    p.doc.root = p.node(node, NONE)?;
                }
            }
        }
        if p.doc.name.is_empty() {
            return Err((
                0,
                "declare <meta name=\"screen\" content=\"name\"> or name=\"component\"".into(),
            ));
        }
        if p.doc.root == NONE {
            return Err((0, "add one root UI element".into()));
        }
        if let Some(attrs) = window {
            p.window(&attrs)?;
        }
        p.join_html()?;
        p.join_labels()?;
        p.join_lists()?;
        p.join_aria()?;
        super::extern_element::finish(&mut p.doc, &mut p.selectors).map_err(|e| (0, e))?;
        super::stylesheet::apply(
            &mut p.doc,
            &p.selectors,
            &p.styles,
            pack.map(|p| p.css.as_str()),
        )?;
        // Inline declarations come last: they win over every rule but an
        // `!important` one, which an `|important` one wins over in turn.
        for (i, declarations) in std::mem::take(&mut p.inline_styles) {
            let line = p.doc.nodes[i].line;
            for declaration in &declarations {
                p.doc.sheet.assets.extend(
                    at_rules::urls(&declaration.source)
                        .iter()
                        .map(|url| at_rules::resolve(source, url)),
                );
            }
            p.doc.nodes[i].rules.push(StyleRule {
                inline: true,
                layer: u32::MAX,
                starting: false,
                query: AtQuery::default(),
                condition: None,
                structure: None,
                props: Vec::new(),
                declarations,
                line,
            });
        }
        super::script::apply(&mut p.doc, &p.consts).map_err(|e| (0, e))?;
        let mut errors = Vec::new();
        p.doc.check(&mut errors);
        if let Some(error) = errors.first() {
            return Err((0, error.clone()));
        }
        Ok(p.doc)
    };
    run().map_err(|e| vec![syntax::diagnostic(source, text, e)])
}

pub(super) fn expression(source: &str, at: usize) -> Result<Expr, Error> {
    let expr = Expr::parse(source).map_err(|why| (
        at,
        format!("{why}; use a side-effect-free expression or t(key, {{ named: value }}); sources come from the game's mounted stores ($name)"),
    ))?;
    let mut paths = Vec::new();
    expr.paths(&mut paths);
    if paths.iter().any(|path| {
        ["$state", "$derived", "$effect", "$props", "$bindable"]
            .contains(&path.split('.').next().unwrap_or_default())
    }) {
        return Err((
            at,
            "runes are not allowed; sources come from the game's mounted stores ($name)".into(),
        ));
    }
    Ok(expr)
}

pub(super) fn property(name: &str) -> String {
    match name {
        "step" => "stepBy".into(),
        "i18n" => "key".into(),
        "alt" | "aria-label" => "name".into(),
        "aria-invalid" => "invalid".into(),
        "src" => "texture".into(),
        "tabindex" => "tabIndex".into(),
        other => css::camel(other),
    }
}

/// The hint a pre-HTML core tag is refused with: every core element whose
/// role HTML names takes HTML's name (docs/ui.md §2).
fn renamed(tag: &str) -> Option<String> {
    let html = match tag {
        "field" => {
            "<input type=\"text\"> (or password, number, email, url, tel, search), or <textarea> for several lines"
        }
        "slider" => "<input type=\"range\">",
        "toggle" => "<input type=\"checkbox\" switch>",
        "divider" => "<hr>",
        "box" => {
            "<div> (or <section>, <header>, <footer>, <nav>, <main>, <aside>, <article>, <ul>, <ol>, <li>)"
        }
        "image" => "<img>",
        "icon" => "<og-icon>",
        "label" => {
            "<label> around one <input type=\"checkbox\"> and its caption, <label for=\"id\"> naming a control, or <p> (or <text>) for text"
        }
        _ => return None,
    };
    Some(format!("<{tag}> is not a core element; write {html}"))
}

/// A directive the parser maps itself ([`directives`]), never a selector
/// fact (`style:--name` stays one: a custom property is data).
fn directive_name(name: &str) -> bool {
    ["transition:", "in:", "out:", "animate:", "use:"]
        .iter()
        .any(|p| name.starts_with(p))
        || name == "bind:this"
        || name == "style"
        || (name.starts_with("style:") && !name.starts_with("style:--"))
}

/// Inline content a text element reads as runs: words, expressions, the
/// formatting elements, a plain `<span>` and a literal `<a href>`.
fn inline(node: &Node) -> bool {
    match node {
        Node::Text(..) | Node::Expression(..) => true,
        Node::Element { tag, attrs, .. } => match tag.as_str() {
            "b" | "strong" | "i" | "em" | "u" | "s" | "del" | "sub" | "sup" | "br" => {
                attrs.is_empty()
            }
            // A plain span, or one naming its part for `text::part(name)`.
            "span" => attrs.iter().all(|a| a.name == "part" && !a.bound),
            // SVG's run inside a `<text>`: its weight, style, decoration and fill.
            "tspan" => attrs
                .iter()
                .all(|a| !a.bound && super::extern_element::TSPAN_ATTRS.contains(&a.name.as_str())),
            "a" => attrs.iter().all(|a| a.name == "href" && !a.bound),
            _ => false,
        },
        _ => false,
    }
}

/// One `<option>` of a `<select>`.
#[derive(Clone)]
struct SelectOption {
    label: String,
    value: String,
    group: String,
    key: Option<String>,
    disabled: bool,
    selected: bool,
}

/// A `<select>`'s options, in order, through its `<optgroup>`s: literal
/// ones (a bound list is `items={…} values={…}` on the select).
fn select_options(children: &[Node]) -> Result<Vec<SelectOption>, Error> {
    fn walk(
        nodes: &[Node],
        group: Option<(&str, bool)>,
        out: &mut Vec<SelectOption>,
    ) -> Result<(), Error> {
        for node in nodes {
            match node {
                Node::Text(t, _) if t.trim().is_empty() => {}
                Node::Element {
                    tag,
                    attrs,
                    children,
                    at,
                    ..
                } if tag == "optgroup" && group.is_none() => {
                    let literal = |name: &str| attrs.iter().find(|a| a.name == name && !a.bound);
                    let label = literal("label")
                        .map(|a| a.value.as_str())
                        .unwrap_or_default();
                    if label.is_empty() {
                        return Err((*at, "an <optgroup> needs a literal label".into()));
                    }
                    walk(children, Some((label, literal("disabled").is_some())), out)?;
                }
                Node::Element {
                    tag,
                    attrs,
                    children,
                    at,
                    ..
                } if tag == "option" => {
                    let mut o = SelectOption {
                        label: String::new(),
                        value: String::new(),
                        group: group.map(|g| g.0.to_string()).unwrap_or_default(),
                        key: None,
                        disabled: group.is_some_and(|g| g.1),
                        selected: false,
                    };
                    let mut value = None;
                    for a in attrs {
                        if a.bound {
                            return Err((
                                a.at,
                                "an <option>'s attributes are literal; bind a list with items={…} values={…} on the <select>".into(),
                            ));
                        }
                        match a.name.as_str() {
                            "value" => value = Some(a.value.clone()),
                            "label" => o.label = a.value.clone(),
                            "i18n" => o.key = Some(a.value.clone()),
                            "disabled" => o.disabled = true,
                            "selected" => o.selected = true,
                            other => {
                                return Err((
                                    a.at,
                                    format!(
                                        "unknown <option> attribute {other}; it takes value, label, i18n, disabled and selected"
                                    ),
                                ));
                            }
                        }
                    }
                    if o.label.is_empty() {
                        match text_value(children, *at)? {
                            Some((PropValue::Lit(Lit::Str(text)), _)) => o.label = text,
                            Some(_) => {
                                return Err((
                                    *at,
                                    "an <option>'s text is literal; bind a list with items={…} values={…} on the <select>".into(),
                                ));
                            }
                            None => {}
                        }
                    }
                    // HTML: an option without a value is valued by its text.
                    o.value = value.unwrap_or_else(|| o.label.clone());
                    out.push(o);
                }
                Node::Each { at, .. } => {
                    return Err((
                        *at,
                        "a <select> lists literal <option>s; bind a list with items={…} values={…} instead of {#each}".into(),
                    ));
                }
                Node::Element { at, .. }
                | Node::Text(_, at)
                | Node::Expression(_, at)
                | Node::If { at, .. }
                | Node::Snippet { at, .. }
                | Node::Render { at, .. }
                | Node::Key { at, .. }
                | Node::Const { at, .. }
                | Node::Await { at, .. }
                | Node::Script(_, at) => {
                    return Err((
                        *at,
                        "a <select> holds <option>s and <optgroup>s (one level) only".into(),
                    ));
                }
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(children, None, &mut out)?;
    Ok(out)
}

/// The text of a text-only element (a `<summary>`, an `<option>`), its
/// whitespace collapsed as HTML renders it, and its source; `None` when it
/// has none.
fn text_value(children: &[Node], at: usize) -> Result<Option<(PropValue, String)>, Error> {
    let collapse = |t: &str| t.split_ascii_whitespace().collect::<Vec<_>>().join(" ");
    let mut source = String::new();
    let mut text = Expr::Lit("".into());
    let mut bound = false;
    for (k, child) in children.iter().enumerate() {
        let part = match child {
            Node::Text(t, _) => {
                source.push_str(t);
                // One space where text meets an expression, none at the ends.
                let lead = k > 0 && t.starts_with(|c: char| c.is_ascii_whitespace());
                let trail =
                    k + 1 < children.len() && t.ends_with(|c: char| c.is_ascii_whitespace());
                let body = collapse(t);
                let gap = |on: bool| if on && !body.is_empty() { " " } else { "" };
                Expr::Lit(
                    format!(
                        "{}{body}{}",
                        gap(lead),
                        if body.is_empty() && (lead || trail) {
                            " "
                        } else {
                            gap(trail)
                        }
                    )
                    .into(),
                )
            }
            Node::Expression(e, at) => {
                source.push_str(&format!("{{{e}}}"));
                bound = true;
                expression(e, *at)?
            }
            _ => return Err((at, "this element holds text only".into())),
        };
        text = Expr::Bin(crate::bind::Op::Add, Box::new(text), Box::new(part));
    }
    if bound {
        return Ok(Some((PropValue::Bind(text, None), source)));
    }
    let literal = collapse(&source);
    Ok((!literal.is_empty()).then_some((PropValue::Lit(Lit::Str(literal)), source)))
}

pub(super) fn literal(ty: Ty, text: &str) -> Result<PropValue, String> {
    let values = match ty {
        Ty::Text | Ty::Name | Ty::Word(_) => vec![Value::String(text.into())],
        _ => text
            .split_whitespace()
            .map(|s| {
                Number::parse(s)
                    .map(Value::Number)
                    .unwrap_or_else(|_| Value::Word(s.into()))
            })
            .collect(),
    };
    parse_lit(ty, &values).map(PropValue::Lit)
}

struct Parser {
    doc: UiDocument,
    ids: BTreeSet<String>,
    selectors: Vec<(usize, String, Vec<Attr>)>,
    styles: Vec<(String, usize)>,
    /// `<label for>`s, joined to their controls once the tree is read:
    /// (the label's node, the control's id, where `for` is written).
    label_fors: Vec<(usize, String, usize)>,
    /// `<datalist>`s by id: their options' values and labels, and their
    /// bound `items` (a text input's suggestions from a store).
    datalists: Vec<Datalist>,
    /// Inputs' `list`s, joined to their datalists once the tree is read:
    /// (the input's node, the datalist's id, where `list` is written).
    list_fors: Vec<(usize, String, usize)>,
    /// `aria-labelledby` and `aria-describedby`, joined once the tree is
    /// read: (the element, the prop the text goes to, the ids, where).
    aria_refs: Vec<(usize, &'static str, String, usize)>,
    /// `{@const}`s: (their block, name, expression), substituted once the
    /// tree is read ([`super::script::apply`]).
    consts: Vec<(usize, String, Expr)>,
    /// Props the script declared, whose type a `<meta name="prop:…">` may
    /// state.
    script_props: Vec<String>,
    /// Elements' `style` attributes and `style:` directives: declarations
    /// that follow every rule of the stylesheet.
    inline_styles: Vec<(usize, Vec<StyleDeclaration>)>,
    /// Header cells with a `scope` ([`html`]).
    scoped: Vec<usize>,
    /// Forms and their onsubmit and onreset actions.
    forms: Vec<(usize, Option<String>, Option<String>)>,
    /// Buttons with a literal `type`, and where it is written.
    buttons: Vec<(usize, String, usize)>,
    /// `<fieldset disabled>`s and their condition.
    fieldsets: Vec<(usize, Expr)>,
    /// Forms with `novalidate` and buttons with `formnovalidate`.
    novalidate: Vec<usize>,
    /// `<output for>`'s ids, checked once the tree is read.
    aria_refs_for: Vec<(String, usize)>,
    /// An SVG `<text>`'s `white-space` while its runs are read: whether
    /// spaces and line breaks are kept (`None`: both collapse).
    white_space: Option<(bool, bool)>,
}
/// A `<datalist>`: its id, its options' values and labels, and its bound
/// `items`.
type Datalist = (String, Vec<SelectOption>, Option<Expr>);

impl Parser {
    fn line(&self, at: usize) -> usize {
        syntax::location(&self.doc.text, at).0
    }
    fn warn(&mut self, at: usize) {
        self.doc.warnings.push(syntax::diagnostic(
            &self.doc.source,
            &self.doc.text,
            (
                at,
                "visible text is not translatable; use t(key, { parameters }) or i18n=\"key\""
                    .into(),
            ),
        ));
    }
    fn meta(&mut self, attrs: &[Attr], at: usize) -> Result<(), Error> {
        let get = |name| {
            attrs
                .iter()
                .find(|a| a.name == name)
                .map(|a| a.value.as_str())
                .unwrap_or("")
        };
        if attrs.iter().any(|a| a.bound) {
            return Err((at, "metadata is literal; do not bind its attributes".into()));
        }
        let name = get("name");
        let content = get("content");
        if matches!(name, "screen" | "component") {
            if !self.doc.name.is_empty() {
                return Err((at, "declare the screen/component once".into()));
            }
            self.doc.kind = if name == "screen" {
                DocKind::Screen
            } else {
                DocKind::Component
            };
            self.doc.name = content.into();
            let layer = get("data-layer");
            if !layer.is_empty() {
                self.doc.layer = LayerSpec::of(LayerKind::from_name(layer).ok_or((at, "choose a reflected layer: hud, menu, modal, popover, toast, tooltip or system".into()))?);
            }
            self.doc.layer.focus = get("data-focus").into();
            for a in attrs {
                match a.name.as_str() {
                    "name" | "content" | "data-layer" | "data-focus" => {}
                    "data-input" => {
                        self.doc.layer.input = match a.value.as_str() {
                            "game" => InputMode::Game,
                            "menu" => InputMode::Menu,
                            "all" => InputMode::All,
                            _ => return Err((a.at, "data-input is game, menu or all".into())),
                        }
                    }
                    "data-under" => {
                        self.doc.layer.under = match a.value.as_str() {
                            "none" => Under::None,
                            "dim" => Under::Dim,
                            "blur" => Under::Blur,
                            "hide" => Under::Hide,
                            _ => {
                                return Err((a.at, "data-under is none, dim, blur or hide".into()));
                            }
                        }
                    }
                    "data-capture" | "data-pause" | "data-remember" => {
                        let value = a
                            .value
                            .parse::<bool>()
                            .map_err(|_| (a.at, "write true or false".into()))?;
                        match a.name.as_str() {
                            "data-capture" => self.doc.layer.capture = value,
                            "data-pause" => self.doc.layer.pause = value,
                            _ => self.doc.layer.remember = value,
                        }
                    }
                    "data-back" => {
                        self.doc.layer.back = match a.value.as_str() {
                            "pop" => Back::Pop,
                            "none" => Back::None,
                            name => Back::Emit(name.into()),
                        }
                    }
                    _ => {
                        return Err((
                            a.at,
                            format!(
                                "unknown metadata attribute {}; use a documented layer option",
                                a.name
                            ),
                        ));
                    }
                }
            }
        } else if let Some(name) = name.strip_prefix("prop:") {
            let from_script = self.script_props.iter().any(|p| p == name);
            if self.doc.props.iter().any(|p| p.name == name) && !from_script {
                return Err((at, format!("duplicate prop {name}; declare it once")));
            }
            if !identifier(name)
                || ![
                    "number", "string", "bool", "list", "object", "any", "content",
                ]
                .contains(&content)
            {
                return Err((at, "prop metadata needs a name and type number/string/bool/list/object/any/content".into()));
            }
            if from_script {
                // The script declared it; the metadata states its type.
                self.script_props.retain(|p| p != name);
                let p = self.doc.props.iter_mut().find(|p| p.name == name).unwrap();
                p.ty = content.into();
                p.doc = get("data-doc").into();
            } else {
                self.doc.props.push(ContractProp {
                    name: name.into(),
                    ty: content.into(),
                    doc: get("data-doc").into(),
                    default: None,
                });
            }
        } else if let Some(name) = name.strip_prefix("action:") {
            self.doc.actions.push((name.into(), content.into()));
        } else if let Some(name) = name.strip_prefix("fixture:") {
            self.doc.fixtures.push((
                name.into(),
                serde_json::from_str(content)
                    .map_err(|e| (at, format!("fixture must be JSON: {e}")))?,
            ));
        } else if name == "parts" {
            self.doc
                .parts
                .extend(content.split_whitespace().map(str::to_string));
        } else if name == "strings" {
            if content.is_empty() || content.starts_with('/') || content.contains([':', '\\']) {
                return Err((
                    at,
                    "strings names a relative cooked string table, such as ./screen.en.ogt".into(),
                ));
            }
            let base = self
                .doc
                .source
                .rsplit_once('/')
                .map_or("", |(base, _)| base);
            let mut parts: Vec<_> = base.split('/').filter(|p| !p.is_empty()).collect();
            for part in content.split('/') {
                match part {
                    "" | "." => {}
                    ".." => {
                        if parts.pop().is_none() {
                            return Err((
                                at,
                                "strings must stay within the game's asset directory".into(),
                            ));
                        }
                    }
                    part => parts.push(part),
                }
            }
            self.doc.strings.push(parts.join("/"));
        } else {
            return Err((
                at,
                "unknown meta name; use screen, component, prop:…, action:…, fixture:…, strings or parts"
                    .into(),
            ));
        }
        Ok(())
    }
    fn push(&mut self, kind: NodeKind, parent: usize, at: usize) -> usize {
        let index = self.doc.nodes.len();
        self.doc.nodes.push(DocNode {
            kind,
            id: String::new(),
            line: self.line(at),
            source_offset: at,
            selector_tag: None,
            selector_attrs: Vec::new(),
            props: Vec::new(),
            enter: None,
            exit: None,
            at: Vec::new(),
            rules: Vec::new(),
            span_rules: Vec::new(),
            groups: Vec::new(),
            triggers: Vec::new(),
            args: Vec::new(),
            children: Vec::new(),
            parent,
            role: None,
            level: 0,
            inline: Vec::new(),
            links: Vec::new(),
            custom: Vec::new(),
            uses: Vec::new(),
            pseudo: Vec::new(),
        });
        index
    }
    fn children(&mut self, nodes: &[Node], parent: usize) -> Result<Vec<usize>, Error> {
        let mut out = Vec::new();
        for n in nodes {
            match n {
                Node::Text(text, _) if text.trim().is_empty() => {}
                Node::Const { name, expr, at } => {
                    let value = expression(expr, *at)?;
                    self.constant(parent, name, value, *at)?;
                }
                n => out.push(self.node(n, parent)?),
            }
        }
        Ok(out)
    }
    /// `{@const name = value}` in block `parent`.
    fn constant(&mut self, parent: usize, name: &str, value: Expr, at: usize) -> Result<(), Error> {
        let block = parent != NONE
            && matches!(
                self.doc.nodes[parent].kind,
                NodeKind::If(_)
                    | NodeKind::For { .. }
                    | NodeKind::Snippet(_)
                    | NodeKind::Use(_)
                    | NodeKind::Key(_)
            );
        if !block {
            return Err((
                at,
                "{@const} sits directly inside a block ({#if}, {#each}, {#key}, {#await}, {#snippet} or a component); for the whole document write $derived in the <script>".into(),
            ));
        }
        if self
            .consts
            .iter()
            .any(|(b, n, _)| *b == parent && n == name)
        {
            return Err((at, format!("{name} is declared twice in this block")));
        }
        if self.doc.props.iter().any(|p| p.name == name)
            || self.doc.derived.iter().any(|(n, _)| n == name)
        {
            return Err((
                at,
                format!("{name} names a prop or derived value already; choose another name"),
            ));
        }
        self.consts.push((parent, name.into(), value));
        Ok(())
    }
}

#[cfg(test)]
mod tests;
