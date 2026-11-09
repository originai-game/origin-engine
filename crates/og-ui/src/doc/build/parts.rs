//! Exposed component parts use the document CSS cascade and box painter.
use super::*;
use crate::doc::matching::{Facts, own};

impl Resolved {
    pub(crate) fn part_type(&self, theme: &crate::Theme, step: crate::TypeStep) -> crate::TypeStep {
        super::super::text_css::inherit(super::super::text_css::Inputs {
            local: &self.cascade.text,
            computed: &mut (*self.cascade.computed).clone(),
            reads: &mut Default::default(),
            diagnostics: &mut Vec::new(),
            theme,
            current: self.color("color").unwrap_or(theme.palette.ink),
            step,
            declared: &|_| false,
        })
        .step(step)
    }
}

impl Ui {
    pub(crate) fn css_root_variables(&self) -> Arc<super::super::variables::Computed> {
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
            root: true,
            attributes: &attributes,
            states: &Default::default(),
            text: "",
        };
        let mut inline = self.root_tokens.clone();
        inline.extend_from_slice(&self.root_type_tokens[self.breakpoint() as usize]);
        cascade::values_with(
            &self.part_rules,
            &cascade::Options {
                inline: &inline,
                inherited: &Default::default(),
                basis: None,
                attr: &|_| None,
                sheet: Some(&self.part_sheet),
                starting: false,
                carry: false,
            },
            |rule| self.part_matches(rule, &facts),
        )
        .computed
    }

    /// Replace the global part stylesheet. Invalid CSS leaves the current
    /// sheet unchanged. This does not install a legacy dressing.
    pub fn set_part_css(&mut self, css: &str) -> Result<(), String> {
        let (rules, sheet) = crate::doc::stylesheet::part_rules(css)?;
        self.part_rules = rules;
        self.part_sheet = sheet;
        self.read_theme_tokens();
        self.kept.epoch += 1;
        Ok(())
    }

    /// Preserve the installed part stylesheet when rebuilding a UI.
    pub fn inherit_part_css(&mut self, source: &Ui) {
        self.part_rules.clone_from(&source.part_rules);
        self.part_sheet.clone_from(&source.part_sheet);
        self.root_tokens.clone_from(&source.root_tokens);
        self.root_type_tokens.clone_from(&source.root_type_tokens);
        self.read_theme_tokens();
        self.kept.epoch += 1;
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
            && rule
                .structure
                .as_ref()
                .is_some_and(|s| s.compounds[0].0.iter().all(|simple| own(facts, simple)))
    }

    pub(crate) fn matching_part_rules(
        &self,
        part: &crate::dress::PartRef,
    ) -> Vec<&crate::doc::StyleRule> {
        part.with_facts(|facts| {
            self.part_rules
                .iter()
                .filter(|rule| self.part_matches(rule, &facts))
                .collect()
        })
    }

    fn part_host(&self, key: u64) -> Option<&PartHost> {
        let mut i = self
            .tree
            .find(key)
            .or_else(|| self.nodes.iter().rposition(|n| n.key == key))?;
        loop {
            let node = &self.nodes[i];
            if let Some(host) = &node.part_host {
                return Some(host);
            }
            if node.parent == NONE {
                return None;
            }
            i = node.parent;
        }
    }

    pub(crate) fn part_values(&self, part: &crate::dress::PartRef, key: Option<u64>) -> Resolved {
        let host = key.and_then(|k| self.part_host(k));
        let document = host.and_then(|h| self.docs.docs.get(&h.document));
        let mut rules = std::borrow::Cow::Borrowed(&self.part_rules);
        let mut sheet = std::borrow::Cow::Borrowed(&self.part_sheet);
        if let Some(doc) = document.filter(|d| !d.part_rules.is_empty()) {
            let rules = rules.to_mut();
            let sheet = sheet.to_mut();
            for layer in &doc.sheet.layers {
                if !sheet.layers.contains(layer) {
                    sheet.layers.push(layer.clone());
                }
            }
            for rule in &doc.part_rules {
                let mut rule = rule.clone();
                if rule.layer != u32::MAX && rule.layer > 0 {
                    let name = &doc.sheet.layers[(rule.layer - 1) as usize];
                    rule.layer = sheet.layers.iter().position(|n| n == name).unwrap() as u32 + 1;
                }
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
        }
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
                sheet: Some(&sheet),
                starting: false,
                carry: false,
            };
            let mut inherited = (*self.css_root_variables()).clone();
            if let Some(host) = host {
                inherited.extend((*host.computed).clone());
            }
            let attr = |name: &str| facts.attributes.get(name).map(crate::bind::text);
            let options = cascade::Options {
                inherited: &inherited,
                attr: &attr,
                ..options
            };
            let mut values =
                cascade::values_with(&rules, &options, |r| self.part_matches(r, &facts));
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
                    Some(&sheet),
                )
            });
            if let (Some(key), Some(commit)) = (key, &style) {
                self.part_commits
                    .borrow_mut()
                    .insert((key, part.part), commit.clone());
            }
            cascade::route(&mut values, crate::layout::Flow::default());
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
            Resolved {
                vals,
                cascade: values,
                transitions,
                animations,
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
            .box_decor(None, r, &mut element, "box", &[], &mut warnings)
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

    pub(crate) fn part_cycle(&self, part: &crate::dress::PartRef, default_ms: f32) -> f32 {
        let values = self.part_values(part, None);
        let animation = values.animations.first();
        let ms = animation.map_or(default_ms, |a| a.duration_ms);
        if ms <= 0.0 {
            return 0.0;
        }
        let period = u64::from(self.ticks_for(ms));
        let phase = (self.state.tick % period) as f32 / period as f32;
        animation.map_or(phase, |a| a.ease.apply(phase))
    }
}
