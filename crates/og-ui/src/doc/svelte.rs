use super::syntax::{Attr, Error, Node, identifier};
use super::*;
use std::collections::BTreeSet;

mod directives;
mod html;

pub(super) fn parse(
    source: &str,
    text: &str,
    imports: Vec<(String, String)>,
) -> Result<UiDocument, Vec<String>> {
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
                keyframes: Vec::new(),
                keyframe_rules: Vec::new(),
                fixtures: Vec::new(),
                derived: Vec::new(),
                sheet: at_rules::Sheet {
                    imports,
                    ..Default::default()
                },
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
        super::stylesheet::apply(&mut p.doc, &p.selectors, &p.styles)?;
        // Inline declarations come last: they win over every rule but an
        // `!important` one, which an `|important` one wins over in turn.
        for (i, declarations) in std::mem::take(&mut p.inline_styles) {
            let line = p.doc.nodes[i].line;
            p.doc.nodes[i].rules.push(StyleRule {
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
type Datalist = (String, Vec<(String, String)>, Option<Expr>);

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
    fn node(&mut self, node: &Node, parent: usize) -> Result<usize, Error> {
        match node {
            Node::If {
                condition,
                yes,
                no,
                at,
            } => {
                let cond = expression(condition, *at)?;
                let i = self.push(NodeKind::If(cond.clone()), parent, *at);
                self.doc.nodes[i].children = self.children(yes, i)?;
                if no.is_empty() {
                    return Ok(i);
                }
                // A transparent branch container preserves both child ranges.
                let wrapper = self.push(NodeKind::If(Expr::Lit(true.into())), parent, *at);
                self.doc.nodes[i].parent = wrapper;
                let other = self.push(NodeKind::If(Expr::Not(Box::new(cond))), wrapper, *at);
                self.doc.nodes[other].children = self.children(no, other)?;
                self.doc.nodes[wrapper].children = vec![i, other];
                Ok(wrapper)
            }
            Node::Each {
                list,
                item,
                key,
                body,
                empty,
                at,
            } => {
                let i = self.push(
                    NodeKind::For {
                        var: item.clone(),
                        list: expression(list, *at)?,
                        key: expression(key, *at)?,
                        empty: Vec::new(),
                    },
                    parent,
                    *at,
                );
                self.doc.nodes[i].children = self.children(body, i)?;
                let empty = self.children(empty, parent)?;
                if let NodeKind::For { empty: target, .. } = &mut self.doc.nodes[i].kind {
                    *target = empty;
                }
                Ok(i)
            }
            Node::Snippet { name, body, at } => {
                if self
                    .doc
                    .nodes
                    .iter()
                    .any(|n| n.parent == parent && n.kind == NodeKind::Snippet(name.clone()))
                {
                    return Err((
                        *at,
                        format!("duplicate snippet {name}; give each snippet a unique name"),
                    ));
                }
                let i = self.push(NodeKind::Snippet(name.clone()), parent, *at);
                self.doc.nodes[i].children = self.children(body, i)?;
                Ok(i)
            }
            Node::Render { name, optional, at } => Ok(self.push(
                NodeKind::Render {
                    name: name.clone(),
                    optional: *optional,
                },
                parent,
                *at,
            )),
            Node::Key { expr, body, at } => {
                let i = self.push(NodeKind::Key(expression(expr, *at)?), parent, *at);
                self.doc.nodes[i].children = self.children(body, i)?;
                Ok(i)
            }
            Node::Const { name, expr, at } => {
                // Reached only outside a block's children (an element's).
                let value = expression(expr, *at)?;
                self.constant(NONE, name, value, *at)?;
                unreachable!("a const outside a block is refused")
            }
            Node::Await {
                expr,
                pending,
                then,
                catch,
                at,
            } => self.await_block(expr, pending, then, catch, parent, *at),
            Node::Script(_, at) => Err((
                *at,
                "a <script> sits at the top of the file, beside the root element".into(),
            )),
            Node::Text(_, at) | Node::Expression(_, at) => Err((
                *at,
                "text needs a text-bearing element such as <text> or <button>".into(),
            )),
            Node::Element {
                tag,
                attrs,
                children,
                at,
                ..
            } => self.element(tag, attrs, children, *at, parent),
        }
    }
    /// `{#await $load}`: three `{#if}`s over the settled record its store
    /// holds (`status` `pending`, `fulfilled` or `rejected`, Promise.allSettled's
    /// vocabulary), `{:then v}` naming its `value` and `{:catch e}` its
    /// `reason` as `{@const}` does. Nothing is awaited in the tick.
    fn await_block(
        &mut self,
        expr: &str,
        pending: &[Node],
        then: &Option<(String, Vec<Node>)>,
        catch: &Option<(String, Vec<Node>)>,
        parent: usize,
        at: usize,
    ) -> Result<usize, Error> {
        let Expr::Path(path) = expression(expr, at)? else {
            return Err((
                at,
                "{#await} reads a store or row field holding { status, value, reason }, such as {#await $load}; a promise never runs in the tick".into(),
            ));
        };
        let status = |s: &str| {
            Expr::Bin(
                crate::bind::Op::Eq,
                Box::new(Expr::Path(format!("{path}.status"))),
                Box::new(Expr::Lit(s.into())),
            )
        };
        let settled = Expr::Bin(
            crate::bind::Op::Or,
            Box::new(status("fulfilled")),
            Box::new(status("rejected")),
        );
        let wrapper = self.push(NodeKind::If(Expr::Lit(true.into())), parent, at);
        let mut arms = Vec::new();
        let branches = [
            (Expr::Not(Box::new(settled)), None, pending),
            (
                status("fulfilled"),
                then.as_ref().map(|(n, _)| (n, "value")),
                then.as_ref().map_or(&[][..], |(_, b)| b),
            ),
            (
                status("rejected"),
                catch.as_ref().map(|(n, _)| (n, "reason")),
                catch.as_ref().map_or(&[][..], |(_, b)| b),
            ),
        ];
        for (condition, alias, body) in branches {
            if body.is_empty() && alias.is_none() {
                continue;
            }
            let arm = self.push(NodeKind::If(condition), wrapper, at);
            if let Some((name, field)) = alias.filter(|(n, _)| !n.is_empty()) {
                let value = Expr::Path(format!("{path}.{field}"));
                self.constant(arm, name, value, at)?;
            }
            self.doc.nodes[arm].children = self.children(body, arm)?;
            arms.push(arm);
        }
        self.doc.nodes[wrapper].children = arms;
        Ok(wrapper)
    }
    /// `<svelte:element this={tag}>`: a literal tag is that element; a
    /// bound one is an `{#if}` over the text tags (when its content is
    /// inline) or the block tags, and nothing for another value.
    fn dynamic_element(
        &mut self,
        attrs: &[Attr],
        children: &[Node],
        at: usize,
        parent: usize,
    ) -> Result<usize, Error> {
        let Some(this) = attrs.iter().find(|a| a.name == "this") else {
            return Err((at, "<svelte:element> needs this={tag}".into()));
        };
        let rest: Vec<Attr> = attrs.iter().filter(|a| a.name != "this").cloned().collect();
        if !this.bound {
            if this.value.starts_with("svelte:") {
                return Err((this.at, "this names an HTML or core tag".into()));
            }
            return self.element(&this.value, &rest, children, at, parent);
        }
        let tags: &[&str] = if children.iter().all(inline) {
            &["p", "span", "small", "h1", "h2", "h3", "h4", "h5", "h6"]
        } else {
            &[
                "div", "section", "header", "footer", "nav", "main", "aside", "article",
            ]
        };
        let mut chain: Vec<Node> = Vec::new();
        for tag in tags.iter().rev() {
            chain = vec![Node::If {
                condition: format!("({}) == '{tag}'", this.value),
                yes: vec![Node::Element {
                    tag: (*tag).into(),
                    attrs: rest.clone(),
                    children: children.to_vec(),
                    at,
                    end: at,
                    close: at,
                }],
                no: chain,
                at,
            }];
        }
        self.node(&chain[0], parent)
    }
    /// `<svelte:window>`'s attributes, on the root once it is read:
    /// `onresize={action}` when the screen's size changes, and
    /// `bind:innerWidth={w}` / `bind:innerHeight={h}` naming its size.
    fn window(&mut self, attrs: &[Attr]) -> Result<(), Error> {
        for a in attrs {
            match a.name.as_str() {
                "onresize" => {
                    if !a.bound || !identifier(&a.value) {
                        return Err((
                            a.at,
                            "onresize names a mounted action: onresize={relayout}".into(),
                        ));
                    }
                    let size = expression("$ui.width + 'x' + $ui.height", a.at)?;
                    let line = self.line(a.at);
                    self.doc.nodes[self.doc.root].triggers.push(Trigger {
                        when: When::Change(size),
                        act: Act::Emit(a.value.clone()),
                        line,
                    });
                }
                "bind:innerWidth" | "bind:innerHeight" => {
                    if !a.bound || !identifier(&a.value) {
                        return Err((
                            a.at,
                            format!(
                                "{} names the size for the document (bind:innerWidth={{width}}); a store reads $ui.width",
                                a.name
                            ),
                        ));
                    }
                    let field = if a.name.ends_with("Width") {
                        "width"
                    } else {
                        "height"
                    };
                    if self.doc.derived.iter().any(|(n, _)| *n == a.value) {
                        return Err((a.at, format!("{} is declared twice", a.value)));
                    }
                    self.doc
                        .derived
                        .push((a.value.clone(), Expr::Path(format!("$ui.{field}"))));
                }
                name if name.starts_with("onkey") => {
                    return Err((
                        a.at,
                        format!(
                            "{name}: keys reach a game as input actions; declare <meta name=\"action:confirm\" content=\"…\"> and write onconfirm={{…}}"
                        ),
                    ));
                }
                other => {
                    return Err((
                        a.at,
                        format!(
                            "<svelte:window> takes onresize, bind:innerWidth and bind:innerHeight, not {other}"
                        ),
                    ));
                }
            }
        }
        Ok(())
    }
    fn element(
        &mut self,
        tag: &str,
        attrs: &[Attr],
        children: &[Node],
        at: usize,
        parent: usize,
    ) -> Result<usize, Error> {
        // An SVG <text>: its white-space read here, its lines split into
        // texts of their own.
        let kept;
        let mut white_space = None;
        self.white_space = None;
        let attrs = if tag == "text" && super::extern_element::in_svg(self.parent_tag(parent)) {
            let mut rest = Vec::new();
            for a in attrs {
                if a.name == "white-space" || a.name == "xml:space" {
                    if a.bound {
                        return Err((a.at, format!("{} is literal", a.name)));
                    }
                    white_space =
                        super::extern_element::white_space(&a.value).map_err(|e| (a.at, e))?;
                } else {
                    rest.push(a.clone());
                }
            }
            if let Some(i) = self.svg_lines(&rest, white_space, children, at, parent)? {
                return Ok(i);
            }
            // Read by its runs (element_node, below).
            self.white_space = white_space;
            kept = rest;
            &kept[..]
        } else {
            attrs
        };
        match tag {
            "svelte:element" => return self.dynamic_element(attrs, children, at, parent),
            "svelte:window" => {
                return Err((
                    at,
                    "<svelte:window> sits at the top of the file, beside the root element".into(),
                ));
            }
            "svelte:document" | "svelte:body" | "svelte:head" => {
                return Err((
                    at,
                    format!(
                        "<{tag}> has nothing to reach in a game: its UI is one layer stack; listen with <svelte:window onresize> or an input action (<meta name=\"action:…\">)"
                    ),
                ));
            }
            "svelte:boundary" => {
                return Err((
                    at,
                    "<svelte:boundary> catches errors a document never throws while it runs: its expressions are checked when it cooks, and a missing store fails the mount with its path".into(),
                ));
            }
            "svelte:options" | "svelte:fragment" | "svelte:self" | "svelte:component" => {
                return Err((
                    at,
                    format!(
                        "<{tag}> is not Svelte 5 markup a document takes; use a capitalised component, a snippet or <svelte:element this>"
                    ),
                ));
            }
            "option" | "optgroup" => {
                return Err((at, format!("<{tag}> belongs inside a <select>")));
            }
            "summary" => {
                return Err((at, "<summary> is the first child of a <details>".into()));
            }
            "datalist" => {
                // HTML renders none: its options are the values a range's
                // `list` marks, or a text input's suggestions.
                let Some(id) = attrs.iter().find(|a| a.name == "id" && !a.bound) else {
                    return Err((
                        at,
                        "a <datalist> needs a literal id, which an input's list names".into(),
                    ));
                };
                if let Some(a) = attrs
                    .iter()
                    .find(|a| a.name != "id" && !(a.name == "items" && a.bound))
                {
                    return Err((
                        a.at,
                        format!(
                            "a <datalist> takes only an id and items={{$names}}, not {}",
                            a.name
                        ),
                    ));
                }
                let items = attrs
                    .iter()
                    .find(|a| a.name == "items")
                    .map(|a| expression(&a.value, a.at))
                    .transpose()?;
                if !self.ids.insert(id.value.clone()) {
                    return Err((
                        id.at,
                        format!(
                            "duplicate id '{}'; IDs must be unique within a document",
                            id.value
                        ),
                    ));
                }
                let options = select_options(children)?;
                // An option labelled only by its value is a tick without text.
                let marks = options
                    .into_iter()
                    .map(|o| {
                        let label = if o.label == o.value {
                            String::new()
                        } else {
                            o.label
                        };
                        (o.value.trim().to_string(), label)
                    })
                    .collect();
                self.datalists.push((id.value.clone(), marks, items));
                return Ok(self.push(NodeKind::If(Expr::Lit(false.into())), parent, at));
            }
            "select" => {
                let options = select_options(children)?;
                let i = self.element_node(tag, attrs, &[], at, parent)?;
                let strs = |v: Vec<String>| PropValue::Lit(Lit::Strs(v));
                let items = options.iter().map(|o| o.label.clone()).collect();
                let values = options.iter().map(|o| o.value.clone()).collect();
                // Without `<option>`s its options are its bound `items`.
                if !options.is_empty() {
                    self.prop(i, tag, "items", strs(items), at, "")?;
                    self.prop(i, tag, "values", strs(values), at, "")?;
                }
                if options.iter().any(|o| !o.group.is_empty()) {
                    let groups = options.iter().map(|o| o.group.clone()).collect();
                    self.prop(i, tag, "groups", strs(groups), at, "")?;
                }
                if options.iter().any(|o| o.key.is_some()) {
                    let keys = options
                        .iter()
                        .map(|o| o.key.clone().unwrap_or_default())
                        .collect();
                    self.prop(i, tag, "keys", strs(keys), at, "")?;
                }
                let off: Vec<String> = (0..options.len())
                    .filter(|&k| options[k].disabled)
                    .map(|k| k.to_string())
                    .collect();
                if !off.is_empty() {
                    self.prop(i, tag, "disabledItems", strs(off), at, "")?;
                }
                // `<option selected>` is the value when nothing is bound.
                let valued = attrs
                    .iter()
                    .any(|a| matches!(a.name.as_str(), "value" | "bind:value"));
                if !valued && let Some(o) = options.iter().find(|o| o.selected) {
                    self.prop(
                        i,
                        tag,
                        "current",
                        PropValue::Lit(Lit::Str(o.value.clone())),
                        at,
                        "",
                    )?;
                }
                return Ok(i);
            }
            "details" => {
                let mut rest = Vec::new();
                let mut summary = None;
                for child in children {
                    match child {
                        Node::Element {
                            tag: t,
                            children: inner,
                            at,
                            ..
                        } if t == "summary" => {
                            if summary.is_some()
                                || rest.iter().any(|c| matches!(c, Node::Element { .. }))
                            {
                                return Err((
                                    *at,
                                    "a <details> takes one <summary>, as its first child".into(),
                                ));
                            }
                            summary = Some((text_value(inner, *at)?, *at));
                        }
                        other => rest.push(other.clone()),
                    }
                }
                let i = self.element_node(tag, attrs, &rest, at, parent)?;
                if let Some((Some((value, source)), at)) = summary {
                    self.prop(i, tag, "summary", value, at, &source)?;
                }
                return Ok(i);
            }
            _ => {}
        }
        self.element_node(tag, attrs, children, at, parent)
    }
    fn element_node(
        &mut self,
        tag: &str,
        attrs: &[Attr],
        children: &[Node],
        at: usize,
        parent: usize,
    ) -> Result<usize, Error> {
        // A label around a checkbox is the checkbox, its text the caption
        // (HTML's implicit label association).
        if tag == "label" {
            // A `<small>` in it is the checkbox's description line.
            let small = |c: &Node| matches!(c, Node::Element { tag, .. } if tag == "small");
            let controls: Vec<&Node> = children
                .iter()
                .filter(|c| matches!(c, Node::Element { .. }) && !small(c))
                .collect();
            if let [
                Node::Element {
                    tag: inner,
                    attrs: inner_attrs,
                    children: inner_children,
                    at: inner_at,
                    ..
                },
            ] = controls[..]
                && inner == "input"
                && inner_children.is_empty()
                && inner_attrs.iter().any(|a| {
                    a.name == "type" && !a.bound && matches!(a.value.as_str(), "checkbox" | "radio")
                })
            {
                let control = if inner_attrs
                    .iter()
                    .any(|a| a.name == "type" && a.value == "radio")
                {
                    "radio"
                } else {
                    "toggle"
                };
                let caption: Vec<Node> = children
                    .iter()
                    .filter(|c| !matches!(c, Node::Element { .. }))
                    .cloned()
                    .collect();
                // HTML lays the two out in source order: a caption written
                // after the checkbox sits after it.
                let input_at = children
                    .iter()
                    .position(|c| matches!(c, Node::Element { tag, .. } if tag == "input"));
                let text_at = children.iter().position(|c| match c {
                    Node::Text(t, _) => !t.trim().is_empty(),
                    Node::Expression(..) => true,
                    _ => false,
                });
                let after = matches!((input_at, text_at), (Some(i), Some(t)) if i < t);
                let mut description = None;
                for c in children.iter().filter(|c| small(c)) {
                    let Node::Element {
                        attrs: small_attrs,
                        children: inner,
                        at: small_at,
                        ..
                    } = c
                    else {
                        continue;
                    };
                    if description.is_some() || !small_attrs.is_empty() {
                        return Err((
                            *small_at,
                            "a checkbox's <label> takes one plain <small>, its description".into(),
                        ));
                    }
                    description = text_value(inner, *small_at)?.map(|d| (d, *small_at));
                }
                // The two are one node: the label's attributes (an id, a
                // class) are the checkbox's, and one set on both is refused.
                let mut merged = inner_attrs.clone();
                for a in attrs {
                    if inner_attrs.iter().any(|b| b.name == a.name) {
                        return Err((
                            a.at,
                            format!(
                                "{} is set on both the <label> and its checkbox; they are one control, set it once",
                                a.name
                            ),
                        ));
                    }
                    merged.push(a.clone());
                }
                let i = self.element("input", &merged, &caption, *inner_at, parent)?;
                if after {
                    self.prop(
                        i,
                        control,
                        "side",
                        PropValue::Lit(Lit::Str("end".into())),
                        *inner_at,
                        "",
                    )?;
                }
                if let Some(((value, source), at)) = description {
                    self.prop(i, control, "description", value, at, &source)?;
                }
                return Ok(i);
            }
        }
        let label_for = match attrs.iter().find(|a| a.name == "for") {
            Some(a) if tag == "label" => {
                if a.bound || a.value.is_empty() {
                    return Err((a.at, "for names a control by its literal id".into()));
                }
                Some(a)
            }
            _ => None,
        };
        if tag == "li" && !self.in_list(parent) {
            return Err((at, "<li> belongs inside a <ul> or an <ol>".into()));
        }
        let html = self.html_kind(tag, parent, at)?;
        let internal = match tag {
            _ if let Some((kind, _)) = html => kind,
            // SVG's text: placed in its view box (extern_element).
            "text" if super::extern_element::in_svg(self.parent_tag(parent)) => "svgText",
            "text" | "p" | "span" | "small" | "a" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                "label"
            }
            "label" if label_for.is_some() => "label",
            "div" | "section" | "header" | "footer" | "nav" | "main" | "aside" | "article"
            | "ul" | "ol" | "li" | "menu" => "box",
            "img" => "image",
            "video" => "video",
            "audio" => "audio",
            "hr" => "divider",
            "textarea" => "field",
            // An input's kind is its literal type's: a range is a slider, a
            // checkbox a toggle, every text type a field.
            "input" => match attrs
                .iter()
                .find(|a| a.name == "type" && !a.bound)
                .map(|a| a.value.as_str())
            {
                Some("range") => "slider",
                Some("checkbox") => "toggle",
                Some("radio") => "radio",
                Some("color") => "color",
                Some("date" | "time" | "datetime-local" | "month" | "week") => "datetime",
                Some("file") => "file",
                _ => "field",
            },
            other => match super::extern_element::kind(other, self.parent_tag(parent)) {
                Some(Ok(kind)) => kind,
                Some(Err(hint)) => return Err((at, hint)),
                None => match renamed(other) {
                    Some(hint) => return Err((at, hint)),
                    None => other,
                },
            },
        };
        let component = tag.starts_with(char::is_uppercase);
        let kind = if component {
            NodeKind::Use(tag.into())
        } else {
            NodeKind::Element(schema::kind(internal).ok_or((at, format!("unknown element <{tag}>; use a reflected core tag or capitalised component")))?.name)
        };
        let i = self.push(kind, parent, at);
        self.doc.nodes[i].selector_tag = Some(tag.into());
        if let Some(a) = label_for {
            self.label_fors.push((i, a.value.clone(), a.at));
        }
        let named = attrs
            .iter()
            .any(|a| matches!(a.name.as_str(), "aria-label" | "name"));
        // HTML's implicit roles (landmarks, lists); a section is a region
        // only when named.
        let in_menu = tag == "button" && self.parent_tag(parent) == Some("menu");
        self.doc.nodes[i].role = match tag {
            "menu" => Some("menu"),
            "button" if in_menu => Some("menuitem"),
            "nav" => Some("navigation"),
            "main" => Some("main"),
            "header" => Some("banner"),
            "footer" => Some("contentinfo"),
            "aside" => Some("complementary"),
            "article" => Some("article"),
            "section" if named => Some("region"),
            "ul" | "ol" => Some("list"),
            "li" => Some("listitem"),
            _ => html.and_then(|(_, role)| role),
        };
        for attr in attrs
            .iter()
            .filter(|a| !a.name.starts_with("on") && !directive_name(&a.name))
        {
            let value = if attr.bound || attr.name.starts_with("class:") {
                expression(&attr.value, attr.at)?
            } else {
                Expr::Lit(attr.value.clone().into())
            };
            self.doc.nodes[i]
                .selector_attrs
                .push((attr.name.clone(), value));
        }
        self.selectors.push((i, tag.into(), attrs.to_vec()));
        let smil = crate::svg::smil::TAGS.contains(&tag);
        if smil || tag == "mpath" {
            if let Some(a) = attrs.iter().find(|a| a.bound) {
                return Err((
                    a.at,
                    format!(
                        "<{tag}>'s attributes are literal: it plays on the UI clock; animate bound values with og.tween or a CSS transition"
                    ),
                ));
            }
            if let Some(a) = attrs
                .iter()
                .find(|a| tag == "mpath" && a.name != "href" && a.name != "id")
            {
                return Err((a.at, format!("<mpath> takes href=\"#id\", not {}", a.name)));
            }
            if smil {
                let literal: Vec<(String, String)> = attrs
                    .iter()
                    .map(|a| (a.name.clone(), a.value.clone()))
                    .collect();
                // An `<mpath>` child names the path later: stand in for it.
                let mpath = (tag == "animateMotion").then(crate::svg::Path::default);
                crate::svg::smil::Animation::parse(tag, &literal, mpath).map_err(|e| (at, e))?;
            }
        }
        for a in attrs {
            if a.name == "id" {
                if a.bound || a.value.is_empty() || a.value.contains(['/', '#']) {
                    return Err((
                        a.at,
                        "id is a nonempty literal without '/' or '#'; bind data, not identity"
                            .into(),
                    ));
                }
                if !self.ids.insert(a.value.clone()) {
                    return Err((
                        a.at,
                        format!(
                            "duplicate id '{}'; IDs must be unique within a document",
                            a.value
                        ),
                    ));
                }
                self.doc.nodes[i].id = a.value.clone();
                continue;
            }
            // An animation's attributes are read as it plays (svg::smil).
            if smil || tag == "mpath" {
                continue;
            }
            if a.name == "class" || a.name.starts_with("class:") {
                if a.bound {
                    expression(&a.value, a.at)?;
                }
                continue;
            }
            if self.directive(i, internal, a)? || self.html_attr(i, tag, a)? {
                continue;
            }
            if let Some(event) = a.name.strip_prefix("on") {
                let when = match event {
                    "click" => When::Click,
                    "change" | "input" => When::Changed,
                    "tap" => When::Tap,
                    "doubletap" => When::DoubleTap,
                    "longpress" => When::LongPress,
                    other if let Some((_, w)) = When::EVENTS.iter().find(|(e, _)| *e == other) => {
                        w.clone()
                    }
                    action if self.doc.actions.iter().any(|(name, _)| name == action) => {
                        When::Action(action.into())
                    }
                    _ => {
                        return Err((
                            a.at,
                            format!(
                                "unknown event {event}; use onclick, onchange, ontap, ondragstart, ondrop, onpointerdown, onwheel…, or declare an input action with <meta name=\"action:{event}\" content=\"Label\">"
                            ),
                        ));
                    }
                };
                if !a.bound || !identifier(&a.value) {
                    return Err((
                        a.at,
                        "event handlers name a mounted action: onclick={next}; no executable calls"
                            .into(),
                    ));
                }
                let line = self.line(a.at);
                self.doc.nodes[i].triggers.push(Trigger {
                    when,
                    act: Act::Emit(a.value.clone()),
                    line,
                });
                continue;
            }
            if let Some(name) = a
                .name
                .strip_prefix("style:")
                .filter(|n| n.starts_with("--"))
            {
                // A bound custom property: the data a rule's var() reads.
                if name.len() < 3
                    || !name[2..]
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
                {
                    return Err((a.at, format!("{name} is not a custom property name")));
                }
                let value = if a.bound {
                    expression(&a.value, a.at)?
                } else {
                    Expr::Lit(serde_json::Value::String(a.value.clone()))
                };
                self.doc.nodes[i].custom.push((name.to_string(), value));
                continue;
            }
            // A block's key translates its anonymous text.
            if internal == "box" && a.name == "i18n" {
                continue;
            }
            // Read as selector facts: a label's control, a list's numbering.
            if (label_for.is_some() && a.name == "for")
                || (tag == "ol" && matches!(a.name.as_str(), "start" | "reversed"))
                || (tag == "li" && a.name == "value")
            {
                if a.name != "for"
                    && a.name != "reversed"
                    && !a.bound
                    && a.value.trim().parse::<i64>().is_err()
                {
                    return Err((a.at, format!("{} takes an integer", a.name)));
                }
                continue;
            }
            // ARIA naming by reference: the named elements' text, joined
            // once the tree is read.
            if let Some(prop) = match a.name.as_str() {
                "aria-labelledby" => Some("name"),
                "aria-describedby" => Some("ariaDescription"),
                _ => None,
            } {
                if a.bound || a.value.trim().is_empty() {
                    return Err((
                        a.at,
                        format!("{} names elements by their literal ids", a.name),
                    ));
                }
                self.aria_refs.push((i, prop, a.value.clone(), a.at));
                continue;
            }
            if a.name == "role" && !a.bound && !crate::a11y::ROLES.contains(&a.value.as_str()) {
                return Err((
                    a.at,
                    format!(
                        "role \"{}\" is not an accessibility role; take one of {}",
                        a.value,
                        crate::a11y::ROLES.join(", ")
                    ),
                ));
            }
            // A range's ticks, or a text input's suggestions: the
            // `<datalist>` it names, joined once the tree is read.
            if tag == "input" && a.name == "list" {
                if internal != "slider" && internal != "field" {
                    return Err((
                        a.at,
                        "list names a range's ticks or a text input's suggestions".into(),
                    ));
                }
                if a.bound || a.value.is_empty() {
                    return Err((a.at, "list names a <datalist> by its literal id".into()));
                }
                self.list_fors.push((i, a.value.clone(), a.at));
                continue;
            }
            // A bare `popover` is HTML's auto.
            if a.name == "popover" && !a.bound && matches!(a.value.as_str(), "" | "true") {
                self.prop(
                    i,
                    internal,
                    "popover",
                    PropValue::Lit(Lit::Str("auto".into())),
                    a.at,
                    "auto",
                )?;
                continue;
            }
            // An input's type chose its kind; a checkbox's `switch` is how
            // it draws, and a toggle is a switch.
            if tag == "input" && internal == "toggle" && a.name == "switch" && !a.bound {
                self.prop(
                    i,
                    internal,
                    "switch",
                    PropValue::Lit(Lit::Bool(true)),
                    a.at,
                    "",
                )?;
                continue;
            }
            if tag == "input" && !matches!(internal, "field" | "datetime") && a.name == "type" {
                continue;
            }
            // A radio's group is the store it binds; its `name` adds nothing.
            if internal == "radio" && a.name == "name" {
                continue;
            }
            if a.name == "multiline" || (tag == "textarea" && a.name == "type") {
                return Err((
                    a.at,
                    "write several lines as <textarea>, one line as <input type=…>".into(),
                ));
            }
            if internal == "field"
                && a.name == "pattern"
                && !a.bound
                && crate::input_type::pattern(&a.value).is_none()
            {
                return Err((
                    a.at,
                    format!("pattern \"{}\" is not a regular expression", a.value),
                ));
            }
            if internal == "field"
                && let Some(html) = match a.name.as_str() {
                    "max-chars" | "maxChars" => Some("maxlength"),
                    "min-chars" | "minChars" => Some("minlength"),
                    "lines" => Some("rows on a <textarea>"),
                    _ => None,
                }
            {
                return Err((
                    a.at,
                    format!("{} is not a field attribute; write {html}", a.name),
                ));
            }
            // HTML's custom data attributes (but the engine's reflected
            // `data-og-*` ones): read by selectors and attr().
            if a.name.starts_with("data-") && schema::prop(&property(&a.name)).is_none() {
                continue;
            }
            let model = a.name.strip_prefix("bind:");
            let mut name = match (internal, model.unwrap_or(&a.name)) {
                ("field", "maxlength") => "maxChars".into(),
                ("field", "minlength") => "minChars".into(),
                ("field", "rows") if tag == "textarea" => "lines".into(),
                (_, other) => super::extern_element::property(internal, other)
                    .unwrap_or_else(|| property(other)),
            };
            if name == "value" && internal == "field" {
                name = "input".into();
            }
            if name == "value" && internal == "select" {
                name = "current".into();
            }
            if name == "value" && internal == "color" {
                name = "colorValue".into();
            }
            if matches!(internal, "video" | "audio") && a.name == "src" {
                if !a.bound {
                    return Err((
                        a.at,
                        format!(
                            "<{tag}> plays a player the game owns, not a file: spawn an og.{} with the asset in a system and bind its entity, src={{$player}}",
                            if tag == "video" {
                                "VideoPlayer"
                            } else {
                                "AudioSource"
                            }
                        ),
                    ));
                }
                name = "mediaSrc".into();
            }
            if internal == "audio"
                && matches!(
                    name.as_str(),
                    "currentTime" | "duration" | "ended" | "poster"
                )
            {
                return Err((
                    a.at,
                    format!(
                        "<audio> has no {name}: an og.AudioSource's playhead is not read; use <video> for footage"
                    ),
                ));
            }
            if internal == "file" && name == "value" {
                name = "fileValue".into();
            }
            if internal == "datetime" {
                match name.as_str() {
                    "value" => name = "dateValue".into(),
                    "type" => name = "dateType".into(),
                    "min" => name = "dateMin".into(),
                    "max" => name = "dateMax".into(),
                    _ => {}
                }
            }
            if name == "value" && internal == "radio" {
                name = "choice".into();
            }
            if name == "value" && internal == "kbd" {
                name = "binding".into();
            }
            if a.name == "hidden" {
                name = "visible".into();
            }
            // Selector facts only: `:lang()`, `:dir()`, `[data-…]` and
            // `:state(name)` (CSS custom states, `state:name={…}`).
            if !component
                && model.is_none()
                && schema::prop(&name).is_none()
                && (matches!(a.name.as_str(), "lang" | "dir")
                    || a.name.starts_with("data-")
                    || a.name.strip_prefix("state:").is_some_and(identifier))
            {
                if a.bound {
                    expression(&a.value, a.at)?;
                }
                continue;
            }
            let value = if model.is_some() {
                let expr = expression(&a.value, a.at)?;
                let Expr::Path(path) = expr else {
                    return Err((
                        a.at,
                        "bind: needs a writable store or row-field path".into(),
                    ));
                };
                if !a.bound {
                    return Err((a.at, "write bind:value={$store}, not a literal".into()));
                }
                PropValue::Model(path)
            } else if a.bound {
                PropValue::Bind(expression(&a.value, a.at)?, None)
            } else if name == "stepBy" && a.value == "any" {
                // HTML's step="any": no step constraint.
                PropValue::Lit(Lit::Num(0.0))
            } else if let Some((_, spec)) = schema::prop(&name) {
                literal(spec.ty, &a.value).map_err(|why| (a.at, why))?
            } else if component {
                PropValue::Lit(Lit::Str(a.value.clone()))
            } else {
                return Err((
                    a.at,
                    format!("unknown attribute {}; use a reflected property", a.name),
                ));
            };
            let value = if a.name == "hidden" {
                match value {
                    PropValue::Bind(e, _) => PropValue::Bind(Expr::Not(Box::new(e)), None),
                    PropValue::Lit(Lit::Bool(b)) => PropValue::Lit(Lit::Bool(!b)),
                    _ => return Err((a.at, "hidden takes a boolean".into())),
                }
            } else {
                value
            };
            if matches!(name.as_str(), "placeholder" | "name") && !a.bound && !a.value.is_empty() {
                self.warn(a.at);
            }
            if component {
                let line = self.line(a.at);
                self.doc.nodes[i].args.push((name, value, line));
            } else {
                self.prop(i, internal, &name, value, a.at, &a.value)?;
            }
        }
        // An action's glyph follows the device: it reads `$ui.device`.
        if internal == "kbd" && attrs.iter().any(|a| a.name == "data-action") {
            let device = expression("$ui.device", at)?;
            self.prop(
                i,
                internal,
                "glyphDevice",
                PropValue::Bind(device, None),
                at,
                "",
            )?;
        }
        if tag == "textarea" {
            self.prop(
                i,
                internal,
                "multiline",
                PropValue::Lit(Lit::Bool(true)),
                at,
                "",
            )?;
        }
        if let Some(level) = tag
            .strip_prefix('h')
            .and_then(|l| l.parse::<u8>().ok())
            .filter(|l| (1..=6).contains(l))
        {
            self.prop(
                i,
                internal,
                "heading",
                PropValue::Lit(Lit::Bool(true)),
                at,
                "",
            )?;
            self.doc.nodes[i].level = level;
        }
        let translated = attrs.iter().any(|a| a.name == "i18n");
        // A button holding elements lays them out in a row inside it, its
        // text between them an anonymous text as a block's is.
        let content_button = internal == "button"
            && !component
            && children
                .iter()
                .any(|c| matches!(c, Node::Element { .. }) && !inline(c));
        if internal == "box" || content_button {
            // Text between a block's elements is an anonymous text
            // element, its inline markup kept (HTML's anonymous block box).
            let key = attrs.iter().find(|a| a.name == "i18n");
            let mut texts = Vec::new();
            let mut run = Vec::new();
            for child in children {
                if inline(child) {
                    run.push(child.clone());
                    continue;
                }
                texts.extend(self.anonymous_text(&mut run, i, translated)?);
                let child = self.node(child, i)?;
                self.doc.nodes[i].children.push(child);
            }
            texts.extend(self.anonymous_text(&mut run, i, translated)?);
            if let Some(a) = key {
                let [text] = texts[..] else {
                    return Err((
                        a.at,
                        format!(
                            "i18n on a <{tag}> translates its one run of text; put each run in its own <span i18n>"
                        ),
                    ));
                };
                let value = if a.bound {
                    PropValue::Bind(expression(&a.value, a.at)?, None)
                } else {
                    PropValue::Lit(Lit::Str(a.value.clone()))
                };
                self.prop(text, "label", "key", value, a.at, &a.value)?;
            }
            return Ok(i);
        }
        if internal == "svgText"
            || internal == "label"
                && !component
                && (tag == "a" || children.iter().any(|c| matches!(c, Node::Element { .. })))
        {
            if translated && children.iter().any(|c| matches!(c, Node::Element { .. })) {
                return Err((
                    at,
                    "i18n replaces the whole text, so it takes no inline elements; put the markup in the string".into(),
                ));
            }
            let own = (tag == "a").then_some(usize::MAX);
            if internal == "svgText" {
                let children = self.text_path(i, children)?;
                let read = self.text_runs(i, &children, own, at, translated);
                self.white_space = None;
                read?;
                return Ok(i);
            }
            self.text_runs(i, children, own, at, translated)?;
            return Ok(i);
        }
        let mut text = Expr::Lit(serde_json::Value::String(String::new()));
        let mut has_text = false;
        let mut bound_text = false;
        let text_bearing = schema::prop("text").is_some_and(|(_, p)| p.applies(internal));
        for (index, child) in children.iter().enumerate() {
            let value = match child {
                Node::Text(value, at) => {
                    if value.trim().is_empty() && !text_bearing {
                        continue;
                    }
                    if !translated && !value.trim().is_empty() {
                        self.warn(*at);
                    }
                    let mut value = value.chars().fold(String::new(), |mut out, c| {
                        if c.is_ascii_whitespace() {
                            if !out.ends_with(' ') {
                                out.push(' ');
                            }
                        } else {
                            out.push(c);
                        }
                        out
                    });
                    if index == 0 {
                        value = value.trim_start_matches(' ').into();
                    }
                    if index + 1 == children.len() {
                        value.truncate(value.trim_end_matches(' ').len());
                    }
                    Some(Expr::Lit(value.into()))
                }
                Node::Expression(value, at) => {
                    bound_text = true;
                    Some(expression(value, *at)?)
                }
                _ => None,
            };
            if let Some(value) = value {
                has_text = true;
                text = Expr::Bin(crate::bind::Op::Add, Box::new(text), Box::new(value));
            } else {
                let child = self.node(child, i)?;
                self.doc.nodes[i].children.push(child);
            }
        }
        if has_text {
            let value = if bound_text {
                PropValue::Bind(text, None)
            } else {
                PropValue::Lit(Lit::Str(crate::bind::text(&text.eval(
                    &crate::bind::Scope {
                        vm: &serde_json::Value::Null,
                        locals: Vec::new(),
                        env: &crate::bind::NoEnv,
                    },
                ))))
            };
            let source = children
                .iter()
                .filter_map(|child| match child {
                    Node::Text(text, _) => Some(text.clone()),
                    Node::Expression(expr, _) => Some(format!("{{{expr}}}")),
                    _ => None,
                })
                .collect::<String>();
            self.prop(i, internal, "text", value, at, &source)?;
        }
        Ok(i)
    }
    /// Whether the element nearest above `parent` (through blocks) is a list.
    /// The selector tag of the element enclosing `parent` (itself, when it
    /// is one), past blocks and other non-elements.
    fn parent_tag(&self, mut parent: usize) -> Option<&str> {
        while parent != NONE {
            let n = &self.doc.nodes[parent];
            match n.kind {
                NodeKind::Element(_) | NodeKind::Use(_) => return n.selector_tag.as_deref(),
                _ => parent = n.parent,
            }
        }
        None
    }

    fn in_list(&self, mut parent: usize) -> bool {
        while parent != NONE {
            let n = &self.doc.nodes[parent];
            match n.kind {
                NodeKind::Element(_) | NodeKind::Use(_) => {
                    return matches!(n.selector_tag.as_deref(), Some("ul" | "ol"));
                }
                _ => parent = n.parent,
            }
        }
        // A component whose root is an item is placed in its user's list.
        true
    }

    /// An anonymous text element of `run`'s inline content under block
    /// `parent`; nothing for whitespace alone.
    fn anonymous_text(
        &mut self,
        run: &mut Vec<Node>,
        parent: usize,
        translated: bool,
    ) -> Result<Option<usize>, Error> {
        let nodes = std::mem::take(run);
        if nodes
            .iter()
            .all(|n| matches!(n, Node::Text(t, _) if t.trim().is_empty()))
        {
            return Ok(None);
        }
        let at = match &nodes[0] {
            Node::Text(_, at) | Node::Expression(_, at) | Node::Element { at, .. } => *at,
            _ => 0,
        };
        let i = self.push(NodeKind::Element("label"), parent, at);
        self.text_runs(i, &nodes, None, at, translated)?;
        self.doc.nodes[parent].children.push(i);
        Ok(Some(i))
    }

    /// Text element `i`'s content as inline runs: its `text` (the runs'
    /// plain text), and the runs themselves when any is styled or a link.
    /// An SVG `<text>` whose `<tspan x>`s start lines: a group carrying
    /// the text's attributes (so they inherit), holding one `<text>` per
    /// line at its own point. A line's `y` is its tspan's, else the line
    /// before's, moved by every `dy` since (SVG's current text position).
    /// `None`: no tspan starts a line.
    fn svg_lines(
        &mut self,
        attrs: &[Attr],
        white_space: Option<(bool, bool)>,
        children: &[Node],
        at: usize,
        parent: usize,
    ) -> Result<Option<usize>, Error> {
        const PLACE: [&str; 4] = ["x", "y", "dx", "dy"];
        let line = |n: &Node| match n {
            Node::Element { tag, attrs, .. } if tag == "tspan" => {
                attrs.iter().any(|a| PLACE.contains(&a.name.as_str()))
            }
            _ => false,
        };
        if !children.iter().any(line) {
            return Ok(None);
        }
        let attr = |name: &str, value: String, at: usize| Attr {
            name: name.into(),
            value,
            bound: false,
            at,
            end: at,
        };
        let own = |name: &str| attrs.iter().find(|a| a.name == name).cloned();
        if let Some(a) = own("dy").filter(|a| a.bound) {
            return Err((a.at, "a <text> of several lines takes a literal dy".into()));
        }
        let mut ws = Vec::new();
        if let Some((spaces, breaks)) = white_space {
            let v = match (spaces, breaks) {
                (true, true) => "pre",
                (false, true) => "pre-line",
                _ => "preserve",
            };
            ws.push(attr("white-space", v.into(), at));
        }
        // The first line: the text's own place and what comes before the
        // first tspan that starts a line.
        let mut lines: Vec<(Vec<Attr>, Vec<Node>)> =
            vec![(PLACE.iter().filter_map(|p| own(p)).collect(), Vec::new())];
        let mut y = own("y");
        let mut dys: Vec<String> = own("dy").map(|a| a.value).into_iter().collect();
        for child in children {
            let Node::Element {
                tag,
                attrs: t,
                children: inner,
                at: tat,
                end,
                close,
            } = child
            else {
                lines.last_mut().expect("a line").1.push(child.clone());
                continue;
            };
            if !line(child) {
                lines.last_mut().expect("a line").1.push(child.clone());
                continue;
            }
            let get = |n: &str| t.iter().find(|a| a.name == n).cloned();
            let Some(x) = get("x") else {
                return Err((
                    *tat,
                    "a <tspan> that moves starts a line: give it x (and y or dy); a run within a line takes no y, dx or dy".into(),
                ));
            };
            if let Some(a) = get("dy").filter(|a| a.bound) {
                return Err((a.at, "a line's dy is literal".into()));
            }
            if let Some(ty) = get("y") {
                y = Some(ty);
                dys.clear();
            }
            dys.extend(get("dy").map(|a| a.value));
            let mut place = vec![x];
            place.extend(y.clone());
            place.extend(get("dx"));
            if !dys.is_empty() {
                place.push(attr("dy", dys.join(" "), *tat));
            }
            let run = Node::Element {
                tag: tag.clone(),
                attrs: t
                    .iter()
                    .filter(|a| !PLACE.contains(&a.name.as_str()))
                    .cloned()
                    .collect(),
                children: inner.clone(),
                at: *tat,
                end: *end,
                close: *close,
            };
            lines.push((place, vec![run]));
        }
        let group: Vec<Attr> = attrs
            .iter()
            .filter(|a| !PLACE.contains(&a.name.as_str()))
            .cloned()
            .collect();
        let body: Vec<Node> = lines
            .into_iter()
            .filter(|(_, content)| {
                content
                    .iter()
                    .any(|n| !matches!(n, Node::Text(t, _) if t.trim().is_empty()))
            })
            .map(|(mut place, content)| {
                place.extend(ws.iter().cloned());
                Node::Element {
                    tag: "text".into(),
                    attrs: place,
                    children: content,
                    at,
                    end: at,
                    close: at,
                }
            })
            .collect();
        let g = self.element("g", &group, &body, at, parent)?;
        // Selected and named as the <text> it was written as.
        self.doc.nodes[g].selector_tag = Some("text".into());
        if let Some(entry) = self.selectors.iter_mut().find(|(n, ..)| *n == g) {
            entry.1 = "text".into();
        }
        Ok(Some(g))
    }

    /// A `<text>` holding a `<textPath>`: its attributes as the text's
    /// props, and its content as the text's. Other content: unchanged.
    fn text_path(&mut self, i: usize, children: &[Node]) -> Result<Vec<Node>, Error> {
        let Some(Node::Element {
            attrs,
            children: inner,
            at,
            ..
        }) = children
            .iter()
            .find(|n| matches!(n, Node::Element { tag, .. } if tag == "textPath"))
        else {
            return Ok(children.to_vec());
        };
        if children.iter().any(|n| match n {
            Node::Element { tag, .. } => tag != "textPath",
            Node::Text(t, _) => !t.trim().is_empty(),
            _ => true,
        }) {
            return Err((
                *at,
                "a <textPath> is the whole content of its <text>".into(),
            ));
        }
        if let Some(n) = inner
            .iter()
            .find(|n| !matches!(n, Node::Text(..) | Node::Expression(..)))
        {
            let at = match n {
                Node::Element { at, .. } => *at,
                _ => *at,
            };
            return Err((
                at,
                "a <textPath> holds words and {expressions}: one run along its path".into(),
            ));
        }
        for a in attrs {
            let prop = match a.name.as_str() {
                "href" | "xlink:href" => "svgTextPathHref",
                "startOffset" => "svgStartOffset",
                "side" => "svgSide",
                "method" => "svgMethod",
                "spacing" => "svgSpacing",
                "path" => "svgPath",
                other => {
                    return Err((
                        a.at,
                        format!(
                            "<textPath> takes href, path, startOffset, side, method and spacing, not {other}; presentation attributes go on its <text>"
                        ),
                    ));
                }
            };
            if a.bound && matches!(prop, "svgTextPathHref" | "svgPath") {
                return Err((a.at, format!("{} names its path literally", a.name)));
            }
            let value = if a.bound {
                PropValue::Bind(expression(&a.value, a.at)?, None)
            } else {
                PropValue::Lit(Lit::Str(a.value.clone()))
            };
            self.prop(i, "svgText", prop, value, a.at, &a.value)?;
        }
        if !attrs
            .iter()
            .any(|a| matches!(a.name.as_str(), "href" | "xlink:href" | "path"))
        {
            return Err((
                *at,
                "a <textPath> names its path: href=\"#id\" or path=\"M…\"".into(),
            ));
        }
        Ok(inner.clone())
    }

    fn text_runs(
        &mut self,
        i: usize,
        nodes: &[Node],
        link: Option<usize>,
        at: usize,
        translated: bool,
    ) -> Result<(), Error> {
        let mut runs = Vec::new();
        let mut links = Vec::new();
        let base = InlineRun {
            text: Expr::Lit("".into()),
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color: None,
            link,
            part: None,
            atom: None,
            valign: None,
        };
        self.runs(nodes, &base, &mut runs, &mut links, translated)?;
        // No space at either end, as a CSS block trims its line (an SVG
        // text keeping its white space trims its line breaks there too).
        let ends: &[char] = if self.white_space.is_some() {
            &[' ', '\n']
        } else {
            &[' ']
        };
        if let Some(InlineRun {
            text: Expr::Lit(serde_json::Value::String(t)),
            ..
        }) = runs.first_mut()
        {
            *t = t.trim_start_matches(ends).into();
        }
        if let Some(InlineRun {
            text: Expr::Lit(serde_json::Value::String(t)),
            ..
        }) = runs.last_mut()
        {
            t.truncate(t.trim_end_matches(ends).len());
        }
        runs.retain(|r| r.text != Expr::Lit("".into()));
        let bound = runs.iter().any(|r| !matches!(r.text, Expr::Lit(_)));
        let plain = runs.iter().fold(Expr::Lit("".into()), |acc, r| {
            Expr::Bin(
                crate::bind::Op::Add,
                Box::new(acc),
                Box::new(r.text.clone()),
            )
        });
        let value = if bound {
            PropValue::Bind(plain, None)
        } else {
            let text: String = runs
                .iter()
                .map(|r| match &r.text {
                    Expr::Lit(v) => crate::bind::text(v),
                    _ => String::new(),
                })
                .collect();
            PropValue::Lit(Lit::Str(text))
        };
        self.prop(i, "label", "text", value, at, "")?;
        if runs.iter().any(|r| {
            r.bold
                || r.italic
                || r.underline
                || r.strike
                || r.color.is_some()
                || r.link.is_some()
                || r.part.is_some()
                || r.atom.is_some()
                || r.valign.is_some()
        }) {
            self.doc.nodes[i].inline = runs;
            self.doc.nodes[i].links = links;
        }
        Ok(())
    }

    fn runs(
        &mut self,
        nodes: &[Node],
        style: &InlineRun,
        out: &mut Vec<InlineRun>,
        links: &mut Vec<String>,
        translated: bool,
    ) -> Result<(), Error> {
        let run = |text: Expr| InlineRun {
            text,
            ..style.clone()
        };
        for node in nodes {
            match node {
                Node::Text(value, at) => {
                    if !translated && !value.trim().is_empty() {
                        self.warn(*at);
                    }
                    let (spaces, breaks) = self.white_space.unwrap_or((false, false));
                    let mut collapsed = String::new();
                    for c in value.chars() {
                        if c == '\n' && breaks {
                            if !spaces {
                                // pre-line: no spaces around a line break.
                                collapsed.truncate(collapsed.trim_end_matches(' ').len());
                            }
                            collapsed.push('\n');
                        } else if c.is_ascii_whitespace() {
                            if spaces || !(collapsed.ends_with(' ') || collapsed.ends_with('\n')) {
                                collapsed.push(' ');
                            }
                        } else {
                            collapsed.push(c);
                        }
                    }
                    if !collapsed.is_empty() {
                        out.push(run(Expr::Lit(collapsed.into())));
                    }
                }
                Node::Expression(value, at) => out.push(run(expression(value, *at)?)),
                Node::Element {
                    tag,
                    attrs,
                    children,
                    at,
                    ..
                } if inline(node) => {
                    let mut inner = style.clone();
                    match tag.as_str() {
                        "b" | "strong" => inner.bold = true,
                        "i" | "em" => inner.italic = true,
                        "u" => inner.underline = true,
                        "s" | "del" => inner.strike = true,
                        "sub" => inner.valign = Some(crate::VerticalAlign::Sub),
                        "sup" => inner.valign = Some(crate::VerticalAlign::Super),
                        "span" => {
                            if let Some(a) = attrs.iter().find(|a| a.name == "part") {
                                if !super::syntax::identifier(&a.value) {
                                    return Err((a.at, "part names a span: an identifier".into()));
                                }
                                inner.part = Some(a.value.clone());
                            }
                        }
                        "br" => {
                            if !children.is_empty() {
                                return Err((*at, "<br> is empty".into()));
                            }
                            out.push(run(Expr::Lit("\n".into())));
                            continue;
                        }
                        "tspan" => {
                            for a in attrs {
                                super::extern_element::tspan(&mut inner, &a.name, &a.value)
                                    .map_err(|e| (a.at, e))?;
                            }
                        }
                        "a" => {
                            if style.link.is_some() {
                                return Err((*at, "a link cannot hold another link".into()));
                            }
                            let Some(href) = attrs.iter().find(|a| a.name == "href") else {
                                return Err((*at, "an inline <a> needs an href".into()));
                            };
                            links.push(href.value.clone());
                            inner.link = Some(links.len() - 1);
                        }
                        _ => {}
                    }
                    self.runs(children, &inner, out, links, translated)?;
                }
                // An inline object among the words: a picture, an icon, an
                // action's glyph, one placeholder character in the text.
                Node::Element {
                    tag,
                    attrs,
                    children,
                    at,
                    ..
                } if matches!(tag.as_str(), "img" | "icon" | "kbd")
                    && children.is_empty()
                    && !translated =>
                {
                    let literal = |names: &[&str]| {
                        attrs
                            .iter()
                            .find(|a| names.contains(&a.name.as_str()))
                            .filter(|a| !a.bound && !a.value.is_empty())
                            .map(|a| a.value.clone())
                    };
                    let atom = match tag.as_str() {
                        "img" => literal(&["src"]).map(crate::rich::InlineAtom::Image),
                        "icon" => literal(&["name", "icon"]).map(crate::rich::InlineAtom::Icon),
                        _ => literal(&["data-action"]).map(crate::rich::InlineAtom::Action),
                    }
                    .ok_or((
                        *at,
                        format!(
                            "an inline <{tag}> names its {} literally",
                            match tag.as_str() {
                                "img" => "picture with src",
                                "icon" => "icon with name",
                                _ => "action with data-action",
                            }
                        ),
                    ))?;
                    if let Some(a) = attrs.iter().find(|a| {
                        !matches!(
                            a.name.as_str(),
                            "src" | "alt" | "name" | "icon" | "data-action" | "class"
                        )
                    }) {
                        return Err((a.at, format!("an inline <{tag}> takes no {}", a.name)));
                    }
                    out.push(InlineRun {
                        text: Expr::Lit("\u{FFFC}".into()),
                        atom: Some(atom),
                        ..style.clone()
                    });
                }
                Node::Element { tag, at, .. } if tag == "tspan" => {
                    return Err((
                        *at,
                        "a <tspan> takes literal font-weight, font-style, text-decoration and fill, and starts a line with x (and y, dx, dy); its other attributes are not read".into(),
                    ));
                }
                Node::Element { tag, at, .. } if tag == "textPath" => {
                    return Err((
                        *at,
                        "a <textPath> is the whole content of its <text>".into(),
                    ));
                }
                Node::Element { tag, at, .. } if crate::svg::smil::TAGS.contains(&tag.as_str()) => {
                    return Err((
                        *at,
                        format!(
                            "<{tag}> animates a <text> from a <g> around it: the group's transform and presentation attributes reach its text"
                        ),
                    ));
                }
                Node::Element { tag, at, .. } => {
                    return Err((
                        *at,
                        format!(
                            "<{tag}> cannot sit inside text: text holds words, {{expressions}} and the inline elements b, strong, i, em, u, s, del, br, a plain <span> and <a href>; put <{tag}> beside the text in a <div>"
                        ),
                    ));
                }
                Node::If { at, .. }
                | Node::Each { at, .. }
                | Node::Snippet { at, .. }
                | Node::Render { at, .. }
                | Node::Key { at, .. }
                | Node::Const { at, .. }
                | Node::Await { at, .. }
                | Node::Script(_, at) => {
                    return Err((
                        *at,
                        "a block cannot sit inside text; put it beside the text in a <div>, or use {cond ? a : b}".into(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Joins each `aria-labelledby` and `aria-describedby` to the text of
    /// the elements its ids name (an element's own text, else its first
    /// text inside): one id's text as written, bound or not; several ids'
    /// literal texts joined by spaces, as ARIA computes a name.
    fn join_aria(&mut self) -> Result<(), Error> {
        for (node, prop, ids, at) in std::mem::take(&mut self.aria_refs) {
            let mut texts = Vec::new();
            for id in ids.split_ascii_whitespace() {
                let Some(target) = self
                    .doc
                    .nodes
                    .iter()
                    .position(|n| n.id == id && matches!(n.kind, NodeKind::Element(_)))
                else {
                    return Err((
                        at,
                        format!("\"{id}\" names no element; give it id=\"{id}\""),
                    ));
                };
                let mut stack = vec![target];
                let mut found = None;
                while let Some(k) = stack.pop() {
                    if let Some(t) = self.doc.nodes[k].props.iter().find(|p| p.name() == "text") {
                        found = Some(t.clone());
                        break;
                    }
                    stack.extend(self.doc.nodes[k].children.iter().rev());
                }
                texts.push(found.ok_or((at, format!("#{id} has no text to read")))?);
            }
            let mut set = if let [one] = &texts[..] {
                one.clone()
            } else {
                let joined = texts
                    .iter()
                    .map(|t| match &t.value {
                        PropValue::Lit(Lit::Str(s)) => Ok(s.clone()),
                        _ => Err((
                            at,
                            "several ids join literal texts; name one element for a bound text"
                                .to_string(),
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .join(" ");
                PropSet {
                    value: PropValue::Lit(Lit::Str(joined)),
                    ..texts[0].clone()
                }
            };
            set.spec = schema::prop(prop).expect("an ARIA prop").0;
            self.doc.nodes[node].props.retain(|p| p.name() != prop);
            self.doc.nodes[node].props.push(set);
        }
        Ok(())
    }

    /// Joins each input's `list` to the `<datalist>` its id names: its
    /// options are a range's marks, or a text input's suggestions.
    fn join_lists(&mut self) -> Result<(), Error> {
        for (range, id, at) in std::mem::take(&mut self.list_fors) {
            let Some((_, marks, items)) = self.datalists.iter().find(|(d, _, _)| *d == id).cloned()
            else {
                return Err((
                    at,
                    format!("list=\"{id}\" names no <datalist>; write <datalist id=\"{id}\">"),
                ));
            };
            let strs = |v: Vec<String>| PropValue::Lit(Lit::Strs(v));
            if matches!(self.doc.nodes[range].kind, NodeKind::Element("field")) {
                // A text input's suggestions: the options' values, or the
                // datalist's bound items.
                let value = match items {
                    Some(e) => PropValue::Bind(e, None),
                    None => strs(marks.iter().map(|m| m.0.clone()).collect()),
                };
                self.prop(range, "field", "suggestions", value, at, "")?;
                continue;
            }
            if items.is_some() {
                return Err((at, "a range's <datalist> lists literal <option>s".into()));
            }
            if let Some(m) = marks.iter().find(|m| m.0.parse::<f32>().is_err()) {
                return Err((
                    at,
                    format!("a range's <datalist> lists numbers; '{}' is not one", m.0),
                ));
            }
            let values = strs(marks.iter().map(|m| m.0.clone()).collect());
            let labels = strs(marks.iter().map(|m| m.1.clone()).collect());
            self.prop(range, "slider", "markValues", values, at, "")?;
            self.prop(range, "slider", "markLabels", labels, at, "")?;
        }
        Ok(())
    }

    /// Joins each `<label for>` to the control its id names: a checkbox
    /// takes the label's text as its caption and the label goes, as the
    /// wrapping form does; any other control keeps the label beside it
    /// and takes its text as its name.
    fn join_labels(&mut self) -> Result<(), Error> {
        for (label, id, at) in std::mem::take(&mut self.label_fors) {
            let Some(target) = self
                .doc
                .nodes
                .iter()
                .position(|n| n.id == id && matches!(n.kind, NodeKind::Element(_)))
            else {
                return Err((
                    at,
                    format!("for=\"{id}\" names no element; give the control id=\"{id}\""),
                ));
            };
            let text = self.doc.nodes[label]
                .props
                .iter()
                .find(|p| p.name() == "text")
                .cloned();
            let NodeKind::Element(kind) = self.doc.nodes[target].kind else {
                unreachable!("found an element");
            };
            match kind {
                "toggle" => {
                    if self.doc.nodes[target]
                        .props
                        .iter()
                        .any(|p| p.name() == "text")
                    {
                        return Err((at, format!("#{id} has a caption already; write it once")));
                    }
                    self.doc.nodes[target].props.extend(text);
                    // Source order: a label written after its checkbox
                    // puts the caption after the switch.
                    if target < label {
                        let line = self.line(at);
                        self.doc.nodes[target].props.push(PropSet {
                            spec: schema::prop("side").expect("side is a prop").0,
                            value: PropValue::Lit(Lit::Str("end".into())),
                            line,
                            source: String::new(),
                        });
                    }
                    let parent = self.doc.nodes[label].parent;
                    if parent != NONE {
                        self.doc.nodes[parent].children.retain(|&c| c != label);
                    }
                    self.selectors.retain(|(n, ..)| *n != label);
                    // Left in no tree: an anonymous text so nothing reads it.
                    self.doc.nodes[label].kind = NodeKind::If(Expr::Lit(false.into()));
                    self.doc.nodes[label].props.clear();
                    self.doc.nodes[label].children.clear();
                }
                "field" | "select" | "slider" | "meter" | "progress" | "tabs" => {
                    if let Some(mut text) = text
                        && !self.doc.nodes[target]
                            .props
                            .iter()
                            .any(|p| p.name() == "name")
                    {
                        text.spec = schema::prop("name").expect("name is a prop").0;
                        self.doc.nodes[target].props.push(text);
                    }
                }
                other => {
                    return Err((
                        at,
                        format!(
                            "for=\"{id}\" names a {other}; a label names a form control (an <input>, <select>, <textarea>, <meter> or <progress>)"
                        ),
                    ));
                }
            }
        }
        Ok(())
    }
    fn prop(
        &mut self,
        node: usize,
        kind: &str,
        name: &str,
        value: PropValue,
        at: usize,
        source: &str,
    ) -> Result<(), Error> {
        let (spec, p) = schema::prop(name).ok_or((
            at,
            format!("unknown property {name}; use a reflected property"),
        ))?;
        if !p.applies(kind) {
            return Err((
                at,
                format!("{name} does not apply to {kind}; use its documented properties"),
            ));
        }
        if matches!(value, PropValue::Model(_)) && !p.two_way {
            return Err((at, format!("{name} is read-only; remove bind:")));
        }
        // A modal's content is built in the top layer, a tree of its own
        // its descendants are named by; HTML chooses it per opening too
        // (showModal or show).
        if name == "modal" && !matches!(value, PropValue::Lit(_)) {
            return Err((
                at,
                "write modal as a literal, not bound: use two dialogs, or {#if} around one".into(),
            ));
        }
        let line = self.line(at);
        self.doc.nodes[node].props.push(PropSet {
            spec,
            value,
            line,
            source: source.into(),
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_selector_inputs_without_any_matching_stylesheet() {
        let doc = UiDocument::parse(
            "facts.svelte",
            r#"
<meta name="component" content="facts">
<column class="root fixed">
  {#if $shown}<text id="Label" class={$classes} class:active={$selected}>Hello</text>{/if}
  <input type="range" id="Amount" step="0.25" bind:value={$amount}/>
  <button onclick={choose} class:quiet="false">Choose</button>
</column>"#,
        )
        .unwrap();
        assert_eq!(
            doc.nodes[doc.root].selector_attrs,
            vec![("class".into(), Expr::Lit("root fixed".into()))]
        );
        let label = doc.nodes.iter().find(|node| node.id == "Label").unwrap();
        assert_eq!(label.selector_tag.as_deref(), Some("text"));
        assert_eq!(
            label.selector_attrs,
            vec![
                ("id".into(), Expr::Lit("Label".into())),
                ("class".into(), Expr::Path("$classes".into())),
                ("class:active".into(), Expr::Path("$selected".into())),
            ]
        );
        let mut paths = Vec::new();
        doc.node_paths(label, &mut paths);
        assert!(paths.contains(&"$classes".into()) && paths.contains(&"$selected".into()));
        let amount = doc.nodes.iter().find(|node| node.id == "Amount").unwrap();
        assert!(
            amount
                .selector_attrs
                .contains(&("step".into(), Expr::Lit("0.25".into())))
        );
        assert!(
            amount
                .selector_attrs
                .contains(&("bind:value".into(), Expr::Path("$amount".into())))
        );
        let button = doc
            .nodes
            .iter()
            .find(|node| node.selector_tag.as_deref() == Some("button"))
            .unwrap();
        assert_eq!(
            button.selector_attrs,
            vec![("class:quiet".into(), Expr::Lit(false.into()))]
        );
        let conditional = doc
            .nodes
            .iter()
            .find(|node| matches!(node.kind, NodeKind::If(_)))
            .unwrap();
        assert!(conditional.selector_tag.is_none() && conditional.selector_attrs.is_empty());
    }

    #[test]
    fn interpolation_keeps_word_boundaries_and_literal_rune_names() {
        let doc = UiDocument::parse(
            "words.svelte",
            r#"
<meta name="screen" content="words">
<text> {'Hello'}   {'world'} &amp; {'$state'} </text>
"#,
        )
        .unwrap();
        let prop = doc.nodes[doc.root]
            .props
            .iter()
            .find(|p| p.name() == "text")
            .unwrap();
        let PropValue::Bind(expr, _) = &prop.value else {
            panic!("interpolated text")
        };
        let scope = crate::bind::Scope {
            vm: &serde_json::Value::Null,
            locals: Vec::new(),
            env: &crate::bind::NoEnv,
        };
        assert_eq!(expr.eval(&scope), serde_json::json!("Hello world & $state"));
        assert!(
            UiDocument::parse(
                "source.svelte",
                "<meta name='screen' content='x'><text>{$stateful}</text>"
            )
            .is_ok()
        );
    }

    #[test]
    fn parses_keyed_branches_models_snippets_and_scoped_css() {
        let doc = UiDocument::parse(
            "test.svelte",
            r#"
<meta name="screen" content="test">
<meta name="prop:items" content="list">
<meta name="prop:amount" content="number">
<meta name="prop:active" content="bool">
<meta name="action:confirm" content="Choose">
{#snippet details()}<text id="Details">{t('details')}</text>{/snippet}
<column id="Root" onconfirm={choose}>
  {#each $items as item (item.id)}
    <button id="Choice" class:active={$active} onclick={choose}>{item.name}</button>
  {:else}<text id="Empty" i18n="empty">Empty</text>{/each}
  {#if $active}<input type="range" id="Value" step="0.1" bind:value={$amount}/>
  {:else if $amount > 4}<text id="High">{t('high')}</text>
  {:else}<text id="Low">{t('low')}</text>{/if}
  {@render details()}
</column>
<style>
button { opacity: 0.2; }
#Choice { opacity: 0.4; }
button.active { opacity: 1; }
@media (width < 600px), (height < 600px) { #Root { padding: 12px; } }
</style>"#,
        )
        .unwrap();
        assert!(doc.warnings.is_empty());
        let root = doc.nodes.iter().find(|n| n.id == "Root").unwrap();
        assert!(matches!(&root.triggers[0].when, When::Action(name) if name == "confirm"));
        assert!(matches!(&root.triggers[0].act, Act::Emit(name) if name == "choose"));
        assert!(doc.nodes.iter().any(|n| matches!(&n.kind, NodeKind::For { key: Expr::Path(k), empty, .. } if k == "item.id" && !empty.is_empty())));
        let value = doc.nodes.iter().find(|n| n.id == "Value").unwrap();
        assert!(
            value
                .props
                .iter()
                .any(|p| p.value == PropValue::Model("$amount".into()))
        );
        assert!(value.props.iter().any(|p| p.name() == "stepBy"));
        let choice = doc.nodes.iter().find(|n| n.id == "Choice").unwrap();
        assert_eq!(choice.rules.len(), 3);
        assert_eq!(
            choice.rules.last().unwrap().props[0].value,
            PropValue::Lit(Lit::Num(0.4)),
            "ID specificity beats a later class rule"
        );
        assert!(doc.nodes.iter().any(|n| matches!(
            n.kind,
            NodeKind::Render {
                optional: false,
                ..
            }
        )));
    }

    #[test]
    fn html_tables_forms_fieldsets_and_figures_take_their_semantics() {
        let doc = UiDocument::parse(
            "html.svelte",
            r#"
<meta name="component" content="html">
<div>
  <table id="T">
    <caption>Scores</caption>
    <thead><tr><th id="H">Name</th><th>Score</th></tr></thead>
    <tbody>
      {#each $rows as row (row.id)}
      <tr><th id="R">{row.name}</th><td colspan="2" rowspan="0">{row.score}</td></tr>
      {/each}
    </tbody>
  </table>
  <form onsubmit={save} onreset={clear}>
    <fieldset disabled={$busy}>
      <legend>Profile <input type="checkbox" id="Inside" bind:checked={$a}></legend>
      <input id="Name" bind:value={$name} required>
      <input type="checkbox" id="Off" disabled={$locked} bind:checked={$b}>
    </fieldset>
    <button id="Go">Save</button>
    <button id="Undo" type="reset">Undo</button>
    <button id="Other" type="button">Other</button>
  </form>
  <figure id="F"><img src={$pic} alt="A"><figcaption>A map</figcaption></figure>
  <output id="O" for="Name">{$name}</output>
</div>
"#,
        )
        .unwrap();
        let node = |id: &str| doc.nodes.iter().find(|n| n.id == id).unwrap();
        let text = |n: &DocNode, prop: &str| {
            n.props
                .iter()
                .rev()
                .find(|p| p.name() == prop)
                .map(|p| p.value.clone())
        };
        assert_eq!(node("T").role, Some("table"));
        assert_eq!(
            text(node("T"), "name"),
            Some(PropValue::Lit(Lit::Str("Scores".into())))
        );
        assert_eq!(node("H").role, Some("columnheader"));
        // A header beside data cells heads its row.
        assert_eq!(node("R").role, Some("rowheader"));
        let cell = doc
            .nodes
            .iter()
            .find(|n| n.selector_tag.as_deref() == Some("td"))
            .unwrap();
        assert_eq!(
            (cell.role, text(cell, "colspan")),
            (Some("cell"), Some(PropValue::Lit(Lit::Num(2.0))))
        );
        assert_eq!(text(cell, "rowspan"), Some(PropValue::Lit(Lit::Num(0.0))));
        // A disabled fieldset's controls, but those in its legend.
        assert_eq!(text(node("Inside"), "disabled"), None);
        assert_eq!(
            text(node("Name"), "disabled"),
            Some(PropValue::Bind(Expr::Path("$busy".into()), None))
        );
        assert_eq!(
            text(node("Off"), "disabled"),
            Some(PropValue::Bind(
                Expr::Bin(
                    crate::bind::Op::Or,
                    Box::new(Expr::Path("$locked".into())),
                    Box::new(Expr::Path("$busy".into()))
                ),
                None
            ))
        );
        let emits = |id: &str| -> Vec<String> {
            node(id)
                .triggers
                .iter()
                .filter_map(|t| match (&t.when, &t.act) {
                    (When::Click, Act::Emit(a) | Act::Submit { action: a, .. }) => Some(a.clone()),
                    _ => None,
                })
                .collect()
        };
        assert_eq!(emits("Go"), ["save"]);
        assert_eq!(emits("Undo"), ["clear"]);
        assert!(emits("Other").is_empty());
        assert_eq!(
            (node("F").role, text(node("F"), "name")),
            (
                Some("figure"),
                Some(PropValue::Lit(Lit::Str("A map".into())))
            )
        );
        assert_eq!(node("O").role, Some("status"));
        for (body, reason) in [
            ("<div><tr><td/></tr></div>", "belongs inside a <table>"),
            ("<table><tr><td colspan='0'/></tr></table>", "colspan takes"),
            ("<table><tr><th scope='all'/></tr></table>", "scope is"),
            (
                "<table><tr><td/></tr><caption>x</caption></table>",
                "first child",
            ),
            ("<div><template><p/></template></div>", "{#snippet"),
            (
                "<div><button type='submit'>Go</button></div>",
                "inside a <form>",
            ),
            ("<div><output for='Nope'>x</output></div>", "no element"),
        ] {
            let text = format!("<meta name='screen' content='t'>\n{body}");
            let errors = UiDocument::parse("bad.svelte", &text).unwrap_err();
            assert!(errors[0].contains(reason), "{body}: {}", errors[0]);
        }
    }

    #[test]
    fn rejects_wrong_syntax_and_reports_authored_locations() {
        for (body, reason) in [
            ("<column><mystery/></column>", "unknown element"),
            (
                "<column><text id='A'/><text id='A'/></column>",
                "duplicate id",
            ),
            (
                "<column>{#each $items as item}<text/>{/each}</column>",
                "stable key",
            ),
            (
                "<column>{#each $items as item ($index)}<text/>{/each}</column>",
                "stable string",
            ),
            ("<text>{$amount = 2}</text>", "side-effect-free"),
            ("<text>{$state(1)}</text>", "game's mounted stores"),
            ("<button on:click={next}/>", "onclick"),
            ("<slot/>", "{@render"),
            ("<script>let count = $state(0)</script><text/>", "TS system"),
            (
                "<script>$effect(() => log())</script><text/>",
                "$effect is not allowed",
            ),
            ("<script>import X from './x'</script><text/>", "imports"),
            (
                "<script>function go() {}</script><text/>",
                "functions are not allowed",
            ),
            (
                "<script>const a = 1</script><text/>",
                "variable, which is not allowed",
            ),
            (
                "<script>let { a = b() } = $props()</script><text/>",
                "not a literal",
            ),
            ("<text>{@html $raw}</text>", "TS system"),
            ("<column>{@const a = 1}</column>", "directly inside a block"),
            (
                "<column>{#await $a + 1}{/await}</column>",
                "a promise never runs",
            ),
            ("<column>{#key $a}<text/></column>", "{/key}"),
            (
                "<svg><path d='M0 0' in:draw={{ speed: 1 }}/></svg>",
                "give duration",
            ),
            ("<div in:draw></div>", "SVG shape"),
            ("<div animate:flip></div>", "keyed {#each}"),
            ("<div transition:spin></div>", "not a built-in transition"),
            ("<div in:fade out:fade transition:fade></div>", "set twice"),
            (
                "<div transition:fly={{ easing: wobble }}></div>",
                "svelte/easing",
            ),
            ("<div style={$css}></div>", "literal CSS"),
            (
                "<div style='colour: red'></div>",
                "unsupported CSS property",
            ),
            ("<svelte:boundary><text/></svelte:boundary>", "never throws"),
            ("<svelte:window onkeydown={go}/><text/>", "input actions"),
            (
                r#"<input type="range" bind:value={$amount + 1}/>"#,
                "writable store",
            ),
            ("<text>{ { value: 1 } }</text>", "side-effect-free"),
            (
                "<text id='T'/><style>#T { animation: missing 1s linear forwards; }</style>",
                "declare @keyframes",
            ),
            (
                "<text id='T'/><style>#T { animation: fade 1s linear forwards; } @keyframes fade { 150% { opacity: 0; } }</style>",
                "percentage from 0% to 100%",
            ),
            (
                "<text id='T'/><style>#T { animation: fade 1s cubic-bezier(2,0,1,1) forwards; } @keyframes fade { from { opacity: 0; } to { opacity: 1; } }</style>",
                "CSS easing",
            ),
            (
                "<text id='T'/><style>#T:hover { animation: fade 1s; } @keyframes fade { from { transition-duration: 1s; } }</style>",
                "transition-duration cannot be animated in @keyframes",
            ),
        ] {
            let text = format!("<meta name='screen' content='test'>\n{body}");
            let errors = UiDocument::parse("bad.svelte", &text).unwrap_err();
            assert!(errors[0].starts_with("bad.svelte:2:"), "{}", errors[0]);
            assert!(errors[0].contains(reason), "{}", errors[0]);
        }
        let doc = UiDocument::parse(
            "text.svelte",
            "<meta name='screen' content='x'><text>Untranslated</text>",
        )
        .unwrap();
        assert_eq!(doc.warnings.len(), 1);
    }
}
