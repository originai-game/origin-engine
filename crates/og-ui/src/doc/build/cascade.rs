//! The cascade operates on one instantiated element. Scheduling and parent
//! identity belong to the retained builder; CSS never creates a second scope.
use super::*;
use crate::doc::{StyleRule, css, variables};

#[derive(Clone, Default)]
pub struct Cascade {
    pub computed: Arc<variables::Computed>,
    pub reads: BTreeSet<String>,
    pub local_names: BTreeSet<String>,
    pub viewport_reads: crate::doc::media::Reads,
    /// Only evaluated lookups outside local declarations, including missing
    /// values and evaluated fallbacks. Inherited values are never re-expanded.
    pub inherited_reads: BTreeSet<String>,
    pub diagnostics: Vec<String>,
    pub props: Vec<crate::doc::PropSet>,
    pub unset: BTreeSet<usize>,
    /// The winning declarations, substituted, with the line each was
    /// written on.
    pub(crate) values: Vec<(String, String, usize)>,
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
                | "tab-size"
                | "caret-color"
                | "accent-color"
                | "color-scheme"
                | "writing-mode"
                | "paint-order"
                | "pointer-events"
                | "image-rendering"
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
/// `scrollbar-color` inherits; CSS Images 3 §5.3: `image-rendering`). A document declaring one carries its
/// values, as one writing `inherit` does.
pub(crate) const CARRIED_BY_DEFAULT: &[&str] = &[
    "scrollbar-color",
    "image-rendering",
    "list-style-type",
    "list-style-position",
    "list-style-image",
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
    /// `revert` or `revert-layer` with nothing below it: as `unset`.
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
}

pub(crate) fn values_with(
    rules: &[StyleRule],
    o: &Options<'_>,
    mut applies: impl FnMut(&StyleRule) -> bool,
) -> Cascade {
    let (inline, inherited, basis, attr) = (o.inline, o.inherited, o.basis, o.attr);
    let mut out = Cascade::default();
    // Every applying declaration of each property: (priority, declaration,
    // line, layer). Normal declarations of a later layer win; important
    // ones of an earlier layer (CSS Cascade 5 §6.4).
    type Candidate<'a> = (
        (bool, bool, u32, usize, usize),
        &'a crate::doc::StyleDeclaration,
        usize,
        u32,
    );
    let mut candidates: BTreeMap<&str, Vec<Candidate<'_>>> = BTreeMap::new();
    for (order, rule) in rules.iter().enumerate() {
        if (rule.starting && !o.starting) || !applies(rule) {
            continue;
        }
        for (position, declaration) in rule.declarations.iter().enumerate() {
            let layer = if declaration.important {
                u32::MAX - rule.layer
            } else {
                rule.layer
            };
            candidates
                .entry(declaration.name.as_str())
                .or_default()
                .push((
                    (declaration.important, rule.inline, layer, order, position),
                    declaration,
                    rule.line,
                    rule.layer,
                ));
        }
    }
    let mut winners = BTreeMap::new();
    for (name, mut list) in candidates {
        list.sort_by_key(|c| std::cmp::Reverse(c.0));
        // `revert` rolls back to the user-agent rules, `revert-layer` to
        // the layers below its own.
        let (mut ua_only, mut below) = (false, None);
        let mut winner = None;
        for (priority, declaration, line, layer) in &list {
            if (ua_only && *layer != 0) || below.is_some_and(|b| *layer >= b) {
                continue;
            }
            match crate::doc::stylesheet::wide_keyword(&declaration.source) {
                Some("revert") => ua_only = true,
                Some("revert-layer") if *layer > 0 => below = Some(*layer),
                Some("revert-layer") => ua_only = true,
                _ => {
                    winner = Some((*priority, Winner::Declared(declaration), *line));
                    break;
                }
            }
        }
        let first = list[0];
        winners.insert(name, winner.unwrap_or((first.0, Winner::Unset, first.2)));
    }
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
    for (name, value) in inline {
        if winners
            .get(name.as_str())
            .is_some_and(|((important, ..), ..)| *important)
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
            if !p.inherits || !map.contains_key(name.as_str()) {
                map.insert(name.clone(), p.initial.clone());
            }
        }
        adjusted = map;
        &adjusted
    };
    let (mut computed, diagnostics, inherited_reads) =
        variables::compute(inherited, &specified, &mut |_| None);
    for (name, p) in &registered {
        let value = computed.get(name.as_str()).cloned().flatten();
        let valid = value
            .as_deref()
            .is_some_and(|v| crate::doc::at_rules::matches_syntax(&p.syntax, v));
        if !valid {
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
    out.diagnostics.extend(diagnostics);
    out.inherited_reads = inherited_reads;
    out.local_names = specified.keys().cloned().collect();
    let mut declarations: Vec<_> = winners
        .into_iter()
        .filter(|(name, _)| !name.starts_with("--"))
        .collect();
    out.reduced_motion_authored = declarations.iter().any(|(name, (priority, winner, _))| {
        name.starts_with("transition")
            && matches!(winner, Winner::Declared(_))
            && names_reduced_motion(&rules[priority.3].query.media)
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
                    if let Some(value) = crate::doc::lists::initial(name) {
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
            (None, Winner::Declared(declaration)) => declaration.value.substitute(&mut |name| {
                out.reads.insert(name.into());
                if !specified.contains_key(name) {
                    out.inherited_reads.insert(name.into());
                }
                match out.computed.get(name) {
                    Some(value) => value.clone(),
                    None => None,
                }
            }),
            (None, Winner::Unset) => unreachable!("unset is a keyword"),
        };
        // attr() takes the element's attribute; viewport, root and font
        // units become px; math functions reduce once their variables are
        // known.
        let value = value.and_then(|v| {
            let v = match winner {
                Winner::Declared(d) if let Some(shorthand) = d.pending_shorthand => {
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
/// logical property as the physical one it names in `flow`.
pub(crate) fn route(out: &mut Cascade, flow: crate::layout::Flow) {
    let mut body = String::new();
    // The winners stay with the cascade: the UI debugger names each one's
    // line (`Ui::debug_element`).
    let values: Vec<(String, String, usize)> = out
        .values
        .iter()
        .flat_map(|(name, value, line)| {
            match crate::layout::flow::physical(name, value, flow) {
                Some(sides) => sides.into_iter().map(|(n, v)| (n, v, *line)).collect(),
                None if name.starts_with("justify-") || name.starts_with("place-") => {
                    // `left` and `right` are the line's physical sides.
                    let side = |w: &str| match (w, flow.rtl) {
                        ("left", false) | ("right", true) => "start".to_string(),
                        ("left", true) | ("right", false) => "end".to_string(),
                        (w, _) => w.to_string(),
                    };
                    let v: Vec<String> = value.split_whitespace().map(side).collect();
                    vec![(name.clone(), v.join(" "), *line)]
                }
                None => vec![(name.clone(), value.clone(), *line)],
            }
        })
        .filter(|(name, ..)| !crate::layout::flow::longhand(name))
        .collect();
    let values = values
        .into_iter()
        .flat_map(|(name, value, line)| {
            if name == "font" {
                match crate::doc::text_css::font(&value) {
                    Ok(values) => return values.into_iter().map(|(n, v)| (n, v, line)).collect(),
                    Err(error) => {
                        out.diagnostics.push(format!("line {line}: font: {error}"));
                        return Vec::new();
                    }
                }
            }
            vec![(name, value, line)]
        })
        .collect::<Vec<_>>();
    for (name, value, line) in values {
        if name == "font-family" {
            match crate::doc::text_css::family_list(&value) {
                Ok((_, role)) => {
                    out.text.push((name, value));
                    body.push_str(&format!("font-family: {role};"));
                }
                Err(error) => out.diagnostics.push(format!("line {line}: {error}")),
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
        match css::declarations(&declaration, line) {
            Ok(_) => body.push_str(&declaration),
            Err(error) => out.diagnostics.push(format!(
                "line {line}: {name} invalid at computed-value time: {error}"
            )),
        }
    }
    match css::declarations(&body, 0) {
        Ok(entries) => {
            for entry in entries {
                let Some((spec, prop)) = schema::prop(&entry.key) else {
                    continue;
                };
                match crate::doc::parse_value(prop.ty, &entry.values) {
                    Ok(value) => out.props.push(crate::doc::PropSet {
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
                    Err(error) => out.diagnostics.push(format!(
                        "{} invalid at computed-value time: {error}",
                        entry.key
                    )),
                }
            }
        }
        Err(error) => out.diagnostics.push(error),
    }
}
