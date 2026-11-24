//! Style inheritance, parent links, depth-first element ordering, and
//! transformed screen bounds for accessibility.

use super::{Inherit, Kind, NONE, RUBY, Tree};
use crate::arena::{Id, Run};

impl Tree {
    /// Applies `e`'s record over what it inherits, then its children's.
    pub(super) fn inherit(&mut self, e: u32) {
        let parent = self.els[e as usize].parent;
        let from = if parent == NONE {
            self.root
        } else {
            self.els[parent as usize].eff
        };
        let el = &self.els[e as usize];
        let r = &el.record;
        let eff = Inherit {
            size: r
                .font_size_em
                .map_or(if r.text.sized { r.font_size } else { from.size }, |em| {
                    em * from.size
                }),
            ink: if r.inked { r.skin.ink } else { from.ink },
            text: r.text.over(&from.text),
            opacity: from.opacity * r.skin.opacity,
            text_shadows: if r.shadowed {
                r.skin.text_shadows
            } else {
                from.text_shadows
            },
            text_stroke: {
                let mut st = from.text_stroke;
                if r.stroked {
                    st.width = r.skin.text_stroke.width;
                }
                let own_ink = if r.inked { r.skin.ink } else { from.ink };
                if r.stroked_color || (r.stroked && st.color[3] == 0.0) {
                    st.color = r.skin.resolve_current(
                        crate::record::Skin::CURRENT_STROKE,
                        r.skin.text_stroke.color,
                        own_ink,
                    );
                }
                st
            },
            decoration: if r.skin.decoration.line != 0 {
                r.skin.decoration
            } else {
                from.decoration
            },
        };
        let mut layout = r.layout_in(self.insets);
        // A `Scroll` is a scroll container on both axes, as `overflow:
        // auto` is: its automatic minimum is 0, so its content does not
        // size it. An anchored node is out of flow: the host places it.
        let scroller = self.input.per(e).is_some_and(|p| p.scroller);
        // A scroll bar's thumb is as long as the tree fitted it, and
        // hidden while its scroller has nothing to scroll.
        let thumb = self
            .input
            .per(e)
            .filter(|p| !p.thumb.is_empty())
            .map(|p| (p.thumb_len, p.thumb_horizontal));
        if el.hidden > 0 || el.anchored().is_some() || scroller || thumb.is_some() {
            let mut s = taffy::Style::from(&layout);
            if scroller {
                s.overflow = taffy::Point {
                    x: taffy::Overflow::Scroll,
                    y: taffy::Overflow::Scroll,
                };
            }
            if let Some((len, horizontal)) = thumb {
                let len = taffy::Dimension::length(len);
                if horizontal {
                    s.size.width = len;
                } else {
                    s.size.height = len;
                }
                s.flex_shrink = 0.0;
                if len == taffy::Dimension::length(0.0) {
                    s.display = taffy::Display::None;
                }
            }
            if el.hidden > 0 {
                s.display = taffy::Display::None;
            } else if el.anchored().is_some() {
                s.position = taffy::Position::Absolute;
            }
            layout = rune_taffy::CompactStyle::from(&s);
        }
        if el.inline != NONE {
            let mut s = taffy::Style::from(&layout);
            Tree::member_style(el.kind, &mut s);
            layout = rune_taffy::CompactStyle::from(&s);
        }
        // A glyph's art is one em square: its element holds that square,
        // whatever the font (its text is empty under the art).
        if self
            .input
            .per(e)
            .is_some_and(|p| p.art != 0 && !p.glyph.is_empty())
        {
            let mut s = taffy::Style::from(&layout);
            let em = taffy::LengthPercentageAuto::length(eff.size * self.text_scale);
            s.min_size = taffy::Size {
                width: em,
                height: em,
            };
            s.flex_shrink = 0.0;
            layout = rune_taffy::CompactStyle::from(&s);
        }
        let in_para = el.inline != NONE || el.para.is_some();
        let (node, leaf, space) = (el.node, el.leaf, r.space);
        self.arena.set_style(node, layout);
        self.arena.set_space(node, space);
        if let Some(leaf) = leaf {
            // The player's text size scales the size and what is a
            // length with it, before the text is laid out.
            let k = self.text_scale;
            let field = self.input.per(e).filter(|p| p.input);
            let one_line = field.is_some_and(|p| !p.multiline);
            // `rows`: a field at least that many lines tall.
            let rows = field.map_or(0, |p| p.rows);
            if field.is_some() {
                let mut s = taffy::Style::default();
                if rows > 0 {
                    let line = eff.text.line_height(eff.size) * k;
                    let line = if line < 0.0 { eff.size * k * 1.2 } else { line };
                    s.min_size.height = taffy::LengthPercentageAuto::length(f32::from(rows) * line);
                }
                self.arena
                    .set_style(leaf, rune_taffy::CompactStyle::from(&s));
            }
            // A span's leaf is its ruby: half its size, on one line, set
            // solid.
            let ruby =
                self.els[e as usize].kind == Kind::Text && self.els[e as usize].inline != NONE;
            let eff = if ruby {
                let mut r = eff;
                r.size *= RUBY;
                r.text.line = 1.0;
                r.text.line_px = false;
                r.text.nowrap = true;
                r
            } else {
                eff
            };
            self.arena.set_run(
                leaf,
                Run {
                    size: eff.size * k,
                    tracking: eff.text.tracking(eff.size) * k,
                    line: eff.text.line_height(eff.size) * k,
                    weight: eff.text.weight,
                    nowrap: eff.text.nowrap || one_line,
                    clamp: eff.text.clamp,
                    family: eff.text.family,
                    italic: eff.text.italic,
                    variant: eff.text.variant,
                    wrap: eff.text.wrap,
                    hyphens: match eff.text.hyphens {
                        nsh_tailwind::Hyphens::Manual => og_text::layout::Hyphens::Manual,
                        nsh_tailwind::Hyphens::Auto => og_text::layout::Hyphens::Auto,
                        nsh_tailwind::Hyphens::None => og_text::layout::Hyphens::None,
                    },
                    ellipsis: eff.text.overflow == 1,
                    align: eff.text.align,
                    transform: if field.is_some() {
                        0
                    } else {
                        eff.text.transform
                    },
                    field: one_line,
                    // A field's text stays in lines: its caret runs along them.
                    vertical: eff.text.vertical && field.is_none(),
                },
            );
        }
        let el = &self.els[e as usize];
        // An icon's or an image's size is its inline box's.
        let changed = el.eff != eff || matches!(el.kind, Kind::Icon | Kind::Image);
        self.els[e as usize].eff = eff;
        // A member's style is its paragraph's span.
        if in_para && changed {
            self.para_touch(e);
        }
    }

    /// Sets `child`'s logical parent to `parent`, keeping `El::children` in sync.
    pub(super) fn set_parent(&mut self, child: u32, parent: u32) {
        let old = self.els[child as usize].parent;
        if old == parent {
            return;
        }
        if old != NONE {
            self.els[old as usize].children.retain(|&c| c != child);
        }
        self.els[child as usize].parent = parent;
        if parent != NONE {
            self.els[parent as usize].children.push(child);
        }
    }

    /// Re-applies `e` and every descendant whose inheritance changed.
    pub(super) fn reapply(&mut self, e: u32) {
        let before = self.els[e as usize].eff;
        self.inherit(e);
        if self.els[e as usize].eff == before {
            return;
        }
        for k in 0..self.els[e as usize].children.len() {
            let c = self.els[e as usize].children[k];
            if !self.els[c as usize].dead {
                self.reapply(c);
            }
        }
    }

    /// The box each element under a transform is drawn in (its painted
    /// box's corners through the transform, bounded), by arena node: the
    /// accessibility tree's bounds for it.
    pub(super) fn drawn_boxes(&self) -> rustc_hash::FxHashMap<Id, [f32; 4]> {
        let mut out = rustc_hash::FxHashMap::default();
        for (e, el) in self.els.iter().enumerate() {
            let xf = self.xfs.get(e).copied().unwrap_or(0);
            let Some(&(x, y, true, _)) = self.abs.get(e) else {
                continue;
            };
            if xf == 0 || el.dead {
                continue;
            }
            let Some(&[a, b, c, d, tx, ty, _, _]) = self.painted_transforms.get(xf as usize) else {
                continue;
            };
            let bx = self
                .motion
                .box_of(el.node)
                .unwrap_or_else(|| self.arena.box_of(el.node));
            let corners = [(x, y), (x + bx.w, y), (x, y + bx.h), (x + bx.w, y + bx.h)]
                .map(|(px, py)| (a * px + c * py + tx, b * px + d * py + ty));
            let x0 = corners.iter().map(|p| p.0).fold(f32::INFINITY, f32::min);
            let y0 = corners.iter().map(|p| p.1).fold(f32::INFINITY, f32::min);
            let x1 = corners
                .iter()
                .map(|p| p.0)
                .fold(f32::NEG_INFINITY, f32::max);
            let y1 = corners
                .iter()
                .map(|p| p.1)
                .fold(f32::NEG_INFINITY, f32::max);
            out.insert(el.node, [x0, y0, x1 - x0, y1 - y0]);
        }
        // A span is where its glyphs are: its first piece, where a press
        // on it lands.
        for e in 0..self.els.len() as u32 {
            if let Some(piece) = self.span_piece(e) {
                out.insert(self.els[e as usize].node, piece);
            }
        }
        out
    }

    /// Makes `order` again when the structure changed: the elements under
    /// the arena's root depth first, in their arena order.
    pub(super) fn order(&mut self) {
        if self.ordered {
            return;
        }
        self.ordered = true;
        self.el_of.clear();
        self.el_of.resize(self.arena.len(), NONE);
        for (e, el) in self.els.iter().enumerate() {
            if !el.dead {
                self.el_of[el.node as usize] = e as u32;
            }
        }
        self.order.clear();
        self.walk.clear();
        self.walk.extend(self.arena.children(0).iter().rev());
        while let Some(n) = self.walk.pop() {
            let e = self.el_of.get(n as usize).copied().unwrap_or(NONE);
            if e != NONE {
                self.order.push(e);
            }
            self.walk.extend(self.arena.children(n).iter().rev());
        }
    }
}
