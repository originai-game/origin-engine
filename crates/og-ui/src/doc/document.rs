use super::*;

impl UiDocument {
    /// Check literal colour references against the styles available to the game.
    /// Length names are schema-defined and already checked by `parse`.
    pub fn validate_tokens(&self, themes: &[crate::Theme]) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let mut check = |paint: &Paint, line: usize| {
            if let Paint::Token(name) = paint
                && !themes.iter().any(|theme| theme.color(name).is_some())
            {
                errors.push(format!(
                    "{}:{line}: unresolved colour token --{}",
                    self.source,
                    css::kebab(name)
                ));
            }
        };
        for node in &self.nodes {
            for prop in node
                .props
                .iter()
                .chain(node.at.iter().flat_map(|(_, props)| props))
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
    /// A document is `version 2`, the CSS dialect (docs/ui.md §10.4); a
    /// version 1 text is refused with the command that migrates it.
    pub fn parse(source: &str, text: &str) -> Result<UiDocument, Vec<String>> {
        match header_version(text) {
            Some(2) => {
                let doc = super::css::lower(text).map_err(|e| {
                    e.into_iter()
                        .map(|e| format!("{source}: {e}"))
                        .collect::<Vec<_>>()
                })?;
                Self::build(source, text, doc)
            }
            Some(1) => Err(vec![format!(
                "{source}: a version 1 document: `og ui migrate {source}` writes it in the CSS dialect (version 2, docs/ui.md §10.4)"
            )]),
            _ => {
                let doc =
                    Document::parse(text).map_err(|e| vec![format!("{source}: {}", e.reason())])?;
                Err(vec![format!(
                    "{source}: version {} of a document is not known (2)",
                    doc.version
                )])
            }
        }
    }

    /// Parses a version 1 document (what `og ui migrate` reads).
    pub fn parse_v1(source: &str, text: &str) -> Result<UiDocument, Vec<String>> {
        let doc = Document::parse(text).map_err(|e| vec![format!("{source}: {}", e.reason())])?;
        if doc.version != 1 {
            return Err(vec![format!("{source}: not a version 1 document")]);
        }
        Self::build(source, text, doc)
    }

    /// A version 1 document's text as version 2 ([`super::css::migrate`]),
    /// checked: the new text reads back as the same document (each line's
    /// element, prop and value; a value written jointly, such as `y` as
    /// `translate: 0 -4px`, gains its partner's neutral value).
    pub fn migrate(source: &str, text: &str) -> Result<String, Vec<String>> {
        Self::parse_v1(source, text)?;
        let new_text = super::css::migrate(text).map_err(|e| {
            e.into_iter()
                .map(|e| format!("{source}: {e}"))
                .collect::<Vec<_>>()
        })?;
        let new = Self::parse(source, &new_text)?;
        let mut doc = Document::parse(text).map_err(|e| vec![e.reason().to_string()])?;
        super::css::partner_neutrals(&mut doc.body);
        let old = Self::build(source, text, doc)?;
        if old.same(&new) {
            Ok(new_text)
        } else {
            Err(vec![format!(
                "{source}: the migrated text reads as another document (a bug in og ui migrate; the text is unchanged)"
            )])
        }
    }

    /// The same document, whatever its lines and text.
    pub fn same(&self, other: &UiDocument) -> bool {
        fn plain(d: &UiDocument) -> UiDocument {
            let mut d = d.clone();
            d.text.clear();
            for n in &mut d.nodes {
                n.line = 0;
                for p in n
                    .props
                    .iter_mut()
                    .chain(n.at.iter_mut().flat_map(|(_, ps)| ps.iter_mut()))
                {
                    p.line = 0;
                    p.source.clear();
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

    fn build(source: &str, text: &str, doc: Document) -> Result<UiDocument, Vec<String>> {
        let kind = match doc.ty.as_str() {
            SCREEN_TYPE => DocKind::Screen,
            COMPONENT_TYPE => DocKind::Component,
            other => {
                return Err(vec![format!(
                    "{source}: an .ogt of type '{other}' is not a document (screen or component)"
                )]);
            }
        };
        let mut ctx = Ctx {
            errors: Vec::new(),
            nodes: Vec::new(),
        };
        let mut d = UiDocument {
            kind,
            name: String::new(),
            source: source.to_string(),
            text: text.to_string(),
            layer: LayerSpec::of(LayerKind::Menu),
            props: Vec::new(),
            parts: Vec::new(),
            actions: Vec::new(),
            timelines: Vec::new(),
            fixtures: Vec::new(),
            nodes: Vec::new(),
            root: NONE,
        };
        // The layer kind first: the other layer lines change its defaults.
        if let Some(e) = doc.body.get("layer") {
            match e
                .values
                .first()
                .and_then(Value::as_word)
                .and_then(LayerKind::from_name)
            {
                Some(k) => d.layer = LayerSpec::of(k),
                None => ctx.err(
                    e.line,
                    format!("layer is one of {}", LayerKind::NAMES.join(", ")),
                ),
            }
        }
        let screen_only = |ctx: &mut Ctx, e: &Entry| {
            if kind != DocKind::Screen {
                ctx.err(
                    e.line,
                    format!(
                        "'{}' is a screen's (a component is placed by the screen that uses it)",
                        e.key
                    ),
                );
            }
        };
        for e in &doc.body.entries {
            let w = words(e);
            let first = w.first().cloned().unwrap_or_default();
            match e.key.as_str() {
                "name" => d.name = first,
                "layer" => screen_only(&mut ctx, e),
                "input" => {
                    screen_only(&mut ctx, e);
                    d.layer.input = match first.as_str() {
                        "game" => InputMode::Game,
                        "menu" => InputMode::Menu,
                        "all" => InputMode::All,
                        _ => {
                            ctx.err(e.line, "input is game, menu or all");
                            continue;
                        }
                    };
                }
                "capture" => d.layer.capture = e.values.first().and_then(Value::as_bool).unwrap_or(true),
                "pause" => d.layer.pause = e.values.first().and_then(Value::as_bool).unwrap_or(true),
                "remember" => d.layer.remember = e.values.first().and_then(Value::as_bool).unwrap_or(true),
                "under" => {
                    d.layer.under = match first.as_str() {
                        "none" => Under::None,
                        "dim" => Under::Dim,
                        "blur" => Under::Blur,
                        "hide" => Under::Hide,
                        _ => {
                            ctx.err(e.line, "under is none, dim, blur or hide");
                            continue;
                        }
                    };
                }
                "back" => {
                    d.layer.back = match (first.as_str(), w.get(1)) {
                        ("pop", None) => Back::Pop,
                        ("none", None) => Back::None,
                        ("emit", Some(n)) => Back::Emit(n.clone()),
                        _ => {
                            ctx.err(e.line, "back is pop, none or emit <name>");
                            continue;
                        }
                    };
                }
                "focus" => d.layer.focus = first,
                "enter" | "exit" => match parse_presence(&e.values, e.key == "enter") {
                    Ok(p) if e.key == "enter" => d.layer.enter = Some(p),
                    Ok(p) => d.layer.exit = Some(p),
                    Err(why) => ctx.err(e.line, why),
                },
                "props" => {
                    for p in e.block.as_ref().map_or(&[][..], |b| &b.entries) {
                        let w = words(p);
                        let ty = w.first().cloned().unwrap_or_else(|| "any".into());
                        if !["number", "string", "bool", "list", "object", "any"].contains(&ty.as_str()) {
                            ctx.err(p.line, format!("'{ty}' is not a prop type: number, string, bool, list, object, any"));
                        }
                        d.props.push(ContractProp { name: p.key.clone(), ty, doc: w.get(1).cloned().unwrap_or_default() });
                    }
                }
                "parts" => d.parts = w,
                "actions" => {
                    for a in e.block.as_ref().map_or(&[][..], |b| &b.entries) {
                        d.actions.push((a.key.clone(), words(a).first().cloned().unwrap_or_default()));
                    }
                }
                "timeline" => {
                    if let Some(t) = parse_timeline(&mut ctx, e) {
                        if d.timelines.iter().any(|(n, _)| *n == first) {
                            ctx.err(e.line, format!("the timeline '{first}' is declared twice"));
                        }
                        d.timelines.push((first, t));
                    }
                }
                "fixture" => match w.get(1).map(|j| serde_json::from_str::<serde_json::Value>(j)) {
                    Some(Ok(v)) => d.fixtures.push((first, v)),
                    Some(Err(why)) => ctx.err(e.line, format!("the fixture's JSON: {why}")),
                    None => ctx.err(e.line, "fixture <name> \"<json>\""),
                },
                k if schema::kind(k).is_some() || k == "use" => {
                    if d.root != NONE {
                        ctx.err(e.line, "a document has one root element");
                        continue;
                    }
                    d.root = ctx.node(e, NONE).unwrap_or(NONE);
                }
                k => ctx.err(e.line, format!("'{k}' is not part of a document: name, layer, input, capture, pause, under, back, focus, remember, enter, exit, props, parts, actions, timeline, fixture, and one root element")),
            }
        }
        if d.name.is_empty() {
            ctx.errors
                .push(format!("{source}: a document has a name (name \"shop\")"));
        }
        if d.root == NONE && ctx.errors.is_empty() {
            ctx.errors.push(format!(
                "{source}: a document has one root element (column, row, stack, panel …)"
            ));
        }
        d.nodes = std::mem::take(&mut ctx.nodes);
        d.check(&mut ctx.errors);
        if ctx.errors.is_empty() {
            Ok(d)
        } else {
            Err(ctx
                .errors
                .into_iter()
                .map(|e| format!("{source}: {e}"))
                .collect())
        }
    }

    /// The checks that read the whole tree: timelines named, parts
    /// declared and used, bindings rooted in the contract.
    fn check(&self, errors: &mut Vec<String>) {
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
                    let root = path.split('.').next().unwrap_or("");
                    let known = self.props.iter().any(|p| p.name == root)
                        || locals.iter().any(|l| l == root)
                        || root.starts_with(|c: char| c.is_ascii_uppercase())
                        || root.starts_with('$');
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
            for p in n.props.iter().chain(n.at.iter().flat_map(|a| &a.1)) {
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
            PropValue::Template(t) => t.paths(out),
            PropValue::Lit(_) => {}
        };
        for p in n.props.iter().chain(n.at.iter().flat_map(|a| &a.1)) {
            value(&p.value, out);
        }
        for (_, v, _) in &n.args {
            value(v, out);
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
            NodeKind::For { list, .. } => list.paths(out),
            NodeKind::If(e) => e.paths(out),
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
            if matches!(n.kind, NodeKind::For { .. } | NodeKind::If(_)) {
                if let Some(x) = self.child_by_id(c, id) {
                    return Some(x);
                }
            } else if n.id == id {
                return Some(c);
            }
        }
        None
    }

    /// Writes `prop value` into node `node`'s block in the text, keeping
    /// every other line: its line is replaced when the node sets the prop,
    /// else one is added after the node's opening line. The document is
    /// parsed again from the new text (refused, unchanged, if it no longer
    /// parses).
    pub fn set_prop(&mut self, node: usize, prop: &str, value: &str) -> Result<(), String> {
        let n = self.nodes.get(node).ok_or("no such node")?;
        let (_, spec) = schema::prop(prop).ok_or_else(|| format!("'{prop}' is not a prop"))?;
        let line = if header_version(&self.text) == Some(2) && super::css::css_name(prop).is_some()
        {
            // A declaration: the line keeps the other props written on it
            // (`translate: 0 -4px` holds `x` and `y`), with this one's value.
            let vals = |src: &str| -> Result<Vec<Value>, String> {
                match og_core::ogt::parse_line(&format!("v {src}"), 0) {
                    Ok(og_core::ogt::Line::Entry { entry, .. }) => Ok(entry.values),
                    Ok(_) => Ok(Vec::new()),
                    Err(e) => Err(format!("{prop}={value}: {}", e.reason())),
                }
            };
            let mut on_line: Vec<(String, Vec<Value>)> = Vec::new();
            if let Some(p) = n.props.iter().rev().find(|p| p.name() == prop) {
                for q in n
                    .props
                    .iter()
                    .filter(|q| q.line == p.line && q.name() != prop)
                {
                    on_line.push((q.name().to_string(), vals(&q.source)?));
                }
            }
            on_line.insert(0, (prop.to_string(), vals(&value_text(spec.ty, value))?));
            let own = |k: &str| {
                n.props
                    .iter()
                    .rev()
                    .find(|p| p.name() == k)
                    .and_then(|p| vals(&p.source).ok())
            };
            super::css::decl_line(&on_line, false, &own)?
        } else {
            format!("{prop} {}", value_text(spec.ty, value))
        };
        let mut lines: Vec<String> = self.text.split('\n').map(str::to_string).collect();
        let indent = |l: &str| l.len() - l.trim_start().len();
        if let Some(p) = n.props.iter().rev().find(|p| p.name() == prop) {
            let old = &lines[p.line - 1];
            lines[p.line - 1] = format!("{}{line}", &old[..indent(old)]);
        } else {
            let open = &lines[n.line - 1];
            let pad = " ".repeat(indent(open) + 2);
            if open.trim_end().ends_with('{') {
                lines.insert(n.line, format!("{pad}{line}"));
            } else {
                let base = open[..indent(open)].to_string();
                let head = open.trim_end().to_string();
                lines[n.line - 1] = format!("{head} {{");
                lines.insert(n.line, format!("{pad}{line}"));
                lines.insert(n.line + 1, format!("{base}}}"));
            }
        }
        let text = lines.join("\n");
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
            "props": self.props.iter().map(|p| json!({ "name": p.name, "type": p.ty, "doc": p.doc })).collect::<Vec<_>>(),
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
/// The `version` of a text's header.
fn header_version(text: &str) -> Option<u32> {
    let head: String = text
        .split('\n')
        .take_while(|l| l.trim() != "---")
        .flat_map(|l| [l, "\n"])
        .collect();
    Document::parse(&format!("{head}---\n"))
        .ok()
        .map(|d| d.version)
}

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
