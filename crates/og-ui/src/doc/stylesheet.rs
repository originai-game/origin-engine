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
    let mut keyframes = std::collections::BTreeMap::new();
    for rule in &rules {
        if let Some(name) = rule.selector.strip_prefix("@keyframes ") {
            if !syntax::identifier(name.trim()) {
                return Err((rule.at, "@keyframes needs a CSS identifier".into()));
            }
            keyframes.insert(name.trim().to_string(), (rule.body.clone(), rule.body_at));
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
    for (_, rule) in parsed {
        if let Some((parent, selector)) = rule.selector.split_once("::part(") {
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
        let declarations = variables::declarations(&rule.body).map_err(|e| (rule.at, e))?;
        let animation = declarations
            .iter()
            .rev()
            .find(|decl| decl.name == "animation")
            .map(|decl| animation(&decl.source, &keyframes, rule.at))
            .transpose()?;
        let declarations: Vec<_> = declarations
            .into_iter()
            .filter(|d| d.name != "animation")
            .collect();
        let mut body = String::new();
        for declaration in &declarations {
            let name = declaration.name.as_str();
            if name.starts_with("--") {
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
            let Some(condition) = matches(doc, elements, *i, &rule.selector, rule.at)? else {
                continue;
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
            if let Some(timeline) = &animation {
                if condition.is_some() || rule.query != AtQuery::default() {
                    return Err((rule.at, "conditional animations are not supported; put the animation on a mounted element and control its presence with #if".into()));
                }
                let name = format!("css-animation-{}", i);
                doc.timelines.retain(|(n, _)| *n != name);
                doc.timelines.push((name.clone(), timeline.clone()));
                doc.nodes[*i]
                    .triggers
                    .retain(|t| !matches!(&t.act, Act::Play(n) if n == &name));
                doc.nodes[*i].triggers.push(Trigger {
                    when: When::Appear,
                    act: Act::Play(name),
                    line: syntax::location(&doc.text, rule.at).0,
                });
            }
            doc.nodes[*i].rules.push(StyleRule {
                query: rule.query.clone(),
                condition,
                props,
                declarations: declarations.clone(),
                line: syntax::location(&doc.text, rule.at).0,
            });
        }
    }
    Ok(())
}

fn animation(
    value: &str,
    definitions: &std::collections::BTreeMap<String, (String, usize)>,
    at: usize,
) -> Result<Timeline, Error> {
    let mut words = value.split_whitespace();
    let name = words.next().unwrap_or_default();
    let duration = words.next().unwrap_or_default();
    let ms = duration
        .strip_suffix("ms")
        .and_then(|n| n.parse::<f32>().ok())
        .or_else(|| {
            duration
                .strip_suffix('s')
                .and_then(|n| n.parse::<f32>().ok())
                .map(|s| s * 1000.0)
        })
        .filter(|n| n.is_finite() && *n > 0.0)
        .ok_or((
            at,
            "animation: <name> <positive duration in ms or s> <CSS easing> forwards".into(),
        ))?;
    let rest = words.collect::<Vec<_>>().join(" ");
    let easing = rest.strip_suffix(" forwards").ok_or((at, "animations currently require forwards fill; write animation: <name> <duration> <CSS easing> forwards".into()))?;
    let ease = Ease::from_css(easing).ok_or((
        at,
        "use a CSS easing keyword, cubic-bezier(x1,y1,x2,y2) or steps(count,position)".into(),
    ))?;
    let (body, base) = definitions.get(name).ok_or((
        at,
        format!("declare @keyframes {name} {{ … }} in this document"),
    ))?;
    let mut frames = Vec::new();
    read_rules(body, *base, AtQuery::default(), &mut frames)?;
    let mut timeline = Timeline {
        duration_ms: ms,
        ..Default::default()
    };
    for frame in frames.into_iter().flat_map(|frame| {
        frame
            .selector
            .split(',')
            .map(|selector| Rule {
                selector: selector.trim().into(),
                ..frame.clone()
            })
            .collect::<Vec<_>>()
    }) {
        let offset = match frame.selector.as_str() {
            "from" => Some(0.0),
            "to" => Some(1.0),
            other => other
                .strip_suffix('%')
                .and_then(|n| n.trim().parse::<f32>().ok())
                .map(|n| n / 100.0),
        }
        .filter(|n| n.is_finite() && (0.0..=1.0).contains(n))
        .ok_or((
            frame.at,
            "keyframe selectors are from, to or percentages from 0% to 100%".into(),
        ))?;
        let declarations = css::split_decls(&frame.body).map_err(|e| (frame.at, e))?;
        let mut body = String::new();
        for (name, value) in declarations {
            if name == "-og-sound" {
                if !syntax::identifier(&value) {
                    return Err((
                        frame.at,
                        "-og-sound names a style sound, for example error".into(),
                    ));
                }
                timeline.events.push(crate::motion::TimelineEvent {
                    at_ms: offset * ms,
                    target: String::new(),
                    kind: crate::motion::EventKind::Sound {
                        name: value,
                        pitch: 1.0,
                    },
                });
            } else {
                body.push_str(&format!("{name}: {value};"));
            }
        }
        for entry in css::declarations(&body, 0).map_err(|e| (frame.at, e))? {
            let prop = Prop::from_name(&entry.key).ok_or((
                frame.at,
                format!(
                    "{} cannot be animated; use opacity, scale, rotate or translate",
                    entry.key
                ),
            ))?;
            let number = entry
                .values
                .first()
                .and_then(Value::as_number)
                .map(Number::as_f32)
                .filter(|v| v.is_finite())
                .ok_or((frame.at, "keyframes require finite scalar values".into()))?;
            let index = timeline
                .tracks
                .iter()
                .position(|t| t.prop == prop)
                .unwrap_or_else(|| {
                    timeline.tracks.push(TimelineTrack {
                        target: String::new(),
                        prop,
                        keys: Vec::new(),
                    });
                    timeline.tracks.len() - 1
                });
            let keys = &mut timeline.tracks[index].keys;
            keys.retain(|key| key.at_ms != offset * ms);
            keys.push(Key {
                at_ms: offset * ms,
                value: number,
                ease: KeyEase::Ease(ease),
            });
        }
    }
    for track in &mut timeline.tracks {
        track.keys.sort_by(|a, b| a.at_ms.total_cmp(&b.at_ms));
        if track.keys.first().is_none_or(|k| k.at_ms != 0.0)
            || track.keys.last().is_none_or(|k| k.at_ms != ms)
        {
            return Err((
                at,
                "each animated property needs explicit from/0% and to/100% keyframes".into(),
            ));
        }
    }
    if timeline.tracks.is_empty() {
        return Err((at, "@keyframes must animate at least one property".into()));
    }
    Ok(timeline)
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
        if let Some(media) = selector.strip_prefix("@media ") {
            let mut q = query.clone();
            for name in media.split_whitespace() {
                match name {
                    "phone" => q.breakpoints.extend([
                        crate::style::Breakpoint::PhonePortrait,
                        crate::style::Breakpoint::PhoneLandscape,
                    ]),
                    "phonePortrait" => q.breakpoints.push(crate::style::Breakpoint::PhonePortrait),
                    "phoneLandscape" => {
                        q.breakpoints.push(crate::style::Breakpoint::PhoneLandscape)
                    }
                    "tablet" => q.breakpoints.push(crate::style::Breakpoint::Tablet),
                    "desktop" => q.breakpoints.push(crate::style::Breakpoint::Desktop),
                    "portrait" => q.orientation = Some(crate::element::Orientation::Portrait),
                    "landscape" => q.orientation = Some(crate::element::Orientation::Landscape),
                    _ => {
                        return Err((
                            base + start,
                            "unsupported media query; use a declared breakpoint".into(),
                        ));
                    }
                }
            }
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
