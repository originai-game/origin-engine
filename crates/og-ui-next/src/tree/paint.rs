//! Frame painting: quads, text runs, nine-slice images, lines, filter and
//! backdrop groups, decorations, gradients, and transforms.

use super::{Anchor, El, Kind, NONE, Shown, Textures, Tree, input, ops};
use crate::arena::Arena;
use crate::icons::{IconAtlas, IconCell};
use crate::record::{Decoration, Rgba, Text, TextShadows};
use og_draw::glyphs::Atlas;
use og_draw::paint::{Paint, PaintEffect, PaintGroup, PaintKind, PaintQuad};
use og_text::fonts::Collection;

/// Theme colour role `role` (`gold`, `icon-outline`) as a toned icon
/// paints it: the theme's in the page's scheme, or the other one's,
/// resolved once a resolver (`cache` is cleared with it).
fn tone(
    classes: &ops::Classes,
    cache: &std::cell::RefCell<std::collections::HashMap<(Box<str>, bool), Rgba>>,
    role: &str,
    other: bool,
) -> Rgba {
    let key = (Box::<str>::from(role), other);
    if let Some(c) = cache.borrow().get(&key) {
        return *c;
    }
    let c = classes.tone(role, other);
    cache.borrow_mut().insert(key, c);
    c
}

/// A `<Line>`'s own props (CONVENTIONS.md §Ops).
#[derive(Clone, Debug, Default)]
pub(crate) struct Line {
    /// x, y pairs: CSS px from the box's top left, or units of `view`.
    pub(crate) points: Vec<f32>,
    pub(crate) closed: bool,
    /// `viewBox` as SVG's min-x, min-y, width and height: the points'
    /// space, which is mapped onto the laid-out box as it paints; `None` px.
    pub(crate) view: Option<[f32; 4]>,
    /// `fit="contain"`: the view scales uniformly and is centred in the box
    /// (SVG's `meet`); otherwise it stretches to fill it.
    pub(crate) contain: bool,
}

impl Line {
    /// The points in the box's px, for a box `w` by `h`.
    fn mapped(&self, w: f32, h: f32) -> impl Iterator<Item = [f32; 2]> + '_ {
        let (s, o) = match self.view {
            Some([vx, vy, vw, vh]) if vw > 0.0 && vh > 0.0 => {
                let (sx, sy) = (w / vw, h / vh);
                let (s, pad) = if self.contain {
                    let k = sx.min(sy);
                    ([k, k], [(w - vw * k) / 2.0, (h - vh * k) / 2.0])
                } else {
                    ([sx, sy], [0.0, 0.0])
                };
                (s, [pad[0] - vx * s[0], pad[1] - vy * s[1]])
            }
            _ => ([1.0, 1.0], [0.0, 0.0]),
        };
        self.points
            .as_chunks::<2>()
            .0
            .iter()
            .map(move |&[x, y]| [o[0] + x * s[0], o[1] + y * s[1]])
    }
}

/// What an element's props set of its box image (CONVENTIONS.md §Images):
/// its source (TSX `src`, .svelte `<img texture>`) and the radial wipe
/// bound per tick, so a moving share never makes a class string.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ImageProps {
    /// The source: the texture handle as the game holds it, which
    /// [`Tree::set_textures`] resolves as it paints; NaN none.
    pub(crate) handle: f64,
    pub(crate) wipe: Option<f32>,
    pub(crate) wipe_start: Option<f32>,
    pub(crate) wipe_ccw: Option<bool>,
}

impl ImageProps {
    /// The box image `base` (its classes') with these props over it, the
    /// source's slot from `textures` (none: 0). `wipe` is already folded
    /// into `base.wipe` (and animated by `Motion`).
    fn over(
        &self,
        base: &crate::record::Slice,
        textures: Option<&Textures>,
    ) -> crate::record::Slice {
        let mut im = *base;
        if !self.handle.is_nan() {
            (im.texture, im.size) = textures.map_or((0, [0.0; 2]), |f| f(self.handle));
        }
        im.wipe_start = self.wipe_start.unwrap_or(im.wipe_start);
        im.wipe_ccw = self.wipe_ccw.unwrap_or(im.wipe_ccw);
        im
    }
}

impl Tree {
    pub(super) fn repaint(&mut self, fonts: &mut Collection, atlas: &mut Atlas, icons: &IconAtlas) {
        /// A line's stroke: its own colour faded, else the ink (already faded).
        fn s_line_color(stroke: &Option<Rgba>, ink: Rgba, alpha: f32) -> Rgba {
            stroke.map_or(ink, |mut c| {
                c[3] *= alpha;
                c
            })
        }
        let (vw, vh) = self.size;
        let screen = [0.0, 0.0, vw, vh];
        self.follow_caret();
        let p = &mut self.drawn;
        p.clear();
        (p.width, p.height) = (vw, vh);
        self.plots.clear();
        let scale = self.scale;
        self.order();
        self.stacking.begin(&self.els, &self.order);
        self.abs.clear();
        self.abs.resize(self.els.len(), (0.0, 0.0, false, [0.0; 4]));
        const ALL: [f32; 4] = [f32::MIN, f32::MIN, f32::MAX, f32::MAX];
        self.cuts.clear();
        self.cuts.resize(self.els.len(), ALL);
        self.rounds.clear();
        self.rounds.resize(self.els.len(), 0);
        self.shown.clear();
        let top = Shown {
            alpha: 1.0,
            ink: self.root.ink,
            text_shadows: self.root.text_shadows,
            text_stroke: self.root.text_stroke,
            out: false,
            touch: true,
            fill: None,
        };
        // NaN: not reached yet this frame.
        let unseen = Shown {
            alpha: f32::NAN,
            ..top
        };
        self.shown.resize(self.els.len(), unseen);
        self.xfs.resize(self.els.len(), 0);
        self.masks.clear();
        self.masks.resize(self.els.len(), 0);
        self.filters.clear();
        self.filters.resize(self.els.len(), 0);
        self.blends.clear();
        self.blends.resize(self.els.len(), 0);
        // The nodes painting for the first time with a `sound` to play.
        let mut sounding: Vec<u32> = Vec::new();
        // The groups open while their subtrees paint, innermost last, and
        // the layers the paint may still draw.
        let mut open: Vec<Open> = Vec::new();
        // The boxes of what the last filter group listed holds, each a
        // group it merged (`close_group`).
        let mut members: Vec<[f32; 4]> = Vec::new();
        let mut room = Room {
            layers: 0,
            area: 0.0,
            limit: og_draw::paint::GROUP_SCREENS * vw * vh,
        };
        for k in 0..self.order.len() {
            let e = self.order[k] as usize;
            // What it is not under is painted: those groups close.
            while open
                .last()
                .is_some_and(|o| !descends(&self.els, e as u32, o.e))
            {
                let o = open.pop().expect("open");
                close_group(
                    &mut self.drawn,
                    &o,
                    screen,
                    (!self.stacking.reorders()).then_some(&mut members),
                );
            }
            let el = &self.els[e];
            // What motion shows: the box a layout transition slides, the
            // skin a transition or keyframes move.
            let b = self
                .motion
                .box_of(el.node)
                .unwrap_or_else(|| self.arena.box_of(el.node));
            let (px, py, pv, pclip) = if let Some(a) = el.anchored() {
                // Its box's top left at its placed point, over the screen;
                // it shows when its target is in view and its parent shows.
                let pv = el.arena_parent == NONE || self.abs[el.arena_parent as usize].2;
                (a.at[0] - b.x, a.at[1] - b.y, pv && a.shown, screen)
            } else if el.arena_parent == NONE {
                (0.0, 0.0, true, screen)
            } else {
                // A scroller's children move by its scroll.
                let [sx, sy] = self.moved_by(e as u32);
                let (px, py, pv, pclip) = self.abs[el.arena_parent as usize];
                (px - sx, py - sy, pv, pclip)
            };
            // What it inherits comes from the element it inherits from,
            // which a fixed one is not the arena child of: one not reached
            // yet gives what its record folds to.
            let from = match el.parent {
                NONE => top,
                p if !self.shown[p as usize].alpha.is_nan() => self.shown[p as usize],
                p => {
                    let eff = &self.els[p as usize].eff;
                    Shown {
                        alpha: eff.opacity,
                        ink: eff.ink,
                        text_shadows: eff.text_shadows,
                        text_stroke: eff.text_stroke,
                        out: false,
                        touch: true,
                        fill: None,
                    }
                }
            };
            // `display: none` leaves a zero box at its parent's corner:
            // nothing of it or under it paints.
            let r = &el.record;
            let s = self.motion.skin(el.node, &r.skin);
            let alpha = from.alpha * s.opacity * el.anchored().map_or(1.0, Anchor::alpha);
            let own_ink = if r.inked { s.ink } else { from.ink };
            let text_shadows = if r.shadowed {
                s.text_shadows
            } else {
                from.text_shadows
            };
            let text_stroke = {
                let mut st = from.text_stroke;
                if r.stroked {
                    st.width = s.text_stroke.width;
                }
                if r.stroked_color || (r.stroked && st.color[3] == 0.0) {
                    st.color = s.resolve_current(
                        crate::record::Skin::CURRENT_STROKE,
                        s.text_stroke.color,
                        own_ink,
                    );
                }
                st
            };
            self.shown[e] = Shown {
                alpha,
                ink: own_ink,
                text_shadows,
                text_stroke,
                out: from.out
                    || self.motion.exiting(el.node)
                    || self.leaving.iter().any(|l| l.0 == e as u32),
                touch: r.touch.unwrap_or(from.touch),
                fill: r.fill.unwrap_or(from.fill),
            };
            let visible = pv
                && !el.dead
                && el.hidden == 0
                && !r.hidden
                && alpha > 0.0
                && r.layout.display() != taffy::Display::None;
            // An icon or an image in a paragraph is drawn where its line
            // put it, and not where a cut line dropped it.
            let at = match el.kind {
                Kind::Icon | Kind::Image if el.inline != NONE => Some(self.inline_box(e as u32)),
                Kind::Text if el.inline != NONE && !el.ruby.is_empty() => {
                    Some(self.ruby_at(e as u32))
                }
                _ => None,
            };
            // A run in a paragraph is drawn by the paragraph.
            let visible = visible
                && at.is_none_or(|a| a.is_some())
                && !(el.inline != NONE && el.kind == Kind::Label);
            let (x, y) = match at.flatten() {
                Some([ix, iy]) => (ix, iy),
                None => (px + b.x, py + b.y),
            };
            let clip = pclip;
            let pcut = match el.arena_parent {
                _ if el.anchor.is_some() => ALL,
                NONE => ALL,
                p => self.cuts[p as usize],
            };
            // What overflows a box that does not show it is cut to it.
            let o = taffy::CoreStyle::overflow(&r.layout);
            let scroller = self.scroll_axes(e as u32) != [false; 2];
            // CSS clips overflow at the padding box: what scrolls or runs
            // over never covers the box's own border.
            let [bt, br, bb, bl] = s.border;
            let cuts =
                scroller || o.x != taffy::Overflow::Visible || o.y != taffy::Overflow::Visible;
            let cut_to = |c: [f32; 4], [x0, y0, x1, y1]: [f32; 4]| {
                [c[0].max(x0), c[1].max(y0), c[2].min(x1), c[3].min(y1)]
            };
            let padding = [x + bl, y + bt, x + b.w - br, y + b.h - bb];
            let inner = if cuts { cut_to(clip, padding) } else { clip };
            self.abs[e] = (x, y, visible, inner);
            self.cuts[e] = if cuts { cut_to(pcut, padding) } else { pcut };
            // The rounded clip it is drawn in, and the one it sets: CSS
            // clips overflow to the padding box with the inner radii.
            let pround = match el.arena_parent {
                _ if el.anchor.is_some() => 0,
                NONE => 0,
                p => self.rounds[p as usize],
            };
            let pxf = if el.arena_parent == NONE {
                0
            } else {
                self.xfs[el.arena_parent as usize]
            };
            self.xfs[e] = pxf;
            self.masks[e] = if el.arena_parent == NONE {
                0
            } else {
                self.masks[el.arena_parent as usize]
            };
            self.filters[e] = if el.arena_parent == NONE {
                0
            } else {
                self.filters[el.arena_parent as usize]
            };
            self.blends[e] = if s.blend != 0 {
                s.blend
            } else if el.arena_parent == NONE {
                0
            } else {
                self.blends[el.arena_parent as usize]
            };
            if !visible {
                continue;
            }
            if !self.input.per[e].cue.is_empty() {
                sounding.push(e as u32);
            }
            // A `knob` node rides its on-screen control's knob.
            let [kx, ky] = self.knob_of(e as u32);
            // A scroll bar's thumb rides its scroller's offset.
            let [hx, hy] = self.thumb_shift(e as u32);
            let mut t = s.transform;
            t.translate = [t.translate[0] + kx + hx, t.translate[1] + ky + hy];
            if let Some(a) = el.anchored() {
                let ds = a.dist_scale();
                t.scale = [t.scale[0] * ds, t.scale[1] * ds];
            }
            let hole = self.hole(e as u32);
            // Its `blur-*` and `drop-shadow-*` draw it and its subtree
            // through a layer of their own, while the paint has room for
            // one; past it, unfiltered, said once.
            let material = el.material.as_ref().map(|b| b.1);
            let needs_group = r.effect_count > 0 || material.is_some();
            let group = needs_group && room.take(reached([x, y, b.w, b.h], r.effects(), screen));
            if needs_group && !group && !self.warned_layers.contains(&el.node) {
                self.warned_layers.push(el.node);
                let name = &self.input.per[e].name;
                self.warnings.push(format!(
                    "node {}{}{} draws without its subtree material, blur and drop shadows: the frame's other filters already take {} layers or {} screens of them",
                    el.node,
                    if name.is_empty() { "" } else { " " },
                    name,
                    og_draw::paint::GROUP_LAYERS,
                    og_draw::paint::GROUP_SCREENS,
                ));
            }
            // Its group's composite takes its opacity, its mask and its
            // ancestors' colour filters (CSS's order: the filter, then the
            // mask, then opacity); inside the layer it and its subtree
            // draw as they are, their own colour filters first.
            let outer_alpha = alpha;
            let alpha = if group {
                self.shown[e].alpha = 1.0;
                1.0
            } else {
                alpha
            };
            // A selected text's selection, and a paragraph's members' inks
            // and decorations, by ink ([`para`]).
            let selected = self.selected_in(e as u32);
            let span_looks: Vec<(Rgba, Decoration)> = el.para.as_ref().map_or(Vec::new(), |para| {
                para.members
                    .iter()
                    .map(|&m| {
                        let m = &self.els[m as usize];
                        let mut c = m.eff.ink;
                        c[3] *= alpha;
                        (c, m.eff.decoration)
                    })
                    .collect()
            });
            let p = &mut self.drawn;
            let start = p.quads.len();
            // Its own mask replaces an ancestor's over its subtree.
            if let Some(g) = &r.mask {
                let area = [x, y, b.w, b.h];
                let gi = p.shaped_gradient(paint_gradient(g, area), g.stops().iter().copied());
                self.masks[e] = gi + 1;
            }
            let (outer_mask, outer_filter, outer_blend) =
                (self.masks[e], self.filters[e], self.blends[e]);
            if group {
                (self.masks[e], self.filters[e], self.blends[e]) = (0, 0, 0);
            }
            // Its own filters, then its ancestors' over them (CSS's filter
            // on each of the nested groups).
            if let Some(own) = &r.filter {
                let m = match self.filters[e] {
                    0 => *own,
                    k => og_draw::paint::color_then(own, &p.filters[k as usize - 1]),
                };
                p.filters.push(m);
                self.filters[e] = p.filters.len() as u32;
            }
            let xf = transformed(p, pxf, &t, [x, y, b.w, b.h]);
            self.xfs[e] = xf;
            self.rounds[e] = pround;
            if cuts {
                let outer = radii(s.radius, b.w, b.h);
                let inner = [
                    outer[0] - bt.max(bl),
                    outer[1] - bt.max(br),
                    outer[2] - bb.max(br),
                    outer[3] - bb.max(bl),
                ]
                .map(|k| k.max(0.0));
                if inner.iter().any(|&k| k > 0.0) {
                    p.rounds.push(og_draw::paint::PaintRound {
                        rect: padding,
                        radius: inner,
                        xf,
                        invert: 0,
                        _pad: [0; 2],
                    });
                    self.rounds[e] = p.rounds.len() as u32;
                }
            }
            // The clip it sets for what it holds is drawn through its
            // transform (an ancestor's or its own): the box that bounds its
            // padding box there, against the clip it is drawn in.
            if xf != 0 && cuts {
                let [ma, mb, mc, md, me, mf, _, _] = p.transforms[xf as usize];
                let [x0, y0, x1, y1] = padding;
                let mut bound = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
                for [cx, cy] in [[x0, y0], [x1, y0], [x0, y1], [x1, y1]] {
                    let (sx, sy) = (ma * cx + mc * cy + me, mb * cx + md * cy + mf);
                    bound = [
                        bound[0].min(sx),
                        bound[1].min(sy),
                        bound[2].max(sx),
                        bound[3].max(sy),
                    ];
                }
                self.abs[e].3 = cut_to(clip, bound);
                self.cuts[e] = cut_to(pcut, bound);
            }
            let rect = [x, y, b.w, b.h];
            let radius = radii(s.radius, b.w, b.h);
            // Its backdrop filter: a filtered copy of what is under it,
            // drawn under its box, rounded as it is. Respects the frame's
            // offscreen layer budget (`room`).
            let mut backdrop = None;
            if (s.backdrop_blur > 0.0 || r.backdrop_filter.is_some()) && room.take(b.w * b.h) {
                let first = p.effects.len() as u32;
                if s.backdrop_blur > 0.0 {
                    p.effects.push(PaintEffect::Blur(s.backdrop_blur));
                }
                if let Some(m) = &r.backdrop_filter {
                    p.filters.push(*m);
                    p.effects
                        .push(PaintEffect::Matrix(p.filters.len() as u32 - 1));
                }
                let at = p.quads.len() as u32;
                p.groups.push(PaintGroup {
                    quad: at,
                    end: at + 1,
                    first,
                    count: p.effects.len() as u32 - first,
                    xf,
                    backdrop: true,
                    material: None,
                });
                p.quads.push(quad(
                    PaintKind::Group,
                    rect,
                    clip,
                    [1.0, 1.0, 1.0, outer_alpha],
                    radius,
                    [0.0; 4],
                    p.groups.len() as u32 - 1,
                    og_draw::paint::GROUP_SHAPED,
                ));
                backdrop = Some(at as usize);
            }
            if group {
                let first = p.effects.len() as u32;
                p.effects.extend_from_slice(r.effects());
                open.push(Open {
                    e: e as u32,
                    quad: p.quads.len(),
                    first,
                    count: u32::from(r.effect_count),
                    xf,
                    material,
                    composite: PaintQuad {
                        mask: outer_mask,
                        filter: outer_filter,
                        round: pround,
                        blend: u32::from(outer_blend),
                        ..quad(
                            PaintKind::Group,
                            [0.0; 4],
                            clip,
                            [1.0, 1.0, 1.0, outer_alpha],
                            [0.0; 4],
                            [0.0; 4],
                            0,
                            0,
                        )
                    },
                });
                // Written when its subtree has painted (`close_group`).
                p.quads.push(open.last().expect("pushed").composite);
            }
            let faded = |mut c: Rgba| {
                c[3] *= alpha;
                c
            };
            let fill_color = s.resolve_current(crate::record::Skin::CURRENT_FILL, s.fill, own_ink);
            let border_color =
                s.resolve_current(crate::record::Skin::CURRENT_BORDER, s.border_color, own_ink);
            let border_side = s.resolve_current(
                crate::record::Skin::CURRENT_BORDER_SIDE,
                s.border_side,
                own_ink,
            );
            let outline_color = s.resolve_current(
                crate::record::Skin::CURRENT_OUTLINE,
                s.outline.color,
                own_ink,
            );
            let im = &match &el.image {
                Some(props) => props.over(&s.image, self.textures.as_ref()),
                None => s.image,
            };
            let image = og_draw::TextureHandle::from_slot(im.texture).filter(|_| im.wipe > 0.0);
            // CSS `border-image`: a nine-sliced frame is the box's edge.
            let framed = image.is_some() && im.slice != [0.0; 4];
            // Outer shadows under the box, the last listed lowest.
            let shadows = &s.shadows[..usize::from(s.shadow_count)];
            for (k, sh) in shadows.iter().enumerate().rev() {
                if sh.inset {
                    continue;
                }
                let col =
                    s.resolve_current(crate::record::Skin::CURRENT_SHADOW0 << k, sh.color, own_ink);
                if col[3] <= 0.0 {
                    continue;
                }
                let p4 = [sh.offset[0], sh.offset[1], sh.blur, sh.spread];
                p.quads.push(quad(
                    PaintKind::Shadow,
                    rect,
                    clip,
                    faded(col),
                    radius,
                    p4,
                    0,
                    0,
                ));
            }
            // A framed box's outline follows its frame, as a browser's
            // follows `border-radius`: the frame's silhouette in the
            // outline's colour, grown by its width (and a positive offset),
            // under the box. It is always outside, since the frame covers
            // what is inside.
            let o = &s.outline;
            if let Some(t) = image.filter(|_| framed && o.width > 0.0 && outline_color[3] > 0.0) {
                let out = o.width + o.offset.max(0.0);
                let grown = [x - out, y - out, b.w + 2.0 * out, b.h + 2.0 * out];
                let width = if im.width == [0.0; 4] {
                    im.slice
                } else {
                    im.width
                };
                let fit = (grown[2] / (width[1] + width[3]))
                    .min(grown[3] / (width[0] + width[2]))
                    .min(1.0);
                p.quads.push(PaintQuad {
                    uv: im.uv,
                    ..quad(
                        PaintKind::Image,
                        grown,
                        clip,
                        faded(outline_color),
                        width.map(|w| w * fit),
                        im.slice,
                        t.to_bits(),
                        og_draw::paint::IMAGE_FILL,
                    )
                });
            }
            // A fill-tinted image takes the fill's colour in its place.
            let fill_tints = image.is_some() && im.tint == crate::record::Tint::Fill;
            if let Some(([hx, hy, hw, hh], hr)) = hole.filter(|_| fill_color[3] > 0.0) {
                // A spotlight's ground covers the surface but its hole,
                // the lit node's border box rounded as it is: one quad
                // cut by the hole's round, inverted.
                p.rounds.push(og_draw::paint::PaintRound {
                    rect: [hx, hy, hx + hw, hy + hh],
                    radius: hr,
                    xf: 0,
                    invert: 1,
                    _pad: [0; 2],
                });
                p.quads.push(PaintQuad {
                    round: p.rounds.len() as u32,
                    ..quad(
                        PaintKind::Solid,
                        screen,
                        screen,
                        faded(fill_color),
                        [0.0; 4],
                        [0.0; 4],
                        0,
                        0,
                    )
                });
            } else if fill_color[3] > 0.0 && !fill_tints {
                p.quads.push(quad(
                    PaintKind::Solid,
                    rect,
                    clip,
                    faded(fill_color),
                    radius,
                    [0.0; 4],
                    0,
                    0,
                ));
            }
            if let Some(g) = &s.gradient {
                let gi = p.shaped_gradient(
                    paint_gradient(g, [x, y, b.w, b.h]),
                    g.stops().iter().copied(),
                );
                p.quads.push(quad(
                    PaintKind::Gradient,
                    rect,
                    clip,
                    [1.0, 1.0, 1.0, alpha],
                    radius,
                    [0.0; 4],
                    gi,
                    0,
                ));
            }
            // Its image over the fill, nine-sliced, radially wiped.
            if let Some(t) = image {
                let width = if im.width == [0.0; 4] {
                    im.slice
                } else {
                    im.width
                };
                // CSS border-image: insets that together overrun the box
                // shrink in proportion until they fit, so a frame's
                // corners never cross on a box smaller than its art.
                let fit = (b.w / (width[1] + width[3]))
                    .min(b.h / (width[0] + width[2]))
                    .min(1.0);
                let width = width.map(|w| w * fit);
                let color = match im.tint {
                    crate::record::Tint::Own => [1.0, 1.0, 1.0, alpha],
                    crate::record::Tint::Ink => faded(own_ink),
                    crate::record::Tint::Fill => faded(fill_color),
                };
                let flags = (u32::from(im.fill) * og_draw::paint::IMAGE_FILL)
                    | (u32::from(im.tile) * og_draw::paint::IMAGE_TILE);
                let cut = if im.wipe < 1.0 { 1.0 - im.wipe } else { 0.0 };
                // `object-*`: the source at its size in the box, placed
                // and cropped to it.
                let (img_rect, uv) = match r.object {
                    Some(object) if im.slice == [0.0; 4] => fitted(object, rect, im.uv, im.size),
                    _ => (rect, im.uv),
                };
                // An image with `rounded-*` (not a nine-slice frame) clips
                // directly to its own rounded border box.
                let img_round = if !framed && radius.iter().any(|&k| k > 0.0) {
                    p.rounds.push(og_draw::paint::PaintRound {
                        rect: [x, y, x + b.w, y + b.h],
                        radius,
                        xf,
                        invert: 0,
                        _pad: [0; 2],
                    });
                    p.rounds.len() as u32
                } else {
                    0
                };
                p.quads.push(PaintQuad {
                    uv,
                    wipe: [cut, im.wipe_start, f32::from(u8::from(im.wipe_ccw)), 0.0],
                    mask: 0,
                    filter: 0,
                    round: img_round,
                    ..quad(
                        PaintKind::Image,
                        img_rect,
                        clip,
                        color,
                        width,
                        im.slice,
                        t.to_bits(),
                        flags,
                    )
                });
            }
            for (k, sh) in shadows.iter().enumerate().rev() {
                if !sh.inset {
                    continue;
                }
                let col =
                    s.resolve_current(crate::record::Skin::CURRENT_SHADOW0 << k, sh.color, own_ink);
                if col[3] <= 0.0 {
                    continue;
                }
                let p4 = [sh.offset[0], sh.offset[1], sh.blur, sh.spread];
                p.quads.push(quad(
                    PaintKind::InsetShadow,
                    rect,
                    clip,
                    faded(col),
                    radius,
                    p4,
                    0,
                    0,
                ));
            }
            let sides = if border_side[3] > 0.0 {
                s.border_sides
            } else {
                0
            };
            // The frame takes the border's place: the border keeps only
            // its room in the layout.
            if !framed && (border_color[3] > 0.0 || sides != 0) && s.border.iter().any(|&w| w > 0.0)
            {
                p.quads.push(PaintQuad {
                    uv: faded(border_side),
                    ..quad(
                        PaintKind::Border,
                        rect,
                        clip,
                        faded(border_color),
                        radius,
                        s.border,
                        0,
                        u32::from(sides),
                    )
                });
            }
            if !framed && o.width > 0.0 && outline_color[3] > 0.0 {
                // The ring of the box grown by the offset and the width; its
                // corners follow the box's, as browsers draw them.
                let out = o.offset + o.width;
                let grown = [x - out, y - out, b.w + 2.0 * out, b.h + 2.0 * out];
                let rr = radius.map(|k| if k > 0.0 { (k + out).max(0.0) } else { 0.0 });
                p.quads.push(quad(
                    PaintKind::Border,
                    grown,
                    clip,
                    faded(outline_color),
                    rr,
                    [o.width; 4],
                    0,
                    0,
                ));
            }
            let mut ink = own_ink;
            ink[3] *= alpha;
            // An icon is painted in its fill, the ink unless `fill-*` says.
            let icon_ink = self.shown[e].fill.map_or(ink, faded);
            if let Some(line) = &el.line {
                let stroke = s_line_color(&r.stroke, ink, alpha);
                let n = line.points.len() / 2;
                if r.stroke_width > 0.0 && n >= 2 && stroke[3] > 0.0 {
                    let first = p.points.len() as u32;
                    p.points.extend(line.mapped(b.w, b.h));
                    p.quads.push(quad(
                        PaintKind::Line,
                        rect,
                        clip,
                        stroke,
                        [0.0; 4],
                        [r.stroke_width, f32::from(u8::from(line.closed)), 0.0, 0.0],
                        first,
                        n as u32,
                    ));
                }
            }
            if el.kind == Kind::Icon && !el.text.is_empty() {
                let size = b.w.min(b.h).max(1.0);
                let square = [x + (b.w - size) / 2.0, y + (b.h - size) / 2.0, size, size];
                match icons.find_drawn(&el.text) {
                    Some((IconCell::Face(face, index), _)) => {
                        // Its fields, in the glyph atlas: put back when the
                        // atlas dropped them, drawn nothing this frame when
                        // it has no room for them yet. A toned icon is a
                        // sticker: its silhouette's shadow and outline, then
                        // each tone in its colour role.
                        let mut layers: Vec<(u32, [f32; 4], Rgba, f32)> = Vec::new();
                        match face.tones.of(index) {
                            Some(tones) => {
                                let [o, dx, dy] = face.tones.sticker.map(|v| v * size);
                                let at =
                                    |dx: f32, dy: f32| [square[0] + dx, square[1] + dy, size, size];
                                let role = |r: &str| {
                                    faded(tone(&self.classes, &self.tone_colors, r, el.other))
                                };
                                layers.push((index, at(dx, dy), role("icon-shadow"), o));
                                layers.push((index, square, role("icon-outline"), o));
                                for (g, r) in tones {
                                    let c = if r.is_empty() { icon_ink } else { role(r) };
                                    layers.push((*g, square, c, 0.0));
                                }
                            }
                            None => layers.push((index, square, icon_ink, 0.0)),
                        }
                        for (glyph, at, color, grow) in layers {
                            if color[3] <= 0.0 {
                                continue;
                            }
                            if let Some((q, plot)) = face.quad(atlas, glyph, at) {
                                if !self.plots.contains(&plot) {
                                    self.plots.push(plot);
                                }
                                p.quads.push(PaintQuad {
                                    color,
                                    clip,
                                    p: [grow, 0.0, q.p[2], q.p[3]],
                                    ..q
                                });
                            }
                        }
                    }
                    // A texture grid's cell: a picture in its own colours, a
                    // mask in the ink.
                    Some((IconCell::Image(t, uv), picture)) => {
                        let color = if picture {
                            [1.0, 1.0, 1.0, alpha]
                        } else {
                            icon_ink
                        };
                        p.quads.push(PaintQuad {
                            uv,
                            ..quad(
                                PaintKind::Image,
                                square,
                                clip,
                                color,
                                [0.0; 4],
                                [0.0; 4],
                                t.to_bits(),
                                og_draw::paint::IMAGE_FILL,
                            )
                        });
                    }
                    Some((IconCell::Builtin(name), _)) => {
                        let set = crate::icons::icons();
                        if p.icons.is_none() {
                            p.icons = Some(set.segments.clone());
                        }
                        if let Some((first, count)) = set.find(name) {
                            p.quads.push(quad(
                                PaintKind::Icon,
                                square,
                                clip,
                                icon_ink,
                                [0.0; 4],
                                [0.0; 4],
                                first,
                                count,
                            ));
                        }
                    }
                    None => {}
                }
            }
            // A glyph with the pack's art for its binding draws it in its
            // text's place: an ink mask one em square, centred on the
            // glyph's box (which holds that square, so it lays out as one).
            if let Some(t) = self
                .input
                .per(e as u32)
                .filter(|g| !g.glyph.is_empty())
                .and_then(|g| og_draw::TextureHandle::from_slot(g.art))
            {
                let size = el.eff.size * self.text_scale;
                let square = [x + (b.w - size) / 2.0, y + (b.h - size) / 2.0, size, size];
                p.quads.push(PaintQuad {
                    uv: [0.0, 0.0, 1.0, 1.0],
                    ..quad(
                        PaintKind::Image,
                        square,
                        clip,
                        ink,
                        [0.0; 4],
                        [0.0; 4],
                        t.to_bits(),
                        og_draw::paint::IMAGE_FILL,
                    )
                });
            }
            // An `Input`'s own placeholder colour, else half its ink.
            let field = self.input.per(e as u32).filter(|f| f.input);
            // A one-line field scrolls horizontally, a multi-line field
            // vertically, clipped to its content box with room for the caret.
            let scroll = field.filter(|f| !f.multiline).map_or(0.0, |f| f.scroll_x);
            let scroll_y = field.filter(|f| f.multiline).map_or(0.0, |f| f.scroll_y);
            let clip = match field {
                Some(f) if !f.multiline => {
                    let [l, _, r, _] = self.arena.box_of(el.node).inset;
                    cut_to(clip, [x + l, y, x + b.w - r + 2.0, y + b.h])
                }
                Some(_) => {
                    let [l, t, r, bt_in] = self.arena.box_of(el.node).inset;
                    cut_to(clip, [x + l, y + t, x + b.w - r + 2.0, y + b.h - bt_in])
                }
                _ => clip,
            };
            let mut text_ink = ink;
            if field.is_some_and(|f| f.value.is_empty()) {
                text_ink = s.placeholder.map_or_else(
                    || {
                        let mut c = ink;
                        c[3] *= 0.5;
                        c
                    },
                    faded,
                );
            }
            // What the leaf lays out is what draws: a rolling number's
            // place on its way, not yet its text.
            if let Some(leaf) = el.leaf
                && !el.text.is_empty()
                && let Some((drawn, run)) = self.arena.text_run(leaf).as_ref()
            {
                let run = *run;
                let lb = self.arena.box_of(leaf);
                // A transform that is more than a move draws distance
                // fields; any other text sits on whole device pixels,
                // where a move puts it.
                let [ma, mb, mc, md, tx, ty, _, _] = match xf {
                    0 => og_draw::paint::IDENTITY,
                    xf => p.transforms[xf as usize],
                };
                let sdf = [ma, mb, mc, md] != [1.0, 0.0, 0.0, 1.0];
                let snap = |v: f32, t: f32| match sdf {
                    true => v,
                    false => ((v + t) * scale).round() / scale - t,
                };
                let (lx, ly) = (snap(x + lb.x - scroll, tx), snap(y + lb.y - scroll_y, ty));
                // `text-overflow` cuts at the box that set it.
                let mut width = lb.w;
                if run.ellipsis
                    && let Some(right) =
                        overflow_edge(&self.els, &self.abs, &self.motion, &self.arena, e as u32)
                {
                    width = width.min(right - (x + lb.x)).max(0.0);
                }
                let sh = text_shadows;
                let st = text_stroke;
                // A raster per distinct blur among the layers that draw.
                let mut blurs = [0.0f32; crate::record::TEXT_SHADOWS];
                let mut n_blurs = 0u8;
                for (l, _) in sh.each().filter(|(l, _)| l.color[3] > 0.0) {
                    if !blurs[..usize::from(n_blurs)].contains(&l.blur) {
                        blurs[usize::from(n_blurs)] = l.blur;
                        n_blurs += 1;
                    }
                }
                let look = crate::text::Look {
                    scale,
                    field: sdf,
                    shadows: blurs,
                    shadow_count: n_blurs,
                    stroke: if st.color[3] > 0.0 {
                        st.width.max(0.0)
                    } else {
                        0.0
                    },
                    ink: [text_ink[0], text_ink[1], text_ink[2]]
                        .map(og_core::color::linear_to_srgb),
                };
                let g = self.texts.paint(
                    fonts,
                    atlas,
                    leaf,
                    drawn,
                    &run,
                    // Columns break at the box's height.
                    if run.vertical { lb.h } else { width },
                    &look,
                    &mut self.missing,
                );
                // A glyph the atlas had no room for draws next frame.
                self.paint |= g.incomplete;
                for &plot in self.texts.plots(leaf) {
                    if !self.plots.contains(&plot) {
                        self.plots.push(plot);
                    }
                }
                let g = self.texts.leaf(leaf).expect("painted");
                if let Some(f) = field
                    && self.input.focus_el() == e as u32
                {
                    if let Some((cs, ce)) = f.composing_display_range() {
                        let line_h = (1.0 * scale).round().max(1.0) / scale;
                        let ucolor = faded(own_ink);
                        g.selection(cs..ce, |[sx, sy, w, h]| {
                            p.quads.push(quad(
                                PaintKind::Solid,
                                [lx + sx, ly + sy + h - line_h, w, line_h],
                                clip,
                                ucolor,
                                [0.; 4],
                                [0.; 4],
                                0,
                                0,
                            ));
                        });
                    } else if f.anchor != f.caret {
                        let (s0, s1) = f.selection_range();
                        let (d0, d1) = (f.display_offset(s0), f.display_offset(s1));
                        let color = faded(s.selection.unwrap_or([0.1, 0.3, 0.8, 0.4]));
                        g.selection(d0..d1, |[sx, sy, w, h]| {
                            p.quads.push(quad(
                                PaintKind::Solid,
                                [lx + sx, ly + sy, w, h],
                                clip,
                                color,
                                [0.; 4],
                                [0.; 4],
                                0,
                                0,
                            ));
                        });
                    }
                }
                if let Some(range) = selected {
                    let color = faded(s.selection.unwrap_or([0.1, 0.3, 0.8, 0.4]));
                    g.selection(range, |[sx, sy, w, h]| {
                        p.quads.push(quad(
                            PaintKind::Solid,
                            [lx + sx, ly + sy, w, h],
                            clip,
                            color,
                            [0.; 4],
                            [0.; 4],
                            0,
                            0,
                        ));
                    });
                }
                let placed = |q: &PaintQuad, dx: f32, dy: f32, color: Rgba| PaintQuad {
                    rect: [
                        q.rect[0] + lx + dx,
                        q.rect[1] + ly + dy,
                        q.rect[2],
                        q.rect[3],
                    ],
                    clip,
                    color,
                    ..*q
                };
                // The shadows lowest, the last layer first as CSS paints
                // them, each a whole number of device pixels off, in its
                // colour over this text's ink.
                for (l, kind) in sh.each().rev().filter(|(l, _)| l.color[3] > 0.0) {
                    let Some(j) = look.blurs().iter().position(|&b| b == l.blur) else {
                        continue;
                    };
                    let mut c = TextShadows::color(l, kind, own_ink);
                    c[3] *= alpha;
                    let [dx, dy] = l.offset.map(|d| match sdf {
                        true => d,
                        false => (d * scale).round() / scale,
                    });
                    p.quads
                        .extend(g.shadow[j].iter().map(|q| placed(q, dx, dy, c)));
                }
                // The outline under the fill: the band the stroke's width
                // wide on each glyph's outline (`-webkit-text-stroke`).
                if look.stroke > 0.0 {
                    let mut c = st.color;
                    c[3] *= alpha;
                    p.quads
                        .extend(g.stroke.iter().map(|q| placed(q, 0.0, 0.0, c)));
                }
                // Each glyph in its span's ink; a text of one run, all in its own.
                let ink_of = |i: u16| match i {
                    0 => text_ink,
                    i => span_looks.get(usize::from(i) - 1).map_or(text_ink, |l| l.0),
                };
                p.quads.extend(
                    g.fill
                        .iter()
                        .zip(g.fill_ink.iter().copied().chain(std::iter::repeat(0)))
                        .map(|(q, i)| placed(q, 0.0, 0.0, ink_of(i))),
                );
                // A paragraph's lines are decorated span by span, each as
                // its run's style says, along the stretches its ink drew.
                let own = {
                    let mut c = own_ink;
                    c[3] *= alpha;
                    c
                };
                // A stretch's decoration and ink: a paragraph's span by
                // span along its inks, any other text's line by line.
                let rich = !span_looks.is_empty();
                let look_of = |i: u16| match i {
                    0 => (el.eff.decoration, own),
                    i => span_looks
                        .get(usize::from(i) - 1)
                        .map_or((el.eff.decoration, own), |&(c, d)| (d, c)),
                };
                // A column's text has no decoration lines yet.
                let decorated = !run.vertical
                    && if rich {
                        g.inks.iter().any(|&(i, _)| look_of(i).0.line != 0)
                    } else {
                        el.eff.decoration.line != 0
                    };
                if decorated {
                    // The lines the primary font gives, each a whole
                    // number of device pixels thick, on one.
                    let face = self.texts.decorations(fonts, leaf);
                    let px = |v: f32| (v * scale).round().max(1.0) / scale;
                    use crate::record::Decoration as D;
                    let g = self.texts.leaf(leaf).expect("painted");
                    let n = if rich { g.inks.len() } else { g.lines.len() };
                    for k in 0..n {
                        let (d, ink, [x0, base, w]) = if rich {
                            let (i, l) = g.inks[k];
                            let (d, c) = look_of(i);
                            (d, c, l)
                        } else {
                            (el.eff.decoration, own, g.lines[k])
                        };
                        if d.line == 0 {
                            continue;
                        }
                        let c = if d.color[3] > 0.0 {
                            let mut c = d.color;
                            c[3] *= alpha;
                            c
                        } else {
                            ink
                        };
                        let at = [
                            (D::UNDERLINE, face.underline),
                            (D::OVERLINE, [face.ascent, face.underline[1]]),
                            (D::LINE_THROUGH, face.through),
                        ];
                        for (bit, [above, thick]) in at {
                            if d.line & bit != 0 {
                                let th = px(if d.thickness > 0.0 {
                                    d.thickness
                                } else {
                                    thick
                                });
                                let top = if let (D::UNDERLINE, Some(offset)) = (bit, d.offset) {
                                    base + offset - th / 2.0
                                } else if bit == D::OVERLINE {
                                    base - above
                                } else {
                                    base - above - th / 2.0
                                };
                                let top = (top * scale).round() / scale;
                                let rect = [lx + x0, ly + top, w, th];
                                decorate(p, rect, clip, c, d.style);
                            }
                        }
                    }
                }
            }
            // The focused `Input`'s caret: a bar its line's height before
            // the character it stands at, blinking at 1 Hz unless reduced
            // motion is active.
            if let Some(f) = field
                && self.input.focus_el() == e as u32
                && (f.anchor == f.caret || !f.composing.is_empty())
                && (self.particles.reduced
                    || !f.composing.is_empty()
                    || ((self.time - f.caret_at).max(0.0) % 1.0) < 0.53)
                && let Some([cx, top, bar, height]) =
                    input::caret_bar(el, f, &self.texts, &self.arena, scale, self.text_scale)
            {
                let snap = |v: f32| (v * scale).round() / scale;
                let rect = [snap(x + cx), snap(y + top), bar, height];
                p.quads.push(quad(
                    PaintKind::Solid,
                    rect,
                    clip,
                    s.caret.map_or(ink, faded),
                    [0.0; 4],
                    [0.0; 4],
                    0,
                    0,
                ));
            }
            if xf != 0 {
                for q in &mut self.drawn.quads[start..] {
                    q.xf = xf;
                }
            }
            // A `<Particles>`' particles over its box, each under the
            // node's transform with its own turn.
            if self.els[e].particles.is_some() {
                self.paint_particles(e, rect, ink, clip, xf);
            }
            // The rounded clip it is drawn in cuts what it drew but a
            // spotlight's shade, which its own hole cuts.
            if pround != 0 {
                for q in &mut self.drawn.quads[start..] {
                    if q.round == 0 {
                        q.round = pround;
                    }
                }
            }
            let (mask, filter) = (self.masks[e], self.filters[e]);
            if mask != 0 || filter != 0 {
                for q in &mut self.drawn.quads[start..] {
                    (q.mask, q.filter) = (mask, filter);
                }
            }
            let blend = self.blends[e];
            if blend != 0 {
                for (i, q) in self.drawn.quads[start..].iter_mut().enumerate() {
                    if Some(start + i) != backdrop && q.blend == 0 {
                        q.blend = u32::from(blend);
                    }
                }
            }
            // A backdrop takes its mask, and its ancestors' colour filters
            // as what is under it would.
            if let Some(at) = backdrop {
                let q = &mut self.drawn.quads[at];
                (q.mask, q.filter) = (outer_mask, outer_filter);
            }
            self.stacking.painted(e, start..self.drawn.quads.len());
        }
        while let Some(o) = open.pop() {
            close_group(
                &mut self.drawn,
                &o,
                screen,
                (!self.stacking.reorders()).then_some(&mut members),
            );
        }
        self.stacking
            .reorder(&mut self.drawn.quads, &mut self.drawn.groups);
        for e in sounding {
            let cue = std::mem::take(&mut self.input.per[e as usize].cue);
            self.sound(e, &cue);
        }
        self.painted_transforms.clone_from(&self.drawn.transforms);
        let mut quads = std::mem::take(&mut self.drawn.quads);
        self.paint_virtual_cursor(&mut quads);
        self.drawn.quads = quads;
        // A screen transition's cover, over everything.
        if let Some((cover, reduced)) = self.cover {
            cover.paint(vw, vh, reduced, &mut self.drawn.quads);
        }
        self.drawn.generation += 1;
    }
}

mod geom;
use geom::*;
pub(super) use geom::{affine, radii};
