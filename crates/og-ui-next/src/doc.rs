//! A document's styles compiled per element by the cook. The parser has
//! already scoped each CSS rule onto the elements it matches (descendant
//! and attribute selectors included), with the reactive condition a
//! dynamic attribute or a state makes it hold under. Here each element's
//! rules become flat step spans: an unconditional base, then each
//! conditional span with the index of its condition, in cascade order.
//! An element's custom properties (`--slot: 48px`) resolve at compile time
//! into the tokens its subtree reads. At runtime a record is the base
//! folded with the spans whose condition bits are set: no matching, no
//! substitution, no strings.

use crate::style::{self, Record, Step};
use crate::tokens::Tokens;
use bytemuck::{Pod, Zeroable};
use og_ui::doc::{NodeKind, UiDocument};

/// One element's compiled styles: spans of [`DocStyle::spans`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct NodeStyle {
    /// Its document node's index.
    pub node: u32,
    /// Its spans: the first is the base (condition `u32::MAX`).
    pub spans: [u32; 2],
}

/// A run of steps that applies while its condition holds.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct Span {
    /// The condition's index in [`DocStyle::conditions`]; `u32::MAX`: always.
    pub condition: u32,
    pub steps: [u32; 2],
}

/// Why a rule or declaration was not compiled.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Skip {
    /// A sibling or structural selector (`:nth-child`, `+`, `~`).
    Structural,
    /// Under `@media` or `@container`.
    Query,
    /// A property or value the compiler does not take.
    Declaration(String),
}

#[derive(Clone, Debug, Default)]
pub struct DocStyle {
    pub nodes: Vec<NodeStyle>,
    pub spans: Vec<Span>,
    pub steps: Vec<Step>,
    /// The rules' conditions, as the document's nodes give them: the
    /// bindings evaluate them and set an element's bits.
    pub conditions: Vec<og_ui::bind::Expr>,
    pub skipped: Vec<Skip>,
}

const ALWAYS: u32 = u32::MAX;

impl DocStyle {
    /// Compiles `doc`'s element styles against `tokens`.
    pub fn compile(doc: &UiDocument, tokens: &Tokens) -> DocStyle {
        let mut out = DocStyle::default();
        let mut local = Vec::new();
        out.visit(doc, doc.root, tokens, &mut local);
        out
    }

    fn visit(
        &mut self,
        doc: &UiDocument,
        i: usize,
        tokens: &Tokens,
        local: &mut Vec<(Box<str>, Box<str>)>,
    ) {
        let Some(n) = doc.nodes.get(i) else { return };
        let mark = local.len();
        if matches!(n.kind, NodeKind::Element(_)) {
            // Its own custom properties first: its declarations read them.
            for rule in n
                .rules
                .iter()
                .filter(|r| r.condition.is_none() && r.structure.is_none())
            {
                for d in &rule.declarations {
                    if d.name.starts_with("--")
                        && let Some(v) = tokens.substitute_with(&d.source, local)
                    {
                        local.push((d.name.as_str().into(), v.into()));
                    }
                }
            }
            let first = self.spans.len() as u32;
            let base = self.steps.len() as u32;
            for rule in n.rules.iter().filter(|r| r.condition.is_none()) {
                self.rule(rule, tokens, local);
            }
            self.spans.push(Span {
                condition: ALWAYS,
                steps: [base, self.steps.len() as u32 - base],
            });
            for rule in n.rules.iter().filter(|r| r.condition.is_some()) {
                let from = self.steps.len() as u32;
                if !self.rule(rule, tokens, local) {
                    continue;
                }
                self.conditions.push(rule.condition.clone().unwrap());
                self.spans.push(Span {
                    condition: self.conditions.len() as u32 - 1,
                    steps: [from, self.steps.len() as u32 - from],
                });
            }
            self.nodes.push(NodeStyle {
                node: i as u32,
                spans: [first, self.spans.len() as u32 - first],
            });
        }
        for &c in &n.children {
            self.visit(doc, c, tokens, local);
        }
        local.truncate(mark);
    }

    /// Compiles one rule's declarations; false when the rule is not taken.
    fn rule(
        &mut self,
        rule: &og_ui::doc::StyleRule,
        tokens: &Tokens,
        local: &[(Box<str>, Box<str>)],
    ) -> bool {
        if rule.structure.is_some() || !rule.instance_states.is_empty() {
            self.skipped.push(Skip::Structural);
            return false;
        }
        if !rule.query.container.is_empty()
            || !matches!(rule.query.media, og_ui::css::media::Query::All)
        {
            self.skipped.push(Skip::Query);
            return false;
        }
        for d in &rule.declarations {
            if d.name.starts_with("--") {
                continue;
            }
            let taken = tokens
                .substitute_with(&d.source, local)
                .is_some_and(|v| style::expand(&mut self.steps, &d.name, v.trim()));
            if !taken {
                self.skipped.push(Skip::Declaration(d.name.clone()));
            }
        }
        true
    }

    /// The record of element `k` (an index of [`DocStyle::nodes`]) under
    /// the conditions `holds` answers.
    pub fn record(&self, k: usize, holds: impl Fn(u32) -> bool) -> Record {
        let mut r = Record::default();
        let [at, n] = self.nodes[k].spans;
        for span in &self.spans[at as usize..(at + n) as usize] {
            if span.condition == ALWAYS || holds(span.condition) {
                let [s, m] = span.steps;
                for step in &self.steps[s as usize..(s + m) as usize] {
                    style::fold(&mut r, step);
                }
            }
        }
        r
    }
}
