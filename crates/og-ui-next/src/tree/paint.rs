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
            let text_stroke = if r.stroked {
                s.text_stroke
            } else {
                from.text_stroke
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
            let group =
                r.effect_count > 0 && room.take(reached([x, y, b.w, b.h], r.effects(), screen));
            if r.effect_count > 0 && !group && !self.warned_layers.contains(&el.node) {
                self.warned_layers.push(el.node);
                let name = &self.input.per[e].name;
                self.warnings.push(format!(
                    "node {}{}{} draws its blur and drop shadows unfiltered: the frame's other filters already take {} layers or {} screens of them",
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
            // drawn under its box, rounded as it is.
            let mut backdrop = None;
            if s.backdrop_blur > 0.0 || r.backdrop_filter.is_some() {
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
            let im = &match &el.image {
                Some(props) => props.over(&s.image, self.textures.as_ref()),
                None => s.image,
            };
            let image = og_draw::TextureHandle::from_slot(im.texture).filter(|_| im.wipe > 0.0);
            // CSS `border-image`: a nine-sliced frame is the box's edge.
            let framed = image.is_some() && im.slice != [0.0; 4];
            // Outer shadows under the box, the last listed lowest.
            let shadows = &s.shadows[..usize::from(s.shadow_count)];
            for sh in shadows
                .iter()
                .rev()
                .filter(|sh| !sh.inset && sh.color[3] > 0.0)
            {
                let p4 = [sh.offset[0], sh.offset[1], sh.blur, sh.spread];
                p.quads.push(quad(
                    PaintKind::Shadow,
                    rect,
                    clip,
                    faded(sh.color),
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
            if let Some(t) = image.filter(|_| framed && o.width > 0.0 && o.color[3] > 0.0) {
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
                        faded(o.color),
                        width.map(|w| w * fit),
                        im.slice,
                        t.to_bits(),
                        og_draw::paint::IMAGE_FILL,
                    )
                });
            }
            // A fill-tinted image takes the fill's colour in its place.
            let fill_tints = image.is_some() && im.tint == crate::record::Tint::Fill;
            if let Some(([hx, hy, hw, hh], hr)) = hole.filter(|_| s.fill[3] > 0.0) {
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
                        faded(s.fill),
                        [0.0; 4],
                        [0.0; 4],
                        0,
                        0,
                    )
                });
            } else if s.fill[3] > 0.0 && !fill_tints {
                p.quads.push(quad(
                    PaintKind::Solid,
                    rect,
                    clip,
                    faded(s.fill),
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
                    crate::record::Tint::Fill => faded(s.fill),
                };
                let flags = (u32::from(im.fill) * og_draw::paint::IMAGE_FILL)
                    | (u32::from(im.tile) * og_draw::paint::IMAGE_TILE);
                let cut = if im.wipe < 1.0 { 1.0 - im.wipe } else { 0.0 };
                // `object-*`: the source at its size in the box, placed
                // and cropped to it.
                let (rect, uv) = match r.object {
                    Some(object) if im.slice == [0.0; 4] => fitted(object, rect, im.uv, im.size),
                    _ => (rect, im.uv),
                };
                p.quads.push(PaintQuad {
                    uv,
                    wipe: [cut, im.wipe_start, f32::from(u8::from(im.wipe_ccw)), 0.0],
                    mask: 0,
                    filter: 0,
                    ..quad(
                        PaintKind::Image,
                        rect,
                        clip,
                        color,
                        width,
                        im.slice,
                        t.to_bits(),
                        flags,
                    )
                });
            }
            for sh in shadows
                .iter()
                .rev()
                .filter(|sh| sh.inset && sh.color[3] > 0.0)
            {
                let p4 = [sh.offset[0], sh.offset[1], sh.blur, sh.spread];
                p.quads.push(quad(
                    PaintKind::InsetShadow,
                    rect,
                    clip,
                    faded(sh.color),
                    radius,
                    p4,
                    0,
                    0,
                ));
            }
            let sides = if s.border_side[3] > 0.0 {
                s.border_sides
            } else {
                0
            };
            // The frame takes the border's place: the border keeps only
            // its room in the layout.
            if !framed
                && (s.border_color[3] > 0.0 || sides != 0)
                && s.border.iter().any(|&w| w > 0.0)
            {
                p.quads.push(PaintQuad {
                    uv: faded(s.border_side),
                    ..quad(
                        PaintKind::Border,
                        rect,
                        clip,
                        faded(s.border_color),
                        radius,
                        s.border,
                        0,
                        u32::from(sides),
                    )
                });
            }
            if !framed && o.width > 0.0 && o.color[3] > 0.0 {
                // The ring of the box grown by the offset and the width; its
                // corners follow the box's, as browsers draw them.
                let out = o.offset + o.width;
                let grown = [x - out, y - out, b.w + 2.0 * out, b.h + 2.0 * out];
                let rr = radius.map(|k| if k > 0.0 { (k + out).max(0.0) } else { 0.0 });
                p.quads.push(quad(
                    PaintKind::Border,
                    grown,
                    clip,
                    faded(o.color),
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
                if r.stroke_width >= 1.0 && n >= 2 && stroke[3] > 0.0 {
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
            // A one-line field's text scrolls inside its content box, with
            // room past its right edge for the caret standing there.
            let scroll = field.filter(|f| !f.multiline).map_or(0.0, |f| f.scroll_x);
            let clip = match field {
                Some(f) if !f.multiline => {
                    let [l, _, r, _] = self.arena.box_of(el.node).inset;
                    cut_to(clip, [x + l, y, x + b.w - r + 2.0, y + b.h])
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
                let (lx, ly) = (snap(x + lb.x - scroll, tx), snap(y + lb.y, ty));
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
            // the character it stands at, in the text's ink.
            if let Some(f) = field
                && self.input.focus_el() == e as u32
                && (f.anchor == f.caret || !f.composing.is_empty())
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

/// A filter group open while its subtree paints: its element, its
/// composite's place in the arena (written by [`close_group`]), its
/// effects in [`Paint::effects`], its element's transform, and its
/// composite's look but the rect.
struct Open {
    e: u32,
    quad: usize,
    first: u32,
    count: u32,
    xf: u32,
    composite: PaintQuad,
}

/// The layers a paint may still draw ([`og_draw::paint::GROUP_LAYERS`])
/// and the area they may cover (CSS px²).
struct Room {
    layers: usize,
    area: f32,
    limit: f32,
}

impl Room {
    /// Takes a layer of `area`, when there is room for it.
    fn take(&mut self, area: f32) -> bool {
        if self.layers >= og_draw::paint::GROUP_LAYERS || self.area + area > self.limit {
            return false;
        }
        self.layers += 1;
        self.area += area;
        true
    }
}

/// How far `effects` spread what they filter, CSS px of the space they
/// are in: three deviations a blur, and a shadow's offset.
fn reach_of(effects: &[PaintEffect]) -> f32 {
    effects
        .iter()
        .map(|f| match *f {
            PaintEffect::Blur(sigma) => 3.0 * sigma,
            PaintEffect::DropShadow { offset, sigma, .. } => {
                3.0 * sigma + (offset[0] * offset[0] + offset[1] * offset[1]).sqrt()
            }
            PaintEffect::Matrix(_) => 0.0,
        })
        .sum()
}

/// The area of the layer a box `rect` (x, y, w, h) filtered by `effects`
/// may take on `screen`: the box grown by their reach, on the screen
/// grown by it.
fn reached(rect: [f32; 4], effects: &[PaintEffect], screen: [f32; 4]) -> f32 {
    let r = reach_of(effects);
    let [x, y, w, h] = rect;
    let w = (x + w + r).min(screen[2] + r) - (x - r).max(screen[0] - r);
    let h = (y + h + r).min(screen[3] + r) - (y - r).max(screen[1] - r);
    w.max(0.0) * h.max(0.0)
}

/// Whether element `e` is drawn under element `a` (its arena descendant).
fn descends(els: &[El], e: u32, a: u32) -> bool {
    let mut c = els[e as usize].arena_parent;
    while c != NONE {
        if c == a {
            return true;
        }
        c = els[c as usize].arena_parent;
    }
    false
}

/// Ends group `o` once everything it holds has painted: its composite
/// covers the screen box of what it holds (each quad's reach, through its
/// transform, inside its clip) grown by its effects' reach (through its
/// element's transform), on what of `screen` it shows in grown as much,
/// and the group is listed. With `members` (no stacking reorder moves its
/// quads: the boxes of the groups the last one listed holds), a group
/// right after the last one listed, with the same effects, transform and
/// composite, and whose box meets none of those, joins it: one layer and
/// one filter run draw them all, as they would alone, since none's
/// effects reach another's box (a row of shadowed icons is one layer, not
/// one each).
fn close_group(p: &mut Paint, o: &Open, screen: [f32; 4], members: Option<&mut Vec<[f32; 4]>>) {
    let mut bounds = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    for q in &p.quads[o.quad + 1..] {
        let [x, y, w, h] = q.rect;
        // A shadow reaches past its box by its offset, spread and three
        // deviations; a line by its stroke; a box's edge by a pixel.
        let g = match q.kind {
            k if k == PaintKind::Shadow as u32 => {
                q.p[0].abs().max(q.p[1].abs()) + q.p[3].max(0.0) + 1.5 * q.p[2] + 1.0
            }
            k if k == PaintKind::Line as u32 => q.p[0] + 1.0,
            k if k == PaintKind::Group as u32
                || k == PaintKind::Image as u32
                || k == PaintKind::Raster as u32 =>
            {
                0.0
            }
            _ => 1.0,
        };
        let [ma, mb, mc, md, me, mf, _, _] = match q.xf {
            0 => og_draw::paint::IDENTITY,
            xf => p.transforms[xf as usize],
        };
        let mut b = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
        for [cx, cy] in [
            [x - g, y - g],
            [x + w + g, y - g],
            [x - g, y + h + g],
            [x + w + g, y + h + g],
        ] {
            let (sx, sy) = (ma * cx + mc * cy + me, mb * cx + md * cy + mf);
            b = [b[0].min(sx), b[1].min(sy), b[2].max(sx), b[3].max(sy)];
        }
        let c = q.clip;
        b = [
            b[0].max(c[0]),
            b[1].max(c[1]),
            b[2].min(c[2]),
            b[3].min(c[3]),
        ];
        if b[0] < b[2] && b[1] < b[3] {
            bounds = [
                bounds[0].min(b[0]),
                bounds[1].min(b[1]),
                bounds[2].max(b[2]),
                bounds[3].max(b[3]),
            ];
        }
    }
    let [ma, mb, mc, md, ..] = match o.xf {
        0 => og_draw::paint::IDENTITY,
        xf => p.transforms[xf as usize],
    };
    // The reach in screen px: the longer of the space's axes.
    let scale = (ma * ma + mb * mb).max(mc * mc + md * md).sqrt();
    let effects = &p.effects[o.first as usize..(o.first + o.count) as usize];
    let r = reach_of(effects) * scale;
    let c = o.composite.clip;
    let shown = [
        c[0].max(screen[0]) - r,
        c[1].max(screen[1]) - r,
        c[2].min(screen[2]) + r,
        c[3].min(screen[3]) + r,
    ];
    let x0 = (bounds[0] - r).max(shown[0]);
    let y0 = (bounds[1] - r).max(shown[1]);
    let x1 = (bounds[2] + r).min(shown[2]);
    let y1 = (bounds[3] + r).min(shown[3]);
    let rect = [x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0)];
    let end = p.quads.len() as u32;
    let mut members = members;
    if let Some(&last) = p.groups.last()
        && let Some(members) = members.as_deref_mut()
        && !last.backdrop
        && last.end == o.quad as u32
        && last.xf == o.xf
        && p.effects[last.first as usize..(last.first + last.count) as usize] == *effects
    {
        let at = last.quad as usize;
        let was = p.quads[at];
        let [ax, ay, aw, ah] = was.rect;
        // Apart when the boxes overlap by no more than a pixel: each box's
        // last pixel is its effects' last tap (3 deviations out, under 1%
        // of the middle's weight) on its content's anti-aliased edge. A
        // blur alone blurs its content too, so one layer differs from two
        // laid over each other by the product of their tails: under half
        // a code value (1/512) while the gap between their contents is 3.5
        // deviations (each tail under 0.044 at its middle), so each box
        // may reach 1.25 deviations into the other's.
        let overlap = 1.0
            + match *effects {
                [PaintEffect::Blur(sigma)] => 2.0 * 1.25 * sigma * scale,
                _ => 0.0,
            };
        let apart = members.iter().all(|&[mx, my, mw, mh]| {
            x1 <= mx + overlap
                || mx + mw <= x0 + overlap
                || y1 <= my + overlap
                || my + mh <= y0 + overlap
        });
        let same = PaintQuad {
            rect: was.rect,
            a: was.a,
            ..o.composite
        } == was;
        if apart && same {
            members.push(rect);
            let (ux, uy) = (ax.min(x0), ay.min(y0));
            p.quads[at].rect = [ux, uy, (ax + aw).max(x1) - ux, (ay + ah).max(y1) - uy];
            // Its own composite draws nothing in the layer they share.
            p.quads[o.quad] = PaintQuad {
                rect: [0.0; 4],
                color: [0.0; 4],
                kind: PaintKind::Solid as u32,
                ..o.composite
            };
            if let Some(g) = p.groups.last_mut() {
                g.end = end;
            }
            return;
        }
    }
    if let Some(members) = members {
        members.clear();
        members.push(rect);
    }
    p.groups.push(PaintGroup {
        quad: o.quad as u32,
        end,
        first: o.first,
        count: o.count,
        xf: o.xf,
        backdrop: false,
    });
    p.quads[o.quad] = PaintQuad {
        rect,
        a: p.groups.len() as u32 - 1,
        ..o.composite
    };
}

/// An image of `size` texels showing `uv` of it, fitted into `rect` as
/// CSS's `object-fit` and `object-position` place it: the part of the box
/// it covers, and the region of the texture drawn there. An unknown size
/// stretches it.
fn fitted(
    (fit, at): (
        nsh_tailwind::ResizeMode,
        nsh_tailwind::style::ObjectPosition,
    ),
    rect: [f32; 4],
    uv: [f32; 4],
    size: [f32; 2],
) -> ([f32; 4], [f32; 4]) {
    use nsh_tailwind::ResizeMode as R;
    let [x, y, w, h] = rect;
    let natural = [
        size[0] * (uv[2] - uv[0]).abs(),
        size[1] * (uv[3] - uv[1]).abs(),
    ];
    if fit == R::Stretch || natural[0] <= 0.0 || natural[1] <= 0.0 || w <= 0.0 || h <= 0.0 {
        return (rect, uv);
    }
    let contain = (w / natural[0]).min(h / natural[1]);
    let scale = match fit {
        R::Contain => contain,
        R::Cover => (w / natural[0]).max(h / natural[1]),
        R::Center => 1.0,
        R::ScaleDown => contain.min(1.0),
        R::Stretch => unreachable!(),
    };
    let (dw, dh) = (natural[0] * scale, natural[1] * scale);
    let dx = at.x.resolve(nsh_tailwind::geometry::Points(w - dw)).0;
    let dy = at.y.resolve(nsh_tailwind::geometry::Points(h - dh)).0;
    // The drawn image's box, then the part of it inside the element's.
    let (ix0, iy0) = (x + dx, y + dy);
    let (x0, y0) = (ix0.max(x), iy0.max(y));
    let (x1, y1) = ((ix0 + dw).min(x + w), (iy0 + dh).min(y + h));
    let along = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let u = |px: f32| along(uv[0], uv[2], (px - ix0) / dw);
    let v = |py: f32| along(uv[1], uv[3], (py - iy0) / dh);
    (
        [x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0)],
        [u(x0), v(y0), u(x1), v(y1)],
    )
}

/// Paints the decoration through the existing solid/polyline primitives.
fn decorate(
    p: &mut Paint,
    rect: [f32; 4],
    clip: [f32; 4],
    color: Rgba,
    style: nsh_tailwind::style::DecorationStyle,
) {
    use nsh_tailwind::style::DecorationStyle::*;
    let [x, y, width, th] = rect;
    let solid = |p: &mut Paint, r, radius| {
        p.quads.push(quad(
            PaintKind::Solid,
            r,
            clip,
            color,
            [radius; 4],
            [0.; 4],
            0,
            0,
        ));
    };
    match style {
        Solid => solid(p, rect, 0.),
        Double => {
            solid(p, [x, y - th, width, th], 0.);
            solid(p, [x, y + th, width, th], 0.);
        }
        Dotted | Dashed => {
            let length = if style == Dotted { th } else { 3. * th };
            let mut at = 0.;
            while at < width {
                solid(
                    p,
                    [x + at, y, length.min(width - at), th],
                    if style == Dotted { th / 2. } else { 0. },
                );
                at += length + th;
            }
        }
        Wavy => {
            let first = p.points.len() as u32;
            let steps = (width / th).ceil() as u32;
            if steps == 0 {
                return;
            }
            for k in 0..=steps {
                let at = width * k as f32 / steps as f32;
                p.points.push([
                    at + th,
                    2. * th + th * og_core::math::sin(at / (4. * th) * std::f32::consts::TAU),
                ]);
            }
            p.quads.push(quad(
                PaintKind::Line,
                [x - th, y - 1.5 * th, width + 2. * th, 4. * th],
                clip,
                color,
                [0.; 4],
                [th, 0., 0., 0.],
                first,
                steps + 1,
            ));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn quad(
    kind: PaintKind,
    rect: [f32; 4],
    clip: [f32; 4],
    color: Rgba,
    radius: [f32; 4],
    p: [f32; 4],
    a: u32,
    b: u32,
) -> PaintQuad {
    PaintQuad {
        rect,
        clip,
        color,
        radius,
        p,
        kind: kind as u32,
        a,
        b,
        xf: 0,
        uv: [0.0; 4],
        wipe: [0.0; 4],
        mask: 0,
        filter: 0,
        round: 0,
        blend: 0,
    }
}

/// Adds the transform of a box `rect` (x, y, w, h) composed onto its
/// parent's `parent` to `p.transforms`: its index, or `parent` when its own
/// is the identity. CSS's order: move to the origin, translate (its
/// percentages of the box's own size), rotate, scale, move back.
fn transformed(p: &mut Paint, parent: u32, t: &crate::record::Transform, rect: [f32; 4]) -> u32 {
    if t.is_identity() {
        return parent;
    }
    if p.transforms.is_empty() {
        p.transforms.push(og_draw::paint::IDENTITY);
    }
    let [a, b, c, d, e, f] = affine(t, rect);
    let [pa, pb, pc, pd, pe, pf, _, _] = p.transforms[parent as usize];
    p.transforms.push([
        pa * a + pc * b,
        pb * a + pd * b,
        pa * c + pc * d,
        pb * c + pd * d,
        pa * e + pc * f + pe,
        pb * e + pd * f + pf,
        0.0,
        0.0,
    ]);
    p.transforms.len() as u32 - 1
}

/// `t` on a box at `rect` as a 2D affine map (a, b, c, d, e, f): a point
/// (x, y) goes to (a x + c y + e, b x + d y + f).
pub(super) fn affine(t: &crate::record::Transform, rect: [f32; 4]) -> [f32; 6] {
    let (sin, cos) = og_core::math::sin_cos(t.rotate);
    let [sx, sy] = t.scale;
    let tan = |r: f32| {
        if r == 0.0 {
            0.0
        } else {
            let (s, c) = og_core::math::sin_cos(r);
            if c.abs() < 1e-6 { 0.0 } else { s / c }
        }
    };
    let (tx, ty) = (tan(t.skew[0]), tan(t.skew[1]));
    let (k00, k01, k10, k11) = ((1.0 + tx * ty) * sx, tx * sy, ty * sx, sy);
    let (a, b, c, d) = (
        cos * k00 - sin * k10,
        sin * k00 + cos * k10,
        cos * k01 - sin * k11,
        sin * k01 + cos * k11,
    );
    let ox = rect[0] + t.origin[0] * rect[2];
    let oy = rect[1] + t.origin[1] * rect[3];
    let e = ox + t.translate[0] + t.percent[0] * rect[2] - (a * ox + c * oy);
    let f = oy + t.translate[1] + t.percent[1] * rect[3] - (b * ox + d * oy);
    [a, b, c, d, e, f]
}

/// Corner radii (top left, top right, bottom right, bottom left) on a
/// `w`×`h` box, all scaled down by one factor when two on a side would
/// overlap, as CSS scales them.
/// Gradient `g` over the box `area` (x y w h, CSS px) as the paint list's:
/// a linear one at its angle (a `to <corner>` one aimed at the box's
/// corner), a radial one's ending shape sized by CSS Images 3's keywords, a
/// conic one from its angle; centres in CSS px from the box's centre.
fn paint_gradient(g: &crate::record::Gradient, area: [f32; 4]) -> og_draw::paint::PaintGradient {
    use crate::record::Shape;
    use nsh_tailwind::effects::GradientSize as Size;
    let [_, _, w, h] = area;
    // A `to <corner>` line runs perpendicular to the diagonal between the
    // other two corners, so its angle is the box's.
    let angle = match g.corner {
        [0, _] | [_, 0] => g.angle,
        [cx, cy] => og_core::math::atan2(f32::from(cx) * h, -f32::from(cy) * w),
    };
    let (cx, cy) = (g.center[0] * w, g.center[1] * h);
    let center = [cx - w / 2.0, cy - h / 2.0];
    // The distances from the centre to the nearer and the farther side.
    let (near, far) = (
        [cx.min(w - cx).abs(), cy.min(h - cy).abs()],
        [cx.max(w - cx).abs(), cy.max(h - cy).abs()],
    );
    let (shape, radii) = match g.shape {
        Shape::Linear => (og_draw::paint::GRADIENT_LINEAR, [0.0; 2]),
        Shape::Conic => (og_draw::paint::GRADIENT_CONIC, [0.0; 2]),
        Shape::Radial { circle, size } => {
            let radii = match (circle, size) {
                (true, Size::ClosestSide) => [near[0].min(near[1]); 2],
                (true, Size::FarthestSide) => [far[0].max(far[1]); 2],
                (true, Size::ClosestCorner) => [(near[0] * near[0] + near[1] * near[1]).sqrt(); 2],
                (true, Size::FarthestCorner) => [(far[0] * far[0] + far[1] * far[1]).sqrt(); 2],
                (false, Size::ClosestSide) => near,
                (false, Size::FarthestSide) => far,
                // The ellipse of the sides' proportions through the corner.
                (false, Size::ClosestCorner) => near.map(|r| r * std::f32::consts::SQRT_2),
                (false, Size::FarthestCorner) => far.map(|r| r * std::f32::consts::SQRT_2),
            };
            (og_draw::paint::GRADIENT_RADIAL, radii)
        }
    };
    og_draw::paint::PaintGradient {
        angle,
        first: 0,
        count: 0,
        shape,
        center,
        radii,
        area,
    }
}

pub(super) fn radii(r: [f32; 4], w: f32, h: f32) -> [f32; 4] {
    let [tl, tr, br, bl] = r.map(|k| k.max(0.0));
    let fit = |side: f32, sum: f32| if sum > side { side / sum } else { 1.0 };
    let f = fit(w, tl + tr)
        .min(fit(w, bl + br))
        .min(fit(h, tl + bl))
        .min(fit(h, tr + br));
    [tl * f, tr * f, br * f, bl * f]
}

/// The right edge of the content box that set `e`'s `text-overflow`
/// (itself or the ancestor it inherits it from), where its text is cut;
/// `None` when that box was not placed this frame.
fn overflow_edge(
    els: &[El],
    abs: &[(f32, f32, bool, [f32; 4])],
    motion: &crate::motion::Motion,
    arena: &Arena,
    e: u32,
) -> Option<f32> {
    let mut a = e;
    while els[a as usize].record.text.set & Text::OVERFLOW == 0 {
        a = els[a as usize].parent;
        if a == NONE {
            return None;
        }
    }
    let &(x, _, shown, _) = abs.get(a as usize)?;
    let node = els[a as usize].node;
    let b = motion.box_of(node).unwrap_or_else(|| arena.box_of(node));
    shown.then_some(x + b.w - arena.box_of(node).inset[2])
}
