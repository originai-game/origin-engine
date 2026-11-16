//! Scoped selectors lower once to matching document nodes and reactive predicates.
use super::StyleDeclaration;
use super::selectors::Selector;
use super::syntax::{Attr, Error};
use super::*;
use crate::bind::InteractionState;
use crate::doc::schema::Kind;
use std::borrow::Cow;

#[derive(Clone, Debug, Default, PartialEq)]
struct Rule {
    selector: String,
    body: String,
    query: AtQuery,
    at: usize,
    body_at: usize,
    order: usize,
    /// HTML's user-agent stylesheet ([`super::lists::UA_CSS`]): cascades
    /// before every author rule, whatever their specificity.
    ua: bool,
    global: bool,
    /// Its cascade layer's full name (`a.b`); `None` is unlayered.
    layer: Option<String>,
    /// Inside `@starting-style`.
    starting: bool,
    /// The box pseudo-element it selects ([`BOXES`]): `body` is what its
    /// originating element takes, `boxed` what the pseudo-element's own
    /// node ([`DocNode::pseudo`]) does.
    pseudo: Option<&'static str>,
    boxed: String,
    /// The state pseudo-classes written after the pseudo-element
    /// (`::backdrop:hover`): the pseudo-element's own states.
    pseudo_states: Vec<InteractionState>,
}

/// Where a rule is read: the conditions, layer, scope and nesting parent
/// around it.
#[derive(Clone, Default)]
struct Context {
    query: AtQuery,
    layer: Option<String>,
    /// The enclosing style rule's selector list (CSS nesting).
    parent: Option<String>,
    /// `@scope (root) to (limit)`.
    scope: Option<(String, Option<String>)>,
    starting: bool,
}

/// What reading a document's stylesheets collects besides rules.
struct Reader<'a> {
    sheet: &'a mut super::at_rules::Sheet,
    /// Cascade layer names in first-seen order.
    layers: Vec<String>,
    anonymous: usize,
    /// The document's asset id, for paths written beside it.
    base: String,
    /// `@import`s may still come (none but `@layer` statements yet).
    imports_open: bool,
    depth: usize,
}

fn rank(layers: &[String], layer: &Option<String>) -> u32 {
    match layer {
        None => u32::MAX,
        Some(name) => layers
            .iter()
            .position(|l| l == name)
            .map_or(u32::MAX - 1, |k| k as u32 + 1),
    }
}

/// `rules` under each selector of their lists, in the order read.
fn selectors(rules: Vec<Rule>) -> Result<Vec<(Selector, Rule)>, Error> {
    // `::scroll-button(*)` is the four buttons, each its own rule.
    let mut rules: Vec<Rule> = rules
        .into_iter()
        .flat_map(|rule| {
            if !rule.selector.to_ascii_lowercase().contains(ANY_BUTTON) {
                return vec![rule];
            }
            ["up", "down", "left", "right"]
                .map(|d| {
                    let mut r = rule.clone();
                    r.selector =
                        replace_ci(&rule.selector, ANY_BUTTON, &format!("::scroll-button({d})"));
                    r
                })
                .to_vec()
        })
        .collect();
    for rule in &mut rules {
        pseudo_element(rule)?;
    }
    let mut parsed = Vec::new();
    for rule in rules {
        for mut selector in Selector::parse_list(&rule.selector).map_err(|e| (rule.at, e))? {
            selector.global = rule.global;
            let mut rule = rule.clone();
            rule.selector = selector.source.clone();
            root_scheme(&mut selector, &mut rule.query);
            parsed.push((selector, rule));
        }
    }
    Ok(parsed)
}

/// `text` with every ASCII-case-insensitive `from` replaced by `to`.
fn replace_ci(text: &str, from: &str, to: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let (mut out, mut at) = (String::new(), 0);
    while let Some(k) = lower[at..].find(from) {
        out.push_str(&text[at..at + k]);
        out.push_str(to);
        at += k + from.len();
    }
    out.push_str(&text[at..]);
    out
}

/// The host sets `data-scheme` on the root to the theme's scheme, as a page
/// sets it on `<html>`. Every document's root is a `:root`, and none of them
/// carries the attribute, so `:root[data-scheme=…]` is read as what it tests:
/// the scheme, a `prefers-color-scheme` condition on the rule. The attribute
/// is replaced by a second `:root`, which keeps the selector's specificity.
fn root_scheme(selector: &mut Selector, query: &mut AtQuery) {
    use super::media::{Feature, Query};
    use super::selectors::{Component, attribute};
    let scheme_on_root = |compound: &[&Component], c: &Component| {
        compound.iter().any(|c| matches!(c, Component::Root))
            && attribute(c).is_some_and(|(name, _)| name == "data-scheme")
    };
    let tests: Vec<_> = selector
        .compounds()
        .0
        .iter()
        .flat_map(|compound| {
            compound
                .iter()
                .filter(|c| scheme_on_root(compound, c))
                .map(|c| attribute(c).and_then(|(_, test)| test))
        })
        .collect();
    if tests.is_empty() {
        return;
    }
    for test in tests {
        let holds = |scheme: &str| test.as_ref().is_none_or(|t| t.test(scheme));
        let scheme = match (holds("light"), holds("dark")) {
            (true, true) => Query::All,
            (false, true) => Query::Feature(Feature::Dark(true)),
            (true, false) => Query::Feature(Feature::Dark(false)),
            (false, false) => Query::Not(Box::new(Query::All)),
        };
        query.media = match std::mem::take(&mut query.media) {
            Query::All => scheme,
            Query::And(mut all) => {
                all.push(scheme);
                Query::And(all)
            }
            media => Query::And(vec![media, scheme]),
        };
    }
    *selector = selector.rewritten(|compound, c| scheme_on_root(compound, c).then_some(":root"));
}

/// Where a rule sorts in the cascade: user-agent rules first, then by
/// layer, specificity and the order read.
fn cascade_key(selector: &Selector, rule: &Rule, layers: &[String]) -> impl Ord + use<> {
    let layer = if rule.ua {
        0
    } else {
        rank(layers, &rule.layer)
    };
    (!rule.ua, layer, selector.specificity(), rule.order)
}

fn parsed_rules(rules: Vec<Rule>, layers: &[String]) -> Result<Vec<(Selector, Rule)>, Error> {
    let mut parsed = selectors(rules)?;
    parsed.sort_by_key(|(selector, rule)| cascade_key(selector, rule, layers));
    Ok(parsed)
}

thread_local! {
    /// The bytes of global stylesheet text this thread has read.
    static TEXT_READ: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// How much global stylesheet text this thread has read, in bytes: a
/// sheet's own text and the stylesheets it was given to `@import`. A test
/// of what is read once counts them.
#[doc(hidden)]
pub fn text_read() -> usize {
    TEXT_READ.get()
}

/// One rule of a [`Reading`] under one of its selectors.
#[derive(Clone, Debug, PartialEq)]
struct ReadRule {
    selector: Selector,
    rule: Rule,
    /// The line of its sheet it was written on.
    line: usize,
    /// Its body lowered with the sheet's keyframes known ([`lower`]): its
    /// declarations, and the text of those checked as written.
    declarations: Vec<StyleDeclaration>,
    checked: String,
}

/// A global stylesheet read once: tokenized, its selectors parsed and its
/// bodies lowered. The style's `::part()` rules ([`Reading::part_rules`])
/// and what dresses a document's elements ([`Global`]) are both made from
/// one reading, and the default style's ([`DEFAULT`]) is kept for every
/// sheet that holds it.
#[derive(Debug, PartialEq)]
pub(crate) struct Reading {
    source: String,
    /// What its at-rules declare, its keyframes, and the stylesheets it
    /// was given to `@import`.
    sheet: super::at_rules::Sheet,
    /// Its cascade layers in first-seen order, and the anonymous ones.
    layers: Vec<String>,
    anonymous: usize,
    /// How many rules it read, HTML's user-agent rules (the first `ua`)
    /// among them: a rule read after it is ordered from here.
    count: usize,
    ua: usize,
    rules: Vec<ReadRule>,
}

/// The default style, read once in the process: the part sheet of a UI
/// without a pack, and what a pack's rules and a part sheet that begins
/// with it are laid beside ([`Reading::with_default`]).
pub(crate) static DEFAULT: std::sync::LazyLock<Reading> = std::sync::LazyLock::new(|| {
    let read = |css| {
        Reading::read("<part-css>", css, 0, Vec::new())
            .expect("generated default CSS is checked by the theme drift test")
    };
    let (default, empty) = (read(crate::DEFAULT_CSS), read(""));
    // What lets its rules be laid beside another sheet's as they were read:
    // nothing but their own text decides how they read.
    assert!(
        default.sheet == empty.sheet
            && default.layers == empty.layers
            && default.anonymous == 0
            && !default
                .rules
                .iter()
                .any(|r| r.rule.body.contains("animation")),
        "the default style is rules in og-default: an at-rule that declares something, another layer or an animation in it needs Reading::with_default to carry it"
    );
    default
});

/// Where the default style's rules go beside a sheet's own.
#[derive(Clone, Copy)]
enum DefaultRules {
    /// Before them: the sheet's text begins with the default style's.
    Before,
    /// After them, their lines counted from the line after this one: a
    /// pack's sheets are read first.
    After(usize),
}

impl Reading {
    /// Reads `css` from `from` on (written beside `source`) with the
    /// stylesheets it `@import`s.
    fn read(
        source: &str,
        css: &str,
        from: usize,
        imports: Vec<(String, String)>,
    ) -> Result<Self, String> {
        let given = imports.iter().map(|(_, text)| text.len()).sum::<usize>();
        TEXT_READ.set(TEXT_READ.get() + css.len() - from + given);
        let breaks: Vec<usize> = css.match_indices('\n').map(|(at, _)| at).collect();
        let line = |at: usize| breaks.partition_point(|&b| b < at) + 1;
        let error = |(at, message): Error| format!("{source}:{}: {message}", line(at));
        let mut sheet = super::at_rules::Sheet {
            imports,
            ..Default::default()
        };
        let mut rules = Vec::new();
        let mut reader = Reader::new(&mut sheet, source);
        reader.user_agent(&mut rules).map_err(error)?;
        let ua = rules.len();
        // What follows another sheet's rules is past its `@import`s.
        reader.imports_open = from == 0;
        reader
            .rules(&css[from..], from, &Context::default(), &mut rules)
            .map_err(error)?;
        let (layers, anonymous) = (std::mem::take(&mut reader.layers), reader.anonymous);
        let count = rules.len();
        keyframes(&mut sheet, &mut rules).map_err(error)?;
        let known = |name: &str| sheet.keyframe_rules.iter().any(|(n, ..)| n == name);
        let mut out = Vec::new();
        for (selector, rule) in selectors(rules).map_err(error)? {
            let components = selector.compounds().0;
            if !rule.ua {
                if components.iter().flatten().any(|c| {
                    matches!(c, super::selectors::Component::LocalName(tag) if tag.name.0 != tag.name.0.to_ascii_lowercase())
                }) {
                    return Err(error((rule.at, "use lowercase markup tags in global CSS".into())));
                }
                if components
                    .iter()
                    .flatten()
                    .any(|c| matches!(c, super::selectors::Component::Part(_)))
                {
                    part_selector(&selector).map_err(|e| error((rule.at, e)))?;
                }
            }
            let (declarations, checked) =
                lower(&rule.body, &known).map_err(|e| error((rule.at, e)))?;
            // Its place from here on is its line: a document it dresses
            // has none for it.
            let line = line(rule.at);
            out.push(ReadRule {
                line,
                selector,
                rule: Rule {
                    at: 0,
                    body_at: 0,
                    ..rule
                },
                declarations,
                checked,
            });
        }
        Ok(Self {
            source: source.to_owned(),
            sheet,
            layers,
            anonymous,
            count,
            ua,
            rules: out,
        })
    }

    /// Reads a part sheet. One that begins with the default style's text
    /// takes that from [`DEFAULT`] and reads only what follows it.
    fn read_sheet(source: &str, css: &str, imports: Vec<(String, String)>) -> Result<Self, String> {
        if css.starts_with(crate::DEFAULT_CSS) {
            let own = Self::read(source, css, crate::DEFAULT_CSS.len(), imports)?;
            Ok(own.with_default(DefaultRules::Before))
        } else {
            Self::read(source, css, 0, imports)
        }
    }

    /// This sheet with the default style's rules beside its own, as
    /// reading the two texts as one reads them.
    fn with_default(mut self, place: DefaultRules) -> Self {
        let default = &*DEFAULT;
        let own = self.count - self.ua;
        let rules = default.rules[default.ua_rules()..].iter().cloned();
        match place {
            DefaultRules::Before => {
                let at = self.ua_rules();
                for r in &mut self.rules[at..] {
                    r.rule.order += default.count - default.ua;
                }
                self.rules.splice(at..at, rules);
            }
            DefaultRules::After(lines) => self.rules.extend(rules.map(|mut r| {
                r.rule.order += own;
                r.line += lines;
                r
            })),
        }
        self.count += default.count - default.ua;
        self
    }

    /// How many of `rules` are HTML's user-agent rules, which come first.
    fn ua_rules(&self) -> usize {
        self.rules.iter().take_while(|r| r.rule.ua).count()
    }

    /// The style's part rules in cascade order, and what the sheet
    /// declares.
    fn part_rules(&self) -> (Vec<StyleRule>, super::at_rules::Sheet) {
        let sheet = super::at_rules::Sheet {
            layers: order_layers(&self.layers),
            ..self.sheet.clone()
        };
        let mut rules: Vec<&ReadRule> = self.rules.iter().filter(|r| !r.rule.ua).collect();
        rules.sort_by_key(|r| cascade_key(&r.selector, &r.rule, &sheet.layers));
        let rules = rules
            .into_iter()
            .map(|r| StyleRule {
                inline: false,
                context: super::TreeContext::Tree,
                depth: 0,
                layer: rank(&sheet.layers, &r.rule.layer),
                starting: r.rule.starting,
                query: r.rule.query.clone(),
                condition: None,
                structure: Some(std::sync::Arc::new(r.selector.clone())),
                instance_states: Vec::new(),
                props: Vec::new(),
                declarations: r.declarations.clone(),
                line: r.line,
            })
            .collect();
        (rules, sheet)
    }
}

/// The part rules and sheet of a style, as a UI holds them.
pub(crate) type PartSheet = (
    std::sync::Arc<Vec<StyleRule>>,
    std::sync::Arc<super::at_rules::Sheet>,
);

fn part_sheet(reading: &Reading) -> PartSheet {
    let (rules, sheet) = reading.part_rules();
    (std::sync::Arc::new(rules), std::sync::Arc::new(sheet))
}

/// The default style's part sheet.
pub(crate) static DEFAULT_PART_SHEET: std::sync::LazyLock<PartSheet> =
    std::sync::LazyLock::new(|| part_sheet(&DEFAULT));

/// A global part sheet, using the same reader, shorthand expansion and
/// cascade order as a document's CSS.
pub(crate) fn part_rules(css: &str) -> Result<PartSheet, String> {
    Reading::read_sheet("<part-css>", css, Vec::new()).map(|r| part_sheet(&r))
}

/// A style pack's sheets: `css` (its `@import`s of them, written beside
/// `source`) over the default style. Its part sheet, and what dresses a
/// document's elements under it, from one reading.
pub(crate) fn pack_sheets(
    source: &str,
    css: &str,
    imports: Vec<(String, String)>,
) -> Result<(PartSheet, Global), String> {
    let reading = pack_reading(source, css, imports)?;
    Ok((part_sheet(&reading), Global::new(reading)?))
}

fn pack_reading(
    source: &str,
    css: &str,
    imports: Vec<(String, String)>,
) -> Result<Reading, String> {
    let lines = css.lines().count();
    Ok(Reading::read(source, css, 0, imports)?.with_default(DefaultRules::After(lines)))
}

/// Checks `css` as a part sheet reads it.
pub fn part_rules_from(
    source: &str,
    css: &str,
    imports: Vec<(String, String)>,
) -> Result<(Vec<StyleRule>, super::at_rules::Sheet), String> {
    Reading::read_sheet(source, css, imports).map(|r| r.part_rules())
}

/// Visit this source's selectors using the stylesheet tokenizer and nesting
/// expansion. Imports are visited by the caller under their own filenames.
pub(crate) fn check_part_selectors(
    source: &str,
    css: &str,
    check: &impl Fn(&Selector) -> Result<(), String>,
) -> Vec<String> {
    fn walk(
        text: &str,
        base: usize,
        parent: Option<&str>,
        check: &impl Fn(&Selector) -> Result<(), String>,
        errors: &mut Vec<Error>,
    ) -> Result<(), Error> {
        for item in items(text, base)? {
            let Item::Block {
                prelude,
                block,
                at,
                block_at,
            } = item
            else {
                continue;
            };
            if let Some((name, _)) = at_rule(prelude) {
                if matches!(
                    name.as_str(),
                    "media" | "supports" | "container" | "layer" | "scope" | "starting-style"
                ) {
                    walk(block, block_at, parent, check, errors)?;
                }
                continue;
            }
            if prelude.trim_start().starts_with("--") {
                continue;
            }
            let combined = resolve_nested(prelude, parent);
            let rule = Rule {
                selector: combined.clone(),
                at,
                ..Default::default()
            };
            match parsed_rules(vec![rule], &[]) {
                Ok(selectors) => {
                    for (selector, _) in selectors {
                        if let Err(error) = check(&selector) {
                            errors.push((at, error));
                        }
                    }
                }
                Err(error) => errors.push(error),
            }
            walk(block, block_at, Some(&combined), check, errors)?;
        }
        Ok(())
    }
    let mut errors = Vec::new();
    if let Err(error) = walk(css, 0, None, check, &mut errors) {
        errors.push(error);
    }
    errors
        .into_iter()
        .map(|e| syntax::diagnostic(source, css, e))
        .collect()
}

pub fn pack_extensions(source: &str, css: &str) -> Result<(), String> {
    fn walk(text: &str, base: usize) -> Result<(), Error> {
        for item in items(text, base)? {
            match item {
                Item::Block {
                    block, block_at, ..
                } => walk(block, block_at)?,
                Item::Statement(text, at) if !text.starts_with('@') => {
                    for declaration in variables::declarations(text).map_err(|e| (at, e))? {
                        let name = declaration.name.to_ascii_lowercase();
                        if name.starts_with("-og-")
                            && !matches!(name.as_str(), "-og-material" | "-og-sound")
                        {
                            return Err((
                                at,
                                format!(
                                    "unknown appearance extension `{}`; CSS packs accept only -og-material and -og-sound",
                                    declaration.name
                                ),
                            ));
                        }
                    }
                }
                _ => (),
            }
        }
        Ok(())
    }
    walk(css, 0).map_err(|(at, e)| format!("{source}:{}: {e}", syntax::location(css, at).0))
}

fn part_selector(selector: &Selector) -> Result<(), String> {
    use super::selectors::{Component, PseudoClass, attribute, nested, structural};
    let bad = || {
        format!(
            "{}: use a lowercase markup tag, host classes/attributes, ::part(name), then state pseudo-classes",
            selector.source
        )
    };
    // The compounds before the host's read the host's own tree (its
    // ancestors and siblings), as a shadow tree's part is matched through
    // its host ([`Selector::local`]).
    let (compounds, _) = selector.compounds();
    let Some(simples) = compounds.last() else {
        return Err(bad());
    };
    if selector.span > 0 {
        return Err(bad());
    }
    if compounds.len() == 1
        && matches!(simples.first(), Some(Component::Root))
        && simples[1..].iter().all(|c| attribute(c).is_some())
    {
        return Ok(());
    }
    let mut part = false;
    for c in simples {
        match c {
            Component::LocalName(tag) if !part && tag.name.0 == tag.name.0.to_ascii_lowercase() => {
            }
            Component::ExplicitUniversalType | Component::Class(_) | Component::ID(_) if !part => {}
            c if !part && attribute(c).is_some() => {}
            c if !part && super::selectors::structural_component(c) => {}
            Component::Negation(_) | Component::Is(_) | Component::Where(_)
                if !part && nested(c).iter().all(|selector| !structural(selector)) => {}
            Component::Part(_) if !part => part = true,
            Component::NonTSPseudoClass(PseudoClass::State(_)) => {}
            _ => return Err(bad()),
        }
    }
    if !part {
        return Err(bad());
    }
    Ok(())
}

/// The CSS-wide keywords every property takes.
pub fn wide_keyword(value: &str) -> Option<&'static str> {
    let v = value.trim();
    [
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
        "revert-rule",
    ]
    .into_iter()
    .find(|k| v.eq_ignore_ascii_case(k))
}

/// Order merged layer names without allowing a late sublayer to escape its parent.
pub(crate) fn order_layers(names: &[String]) -> Vec<String> {
    fn visit(prefix: &str, names: &[String], out: &mut Vec<String>) {
        for name in names {
            if name
                .strip_prefix(prefix)
                .is_some_and(|rest| !rest.is_empty() && !rest.contains('.'))
            {
                visit(&format!("{name}."), names, out);
                out.push(name.clone());
            }
        }
    }
    let mut out = Vec::new();
    visit("", names, &mut out);
    out
}

/// What a rule's body lowers to: its declarations and their prop entries.
type Lowered = (Vec<StyleDeclaration>, Vec<og_core::ogt::Entry>);

/// One rule of a [`Global`] stylesheet under one of its selectors.
struct GlobalRule {
    selector: std::sync::Arc<Selector>,
    rule: Rule,
    /// Its body, and its box pseudo-element's, lowered with the sheet's own
    /// keyframes known. A `::part()` rule is lowered where it is placed.
    lowered: Option<(Lowered, Option<Lowered>)>,
    /// Its body names an animation, which reads differently once a
    /// document defines keyframes of its own.
    animates: bool,
}

/// A global stylesheet read once for every document it dresses: HTML's
/// user-agent rules, then a style pack's (or the default style's role
/// rules), tokenized, their selectors parsed and their bodies lowered. A
/// document's own `<style>` is read after it ([`apply`]), and only matching
/// its elements is left to do per document.
pub struct Global {
    /// What its at-rules declare, and its keyframes.
    sheet: super::at_rules::Sheet,
    /// Its cascade layers in first-seen order, and the anonymous ones.
    layers: Vec<String>,
    anonymous: usize,
    /// How many rules it read: a document's rules are ordered after them.
    count: usize,
    rules: Vec<GlobalRule>,
}

impl std::fmt::Debug for Global {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Global")
            .field("rules", &self.rules.len())
            .finish_non_exhaustive()
    }
}

/// The default style's role rules (`:where([part$=".frame"])` and the
/// rest, its last section), for a document shown without a pack: what
/// dresses a document's own elements. A style pack's sheet holds the whole
/// default; a document without a pack takes these, and reads their tokens
/// from the root cascade.
pub(crate) static DEFAULT_GLOBAL: std::sync::LazyLock<Global> = std::sync::LazyLock::new(|| {
    let roles = crate::DEFAULT_CSS[..role_rules()].matches('\n').count() + 1;
    let default = &*DEFAULT;
    let rules = default.rules.iter();
    Global::new(Reading {
        source: default.source.clone(),
        sheet: default.sheet.clone(),
        layers: default.layers.clone(),
        rules: rules
            .filter(|r| r.rule.ua || r.line >= roles)
            .cloned()
            .collect(),
        ..*default
    })
    .expect("the default style's CSS reads")
});

/// Where the default style's role rules begin in its text.
fn role_rules() -> usize {
    crate::DEFAULT_CSS
        .find("/* Roles:")
        .expect("the generated default CSS ends with its role rules")
}

impl Global {
    /// What dresses a document's elements, of a sheet as it was read.
    fn new(reading: Reading) -> Result<Self, String> {
        let Reading {
            source,
            mut sheet,
            layers,
            anonymous,
            count,
            rules,
            ..
        } = reading;
        sheet.imports.clear();
        let line = syntax::location("", 0).0;
        let known = |name: &str| sheet.keyframe_rules.iter().any(|(n, ..)| n == name);
        let mut out = Vec::new();
        for read in rules {
            let ReadRule {
                mut selector,
                mut rule,
                declarations,
                checked,
                line: written,
            } = read;
            let error = |e| format!("{source}:{written}: {e}");
            if !rule.ua {
                rule.global = true;
                selector.global = true;
            }
            let lowered = if rule.selector.contains("::part(") {
                None
            } else {
                let entries = css::declarations(&checked, line).map_err(error)?;
                let boxed = match rule.pseudo {
                    Some(_) if !rule.boxed.is_empty() => {
                        Some(lowered(&rule.boxed, line, &known).map_err(error)?)
                    }
                    _ => None,
                };
                Some(((declarations, entries), boxed))
            };
            out.push(GlobalRule {
                selector: std::sync::Arc::new(selector),
                animates: rule.body.contains("animation") || rule.boxed.contains("animation"),
                rule,
                lowered,
            });
        }
        Ok(Self {
            sheet,
            layers,
            anonymous,
            count,
            rules: out,
        })
    }
}

/// A rule body lowered and read as props, its entries at `line`.
fn lowered(body: &str, line: usize, known: &dyn Fn(&str) -> bool) -> Result<Lowered, String> {
    let (declarations, body) = lower(body, known)?;
    let entries = css::declarations(&body, line)?;
    Ok((declarations, entries))
}

/// Takes the `@keyframes` out of `rules` into `sheet`, each parsed: a
/// later definition of a name replaces an earlier one under the same
/// conditions.
fn keyframes(sheet: &mut super::at_rules::Sheet, rules: &mut Vec<Rule>) -> Result<(), Error> {
    for rule in rules.iter() {
        if let Some(name) = rule
            .selector
            .strip_prefix("@keyframes")
            .filter(|n| n.starts_with(char::is_whitespace))
        {
            let name = name.trim();
            if !matches!(
                super::css::tokens(name).as_deref(),
                Ok([super::css::Tok::Ident(_)])
            ) {
                return Err((rule.at, "@keyframes needs a CSS identifier".into()));
            }
            let mut frames = Vec::new();
            for item in items(&rule.body, rule.body_at)? {
                match item {
                    Item::Block { prelude, block, .. } => {
                        frames.push((prelude.to_string(), block.to_string()))
                    }
                    Item::Statement(_, at) => {
                        return Err((at, "a keyframe needs { declarations }".into()));
                    }
                }
            }
            let parsed = super::motion::keyframes(&frames)
                .map_err(|e| (rule.at, format!("@keyframes {name}: {e}")))?;
            let key = if rule.query == AtQuery::default() {
                name.to_string()
            } else {
                format!("{name}@{}", sheet.keyframe_rules.len())
            };
            sheet.keyframe_rules.retain(|(_, _, k)| *k != key);
            sheet.keyframes.retain(|(k, _)| *k != key);
            sheet
                .keyframe_rules
                .push((name.to_string(), rule.query.clone(), key.clone()));
            sheet.keyframes.push((key, parsed));
        }
    }
    rules.retain(|rule| !rule.selector.to_ascii_lowercase().starts_with("@keyframes"));
    Ok(())
}

/// Reads a document's `styles` into `sheet` and answers their rules. They
/// continue `global`'s layers, its keyframes and its rule order, and what
/// it declares is the sheet's.
fn read(
    source: &str,
    sheet: &mut super::at_rules::Sheet,
    styles: &[(String, usize)],
    global: &Global,
) -> Result<Vec<Rule>, Error> {
    let imports = std::mem::take(&mut sheet.imports);
    *sheet = super::at_rules::Sheet {
        imports,
        ..global.sheet.clone()
    };
    // Its rules' places, so the document's are ordered after them.
    let mut rules = Vec::new();
    rules.resize_with(global.count, Rule::default);
    let mut reader = Reader::new(sheet, source);
    reader.layers.clone_from(&global.layers);
    reader.anonymous = global.anonymous;
    for (text, at) in styles {
        reader.imports_open = true;
        // A built-in component's rules sit below every pack's.
        let context = Context {
            layer: source
                .trim_start_matches('/')
                .starts_with("@ui/")
                .then(|| "og-components".into()),
            ..Context::default()
        };
        reader.rules(text, *at, &context, &mut rules)?;
    }
    let layers = order_layers(&reader.layers);
    sheet.layers = layers;
    rules.drain(..global.count);
    keyframes(sheet, &mut rules)?;
    Ok(rules)
}

/// Places `global`'s rules and the document's own `styles` on the
/// document's elements, in cascade order.
pub fn apply(
    doc: &mut UiDocument,
    elements: &[(usize, String, Vec<Attr>)],
    styles: &[(String, usize)],
    global: &Global,
) -> Result<(), Error> {
    let mut rules = read(&doc.source, &mut doc.sheet, styles, global)?;
    doc.style_faces = global.sheet.font_faces.len();
    let layers = doc.sheet.layers.clone();
    let transitions: Vec<(&str, &str, usize, bool)> = rules
        .iter()
        .filter(|r| super::view_transition::selects(&r.selector))
        .map(|r| {
            let conditional = r.query != AtQuery::default();
            (r.selector.as_str(), r.body.as_str(), r.at, conditional)
        })
        .collect();
    super::view_transition::apply(doc, &transitions)?;
    rules.retain(|r| !super::view_transition::selects(&r.selector));
    let own: Vec<(std::sync::Arc<Selector>, Rule)> = selectors(rules)?
        .into_iter()
        .map(|(selector, rule)| (std::sync::Arc::new(selector), rule))
        .collect();
    let known: Vec<String> = doc
        .sheet
        .keyframe_rules
        .iter()
        .map(|(n, ..)| n.clone())
        .collect();
    let known = |name: &str| known.iter().any(|k| k == name);
    // A global rule's lowering holds unless the document's own keyframes
    // could change how its animation reads.
    let more_keyframes = doc.sheet.keyframe_rules.len() != global.sheet.keyframe_rules.len();
    // Cascade order: the global sheet's rules and the document's together.
    let mut order: Vec<(&std::sync::Arc<Selector>, &Rule, Option<&GlobalRule>)> = global
        .rules
        .iter()
        .map(|g| (&g.selector, &g.rule, Some(g)))
        .chain(own.iter().map(|(selector, rule)| (selector, rule, None)))
        .collect();
    order.sort_by_key(|(selector, rule, _)| cascade_key(selector, rule, &layers));
    for (parsed, rule, kept) in order {
        // A control document's rule whose subject is its host: kept apart
        // and applied to the host element where the document is
        // instantiated, under every rule written for the host there.
        if doc.control.is_some() && kept.is_none() {
            let condition = parsed
                .condition(doc, elements, super::HOST_NODE)
                .map_err(|e| (rule.at, e))?;
            if condition != Expr::Lit(false.into()) {
                if rule.pseudo.is_some() {
                    return Err((
                        rule.at,
                        format!(
                            "{}: a pseudo-element of the host is the host document's to style; give the control an element with a part",
                            rule.selector
                        ),
                    ));
                }
                let (declarations, _) = lower(&rule.body, &known).map_err(|e| (rule.at, e))?;
                doc.host_rules.push(StyleRule {
                    inline: false,
                    context: super::TreeContext::Tree,
                    depth: -1,
                    layer: rank(&layers, &rule.layer),
                    starting: rule.starting,
                    query: rule.query.clone(),
                    condition: (condition != Expr::Lit(true.into())).then_some(condition),
                    structure: None,
                    props: Vec::new(),
                    instance_states: Vec::new(),
                    declarations,
                    line: syntax::location(&doc.text, rule.at).0,
                });
                continue;
            }
        }
        // A `:host` selector matched on the instantiated tree is matched
        // there as the element it selects; what it asks of the host
        // beside that is the rule's condition.
        let hosted = match parsed.structural() {
            true => parsed.without_host(doc).map_err(|e| (rule.at, e))?,
            false => None,
        };
        let hosted = hosted.map(|(selector, condition)| (std::sync::Arc::new(selector), condition));
        let (parsed, host) = match &hosted {
            Some((selector, condition)) => (selector, Some(condition)),
            None => (parsed, None),
        };
        let structure = parsed.structural().then(|| parsed.clone());
        // `::slotted()`: the rule is this document's and its subject is
        // content another document passes it, so it is kept apart and
        // matched where that content is rendered. It ranks under the
        // content's own document's rules.
        if parsed.slotted() {
            let refused = if rule.pseudo.is_some() {
                Some("a pseudo-element after ::slotted() is not read")
            } else if host.is_some() {
                Some(
                    ":host() before ::slotted() is not read; give the element a class from the prop",
                )
            } else {
                None
            };
            if let Some(why) = refused {
                return Err((rule.at, format!("{}: {why}", rule.selector)));
            }
            let (declarations, _) = lower(&rule.body, &known).map_err(|e| (rule.at, e))?;
            doc.slotted_rules.push(StyleRule {
                inline: false,
                context: super::TreeContext::Tree,
                depth: -1,
                layer: rank(&layers, &rule.layer),
                starting: rule.starting,
                query: rule.query.clone(),
                condition: None,
                structure: Some(parsed.clone()),
                instance_states: Vec::new(),
                props: Vec::new(),
                declarations,
                line: syntax::location(&doc.text, rule.at).0,
            });
            continue;
        }
        if let Some((parent, selector)) = rule.selector.split_once("::part(") {
            let parent = if parent.is_empty() { "text" } else { parent };
            let host = Selector::parse_list(parent)
                .map_err(|e| (rule.at, e))?
                .swap_remove(0);
            let compounds = parsed.compounds().0;
            let part_name = compounds.iter().flatten().find_map(|c| match c {
                super::selectors::Component::Part(names) => names.first().map(|n| n.0.as_str()),
                _ => None,
            });
            let mut text_span = false;
            for (i, _, _) in elements {
                // A text element's spans: `text::part(name)`, `p::part(…)`.
                if doc.nodes[*i].kind != NodeKind::Element(Kind::Label)
                    || doc.nodes[*i].selector_tag.as_deref().is_some_and(|tag| {
                        part_name.is_some_and(|name| schema::parts::part(tag, name).is_some())
                    })
                {
                    continue;
                }
                if structure.is_some() {
                    if host
                        .subject_filter()
                        .condition(doc, elements, *i)
                        .map_err(|e| (rule.at, e))?
                        == Expr::Lit(false.into())
                    {
                        continue;
                    }
                    return Err((
                        rule.at,
                        format!(
                            "{}: sibling combinators and structural pseudo-classes cannot select ::part spans; select the text element itself",
                            parsed.source
                        ),
                    ));
                }
                let Some(condition) = matches(doc, elements, *i, parent, rule.at)? else {
                    continue;
                };
                text_span = true;
                let mut span =
                    super::spans::parse(selector, &rule.body).map_err(|e| (rule.at, e))?;
                span.query = rule.query.clone();
                span.condition = Some(condition);
                doc.nodes[*i].span_rules.push(span);
            }
            // A global part rule is the style's (`Style::part_rules`), not
            // the document's own: a button's plate is above it.
            if !text_span && kept.is_none() {
                part_selector(parsed).map_err(|e| (rule.at, e))?;
                let (declarations, _) = lower(&rule.body, &known).map_err(|e| (rule.at, e))?;
                doc.part_rules.push(StyleRule {
                    inline: false,
                    context: super::TreeContext::Outer,
                    depth: 0,
                    layer: rank(&layers, &rule.layer),
                    starting: rule.starting,
                    query: rule.query.clone(),
                    condition: None,
                    structure: Some(parsed.clone()),
                    instance_states: Vec::new(),
                    props: Vec::new(),
                    declarations,
                    line: syntax::location(&doc.text, rule.at).0,
                });
            }
            // What the selector reads of the host's tree is matched on the
            // host, which declares the selector's marker while it holds.
            if !text_span && parsed.local.is_some() {
                let (declarations, _) = lower(&format!("{}: 1", parsed.host_marker()), &known)
                    .map_err(|e| (rule.at, e))?;
                let host_structure = (parsed.global || host.structural()).then(|| {
                    std::sync::Arc::new(Selector {
                        global: parsed.global,
                        ..host.clone()
                    })
                });
                let filter = host_structure.as_ref().map(|_| host.subject_filter());
                for (i, _, _) in elements {
                    if !matches!(doc.nodes[*i].kind, NodeKind::Element(_)) {
                        continue;
                    }
                    let condition = match &filter {
                        Some(filter) => match filter
                            .condition(doc, elements, *i)
                            .map_err(|e| (rule.at, e))?
                        {
                            Expr::Lit(serde_json::Value::Bool(false)) => continue,
                            _ => None,
                        },
                        None => match matches(doc, elements, *i, parent, rule.at)? {
                            Some(Expr::Lit(serde_json::Value::Bool(true))) => None,
                            Some(condition) => Some(condition),
                            None => continue,
                        },
                    };
                    doc.nodes[*i].rules.push(StyleRule {
                        inline: false,
                        context: super::TreeContext::Tree,
                        depth: 0,
                        layer: rank(&layers, &rule.layer),
                        starting: false,
                        query: rule.query.clone(),
                        condition,
                        structure: host_structure.clone(),
                        instance_states: Vec::new(),
                        props: Vec::new(),
                        declarations: declarations.clone(),
                        line: syntax::location(&doc.text, rule.at).0,
                    });
                }
            }
            continue;
        }
        let structure = if parsed.global {
            Some(parsed.clone())
        } else {
            structure
        };
        let line = syntax::location(&doc.text, rule.at).0;
        let fresh;
        let kept = kept
            .filter(|g| !(more_keyframes && g.animates))
            .and_then(|g| g.lowered.as_ref());
        let (own, boxed) = match kept {
            Some(kept) => (&kept.0, kept.1.as_ref()),
            None => {
                let own = lowered(&rule.body, line, &known).map_err(|e| (rule.at, e))?;
                let boxed = match rule.pseudo {
                    Some(_) if !rule.boxed.is_empty() => {
                        Some(lowered(&rule.boxed, line, &known).map_err(|e| (rule.at, e))?)
                    }
                    _ => None,
                };
                fresh = (own, boxed);
                (&fresh.0, fresh.1.as_ref())
            }
        };
        let subject_filter = structure.as_ref().map(|_| parsed.subject_filter());
        for (i, _, _) in elements {
            let condition = match &subject_filter {
                // The subject's static facts prune elements at cook; the whole
                // selector is matched on the instantiated tree.
                Some(filter) => match filter
                    .condition(doc, elements, *i)
                    .map_err(|e| (rule.at, e))?
                {
                    Expr::Lit(serde_json::Value::Bool(false)) => continue,
                    _ => host.cloned().unwrap_or(Expr::Lit(true.into())),
                },
                None => match matches(doc, elements, *i, &rule.selector, rule.at)? {
                    Some(condition) => condition,
                    None => continue,
                },
            };
            let condition = match condition {
                Expr::Lit(serde_json::Value::Bool(true)) => None,
                c => Some(c),
            };
            // A component is styled through what it declares. A rule that
            // names an instance is refused; one that only sweeps over it
            // (`*`, a pack's global rule) passes it by.
            if let NodeKind::Use(component) = &doc.nodes[*i].kind {
                let hosts = |c: &&super::selectors::Component| {
                    matches!(c, super::selectors::Component::Host(_))
                };
                if parsed
                    .compounds()
                    .0
                    .last()
                    .is_some_and(|s| s.iter().any(hosts))
                {
                    return Err((
                        rule.at,
                        format!(
                            "{} styles the component's root element, and this component's root is a <{component}> instance: wrap it in an element, or pass <{component}> a prop",
                            parsed.source
                        ),
                    ));
                }
                let names = parsed.compounds().0.last().is_some_and(|subject| {
                    subject.iter().any(|c| {
                        matches!(
                            c,
                            super::selectors::Component::LocalName(_)
                                | super::selectors::Component::ID(_)
                                | super::selectors::Component::Class(_)
                        ) || super::selectors::attribute(c).is_some()
                    })
                });
                if names && !parsed.global && !rule.ua {
                    return Err((
                        rule.at,
                        format!(
                            "{} selects a <{component}> instance, and a document's rules stop at a component: pass a prop, set a custom property on the instance (<{component} --name=\"value\"/>), or dress a part it declares from the style pack ([part=\"{component}.…\"])",
                            parsed.source
                        ),
                    ));
                }
                continue;
            }
            let kind = match doc.nodes[*i].kind {
                NodeKind::Element(k) => k,
                _ => Kind::Column,
            };
            // The pseudo-element's box, when its element draws one: its
            // node takes the box declarations under the element's condition.
            let stateful = !rule.pseudo_states.is_empty();
            let target = match (rule.pseudo, boxed) {
                (Some(pseudo), boxed)
                    if (boxed.is_some() || stateful) && holds(pseudo, kind, &doc.nodes[*i]) =>
                {
                    Some(pseudo_node(doc, *i, pseudo))
                }
                _ => None,
            };
            // The pseudo-element's own states are read on its node: without
            // one the rule has nothing to match. One with an instance per
            // fragment matches them on each instance ([`INSTANCED`]).
            let instanced = rule.pseudo.is_some_and(|p| INSTANCED.contains(&p));
            let instance = if instanced {
                rule.pseudo_states.clone()
            } else {
                Vec::new()
            };
            let condition = match target {
                Some(_) if instanced => condition,
                Some(node) if stateful => {
                    let own = rule.pseudo_states.iter().fold(
                        condition.unwrap_or(Expr::Lit(true.into())),
                        |all, &state| {
                            super::selectors::combine(
                                crate::bind::Op::And,
                                all,
                                Expr::InteractionAt { node, state },
                            )
                        },
                    );
                    Some(own)
                }
                None if stateful => continue,
                _ => condition,
            };
            let mut parts = vec![(*i, kind, own)];
            if let (Some(node), Some(boxed)) = (target, boxed) {
                parts.push((node, Kind::Box, boxed));
            }
            for (node, kind, (declarations, entries)) in parts {
                if rule.pseudo.is_some() && declarations.is_empty() {
                    continue;
                }
                let props = rule_props(entries, kind).map_err(|e| (rule.at, e))?;
                doc.nodes[node].rules.push(StyleRule {
                    inline: false,
                    context: super::TreeContext::Tree,
                    depth: 0,
                    layer: if rule.ua {
                        0
                    } else {
                        rank(&layers, &rule.layer)
                    },
                    starting: rule.starting,
                    query: rule.query.clone(),
                    condition: condition.clone(),
                    structure: structure.clone(),
                    instance_states: instance.clone(),
                    props,
                    declarations: declarations.clone(),
                    line,
                });
            }
        }
    }
    Ok(())
}

/// One top-level item of a stylesheet or rule body.
enum Item<'a> {
    /// `prelude { block }` at `at`, its block at `block_at`.
    Block {
        prelude: &'a str,
        block: &'a str,
        at: usize,
        block_at: usize,
    },
    /// `text;`: a declaration in a body, a statement at-rule elsewhere.
    Statement(&'a str, usize),
}

/// Splits `text` into its top-level blocks and statements. A custom
/// property whose value holds a `{}` block stays one declaration.
fn items(text: &str, base: usize) -> Result<Vec<Item<'_>>, Error> {
    let mut out = Vec::new();
    let mut at = 0;
    let bytes = text.as_bytes();
    while at < text.len() {
        while at < text.len() && bytes[at].is_ascii_whitespace() {
            at += 1;
        }
        if text[at..].starts_with("/*") {
            let end = text[at + 2..]
                .find("*/")
                .ok_or((base + at, "close CSS comment with */".into()))?;
            at += end + 4;
            continue;
        }
        if at >= text.len() {
            break;
        }
        let start = at;
        let custom = text[at..].trim_start().starts_with("--");
        let (mut depth, mut quote, mut open) = (0i32, None, None);
        let mut end = text.len();
        let mut i = at;
        while i < text.len() {
            let c = text[i..].chars().next().unwrap();
            let next = i + c.len_utf8();
            if let Some(q) = quote {
                if c == '\\' {
                    i = next + text[next..].chars().next().map_or(0, char::len_utf8);
                    continue;
                }
                if c == q {
                    quote = None;
                }
            } else if text[i..].starts_with("/*") {
                i = text[i + 2..].find("*/").map_or(text.len(), |e| i + e + 4);
                continue;
            } else {
                match c {
                    '\'' | '"' => quote = Some(c),
                    '(' | '[' => depth += 1,
                    ')' | ']' => depth -= 1,
                    '{' if depth == 0 && !custom => {
                        open = Some(i);
                        break;
                    }
                    '{' => depth += 1,
                    '}' if depth > 0 => depth -= 1,
                    '}' => return Err((base + i, "unexpected '}' in CSS".into())),
                    ';' if depth == 0 => {
                        end = i;
                        break;
                    }
                    _ => {}
                }
            }
            i = next;
        }
        match open {
            Some(open) => {
                let (mut depth, mut quote) = (1, None);
                let mut j = open + 1;
                while j < text.len() && depth > 0 {
                    // `c` starts at `at`: a comment opens there, not inside
                    // a character of more than one byte.
                    let at = j;
                    let c = text[at..].chars().next().unwrap();
                    j += c.len_utf8();
                    if let Some(q) = quote {
                        if c == '\\' && j < text.len() {
                            j += text[j..].chars().next().unwrap().len_utf8();
                        } else if c == q {
                            quote = None;
                        }
                    } else if text[at..].starts_with("/*") {
                        j = text[j + 1..]
                            .find("*/")
                            .map_or(text.len(), |e| j + 1 + e + 2);
                    } else {
                        match c {
                            '\'' | '"' => quote = Some(c),
                            '{' => depth += 1,
                            '}' => depth -= 1,
                            _ => {}
                        }
                    }
                }
                if depth != 0 {
                    return Err((base + open, "close CSS rule with '}'".into()));
                }
                out.push(Item::Block {
                    prelude: text[start..open].trim(),
                    block: &text[open + 1..j - 1],
                    at: base + start,
                    block_at: base + open + 1,
                });
                at = j;
            }
            None => {
                let statement = text[start..end].trim();
                if !statement.is_empty() {
                    out.push(Item::Statement(statement, base + start));
                }
                at = end + 1;
            }
        }
    }
    Ok(out)
}

/// The at-rule an item's prelude names, lowercased, and the rest.
fn at_rule(prelude: &str) -> Option<(String, &str)> {
    let rest = prelude.strip_prefix('@')?;
    let end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .unwrap_or(rest.len());
    Some((rest[..end].to_ascii_lowercase(), rest[end..].trim()))
}

fn layer_name(name: &str) -> bool {
    name.split('.').all(|part| {
        matches!(
            super::css::tokens(part).as_deref(),
            Ok([super::css::Tok::Ident(_)])
        )
    })
}

impl<'a> Reader<'a> {
    /// A reader of the stylesheets written beside `source`, into `sheet`.
    fn new(sheet: &'a mut super::at_rules::Sheet, source: &str) -> Self {
        Reader {
            sheet,
            layers: ["og-default", "og-components", "og-pack", "og-game"]
                .map(str::to_owned)
                .to_vec(),
            anonymous: 0,
            base: source.to_owned(),
            imports_open: true,
            depth: 0,
        }
    }

    /// HTML's user-agent rules, before any other.
    fn user_agent(&mut self, rules: &mut Vec<Rule>) -> Result<(), Error> {
        self.rules(super::lists::UA_CSS, 0, &Context::default(), rules)?;
        for rule in rules {
            // Its offsets are in the UA text, not the document's.
            rule.ua = true;
            rule.at = 0;
            rule.body_at = 0;
        }
        self.imports_open = true;
        Ok(())
    }

    /// A layer's full name inside `context`, registered on first sight.
    fn layer(&mut self, context: &Context, name: Option<&str>) -> String {
        let name = name.map_or_else(
            || {
                self.anonymous += 1;
                format!("\u{0}{}", self.anonymous)
            },
            str::to_string,
        );
        let full = match &context.layer {
            Some(outer) => format!("{outer}.{name}"),
            None => name,
        };
        // Register every prefix: `a.b` declares `a`.
        let mut prefix = String::new();
        for part in full.split('.') {
            if !prefix.is_empty() {
                prefix.push('.');
            }
            prefix.push_str(part);
            if !self.layers.contains(&prefix) {
                self.layers.push(prefix.clone());
            }
        }
        full
    }

    /// Reads a stylesheet (or a group rule's contents) of style rules.
    fn rules(
        &mut self,
        text: &str,
        base: usize,
        context: &Context,
        out: &mut Vec<Rule>,
    ) -> Result<(), Error> {
        let items = items(text, base)?;
        // `@custom-media` defines its name for the whole sheet, before or
        // after a query that names it.
        for item in &items {
            if let Item::Statement(statement, at) = item
                && let Some((name, rest)) = at_rule(statement)
                && name == "custom-media"
            {
                let defined = super::media::Query::parse_custom(rest).map_err(|e| (*at, e))?;
                self.sheet.custom_media.push(defined);
            }
        }
        for item in items {
            match item {
                Item::Statement(statement, at) => {
                    self.statement(statement, at, context, out)?;
                }
                Item::Block {
                    prelude,
                    block,
                    at,
                    block_at,
                } => {
                    let (assets, remote) = super::at_rules::sheet_urls(&self.base, block);
                    self.sheet.assets.extend(assets);
                    self.sheet.remote.extend(remote);
                    if prelude.starts_with('@') {
                        self.group(prelude, block, at, block_at, context, out, false)?;
                    } else {
                        self.imports_open = false;
                        self.style_rule(prelude, block, at, block_at, context, out)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// `@layer a, b;` and `@import …;`.
    fn statement(
        &mut self,
        statement: &str,
        at: usize,
        context: &Context,
        out: &mut Vec<Rule>,
    ) -> Result<(), Error> {
        let Some((name, rest)) = at_rule(statement) else {
            return Err((
                at,
                format!("a CSS rule needs {{ declarations }}: `{statement}`"),
            ));
        };
        match name.as_str() {
            "layer" => {
                for layer in rest.split(',') {
                    let layer = layer.trim();
                    if !layer_name(layer) {
                        return Err((at, format!("@layer {layer} is not a layer name")));
                    }
                    self.layer(context, Some(layer));
                }
                Ok(())
            }
            "import" => self.import(rest, at, context, out),
            "charset" | "namespace" | "custom-media" => Ok(()),
            other => Err((at, format!("@{other} needs a {{ block }}"))),
        }
    }

    /// `@import url("x.css") [layer | layer(name)] [supports(…)] [media];`
    fn import(
        &mut self,
        rest: &str,
        at: usize,
        context: &Context,
        out: &mut Vec<Rule>,
    ) -> Result<(), Error> {
        if !self.imports_open {
            return Err((at, "write @import before the stylesheet's rules".into()));
        }
        let mut p = cssparser::Parser::new(rest);
        let path = match p.next() {
            Ok(cssparser::Token::QuotedString(s)) | Ok(cssparser::Token::UnquotedUrl(s)) => {
                s.to_string()
            }
            Ok(cssparser::Token::Function(f)) if f.eq_ignore_ascii_case("url") => p
                .parse_nested_block(|p| {
                    Ok::<_, cssparser::ParseError<()>>(p.expect_string()?.to_string())
                })
                .map_err(|_| (at, "@import url() takes a quoted path".into()))?,
            _ => return Err((at, "@import takes a path: @import \"theme.css\";".into())),
        };
        let mut inner = context.clone();
        let mut supported = true;
        loop {
            let state = p.state();
            match p.next() {
                Ok(cssparser::Token::Ident(w)) if w.eq_ignore_ascii_case("layer") => {
                    let layer = self.layer(context, None);
                    inner.layer = Some(layer);
                }
                Ok(cssparser::Token::Function(f)) if f.eq_ignore_ascii_case("layer") => {
                    let name = p
                        .parse_nested_block(|p| {
                            let start = p.position();
                            while p.next().is_ok() {}
                            Ok::<_, cssparser::ParseError<()>>(
                                p.slice_from(start).trim().to_string(),
                            )
                        })
                        .unwrap_or_default();
                    inner.layer = Some(self.layer(context, Some(&name)));
                }
                Ok(cssparser::Token::Function(f)) if f.eq_ignore_ascii_case("supports") => {
                    let condition = p
                        .parse_nested_block(|p| {
                            let start = p.position();
                            while p.next().is_ok() {}
                            Ok::<_, cssparser::ParseError<()>>(p.slice_from(start).to_string())
                        })
                        .unwrap_or_default();
                    // `supports(display: grid)` is a bare declaration.
                    let condition = if supports(&condition).is_ok() {
                        condition
                    } else {
                        format!("({condition})")
                    };
                    supported = supports(&condition).map_err(|e| (at, e))?;
                }
                _ => {
                    p.reset(&state);
                    break;
                }
            }
        }
        let start = p.position();
        while p.next().is_ok() {}
        let media = p.slice_from(start).trim();
        if !media.is_empty() {
            let parsed = super::media::Query::parse(media)
                .and_then(|q| q.resolve(&self.sheet.custom_media))
                .map_err(|e| (at, e))?;
            inner.query.media = inner.query.media.clone().and(parsed);
        }
        let resolved = super::at_rules::resolve(&self.base, &path);
        let Some((_, text)) = self
            .sheet
            .imports
            .iter()
            .find(|(p, _)| *p == resolved)
            .or_else(|| self.sheet.imports.iter().find(|(p, _)| *p == path))
            .cloned()
        else {
            return Err((
                at,
                format!(
                    "@import \"{path}\": the cook reads it from beside the document ({}); it was not supplied",
                    super::at_rules::resolve(&self.base, &path)
                ),
            ));
        };
        if !supported {
            return Ok(());
        }
        self.depth += 1;
        if self.depth > 16 {
            return Err((at, format!("@import \"{path}\" imports itself")));
        }
        // An imported sheet's offsets are its own; errors name the @import.
        let mut rules = Vec::new();
        let open = self.imports_open;
        self.imports_open = true;
        let base = std::mem::replace(&mut self.base, resolved);
        self.rules(&text, 0, &inner, &mut rules)
            .map_err(|(_, e)| (at, format!("in @import \"{path}\": {e}")))?;
        self.base = base;
        self.imports_open = open;
        self.depth -= 1;
        for mut rule in rules {
            rule.at = at;
            rule.body_at = at;
            rule.order = out.len();
            out.push(rule);
        }
        Ok(())
    }

    /// A block at-rule. `nested`: inside a style rule, where its contents
    /// are a body for the enclosing selector.
    #[allow(clippy::too_many_arguments)]
    fn group(
        &mut self,
        prelude: &str,
        block: &str,
        at: usize,
        block_at: usize,
        context: &Context,
        out: &mut Vec<Rule>,
        nested: bool,
    ) -> Result<(), Error> {
        let (name, rest) = at_rule(prelude).expect("an at-rule");
        let mut inner = context.clone();
        let descend = |reader: &mut Self, inner: &Context, out: &mut Vec<Rule>| {
            if nested {
                reader.body(block, at, block_at, inner, out)
            } else {
                reader.rules(block, block_at, inner, out)
            }
        };
        if !matches!(name.as_str(), "layer" | "import") {
            self.imports_open = false;
        }
        match name.as_str() {
            "media" => {
                let parsed = super::media::Query::parse(rest)
                    .and_then(|q| q.resolve(&self.sheet.custom_media))
                    .map_err(|error| (at, error))?;
                inner.query.media = context.query.media.clone().and(parsed);
                descend(self, &inner, out)
            }
            "container" => {
                let condition =
                    super::container::Condition::parse(rest).map_err(|error| (at, error))?;
                inner.query.container.push(condition);
                descend(self, &inner, out)
            }
            "supports" => {
                if supports(rest).map_err(|e| (at, e))? {
                    descend(self, &inner, out)
                } else {
                    Ok(())
                }
            }
            "layer" => {
                let name = (!rest.is_empty()).then_some(rest);
                if let Some(name) = name
                    && !layer_name(name)
                {
                    return Err((at, format!("@layer {name} {{ … }} takes one layer name")));
                }
                inner.layer = Some(self.layer(context, name));
                descend(self, &inner, out)
            }
            "starting-style" => {
                inner.starting = true;
                descend(self, &inner, out)
            }
            "scope" => {
                let (root, limit) = scope_prelude(rest).map_err(|e| (at, e))?;
                let root = match (root, &context.parent) {
                    (Some(root), _) => resolve_nested(&root, context.parent.as_deref()),
                    (None, Some(parent)) => parent.clone(),
                    (None, None) => ":root".into(),
                };
                inner.scope = Some((root, limit));
                inner.parent = None;
                if nested {
                    // Its contents are rules relative to the new root.
                    self.rules(block, block_at, &inner, out)
                } else {
                    self.rules(block, block_at, &inner, out)
                }
            }
            "font-face" if !nested => self.sheet.font_face(block, &self.base).map_err(|e| (at, e)),
            "property" if !nested => self.sheet.property(rest, block).map_err(|e| (at, e)),
            "counter-style" if !nested => {
                self.sheet.counter_style(rest, block).map_err(|e| (at, e))
            }
            "position-try" if !nested => self.sheet.position_try(rest, block).map_err(|e| (at, e)),
            "font-feature-values" if !nested => self
                .sheet
                .font_feature_values(rest, block)
                .map_err(|e| (at, e)),
            "font-palette-values" if !nested => self
                .sheet
                .font_palette_values(rest, block)
                .map_err(|e| (at, e)),
            "function" if !nested => {
                let function = function_rule(rest, block, block_at, &self.sheet.custom_media)
                    .map_err(|(i, e)| (if i == 0 { at } else { i }, e))?;
                self.sheet.functions.retain(|(n, _)| *n != function.0);
                self.sheet.functions.push(function);
                Ok(())
            }
            "keyframes" | "view-transition" if !nested => {
                out.push(Rule {
                    selector: prelude.to_string(),
                    body: super::at_rules::rebase_urls(&self.base, block),
                    query: context.query.clone(),
                    at,
                    body_at: block_at,
                    order: out.len(),
                    ua: false,
                    global: false,
                    layer: context.layer.clone(),
                    starting: false,
                    pseudo: None,
                    boxed: String::new(),
                    pseudo_states: Vec::new(),
                });
                Ok(())
            }
            "page" => Err((
                at,
                "@page is print media, which documents do not have".into(),
            )),
            other => Err((
                at,
                format!(
                    "unsupported at-rule @{other}{}",
                    if nested { " inside a style rule" } else { "" }
                ),
            )),
        }
    }

    /// A style rule: its selector resolved against the nesting parent and
    /// the scope, then its body.
    fn style_rule(
        &mut self,
        prelude: &str,
        block: &str,
        at: usize,
        block_at: usize,
        context: &Context,
        out: &mut Vec<Rule>,
    ) -> Result<(), Error> {
        let mut selector = resolve_nested(prelude, context.parent.as_deref());
        if context.parent.is_none()
            && let Some((root, limit)) = &context.scope
        {
            selector = scoped(&selector, root, limit.as_deref());
        }
        let mut inner = context.clone();
        inner.parent = Some(selector);
        inner.scope = None;
        self.body(block, at, block_at, &inner, out)
    }

    /// A style rule's body: its declarations, as one rule for the parent
    /// selector, and its nested rules after it.
    fn body(
        &mut self,
        text: &str,
        at: usize,
        body_at: usize,
        context: &Context,
        out: &mut Vec<Rule>,
    ) -> Result<(), Error> {
        let selector = context
            .parent
            .clone()
            .expect("a body has a parent selector");
        let mut declarations = String::new();
        let mut nested = Vec::new();
        for item in items(text, body_at)? {
            match item {
                Item::Statement(d, _) => {
                    declarations.push_str(&super::at_rules::rebase_urls(&self.base, d));
                    declarations.push(';');
                }
                block => nested.push(block),
            }
        }
        if !declarations.trim().is_empty() || nested.is_empty() {
            out.push(Rule {
                selector: selector.clone(),
                body: declarations,
                query: context.query.clone(),
                at,
                body_at,
                order: out.len(),
                ua: false,
                global: false,
                layer: context.layer.clone(),
                starting: context.starting,
                pseudo: None,
                boxed: String::new(),
                pseudo_states: Vec::new(),
            });
        }
        for item in nested {
            let Item::Block {
                prelude,
                block,
                at,
                block_at,
            } = item
            else {
                unreachable!()
            };
            if prelude.starts_with('@') {
                self.group(prelude, block, at, block_at, context, out, true)?;
            } else {
                self.style_rule(prelude, block, at, block_at, context, out)?;
            }
        }
        Ok(())
    }
}

/// Splits a selector list at its top-level commas.
fn split_list(list: &str) -> Vec<&str> {
    let (mut depth, mut quote, mut start) = (0, None, 0);
    let mut out = Vec::new();
    for (i, c) in list.char_indices() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '\'' | '"') => quote = Some(c),
            (None, '(' | '[') => depth += 1,
            (None, ')' | ']') => depth -= 1,
            (None, ',') if depth == 0 => {
                out.push(list[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(list[start..].trim());
    out
}

/// A nested rule's selector list in terms of its parent's (CSS Nesting):
/// `&` is the parent; a selector without one is relative to it. A parent
/// of one compound is written in place (`&:hover` → `.a:hover`), any other
/// as `:is(parent)`, whose specificity is the list's highest.
fn resolve_nested(list: &str, parent: Option<&str>) -> String {
    let Some(parent) = parent else {
        return list.to_string();
    };
    let simple = !parent.contains(',')
        && Selector::parse_list(parent)
            .is_ok_and(|s| s.len() == 1 && s[0].compounds().0.len() == 1);
    let replacement = if simple {
        parent.to_string()
    } else {
        format!(":is({parent})")
    };
    split_list(list)
        .into_iter()
        .map(|s| {
            if s.contains('&') {
                s.replace('&', &replacement)
            } else {
                format!("{replacement} {s}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// A selector inside `@scope (root) to (limit)`: matched in the root's
/// subtree (or at the root through `:scope`), above any limit.
fn scoped(list: &str, root: &str, limit: Option<&str>) -> String {
    let root = format!(":where({root})");
    split_list(list)
        .into_iter()
        .map(|s| {
            let s = if s.contains(":scope") || s.contains('&') {
                s.replace(":scope", &root).replace('&', &root)
            } else {
                format!("{root} {s}")
            };
            match limit {
                Some(limit) => format!("{s}:not(:where({limit}) *, :where({limit}))"),
                None => s,
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// `(root) to (limit)`, either part optional.
fn scope_prelude(rest: &str) -> Result<(Option<String>, Option<String>), String> {
    let mut p = cssparser::Parser::new(rest);
    let group = |p: &mut cssparser::Parser<'_>| -> Result<String, String> {
        p.expect_parenthesis_block()
            .map_err(|_| "@scope takes (root) to (limit)".to_string())?;
        p.parse_nested_block(|p| {
            let start = p.position();
            while p.next().is_ok() {}
            Ok::<_, cssparser::ParseError<()>>(p.slice_from(start).trim().to_string())
        })
        .map_err(|_| "@scope takes (root) to (limit)".to_string())
    };
    let root = if p.is_exhausted() || p.try_parse(|p| p.expect_ident_matching("to")).is_ok() {
        None
    } else {
        Some(group(&mut p)?)
    };
    let limit = if p.is_exhausted() {
        None
    } else {
        if root.is_some() {
            p.expect_ident_matching("to")
                .map_err(|_| "@scope takes (root) to (limit)".to_string())?;
        }
        Some(group(&mut p)?)
    };
    for s in root.iter().chain(&limit) {
        Selector::parse_list(s)?;
    }
    Ok((root, limit))
}

/// An `@supports` condition (CSS Conditional 3 to 5): `(property: value)`
/// holds when the document's cascade takes the declaration as written,
/// `selector(…)` when it takes the selector, `at-rule(…)`,
/// `named-feature(…)` and `env(…)` when the engine has them; `not`, `and`,
/// `or` combine.
pub(super) fn supports(condition: &str) -> Result<bool, String> {
    use cssparser::{ParseError, Parser, Token};
    fn test(p: &mut Parser<'_>) -> Result<bool, ParseError<String>> {
        if p.try_parse(|p| p.expect_ident_matching("not")).is_ok() {
            return Ok(!test_group(p)?);
        }
        let first = test_group(p)?;
        let mut result = first;
        let mut operator: Option<String> = None;
        while !p.is_exhausted() {
            let word = p.expect_ident_cloned()?.to_ascii_lowercase();
            if word != "and" && word != "or" {
                return Err(ParseError::custom(format!(
                    "expected and or or, found {word}"
                )));
            }
            if operator.as_ref().is_some_and(|o| *o != word) {
                return Err(ParseError::custom(
                    "group mixed and/or in parentheses".to_string(),
                ));
            }
            let next = test_group(p)?;
            result = if word == "and" {
                result && next
            } else {
                result || next
            };
            operator = Some(word);
        }
        Ok(result)
    }
    fn test_group(p: &mut Parser<'_>) -> Result<bool, ParseError<String>> {
        let token = p.next()?.clone();
        match token {
            Token::ParenthesisBlock => p.parse_nested_block(|p| {
                let state = p.state();
                if matches!(p.next(), Ok(Token::ParenthesisBlock)) || {
                    p.reset(&state);
                    p.try_parse(|p| p.expect_ident_matching("not")).is_ok()
                } {
                    p.reset(&state);
                    return test(p);
                }
                p.reset(&state);
                let Ok(name) = p.expect_ident_cloned() else {
                    while p.next().is_ok() {}
                    return Ok(false);
                };
                if p.expect_colon().is_err() {
                    while p.next().is_ok() {}
                    return Ok(false);
                }
                let start = p.position();
                while p.next().is_ok() {}
                let value = p.slice_from(start).trim().to_string();
                Ok(declaration_supported(&name, &value))
            }),
            Token::Function(name) => {
                let name = name.to_ascii_lowercase();
                p.parse_nested_block(|p| {
                    let start = p.position();
                    while p.next().is_ok() {}
                    let arg = p.slice_from(start).trim().to_string();
                    Ok(match name.as_str() {
                        "selector" => Selector::parse_list(&arg).is_ok(),
                        "font-format" => matches!(
                            arg.trim_matches(['"', '\'']),
                            "truetype" | "opentype" | "woff" | "woff2" | "collection"
                        ),
                        "font-tech" => matches!(
                            arg.as_str(),
                            "color-colrv0" | "color-colrv1" | "variations" | "features-opentype"
                        ),
                        // An at-rule the reader takes in some context
                        // (CSS Conditional 5 §2.1.2; @charset is no
                        // at-rule).
                        "at-rule" => matches!(
                            arg.to_ascii_lowercase().as_str(),
                            "@media"
                                | "@container"
                                | "@supports"
                                | "@layer"
                                | "@starting-style"
                                | "@scope"
                                | "@font-face"
                                | "@property"
                                | "@counter-style"
                                | "@position-try"
                                | "@keyframes"
                                | "@view-transition"
                                | "@import"
                                | "@namespace"
                                | "@custom-media"
                        ),
                        // §2.1.3's list: colours out of the output's gamut
                        // are CSS Color 4 §13.2 mapped. Anchors do not
                        // follow their transforms, and one scrollable axis
                        // beside a clipped one is not established.
                        "named-feature" => arg == "gamut-mapping",
                        "env" => {
                            let edge = |prefix: &str| {
                                arg.strip_prefix(prefix).is_some_and(|edge| {
                                    matches!(edge, "top" | "right" | "bottom" | "left")
                                })
                            };
                            edge("safe-area-inset-")
                                || edge("safe-area-max-inset-")
                                || edge("-og-hud-inset-")
                        }
                        _ => false,
                    })
                })
            }
            _ => Err(ParseError::custom(
                "@supports takes (property: value), selector(…), font-format(…), font-tech(…), at-rule(…), named-feature(…), env(…), not, and and or".to_string(),
            )),
        }
    }
    Parser::new(condition)
        .parse_entirely(test)
        .map_err(|e| format!("invalid @supports condition {condition:?}: {:?}", e.kind))
}

/// What an `if()` test reads where its element resolves (CSS Values 5
/// §8.3): `media()` the media context, `style()` the element's custom
/// properties (`None`: guaranteed-invalid). `supports()` asks
/// [`supports`].
pub(crate) struct IfTests<'a> {
    pub media: &'a mut dyn FnMut(&super::media::Query) -> bool,
    pub style: &'a mut dyn FnMut(&str) -> Option<String>,
}

/// `text` with each `if(<condition>: <value>; …)` replaced by the value of
/// its first branch whose condition holds (`else` always does), its own
/// `if()`s replaced in turn, or by nothing when none holds (CSS Values 5
/// §8.3). A branch without a `:` or a test that does not parse is an
/// error.
pub(crate) fn replace_ifs(text: &str, tests: &mut IfTests<'_>) -> Result<String, String> {
    ifs(text, tests, false)
}

/// Checks every `if()` in `text` as written: each branch's condition and
/// value, with no test holding.
pub(crate) fn check_ifs(text: &str) -> Result<(), String> {
    let tests = &mut IfTests {
        media: &mut |_| false,
        style: &mut |_| None,
    };
    ifs(text, tests, true).map(drop)
}

fn ifs(text: &str, tests: &mut IfTests<'_>, every: bool) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    loop {
        let at = rest.match_indices("if(").map(|(at, _)| at).find(|&at| {
            at == 0 || {
                let p = rest.as_bytes()[at - 1];
                !(p.is_ascii_alphanumeric() || p == b'-' || p == b'_')
            }
        });
        let Some(at) = at else { break };
        let open = at + 2;
        let close = top_level(&rest[open + 1..], ')')
            .first()
            .map(|inside| open + 1 + inside.len())
            .filter(|&c| rest.as_bytes().get(c) == Some(&b')'))
            .ok_or("if( is not closed")?;
        out.push_str(&rest[..at]);
        let mut chosen = None;
        for branch in top_level(&rest[open + 1..close], ';') {
            // A branch spread from a variable (`...var(--clause)`, CSS
            // Values 5 Appendix A) is known once it substitutes, which it
            // has where the element resolves.
            if branch.trim().is_empty() || (every && branch.contains("var(")) {
                continue;
            }
            let branch = branch.trim_start();
            let branch = branch.strip_prefix("...").unwrap_or(branch);
            let condition = match top_level(branch, ':').as_slice() {
                [condition, _, ..] => *condition,
                _ => {
                    return Err(format!(
                        "if(): write each branch as <condition>: <value>, not {:?}",
                        branch.trim()
                    ));
                }
            };
            let value = &branch[condition.len() + 1..];
            let holds = if_condition(condition.trim(), tests)?;
            if every {
                ifs(value, tests, true)?;
            } else if holds {
                chosen = Some(value.trim());
                break;
            }
        }
        if let Some(value) = chosen {
            out.push_str(&ifs(value, tests, false)?);
        }
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// `text` split at each top-level `sep`: one inside brackets, braces or a
/// string does not split. A `)` that closes nothing ends the last piece
/// there.
fn top_level(text: &str, sep: char) -> Vec<&str> {
    let (mut out, mut depth, mut quote, mut from) = (Vec::new(), 0i32, None, 0);
    for (i, c) in text.char_indices() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '(' | '[' | '{') => depth += 1,
            (None, ')') if depth == 0 => {
                out.push(&text[from..i]);
                return out;
            }
            (None, ')' | ']' | '}') => depth -= 1,
            (None, c) if c == sep && depth == 0 => {
                out.push(&text[from..i]);
                from = i + 1;
            }
            _ => {}
        }
    }
    out.push(&text[from..]);
    out
}

/// An `<if-condition>`: `else`, or `media()`, `supports()` and `style()`
/// tests joined by `not`, `and` and `or` (CSS Values 5 §8.3).
fn if_condition(condition: &str, tests: &mut IfTests<'_>) -> Result<bool, String> {
    use cssparser::{ParseError, Parser, Token};
    type Error = ParseError<String>;
    if condition == "else" {
        return Ok(true);
    }
    fn test(p: &mut Parser<'_>, tests: &mut IfTests<'_>) -> Result<bool, Error> {
        if p.try_parse(|p| p.expect_ident_matching("not")).is_ok() {
            return Ok(!group(p, tests)?);
        }
        let mut result = group(p, tests)?;
        let mut operator: Option<String> = None;
        while !p.is_exhausted() {
            let word = p.expect_ident_cloned()?.to_ascii_lowercase();
            if (word != "and" && word != "or") || operator.as_ref().is_some_and(|o| *o != word) {
                return Err(ParseError::custom(
                    "join if() tests with one of and or or; group mixed ones in parentheses"
                        .to_string(),
                ));
            }
            // Every test parses, held or not.
            let next = group(p, tests)?;
            result = if word == "and" {
                result && next
            } else {
                result || next
            };
            operator = Some(word);
        }
        Ok(result)
    }
    fn group(p: &mut Parser<'_>, tests: &mut IfTests<'_>) -> Result<bool, Error> {
        match p.next()?.clone() {
            Token::ParenthesisBlock => p.parse_nested_block(|p| test(p, tests)),
            Token::Function(name) => {
                let name = name.to_ascii_lowercase();
                p.parse_nested_block(|p| {
                    let start = p.position();
                    while p.next().is_ok() {}
                    let arg = p.slice_from(start).trim();
                    leaf(&name, arg, tests).map_err(ParseError::custom)
                })
            }
            _ => Err(ParseError::custom(
                "if() tests are media(…), supports(…) and style(…), joined by not, and and or"
                    .to_string(),
            )),
        }
    }
    fn leaf(name: &str, arg: &str, tests: &mut IfTests<'_>) -> Result<bool, String> {
        match name {
            // A feature or a condition: `media(width >= 600px)`.
            "media" => super::media::Query::parse(&format!("({arg})")).map(|q| (tests.media)(&q)),
            // A declaration or a condition: `supports(display: grid)`.
            "supports" => supports(arg).or_else(|_| supports(&format!("({arg})"))),
            "style" => super::media::Query::parse_container(&format!("style({arg})")).map(|q| {
                q.test(&mut |f| {
                    super::container::style_holds(f, &mut |n: &String| (tests.style)(n))
                })
            }),
            _ => Err(format!(
                "if() tests are media(…), supports(…) and style(…), not {name}()"
            )),
        }
    }
    cssparser::Parser::new(condition)
        .parse_entirely(|p| test(p, tests))
        .map_err(|e| format!("if(): the condition {condition:?}: {:?}", e.kind))
}

/// `@function --name(<param>#?) returns <css-type>? { … }` after the
/// at-keyword (CSS Custom Functions and Mixins 1 §2.1): its name and
/// definition. A parameter is `--name <css-type>? [: <default>]?`, a type
/// a syntax component or `type(<syntax>)`; the body holds custom
/// properties and `result`, in `@media` and `@supports` too.
fn function_rule(
    prelude: &str,
    block: &str,
    block_at: usize,
    custom_media: &[(String, super::media::Query)],
) -> Result<(String, super::at_rules::Function), Error> {
    use super::at_rules::{Function, Param, matches_syntax};
    const HOW: &str =
        "@function --name(--param <type>: <default>, …) returns <type> { result: …; }";
    let fail = |e: String| (0, e);
    let open = prelude.find('(').ok_or_else(|| fail(HOW.into()))?;
    let name = prelude[..open].trim();
    if !name.starts_with("--")
        || name.len() < 3
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(fail(format!(
            "@function names a --name, not {name:?}: {HOW}"
        )));
    }
    let inside = top_level(&prelude[open + 1..], ')');
    let params_text = inside.join(",");
    let after = prelude[open + 1 + params_text.len()..]
        .strip_prefix(')')
        .ok_or_else(|| fail(format!("@function {name}( is not closed")))?
        .trim();
    // `type(<syntax>)` or one syntax component.
    let css_type = |t: &str| -> String {
        let t = t.trim();
        t.strip_prefix("type(")
            .and_then(|t| t.strip_suffix(')'))
            .unwrap_or(t)
            .trim()
            .to_string()
    };
    let returns = match after {
        "" => None,
        a => Some(css_type(a.strip_prefix("returns").ok_or_else(|| {
            fail(format!(
                "@function {name}: write returns <type> after the parameters, not {a:?}"
            ))
        })?)),
    };
    let mut params: Vec<Param> = Vec::new();
    for param in top_level(&params_text, ',') {
        let param = param.trim();
        if param.is_empty() && params_text.trim().is_empty() {
            break;
        }
        let (head, default) = match top_level(param, ':').as_slice() {
            [head, _, ..] => (*head, Some(param[head.len() + 1..].trim().to_string())),
            _ => (param, None),
        };
        let (pname, ty) = head
            .trim()
            .split_once(char::is_whitespace)
            .unwrap_or((head.trim(), ""));
        if !pname.starts_with("--") || pname.len() < 3 || params.iter().any(|p| p.name == pname) {
            return Err(fail(format!(
                "@function {name}: each parameter is a distinct --name, not {pname:?}"
            )));
        }
        let syntax = (!ty.trim().is_empty()).then(|| css_type(ty));
        if let (Some(s), Some(d)) = (&syntax, &default)
            && !matches_syntax(s, d)
        {
            return Err(fail(format!(
                "@function {name}: {pname}'s default {d} does not match its type {s}"
            )));
        }
        params.push(Param {
            name: pname.to_string(),
            syntax,
            default,
        });
    }
    fn body(
        name: &str,
        block: &str,
        block_at: usize,
        conditions: &[super::media::Query],
        custom_media: &[(String, super::media::Query)],
        out: &mut Vec<(Vec<super::media::Query>, StyleDeclaration)>,
    ) -> Result<(), Error> {
        for item in items(block, block_at)? {
            match item {
                Item::Statement(statement, at) => {
                    for d in variables::declarations(statement).map_err(|e| (at, e))? {
                        if !d.name.starts_with("--") && d.name != "result" {
                            return Err((
                                at,
                                format!(
                                    "@function {name} takes custom properties and result, not {}",
                                    d.name
                                ),
                            ));
                        }
                        if d.source.contains("if(") {
                            check_ifs(&d.source).map_err(|e| (at, e))?;
                        }
                        out.push((conditions.to_vec(), d));
                    }
                }
                Item::Block {
                    prelude,
                    block,
                    at,
                    block_at,
                } => match at_rule(prelude) {
                    Some((rule, rest)) if rule == "media" => {
                        let q = super::media::Query::parse(rest)
                            .and_then(|q| q.resolve(custom_media))
                            .map_err(|e| (at, e))?;
                        let mut inner = conditions.to_vec();
                        inner.push(q);
                        body(name, block, block_at, &inner, custom_media, out)?;
                    }
                    Some((rule, rest)) if rule == "supports" => {
                        if supports(rest).map_err(|e| (at, e))? {
                            body(name, block, block_at, conditions, custom_media, out)?;
                        }
                    }
                    _ => {
                        return Err((
                            at,
                            format!("@function {name} takes @media and @supports, not {prelude}"),
                        ));
                    }
                },
            }
        }
        Ok(())
    }
    let mut decls = Vec::new();
    body(name, block, block_at, &[], custom_media, &mut decls)?;
    Ok((
        name.to_string(),
        Function {
            params,
            returns,
            body: decls,
        },
    ))
}

/// `text` with each custom function call (`--name(<arg>#?)`) replaced by
/// its result (CSS Custom Functions and Mixins 1 §3): its body evaluated
/// on a hypothetical child of the calling context, whose custom
/// properties `site` gives, `media` testing its `@media`. An argument is
/// substituted in the calling context, braces around it dropped; a
/// parameter takes it or its default, and one that does not match its
/// type, too many arguments, an unknown function, a cycle, no `result`
/// or a result that does not match `returns` make the call (and so its
/// property) invalid at computed-value time.
pub(crate) fn replace_functions(
    text: &str,
    functions: &[(String, super::at_rules::Function)],
    site: &mut dyn FnMut(&str) -> Option<String>,
    media: &mut dyn FnMut(&super::media::Query) -> bool,
    stack: &mut Vec<String>,
) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = dashed_call(rest) {
        let open = rest[at..].find('(').map(|i| at + i).expect("a call");
        let name = &rest[at..open];
        let inside = top_level(&rest[open + 1..], ')');
        let args_text = inside.join(",");
        let close = open + 1 + args_text.len();
        if rest.as_bytes().get(close) != Some(&b')') {
            return Err(format!("{name}( is not closed"));
        }
        out.push_str(&rest[..at]);
        let mut args = Vec::new();
        if !args_text.trim().is_empty() {
            for arg in top_level(&args_text, ',') {
                let arg = arg.trim();
                let arg = arg
                    .strip_prefix('{')
                    .and_then(|a| a.strip_suffix('}'))
                    .unwrap_or(arg)
                    .trim();
                args.push(replace_functions(arg, functions, site, media, stack)?);
            }
        }
        out.push_str(&call(name, args, functions, site, media, stack)?);
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Where the first `--name(` outside a string starts in `text`.
fn dashed_call(text: &str) -> Option<usize> {
    let b = text.as_bytes();
    let mut quote = None;
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None if c == b'"' || c == b'\'' => quote = Some(c),
            None if text[i..].starts_with("--")
                && (i == 0 || {
                    let p = b[i - 1];
                    !(p.is_ascii_alphanumeric() || p == b'-' || p == b'_')
                }) =>
            {
                let end = i + text[i..]
                    .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
                    .unwrap_or(text.len() - i);
                if end > i + 2 && b.get(end) == Some(&b'(') {
                    return Some(i);
                }
                i = end.max(i + 1);
                continue;
            }
            None => {}
        }
        i += 1;
    }
    None
}

/// One custom function call with its substituted arguments.
fn call(
    name: &str,
    args: Vec<String>,
    functions: &[(String, super::at_rules::Function)],
    site: &mut dyn FnMut(&str) -> Option<String>,
    media: &mut dyn FnMut(&super::media::Query) -> bool,
    stack: &mut Vec<String>,
) -> Result<String, String> {
    use super::at_rules::matches_syntax;
    let Some((_, f)) = functions.iter().rev().find(|(n, _)| n == name) else {
        return Err(format!("no @function {name}"));
    };
    if stack.iter().any(|n| n == name) {
        return Err(format!("{name}() calls itself"));
    }
    if args.len() > f.params.len() {
        return Err(format!(
            "{name}() takes {} arguments, not {}",
            f.params.len(),
            args.len()
        ));
    }
    // The hypothetical element's own custom properties: its parameters,
    // then the body's in order where its @media holds (a local shadows a
    // parameter), and its `result`.
    let mut defs: std::collections::BTreeMap<String, Option<String>> = Default::default();
    for (i, p) in f.params.iter().enumerate() {
        defs.insert(p.name.clone(), args.get(i).cloned().or(p.default.clone()));
    }
    for (conditions, d) in &f.body {
        if conditions.iter().all(&mut *media) {
            defs.insert(d.name.clone(), Some(d.source.clone()));
        }
    }
    struct Frame {
        defs: std::collections::BTreeMap<String, Option<String>>,
        done: std::collections::BTreeMap<String, Option<String>>,
        resolving: Vec<String>,
    }
    impl Frame {
        fn resolve(
            &mut self,
            n: &str,
            site: &mut dyn FnMut(&str) -> Option<String>,
        ) -> Option<String> {
            if let Some(v) = self.done.get(n) {
                return v.clone();
            }
            let Some(text) = self.defs.get(n).cloned() else {
                return site(n);
            };
            if self.resolving.iter().any(|r| r == n) {
                return None;
            }
            self.resolving.push(n.to_string());
            let v = text.and_then(|t: String| {
                let value = variables::Value::parse(&t).ok()?;
                value.substitute(&mut |m| self.resolve(m, site)).ok()
            });
            self.resolving.pop();
            self.done.insert(n.to_string(), v.clone());
            v
        }
    }
    let mut frame = Frame {
        defs,
        done: Default::default(),
        resolving: Vec::new(),
    };
    for p in &f.params {
        let value = frame.resolve(&p.name, site);
        if let (Some(s), Some(v)) = (&p.syntax, &value)
            && !matches_syntax(s, v)
        {
            return Err(format!("{name}(): {} {} is not a {s}", p.name, v.trim()));
        }
    }
    let Some(result) = frame.resolve("result", site) else {
        return Err(format!("{name}() has no result"));
    };
    let frame = std::cell::RefCell::new(frame);
    let site = std::cell::RefCell::new(site);
    let mut local = |n: &str| frame.borrow_mut().resolve(n, *site.borrow_mut());
    let result = replace_ifs(
        &result,
        &mut IfTests {
            media: &mut |q| media(q),
            style: &mut local,
        },
    )?;
    stack.push(name.to_string());
    let result = replace_functions(&result, functions, &mut local, media, stack);
    stack.pop();
    let result = result?.trim().to_string();
    if let Some(t) = &f.returns {
        let checked = if super::calc::has_math(&result) {
            super::calc::reduce(&result).ok()
        } else {
            Some(result.clone())
        };
        if checked.is_some_and(|c| !matches_syntax(t, &c)) {
            return Err(format!("{name}() returns a {t}, not {result}"));
        }
    }
    Ok(result)
}

/// Whether the cascade takes `name: value` as written.
fn declaration_supported(name: &str, value: &str) -> bool {
    if name.starts_with("--") {
        return true;
    }
    lower(&format!("{name}: {value};"), &|_| false).is_ok()
}

/// The properties `all` sets (CSS Cascade 5 §3.2.1): every longhand the
/// cascade keeps but `direction`, `unicode-bidi` and the custom properties.
fn all_longhands() -> impl Iterator<Item = &'static str> {
    let mut names: Vec<&'static str> = css::STYLE
        .iter()
        .map(|&(_, name)| name)
        .chain(super::motion::TRANSITION)
        .chain(super::motion::ANIMATION)
        .chain(super::motion::ANIMATION_EXTRA)
        .chain(super::motion::TIMELINES)
        .chain(super::motion::TRIGGERS)
        .chain(super::container::LONGHANDS)
        .chain(crate::transform3d::LONGHANDS)
        .chain(super::text_css::INHERITED.iter().copied())
        .chain(super::text_css::LOCAL.iter().copied())
        .chain(super::extern_element::PRESENTATION.iter().copied())
        .chain([
            "text-anchor",
            "dominant-baseline",
            "writing-mode",
            "text-orientation",
            "text-combine-upright",
            "scroll-initial-target",
            "counter-reset",
            "counter-increment",
            "counter-set",
            "quotes",
            "list-style-image",
            "list-style-type",
            "list-style-position",
        ])
        .filter(|name| !matches!(*name, "direction" | "unicode-bidi"))
        .collect();
    names.sort_unstable();
    names.dedup();
    names.into_iter()
}

/// A rule body's declarations, expanded to the longhands the cascade
/// keeps, and the text of those it can check at cook (no `var()`, no
/// CSS-wide keyword). `keyframes`: whether a name is a defined
/// `@keyframes`.
pub(super) fn lower(
    text: &str,
    keyframes: &dyn Fn(&str) -> bool,
) -> Result<(Vec<StyleDeclaration>, String), String> {
    let mut declarations = Vec::new();
    for declaration in variables::declarations(text)? {
        // An `if()`'s branches and tests are checked as written; which
        // branch it takes is known where its element resolves.
        if declaration.source.contains("if(") {
            check_ifs(&declaration.source).map_err(|e| format!("{}: {e}", declaration.name))?;
        }
        // A random function's key names the property it is parsed for,
        // a shorthand's before it expands (CSS Values 5 §9.4.1); a custom
        // property's is named where it is substituted.
        let declaration =
            if declaration.name.starts_with("--") || !declaration.source.contains("random") {
                declaration
            } else {
                let source = super::calc::random_keys(&declaration.source, &declaration.name);
                StyleDeclaration {
                    value: variables::Value::parse(&source)?,
                    source,
                    ..declaration
                }
            };
        if !declaration.name.starts_with("--") && wide_keyword(&declaration.source).is_some() {
            // A CSS-wide keyword applies to each longhand of a shorthand.
            let longhands: Vec<String> = match declaration.name.as_str() {
                "transition" => super::motion::TRANSITION
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                "animation" => super::motion::ANIMATION
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                "container" => super::container::LONGHANDS
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                "list-style" => vec![
                    "list-style-image".into(),
                    "list-style-type".into(),
                    "list-style-position".into(),
                ],
                "all" => all_longhands().map(String::from).collect(),
                "glyph-orientation-vertical" => vec!["text-orientation".into()],
                name => vec![name.to_string()],
            };
            for name in longhands {
                declarations.push(StyleDeclaration {
                    name,
                    ..declaration.clone()
                });
            }
            continue;
        }
        if declaration.name == "all" {
            return Err(format!(
                "all takes only inherit, initial, unset, revert, revert-layer or revert-rule, written out, not {}",
                declaration.source
            ));
        }
        for declaration in super::container::expand(declaration).and_then(|ds| {
            let mut out = Vec::new();
            for d in ds {
                for d in super::lists::expand(d)? {
                    out.extend(super::text_css::expand(d)?);
                }
            }
            Ok(out)
        })? {
            declarations.extend(super::motion::expand(declaration, keyframes)?);
        }
    }
    let mut body = String::new();
    for declaration in &declarations {
        let name = declaration.name.as_str();
        if crate::layout::flow::longhand(name) {
            if wide_keyword(&declaration.source).is_none() && !declaration.value.has_variables() {
                crate::layout::flow::check(name, &declaration.source)?;
            }
            continue;
        }
        if name.starts_with("--")
            || name == "font-family"
            || super::motion::longhand(name)
            || super::container::longhand(name)
            || super::extern_element::svg_longhand(name)
            || super::text_css::longhand(name)
        {
            continue;
        }
        if crate::transform3d::LONGHANDS.contains(&name) {
            if wide_keyword(&declaration.source).is_none() && !declaration.value.has_variables() {
                let mut errors = Vec::new();
                if let Ok(value) = declaration.value.substitute(&mut |_| None)
                    && !super::calc::has_absolute(&value)
                {
                    let value = super::calc::reduce(&value)?;
                    crate::transform3d::resolve(&[(name.into(), value)], &mut errors);
                }
                if let Some(e) = errors.into_iter().next() {
                    return Err(e);
                }
            }
            continue;
        }
        if super::generated::longhand(name) {
            if wide_keyword(&declaration.source).is_none() && !declaration.value.has_variables() {
                super::generated::check(name, &declaration.source)?;
            }
            continue;
        }
        if css::targets(name).next().is_none() {
            return Err(format!(
                "unsupported CSS property {name}: {}",
                css::unsupported(name)
                    .unwrap_or("use a property listed in the document CSS reference")
            ));
        }
        if wide_keyword(&declaration.source).is_some() || declaration.value.has_variables() {
            continue;
        }
        let value = declaration.value.substitute(&mut |_| None)?;
        // Viewport, root and font units, env() and attr() resolve when
        // the element does.
        if !super::calc::per_element(name, &value) {
            let value = super::calc::reduce(&value)?;
            body.push_str(&format!("{name}: {value};"));
        }
    }
    // The values the cascade takes as written (`@supports` asks this).
    css::declarations(&body, 0)?;
    Ok((declarations, body))
}

fn matches(
    doc: &UiDocument,
    elements: &[(usize, String, Vec<Attr>)],
    index: usize,
    selector: &str,
    at: usize,
) -> Result<Option<Expr>, Error> {
    let parsed = Selector::parse_list(selector).map_err(|e| (at, e))?;
    let condition = parsed[0]
        .condition(doc, elements, index)
        .map_err(|e| (at, e))?;
    Ok(match condition {
        Expr::Lit(serde_json::Value::Bool(false)) => None,
        condition => Some(condition),
    })
}

/// A rule's reflected values for an element of `kind`.
fn rule_props(entries: &[og_core::ogt::Entry], kind: Kind) -> Result<Vec<PropSet>, String> {
    let mut props = Vec::new();
    for e in entries {
        // `flex-direction` is a box's layout and a checkbox's
        // label's caption side: each element takes its own.
        let key = e.key.as_str();
        if (key == "direction" && kind == Kind::Toggle)
            || (key == "labelDirection" && kind != Kind::Toggle)
        {
            continue;
        }
        let (spec, p) = schema::prop(key).ok_or(format!("unknown CSS target {}", e.key))?;
        // An element that is no box takes the outer display only:
        // its inner layout is its tag's.
        let inner = matches!(
            written(&e.values).as_str(),
            "block" | "inline" | "inlineBlock" | "flowRoot" | "listItem"
        );
        let inherited = css::STYLE.iter().any(|(property, name)| {
            *property == key && super::build::cascade::inherited_property(name)
        });
        let ruby = written(&e.values).starts_with("ruby");
        // An inline element of a text has no box of its own yet: what does
        // not apply to it is ignored, as CSS ignores it on an inline box.
        if kind == Kind::Inline {
            let shown = matches!(written(&e.values).as_str(), "inline" | "none") || ruby;
            if (!p.applies(kind) && !inherited) || (e.key == "display" && !shown) {
                continue;
            }
        } else if e.key == "display" && ruby {
            return Err(format!(
                "display: {} is an inline element's of a text (a <span>, <rt>…): a {kind} is no ruby box",
                written(&e.values)
            ));
        }
        if kind != Kind::Inline
            && ((!p.applies(kind) && !inherited)
                || (e.key == "display" && kind != Kind::Box && !inner))
        {
            return Err(if e.key == "display" {
                format!(
                    "display: {} lays out a <div> (or another HTML block); a {kind}'s layout is its tag, so it takes only none, block, inline, inline-block or flow-root",
                    written(&e.values)
                )
            } else {
                format!("CSS property {} does not apply to {kind}", e.key)
            });
        }
        props.push(PropSet {
            spec,
            value: parse_value(p.ty, &e.values)?,
            line: e.line,
            source: written(&e.values),
        });
    }
    Ok(props)
}

/// The pseudo-elements that are boxes of their own (CSS Pseudo-Elements 4
/// §3, CSS Positioned Layout 4's `::backdrop`, the customizable select's
/// `::picker(select)`, a disclosure's `::details-content` (CSS
/// Pseudo-Elements 4 §5.2), and the engine's `::-og-hold`, the fill a
/// `data-hold` press grows, and `::-og-arrow`, a popover's arrow toward its
/// anchor), and CSS Overflow 5's scroll controls (`::scroll-marker-group`,
/// `::scroll-marker`, `::scroll-button(<dir>)`, the logical directions as
/// horizontal-tb left to right names them), and a multi-column
/// container's `::column::scroll-marker` (CSS Multicol 2 §4.8), by their
/// name in [`DocNode::pseudo`]. Longer spellings come first: a selector is
/// matched by its ending.
const BOXES: [(&str, &str); 19] = [
    ("::before", "before"),
    ("::after", "after"),
    ("::backdrop", "backdrop"),
    ("::picker(select)", "picker"),
    ("::details-content", "details-content"),
    ("::file-selector-button", "file-selector-button"),
    ("::-og-hold", "hold"),
    ("::-og-arrow", "arrow"),
    ("::scroll-marker-group", "scroll-marker-group"),
    ("::column::scroll-marker", "column-scroll-marker"),
    ("::scroll-marker", "scroll-marker"),
    ("::scroll-button(up)", "scroll-button-up"),
    ("::scroll-button(block-start)", "scroll-button-up"),
    ("::scroll-button(down)", "scroll-button-down"),
    ("::scroll-button(block-end)", "scroll-button-down"),
    ("::scroll-button(left)", "scroll-button-left"),
    ("::scroll-button(inline-start)", "scroll-button-left"),
    ("::scroll-button(right)", "scroll-button-right"),
    ("::scroll-button(inline-end)", "scroll-button-right"),
];

/// The box pseudo-elements an element has one instance of per fragment
/// (CSS Multicol 2 §4.8: a `::column::scroll-marker` per column), each
/// with states of its own: their rules match those states per instance
/// (`StyleRule::instance_states`), not on one node.
pub(crate) const INSTANCED: [&str; 1] = ["column-scroll-marker"];

/// `::scroll-button(*)`, which selects all four of a container's buttons.
const ANY_BUTTON: &str = "::scroll-button(*)";

/// Whether `node`, an element of `kind`, draws `pseudo` as a box: a box
/// element's `::before` and `::after` (a text element's join its text as
/// runs), a dialog's or a popover's `::backdrop`, a select's `::picker(select)`, a
/// details element's `::details-content`, a file input's
/// `::file-selector-button`, a `data-hold` element's
/// `::-og-hold`, a popover's `::-og-arrow`.
fn holds(pseudo: &str, kind: Kind, node: &DocNode) -> bool {
    match pseudo {
        "before" | "after" => {
            kind.container()
                && kind != Kind::Inline
                && (kind != Kind::Button || !node.children.is_empty())
        }
        "backdrop" => kind == Kind::Dialog || node.props.iter().any(|p| p.name() == "popover"),
        "picker" => kind == Kind::Select,
        "details-content" => kind == Kind::Details,
        "file-selector-button" => kind == Kind::File,
        "hold" => node.props.iter().any(|p| p.name() == "dataHold"),
        "arrow" => node.props.iter().any(|p| p.name() == "popover"),
        // Any element may have a marker (its nearest scroll container
        // decides whether it shows); the group and buttons a container's.
        "scroll-marker" => true,
        "scroll-marker-group"
        | "column-scroll-marker"
        | "scroll-button-up"
        | "scroll-button-down"
        | "scroll-button-left"
        | "scroll-button-right" => kind.container(),
        _ => false,
    }
}

/// The synthetic node of element `i`'s `pseudo`, made on first use. A
/// `::before` or `::after` is inline unless its rules say otherwise.
fn pseudo_node(doc: &mut UiDocument, i: usize, pseudo: &'static str) -> usize {
    if let Some(&(_, at)) = doc.nodes[i].pseudo.iter().find(|(p, _)| *p == pseudo) {
        return at;
    }
    let at = doc.nodes.len();
    let mut node = DocNode {
        kind: NodeKind::Element(Kind::Box),
        id: String::new(),
        selector_tag: None,
        selector_attrs: Vec::new(),
        props: Vec::new(),
        enter: None,
        exit: None,
        at: Vec::new(),
        rules: Vec::new(),
        span_rules: Vec::new(),
        groups: Vec::new(),
        triggers: Vec::new(),
        args: Vec::new(),
        children: Vec::new(),
        parent: i,
        role: None,
        level: 0,
        inline: Vec::new(),
        links: Vec::new(),
        source_text: None,
        custom: Vec::new(),
        uses: Vec::new(),
        pseudo: Vec::new(),
        frame: None,
        ..doc.nodes[i].clone()
    };
    // A file input's button is a `<button>`: its box declarations are its
    // plate's, as an HTML button's are. It is no element a selector
    // reaches (those are the document's own, listed before this node).
    if pseudo == "file-selector-button" {
        node.kind = NodeKind::Element(Kind::Button);
        node.selector_tag = Some("button".into());
    }
    if matches!(pseudo, "before" | "after") {
        let entries = css::declarations("display: inline;", node.line).expect("a display");
        node.props = rule_props(&entries, Kind::Box).expect("a box display");
    }
    doc.nodes.push(node);
    doc.nodes[i].pseudo.push((pseudo, at));
    at
}

/// The states a pseudo-class after an element-backed pseudo-element may
/// name (Selectors 4 §3.6.4's user-action pseudo-classes, a control's
/// `:disabled` and `:enabled`, and CSS Overflow 5's scroll-marker states):
/// they are the pseudo-element's own, not its originating element's.
const PSEUDO_STATES: [InteractionState; 9] = {
    use InteractionState::*;
    [
        Hover,
        Active,
        Focus,
        FocusVisible,
        Disabled,
        Enabled,
        TargetCurrent,
        TargetBefore,
        TargetAfter,
    ]
};

/// `selector` without the state pseudo-classes it ends with, and those
/// states in the order written.
fn trailing_states(selector: &str) -> (&str, Vec<InteractionState>) {
    let (mut rest, mut states) = (selector, Vec::new());
    while let Some(at) = rest.rfind(':') {
        if at == 0 || rest.as_bytes()[at - 1] == b':' {
            break;
        }
        let name = rest[at + 1..].to_ascii_lowercase();
        let Some(state) = super::selectors::state(&name).filter(|s| PSEUDO_STATES.contains(s))
        else {
            break;
        };
        states.insert(0, state);
        rest = &rest[..at];
    }
    (rest, states)
}

/// The element-backed pseudo-element a selector ends with
/// ([`super::generated::PSEUDOS`], [`BOXES`]), the selector without it,
/// and the state pseudo-classes after it (`dialog::backdrop:hover`,
/// `::scroll-marker:target-current`), which only a pseudo-element that is
/// a box of its own beside its element's content takes. CSS 2's `:before`,
/// `:after`, `:first-line` and `:first-letter` count too.
fn pseudo_suffix(selector: &str) -> Option<(Cow<'static, str>, &str, Vec<InteractionState>)> {
    let lower = selector.to_ascii_lowercase();
    let boxed = |lower: &str| {
        BOXES[2..]
            .iter()
            .find(|(w, _)| lower.ends_with(w))
            .map(|(written, name)| (*name, lower.len() - written.len()))
    };
    if let Some((name, at)) = boxed(&lower) {
        return Some((name.into(), &selector[..at], Vec::new()));
    }
    let (stem, states) = trailing_states(&lower);
    if !states.is_empty()
        && let Some((name, at)) = boxed(stem)
    {
        return Some((name.into(), &selector[..at], states));
    }
    // `::highlight(<custom-ident>)`, its name as written (case matters).
    const HIGHLIGHT: &str = "::highlight(";
    if let Some(at) = lower.rfind(HIGHLIGHT)
        && lower.ends_with(')')
    {
        let name = selector[at + HIGHLIGHT.len()..selector.len() - 1].trim();
        let mut parser = cssparser::Parser::new(name);
        let ident = parser.expect_ident().is_ok() && parser.is_exhausted();
        return ident.then(|| {
            let pseudo = format!("{}{name}", super::generated::HIGHLIGHT);
            (pseudo.into(), &selector[..at], Vec::new())
        });
    }
    // A scroll control is written as its box ([`BOXES`]), never by name.
    super::generated::PSEUDOS
        .iter()
        .filter(|p| !p.starts_with("scroll-") && **p != "column-scroll-marker")
        .find_map(|&p| {
            let double = format!("::{p}");
            if lower.ends_with(&double) {
                return Some((
                    p.into(),
                    &selector[..selector.len() - double.len()],
                    Vec::new(),
                ));
            }
            let single = format!(":{p}");
            (matches!(p, "before" | "after" | "first-line" | "first-letter")
                && lower.ends_with(&single))
            .then(|| {
                (
                    p.into(),
                    &selector[..selector.len() - single.len()],
                    Vec::new(),
                )
            })
        })
}

/// A rule on an element-backed pseudo-element selects its originating
/// element. The declarations generated content and text pseudo-elements
/// read become the element's own, each named `-og-pe-<pseudo>__<property>`:
/// they cascade with the element's (states, layers, media, variables) and
/// the build generates or styles the pseudo-element from the winners. A
/// box pseudo-element's other declarations (its box: display, size,
/// padding, border, background, position, transforms…) go to its own node
/// ([`Rule::boxed`]), cascading under the same selector.
fn pseudo_element(rule: &mut Rule) -> Result<(), (usize, String)> {
    let selectors: Vec<&str> = rule.selector.split(',').map(str::trim).collect();
    let Some((pseudo, _, states)) = selectors.iter().find_map(|s| pseudo_suffix(s)) else {
        return Ok(());
    };
    let mut originating = Vec::new();
    for s in &selectors {
        match pseudo_suffix(s) {
            Some((p, rest, after)) if p == pseudo && after == states => {
                originating.push(if rest.is_empty() { "*" } else { rest });
            }
            _ => {
                return Err((
                    rule.at,
                    format!(
                        "{}: write ::{pseudo} rules apart from other selectors",
                        rule.selector
                    ),
                ));
            }
        }
    }
    let boxed = BOXES.iter().find(|(_, name)| *name == pseudo).map(|b| b.1);
    let (mut body, mut own) = (String::new(), String::new());
    for d in variables::declarations(&rule.body).map_err(|e| (rule.at, e))? {
        let important = if d.important { " !important" } else { "" };
        let generated = !matches!(
            pseudo.as_ref(),
            "backdrop" | "picker" | "details-content" | "hold" | "arrow" | "scroll-marker-group"
        ) && super::generated::allows(&pseudo, &d.name);
        if generated {
            body.push_str(&format!(
                "{}: {}{important};",
                super::generated::encode(&pseudo, &d.name),
                d.source,
            ));
        } else if boxed.is_none() {
            return Err((rule.at, format!("::{pseudo} does not take {}", d.name)));
        }
        // `display` lays out the box and hides the generated text.
        if boxed.is_some() && (!generated || d.name == "display") {
            own.push_str(&format!("{}: {}{important};", d.name, d.source));
        }
    }
    rule.selector = originating.join(", ");
    rule.body = body;
    if let Some(pseudo) = boxed {
        rule.pseudo = Some(pseudo);
        rule.boxed = own;
        rule.pseudo_states = states;
    }
    Ok(())
}

/// An element's own declarations (its `style` attribute or a `style:`
/// directive, `body` at `at`), expanded and checked as a rule's are.
pub fn inline(
    doc: &UiDocument,
    node: &super::DocNode,
    body: &str,
    at: usize,
) -> Result<Vec<StyleDeclaration>, Error> {
    let known: Vec<String> = doc
        .sheet
        .keyframe_rules
        .iter()
        .map(|(n, ..)| n.clone())
        .collect();
    let mut out = Vec::new();
    let body = super::at_rules::rebase_urls(&doc.source, body);
    for declaration in variables::declarations(&body).map_err(|e| (at, e))? {
        for d in super::container::expand(declaration)
            .and_then(|ds| {
                let mut v = Vec::new();
                for d in ds {
                    v.extend(super::lists::expand(d)?);
                }
                Ok(v)
            })
            .map_err(|e| (at, e))?
        {
            out.extend(
                super::motion::expand(d, &|name| known.iter().any(|k| k == name))
                    .map_err(|e| (at, e))?,
            );
        }
    }
    let kind = match node.kind {
        NodeKind::Element(k) => k,
        _ => Kind::Column,
    };
    for d in &out {
        let name = d.name.as_str();
        if name.starts_with("--")
            || super::motion::longhand(name)
            || super::container::longhand(name)
            || super::extern_element::svg_longhand(name)
            || super::text_css::longhand(name)
            || crate::layout::flow::longhand(name)
        {
            continue;
        }
        if css::targets(name).next().is_none() {
            return Err((
                at,
                format!(
                    "unsupported CSS property {name}: {}",
                    css::unsupported(name)
                        .unwrap_or("use a property listed in the document CSS reference")
                ),
            ));
        }
        if d.value.has_variables() {
            continue;
        }
        let value = d.value.substitute(&mut |_| None).map_err(|e| (at, e))?;
        if super::calc::has_absolute(&value) {
            continue;
        }
        let value = super::calc::reduce(&value).map_err(|e| (at, e))?;
        for e in css::declarations(&format!("{name}: {value};"), 0).map_err(|e| (at, e))? {
            let applies = schema::prop(&e.key).is_some_and(|(_, p)| p.applies(kind));
            let side = (e.key == "direction" && kind == Kind::Toggle)
                || (e.key == "labelDirection" && kind != Kind::Toggle);
            if !applies && !side && !super::build::cascade::inherited_property(name) {
                return Err((at, format!("CSS property {name} does not apply to {kind}")));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A pack's direct sheets and the stylesheets it may `@import`.
    fn pack(name: &str) -> (String, Vec<(String, String)>) {
        let directory = format!("styles/{name}");
        let files = std::fs::read_dir(format!(
            "{}/../../templates/{directory}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .expect("a template style pack")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "css"))
        .map(|path| {
            (
                format!(
                    "{directory}/{}",
                    path.file_name().unwrap().to_str().unwrap()
                ),
                std::fs::read_to_string(&path).unwrap(),
            )
        })
        .collect();
        (directory, files)
    }

    /// Rules that tie with the default style's and declare what a sheet
    /// can: layers out of order, keyframes, a scheme, a query, a box
    /// pseudo-element.
    const OWN: &str = r#"
@layer og-default { button::part(base) { padding: 3px; } :root { --own: 1; } [part$=".frame"] { gap: 3px; } }
@layer og-pack.late, og-pack.early;
@layer og-pack.early { .a { color: red; } }
@layer { .anonymous { color: blue; } }
@keyframes own-spin { to { rotate: 1turn; } }
.b { animation: own-spin 1s; }
button::part(base):hover { background: red; }
@media (width < 600px) { :root { --narrow: 2; } }
:root[data-scheme=light] { --light: 3; }
.c::before { content: "x"; }
"#;

    /// A pack is read as its sheets' text before the default style's, and
    /// the default style's is not read for it.
    ///
    /// Ways it could fail: the default style's rules ordered before the
    /// pack's (a pack's rule in `og-default` then wins a tie it loses),
    /// their lines not counted from the pack's `@import`s, a layer the
    /// pack declares ranked apart from the whole text's.
    #[test]
    fn a_pack_reads_as_its_sheets_before_the_default_text() {
        let synthetic = (
            "styles/own".to_string(),
            vec![
                (
                    "styles/own/a.css".to_string(),
                    format!("@import \"sub/b.css\" layer(inner);\n{OWN}"),
                ),
                (
                    "styles/own/sub/b.css".to_string(),
                    ".d { width: 4px; }".to_string(),
                ),
            ],
        );
        for (directory, files) in ["cartoon", "fantasy", "scifi"]
            .map(pack)
            .into_iter()
            .chain([synthetic])
        {
            let css = crate::style_pack::StylePack::sheet(&directory, &files).unwrap();
            let source = format!("{directory}/");
            std::sync::LazyLock::force(&DEFAULT);
            let before = text_read();
            let read = pack_reading(&source, &css, files.clone()).unwrap();
            let own = css.len() + files.iter().map(|(_, text)| text.len()).sum::<usize>();
            assert_eq!(text_read() - before, own, "{directory}: only its own text");
            let whole = format!("{css}{}", crate::DEFAULT_CSS);
            let whole = Reading::read(&source, &whole, 0, files).unwrap();
            assert!(read == whole, "{directory}: as the whole text reads");
            assert!(read.count > DEFAULT.count, "{directory}");
        }
    }

    /// The role rules a document without a pack takes are those of the
    /// default style's last section, from the process's one reading.
    #[test]
    fn the_role_rules_are_the_default_readings_last_section() {
        let alone = format!(
            "@layer og-default {{\n{}",
            &crate::DEFAULT_CSS[role_rules()..]
        );
        let alone = Global::new(Reading::read("", &alone, 0, Vec::new()).unwrap()).unwrap();
        // All but its place in the order, which counts the rules before it.
        let rule = |g: &GlobalRule| {
            let lowered = g.lowered.as_ref().map(|(own, _)| own.0.clone());
            let rule = Rule {
                order: 0,
                ..g.rule.clone()
            };
            (g.selector.clone(), rule, lowered)
        };
        let rules = |g: &Global| g.rules.iter().map(rule).collect::<Vec<_>>();
        assert!(rules(&alone).len() > 20 && rules(&alone) == rules(&DEFAULT_GLOBAL));
        let orders = |g: &Global| g.rules.iter().map(|g| g.rule.order).collect::<Vec<_>>();
        assert!(orders(&DEFAULT_GLOBAL).is_sorted());
        assert!(DEFAULT_GLOBAL.count > *orders(&DEFAULT_GLOBAL).last().unwrap());
    }

    /// A part sheet that begins with the default style's text reads what
    /// follows it, after the default style's rules, and refuses what the
    /// whole text refuses.
    #[test]
    fn a_sheet_that_begins_with_the_default_text_reads_what_follows() {
        let css = format!("{}\n{OWN}", crate::DEFAULT_CSS);
        std::sync::LazyLock::force(&DEFAULT);
        let before = text_read();
        let read = Reading::read_sheet("<part-css>", &css, Vec::new()).unwrap();
        assert_eq!(text_read() - before, OWN.len() + 1, "only what follows");
        let whole = Reading::read("<part-css>", &css, 0, Vec::new()).unwrap();
        assert!(read == whole, "as the whole text reads");
        // The default style alone is the reading the process keeps.
        let alone = Reading::read_sheet("<part-css>", crate::DEFAULT_CSS, Vec::new()).unwrap();
        assert!(alone == *DEFAULT);
        for refused in [
            "@import \"x.css\";",
            ".a { colour: red; }",
            "Button { gap: 1px; }",
        ] {
            let css = format!("{}\n\n{refused}", crate::DEFAULT_CSS);
            let imports = vec![("x.css".to_string(), String::new())];
            let read = Reading::read_sheet("<part-css>", &css, imports.clone()).unwrap_err();
            let whole = Reading::read("<part-css>", &css, 0, imports).unwrap_err();
            assert_eq!(read, whole);
            assert!(read.starts_with("<part-css>:"), "{read}");
        }
    }
}
