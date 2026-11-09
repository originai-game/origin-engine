//! Exposed component parts use the document CSS cascade and box painter.
use super::*;
use crate::doc::matching::{Facts, own};
use crate::doc::selectors::state;

impl Ui {
    /// Replace the global part stylesheet. Invalid CSS leaves the current
    /// sheet unchanged. This does not install a legacy dressing.
    pub fn set_part_css(&mut self, css: &str) -> Result<(), String> {
        let (rules, sheet) = crate::doc::stylesheet::part_rules(css)?;
        self.part_rules = rules;
        self.part_sheet = sheet;
        self.kept.epoch += 1;
        Ok(())
    }

    /// Resolve a part's CSS for its placed size. The result has no implicit
    /// class-dependent fill, outline, radius or shadow.
    pub fn resolve_part(
        &self,
        part: &crate::dress::PartRef,
        size: [f32; 2],
    ) -> Result<crate::paint::ElementDraw, Vec<String>> {
        let mut attributes: BTreeMap<String, Value> = part
            .attrs
            .iter()
            .filter(|(k, _)| !k.is_empty())
            .map(|(k, v)| ((*k).into(), Value::String((*v).into())))
            .collect();
        // Internal widget identities are never selector aliases. A control
        // exposes exactly its HTML host and that host's distinguishing attrs.
        let tag = match part.component {
            "Slider" => {
                attributes.insert("type".into(), "range".into());
                "input".into()
            }
            "Checkbox" => {
                attributes.insert("type".into(), "checkbox".into());
                "input".into()
            }
            "Toggle" => {
                attributes.insert("type".into(), "checkbox".into());
                attributes.insert("switch".into(), true.into());
                "input".into()
            }
            "Radio" => {
                attributes.insert("type".into(), "radio".into());
                "input".into()
            }
            "Dropdown" => "select".into(),
            "Field" => "input".into(),
            "TextEdit" => "textarea".into(),
            tag => tag.to_ascii_lowercase(),
        };
        let mut states = BTreeSet::new();
        let mut classes = attributes
            .get("class")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        for flag in part.flags() {
            let name = match flag {
                "pressed" => "active",
                "focused" => "focus-visible",
                _ => flag,
            };
            if let Some(state) = state(name) {
                states.insert(state);
            } else {
                if !classes.is_empty() {
                    classes.push(' ');
                }
                classes.push_str(flag);
            }
        }
        if part.has("on") {
            if attributes
                .get("type")
                .and_then(Value::as_str)
                .is_some_and(|t| matches!(t, "checkbox" | "radio"))
            {
                states.insert(crate::bind::InteractionState::Checked);
            } else {
                attributes.insert("aria-pressed".into(), "true".into());
            }
        }
        if part.has("selected") {
            attributes.insert("aria-selected".into(), "true".into());
        }
        if !states.contains(&crate::bind::InteractionState::Disabled) {
            states.insert(crate::bind::InteractionState::Enabled);
        }
        attributes.insert("class".into(), classes.into());
        let facts = Facts {
            scope: "",
            tag: &tag,
            part: Some(part.part),
            root: false,
            attributes: &attributes,
            states: &states,
            text: "",
        };
        let root_attributes = BTreeMap::from([(
            "data-scheme".into(),
            Value::String(
                if self.theme().scheme == crate::Scheme::Light {
                    "light"
                } else {
                    "dark"
                }
                .into(),
            ),
        )]);
        let root = Facts {
            tag: "",
            part: None,
            root: true,
            attributes: &root_attributes,
            ..facts
        };
        let context = self.media_context();
        let matches = |rule: &crate::doc::StyleRule, facts: &Facts<'_>| {
            rule.query.media.matches(context, &mut Default::default())
                && rule.query.container.is_empty()
                && rule
                    .structure
                    .as_ref()
                    .is_some_and(|s| s.compounds[0].0.iter().all(|s| own(facts, s)))
        };
        let empty = Default::default();
        let vp = self.viewport();
        let body = self.theme().type_ramp.body;
        let options = cascade::Options {
            inline: &[],
            inherited: &empty,
            theme: self.theme(),
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
            sheet: Some(&self.part_sheet),
            starting: false,
            carry: false,
        };
        let root = cascade::values_with(&self.part_rules, &options, |r| matches(r, &root));
        let attr = |name: &str| attributes.get(name).map(crate::bind::text);
        let options = cascade::Options {
            inherited: &root.computed,
            attr: &attr,
            ..options
        };
        let mut values = cascade::values_with(&self.part_rules, &options, |r| matches(r, &facts));
        values.diagnostics.extend(root.diagnostics);
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
        let r = Resolved {
            vals,
            cascade: values,
            ..Default::default()
        };
        let mut element = Element {
            background: r.color("background"),
            radius: r.num("radius").unwrap_or(0.0),
            ..Element::default()
        };
        let mut warnings = r.cascade.diagnostics.clone();
        let mut decor = self
            .box_decor(None, &r, &mut element, "box", &[], &mut warnings)
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
            None,
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
