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
    /// Winning container-name and container-type, substituted.
    pub(crate) container: Vec<(String, String)>,
    /// Winning SVG presentation properties (`fill`, `stroke`…), substituted.
    pub(crate) svg: Vec<(String, String)>,
    /// Winning text and font properties ([`crate::doc::text_css`]), substituted.
    pub(crate) text: Vec<(String, String)>,
    /// Winning `transform` and 3D properties ([`crate::transform3d`]).
    pub(crate) transform: Vec<(String, String)>,
    /// Query containers whose settled size the matched rules depend on.
    pub container_reads: BTreeSet<u64>,
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
        )
}

/// The computed-map key that carries a property's value to the children
/// that write `inherit` (or `unset` on an inherited property).
pub(crate) fn carried(name: &str) -> String {
    format!("--og-v-{name}")
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
    theme: &crate::Theme,
    basis: Option<crate::doc::calc::Basis>,
    attr: &dyn Fn(&str) -> Option<String>,
    applies: impl FnMut(&StyleRule) -> bool,
) -> Cascade {
    values_with(
        rules,
        &Options {
            inline,
            inherited,
            theme,
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
    pub theme: &'a crate::Theme,
    pub basis: Option<crate::doc::calc::Basis>,
    /// The element's attributes, for `attr()`.
    pub attr: &'a dyn Fn(&str) -> Option<String>,
    /// Its document's `@property` registrations.
    pub sheet: Option<&'a crate::doc::at_rules::Sheet>,
    /// Include `@starting-style` rules (the values a first style's
    /// transitions start from).
    pub starting: bool,
    /// Carry the winners to children's `inherit` ([`carried`]).
    pub carry: bool,
}

pub(crate) fn values_with(
    rules: &[StyleRule],
    o: &Options<'_>,
    mut applies: impl FnMut(&StyleRule) -> bool,
) -> Cascade {
    let (inline, inherited, theme, basis, attr) = (o.inline, o.inherited, o.theme, o.basis, o.attr);
    let mut out = Cascade::default();
    // Every applying declaration of each property: (priority, declaration,
    // line, layer). Normal declarations of a later layer win; important
    // ones of an earlier layer (CSS Cascade 5 §6.4).
    type Candidate<'a> = (
        (bool, u32, usize, usize),
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
                    (declaration.important, layer, order, position),
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
        variables::compute(inherited, &specified, &mut |name| token(theme, name));
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
    declarations.sort_by_key(|(_, (priority, _, _))| *priority);
    let carry_prefix = carried("");
    if o.carry {
        // A child inherits the carried values of inherited properties
        // only; the rest are this element's own.
        computed.retain(|name, _| {
            name.strip_prefix(&carry_prefix)
                .is_none_or(inherited_property)
        });
    }
    out.computed = Arc::new(computed);
    // The element's own type step sizes its font-relative units.
    let basis = basis.map(|mut basis| {
        let step = declarations
            .iter()
            .find(|(name, _)| *name == "font")
            .and_then(|(_, (_, w, _))| {
                let Winner::Declared(d) = w else { return None };
                d.value
                    .substitute(&mut |n| match out.computed.get(n) {
                        Some(v) => v.clone(),
                        None => token(theme, n),
                    })
                    .ok()
            })
            .and_then(|v| {
                let name = v
                    .trim()
                    .strip_prefix("var(--text-")?
                    .strip_suffix(')')?
                    .to_string();
                let mut ramp = theme.type_ramp;
                crate::style_file::ramp_step(&mut ramp, &css::camel(&name)).copied()
            });
        if let Some(step) = step {
            basis.em = step.size;
            basis.lh = step.line_height;
        }
        basis
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
                    continue;
                }
                let key = carried(name);
                out.inherited_reads.insert(key.clone());
                match inherited.get(&key).cloned().flatten() {
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
                    None => token(theme, name),
                }
            }),
            (None, Winner::Unset) => unreachable!("unset is a keyword"),
        };
        // attr() takes the element's attribute; viewport, root and font
        // units become px; math functions reduce once their variables are
        // known.
        let value = value.and_then(|v| {
            let v = if v.contains("attr(") {
                crate::doc::calc::attr(&v, attr)?
            } else {
                v
            };
            let v = match basis {
                Some(basis) if crate::doc::calc::per_element(name, &v) => {
                    let (v, read) = crate::doc::calc::absolute(&v, basis, name);
                    out.viewport_reads.width |= read.width;
                    out.viewport_reads.height |= read.height;
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
        let computed = Arc::make_mut(&mut out.computed);
        for (name, value, _) in &out.values {
            computed.insert(carried(name), Some(value.clone()));
        }
    }
    out
}

pub(crate) fn resolve(
    rules: &[StyleRule],
    o: &Options<'_>,
    applies: impl FnMut(&StyleRule) -> bool,
) -> Cascade {
    let mut out = values_with(rules, o, applies);
    let mut body = String::new();
    // The winners stay with the cascade: the UI debugger names each one's
    // line (`Ui::debug_element`).
    for (name, value, line) in out.values.clone() {
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
    out
}

fn token(theme: &crate::Theme, name: &str) -> Option<String> {
    let bare = name.strip_prefix("--")?;
    if let Some(value) = theme.length(bare) {
        return Some(format!("{value}px"));
    }
    if theme.color(&css::camel(bare)).is_some() {
        return Some(format!("var({name})"));
    }
    if let Some(step) = bare.strip_prefix("text-")
        && crate::style_file::ramp_step(&mut theme.type_ramp.clone(), &css::camel(step)).is_some()
    {
        return Some(format!("var({name})"));
    }
    // Motion roles remain symbolic until the active style resolves them.
    if let Some(role) = bare.strip_prefix("motion-")
        && crate::motion::MotionToken::from_name(role).is_some()
    {
        return Some(format!("var({name})"));
    }
    if let Some(role) = bare.strip_prefix("font-")
        && FontRole::from_name(role).is_some()
    {
        return Some(format!("var({name})"));
    }
    None
}
