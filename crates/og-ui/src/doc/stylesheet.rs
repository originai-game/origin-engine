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
    /// HTML's user-agent stylesheet ([`super::lists::UA_CSS`]): cascades
    /// before every author rule, whatever their specificity.
    ua: bool,
}

pub(super) fn apply(
    doc: &mut UiDocument,
    elements: &[(usize, String, Vec<Attr>)],
    styles: &[(String, usize)],
) -> Result<(), Error> {
    let mut rules = Vec::new();
    read_rules(super::lists::UA_CSS, 0, AtQuery::default(), &mut rules)?;
    for rule in &mut rules {
        // Its offsets are in the UA text, not the document's.
        rule.ua = true;
        rule.at = 0;
        rule.body_at = 0;
    }
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
    parsed.sort_by_key(|(selector, rule)| (!rule.ua, selector.specificity(), rule.order));
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
        let mut declarations = Vec::new();
        for declaration in variables::declarations(&rule.body).map_err(|e| (rule.at, e))? {
            for declaration in super::container::expand(declaration)
                .and_then(|ds| {
                    let mut out = Vec::new();
                    for d in ds {
                        for d in super::lists::expand(d)? {
                            out.extend(super::text_css::expand(d)?);
                        }
                    }
                    Ok(out)
                })
                .map_err(|e| (rule.at, e))?
            {
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
                || super::extern_element::svg_longhand(name)
                || super::text_css::longhand(name)
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
                // Viewport units and rem resolve when the element does.
                if !super::calc::has_absolute(&value) {
                    let value = super::calc::reduce(&value).map_err(|error| (rule.at, error))?;
                    body.push_str(&format!("{name}: {value};"));
                }
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
                ua: false,
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
