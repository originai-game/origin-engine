//! Scoped selectors lower once to matching document nodes and reactive predicates.
use super::StyleDeclaration;
use super::selectors::Selector;
use super::syntax::{Attr, Error};
use super::*;

#[derive(Clone)]
struct Rule {
    selector: String,
    body: String,
    query: AtQuery,
    at: usize,
    body_at: usize,
    order: usize,
    /// HTML's user-agent stylesheet ([`super::lists::UA_CSS`]): cascades
    /// before every author rule, whatever their specificity.
    ua: bool,
    /// Its cascade layer's full name (`a.b`); `None` is unlayered.
    layer: Option<String>,
    /// Inside `@starting-style`.
    starting: bool,
}

/// Where a rule is read: the conditions, layer, scope and nesting parent
/// around it.
#[derive(Clone, Default)]
struct Context {
    query: AtQuery,
    layer: Option<String>,
    /// The enclosing style rule's selector list (CSS nesting).
    parent: Option<String>,
    /// `@scope (root) to (limit)`.
    scope: Option<(String, Option<String>)>,
    starting: bool,
}

/// What reading a document's stylesheets collects besides rules.
struct Reader<'a> {
    sheet: &'a mut super::at_rules::Sheet,
    /// Cascade layer names in first-seen order.
    layers: Vec<String>,
    anonymous: usize,
    /// The document's asset id, for paths written beside it.
    base: String,
    /// `@import`s may still come (none but `@layer` statements yet).
    imports_open: bool,
    depth: usize,
}

/// The CSS-wide keywords every property takes.
pub(super) fn wide_keyword(value: &str) -> Option<&'static str> {
    let v = value.trim();
    ["inherit", "initial", "unset", "revert", "revert-layer"]
        .into_iter()
        .find(|k| v.eq_ignore_ascii_case(k))
}

pub(super) fn apply(
    doc: &mut UiDocument,
    elements: &[(usize, String, Vec<Attr>)],
    styles: &[(String, usize)],
) -> Result<(), Error> {
    let mut rules = Vec::new();
    let mut sheet = std::mem::take(&mut doc.sheet);
    let mut reader = Reader {
        sheet: &mut sheet,
        layers: Vec::new(),
        anonymous: 0,
        base: doc.source.clone(),
        imports_open: true,
        depth: 0,
    };
    reader.rules(super::lists::UA_CSS, 0, &Context::default(), &mut rules)?;
    for rule in &mut rules {
        // Its offsets are in the UA text, not the document's.
        rule.ua = true;
        rule.at = 0;
        rule.body_at = 0;
    }
    for (text, at) in styles {
        reader.imports_open = true;
        reader.rules(text, *at, &Context::default(), &mut rules)?;
    }
    // Precedence, lowest first: a layer's sublayers before its own rules,
    // siblings in the order first seen.
    fn post_order(prefix: &str, names: &[String], out: &mut Vec<String>) {
        for name in names {
            let child = match name.strip_prefix(prefix) {
                Some(rest) if !rest.is_empty() && !rest.contains('.') => name,
                _ => continue,
            };
            post_order(&format!("{child}."), names, out);
            out.push(child.clone());
        }
    }
    let mut layers = Vec::new();
    post_order("", &reader.layers, &mut layers);
    sheet.layers = layers;
    doc.sheet = sheet;
    let layers = doc.sheet.layers.clone();
    let rank = |layer: &Option<String>| -> u32 {
        match layer {
            None => u32::MAX,
            Some(name) => layers
                .iter()
                .position(|l| l == name)
                .map_or(u32::MAX - 1, |k| k as u32 + 1),
        }
    };
    for rule in &rules {
        if let Some(name) = rule
            .selector
            .strip_prefix("@keyframes")
            .filter(|n| n.starts_with(char::is_whitespace))
        {
            let name = name.trim();
            if !syntax::identifier(name) {
                return Err((rule.at, "@keyframes needs a CSS identifier".into()));
            }
            let mut frames = Vec::new();
            for item in items(&rule.body, rule.body_at)? {
                match item {
                    Item::Block { prelude, block, .. } => {
                        frames.push((prelude.to_string(), block.to_string()))
                    }
                    Item::Statement(_, at) => {
                        return Err((at, "a keyframe needs { declarations }".into()));
                    }
                }
            }
            let parsed = super::motion::keyframes(&frames)
                .map_err(|e| (rule.at, format!("@keyframes {name}: {e}")))?;
            let key = if rule.query == AtQuery::default() {
                name.to_string()
            } else {
                format!("{name}@{}", doc.keyframe_rules.len())
            };
            doc.keyframe_rules.retain(|(_, _, k)| *k != key);
            doc.keyframes.retain(|(k, _)| *k != key);
            doc.keyframe_rules
                .push((name.to_string(), rule.query.clone(), key.clone()));
            doc.keyframes.push((key, parsed));
        }
    }
    rules.retain(|rule| !rule.selector.to_ascii_lowercase().starts_with("@keyframes"));
    let transitions: Vec<(&str, &str, usize, bool)> = rules
        .iter()
        .filter(|r| super::view_transition::selects(&r.selector))
        .map(|r| {
            let conditional = r.query != AtQuery::default();
            (r.selector.as_str(), r.body.as_str(), r.at, conditional)
        })
        .collect();
    super::view_transition::apply(doc, &transitions)?;
    rules.retain(|r| !super::view_transition::selects(&r.selector));
    for rule in &mut rules {
        backdrop(rule)?;
    }
    let mut parsed = Vec::new();
    for rule in rules {
        for selector in Selector::parse_list(&rule.selector).map_err(|e| (rule.at, e))? {
            let mut rule = rule.clone();
            rule.selector = selector.source.clone();
            parsed.push((selector, rule));
        }
    }
    parsed.sort_by_key(|(selector, rule)| {
        let layer = if rule.ua { 0 } else { rank(&rule.layer) };
        (!rule.ua, layer, selector.specificity(), rule.order)
    });
    for (parsed, rule) in parsed {
        let structure = parsed
            .structural()
            .then(|| std::sync::Arc::new(parsed.clone()));
        if let Some((parent, selector)) = rule.selector.split_once("::part(") {
            if structure.is_some() {
                return Err((
                    rule.at,
                    format!(
                        "{}: sibling combinators and structural pseudo-classes cannot select ::part spans; select the text element itself",
                        parsed.source
                    ),
                ));
            }
            let span = super::spans::parse(selector, &rule.body).map_err(|e| (rule.at, e))?;
            let parent = if parent.is_empty() { "text" } else { parent };
            for (i, tag, _) in elements {
                let Some(condition) = matches(doc, elements, *i, parent, rule.at)? else {
                    continue;
                };
                // A text element's spans: `text::part(name)`, `p::part(…)`.
                if doc.nodes[*i].kind != NodeKind::Element("label") {
                    return Err((
                        rule.at,
                        format!("<{tag}> has no text spans; ::part(name) selects a text element's"),
                    ));
                }
                let mut span = span.clone();
                span.query = rule.query.clone();
                span.condition = Some(condition);
                doc.nodes[*i].span_rules.push(span);
            }
            continue;
        }
        let known: Vec<String> = doc.keyframe_rules.iter().map(|(n, ..)| n.clone()).collect();
        let (declarations, body) =
            lower(&rule.body, &|name| known.iter().any(|k| k == name)).map_err(|e| (rule.at, e))?;
        let entries = css::declarations(&body, syntax::location(&doc.text, rule.at).0)
            .map_err(|e| (rule.at, e))?;
        for (i, _, _) in elements {
            let condition = match &structure {
                // The subject's static facts prune elements at cook; the whole
                // selector is matched on the instantiated tree.
                Some(_) => match parsed
                    .subject_filter()
                    .condition(doc, elements, *i)
                    .map_err(|e| (rule.at, e))?
                {
                    Expr::Lit(serde_json::Value::Bool(false)) => continue,
                    _ => Expr::Lit(true.into()),
                },
                None => match matches(doc, elements, *i, &rule.selector, rule.at)? {
                    Some(condition) => condition,
                    None => continue,
                },
            };
            let kind = match doc.nodes[*i].kind {
                NodeKind::Element(k) => k,
                _ => "column",
            };
            let mut props = Vec::new();
            for e in &entries {
                // `flex-direction` is a box's layout and a checkbox's
                // label's caption side: each element takes its own.
                let key = e.key.as_str();
                if (key == "direction" && kind == "toggle")
                    || (key == "labelDirection" && kind != "toggle")
                {
                    continue;
                }
                let (spec, p) =
                    schema::prop(key).ok_or((rule.at, format!("unknown CSS target {}", e.key)))?;
                if !p.applies(kind) {
                    return Err((
                        rule.at,
                        if e.key == "display" {
                            format!(
                                "display: {} lays out a <div> (or another HTML block); a {kind}'s layout is its tag, so it takes only display: none",
                                written(&e.values)
                            )
                        } else {
                            format!("CSS property {} does not apply to {kind}", e.key)
                        },
                    ));
                }
                props.push(PropSet {
                    spec,
                    value: parse_value(p.ty, &e.values).map_err(|e| (rule.at, e))?,
                    line: e.line,
                    source: written(&e.values),
                });
            }
            let condition = match condition {
                Expr::Lit(serde_json::Value::Bool(true)) => None,
                c => Some(c),
            };
            doc.nodes[*i].rules.push(StyleRule {
                layer: if rule.ua { 0 } else { rank(&rule.layer) },
                starting: rule.starting,
                query: rule.query.clone(),
                condition,
                structure: structure.clone(),
                props,
                declarations: declarations.clone(),
                line: syntax::location(&doc.text, rule.at).0,
            });
        }
    }
    Ok(())
}

/// One top-level item of a stylesheet or rule body.
enum Item<'a> {
    /// `prelude { block }` at `at`, its block at `block_at`.
    Block {
        prelude: &'a str,
        block: &'a str,
        at: usize,
        block_at: usize,
    },
    /// `text;`: a declaration in a body, a statement at-rule elsewhere.
    Statement(&'a str, usize),
}

/// Splits `text` into its top-level blocks and statements. A custom
/// property whose value holds a `{}` block stays one declaration.
fn items(text: &str, base: usize) -> Result<Vec<Item<'_>>, Error> {
    let mut out = Vec::new();
    let mut at = 0;
    let bytes = text.as_bytes();
    while at < text.len() {
        while at < text.len() && bytes[at].is_ascii_whitespace() {
            at += 1;
        }
        if text[at..].starts_with("/*") {
            let end = text[at + 2..]
                .find("*/")
                .ok_or((base + at, "close CSS comment with */".into()))?;
            at += end + 4;
            continue;
        }
        if at >= text.len() {
            break;
        }
        let start = at;
        let custom = text[at..].trim_start().starts_with("--");
        let (mut depth, mut quote, mut open) = (0i32, None, None);
        let mut end = text.len();
        let mut i = at;
        while i < text.len() {
            let c = text[i..].chars().next().unwrap();
            let next = i + c.len_utf8();
            if let Some(q) = quote {
                if c == '\\' {
                    i = next + text[next..].chars().next().map_or(0, char::len_utf8);
                    continue;
                }
                if c == q {
                    quote = None;
                }
            } else if text[i..].starts_with("/*") {
                i = text[i + 2..].find("*/").map_or(text.len(), |e| i + e + 4);
                continue;
            } else {
                match c {
                    '\'' | '"' => quote = Some(c),
                    '(' | '[' => depth += 1,
                    ')' | ']' => depth -= 1,
                    '{' if depth == 0 && !custom => {
                        open = Some(i);
                        break;
                    }
                    '{' => depth += 1,
                    '}' if depth > 0 => depth -= 1,
                    '}' => return Err((base + i, "unexpected '}' in CSS".into())),
                    ';' if depth == 0 => {
                        end = i;
                        break;
                    }
                    _ => {}
                }
            }
            i = next;
        }
        match open {
            Some(open) => {
                let (mut depth, mut quote) = (1, None);
                let mut j = open + 1;
                while j < text.len() && depth > 0 {
                    let c = text[j..].chars().next().unwrap();
                    j += c.len_utf8();
                    if let Some(q) = quote {
                        if c == '\\' && j < text.len() {
                            j += text[j..].chars().next().unwrap().len_utf8();
                        } else if c == q {
                            quote = None;
                        }
                    } else if text[j - 1..].starts_with("/*") {
                        j = text[j + 1..]
                            .find("*/")
                            .map_or(text.len(), |e| j + 1 + e + 2);
                    } else {
                        match c {
                            '\'' | '"' => quote = Some(c),
                            '{' => depth += 1,
                            '}' => depth -= 1,
                            _ => {}
                        }
                    }
                }
                if depth != 0 {
                    return Err((base + open, "close CSS rule with '}'".into()));
                }
                out.push(Item::Block {
                    prelude: text[start..open].trim(),
                    block: &text[open + 1..j - 1],
                    at: base + start,
                    block_at: base + open + 1,
                });
                at = j;
            }
            None => {
                let statement = text[start..end].trim();
                if !statement.is_empty() {
                    out.push(Item::Statement(statement, base + start));
                }
                at = end + 1;
            }
        }
    }
    Ok(out)
}

/// The at-rule an item's prelude names, lowercased, and the rest.
fn at_rule(prelude: &str) -> Option<(String, &str)> {
    let rest = prelude.strip_prefix('@')?;
    let end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .unwrap_or(rest.len());
    Some((rest[..end].to_ascii_lowercase(), rest[end..].trim()))
}

impl Reader<'_> {
    /// A layer's full name inside `context`, registered on first sight.
    fn layer(&mut self, context: &Context, name: Option<&str>) -> String {
        let name = name.map_or_else(
            || {
                self.anonymous += 1;
                format!("\u{0}{}", self.anonymous)
            },
            str::to_string,
        );
        let full = match &context.layer {
            Some(outer) => format!("{outer}.{name}"),
            None => name,
        };
        // Register every prefix: `a.b` declares `a`.
        let mut prefix = String::new();
        for part in full.split('.') {
            if !prefix.is_empty() {
                prefix.push('.');
            }
            prefix.push_str(part);
            if !self.layers.contains(&prefix) {
                self.layers.push(prefix.clone());
            }
        }
        full
    }

    /// Reads a stylesheet (or a group rule's contents) of style rules.
    fn rules(
        &mut self,
        text: &str,
        base: usize,
        context: &Context,
        out: &mut Vec<Rule>,
    ) -> Result<(), Error> {
        for item in items(text, base)? {
            match item {
                Item::Statement(statement, at) => {
                    self.statement(statement, at, context, out)?;
                }
                Item::Block {
                    prelude,
                    block,
                    at,
                    block_at,
                } => {
                    if prelude.starts_with('@') {
                        self.group(prelude, block, at, block_at, context, out, false)?;
                    } else {
                        self.imports_open = false;
                        self.style_rule(prelude, block, at, block_at, context, out)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// `@layer a, b;` and `@import …;`.
    fn statement(
        &mut self,
        statement: &str,
        at: usize,
        context: &Context,
        out: &mut Vec<Rule>,
    ) -> Result<(), Error> {
        let Some((name, rest)) = at_rule(statement) else {
            return Err((
                at,
                format!("a CSS rule needs {{ declarations }}: `{statement}`"),
            ));
        };
        match name.as_str() {
            "layer" => {
                for layer in rest.split(',') {
                    let layer = layer.trim();
                    if layer.split('.').any(|p| !syntax::identifier(p)) {
                        return Err((at, format!("@layer {layer} is not a layer name")));
                    }
                    self.layer(context, Some(layer));
                }
                Ok(())
            }
            "import" => self.import(rest, at, context, out),
            "charset" | "namespace" => Ok(()),
            other => Err((at, format!("@{other} needs a {{ block }}"))),
        }
    }

    /// `@import url("x.css") [layer | layer(name)] [supports(…)] [media];`
    fn import(
        &mut self,
        rest: &str,
        at: usize,
        context: &Context,
        out: &mut Vec<Rule>,
    ) -> Result<(), Error> {
        if !self.imports_open {
            return Err((at, "write @import before the stylesheet's rules".into()));
        }
        let mut p = cssparser::Parser::new(rest);
        let path = match p.next() {
            Ok(cssparser::Token::QuotedString(s)) | Ok(cssparser::Token::UnquotedUrl(s)) => {
                s.to_string()
            }
            Ok(cssparser::Token::Function(f)) if f.eq_ignore_ascii_case("url") => p
                .parse_nested_block(|p| {
                    Ok::<_, cssparser::ParseError<()>>(p.expect_string()?.to_string())
                })
                .map_err(|_| (at, "@import url() takes a quoted path".into()))?,
            _ => return Err((at, "@import takes a path: @import \"theme.css\";".into())),
        };
        let mut inner = context.clone();
        let mut supported = true;
        loop {
            let state = p.state();
            match p.next() {
                Ok(cssparser::Token::Ident(w)) if w.eq_ignore_ascii_case("layer") => {
                    let layer = self.layer(context, None);
                    inner.layer = Some(layer);
                }
                Ok(cssparser::Token::Function(f)) if f.eq_ignore_ascii_case("layer") => {
                    let name = p
                        .parse_nested_block(|p| {
                            let start = p.position();
                            while p.next().is_ok() {}
                            Ok::<_, cssparser::ParseError<()>>(
                                p.slice_from(start).trim().to_string(),
                            )
                        })
                        .unwrap_or_default();
                    inner.layer = Some(self.layer(context, Some(&name)));
                }
                Ok(cssparser::Token::Function(f)) if f.eq_ignore_ascii_case("supports") => {
                    let condition = p
                        .parse_nested_block(|p| {
                            let start = p.position();
                            while p.next().is_ok() {}
                            Ok::<_, cssparser::ParseError<()>>(p.slice_from(start).to_string())
                        })
                        .unwrap_or_default();
                    let condition = if condition.trim_start().starts_with('(')
                        || condition.trim_start().starts_with("not")
                        || condition.trim_start().starts_with("selector(")
                    {
                        condition
                    } else {
                        format!("({condition})")
                    };
                    supported = supports(&condition).map_err(|e| (at, e))?;
                }
                _ => {
                    p.reset(&state);
                    break;
                }
            }
        }
        let start = p.position();
        while p.next().is_ok() {}
        let media = p.slice_from(start).trim();
        if !media.is_empty() {
            let parsed = super::media::Query::parse(media).map_err(|e| (at, e))?;
            inner.query.media = inner.query.media.clone().and(parsed);
        }
        let Some((_, text)) = self.sheet.imports.iter().find(|(p, _)| *p == path).cloned() else {
            return Err((
                at,
                format!(
                    "@import \"{path}\": the cook reads it from beside the document ({}); it was not supplied",
                    super::at_rules::resolve(&self.base, &path)
                ),
            ));
        };
        if !supported {
            return Ok(());
        }
        self.depth += 1;
        if self.depth > 16 {
            return Err((at, format!("@import \"{path}\" imports itself")));
        }
        // An imported sheet's offsets are its own; errors name the @import.
        let mut rules = Vec::new();
        let open = self.imports_open;
        self.imports_open = true;
        self.rules(&text, 0, &inner, &mut rules)
            .map_err(|(_, e)| (at, format!("in @import \"{path}\": {e}")))?;
        self.imports_open = open;
        self.depth -= 1;
        for mut rule in rules {
            rule.at = at;
            rule.body_at = at;
            rule.order = out.len();
            out.push(rule);
        }
        Ok(())
    }

    /// A block at-rule. `nested`: inside a style rule, where its contents
    /// are a body for the enclosing selector.
    #[allow(clippy::too_many_arguments)]
    fn group(
        &mut self,
        prelude: &str,
        block: &str,
        at: usize,
        block_at: usize,
        context: &Context,
        out: &mut Vec<Rule>,
        nested: bool,
    ) -> Result<(), Error> {
        let (name, rest) = at_rule(prelude).expect("an at-rule");
        let mut inner = context.clone();
        let descend = |reader: &mut Self, inner: &Context, out: &mut Vec<Rule>| {
            if nested {
                reader.body(block, at, block_at, inner, out)
            } else {
                reader.rules(block, block_at, inner, out)
            }
        };
        if !matches!(name.as_str(), "layer" | "import") {
            self.imports_open = false;
        }
        match name.as_str() {
            "media" => {
                let parsed = super::media::Query::parse(rest).map_err(|error| (at, error))?;
                inner.query.media = context.query.media.clone().and(parsed);
                descend(self, &inner, out)
            }
            "container" => {
                let condition =
                    super::container::Condition::parse(rest).map_err(|error| (at, error))?;
                inner.query.container.push(condition);
                descend(self, &inner, out)
            }
            "supports" => {
                if supports(rest).map_err(|e| (at, e))? {
                    descend(self, &inner, out)
                } else {
                    Ok(())
                }
            }
            "layer" => {
                let name = (!rest.is_empty()).then_some(rest);
                if let Some(name) = name
                    && (name.contains(',') || name.split('.').any(|p| !syntax::identifier(p)))
                {
                    return Err((at, format!("@layer {name} {{ … }} takes one layer name")));
                }
                inner.layer = Some(self.layer(context, name));
                descend(self, &inner, out)
            }
            "starting-style" => {
                inner.starting = true;
                descend(self, &inner, out)
            }
            "scope" => {
                let (root, limit) = scope_prelude(rest).map_err(|e| (at, e))?;
                let root = match (root, &context.parent) {
                    (Some(root), _) => resolve_nested(&root, context.parent.as_deref()),
                    (None, Some(parent)) => parent.clone(),
                    (None, None) => ":root".into(),
                };
                inner.scope = Some((root, limit));
                inner.parent = None;
                if nested {
                    // Its contents are rules relative to the new root.
                    self.rules(block, block_at, &inner, out)
                } else {
                    self.rules(block, block_at, &inner, out)
                }
            }
            "font-face" if !nested => self.sheet.font_face(block, &self.base).map_err(|e| (at, e)),
            "property" if !nested => self.sheet.property(rest, block).map_err(|e| (at, e)),
            "counter-style" if !nested => {
                self.sheet.counter_style(rest, block).map_err(|e| (at, e))
            }
            "position-try" if !nested => self.sheet.position_try(rest, block).map_err(|e| (at, e)),
            "keyframes" | "view-transition" if !nested => {
                out.push(Rule {
                    selector: prelude.to_string(),
                    body: block.into(),
                    query: context.query.clone(),
                    at,
                    body_at: block_at,
                    order: out.len(),
                    ua: false,
                    layer: context.layer.clone(),
                    starting: false,
                });
                Ok(())
            }
            "page" => Err((
                at,
                "@page is print media, which documents do not have".into(),
            )),
            other => Err((
                at,
                format!(
                    "unsupported at-rule @{other}{}",
                    if nested { " inside a style rule" } else { "" }
                ),
            )),
        }
    }

    /// A style rule: its selector resolved against the nesting parent and
    /// the scope, then its body.
    fn style_rule(
        &mut self,
        prelude: &str,
        block: &str,
        at: usize,
        block_at: usize,
        context: &Context,
        out: &mut Vec<Rule>,
    ) -> Result<(), Error> {
        let mut selector = resolve_nested(prelude, context.parent.as_deref());
        if context.parent.is_none()
            && let Some((root, limit)) = &context.scope
        {
            selector = scoped(&selector, root, limit.as_deref());
        }
        let mut inner = context.clone();
        inner.parent = Some(selector);
        inner.scope = None;
        self.body(block, at, block_at, &inner, out)
    }

    /// A style rule's body: its declarations, as one rule for the parent
    /// selector, and its nested rules after it.
    fn body(
        &mut self,
        text: &str,
        at: usize,
        body_at: usize,
        context: &Context,
        out: &mut Vec<Rule>,
    ) -> Result<(), Error> {
        let selector = context
            .parent
            .clone()
            .expect("a body has a parent selector");
        let mut declarations = String::new();
        let mut nested = Vec::new();
        for item in items(text, body_at)? {
            match item {
                Item::Statement(d, _) => {
                    declarations.push_str(d);
                    declarations.push(';');
                }
                block => nested.push(block),
            }
        }
        if !declarations.trim().is_empty() || nested.is_empty() {
            out.push(Rule {
                selector: selector.clone(),
                body: declarations,
                query: context.query.clone(),
                at,
                body_at,
                order: out.len(),
                ua: false,
                layer: context.layer.clone(),
                starting: context.starting,
            });
        }
        for item in nested {
            let Item::Block {
                prelude,
                block,
                at,
                block_at,
            } = item
            else {
                unreachable!()
            };
            if prelude.starts_with('@') {
                self.group(prelude, block, at, block_at, context, out, true)?;
            } else {
                self.style_rule(prelude, block, at, block_at, context, out)?;
            }
        }
        Ok(())
    }
}

/// Splits a selector list at its top-level commas.
fn split_list(list: &str) -> Vec<&str> {
    let (mut depth, mut quote, mut start) = (0, None, 0);
    let mut out = Vec::new();
    for (i, c) in list.char_indices() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '\'' | '"') => quote = Some(c),
            (None, '(' | '[') => depth += 1,
            (None, ')' | ']') => depth -= 1,
            (None, ',') if depth == 0 => {
                out.push(list[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(list[start..].trim());
    out
}

/// A nested rule's selector list in terms of its parent's (CSS Nesting):
/// `&` is the parent; a selector without one is relative to it. A parent
/// of one compound is written in place (`&:hover` → `.a:hover`), any other
/// as `:is(parent)`, whose specificity is the list's highest.
fn resolve_nested(list: &str, parent: Option<&str>) -> String {
    let Some(parent) = parent else {
        return list.to_string();
    };
    let simple = !parent.contains(',')
        && Selector::parse_list(parent).is_ok_and(|s| s.len() == 1 && s[0].compounds.len() == 1);
    let replacement = if simple {
        parent.to_string()
    } else {
        format!(":is({parent})")
    };
    split_list(list)
        .into_iter()
        .map(|s| {
            if s.contains('&') {
                s.replace('&', &replacement)
            } else {
                format!("{replacement} {s}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// A selector inside `@scope (root) to (limit)`: matched in the root's
/// subtree (or at the root through `:scope`), above any limit.
fn scoped(list: &str, root: &str, limit: Option<&str>) -> String {
    let root = format!(":where({root})");
    split_list(list)
        .into_iter()
        .map(|s| {
            let s = if s.contains(":scope") || s.contains('&') {
                s.replace(":scope", &root).replace('&', &root)
            } else {
                format!("{root} {s}")
            };
            match limit {
                Some(limit) => format!("{s}:not(:where({limit}) *, :where({limit}))"),
                None => s,
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// `(root) to (limit)`, either part optional.
fn scope_prelude(rest: &str) -> Result<(Option<String>, Option<String>), String> {
    let mut p = cssparser::Parser::new(rest);
    let group = |p: &mut cssparser::Parser<'_>| -> Result<String, String> {
        p.expect_parenthesis_block()
            .map_err(|_| "@scope takes (root) to (limit)".to_string())?;
        p.parse_nested_block(|p| {
            let start = p.position();
            while p.next().is_ok() {}
            Ok::<_, cssparser::ParseError<()>>(p.slice_from(start).trim().to_string())
        })
        .map_err(|_| "@scope takes (root) to (limit)".to_string())
    };
    let root = if p.is_exhausted() || p.try_parse(|p| p.expect_ident_matching("to")).is_ok() {
        None
    } else {
        Some(group(&mut p)?)
    };
    let limit = if p.is_exhausted() {
        None
    } else {
        if root.is_some() {
            p.expect_ident_matching("to")
                .map_err(|_| "@scope takes (root) to (limit)".to_string())?;
        }
        Some(group(&mut p)?)
    };
    for s in root.iter().chain(&limit) {
        Selector::parse_list(s)?;
    }
    Ok((root, limit))
}

/// An `@supports` condition (CSS Conditional 3 and 4): `(property: value)`
/// holds when the document's cascade takes the declaration as written,
/// `selector(…)` when it takes the selector; `not`, `and`, `or` combine.
pub(super) fn supports(condition: &str) -> Result<bool, String> {
    use cssparser::{ParseError, Parser, Token};
    fn test(p: &mut Parser<'_>) -> Result<bool, ParseError<String>> {
        if p.try_parse(|p| p.expect_ident_matching("not")).is_ok() {
            return Ok(!test_group(p)?);
        }
        let first = test_group(p)?;
        let mut result = first;
        let mut operator: Option<String> = None;
        while !p.is_exhausted() {
            let word = p.expect_ident_cloned()?.to_ascii_lowercase();
            if word != "and" && word != "or" {
                return Err(ParseError::custom(format!(
                    "expected and or or, found {word}"
                )));
            }
            if operator.as_ref().is_some_and(|o| *o != word) {
                return Err(ParseError::custom(
                    "group mixed and/or in parentheses".to_string(),
                ));
            }
            let next = test_group(p)?;
            result = if word == "and" {
                result && next
            } else {
                result || next
            };
            operator = Some(word);
        }
        Ok(result)
    }
    fn test_group(p: &mut Parser<'_>) -> Result<bool, ParseError<String>> {
        let token = p.next()?.clone();
        match token {
            Token::ParenthesisBlock => p.parse_nested_block(|p| {
                let state = p.state();
                if matches!(p.next(), Ok(Token::ParenthesisBlock)) || {
                    p.reset(&state);
                    p.try_parse(|p| p.expect_ident_matching("not")).is_ok()
                } {
                    p.reset(&state);
                    return test(p);
                }
                p.reset(&state);
                let Ok(name) = p.expect_ident_cloned() else {
                    while p.next().is_ok() {}
                    return Ok(false);
                };
                if p.expect_colon().is_err() {
                    while p.next().is_ok() {}
                    return Ok(false);
                }
                let start = p.position();
                while p.next().is_ok() {}
                let value = p.slice_from(start).trim().to_string();
                Ok(declaration_supported(&name, &value))
            }),
            Token::Function(name) => {
                let name = name.to_ascii_lowercase();
                p.parse_nested_block(|p| {
                    let start = p.position();
                    while p.next().is_ok() {}
                    let arg = p.slice_from(start).trim().to_string();
                    Ok(match name.as_str() {
                        "selector" => Selector::parse_list(&arg).is_ok(),
                        "font-format" => matches!(
                            arg.trim_matches(['"', '\'']),
                            "truetype" | "opentype" | "woff" | "woff2" | "collection"
                        ),
                        "font-tech" => matches!(
                            arg.as_str(),
                            "color-colrv0" | "color-colrv1" | "variations" | "features-opentype"
                        ),
                        _ => false,
                    })
                })
            }
            _ => Err(ParseError::custom(
                "@supports takes (property: value), selector(…), not, and and or".to_string(),
            )),
        }
    }
    Parser::new(condition)
        .parse_entirely(test)
        .map_err(|e| format!("invalid @supports condition {condition:?}: {:?}", e.kind))
}

/// Whether the cascade takes `name: value` as written.
fn declaration_supported(name: &str, value: &str) -> bool {
    if name.starts_with("--") {
        return true;
    }
    lower(&format!("{name}: {value};"), &|_| false).is_ok()
}

/// A rule body's declarations, expanded to the longhands the cascade
/// keeps, and the text of those it can check at cook (no `var()`, no
/// CSS-wide keyword). `keyframes`: whether a name is a defined
/// `@keyframes`.
pub(super) fn lower(
    text: &str,
    keyframes: &dyn Fn(&str) -> bool,
) -> Result<(Vec<StyleDeclaration>, String), String> {
    let mut declarations = Vec::new();
    for declaration in variables::declarations(text)? {
        if !declaration.name.starts_with("--") && wide_keyword(&declaration.source).is_some() {
            // A CSS-wide keyword applies to each longhand of a shorthand.
            let longhands: Vec<String> = match declaration.name.as_str() {
                "transition" => super::motion::TRANSITION
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                "animation" => super::motion::ANIMATION
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                "container" => super::container::LONGHANDS
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                "list-style" => vec![
                    "--og-list-style-type".into(),
                    "--og-list-style-position".into(),
                ],
                "list-style-type" => vec!["--og-list-style-type".into()],
                "list-style-position" => vec!["--og-list-style-position".into()],
                name => vec![name.to_string()],
            };
            for name in longhands {
                declarations.push(StyleDeclaration {
                    name,
                    ..declaration.clone()
                });
            }
            continue;
        }
        for declaration in super::container::expand(declaration).and_then(|ds| {
            let mut out = Vec::new();
            for d in ds {
                for d in super::lists::expand(d)? {
                    out.extend(super::text_css::expand(d)?);
                }
            }
            Ok(out)
        })? {
            declarations.extend(super::motion::expand(declaration, keyframes)?);
        }
    }
    let mut body = String::new();
    for declaration in &declarations {
        let name = declaration.name.as_str();
        if name.starts_with("--")
            || super::motion::longhand(name)
            || super::container::longhand(name)
            || super::extern_element::svg_longhand(name)
            || super::text_css::longhand(name)
        {
            continue;
        }
        if crate::transform3d::LONGHANDS.contains(&name) {
            if wide_keyword(&declaration.source).is_none() && !declaration.value.has_variables() {
                let mut errors = Vec::new();
                if let Ok(value) = declaration.value.substitute(&mut |_| None)
                    && !super::calc::has_absolute(&value)
                {
                    let value = super::calc::reduce(&value)?;
                    crate::transform3d::resolve(&[(name.into(), value)], &mut errors);
                }
                if let Some(e) = errors.into_iter().next() {
                    return Err(e);
                }
            }
            continue;
        }
        if css::targets(name).next().is_none() {
            return Err(format!(
                "unsupported CSS property {name}: {}",
                css::unsupported(name)
                    .unwrap_or("use a property listed in the document CSS reference")
            ));
        }
        if wide_keyword(&declaration.source).is_some() || declaration.value.has_variables() {
            continue;
        }
        let value = declaration.value.substitute(&mut |_| None)?;
        // Viewport, root and font units, env() and attr() resolve when
        // the element does.
        if !super::calc::per_element(name, &value) {
            let value = super::calc::reduce(&value)?;
            body.push_str(&format!("{name}: {value};"));
        }
    }
    // The values the cascade takes as written (`@supports` asks this).
    css::declarations(&body, 0)?;
    Ok((declarations, body))
}

fn matches(
    doc: &UiDocument,
    elements: &[(usize, String, Vec<Attr>)],
    index: usize,
    selector: &str,
    at: usize,
) -> Result<Option<Expr>, Error> {
    let parsed = Selector::parse_list(selector).map_err(|e| (at, e))?;
    let condition = parsed[0]
        .condition(doc, elements, index)
        .map_err(|e| (at, e))?;
    Ok(match condition {
        Expr::Lit(serde_json::Value::Bool(false)) => None,
        condition => Some(condition),
    })
}

/// The pseudo-elements a control draws itself, and the declarations each
/// takes as the control's own property: `dialog::backdrop { background }`
/// (a modal's scrim) and `select::picker(select) { max-height }` (the
/// open list, CSS's customizable select).
const PSEUDO: &[(&str, &[(&str, &str)])] = &[
    ("::backdrop", &[("background", "-og-backdrop")]),
    (
        "::picker(select)",
        &[("max-height", "-og-picker-max-height")],
    ),
];

/// A rule on a pseudo-element of [`PSEUDO`] becomes the element's own
/// declarations, so it cascades (states, variables, media) as any does.
fn backdrop(rule: &mut Rule) -> Result<(), (usize, String)> {
    let selectors: Vec<&str> = rule.selector.split(',').map(str::trim).collect();
    let Some(&(pseudo, takes)) = PSEUDO
        .iter()
        .find(|(p, _)| selectors.iter().any(|s| s.ends_with(p)))
    else {
        return Ok(());
    };
    if !selectors.iter().all(|s| s.ends_with(pseudo)) {
        return Err((
            rule.at,
            format!(
                "{}: write {pseudo} rules apart from the element's own",
                rule.selector
            ),
        ));
    }
    let mut body = String::new();
    for d in variables::declarations(&rule.body).map_err(|e| (rule.at, e))? {
        let Some((_, own)) = takes.iter().find(|(css, _)| *css == d.name) else {
            return Err((
                rule.at,
                format!(
                    "{pseudo} takes {}, not {}",
                    takes
                        .iter()
                        .map(|(css, _)| *css)
                        .collect::<Vec<_>>()
                        .join(", "),
                    d.name
                ),
            ));
        };
        body.push_str(&format!(
            "{own}: {}{};",
            d.source,
            if d.important { " !important" } else { "" }
        ));
    }
    rule.selector = selectors
        .iter()
        .map(|s| s.trim_end_matches(pseudo))
        .collect::<Vec<_>>()
        .join(", ");
    rule.body = body;
    Ok(())
}

/// An element's own declarations (its `style` attribute or a `style:`
/// directive, `body` at `at`), expanded and checked as a rule's are.
pub(super) fn inline(
    doc: &UiDocument,
    node: usize,
    body: &str,
    at: usize,
) -> Result<Vec<StyleDeclaration>, Error> {
    let known: Vec<String> = doc.keyframe_rules.iter().map(|(n, ..)| n.clone()).collect();
    let mut out = Vec::new();
    for declaration in variables::declarations(body).map_err(|e| (at, e))? {
        for d in super::container::expand(declaration)
            .and_then(|ds| {
                let mut v = Vec::new();
                for d in ds {
                    v.extend(super::lists::expand(d)?);
                }
                Ok(v)
            })
            .map_err(|e| (at, e))?
        {
            out.extend(
                super::motion::expand(d, &|name| known.iter().any(|k| k == name))
                    .map_err(|e| (at, e))?,
            );
        }
    }
    let kind = match doc.nodes[node].kind {
        NodeKind::Element(k) => k,
        _ => "column",
    };
    for d in &out {
        let name = d.name.as_str();
        if name.starts_with("--")
            || super::motion::longhand(name)
            || super::container::longhand(name)
            || super::extern_element::svg_longhand(name)
        {
            continue;
        }
        if css::targets(name).next().is_none() {
            return Err((
                at,
                format!(
                    "unsupported CSS property {name}: {}",
                    css::unsupported(name)
                        .unwrap_or("use a property listed in the document CSS reference")
                ),
            ));
        }
        if d.value.has_variables() {
            continue;
        }
        let value = d.value.substitute(&mut |_| None).map_err(|e| (at, e))?;
        if super::calc::has_absolute(&value) {
            continue;
        }
        let value = super::calc::reduce(&value).map_err(|e| (at, e))?;
        for e in css::declarations(&format!("{name}: {value};"), 0).map_err(|e| (at, e))? {
            let applies = schema::prop(&e.key).is_some_and(|(_, p)| p.applies(kind));
            let side = (e.key == "direction" && kind == "toggle")
                || (e.key == "labelDirection" && kind != "toggle");
            if !applies && !side {
                return Err((at, format!("CSS property {name} does not apply to {kind}")));
            }
        }
    }
    Ok(out)
}
