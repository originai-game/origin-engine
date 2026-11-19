//! CSS Variables Level 1: compute on the declaring element, then inherit.
//! Dependency edges include fallback references even when the fallback is unused.
use cssparser::{Delimiter, ParseError, Parser, Token};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
enum Piece {
    Text(String),
    Block(String, Value, char),
    Var(String, Option<Value>),
    Bound(Box<crate::bind::Expr>),
}

/// A token stream, not concatenated identifiers: `var(--n)px` is not a length.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Value(Vec<Piece>);

/// Custom values and inherited standard declarations have separate namespaces.
/// None is the guaranteed-invalid custom value; an empty string is valid.
///
/// An element shares its parent's custom values. They are a shared base
/// (the root's tokens, most often) and a small layer of the names set over
/// it between that base and this element, so an element that sets one
/// copies the layer, never the base: a style's thousand tokens cost an
/// element nothing to inherit or to set one over.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(from = "ComputedSnapshot", into = "ComputedSnapshot")]
pub struct Computed {
    base: Arc<Map>,
    /// The names set over `base`: `Some` a value, `None` a name removed.
    layer: Arc<BTreeMap<String, Option<Option<String>>>>,
    pub(crate) properties: BTreeMap<String, String>,
    /// Current-tick before-change inheritance; derived, never snapshotted.
    /// Equality and dependency invalidation compare effective values only.
    pub(crate) before: Option<Arc<Computed>>,
    /// Current-tick target inheritance, with keyframes but no transitions.
    /// Like `before`, this is derived and excluded from snapshots/equality.
    pub(crate) after: Option<Arc<Computed>>,
    pub(crate) animation_tainted: BTreeSet<String>,
}

type Map = BTreeMap<String, Option<String>>;

/// Effective values only: Arc layering and motion contexts are runtime details.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct ComputedSnapshot {
    custom: Map,
    properties: BTreeMap<String, String>,
    animation_tainted: BTreeSet<String>,
}

impl From<Computed> for ComputedSnapshot {
    fn from(value: Computed) -> Self {
        Self {
            custom: value.iter().map(|(n, v)| (n.clone(), v.clone())).collect(),
            properties: value.properties,
            animation_tainted: value.animation_tainted,
        }
    }
}

impl From<ComputedSnapshot> for Computed {
    fn from(value: ComputedSnapshot) -> Self {
        Self {
            base: Arc::new(value.custom),
            properties: value.properties,
            animation_tainted: value.animation_tainted,
            ..Default::default()
        }
    }
}

/// The layer folds into a new base once it holds more names than this and
/// a quarter of the base: a layer stays small next to what it shares.
const LAYER: usize = 32;

impl PartialEq for Computed {
    fn eq(&self, other: &Self) -> bool {
        self.properties == other.properties
            && self.animation_tainted == other.animation_tainted
            && (self.shares(other)
                || if Arc::ptr_eq(&self.base, &other.base) {
                    self.layer == other.layer
                } else {
                    self.iter().eq(other.iter())
                })
    }
}

/// The names of a base and a layer over it, in order, the layer's over
/// the base's and without the names it removes.
struct Merged<'a> {
    base: std::iter::Peekable<std::collections::btree_map::Iter<'a, String, Option<String>>>,
    layer:
        std::iter::Peekable<std::collections::btree_map::Iter<'a, String, Option<Option<String>>>>,
}

impl<'a> Iterator for Merged<'a> {
    type Item = (&'a String, &'a Option<String>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let order = match (self.base.peek(), self.layer.peek()) {
                (None, None) => return None,
                (Some(_), None) => std::cmp::Ordering::Greater,
                (None, Some(_)) => std::cmp::Ordering::Less,
                (Some((b, _)), Some((l, _))) => l.cmp(b),
            };
            if order == std::cmp::Ordering::Equal {
                self.base.next();
            }
            let from_layer = order != std::cmp::Ordering::Greater;
            if !from_layer {
                return self.base.next();
            }
            if let (name, Some(value)) = self.layer.next()? {
                return Some((name, value));
            }
        }
    }
}

impl Computed {
    pub fn new() -> Self {
        Self::default()
    }

    /// A self-contained copy suitable for a retained cascade recipe.
    pub(crate) fn snapshot_values(&self) -> Self {
        Self {
            before: None,
            after: None,
            ..self.clone()
        }
    }

    /// Whether both hold the same maps (one was cloned from the other and
    /// neither set a name since).
    fn shares(&self, other: &Computed) -> bool {
        Arc::ptr_eq(&self.base, &other.base) && Arc::ptr_eq(&self.layer, &other.layer)
    }

    /// Whether both hold the same values, answered only where that is
    /// cheap: they share their base. `false` may still be equal maps.
    pub(crate) fn same(&self, other: &Computed) -> bool {
        self.properties == other.properties
            && self.before == other.before
            && self.after == other.after
            && self.animation_tainted == other.animation_tainted
            && (self.shares(other)
                || (Arc::ptr_eq(&self.base, &other.base) && self.layer == other.layer))
    }

    pub fn get(&self, name: &str) -> Option<&Option<String>> {
        match self.layer.get(name) {
            Some(set) => set.as_ref(),
            None => self.base.get(name),
        }
    }

    pub fn contains_key(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Option<String>)> {
        Merged {
            base: self.base.iter().peekable(),
            layer: self.layer.iter().peekable(),
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.iter().map(|(name, _)| name)
    }

    pub fn insert(&mut self, name: String, value: Option<String>) -> Option<Option<String>> {
        if name.starts_with("--og-") {
            return None;
        }
        let old = self.get(&name).cloned();
        // An unchanged value (a child's carried flow, most often) must not
        // copy the layer it shares.
        if old.as_ref() == Some(&value) {
            return old;
        }
        if self.base.get(&name) == Some(&value) {
            Arc::make_mut(&mut self.layer).remove(&name);
        } else {
            Arc::make_mut(&mut self.layer).insert(name, Some(value));
        }
        self.fold();
        old
    }

    pub fn remove(&mut self, name: &str) -> Option<Option<String>> {
        let old = self.get(name).cloned()?;
        let layer = Arc::make_mut(&mut self.layer);
        if self.base.contains_key(name) {
            layer.insert(name.to_owned(), None);
        } else {
            layer.remove(name);
        }
        self.fold();
        Some(old)
    }

    fn fold(&mut self) {
        if self.layer.len() <= LAYER.max(self.base.len() / 4) {
            return;
        }
        self.share_base();
    }

    /// Finish a common ancestor's values before descendants inherit them.
    /// Its last, below-threshold batch of tokens belongs in the shared
    /// base too, not in every descendant's copy-on-write overlay.
    pub(crate) fn share_base(&mut self) {
        if self.layer.is_empty() {
            return;
        }
        let base: Map = self
            .iter()
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect();
        self.base = Arc::new(base);
        self.layer = Default::default();
    }

    pub fn extend(&mut self, values: impl IntoIterator<Item = (String, Option<String>)>) {
        for (name, value) in values {
            self.insert(name, value);
        }
    }

    /// These values with `over`'s laid on top: every name `over` holds with
    /// its value, the rest with these. Starts from `over`'s maps (shared,
    /// not copied) and, where both share a base, visits only the layers:
    /// a host's computed values over the root tokens cost a lookup a name
    /// either of them set, not one a token.
    pub fn under(mut self, over: &Computed) -> Computed {
        let mut out = Computed {
            base: over.base.clone(),
            layer: over.layer.clone(),
            properties: std::mem::take(&mut self.properties),
            before: over.before.clone(),
            after: over.after.clone(),
            animation_tainted: over.animation_tainted.clone(),
        };
        out.properties
            .extend(over.properties.iter().map(|(k, v)| (k.clone(), v.clone())));
        if Arc::ptr_eq(&self.base, &over.base) {
            // What `over` lacks of these is in their layer, or a base
            // name `over`'s layer removed.
            for (name, value) in self.layer.iter() {
                if let Some(value) = value
                    && !over.contains_key(name)
                {
                    out.insert(name.clone(), value.clone());
                }
            }
            for (name, removed) in over.layer.iter() {
                if removed.is_none()
                    && let Some(value) = self.get(name)
                {
                    out.insert(name.clone(), value.clone());
                }
            }
        } else {
            for (name, value) in self.iter() {
                if !over.contains_key(name) {
                    out.insert(name.clone(), value.clone());
                }
            }
        }
        out
    }

    pub fn dependency_names(&self) -> impl Iterator<Item = &String> {
        self.keys().chain(self.properties.keys())
    }

    /// The names whose [`Self::dependency`] differs between these values
    /// and `other`, in order. The custom maps both share (an element's,
    /// carried from its parent unchanged) are not walked.
    pub fn changed<'a>(&'a self, other: &'a Computed) -> Vec<&'a String> {
        let contexts = [
            (self, other),
            (
                self.before.as_deref().unwrap_or(self),
                other.before.as_deref().unwrap_or(other),
            ),
            (
                self.after.as_deref().unwrap_or(self),
                other.after.as_deref().unwrap_or(other),
            ),
        ];
        let mut names: BTreeSet<&String> = self
            .animation_tainted
            .symmetric_difference(&other.animation_tainted)
            .collect();
        for (a, b) in contexts {
            if a.properties != b.properties {
                names.extend(a.properties.keys().chain(b.properties.keys()));
            }
            // Before/after inheritance shares the root token maps too.
            // Inspect only the overlays where those maps have changed.
            if a.shares(b) {
                continue;
            }
            if Arc::ptr_eq(&a.base, &b.base) {
                names.extend(a.layer.keys().chain(b.layer.keys()));
            } else {
                names.extend(a.keys().chain(b.keys()));
            }
        }
        names
            .into_iter()
            .filter(|name| {
                contexts
                    .iter()
                    .any(|(a, b)| a.dependency(name) != b.dependency(name))
                    || self.animation_tainted.contains(*name)
                        != other.animation_tainted.contains(*name)
            })
            .collect()
    }

    pub fn dependency(&self, name: &str) -> Option<&str> {
        self.properties
            .get(name)
            .map(String::as_str)
            .or_else(|| self.get(name).and_then(Option::as_deref))
    }
}

impl std::ops::Index<&str> for Computed {
    type Output = Option<String>;

    fn index(&self, name: &str) -> &Option<String> {
        self.get(name).expect("no custom property of that name")
    }
}

impl<const N: usize> From<[(String, Option<String>); N]> for Computed {
    fn from(values: [(String, Option<String>); N]) -> Self {
        let mut computed = Self::new();
        computed.extend(values);
        computed
    }
}

impl Value {
    pub fn bound(expression: crate::bind::Expr) -> Self {
        Self(vec![Piece::Bound(Box::new(expression))])
    }

    pub fn binding(&self) -> Option<&crate::bind::Expr> {
        match self.0.as_slice() {
            [Piece::Bound(expression)] => Some(expression),
            _ => None,
        }
    }

    pub fn binding_mut(&mut self) -> Option<&mut crate::bind::Expr> {
        match self.0.as_mut_slice() {
            [Piece::Bound(expression)] => Some(expression),
            _ => None,
        }
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        parse(&mut Parser::new(text)).map_err(|e| format!("invalid CSS value: {e:?}"))
    }

    /// The evaluated token stream without substituting any custom properties.
    /// A bound expression must have been evaluated before retaining a recipe.
    fn unresolved(&self) -> Result<String, String> {
        let mut out = String::new();
        for piece in &self.0 {
            match piece {
                Piece::Text(text) => out.push_str(text),
                Piece::Block(open, value, close) => {
                    out.push_str(open);
                    out.push_str(&value.unresolved()?);
                    out.push(*close);
                }
                Piece::Var(name, fallback) => {
                    out.push_str("var(");
                    cssparser::serialize_identifier(name, &mut out).expect("writing to a string");
                    if let Some(fallback) = fallback {
                        out.push(',');
                        out.push_str(&fallback.unresolved()?);
                    }
                    out.push(')');
                }
                Piece::Bound(_) => return Err("unevaluated inline style binding".into()),
            }
        }
        Ok(out)
    }

    pub fn dependencies(&self, out: &mut BTreeSet<String>) {
        for piece in &self.0 {
            match piece {
                Piece::Var(name, fallback) => {
                    out.insert(name.clone());
                    if let Some(fallback) = fallback {
                        fallback.dependencies(out);
                    }
                }
                Piece::Block(_, value, _) => value.dependencies(out),
                Piece::Text(_) | Piece::Bound(_) => {}
            }
        }
    }

    /// Whether it substitutes when its element resolves: it reads a
    /// `var()` or has a `random-item()`, an `if()` or a custom function
    /// call, which substitute as `var()` does (CSS Values 5 §8.3, §9.2;
    /// CSS Mixins 1 §3).
    pub fn has_variables(&self) -> bool {
        let mut names = BTreeSet::new();
        self.dependencies(&mut names);
        !names.is_empty()
            || self.has_function("random-item(")
            || self.has_function("if(")
            || self.has_dashed_call()
    }

    /// Whether it calls a custom function (`--name(…)`).
    fn has_dashed_call(&self) -> bool {
        self.0.iter().any(|piece| match piece {
            Piece::Block(o, value, _) => {
                (o.starts_with("--") && o.ends_with('(')) || value.has_dashed_call()
            }
            Piece::Var(_, Some(fallback)) => fallback.has_dashed_call(),
            _ => false,
        })
    }

    fn has_function(&self, open: &str) -> bool {
        self.0.iter().any(|piece| match piece {
            Piece::Block(o, value, _) => o.eq_ignore_ascii_case(open) || value.has_function(open),
            Piece::Var(_, Some(fallback)) => fallback.has_function(open),
            _ => false,
        })
    }

    /// Unresolved engine tokens can be supplied by the active style. The caller
    /// must not supply a token for a locally declared guaranteed-invalid value.
    pub fn substitute(
        &self,
        lookup: &mut impl FnMut(&str) -> Option<String>,
    ) -> Result<String, String> {
        let mut out = String::new();
        for piece in &self.0 {
            match piece {
                Piece::Bound(_) => return Err("unevaluated inline style binding".into()),
                Piece::Text(text) => out.push_str(text),
                Piece::Block(open, value, close) => {
                    out.push_str(open);
                    out.push_str(&value.substitute(lookup)?);
                    out.push(*close);
                }
                Piece::Var(name, fallback) => {
                    let value = match lookup(name) {
                        Some(value) => value,
                        None => match fallback {
                            Some(value) => value.substitute(lookup)?,
                            None => {
                                return Err(format!("{name} is invalid at computed-value time"));
                            }
                        },
                    };
                    out.push(' ');
                    out.push_str(&value);
                    out.push(' ');
                }
            }
            if out.len() > 1_048_576 {
                return Err("custom property expansion exceeds 1 MiB".into());
            }
        }
        Ok(out)
    }
}

impl serde::Serialize for Value {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let text = self.unresolved().map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&text)
    }
}

impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = <String as serde::Deserialize>::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

pub(crate) fn declarations(text: &str) -> Result<Vec<super::StyleDeclaration>, String> {
    let mut input = Parser::new(text);
    let mut result = Vec::new();
    while !input.is_exhausted() {
        if input.try_parse(|p| p.expect_semicolon()).is_ok() {
            continue;
        }
        let declaration = input
            .parse_until_before(Delimiter::Semicolon, |input| {
                let name = input.expect_ident_cloned()?.to_string();
                input.expect_colon()?;
                let start = input.position();
                let value = input.parse_until_before(Delimiter::Bang, parse)?;
                let source = input.slice_from(start).trim().to_string();
                let important = if input.is_exhausted() {
                    false
                } else {
                    input.expect_delim('!')?;
                    input.expect_ident_matching("important")?;
                    true
                };
                Ok(super::StyleDeclaration {
                    name,
                    value,
                    source,
                    important,
                    pending_shorthand: None,
                })
            })
            .map_err(|error: ParseError<String>| format!("invalid CSS declaration: {error:?}"))?;
        result.push(declaration);
    }
    Ok(result)
}

fn parse(input: &mut Parser<'_>) -> Result<Value, ParseError<String>> {
    let mut pieces = Vec::new();
    while !input.is_exhausted() {
        let start = input.position();
        let token = input.next_including_whitespace_and_comments()?.clone();
        let open = input.slice_from(start).to_string();
        match token {
            Token::Function(name) if name.eq_ignore_ascii_case("var") => {
                let (name, fallback) = input.parse_nested_block(|input| {
                    let name = input.expect_ident_cloned()?.to_string();
                    if !name.starts_with("--") || name.len() == 2 {
                        return Err(ParseError::custom("var() requires a --name"));
                    }
                    let fallback = if input.is_exhausted() {
                        None
                    } else {
                        input.expect_comma()?;
                        Some(parse(input)?)
                    };
                    Ok((name, fallback))
                })?;
                pieces.push(Piece::Var(name, fallback));
            }
            Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock => {
                let close = match token {
                    Token::SquareBracketBlock => ']',
                    Token::CurlyBracketBlock => '}',
                    _ => ')',
                };
                let body = input.parse_nested_block(parse)?;
                pieces.push(Piece::Block(open, body, close));
            }
            Token::BadUrl(_)
            | Token::BadString(_)
            | Token::CloseParenthesis
            | Token::CloseSquareBracket
            | Token::CloseCurlyBracket => {
                return Err(ParseError::custom("unbalanced CSS value"));
            }
            Token::Comment(_) => pieces.push(Piece::Text(" ".into())),
            _ => pieces.push(Piece::Text(open)),
        }
    }
    Ok(Value(pieces))
}

/// Values have already won the cascade. Inherited values are computed, so
/// changing --a on this element cannot change an inherited --b: var(--a).
pub fn compute(
    inherited: &Computed,
    specified: &BTreeMap<String, Value>,
    tokens: &mut impl FnMut(&str) -> Option<String>,
) -> (Computed, Vec<String>, BTreeSet<String>) {
    let graph: BTreeMap<_, _> = specified
        .iter()
        .map(|(name, value)| {
            let mut dependencies = BTreeSet::new();
            value.dependencies(&mut dependencies);
            (name.clone(), dependencies)
        })
        .collect();
    fn visit(
        name: &str,
        graph: &BTreeMap<String, BTreeSet<String>>,
        stack: &mut Vec<String>,
        seen: &mut BTreeSet<String>,
        cycles: &mut BTreeSet<String>,
    ) {
        if let Some(start) = stack.iter().position(|n| n == name) {
            cycles.extend(stack[start..].iter().cloned());
            return;
        }
        if !seen.insert(name.into()) {
            return;
        }
        stack.push(name.into());
        if let Some(edges) = graph.get(name) {
            for edge in edges {
                visit(edge, graph, stack, seen, cycles);
            }
        }
        stack.pop();
    }
    let mut cycles = BTreeSet::new();
    for name in specified.keys() {
        visit(
            name,
            &graph,
            &mut Vec::new(),
            &mut BTreeSet::new(),
            &mut cycles,
        );
    }
    // The declared names resolve into their own map, cycles as
    // guaranteed-invalid; they are laid over the inherited values last,
    // where one that is unchanged copies nothing.
    let mut done: Map = cycles.iter().map(|name| (name.clone(), None)).collect();
    let mut errors: Vec<_> = cycles
        .iter()
        .map(|name| {
            format!(
                "custom property {name} participates in a cycle; invalid at computed-value time"
            )
        })
        .collect();
    fn resolve(
        name: &str,
        specified: &BTreeMap<String, Value>,
        inherited: &Computed,
        done: &mut Map,
        tokens: &mut impl FnMut(&str) -> Option<String>,
        errors: &mut Vec<String>,
        inherited_reads: &mut BTreeSet<String>,
    ) -> Option<String> {
        if name.starts_with("--og-") {
            return None;
        }
        let Some(value) = specified.get(name) else {
            inherited_reads.insert(name.into());
            return match inherited.get(name) {
                Some(value) => value.clone(),
                None => tokens(name),
            };
        };
        if let Some(value) = done.get(name) {
            return value.clone();
        }
        let mut significant = value.0.iter().filter(|piece| {
            !matches!(piece,
                Piece::Text(text) if text.trim().is_empty() || text.starts_with("/*")
            )
        });
        let keyword = match (significant.next(), significant.next()) {
            (Some(Piece::Text(word)), None) => word.to_ascii_lowercase(),
            _ => String::new(),
        };
        let result = match keyword.as_str() {
            "initial" => None,
            "inherit" | "unset" => {
                inherited_reads.insert(name.into());
                inherited.get(name).cloned().flatten()
            }
            _ => match value.substitute(&mut |dependency| {
                resolve(
                    dependency,
                    specified,
                    inherited,
                    done,
                    tokens,
                    errors,
                    inherited_reads,
                )
            }) {
                Ok(value) => Some(value),
                Err(error) => {
                    errors.push(format!("{name}: {error}"));
                    None
                }
            },
        };
        done.insert(name.into(), result.clone());
        result
    }
    let mut inherited_reads = BTreeSet::new();
    for name in specified.keys() {
        resolve(
            name,
            specified,
            inherited,
            &mut done,
            tokens,
            &mut errors,
            &mut inherited_reads,
        );
    }
    let mut out = inherited.clone();
    out.extend(done);
    (out, errors, inherited_reads)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn declarations(values: &[(&str, &str)]) -> BTreeMap<String, Value> {
        values
            .iter()
            .map(|(name, value)| (name.to_string(), Value::parse(value).unwrap()))
            .collect()
    }

    #[test]
    fn engine_names_cannot_enter_or_be_read_from_custom_properties() {
        let mut inherited = Computed::new();
        inherited.properties.insert("color".into(), "blue".into());
        inherited.insert("--og-v-color".into(), Some("red".into()));
        let (computed, errors, _) = compute(
            &inherited,
            &declarations(&[
                ("--og-v-color", "red"),
                ("--og-style-color", "red"),
                ("--first", "var(--og-v-color, green)"),
                ("--second", "var(--og-style-color, yellow)"),
            ]),
            &mut |_| Some("red".into()),
        );
        assert!(errors.is_empty());
        assert!(computed.keys().all(|name| !name.starts_with("--og-")));
        assert_eq!(computed["--first"].as_deref().unwrap().trim(), "green");
        assert_eq!(computed["--second"].as_deref().unwrap().trim(), "yellow");
        assert_eq!(computed.properties["color"], "blue");
    }

    #[test]
    fn inheritance_uses_computed_values_and_cycles_include_unused_fallbacks() {
        let (parent, errors, _) = compute(
            &Computed::new(),
            &declarations(&[("--a", "12px"), ("--b", "var(--a)")]),
            &mut |_| None,
        );
        assert!(errors.is_empty());
        let (child, errors, reads) = compute(
            &parent,
            &declarations(&[
                ("--a", "var(--b)"),
                ("--safe", "green"),
                ("--cycle", "var(--safe, var(--cycle))"),
                ("--use", "var(--cycle, 7px)"),
            ]),
            &mut |_| None,
        );
        assert_eq!(child["--a"].as_deref().unwrap().trim(), "12px");
        assert_eq!(child["--cycle"], None);
        assert_eq!(child["--use"].as_deref().unwrap().trim(), "7px");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("--cycle"));
        assert_eq!(reads, BTreeSet::from(["--b".into()]));
    }

    #[test]
    fn empty_fallback_and_token_boundaries_are_not_string_concatenation() {
        let (values, errors, reads) = compute(
            &Computed::new(),
            &declarations(&[
                ("--empty", "var(--missing,)"),
                ("--n", "20"),
                ("--dimension", "var(--n)px"),
                ("--quoted", "'var(--missing)'"),
                ("--comment", "18px /* note */"),
            ]),
            &mut |_| None,
        );
        assert!(errors.is_empty());
        assert_eq!(values["--empty"].as_deref().unwrap().trim(), "");
        assert_eq!(values["--dimension"].as_deref().unwrap().trim(), "20 px");
        assert_eq!(values["--quoted"].as_deref(), Some("'var(--missing)'"));
        assert_eq!(values["--comment"].as_deref().unwrap().trim(), "18px");
        assert_eq!(reads, BTreeSet::from(["--missing".into()]));
        for invalid in ["var(color)", "var(--a red)", "var()"] {
            assert!(Value::parse(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn dependency_reads_follow_only_evaluated_inherited_lookups() {
        let inherited = Computed::from([
            ("--parent".into(), Some("18px".into())),
            ("--reset".into(), Some("4px".into())),
            ("--invalid".into(), None),
        ]);
        let (values, errors, reads) = compute(
            &inherited,
            &declarations(&[
                ("--local", "32px"),
                ("--uses-local", "var(--local, var(--unused))"),
                ("--uses-fallback", "var(--invalid, var(--parent))"),
                ("--reset", " /* comment */ initial "),
                ("--reset-use", "var(--reset, 6px)"),
            ]),
            &mut |_| None,
        );
        assert!(errors.is_empty());
        assert_eq!(values["--reset"], None);
        assert_eq!(values["--reset-use"].as_deref().unwrap().trim(), "6px");
        assert_eq!(
            reads,
            BTreeSet::from(["--invalid".into(), "--parent".into()])
        );
    }
}
