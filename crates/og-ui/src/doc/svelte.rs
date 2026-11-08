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
        "image" => "<img>",
        "label" => {
            "<label> around one <input type=\"checkbox\"> and its caption, or <text> for text"
        }
        _ => return None,
    };
    Some(format!("<{tag}> is not a core element; write {html}"))
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
        let internal = match tag {
            "text" => "label",
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
            other => match renamed(other) {
                Some(hint) => return Err((at, hint)),
                None => other,
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
            if a.name.starts_with("style:") || a.name == "style" {
                return Err((a.at, "declare layout and appearance in scoped <style>; style directives require a CSS token binding".into()));
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
                (_, other) => property(other),
            };
            if name == "value" && internal == "field" {
                name = "input".into();
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
        let mut text = Expr::Lit(serde_json::Value::String(String::new()));
        let mut has_text = false;
        let mut bound_text = false;
        let translated = attrs.iter().any(|a| a.name == "i18n");
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
