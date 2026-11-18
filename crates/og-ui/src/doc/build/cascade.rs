//! The cascade operates on one instantiated element. Scheduling and parent
//! identity belong to the retained builder; CSS never creates a second scope.
use super::*;
use crate::doc::{StyleRule, TreeContext, css, variables};

#[derive(Clone, Debug, Default)]
pub struct Cascade {
    pub computed: Arc<variables::Computed>,
    pub reads: BTreeSet<String>,
    pub local_names: BTreeSet<String>,
    pub viewport_reads: crate::doc::media::Reads,
    pub(crate) media: crate::css::media::Dependencies,
    /// Only evaluated lookups outside local declarations, including missing
    /// values and evaluated fallbacks. Inherited values are never re-expanded.
    pub inherited_reads: BTreeSet<String>,
    pub diagnostics: Vec<String>,
    pub props: Vec<crate::doc::PropSet>,
    pub unset: BTreeSet<usize>,
    /// The winning declarations, substituted, with the line each was
    /// written on.
    pub values: Vec<(String, String, usize)>,
    /// Winning transition and animation longhands, substituted.
    pub(crate) motion: Vec<(String, String)>,
    /// A winning `transition` longhand was written under a media query
    /// naming `prefers-reduced-motion`: the document chose its reduced
    /// motion, so its transitions play as written rather than through the
    /// engine's reduced-motion fallback.
    pub(crate) reduced_motion_authored: bool,
    /// Winning container-name and container-type, substituted.
    pub(crate) container: Vec<(String, String)>,
    /// Winning SVG presentation properties (`fill`, `stroke`…), substituted.
    pub(crate) svg: Vec<(String, String)>,
    /// Winning text and font properties ([`crate::doc::text_css`]), substituted.
    pub(crate) text: Vec<(String, String)>,
    /// Winning `transform` and 3D properties ([`crate::transform3d`]).
    pub(crate) transform: Vec<(String, String)>,
    /// Winning generated-content properties and pseudo-element
    /// declarations (`::before/content`…), substituted.
    pub(crate) generated: Vec<(String, String)>,
    /// Query containers whose settled size the matched rules depend on.
    pub container_reads: BTreeSet<u64>,
    /// Containers whose scroll state (`scroll-state()`) the matched rules
    /// depend on.
    pub scroll_reads: BTreeSet<u64>,
    /// Unsubstituted inputs, including on idle inheritance bridges.
    replay: Option<Arc<Recipe>>,
    /// Animation effects cannot override important custom declarations.
    pub(super) important_custom: BTreeSet<String>,
}

/// A previous cascade's evaluated inputs, independent of selector/query graphs.
/// Rules remain parsed at runtime; only snapshot restoration parses their values.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Recipe {
    #[serde(with = "matched_rules")]
    rules: Vec<StyleRule>,
    inline: Vec<(String, String)>,
    inherited: variables::Computed,
    attributes: Option<BTreeMap<String, String>>,
    basis: Option<crate::doc::calc::Basis>,
    carry: bool,
    text_scale: f32,
    media: Option<crate::doc::media::Context>,
}

/// Only inputs read after matching belong in a recipe. Preserve rule boundaries
/// and all candidates: `revert-rule` and substituted `revert-layer` need them.
mod matched_rules {
    use super::*;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct MatchedRule {
        inline: bool,
        context: TreeContext,
        depth: i8,
        layer: u32,
        starting: bool,
        reduced_motion: bool,
        declarations: Vec<crate::doc::StyleDeclaration>,
        line: usize,
    }

    impl From<&StyleRule> for MatchedRule {
        fn from(rule: &StyleRule) -> Self {
            Self {
                inline: rule.inline,
                context: rule.context,
                depth: rule.depth,
                layer: rule.layer,
                starting: rule.starting,
                reduced_motion: names_reduced_motion(&rule.query.media),
                declarations: rule.declarations.clone(),
                line: rule.line,
            }
        }
    }

    impl From<MatchedRule> for StyleRule {
        fn from(rule: MatchedRule) -> Self {
            // Matching is already settled. Only this query fact is read by
            // values_effects after matching, to honor authored reduced motion.
            let media = if rule.reduced_motion {
                static REDUCED: std::sync::OnceLock<crate::doc::media::Query> =
                    std::sync::OnceLock::new();
                REDUCED
                    .get_or_init(|| {
                        crate::doc::media::Query::parse("(prefers-reduced-motion: reduce)")
                            .expect("a reduced-motion query")
                    })
                    .clone()
            } else {
                Default::default()
            };
            Self {
                inline: rule.inline,
                context: rule.context,
                depth: rule.depth,
                layer: rule.layer,
                starting: rule.starting,
                declarations: rule.declarations,
                line: rule.line,
                query: crate::doc::AtQuery {
                    media,
                    ..Default::default()
                },
                condition: None,
                structure: None,
                instance_states: Vec::new(),
                props: Vec::new(),
            }
        }
    }

    pub(super) fn retain(rule: &StyleRule) -> StyleRule {
        MatchedRule::from(rule).into()
    }

    pub(super) fn serialize<S: serde::Serializer>(
        rules: &[StyleRule],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let matched: Vec<_> = rules.iter().map(MatchedRule::from).collect();
        serde::Serialize::serialize(&matched, serializer)
    }

    pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<StyleRule>, D::Error> {
        let matched = <Vec<MatchedRule> as serde::Deserialize>::deserialize(deserializer)?;
        Ok(matched.into_iter().map(StyleRule::from).collect())
    }
}

impl Recipe {
    pub(crate) fn inherited(&self) -> &variables::Computed {
        &self.inherited
    }

    pub(crate) fn attribute(&self, name: &str) -> Option<String> {
        self.attributes.as_ref()?.get(name).cloned()
    }

    pub(crate) fn basis(&self) -> Option<crate::doc::calc::Basis> {
        self.basis
    }

    /// Replay the old declarations with the parent's current before-change
    /// inheritance. Attributes, units and other environmental inputs stay old.
    pub(crate) fn cascade(
        &self,
        inherited: &variables::Computed,
        sheet: Option<&crate::doc::at_rules::Sheet>,
    ) -> Cascade {
        let mut result = values_effects(
            &self.rules.iter().collect::<Vec<_>>(),
            &Options {
                inline: &self.inline,
                inherited,
                basis: self.basis,
                attr: &|name| self.attribute(name),
                sheet,
                starting: false,
                carry: self.carry,
                text_scale: self.text_scale,
                media: self.media,
            },
            &mut |_| true,
            None,
        );
        let mut recipe = self.clone();
        recipe.inherited = inherited.clone();
        result.replay = Some(Arc::new(recipe));
        result
    }
}

impl Cascade {
    /// Capture complete evaluated attributes alongside the already-matched
    /// cascade inputs; no attribute callback is retained in snapshot state.
    pub(crate) fn recipe_with_attributes(
        &self,
        attributes: &BTreeMap<String, String>,
    ) -> Arc<Recipe> {
        let mut recipe = self.replay.clone().expect("cascade retains its inputs");
        if recipe.attributes.as_ref() != Some(attributes) {
            Arc::make_mut(&mut recipe).attributes = Some(attributes.clone());
        }
        recipe
    }

    /// Recompute aliases and ordinary candidates against sampled custom values.
    /// Cached cascades remain immutable; the caller routes the fresh result once.
    pub(super) fn with_custom_effects(
        &self,
        effects: &BTreeMap<String, String>,
        attr: &dyn Fn(&str) -> Option<String>,
        sheet: Option<&crate::doc::at_rules::Sheet>,
    ) -> Self {
        let input = self.replay.as_ref().expect("motion retains cascade inputs");
        let attr = |name: &str| match &input.attributes {
            Some(attributes) => attributes.get(name).cloned(),
            None => attr(name),
        };
        let mut result = values_effects(
            &input.rules.iter().collect::<Vec<_>>(),
            &Options {
                inline: &input.inline,
                inherited: &input.inherited,
                basis: input.basis,
                attr: &attr,
                sheet,
                starting: false,
                carry: input.carry,
                text_scale: input.text_scale,
                media: input.media,
            },
            &mut |_| true,
            Some(effects),
        );
        result.replay = self.replay.clone();
        result.media.extend(&self.media);
        result.viewport_reads.extend(self.viewport_reads);
        result.container_reads.extend(&self.container_reads);
        result.scroll_reads.extend(&self.scroll_reads);
        result
    }
}

/// An element's last cascades, by everything [`values_with`] read for
/// them: an element styled again into a state it had (a card lifted and
/// set down, a row hovered and left) cascades nothing again. Kept from its
/// second style on, so an element styled once keeps none.
#[derive(Clone, Default)]
pub(crate) struct Kept {
    cascades: Vec<(Inputs, Cascade)>,
    /// Its text styles ([`crate::doc::text_css::inherit`]), by the text
    /// declarations and computed map they read and [`TextInputs`].
    texts: Vec<(TextRead, TextGave)>,
}

/// What an element's text style reads beyond its cascade and its theme
/// (whose change drops what the element kept): its text colour and type
/// step, and how many font families are installed (what a named family's
/// availability changes with).
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct TextInputs {
    pub current: crate::Srgba,
    pub step: crate::theme::TypeStep,
    pub fonts: usize,
}

/// What a text style read: the element's computed map and text
/// declarations, and [`TextInputs`].
#[derive(Clone)]
struct TextRead {
    computed: Arc<variables::Computed>,
    text: Vec<(String, String)>,
    inputs: TextInputs,
}

/// What a text style gave: the computed map with the text values, the
/// names read and the diagnostics, and the style.
#[derive(Clone)]
struct TextGave {
    computed: Arc<variables::Computed>,
    reads: Vec<String>,
    diagnostics: Vec<String>,
    style: crate::doc::text_css::TextCss,
}

/// How many cascades an element keeps: the two states it moves between.
const KEPT: usize = 2;

/// What one element's cascade read: its document node, the rules that
/// applied, its own custom properties and attributes, what it inherits,
/// its units, and whether it carries its winners.
#[derive(Clone)]
pub(crate) struct Inputs {
    pub doc: Arc<crate::doc::UiDocument>,
    pub node: usize,
    pub rules: Vec<usize>,
    pub inline: Vec<(String, String)>,
    pub attrs: Vec<Option<String>>,
    pub inherited: variables::Computed,
    pub basis: Option<crate::doc::calc::Basis>,
    pub carry: bool,
    /// What an `if()`'s `media()` reads.
    pub media: Option<crate::doc::media::Context>,
}

impl Inputs {
    fn same(&self, other: &Inputs) -> bool {
        Arc::ptr_eq(&self.doc, &other.doc)
            && self.node == other.node
            && self.carry == other.carry
            && self.basis == other.basis
            && self.media == other.media
            && self.rules == other.rules
            && self.inline == other.inline
            && self.attrs == other.attrs
            && self.inherited.same(&other.inherited)
    }
}

impl Kept {
    /// The cascade of `inputs`: a kept one, or `cascade`'s, then kept.
    pub(crate) fn cascade(&mut self, inputs: Inputs, cascade: impl FnOnce() -> Cascade) -> Cascade {
        if let Some(at) = self
            .cascades
            .iter()
            .position(|(kept, _)| kept.same(&inputs))
        {
            let kept = self.cascades.remove(at);
            let out = kept.1.clone();
            self.cascades.push(kept);
            return out;
        }
        let out = cascade();
        if self.cascades.len() >= KEPT {
            self.cascades.remove(0);
        }
        self.cascades.push((inputs, out.clone()));
        out
    }

    /// The text style of `c` ([`crate::doc::text_css::inherit`]): its
    /// inherited text values written into its computed map, the names it
    /// read and what was wrong, and the style. A kept one when `kept` has
    /// one for these inputs.
    pub(crate) fn text(
        kept: Option<&mut Kept>,
        c: &mut Cascade,
        theme: &crate::Theme,
        inputs: TextInputs,
        declared: &dyn Fn(&str) -> bool,
        feature_values: &[crate::css::at_rules::FontFeatureValues],
    ) -> crate::doc::text_css::TextCss {
        let found = kept.as_ref().and_then(|kept| {
            kept.texts.iter().position(|(read, _)| {
                read.inputs == inputs
                    && read.text == c.text
                    && (Arc::ptr_eq(&read.computed, &c.computed) || read.computed.same(&c.computed))
            })
        });
        let mut kept = kept;
        if let (Some(kept), Some(at)) = (kept.as_deref_mut(), found) {
            let entry = kept.texts.remove(at);
            let gave = &entry.1;
            c.computed = gave.computed.clone();
            c.inherited_reads.extend(gave.reads.iter().cloned());
            c.diagnostics.extend(gave.diagnostics.iter().cloned());
            let style = gave.style.clone();
            kept.texts.push(entry);
            return style;
        }
        let read = c.computed.clone();
        let (mut reads, mut diagnostics) = (BTreeSet::new(), Vec::new());
        let style = crate::doc::text_css::inherit(crate::doc::text_css::Inputs {
            local: &c.text,
            computed: Arc::make_mut(&mut c.computed),
            reads: &mut reads,
            diagnostics: &mut diagnostics,
            theme,
            current: inputs.current,
            step: inputs.step,
            declared,
            feature_values,
        });
        c.diagnostics.extend(diagnostics.iter().cloned());
        let reads: Vec<String> = reads.into_iter().collect();
        c.inherited_reads.extend(reads.iter().cloned());
        if let Some(kept) = kept {
            if kept.texts.len() >= KEPT {
                kept.texts.remove(0);
            }
            kept.texts.push((
                TextRead {
                    computed: read,
                    text: c.text.clone(),
                    inputs,
                },
                TextGave {
                    computed: c.computed.clone(),
                    reads,
                    diagnostics,
                    style: style.clone(),
                },
            ));
        }
        style
    }
}

/// The CSS properties whose computed value an element inherits when it
/// does not declare them (CSS's "Inherited: yes").
pub(crate) fn inherited_property(name: &str) -> bool {
    name.starts_with("font")
        || name.starts_with("text-")
        || name.starts_with("list-style")
        || name.starts_with("fill")
        || name.starts_with("stroke")
        || name.starts_with("-webkit-text-stroke")
        || matches!(
            name,
            "color"
                | "cursor"
                | "interpolate-size"
                | "direction"
                | "letter-spacing"
                | "line-height"
                | "quotes"
                | "visibility"
                | "white-space"
                | "white-space-collapse"
                | "word-spacing"
                | "word-break"
                | "overflow-wrap"
                | "hyphens"
                | "hyphenate-character"
                | "hyphenate-limit-chars"
                | "tab-size"
                | "caret-color"
                | "caret-shape"
                | "accent-color"
                | "color-scheme"
                | "writing-mode"
                | "paint-order"
                | "pointer-events"
                | "image-rendering"
                | "image-orientation"
                | "border-collapse"
                | "border-spacing"
                | "caption-side"
                | "empty-cells"
                | "orphans"
                | "widows"
                | "line-break"
                | "hanging-punctuation"
                | "ruby-position"
                | "dominant-baseline"
                | "-og-color-vision"
                | "scrollbar-color"
        )
}

/// The inherited properties an element reads as it is built rather than
/// from the frame's orders: with nothing declared, each takes its
/// parent's value through the cascade (CSS Scrollbars 1 §2:
/// `scrollbar-color` inherits; CSS Images 3 §5.3 and §5.1:
/// `image-rendering`, `image-orientation`; CSS Fragmentation 3 §4.4:
/// `orphans` and `widows`; CSS 2 §17.6: `border-spacing` and
/// `border-collapse`; CSS Values 5 §8.2: `interpolate-size`; CSS Text
/// Decoration 3 §4: `text-shadow`, so a halo on a HUD's root reaches each
/// of its texts in the text's own `currentColor`). A document declaring
/// one carries its values, as one writing `inherit` does.
pub const CARRIED_BY_DEFAULT: &[&str] = &[
    "scrollbar-color",
    "text-shadow",
    "interpolate-size",
    "image-rendering",
    "image-orientation",
    "list-style-type",
    "list-style-position",
    "list-style-image",
    "orphans",
    "widows",
    "border-spacing",
    "border-collapse",
];

/// The query names `prefers-reduced-motion`, evaluated or not.
fn names_reduced_motion(query: &crate::doc::media::Query) -> bool {
    let mut named = false;
    query.features(&mut |f| {
        named |= matches!(f, crate::doc::media::Feature::ReducedMotion(_));
    });
    named
}

/// What won the cascade for one property.
#[derive(Clone, Copy)]
enum Winner<'a> {
    Declared(&'a crate::doc::StyleDeclaration),
    /// `revert`, `revert-layer` or `revert-rule` with nothing below it: as
    /// `unset`.
    Unset,
}

/// `inline` are the element's own custom properties (`style:--name`),
/// which win over every rule but an `!important` one, as an HTML `style`
/// attribute's declarations do. `basis` gives viewport units and `rem`
/// their px; without one they stay as written.
pub(crate) fn resolve_values(
    rules: &[StyleRule],
    inline: &[(String, String)],
    inherited: &variables::Computed,
    basis: Option<crate::doc::calc::Basis>,
    attr: &dyn Fn(&str) -> Option<String>,
    applies: impl FnMut(&StyleRule) -> bool,
) -> Cascade {
    values_with(
        rules,
        &Options {
            inline,
            inherited,
            basis,
            attr,
            sheet: None,
            starting: false,
            carry: false,
            text_scale: 1.0,
            media: None,
        },
        applies,
    )
}

/// The inputs of one element's cascade beyond its rules.
pub(crate) struct Options<'a> {
    pub inline: &'a [(String, String)],
    pub inherited: &'a variables::Computed,
    pub basis: Option<crate::doc::calc::Basis>,
    /// The element's attributes, for `attr()`.
    pub attr: &'a dyn Fn(&str) -> Option<String>,
    /// Its document's `@property` registrations.
    pub sheet: Option<&'a crate::doc::at_rules::Sheet>,
    /// Include `@starting-style` rules (the values a first style's
    /// transitions start from).
    pub starting: bool,
    /// Carry the winners in the standard-property map for children's `inherit`.
    pub carry: bool,
    /// The player's text scale: a type ramp length (`--type-*-size`,
    /// `--type-*-line-height`) set here in px is grown by it, wherever it is
    /// set, as the ramp widgets draw with is (`Theme::to_text_scale`).
    pub text_scale: f32,
    /// The media context an `if()`'s `media()` tests read; none: they are
    /// false.
    pub media: Option<crate::doc::media::Context>,
}

/// `rules` in cascade order; a subset in that order (rules `applies`
/// would refuse left out) cascades the same.
pub(crate) fn values_with<'r>(
    rules: impl IntoIterator<Item = &'r StyleRule>,
    o: &Options<'_>,
    mut applies: impl FnMut(&StyleRule) -> bool,
) -> Cascade {
    values_effects(
        &rules.into_iter().collect::<Vec<_>>(),
        o,
        &mut applies,
        None,
    )
}

fn values_effects(
    rules: &[&StyleRule],
    o: &Options<'_>,
    applies: &mut dyn FnMut(&StyleRule) -> bool,
    effects: Option<&BTreeMap<String, String>>,
) -> Cascade {
    let (inline, inherited, basis, attr) = (o.inline, o.inherited, o.basis, o.attr);
    let mut out = Cascade::default();
    /// A declaration's rank, its fields in the order they compare.
    /// Importance ranks first, then the origin (the user-agent sheet under
    /// the author's), the rule's depth and tree context, the style
    /// attribute, the layer and order (CSS Cascade 5 §6): normal
    /// declarations of a higher depth and a later context or layer win,
    /// important ones of a lower and an earlier.
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct Priority {
        important: bool,
        author: bool,
        depth: i8,
        context: u8,
        inline: bool,
        layer: u32,
        /// The rule's index in `rules`.
        order: usize,
        position: usize,
    }
    // Every applying declaration of each property: (priority, declaration,
    // line, layer).
    type Candidate<'a> = (Priority, &'a crate::doc::StyleDeclaration, usize, u32);
    let mut candidates: BTreeMap<&str, Vec<Candidate<'_>>> = BTreeMap::new();
    for (order, rule) in rules.iter().enumerate() {
        if (rule.starting && !o.starting) || !applies(rule) {
            continue;
        }
        for (position, declaration) in rule.declarations.iter().enumerate() {
            let (context, layer) = if declaration.important {
                (
                    TreeContext::Outer as u8 - rule.context as u8,
                    u32::MAX - rule.layer,
                )
            } else {
                (rule.context as u8, rule.layer)
            };
            // The user-agent sheet is an origin of its own, under every
            // author rule whatever its depth (and over it when important).
            let author = (rule.layer != 0) != declaration.important;
            let depth = if declaration.important {
                -rule.depth
            } else {
                rule.depth
            };
            candidates
                .entry(declaration.name.as_str())
                .or_default()
                .push((
                    Priority {
                        important: declaration.important,
                        author,
                        depth,
                        context,
                        inline: rule.inline,
                        layer,
                        order,
                        position,
                    },
                    declaration,
                    rule.line,
                    rule.layer,
                ));
        }
    }
    for list in candidates.values_mut() {
        list.sort_by_key(|c| std::cmp::Reverse(c.0));
    }
    // The winner of a property's declarations, highest first, where
    // `keyword` is a declaration's CSS-wide keyword: `revert` rolls back to
    // the user-agent rules, `revert-layer` to the layers below its own,
    // `revert-rule` to what the property is without the keyword's own rule
    // (CSS Cascade 5 §7.3).
    fn pick<'a>(
        list: &[Candidate<'a>],
        keyword: &dyn Fn(&crate::doc::StyleDeclaration) -> Option<&'static str>,
    ) -> (Priority, Winner<'a>, usize) {
        let (mut ua_only, mut below) = (false, None);
        // The rules rolled back by `revert-rule`, by their order.
        let mut without: Vec<usize> = Vec::new();
        for (priority, declaration, line, layer) in list {
            // `revert-layer` rolls back the layers of its own encapsulation
            // context: a rule reaching in from another document reverts
            // to that document's lower layers, then to what the element's
            // own document declares.
            if (ua_only && *layer != 0)
                || below.is_some_and(|(depth, b)| priority.depth == depth && *layer >= b)
                || without.contains(&priority.order)
            {
                continue;
            }
            match keyword(declaration) {
                Some("revert-rule") => without.push(priority.order),
                Some("revert") => ua_only = true,
                Some("revert-layer") if *layer > 0 => below = Some((priority.depth, *layer)),
                Some("revert-layer") => ua_only = true,
                _ => return (*priority, Winner::Declared(declaration), *line),
            }
        }
        (list[0].0, Winner::Unset, list[0].2)
    }
    let written =
        |d: &crate::doc::StyleDeclaration| crate::doc::stylesheet::wide_keyword(&d.source);
    let mut winners: BTreeMap<&str, _> = candidates
        .iter()
        .filter(|(name, _)| name.starts_with("--"))
        .map(|(name, list)| (*name, pick(list, &written)))
        .collect();
    let unset = variables::Value::parse("unset").expect("a keyword");
    let mut specified: BTreeMap<_, _> = winners
        .iter()
        .filter(|(name, _)| name.starts_with("--"))
        .map(|(name, (_, winner, _))| match winner {
            Winner::Declared(declaration) => {
                declaration.value.dependencies(&mut out.reads);
                (name.to_string(), declaration.value.clone())
            }
            Winner::Unset => (name.to_string(), unset.clone()),
        })
        .collect();
    out.important_custom = winners
        .iter()
        .filter(|(_, (priority, ..))| priority.important)
        .map(|(name, _)| name.to_string())
        .collect();
    for (name, value) in inline {
        if winners
            .get(name.as_str())
            .is_some_and(|(priority, ..)| priority.important)
        {
            continue;
        }
        match variables::Value::parse(value) {
            Ok(value) => {
                specified.insert(name.clone(), value);
            }
            Err(error) => out.diagnostics.push(format!("style:{name}: {error}")),
        }
    }
    // The type ramp is authored at 100 %: the lengths that won here are set
    // grown, before the tokens built from them (`--text-sm`) resolve. Only
    // what this element sets is grown; what it inherits already is.
    if o.text_scale != 1.0 {
        for (name, value) in &mut specified {
            if let Some(px) = (!value.has_variables())
                .then(|| value.substitute(&mut |_| None).ok())
                .flatten()
                .and_then(|text| super::parts::type_length(name, &text))
            {
                *value =
                    variables::Value::parse(&format!("{}px", px * o.text_scale)).expect("a length");
            }
        }
    }
    let local_names = specified.keys().cloned().collect();
    if let Some(effects) = effects {
        for (name, value) in effects {
            specified.insert(
                name.clone(),
                variables::Value::parse(value).expect("a computed custom value"),
            );
        }
    }
    // Registered custom properties (`@property`): one that does not
    // inherit starts from its initial value, as does one that is unset.
    let registered: Vec<_> = o
        .sheet
        .map(|s| s.properties.iter().collect())
        .unwrap_or_default();
    let adjusted;
    let inherited = if registered.is_empty() {
        inherited
    } else {
        let mut map = inherited.clone();
        for (name, p) in &registered {
            let explicit_inherit = specified.get(name.as_str()).is_some_and(|v| {
                !v.has_variables()
                    && v.substitute(&mut |_| None)
                        .is_ok_and(|v| v.trim().eq_ignore_ascii_case("inherit"))
            });
            if (!p.inherits && !explicit_inherit) || !map.contains_key(name.as_str()) {
                map.insert(name.clone(), p.initial.clone());
            }
        }
        adjusted = map;
        &adjusted
    };
    let (mut computed, diagnostics, inherited_reads) =
        variables::compute(inherited, &specified, &mut |_| None);
    for name in specified.keys() {
        computed.animation_tainted.remove(name);
    }
    if let Some(sheet) = o.sheet {
        computed
            .animation_tainted
            .extend(sheet.keyframes.iter().flat_map(|(_, k)| {
                k.styles
                    .iter()
                    .filter(|t| t.property.starts_with("--"))
                    .map(|t| t.property.clone())
            }));
    }
    while !computed.animation_tainted.is_empty() {
        let mut added = false;
        for (name, value) in &specified {
            let mut dependencies = BTreeSet::new();
            value.dependencies(&mut dependencies);
            if dependencies
                .iter()
                .any(|n| computed.animation_tainted.contains(n))
            {
                added |= computed.animation_tainted.insert(name.clone());
            }
        }
        if !added {
            break;
        }
    }
    for (name, p) in &registered {
        let value = computed.get(name.as_str()).cloned().flatten();
        // Properties and Values API 1 §2.4: a typed property's math
        // computes before syntax checking and substitution. Universal
        // properties keep their tokens; relative lengths compute below
        // once this element's font and viewport basis is known.
        let checked = value.as_ref().and_then(|v| {
            if p.syntax.trim() == "*" || crate::doc::calc::per_element(name, v) {
                Some(v.clone())
            } else {
                crate::doc::calc::reduce(v).ok()
            }
        });
        let valid = checked
            .as_deref()
            .is_some_and(|v| crate::doc::at_rules::matches_syntax(&p.syntax, v));
        if valid {
            computed.insert(name.clone(), checked);
        } else {
            if let Some(v) = &value {
                out.diagnostics.push(format!(
                    "{name}: {} does not match its @property syntax {}; it takes its initial value",
                    v.trim(),
                    p.syntax
                ));
            }
            computed.insert(name.clone(), p.initial.clone());
        }
    }
    // A custom property's `if()` resolves where it is computed, so its
    // children inherit the branch taken (CSS Values 5 §8.3).
    let mut style_reads = BTreeSet::new();
    let mut media_reads = crate::doc::media::Reads::default();
    let mut resolved = BTreeMap::new();
    for name in specified.keys() {
        let mut media = |q: &crate::doc::media::Query| {
            o.media
                .is_some_and(|c| out.media.test(q, c, &mut media_reads))
        };
        if let Err(error) = custom_if(
            name,
            &computed,
            &mut resolved,
            &mut Vec::new(),
            &mut media,
            &mut style_reads,
        ) {
            out.diagnostics.push(format!("{name}: {error}"));
        }
    }
    for (name, value) in resolved {
        computed.insert(name, value);
    }
    // Then its custom function calls, in the element's context (CSS
    // Custom Functions and Mixins 1 §3); a call that fails makes the
    // property guaranteed-invalid.
    let functions = o.sheet.map_or(&[][..], |s| s.functions.as_slice());
    if !functions.is_empty() {
        let mut called = Vec::new();
        for name in specified.keys() {
            let Some(text) = computed.get(name).cloned().flatten() else {
                continue;
            };
            if !text.contains("--") {
                continue;
            }
            let v = crate::doc::stylesheet::replace_functions(
                &text,
                functions,
                &mut |n| {
                    style_reads.insert(n.to_string());
                    computed.get(n).cloned().flatten()
                },
                &mut |q| {
                    o.media
                        .is_some_and(|c| out.media.test(q, c, &mut media_reads))
                },
                &mut Vec::new(),
            );
            match v {
                Ok(v) if v != text => called.push((name.clone(), Some(v))),
                Ok(_) => {}
                Err(error) => {
                    out.diagnostics.push(format!("{name}: {error}"));
                    called.push((name.clone(), None));
                }
            }
        }
        for (name, value) in called {
            computed.insert(name, value);
        }
    }
    out.viewport_reads.extend(media_reads);
    out.diagnostics.extend(diagnostics);
    out.inherited_reads = inherited_reads;
    out.inherited_reads.extend(
        style_reads
            .iter()
            .filter(|n| !specified.contains_key(n.as_str()))
            .cloned(),
    );
    out.reads.extend(style_reads);
    out.local_names = local_names;
    // A property's winner, once its custom properties are known: a `var()`
    // whose value is `revert` or `revert-layer` rolls the cascade back as
    // the keyword written would (CSS Cascade 5 §7.3, CSS Variables 2 §3:
    // `background: var(--fill, revert-layer)` falls back to the layers
    // below when `--fill` is not set).
    let substituted = |d: &crate::doc::StyleDeclaration| {
        written(d).or_else(|| {
            d.value
                .has_variables()
                .then(|| {
                    d.value
                        .substitute(&mut |n| computed.get(n).cloned().flatten())
                        .ok()
                })
                .flatten()
                .and_then(|text| crate::doc::stylesheet::wide_keyword(&text))
                .filter(|k| k.starts_with("revert"))
        })
    };
    for (name, list) in candidates
        .iter()
        .filter(|(name, _)| !name.starts_with("--"))
    {
        winners.insert(name, pick(list, &substituted));
    }
    let mut declarations: Vec<_> = winners
        .into_iter()
        .filter(|(name, _)| !name.starts_with("--"))
        .collect();
    out.reduced_motion_authored = declarations.iter().any(|(name, (priority, winner, _))| {
        name.starts_with("transition")
            && matches!(winner, Winner::Declared(_))
            && names_reduced_motion(&rules[priority.order].query.media)
    });
    declarations.sort_by_key(|(_, (priority, _, _))| *priority);
    if o.carry {
        // A child inherits the carried values of inherited properties
        // only; the rest are this element's own.
        computed
            .properties
            .retain(|name, _| inherited_property(name));
    }
    out.computed = Arc::new(computed);
    // Font size reads the parent's em; other lengths read this element's
    // winning size, including when the font shorthand expanded earlier.
    let parent_basis = basis.map(|mut b| {
        let key = crate::doc::text_css::key("font-size");
        out.inherited_reads.insert(key.clone());
        b.em = inherited
            .get(&key)
            .and_then(|v| v.as_deref())
            .and_then(|v| v.parse().ok())
            .unwrap_or(b.rem);
        let key = crate::doc::text_css::key("line-height");
        out.inherited_reads.insert(key.clone());
        b.lh = inherited
            .get(&key)
            .and_then(|v| v.as_deref())
            .and_then(|v| {
                if v == "normal" {
                    Some(b.em * 1.2)
                } else {
                    v.parse::<f32>()
                        .ok()
                        .map(|f| f * b.em)
                        .or_else(|| crate::doc::text_css::length_of(v, b.em, b.em))
                }
            })
            .unwrap_or(b.em * b.rlh / b.rem.max(1e-3));
        b
    });
    let basis = parent_basis.map(|mut b| {
        let parent = b;
        let line_key = crate::doc::text_css::key("line-height");
        out.inherited_reads.insert(line_key.clone());
        let mut size = None;
        let mut line = inherited.get(&line_key).cloned().flatten();
        for (name, (_, winner, _)) in &declarations {
            if !matches!(*name, "font" | "font-size" | "line-height") {
                continue;
            }
            let Winner::Declared(d) = winner else {
                continue;
            };
            let Ok(value) = d
                .value
                .substitute(&mut |n| out.computed.get(n).cloned().flatten())
            else {
                continue;
            };
            let values = if *name == "font" {
                crate::doc::text_css::font(&value).unwrap_or_default()
            } else {
                vec![(name.to_string(), value)]
            };
            for (name, value) in values {
                match name.as_str() {
                    "font-size" => size = Some(value),
                    "line-height" if matches!(value.trim(), "inherit" | "unset") => {
                        line = inherited.get(&line_key).cloned().flatten();
                    }
                    "line-height" => line = Some(value),
                    _ => {}
                }
            }
        }
        if let Some(value) = size {
            let (value, _) = crate::doc::calc::absolute(&value, parent, "font-size");
            if let Ok(value) = crate::doc::calc::reduce(&value) {
                b.em = match value.trim() {
                    "inherit" | "unset" => parent.em,
                    "initial" => b.rem,
                    v => crate::doc::text_css::font_size(v, parent.em, b.rem).unwrap_or(parent.em),
                };
            }
        }
        b.lh = b.em * b.rlh / b.rem.max(1e-3);
        if let Some(value) = line {
            let line_basis = crate::doc::calc::Basis { lh: parent.lh, ..b };
            let (value, _) = crate::doc::calc::absolute(&value, line_basis, "line-height");
            if let Ok(value) = crate::doc::calc::reduce(&value) {
                b.lh = match value.trim() {
                    "normal" | "initial" => b.em * 1.2,
                    v => v
                        .parse::<f32>()
                        .ok()
                        .map(|factor| factor * b.em)
                        .or_else(|| crate::doc::text_css::length_of(v, b.em, b.em))
                        .unwrap_or(b.lh),
                };
            }
        }
        b
    });
    // A registered length computes to an absolute one (CSS Properties and
    // Values API 1 §2.4): `--gap: 2em` set here is this element's em, and
    // its children inherit the px, not the text.
    if let Some(b) = basis {
        for (name, p) in &registered {
            if !p.syntax.contains("<length") || !specified.contains_key(name.as_str()) {
                continue;
            }
            let Some(Some(text)) = out.computed.get(name.as_str()) else {
                continue;
            };
            if !crate::doc::calc::per_element(name, text) {
                continue;
            }
            let (text, read) = crate::doc::calc::absolute(text, b, name);
            let text = if crate::doc::calc::has_math(&text) {
                match crate::doc::calc::reduce(&text) {
                    Ok(text) => text,
                    Err(_) => continue,
                }
            } else {
                text
            };
            out.viewport_reads.width |= read.width;
            out.viewport_reads.height |= read.height;
            if let Some(c) = o.media {
                out.media.size(read.width, read.height, c);
            }
            out.container_reads
                .extend(read.containers.into_iter().flatten());
            Arc::make_mut(&mut out.computed).insert(name.clone(), Some(text));
        }
    }
    for (name, (_, winner, line)) in declarations {
        for prop in css::targets(name) {
            if let Some((spec, _)) = schema::prop(prop) {
                out.unset.insert(spec);
            }
        }
        let keyword = match winner {
            Winner::Unset => Some("unset"),
            Winner::Declared(d) => crate::doc::stylesheet::wide_keyword(&d.source),
        };
        let value = match (keyword, winner) {
            (Some(keyword), _) => {
                let inherit =
                    keyword == "inherit" || (keyword == "unset" && inherited_property(name));
                if !inherit {
                    // `initial`: the property's own default.
                    if let Some(value) = crate::doc::lists::initial(name)
                        .or_else(|| crate::layout::flow::initial(name))
                    {
                        out.values.push((name.into(), value.into(), line));
                    }
                    continue;
                }
                out.inherited_reads.insert(name.into());
                match inherited.properties.get(name).cloned() {
                    Some(value) => Ok(value),
                    None => continue,
                }
            }
            (None, Winner::Declared(declaration)) => {
                declaration.value.substitute(&mut |variable| {
                    out.reads.insert(variable.into());
                    if !specified.contains_key(variable) {
                        out.inherited_reads.insert(variable.into());
                    }
                    if !crate::motion::interpolate::animatable(name)
                        && out.computed.animation_tainted.contains(variable)
                    {
                        return None;
                    }
                    match out.computed.get(variable) {
                        Some(value) => value.clone(),
                        None => None,
                    }
                })
            }
            (None, Winner::Unset) => unreachable!("unset is a keyword"),
        };
        // An `if()` takes the branch its tests pick on this element.
        let value = value.and_then(|v| {
            if !v.contains("if(") {
                return Ok(v);
            }
            let computed = out.computed.clone();
            let mut media_reads = crate::doc::media::Reads::default();
            let mut style_reads = Vec::new();
            let v = crate::doc::stylesheet::replace_ifs(
                &v,
                &mut crate::doc::stylesheet::IfTests {
                    media: &mut |q| {
                        o.media
                            .is_some_and(|c| out.media.test(q, c, &mut media_reads))
                    },
                    style: &mut |n| {
                        style_reads.push(n.to_string());
                        computed.get(n).cloned().flatten()
                    },
                },
            );
            out.viewport_reads.extend(media_reads);
            for n in style_reads {
                if !specified.contains_key(n.as_str()) {
                    out.inherited_reads.insert(n.clone());
                }
                out.reads.insert(n);
            }
            v
        });
        // A custom function call is its result on this element.
        let functions = o.sheet.map_or(&[][..], |s| s.functions.as_slice());
        let value = value.and_then(|v| {
            if functions.is_empty() || !v.contains("--") {
                return Ok(v);
            }
            let computed = out.computed.clone();
            let mut media_reads = crate::doc::media::Reads::default();
            let mut style_reads = Vec::new();
            let v = crate::doc::stylesheet::replace_functions(
                &v,
                functions,
                &mut |n| {
                    style_reads.push(n.to_string());
                    computed.get(n).cloned().flatten()
                },
                &mut |q| {
                    o.media
                        .is_some_and(|c| out.media.test(q, c, &mut media_reads))
                },
                &mut Vec::new(),
            );
            out.viewport_reads.extend(media_reads);
            for n in style_reads {
                if !specified.contains_key(n.as_str()) {
                    out.inherited_reads.insert(n.clone());
                }
                out.reads.insert(n);
            }
            v
        });
        // `currentcolor` as the value of `color` is `inherit` (CSS Color 4
        // §4.4): the element takes its parent's colour, as one that
        // declares none does (a button's content its plate's ink).
        if name == "color"
            && value
                .as_ref()
                .is_ok_and(|v| v.trim().eq_ignore_ascii_case("currentcolor"))
        {
            out.inherited_reads.insert(name.into());
            continue;
        }
        // attr() takes the element's attribute; viewport, root and font
        // units become px; math functions reduce once their variables are
        // known.
        let value = value.and_then(|v| {
            let v = match winner {
                Winner::Declared(d) if let Some(shorthand) = d.pending_shorthand => {
                    // Its element's units, tree counts and random keys
                    // (naming the shorthand) resolve before it splits.
                    let v = match basis {
                        Some(b) if crate::doc::calc::per_element(shorthand, &v) => {
                            let (v, read) = crate::doc::calc::absolute(&v, b, shorthand);
                            out.viewport_reads.width |= read.width;
                            out.viewport_reads.height |= read.height;
                            if let Some(c) = o.media {
                                out.media.size(read.width, read.height, c);
                            }
                            out.container_reads
                                .extend(read.containers.into_iter().flatten());
                            v
                        }
                        _ => v,
                    };
                    crate::doc::motion::substituted(shorthand, name, v)?
                }
                _ => v,
            };
            let v = if v.contains("attr(") {
                crate::doc::calc::attr(&v, attr)?
            } else {
                v
            };
            let unit_basis = if matches!(name, "font" | "font-size") {
                parent_basis
            } else if name == "line-height" {
                basis
                    .zip(parent_basis)
                    .map(|(b, parent)| crate::doc::calc::Basis { lh: parent.lh, ..b })
            } else {
                basis
            };
            let v = match unit_basis {
                Some(basis) if crate::doc::calc::per_element(name, &v) => {
                    let (v, read) = crate::doc::calc::absolute(&v, basis, name);
                    out.viewport_reads.width |= read.width;
                    out.viewport_reads.height |= read.height;
                    if let Some(c) = o.media {
                        out.media.size(read.width, read.height, c);
                    }
                    out.container_reads
                        .extend(read.containers.into_iter().flatten());
                    v
                }
                _ => v,
            };
            if crate::doc::calc::has_math(&v) {
                crate::doc::calc::reduce(&v)
            } else {
                Ok(v)
            }
        });
        match value {
            Ok(value) => out.values.push((name.into(), value, line)),
            Err(error) => out
                .diagnostics
                .push(format!("line {line}: {name}: {error}")),
        }
    }
    if o.carry {
        for name in CARRIED_BY_DEFAULT {
            if out.values.iter().any(|(n, ..)| n == name) {
                continue;
            }
            out.inherited_reads.insert((*name).into());
            if let Some(value) = o.inherited.properties.get(*name).cloned() {
                out.values.push(((*name).into(), value, 0));
            }
        }
        let computed = Arc::make_mut(&mut out.computed);
        for (name, value, _) in &out.values {
            computed.properties.insert(name.clone(), value.clone());
        }
    }
    if effects.is_none() {
        let matched: BTreeSet<_> = candidates
            .values()
            .flat_map(|list| list.iter().map(|(priority, ..)| priority.order))
            .collect();
        let matched: Vec<_> = matched.iter().map(|&index| rules[index]).collect();
        out.replay = Some(Arc::new(Recipe {
            rules: matched.into_iter().map(matched_rules::retain).collect(),
            inline: inline.to_vec(),
            inherited: o.inherited.clone(),
            attributes: None,
            basis: o.basis,
            carry: o.carry,
            text_scale: o.text_scale,
            media: o.media,
        }));
    }
    out
}

/// Whether document CSS takes `name: value` (a variable-free value).
pub(crate) fn known(name: &str, value: &str) -> bool {
    crate::doc::motion::longhand(name)
        || crate::transform3d::LONGHANDS.contains(&name)
        || crate::doc::container::longhand(name)
        || crate::doc::extern_element::svg_longhand(name)
        || crate::doc::generated::longhand(name)
        || crate::doc::text_css::longhand(name)
        || (crate::layout::flow::longhand(name) && crate::layout::flow::check(name, value).is_ok())
        || css::declarations(&format!("{name}: {value};"), 0).is_ok()
}

/// The computed-map key an element's flow is carried to its children under.
const FLOW: &str = " flow";

/// The computed-map key under which a document's root element carries
/// whether its writing mode is vertical (what `vi` and `vb` read).
const ROOT_VERTICAL: &str = " root vertical";

/// Whether the document root's writing mode is vertical, for an element
/// below it; `None` at the root itself (its own decides).
pub(crate) fn root_vertical(inherited: &variables::Computed) -> Option<bool> {
    inherited.get(FLOW)?;
    Some(matches!(inherited.get(ROOT_VERTICAL), Some(Some(v)) if v == "1"))
}

/// The writing mode `values` declare is vertical.
pub(crate) fn declares_vertical(values: &[(String, String, usize)]) -> bool {
    values
        .iter()
        .rev()
        .find(|(name, ..)| name == "writing-mode")
        .and_then(|(_, v, _)| crate::layout::flow::WritingMode::parse(v))
        .is_some_and(|m| m.vertical())
}

/// An element's flow ([`crate::layout::Flow`]): its parent's (the
/// surface's direction at a document's root), its `dir` attribute's
/// direction, then its declared `direction`, `unicode-bidi`,
/// `writing-mode`, `text-orientation` and `text-combine-upright`; carried
/// to its children.
pub(crate) fn flow(
    out: &mut Cascade,
    inherited: &variables::Computed,
    surface_rtl: bool,
    dir: Option<&str>,
) -> crate::layout::Flow {
    use crate::layout::flow::{Bidi, Flow, TextOrientation, WritingMode, combine};
    out.inherited_reads.insert(FLOW.into());
    let root = inherited.get(FLOW).is_none();
    let mut flow = match inherited.get(FLOW).cloned().flatten() {
        Some(v) => Flow::carried(&v),
        None => Flow {
            rtl: surface_rtl,
            ..Flow::default()
        },
    };
    flow.doc = true;
    // HTML's `[dir]`: its direction, isolated (`auto` reads its text once
    // its props are known).
    if let Some(d) = dir.map(str::to_ascii_lowercase) {
        flow.declared = true;
        flow.bidi = Bidi::Isolate;
        match d.as_str() {
            "rtl" => flow.rtl = true,
            "ltr" | "auto" => flow.rtl = false,
            _ => {}
        }
    }
    let mut unknown = |name: &str, value: &str| {
        out.diagnostics
            .push(format!("{name}: unknown value {}", value.trim()))
    };
    for (name, value, _) in &out.values {
        match name.as_str() {
            "direction" if matches!(value.trim(), "ltr" | "rtl") => {
                flow.rtl = value.trim() == "rtl";
                flow.declared = true;
            }
            "unicode-bidi" => match Bidi::parse(value) {
                Some(b) => {
                    flow.bidi = b;
                    flow.declared = true;
                }
                None => unknown(name, value),
            },
            "writing-mode" => match WritingMode::parse(value) {
                Some(m) => flow.mode = m,
                None => unknown(name, value),
            },
            "text-orientation" => match TextOrientation::parse(value) {
                Some(o) => flow.orient = o,
                None => unknown(name, value),
            },
            "text-combine-upright" => match combine(value) {
                Some(c) => flow.combine = c,
                None => unknown(name, value),
            },
            "direction" => unknown(name, value),
            _ => {}
        }
    }
    let computed = Arc::make_mut(&mut out.computed);
    computed.insert(FLOW.into(), Some(flow.carry()));
    if root {
        let v = if flow.mode.vertical() { "1" } else { "0" };
        computed.insert(ROOT_VERTICAL.into(), Some(v.into()));
    } else {
        out.inherited_reads.insert(ROOT_VERTICAL.into());
    }
    flow
}

/// Sends each winning value to the part of the build that reads it, a
/// logical property as the physical one it names in `flow`. What it sends
/// is a pure function of the winners and the flow, so it is kept by them
/// ([`crate::css::memo`]): an element styled again into a state it had (a
/// card lifted and set down) routes nothing again.
pub(crate) fn route(out: &mut Cascade, flow: crate::layout::Flow) {
    let key = (flow, Winners(std::mem::take(&mut out.values)));
    let kept = ROUTED.with_borrow_mut(|m| m.get(&key).cloned());
    let routed = match kept {
        Some(routed) => routed,
        None => {
            let routed = std::sync::Arc::new(routed(&key.1.0, flow));
            ROUTED.with_borrow_mut(|m| m.keep(key.clone(), routed.clone()));
            routed
        }
    };
    out.values = key.1.0;
    let Routed {
        text,
        motion,
        transform,
        container,
        svg,
        generated,
        props,
        diagnostics,
    } = &*routed;
    out.text.extend_from_slice(text);
    out.motion.extend_from_slice(motion);
    out.transform.extend_from_slice(transform);
    out.container.extend_from_slice(container);
    out.svg.extend_from_slice(svg);
    out.generated.extend_from_slice(generated);
    out.props.extend_from_slice(props);
    // Each with the line of the winner it is about: the same winners on
    // other lines route the same.
    out.diagnostics
        .extend(diagnostics.iter().map(|(winner, said)| match winner {
            Some(i) => format!("line {}: {said}", out.values[*i].2),
            None => said.clone(),
        }));
}

/// A cascade's winners as [`route`] keeps them: by name and value, not by
/// the line each was written on.
#[derive(Clone)]
struct Winners(Vec<(String, String, usize)>);

impl PartialEq for Winners {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len()
            && self
                .0
                .iter()
                .zip(&other.0)
                .all(|(a, b)| a.0 == b.0 && a.1 == b.1)
    }
}
impl Eq for Winners {}
impl std::hash::Hash for Winners {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.0.len());
        for (name, value, _) in &self.0 {
            name.hash(state);
            value.hash(state);
        }
    }
}

/// What [`route`] adds to a cascade's lists.
#[derive(Default)]
struct Routed {
    text: Vec<(String, String)>,
    motion: Vec<(String, String)>,
    transform: Vec<(String, String)>,
    container: Vec<(String, String)>,
    svg: Vec<(String, String)>,
    generated: Vec<(String, String)>,
    props: Vec<crate::doc::PropSet>,
    /// What was wrong, with the place among the winners of the one it is
    /// about.
    diagnostics: Vec<(Option<usize>, String)>,
}

fn routed(values: &[(String, String, usize)], flow: crate::layout::Flow) -> Routed {
    let mut out = Routed::default();
    let mut body = String::new();
    // The winners stay with the cascade: the UI debugger names each one's
    // line (`Ui::debug_element`). Each is carried by its place among them.
    let values: Vec<(String, String, usize)> = values
        .iter()
        .enumerate()
        .flat_map(|(winner, (name, value, _))| {
            match crate::layout::flow::physical(name, value, flow) {
                Some(sides) => sides.into_iter().map(|(n, v)| (n, v, winner)).collect(),
                None if name.starts_with("justify-") || name.starts_with("place-") => {
                    // `left` and `right` are the line's physical sides.
                    let side = |w: &str| match (w, flow.rtl) {
                        ("left", false) | ("right", true) => "start".to_string(),
                        ("left", true) | ("right", false) => "end".to_string(),
                        (w, _) => w.to_string(),
                    };
                    let v: Vec<String> = value.split_whitespace().map(side).collect();
                    vec![(name.clone(), v.join(" "), winner)]
                }
                None => vec![(name.clone(), value.clone(), winner)],
            }
        })
        .filter(|(name, ..)| !crate::layout::flow::longhand(name))
        .collect();
    let values = values
        .into_iter()
        .flat_map(|(name, value, winner)| {
            if name == "font" {
                match crate::doc::text_css::font(&value) {
                    Ok(values) => return values.into_iter().map(|(n, v)| (n, v, winner)).collect(),
                    Err(error) => {
                        out.diagnostics
                            .push((Some(winner), format!("font: {error}")));
                        return Vec::new();
                    }
                }
            }
            vec![(name, value, winner)]
        })
        .collect::<Vec<_>>();
    for (name, value, winner) in values {
        if name == "font-family" {
            match crate::doc::text_css::family_list(&value) {
                Ok((_, role)) => {
                    out.text.push((name, value));
                    body.push_str(&format!("font-family: {role};"));
                }
                Err(error) => out.diagnostics.push((Some(winner), error)),
            }
            continue;
        }
        if crate::doc::motion::longhand(&name) {
            out.motion.push((name, value));
            continue;
        }
        if crate::transform3d::LONGHANDS.contains(&name.as_str()) {
            out.transform.push((name, value));
            continue;
        }
        if crate::doc::container::longhand(&name) {
            out.container.push((name, value));
            continue;
        }
        if crate::doc::extern_element::svg_longhand(&name) {
            out.svg.push((name, value));
            continue;
        }
        if crate::doc::generated::longhand(&name) {
            out.generated.push((name, value));
            continue;
        }
        if crate::doc::text_css::longhand(&name) {
            // SVG text reads its font as a presentation attribute too.
            if crate::doc::extern_element::TEXT_PRESENTATION.contains(&name.as_str()) {
                out.svg.push((name.clone(), value.clone()));
            }
            out.text.push((name, value));
            continue;
        }
        let declaration = format!("{name}: {value};");
        let valid = VALID.with_borrow_mut(|v| v.get(declaration.as_str()).is_some());
        match if valid {
            Ok(())
        } else {
            css::declarations(&declaration, 0).map(|_| ())
        } {
            Ok(()) => {
                if !valid {
                    VALID.with_borrow_mut(|v| v.keep(declaration.clone(), ()));
                }
                body.push_str(&declaration);
            }
            Err(error) => out.diagnostics.push((
                Some(winner),
                format!("{name} invalid at computed-value time: {error}"),
            )),
        }
    }
    let lowered = LOWERED.with_borrow_mut(|l| l.get(body.as_str()).cloned());
    let (props, said) = match lowered {
        Some(lowered) => lowered,
        None => {
            let lowered = lower(&body);
            LOWERED.with_borrow_mut(|l| l.keep(body, lowered.clone()));
            lowered
        }
    };
    out.props.extend(props);
    out.diagnostics
        .extend(said.into_iter().map(|said| (None, said)));
    out
}

/// What [`route`] knows of computed declarations, by their text
/// ([`crate::css::memo`]): those that parse, and each body's props.
type Lowered = (Vec<crate::doc::PropSet>, Vec<String>);
type RouteKey = (crate::layout::Flow, Winners);
thread_local! {
    static ROUTED: std::cell::RefCell<crate::css::memo::Memo<RouteKey, std::sync::Arc<Routed>>> =
        Default::default();
    static VALID: std::cell::RefCell<crate::css::memo::Memo<String, ()>> = Default::default();
    static LOWERED: std::cell::RefCell<crate::css::memo::Memo<String, Lowered>> =
        Default::default();
}

/// The reflected props of declarations `body` and what was wrong with them.
fn lower(body: &str) -> Lowered {
    let (mut props, mut said) = (Vec::new(), Vec::new());
    match css::declarations(body, 0) {
        Ok(entries) => {
            for entry in entries {
                let Some((spec, prop)) = schema::prop(&entry.key) else {
                    continue;
                };
                match crate::doc::parse_value(prop.ty, &entry.values) {
                    Ok(value) => props.push(crate::doc::PropSet {
                        spec,
                        value,
                        line: entry.line,
                        source: crate::doc::written(&entry.values),
                    }),
                    // A checkbox label's `row-reverse` is its caption side
                    // (`labelDirection`), not a box layout it lacks.
                    Err(_)
                        if entry.key == "direction"
                            && entry.values.iter().any(
                                |v| matches!(v, og_core::ogt::Value::Word(w) if w == "rowReverse"),
                            ) => {}
                    Err(error) => said.push(format!(
                        "{} invalid at computed-value time: {error}",
                        entry.key
                    )),
                }
            }
        }
        Err(error) => said.push(error),
    }
    (props, said)
}

/// Custom property `name`'s value with its `if()`s replaced, kept in
/// `done`; a `style()` test of another custom property reads that one's
/// value resolved the same way, and one of a property being resolved (a
/// cycle) is false (CSS Values 5 §8.3). `reads`: the names tests read.
fn custom_if(
    name: &str,
    computed: &variables::Computed,
    done: &mut BTreeMap<String, Option<String>>,
    stack: &mut Vec<String>,
    media: &mut dyn FnMut(&crate::doc::media::Query) -> bool,
    reads: &mut BTreeSet<String>,
) -> Result<Option<String>, String> {
    if let Some(v) = done.get(name) {
        return Ok(v.clone());
    }
    let value = computed.get(name).cloned().flatten();
    let Some(text) = value.as_deref().filter(|v| v.contains("if(")) else {
        return Ok(value);
    };
    stack.push(name.to_string());
    let mut error = None;
    // The tests run one at a time; a style() test's own media() tests too.
    let media = std::cell::RefCell::new(media);
    let result = {
        let mut style = |n: &str| {
            reads.insert(n.to_string());
            if stack.iter().any(|s| s == n) {
                return None;
            }
            match custom_if(n, computed, done, stack, *media.borrow_mut(), reads) {
                Ok(v) => v,
                Err(e) => {
                    error.get_or_insert(e);
                    None
                }
            }
        };
        let mut media_test = |q: &crate::doc::media::Query| (media.borrow_mut())(q);
        crate::doc::stylesheet::replace_ifs(
            text,
            &mut crate::doc::stylesheet::IfTests {
                media: &mut media_test,
                style: &mut style,
            },
        )
    };
    stack.pop();
    let value = result.map(Some);
    let value = match error {
        Some(e) => Err(e),
        None => value,
    };
    // An if() that does not resolve makes its property guaranteed-invalid.
    done.insert(name.to_string(), value.clone().ok().flatten());
    value
}
