//! CSS Containment Level 3 container queries: `container-type`,
//! `container-name`, the `container` shorthand and `@container` size,
//! `style()` and `scroll-state()` conditions.
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

/// A container's whole `container-type`: `normal | [ size | inline-size ]
/// || scroll-state || anchored`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Type {
    pub size: Kind,
    pub scroll_state: bool,
    /// An anchor-positioned element's fallback state (`anchored(fallback:
    /// …)`), which anchor positioning publishes.
    pub anchored: bool,
}

/// A query container among an element's ancestors.
#[derive(Clone, Debug, PartialEq)]
pub struct Container {
    pub key: u64,
    pub kind: Kind,
    /// A vertical writing mode: its inline axis is its height.
    pub vertical: bool,
    pub scroll_state: bool,
    pub names: Vec<String>,
}

/// What a container condition is evaluated against besides the chain: the
/// settled boxes, a container's scroll-state flags ([`scroll`]) and the
/// element's inherited computed values (for `style()`).
pub struct Probe<'a> {
    pub sizes: &'a Sizes,
    pub scroll: &'a dyn Fn(u64) -> u16,
    pub inherited: &'a variables::Computed,
}

/// What evaluating a container condition read: the containers whose box
/// or scroll state it depends on, and the inherited entries `style()` read.
#[derive(Clone, Debug, Default)]
pub struct Read {
    pub sizes: BTreeSet<u64>,
    pub scroll: BTreeSet<u64>,
    pub inherited: BTreeSet<String>,
}

/// The containers container units resolve against, with their settled
/// content size along the axis (0 until measured): the nearest with size
/// containment across (`cqw`), down (`cqh`), on its own inline axis
/// (`cqi`, `inline-size` or `size`) and on its block axis (`cqb`, `size`).
/// A container's inline axis is its height in a vertical writing mode.
pub(crate) fn unit_basis(chain: &[Container], sizes: &Sizes) -> [Option<(u64, f32)>; 4] {
    let pick = |eligible: &dyn Fn(&Container) -> bool, height: &dyn Fn(&Container) -> bool| {
        let c = chain.iter().rev().find(|c| eligible(c))?;
        let size = sizes
            .get(&c.key)
            .map_or(0.0, |s| if height(c) { s.1 } else { s.0 });
        Some((c.key, size))
    };
    // Containment on its inline axis: either size type.
    let inline = |c: &Container| c.kind != Kind::Normal;
    let both = |c: &Container| c.kind == Kind::Size;
    [
        pick(&|c| both(c) || (inline(c) && !c.vertical), &|_| false),
        pick(&|c| both(c) || (inline(c) && c.vertical), &|_| true),
        pick(&inline, &|c| c.vertical),
        pick(&both, &|c| !c.vertical),
    ]
}

/// The computed-value entry holding container `key`'s custom property
/// `name` for its descendants' `style()` queries.
pub(crate) fn style_entry(key: u64, name: &str) -> String {
    format!("@cq:{key}:{name}")
}

/// The size features a condition reads ([`Condition::needs`]).
#[derive(Clone, Copy, Debug, Default)]
struct Needs {
    width: bool,
    height: bool,
    inline: bool,
    /// Both axes (`aspect-ratio`, `orientation`, `block-size`).
    both: bool,
    scroll: bool,
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

fn kind(value: &str) -> Result<Type, String> {
    let value = value.trim().to_ascii_lowercase();
    let mut out = Type::default();
    let words: Vec<&str> = value.split_whitespace().collect();
    if words == ["normal"] {
        return Ok(out);
    }
    let error = || {
        format!(
            "container-type: `{value}` is invalid; write normal, or size or inline-size, scroll-state and anchored, each once"
        )
    };
    for word in &words {
        match *word {
            "size" | "inline-size" if out.size == Kind::Normal => {
                out.size = if *word == "size" {
                    Kind::Size
                } else {
                    Kind::InlineSize
                };
            }
            "scroll-state" if !out.scroll_state => out.scroll_state = true,
            "anchored" if !out.anchored => out.anchored = true,
            _ => return Err(error()),
        }
    }
    if words.is_empty() {
        return Err(error());
    }
    Ok(out)
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
) -> Option<(Type, Vec<String>)> {
    let mut out = (Type::default(), Vec::new());
    for (name, value) in values {
        let parsed = match name.as_str() {
            "container-type" => kind(value).map(|k| out.0 = k),
            _ => names(value).map(|n| out.1 = n),
        };
        if let Err(error) = parsed {
            diagnostics.push(format!("{error} (invalid at computed-value time)"));
        }
    }
    // A named element is a style container whatever its type.
    (out.0 != Type::default() || !out.1.is_empty()).then_some(out)
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
            let flags = parts.next()?;
            let kind = match flags.as_bytes().first() {
                Some(b's') => Kind::Size,
                Some(b'i') => Kind::InlineSize,
                _ => Kind::Normal,
            };
            let scroll_state = flags.contains('S');
            let vertical = flags.contains('V');
            let names = parts
                .next()
                .unwrap_or_default()
                .split_whitespace()
                .map(str::to_string)
                .collect();
            Some(Container {
                key,
                kind,
                vertical,
                scroll_state,
                names,
            })
        })
        .collect()
}

/// Adds the element's own container for its descendants, with a copy of
/// its custom properties for their `style()` queries; `vertical`: its
/// writing mode is vertical.
pub(crate) fn push(
    computed: &mut variables::Computed,
    key: u64,
    ty: Type,
    names: &[String],
    vertical: bool,
) {
    let custom: Vec<(String, Option<String>)> = computed
        .iter()
        .filter(|(name, _)| name.starts_with("--"))
        .map(|(name, value)| (style_entry(key, name), value.clone()))
        .collect();
    computed.extend(custom);
    let entry = format!(
        "{key}:{}{}{}:{}",
        match ty.size {
            Kind::Size => "s",
            Kind::InlineSize => "i",
            Kind::Normal => "n",
        },
        if ty.scroll_state { "S" } else { "" },
        if vertical { "V" } else { "" },
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
        // A word directly before `(` is a function: style(), scroll-state().
        let function = prelude[word.len()..].starts_with('(');
        let named = !word.is_empty()
            && !function
            && !["not", "and", "or"].contains(&word.to_ascii_lowercase().as_str());
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

    /// What the query container must be for this condition: the size
    /// features it reads (width, height, inline size, both axes), scroll
    /// state, or (style queries alone) any element.
    fn needs(&self) -> Needs {
        let mut n = Needs::default();
        self.query.features(&mut |f| match f {
            Feature::Width(..) => n.width = true,
            Feature::Height(..) => n.height = true,
            Feature::InlineSize(..) => n.inline = true,
            Feature::ScrollState { .. } => n.scroll = true,
            Feature::Style(..) => {}
            _ => n.both = true,
        });
        n
    }

    /// Evaluates against the nearest eligible ancestor in `chain`. No eligible
    /// container, or one not measured yet, is unknown and so false; `read`
    /// gains what the answer depends on. An unnamed style-only condition
    /// queries the parent element, every element being a style container.
    pub fn matches(&self, chain: &[Container], probe: &Probe<'_>, read: &mut Read) -> bool {
        let needs = self.needs();
        let sized = needs.width || needs.height || needs.inline || needs.both;
        let (inline, block, needs_scroll) = (sized, sized, needs.scroll);
        let parent = self.name.is_none() && !sized && !needs_scroll;
        let container = if parent {
            None
        } else {
            let Some(c) = chain.iter().rev().find(|c| {
                // Its inline axis is its height in a vertical writing mode;
                // a feature on its block axis needs `size`.
                let block_axis =
                    needs.both || (needs.width && c.vertical) || (needs.height && !c.vertical);
                let inline_axis = needs.inline || needs.width || needs.height;
                (!block_axis || c.kind == Kind::Size)
                    && (!inline_axis || c.kind != Kind::Normal)
                    && (!needs_scroll || c.scroll_state)
                    && self.name.as_ref().is_none_or(|n| c.names.contains(n))
            }) else {
                return false;
            };
            Some(c)
        };
        let size = match container {
            Some(c) if inline || block => {
                read.sizes.insert(c.key);
                let Some(&size) = probe.sizes.get(&c.key) else {
                    return false;
                };
                Some(size)
            }
            _ => None,
        };
        let flags = match container {
            Some(c) if needs_scroll => {
                read.scroll.insert(c.key);
                (probe.scroll)(c.key)
            }
            _ => 0,
        };
        let (width, height) = size.unwrap_or_default();
        let context = Context {
            width,
            height,
            ..Context::default()
        };
        self.query.test(&mut |feature| match feature {
            Feature::Style(name, value) => {
                let entry = container.map_or_else(|| name.clone(), |c| style_entry(c.key, name));
                let actual = probe.inherited.get(&entry).cloned().flatten();
                read.inherited.insert(entry);
                match (value, actual) {
                    (_, None) => false,
                    (None, Some(_)) => true,
                    (Some(v), Some(a)) => super::media::normalize(&a) == *v,
                }
            }
            &Feature::ScrollState { group, bits } => {
                if bits == 0 {
                    flags & group == 0
                } else {
                    flags & bits != 0
                }
            }
            // Its logical sizes through its writing mode.
            &Feature::InlineSize(cmp, v) | &Feature::BlockSize(cmp, v)
                if container.is_some_and(|c| c.vertical) =>
            {
                let f = match feature {
                    Feature::InlineSize(..) => Feature::Height(cmp, v),
                    _ => Feature::Width(cmp, v),
                };
                context.feature(&f, &mut Reads::default())
            }
            f => context.feature(f, &mut Reads::default()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn probe<'a>(sizes: &'a Sizes, inherited: &'a variables::Computed) -> Probe<'a> {
        Probe {
            sizes,
            scroll: &|key| if key == 5 { 0b0100 | (1 << 4) } else { 0 },
            inherited,
        }
    }

    #[test]
    fn conditions_pick_the_nearest_eligible_named_container() {
        let mut computed = variables::Computed::new();
        computed.insert("--tone".into(), Some(" warm  red ".into()));
        let scroller = Type {
            scroll_state: true,
            ..Type::default()
        };
        push(&mut computed, 5, scroller, &[], false);
        let size = Type {
            size: Kind::Size,
            ..Type::default()
        };
        push(&mut computed, 7, size, &["card".into()], false);
        computed.insert("--tone".into(), Some("cool".into()));
        let inline = Type {
            size: Kind::InlineSize,
            ..Type::default()
        };
        push(&mut computed, 9, inline, &[], false);
        let chain = chain(&computed);
        assert_eq!(chain.len(), 3);
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
            let mut read_ = Read::default();
            let c = Condition::parse(prelude).unwrap();
            let p = probe(&sizes, &computed);
            assert_eq!(c.matches(&chain, &p, &mut read_), expected, "{prelude}");
            assert_eq!(read_.sizes, BTreeSet::from([read]), "{prelude}");
        }
        // Scroll state: only the scroll-state container answers; it can
        // scroll down and is snapped on x.
        for (prelude, expected) in [
            ("scroll-state(scrollable: bottom)", true),
            ("scroll-state(scrollable: top)", false),
            ("scroll-state(scrollable: y)", true),
            ("scroll-state(scrollable)", true),
            ("scroll-state(snapped: x)", true),
            ("scroll-state(snapped: both)", false),
            ("scroll-state(stuck: none)", true),
            ("not scroll-state(scrolled)", true),
        ] {
            let mut read_ = Read::default();
            let c = Condition::parse(prelude).unwrap();
            let p = probe(&sizes, &computed);
            assert_eq!(c.matches(&chain, &p, &mut read_), expected, "{prelude}");
            assert_eq!(read_.scroll, BTreeSet::from([5]), "{prelude}");
        }
        // Style: unnamed queries the parent's computed value; a named one
        // the container's own, as it was where it was declared.
        for (prelude, expected, entry) in [
            ("style(--tone: cool)", true, "--tone".to_string()),
            ("style(--tone)", true, "--tone".to_string()),
            ("style(--missing)", false, "--missing".to_string()),
            (
                "card style(--tone: warm red)",
                true,
                style_entry(7, "--tone"),
            ),
            ("card style(--tone: cool)", false, style_entry(7, "--tone")),
            (
                "card style((--tone: warm red) and (not (--tone: x)))",
                true,
                style_entry(7, "--tone"),
            ),
        ] {
            let mut read_ = Read::default();
            let c = Condition::parse(prelude).unwrap();
            let p = probe(&sizes, &computed);
            assert_eq!(c.matches(&chain, &p, &mut read_), expected, "{prelude}");
            assert_eq!(read_.inherited, BTreeSet::from([entry]), "{prelude}");
        }
        // Mixed: the nearest container that is both a size container and named.
        let c = Condition::parse("card (width > 400px) and style(--tone: warm red)").unwrap();
        assert!(c.matches(&chain, &probe(&sizes, &computed), &mut Read::default()));
        let mut read_ = Read::default();
        let missing = Condition::parse("sidebar (width > 0px)").unwrap();
        assert!(!missing.matches(&chain, &probe(&sizes, &computed), &mut read_));
        assert!(read_.sizes.is_empty());
        // Not measured yet: unknown, false, but subscribed.
        let empty = Sizes::new();
        assert!(!Condition::parse("(width > 0px)").unwrap().matches(
            &chain,
            &probe(&empty, &computed),
            &mut read_
        ));
        assert_eq!(read_.sizes, BTreeSet::from([9]));
        assert_eq!(
            unit_basis(&chain, &sizes),
            [
                Some((9, 300.0)),
                Some((7, 120.0)),
                Some((9, 300.0)),
                Some((7, 120.0))
            ]
        );
        // A vertical inline-size container inside: its inline size is its
        // height; its width is its block axis, which it does not contain.
        let mut vertical = computed.clone();
        push(&mut vertical, 11, inline, &[], true);
        let chain = super::chain(&vertical);
        let sizes = Sizes::from([(7, (500.0, 120.0)), (9, (300.0, 40.0)), (11, (60.0, 250.0))]);
        for (prelude, expected, read) in [
            ("(inline-size > 200px)", true, 11),
            ("(height > 200px)", true, 11),
            ("(width < 100px)", false, 9),
            ("(block-size > 100px)", true, 7),
        ] {
            let mut read_ = Read::default();
            let c = Condition::parse(prelude).unwrap();
            let p = probe(&sizes, &vertical);
            assert_eq!(c.matches(&chain, &p, &mut read_), expected, "{prelude}");
            assert_eq!(read_.sizes, BTreeSet::from([read]), "{prelude}");
        }
        assert_eq!(
            unit_basis(&chain, &sizes),
            [
                Some((9, 300.0)),
                Some((11, 250.0)),
                Some((11, 250.0)),
                Some((7, 120.0))
            ]
        );
    }

    #[test]
    fn unsupported_container_syntax_fails_with_the_name() {
        for (prelude, needle) in [
            ("(prefers-reduced-motion)", "unsupported container feature"),
            ("screen and (width > 1px)", "not container syntax"),
            ("none (width > 1px)", "reserved"),
            ("card", "needs a size condition"),
            ("(width > 10cqw)", "unsupported media length unit cqw"),
            (
                "scroll-state(sticky: top)",
                "unsupported scroll-state feature",
            ),
            (
                "scroll-state(stuck: x)",
                "unsupported scroll-state(stuck) value",
            ),
            ("style(color: red)", "custom properties"),
            ("media(width > 1px)", "unsupported container function"),
        ] {
            let error = Condition::parse(prelude).unwrap_err();
            assert!(error.contains(needle), "{prelude}: {error}");
        }
        for (name, source, needle) in [
            ("container-type", "size inline-size", "invalid"),
            ("container-type", "scroll-state scroll-state", "invalid"),
            ("container-name", "none card", "reserved"),
            ("container", "card / normal size", "invalid"),
        ] {
            let d = declaration(name, source.into(), false).unwrap();
            let error = expand(d).unwrap_err();
            assert!(error.contains(needle), "{name}: {error}");
        }
        assert_eq!(
            kind("anchored inline-size scroll-state").unwrap(),
            Type {
                size: Kind::InlineSize,
                scroll_state: true,
                anchored: true
            }
        );
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
