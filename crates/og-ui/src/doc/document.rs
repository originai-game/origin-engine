use super::*;

impl UiDocument {
    /// Check literal colour references against the styles available to the game.
    /// Length names are schema-defined and already checked by `parse`.
    pub fn validate_tokens(&self, themes: &[crate::Theme]) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let mut check = |paint: &Paint, line: usize| {
            let mut names = Vec::new();
            match paint {
                Paint::Token(name) => names.push(name.clone()),
                Paint::Css(c) => c.tokens(&mut names),
                Paint::Color(_) => {}
            }
            for name in names {
                if !themes.iter().any(|theme| theme.color(&name).is_some()) {
                    errors.push(format!(
                        "{}:{line}: unresolved colour token --{}",
                        self.source,
                        css::kebab(&name)
                    ));
                }
            }
        };
        for node in &self.nodes {
            for rule in &node.span_rules {
                if let Some(crate::paint::ColorRef::Token { name, .. }) = &rule.style.color {
                    check(&Paint::Token(name.clone()), node.line);
                }
            }
            for prop in node
                .props
                .iter()
                .chain(node.at.iter().flat_map(|(_, props)| props))
                .chain(node.rules.iter().flat_map(|rule| &rule.props))
            {
                if let PropValue::Lit(Lit::Paint(paint)) = &prop.value {
                    check(paint, prop.line);
                }
            }
            for value in node
                .groups
                .iter()
                .flat_map(|g| &g.states)
                .flat_map(|s| &s.values)
            {
                if let StateValue::Background(paint) | StateValue::Ink(paint) = value {
                    check(paint, node.line);
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Parses and validates a document (`source`: its asset id, for errors
    /// and the write-back). Every error has its line and its fix.
    ///
    /// Documents use scriptless Svelte markup and scoped CSS (docs/ui.md §3.2).
    pub fn parse(source: &str, text: &str) -> Result<UiDocument, Vec<String>> {
        super::svelte::parse(source, text, Vec::new())
    }

    /// [`UiDocument::parse`] with the stylesheets its `@import`s name, by
    /// the path written ([`UiDocument::imports`] lists them).
    pub fn parse_with_imports(
        source: &str,
        text: &str,
        imports: Vec<(String, String)>,
    ) -> Result<UiDocument, Vec<String>> {
        super::svelte::parse(source, text, imports)
    }

    /// The paths a stylesheet's `@import`s name, as written: what the cook
    /// reads from beside the document (relative to `source`, through
    /// [`super::at_rules::resolve`]) before [`UiDocument::parse_with_imports`].
    pub fn imports(css: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut rest = css;
        while let Some(at) = rest.find("@import") {
            rest = &rest[at + 7..];
            let path = rest.trim_start();
            let path = path.strip_prefix("url(").unwrap_or(path).trim_start();
            if let Some(q) = path.chars().next().filter(|c| *c == '"' || *c == '\'')
                && let Some(end) = path[1..].find(q)
            {
                out.push(path[1..1 + end].to_string());
            }
        }
        out
    }

    /// The same document, whatever its lines and text.
    pub fn same(&self, other: &UiDocument) -> bool {
        fn plain(d: &UiDocument) -> UiDocument {
            let mut d = d.clone();
            d.text.clear();
            for n in &mut d.nodes {
                n.line = 0;
                n.source_offset = 0;
                for p in n
                    .props
                    .iter_mut()
                    .chain(n.at.iter_mut().flat_map(|(_, ps)| ps.iter_mut()))
                    .chain(n.rules.iter_mut().flat_map(|rule| rule.props.iter_mut()))
                {
                    p.line = 0;
                    p.source.clear();
                }
                for rule in &mut n.rules {
                    rule.line = 0;
                    for declaration in &mut rule.declarations {
                        declaration.source.clear();
                    }
                }
                for rule in &mut n.span_rules {
                    for declaration in &mut rule.declarations {
                        declaration.source.clear();
                    }
                }
                for t in &mut n.triggers {
                    t.line = 0;
                }
                for a in &mut n.args {
                    a.2 = 0;
                }
            }
            d
        }
        plain(self) == plain(other)
    }

    /// The checks that read the whole tree: timelines named, parts
    /// declared and used, bindings rooted in the contract.
    pub(super) fn check(&self, errors: &mut Vec<String>) {
        let timelines: Vec<&str> = self.timelines.iter().map(|(n, _)| n.as_str()).collect();
        let mut used_parts = Vec::new();
        for (i, n) in self.nodes.iter().enumerate() {
            let line = n.line;
            for t in &n.triggers {
                if let Act::Play(tl) = &t.act
                    && !timelines.contains(&tl.as_str())
                    && !tl.contains('/')
                {
                    errors.push(format!("line {}: the timeline '{tl}' is not declared in this document (timeline {tl} {{ … }}; another document's is <doc>/<name>)", t.line));
                }
            }
            for g in &n.groups {
                for s in &g.states {
                    for tl in [&s.on_enter, &s.on_exit] {
                        if !tl.is_empty() && !timelines.contains(&tl.as_str()) && !tl.contains('/')
                        {
                            errors.push(format!(
                                "line {line}: the state '{}' plays '{tl}', which is not declared",
                                s.name
                            ));
                        }
                    }
                }
            }
            if let Some(p) = n.props.iter().find(|p| p.name() == "part")
                && let PropValue::Lit(Lit::Str(name)) = &p.value
            {
                used_parts.push(name.clone());
                if !self.parts.contains(name) {
                    errors.push(format!(
                        "line {}: the part '{name}' is not in the document's `parts` line",
                        p.line
                    ));
                }
            }
            if !self.props.is_empty() {
                let locals = self.locals(i);
                for (path, line) in self.node_paths_at(n) {
                    let root = path.split('.').next().unwrap_or("").trim_start_matches('$');
                    let known = self.props.iter().any(|p| p.name == root)
                        || locals.iter().any(|l| l == root)
                        || root.starts_with(|c: char| c.is_ascii_uppercase())
                        || root.starts_with('$')
                        // The engine's `$ui` store (docs/ui.md §2), bound at every mount.
                        || (root == "ui" && path.starts_with('$'));
                    if !known {
                        errors.push(format!("line {line}: '{path}' reads '{root}', which is not a prop of the document (props {{ … }}), a list item, or a resource (Capitalised)"));
                    }
                }
            }
        }
        for p in &self.parts {
            if !used_parts.contains(p) {
                errors.push(format!(
                    "the part '{p}' is listed in `parts` but no element is it (part {p})"
                ));
            }
        }
    }

    /// The `for` variables in scope at node `i`.
    fn locals(&self, mut i: usize) -> Vec<String> {
        let mut out = Vec::new();
        while i != NONE {
            if let NodeKind::For { var, .. } = &self.nodes[i].kind {
                out.push(var.clone());
            }
            i = self.nodes[i].parent;
        }
        out
    }

    /// Every path a node's bindings read, each with the line that reads it.
    fn node_paths_at(&self, n: &DocNode) -> Vec<(String, usize)> {
        let mut out = Vec::new();
        let mut add = |v: &PropValue, line: usize| {
            let mut p = Vec::new();
            match v {
                PropValue::Bind(e, _) => e.paths(&mut p),
                PropValue::Model(path) => p.push(path.clone()),
                PropValue::Template(t) => t.paths(&mut p),
                PropValue::Lit(_) => {}
            }
            out.extend(p.into_iter().map(|p| (p, line)));
        };
        for p in n.props.iter().chain(n.at.iter().flat_map(|a| &a.1)) {
            add(&p.value, p.line);
        }
        for (_, v, line) in &n.args {
            add(v, *line);
        }
        let mut rest = Vec::new();
        let bare = DocNode {
            props: Vec::new(),
            at: Vec::new(),
            args: Vec::new(),
            ..n.clone()
        };
        self.node_paths(&bare, &mut rest);
        out.extend(rest.into_iter().map(|p| (p, n.line)));
        out
    }

    /// The Rive artboards its elements play (`-og-frame: rive("<id>")`), each
    /// once, in document order: what loading it loads beside it.
    pub fn rive_ids(&self) -> Vec<String> {
        let Some((spec, _)) = super::schema::prop("rive") else {
            return Vec::new();
        };
        let mut out: Vec<String> = Vec::new();
        for n in &self.nodes {
            for p in n
                .props
                .iter()
                .chain(n.at.iter().flat_map(|a| &a.1))
                .chain(n.rules.iter().flat_map(|rule| &rule.props))
            {
                if p.spec == spec
                    && let PropValue::Lit(Lit::Str(id)) = &p.value
                    && !out.contains(id)
                {
                    out.push(id.clone());
                }
            }
        }
        out
    }

    /// Every path a node's bindings read (its own, not its children's).
    pub fn node_paths(&self, n: &DocNode, out: &mut Vec<String>) {
        let value = |v: &PropValue, out: &mut Vec<String>| match v {
            PropValue::Bind(e, _) => e.paths(out),
            PropValue::Model(path) => out.push(path.clone()),
            PropValue::Template(t) => t.paths(out),
            PropValue::Lit(_) => {}
        };
        for p in n.props.iter().chain(n.at.iter().flat_map(|a| &a.1)) {
            value(&p.value, out);
        }
        for (_, v, _) in &n.args {
            value(v, out);
        }
        for (_, expression) in &n.selector_attrs {
            expression.paths(out);
        }
        for rule in &n.rules {
            if let Some(condition) = &rule.condition {
                condition.paths(out);
            }
            for p in &rule.props {
                value(&p.value, out);
            }
        }
        for rule in &n.span_rules {
            if let Some(condition) = &rule.condition {
                condition.paths(out);
            }
        }
        for g in &n.groups {
            if let Some(b) = &g.bind {
                b.paths(out);
            }
        }
        for t in &n.triggers {
            if let When::Change(e) = &t.when {
                e.paths(out);
            }
            if let Act::Set(_, e) = &t.act {
                e.paths(out);
            }
        }
        match &n.kind {
            NodeKind::For { list, key, .. } => {
                list.paths(out);
                key.paths(out);
            }
            NodeKind::If(e) | NodeKind::Key(e) => e.paths(out),
            _ => {}
        }
    }

    /// The node at a path of ids below the root (`Header/Title`; the
    /// root's own id first is allowed too), through `for` templates
    /// (`Items/Card#3/Buy` is the template's `Buy`) and `if` blocks.
    pub fn node_at(&self, path: &str) -> Option<usize> {
        let mut at = self.root;
        let mut parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        if parts.first() == Some(&self.nodes.get(at)?.id.as_str()) {
            parts.remove(0);
        }
        for p in parts {
            let id = p.split('#').next().unwrap_or(p);
            at = self.child_by_id(at, id)?;
        }
        Some(at)
    }

    fn child_by_id(&self, at: usize, id: &str) -> Option<usize> {
        for &c in &self.nodes[at].children {
            let n = &self.nodes[c];
            if matches!(
                n.kind,
                NodeKind::For { .. } | NodeKind::If(_) | NodeKind::Key(_)
            ) {
                if let Some(x) = self.child_by_id(c, id) {
                    return Some(x);
                }
            } else if n.id == id {
                return Some(c);
            }
        }
        None
    }

    /// Saves a probe edit as an authored attribute, text or scoped CSS rule.
    /// Unrelated source is preserved; an invalid edit leaves the document unchanged.
    pub fn set_prop(&mut self, node: usize, prop: &str, value: &str) -> Result<(), String> {
        let text = super::writeback::edit(self, node, prop, value)?;
        let doc = UiDocument::parse(&self.source, &text).map_err(|e| e.join("; "))?;
        *self = doc;
        Ok(())
    }

    /// A fixture by name (`default`: the first).
    pub fn fixture(&self, name: &str) -> Option<&serde_json::Value> {
        self.fixtures
            .iter()
            .find(|(n, _)| n == name)
            .or_else(|| (name == "default").then(|| self.fixtures.first()).flatten())
            .map(|(_, v)| v)
    }

    /// Its static structure as JSON (dump_ui's `documents`, og.ui.document).
    pub fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        let mut slots = Vec::new();
        let mut arts = Vec::new();
        for n in &self.nodes {
            match n.kind {
                NodeKind::Element("slot") => slots.push(n.id.clone()),
                NodeKind::Element("art") => arts.push(n.id.clone()),
                _ => {}
            }
        }
        json!({
            "name": self.name,
            "kind": match self.kind { DocKind::Screen => "screen", DocKind::Component => "component" },
            "source": self.source,
            "layer": {
                "kind": self.layer.kind.name(), "input": self.layer.input.name(),
                "capture": self.layer.capture, "pause": self.layer.pause,
                "under": self.layer.under.name(),
                "back": match &self.layer.back { Back::Pop => "pop".to_string(), Back::None => "none".into(), Back::Emit(n) => format!("emit {n}") },
                "focus": self.layer.focus, "remember": self.layer.remember,
            },
            "props": self.props.iter().map(|p| json!({ "name": p.name, "type": p.ty, "doc": p.doc, "default": p.default })).collect::<Vec<_>>(),
            "parts": self.parts,
            "slots": slots,
            "arts": arts,
            "actions": self.actions.iter().map(|(a, h)| json!({ "action": a, "hint": h })).collect::<Vec<_>>(),
            "timelines": self.timelines.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>(),
            "fixtures": self.fixtures.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>(),
            "nodes": self.nodes.len(),
        })
    }
}

/// A value as a document line writes it for a prop of type `ty`: a text
/// prop's bare value is quoted; anything else is taken as written.
pub(crate) fn value_text(ty: Ty, value: &str) -> String {
    let v = value.trim();
    let quoted = v.starts_with('"') && v.ends_with('"') && v.len() >= 2;
    match ty {
        Ty::Text if !quoted && !v.starts_with("bind ") => quote(v),
        Ty::Name if !quoted && v.contains(char::is_whitespace) => quote(v),
        _ => v.to_string(),
    }
}

/// A probe's value text for a prop (`ui:set`): parsed as a document line
/// would be.
pub(crate) fn parse_probe_value(prop: &str, value: &str) -> Result<(usize, PropValue), String> {
    let (spec, p) = schema::prop(prop)
        .ok_or_else(|| format!("'{prop}' is not a prop (og.ui.documentSchema lists them)"))?;
    let text = format!("type x\nversion 1\n---\nv {}\n", value_text(p.ty, value));
    let doc = Document::parse(&text).map_err(|e| format!("{prop}={value}: {}", e.reason()))?;
    let vals = &doc.body.get("v").ok_or("no value")?.values;
    parse_value(p.ty, vals)
        .map(|v| (spec, v))
        .map_err(|e| format!("{prop}: {e}"))
}
