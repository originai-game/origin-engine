//! What the build reads of an element for structural selectors, which
//! match on the mounted tree ([`crate::doc::topology`]): its selector
//! facts and a list item's number.
use super::*;
use crate::doc::selectors::Selector;

/// The selector facts a node's authored inputs evaluate to, as the retained
/// topology records them: `class` joins the class list and true `class:` names.
pub fn selector_facts(
    node: &DocNode,
    eval: impl Fn(&crate::bind::Expr) -> Value,
) -> BTreeMap<String, Value> {
    let mut out = BTreeMap::new();
    let mut classes = BTreeSet::new();
    let mut has_classes = false;
    for (name, expression) in &node.selector_attrs {
        let value = eval(expression);
        if name == "class" {
            has_classes = true;
            if let Some(value) = value.as_str() {
                classes.extend(value.split_whitespace().map(str::to_owned));
            }
            continue;
        }
        if let Some(class) = name.strip_prefix("class:") {
            has_classes = true;
            if truthy(&value) {
                classes.insert(class.to_owned());
            }
        }
        out.insert(name.clone(), value);
    }
    if has_classes {
        out.insert(
            "class".into(),
            json!(classes.into_iter().collect::<Vec<_>>().join(" ")),
        );
    }
    out
}

impl crate::ui::Docs {
    /// A `<li>`'s number when its marker shows one: `count` reads it from
    /// the mounted tree.
    pub fn ordinal_of(
        &self,
        n: &DocNode,
        r: &Resolved,
        count: impl FnOnce(&[Selector], bool) -> i64,
    ) -> Option<i64> {
        let profile = &self.library.structure;
        (profile.ordinals
            && n.selector_tag.as_deref() == Some("li")
            && r.str("markerType").is_some_and(crate::doc::lists::numbered))
        .then(|| count(&crate::doc::lists::items(), profile.item_values))
    }
}
