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
    type Id: Clone;
    fn facts(&self, id: &Self::Id) -> Facts<'_>;
    fn parent(&self, id: &Self::Id) -> Option<Self::Id>;
    /// The immediately preceding element sibling.
    fn previous(&self, id: &Self::Id) -> Option<Self::Id>;
    /// The 0-based index among the element's siblings and their count.
    fn position(&self, id: &Self::Id) -> (usize, usize);
    /// Siblings before the element that match `filter`, and all that do.
    fn filtered(&self, id: &Self::Id, filter: &[Selector]) -> (usize, usize);
    fn has_children(&self, id: &Self::Id) -> bool;
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
        work,
    )
}

fn chain<T: Tree>(
    tree: &T,
    document: &UiDocument,
    selector: &Selector,
    part: usize,
    id: &T::Id,
    work: &mut MatchWork,
) -> bool {
    if !compound(tree, document, &selector.compounds[part], id, work) {
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
        if chain(tree, document, selector, part - 1, &candidate, work) {
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
    let facts = tree.facts(id);
    if facts.document.name != document.name {
        return false;
    }
    compound.0.iter().all(|simple| match simple {
        Simple::Nth(nth) => position(tree, document, nth, id, work),
        Simple::OnlyChild => tree.position(id).1 == 1,
        Simple::Empty => facts.text.is_empty() && !tree.has_children(id),
        Simple::Not(list) => !list
            .iter()
            .any(|inner| matches(tree, document, inner, id, work)),
        Simple::Is(list) | Simple::Where(list) => list
            .iter()
            .any(|inner| matches(tree, document, inner, id, work)),
        simple => own(&facts, simple),
    })
}

fn position<T: Tree>(
    tree: &T,
    document: &UiDocument,
    nth: &Nth,
    id: &T::Id,
    work: &mut MatchWork,
) -> bool {
    let (before, count) = match &nth.of {
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
        Simple::Attribute { name, value } => match (facts.attributes.get(name), value) {
            (None | Some(Value::Null), _) => false,
            (Some(value), None)
                if !value.is_string()
                    && matches!(name.as_str(), "disabled" | "checked" | "selected") =>
            {
                crate::bind::truthy(value)
            }
            (Some(_), None) => true,
            (Some(actual), Some(expected)) => crate::bind::text(actual) == *expected,
        },
        Simple::State(state) => facts.states.contains(state),
        Simple::Not(list) => !list
            .iter()
            .any(|s| s.compounds[0].0.iter().all(|x| own(facts, x))),
        Simple::Is(list) | Simple::Where(list) => list
            .iter()
            .any(|s| s.compounds[0].0.iter().all(|x| own(facts, x))),
        Simple::Part(_) | Simple::Nth(_) | Simple::OnlyChild | Simple::Empty => false,
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
    /// Fact names (`class`, attribute names) read by structural selectors.
    pub attributes: BTreeSet<String>,
    pub states: BTreeSet<InteractionState>,
    /// Distinct `of S` filters, in first-seen order.
    pub filters: Vec<Vec<Selector>>,
    /// List items whose marker may show a number (an `<ol>`, or a
    /// `list-style` declaration): each reads its place among its `<li>`
    /// siblings, as `:nth-child(n of li)` does.
    pub ordinals: bool,
    /// Some `<li>` sets `value`: an item's number may come from an earlier one's.
    pub item_values: bool,
}

impl Profile {
    pub fn add(&mut self, selector: &Selector) {
        if !selector.structural() {
            return;
        }
        self.any = true;
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
                let end = nth.a == 0 && nth.b == 1 && nth.of.is_none();
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
            Simple::OnlyChild => {
                self.ends = true;
                true
            }
            Simple::Empty => {
                self.empty = true;
                true
            }
            Simple::Not(list) | Simple::Is(list) | Simple::Where(list) => {
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
            Simple::Type(_) | Simple::Root | Simple::Part(_) => false,
        }
    }

    pub fn of_document(document: &UiDocument) -> Self {
        let mut profile = Self::default();
        for node in &document.nodes {
            for rule in &node.rules {
                if let Some(selector) = &rule.structure {
                    profile.add(selector);
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
        self.ancestors |= other.ancestors;
        self.ordinals |= other.ordinals;
        self.item_values |= other.item_values;
        self.attributes.extend(other.attributes.iter().cloned());
        self.states.extend(other.states.iter().copied());
        for filter in &other.filters {
            if !self.filters.contains(filter) {
                self.filters.push(filter.clone());
            }
        }
    }
}
