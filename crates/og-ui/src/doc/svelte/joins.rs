use super::*;

impl Parser {
    /// Joins each `aria-labelledby` and `aria-describedby` to the text of
    /// the elements its ids name (an element's own text, else its first
    /// text inside): one id's text as written, bound or not; several ids'
    /// literal texts joined by spaces, as ARIA computes a name.
    pub(super) fn join_aria(&mut self) -> Result<(), Error> {
        for (node, prop, ids, at) in std::mem::take(&mut self.aria_refs) {
            let mut texts = Vec::new();
            for id in ids.split_ascii_whitespace() {
                let Some(target) = self
                    .doc
                    .nodes
                    .iter()
                    .position(|n| n.id == id && matches!(n.kind, NodeKind::Element(_)))
                else {
                    return Err((
                        at,
                        format!("\"{id}\" names no element; give it id=\"{id}\""),
                    ));
                };
                let mut stack = vec![target];
                let mut found = None;
                while let Some(k) = stack.pop() {
                    if let Some(t) = self.doc.nodes[k].props.iter().find(|p| p.name() == "text") {
                        found = Some(t.clone());
                        break;
                    }
                    stack.extend(self.doc.nodes[k].children.iter().rev());
                }
                texts.push(found.ok_or((at, format!("#{id} has no text to read")))?);
            }
            let mut set = if let [one] = &texts[..] {
                one.clone()
            } else {
                let joined = texts
                    .iter()
                    .map(|t| match &t.value {
                        PropValue::Lit(Lit::Str(s)) => Ok(s.clone()),
                        _ => Err((
                            at,
                            "several ids join literal texts; name one element for a bound text"
                                .to_string(),
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .join(" ");
                PropSet {
                    value: PropValue::Lit(Lit::Str(joined)),
                    ..texts[0].clone()
                }
            };
            set.spec = schema::prop(prop).expect("an ARIA prop").0;
            self.doc.nodes[node].props.retain(|p| p.name() != prop);
            self.doc.nodes[node].props.push(set);
        }
        Ok(())
    }

    /// Joins each input's `list` to the `<datalist>` its id names: its
    /// options are a range's marks, or a text input's suggestions.
    pub(super) fn join_lists(&mut self) -> Result<(), Error> {
        for (range, id, at) in std::mem::take(&mut self.list_fors) {
            let Some((_, marks, items)) = self.datalists.iter().find(|(d, _, _)| *d == id).cloned()
            else {
                return Err((
                    at,
                    format!("list=\"{id}\" names no <datalist>; write <datalist id=\"{id}\">"),
                ));
            };
            let strs = |v: Vec<String>| PropValue::Lit(Lit::Strs(v));
            if matches!(self.doc.nodes[range].kind, NodeKind::Element("field")) {
                // A text input's suggestions: the options' values, or the
                // datalist's bound items. Labels and translations never
                // replace the value written back on selection.
                let options: Vec<_> = marks
                    .iter()
                    .filter(|o| items.is_none() && !o.disabled)
                    .collect();
                let value = match items {
                    Some(e) => PropValue::Bind(e, None),
                    None => strs(options.iter().map(|o| o.value.clone()).collect()),
                };
                self.prop(range, "field", "suggestions", value, at, "")?;
                for (name, values) in [
                    (
                        "suggestionLabels",
                        options.iter().map(|o| o.label.clone()).collect(),
                    ),
                    (
                        "suggestionKeys",
                        options
                            .iter()
                            .map(|o| o.key.clone().unwrap_or_default())
                            .collect(),
                    ),
                ] {
                    self.prop(range, "field", name, strs(values), at, "")?;
                }
                continue;
            }
            if items.is_some() {
                return Err((at, "a range's <datalist> lists literal <option>s".into()));
            }
            if let Some(m) = marks
                .iter()
                .find(|m| m.value.trim().parse::<f32>().is_err())
            {
                return Err((
                    at,
                    format!(
                        "a range's <datalist> lists numbers; '{}' is not one",
                        m.value
                    ),
                ));
            }
            let values = strs(marks.iter().map(|m| m.value.trim().to_string()).collect());
            // An option labelled only by its value is a tick without text.
            let labels = strs(
                marks
                    .iter()
                    .map(|m| {
                        if m.label == m.value {
                            String::new()
                        } else {
                            m.label.clone()
                        }
                    })
                    .collect(),
            );
            self.prop(range, "slider", "markValues", values, at, "")?;
            self.prop(range, "slider", "markLabels", labels, at, "")?;
        }
        Ok(())
    }

    /// Joins each `<label for>` to the control its id names: a checkbox
    /// takes the label's text as its caption and the label goes, as the
    /// wrapping form does; any other control keeps the label beside it
    /// and takes its text as its name.
    pub(super) fn join_labels(&mut self) -> Result<(), Error> {
        for (label, id, at) in std::mem::take(&mut self.label_fors) {
            let Some(target) = self
                .doc
                .nodes
                .iter()
                .position(|n| n.id == id && matches!(n.kind, NodeKind::Element(_)))
            else {
                return Err((
                    at,
                    format!("for=\"{id}\" names no element; give the control id=\"{id}\""),
                ));
            };
            let text = self.doc.nodes[label]
                .props
                .iter()
                .find(|p| p.name() == "text")
                .cloned();
            let NodeKind::Element(kind) = self.doc.nodes[target].kind else {
                unreachable!("found an element");
            };
            match kind {
                "toggle" => {
                    if self.doc.nodes[target]
                        .props
                        .iter()
                        .any(|p| p.name() == "text")
                    {
                        return Err((at, format!("#{id} has a caption already; write it once")));
                    }
                    self.doc.nodes[target].props.extend(text);
                    // Source order: a label written after its checkbox
                    // puts the caption after the switch.
                    if target < label {
                        let line = self.line(at);
                        self.doc.nodes[target].props.push(PropSet {
                            spec: schema::prop("side").expect("side is a prop").0,
                            value: PropValue::Lit(Lit::Str("end".into())),
                            line,
                            source: String::new(),
                        });
                    }
                    let parent = self.doc.nodes[label].parent;
                    if parent != NONE {
                        self.doc.nodes[parent].children.retain(|&c| c != label);
                    }
                    self.selectors.retain(|(n, ..)| *n != label);
                    // Left in no tree: an anonymous text so nothing reads it.
                    self.doc.nodes[label].kind = NodeKind::If(Expr::Lit(false.into()));
                    self.doc.nodes[label].props.clear();
                    self.doc.nodes[label].children.clear();
                }
                "field" | "select" | "slider" | "meter" | "progress" | "tabs" => {
                    if let Some(mut text) = text
                        && !self.doc.nodes[target]
                            .props
                            .iter()
                            .any(|p| p.name() == "name")
                    {
                        text.spec = schema::prop("name").expect("name is a prop").0;
                        self.doc.nodes[target].props.push(text);
                    }
                }
                other => {
                    return Err((
                        at,
                        format!(
                            "for=\"{id}\" names a {other}; a label names a form control (an <input>, <select>, <textarea>, <meter> or <progress>)"
                        ),
                    ));
                }
            }
        }
        Ok(())
    }
}
