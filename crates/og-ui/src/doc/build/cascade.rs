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
    pub(crate) values: Vec<(String, String, usize)>,
}

pub(crate) fn resolve_values(
    rules: &[StyleRule],
    inherited: &variables::Computed,
    theme: &crate::Theme,
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
    let specified: BTreeMap<_, _> = winners
        .iter()
        .filter(|(name, _)| name.starts_with("--"))
        .map(|(name, (_, declaration, _))| {
            declaration.value.dependencies(&mut out.reads);
            (name.to_string(), declaration.value.clone())
        })
        .collect();
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
        match value {
            Ok(value) => out.values.push((name.into(), value, line)),
            Err(error) => out
                .diagnostics
                .push(format!("line {line}: {name}: {error}")),
        }
    }
    out
}

pub fn resolve(
    rules: &[StyleRule],
    inherited: &variables::Computed,
    theme: &crate::Theme,
    applies: impl FnMut(&StyleRule) -> bool,
) -> Cascade {
    let mut out = resolve_values(rules, inherited, theme, applies);
    let mut body = String::new();
    for (name, value, line) in out.values.drain(..) {
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
