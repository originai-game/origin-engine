//! Runtime matching of structural selectors against an instantiated element
//! tree: retained mounts read the logical [`super::topology::Topology`],
//! pushed screens the sibling levels their build instantiates. Matching reads
//! selector facts only (tag, attributes, classes, states, text); it never
//! evaluates viewmodel expressions. Compounds match only elements of the
//! rule's own document, so styles stay scoped across component boundaries,
//! while every instantiated element counts as a sibling.
use super::UiDocument;
use super::selectors::{Combinator, Compound, Nth, Selector, Simple};
use crate::bind::InteractionState;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// The selector inputs of one instantiated element.
pub struct Facts<'a> {
    pub document: &'a UiDocument,
    pub node: usize,
    pub attributes: &'a BTreeMap<String, Value>,
    pub states: &'a BTreeSet<InteractionState>,
    pub text: &'a str,
}

/// An instantiated element tree in logical order.
pub trait Tree {
    type Id: Clone + PartialEq;
    fn facts(&self, id: &Self::Id) -> Facts<'_>;
    fn parent(&self, id: &Self::Id) -> Option<Self::Id>;
    /// The immediately preceding element sibling.
    fn previous(&self, id: &Self::Id) -> Option<Self::Id>;
    /// The 0-based index among the element's siblings and their count.
    fn position(&self, id: &Self::Id) -> (usize, usize);
    /// Siblings before the element that match `filter`, and all that do.
    fn filtered(&self, id: &Self::Id, filter: &[Selector]) -> (usize, usize);
    fn has_children(&self, id: &Self::Id) -> bool;
    /// The element's instantiated element children, in order (`:has()`).
    fn children(&self, id: &Self::Id) -> Vec<Self::Id>;
    /// The immediately following element sibling (`:has(+ …)`).
    fn following(&self, id: &Self::Id) -> Option<Self::Id>;
}

/// The `of S` filter `:nth-of-type()` counts with: the element's own type.
pub fn type_filter(tag: &str) -> Vec<Selector> {
    vec![Selector {
        source: tag.to_string(),
        compounds: vec![Compound(vec![Simple::Type(tag.to_string())])],
        combinators: Vec::new(),
    }]
}

/// Work done by structural matching, for bounded-cost tests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MatchWork {
    /// Structural rule selectors evaluated against an element.
    pub selectors: usize,
    /// Sibling and ancestor elements visited while matching them.
    pub candidates: usize,
}

/// Whether `id` is the subject of `selector`, a rule of `document`.
pub fn matches<T: Tree>(
    tree: &T,
    document: &UiDocument,
    selector: &Selector,
    id: &T::Id,
    work: &mut MatchWork,
) -> bool {
    work.selectors += 1;
    chain(
        tree,
        document,
        selector,
        selector.compounds.len() - 1,
        id,
        None,
        work,
    )
}

fn chain<T: Tree>(
    tree: &T,
    document: &UiDocument,
    selector: &Selector,
    part: usize,
    id: &T::Id,
    anchor: Option<&T::Id>,
    work: &mut MatchWork,
) -> bool {
    if !compound_at(tree, document, &selector.compounds[part], id, anchor, work) {
        return false;
    }
    if part == 0 {
        return true;
    }
    let combinator = selector.combinators[part - 1];
    let mut next = match combinator {
        Combinator::Descendant | Combinator::Child => tree.parent(id),
        Combinator::Next | Combinator::Subsequent => tree.previous(id),
    };
    while let Some(candidate) = next {
        work.candidates += 1;
        if chain(tree, document, selector, part - 1, &candidate, anchor, work) {
            return true;
        }
        next = match combinator {
            Combinator::Descendant => tree.parent(&candidate),
            Combinator::Subsequent => tree.previous(&candidate),
            Combinator::Child | Combinator::Next => None,
        };
    }
    false
}

/// Whether one compound matches the element's own facts and position.
pub fn compound<T: Tree>(
    tree: &T,
    document: &UiDocument,
    compound: &Compound,
    id: &T::Id,
    work: &mut MatchWork,
) -> bool {
    compound_at(tree, document, compound, id, None, work)
}

fn compound_at<T: Tree>(
    tree: &T,
    document: &UiDocument,
    compound: &Compound,
    id: &T::Id,
    anchor: Option<&T::Id>,
    work: &mut MatchWork,
) -> bool {
    let facts = tree.facts(id);
    // The `:has()` anchor is the element it was matched from, whichever
    // document styles it.
    if let Some(anchor) = anchor
        && compound.0.first() == Some(&Simple::Anchor)
    {
        return id == anchor;
    }
    if facts.document.name != document.name {
        return false;
    }
    compound.0.iter().all(|simple| match simple {
        Simple::Nth(nth) => position(tree, document, nth, id, work),
        Simple::Only { of_type: false } => tree.position(id).1 == 1,
        Simple::Only { of_type: true } => {
            let filter = type_filter(tag(&facts));
            tree.filtered(id, &filter).1 == 1
        }
        Simple::Empty => facts.text.is_empty() && !tree.has_children(id),
        Simple::Has(list) => list
            .iter()
            .any(|relative| has(tree, document, relative, id, work)),
        Simple::Lang(ranges) => match inherited(tree, id, "lang") {
            Some(tag) => ranges.iter().any(|r| crate::bind::lang_matches(r, &tag)),
            None => ranges.iter().any(|r| crate::bind::lang_matches(r, "en")),
        },
        Simple::Dir(rtl) => {
            let written = inherited(tree, id, "dir");
            (written.as_deref() == Some("rtl")) == *rtl
        }
        Simple::Pseudo(list) => list
            .iter()
            .any(|inner| matches(tree, document, inner, id, work)),
        Simple::Not(list) => !list
            .iter()
            .any(|inner| matches(tree, document, inner, id, work)),
        Simple::Is(list) | Simple::Where(list) => list
            .iter()
            .any(|inner| matches(tree, document, inner, id, work)),
        simple => own(&facts, simple),
    })
}

/// The element's own type selector name.
fn tag<'a>(facts: &Facts<'a>) -> &'a str {
    facts.document.nodes[facts.node]
        .selector_tag
        .as_deref()
        .unwrap_or_default()
}

/// The nearest `name` attribute of the element or an ancestor, as text.
fn inherited<T: Tree>(tree: &T, id: &T::Id, name: &str) -> Option<String> {
    let mut at = Some(id.clone());
    while let Some(node) = at {
        if let Some(value) = tree.facts(&node).attributes.get(name)
            && !value.is_null()
        {
            return Some(crate::bind::text(value));
        }
        at = tree.parent(&node);
    }
    None
}

/// Whether `relative` (a `:has()` argument) matches an element reached
/// from `anchor`: the anchor's descendants for `>` and descendant
/// combinators, its later siblings and their descendants for `+` and `~`.
/// A selector of child or next-sibling steps only searches as deep or as
/// far as its steps reach.
fn has<T: Tree>(
    tree: &T,
    document: &UiDocument,
    relative: &Selector,
    anchor: &T::Id,
    work: &mut MatchWork,
) -> bool {
    let steps = &relative.combinators;
    let sibling = matches!(steps[0], Combinator::Next | Combinator::Subsequent);
    let down = steps
        .iter()
        .any(|c| matches!(c, Combinator::Descendant | Combinator::Child));
    // How deep below the start the subject can be: unbounded when any
    // step is a descendant combinator.
    let depth = if steps.contains(&Combinator::Descendant) {
        usize::MAX
    } else {
        steps.iter().filter(|c| **c == Combinator::Child).count()
    };
    let far = steps.contains(&Combinator::Subsequent);
    let reach = steps.iter().filter(|c| **c == Combinator::Next).count();
    let mut starts = Vec::new();
    if sibling {
        let mut next = tree.following(anchor);
        let mut k = 0;
        while let Some(s) = next {
            k += 1;
            if !far && k > reach {
                break;
            }
            next = tree.following(&s);
            starts.push((s, 0));
        }
    } else {
        starts.extend(tree.children(anchor).into_iter().map(|c| (c, 1)));
    }
    let subject = relative.compounds.len() - 1;
    while let Some((id, level)) = starts.pop() {
        work.candidates += 1;
        if chain(tree, document, relative, subject, &id, Some(anchor), work) {
            return true;
        }
        if down && level < depth {
            starts.extend(tree.children(&id).into_iter().map(|c| (c, level + 1)));
        }
    }
    false
}

fn position<T: Tree>(
    tree: &T,
    document: &UiDocument,
    nth: &Nth,
    id: &T::Id,
    work: &mut MatchWork,
) -> bool {
    let (before, count) = match &nth.of {
        None if nth.of_type => {
            let filter = type_filter(tag(&tree.facts(id)));
            tree.filtered(id, &filter)
        }
        None => tree.position(id),
        Some(filter) => {
            if !filter
                .iter()
                .any(|inner| compound(tree, document, &inner.compounds[0], id, work))
            {
                return false;
            }
            tree.filtered(id, filter)
        }
    };
    nth.matches(if nth.last { count - before } else { before + 1 })
}

/// A simple selector that reads only the element's own facts; also used to
/// evaluate `of S` filters on a single element.
pub fn own(facts: &Facts<'_>, simple: &Simple) -> bool {
    let node = &facts.document.nodes[facts.node];
    match simple {
        Simple::Type(name) => node.selector_tag.as_deref() == Some(name.as_str()),
        Simple::Root => facts.node == facts.document.root,
        Simple::Id(name) => facts.attributes.get("id").and_then(Value::as_str) == Some(name),
        Simple::Class(name) => facts
            .attributes
            .get("class")
            .and_then(Value::as_str)
            .is_some_and(|classes| classes.split_ascii_whitespace().any(|c| c == name)),
        Simple::Attribute { name, test } => match (facts.attributes.get(name), test) {
            (None | Some(Value::Null), _) => false,
            (Some(value), None)
                if !value.is_string()
                    && matches!(name.as_str(), "disabled" | "checked" | "selected") =>
            {
                crate::bind::truthy(value)
            }
            (Some(_), None) => true,
            (Some(actual), Some(test)) => test.test(&crate::bind::text(actual)),
        },
        Simple::State(state) => facts.states.contains(state),
        Simple::Custom(name) => facts
            .attributes
            .get(&format!("state:{name}"))
            .is_some_and(|v| crate::bind::truthy(v) && v.as_str() != Some("false")),
        Simple::Defined => true,
        Simple::Scope => facts.node == facts.document.root,
        Simple::Not(list) => !list
            .iter()
            .any(|s| s.compounds[0].0.iter().all(|x| own(facts, x))),
        Simple::Is(list) | Simple::Where(list) | Simple::Pseudo(list) => list
            .iter()
            .any(|s| s.compounds[0].0.iter().all(|x| own(facts, x))),
        Simple::Part(_)
        | Simple::Nth(_)
        | Simple::Only { .. }
        | Simple::Empty
        | Simple::Has(_)
        | Simple::Anchor
        | Simple::Lang(_)
        | Simple::Dir(_) => false,
    }
}

/// Whether an element's own facts match one `of S` filter list.
pub fn filter_matches(facts: &Facts<'_>, filter: &[Selector]) -> bool {
    filter
        .iter()
        .any(|s| s.compounds[0].0.iter().all(|simple| own(facts, simple)))
}

/// What structural rules of a set of documents can depend on; it bounds the
/// invalidation that sibling edits and fact changes cause.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Profile {
    /// Any structural rule at all.
    pub any: bool,
    /// `+` anywhere: an element depends on its previous sibling.
    pub next: bool,
    /// `~` anywhere: an element depends on every earlier sibling.
    pub subsequent: bool,
    /// Forward nth positions other than `:first-child` (`nth(0, 1)`).
    pub forward: bool,
    /// Backward nth positions other than `:last-child`.
    pub backward: bool,
    /// `:first-child`, `:last-child` or `:only-child`.
    pub ends: bool,
    /// `:empty`: a parent depends on its child count.
    pub empty: bool,
    /// A structural selector with a descendant or child combinator: an
    /// element's match depends on its ancestors' facts and positions.
    pub ancestors: bool,
    /// `:has()` anywhere: an element depends on its descendants' facts
    /// and order, so a change restyles the changed element's ancestors.
    pub has: bool,
    /// How many next siblings a `:has(+ …)` reaches; `usize::MAX` for
    /// `:has(~ …)`: a change restyles that many earlier siblings too.
    pub has_siblings: usize,
    /// The own facts of compounds with a `:has()` that are a selector's
    /// subject: an ancestor matching one restyles itself.
    pub has_anchors: Vec<Selector>,
    /// A `:has()` before the subject (`.a:has(.b) > .c`): when an anchor's
    /// match may flip, its descendants down to `depth` that can be the
    /// subject restyle, not the anchor.
    pub has_context: Vec<HasContext>,
    /// `:nth-of-type()` and the `-of-type` ends: positions are counted
    /// among siblings of one type ([`type_filter`], one per tag in
    /// [`Profile::filters`]).
    pub of_type: bool,
    /// Fact names (`class`, attribute names) read by structural selectors.
    pub attributes: BTreeSet<String>,
    pub states: BTreeSet<InteractionState>,
    /// Every state any rule of the document reads, structural or not:
    /// the states worth keeping on its elements.
    pub reads: BTreeSet<InteractionState>,
    /// Distinct `of S` filters, in first-seen order.
    pub filters: Vec<Vec<Selector>>,
    /// List items whose marker may show a number (an `<ol>`, or a
    /// `list-style` declaration): each reads its place among its `<li>`
    /// siblings, as `:nth-child(n of li)` does.
    pub ordinals: bool,
    /// Some `<li>` sets `value`: an item's number may come from an earlier one's.
    pub item_values: bool,
}

/// One `:has()` before a selector's subject ([`Profile::has_context`]).
#[derive(Clone, Debug, PartialEq)]
pub struct HasContext {
    /// The anchor compound's own facts (a one-compound selector).
    pub anchor: Selector,
    /// How far below the anchor the subject is; `usize::MAX` through a
    /// descendant combinator.
    pub depth: usize,
    /// The subject compound's own facts.
    pub subject: Selector,
}

fn has_in(simple: &Simple) -> bool {
    match simple {
        Simple::Has(_) => true,
        Simple::Not(l) | Simple::Is(l) | Simple::Where(l) | Simple::Pseudo(l) => l
            .iter()
            .flat_map(|s| s.compounds.iter().flat_map(|c| &c.0))
            .any(has_in),
        _ => false,
    }
}

impl Profile {
    pub fn add(&mut self, selector: &Selector) {
        if !selector.structural() {
            return;
        }
        self.any = true;
        let last = selector.compounds.len() - 1;
        for (i, compound) in selector.compounds.iter().enumerate() {
            if !compound.0.iter().any(has_in) {
                continue;
            }
            let own = |c: &Compound| Selector {
                source: selector.source.clone(),
                compounds: vec![Compound(
                    c.0.iter()
                        .filter(|s| {
                            !s.structural() && !matches!(s, Simple::Lang(_) | Simple::Dir(_))
                        })
                        .cloned()
                        .collect(),
                )],
                combinators: Vec::new(),
            };
            let steps = &selector.combinators[i..];
            if i == last
                || steps
                    .iter()
                    .any(|c| matches!(c, Combinator::Next | Combinator::Subsequent))
            {
                // The anchor is the subject, or its subject is beside the
                // anchor's subtree: the anchor restyles, and with it what
                // follows it.
                let anchor = if i == last {
                    own(compound)
                } else {
                    self.has_siblings = usize::MAX;
                    self.ancestors = true;
                    own(&Compound(Vec::new()))
                };
                if !self.has_anchors.contains(&anchor) {
                    self.has_anchors.push(anchor);
                }
                continue;
            }
            let context = HasContext {
                anchor: own(compound),
                depth: if steps.contains(&Combinator::Descendant) {
                    usize::MAX
                } else {
                    steps.len()
                },
                subject: own(&selector.compounds[last]),
            };
            if !self.has_context.contains(&context) {
                self.has_context.push(context);
            }
        }
        for combinator in &selector.combinators {
            match combinator {
                Combinator::Next => self.next = true,
                Combinator::Subsequent => self.subsequent = true,
                Combinator::Descendant | Combinator::Child => self.ancestors = true,
            }
        }
        for simple in selector.compounds.iter().flat_map(|c| &c.0) {
            self.simple(simple);
        }
    }

    /// Records one simple selector's reads; answers whether it is structural.
    fn simple(&mut self, simple: &Simple) -> bool {
        match simple {
            Simple::Class(_) => {
                self.attributes.insert("class".into());
                false
            }
            Simple::Id(_) => {
                self.attributes.insert("id".into());
                false
            }
            Simple::Attribute { name, .. } => {
                self.attributes.insert(name.clone());
                false
            }
            Simple::State(state) => {
                self.states.insert(*state);
                false
            }
            Simple::Nth(nth) => {
                self.of_type |= nth.of_type;
                let end = nth.a == 0 && nth.b == 1 && nth.of.is_none() && !nth.of_type;
                match (nth.last, end) {
                    (_, true) => self.ends = true,
                    (false, false) => self.forward = true,
                    (true, false) => self.backward = true,
                }
                if let Some(filter) = &nth.of {
                    for s in filter {
                        for simple in &s.compounds[0].0 {
                            self.simple(simple);
                        }
                    }
                    if !self.filters.contains(filter) {
                        self.filters.push(filter.clone());
                    }
                }
                true
            }
            Simple::Only { of_type } => {
                self.ends = true;
                if *of_type {
                    self.of_type = true;
                    self.forward = true;
                    self.backward = true;
                }
                true
            }
            Simple::Has(list) => {
                self.has = true;
                for relative in list {
                    if matches!(
                        relative.combinators[0],
                        Combinator::Next | Combinator::Subsequent
                    ) {
                        let reach = if relative.combinators.contains(&Combinator::Subsequent) {
                            usize::MAX
                        } else {
                            relative
                                .combinators
                                .iter()
                                .filter(|c| **c == Combinator::Next)
                                .count()
                        };
                        self.has_siblings = self.has_siblings.max(reach);
                    }
                    for simple in relative.compounds.iter().flat_map(|c| &c.0) {
                        self.simple(simple);
                    }
                }
                true
            }
            Simple::Lang(_) | Simple::Dir(_) => {
                self.attributes.insert(
                    if matches!(simple, Simple::Lang(_)) {
                        "lang"
                    } else {
                        "dir"
                    }
                    .into(),
                );
                self.ancestors = true;
                false
            }
            Simple::Custom(name) => {
                self.attributes.insert(format!("state:{name}"));
                false
            }
            Simple::Empty => {
                self.empty = true;
                true
            }
            Simple::Not(list) | Simple::Is(list) | Simple::Where(list) | Simple::Pseudo(list) => {
                // Nested complex selectors are anchored at this element; their
                // own sibling and ancestor reads count as this selector's.
                for selector in list {
                    if selector.structural() {
                        self.add(selector);
                    } else {
                        for simple in selector.compounds.iter().flat_map(|c| &c.0) {
                            self.simple(simple);
                        }
                    }
                }
                list.iter().any(Selector::structural)
            }
            Simple::Type(_)
            | Simple::Root
            | Simple::Part(_)
            | Simple::Anchor
            | Simple::Defined
            | Simple::Scope => false,
        }
    }

    pub fn of_document(document: &UiDocument) -> Self {
        fn states(e: &crate::bind::Expr, out: &mut BTreeSet<InteractionState>) {
            use crate::bind::Expr;
            match e {
                Expr::Interaction(s) | Expr::InteractionAt { state: s, .. } => {
                    out.insert(*s);
                }
                Expr::Not(a)
                | Expr::Neg(a)
                | Expr::Format(_, a)
                | Expr::HasToken(a, _)
                | Expr::AttrTest(a, _) => states(a, out),
                Expr::Lang(a, _) | Expr::Dir(a, _) => {
                    if let Some(a) = a {
                        states(a, out);
                    }
                }
                Expr::Bin(_, a, b) => {
                    states(a, out);
                    states(b, out);
                }
                Expr::If(c, a, b) => {
                    states(c, out);
                    states(a, out);
                    states(b, out);
                }
                Expr::Translate(k, args) => {
                    states(k, out);
                    for (_, a) in args {
                        states(a, out);
                    }
                }
                Expr::Lit(_) | Expr::Path(_) => {}
            }
        }
        let mut profile = Self::default();
        for node in &document.nodes {
            for rule in &node.rules {
                if let Some(selector) = &rule.structure {
                    profile.add(selector);
                }
                if let Some(condition) = &rule.condition {
                    states(condition, &mut profile.reads);
                }
            }
            for rule in &node.span_rules {
                if let Some(condition) = &rule.condition {
                    states(condition, &mut profile.reads);
                }
            }
        }
        profile.reads.extend(profile.states.iter().copied());
        if profile.of_type {
            let mut tags: Vec<&str> = document
                .nodes
                .iter()
                .filter_map(|n| n.selector_tag.as_deref())
                .collect();
            tags.sort_unstable();
            tags.dedup();
            for t in tags {
                let filter = type_filter(t);
                if !profile.filters.contains(&filter) {
                    profile.filters.push(filter);
                }
            }
        }
        let tag = |t: &str| {
            document
                .nodes
                .iter()
                .any(|n| n.selector_tag.as_deref() == Some(t))
        };
        if tag("li") && (tag("ol") || document.text.contains("list-style")) {
            profile.any = true;
            profile.ordinals = true;
            let items = super::lists::items();
            if !profile.filters.contains(&items) {
                profile.filters.push(items);
            }
            for name in ["value", "start", "reversed"] {
                profile.attributes.insert(name.into());
            }
            let bound = |name: &str| {
                document.nodes.iter().any(|n| {
                    n.selector_attrs
                        .iter()
                        .any(|(a, e)| a == name && !matches!(e, crate::bind::Expr::Lit(_)))
                })
            };
            // A list's bound `start` or `reversed` renumbers its items.
            profile.ancestors |= bound("start") || bound("reversed");
            profile.item_values = document.nodes.iter().any(|n| {
                n.selector_tag.as_deref() == Some("li")
                    && n.selector_attrs.iter().any(|(a, _)| a == "value")
            });
        }
        profile
    }

    pub fn merge(&mut self, other: &Self) {
        self.any |= other.any;
        self.next |= other.next;
        self.subsequent |= other.subsequent;
        self.forward |= other.forward;
        self.backward |= other.backward;
        self.ends |= other.ends;
        self.empty |= other.empty;
        self.has |= other.has;
        self.has_siblings = self.has_siblings.max(other.has_siblings);
        for a in &other.has_anchors {
            if !self.has_anchors.contains(a) {
                self.has_anchors.push(a.clone());
            }
        }
        for c in &other.has_context {
            if !self.has_context.contains(c) {
                self.has_context.push(c.clone());
            }
        }
        self.of_type |= other.of_type;
        self.ancestors |= other.ancestors;
        self.ordinals |= other.ordinals;
        self.item_values |= other.item_values;
        self.attributes.extend(other.attributes.iter().cloned());
        self.states.extend(other.states.iter().copied());
        self.reads.extend(other.reads.iter().copied());
        for filter in &other.filters {
            if !self.filters.contains(filter) {
                self.filters.push(filter.clone());
            }
        }
    }
}
