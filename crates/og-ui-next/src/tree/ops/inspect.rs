//! Probe dumps (`dump_ops`), accessibility tree snapshots, and op-node box
//! queries.

use super::*;

/// An op node as the probe lists it ([`Tree::dump_ops`]).
#[derive(Clone, Debug, PartialEq)]
pub struct OpNode<'a> {
    pub id: u32,
    pub parent: u32,
    pub node: Id,
    pub rect: [f32; 4],
    /// The part of it a scroller or a box cutting its overflow lets show
    /// on screen (x0, y0, x1, y1): `rect`, or under a transform the box
    /// bounding `rect` drawn through it, cut to `clip`.
    pub visible: [f32; 4],
    /// The rectangle on screen it is drawn cut to (x0, y0, x1, y1), by
    /// its ancestors' scrollers and overflow; `None` when nothing cuts it.
    pub clip: Option<[f32; 4]>,
    /// Whether `clip` cuts any of it.
    pub clipped: bool,
    pub shown: bool,
    pub text: &'a str,
    /// Where its laid-out lines cut a word with no UAX #14 break there,
    /// each as `end of the line|start of the next`: a text measured
    /// narrower than it draws. Empty: none.
    pub broken: Vec<String>,
    /// Its text as drawn: the ink (linear RGB, alpha with its opacity),
    /// the font size in CSS px, and each line's box (x, y, width, height:
    /// the em at its baseline) before `transform`. Empty lines: it draws
    /// no text.
    pub ink: [f32; 4],
    pub size: f32,
    pub lines: Vec<[f32; 4]>,
    /// The transform its lines are drawn through, `[a, b, c, d, tx, ty]`
    /// (a point `(x, y)` lands at `(a x + c y + tx, b x + d y + ty)`), when
    /// it or an ancestor has one: a line's quad on screen is its box
    /// through it.
    pub transform: Option<[f32; 6]>,
    /// A press squashes it: an enabled control a finger presses that its
    /// own classes or the pack's `Press.feedback` move under a press.
    pub press: bool,
    /// Its classes give it a `starting:` record it enters from (0) and an
    /// `exit:` record it leaves to (1).
    pub presence: [bool; 2],
    /// A `modal` view: a dialog the rest waits behind.
    pub modal: bool,
    /// It draws a nine-sliced frame its fill colours (a tint mask): that
    /// fill (linear RGB, alpha), at rest. `None`: no such frame.
    pub frame: Option<[f32; 4]>,
}

impl Tree {
    /// The accessibility update to send `client` after this frame, when
    /// what it shows changed: the arena's nodes with their semantics, at
    /// the viewport's size and `scale` (physical px per CSS px). `focus`:
    /// the arena node the keyboard's focus is on.
    pub fn a11y(
        &self,
        client: &mut a11y::Tree,
        focus: Option<Id>,
        scale: f32,
    ) -> Option<accesskit::TreeUpdate> {
        let (width, height) = self.size;
        let viewport = a11y::Viewport {
            width,
            height,
            scale,
        };
        client.update(&self.arena, &self.sem, focus, viewport)
    }

    /// Every live op node, for the probe: its op id, its op parent's (0 at
    /// a mount's top), its arena node, its box on screen (x, y, width,
    /// height) as last painted, whether it shows, and its text. What is on
    /// its way out (an exit playing) is gone already, as for a reader.
    pub fn dump_ops(&self) -> Vec<OpNode<'_>> {
        self.els
            .iter()
            .enumerate()
            .filter(|&(e, el)| !el.dead && el.js != NONE && !self.leaving_in(e as u32))
            .filter_map(|(e, el)| {
                let &(x, y, shown, _) = self.abs.get(e)?;
                let mut b = self.arena.box_of(el.node);
                // A span in a paragraph is where a press on it lands.
                let (x, y) = match self.span_piece(e as u32) {
                    Some([sx, sy, w, h]) => {
                        (b.w, b.h) = (w, h);
                        (sx, sy)
                    }
                    None => (x, y),
                };
                let transform = self
                    .xfs
                    .get(e)
                    .copied()
                    .filter(|&xf| xf != 0)
                    .and_then(|xf| self.painted_transforms.get(xf as usize))
                    .map(|&[a, b, c, d, tx, ty, _, _]| [a, b, c, d, tx, ty]);
                // Its box on screen: through its transform, the box that
                // bounds it there, where the cuts (screen space) apply.
                let own = match transform {
                    None => [x, y, x + b.w, y + b.h],
                    Some([ma, mb, mc, md, me, mf]) => {
                        let mut o = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
                        for [cx, cy] in [[x, y], [x + b.w, y], [x, y + b.h], [x + b.w, y + b.h]] {
                            let (sx, sy) = (ma * cx + mc * cy + me, mb * cx + md * cy + mf);
                            o = [o[0].min(sx), o[1].min(sy), o[2].max(sx), o[3].max(sy)];
                        }
                        o
                    }
                };
                // An anchored node escapes its ancestors' clips.
                let clip = match el.arena_parent {
                    _ if el.anchored().is_some() => None,
                    NONE => None,
                    p => self.cuts.get(p as usize).copied().filter(|c| {
                        c[0] > f32::MIN || c[1] > f32::MIN || c[2] < f32::MAX || c[3] < f32::MAX
                    }),
                };
                let cut = clip.unwrap_or(own);
                let visible = [
                    own[0].max(cut[0]),
                    own[1].max(cut[1]),
                    own[2].min(cut[2]).max(own[0].max(cut[0])),
                    own[3].min(cut[3]).max(own[1].max(cut[1])),
                ];
                let clipped = visible != own;
                // Its text as the last paint laid it out.
                let glyphs = el
                    .leaf
                    .filter(|_| !el.text.is_empty())
                    .and_then(|leaf| self.texts.leaf(leaf))
                    .filter(|g| !g.fill.is_empty() && *g.text == *el.text);
                let parent = match el.parent {
                    NONE => 0,
                    p => self.els[p as usize].js,
                };
                Some(OpNode {
                    id: el.js,
                    parent,
                    node: el.node,
                    rect: [x, y, b.w, b.h],
                    visible,
                    clip,
                    clipped,
                    shown,
                    text: &el.text,
                    broken: glyphs.map_or(Vec::new(), crate::text::Leaf::broken_words),
                    ink: self.shown.get(e).map_or([0.0; 4], |s| {
                        [s.ink[0], s.ink[1], s.ink[2], s.ink[3] * s.alpha]
                    }),
                    size: glyphs.map_or(0.0, |g| g.size()),
                    lines: glyphs.zip(el.leaf).map_or(Vec::new(), |(g, leaf)| {
                        let lb = self.arena.box_of(leaf);
                        let size = g.size();
                        g.lines
                            .iter()
                            .map(|&[dx, baseline, w]| {
                                [x + lb.x + dx, y + lb.y + baseline - 0.8 * size, w, size]
                            })
                            .collect()
                    }),
                    transform,
                    press: el.squash
                        && self.presses(e as u32)
                        && self
                            .input
                            .per(e as u32)
                            .is_none_or(|p| p.states & DISABLED == 0)
                        && !self.catcher(e as u32),
                    presence: [0, 1].map(|k| self.classes.presence(el.class, k).is_some()),
                    modal: self.input.per(e as u32).is_some_and(|p| p.modal),
                    frame: {
                        let im = &el.record.skin.image;
                        (im.texture != 0
                            && im.slice != [0.0; 4]
                            && im.tint == crate::record::Tint::Fill)
                            .then_some(el.record.skin.fill)
                    },
                })
            })
            .collect()
    }

    /// The accessibility tree now, built whole ([`a11y::snapshot`]), with
    /// the focus where input put it: each arena node's facts and the focus
    /// order. For readers that ask once (the probe).
    pub fn a11y_snapshot(&self) -> (rustc_hash::FxHashMap<Id, a11y::Facts>, Vec<Id>) {
        let (width, height) = self.size;
        let viewport = a11y::Viewport {
            width,
            height,
            scale: 1.0,
        };
        a11y::snapshot(&self.arena, &self.sem, self.focused(), viewport)
    }

    /// The viewport the tree lays out in, CSS px.
    pub fn size(&self) -> (f32, f32) {
        self.size
    }

    /// Op node `id`'s box (x, y, width, height) as last painted, in the
    /// tree's units, when it shows.
    pub fn op_box(&self, id: u32) -> Option<[f32; 4]> {
        let e = self.op(id).ok()? as usize;
        let &(x, y, shown, _) = self.abs.get(e)?;
        let b = self.arena.box_of(self.els[e].node);
        shown.then_some([x, y, b.w, b.h])
    }
}
