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
            NodeKind::For { var, list } => {
                let items = b.eval(list);
                let origin = match list {
                    crate::bind::Expr::Path(p) => b.write_path(p),
                    _ => None,
                };
                let items = items.as_array().cloned().unwrap_or_default();
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
                let r = self.mount_doc(cdoc, mount, id, &mut vm, b.env);
                // The instance's own element props apply to its root.
                if !n.props.is_empty()
                    && let Some(&last) = Some(&self.last_closed)
                    && last != NONE
                {
                    let mut built = Built::default();
                    let rp = self.resolve(b, n, &r.path, &mut built);
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
        let r = self.resolve(b, n, &path, &mut built);
        if r.has("visible") && !r.flag("visible") {
            return None;
        }
        self.docs.seen_now.insert(key);
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
        if container {
            match kind {
                "panel" => {
                    self.open_panel(
                        id.as_str(),
                        crate::ui::PanelDesc {
                            row: false,
                            width: r.size("width").unwrap_or_default(),
                            height: r.size("height").unwrap_or_default(),
                            ..Default::default()
                        },
                    );
                    let at = self.parent();
                    self.nodes[at].el = Ui::box_of(&r, self.nodes[at].el);
                    self.nodes[at].props = props;
                }
                "scroll" => {
                    self.open_scroll(
                        id.as_str(),
                        crate::ui::ScrollDesc {
                            width: r.size("width").unwrap_or(Size::grow()),
                            height: r.size("height").unwrap_or(Size::grow()),
                            ..Default::default()
                        },
                    );
                    let at = self.parent();
                    self.nodes[at].el = Ui::box_of(&r, self.nodes[at].el);
                    self.nodes[at].props = props;
                }
                "list" | "cells" => {
                    self.build_virtual(b, n, kind, &id, &r, props, &path);
                    self.docs.built.push(built);
                    self.fire(b, n, key, &path, resp);
                    return Some(path);
                }
                _ => {
                    let el = Ui::box_of(
                        &r,
                        Element {
                            direction: if kind == "row" {
                                Direction::Row
                            } else {
                                Direction::Column
                            },
                            ..Element::default()
                        },
                    );
                    self.open_with(id.as_str(), el, props);
                }
            }
            if kind == "slot" {
                let at = self.parent();
                self.docs.slots.insert(path.clone(), at);
                let short = format!("{}/{}", b.mount, n.id);
                self.docs.slots.entry(short).or_insert(at);
            }
            self.docs.built.push(built);
            for &c in &n.children {
                self.build_node(b, c, None);
            }
            resp = self.close();
            self.fire(b, n, key, &path, resp);
            return Some(path);
        }
        // A leaf: its widget, then its props and box over the widget's.
        match kind {
            "label" => {
                let text = text_of(&r, b);
                let desc = LabelDesc {
                    step: r.str("step").and_then(|s| {
                        crate::style_file::ramp_step(&mut t.type_ramp.clone(), s).copied()
                    }),
                    color: r.color("color"),
                    align: match r.str("textAlign") {
                        Some("center") => TextAlign::Center,
                        Some("end") => TextAlign::End,
                        _ => TextAlign::Start,
                    },
                    nowrap: r.flag("nowrap"),
                    width: r.size("width").unwrap_or_default(),
                    font: r
                        .str("font")
                        .and_then(FontRole::from_name)
                        .unwrap_or_default(),
                    heading: r.flag("heading"),
                    ellipsis: r.flag("ellipsis"),
                    max_lines: r.num("maxLines").unwrap_or(0.0) as u32,
                    fit_min: r.str("fitMin").and_then(|s| {
                        crate::style_file::ramp_step(&mut t.type_ramp.clone(), s).copied()
                    }),
                };
                self.label(id.as_str(), &text, desc);
            }
            "button" => {
                let text = text_of(&r, b);
                resp = self.button(
                    id.as_str(),
                    &text,
                    ButtonDesc {
                        variant: match r.str("variant") {
                            Some("primary") => ButtonVariant::Primary,
                            Some("quiet") => ButtonVariant::Quiet,
                            Some("danger") => ButtonVariant::Danger,
                            Some("ghost") => ButtonVariant::Ghost,
                            _ => ButtonVariant::Secondary,
                        },
                        size: match r.str("size") {
                            Some("small") => ControlSize::Small,
                            Some("large") => ControlSize::Large,
                            _ => ControlSize::Medium,
                        },
                        disabled: r.flag("disabled"),
                        width: r.size("width"),
                        autofocus: r.flag("autofocus"),
                        square: r.flag("square"),
                    },
                );
            }
            "art" => {
                // An art slot: its art fitted by its spec, or the style's
                // placeholder for its kind (docs/ui.md §13.1).
                self.art_spec(&path, &r);
                let spec = self
                    .docs
                    .arts
                    .last()
                    .map(|(_, s)| s.clone())
                    .expect("pushed");
                let texture = r.num("texture").and_then(|h| b.env.texture(f64::from(h)));
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
                let texture = r.num("texture").and_then(|h| b.env.texture(f64::from(h)));
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
                        label: r.str("name").map(str::to_string),
                    },
                );
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
                let mut v = r.num("value").unwrap_or(0.0);
                resp = self.slider(
                    id.as_str(),
                    &mut v,
                    crate::ui::SliderDesc {
                        min: r.num("min").unwrap_or(0.0),
                        max: r.num("max").unwrap_or(1.0),
                        step: r.num("stepBy").unwrap_or(0.0),
                        width: r.size("width"),
                    },
                );
                if resp.changed {
                    self.write_back(b, n, "value", json!(v));
                }
            }
            "toggle" => {
                let mut on = r.flag("checked");
                let text = text_of(&r, b);
                resp = self.toggle(id.as_str(), &mut on, &text);
                if resp.changed {
                    self.write_back(b, n, "checked", json!(on));
                }
            }
            "field" => {
                let mut s = r.str("input").unwrap_or_default().to_string();
                resp = self.text_field(
                    id.as_str(),
                    &mut s,
                    &crate::ui::FieldDesc {
                        placeholder: r.str("placeholder").unwrap_or_default().to_string(),
                        width: r.size("width"),
                        max_chars: r.num("maxChars").unwrap_or(0.0) as u32,
                        invalid: r.flag("invalid"),
                        bare: false,
                    },
                );
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
            let mut el = self.nodes[last].el;
            // The widget's own box stays where the document says nothing.
            for name in [
                "height",
                "padding",
                "gap",
                "align",
                "cross",
                "background",
                "radius",
                "frame",
                "aspect",
                "interactive",
                "focusable",
                "adjustable",
                "wrap",
            ] {
                if r.has(name) {
                    el = Ui::box_of(
                        &Resolved {
                            vals: r
                                .vals
                                .iter()
                                .filter(|(s, _)| PROPS[*s].name == name)
                                .cloned()
                                .collect(),
                        },
                        el,
                    );
                }
            }
            if r.has("width")
                && !matches!(
                    kind,
                    "label" | "button" | "image" | "art" | "progress" | "slider" | "field"
                )
            {
                el.width = r.size("width").unwrap_or(el.width);
            }
            self.nodes[last].el = el;
            if props != Props::default() {
                let mut p = props;
                // A widget's own states (a button's interaction) stay.
                let own = std::mem::take(&mut self.nodes[last].props.states);
                p.states.extend(
                    own.into_iter()
                        .filter(|g| !n.groups.iter().any(|x| x.name == g.name)),
                );
                self.nodes[last].props = p;
            }
        }
        self.docs.built.push(built);
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
        let doc = b.doc.clone();
        for &c in &n.children {
            // Each item's elements are told apart by its index (`Item#3`).
            let id = match &doc.nodes[c].kind {
                NodeKind::Element(kind) if doc.nodes[c].id.is_empty() => {
                    Some(format!("{kind}#{k}"))
                }
                NodeKind::Element(_) | NodeKind::Use(_) => Some(format!("{}#{k}", doc.nodes[c].id)),
                _ => None,
            };
            self.build_node(b, c, id.as_deref());
        }
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
        let NodeKind::For { var, list } = &doc.nodes[f].kind else {
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
        let kind = r
            .str("kind")
            .and_then(crate::slot::SlotKind::from_name)
            .unwrap_or(crate::slot::SlotKind::ALL[0]);
        let mut s = crate::slot::SlotSpec::new(path, kind);
        if let Some(n) = r.nums("ratio").filter(|n| n.len() == 2) {
            s.aspect =
                crate::slot::SlotAspect::One(crate::slot::Aspect::new(n[0] as u16, n[1] as u16));
        }
        if let Some(n) = r.nums("focal").filter(|n| n.len() == 2) {
            s.focal = [n[0], n[1]];
        }
        if let Some(n) = r.nums("safe").filter(|n| n.len() == 4) {
            s.safe = Some([n[0], n[1], n[2], n[3]]);
        }
        if let Some(f) = r.str("fit") {
            s.fit = match f {
                "contain" => crate::slot::Fit::Contain,
                "fill" => crate::slot::Fit::Fill,
                "nineSlice" => crate::slot::Fit::NineSlice,
                "tile" => crate::slot::Fit::Tile,
                _ => crate::slot::Fit::Cover,
            };
        }
        self.docs.arts.push((path.to_string(), s));
    }

    /// A control's two-way prop: its binding's path takes the new value.
    fn write_back(&mut self, b: &mut Build<'_>, n: &DocNode, prop: &str, v: Value) {
        let bound = n.props.iter().rev().find(|p| p.name() == prop);
        if let Some(p) = bound
            && let PropValue::Bind(crate::bind::Expr::Path(path), _) = &p.value
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
                Act::Pop => self.docs.ops.push(crate::layers::LayerOp::Pop),
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
