//! A document's styles compiled per element by the cook. The parser has
//! already scoped each CSS rule onto the elements it matches (descendant
//! and attribute selectors included), with the reactive condition a
//! dynamic attribute or a state makes it hold under. Here each element's
//! rules become flat step spans: an unconditional base, then each
//! conditional span with the index of its condition, in cascade order.
//! An element's custom properties (`--slot: 48px`) resolve at compile time
//! into the tokens its subtree reads. Beneath its rules lie its layout
//! fields ([`prim::Fields`]) and its surface's record: the style's
//! surfaces sheet read under its `part`, `variant` and `tier`
//! (`.frame.hud:hover`, `.cell.rare`).
//! At runtime a record is the element's spans folded in cascade order,
//! each while its condition holds: no matching, no substitution, no
//! strings.

use crate::prim;
use crate::style::{self, Record, Sheet, State, Step, Under};
use crate::tokens::Tokens;
use bytemuck::{Pod, Zeroable};
use og_ui::doc::{NodeKind, UiDocument};

/// One element's compiled styles: spans of [`DocStyle::spans`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct NodeStyle {
    /// Its document node's index.
    pub node: u32,
    /// Its spans, in cascade order.
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
    /// A layout attribute with a value the compiler does not take.
    Field(prim::Unknown),
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

/// The attributes whose values name a surface (`.cell.hud.rare.round`).
const SURFACE_NAMES: [&str; 7] = ["part", "variant", "tier", "tone", "shape", "team", "trend"];

/// What the compile reads beside the document.
struct Cx<'a> {
    tokens: &'a Tokens,
    surfaces: &'a Sheet,
    scale: prim::Scale,
}

impl DocStyle {
    /// Compiles `doc`'s element styles against `tokens` and the style's
    /// `surfaces`.
    pub fn compile(doc: &UiDocument, tokens: &Tokens, surfaces: &Sheet) -> DocStyle {
        let mut out = DocStyle::default();
        let mut local = Vec::new();
        let cx = Cx {
            tokens,
            surfaces,
            scale: prim::Scale::of(tokens),
        };
        out.visit(doc, doc.root, &cx, &mut local);
        out
    }

    /// Closes the steps from `from` into a span under `condition`.
    fn span(&mut self, condition: u32, from: u32) {
        let n = self.steps.len() as u32 - from;
        if n > 0 {
            self.spans.push(Span {
                condition,
                steps: [from, n],
            });
        }
    }

    /// The element's layout fields and surface record.
    fn own(&mut self, n: &og_ui::doc::DocNode, cx: &Cx) {
        use og_ui::bind::{Expr, InteractionState as I, Op};
        let mut fields = prim::Fields::default();
        // Its surface names written as literals, and the bindings that
        // pick one (`variant={bare ? 'bare' : 'hud'}`, `tier={rarity}`).
        let mut names: Vec<u32> = Vec::new();
        let mut bound: Vec<&Expr> = Vec::new();
        // On SVG `fill` is its paint, not the layout field.
        let svg = matches!(n.kind, NodeKind::Element(k) if k.svg());
        for (name, e) in &n.selector_attrs {
            let surface = SURFACE_NAMES.contains(&name.as_str());
            let text = match e {
                Expr::Lit(serde_json::Value::String(s)) => s.as_str(),
                Expr::Lit(serde_json::Value::Bool(true)) => "true",
                Expr::Lit(serde_json::Value::Bool(false)) => "false",
                Expr::Lit(_) => continue,
                e => {
                    if surface {
                        bound.push(e);
                    }
                    continue;
                }
            };
            match name.as_str() {
                _ if surface => names.extend(
                    text.split_ascii_whitespace()
                        .filter_map(|v| cx.surfaces.name(&format!(".{v}"))),
                ),
                "fill" if svg => {}
                name => {
                    if let Err(u) = fields.set(name, text) {
                        self.skipped.push(Skip::Field(u));
                    }
                }
            }
        }
        let from = self.steps.len() as u32;
        fields.steps(&cx.scale, &mut self.steps);
        self.span(ALWAYS, from);
        if names.is_empty() && bound.is_empty() {
            return;
        }
        // The rules that apply, each with its condition: a name it needs
        // that is not written holds while one of the bindings gives it.
        let mut taken: Vec<(usize, Option<Expr>, &[Step])> = Vec::new();
        cx.surfaces.rules(|needs, under, steps| {
            let mut cond: Option<Expr> = None;
            let and = |c: &mut Option<Expr>, e: Expr| {
                *c = Some(match c.take() {
                    None => e,
                    Some(a) => Expr::Bin(Op::And, Box::new(a), Box::new(e)),
                })
            };
            for &need in needs {
                if names.contains(&need) {
                    continue;
                }
                let value = &cx.surfaces.name_of(need)[1..];
                let any = bound
                    .iter()
                    .map(|&e| {
                        Expr::Bin(
                            Op::Eq,
                            Box::new(e.clone()),
                            Box::new(Expr::Lit(value.into())),
                        )
                    })
                    .reduce(|a, b| Expr::Bin(Op::Or, Box::new(a), Box::new(b)));
                let Some(any) = any else { return };
                and(&mut cond, any);
            }
            let state = match under {
                Under::State(State::Base) => None,
                Under::State(State::Hover) => Some(Expr::Interaction(I::Hover)),
                Under::State(State::Active) => Some(Expr::Interaction(I::Active)),
                Under::State(State::Focus) => Some(Expr::Interaction(I::FocusVisible)),
                Under::State(State::Disabled) => Some(Expr::Interaction(I::Disabled)),
                // A custom state holds while the element's binding of it
                // is set and not "false", as the parser reads `:state()`.
                Under::Custom(name) => {
                    let key = format!("state:{name}");
                    let Some((_, e)) = n.selector_attrs.iter().find(|(k, _)| *k == key) else {
                        return;
                    };
                    let not_false = Expr::Bin(
                        Op::Ne,
                        Box::new(e.clone()),
                        Box::new(Expr::Lit("false".into())),
                    );
                    Some(Expr::Bin(Op::And, Box::new(e.clone()), Box::new(not_false)))
                }
            };
            if let Some(st) = state {
                and(&mut cond, st);
            }
            // Class-only specificity: its names, and its state.
            let weight = needs.len() + usize::from(!matches!(under, Under::State(State::Base)));
            taken.push((weight, cond, steps));
        });
        // Stable: sheet order among rules of one weight.
        taken.sort_by_key(|(w, _, _)| *w);
        for (_, cond, steps) in taken {
            let from = self.steps.len() as u32;
            self.steps.extend_from_slice(steps);
            match cond {
                None => match self.spans.last_mut() {
                    // Consecutive unconditional rules share one span.
                    Some(last)
                        if last.condition == ALWAYS && last.steps[0] + last.steps[1] == from =>
                    {
                        last.steps[1] += steps.len() as u32;
                    }
                    _ => self.span(ALWAYS, from),
                },
                Some(c) => {
                    self.conditions.push(c);
                    self.span(self.conditions.len() as u32 - 1, from);
                }
            }
        }
    }

    fn visit(
        &mut self,
        doc: &UiDocument,
        i: usize,
        cx: &Cx,
        local: &mut Vec<(Box<str>, Box<str>)>,
    ) {
        let tokens = cx.tokens;
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
            self.own(n, cx);
            let base = self.steps.len() as u32;
            for rule in n.rules.iter().filter(|r| r.condition.is_none()) {
                self.rule(rule, tokens, local);
            }
            self.span(ALWAYS, base);
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
            self.visit(doc, c, cx, local);
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
        // The old default style's global rules (`[data-motion=pulse]`,
        // `[part$=".frame"]`) are not the new core's: its fields and
        // surfaces replace them, and they are deleted at the switch.
        if rule.structure.as_ref().is_some_and(|s| s.global) {
            return false;
        }
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
