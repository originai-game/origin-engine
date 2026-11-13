//! Exposed component parts use the document CSS cascade and box painter.
use super::*;
use crate::doc::matching::{Facts, own};
use crate::doc::schema::Kind;

impl Resolved {
    pub(crate) fn part_type(&self, theme: &crate::Theme, step: crate::TypeStep) -> crate::TypeStep {
        self.part_text(theme, step, &|_| false).step(step)
    }

    fn part_text(
        &self,
        theme: &crate::Theme,
        step: crate::TypeStep,
        declared: &dyn Fn(&str) -> bool,
    ) -> super::super::text_css::TextCss {
        super::super::text_css::inherit(super::super::text_css::Inputs {
            local: &self.cascade.text,
            computed: &mut (*self.cascade.computed).clone(),
            reads: &mut Default::default(),
            diagnostics: &mut Vec::new(),
            theme,
            current: self.color("color").unwrap_or(theme.palette.ink),
            step,
            declared,
        })
    }
}

/// A part rule list grouped by what [`Ui::part_matches`] requires of a
/// part, so a part's cascade reads only the rules that can match it. A rule
/// naming a part in its subject compound can match only that part; one
/// naming a part elsewhere, any part; one naming none, only the host (the
/// `part: None` pass), and those are kept by the id, class or tag their
/// subject compound requires, as a browser's rule hash keeps them. Each
/// candidate is still tested in full.
#[derive(Clone, Debug, Default)]
pub(crate) struct PartIndex {
    host: BTreeMap<Subject, Vec<usize>>,
    named: BTreeMap<String, Vec<usize>>,
    loose: Vec<usize>,
}

/// What a host rule's subject compound requires of the host first.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Subject {
    Any,
    Id(String),
    Class(String),
    Tag(String),
}

impl PartIndex {
    pub(crate) fn new(rules: &[crate::doc::StyleRule]) -> Self {
        use super::super::selectors::Simple;
        let mut index = Self::default();
        for (i, rule) in rules.iter().enumerate() {
            let Some(s) = &rule.structure else { continue };
            let part = |c: &super::super::selectors::Compound| {
                c.0.iter().find_map(|simple| match simple {
                    Simple::Part(name) => Some(name.clone()),
                    _ => None,
                })
            };
            let subject = s.compounds.first();
            if let Some(name) = subject.and_then(part) {
                index.named.entry(name).or_default().push(i);
            } else if s.compounds.iter().any(|c| part(c).is_some()) {
                index.loose.push(i);
            } else {
                let simples = subject.map_or(&[][..], |c| &c.0);
                let find = |f: fn(&Simple) -> Option<Subject>| simples.iter().find_map(f);
                let key = find(|x| match x {
                    Simple::Id(n) => Some(Subject::Id(n.clone())),
                    _ => None,
                })
                .or_else(|| {
                    find(|x| match x {
                        Simple::Class(n) => Some(Subject::Class(n.clone())),
                        _ => None,
                    })
                })
                .or_else(|| {
                    find(|x| match x {
                        Simple::Type(n) => Some(Subject::Tag(n.clone())),
                        _ => None,
                    })
                })
                .unwrap_or(Subject::Any);
                index.host.entry(key).or_default().push(i);
            }
        }
        index
    }

    /// The rules of `rules` (the list this was made from) that can match
    /// what `facts` describe, in cascade order.
    pub(crate) fn candidates<'r>(
        &self,
        rules: &'r [crate::doc::StyleRule],
        facts: &Facts<'_>,
    ) -> impl Iterator<Item = &'r crate::doc::StyleRule> {
        let mut at: Vec<usize> = Vec::new();
        match facts.part {
            None => {
                let text = |name: &str| facts.attributes.get(name).and_then(|v| v.as_str());
                let keys = [Subject::Any, Subject::Tag(facts.tag.to_string())]
                    .into_iter()
                    .chain(text("id").map(|id| Subject::Id(id.to_string())))
                    .chain(
                        text("class")
                            .into_iter()
                            .flat_map(str::split_ascii_whitespace)
                            .map(|c| Subject::Class(c.to_string())),
                    );
                for key in keys {
                    at.extend(self.host.get(&key).into_iter().flatten());
                }
            }
            Some(part) => {
                at.extend(&self.loose);
                for name in std::iter::once(part).chain(facts.own_part.filter(|o| *o != part)) {
                    at.extend(self.named.get(name).into_iter().flatten());
                }
            }
        }
        at.sort_unstable();
        at.dedup();
        at.into_iter().map(move |i| &rules[i])
    }
}

impl Ui {
    pub(super) fn apply_part_box(&mut self, i: usize) {
        self.tree.rewrite(i);
        let part = self.tree.nodes[i].part_box.clone().expect("part node");
        let owner = &self.tree.nodes[part.owner];
        let reference = if owner.content_slot == i {
            owner.part().unwrap_or(part.reference)
        } else {
            part.reference
        };
        let r = self.part_values(&reference, Some(self.tree.nodes[i].key));
        let document = self
            .tree
            .part_host(self.tree.nodes[part.owner].key)
            .and_then(|host| self.docs.library.documents.get(&host.document))
            .cloned();
        *self.tree.el_mut(i) = part.element;
        self.apply_box_fields(
            i,
            &r,
            fields::BoxSource {
                document: document.as_deref(),
                kind: if r.has("display") {
                    Some(Kind::Box)
                } else if part.element.direction == Direction::Row {
                    Some(Kind::Row)
                } else {
                    Some(Kind::Column)
                },
                tag: None,
                root: false,
                line: 0,
            },
        );
        let mut props = Self::box_props(&NodeKind::Element(Kind::Box), &r);
        // CSS's physical sides map onto the mirrored surface; the widget's
        // own placement below is already in its layout's terms.
        if self.env.direction == og_i18n::Direction::Rtl {
            crate::layout::flow::mirror_props(&mut props);
        }
        let intrinsic = part.intrinsic_props;
        props.states = intrinsic.states;
        props.state = intrinsic.state;
        for group in &mut props.states {
            for state in &mut group.states {
                state.values.retain(|(property, _)| {
                    use crate::element::Prop;
                    match property {
                        Prop::BackgroundR
                        | Prop::BackgroundG
                        | Prop::BackgroundB
                        | Prop::BackgroundA => !r.has("background"),
                        Prop::Opacity => !r.has("opacity"),
                        Prop::X => !r.has("x") && r.space3d.is_none(),
                        Prop::Y => !r.has("y") && r.space3d.is_none(),
                        Prop::Scale | Prop::ScaleX | Prop::ScaleY => {
                            !r.has("scale")
                                && !r.has("scaleX")
                                && !r.has("scaleY")
                                && r.space3d.is_none()
                        }
                        _ => true,
                    }
                });
            }
        }
        props.motion = props.motion.or(intrinsic.motion);
        props.layout_motion = props.layout_motion.or(intrinsic.layout_motion);
        props.overlaps = intrinsic.overlaps;
        // A widget's presence (a value bubble that fades in) is its own.
        props.enter = props.enter.or(intrinsic.enter);
        props.exit = props.exit.or(intrinsic.exit);
        props.reduced = props.reduced.or(intrinsic.reduced);
        props.inert |= intrinsic.inert;
        if !r.has("z") {
            props.z = intrinsic.z;
        }
        if !r.has("position") && !props.insets.is_set() {
            props.position = intrinsic.position;
            props.insets = intrinsic.insets;
        }
        if r.cascade.transform.is_empty()
            && ![
                "x", "y", "xShift", "yShift", "scale", "scaleX", "scaleY", "rotate", "pivot",
            ]
            .iter()
            .any(|name| r.has(name))
        {
            props.transform = intrinsic.transform;
        }
        if !r.has("opacity") {
            props.opacity = intrinsic.opacity;
        }
        props.flow = r.own_flow();
        *self.tree.props_mut(i) = props;
        let text_style = match &*self.tree.nodes[i].content {
            crate::layout::Content::Text(text) => {
                Some(r.part_text(&self.env.theme, text.step, &|name| {
                    self.style.family_declared(name, &self.env)
                }))
            }
            _ => None,
        };
        if let crate::layout::Content::Text(text) = self.tree.content_mut(i) {
            let style = text_style.unwrap();
            text.step = style.step(text.step);
            let role = style.font_role.unwrap_or(text.font);
            text.font = style
                .family
                .as_deref()
                .map_or(role, |names| self.env.fonts.family_list(names, role));
            if let Some(ink) = r.color("color") {
                text.color = ink;
            }
        }
        self.text_frame_box(i, &r);
    }

    pub(crate) fn css_feedback(
        &self,
        key: Option<u64>,
        event: crate::tokens::SoundEvent,
    ) -> crate::tokens::Feedback {
        let fallback = self.env.theme.tokens.feedback[event.index()];
        let cascade = key
            .and_then(|k| {
                self.tree
                    .nodes
                    .iter()
                    .rfind(|n| n.key == k)
                    .and_then(|n| n.part())
                    .map(|p| Arc::new(self.part_values(&p, Some(k)).cascade))
            })
            .unwrap_or_else(|| self.css_root_cascade());
        let root = self.css_root_cascade();
        let Some((_, value, _)) = cascade
            .values
            .iter()
            .find(|(name, _, _)| name == "-og-sound")
            .or_else(|| root.values.iter().find(|(name, _, _)| name == "-og-sound"))
        else {
            return fallback;
        };
        crate::sound_css::parse(value)
            .ok()
            .and_then(|sounds| sounds.into_iter().find(|s| s.event == event))
            .map_or(
                crate::tokens::Feedback {
                    haptic: fallback.haptic,
                    ..crate::tokens::Feedback::SILENT
                },
                |s| s.bind(&self.style.css_sounds, fallback.haptic),
            )
    }

    pub(crate) fn css_root_variables(&self) -> Arc<super::super::variables::Computed> {
        self.css_root_cascade().computed.clone()
    }

    /// The cascade of the `:root` rules and theme tokens. It depends only on
    /// the style inputs and the media context, so it is computed once for
    /// each and kept until `Style::root_inputs_changed`.
    pub(crate) fn css_root_cascade(&self) -> Arc<cascade::Cascade> {
        let context = self.media_context();
        if let Some((_, cascade)) = self
            .style
            .root_cascade
            .borrow()
            .as_ref()
            .filter(|(of, _)| *of == context)
        {
            return cascade.clone();
        }
        let cascade = Arc::new(self.compute_root_cascade());
        *self.style.root_cascade.borrow_mut() = Some((context, cascade.clone()));
        cascade
    }

    fn compute_root_cascade(&self) -> cascade::Cascade {
        let attributes = BTreeMap::from([(
            "data-scheme".into(),
            Value::String(if self.theme().scheme == crate::Scheme::Light {
                "light".into()
            } else {
                "dark".into()
            }),
        )]);
        let facts = Facts {
            scope: "",
            tag: "",
            part: None,
            own_part: None,
            root: true,
            attributes: &attributes,
            states: &Default::default(),
            text: "",
        };
        let mut inline = self.style.root_tokens.clone();
        inline.extend_from_slice(&self.style.root_type_tokens[self.breakpoint() as usize]);
        inline.extend(self.env.theme.density_tokens(&self.env.viewport));
        cascade::values_with(
            &self.style.part_rules,
            &cascade::Options {
                inline: &inline,
                inherited: &Default::default(),
                basis: None,
                attr: &|_| None,
                sheet: Some(&self.style.part_sheet),
                starting: false,
                carry: false,
            },
            |rule| self.part_matches(rule, &facts),
        )
    }

    /// Replace the global part stylesheet. Invalid CSS leaves the current
    /// sheet unchanged. This does not install a legacy dressing.
    pub fn set_part_css(&mut self, css: &str) -> Result<(), String> {
        let (rules, sheet) = crate::doc::stylesheet::part_rules(css)?;
        self.style.part_rules = rules;
        self.style.part_sheet = sheet;
        self.style.part_rules_changed();
        self.style.root_inputs_changed();
        self.read_theme_tokens();
        self.tree.invalidation.epoch += 1;
        Ok(())
    }

    /// Preserve the installed part stylesheet when rebuilding a UI.
    pub fn inherit_part_css(&mut self, source: &Ui) {
        self.style.part_rules.clone_from(&source.style.part_rules);
        self.style.part_sheet.clone_from(&source.style.part_sheet);
        self.style.css_sounds.clone_from(&source.style.css_sounds);
        self.style.root_tokens.clone_from(&source.style.root_tokens);
        self.style
            .root_type_tokens
            .clone_from(&source.style.root_type_tokens);
        self.style.root_inputs_changed();
        self.read_theme_tokens();
        self.tree.invalidation.epoch += 1;
    }

    /// Resolve a part's CSS for its placed size. The result has no implicit
    /// class-dependent fill, outline, radius or shadow.
    pub fn resolve_part(
        &self,
        part: &crate::dress::PartRef,
        size: [f32; 2],
    ) -> Result<crate::paint::ElementDraw, Vec<String>> {
        self.part_draw_values(&self.part_values(part, None), size, None, None)
    }

    fn part_matches(&self, rule: &crate::doc::StyleRule, facts: &Facts<'_>) -> bool {
        rule.query
            .media
            .matches(self.media_context(), &mut Default::default())
            && rule.query.container.is_empty()
            && rule.structure.as_ref().is_some_and(|s| {
                let has_part = s
                    .compounds
                    .iter()
                    .flat_map(|c| &c.0)
                    .any(|simple| matches!(simple, super::super::selectors::Simple::Part(_)));
                has_part == facts.part.is_some()
                    && s.compounds[0].0.iter().all(|simple| own(facts, simple))
            })
    }

    pub fn matching_part_rules(&self, part: &crate::dress::PartRef) -> Vec<&crate::doc::StyleRule> {
        part.with_facts(|facts| {
            self.style
                .part_rules
                .iter()
                .filter(|rule| self.part_matches(rule, &facts))
                .collect()
        })
    }

    /// The global part rules and sheet with a document's own laid in (its
    /// layers ordered among the global ones, its rules in cascade order),
    /// cached by the document until the global sheet changes.
    fn document_part_rules(
        &self,
        name: &str,
        doc: &Arc<UiDocument>,
    ) -> Arc<crate::ui::MergedPartRules> {
        if let Some((d, m)) = self.style.document_part_rules.borrow().get(name)
            && Arc::ptr_eq(d, doc)
        {
            return m.clone();
        }
        let mut rules = self.style.part_rules.clone();
        let mut sheet = self.style.part_sheet.clone();
        let global_layers = sheet.layers.clone();
        for layer in &doc.sheet.layers {
            if !sheet.layers.contains(layer) {
                sheet.layers.push(layer.clone());
            }
        }
        sheet.layers = crate::doc::stylesheet::order_layers(&sheet.layers);
        let remap = |rule: &mut crate::doc::StyleRule, old: &[String]| {
            if rule.layer != u32::MAX && rule.layer > 0 {
                let name = &old[(rule.layer - 1) as usize];
                rule.layer = sheet.layers.iter().position(|n| n == name).unwrap() as u32 + 1;
            }
        };
        for rule in rules.iter_mut() {
            remap(rule, &global_layers);
        }
        for rule in &doc.part_rules {
            let mut rule = rule.clone();
            remap(&mut rule, &doc.sheet.layers);
            rules.push(rule);
        }
        rules.sort_by_key(|r| (r.layer, r.structure.as_ref().map(|s| s.specificity())));
        for (name, property) in &doc.sheet.properties {
            sheet.properties.retain(|(n, _)| n != name);
            sheet.properties.push((name.clone(), property.clone()));
        }
        for (name, frames) in &doc.sheet.keyframes {
            sheet.keyframes.retain(|(n, _)| n != name);
            sheet.keyframes.push((name.clone(), frames.clone()));
        }
        sheet
            .keyframe_rules
            .extend(doc.sheet.keyframe_rules.clone());
        let index = PartIndex::new(&rules);
        let merged = Arc::new((rules, sheet, index));
        self.style
            .document_part_rules
            .borrow_mut()
            .insert(name.to_string(), (doc.clone(), merged.clone()));
        merged
    }

    pub(crate) fn part_values(&self, part: &crate::dress::PartRef, key: Option<u64>) -> Resolved {
        let host = key.and_then(|k| self.tree.part_host(k));
        let document = host.and_then(|h| self.docs.library.documents.get(&h.document));
        // A document's own part rules laid into the global ones: merged
        // once per document and global sheet, not for every part each tick.
        let merged = host
            .zip(document)
            .filter(|(_, d)| !d.part_rules.is_empty())
            .map(|(h, d)| self.document_part_rules(&h.document, d));
        let global_index;
        let (rules, sheet, index) = match &merged {
            Some(m) => (&m.0, &m.1, &m.2),
            None => {
                global_index = self
                    .style
                    .part_index
                    .borrow_mut()
                    .get_or_insert_with(|| Arc::new(PartIndex::new(&self.style.part_rules)))
                    .clone();
                (
                    &self.style.part_rules,
                    &self.style.part_sheet,
                    &*global_index,
                )
            }
        };
        part.with_facts(|facts| {
            let mut attributes = facts.attributes.clone();
            if let Some(host) = host {
                let classes = format!(
                    "{} {}",
                    attributes
                        .get("class")
                        .map(crate::bind::text)
                        .unwrap_or_default(),
                    host.attributes
                        .get("class")
                        .map(crate::bind::text)
                        .unwrap_or_default()
                );
                attributes.extend(host.attributes.clone());
                attributes.insert("class".into(), classes.into());
            }
            let facts = Facts {
                tag: host.map_or(facts.tag, |h| h.tag.as_str()),
                attributes: &attributes,
                ..facts
            };
            let empty = Default::default();
            let vp = self.viewport();
            let body = self.theme().type_ramp.body;
            let options = cascade::Options {
                inline: &[],
                inherited: &empty,
                basis: Some(crate::doc::calc::Basis {
                    width: vp.width,
                    height: vp.height,
                    small: [vp.width, vp.height],
                    large: [vp.width, vp.height],
                    rem: body.size,
                    rlh: body.line_height,
                    ..Default::default()
                }),
                attr: &|_| None,
                sheet: Some(sheet),
                starting: false,
                carry: false,
            };
            let mut inherited = (*self.css_root_variables()).clone();
            let host_facts = Facts {
                part: None,
                own_part: None,
                ..facts
            };
            let mut global = cascade::values_with(
                index.candidates(rules, &host_facts),
                &cascade::Options {
                    inherited: &inherited,
                    carry: true,
                    ..options
                },
                |rule| {
                    self.part_matches(
                        rule,
                        &Facts {
                            part: None,
                            own_part: None,
                            ..facts
                        },
                    )
                },
            );
            cascade::route(&mut global, crate::layout::Flow::default());
            super::super::text_css::inherit(super::super::text_css::Inputs {
                local: &global.text,
                computed: Arc::make_mut(&mut global.computed),
                reads: &mut global.inherited_reads,
                diagnostics: &mut global.diagnostics,
                theme: self.paint_theme(),
                current: self.paint_theme().palette.ink,
                step: body,
                declared: &|name| self.style.family_declared(name, &self.env),
            });
            inherited = (*global.computed).clone();
            if let Some(host) = host {
                inherited = inherited.under(&host.computed);
            }
            let attr = |name: &str| facts.attributes.get(name).map(crate::bind::text);
            let options = cascade::Options {
                inherited: &inherited,
                attr: &attr,
                ..options
            };
            let mut values = cascade::values_with(index.candidates(rules, &facts), &options, |r| {
                self.part_matches(r, &facts)
            });
            // The host's `font` shorthand is not carried over: its
            // longhands come computed with the host's own (`h2 { font: … }`
            // under `font-size: 2em`), and expanded again here it would
            // undo the host's later longhands.
            for (name, value) in &inherited.properties {
                if cascade::inherited_property(name)
                    && name != "font"
                    && !super::super::text_css::longhand(name)
                    && !values.values.iter().any(|(own, ..)| own == name)
                {
                    values.values.push((name.clone(), value.clone(), 0));
                }
            }
            let motion = values
                .values
                .iter()
                .filter(|(name, ..)| super::super::motion::longhand(name))
                .map(|(name, value, _)| (name.clone(), value.clone()))
                .collect::<Vec<_>>();
            let (transitions, mut animations) =
                super::super::motion::resolve(&motion, "", &mut values.diagnostics);
            for animation in &mut animations {
                let name = animation.name.trim_start_matches('/');
                if let Some((_, _, key)) =
                    sheet.keyframe_rules.iter().rev().find(|(n, query, _)| {
                        n == name
                            && query
                                .media
                                .matches(self.media_context(), &mut Default::default())
                            && query.container.is_empty()
                    })
                {
                    animation.name = key.clone();
                }
            }
            let style = key.and_then(|key| {
                self.animate_values(
                    animate::StyleTarget::Part(key, part.part),
                    &mut values,
                    &[],
                    &transitions,
                    &animations,
                    options.basis.expect("part CSS has viewport units"),
                    Some(sheet),
                )
            });
            if let (Some(key), Some(commit)) = (key, &style) {
                self.style
                    .part_commits
                    .borrow_mut()
                    .insert((key, part.part), commit.clone());
            }
            let flow = cascade::flow(
                &mut values,
                &inherited,
                self.env.direction == og_i18n::Direction::Rtl,
                None,
            );
            cascade::route(&mut values, flow);
            let space3d = crate::transform3d::resolve(&values.transform, &mut values.diagnostics)
                .map(Arc::new);
            let mut vals: Vec<_> = values
                .props
                .iter()
                .filter_map(|p| match &p.value {
                    PropValue::Lit(value) => lit(self, value).map(|v| (p.spec, v)),
                    _ => None,
                })
                .collect();
            if let Some((_, color)) = values.text.iter().find(|(name, _)| name == "color")
                && let Ok(color) = crate::css_color::parse_str(color)
            {
                let (spec, _) = schema::prop("color").expect("color is a reflected property");
                vals.push((
                    spec,
                    V::Color(color.resolve(&crate::css_color::Context::of(self.theme()))),
                ));
            }
            // A button's own plate, above the style's (`Resolved::get`
            // reads the last) and below what its document writes for the
            // part itself (`button.buy::part(base):hover`). The focus ring
            // is the same host's part, without the class.
            if let Some(host) = host.filter(|h| {
                part.class == Some(crate::tokens::ElementClass::Button) && !h.plate.is_empty()
            }) {
                let own = document.filter(|d| !d.part_rules.is_empty()).map(|d| {
                    let mut own = cascade::values_with(&d.part_rules, &options, |r| {
                        self.part_matches(r, &facts)
                    });
                    cascade::route(&mut own, flow);
                    own.props
                });
                let own = own.unwrap_or_default();
                vals.extend(
                    host.plate
                        .iter()
                        .filter(|(spec, _)| !own.iter().any(|p| p.spec == *spec))
                        .cloned(),
                );
            }
            Resolved {
                vals,
                cascade: values,
                flow,
                space3d,
                style,
                ..Default::default()
            }
        })
    }

    pub(crate) fn part_draw_values(
        &self,
        r: &Resolved,
        size: [f32; 2],
        fill: Option<Srgba>,
        base: Option<&crate::paint::ElementDraw>,
    ) -> Result<crate::paint::ElementDraw, Vec<String>> {
        let vp = self.viewport();
        let mut element = Element {
            background: r
                .color("background")
                .or(fill)
                .filter(|color| color.alpha > 0.0),
            radius: r.num("radius").unwrap_or(0.0),
            ..Element::default()
        };
        let mut warnings = r.cascade.diagnostics.clone();
        let mut decor = self
            .box_decor(None, r, &mut element, Some(Kind::Box), &mut warnings)
            .unwrap_or_default();
        if !warnings.is_empty() {
            return Err(warnings);
        }
        if decor.background.is_none() {
            decor.background = element
                .background
                .map(|c| crate::paint::Paint::Solid(crate::paint::ColorRef::Literal(c)));
        }
        decor.radius = r.num("radius");
        let mut draw = decor.over(
            base,
            crate::doc::paint_css::LayerFrame {
                size,
                viewport: [vp.width, vp.height],
                ..Default::default()
            },
        );
        draw.opacity = r.num("opacity").unwrap_or(1.0);
        if decor.borders.iter().any(Option::is_some) || decor.outline.is_some() {
            draw.borders = Some(Box::new(decor));
        }
        Ok(draw)
    }
}

impl crate::ui::Tree {
    fn part_host(&self, key: u64) -> Option<&PartHost> {
        let mut i = self
            .index
            .find(key)
            .or_else(|| self.nodes.iter().rposition(|n| n.key == key))?;
        loop {
            let node = &self.nodes[i];
            if let Some(host) = &node.part_host {
                return Some(host);
            }
            let parent = if node.part_source != NONE {
                node.part_source
            } else {
                node.parent
            };
            if parent == NONE {
                return None;
            }
            i = parent;
        }
    }
}
