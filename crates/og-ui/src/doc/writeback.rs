use super::syntax::Node;
use super::*;

fn find(nodes: &[Node], offset: usize) -> Option<&Node> {
    for node in nodes {
        let found = match node {
            Node::Element { at, .. } if *at == offset => Some(node),
            Node::Element { children, .. } => find(children, offset),
            Node::If { yes, no, .. } => find(yes, offset).or_else(|| find(no, offset)),
            Node::Each { body, empty, .. } => find(body, offset).or_else(|| find(empty, offset)),
            Node::Snippet { body, .. } => find(body, offset),
            _ => None,
        };
        if found.is_some() {
            return found;
        }
    }
    None
}

fn escaped(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('"', "&quot;")
        .replace('{', "&#123;")
        .replace('}', "&#125;")
}

pub(super) fn edit(
    doc: &UiDocument,
    node: usize,
    prop: &str,
    value: &str,
) -> Result<String, String> {
    let target = doc.nodes.get(node).ok_or("no such node")?;
    let (_, spec) = schema::prop(prop).ok_or_else(|| format!("'{prop}' is not a prop"))?;
    let nodes =
        syntax::parse(&doc.text).map_err(|e| syntax::diagnostic(&doc.source, &doc.text, e))?;
    let Some(Node::Element {
        tag,
        attrs,
        children,
        end,
        close,
        ..
    }) = find(&nodes, target.source_offset)
    else {
        return Err("edit a core element, not a structural block".into());
    };
    let value = value_text(spec.ty, value);
    let mut text = doc.text.clone();
    if css::css_name(prop).is_some() {
        if target.id.is_empty() {
            return Err("add a literal id to the element before saving a scoped CSS edit".into());
        }
        let vals = |source: &str| -> Result<Vec<Value>, String> {
            match og_core::ogt::parse_line(&format!("v {source}"), 0) {
                Ok(og_core::ogt::Line::Entry { entry, .. }) => Ok(entry.values),
                _ => Err(format!("invalid value for {prop}")),
            }
        };
        let own = |name: &str| {
            target
                .props
                .iter()
                .chain(
                    target
                        .rules
                        .iter()
                        .filter(|rule| rule.condition.is_none())
                        .flat_map(|rule| &rule.props),
                )
                .rev()
                .find(|p| p.name() == name)
                .and_then(|p| vals(&p.source).ok())
        };
        let declaration = css::decl_line(&[(prop.into(), vals(&value)?)], false, &own)?;
        // A new id rule changes only this element, never a shared class rule.
        let rule = format!("\n#{} {{ {declaration} }}\n", target.id);
        if let Some(Node::Element { close, .. }) = nodes
            .iter()
            .find(|n| matches!(n, Node::Element { tag, .. } if tag == "style"))
        {
            text.insert_str(*close, &rule);
        } else {
            text.push_str(&format!("\n<style>{rule}</style>\n"));
        }
        return Ok(text);
    }
    let literal = serde_json::from_str::<String>(&value).unwrap_or(value);
    if prop == "text" {
        if children
            .iter()
            .any(|child| !matches!(child, Node::Text(..) | Node::Expression(..)))
        {
            return Err("edit a text-only element; preserve its structural children".into());
        }
        if end == close && text[..*end].ends_with("/>") {
            text.replace_range(end - 2..*end, &format!(">{}</{tag}>", escaped(&literal)));
        } else {
            text.replace_range(*end..*close, &escaped(&literal));
        }
        return Ok(text);
    }
    let name = match prop {
        "stepBy" => "step",
        "key" => "i18n",
        "name" => "aria-label",
        "texture" => "src",
        "tabIndex" => "tabindex",
        "input" if matches!(tag.as_str(), "input" | "textarea") => "value",
        "current" if tag == "select" => "value",
        "maxChars" => "maxlength",
        "minChars" => "minlength",
        "lines" => "rows",
        _ => prop,
    };
    let attribute = format!("{name}=\"{}\"", escaped(&literal));
    if let Some(attr) = attrs.iter().find(|a| {
        let name = a.name.strip_prefix("bind:").unwrap_or(&a.name);
        svelte::property(name) == prop
            || (prop == "input" && matches!(tag.as_str(), "input" | "textarea") && name == "value")
            || (prop == "current" && tag == "select" && name == "value")
            || matches!(
                (prop, name),
                ("maxChars", "maxlength") | ("minChars", "minlength") | ("lines", "rows")
            )
    }) {
        text.replace_range(attr.at..attr.end, &attribute);
    } else {
        let insert = end - if text[..*end].ends_with("/>") { 2 } else { 1 };
        text.insert_str(insert, &format!(" {attribute}"));
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_saves_change_only_targeted_markup_and_scope_css_to_one_element() {
        let source = r#"<meta name="screen" content="edit">
<!-- keep this comment -->
<column id="Root">
  <text id="Title">Before</text><text id="Other">Other</text>
  <input type="range" id="Amount" bind:value={$amount} step="0.25"/>
</column>
<style>text { opacity: 0.4; } #Title { translate: 3px 7px; }</style>
"#;
        let mut doc = UiDocument::parse("edit.svelte", source).unwrap();
        let title = doc.node_at("Root/Title").unwrap();
        doc.set_prop(title, "text", "Après & {literal}").unwrap();
        assert_eq!(
            doc.text,
            source.replace("Before", "Après &amp; &#123;literal&#125;")
        );
        let amount = doc.node_at("Root/Amount").unwrap();
        doc.set_prop(amount, "value", "0.75").unwrap();
        assert!(doc.text.contains("value=\"0.75\" step=\"0.25\""));
        assert!(!doc.text.contains("bind:value"));
        doc.set_prop(title, "x", "11").unwrap();
        let value = |name| {
            doc.nodes[title]
                .rules
                .iter()
                .flat_map(|rule| &rule.props)
                .rev()
                .find(|p| p.name() == name)
                .unwrap()
                .value
                .clone()
        };
        assert_eq!(value("x"), PropValue::Lit(Lit::Num(11.0)));
        assert_eq!(value("y"), PropValue::Lit(Lit::Num(7.0)));
        assert_eq!(doc.text.matches("<style>").count(), 1);
        let other = doc.node_at("Root/Other").unwrap();
        assert!(
            doc.nodes[other]
                .rules
                .iter()
                .all(|rule| rule.props.iter().all(|p| p.name() != "x"))
        );
        let before = doc.clone();
        assert!(doc.set_prop(amount, "value", "not-a-number").is_err());
        assert_eq!(doc, before);
    }
}
