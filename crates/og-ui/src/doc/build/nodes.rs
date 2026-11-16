use super::fields::{FILE_LABEL, NO_FILE};
use super::*;
use crate::doc::schema::Kind;

pub(crate) mod events;
pub(crate) mod lists;

/// What a container element opened around its content, and so how it
/// closes.
enum Shell {
    /// A plain box, a panel or a scroll area: closed by `close`.
    Box,
    /// A button holding content.
    Button,
    /// A disclosure.
    Details,
    /// A dialog, modal or not.
    Dialog,
    /// A spotlight's callout layer.
    Spotlight,
}

/// CSS's text-transform and white-space processing over a text's source.
fn css_text(r: &Resolved, language: &str, raw: &str) -> String {
    crate::css::text_css::shown(r.text.as_deref(), raw, language)
}

/// What an element's build asks its host to build inside it, under the
/// box it has open.
pub(crate) enum Inner<'a> {
    /// Its children; a virtual container's with the props its rows take.
    Children(Option<Box<Props>>),
    /// Those of its children in this range: a disclosure's `<summary>` in
    /// its header, the rest in its content.
    Range(std::ops::Range<usize>),
    /// An element image of its filters (`<feImage href="#id">`,
    /// [`Ui::build_element_images`]).
    Image(&'a mut super::svg::ElementImage),
}

/// Whether disclosure `n`'s first child is its `<summary>`.
fn summary_of(b: &Build<'_>, n: &DocNode) -> bool {
    n.children
        .first()
        .is_some_and(|&c| b.doc.nodes[c].selector_tag.as_deref() == Some("summary"))
}

impl Ui {
    /// Builds element `i` from its resolved values. `build_inner` builds
    /// what stands inside it: its children once, where it shows them, and
    /// before them each element image its filters draw.
    pub(crate) fn build_resolved_node(
        &mut self,
        b: &mut Build<'_>,
        i: usize,
        resolved: &ResolvedNode,
        build_inner: impl FnMut(&mut Ui, &mut Build<'_>, Inner<'_>),
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
            probe,
        } = resolved;
        let (id, path, key) = (id.clone(), path.clone(), *key);
        // A text's inline elements style its runs.
        let lowered;
        let r = if matches!(kind, Kind::Label | Kind::SvgText) && !n.inline.is_empty() {
            lowered = self.lower_inline(b, n, r);
            &lowered
        } else {
            r
        };
        let mut built = probe.clone();
        if let Some(commit) = &r.style {
            self.style
                .commit_styles(commit, &mut self.interaction, &mut self.tree);
        }
        let keeps = std::mem::take(&mut self.docs.state_mut().keeps_hidden);
        // A discrete display exit retains the subtree until the sampled
        // display becomes none, then completes its removal lifecycle.
        let exited = r
            .transitions
            .iter()
            .any(|t| t.discrete && t.covers("display"))
            && r.cascade
                .values
                .iter()
                .any(|(name, value, _)| name == "display" && value.trim() == "none");
        if r.has("visible")
            && !r.flag("visible")
            && r.cascade.container_reads.is_empty()
            && (!keeps || exited)
        {
            return None;
        }
        self.docs.state_mut().memory.seen_now.insert(key);
        self.note_selection(key, r);
        self.docs
            .containers
            .scroll_read
            .extend(r.cascade.scroll_reads.iter().copied());
        if let Some(container) = r.container {
            self.build_container(container);
        }
        let props = self.props_of(b, n, key, r, &mut built);
        if r.str("scrollInitialTarget") == Some("nearest") {
            self.note_initial_target(key);
        }
        if self.docs.library.scroll_controls {
            self.note_scroll_controls(b, n, r, &path, key);
        }
        if kind == Kind::Viewport {
            return self.build_viewport(b, n, id.as_str(), r, props, path, key, built);
        }
        if kind == Kind::Anchor {
            return self.build_anchor(b, n, r, props, path, key, built, build_inner);
        }
        if kind == Kind::Ground || (kind == Kind::Panel && r.flag("world")) {
            return self.build_world_panel(b, n, r, props, path, key, built, build_inner);
        }
        if kind.svg() {
            return self.build_svg(
                b,
                n,
                kind,
                id.as_str(),
                r,
                props,
                path,
                key,
                built,
                build_inner,
            );
        }
        if r.has("popover") {
            return self.build_popover(b, n, id.as_str(), r, props, path, key, built, build_inner);
        }
        let t = *self.env.theme();
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
            css_text(r, b.env.language(), &raw_of(r, b))
        };
        let mut resp = crate::ui::Response::default();
        let mut generated = None;
        // A button with only text is the leaf control; with content it holds it.
        // A control's host holds its control document: it is the box
        // around it, whatever its kind draws without one.
        let container = b.controlled
            || kind.container() && (kind != Kind::Button || !n.children.is_empty())
            || super::base_select::is_base_select(kind, r);
        if container {
            self.tree.pending_host = None;
            return self.doc_container(
                b,
                n,
                &doc,
                kind,
                &id,
                path,
                key,
                r,
                props,
                built,
                build_inner,
            );
        }
        // Keep the widget's intrinsic box separate from CSS-owned fields.
        // Settlement can replace those fields without processing its input again.
        let styled = r;
        let r = styled.intrinsic();
        // A bar control's way, by its flow (the intrinsic props leave CSS's).
        let axis = crate::widgets::ValueAxis::of(styled.own_flow());
        let mut texture = None;
        self.tree.pending_host = styled.part_host.clone().map(|host| (key, host));
        *self.front.accent_mut() = styled.text.as_deref().and_then(|t| t.accent);
        match kind {
            Kind::Label => {
                (resp, generated) =
                    self.doc_label(b, n, &doc, &id, &path, key, &r, styled, t, &raw_of)?;
            }
            Kind::Button => {
                let text = text_of(&r, b);
                resp = self.button(id.as_str(), &text, r.button_desc());
            }
            Kind::Art => {
                texture = self.doc_art(b, &id, &path, &r);
            }
            Kind::Image => {
                texture = self.doc_image(b, &id, &r);
            }
            Kind::Icon => {
                self.doc_icon(&id, &r);
            }
            Kind::Meter => {
                self.doc_meter(&id, &r, axis);
            }
            Kind::Select if r.flag("dataCycle") => {
                let changed = self.cycler(b, n, id.as_str(), &r);
                resp = crate::Response {
                    changed,
                    ..crate::Response::default()
                };
            }
            Kind::Select => {
                resp = self.doc_select(b, n, &id, &path, &r, styled);
            }
            Kind::Progress => {
                self.doc_progress(&id, &r, axis);
            }
            Kind::Slider if r.has("high") => {
                resp = self.doc_slider_range(b, n, &id, &r, axis, &text_of);
            }
            Kind::Slider => {
                resp = self.doc_slider(b, n, &id, &r, axis, &text_of);
            }
            Kind::Toggle => {
                resp = self.doc_toggle(b, n, &id, &r, styled, &text_of);
            }
            Kind::Color => {
                resp = self.color_input(b, n, id.as_str(), &r);
            }
            Kind::Video | Kind::Audio => {
                resp = self.media_element(b, n, id.as_str(), key, &r, kind == Kind::Video);
            }
            Kind::File => {
                resp = self.doc_file(b, n, &doc, &id, &path, &r, styled, &text_of)?;
            }
            Kind::Datetime => {
                resp = self.doc_datetime(b, n, i, &doc, &id, key, &r);
            }
            Kind::Radio => {
                resp = self.doc_radio(b, n, &id, &r, &text_of);
            }
            Kind::Tabs => {
                resp = self.doc_tabs(b, n, &id, &r, &text_of);
            }
            Kind::Field => {
                resp = self.doc_field(b, n, i, &doc, &id, &path, key, &r, styled);
            }
            Kind::Badge => {
                self.doc_badge(b, &id, &r, &text_of);
            }
            Kind::Kbd => {
                resp = self.kbd(b, n, id.as_str(), &r, text_of(&r, b));
            }
            Kind::Spacer => {
                let s = r.size("width").or(r.size("height"));
                self.spacer(id.as_str(), s);
            }
            Kind::Divider => self.divider(id.as_str()),
            Kind::Joystick | Kind::Dpad => {
                resp = self.build_touch_control(kind, id.as_str(), &r, styled);
            }
            // Every kind is named here, so a new one cannot go unbuilt:
            // these have a builder of their own or hold content, above.
            Kind::Viewport
            | Kind::Anchor
            | Kind::Ground
            | Kind::Svg
            | Kind::SvgGroup
            | Kind::SvgShape
            | Kind::SvgText
            | Kind::SvgForeign
            | Kind::SvgUse
            | Kind::SvgDef
            | Kind::Column
            | Kind::Row
            | Kind::Stack
            | Kind::Grid
            | Kind::Panel
            | Kind::Scroll
            | Kind::List
            | Kind::Cells
            | Kind::Dialog
            | Kind::Details
            | Kind::Inline
            | Kind::Box => unreachable!("'{kind}' is built before the leaf controls"),
        }
        self.tree.pending_host = None;
        *self.front.accent_mut() = None;
        // A text's inline elements: nodes under its own, on its lines.
        if kind == Kind::Label && !b.inline.is_empty() && !styled.span_styles.is_empty() {
            let text = self.tree.last_closed();
            if text != NONE && self.tree.kind_of(text) == "richtext" {
                self.inline_nodes(b, i, text);
            }
        }
        let last = self.tree.last_closed();
        if last != NONE && last < self.tree.nodes().len() {
            let mut fields =
                fields::RenderFields::capture(self, key, last, Some(kind), resp, props, texture);
            // A text keeps its source: settlement shows it in its new style.
            // A typewriter's is the part it shows; a counter has none, and
            // keeps the number it wrote.
            if kind == Kind::Label && styled.generated.is_none() && !styled.has("count") {
                fields.text = self.revealed(key, raw_of(styled, b));
            }
            // Its inline elements' runs: settlement styles them again.
            fields.generated = generated.or_else(|| {
                (!n.inline.is_empty() && !styled.span_styles.is_empty())
                    .then(|| std::sync::Arc::new((raw_of(styled, b), styled.span_styles.clone())))
            });
            self.apply_document_fields(&doc, n, styled, &fields);
            self.docs.note_render_fields(key, fields);
            if styled.has("dataOgTouch") {
                self.tree.note_touch_button(last, styled, &mut self.docs);
            }
            if styled.has("emitter")
                || styled.has("meta")
                || self.world.declared.kept_effects.contains_key(&key)
            {
                self.world_effects(b, n, styled, last, &path);
            }
        }
        // A text list item's marker outside hangs before it, a sibling
        // anchored to it; inside it is the text's first run
        // (`generated_text`).
        if kind == Kind::Label
            && styled.str("display") == Some("listItem")
            && styled.str("markerPosition") != Some("inside")
        {
            self.item_marker(b, n, styled, &format!("{id}::marker"), &path);
        }
        self.popover_invoker(b, styled, key, resp);
        self.docs.note_built(built);
        self.hold(key, n, styled, last);
        self.fire(b, n, key, &path, resp);
        Some(path)
    }

    /// The document's `art` element.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_art(
        &mut self,
        b: &mut Build<'_>,
        id: &String,
        path: &String,
        r: &Resolved,
    ) -> Option<og_draw::TextureHandle> {
        // An art slot: its art fitted by its spec, or the style's
        // placeholder for its kind (docs/ui.md §3.3).
        self.docs.art_spec(path, r);
        let mut spec = self.docs.last_art().clone();
        let (at, nearest, orient) = r.object_css();
        if let Some([focal, _]) = at {
            spec.focal = focal;
        }
        let texture = r.handle("texture").and_then(|h| b.env.texture(h));
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
                orient,
            },
        );
        texture
    }

    /// The document's `image` element.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_image(
        &mut self,
        b: &mut Build<'_>,
        id: &String,
        r: &Resolved,
    ) -> Option<og_draw::TextureHandle> {
        let texture = r.handle("texture").and_then(|h| b.env.texture(h));
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
        self.tree.object_fit(i, r);
        texture
    }

    /// The document's `icon` element.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_icon(&mut self, id: &String, r: &Resolved) {
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
                secondary_tint: None,
                label: r.str("name").map(str::to_string),
            },
        );
    }

    /// The role and value a control's host of kind `kind` tells the
    /// accessibility tree: what its control document does not draw.
    fn host_semantics(&mut self, at: usize, kind: Kind, r: &Resolved) {
        let semantics = self.tree.semantics_mut(at);
        match kind {
            Kind::Progress => {
                let value = r.num("value").filter(|v| v.is_finite()).unwrap_or(0.0);
                let percent = (value.clamp(0.0, 1.0) * 100.0).round();
                semantics.role = Some("progressbar");
                semantics.value = Some(format!("{percent:.0}%"));
                semantics.range = Some(crate::a11y::A11yRange {
                    min: Some(0.0),
                    max: Some(100.0),
                    now: percent,
                });
            }
            Kind::Meter => {
                let min = r.num("min").unwrap_or(0.0);
                let max = r.num("max").unwrap_or(1.0).max(min);
                let value = r
                    .num("value")
                    .filter(|v| v.is_finite())
                    .unwrap_or(min)
                    .clamp(min, max);
                semantics.role = Some("meter");
                semantics.value = Some(format!("{value}"));
                semantics.range = Some(crate::a11y::A11yRange {
                    min: Some(min),
                    max: Some(max),
                    now: value,
                });
            }
            _ => {}
        }
    }

    /// The document's `meter` element.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_meter(&mut self, id: &String, r: &Resolved, axis: crate::widgets::ValueAxis) {
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
                height: r.size("height"),
                axis,
            },
        );
    }

    /// The document's `progress` element.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_progress(
        &mut self,
        id: &String,
        r: &Resolved,
        axis: crate::widgets::ValueAxis,
    ) {
        let v = r.num("value").unwrap_or(0.0);
        self.progress(
            id.as_str(),
            v,
            crate::ui::ProgressDesc {
                width: r.size("width"),
                height: r.size("height"),
                axis,
            },
        );
    }

    /// The document's `slider` element.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_slider_range(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &String,
        r: &Resolved,
        axis: crate::widgets::ValueAxis,
        text_of: &impl Fn(&Resolved, &Build<'_>) -> String,
    ) -> crate::ui::Response {
        let mut resp = crate::ui::Response::default();
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
                label: text_of(r, b),
                show_value: !r.has("showValue") || r.flag("showValue"),
                width: r.size("width"),
                height: r.size("height"),
                axis,
                disabled: r.flag("disabled"),
                ..crate::widgets::RangeSliderDesc::default()
            },
        );
        resp.changed = out.changed;
        if out.changed {
            self.docs.write_back(b, n, "low", json!(out.low));
            self.docs.write_back(b, n, "high", json!(out.high));
        }
        resp
    }

    /// The document's `datetime` element.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_datetime(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        i: usize,
        doc: &UiDocument,
        id: &String,
        key: u64,
        r: &Resolved,
    ) -> crate::ui::Response {
        let resp = self.datetime_input(b, n, id.as_str(), r);
        let barred = r.flag("disabled") || r.flag("readonly");
        self.docs.state_mut().validity.insert(
            key,
            super::host::FieldValidity {
                mount: b.mount.clone(),
                doc: doc.name.clone(),
                node: i,
                row_key: b.row_key.clone(),
                invalid: !barred && (r.flag("invalid") || super::datetime::invalid(r)),
            },
        );
        resp
    }

    /// The document's `badge` element.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_badge(
        &mut self,
        b: &mut Build<'_>,
        id: &String,
        r: &Resolved,
        text_of: &impl Fn(&Resolved, &Build<'_>) -> String,
    ) {
        let text = text_of(r, b);
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

    /// A container element and its content: a dialog, a disclosure, a
    /// spotlight, a button with content and the rest. What it opens is its
    /// [`Shell`], which says how it closes.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_container(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        doc: &UiDocument,
        kind: Kind,
        id: &String,
        path: String,
        key: u64,
        r: &Resolved,
        props: Props,
        built: Built,
        mut build_inner: impl FnMut(&mut Ui, &mut Build<'_>, Inner<'_>),
    ) -> Option<String> {
        let mut resp = crate::ui::Response::default();
        // A dialog's or a disclosure's `open`: the bound store's, else the
        // literal's until the player toggles it (its own open state).
        let open_now = |ui: &mut Ui| -> bool {
            let literal = r.flag("open");
            if bound(n, "open") {
                literal
            } else {
                let own = ui.docs.state().memory.open.get(&key).copied();
                own.unwrap_or(literal)
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
        // A customizable select's parts (`appearance: base-select`).
        let tag = n.selector_tag.as_deref();
        let mut picker_from = 0;
        // A closed dialog or disclosure builds no content (HTML renders
        // none): opened, its controls are built fresh and take input.
        let mut shows_content = true;
        // What a controlled host keeps of its kind is its semantics.
        let hosted = b.controlled && !kind.container();
        let kind = if hosted { Kind::Box } else { kind };
        let shell = match kind {
            _ if hosted => {
                self.open(id.as_str(), Element::default());
                let at = self.tree.parent();
                let NodeKind::Element(own) = n.kind else {
                    unreachable!("a host is an element");
                };
                self.host_semantics(at, own, r);
                Shell::Box
            }
            Kind::Dialog if open_now(self) && r.str("spotlight").is_some() => {
                let target = r.str("spotlight").unwrap_or_default();
                let shade = self.open_spotlight(id.as_str(), (key, &path), target, r);
                if shade.clicked && r.str("closedby") == Some("any") {
                    opened = Some(false);
                }
                Shell::Spotlight
            }
            Kind::Dialog => {
                (opened, shows_content) =
                    self.doc_dialog(b, id, &path, r, &dialog, &open_now, opened);
                Shell::Dialog
            }
            Kind::Details => {
                (opened, shows_content) =
                    self.doc_details(b, n, id, &path, key, r, &open_now, opened, &mut build_inner);
                Shell::Details
            }
            Kind::Button => {
                resp = self.open_button(id.as_str(), r.button_desc());
                Shell::Button
            }
            Kind::Select => {
                self.open_base_select(n, id, key, r);
                Shell::Box
            }
            Kind::Panel => {
                self.open_panel(
                    id.as_str(),
                    crate::ui::PanelDesc {
                        row: false,
                        ..Default::default()
                    },
                );
                Shell::Box
            }
            Kind::Scroll => {
                self.open_scroll(
                    id.as_str(),
                    crate::ui::ScrollDesc {
                        width: Size::grow(),
                        height: Size::grow(),
                        ..Default::default()
                    },
                );
                Shell::Box
            }
            Kind::List | Kind::Cells => {
                let inherited = std::mem::replace(&mut b.inherited, r.cascade.computed.clone());
                build_inner(self, b, Inner::Children(Some(Box::new(props.clone()))));
                b.inherited = inherited;
                let at = self.tree.last_closed();
                if at != NONE && at < self.tree.nodes().len() && self.tree.nodes()[at].key == key {
                    // Settlement replaces the box the list was opened with;
                    // its window and stride follow on the next build.
                    let mut fields =
                        fields::RenderFields::capture(self, key, at, Some(kind), resp, props, None);
                    fields.unstyle_virtual();
                    self.docs.note_render_fields(key, fields);
                }
                self.docs.note_built(built);
                self.fire(b, n, key, &path, resp);
                return Some(path);
            }
            // A plain box, a row's content in a row; CSS lays out the rest.
            Kind::Column | Kind::Row | Kind::Stack | Kind::Grid | Kind::Box => {
                self.open(
                    id.as_str(),
                    Element {
                        direction: if kind == Kind::Row {
                            Direction::Row
                        } else {
                            Direction::Column
                        },
                        ..Element::default()
                    },
                );
                Shell::Box
            }
            Kind::Anchor
            | Kind::Ground
            | Kind::Svg
            | Kind::SvgGroup
            | Kind::SvgForeign
            | Kind::SvgUse
            | Kind::SvgDef => unreachable!("'{kind}' is built before the containers"),
            Kind::Inline => unreachable!("an inline element is built by its text"),
            Kind::Label
            | Kind::Kbd
            | Kind::Image
            | Kind::Art
            | Kind::Icon
            | Kind::Progress
            | Kind::Slider
            | Kind::Toggle
            | Kind::Color
            | Kind::Datetime
            | Kind::Video
            | Kind::Audio
            | Kind::File
            | Kind::Radio
            | Kind::Tabs
            | Kind::Field
            | Kind::Badge
            | Kind::Spacer
            | Kind::Divider
            | Kind::Meter
            | Kind::Joystick
            | Kind::Dpad
            | Kind::Viewport
            | Kind::SvgShape
            | Kind::SvgText => unreachable!("'{kind}' holds no content"),
        };
        let fields = fields::RenderFields::capture(
            self,
            key,
            self.tree.parent(),
            Some(kind),
            resp,
            props,
            None,
        );
        self.apply_document_fields(doc, n, r, &fields);
        self.docs.note_render_fields(key, fields);
        // A content button feeds og-input's action as a text one does. It
        // is still open here: the zone is its own, not the last element's
        // closed before it.
        if kind == Kind::Button && r.has("dataOgTouch") {
            let open = self.tree.parent();
            self.tree.note_touch_button(open, r, &mut self.docs);
        }
        // A list item: an `<li>`, numbered by its list, or any box with
        // `display: list-item`, numbered by the run's `list-item` counter.
        if kind == Kind::Box && r.str("display") == Some("listItem") {
            self.item_marker(b, n, r, "Marker", &path);
        }
        if let Some(id) = r.str("rive") {
            let at = self.tree.parent();
            self.docs.play_rive(
                at,
                id,
                &path,
                &self.env,
                &mut self.out,
                &mut self.interaction,
                &mut self.tree,
            );
        }
        if r.has("emitter") || r.has("meta") || self.world.declared.kept_effects.contains_key(&key)
        {
            let at = self.tree.parent();
            self.world_effects(b, n, r, at, &path);
        }
        self.docs.note_built(built);
        if shows_content {
            // Its filters' `feImage`s of elements, as its first children.
            let at = self.tree.parent();
            let css = self.tree.nodes()[at]
                .decor
                .as_ref()
                .and_then(|d| d.css.as_ref());
            let images = css.is_some_and(|c| {
                crate::paint::filter_uses(&c.effects, c.layer.as_ref(), &c.filters)
                    .iter()
                    .any(|u| !u.filter.elements.is_empty())
            });
            if images && let Some(c) = css.cloned() {
                let uses = crate::paint::filter_uses(&c.effects, c.layer.as_ref(), &c.filters);
                self.build_element_images(
                    b,
                    n,
                    &r.cascade.computed,
                    &uses,
                    super::svg::ImageSpace::Css,
                    &mut build_inner,
                );
            }
            let inherited = std::mem::replace(&mut b.inherited, r.cascade.computed.clone());
            self.generated_box(b, n, r, &path, true, &mut build_inner);
            // A disclosure's `<summary>` is in its header.
            if kind == Kind::Details && summary_of(b, n) {
                build_inner(self, b, Inner::Range(1..usize::MAX));
            } else if kind == Kind::Select {
                picker_from = self.base_select_trigger(b, n, r, &mut build_inner);
            } else if tag == Some("selectedcontent") {
                self.selected_content(b, key, &mut build_inner);
            } else {
                build_inner(self, b, Inner::Children(None));
            }
            self.generated_box(b, n, r, &path, false, &mut build_inner);
            b.inherited = inherited;
        }
        match shell {
            Shell::Spotlight => {
                self.close();
            }
            Shell::Dialog => self.close_dialog(&dialog),
            Shell::Details => self.close_details(),
            Shell::Button => self.close_button(),
            Shell::Box => resp = self.close(),
        }
        if kind == Kind::Select {
            self.close_base_select(
                b,
                n,
                key,
                &path,
                r,
                &mut resp,
                picker_from,
                &mut build_inner,
            );
        } else if tag == Some("option")
            && let Some(k) = super::base_select::option_index(&b.doc, n)
        {
            self.note_option(key, k, resp);
        }
        if self.docs.library.scroll_controls {
            let own = self.tree.last_closed();
            self.build_scroll_controls(b, n, r, &path, own);
        }
        if let Some(open) = opened {
            self.docs.state_mut().happened.push(When::Toggle);
            if kind == Kind::Dialog && !open {
                self.docs.state_mut().happened.push(When::Close);
            }
            if !bound(n, "open") {
                let state = self.docs.state_mut();
                state.memory.open.insert(key, open);
                state.edited.insert(key);
            }
            self.docs.write_back(b, n, "open", json!(open));
        }
        self.popover_invoker(b, r, key, resp);
        self.hold(key, n, r, self.tree.last_closed());
        self.fire(b, n, key, &path, resp);
        Some(path)
    }

    /// The document's `dialog` element: the dialog it opens; whether it is opened and shows
    /// its content.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_dialog(
        &mut self,
        b: &mut Build<'_>,
        id: &String,
        path: &String,
        r: &Resolved,
        dialog: &crate::widgets::DialogDesc,
        open_now: &impl Fn(&mut Ui) -> bool,
        mut opened: Option<bool>,
    ) -> (Option<bool>, bool) {
        let open = open_now(self);
        let out = self.open_dialog(id.as_str(), open, dialog);
        // `::backdrop` styles the scrim, the panel's parent.
        if dialog.modal
            && let Some(backdrop) = self.style.pseudo_node(b, r, path, "backdrop")
        {
            let scrim = self.tree.nodes()[self.tree.parent()].parent;
            self.style_pseudo_box(b, scrim, &backdrop);
        }
        let shows_content = out.visible;
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
        (opened, shows_content)
    }

    /// The document's `details` element: the disclosure it opens; whether it is opened and shows
    /// its content.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_details(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &String,
        path: &str,
        key: u64,
        r: &Resolved,
        open_now: &impl Fn(&mut Ui) -> bool,
        mut opened: Option<bool>,
        build_inner: &mut dyn FnMut(&mut Ui, &mut Build<'_>, Inner<'_>),
    ) -> (Option<bool>, bool) {
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
        // Its `<summary>`, a list item whose marker its style draws.
        let own = summary_of(b, n);
        let mut summary = |ui: &mut Ui| build_inner(ui, b, Inner::Range(0..1));
        let out = self.open_details(
            id.as_str(),
            &crate::widgets::DetailsDesc {
                summary: r.str("summary").unwrap_or_default().to_string(),
                open,
                disabled: r.flag("disabled"),
                width: r.size("width"),
            },
            own.then_some(&mut summary as &mut dyn FnMut(&mut Ui)),
        );
        // `::details-content` styles the content box the disclosure just
        // opened (its rules over the disclosure's own padding and gap).
        if let Some(content) = self.style.pseudo_node(b, r, path, "details-content") {
            let at = self.tree.parent();
            self.style_pseudo_box(b, at, &content);
        }
        let shows_content = out.open;
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
                self.interaction
                    .actions_mut()
                    .pending
                    .push(details_action(name));
            }
        }
        (opened, shows_content)
    }

    /// The document's `label` element. `raw_of` is its source: the text
    /// before CSS's processing.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_label(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        doc: &UiDocument,
        id: &String,
        path: &String,
        key: u64,
        r: &Resolved,
        styled: &Resolved,
        t: crate::style::Theme,
        raw_of: &impl Fn(&Resolved, &Build<'_>) -> String,
    ) -> Option<(
        crate::ui::Response,
        Option<std::sync::Arc<fields::GeneratedRuns>>,
    )> {
        let mut resp = crate::ui::Response::default();
        let mut generated = None;
        let source = raw_of(r, b);
        let text = css_text(r, b.env.language(), &source);
        let desc = r.label_desc(&t, self.fonts(), r.own_flow().rtl);
        if !n.inline.is_empty() {
            self.inline_links(b, n, r, path, key);
        }
        // `::before`, `::after` and `::first-letter` join the text
        // as styled runs.
        if let Some(((full, runs), spoken)) = self.generated_text(b, n, styled, key, &text) {
            let attributes: Vec<crate::TextSpan> =
                runs.iter().map(|(span, _)| span.clone()).collect();
            resp = self
                .rich_text_spans(
                    id.as_str(),
                    &full,
                    &attributes,
                    r.rich_desc(&desc),
                    |span| {
                        runs.iter()
                            .find(|(s, _)| s == span)
                            .map(|(_, style)| style.clone())
                            .unwrap_or_default()
                    },
                )
                .response;
            let i = self.tree.last_closed;
            self.tree.nodes[i].a11y.spoken = spoken;
            // Its CSS-owned fields style these runs again
            // (`apply_document_fields`), not a plain label.
            generated = Some(std::sync::Arc::new((full, runs)));
        } else if let Some(V::Spans(spans)) = r.get("spans") {
            let spans = match spans {
                Ok(spans) => spans,
                Err(error) => {
                    self.out.warnings_mut().push(format!(
                        "{}:{}: {path}.spans: expected TextSpan[]: {error}",
                        doc.source, n.line
                    ));
                    *self.front.accent_mut() = None;
                    return None;
                }
            };
            resp = self
                .rich_text_spans(id.as_str(), &text, spans, r.rich_desc(&desc), |span| {
                    r.span_styles
                        .iter()
                        .find(|(attribute, _)| attribute == span)
                        .map(|(_, style)| style.clone())
                        .unwrap_or_default()
                })
                .response;
        } else if let Some((span, style)) = r.css_span(&text, &doc.sheet, self.fonts(), &desc) {
            resp = self
                .rich_text_spans(
                    id.as_str(),
                    &text,
                    std::slice::from_ref(&span),
                    r.rich_desc(&desc),
                    |_| style.clone(),
                )
                .response;
        } else {
            // Every bit of the value: a score past 2^24 counts exactly.
            if let Some(count) = styled.handle("count") {
                self.count_text(id.as_str(), count, b.env.language(), &desc, styled);
            } else {
                let part = self.reveal(key, &source, styled.num("reveal"));
                let i = self.leaf("label", id.as_str().into(), Default::default());
                if part.len() < source.len() {
                    self.apply_label(i, &css_text(r, b.env.language(), part), desc);
                    // A screen reader hears the line once and whole, not
                    // each tick's part of it.
                    self.tree.semantics_mut(i).name = Some(text);
                } else {
                    self.apply_label(i, &text, desc);
                }
                self.tree.css_text_content(i, styled, &doc.sheet);
            }
        }
        Some((resp, generated))
    }

    /// The document's `select` element.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_select(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &String,
        path: &String,
        r: &Resolved,
        styled: &Resolved,
    ) -> crate::ui::Response {
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
            .style
            .pseudo_node(b, styled, path, "picker")
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
                search_placeholder: r
                    .str("dataFilterPlaceholder")
                    .unwrap_or_default()
                    .to_string(),
                ..base
            },
            r.flag("invalid"),
            &mut |ui, list| {
                if let Some((_, p)) = &picker {
                    ui.style_pseudo_box(build, list, p);
                }
            },
        );
        let resp = crate::Response {
            changed: out.changed,
            ..crate::Response::default()
        };
        if out.changed
            && let Some(v) = usize::try_from(out.selected)
                .ok()
                .and_then(|k| values.get(k))
        {
            self.docs.write_back(b, n, "current", json!(v));
        }
        resp
    }

    /// The document's `slider` element.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_slider(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &String,
        r: &Resolved,
        axis: crate::widgets::ValueAxis,
        text_of: &impl Fn(&Resolved, &Build<'_>) -> String,
    ) -> crate::ui::Response {
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
                label: text_of(r, b),
                show_value: !r.has("showValue") || r.flag("showValue"),
                width: r.size("width"),
                height: r.size("height"),
                axis,
                marks,
                origin: r.num("dataFillOrigin"),
                disabled: r.flag("disabled"),
                ..crate::widgets::ValueSliderDesc::default()
            },
        );
        let resp = crate::Response {
            changed: out.changed,
            ..out.response
        };
        if out.changed {
            self.docs.write_back(b, n, "value", json!(out.value));
        }
        resp
    }

    /// The document's `toggle` element.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_toggle(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &String,
        r: &Resolved,
        styled: &Resolved,
        text_of: &impl Fn(&Resolved, &Build<'_>) -> String,
    ) -> crate::ui::Response {
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
                    label: text_of(r, b),
                    description: r.str("description").unwrap_or_default().to_string(),
                    disabled: r.flag("disabled"),
                    side,
                    size: Default::default(),
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
                    label: text_of(r, b),
                    description: r.str("description").unwrap_or_default().to_string(),
                    mixed: r.flag("indeterminate"),
                    disabled: r.flag("disabled"),
                    invalid: r.flag("invalid"),
                    side,
                    width: r.size("width"),
                },
            )
        };
        let resp = crate::Response {
            changed: out.changed,
            ..out.response
        };
        if out.changed {
            self.docs.write_back(b, n, "checked", json!(out.on));
        }
        resp
    }

    /// The document's `file` element.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_file(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        doc: &UiDocument,
        id: &String,
        path: &String,
        r: &Resolved,
        styled: &Resolved,
        text_of: &impl Fn(&Resolved, &Build<'_>) -> String,
    ) -> Option<crate::ui::Response> {
        if !self.env.file_picker() {
            self.out.warnings_mut().push(format!(
                "{}:{}: <input type=\"file\"> needs a host file picker, which this host lacks (og.ui.filePicker is false); take files in a TS system",
                doc.source, n.line
            ));
            return None;
        }
        let chosen = r.str("fileValue").filter(|c| !c.is_empty());
        let text = match text_of(r, b) {
            t if !t.is_empty() => t,
            _ => FILE_LABEL.to_string(),
        };
        let label = crate::LabelDesc {
            // `field-sizing: fixed` keeps the names at the width "none"
            // takes; `content` fits them.
            ellipsis: styled.str("fieldSizing") != Some("content"),
            ..r.label_desc(self.theme(), self.fonts(), r.own_flow().rtl)
        };
        let names = chosen.unwrap_or(NO_FILE);
        let (resp, button) = self.file_input(id.as_str(), &text, names, r.button_desc(), label);
        // `::file-selector-button` styles the button: its box, and its
        // plate as its host (a `<button>`'s plate declarations).
        if let Some(pseudo) = self
            .style
            .pseudo_node(b, styled, path, "file-selector-button")
        {
            self.style_pseudo_box(b, button, &pseudo);
        }
        if resp.clicked && !r.flag("disabled") {
            b.event(
                path,
                "filepick",
                &n.id,
                json!({ "accept": r.str("accept").unwrap_or_default(), "multiple": r.flag("multiple") }),
            );
        }
        Some(resp)
    }

    /// The document's `radio` element.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_radio(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &String,
        r: &Resolved,
        text_of: &impl Fn(&Resolved, &Build<'_>) -> String,
    ) -> crate::ui::Response {
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
                    label: text_of(r, b),
                    description: r.str("description").unwrap_or_default().to_string(),
                    ..Default::default()
                }],
                selected: if on { 0 } else { -1 },
                disabled: r.flag("disabled"),
                width: r.size("width"),
                ..Default::default()
            },
        );
        let resp = crate::Response {
            changed: out.changed,
            ..crate::Response::default()
        };
        if out.changed && out.selected == 0 {
            self.docs.write_back(b, n, "group", json!(choice));
        }
        resp
    }

    /// The document's `tabs` element.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_tabs(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &String,
        r: &Resolved,
        text_of: &impl Fn(&Resolved, &Build<'_>) -> String,
    ) -> crate::ui::Response {
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
                label: text_of(r, b),
                size: Default::default(),
                disabled: r.flag("disabled"),
            },
        );
        selected = usize::try_from(out.selected).unwrap_or(0);
        let resp = crate::Response {
            changed: out.changed || stepped,
            focused: out.focused >= 0,
            ..crate::Response::default()
        };
        if resp.changed {
            match values.get(selected) {
                Some(v) => self.docs.write_back(b, n, "current", json!(v)),
                None => self.docs.write_back(b, n, "selected", json!(selected)),
            }
        }
        resp
    }

    /// The document's `field` element.
    #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
    pub(crate) fn doc_field(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        i: usize,
        doc: &UiDocument,
        id: &String,
        path: &String,
        key: u64,
        r: &Resolved,
        styled: &Resolved,
    ) -> crate::ui::Response {
        let mut resp;
        // Without `bind:value` the field keeps its own value once the
        // player edits it, as an HTML input does.
        let own = !bound(n, "input");
        let mut s = own
            .then(|| self.docs.state().memory.values.get(&key).cloned())
            .flatten()
            .unwrap_or_else(|| r.str("input").unwrap_or_default().to_string());
        let input_type = r
            .str("type")
            .and_then(crate::InputType::from_name)
            .unwrap_or_default();
        let constraints = crate::input_type::Constraints {
            required: r.flag("required"),
            min_chars: r.num("minChars").unwrap_or(0.0) as u32,
            pattern: r.str("pattern").map(str::to_string),
            min: r.num64("min"),
            max: r.num64("max"),
            step: r.num64("stepBy"),
        };
        resp = if r.flag("multiline") {
            self.docs.state_mut().placeholder_ink = self.pseudo_ink(styled);
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
            self.docs.state_mut().placeholder_ink = None;
            out
        } else {
            self.docs.state_mut().placeholder_ink = self.pseudo_ink(styled);
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
                    number: (r.flag("dataWrap")
                        || r.num("dataLargeStep").is_some()
                        || r.str("dataFormat").is_some())
                    .then(|| crate::NumberInputOptions {
                        wrap: r.flag("dataWrap"),
                        large_step: r.num64("dataLargeStep").unwrap_or(0.0),
                        format: crate::widgets::ValueFormat {
                            message: r
                                .str("dataFormat")
                                .filter(|f| !f.is_empty())
                                .and_then(|f| og_i18n::Message::parse(f).ok()),
                            locale: self.env.locale().clone(),
                            ..Default::default()
                        },
                    }),
                },
            );
            self.docs.state_mut().placeholder_ink = None;
            out
        };
        if !r.flag("readonly") {
            self.pad_keyboard(path, r, &mut s, &mut resp);
        }
        if let Some(V::Strs(all)) = r.get("suggestions") {
            let labels = match r.get("suggestionLabels") {
                Some(V::Strs(labels)) => labels.as_slice(),
                _ => &[],
            };
            let keys = match r.get("suggestionKeys") {
                Some(V::Strs(keys)) => keys.as_slice(),
                _ => &[],
            };
            let labels: Vec<_> = all
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    keys.get(index)
                        .filter(|key| !key.is_empty())
                        .and_then(|key| b.env.translate(key))
                        .map(|text| b.env.shown(text))
                        .unwrap_or_else(|| labels.get(index).unwrap_or(value).clone())
                })
                .collect();
            let field = self.tree.last_closed();
            if let Some(c) =
                self.suggestions(path, field, (resp.focused, resp.changed), &s, all, &labels)
            {
                s = c;
                resp.changed = true;
            }
        }
        if resp.changed {
            if own {
                let state = self.docs.state_mut();
                state.memory.values.insert(key, s.clone());
                state.edited.insert(key);
            }
            self.docs.write_back(b, n, "input", json!(s));
        }
        // HTML bars disabled and read-only controls from validation.
        let barred = r.flag("disabled") || r.flag("readonly");
        self.docs.state_mut().validity.insert(
            key,
            super::host::FieldValidity {
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
        resp
    }

    /// A text's inline `<a href>`s: a click sends a `link` event named by
    /// its href, through the document's events as `onclick` does.
    pub(crate) fn inline_links(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        r: &Resolved,
        path: &str,
        key: u64,
    ) {
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
                let state = self.docs.state_mut();
                state.target = (!id.is_empty()).then(|| id.to_string());
                state.scroll_to_target = true;
            }
            b.event(path, "link", &href, Value::Null);
        }
    }
}

impl crate::ui::Docs {
    /// A control's two-way prop: its binding's path takes the new value.
    pub(crate) fn write_back(&mut self, b: &mut Build<'_>, n: &DocNode, prop: &str, v: Value) {
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
}

/// Whether `n`'s `prop` reads a store (`bind:` or `{$x}`) rather than a literal.
pub(crate) fn bound(n: &DocNode, prop: &str) -> bool {
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
    pub(crate) fn focus_in_row(&self, path: &str, row_key: &str) -> bool {
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
