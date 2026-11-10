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
            // An immediate build makes every element each tick: a key block
            // is its content.
            NodeKind::Key(_) => {
                for &c in &n.children {
                    self.build_node(b, c, None);
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
                for p in &cdoc.props {
                    if let Some(default) = &p.default {
                        props
                            .entry(p.name.clone())
                            .or_insert_with(|| default.clone());
                    }
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
                    let el = Ui::box_of(&rp, self.nodes[last].el, self.em_of(&rp));
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
        let mut r = self.resolve(b, n, &path, &mut built, &b.inherited, &mut |s| {
            self.structure_matches(&doc, s)
        });
        r.ordinal = self.ordinal_of(n, &r, |filter, item_values| {
            self.structure_ordinal(filter, item_values)
        });
        self.warnings.extend(
            r.cascade
                .diagnostics
                .iter()
                .map(|error| format!("{}:{} ({path}): {error}", doc.source, n.line)),
        );
        let structural = children.is_some();
        self.docs.levels.extend(children);
        self.run_enter(&r);
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
        self.run_exit();
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
        if let Some(commit) = &r.style {
            self.commit_styles(commit);
        }
        let keeps = std::mem::take(&mut self.docs.keeps_hidden);
        if r.has("visible") && !r.flag("visible") && r.cascade.container_reads.is_empty() && !keeps
        {
            return None;
        }
        self.docs.seen_now.insert(key);
        self.note_selection(key, &r);
        self.remember_queried(b, i, &path, key, &r);
        self.docs
            .scroll_now
            .extend(r.cascade.scroll_reads.iter().copied());
        if let Some(container) = r.container {
            self.build_container(container);
        }
        let props = self.props_of(b, n, key, &r, &mut built);
        if kind == "viewport" {
            return self.build_viewport(b, n, id.as_str(), &r, props, path, key, built);
        }
        if kind == "anchor" {
            return self.build_anchor(b, n, &r, props, path, key, built, build_children);
        }
        if kind == "ground" || (kind == "panel" && r.flag("world")) {
            return self.build_world_panel(b, n, &r, props, path, key, built, build_children);
        }
        if svg::KINDS.contains(&kind) {
            return self.build_svg(
                b,
                n,
                kind,
                id.as_str(),
                &r,
                props,
                path,
                key,
                built,
                build_children,
            );
        }
        if r.has("popover") {
            return self.build_popover(b, n, id.as_str(), &r, path, key, built, build_children);
        }
        let t = self.theme;
        let raw_of = |r: &Resolved, b: &Build<'_>| -> String {
            if let Some(k) = r.str("key")
                && let Some(s) = b.env.translate(k).map(|t| b.env.shown(t))
            {
                // Its markup styles runs; the text is without it.
                if crate::doc::mf2::has_markup(&s) {
                    crate::doc::mf2::plain(&s)
                } else {
                    s
                }
            } else {
                r.str("text").unwrap_or_default().to_string()
            }
        };
        // CSS's text-transform and white-space processing over its source.
        let text_of = |r: &Resolved, b: &Build<'_>| -> String {
            let raw = raw_of(r, b);
            match r.text.as_deref() {
                Some(t) => t.shown(&raw, b.env.language()),
                None => raw,
            }
        };
        let mut resp = crate::ui::Response::default();
        // A button with only text is the leaf control; with content it holds it.
        let container = schema::kind(kind).is_some_and(|k| k.container)
            && (kind != "button" || !n.children.is_empty());
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
            backdrop: None,
            closedby: match r.str("closedby") {
                Some("any") => crate::widgets::ClosedBy::Any,
                Some("closerequest") => crate::widgets::ClosedBy::CloseRequest,
                Some("none") => crate::widgets::ClosedBy::None,
                _ => crate::widgets::ClosedBy::Auto,
            },
            bare: r.str("appearance") == Some("none"),
        };
        let mut opened: Option<bool> = None;
        // A spotlight's callout layer, open (closed by `close`).
        let mut spot = false;
        // A closed dialog or disclosure builds no content (HTML renders
        // none): opened, its controls are built fresh and take input.
        let mut shows_content = true;
        if container {
            match kind {
                "dialog" if open_now(self) && r.str("spotlight").is_some() => {
                    let target = r.str("spotlight").unwrap_or_default();
                    let (shade, anchored) =
                        self.open_spotlight(id.as_str(), (key, &path), target, &r);
                    spot = anchored;
                    if shade.clicked && r.str("closedby") == Some("any") {
                        opened = Some(false);
                    }
                }
                "dialog" => {
                    let open = open_now(self);
                    let out = self.open_dialog(id.as_str(), open, &dialog);
                    // `::backdrop` styles the scrim, the panel's parent.
                    if dialog.modal
                        && let Some(backdrop) = self.pseudo_node(b, &r, &path, "backdrop")
                    {
                        let scrim = self.nodes[self.parent()].parent;
                        self.style_pseudo_box(b, scrim, "backdrop", &backdrop);
                    }
                    shows_content = out.visible;
                    if out.dismissed {
                        opened = Some(false);
                    }
                    if dialog.modal
                        && let Some(d) = r.str("dataDetents")
                        && self.sheet(
                            id.as_str(),
                            d,
                            r.str("dataEdge")
                                .and_then(|e| super::gestures::sheet_edge(e).ok())
                                .unwrap_or_default(),
                            open,
                        )
                    {
                        opened = Some(false);
                    }
                }
                "details" => {
                    let mut open = open_now(self);
                    // An exclusive accordion (`<details name>`): the open one
                    // of its name in this mount is widget memory; another
                    // open one closes.
                    let group = r
                        .str("exclusive")
                        .filter(|g| !g.is_empty())
                        .map(|g| crate::id::child_key(crate::id::child_key(0, &b.mount), g));
                    let holder = group.map_or(0, |g| self.anim(g).h);
                    let closing = open && holder != 0 && holder != key;
                    open &= !closing;
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
                    if out.toggled || closing {
                        opened = Some(out.open);
                    }
                    if let Some(g) = group {
                        let now = if out.open {
                            key
                        } else if holder == key {
                            0
                        } else {
                            holder
                        };
                        if now != holder || (out.open && !self.has_anim(g)) {
                            self.set_anim(
                                g,
                                crate::ui::Anim {
                                    h: now,
                                    ..Default::default()
                                },
                            );
                        }
                        // The others of its name build again next tick, and close.
                        if out.toggled && out.open {
                            let name = r.str("exclusive").unwrap_or_default();
                            self.actions.pending.push(details_action(name));
                        }
                    }
                }
                "button" => {
                    resp = self.open_button(id.as_str(), r.button_desc());
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
                            fields::RenderFields::capture(self, key, at, kind, resp, props, None)
                                .with_textures(&r, b.env);
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
                fields::RenderFields::capture(self, key, self.parent(), kind, resp, props, None)
                    .with_textures(&r, b.env);
            self.apply_document_fields(&doc, n, &r, &fields);
            self.note_render_fields(key, fields);
            // A content button feeds og-input's action as a text one does.
            if kind == "button" && r.has("dataOgTouch") {
                self.note_touch_button(&r);
            }
            if kind == "box"
                && n.selector_tag.as_deref() == Some("li")
                && let Some(marker) = r.str("markerType")
            {
                // `::marker { content }` replaces the list style's text;
                // its color inks either.
                let (content, ink) = self.marker_override(b, n, &r);
                let style = crate::doc::lists::marker(marker, r.ordinal.unwrap_or(1));
                let inside = r.str("markerPosition") == Some("inside");
                // `list-style-image`: the picture stands for the marker.
                let picture = r
                    .str("markerImage")
                    .and_then(|v| crate::doc::lists::image(v).ok().flatten());
                if let Some(name) = picture.filter(|_| content.is_none()) {
                    let text = if inside { "\u{FFFC}\u{a0}" } else { "\u{FFFC}" };
                    let span = crate::TextSpan {
                        name: "marker".into(),
                        position: 0,
                        length: 1,
                        properties: Default::default(),
                    };
                    let style = crate::TextSpanStyle {
                        atom: Some(crate::rich::InlineAtom::Image(name)),
                        ..Default::default()
                    };
                    self.rich_text_spans(
                        "Marker",
                        text,
                        std::slice::from_ref(&span),
                        crate::RichTextDesc::default(),
                        |_| style.clone(),
                    );
                } else {
                    match content {
                        Some(Some(text)) => self.list_marker(&path, "decimal", &text, inside, ink),
                        Some(None) => {}
                        None => {
                            if let Some(text) = style {
                                self.list_marker(&path, marker, &text, inside, ink);
                            }
                        }
                    }
                }
            }
            if let Some(id) = r.str("rive") {
                let at = self.parent();
                self.play_rive(at, id, &path);
            }
            if r.has("emitter") || r.has("meta") || self.world.kept_effects.contains_key(&key) {
                let at = self.parent();
                self.world_effects(b, n, &r, at, &path);
            }
            if kind == "slot" {
                let at = self.parent();
                self.docs.slots.insert(path.clone(), at);
                let short = format!("{}/{}", b.mount, n.id);
                self.docs.slots.entry(short).or_insert(at);
            }
            self.note_built(built);
            if shows_content {
                // Its filters' `feImage`s of elements, as its first children.
                let at = self.parent();
                let css = self.nodes[at].decor.as_ref().and_then(|d| d.css.as_ref());
                let images = css.is_some_and(|c| {
                    crate::paint::filter_uses(&c.effects, c.layer.as_ref(), &c.filters)
                        .iter()
                        .any(|u| !u.filter.elements.is_empty())
                });
                if images && let Some(c) = css.cloned() {
                    let uses = crate::paint::filter_uses(&c.effects, c.layer.as_ref(), &c.filters);
                    {
                        self.build_element_images(
                            b,
                            n,
                            &r.cascade.computed,
                            &uses,
                            super::svg::ImageSpace::Css,
                        );
                    }
                }
                let inherited = std::mem::replace(&mut b.inherited, r.cascade.computed.clone());
                self.generated_box(b, n, &r, &path, true);
                build_children(self, b, None);
                self.generated_box(b, n, &r, &path, false);
                b.inherited = inherited;
            }
            match kind {
                "dialog" if spot => {
                    self.close();
                }
                "dialog" => self.close_dialog(&dialog),
                "details" => self.close_details(),
                "button" => self.close_button(),
                _ => resp = self.close(),
            }
            if let Some(open) = opened {
                self.docs.happened.push(When::Toggle);
                if kind == "dialog" && !open {
                    self.docs.happened.push(When::Close);
                }
                if !bound(n, "open") {
                    self.remember(
                        &format!("{}~doc-open", id.as_str()),
                        f64::from(u8::from(open)),
                    );
                }
                self.write_back(b, n, "open", json!(open));
            }
            self.popover_invoker(b, &r, key, resp);
            self.hold(key, n, &r, self.last_closed);
            self.fire(b, n, key, &path, resp);
            return Some(path);
        }
        // Keep the widget's intrinsic box separate from CSS-owned fields.
        // Settlement can replace those fields without processing its input again.
        let styled = r;
        let r = styled.intrinsic();
        let mut texture = None;
        self.accent = styled.text.as_deref().and_then(|t| t.accent);
        match kind {
            "label" => {
                let text = text_of(&r, b);
                let desc = r.label_desc(&t, r.own_flow().rtl);
                if !n.inline.is_empty() {
                    self.inline_links(b, n, &r, &path, key);
                }
                // `::before`, `::after` and `::first-letter` join the text
                // as styled runs.
                if let Some((full, runs)) = self.generated_text(b, n, &styled, key, &text) {
                    let attributes: Vec<crate::TextSpan> =
                        runs.iter().map(|(span, _)| span.clone()).collect();
                    resp = self
                        .rich_text_spans(
                            id.as_str(),
                            &full,
                            &attributes,
                            crate::RichTextDesc {
                                step: desc.step,
                                color: desc.color,
                                align: desc.align,
                                nowrap: desc.nowrap,
                                width: desc.width,
                                font: desc.font,
                            },
                            |span| {
                                runs.iter()
                                    .find(|(s, _)| s == span)
                                    .map(|(_, style)| style.clone())
                                    .unwrap_or_default()
                            },
                        )
                        .response;
                } else if let Some(V::Spans(spans)) = r.get("spans") {
                    let spans = match spans {
                        Ok(spans) => spans,
                        Err(error) => {
                            self.warnings.push(format!(
                                "{}:{}: {path}.spans: expected TextSpan[]: {error}",
                                doc.source, n.line
                            ));
                            self.accent = None;
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
                } else if let Some((span, style)) = r.css_span(&text) {
                    resp = self
                        .rich_text_spans(
                            id.as_str(),
                            &text,
                            std::slice::from_ref(&span),
                            crate::RichTextDesc {
                                step: desc.step,
                                color: desc.color,
                                align: desc.align,
                                nowrap: desc.nowrap,
                                width: desc.width,
                                font: desc.font,
                            },
                            |_| style.clone(),
                        )
                        .response;
                } else {
                    if let Some(count) = styled.num("count") {
                        self.count_text(id.as_str(), count, &desc, &styled);
                    } else {
                        let shown = self.reveal(key, &text, styled.num("reveal"));
                        let i = self.leaf("label", id.as_str().into(), Default::default());
                        self.apply_label(i, shown, desc);
                        self.css_text_content(i, &styled);
                    }
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
                let mut spec = self.last_art().clone();
                let (at, nearest) = r.object_css();
                if let Some([focal, _]) = at {
                    spec.focal = focal;
                }
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
                        offset: at.map_or([0.0; 2], |a| a[1]),
                        nearest,
                    },
                );
            }
            "image" => {
                texture = r.handle("texture").and_then(|h| b.env.texture(h));
                let i = self.leaf("image", id.as_str().into(), Default::default());
                self.apply_image(
                    i,
                    crate::ui::ImageDesc {
                        texture,
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
                self.object_fit(i, &r);
            }
            "icon" => {
                let name = match (r.str("icon"), r.str("cue")) {
                    (Some(icon), _) => icon.to_string(),
                    (None, Some(token)) => self.cue_icon(token).unwrap_or_default(),
                    (None, None) => String::new(),
                };
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
            "select" if r.flag("dataCycle") => {
                let changed = self.cycler(b, n, id.as_str(), &r);
                resp = crate::Response {
                    changed,
                    ..crate::Response::default()
                };
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
                            .map(|t| b.env.shown(t))
                            .unwrap_or_else(|| item.clone()),
                        group: groups.get(k).cloned().unwrap_or_default(),
                        disabled: off.iter().any(|o| o == &k.to_string()),
                        ..crate::widgets::DropdownOption::default()
                    })
                    .collect();
                let base = crate::widgets::DropdownDesc::default();
                // `::picker(select)` styles the open list; its `max-height`
                // bounds the rows before they scroll.
                let picker = self
                    .pseudo_node(b, &styled, &path, "picker")
                    .map(|(at, mut p)| {
                        let max = p.values.size("maxHeight");
                        if let Some((spec, _)) = schema::prop("maxHeight") {
                            p.values.vals.retain(|(s, _)| *s != spec);
                        }
                        (max, (at, p))
                    });
                let max_height = match picker.as_ref().and_then(|(max, _)| *max) {
                    Some(Size::Fixed(px)) => px,
                    _ => base.max_height,
                };
                let build: &Build<'_> = b;
                let out = self.dropdown_marked(
                    id.as_str(),
                    &crate::widgets::DropdownDesc {
                        options,
                        selected,
                        placeholder: r.str("placeholder").unwrap_or_default().to_string(),
                        disabled: r.flag("disabled"),
                        width: r.size("width"),
                        max_height,
                        searchable: match styled.str("selectFilter") {
                            Some("on") => Some(true),
                            Some("off") => Some(false),
                            _ => None,
                        },
                        search_placeholder: r
                            .str("dataFilterPlaceholder")
                            .unwrap_or_default()
                            .to_string(),
                        ..base
                    },
                    r.flag("invalid"),
                    &mut |ui, list| {
                        if let Some((_, p)) = &picker {
                            ui.style_pseudo_box(build, list, "picker", p);
                        }
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
            "slider" if r.has("high") => {
                let out = self.range_slider(
                    id.as_str(),
                    (
                        r.num("low").unwrap_or(f32::MIN),
                        r.num("high").unwrap_or(f32::MAX),
                    ),
                    &crate::widgets::RangeSliderDesc {
                        min: r.num("min").unwrap_or(0.0),
                        max: r.num("max").unwrap_or(1.0),
                        step: r.num("stepBy").unwrap_or(0.0),
                        min_gap: r.num("dataMinGap").unwrap_or(0.0),
                        label: text_of(&r, b),
                        show_value: !r.has("showValue") || r.flag("showValue"),
                        width: r.size("width"),
                        disabled: r.flag("disabled"),
                        ..crate::widgets::RangeSliderDesc::default()
                    },
                );
                resp.changed = out.changed;
                if out.changed {
                    self.write_back(b, n, "low", json!(out.low));
                    self.write_back(b, n, "high", json!(out.high));
                }
            }
            "slider" => {
                let v0 = r.num("value").unwrap_or(0.0);
                // Its `<datalist>`'s options: a tick at each value.
                let marks: Vec<crate::widgets::SliderMark> =
                    match (r.get("markValues"), r.get("markLabels")) {
                        (Some(V::Strs(values)), labels) => values
                            .iter()
                            .enumerate()
                            .filter_map(|(k, v)| {
                                Some(crate::widgets::SliderMark {
                                    value: v.parse().ok()?,
                                    label: match labels {
                                        Some(V::Strs(l)) => l.get(k).cloned().unwrap_or_default(),
                                        _ => String::new(),
                                    },
                                })
                            })
                            .collect(),
                        _ => Vec::new(),
                    };
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
                        marks,
                        origin: r.num("dataFillOrigin"),
                        disabled: r.flag("disabled"),
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
                // Source order, swapped by the label's row-reverse.
                let side = if (r.str("side") == Some("end"))
                    != (styled.str("labelDirection") == Some("rowReverse"))
                {
                    crate::widgets::LabelSide::End
                } else {
                    crate::widgets::LabelSide::Start
                };
                let out = if r.flag("switch") {
                    self.toggle_switch(
                        id.as_str(),
                        on,
                        &crate::widgets::ToggleSwitchDesc {
                            label: text_of(&r, b),
                            description: r.str("description").unwrap_or_default().to_string(),
                            disabled: r.flag("disabled"),
                            side,
                            size: styled.control_size(),
                            width: r.size("width"),
                            on_label: r.str("dataOn").unwrap_or_default().to_string(),
                            off_label: r.str("dataOff").unwrap_or_default().to_string(),
                        },
                    )
                } else {
                    // HTML's checkbox: a box with a check (a dash when
                    // indeterminate).
                    self.check_field(
                        id.as_str(),
                        on,
                        &crate::widgets::CheckFieldDesc {
                            label: text_of(&r, b),
                            description: r.str("description").unwrap_or_default().to_string(),
                            mixed: r.flag("indeterminate"),
                            disabled: r.flag("disabled"),
                            invalid: r.flag("invalid"),
                            side,
                            width: r.size("width"),
                        },
                    )
                };
                resp = crate::Response {
                    changed: out.changed,
                    ..out.response
                };
                if out.changed {
                    self.write_back(b, n, "checked", json!(out.on));
                }
            }
            "color" => {
                resp = self.color_input(b, n, id.as_str(), &r);
            }
            "video" | "audio" => {
                resp = self.media_element(b, n, id.as_str(), key, &r, kind == "video");
            }
            "file" => {
                if !self.file_picker {
                    self.warnings.push(format!(
                        "{}:{}: <input type=\"file\"> needs a host file picker, which this host lacks (og.ui.filePicker is false); take files in a TS system",
                        doc.source, n.line
                    ));
                    return None;
                }
                let chosen = r.str("fileValue").unwrap_or_default();
                let text = match text_of(&r, b) {
                    t if !t.is_empty() => t,
                    _ if !chosen.is_empty() => chosen.to_string(),
                    _ => "Choose file…".to_string(),
                };
                resp = self.button(id.as_str(), &text, r.button_desc());
                if resp.clicked && !r.flag("disabled") {
                    b.event(
                        &path,
                        "filepick",
                        &n.id,
                        json!({ "accept": r.str("accept").unwrap_or_default(), "multiple": r.flag("multiple") }),
                    );
                }
            }
            "datetime" => {
                resp = self.datetime_input(b, n, id.as_str(), &r);
                let barred = r.flag("disabled") || r.flag("readonly");
                self.docs.validity.insert(
                    key,
                    super::mount::FieldValidity {
                        mount: b.mount.clone(),
                        doc: doc.name.clone(),
                        node: i,
                        row_key: b.row_key.clone(),
                        invalid: !barred && (r.flag("invalid") || super::datetime::invalid(&r)),
                    },
                );
            }
            "radio" => {
                let choice = r.str("choice").unwrap_or("on").to_string();
                let on = if r.has("group") {
                    r.str("group") == Some(choice.as_str())
                        || r.num("group").is_some_and(|g| g.to_string() == choice)
                } else {
                    r.flag("checked")
                };
                let out = self.radio_group(
                    id.as_str(),
                    &crate::widgets::RadioGroupDesc {
                        options: vec![crate::widgets::RadioOption {
                            label: text_of(&r, b),
                            description: r.str("description").unwrap_or_default().to_string(),
                            ..Default::default()
                        }],
                        selected: if on { 0 } else { -1 },
                        disabled: r.flag("disabled"),
                        width: r.size("width"),
                        ..Default::default()
                    },
                );
                resp = crate::Response {
                    changed: out.changed,
                    ..crate::Response::default()
                };
                if out.changed && out.selected == 0 {
                    self.write_back(b, n, "group", json!(choice));
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
                    .map(|(i, item)| {
                        keys.get(i)
                            .and_then(|k| b.env.translate(k))
                            .map(|t| b.env.shown(t))
                            .unwrap_or(item)
                    })
                    .collect();
                // With values, `current` holds the selected tab's name (a screen's mode) instead of `selected`'s index.
                let values = strs("values");
                let mut selected = if values.is_empty() {
                    r.num("selected").unwrap_or(0.0) as usize
                } else {
                    let now = r.str("current").unwrap_or_default();
                    values.iter().position(|v| v == now).unwrap_or(0)
                };
                // The bumpers: tabPrev and tabNext (Q and E, LB and RB) step
                // through the tabs from anywhere on the page, wrapping.
                let step = i32::from(self.action("tabNext")) - i32::from(self.action("tabPrev"));
                let stepped = step != 0 && items.len() > 1 && !r.flag("disabled");
                if stepped {
                    selected = (selected as i32 + step).rem_euclid(items.len() as i32) as usize;
                }
                let fill = r.flag("fill");
                let (icons, badges) = (strs("icons"), strs("badges"));
                let out = self.segmented(
                    id.as_str(),
                    &crate::widgets::SegmentedDesc {
                        items: items
                            .iter()
                            .enumerate()
                            .map(|(i, l)| crate::widgets::SegmentItem {
                                label: l.clone(),
                                icon: icons.get(i).cloned().unwrap_or_default(),
                                badge: badges.get(i).cloned().unwrap_or_default(),
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
                        size: styled.control_size(),
                        disabled: r.flag("disabled"),
                    },
                );
                selected = usize::try_from(out.selected).unwrap_or(0);
                resp = crate::Response {
                    changed: out.changed || stepped,
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
                let input_type = r
                    .str("type")
                    .and_then(crate::InputType::from_name)
                    .unwrap_or_default();
                let constraints = crate::input_type::Constraints {
                    required: r.flag("required"),
                    min_chars: r.num("minChars").unwrap_or(0.0) as u32,
                    pattern: r.str("pattern").map(str::to_string),
                    min: r.num("min").map(f64::from),
                    max: r.num("max").map(f64::from),
                    step: r.num("stepBy").map(f64::from),
                };
                resp = if styled.str("stepper") == Some("buttons")
                    && r.str("type") == Some("number")
                {
                    let out = self.number_stepper(
                        id.as_str(),
                        s.trim().parse().unwrap_or(0.0),
                        &crate::widgets::NumberStepperDesc {
                            min: r.num("min"),
                            max: r.num("max"),
                            step: r.num("stepBy").filter(|s| *s > 0.0).unwrap_or(1.0),
                            large_step: styled.num("stepperLargeStep").unwrap_or(0.0),
                            wrap: styled.str("stepperWrap") == Some("wrap"),
                            format: crate::widgets::ValueFormat {
                                message: r
                                    .str("dataFormat")
                                    .filter(|f| !f.is_empty())
                                    .and_then(|f| og_i18n::Message::parse(f).ok()),
                                locale: self.locale.clone(),
                                ..Default::default()
                            },
                            disabled: r.flag("disabled") || r.flag("readonly"),
                            width: r.size("width"),
                            ..Default::default()
                        },
                    );
                    if out.changed {
                        s = out.value.to_string();
                    }
                    crate::Response {
                        changed: out.changed,
                        ..crate::Response::default()
                    }
                } else if r.flag("multiline") {
                    self.docs.placeholder_ink = self.pseudo_ink(&styled);
                    let out = self.text_area(
                        id.as_str(),
                        &mut s,
                        &crate::ui::TextAreaDesc {
                            placeholder: r.str("placeholder").unwrap_or_default().to_string(),
                            width: r.size("width"),
                            rows: r.num("lines").unwrap_or(0.0) as u32,
                            max_chars: r.num("maxChars").unwrap_or(0.0) as u32,
                            invalid: r.flag("invalid"),
                            bare: styled.str("appearance") == Some("none"),
                            tight: r.flag("tight"),
                            component: r.str("component").map(str::to_string),
                            readonly: r.flag("readonly"),
                            constraints: constraints.clone(),
                        },
                    );
                    self.docs.placeholder_ink = None;
                    out
                } else {
                    self.docs.placeholder_ink = self.pseudo_ink(&styled);
                    let out = self.text_field(
                        id.as_str(),
                        &mut s,
                        &crate::ui::FieldDesc {
                            placeholder: r.str("placeholder").unwrap_or_default().to_string(),
                            width: r.size("width"),
                            max_chars: r.num("maxChars").unwrap_or(0.0) as u32,
                            invalid: r.flag("invalid"),
                            bare: styled.str("appearance") == Some("none"),
                            tight: r.flag("tight"),
                            component: r.str("component").map(str::to_string),
                            input_type,
                            inputmode: r.str("inputmode").and_then(crate::InputMode::from_name),
                            autocomplete: r.str("autocomplete").unwrap_or_default().to_string(),
                            readonly: r.flag("readonly"),
                            constraints: constraints.clone(),
                        },
                    );
                    self.docs.placeholder_ink = None;
                    out
                };
                if styled.str("stepper") != Some("buttons") && !r.flag("readonly") {
                    self.pad_keyboard(&path, &r, &mut s, &mut resp);
                }
                if let Some(V::Strs(all)) = r.get("suggestions") {
                    let field = self.last_closed;
                    if let Some(c) =
                        self.suggestions(&path, field, (resp.focused, resp.changed), &s, all)
                    {
                        s = c;
                        resp.changed = true;
                    }
                }
                if resp.changed {
                    self.write_back(b, n, "input", json!(s));
                }
                // HTML bars disabled and read-only controls from validation.
                let barred = r.flag("disabled") || r.flag("readonly");
                self.docs.validity.insert(
                    key,
                    super::mount::FieldValidity {
                        mount: b.mount.clone(),
                        doc: doc.name.clone(),
                        node: i,
                        row_key: b.row_key.clone(),
                        invalid: !barred
                            && (r.flag("invalid")
                                || input_type.mismatch(&s)
                                || constraints.fails(input_type, &s)),
                    },
                );
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
            "kbd" => {
                resp = self.kbd(b, n, id.as_str(), &r, text_of(&r, b));
            }
            "spacer" => {
                let s = r.size("width").or(r.size("height"));
                self.spacer(id.as_str(), s);
            }
            "divider" => self.divider(id.as_str(), r.flag("vertical")),
            "joystick" | "dpad" => {
                resp = self.build_touch_control(kind, id.as_str(), &r, &styled);
            }
            other => {
                self.warnings
                    .push(format!("{}:{}: '{other}' is not built", doc.source, n.line));
                self.accent = None;
                return None;
            }
        }
        self.accent = None;
        let last = self.last_closed;
        if last != NONE && last < self.nodes.len() {
            let mut fields =
                fields::RenderFields::capture(self, key, last, kind, resp, props, texture)
                    .with_textures(&styled, b.env);
            // A text keeps its source: settlement shows it in its new style.
            if kind == "label" && styled.text.is_some() && styled.generated.is_none() {
                fields.text = raw_of(&styled, b);
            }
            self.apply_document_fields(&doc, n, &styled, &fields);
            self.note_render_fields(key, fields);
            if styled.has("dataOgTouch") {
                self.note_touch_button(&styled);
            }
            if styled.has("emitter")
                || styled.has("meta")
                || self.world.kept_effects.contains_key(&key)
            {
                self.world_effects(b, n, &styled, last, &path);
            }
        }
        self.popover_invoker(b, &styled, key, resp);
        self.note_built(built);
        self.hold(key, n, &styled, last);
        self.fire(b, n, key, &path, resp);
        Some(path)
    }

    /// An `<li>`'s marker: hung before its start edge (`outside`, CSS's
    /// default: anchored to the item, so it costs the item no room), or
    /// first in its row (`inside`). A disc, circle or square is a shape on
    /// the first line, as browsers paint it, so no font has to carry the
    /// bullet; a number is text.
    fn list_marker(
        &mut self,
        item: &str,
        kind: &str,
        text: &str,
        inside: bool,
        ink: Option<Srgba>,
    ) {
        let t = self.theme;
        let body = t.type_ramp.body;
        // CSS's gap between marker and content: half an em.
        let gap = (body.size * 0.5).round();
        if crate::doc::lists::numbered(kind) {
            let text = if inside {
                format!("{text}\u{a0}")
            } else {
                text.to_string()
            };
            self.label(
                "Marker",
                &text,
                crate::LabelDesc {
                    color: ink,
                    ..crate::LabelDesc::default()
                },
            );
        } else {
            // Blink's bullet: a third of an em across, at least 3 px.
            let d = (body.size / 3.0).round().max(3.0);
            let ink = ink.unwrap_or(t.palette.ink);
            self.open(
                "Marker",
                Element {
                    width: Size::px(if inside { d + gap } else { d }),
                    height: Size::px(body.line_height),
                    cross: crate::layout::Cross::Center,
                    ..Element::default()
                },
            );
            self.open(
                kind,
                Element {
                    width: Size::px(d),
                    height: Size::px(d),
                    background: (kind == "square").then_some(ink),
                    ..Element::default()
                },
            );
            if kind != "square" {
                use crate::paint::{
                    Cap, ColorRef, ElementDraw, Paint, ShapeKind, Stroke, StrokeAlign,
                };
                let paint = Paint::Solid(ColorRef::Literal(ink));
                self.set_draw(ElementDraw {
                    shape: ShapeKind::Ellipse,
                    fills: if kind == "disc" {
                        vec![paint.clone()]
                    } else {
                        Vec::new()
                    },
                    strokes: if kind == "circle" {
                        vec![Stroke {
                            width: 1.0,
                            align: StrokeAlign::Inside,
                            dash: [0.0; 2],
                            dash_offset: 0.0,
                            dash_cap: Cap::Butt,
                            paint,
                        }]
                    } else {
                        Vec::new()
                    },
                    ..Default::default()
                });
            }
            self.close();
            self.close();
        }
        if inside {
            return;
        }
        let at = self.last_closed;
        let props = &mut self.nodes[at].props;
        props.position = crate::element::Position::Anchored;
        props.anchor_to = item.to_string();
        props.anchor_at = crate::layout::Anchor::TopLeft;
        props.anchor_self = crate::layout::Anchor::TopRight;
        props.insets.start = Some(-gap);
        props.overlaps.push(item.to_string());
    }

    /// A text's inline `<a href>`s: a click sends a `link` event named by
    /// its href, through the document's events as `onclick` does.
    fn inline_links(&mut self, b: &mut Build<'_>, n: &DocNode, r: &Resolved, path: &str, key: u64) {
        let ids: Vec<String> = n
            .inline
            .iter()
            .filter_map(|run| run.link)
            .map(super::inline_link)
            .fold(Vec::new(), |mut v, l| {
                if !v.contains(&l) {
                    v.push(l);
                }
                v
            });
        if let Some(l) = self.respond_links(key, &ids).link {
            let href = match l.strip_prefix("link").and_then(|k| k.parse::<usize>().ok()) {
                Some(k) => n.links.get(k).cloned().unwrap_or_default(),
                None => r.str("href").unwrap_or_default().to_string(),
            };
            if let Some(id) = href.strip_prefix('#') {
                self.docs.target = (!id.is_empty()).then(|| id.to_string());
            }
            b.event(path, "link", &href, Value::Null);
        }
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
    pub(super) fn write_back(&mut self, b: &mut Build<'_>, n: &DocNode, prop: &str, v: Value) {
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

    /// The on-screen keyboard of the text input at `path`: it opens when a
    /// pad gives the input the focus (unless `inputmode="none"`, the
    /// game's own keyboard), edits `s` in place, and Done submits; Done or
    /// Back return the focus to the input, which opens it again only once
    /// it has lost the focus. Its layout follows `type` and `inputmode`,
    /// `enterkeyhint` labels Done and `lang="ja"` offers kana.
    fn pad_keyboard(
        &mut self,
        path: &str,
        r: &Resolved,
        s: &mut String,
        resp: &mut crate::Response,
    ) {
        use crate::widgets::KeyboardLayout as L;
        let id = format!("{path}~keyboard");
        let (open_key, off_key, field_key) = (
            format!("{id}~open"),
            format!("{id}~off"),
            format!("{id}~field"),
        );
        let was_open = self.memory(&open_key, 0.0) > 0.0;
        let off = self.memory(&off_key, 0.0) > 0.0;
        if resp.focused {
            self.remember(&field_key, f64::from_bits(self.state.focus));
        }
        let wants =
            resp.focused && self.device().is_pad() && r.str("inputmode") != Some("none") && !off;
        let open = was_open || wants;
        if !open {
            if !resp.focused {
                self.remember(&off_key, 0.0);
            }
            return;
        }
        let ty = r.str("type").unwrap_or("text");
        let mode = r.str("inputmode").unwrap_or(match ty {
            "number" => "decimal",
            "tel" => "tel",
            _ => "text",
        });
        let password = ty == "password";
        let desc = crate::widgets::KeyboardDesc {
            layout: match mode {
                "numeric" | "decimal" | "tel" => L::Numeric,
                _ => L::Letters,
            },
            kana: r
                .str("lang")
                .is_some_and(|l| l == "ja" || l.starts_with("ja-")),
            max_chars: r.num("maxChars").unwrap_or(0.0) as u32,
            title: r
                .str("label")
                .or(r.str("name"))
                .unwrap_or_default()
                .to_string(),
            placeholder: r.str("placeholder").unwrap_or_default().to_string(),
            submit_label: r
                .str("enterkeyhint")
                .map(|h| {
                    let mut c = h.chars();
                    c.next()
                        .map(|f| f.to_uppercase().chain(c).collect())
                        .unwrap_or_default()
                })
                .unwrap_or_default(),
            password,
        };
        // Open, it builds each tick: its keys take input and its caret
        // blinks.
        if let Some(owned) = self.owning.last_mut() {
            owned.moved = true;
        }
        let out = self.on_screen_keyboard(&id, open, s, &desc);
        resp.changed |= out.changed;
        resp.submitted |= out.submitted;
        let done = out.submitted || out.closed;
        if done {
            let back = self.memory(&field_key, 0.0).to_bits();
            if back != 0 {
                self.request_focus = back;
            }
        }
        self.remember(&open_key, if open && !done { 1.0 } else { 0.0 });
        self.remember(&off_key, if done { 1.0 } else { f64::from(u8::from(off)) });
    }

    /// A `data-hold` (or `onhold`) element's hold this tick: counts the
    /// ticks it has been held (a press on it; Enter, Space or a pad's A
    /// while it has the focus; or its `data-action`'s input down) and
    /// whether the count reached its `data-hold` ms (default 500), then
    /// draws its fill (`::-og-hold`) from its start edge across the share
    /// reached, over its background and under its content. The count is
    /// widget memory: in snapshots, and the element builds each tick while
    /// it moves.
    fn hold(&mut self, key: u64, n: &DocNode, r: &Resolved, at: usize) {
        let fill = r.has("dataHold") || n.props.iter().any(|p| p.name() == "dataHold");
        if !fill && !n.triggers.iter().any(|t| t.when == When::Hold) {
            return;
        }
        let ms = r.num("dataHold").unwrap_or(500.0);
        let need = ((ms * self.ticks_per_second as f32 / 1000.0).ceil() as u64).max(1);
        let focused = self.state.focus == key
            && (self
                .input
                .keys_held
                .iter()
                .any(|k| matches!(*k, "Enter" | "NumpadEnter" | "Space"))
                || self.input.gamepad.held & crate::input::GamepadFrame::A != 0);
        let pressing = (self.input.down && self.state.active == key)
            || focused
            || r.str("dataAction").is_some_and(|a| self.action_down(a));
        let memory = crate::id::child_key(key, "~hold");
        let ticks = if pressing { self.anim(memory).h + 1 } else { 0 };
        if pressing || self.has_anim(memory) {
            self.set_anim(
                memory,
                crate::ui::Anim {
                    h: ticks,
                    t: need,
                    ..Default::default()
                },
            );
        }
        let frac = (ticks as f32 / need as f32).min(1.0);
        if !fill || frac <= 0.0 || at >= self.nodes.len() || self.nodes[at].key != key {
            return;
        }
        // `::-og-hold`'s box (a pseudo-element node) colours it.
        let color = r
            .pseudo
            .iter()
            .find(|(w, ..)| *w == "hold")
            .and_then(|(_, _, p)| p.color("background"))
            .unwrap_or_else(|| self.theme.palette.primary.fade(0.33));
        let frame = if self.direction() == og_i18n::Direction::Rtl {
            crate::layout::Frame::left(1.0 - frac, 1.0)
        } else {
            crate::layout::Frame::left(0.0, frac)
        };
        let radius = self.nodes[at].el.radius;
        self.nodes[at].shapes.push(crate::layout::Shape::Rect {
            frame,
            radius,
            color,
        });
    }

    /// Whether the press [`Self::hold`] counted on `key` reached its
    /// `data-hold` ms this tick (once per press).
    fn held(&mut self, key: u64, _n: &DocNode) -> bool {
        let a = self.anim(crate::id::child_key(key, "~hold"));
        a.h > 0 && a.h == a.t
    }

    /// Fires a node's triggers for what happened to it this tick.
    pub(super) fn fire(
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
        // `use:` in an immediate build: made the first tick it is built,
        // updated when its params change; gone is found in docs_begin. A
        // mounted element's are sent by the retained host.
        if b.cached.is_none() && !n.uses.is_empty() {
            for (k, (name, params)) in n.uses.iter().enumerate() {
                let value = params.as_ref().map_or(Value::Null, |p| b.eval(p));
                let old = self.docs.last.insert((key, usize::MAX - k), value.clone());
                let phase = match old {
                    None => "create",
                    Some(old) if old != value => "update",
                    Some(_) => continue,
                };
                b.event(
                    path,
                    "use",
                    name,
                    json!({ "phase": phase, "params": value }),
                );
            }
            let names = n.uses.iter().map(|(name, _)| name.clone()).collect();
            let event = DocEvent {
                mount: b.mount.clone(),
                path: path.to_string(),
                row_key: b.row_key.clone(),
                ..DocEvent::default()
            };
            self.docs.uses.insert(key, (event, names));
        }
        let happened = std::mem::take(&mut self.docs.happened);
        let held = n.triggers.iter().any(|t| t.when == When::Hold) && self.held(key, n);
        let context = long || (self.input.secondary_pressed && self.state.hot == key);
        for (k, t) in n.triggers.iter().enumerate() {
            let go = match &t.when {
                When::Click => resp.clicked,
                When::Changed => resp.changed,
                When::Appear => appeared,
                When::Tap => tap,
                When::DoubleTap => double,
                When::LongPress => long,
                When::Hold => held,
                When::ContextMenu => context,
                When::Toggle | When::Close | When::Revealed => happened.contains(&t.when),
                When::Action(a) => {
                    self.docs.accepts
                        && self.action_fired(a)
                        // A disabled control's shortcut activates nothing,
                        // as its press does not (a `<fieldset disabled>`'s
                        // controls included).
                        && !self.tree.find(key).is_some_and(|i| self.nodes[i].disabled)
                        // An exclusive `<details name>`'s own action
                        // reaches every disclosure of its name.
                        && (a.starts_with('~') || self.focus_in_row(path, &b.row_key))
                }
                When::Change(e) => {
                    let v = b.eval(e);
                    let old = self.docs.last.insert((key, k), v.clone());
                    old.is_some_and(|o| o != v)
                }
                // The input pass fires these ([`super::pointer`]); the
                // capabilities' controls fire the rest when they land.
                _ => false,
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
                Act::Submit {
                    action,
                    controls,
                    row,
                } => {
                    // The first invalid control in document order takes
                    // the focus, and the form is not submitted.
                    let first = self
                        .docs
                        .validity
                        .iter()
                        .filter(|(_, v)| {
                            v.invalid
                                && v.mount == b.mount
                                && v.doc == b.doc.name
                                && controls.contains(&v.node)
                                && (!row || v.row_key == b.row_key)
                        })
                        .min_by(|a, b| (a.1.node, &a.1.row_key).cmp(&(b.1.node, &b.1.row_key)))
                        .map(|(k, _)| *k);
                    match first {
                        Some(k) => self.request_focus = k,
                        None => b.event(path, "emit", action, Value::Null),
                    }
                }
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
pub(super) fn bound(n: &DocNode, prop: &str) -> bool {
    n.props
        .iter()
        .rev()
        .find(|p| p.name() == prop)
        .is_some_and(|p| !matches!(p.value, PropValue::Lit(_)))
}

/// The internal action an exclusive `<details name>` fires as it opens,
/// which the others of its name listen to (they close).
pub(crate) fn details_action(name: &str) -> String {
    format!("~details/{name}")
}

impl Ui {
    /// Whether a declared action's handler at `path` takes the action: a
    /// handler outside any `{#each}` row is the page's and always does; one
    /// inside a row does only while a focus (shared or a pad's) is within
    /// that row, as a key event reaches only the focused element and its
    /// ancestors, so `ondetails` on every row acts on the focused row alone.
    /// The row is the innermost element of `path` named `…#row_key` (an
    /// `{#each}` child, or a virtual list's `Row#`/`Cell#` with the key
    /// escaped); the handler's own last segment may still hold the raw key.
    fn focus_in_row(&self, path: &str, row_key: &str) -> bool {
        if row_key.is_empty() {
            return true;
        }
        let named = row_key.replace('/', "_");
        let escaped = row_key
            .replace('%', "%25")
            .replace('/', "%2F")
            .replace('#', "%23");
        let Some(at) = [row_key, &named, &escaped]
            .into_iter()
            .filter_map(|key| {
                let mark = format!("#{key}");
                path.rmatch_indices(mark.as_str())
                    .map(|(at, _)| at)
                    .find(|&at| {
                        let end = at + mark.len();
                        end == path.len() || path[end..].starts_with('/')
                    })
            })
            .max()
        else {
            return true;
        };
        // The row's element as a focus path names it: a key's `/` taken as
        // `_` (an immediate build) or escaped (a mounted one).
        let rows = [named, escaped].map(|key| format!("{}#{key}", &path[..at]));
        (0..4).any(|pad| {
            let focus = self.focus_path(pad);
            rows.iter().any(|row| {
                focus
                    .strip_prefix(row.as_str())
                    .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
            })
        })
    }
}
