//! Render-field replacement from evaluated inputs. No binding evaluation,
//! widget input processing, animation sampling or document emission lives here.
use super::*;

#[derive(Clone, Debug)]
pub(crate) struct RenderFields {
    pub(super) index: usize,
    kind: &'static str,
    element: Element,
    widget_props: Props,
    document_props: Props,
    text: String,
    response: crate::Response,
    amounts: (f32, f32),
    press: f32,
    texture: Option<og_render::TextureHandle>,
    art: Option<usize>,
}

impl RenderFields {
    pub(super) fn capture(
        ui: &Ui,
        index: usize,
        kind: &'static str,
        response: crate::Response,
        document_props: Props,
        texture: Option<og_render::TextureHandle>,
    ) -> Self {
        let node = &ui.nodes[index];
        Self {
            index,
            kind,
            element: node.el,
            widget_props: node.props.clone(),
            document_props,
            text: node.probe_text.clone(),
            response,
            amounts: node.el.part.map_or((0.0, 0.0), |p| (p.hover, p.press)),
            press: match node.widget {
                crate::layout::Widget::Button { press, .. } => press,
                _ => 0.0,
            },
            texture,
            art: (kind == "art").then(|| ui.docs.arts.len() - 1),
        }
    }
}

impl Ui {
    pub(super) fn apply_document_fields(&mut self, n: &DocNode, r: &Resolved, base: &RenderFields) {
        let i = base.index;
        self.nodes[i].el = base.element;
        self.nodes[i].props = base.widget_props.clone();
        match base.kind {
            "label" => {
                let desc = r.label_desc(self.theme());
                if let Some(V::Spans(Ok(spans))) = r.get("spans") {
                    self.apply_text_spans(
                        i,
                        &base.text,
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
                    );
                } else {
                    self.apply_label(i, &base.text, desc);
                }
            }
            "button" => self.apply_button(
                i,
                &base.text,
                r.button_desc(),
                base.response,
                base.amounts,
                base.press,
            ),
            "image" => self.apply_image(
                i,
                crate::ImageDesc {
                    texture: base.texture,
                    tint: r.color("tint"),
                    width: r.size("width").unwrap_or_default(),
                    height: r.size("height").unwrap_or_default(),
                    aspect: r
                        .nums("ratio")
                        .filter(|n| n.len() == 2 && n[1] > 0.0)
                        .map_or(r.num("aspect").unwrap_or(0.0), |n| n[0] / n[1]),
                    label: base.text.clone(),
                    ..Default::default()
                },
            ),
            "icon" => self.apply_icon(
                i,
                r.str("icon").unwrap_or_default(),
                crate::widgets::IconDesc {
                    size: r.num("iconSize").unwrap_or(0.0),
                    tint: r.color("color"),
                    secondary: r.str("secondary").map(str::to_string),
                    secondary_tint: r.color("secondaryTint"),
                    label: r.str("name").map(str::to_string),
                },
            ),
            "art" => {
                let art = base.art.expect("art render fields");
                let path = self.docs.arts[art].0.clone();
                let spec = crate::slot::SlotSpec::from_props(
                    &path,
                    |name| r.str(name).map(str::to_string),
                    |name| r.nums(name).map(<[f32]>::to_vec),
                );
                self.docs.arts[art].1 = spec.clone();
                self.apply_art(
                    i,
                    &crate::widgets::ArtDesc {
                        spec,
                        texture: base.texture,
                        size: None,
                        width: r.size("width").unwrap_or_default(),
                        height: r.size("height").unwrap_or_default(),
                        label: r.str("name").unwrap_or_default().to_string(),
                    },
                );
            }
            "spacer" => {
                let size = r.size("width").or(r.size("height")).unwrap_or(Size::grow());
                self.nodes[i].el.width = size;
                self.nodes[i].el.height = size;
            }
            _ => {}
        }
        // Widget-owned sizing is established above; document box overrides
        // replace their previous values, including when a declaration vanishes.
        let mut el = self.nodes[i].el;
        if schema::kind(base.kind).is_some_and(|kind| kind.container) {
            el = Self::box_of(r, el);
            if !matches!(base.kind, "panel" | "scroll") {
                el.direction = match r.str("direction").unwrap_or(base.kind) {
                    "row" => Direction::Row,
                    _ => Direction::Column,
                };
            }
        } else {
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
                    el = Self::box_of(
                        &Resolved {
                            vals: r
                                .vals
                                .iter()
                                .filter(|(s, _)| PROPS[*s].name == name)
                                .cloned()
                                .collect(),
                            ..Default::default()
                        },
                        el,
                    );
                }
            }
            if r.has("width") && !matches!(base.kind, "label" | "button" | "image" | "art") {
                el.width = r.size("width").unwrap_or(el.width);
            }
        }
        self.nodes[i].el = el;
        let mut props = Self::resolved_props(n, r);
        props.states = base.document_props.states.clone();
        props.state = base.document_props.state.clone();
        if props != Props::default() {
            props.states.extend(
                base.widget_props
                    .states
                    .iter()
                    .filter(|g| !n.groups.iter().any(|x| x.name == g.name))
                    .cloned(),
            );
            self.nodes[i].props = props;
        }
        if !self.building {
            self.apply_part_appearance(i);
            self.prepare_layout_node(i);
        }
    }
}

impl Resolved {
    pub(super) fn intrinsic(&self) -> Self {
        Self {
            vals: self
                .vals
                .iter()
                .filter(|(spec, _)| {
                    let name = PROPS[*spec].name;
                    name != "size" && !super::super::css::STYLE.iter().any(|(p, _)| *p == name)
                })
                .cloned()
                .collect(),
            ..Self::default()
        }
    }

    pub(super) fn label_desc(&self, theme: &crate::Theme) -> LabelDesc {
        LabelDesc {
            step: self.str("step").and_then(|s| {
                crate::style_file::ramp_step(&mut theme.type_ramp.clone(), s).copied()
            }),
            color: self.color("color"),
            on_scene: self.flag("onScene"),
            uppercase: self.flag("uppercase"),
            align: match self.str("textAlign") {
                Some("center") => TextAlign::Center,
                Some("end") => TextAlign::End,
                _ => TextAlign::Start,
            },
            nowrap: self.flag("nowrap"),
            width: self.size("width").unwrap_or_default(),
            font: self
                .str("font")
                .and_then(FontRole::from_name)
                .unwrap_or_default(),
            heading: self.flag("heading"),
            ellipsis: self.flag("ellipsis"),
            max_lines: self.num("maxLines").unwrap_or(0.0) as u32,
            fit_min: self.str("fitMin").and_then(|s| {
                crate::style_file::ramp_step(&mut theme.type_ramp.clone(), s).copied()
            }),
        }
    }

    pub(super) fn button_desc(&self) -> ButtonDesc {
        ButtonDesc {
            variant: match self.str("variant") {
                Some("primary") => ButtonVariant::Primary,
                Some("quiet") => ButtonVariant::Quiet,
                Some("danger") => ButtonVariant::Danger,
                Some("ghost") => ButtonVariant::Ghost,
                _ => ButtonVariant::Secondary,
            },
            size: match self.str("size") {
                Some("small") => ControlSize::Small,
                Some("large") => ControlSize::Large,
                _ => ControlSize::Medium,
            },
            disabled: self.flag("disabled"),
            width: self.size("width"),
            autofocus: self.flag("autofocus"),
            square: self.flag("square"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(styled: bool) -> UiDocument {
        let css = if styled {
            "#Root { padding: 13px; gap: 7px; } #Title { text-transform: uppercase; color: #f0a020; position: absolute; top: 230px; } #Action { -og-control-size: large; width: 173px; } #Check, #Range { width: 219px; padding: 5px; }"
        } else {
            ""
        };
        UiDocument::parse(
            "fields.svelte",
            &format!(
                r#"
<meta name="component" content="fields">
<column id="Root">
  <text id="Title">Mixed Case</text>
  <button id="Action" variant="primary">Apply</button>
  <toggle id="Check" checked={{checked}}>Enabled</toggle>
  <slider id="Range" value={{amount}} show-value="false" />
</column>
<style>{css}</style>
"#
            ),
        )
        .unwrap()
    }

    #[test]
    fn settled_document_fields_match_fresh_styles_without_rebuilding_controls() {
        let env = crate::bind::NoEnv;
        let mut patched = Ui::default();
        let mut direct = Ui::default();
        for ui in [&mut patched, &mut direct] {
            ui.set_fonts(crate::Fonts {
                body: crate::UiFont {
                    handle: Some(og_render::FontHandle::from_bits(1)),
                    metrics: crate::FontMetrics::fixed(0.55),
                },
                ..Default::default()
            });
        }
        for styled in [true, false, true] {
            let target = Arc::new(document(styled));
            let mut vm = json!({"checked": true, "amount": 0.37});
            for (ui, doc) in [
                (&mut patched, document(!styled)),
                (&mut direct, (*target).clone()),
            ] {
                ui.define_document(doc);
                ui.begin(
                    crate::InputFrame::default(),
                    crate::Viewport::new(640.0, 480.0),
                );
                ui.component("Root", "fields", &mut vm, &env);
            }
            let build = Build {
                doc: target.clone(),
                mount: "Root".into(),
                vm: &vm,
                env: &env,
                locals: Vec::new(),
                row_key: String::new(),
                inherited: Arc::default(),
                events: Vec::new(),
                writes: Vec::new(),
                cached: None,
            };
            // Resolve before settlement; the callback has only evaluated values
            // and the existing controls, not a viewmodel or a second build.
            let replacements: Vec<_> = patched
                .docs
                .built
                .iter()
                .map(|node| {
                    let key = crate::id::key_of_path(&node.path);
                    let values = patched.resolve(
                        &build,
                        &target.nodes[node.node],
                        &node.path,
                        &mut Built::default(),
                        &build.inherited,
                    );
                    (node.node, values, patched.docs.render_fields[&key].clone())
                })
                .collect();
            patched.watch_layout_container(patched.nodes[0].key);
            // The first measure of this subscription must run on every test frame.
            patched.unwatch_layout_container(patched.nodes[0].key);
            patched.watch_layout_container(patched.nodes[0].key);
            patched
                .end_with_layout_queries(|ui, _| {
                    let state = ui.save_state();
                    let events = ui.docs.queued.clone();
                    let count = ui.nodes.len();
                    for (node, values, fields) in &replacements {
                        ui.apply_document_fields(&target.nodes[*node], values, fields);
                    }
                    assert_eq!(ui.save_state(), state);
                    assert_eq!(ui.docs.queued, events);
                    assert_eq!(ui.nodes.len(), count);
                    Ok(replacements
                        .iter()
                        .map(|(_, _, fields)| fields.index)
                        .collect())
                })
                .unwrap();
            direct.end();
            assert_eq!(
                patched.frame().elements,
                direct.frame().elements,
                "styled={styled}"
            );
            assert_eq!(
                patched.frame().items,
                direct.frame().items,
                "styled={styled}"
            );
            assert_eq!(patched.a11y(), direct.a11y(), "styled={styled}");
        }
    }
}
