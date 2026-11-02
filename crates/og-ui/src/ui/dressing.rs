use super::*;

impl Ui {
    /// [`nine`] under the element's map: an upright, unmirrored scale cuts
    /// the slice at its size on the screen (its corners keep their size, as
    /// a hover's lift or a press's squash always drew), then maps the pieces
    /// back into the layout box for [`bake`] to map forward.
    fn nine(
        &self,
        atlas: TextureHandle,
        s: Slice,
        rect: Rect,
        tint: Srgba,
        clip: Rect,
        cmds: &mut Vec<DrawCmd>,
    ) {
        let m = self.drawing;
        let inv = m
            .invert()
            .filter(|_| m.is_axis_aligned() && m.a > 0.0 && m.d > 0.0);
        let Some(inv) = inv.filter(|_| !m.is_identity()) else {
            return nine(atlas, s, rect, tint, clip, cmds);
        };
        let mut pieces = Vec::new();
        nine(
            atlas,
            s,
            m.map_rect(&rect),
            tint,
            m.map_rect(&clip),
            &mut pieces,
        );
        cmds.extend(pieces.into_iter().map(|c| match c {
            DrawCmd::Image {
                rect,
                texture,
                uv,
                color,
                clip,
            } => DrawCmd::Image {
                rect: inv.map_rect(&rect),
                texture,
                uv,
                color,
                clip: inv.map_rect(&clip),
            },
            c => c,
        }));
    }

    /// A styled background: the part's shadow and glow, its nine-slice
    /// frame (tinted by `bg`), the tiled texture and the corner ornaments,
    /// as its dressing (`frame`, resolved by selector) sets them.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn dress(
        &self,
        frame: &crate::dress::DressFrame,
        slice: Slice,
        rect: Rect,
        radius: f32,
        bg: Srgba,
        alpha: f32,
        clip: Rect,
        cmds: &mut Vec<DrawCmd>,
    ) {
        let art = &self.theme.art;
        let Some(atlas) = art.atlas else { return };
        let p = &self.theme.palette;
        let fade = alpha * if slice.tint { 1.0 } else { bg.alpha };
        if let Some(sh) = frame.shadow {
            let r = rect.outset(sh.spread);
            let r = Rect::new(r.x + sh.dx, r.y + sh.dy, r.width, r.height);
            match art.soft.filter(|_| sh.blur > 0.0) {
                Some(soft) => self.nine(
                    atlas,
                    soft,
                    r.outset(sh.blur),
                    sh.color.fade(fade),
                    clip,
                    cmds,
                ),
                None => cmds.push(DrawCmd::Quad {
                    rect: r,
                    radius,
                    color: sh.color.fade(fade).linear(),
                    clip,
                }),
            }
        }
        if let (Some(g), Some(soft)) = (frame.glow, art.soft) {
            let c = g.ink.color(p).fade(g.alpha * fade);
            self.nine(atlas, soft, rect.outset(g.size), c, clip, cmds);
        }
        // Glass: the blurred frame under the part, then its own frame
        // thinned to the glass's fill; under reduced transparency an opaque
        // surface instead, its frame whole.
        let solid = self.theme.reduced_transparency && frame.glass.is_some();
        let glass = frame.glass.filter(|_| !solid);
        let (bg, fade) = if solid {
            (Srgba { alpha: 1.0, ..bg }, alpha)
        } else {
            (bg, fade)
        };
        if let Some(g) = glass {
            let mut tint = bg.linear();
            tint[3] = g.tint;
            cmds.push(DrawCmd::Glass {
                rect,
                radius,
                blur: g.blur,
                tint,
                alpha,
                clip,
            });
        }
        let thin = glass.map_or(1.0, |g| g.fill);
        let tint = if slice.tint {
            bg.fade(alpha * thin)
        } else {
            Srgba::WHITE.fade(fade * thin)
        };
        self.nine(atlas, slice, rect, tint, clip, cmds);
        if let Some(tile) = art.tile.filter(|_| frame.tile) {
            let inner = Rect::new(
                rect.x + tile.inset,
                rect.y + tile.inset,
                (rect.width - 2.0 * tile.inset).max(0.0),
                (rect.height - 2.0 * tile.inset).max(0.0),
            );
            let c = clip.intersect(&inner);
            let (tw, th) = (tile.w.max(4.0), tile.h.max(4.0));
            let mut y = inner.y;
            while y < inner.bottom() {
                let mut x = inner.x;
                while x < inner.right() {
                    cmds.push(DrawCmd::Image {
                        rect: Rect::new(x, y, tw, th),
                        texture: Some(atlas),
                        uv: tile.uv,
                        color: Srgba::WHITE.fade(tile.alpha * fade).linear(),
                        clip: c,
                    });
                    x += tw;
                }
                y += th;
            }
        }
        if let Some(o) = art.corner.filter(|_| frame.corners) {
            let (w, h) = (o.w, o.h);
            let d = o.offset;
            let (l, t, r, b) = (
                rect.x - d,
                rect.y - d,
                rect.right() + d - w,
                rect.bottom() + d - h,
            );
            for (x, y, fx, fy) in [
                (l, t, false, false),
                (r, t, true, false),
                (l, b, false, true),
                (r, b, true, true),
            ] {
                self.ornament(o, Rect::new(x, y, w, h), fx, fy, alpha, clip, cmds);
            }
        }
    }

    /// An ornament in `rect`, mirrored across x and or y.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn ornament(
        &self,
        o: Ornament,
        rect: Rect,
        flip_x: bool,
        flip_y: bool,
        alpha: f32,
        clip: Rect,
        cmds: &mut Vec<DrawCmd>,
    ) {
        let Some(atlas) = self.theme.art.atlas else {
            return;
        };
        let [u0, v0, u1, v1] = o.uv;
        let (u0, u1) = if flip_x { (u1, u0) } else { (u0, u1) };
        let (v0, v1) = if flip_y { (v1, v0) } else { (v0, v1) };
        let color = match o.ink {
            Ink::Authored => Srgba::WHITE,
            ink => ink.color(&self.theme.palette),
        };
        cmds.push(DrawCmd::Image {
            rect,
            texture: Some(atlas),
            uv: [u0, v0, u1, v1],
            color: color.fade(alpha).linear(),
            clip,
        });
    }

    /// What part `part` draws for a widget whose own colour for its state
    /// is `fill`: its dressing, else the tokens (docs/ui.md §12.3), moved
    /// between its states by how far into hover and press it is.
    pub(crate) fn part_draw(
        &self,
        part: &crate::dress::PartRef,
        fill: Option<Srgba>,
    ) -> crate::paint::ElementDraw {
        let t = &self.theme;
        let draw = |p: crate::dress::PartRef| self.dress.resolve(&p).element_draw(&p, t, fill);
        if part.hover <= 0.0 && part.press <= 0.0 {
            return draw(*part);
        }
        let mut d = draw(part.in_state(false, false));
        if part.hover > 0.0 {
            let h = draw(part.in_state(true, false));
            d = crate::dress::mix_draw(t, &d, &h, part.hover.min(1.0));
        }
        if part.press > 0.0 {
            let pr = draw(part.in_state(part.hover >= 0.5, true));
            d = crate::dress::mix_draw(t, &d, &pr, part.press.min(1.0));
        }
        d
    }

    /// What part `part` draws at its centre for `fill` (straight alpha):
    /// its atlas art's tone, its dressing's paints (their mean), or the
    /// fill itself; what a label on it must read against.
    pub(crate) fn part_ground(&self, part: &crate::dress::PartRef, fill: Srgba) -> Srgba {
        let r = self.dress.resolve(part);
        if let (Some(s), Some(_)) = (r.frame.slice, self.theme.art.atlas) {
            return s.drawn(fill).unwrap_or(fill);
        }
        if !r.draw.fills.is_empty() {
            let d = r.element_draw(part, &self.theme, Some(fill));
            return crate::dress::ground_of(&d, &self.theme, Srgba::TRANSPARENT);
        }
        fill
    }

    /// Part `part` drawn in `rect` (a widget's piece: a meter's fill, a
    /// toggle's knob): its atlas art when its dressing has a slice and the
    /// style an atlas, else through the UI pass ([`Ui::part_draw`]).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn part_commands(
        &self,
        part: &crate::dress::PartRef,
        rect: Rect,
        fill: Option<Srgba>,
        alpha: f32,
        clip: Rect,
        cmds: &mut Vec<DrawCmd>,
    ) {
        let t = &self.theme;
        let r = self.dress.resolve(part);
        let radius = crate::dress::corner_of(r.shape(part, t)).1;
        if let (Some(slice), Some(_)) = (r.frame.slice, t.art.atlas) {
            if let Some(bg) = fill {
                self.dress(&r.frame, slice, rect, radius, bg, alpha, clip, cmds);
            }
            return;
        }
        let d = self.part_draw(part, fill);
        // The UI pass's commands keep the screen's clip (`bake`), while an
        // element under a map is drawn in its layout box with its clip taken
        // there: a part inside a moved sheet was cut by a clip left behind.
        let at = crate::paint::Placed {
            rect,
            radius,
            clip: if self.drawing.is_identity() {
                clip
            } else {
                self.drawing.map_rect(&clip)
            },
            alpha,
        };
        cmds.extend(d.enter(t, &at));
        cmds.extend(d.children(&at));
        cmds.extend(d.leave(t, &at));
        self.procedural_ornaments(&r, part, rect, alpha, clip, cmds);
    }

    /// The parts of this frame drawn through the UI pass: every element
    /// with a part (or a version 1 frame) whose dressing has no atlas art,
    /// its draw made from its dressing or the tokens.
    pub(super) fn build_part_draws(&mut self, order: &[usize]) {
        self.part_draws.clear();
        for &i in order {
            if self.draws.contains_key(&i) {
                continue;
            }
            let n = &self.nodes[i];
            // A version 1 frame dresses a background: an element with a
            // frame and none (an anchor, a layout box) draws nothing, as
            // it always has; a part an element names draws its dressing.
            let part = match (n.el.part, n.el.frame) {
                (Some(p), _) => p,
                (None, Some(f)) if n.el.background.is_some() => f.part_ref(),
                _ => continue,
            };
            let r = self.dress.resolve(&part);
            if r.frame.slice.is_some() && self.theme.art.atlas.is_some() {
                continue;
            }
            let d = self.part_draw(&part, n.el.background);
            if d.fills.is_empty()
                && d.strokes.is_empty()
                && d.effects.is_empty()
                && d.layer.is_none()
            {
                continue;
            }
            self.part_draws.insert(i, d);
        }
    }

    /// The ornaments an atlas-free style draws for a part's `ornament`
    /// lines: corner brackets in `lineStrong`, drawn as paths at the
    /// part's corners (an atlas style's ornaments are its art).
    fn procedural_ornaments(
        &self,
        r: &crate::dress::Resolved,
        part: &crate::dress::PartRef,
        rect: Rect,
        alpha: f32,
        clip: Rect,
        cmds: &mut Vec<DrawCmd>,
    ) {
        if self.theme.art.atlas.is_some()
            || !r
                .draw
                .ornaments
                .contains(&crate::dress::OrnamentKind::Corner)
        {
            return;
        }
        let t = &self.theme;
        let len = (t.tokens.space(crate::tokens::SpaceStep::Md))
            .min(rect.width / 4.0)
            .min(rect.height / 4.0);
        if len < 3.0 {
            return;
        }
        let w = t.tokens.stroke(crate::tokens::StrokeWidth::Regular);
        let o = w / 2.0 + t.tokens.space(crate::tokens::SpaceStep::Xxs);
        let ink = if part.has("selected") || part.has("focused") {
            "primary"
        } else {
            "lineStrong"
        };
        for (cx, cy, sx, sy) in [
            (rect.x - o, rect.y - o, 1.0, 1.0),
            (rect.right() + o, rect.y - o, -1.0, 1.0),
            (rect.x - o, rect.bottom() + o, 1.0, -1.0),
            (rect.right() + o, rect.bottom() + o, -1.0, -1.0),
        ] {
            let box_ = Rect::new(cx.min(cx + sx * len), cy.min(cy + sy * len), len, len);
            let pt = |x: f32, y: f32| [x - box_.x, y - box_.y];
            let d = crate::paint::ElementDraw {
                shape: crate::paint::ShapeKind::Path {
                    points: vec![pt(cx, cy + sy * len), pt(cx, cy), pt(cx + sx * len, cy)],
                    closed: false,
                    filled: false,
                    curves: false,
                    width: w,
                    cap: crate::paint::Cap::Square,
                    join: crate::paint::Join::Miter(4.0),
                },
                fills: vec![crate::paint::Paint::Solid(crate::paint::ColorRef::token(
                    ink,
                ))],
                ..Default::default()
            };
            let at = crate::paint::Placed {
                rect: box_,
                radius: 0.0,
                clip,
                alpha,
            };
            cmds.extend(d.enter(t, &at));
        }
    }

    /// What is behind node `i`: its nearest ancestor's fill over the theme's
    /// ground, opaque (a focus ring's gap).
    pub(crate) fn behind(&self, i: usize) -> Srgba {
        let mut fills = Vec::new();
        let mut j = self.nodes[i].parent;
        while j != NONE {
            let nj = &self.nodes[j];
            let part = nj.el.part.or_else(|| nj.el.frame.map(StylePart::part_ref));
            // A dressing whose paints are its own (a gradient panel) is what
            // is behind, as the frame draws it.
            if let Some(p) = part {
                let r = self.dress.resolve(&p);
                let atlas = r.frame.slice.is_some() && self.theme.art.atlas.is_some();
                if !atlas && !r.draw.fills.is_empty() {
                    let d = r.element_draw(&p, &self.theme, nj.el.background);
                    let c = crate::dress::ground_of(&d, &self.theme, Srgba::TRANSPARENT);
                    fills.push(c);
                    if c.alpha >= 1.0 {
                        break;
                    }
                    j = nj.parent;
                    continue;
                }
            }
            if let Some(mut bg) = self.nodes[j].el.background {
                // A styled frame draws its art's centre (`tone`), not the flat fill.
                if let Some(d) = part
                    .and_then(|p| {
                        self.theme.art.atlas?;
                        self.dress.resolve(&p).frame.slice
                    })
                    .and_then(|s| s.drawn(bg))
                {
                    bg = d;
                }
                fills.push(bg);
                if bg.alpha >= 1.0 {
                    break;
                }
            }
            j = self.nodes[j].parent;
        }
        let mut c = self.theme.palette.backdrop;
        for f in fills.iter().rev() {
            c = f.over(c);
        }
        c
    }
}

/// `r` scaled by `sx` and `sy` around its centre.
pub(super) fn scaled_xy(r: Rect, sx: f32, sy: f32) -> Rect {
    let (cx, cy) = r.center();
    Rect::new(
        cx - r.width * sx / 2.0,
        cy - r.height * sy / 2.0,
        r.width * sx,
        r.height * sy,
    )
}

/// The nine pieces of a slice over `rect` (corners keep their size, shrunk
/// together when the rectangle is smaller than two of them); a tiling
/// slice cuts its edges and centre into tiles of the art's middle.
fn nine(
    atlas: TextureHandle,
    s: Slice,
    rect: Rect,
    tint: Srgba,
    clip: Rect,
    cmds: &mut Vec<DrawCmd>,
) {
    if rect.is_empty() {
        return;
    }
    let [l, t, r, b] = s.border;
    let kx = if l + r > rect.width {
        rect.width / (l + r)
    } else {
        1.0
    };
    let ky = if t + b > rect.height {
        rect.height / (t + b)
    } else {
        1.0
    };
    let (l, r, t, b) = (l * kx, r * kx, t * ky, b * ky);
    let xs = [rect.x, rect.x + l, rect.right() - r, rect.right()];
    let ys = [rect.y, rect.y + t, rect.bottom() - b, rect.bottom()];
    let [u0, v0, u1, v1] = s.uv;
    let [bl, bt, br, bb] = s.border_uv;
    let us = [u0, u0 + bl, u1 - br, u1];
    let vs = [v0, v0 + bt, v1 - bb, v1];
    // The middle column's and row's cut of the art. Squeezed more than
    // twice, the middle shows the centre of the art's middle, squeezed
    // twice, rather than all of it. og-render's sprite sampler filters
    // anisotropically where the device can (RenderCapabilities.
    // spriteAnisotropy), so a squeezed edge piece reads the art's own mip
    // level and its border stays as crisp as the corners'; where it cannot,
    // a squeeze past two would sample a level that blurs the border into
    // the fill (a light band across both ends of a button a few px taller
    // than its corners, Rumpus Isle's small Pause), and at two it is one
    // level softer. The corners keep their cut; Repeat cuts its tile.
    let (mu, mv) = if s.fill == SliceFill::Repeat {
        ((us[1], us[2]), (vs[1], vs[2]))
    } else {
        (
            middle_cut(us[1], us[2], xs[2] - xs[1], s.middle[0]),
            middle_cut(vs[1], vs[2], ys[2] - ys[1], s.middle[1]),
        )
    };
    let color = tint.linear();
    for row in 0..3 {
        for col in 0..3 {
            let piece = Rect::new(
                xs[col],
                ys[row],
                xs[col + 1] - xs[col],
                ys[row + 1] - ys[row],
            );
            if piece.width <= 0.0 || piece.height <= 0.0 {
                continue;
            }
            let (ua, ub) = if col == 1 { mu } else { (us[col], us[col + 1]) };
            let (va, vb) = if row == 1 { mv } else { (vs[row], vs[row + 1]) };
            let uv = [ua, va, ub, vb];
            // Corners never tile; an edge tiles along its length, the centre both ways.
            let along_x = col == 1 && s.fill != SliceFill::Stretch;
            let along_y = row == 1 && s.fill != SliceFill::Stretch;
            let cols = tiles(piece.x, piece.width, s.middle[0], s.fill, along_x);
            let rows = tiles(piece.y, piece.height, s.middle[1], s.fill, along_y);
            for &(y, h, fv) in &rows {
                for &(x, w, fu) in &cols {
                    cmds.push(DrawCmd::Image {
                        rect: Rect::new(x, y, w, h),
                        texture: Some(atlas),
                        uv: [
                            uv[0],
                            uv[1],
                            uv[0] + (uv[2] - uv[0]) * fu,
                            uv[1] + (uv[3] - uv[1]) * fv,
                        ],
                        color,
                        clip,
                    });
                }
            }
        }
    }
}

/// The cut `(a, b)` of the art's middle a middle `len` long on screen
/// shows, when the art's middle (`art`, on screen px) would be squeezed
/// into it more than twice: the centre `2 × len` of the art, so the squeeze
/// stays at 2 (continuous with the plain squeeze above it).
fn middle_cut(a: f32, b: f32, len: f32, art: f32) -> (f32, f32) {
    if 2.0 * len >= art {
        return (a, b);
    }
    let (c, h) = ((a + b) / 2.0, (b - a) * len.max(0.0) / art);
    (c - h, c + h)
}

/// The tiles along one axis of a piece starting at `at`, `len` long: each
/// `(start, length, the share of the art's middle it shows)`. Not tiling:
/// one piece showing all of it.
fn tiles(at: f32, len: f32, tile: f32, fill: SliceFill, tiled: bool) -> Vec<(f32, f32, f32)> {
    if !tiled || tile <= 0.0 {
        return vec![(at, len, 1.0)];
    }
    match fill {
        SliceFill::Stretch => vec![(at, len, 1.0)],
        SliceFill::Round => {
            let n = (len / tile).round().clamp(1.0, 4096.0) as usize;
            let w = len / n as f32;
            (0..n).map(|i| (at + i as f32 * w, w, 1.0)).collect()
        }
        SliceFill::Repeat => {
            let n = (len / tile).ceil().clamp(1.0, 4096.0) as usize;
            (0..n)
                .map(|i| {
                    let x = i as f32 * tile;
                    let w = (len - x).min(tile);
                    (at + x, w, w / tile)
                })
                .collect()
        }
    }
}
