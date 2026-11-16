//! Runtime matching of structural selectors against an instantiated element
//! tree, by the `selectors` crate over [`DocElement`]: retained mounts read
//! the logical [`super::topology::Topology`]. Matching reads selector facts
//! only (tag, attributes, classes, states, text); it never evaluates
//! viewmodel expressions.
//!
//! A document's rules see its own tree, as a page's styles see its light
//! tree: an instantiated component is a shadow root ([`DocElement`]), so its
//! elements are past the document's ancestors, and content the document
//! passes into it keeps the document's elements as its ancestors. Every
//! instantiated element counts as a sibling; one of another document
//! matches nothing of the rule. A global sheet matches the whole tree.
use super::UiDocument;
use super::selectors::{
    Atom, Combinator, Component, Inner, OgImpl, PseudoClass, Selector, compound_of, compounds,
    nested, structural, structural_component,
};
use crate::bind::InteractionState;
use cssparser::ToCss;
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::bloom::BloomFilter;
use selectors::context::{
    MatchingContext, MatchingForInvalidation, MatchingMode, NeedsSelectorFlags, QuirksMode,
    SelectorCaches,
};
use selectors::matching::{ElementSelectorFlags, matches_selector};
use selectors::{Element, OpaqueElement};
use serde_json::Value;
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

/// The selector inputs of one instantiated element.
#[derive(Clone, Copy)]
pub struct Facts<'a> {
    pub scope: &'a str,
    pub tag: &'a str,
    /// The exposed part being styled, not the host's `part` attribute.
    pub part: Option<&'a str>,
    /// The control's own part beside a named one ([`crate::dress::PartRef::own`]).
    pub own_part: Option<&'a str>,
    pub root: bool,
    pub attributes: &'a BTreeMap<String, Value>,
    pub states: &'a BTreeSet<InteractionState>,
    pub text: &'a str,
}

/// An instantiated element tree in logical order. Ids it answers are its
/// own, so one element is one address ([`Tree::canonical`]).
pub trait Tree {
    type Id: PartialEq + std::fmt::Debug;
    fn facts(&self, id: &Self::Id) -> Facts<'_>;
    fn parent(&self, id: &Self::Id) -> Option<&Self::Id>;
    /// The immediately preceding element sibling.
    fn previous(&self, id: &Self::Id) -> Option<&Self::Id>;
    /// The immediately following element sibling.
    fn following(&self, id: &Self::Id) -> Option<&Self::Id>;
    fn first_child(&self, id: &Self::Id) -> Option<&Self::Id>;
    /// Siblings before the element that match `filter`, and all that do.
    fn filtered(&self, id: &Self::Id, filter: &[Selector]) -> (usize, usize);
    /// The tree's own id for `id`.
    fn canonical(&self, id: &Self::Id) -> Option<&Self::Id>;
    /// The host a part is exposed by.
    fn part_host(&self, _id: &Self::Id) -> Option<&Self::Id> {
        None
    }
}

/// Work done by structural matching, for bounded-cost tests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MatchWork {
    /// Structural rule selectors evaluated against an element.
    pub selectors: usize,
    /// Sibling and ancestor elements visited while matching them.
    pub candidates: usize,
}

/// What one match reads beside the tree.
struct Context<'a> {
    /// The rule's document; `None` for a global sheet's.
    scope: Option<&'a str>,
    candidates: Cell<usize>,
}

/// An element of a [`Tree`] as the `selectors` crate matches it.
pub struct DocElement<'a, T: Tree> {
    tree: &'a T,
    id: &'a T::Id,
    facts: Facts<'a>,
    /// Of another document than the rule's: it matches nothing.
    foreign: bool,
    context: &'a Context<'a>,
}

impl<T: Tree> Clone for DocElement<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Tree> Copy for DocElement<'_, T> {}

impl<T: Tree> std::fmt::Debug for DocElement<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{} {:?}>", self.facts.tag, self.id)
    }
}

impl<'a, T: Tree> DocElement<'a, T> {
    fn new(tree: &'a T, id: &'a T::Id, facts: Facts<'a>, context: &'a Context<'a>) -> Self {
        Self {
            tree,
            id,
            facts,
            foreign: context.scope.is_some_and(|s| s != facts.scope),
            context,
        }
    }

    fn at(&self, id: &'a T::Id) -> Self {
        let count = &self.context.candidates;
        count.set(count.get() + 1);
        Self::new(self.tree, id, self.tree.facts(id), self.context)
    }

    fn foreign(&self) -> bool {
        self.foreign
    }

    /// Whether the element is a component's root under its host, `parent`
    /// in the instantiated tree: the first child of a shadow root.
    fn shadow_child_of(&self, parent: &Facts<'_>) -> bool {
        self.context.scope.is_some() && self.facts.root && parent.scope != self.facts.scope
    }

    fn shadow_child(&self) -> bool {
        self.tree
            .parent(self.id)
            .is_some_and(|p| self.shadow_child_of(&self.tree.facts(p)))
    }

    /// The nearest `name` attribute of the element or an ancestor, as text.
    fn inherited(&self, name: &str) -> Option<String> {
        let mut at = Some(self.id);
        while let Some(node) = at {
            if let Some(value) = self.tree.facts(node).attributes.get(name)
                && !value.is_null()
            {
                return Some(crate::bind::text(value));
            }
            at = self.tree.parent(node);
        }
        None
    }
}

impl<'a, T: Tree> Element for DocElement<'a, T> {
    type Impl = OgImpl;

    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(self.id)
    }

    /// The parent in the element's own tree: none for a component's root
    /// (its parent is a shadow root); for content a document passed into a
    /// component, its nearest ancestor of that document.
    fn parent_element(&self) -> Option<Self> {
        let mut parent = self.at(self.tree.parent(self.id)?);
        if self.context.scope.is_none() || parent.facts.scope == self.facts.scope {
            return Some(parent);
        }
        if self.shadow_child_of(&parent.facts) {
            return None;
        }
        loop {
            parent = parent.at(self.tree.parent(parent.id)?);
            if parent.facts.scope == self.facts.scope {
                return Some(parent);
            }
        }
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        self.shadow_child()
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        if let Some(host) = self.tree.part_host(self.id) {
            return Some(self.at(host));
        }
        self.context.scope?;
        let mut at = self.id;
        loop {
            let parent = self.tree.parent(at)?;
            if self.tree.facts(at).root && self.tree.facts(parent).scope != self.facts.scope {
                return Some(self.at(parent));
            }
            at = parent;
        }
    }

    fn is_pseudo_element(&self) -> bool {
        false
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        self.tree.previous(self.id).map(|id| self.at(id))
    }

    fn next_sibling_element(&self) -> Option<Self> {
        self.tree.following(self.id).map(|id| self.at(id))
    }

    fn first_element_child(&self) -> Option<Self> {
        self.tree.first_child(self.id).map(|id| self.at(id))
    }

    /// Attribute values HTML compares caselessly do so here too (HTML
    /// §4.16.2), unless the selector has the `s` flag.
    fn is_html_element_in_html_document(&self) -> bool {
        true
    }

    fn has_local_name(&self, name: &str) -> bool {
        !self.foreign() && (self.facts.tag == name || self.facts.tag.eq_ignore_ascii_case(name))
    }

    fn has_namespace(&self, _: &str) -> bool {
        true
    }

    fn is_same_type(&self, other: &Self) -> bool {
        self.facts.tag == other.facts.tag
    }

    fn attr_matches(
        &self,
        _: &NamespaceConstraint<&Atom>,
        name: &Atom,
        operation: &AttrSelectorOperation<&Atom>,
    ) -> bool {
        if self.foreign() {
            return false;
        }
        match (self.facts.attributes.get(&name.0), operation) {
            (None | Some(Value::Null), _) => false,
            // A bound boolean attribute is present while true.
            (Some(value), AttrSelectorOperation::Exists)
                if !value.is_string()
                    && matches!(name.0.as_str(), "disabled" | "checked" | "selected") =>
            {
                crate::bind::truthy(value)
            }
            (Some(_), AttrSelectorOperation::Exists) => true,
            (Some(value), operation) => operation.eval_str(&crate::bind::text(value)),
        }
    }

    fn match_non_ts_pseudo_class(&self, pc: &PseudoClass, _: &mut MatchingContext<OgImpl>) -> bool {
        if self.foreign() {
            return false;
        }
        match pc {
            PseudoClass::State(state) => self.facts.states.contains(state),
            PseudoClass::Lang(ranges) => {
                let tag = self.inherited("lang");
                let tag = tag.as_deref().unwrap_or("en");
                ranges.iter().any(|r| crate::bind::lang_matches(r, tag))
            }
            PseudoClass::Dir(rtl) => (self.inherited("dir").as_deref() == Some("rtl")) == *rtl,
            PseudoClass::Custom(name) => self
                .facts
                .attributes
                .get(&format!("state:{}", name.0))
                .is_some_and(|v| crate::bind::truthy(v) && v.as_str() != Some("false")),
            PseudoClass::Defined => true,
            PseudoClass::AnyLink => self.is_link(),
        }
    }

    fn match_pseudo_element(
        &self,
        pe: &super::selectors::PseudoElement,
        _: &mut MatchingContext<OgImpl>,
    ) -> bool {
        match *pe {}
    }

    fn apply_selector_flags(&self, _: ElementSelectorFlags) {}

    fn is_link(&self) -> bool {
        !self.foreign()
            && matches!(self.facts.tag, "a" | "area")
            && self
                .facts
                .attributes
                .get("href")
                .is_some_and(|v| !v.is_null())
    }

    fn is_html_slot_element(&self) -> bool {
        false
    }

    fn has_id(&self, id: &Atom, _: CaseSensitivity) -> bool {
        !self.foreign() && self.facts.attributes.get("id").and_then(Value::as_str) == Some(&id.0)
    }

    fn has_class(&self, name: &Atom, _: CaseSensitivity) -> bool {
        !self.foreign()
            && self
                .facts
                .attributes
                .get("class")
                .and_then(Value::as_str)
                .is_some_and(|classes| classes.split_ascii_whitespace().any(|c| c == name.0))
    }

    fn has_custom_state(&self, _: &Atom) -> bool {
        false
    }

    fn imported_part(&self, _: &Atom) -> Option<Atom> {
        None
    }

    /// A part list names the element by each of its names (CSS Shadow
    /// Parts §4.1).
    fn is_part(&self, name: &Atom) -> bool {
        !self.foreign()
            && (self
                .facts
                .part
                .is_some_and(|p| p.split_ascii_whitespace().any(|n| n == name.0))
                || self.facts.own_part == Some(name.0.as_str()))
    }

    fn is_empty(&self) -> bool {
        !self.foreign() && self.facts.text.is_empty() && self.tree.first_child(self.id).is_none()
    }

    fn is_root(&self) -> bool {
        !self.foreign() && self.facts.root
    }

    fn add_element_unique_hashes(&self, _: &mut BloomFilter) -> bool {
        false
    }
}

/// Matches selectors against one element of a tree: what every match reads
/// of the element, and the crate's caches, are set up once.
pub struct Matcher<'a, T: Tree> {
    tree: &'a T,
    subject: Option<(&'a T::Id, Facts<'a>)>,
    /// The rule's document, for its own rules; none for a global sheet's.
    scoped: Context<'a>,
    global: Context<'a>,
    caches: SelectorCaches,
}

impl<'a, T: Tree> Matcher<'a, T> {
    /// Document sheets supply their scope; global sheets have no scope
    /// restriction.
    pub fn new(tree: &'a T, scope: Option<&'a str>, id: &T::Id) -> Self {
        Self {
            tree,
            subject: tree.canonical(id).map(|id| (id, tree.facts(id))),
            scoped: Context {
                scope,
                candidates: Cell::new(0),
            },
            global: Context {
                scope: None,
                candidates: Cell::new(0),
            },
            caches: SelectorCaches::default(),
        }
    }

    /// Whether the element is the subject of `selector`.
    pub fn matches(&mut self, selector: &Selector, work: &mut MatchWork) -> bool {
        work.selectors += 1;
        let Some((id, facts)) = self.subject else {
            return false;
        };
        let context = if selector.global {
            &self.global
        } else {
            &self.scoped
        };
        context.candidates.set(0);
        let mut matching = MatchingContext::new(
            MatchingMode::Normal,
            None,
            &mut self.caches,
            QuirksMode::NoQuirks,
            NeedsSelectorFlags::No,
            MatchingForInvalidation::No,
        );
        let element = DocElement::new(self.tree, id, facts, context);
        let hit = matches_selector(&selector.inner, 0, None, element, &mut matching);
        work.candidates += context.candidates.get();
        hit
    }
}

/// One element's facts as a tree of its own: a part is exposed by a host
/// that has the same facts.
struct Lone<'f>(Facts<'f>);

#[derive(Debug, PartialEq)]
enum LoneNode {
    Part,
    Host,
}

static PART: LoneNode = LoneNode::Part;
static HOST: LoneNode = LoneNode::Host;

impl Tree for Lone<'_> {
    type Id = LoneNode;

    fn facts(&self, id: &LoneNode) -> Facts<'_> {
        match id {
            LoneNode::Part => self.0,
            LoneNode::Host => Facts {
                part: None,
                own_part: None,
                ..self.0
            },
        }
    }

    fn parent(&self, _: &LoneNode) -> Option<&LoneNode> {
        None
    }

    fn previous(&self, _: &LoneNode) -> Option<&LoneNode> {
        None
    }

    fn following(&self, _: &LoneNode) -> Option<&LoneNode> {
        None
    }

    fn first_child(&self, _: &LoneNode) -> Option<&LoneNode> {
        None
    }

    fn filtered(&self, _: &LoneNode, _: &[Selector]) -> (usize, usize) {
        (0, 0)
    }

    fn canonical(&self, id: &LoneNode) -> Option<&LoneNode> {
        Some(match id {
            LoneNode::Part => &PART,
            LoneNode::Host => &HOST,
        })
    }

    fn part_host(&self, id: &LoneNode) -> Option<&LoneNode> {
        (*id == LoneNode::Part).then_some(&HOST)
    }
}

/// Whether an element's own facts match `selector`: its part, when the
/// facts name one, under a host with the same facts.
pub fn matches_facts(facts: &Facts<'_>, selector: &Selector) -> bool {
    let subject = if facts.part.is_some() { &PART } else { &HOST };
    let lone = Lone(*facts);
    Matcher::new(&lone, None, subject).matches(selector, &mut MatchWork::default())
}

/// Whether an element's own facts match one `of S` filter list.
pub fn filter_matches(facts: &Facts<'_>, filter: &[Selector]) -> bool {
    filter.iter().any(|s| matches_facts(facts, s))
}

/// The `of S` filter `:nth-of-type()` counts with: the element's own type.
pub fn type_filter(tag: &str) -> Option<Vec<Selector>> {
    Selector::parse_list(tag).ok()
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

fn has_in(c: &Component) -> bool {
    matches!(c, Component::Has(_))
        || nested(c)
            .iter()
            .any(|s| s.iter_raw_parse_order_from(0).any(has_in))
}

/// The relative selector's combinators, from the anchor out.
fn relative_steps(relative: &Inner) -> Vec<Combinator> {
    compounds(relative).1
}

impl Profile {
    pub fn add(&mut self, selector: &Selector) {
        self.add_inner(&selector.inner, selector.global);
    }

    fn add_inner(&mut self, selector: &Inner, global: bool) {
        if !structural(selector) && !global {
            return;
        }
        self.any = true;
        let (compounds, combinators) = compounds(selector);
        let last = compounds.len() - 1;
        let own = |c: &[&Component]| {
            compound_of(c.iter().copied().filter(|c| {
                !structural_component(c)
                    && !matches!(
                        c,
                        Component::NonTSPseudoClass(PseudoClass::Lang(_) | PseudoClass::Dir(_))
                    )
            }))
        };
        for (i, compound) in compounds.iter().enumerate() {
            if !compound.iter().any(|c| has_in(c)) {
                continue;
            }
            let steps = &combinators[i..];
            if i == last || steps.iter().any(Combinator::is_sibling) {
                // The anchor is the subject, or its subject is beside the
                // anchor's subtree: the anchor restyles, and with it what
                // follows it.
                let anchor = if i == last {
                    own(compound)
                } else {
                    self.has_siblings = usize::MAX;
                    self.ancestors = true;
                    own(&[])
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
                subject: own(&compounds[last]),
            };
            if !self.has_context.contains(&context) {
                self.has_context.push(context);
            }
        }
        for combinator in &combinators {
            match combinator {
                Combinator::NextSibling => self.next = true,
                Combinator::LaterSibling => self.subsequent = true,
                _ => self.ancestors = true,
            }
        }
        for c in compounds.iter().flatten() {
            self.component(c);
        }
    }

    /// Records one simple selector's reads.
    fn component(&mut self, c: &Component) {
        match c {
            Component::Class(_) => {
                self.attributes.insert("class".into());
                self.attributes.insert("variant".into());
            }
            Component::ID(_) => {
                self.attributes.insert("id".into());
            }
            Component::AttributeInNoNamespaceExists { .. }
            | Component::AttributeInNoNamespace { .. }
            | Component::AttributeOther(_) => {
                if let Some((name, _)) = super::selectors::attribute(c) {
                    self.attributes.insert(name.into());
                }
            }
            Component::NonTSPseudoClass(pc) => match pc {
                PseudoClass::State(state) => {
                    self.states.insert(*state);
                }
                PseudoClass::Lang(_) | PseudoClass::Dir(_) => {
                    self.attributes.insert(
                        if matches!(pc, PseudoClass::Lang(_)) {
                            "lang"
                        } else {
                            "dir"
                        }
                        .into(),
                    );
                    self.ancestors = true;
                }
                PseudoClass::Custom(name) => {
                    self.attributes.insert(format!("state:{}", name.0));
                }
                PseudoClass::AnyLink => {
                    self.attributes.insert("href".into());
                }
                PseudoClass::Defined => {}
            },
            Component::Nth(nth) => self.nth(nth, false),
            Component::NthOf(nth) => {
                self.nth(nth.nth_data(), true);
                for s in nth.selectors() {
                    for c in s.iter_raw_parse_order_from(0) {
                        self.component(c);
                    }
                }
                let filter: Vec<Selector> = nth
                    .selectors()
                    .iter()
                    .map(|s| Selector {
                        source: s.to_css_string(),
                        global: false,
                        inner: s.clone(),
                        span: 0,
                        local: None,
                    })
                    .collect();
                if !self.filters.contains(&filter) {
                    self.filters.push(filter);
                }
            }
            Component::Has(list) => {
                self.has = true;
                for relative in list.iter() {
                    let steps = relative_steps(&relative.selector);
                    if steps.first().is_some_and(Combinator::is_sibling) {
                        let reach = if steps.contains(&Combinator::LaterSibling) {
                            usize::MAX
                        } else {
                            steps
                                .iter()
                                .filter(|c| **c == Combinator::NextSibling)
                                .count()
                        };
                        self.has_siblings = self.has_siblings.max(reach);
                    }
                    for c in relative.selector.iter_raw_parse_order_from(0) {
                        self.component(c);
                    }
                }
            }
            Component::Empty => self.empty = true,
            Component::Negation(_) | Component::Is(_) | Component::Where(_) => {
                // Nested complex selectors are anchored at this element; their
                // own sibling and ancestor reads count as this selector's.
                for selector in nested(c) {
                    if structural(selector) {
                        self.add_inner(selector, false);
                    } else {
                        for c in selector.iter_raw_parse_order_from(0) {
                            self.component(c);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn nth(&mut self, nth: &selectors::parser::NthSelectorData, of: bool) {
        let of_type = nth.ty.is_of_type();
        self.of_type |= of_type;
        if nth.ty.is_only() {
            self.ends = true;
            if of_type {
                self.forward = true;
                self.backward = true;
            }
            return;
        }
        let end = nth.an_plus_b.0 == 0 && nth.an_plus_b.1 == 1 && !of && !of_type;
        match (nth.ty.is_from_end(), end) {
            (_, true) => self.ends = true,
            (false, false) => self.forward = true,
            (true, false) => self.backward = true,
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
                | Expr::Numbers(a)
                | Expr::HasToken(a, _)
                | Expr::AttrTest(a, _) => states(a, out),
                Expr::Lang(a, _) | Expr::Dir(a, _) => {
                    if let Some(a) = a {
                        states(a, out);
                    }
                }
                Expr::Bin(_, a, b) | Expr::Index(a, b) => {
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
            for filter in tags.into_iter().filter_map(type_filter) {
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
