//! CSS Containment Level 3 size queries: `container-type`, `container-name`,
//! the `container` shorthand and `@container` conditions.
//!
//! A query container publishes its content box through the settlement hook
//! (`Ui::watch_layout_container`); conditions read the last settled box of
//! the nearest eligible ancestor, so the first frame and every resize settle
//! before paint. A container does not apply size containment: a query whose
//! styles keep changing its container's size fails as a settlement cycle.
use super::media::{Context, Feature, Query, Reads};
use super::{StyleDeclaration, variables};
use std::collections::{BTreeMap, BTreeSet};

/// The reserved computed-value entry that carries the ancestor container
/// chain. Custom property names start with `--`, so it cannot collide, and it
/// inherits and invalidates exactly like a custom property.
pub(crate) const CHAIN: &str = "@container";

pub(crate) const LONGHANDS: [&str; 2] = ["container-name", "container-type"];

/// A container's `container-type`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    #[default]
    Normal,
    InlineSize,
    Size,
}

/// A query container among an element's ancestors.
#[derive(Clone, Debug, PartialEq)]
pub struct Container {
    pub key: u64,
    pub kind: Kind,
    pub names: Vec<String>,
}

/// One `@container [name] <condition>` prelude.
#[derive(Clone, Debug, PartialEq)]
pub struct Condition {
    pub name: Option<String>,
    pub query: Query,
}

/// A container's settled content box, CSS px.
pub type Sizes = BTreeMap<u64, (f32, f32)>;

pub(crate) fn longhand(name: &str) -> bool {
    LONGHANDS.contains(&name)
}

fn name_error(name: &str) -> Option<String> {
    if !super::syntax::identifier(name) {
        return Some(format!("container-name: `{name}` is not a CSS identifier"));
    }
    [
        "none", "and", "or", "not", "default", "initial", "inherit", "unset",
    ]
    .contains(&name.to_ascii_lowercase().as_str())
    .then(|| format!("container-name: `{name}` is reserved; choose another name"))
}

fn kind(value: &str) -> Result<Kind, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Ok(Kind::Normal),
        "size" => Ok(Kind::Size),
        "inline-size" => Ok(Kind::InlineSize),
        other => Err(format!(
            "container-type: `{other}` is unsupported; write normal, size or inline-size"
        )),
    }
}

fn names(value: &str) -> Result<Vec<String>, String> {
    let words: Vec<&str> = value.split_whitespace().collect();
    if words.len() == 1 && words[0].eq_ignore_ascii_case("none") {
        return Ok(Vec::new());
    }
    if words.is_empty() {
        return Err("container-name: write none or one or more names".into());
    }
    words
        .into_iter()
        .map(|w| name_error(w).map_or_else(|| Ok(w.to_string()), Err))
        .collect()
}

fn declaration(name: &str, value: String, important: bool) -> Result<StyleDeclaration, String> {
    Ok(StyleDeclaration {
        name: name.into(),
        value: variables::Value::parse(&value)?,
        source: value,
        important,
    })
}

/// Expands the `container` shorthand and validates the longhands at cook;
/// other declarations pass through.
pub(crate) fn expand(d: StyleDeclaration) -> Result<Vec<StyleDeclaration>, String> {
    match d.name.as_str() {
        "container" => {
            if d.value.has_variables() {
                return Err(
                    "container: write var() in container-name or container-type, not in the shorthand"
                        .into(),
                );
            }
            let (name, ty) = d.source.split_once('/').unwrap_or((&d.source, "normal"));
            names(name)?;
            kind(ty)?;
            Ok(vec![
                declaration("container-name", name.trim().to_string(), d.important)?,
                declaration("container-type", ty.trim().to_string(), d.important)?,
            ])
        }
        "container-type" if !d.value.has_variables() => kind(&d.source).map(|_| vec![d]),
        "container-name" if !d.value.has_variables() => names(&d.source).map(|_| vec![d]),
        _ => Ok(vec![d]),
    }
}

/// The element's own container from its winning longhands (substituted).
pub(crate) fn declared(
    values: &[(String, String)],
    diagnostics: &mut Vec<String>,
) -> Option<(Kind, Vec<String>)> {
    let mut out = (Kind::Normal, Vec::new());
    for (name, value) in values {
        let parsed = match name.as_str() {
            "container-type" => kind(value).map(|k| out.0 = k),
            _ => names(value).map(|n| out.1 = n),
        };
        if let Err(error) = parsed {
            diagnostics.push(format!("{error} (invalid at computed-value time)"));
        }
    }
    (out.0 != Kind::Normal).then_some(out)
}

/// The ancestor containers carried by a computed map, innermost last.
pub(crate) fn chain(computed: &variables::Computed) -> Vec<Container> {
    let Some(Some(text)) = computed.get(CHAIN) else {
        return Vec::new();
    };
    text.split(';')
        .filter_map(|entry| {
            let mut parts = entry.splitn(3, ':');
            let key = parts.next()?.parse().ok()?;
            let kind = match parts.next()? {
                "s" => Kind::Size,
                _ => Kind::InlineSize,
            };
            let names = parts
                .next()
                .unwrap_or_default()
                .split_whitespace()
                .map(str::to_string)
                .collect();
            Some(Container { key, kind, names })
        })
        .collect()
}

/// Adds the element's own container for its descendants.
pub(crate) fn push(computed: &mut variables::Computed, key: u64, kind: Kind, names: &[String]) {
    let entry = format!(
        "{key}:{}:{}",
        if kind == Kind::Size { "s" } else { "i" },
        names.join(" ")
    );
    let chain = match computed.get(CHAIN) {
        Some(Some(previous)) => format!("{previous};{entry}"),
        _ => entry,
    };
    computed.insert(CHAIN.into(), Some(chain));
}

impl Condition {
    pub fn parse(prelude: &str) -> Result<Self, String> {
        let prelude = prelude.trim();
        let word = prelude
            .split(|c: char| c.is_whitespace() || c == '(')
            .next()
            .unwrap_or_default();
        let named =
            !word.is_empty() && !["not", "and", "or"].contains(&word.to_ascii_lowercase().as_str());
        if named && let Some(error) = name_error(word) {
            return Err(format!("@container {error}"));
        }
        let (name, rest) = if named {
            (Some(word.to_string()), &prelude[word.len()..])
        } else {
            (None, prelude)
        };
        let first = rest.split_whitespace().next().unwrap_or_default();
        if name.is_some() && ["and", "or"].contains(&first.to_ascii_lowercase().as_str()) {
            return Err(format!(
                "@container {prelude}: a container name is followed by its condition; media types and `{first}` after it are not container syntax"
            ));
        }
        if rest.trim().is_empty() {
            return Err(format!(
                "@container {prelude}: needs a size condition, for example (width > 400px)"
            ));
        }
        Ok(Self {
            name,
            query: Query::parse_container(rest)
                .map_err(|error| format!("@container {prelude}: {error}"))?,
        })
    }

    /// Whether the condition reads the block axis, which only `size`
    /// containers answer.
    fn block(&self) -> bool {
        fn walk(q: &Query) -> bool {
            match q {
                Query::All => false,
                Query::Not(q) => walk(q),
                Query::And(qs) | Query::Or(qs) => qs.iter().any(walk),
                Query::Feature(f) => !matches!(f, Feature::Width(..)),
            }
        }
        walk(&self.query)
    }

    /// Evaluates against the nearest eligible ancestor in `chain`. No eligible
    /// container, or one not measured yet, is unknown and so false; `reads`
    /// gains the container whose size the answer depends on.
    pub fn matches(&self, chain: &[Container], sizes: &Sizes, reads: &mut BTreeSet<u64>) -> bool {
        let block = self.block();
        let Some(container) = chain.iter().rev().find(|c| {
            (!block || c.kind == Kind::Size)
                && self.name.as_ref().is_none_or(|n| c.names.contains(n))
        }) else {
            return false;
        };
        reads.insert(container.key);
        let Some(&(width, height)) = sizes.get(&container.key) else {
            return false;
        };
        self.query.matches(
            Context {
                width,
                height,
                reduced_motion: false,
            },
            &mut Reads::default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conditions_pick_the_nearest_eligible_named_container() {
        let mut computed = variables::Computed::new();
        push(&mut computed, 7, Kind::Size, &["card".into()]);
        push(&mut computed, 9, Kind::InlineSize, &[]);
        let chain = chain(&computed);
        assert_eq!(chain.len(), 2);
        let sizes = Sizes::from([(7, (500.0, 120.0)), (9, (300.0, 40.0))]);
        for (prelude, expected, read) in [
            // Width only: the inner inline-size container answers.
            ("(width > 299px)", true, 9),
            ("(inline-size >= 301px)", false, 9),
            // Height needs a size container: the outer one.
            ("(height > 100px)", true, 7),
            ("(min-block-size: 121px)", false, 7),
            ("card (width >= 500px)", true, 7),
            ("card (300px < width < 400px)", false, 7),
            ("not (width < 300px)", true, 9),
            ("(orientation: landscape)", true, 7),
        ] {
            let mut reads = BTreeSet::new();
            let c = Condition::parse(prelude).unwrap();
            assert_eq!(c.matches(&chain, &sizes, &mut reads), expected, "{prelude}");
            assert_eq!(reads, BTreeSet::from([read]), "{prelude}");
        }
        let mut reads = BTreeSet::new();
        let missing = Condition::parse("sidebar (width > 0px)").unwrap();
        assert!(!missing.matches(&chain, &sizes, &mut reads));
        assert!(reads.is_empty());
        // Not measured yet: unknown, false, but subscribed.
        assert!(!Condition::parse("(width > 0px)").unwrap().matches(
            &chain,
            &Sizes::new(),
            &mut reads
        ));
        assert_eq!(reads, BTreeSet::from([9]));
    }

    #[test]
    fn unsupported_container_syntax_fails_with_the_name() {
        for (prelude, needle) in [
            ("(prefers-reduced-motion)", "unsupported container feature"),
            ("screen and (width > 1px)", "not container syntax"),
            ("none (width > 1px)", "reserved"),
            ("card", "needs a size condition"),
            ("(width > 10cqw)", "unsupported media length unit cqw"),
            ("(scroll-state(stuck: top))", "invalid container query"),
        ] {
            let error = Condition::parse(prelude).unwrap_err();
            assert!(error.contains(needle), "{prelude}: {error}");
        }
        for (name, source, needle) in [
            ("container-type", "scroll-state", "scroll-state"),
            ("container-name", "none card", "reserved"),
            ("container", "card / anchored", "anchored"),
        ] {
            let d = declaration(name, source.into(), false).unwrap();
            let error = expand(d).unwrap_err();
            assert!(error.contains(needle), "{name}: {error}");
        }
        let expanded =
            expand(declaration("container", "card side / size".into(), false).unwrap()).unwrap();
        assert_eq!(
            expanded
                .iter()
                .map(|d| (d.name.as_str(), d.source.as_str()))
                .collect::<Vec<_>>(),
            [("container-name", "card side"), ("container-type", "size")]
        );
    }
}
