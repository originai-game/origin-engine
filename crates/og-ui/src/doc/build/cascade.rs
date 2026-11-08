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
    /// Query containers whose settled size the matched rules depend on.
    pub container_reads: BTreeSet<u64>,
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
    mut applies: impl FnMut(&StyleRule) -> bool,
) -> Cascade {
    let mut out = Cascade::default();
    let mut winners = BTreeMap::new();
    for (order, rule) in rules.iter().enumerate() {
        if !applies(rule) {
            continue;
        }
        for (position, declaration) in rule.declarations.iter().enumerate() {
            let priority = (declaration.important, order, position);
            let entry = winners.entry(declaration.name.as_str()).or_insert((
                priority,
                declaration,
                rule.line,
            ));
            if priority >= entry.0 {
                *entry = (priority, declaration, rule.line);
            }
        }
    }
    let mut specified: BTreeMap<_, _> = winners
        .iter()
        .filter(|(name, _)| name.starts_with("--"))
        .map(|(name, (_, declaration, _))| {
            declaration.value.dependencies(&mut out.reads);
            (name.to_string(), declaration.value.clone())
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
    let (computed, diagnostics, inherited_reads) =
        variables::compute(inherited, &specified, &mut |name| token(theme, name));
    out.diagnostics.extend(diagnostics);
    out.inherited_reads = inherited_reads;
    out.local_names = specified.keys().cloned().collect();
    out.computed = Arc::new(computed);
    let mut declarations: Vec<_> = winners
        .into_iter()
        .filter(|(name, _)| !name.starts_with("--"))
        .collect();
    declarations.sort_by_key(|(_, (priority, _, _))| *priority);
    for (name, (_, declaration, line)) in declarations {
        for prop in css::targets(name) {
            if let Some((spec, _)) = schema::prop(prop) {
                out.unset.insert(spec);
            }
        }
        let value = declaration.value.substitute(&mut |name| {
            out.reads.insert(name.into());
            if !specified.contains_key(name) {
                out.inherited_reads.insert(name.into());
            }
            match out.computed.get(name) {
                Some(value) => value.clone(),
                None => token(theme, name),
            }
        });
        // Viewport units and rem become px; math functions reduce once
        // their variables are known.
        let value = value.and_then(|v| {
            let v = match basis {
                Some(basis) if crate::doc::calc::has_absolute(&v) => {
                    let (v, read) = crate::doc::calc::absolute(&v, basis);
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
    out
}

pub(crate) fn resolve(
    rules: &[StyleRule],
    inline: &[(String, String)],
    inherited: &variables::Computed,
    theme: &crate::Theme,
    basis: crate::doc::calc::Basis,
    applies: impl FnMut(&StyleRule) -> bool,
) -> Cascade {
    let mut out = resolve_values(rules, inline, inherited, theme, Some(basis), applies);
    let mut body = String::new();
    // The winners stay with the cascade: the UI debugger names each one's
    // line (`Ui::debug_element`).
    for (name, value, line) in out.values.clone() {
        if crate::doc::motion::longhand(&name) {
            out.motion.push((name, value));
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
