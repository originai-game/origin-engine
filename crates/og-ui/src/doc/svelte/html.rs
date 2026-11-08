//! HTML's table, form, fieldset, figure and output elements (docs/ui.md
//! §2): their kinds, roles and attributes, and what joins them once the
//! tree is read (a header cell's role, a caption naming its table, a
//! disabled fieldset's controls, a form's submit and reset buttons).
//! A table's layout is CSS table layout's, which reads `colspan` and
//! `rowspan` from the cells.

use super::*;

/// Where each table part may sit: its tag and the tags of the elements
/// it belongs in.
const TABLE_PARENTS: &[(&str, &[&str])] = &[
    ("caption", &["table"]),
    ("thead", &["table"]),
    ("tbody", &["table"]),
    ("tfoot", &["table"]),
    ("tr", &["table", "thead", "tbody", "tfoot"]),
    ("th", &["tr"]),
    ("td", &["tr"]),
];

impl Parser {
    /// The kind and role of an HTML element this module owns; `None` for
    /// another tag.
    pub(super) fn html_kind(
        &self,
        tag: &str,
        parent: usize,
        at: usize,
    ) -> Result<Option<(&'static str, Option<&'static str>)>, Error> {
        if let Some((_, parents)) = TABLE_PARENTS.iter().find(|(t, _)| *t == tag) {
            // A component's root is placed where its user puts it.
            let inside = self.parent_tag(parent);
            if inside.is_some_and(|p| !parents.contains(&p) && !p.starts_with(char::is_uppercase)) {
                return Err((
                    at,
                    format!("<{tag}> belongs inside a <{}>", parents.join(">, <")),
                ));
            }
        }
        Ok(Some(match tag {
            "table" => ("box", Some("table")),
            "tr" => ("box", Some("row")),
            "td" => ("box", Some("cell")),
            // A header cell's role is joined once its row is read.
            "th" => ("box", Some("columnheader")),
            "caption" | "thead" | "tbody" | "tfoot" => ("box", None),
            "form" => ("box", None),
            "fieldset" => ("box", Some("group")),
            "legend" | "figcaption" => ("box", None),
            "figure" => ("box", Some("figure")),
            "output" => ("label", Some("status")),
            "template" => {
                return Err((
                    at,
                    "Svelte snippets are HTML's <template>: write {#snippet row()}…{/snippet} and {@render row()}".into(),
                ));
            }
            _ => return Ok(None),
        }))
    }

    /// Handles attribute `a` of element `i` when it is one of these
    /// elements' own; `Ok(false)` otherwise.
    pub(super) fn html_attr(&mut self, i: usize, tag: &str, a: &Attr) -> Result<bool, Error> {
        match (tag, a.name.as_str()) {
            ("td" | "th", "colspan" | "rowspan") => {
                let n = a
                    .value
                    .trim()
                    .parse::<u32>()
                    .ok()
                    .filter(|n| *n <= 1000 && (*n >= 1 || a.name == "rowspan"));
                if a.bound || n.is_none() {
                    return Err((
                        a.at,
                        format!(
                            "{} takes a literal whole number ({} to 1000)",
                            a.name,
                            if a.name == "rowspan" { 0 } else { 1 }
                        ),
                    ));
                }
                let value = PropValue::Lit(Lit::Num(n.unwrap() as f32));
                self.prop(i, "box", &a.name, value, a.at, &a.value)?;
            }
            ("th", "scope") => {
                if a.bound || !matches!(a.value.as_str(), "row" | "col" | "rowgroup" | "colgroup") {
                    return Err((a.at, "scope is row, col, rowgroup or colgroup".into()));
                }
                self.doc.nodes[i].role = Some(if a.value.starts_with("row") {
                    "rowheader"
                } else {
                    "columnheader"
                });
                self.scoped.push(i);
            }
            ("form", "onsubmit" | "onreset") => {
                if !a.bound || !identifier(&a.value) {
                    return Err((
                        a.at,
                        format!("{} names a mounted action: {}={{save}}", a.name, a.name),
                    ));
                }
                let submit = a.name == "onsubmit";
                match self.forms.iter_mut().find(|f| f.0 == i) {
                    Some(f) if submit => f.1 = Some(a.value.clone()),
                    Some(f) => f.2 = Some(a.value.clone()),
                    None if submit => self.forms.push((i, Some(a.value.clone()), None)),
                    None => self.forms.push((i, None, Some(a.value.clone()))),
                }
            }
            ("form", "novalidate") | ("button", "formnovalidate") => {
                if a.bound {
                    return Err((a.at, format!("{} is a boolean attribute", a.name)));
                }
                self.novalidate.push(i);
            }
            ("button", "type") => {
                if a.bound || !matches!(a.value.as_str(), "submit" | "reset" | "button") {
                    return Err((a.at, "a button's type is submit, reset or button".into()));
                }
                self.buttons.push((i, a.value.clone(), a.at));
            }
            ("fieldset", "disabled") => {
                let value = if a.bound {
                    expression(&a.value, a.at)?
                } else {
                    Expr::Lit(true.into())
                };
                self.fieldsets.push((i, value));
            }
            ("output", "for") => {
                if a.bound || a.value.trim().is_empty() {
                    return Err((
                        a.at,
                        "for names the controls an output reads by their literal ids".into(),
                    ));
                }
                self.aria_refs_for.push((a.value.clone(), a.at));
            }
            ("output", "name") => {}
            ("input", "type") if !a.bound => match a.value.as_str() {
                "file" => {
                    return Err((
                        a.at,
                        "<input type=\"file\"> needs a host file picker, which no host offers; take files in a TS system".into(),
                    ));
                }
                _ => return Ok(false),
            },
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// Once the tree is read: header cells' roles, captions, legends and
    /// figcaptions naming their elements, disabled fieldsets' controls,
    /// and forms' submit and reset buttons.
    pub(super) fn join_html(&mut self) -> Result<(), Error> {
        let tag =
            |doc: &UiDocument, k: usize| doc.nodes[k].selector_tag.clone().unwrap_or_default();
        let element_children = |doc: &UiDocument, k: usize| -> Vec<usize> {
            let mut out = Vec::new();
            let mut stack: Vec<usize> = doc.nodes[k].children.iter().rev().copied().collect();
            while let Some(c) = stack.pop() {
                match doc.nodes[c].kind {
                    NodeKind::Element(_) | NodeKind::Use(_) => out.push(c),
                    _ => stack.extend(doc.nodes[c].children.iter().rev().copied()),
                }
            }
            out
        };
        let enclosing = |doc: &UiDocument, mut k: usize, want: &str| {
            k = doc.nodes[k].parent;
            while k != NONE {
                if doc.nodes[k].selector_tag.as_deref() == Some(want) {
                    return Some(k);
                }
                k = doc.nodes[k].parent;
            }
            None
        };
        for i in 0..self.doc.nodes.len() {
            match tag(&self.doc, i).as_str() {
                // A header cell without scope heads its column in a head
                // group or a row of headers, else its row (HTML's auto).
                "th" if !self.scoped.contains(&i) => {
                    let row = self.doc.nodes[i].parent;
                    let in_head = enclosing(&self.doc, i, "thead").is_some();
                    let data = row != NONE
                        && element_children(&self.doc, row)
                            .iter()
                            .any(|&c| tag(&self.doc, c) == "td");
                    if !in_head && data {
                        self.doc.nodes[i].role = Some("rowheader");
                    }
                }
                // The first caption, legend or figcaption names its element.
                "table" | "fieldset" | "figure" => {
                    let want = match tag(&self.doc, i).as_str() {
                        "table" => "caption",
                        "fieldset" => "legend",
                        _ => "figcaption",
                    };
                    let kids = element_children(&self.doc, i);
                    let Some(&first) = kids.iter().find(|&&c| tag(&self.doc, c) == want) else {
                        continue;
                    };
                    if want != "figcaption" && kids.first() != Some(&first) {
                        return Err((
                            self.doc.nodes[first].source_offset,
                            format!(
                                "a <{want}> is the first child of its <{}>",
                                tag(&self.doc, i)
                            ),
                        ));
                    }
                    if self.doc.nodes[i].props.iter().any(|p| p.name() == "name") {
                        continue;
                    }
                    let mut stack = vec![first];
                    while let Some(k) = stack.pop() {
                        if let Some(t) = self.doc.nodes[k].props.iter().find(|p| p.name() == "text")
                        {
                            let mut t = t.clone();
                            t.spec = schema::prop("name").expect("name is a prop").0;
                            self.doc.nodes[i].props.push(t);
                            break;
                        }
                        stack.extend(self.doc.nodes[k].children.iter().rev());
                    }
                }
                _ => {}
            }
        }
        for (id, at) in std::mem::take(&mut self.aria_refs_for) {
            for id in id.split_ascii_whitespace() {
                if !self.doc.nodes.iter().any(|n| n.id == id) {
                    return Err((
                        at,
                        format!("for names \"{id}\", which no element has as its id"),
                    ));
                }
            }
        }
        // A disabled fieldset disables its controls but those in its
        // first legend.
        let disabled = schema::prop("disabled").expect("disabled is a prop");
        for (fieldset, value) in std::mem::take(&mut self.fieldsets) {
            let kids = element_children(&self.doc, fieldset);
            let legend = kids
                .first()
                .copied()
                .filter(|&c| tag(&self.doc, c) == "legend");
            let mut stack = kids;
            while let Some(k) = stack.pop() {
                if Some(k) == legend {
                    continue;
                }
                stack.extend(element_children(&self.doc, k));
                let NodeKind::Element(kind) = self.doc.nodes[k].kind else {
                    continue;
                };
                if !disabled.1.applies(kind) || kind == "details" {
                    continue;
                }
                let line = self.doc.nodes[k].line;
                let own = self.doc.nodes[k]
                    .props
                    .iter()
                    .position(|p| p.spec == disabled.0);
                let value = match own.map(|p| self.doc.nodes[k].props.remove(p).value) {
                    None | Some(PropValue::Lit(Lit::Bool(false))) => value.clone(),
                    Some(PropValue::Lit(Lit::Bool(true))) => Expr::Lit(true.into()),
                    Some(PropValue::Bind(e, _)) => {
                        Expr::Bin(crate::bind::Op::Or, Box::new(e), Box::new(value.clone()))
                    }
                    Some(_) => value.clone(),
                };
                let value = match value {
                    Expr::Lit(serde_json::Value::Bool(b)) => PropValue::Lit(Lit::Bool(b)),
                    e => PropValue::Bind(e, None),
                };
                self.doc.nodes[k].props.push(PropSet {
                    spec: disabled.0,
                    value,
                    line,
                    source: String::new(),
                });
            }
        }
        // A button in a form submits it (its default type) or resets it,
        // sending the form's onsubmit or onreset action.
        let typed = std::mem::take(&mut self.buttons);
        for i in 0..self.doc.nodes.len() {
            if self.doc.nodes[i].kind != NodeKind::Element("button")
                || tag(&self.doc, i) != "button"
            {
                continue;
            }
            let ty = typed
                .iter()
                .find(|(b, ..)| *b == i)
                .map_or("submit", |(_, t, _)| t.as_str());
            let Some(form) = enclosing(&self.doc, i, "form") else {
                if let Some((_, t, at)) = typed.iter().find(|(b, ..)| *b == i)
                    && t != "button"
                {
                    return Err((*at, format!("a {t} button sits inside a <form>")));
                }
                continue;
            };
            let Some((_, submit, reset)) = self.forms.iter().find(|f| f.0 == form) else {
                continue;
            };
            let action = match ty {
                "submit" => submit,
                "reset" => reset,
                _ => continue,
            };
            let Some(action) = action.clone() else {
                continue;
            };
            // A submit validates the form's text fields first, unless the
            // form or the button says novalidate.
            let act = if ty == "submit"
                && !self.novalidate.contains(&form)
                && !self.novalidate.contains(&i)
            {
                let mut controls = Vec::new();
                let mut stack = element_children(&self.doc, form);
                while let Some(k) = stack.pop() {
                    if matches!(
                        self.doc.nodes[k].kind,
                        NodeKind::Element("field" | "datetime")
                    ) {
                        controls.push(k);
                    }
                    stack.extend(element_children(&self.doc, k));
                }
                controls.sort_unstable();
                let mut row = false;
                let mut k = self.doc.nodes[form].parent;
                while k != NONE {
                    row |= matches!(self.doc.nodes[k].kind, NodeKind::For { .. });
                    k = self.doc.nodes[k].parent;
                }
                Act::Submit {
                    action,
                    controls,
                    row,
                }
            } else {
                Act::Emit(action)
            };
            let line = self.doc.nodes[i].line;
            self.doc.nodes[i].triggers.push(Trigger {
                when: When::Click,
                act,
                line,
            });
        }
        Ok(())
    }
}
