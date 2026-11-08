//! Scoped selectors lower once to matching document nodes and reactive predicates.
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
}

pub(super) fn apply(
    doc: &mut UiDocument,
    elements: &[(usize, String, Vec<Attr>)],
    styles: &[(String, usize)],
) -> Result<(), Error> {
    let mut rules = Vec::new();
    for (text, at) in styles {
        read_rules(text, *at, AtQuery::default(), &mut rules)?;
    }
    for rule in &rules {
        if let Some(name) = rule.selector.strip_prefix("@keyframes ") {
            let name = name.trim();
            if !syntax::identifier(name) {
                return Err((rule.at, "@keyframes needs a CSS identifier".into()));
            }
            let mut frames = Vec::new();
            read_rules(&rule.body, rule.body_at, AtQuery::default(), &mut frames)?;
            let frames: Vec<_> = frames.into_iter().map(|f| (f.selector, f.body)).collect();
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
    rules.retain(|rule| !rule.selector.starts_with("@keyframes "));
    let mut parsed = Vec::new();
    for rule in rules {
        for selector in Selector::parse_list(&rule.selector).map_err(|e| (rule.at, e))? {
            let mut rule = rule.clone();
            rule.selector = selector.source.clone();
            parsed.push((selector, rule));
        }
    }
    parsed.sort_by_key(|(selector, rule)| (selector.specificity(), rule.order));
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
                if tag != "text" {
                    return Err((
                        rule.at,
                        "text spans require a text::part(name) selector".into(),
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
        let mut declarations = Vec::new();
        for declaration in variables::declarations(&rule.body).map_err(|e| (rule.at, e))? {
            for declaration in super::container::expand(declaration).map_err(|e| (rule.at, e))? {
                declarations.extend(
                    super::motion::expand(declaration, &|name| known.iter().any(|k| k == name))
                        .map_err(|e| (rule.at, e))?,
                );
            }
        }
        let mut body = String::new();
        for declaration in &declarations {
            let name = declaration.name.as_str();
            if name.starts_with("--")
                || super::motion::longhand(name)
                || super::container::longhand(name)
            {
                continue;
            }
            if css::targets(name).next().is_none() {
                return Err((
                    rule.at,
                    format!(
                        "unsupported CSS property {name}: {}",
                        css::unsupported(name)
                            .unwrap_or("use a property listed in the document CSS reference")
                    ),
                ));
            }
            if !declaration.value.has_variables() {
                let value = declaration
                    .value
                    .substitute(&mut |_| None)
                    .map_err(|error| (rule.at, error))?;
                body.push_str(&format!("{name}: {value};"));
            }
        }
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
                let (spec, p) = schema::prop(&e.key)
                    .ok_or((rule.at, format!("unknown CSS target {}", e.key)))?;
                if !p.applies(kind) {
                    return Err((
                        rule.at,
                        format!("CSS property {} does not apply to {kind}", e.key),
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

fn read_rules(text: &str, base: usize, query: AtQuery, out: &mut Vec<Rule>) -> Result<(), Error> {
    let mut at = 0;
    while at < text.len() {
        while text[at..].starts_with(char::is_whitespace) {
            at += text[at..].chars().next().unwrap().len_utf8();
        }
        if at == text.len() {
            break;
        }
        if text[at..].starts_with("/*") {
            let end = text[at + 2..]
                .find("*/")
                .ok_or((base + at, "close CSS comment with */".into()))?;
            at += end + 4;
            continue;
        }
        let start = at;
        let open = text[at..]
            .find('{')
            .ok_or((base + at, "a CSS rule needs { declarations }".into()))?
            + at;
        let selector = text[at..open].trim();
        at = open + 1;
        let mut depth = 1;
        let mut quote = None;
        while at < text.len() && depth > 0 {
            let c = text[at..].chars().next().unwrap();
            at += c.len_utf8();
            if let Some(q) = quote {
                if c == '\\' && at < text.len() {
                    at += text[at..].chars().next().unwrap().len_utf8();
                } else if c == q {
                    quote = None;
                }
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
        let body = &text[open + 1..at - 1];
        if let Some(media) = selector.get(6..).filter(|rest| {
            selector[..6].eq_ignore_ascii_case("@media")
                && (rest.starts_with(char::is_whitespace) || rest.starts_with('('))
        }) {
            let parsed =
                super::media::Query::parse(media).map_err(|error| (base + start, error))?;
            let q = AtQuery {
                media: query.media.clone().and(parsed),
                container: query.container.clone(),
            };
            read_rules(body, base + open + 1, q, out)?;
        } else if let Some(prelude) = selector.get(10..).filter(|rest| {
            selector[..10].eq_ignore_ascii_case("@container")
                && (rest.starts_with(char::is_whitespace) || rest.starts_with('('))
        }) {
            let condition = super::container::Condition::parse(prelude)
                .map_err(|error| (base + start, error))?;
            let mut q = query.clone();
            q.container.push(condition);
            read_rules(body, base + open + 1, q, out)?;
        } else {
            out.push(Rule {
                selector: selector.trim().into(),
                body: body.into(),
                query: query.clone(),
                at: base + start,
                body_at: base + open + 1,
                order: out.len(),
            });
        }
    }
    Ok(())
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
