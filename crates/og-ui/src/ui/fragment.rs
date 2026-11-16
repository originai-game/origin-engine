//! Fragment links and the highlight overlays.
//!
//! HTML's "scroll to the fragment": a document link's `#id` makes the
//! element with that id `:target` and scrolls its scroll-margin box to the
//! start of its scroll containers. The overlays paint the custom
//! highlights' marks ([`crate::ui::highlight`]) over a text element's own
//! commands.

use super::select::SelectEnd;
use super::*;

impl crate::ui::Tree {
    /// The elements and their ranges a list of passages covers, in the
    /// trees' order: every text element from each passage's first end to
    /// its last.
    pub(crate) fn passage_ranges(
        &self,
        passages: &[(SelectEnd, SelectEnd)],
    ) -> Vec<(usize, Vec<(u32, u32)>)> {
        let o = &self.orders;
        let mut out: Vec<(usize, Vec<(u32, u32)>)> = Vec::new();
        for &((ka, a), (kb, b)) in passages {
            let (Some(ia), Some(ib)) = (self.index.find(ka), self.index.find(kb)) else {
                continue;
            };
            let (pa, pb) = (o.pre_pos[ia], o.pre_pos[ib]);
            if pa == u32::MAX || pb == u32::MAX || pa > pb {
                continue;
            }
            for pos in pa..=pb {
                let i = o.pre[pos as usize];
                let Some((_, lines)) = self.text_source(i) else {
                    continue;
                };
                let n = lines
                    .iter()
                    .map(|l| l.text.chars().count() as u32 + 1)
                    .sum::<u32>()
                    .saturating_sub(1);
                let from = if pos == pa { a.min(n) } else { 0 };
                let to = if pos == pb { b.min(n) } else { n };
                if from >= to {
                    continue;
                }
                match out.iter_mut().find(|(j, _)| *j == i) {
                    Some((_, r)) => r.push((from, to)),
                    None => out.push((i, vec![(from, to)])),
                }
            }
        }
        out
    }
}

impl Ui {
    /// HTML's "scroll to the fragment" for a link naming an element by id:
    /// the first document element with the id, in tree order, scrolled so
    /// its scroll-margin box starts at each scroller's view (`block:
    /// start`, `inline: nearest`). Whether an offset moved.
    pub(crate) fn scroll_to_target_id(&mut self) -> bool {
        let Some(id) = &self.docs.state().target else {
            return false;
        };
        let elements = &self.docs.state().memory.retained.by_key;
        let Some(&i) = self.tree.orders.pre.iter().find(|&&i| {
            self.tree.nodes[i].id == *id && elements.contains_key(&self.tree.nodes[i].key)
        }) else {
            return false;
        };
        let n = &self.tree.nodes[i];
        let (r, m) = (n.placed.rect, n.el.scroll_css.margin);
        let r = Rect::new(
            r.x - m[3],
            r.y - m[0],
            r.width + m[1] + m[3],
            r.height + m[0] + m[2],
        );
        self.interaction.scroll_to_fragment(&self.tree, i, r)
    }
}

impl crate::ui::Interaction {
    /// CSSOM View's scroll into view for a fragment: each scroll container
    /// around node `i`, innermost first, moves so `r` starts at its view
    /// along the block axis, and is shown by the least move across
    /// (`inline: nearest`); the view is inside the container's padding and
    /// scroll-padding, the offset clamped to its range. Whether an offset
    /// moved.
    fn scroll_to_fragment(&mut self, tree: &crate::ui::Tree, i: usize, mut r: Rect) -> bool {
        let mut moved = false;
        let mut c = tree.nodes[i].parent;
        while c != NONE {
            let cn = &tree.nodes[c];
            if cn.el.scroll || cn.el.scroll_x {
                let pad = cn.layout.used.padding;
                let p = cn.placed.rect;
                let (w, h) = (
                    (p.width - pad.horizontal()).max(0.0),
                    (p.height - pad.vertical()).max(0.0),
                );
                let sp = cn.el.scroll_css.padding_px(w, h);
                let view = Rect::new(
                    p.x + pad.left + sp[3],
                    p.y + pad.top + sp[0],
                    (w - sp[1] - sp[3]).max(0.0),
                    (h - sp[0] - sp[2]).max(0.0),
                );
                let near = |lo: f32, hi: f32, vlo: f32, vhi: f32| {
                    if hi - lo > vhi - vlo || lo < vlo {
                        lo - vlo
                    } else if hi > vhi {
                        hi - vhi
                    } else {
                        0.0
                    }
                };
                let d = Vec2::new(near(r.x, r.right(), view.x, view.right()), r.y - view.y);
                let at = self.offset(cn.key);
                let range = Vec2::new(cn.max_scroll_x(), cn.max_scroll()).max(Vec2::ZERO);
                let to = (at + d).clamp(Vec2::ZERO, range);
                let s = &mut self.state;
                if cn.el.scroll_x && to.x != at.x {
                    s.scroll_x.insert(cn.key, to.x);
                    moved = true;
                    r.x -= to.x - at.x;
                }
                if cn.el.scroll && to.y != at.y {
                    s.scroll.insert(cn.key, to.y);
                    moved = true;
                    r.y -= to.y - at.y;
                }
            }
            c = cn.parent;
        }
        moved
    }
}

impl Ui {
    /// The screen rectangles of node `i`'s characters in `marks` (each
    /// `[start, end)` as [`crate::layout::node::NodeMarks::selected`]
    /// counts them), adjacent glyphs merged, and whether its lines run
    /// down (each rectangle a stretch of a column) rather than across.
    fn mark_bands(&self, i: usize, marks: &[(u32, u32)]) -> (Vec<Rect>, bool) {
        let n = &self.tree.nodes[i];
        let hit = |k: usize| {
            marks
                .iter()
                .any(|&(a, b)| (a as usize..b as usize).contains(&k))
        };
        let mut out: Vec<Rect> = Vec::new();
        let mut push = |r: Rect, down: bool| match out.last_mut() {
            Some(l) if !down && (l.y - r.y).abs() < 0.01 && (l.right() - r.x).abs() < 0.01 => {
                l.width = r.right() - l.x;
            }
            Some(l) if down && (l.x - r.x).abs() < 0.01 && (l.bottom() - r.y).abs() < 0.01 => {
                l.height = r.bottom() - l.y;
            }
            _ => out.push(r),
        };
        if let Some(t) = self.text_lines(i) {
            let lh = n.text().map_or(0.0, |tc| tc.line_height() * n.placed.scale);
            for l in &t.lines {
                for b in l.boxes.iter().filter(|b| hit(l.start as usize + b.index)) {
                    let (lo, hi) = (b.x0.min(b.x1), b.x0.max(b.x1));
                    match t.column {
                        Some(w) => push(Rect::new(l.top, l.x + lo, w, hi - lo), true),
                        None => push(Rect::new(l.x + lo, l.top, hi - lo, lh), false),
                    }
                }
            }
            return (out, t.column.is_some());
        }
        let Content::Rich(rc) = &*n.content else {
            return (out, false);
        };
        for p in self.rich_pieces(i) {
            let s = &rc.spans[p.span];
            let r = p.rect;
            if s.atom.is_some() {
                if hit(p.at) {
                    push(r, false);
                }
                continue;
            }
            let size = s.step.size * n.placed.scale;
            match &p.glyphs {
                Some(glyphs) => {
                    let mut x = r.x;
                    for g in glyphs {
                        let w = g.advance * size;
                        if hit(p.at + g.index) {
                            push(Rect::new(x, r.y, w, r.height), false);
                        }
                        x += w;
                    }
                }
                None => {
                    // Drawn as written: each character from where the
                    // text before it ends, kerned as the renderer kerns.
                    let m = &self
                        .env
                        .fonts
                        .for_styled(s.font, s.query(), &p.text)
                        .0
                        .metrics;
                    let tracking = crate::rich::span_tracking(s, &p.text);
                    let mut from = 0.0;
                    for (k, (b, c)) in p.text.char_indices().enumerate() {
                        let to = m.measure_tracked(&p.text[..b + c.len_utf8()], size, tracking);
                        if hit(p.at + k) {
                            push(Rect::new(r.x + from, r.y, to - from, r.height), false);
                        }
                        from = to;
                    }
                }
            }
        }
        (out, false)
    }

    /// A highlight overlay (CSS Pseudo-Elements 4 §3.6) of node `i`'s
    /// characters in `marks`, over the text commands from `under` on (the
    /// text and the overlays below this one): `fill` under each marked
    /// stretch, above the fills already there, and with `style`'s color,
    /// shadow or decoration the marked glyphs drawn again in it, the
    /// commands below cut around them. Where the next overlay's fills go.
    pub(super) fn overlay_commands(
        &self,
        i: usize,
        under: usize,
        marks: &[(u32, u32)],
        style: Option<&crate::doc::generated::Selection>,
        fill: Option<crate::Srgba>,
        cmds: &mut Vec<DrawCmd>,
    ) -> usize {
        let (bands, down) = self.mark_bands(i, marks);
        if bands.is_empty() {
            return under;
        }
        let n = &self.tree.nodes[i];
        let (alpha, sk, dpr) = (n.placed.alpha, n.placed.scale, self.env.viewport.dpr());
        let clip = self.paint_clip(n);
        let fills: Vec<DrawCmd> = fill
            .map(|fill| {
                bands
                    .iter()
                    .map(|r| DrawCmd::Quad {
                        rect: r.snapped(dpr),
                        radius: 0.0,
                        color: fill.fade(alpha).linear(),
                        clip,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let at = under + fills.len();
        cmds.splice(under..under, fills);
        let Some(sel) =
            style.filter(|s| s.color.is_some() || s.decoration.is_some() || s.shadows.is_some())
        else {
            return at;
        };
        // Along the line (x across, y down a column) and across it.
        let along = |r: &Rect| {
            if down {
                (r.y, r.bottom())
            } else {
                (r.x, r.right())
            }
        };
        let meets = |r: &Rect, b: &Rect| {
            if down {
                r.x < b.right() && r.right() > b.x
            } else {
                r.y < b.bottom() && r.bottom() > b.y
            }
        };
        let cut = |cl: Rect, (lo, hi): (f32, f32)| {
            let (a, b) = along(&cl);
            let (lo, hi) = (lo.max(a), hi.min(b));
            (hi > lo).then(|| {
                if down {
                    Rect::new(cl.x, lo, cl.width, hi - lo)
                } else {
                    Rect::new(lo, cl.y, hi - lo, cl.height)
                }
            })
        };
        let text = |c: &DrawCmd| match c {
            DrawCmd::StyledText { text, .. } => match text.as_ref() {
                DrawCmd::Text {
                    bounds,
                    size,
                    baseline,
                    ..
                } => Some((*bounds, *size, *baseline)),
                _ => None,
            },
            DrawCmd::Text {
                bounds,
                size,
                baseline,
                ..
            } => Some((*bounds, *size, *baseline)),
            _ => None,
        };
        let drawn: Vec<DrawCmd> = cmds.drain(at..).collect();
        // The text's own commands, cut where marked stretches cross them.
        for c in &drawn {
            let Some((bounds, ..)) = text(c) else {
                cmds.push(c.clone());
                continue;
            };
            let mut on: Vec<(f32, f32)> = bands
                .iter()
                .filter(|b| meets(b, &bounds))
                .map(along)
                .collect();
            on.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut from = f32::NEG_INFINITY;
            for gap in on.iter().copied().chain([(f32::INFINITY, f32::INFINITY)]) {
                let mut piece = c.clone();
                if let Some(cl) = crate::rich::text_clip(&mut piece)
                    && let Some(r) = cut(*cl, (from, gap.0))
                {
                    *cl = r;
                    cmds.push(piece);
                }
                from = gap.1;
            }
        }
        // The marked stretches again, in its ink and shadows.
        let shadows = |a: f32| -> Option<Vec<og_draw::UiTextShadow>> {
            sel.shadows.as_ref().map(|list| {
                list.iter()
                    .map(|(o, blur, c)| og_draw::UiTextShadow {
                        offset: Vec2::new(o[0] * sk, o[1] * sk),
                        blur: blur * sk,
                        color: c.fade(a).linear(),
                    })
                    .collect()
            })
        };
        for band in &bands {
            let mut line: Option<(f32, f32)> = None;
            for c in &drawn {
                let Some((_, size, baseline)) = text(c).filter(|t| meets(band, &t.0)) else {
                    continue;
                };
                line.get_or_insert((size, baseline));
                let mut c = c.clone();
                let Some(cl) = crate::rich::text_clip(&mut c) else {
                    continue;
                };
                let Some(r) = cut(*cl, along(band)) else {
                    continue;
                };
                *cl = r;
                let (mut inner, effects) = match c {
                    DrawCmd::StyledText { text, effects } => (*text, Some(effects)),
                    c => (c, None),
                };
                let DrawCmd::Text { color, .. } = &mut inner else {
                    continue;
                };
                let a = color[3];
                if let Some(ink) = sel.color {
                    *color = ink.fade(a).linear();
                }
                let effects = match (effects, shadows(a)) {
                    (Some(mut e), Some(sh)) => {
                        e.shadows = sh;
                        Some(e)
                    }
                    (Some(e), None) => Some(e),
                    (None, Some(sh)) => Some(Box::new(og_draw::UiTextEffects {
                        shadows: sh,
                        transform: [1.0, 0.0, 0.0, 1.0],
                        ..Default::default()
                    })),
                    (None, None) => None,
                };
                cmds.push(match effects {
                    Some(effects) => DrawCmd::StyledText {
                        text: Box::new(inner),
                        effects,
                    },
                    None => inner,
                });
            }
            // Its lines along the stretch, where the text's own go.
            let (Some(d), Some((size, baseline))) = (&sel.decoration, line) else {
                continue;
            };
            let m = match &*n.content {
                Content::Text(tc) => {
                    let tc = n.used_text(tc);
                    &self
                        .env
                        .fonts
                        .for_styled(tc.font, tc.query(), &tc.text)
                        .0
                        .metrics
                }
                Content::Rich(rc) => match rc.spans.iter().find(|s| s.atom.is_none()) {
                    Some(s) => {
                        &self
                            .env
                            .fonts
                            .for_styled(s.font, s.query(), &s.text)
                            .0
                            .metrics
                    }
                    None => continue,
                },
                _ => continue,
            };
            let ink = d
                .color
                .or(sel.color)
                .unwrap_or(n.style.sampled.ink)
                .fade(alpha)
                .linear();
            if down {
                let (th, off) = crate::css::text_css::Decoration {
                    under: false,
                    ..d.clone()
                }
                .placed(m, size, sk, dpr);
                let mid = band.x + band.width / 2.0;
                let mut rule = |x: f32| {
                    crate::rich::vertical_decoration_line(
                        x - th / 2.0,
                        band.y,
                        band.height,
                        th,
                        d.style,
                        ink,
                        clip,
                        dpr,
                        cmds,
                    )
                };
                if d.underline {
                    rule(mid - size * 0.5 - off);
                }
                if d.overline {
                    rule(mid + size * 0.6);
                }
                if d.through {
                    rule(mid);
                }
            } else {
                let (th, off) = d.placed(m, size, sk, dpr);
                let mut rule = |y: f32| {
                    crate::rich::decoration_line(
                        band.x, band.width, y, th, d.style, ink, clip, dpr, cmds,
                    )
                };
                if d.underline {
                    rule(baseline + off);
                }
                if d.overline {
                    rule(baseline - size * 0.95);
                }
                if d.through {
                    rule(baseline - size * 0.3);
                }
            }
        }
        at
    }
}
