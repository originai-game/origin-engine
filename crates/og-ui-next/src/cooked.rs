//! A document's compiled styles as the cook writes them and the runtime
//! maps them: the Pod tables as they are, and each condition as a small
//! tagged tree. Nothing here parses CSS or resolves a token; a document
//! whose style was compiled against another look (its key differs), or
//! whose node count differs from the parse the runtime holds, is not
//! taken, and the tree compiles it instead.

use crate::doc::{DocStyle, NodeStyle, Span};
use crate::style::Step;
use bytemuck::Pod;
use og_reactive::bind::{AttrOp, AttrTest, Expr, InteractionState as I, Op, Screen};
use std::rc::Rc;

/// The format's version, part of every key: a reader never takes another
/// version's bytes.
const VERSION: u64 = 1;

/// The interaction states a cooked condition may name, by index.
const STATES: [I; 13] = [
    I::Hover,
    I::Active,
    I::Focus,
    I::FocusVisible,
    I::FocusWithin,
    I::Disabled,
    I::Checked,
    I::Selected,
    I::Open,
    I::Enabled,
    I::More,
    I::ActiveViewTransition,
    I::Invalid,
];
const SCREENS: [Screen; 4] = [
    Screen::Narrow,
    Screen::Short,
    Screen::Compact,
    Screen::Touch,
];
const ATTR_OPS: [AttrOp; 6] = [
    AttrOp::Equals,
    AttrOp::Includes,
    AttrOp::Dash,
    AttrOp::Prefix,
    AttrOp::Suffix,
    AttrOp::Substring,
];
const OPS: [Op; 13] = [
    Op::Mul,
    Op::Div,
    Op::Rem,
    Op::Add,
    Op::Sub,
    Op::Lt,
    Op::Le,
    Op::Gt,
    Op::Ge,
    Op::Eq,
    Op::Ne,
    Op::And,
    Op::Or,
];

/// `style` of a document of `nodes` nodes, compiled against `look`, with
/// the look's root; `None` when a condition uses a form the format does
/// not carry (the runtime then compiles it).
pub fn encode(style: &DocStyle, nodes: usize, look: &crate::look::Look) -> Option<Vec<u8>> {
    let mut w = Vec::new();
    w.extend_from_slice(&(look.key ^ VERSION).to_le_bytes());
    table(&mut w, &look.root_steps);
    put_u32(&mut w, nodes as u32);
    table(&mut w, &style.nodes);
    table(&mut w, &style.spans);
    table(&mut w, &style.steps);
    put_u32(&mut w, style.conditions.len() as u32);
    for c in &style.conditions {
        expr(&mut w, c)?;
    }
    Some(w)
}

/// The style `bytes` holds, when it was compiled against the look `key`
/// for a document of `nodes` nodes.
pub fn decode(bytes: &[u8], nodes: usize, key: u64) -> Option<DocStyle> {
    let mut r = Reader::at_key(bytes, key)?;
    r.table::<Step>()?;
    if r.u32()? as usize != nodes {
        return None;
    }
    let nodes = r.table::<NodeStyle>()?;
    let spans = r.table::<Span>()?;
    let steps = r.table::<Step>()?;
    let n = r.u32()? as usize;
    let mut conditions = Vec::with_capacity(n);
    for _ in 0..n {
        conditions.push(Rc::new(r.expr()?));
    }
    (r.at == bytes.len()).then_some(DocStyle {
        nodes,
        spans,
        steps,
        conditions,
        skipped: Vec::new(),
    })
}

/// The look's root steps `bytes` carries, when they are of look `key`.
pub fn root(bytes: &[u8], key: u64) -> Option<Vec<Step>> {
    Reader::at_key(bytes, key)?.table::<Step>()
}

fn put_u32(w: &mut Vec<u8>, v: u32) {
    w.extend_from_slice(&v.to_le_bytes());
}

fn put_str(w: &mut Vec<u8>, s: &str) {
    put_u32(w, s.len() as u32);
    w.extend_from_slice(s.as_bytes());
}

fn table<T: Pod>(w: &mut Vec<u8>, v: &[T]) {
    put_u32(w, v.len() as u32);
    w.extend_from_slice(bytemuck::cast_slice(v));
}

fn opt(w: &mut Vec<u8>, e: &Option<Box<Expr>>) -> Option<()> {
    match e {
        None => w.push(0),
        Some(e) => {
            w.push(1);
            expr(w, e)?;
        }
    }
    Some(())
}

fn expr(w: &mut Vec<u8>, e: &Expr) -> Option<()> {
    match e {
        Expr::Lit(v) => {
            w.push(0);
            put_str(w, &v.to_string());
        }
        Expr::Path(p) => {
            w.push(1);
            put_str(w, p);
        }
        Expr::Interaction(s) => {
            w.push(2);
            w.push(STATES.iter().position(|x| x == s)? as u8);
        }
        Expr::InteractionAt { node, state } => {
            w.push(3);
            put_u32(w, *node as u32);
            w.push(STATES.iter().position(|x| x == state)? as u8);
        }
        Expr::Screen(s) => {
            w.push(4);
            w.push(SCREENS.iter().position(|x| x == s)? as u8);
        }
        Expr::Lang(on, tags) => {
            w.push(5);
            opt(w, on)?;
            put_u32(w, tags.len() as u32);
            for t in tags {
                put_str(w, t);
            }
        }
        Expr::Dir(on, rtl) => {
            w.push(6);
            opt(w, on)?;
            w.push(u8::from(*rtl));
        }
        Expr::Not(a) => {
            w.push(7);
            expr(w, a)?;
        }
        Expr::Neg(a) => {
            w.push(8);
            expr(w, a)?;
        }
        Expr::Bin(op, a, b) => {
            w.push(9);
            w.push(OPS.iter().position(|x| x == op)? as u8);
            expr(w, a)?;
            expr(w, b)?;
        }
        Expr::If(a, b, c) => {
            w.push(10);
            expr(w, a)?;
            expr(w, b)?;
            expr(w, c)?;
        }
        Expr::Index(a, b) => {
            w.push(11);
            expr(w, a)?;
            expr(w, b)?;
        }
        Expr::Numbers(a) => {
            w.push(12);
            expr(w, a)?;
        }
        // `class:x` over a bound class list (`class={tone}`).
        Expr::HasToken(a, token) => {
            w.push(13);
            expr(w, a)?;
            put_str(w, token);
        }
        Expr::AttrTest(a, t) => {
            w.push(14);
            expr(w, a)?;
            w.push(ATTR_OPS.iter().position(|x| *x == t.op)? as u8);
            put_str(w, &t.value);
            w.push(u8::from(t.insensitive));
        }
        // Not in a style's conditions; a document with one compiles at
        // run time.
        Expr::Translate(..) | Expr::Format(..) => {
            return None;
        }
    }
    Some(())
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn at_key(b: &'a [u8], key: u64) -> Option<Reader<'a>> {
        let mut r = Reader { b, at: 0 };
        (u64::from_le_bytes(r.take(8)?.try_into().ok()?) == key ^ VERSION).then_some(r)
    }

    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let s = self.b.get(self.at..self.at.checked_add(n)?)?;
        self.at += n;
        Some(s)
    }

    fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }

    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    fn str(&mut self) -> Option<String> {
        let n = self.u32()? as usize;
        String::from_utf8(self.take(n)?.to_vec()).ok()
    }

    fn table<T: Pod>(&mut self) -> Option<Vec<T>> {
        let n = self.u32()? as usize;
        let bytes = self.take(n.checked_mul(size_of::<T>())?)?;
        Some(bytemuck::pod_collect_to_vec(bytes))
    }

    fn opt(&mut self) -> Option<Option<Box<Expr>>> {
        Some(match self.u8()? {
            0 => None,
            _ => Some(Box::new(self.expr()?)),
        })
    }

    fn boxed(&mut self) -> Option<Box<Expr>> {
        Some(Box::new(self.expr()?))
    }

    fn expr(&mut self) -> Option<Expr> {
        Some(match self.u8()? {
            0 => Expr::Lit(serde_json::from_str(&self.str()?).ok()?),
            1 => Expr::Path(self.str()?),
            2 => Expr::Interaction(*STATES.get(self.u8()? as usize)?),
            3 => Expr::InteractionAt {
                node: self.u32()? as usize,
                state: *STATES.get(self.u8()? as usize)?,
            },
            4 => Expr::Screen(*SCREENS.get(self.u8()? as usize)?),
            5 => {
                let on = self.opt()?;
                let n = self.u32()? as usize;
                let mut tags = Vec::with_capacity(n.min(64));
                for _ in 0..n {
                    tags.push(self.str()?);
                }
                Expr::Lang(on, tags)
            }
            6 => Expr::Dir(self.opt()?, self.u8()? != 0),
            7 => Expr::Not(self.boxed()?),
            8 => Expr::Neg(self.boxed()?),
            9 => {
                let op = *OPS.get(self.u8()? as usize)?;
                Expr::Bin(op, self.boxed()?, self.boxed()?)
            }
            10 => Expr::If(self.boxed()?, self.boxed()?, self.boxed()?),
            11 => Expr::Index(self.boxed()?, self.boxed()?),
            12 => Expr::Numbers(self.boxed()?),
            13 => Expr::HasToken(self.boxed()?, self.str()?),
            14 => Expr::AttrTest(
                self.boxed()?,
                AttrTest {
                    op: *ATTR_OPS.get(self.u8()? as usize)?,
                    value: self.str()?,
                    insensitive: self.u8()? != 0,
                },
            ),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::look::{Layer, Look};

    /// A document's cooked style maps back to the style the runtime would
    /// compile, conditions and all, and a style of another look or another
    /// parse is not taken.
    #[test]
    fn a_cooked_style_is_the_compiled_one_and_only_for_its_look() {
        let doc = og_ui::UiDocument::parse(
            "c.svelte",
            "<meta name=\"component\" content=\"c\">\n\
             <meta name=\"prop:on\" content=\"bool\">\n\
             <div id=\"A\" part=\"frame\" variant=\"hud\" class={on ? 'x on' : 'x'} state:done={on}><span id=\"T\">x</span></div>\n\
             <style>\n#A { padding: var(--space-sm); }\n#A.on { opacity: 0.5; }\n#A:size(compact) { padding: 0; }\n\
             #A:state(done) { opacity: 0.25; }\n#A:dir(rtl) { margin-left: 4px; }\n</style>",
        )
        .unwrap();
        let layers = [Layer {
            tokens: crate::look::default_tokens(),
            surfaces: crate::look::DEFAULT_SURFACES,
        }];
        let look = Look::load(&layers, "dark");
        let style = DocStyle::compile(&doc, look.tokens(), look.surfaces());
        assert!(!style.conditions.is_empty(), "the test needs conditions");
        let bytes = encode(&style, doc.nodes.len(), &look).unwrap();
        let back = decode(&bytes, doc.nodes.len(), look.key).unwrap();
        assert_eq!(back.nodes, style.nodes);
        assert_eq!(back.spans, style.spans);
        assert_eq!(back.steps, style.steps);
        assert_eq!(back.conditions, style.conditions);
        assert_eq!(root(&bytes, look.key).unwrap(), look.root_steps);
        let light = Look::key_of(&layers, "light");
        assert!(
            decode(&bytes, doc.nodes.len(), light).is_none(),
            "another look"
        );
        assert!(
            decode(&bytes, doc.nodes.len() + 1, look.key).is_none(),
            "another parse"
        );
        let cooked = Look::cooked(&layers, "dark", root(&bytes, look.key).unwrap());
        assert_eq!(cooked.root, look.root);
        assert!(!cooked.was_read(), "a cooked look reads no CSS");
    }
}
