use super::*;

impl Ui {
    /// Builds node `i` (and its subtree); `root_id` names a mount's root.
    /// Answers its path (none when it is not shown).
    pub(super) fn build_node(
        &mut self,
        b: &mut Build<'_>,
        i: usize,
        root_id: Option<&str>,
    ) -> Option<String> {
        let doc = b.doc.clone();
        let n = &doc.nodes[i];
        match &n.kind {
            NodeKind::If(e) => {
                if truthy(&b.eval(e)) {
                    for &c in &n.children {
                        self.build_node(b, c, None);
                    }
                }
                return None;
            }
            NodeKind::Snippet(_) => return None,
            NodeKind::Render { name, optional } => {
                if let Some(snippet) = doc
                    .nodes
                    .iter()
                    .find(|n| n.kind == NodeKind::Snippet(name.clone()))
                {
                    for &child in &snippet.children {
                        self.build_node(b, child, None);
                    }
                } else if !optional {
                    self.warnings.push(format!(
                        "{}:{}: supply content '{name}' at mount",
                        doc.source, n.line
                    ));
                }
                return None;
            }
            NodeKind::For {
                var, list, empty, ..
            } => {
                let items = b.eval(list);
                let origin = match list {
                    crate::bind::Expr::Path(p) => b.write_path(p),
                    _ => None,
                };
                let items = items.as_array().cloned().unwrap_or_default();
                if items.is_empty() {
                    for &child in empty {
                        self.build_node(b, child, None);
                    }
                }
                for (k, item) in items.into_iter().enumerate() {
                    self.build_item(b, n, var, item, k, origin.as_deref());
                }
                return None;
            }
            NodeKind::Use(comp) => {
                let Some(cdoc) = self.docs.docs.get(comp).cloned() else {
                    self.warnings.push(format!(
                        "{}:{}: no component '{comp}' (load its document)",
                        doc.source, n.line
                    ));
                    return None;
                };
                let mut props = serde_json::Map::new();
                let mut links = Vec::new();
                for (k, v, _) in &n.args {
                    let x = match v {
                        PropValue::Lit(l) => match l {
                            Lit::Num(x) => json!(x),
                            Lit::Bool(x) => json!(x),
                            Lit::Str(s) => json!(s),
                            Lit::Strs(s) => json!(s),
                            _ => Value::Null,
                        },
                        PropValue::Model(p) => {
                            if let Some(w) = b.write_path(p) {
                                links.push((k.clone(), w));
                            }
                            b.scope().get(p)
                        }
                        PropValue::Bind(e, f) => {
                            if let crate::bind::Expr::Path(p) = e
                                && let Some(w) = b.write_path(p)
                            {
                                links.push((k.clone(), w));
                            }
                            let x = b.eval(e);
                            match f {
                                Some(f) => {
                                    json!(crate::bind::format_value(&x, *f, b.env.language()))
                                }
                                None => x,
                            }
                        }
                        PropValue::Template(t) => json!(t.format(&b.scope())),
                    };
                    props.insert(k.clone(), x);
                }
                let mut vm = Value::Object(props);
                let id = root_id.unwrap_or(if n.id.is_empty() {
                    comp.as_str()
                } else {
                    n.id.as_str()
                });
                let mount = format!("{}/{id}", self.path_of(self.parent()));
                let before = vm.clone();
                let mut built = Built::default();
                let opened = self.enter_structure(b, i);
                let rp = self.resolve(b, n, &mount, &mut built, &b.inherited, &mut |s| {
                    self.structure_matches(&doc, s)
                });
                self.docs.component_root = self.docs.structure.any;
                self.warnings.extend(
                    rp.cascade
                        .diagnostics
                        .iter()
                        .map(|error| format!("{}:{} ({mount}): {error}", doc.source, n.line)),
                );
                let r =
                    self.mount_doc(cdoc, mount, id, &mut vm, b.env, rp.cascade.computed.clone());
                self.docs.component_root = false;
                if opened {
                    self.docs.levels.pop();
                }
                // The instance's own element props apply to its root.
                if !rp.vals.is_empty()
                    && let Some(&last) = Some(&self.last_closed)
                    && last != NONE
                {
                    let el = Ui::box_of(&rp, self.nodes[last].el);
                    let props = self.props_of(b, n, self.nodes[last].key, &rp, &mut built);
                    self.nodes[last].el = el;
                    if props != Props::default() {
                        self.nodes[last].props = props;
                    }
                }
                // Its two-way writes reach the bound paths of the user.
                for (k, w) in links {
                    if vm[&k] != before[&k] {
                        b.writes.push((w, vm[&k].clone()));
                    }
                }
                b.events.extend(r.events);
                return Some(r.path);
            }
            NodeKind::Element(_) => {}
        }
        let NodeKind::Element(kind) = n.kind else {
            return None;
        };
        let id = match root_id {
            Some(r) => r.to_string(),
            None if n.id.is_empty() => {
                // Anonymous: its kind and its place among same-kind siblings.
                let k = doc.nodes[n.parent]
                    .children
                    .iter()
                    .take_while(|&&c| c != i)
                    .filter(|&&c| doc.nodes[c].id.is_empty() && doc.nodes[c].kind == n.kind)
                    .count();
                format!("{kind}#{k}")
            }
            None => n.id.clone(),
        };
        let parent_path = self.path_of(self.parent());
        let path = if parent_path == "root" {
            id.clone()
        } else {
            format!("{parent_path}/{id}")
        };
        let key = crate::id::key_of_path(&path);
        let mut built = Built {
            path: path.clone(),
            doc: doc.name.clone(),
            node: i,
            line: n.line,
            mount: b.mount.clone(),
            ..Built::default()
        };
        let opened = self.enter_structure(b, i);
        let children = self.child_level(b, n);
        let r = self.resolve(b, n, &path, &mut built, &b.inherited, &mut |s| {
            self.structure_matches(&doc, s)
        });
        self.warnings.extend(
            r.cascade
                .diagnostics
                .iter()
                .map(|error| format!("{}:{} ({path}): {error}", doc.source, n.line)),
        );
        let structural = children.is_some();
        self.docs.levels.extend(children);
        let built = self.build_resolved_node(
            b,
            i,
            ResolvedNode {
                id,
                path,
                key,
                values: r,
                probe: built,
            },
            |ui, b, _| {
                for &child in &n.children {
                    ui.build_node(b, child, None);
                }
            },
        );
        if structural {
            self.docs.levels.pop();
        }
        if opened {
            self.docs.levels.pop();
        }
        built
    }

    pub(super) fn build_resolved_node(
        &mut self,
        b: &mut Build<'_>,
        i: usize,
        resolved: ResolvedNode,
        build_children: impl FnOnce(&mut Ui, &mut Build<'_>, Option<Props>),
    ) -> Option<String> {
        let doc = b.doc.clone();
        let n = &doc.nodes[i];
        let NodeKind::Element(kind) = n.kind else {
            unreachable!("only element nodes have resolved widget properties");
        };
        let ResolvedNode {
            id,
            path,
            key,
            values: r,
            probe: mut built,
        } = resolved;
        if r.has("visible") && !r.flag("visible") && r.cascade.container_reads.is_empty() {
            return None;
        }
        self.docs.seen_now.insert(key);
        self.remember_queried(b, i, &path, key, &r);
        if let Some(container) = r.container {
            self.build_container(container);
        }
        let props = self.props_of(b, n, key, &r, &mut built);
        let t = self.theme;
        let text_of = |r: &Resolved, b: &Build<'_>| -> String {
            if let Some(k) = r.str("key")
                && let Some(s) = b.env.translate(k)
            {
                return s;
            }
            r.str("text").unwrap_or_default().to_string()
        };
        let mut resp = crate::ui::Response::default();
        let container = schema::kind(kind).is_some_and(|k| k.container);
        // A dialog's or a disclosure's `open`: the bound store's, else the
        // literal's until the player toggles it (kept as widget memory).
        let open_now = |ui: &mut Ui| -> bool {
            let literal = r.flag("open");
            if bound(n, "open") {
                literal
            } else {
                ui.memory(
                    &format!("{}~doc-open", id.as_str()),
                    f64::from(u8::from(literal)),
                ) > 0.0
            }
        };
        let dialog = crate::widgets::DialogDesc {
            modal: r.flag("modal"),
            name: r.str("name").unwrap_or_default().to_string(),
            width: r.size("width"),
        };
        let mut opened: Option<bool> = None;
        // A closed dialog or disclosure builds no content (HTML renders
        // none): opened, its controls are built fresh and take input.
        let mut shows_content = true;
        if container {
            match kind {
                "dialog" => {
                    let open = open_now(self);
                    let out = self.open_dialog(id.as_str(), open, &dialog);
                    shows_content = out.visible;
                    if out.dismissed {
                        opened = Some(false);
                    }
                }
                "details" => {
                    let open = open_now(self);
                    let out = self.open_details(
                        id.as_str(),
                        &crate::widgets::DetailsDesc {
                            summary: r.str("summary").unwrap_or_default().to_string(),
                            open,
                            disabled: r.flag("disabled"),
                            width: r.size("width"),
                        },
                    );
                    shows_content = out.open;
                    if out.toggled {
                        opened = Some(out.open);
                    }
                }
                "panel" => {
                    self.open_panel(
                        id.as_str(),
                        crate::ui::PanelDesc {
                            row: false,
                            ..Default::default()
                        },
                    );
                }
                "scroll" => {
                    self.open_scroll(
                        id.as_str(),
                        crate::ui::ScrollDesc {
                            width: Size::grow(),
                            height: Size::grow(),
                            ..Default::default()
                        },
                    );
                }
                "list" | "cells" => {
                    let inherited = std::mem::replace(&mut b.inherited, r.cascade.computed.clone());
                    if b.cached.is_some() {
                        build_children(self, b, Some(props.clone()));
                    } else {
                        self.build_virtual(b, n, kind, &id, &r, props.clone(), &path);
                    }
                    b.inherited = inherited;
                    let at = self.last_closed;
                    if at != NONE && at < self.nodes.len() && self.nodes[at].key == key {
                        // Settlement replaces the box the list was opened with;
                        // its window and stride follow on the next build.
                        let mut fields =
                            fields::RenderFields::capture(self, at, kind, resp, props, None);
                        fields.unstyle_virtual();
                        self.note_render_fields(key, fields);
                    }
                    self.note_built(built);
                    self.fire(b, n, key, &path, resp);
                    return Some(path);
                }
                _ => {
                    self.open(
                        id.as_str(),
                        Element {
                            direction: match kind {
                                "row" => Direction::Row,
                                _ => Direction::Column,
                            },
                            ..Element::default()
                        },
                    );
                }
            }
            let fields =
                fields::RenderFields::capture(self, self.parent(), kind, resp, props, None);
            self.apply_document_fields(n, &r, &fields);
            self.note_render_fields(key, fields);
            if let Some(id) = r.str("rive") {
                let at = self.parent();
                self.play_rive(at, id, &path);
            }
            if kind == "slot" {
                let at = self.parent();
                self.docs.slots.insert(path.clone(), at);
                let short = format!("{}/{}", b.mount, n.id);
                self.docs.slots.entry(short).or_insert(at);
            }
            self.note_built(built);
            if shows_content {
                let inherited = std::mem::replace(&mut b.inherited, r.cascade.computed.clone());
                build_children(self, b, None);
                b.inherited = inherited;
            }
            match kind {
                "dialog" => self.close_dialog(&dialog),
                "details" => self.close_details(),
                _ => resp = self.close(),
            }
            if let Some(open) = opened {
                if !bound(n, "open") {
                    self.remember(
                        &format!("{}~doc-open", id.as_str()),
                        f64::from(u8::from(open)),
                    );
                }
                self.write_back(b, n, "open", json!(open));
            }
            self.fire(b, n, key, &path, resp);
            return Some(path);
        }
        // Keep the widget's intrinsic box separate from CSS-owned fields.
        // Settlement can replace those fields without processing its input again.
        let styled = r;
        let r = styled.intrinsic();
        let mut texture = None;
        match kind {
            "label" => {
                let text = text_of(&r, b);
                let desc = r.label_desc(&t);
                if let Some(V::Spans(spans)) = r.get("spans") {
                    let spans = match spans {
                        Ok(spans) => spans,
                        Err(error) => {
                            self.warnings.push(format!(
                                "{}:{}: {path}.spans: expected TextSpan[]: {error}",
                                doc.source, n.line
                            ));
                            return None;
                        }
                    };
                    resp = self
                        .rich_text_spans(
                            id.as_str(),
                            &text,
                            spans,
                            crate::RichTextDesc {
                                step: desc.step,
                                color: desc.color,
                                align: desc.align,
                                nowrap: desc.nowrap,
                                width: desc.width,
                                font: desc.font,
                            },
                            |span| {
                                r.span_styles
                                    .iter()
                                    .find(|(attribute, _)| attribute == span)
                                    .map(|(_, style)| style.clone())
                                    .unwrap_or_default()
                            },
                        )
                        .response;
                } else {
                    self.label(id.as_str(), &text, desc);
                }
            }
            "button" => {
                let text = text_of(&r, b);
                resp = self.button(id.as_str(), &text, r.button_desc());
            }
            "art" => {
                // An art slot: its art fitted by its spec, or the style's
                // placeholder for its kind (docs/ui.md §3.3).
                self.art_spec(&path, &r);
                let spec = self.last_art().clone();
                texture = r.handle("texture").and_then(|h| b.env.texture(h));
                self.art(
                    id.as_str(),
                    &crate::widgets::ArtDesc {
                        spec,
                        texture,
                        size: None,
                        width: r.size("width").unwrap_or_default(),
                        height: r.size("height").unwrap_or_default(),
                        label: r.str("name").unwrap_or_default().to_string(),
                    },
                );
            }
            "image" => {
                texture = r.handle("texture").and_then(|h| b.env.texture(h));
                let tint = r.color("tint");
                self.image(
                    id.as_str(),
                    crate::ui::ImageDesc {
                        texture,
                        tint,
                        width: r.size("width").unwrap_or_default(),
                        height: r.size("height").unwrap_or_default(),
                        aspect: r
                            .nums("ratio")
                            .filter(|n| n.len() == 2 && n[1] > 0.0)
                            .map_or(r.num("aspect").unwrap_or(0.0), |n| n[0] / n[1]),
                        label: r.str("name").unwrap_or_default().to_string(),
                        ..Default::default()
                    },
                );
            }
            "icon" => {
                let name = r.str("icon").unwrap_or_default().to_string();
                self.icon(
                    id.as_str(),
                    &name,
                    crate::widgets::IconDesc {
                        size: r.num("iconSize").unwrap_or(0.0),
                        tint: r.color("color"),
                        secondary: r.str("secondary").map(str::to_string),
                        secondary_tint: r.color("secondaryTint"),
                        label: r.str("name").map(str::to_string),
                    },
                );
            }
            "meter" => {
                self.gauge(
                    id.as_str(),
                    &crate::widgets::GaugeDesc {
                        value: r.num("value").unwrap_or(0.0),
                        min: r.num("min").unwrap_or(0.0),
                        max: r.num("max").unwrap_or(1.0),
                        low: r.num("low"),
                        high: r.num("high"),
                        optimum: r.num("optimum"),
                        width: r.size("width"),
                    },
                );
            }
            "select" => {
                let strs = |name: &str| match r.get(name) {
                    Some(V::Strs(s)) => s.clone(),
                    _ => Vec::new(),
                };
                let (items, keys, groups, off) = (
                    strs("items"),
                    strs("keys"),
                    strs("groups"),
                    strs("disabledItems"),
                );
                let mut values = strs("values");
                values.resize(items.len(), String::new());
                for (v, item) in values.iter_mut().zip(&items) {
                    if v.is_empty() {
                        v.clone_from(item);
                    }
                }
                let selected = r
                    .str("current")
                    .and_then(|now| values.iter().position(|v| v == now))
                    .map_or(-1, |k| k as i32);
                let options = items
                    .iter()
                    .enumerate()
                    .map(|(k, item)| crate::widgets::DropdownOption {
                        label: keys
                            .get(k)
                            .filter(|key| !key.is_empty())
                            .and_then(|key| b.env.translate(key))
                            .unwrap_or_else(|| item.clone()),
                        group: groups.get(k).cloned().unwrap_or_default(),
                        disabled: off.iter().any(|o| o == &k.to_string()),
                        ..crate::widgets::DropdownOption::default()
                    })
                    .collect();
                let out = self.dropdown(
                    id.as_str(),
                    &crate::widgets::DropdownDesc {
                        options,
                        selected,
                        placeholder: r.str("placeholder").unwrap_or_default().to_string(),
                        disabled: r.flag("disabled"),
                        width: r.size("width"),
                        ..crate::widgets::DropdownDesc::default()
                    },
                );
                resp = crate::Response {
                    changed: out.changed,
                    ..crate::Response::default()
                };
                if out.changed
                    && let Some(v) = usize::try_from(out.selected)
                        .ok()
                        .and_then(|k| values.get(k))
                {
                    self.write_back(b, n, "current", json!(v));
                }
            }
            "progress" => {
                let v = r.num("value").unwrap_or(0.0);
                self.progress(
                    id.as_str(),
                    v,
                    crate::ui::ProgressDesc {
                        width: r.size("width"),
                    },
                );
            }
            "slider" => {
                let v0 = r.num("value").unwrap_or(0.0);
                let out = self.value_slider(
                    id.as_str(),
                    v0,
                    &crate::widgets::ValueSliderDesc {
                        min: r.num("min").unwrap_or(0.0),
                        max: r.num("max").unwrap_or(1.0),
                        step: r.num("stepBy").unwrap_or(0.0),
                        label: text_of(&r, b),
                        show_value: !r.has("showValue") || r.flag("showValue"),
                        width: r.size("width"),
                        ..crate::widgets::ValueSliderDesc::default()
                    },
                );
                resp = crate::Response {
                    changed: out.changed,
                    ..out.response
                };
                if out.changed {
                    self.write_back(b, n, "value", json!(out.value));
                }
            }
            "toggle" => {
                let on = r.flag("checked");
                let out = self.toggle_switch(
                    id.as_str(),
                    on,
                    &crate::widgets::ToggleSwitchDesc {
                        label: text_of(&r, b),
                        width: r.size("width"),
                        ..crate::widgets::ToggleSwitchDesc::default()
                    },
                );
                resp = crate::Response {
                    changed: out.changed,
                    ..out.response
                };
                if out.changed {
                    self.write_back(b, n, "checked", json!(out.on));
                }
            }
            "tabs" => {
                let strs = |name: &str| match r.get(name) {
                    Some(V::Strs(s)) => s.clone(),
                    _ => Vec::new(),
                };
                // A tab's label: its string table key's text where it has one, else its item.
                let keys = strs("keys");
                let items: Vec<String> = strs("items")
                    .into_iter()
                    .enumerate()
                    .map(|(i, item)| keys.get(i).and_then(|k| b.env.translate(k)).unwrap_or(item))
                    .collect();
                // With values, `current` holds the selected tab's name (a screen's mode) instead of `selected`'s index.
                let values = strs("values");
                let mut selected = if values.is_empty() {
                    r.num("selected").unwrap_or(0.0) as usize
                } else {
                    let now = r.str("current").unwrap_or_default();
                    values.iter().position(|v| v == now).unwrap_or(0)
                };
                let fill = r.flag("fill");
                let out = self.segmented(
                    id.as_str(),
                    &crate::widgets::SegmentedDesc {
                        items: items
                            .iter()
                            .map(|l| crate::widgets::SegmentItem {
                                label: l.clone(),
                                ..crate::widgets::SegmentItem::default()
                            })
                            .collect(),
                        selected: selected as i32,
                        equal: fill,
                        width: Some(r.size("width").unwrap_or(if fill {
                            Size::grow()
                        } else {
                            Size::fit()
                        })),
                        label: text_of(&r, b),
                        ..crate::widgets::SegmentedDesc::default()
                    },
                );
                selected = usize::try_from(out.selected).unwrap_or(0);
                resp = crate::Response {
                    changed: out.changed,
                    focused: out.focused >= 0,
                    ..crate::Response::default()
                };
                if resp.changed {
                    match values.get(selected) {
                        Some(v) => self.write_back(b, n, "current", json!(v)),
                        None => self.write_back(b, n, "selected", json!(selected)),
                    }
                }
            }
            "field" => {
                let mut s = r.str("input").unwrap_or_default().to_string();
                let constraints = crate::input_type::Constraints {
                    required: r.flag("required"),
                    min_chars: r.num("minChars").unwrap_or(0.0) as u32,
                    pattern: r.str("pattern").map(str::to_string),
                    min: r.num("min").map(f64::from),
                    max: r.num("max").map(f64::from),
                    step: r.num("stepBy").map(f64::from),
                };
                resp = if r.flag("multiline") {
                    self.text_area(
                        id.as_str(),
                        &mut s,
                        &crate::ui::TextAreaDesc {
                            placeholder: r.str("placeholder").unwrap_or_default().to_string(),
                            width: r.size("width"),
                            rows: r.num("lines").unwrap_or(0.0) as u32,
                            max_chars: r.num("maxChars").unwrap_or(0.0) as u32,
                            invalid: r.flag("invalid"),
                            bare: false,
                            tight: r.flag("tight"),
                            component: r.str("component").map(str::to_string),
                            readonly: r.flag("readonly"),
                            constraints,
                        },
                    )
                } else {
                    self.text_field(
                        id.as_str(),
                        &mut s,
                        &crate::ui::FieldDesc {
                            placeholder: r.str("placeholder").unwrap_or_default().to_string(),
                            width: r.size("width"),
                            max_chars: r.num("maxChars").unwrap_or(0.0) as u32,
                            invalid: r.flag("invalid"),
                            bare: false,
                            tight: r.flag("tight"),
                            component: r.str("component").map(str::to_string),
                            input_type: r
                                .str("type")
                                .and_then(crate::InputType::from_name)
                                .unwrap_or_default(),
                            inputmode: r.str("inputmode").and_then(crate::InputMode::from_name),
                            autocomplete: r.str("autocomplete").unwrap_or_default().to_string(),
                            readonly: r.flag("readonly"),
                            constraints,
                        },
                    )
                };
                if resp.changed {
                    self.write_back(b, n, "input", json!(s));
                }
            }
            "badge" => {
                let text = text_of(&r, b);
                let tone = match r.str("tone") {
                    Some("info") => crate::widgets::Tone::Info,
                    Some("success") => crate::widgets::Tone::Success,
                    Some("warn") => crate::widgets::Tone::Warn,
                    Some("danger") => crate::widgets::Tone::Danger,
                    Some("accent") => crate::widgets::Tone::Primary,
                    _ => crate::widgets::Tone::Neutral,
                };
                self.badge(id.as_str(), &text, tone);
            }
            "spacer" => {
                let s = r.size("width").or(r.size("height"));
                self.spacer(id.as_str(), s);
            }
            "divider" => self.divider(id.as_str(), r.flag("vertical")),
            other => {
                self.warnings
                    .push(format!("{}:{}: '{other}' is not built", doc.source, n.line));
                return None;
            }
        }
        let last = self.last_closed;
        if last != NONE && last < self.nodes.len() {
            let fields = fields::RenderFields::capture(self, last, kind, resp, props, texture);
            self.apply_document_fields(n, &styled, &fields);
            self.note_render_fields(key, fields);
        }
        self.note_built(built);
        self.fire(b, n, key, &path, resp);
        Some(path)
    }

    /// One item of a `for` outside a virtualised container.
    fn build_item(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        var: &str,
        item: Value,
        k: usize,
        origin: Option<&str>,
    ) {
        b.locals
            .push((var.to_string(), item, origin.map(|o| format!("{o}.{k}"))));
        b.locals.push(("$index".into(), json!(k), None));
        let NodeKind::For { key, .. } = &n.kind else {
            unreachable!()
        };
        let row_key = b.eval(key);
        let Some(row_key) = row_key.as_str().filter(|key| !key.is_empty()) else {
            self.warnings.push(format!(
                "{}:{}: each key must be a nonempty string",
                b.doc.source, n.line
            ));
            b.locals.pop();
            b.locals.pop();
            return;
        };
        let outer_key = std::mem::replace(&mut b.row_key, row_key.into());
        let doc = b.doc.clone();
        for &c in &n.children {
            let id = match &doc.nodes[c].kind {
                NodeKind::Element(kind) if doc.nodes[c].id.is_empty() => {
                    Some(format!("{kind}#{row_key}"))
                }
                NodeKind::Element(_) | NodeKind::Use(_) => {
                    Some(format!("{}#{row_key}", doc.nodes[c].id))
                }
                _ => None,
            };
            self.build_node(b, c, id.as_deref());
        }
        b.row_key = outer_key;
        b.locals.pop();
        b.locals.pop();
    }

    /// A `list` or `cells` and its `for`: only the rows in view are built.
    #[allow(clippy::too_many_arguments)]
    fn build_virtual(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        kind: &str,
        id: &str,
        r: &Resolved,
        props: Props,
        path: &str,
    ) {
        let doc = b.doc.clone();
        let Some(&f) = n.children.first() else { return };
        let NodeKind::For { var, list, .. } = &doc.nodes[f].kind else {
            return;
        };
        let items = b.eval(list).as_array().cloned().unwrap_or_default();
        let origin = match list {
            crate::bind::Expr::Path(p) => b.write_path(p),
            _ => None,
        };
        let count = items.len() as u32;
        let padding = r.nums("padding").map(insets_of).unwrap_or_default();
        let range = if kind == "list" {
            self.open_list(
                id,
                crate::widgets::ListDesc {
                    count,
                    row_height: r.num("rowHeight").unwrap_or(0.0),
                    gap: r.num("gap").unwrap_or(0.0),
                    width: r.size("width").unwrap_or(Size::grow()),
                    height: r.size("height"),
                    padding,
                },
            )
        } else {
            self.open_grid(
                id,
                crate::widgets::GridDesc {
                    count,
                    cell_width: r.num("cellWidth").unwrap_or(0.0),
                    cell_height: r.num("cellHeight").unwrap_or(0.0),
                    columns: r.num("cellColumns").unwrap_or(0.0) as u32,
                    gap: r.num("gap").unwrap_or(0.0),
                    width: r.size("width").unwrap_or(Size::grow()),
                    height: r.size("height"),
                    padding,
                },
            )
        };
        let at = self.parent();
        if props != Props::default() {
            self.nodes[at].props = props;
        }
        for k in range.first..range.last.min(count) {
            self.structure_row(k as usize);
            let resp = if kind == "list" {
                self.list_item(k, false)
            } else {
                self.grid_item(k, false)
            };
            let row = format!(
                "{path}/{}",
                if kind == "list" {
                    crate::ui_idi("Row", k)
                } else {
                    crate::ui_idi("Cell", k)
                }
                .as_str()
            );
            if resp.clicked {
                b.event(&row, "select", id, json!(k));
            }
            self.build_item(
                b,
                &doc.nodes[f],
                var,
                items[k as usize].clone(),
                k as usize,
                origin.as_deref(),
            );
            self.close();
        }
        self.close_list();
    }

    fn art_spec(&mut self, path: &str, r: &Resolved) {
        let s = crate::slot::SlotSpec::from_props(
            path,
            |p| r.str(p).map(str::to_string),
            |p| r.nums(p).map(<[f32]>::to_vec),
        );
        self.note_art(path.to_string(), s);
    }

    /// A control's two-way prop: its binding's path takes the new value.
    fn write_back(&mut self, b: &mut Build<'_>, n: &DocNode, prop: &str, v: Value) {
        let bound = n.props.iter().rev().find(|p| p.name() == prop);
        if let Some(p) = bound
            && let PropValue::Model(path) | PropValue::Bind(crate::bind::Expr::Path(path), _) =
                &p.value
            && let Some(w) = b.write_path(path)
        {
            b.writes.push((w.clone(), v.clone()));
            b.event("", "set", &w, v.clone());
        }
        b.event("", "changed", prop, v);
    }

    /// Fires a node's triggers for what happened to it this tick.
    fn fire(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        key: u64,
        path: &str,
        resp: crate::ui::Response,
    ) {
        // Paths of the events this element pushed without one.
        for e in b.events.iter_mut().filter(|e| e.path.is_empty()) {
            e.path = path.to_string();
        }
        if resp.clicked {
            b.event(path, "click", &n.id, Value::Null);
        }
        // A recogniser on a container that is not interactive answers
        // through its gesture, not its response.
        let g = self.gesture_of(key);
        let (tap, double, long) = (
            resp.tapped || g.tapped,
            resp.double_tapped || g.double_tapped,
            resp.long_pressed || g.long_pressed,
        );
        for (on, kind) in [(tap, "tap"), (double, "doubleTap"), (long, "longPress")] {
            if on {
                b.event(path, kind, &n.id, Value::Null);
            }
        }
        let appeared = !self.docs.seen.contains(&key);
        for (k, t) in n.triggers.iter().enumerate() {
            let go = match &t.when {
                When::Click => resp.clicked,
                When::Changed => resp.changed,
                When::Appear => appeared,
                When::Tap => tap,
                When::DoubleTap => double,
                When::LongPress => long,
                When::Action(a) => self.docs.accepts && self.action_fired(a),
                When::Change(e) => {
                    let v = b.eval(e);
                    let old = self.docs.last.insert((key, k), v.clone());
                    old.is_some_and(|o| o != v)
                }
            };
            if !go {
                continue;
            }
            match &t.act {
                Act::Play(tl) => {
                    let name = qualify(&b.doc.name, tl);
                    let id = self.play_timeline(path, &name, crate::motion::PlayDesc::default());
                    self.docs.plays.insert((path.to_string(), tl.clone()), id);
                }
                Act::State(g, s) => {
                    self.docs.states.insert((key, g.clone()), s.clone());
                }
                Act::Emit(name) => b.event(path, "emit", name, Value::Null),
                Act::Push(screen) => self
                    .docs
                    .ops
                    .push(crate::layers::LayerOp::Push(screen.clone(), Value::Null)),
                Act::Pop => {
                    let l = self.docs.building;
                    self.docs.ops.push(crate::layers::LayerOp::Pop(l))
                }
                Act::Action(a) => {
                    self.fire_action(a);
                    b.event(path, "action", a, Value::Null);
                }
                Act::Set(p, e) => {
                    let v = b.eval(e);
                    if let Some(w) = b.write_path(p) {
                        b.writes.push((w.clone(), v.clone()));
                        b.event(path, "set", &w, v);
                    }
                }
            }
        }
    }
}

/// Whether `n`'s `prop` reads a store (`bind:` or `{$x}`) rather than a literal.
fn bound(n: &DocNode, prop: &str) -> bool {
    n.props
        .iter()
        .rev()
        .find(|p| p.name() == prop)
        .is_some_and(|p| !matches!(p.value, PropValue::Lit(_)))
}
