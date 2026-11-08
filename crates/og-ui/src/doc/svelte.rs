use super::syntax::{Attr, Error, Node, identifier};
use super::*;
use std::collections::BTreeSet;

pub(super) fn parse(source: &str, text: &str) -> Result<UiDocument, Vec<String>> {
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
                nodes: Vec::new(),
                root: NONE,
                warnings: Vec::new(),
            },
            ids: BTreeSet::new(),
            selectors: Vec::new(),
            styles: Vec::new(),
            label_fors: Vec::new(),
        };
        for node in &nodes {
            match node {
                Node::Text(text, _) if text.trim().is_empty() => {}
                Node::Element { tag, attrs, at, .. } if tag == "meta" => p.meta(attrs, *at)?,
                Node::Element { tag, children, .. } if tag == "style" => {
                    if let [Node::Text(css, at)] = children.as_slice() {
                        p.styles.push((css.clone(), *at));
                    }
                }
                Node::Snippet { .. } => {
                    p.node(node, NONE)?;
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
        p.join_labels()?;
        super::stylesheet::apply(&mut p.doc, &p.selectors, &p.styles)?;
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

/// Inline content a text element reads as runs: words, expressions, the
/// formatting elements, a plain `<span>` and a literal `<a href>`.
fn inline(node: &Node) -> bool {
    match node {
        Node::Text(..) | Node::Expression(..) => true,
        Node::Element { tag, attrs, .. } => match tag.as_str() {
            "b" | "strong" | "i" | "em" | "u" | "s" | "del" | "br" | "span" => attrs.is_empty(),
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
                | Node::Render { at, .. } => {
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
}
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
            if self.doc.props.iter().any(|p| p.name == name) {
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
            self.doc.props.push(ContractProp {
                name: name.into(),
                ty: content.into(),
                doc: get("data-doc").into(),
            });
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
        });
        index
    }
    fn children(&mut self, nodes: &[Node], parent: usize) -> Result<Vec<usize>, Error> {
        nodes
            .iter()
            .filter(|n| !matches!(n, Node::Text(text, _) if text.trim().is_empty()))
            .map(|n| self.node(n, parent))
            .collect()
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
    fn element(
        &mut self,
        tag: &str,
        attrs: &[Attr],
        children: &[Node],
        at: usize,
        parent: usize,
    ) -> Result<usize, Error> {
        match tag {
            "option" | "optgroup" => {
                return Err((at, format!("<{tag}> belongs inside a <select>")));
            }
            "summary" => {
                return Err((at, "<summary> is the first child of a <details>".into()));
            }
            "select" => {
                let options = select_options(children)?;
                let i = self.element_node(tag, attrs, &[], at, parent)?;
                let strs = |v: Vec<String>| PropValue::Lit(Lit::Strs(v));
                let items = options.iter().map(|o| o.label.clone()).collect();
                let values = options.iter().map(|o| o.value.clone()).collect();
                self.prop(i, tag, "items", strs(items), at, "")?;
                self.prop(i, tag, "values", strs(values), at, "")?;
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
            let controls: Vec<&Node> = children
                .iter()
                .filter(|c| matches!(c, Node::Element { .. }))
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
                && inner_attrs
                    .iter()
                    .any(|a| a.name == "type" && !a.bound && a.value == "checkbox")
            {
                let caption: Vec<Node> = children
                    .iter()
                    .filter(|c| !matches!(c, Node::Element { .. }))
                    .cloned()
                    .collect();
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
                return self.element("input", &merged, &caption, *inner_at, parent);
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
        let internal = match tag {
            "text" | "p" | "span" | "a" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => "label",
            "label" if label_for.is_some() => "label",
            "div" | "section" | "header" | "footer" | "nav" | "main" | "aside" | "article"
            | "ul" | "ol" | "li" => "box",
            "img" => "image",
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
        self.doc.nodes[i].role = match tag {
            "nav" => Some("navigation"),
            "main" => Some("main"),
            "header" => Some("banner"),
            "footer" => Some("contentinfo"),
            "aside" => Some("complementary"),
            "article" => Some("article"),
            "section" if named => Some("region"),
            "ul" | "ol" => Some("list"),
            "li" => Some("listitem"),
            _ => None,
        };
        for attr in attrs.iter().filter(|a| !a.name.starts_with("on")) {
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
            if a.name == "class" || a.name.starts_with("class:") {
                if a.bound {
                    expression(&a.value, a.at)?;
                }
                continue;
            }
            if let Some(event) = a.name.strip_prefix("on") {
                let when = match event {
                    "click" => When::Click,
                    "change" | "input" => When::Changed,
                    "tap" => When::Tap,
                    "doubletap" => When::DoubleTap,
                    "longpress" => When::LongPress,
                    action if self.doc.actions.iter().any(|(name, _)| name == action) => {
                        When::Action(action.into())
                    }
                    _ => {
                        return Err((
                            a.at,
                            format!(
                                "unknown event {event}; use onclick/onchange/ontap, or declare an input action with <meta name=\"action:{event}\" content=\"Label\">"
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
            if a.name.starts_with("style:") || a.name == "style" {
                return Err((a.at, "declare layout and appearance in scoped <style>; bind data to it with a custom property, style:--fill={$hp / $max}, read as var(--fill)".into()));
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
            // An input's type chose its kind; a checkbox's `switch` is how
            // it draws, and a toggle is a switch.
            if tag == "input" && internal != "field" && (a.name == "type" || a.name == "switch") {
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
            if a.name == "hidden" {
                name = "visible".into();
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
        if internal == "box" {
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
        if internal == "label"
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
            link,
        };
        self.runs(nodes, &base, &mut runs, &mut links, translated)?;
        // No space at either end, as a CSS block trims its line.
        if let Some(InlineRun {
            text: Expr::Lit(serde_json::Value::String(t)),
            ..
        }) = runs.first_mut()
        {
            *t = t.trim_start_matches(' ').into();
        }
        if let Some(InlineRun {
            text: Expr::Lit(serde_json::Value::String(t)),
            ..
        }) = runs.last_mut()
        {
            t.truncate(t.trim_end_matches(' ').len());
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
        if runs
            .iter()
            .any(|r| r.bold || r.italic || r.underline || r.strike || r.link.is_some())
        {
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
                    let mut collapsed = String::new();
                    for c in value.chars() {
                        if c.is_ascii_whitespace() {
                            if !collapsed.ends_with(' ') {
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
                        "br" => {
                            if !children.is_empty() {
                                return Err((*at, "<br> is empty".into()));
                            }
                            out.push(run(Expr::Lit("\n".into())));
                            continue;
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
                | Node::Render { at, .. } => {
                    return Err((
                        *at,
                        "a block cannot sit inside text; put it beside the text in a <div>, or use {cond ? a : b}".into(),
                    ));
                }
            }
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
            ("<script>const a = 1</script>", "scripts are not allowed"),
            ("<text>{@html $raw}</text>", "unsupported Svelte block"),
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
                "<text id='T'/><style>#T:hover { animation: fade 1s; } @keyframes fade { from { width: 10px; } }</style>",
                "width cannot be animated in @keyframes",
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
