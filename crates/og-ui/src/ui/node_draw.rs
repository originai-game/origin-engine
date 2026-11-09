use super::*;

impl Ui {
    /// The style's drop under text of font role `role` (`FontRole::effect_index`)
    /// at `size`: its offset, blur and colour. No more than 6 % of the size
    /// off (at least a device pixel), so a badge's x3 keeps a tight shadow,
    /// not a second print; under a pixel font (`grid` > 0) one of its pixels,
    /// hard, so the cells keep their edges.
    pub(crate) fn text_drop(
        &self,
        role: usize,
        size: f32,
        grid: u32,
    ) -> Option<(f32, f32, f32, crate::Srgba)> {
        let sh = self.theme.art.text_shadows[role]?;
        if grid > 0 {
            let cell = size / grid as f32;
            let step = |v: f32| cell * v.signum() * f32::from(v != 0.0);
            return Some((step(sh.dx), step(sh.dy), 0.0, sh.color));
        }
        let reach = sh.dx.hypot(sh.dy);
        let most = (size * 0.06).max(1.0 / self.viewport.dpr());
        let k = if reach > most { most / reach } else { 1.0 };
        Some((sh.dx * k, sh.dy * k, sh.blur * k, sh.color))
    }

    /// Hollow focus strokes, drawn after the whole subtree of the screen
    /// they are in (`Ui::ring_over`): over the control's children, its
    /// neighbours and the fade and bars of a scroll around it, clipped to
    /// that scroll widened by the ring's reach. No background-coloured cut
    /// can cover art.
    pub(super) fn focus_ring_commands(&self, i: usize, cmds: &mut Vec<DrawCmd>) {
        let t = &self.theme;
        let p = &t.palette;
        let n = &self.nodes[i];
        let rect = n.rect;
        // A ring sits outside its control, so a scroll that clips the
        // control to its box cut the ring of a row as wide as the scroll
        // to its top and bottom: the ring takes the scroll's clip widened
        // by its reach, still inside what clips the scroll itself.
        let pads = if n.pad_rings != 0 { 8.0 } else { 0.0 };
        let reach = (t.space.xxs * pads)
            .max(t.tokens.stroke(crate::tokens::StrokeWidth::Focus) + t.space.xxs);
        let clip = match n.clip_src {
            src if src != NONE && self.drawing.is_identity() => {
                n.clip.outset(reach).intersect(&self.nodes[src].clip)
            }
            _ => n.clip,
        };
        let ring = |player: usize, gap: f32, width: f32, color: Srgba, outline: bool| {
            use crate::paint::{Cap, ElementDraw, Paint, Placed, Stroke, StrokeAlign};
            let mut part = n.el.part.unwrap_or_default();
            part.part = "focus";
            part.class = None;
            part.level = None;
            part = part
                .attr("player", ["0", "1", "2", "3"][player])
                .flag_if(outline, "outline");
            let dressing = self.dress.resolve(&part);
            let mut draw = ElementDraw {
                strokes: vec![Stroke {
                    width,
                    align: StrokeAlign::Outside,
                    dash: [0.0; 2],
                    dash_offset: 0.0,
                    dash_cap: Cap::Butt,
                    paint: Paint::Solid(color.into()),
                }],
                ..ElementDraw::default()
            };
            let mut at = Placed {
                rect: rect.outset(gap),
                radius: n.sampled.radius + gap,
                clip,
                alpha: n.alpha,
            };
            if dressing.matched == 0 {
                return draw.strokes(t, &at);
            }
            let fallback = draw.strokes;
            draw = dressing.element_draw(&part, t, None);
            if dressing.draw.strokes.is_empty()
                && dressing.draw.cleared & crate::dress::CLEAR_STROKES == 0
            {
                draw.strokes = fallback;
                draw.strokes[0].paint = Paint::Solid(dressing.draw.subpart_color(color, t).into());
            }
            at.rect = dressing.draw.subpart_rect(at.rect, rect);
            at.alpha *= dressing.draw.subpart_opacity();
            // The authored ring's complete bounds, including its strokes,
            // may extend beyond the legacy clip reach.
            if n.clip_src != NONE && self.drawing.is_identity() {
                let width = draw.strokes.iter().map(|s| s.width).fold(0.0_f32, f32::max);
                let reach = [
                    rect.x - at.rect.x,
                    rect.y - at.rect.y,
                    at.rect.right() - rect.right(),
                    at.rect.bottom() - rect.bottom(),
                ]
                .into_iter()
                .fold(0.0_f32, f32::max)
                    + width;
                at.clip = n.clip.outset(reach).intersect(&self.nodes[n.clip_src].clip);
            }
            let mut commands = draw.enter(t, &at);
            commands.extend(draw.children(&at));
            commands.extend(draw.leave(t, &at));
            commands
        };
        if n.pad_rings != 0 {
            // Pads 1 to 3 outside pad 0's ring, outermost first.
            let w = t.space.xxs;
            for k in (0..3).rev().filter(|k| n.pad_rings & (1 << k) != 0) {
                let at = w * (2.0 * k as f32 + 3.0);
                let color = [p.info, p.success, p.warn][k];
                cmds.extend(ring(k + 1, at, w, color, false));
            }
        }
        // A bare field (no frame of its own) leaves the ring to its frame.
        let field_ring = match &n.widget {
            Widget::Field(e) | Widget::TextArea { edit: e, .. } => {
                e.focused && n.sampled.background.is_some()
            }
            _ => false,
        };
        let outline = t.outline && (n.nav || n.focusable) && n.sampled.background.is_some();
        if field_ring || n.ring || outline {
            // The style's focus stroke (`stroke { focus … }`): cartoon's
            // 3 px ring stands clear of its 2 px outlines.
            let w = if field_ring || n.ring {
                t.tokens.stroke(crate::tokens::StrokeWidth::Focus)
            } else {
                1.0
            };
            let color = if field_ring || n.ring {
                p.focus
            } else {
                p.line_strong
            };
            let gap = if field_ring || !n.ring {
                0.0
            } else {
                t.space.xxs
            };
            cmds.extend(ring(0, gap, w, color, outline && !field_ring && !n.ring));
        }
    }

    /// The element's background, then (after `between`, what its draw
    /// paints over the background: `docs/ui.md` §3.3) its content and
    /// selection markers, without focus rings. Returns how many commands
    /// the background took.
    pub(super) fn node_commands(
        &self,
        i: usize,
        cmds: &mut Vec<DrawCmd>,
        snap: &dyn Fn(f32) -> f32,
    ) -> usize {
        let start = cmds.len();
        self.background_commands(i, cmds);
        let background = cmds.len() - start;
        self.content_commands(i, cmds, snap);
        background
    }

    /// The element's background (its fill or dressed atlas frame).
    fn background_commands(&self, i: usize, cmds: &mut Vec<DrawCmd>) {
        let t = &self.theme;
        let n = &self.nodes[i];
        let rect = n.rect;
        let clip = n.clip;
        let alpha = n.alpha;
        let quad = |rect: Rect, radius: f32, color: Srgba| DrawCmd::Quad {
            rect,
            radius,
            color: color.fade(alpha).linear(),
            clip,
        };
        // A part the UI pass draws (`part_draws`) has its background there.
        let part = n.el.part.or_else(|| n.el.frame.map(StylePart::part_ref));
        let ornament_tint = part.and_then(|p| {
            self.dress
                .resolve(&p)
                .draw
                .ornament_tint
                .as_ref()
                .map(|c| c.resolve(t))
        });
        let atlas_dressed = part.and_then(|p| {
            t.art.atlas?;
            let r = self.dress.resolve(&p);
            let s = r.frame.slice?;
            Some((p, r.frame, s))
        });
        if let Some(bg) = n
            .sampled
            .background
            .filter(|_| !self.part_draws.contains_key(&i) || atlas_dressed.is_some())
        {
            let (pressed, press) = match n.widget {
                Widget::Button { pressed, press } => (pressed, press),
                _ => (false, 0.0),
            };
            match atlas_dressed {
                Some((_, frame, slice)) => {
                    let r = match (frame.press.or(t.art.press), pressed) {
                        // The squash follows the spring, in and back out.
                        (Some(pr), _) if press != 0.0 => scaled_xy(
                            rect,
                            1.0 + (pr.sx - 1.0) * press,
                            1.0 + (pr.sy - 1.0) * press,
                        ),
                        (None, true) => rect.scaled(t.press_scale),
                        _ => rect,
                    };
                    self.dress(
                        &frame,
                        ornament_tint,
                        slice,
                        r,
                        n.sampled.radius,
                        bg,
                        alpha,
                        clip,
                        cmds,
                    );
                }
                None => {
                    let r = if pressed {
                        rect.scaled(t.press_scale)
                    } else {
                        rect
                    };
                    cmds.push(quad(r, n.sampled.radius, bg));
                }
            }
        }
        // A multi-column container's rules, in the middle of its gaps.
        if let Some(m) = n.el.multicol
            && let Some(line) = m.rule
        {
            let w = line.width;
            let rtl = self.direction == og_i18n::Direction::Rtl;
            for &[x, y, h] in &n.col_rules {
                let x = rect.x + if rtl { rect.width - x } else { x } - w / 2.0;
                let color = if m.current { t.palette.ink } else { line.color };
                let y = rect.y + y;
                let strip = |x: f32, w: f32, cmds: &mut Vec<DrawCmd>| match line.style {
                    crate::decor::LineStyle::Dashed | crate::decor::LineStyle::Dotted => {
                        let on = if line.style == crate::decor::LineStyle::Dashed {
                            3.0 * w
                        } else {
                            w
                        };
                        let mut at = 0.0;
                        while at < h {
                            cmds.push(quad(Rect::new(x, y + at, w, on.min(h - at)), 0.0, color));
                            at += 2.0 * on.max(0.5);
                        }
                    }
                    _ => cmds.push(quad(Rect::new(x, y, w, h), 0.0, color)),
                };
                if line.style == crate::decor::LineStyle::Double && w >= 3.0 {
                    strip(x, w / 3.0, cmds);
                    strip(x + 2.0 * w / 3.0, w / 3.0, cmds);
                } else {
                    strip(x, w, cmds);
                }
            }
        }
    }

    /// The element's content and selection markers (after its background).
    fn content_commands(&self, i: usize, cmds: &mut Vec<DrawCmd>, snap: &dyn Fn(f32) -> f32) {
        let t = &self.theme;
        let p = &t.palette;
        let n = &self.nodes[i];
        let rect = n.rect;
        let clip = n.clip;
        let dpr = self.viewport.dpr();
        let alpha = n.alpha;
        let part = n.el.part.or_else(|| n.el.frame.map(StylePart::part_ref));
        let ornament_tint = part.and_then(|p| {
            self.dress
                .resolve(&p)
                .draw
                .ornament_tint
                .as_ref()
                .map(|c| c.resolve(t))
        });
        if self.kinds[i].0 == "divider"
            && let Some(o) = t.art.divider.filter(|_| t.art.atlas.is_some())
        {
            let (cx, cy) = rect.center();
            self.ornament(
                o,
                Rect::new(cx - o.w / 2.0, cy - o.h / 2.0, o.w, o.h),
                false,
                false,
                ornament_tint,
                alpha,
                clip,
                cmds,
            );
        }
        use crate::dress::PartRef;
        use crate::tokens::ElementClass as C;
        let decoration =
            |part: &PartRef, r: Rect| self.dress.resolve(part).draw.subpart_rect(r, rect);
        let bar = |rect: Rect, value: f32, part: PartRef| {
            let min = self
                .dress
                .resolve(&part)
                .frame
                .slice
                .filter(|_| t.art.atlas.is_some())
                .map_or(rect.height, |s| s.border[0] + s.border[2]);
            let w = (rect.width * value).max(min.min(rect.width));
            Rect::new(rect.x, rect.y, w, rect.height).snapped(dpr)
        };
        if let Widget::Progress { value } = n.widget {
            let track = PartRef::new("Meter", "track", C::Bar);
            let fill = PartRef::new("Meter", "fill", C::Bar).attr("kind", "primary");
            let fills = (p.control_strong, p.primary);
            if !self.segment_commands(&track, &fill, rect, rect, value, fills, alpha, clip, cmds) {
                self.part_commands(
                    &track,
                    decoration(&track, rect),
                    Some(p.control_strong),
                    alpha,
                    clip,
                    cmds,
                );
                if value > 0.0 {
                    let r = bar(rect, value, fill);
                    self.part_commands(
                        &fill,
                        decoration(&fill, r),
                        Some(p.primary),
                        alpha,
                        clip,
                        cmds,
                    );
                }
            }
        }
        if matches!(n.widget, Widget::TextArea { .. }) {
            self.text_area_commands(i, rect, clip, cmds);
        }
        match &n.content {
            Content::Text(tc) if let Some(tp) = &n.props.text_path => {
                self.path_text_commands(i, tc, tp, rect, clip, cmds);
            }
            Content::Text(tc) if let Some(v) = tc.vertical => {
                self.vertical_text_commands(i, tc, v, rect, clip, cmds);
            }
            Content::Text(tc) => {
                let font = self
                    .fonts
                    .for_styled(tc.font, tc.step.weight, false, &tc.text)
                    .0;
                if font.handle.is_some() {
                    let m = &font.metrics;
                    let sk = n.scale;
                    let size = tc.step.size * sk;
                    let lh = tc.line_height() * sk;
                    let pad = n.el.padding;
                    let inner = Rect::new(
                        rect.x + pad.left * sk,
                        rect.y + pad.top * sk,
                        rect.width - pad.horizontal() * sk,
                        rect.height - pad.vertical() * sk,
                    );
                    // Lines shortened around floats (`Node::wrap_lines`):
                    // the last one's drop adds to the block.
                    let drop = n.wrap_lines.last().map_or(0.0, |w| w[1]) * sk;
                    let block = lh * n.lines.len() as f32 + drop;
                    let top = inner.y + ((inner.height - block) / 2.0).max(0.0);
                    // Text overflows its own box (a snapped edge must not eat a glyph)
                    // except in a field, which scrolls its text and clips.
                    let text_clip = match n.widget {
                        Widget::Field { .. } => clip.intersect(&rect),
                        _ => clip,
                    };
                    let field = match &n.widget {
                        Widget::Field(e) => Some(*e),
                        _ => None,
                    };
                    let mut field_x = inner.x;
                    // The first character of each line, for its selection.
                    let mut first = 0u32;
                    for (k, line) in n.lines.iter().enumerate() {
                        // A line a column break moved (`Node::frag`).
                        let [fx, fy] = n.frag_at(k);
                        // Right to left, start is the right edge.
                        let (rtl, para) = self.text_flow(i);
                        // Its line box: the room floats leave it, or its box.
                        let (bx, bw, dy) = match n.wrap_lines.get(k) {
                            Some(&[dx, dy, w]) if rtl => {
                                (inner.x + inner.width - (dx + w) * sk, w * sk, dy * sk)
                            }
                            Some(&[dx, dy, w]) => (inner.x + dx * sk, w * sk, dy * sk),
                            None => (inner.x, inner.width, 0.0),
                        };
                        let ly = top + (k as f32 * lh) + fy * sk + dy;
                        let free = bw - line.width * sk;
                        let x = bx
                            + if rtl { -fx } else { fx } * sk
                            + match (tc.align, rtl) {
                                (TextAlign::Start | TextAlign::Justify, false)
                                | (TextAlign::End, true) => 0.0,
                                (TextAlign::Center, _) => free / 2.0,
                                (TextAlign::End, false)
                                | (TextAlign::Start | TextAlign::Justify, true) => free,
                            };
                        // A focused field: scrolled, its selection under the text.
                        let x = match &field {
                            Some(e) if e.focused && !e.placeholder => {
                                let x = x - e.scroll;
                                self.edit_marks(
                                    i,
                                    &EditMarks {
                                        m,
                                        text: &line.text,
                                        x,
                                        top: ly,
                                        line_h: lh,
                                        size,
                                        tracking: tc.tracking(),
                                        first: 0,
                                        edit: e,
                                        clip: text_clip,
                                        alpha,
                                        under: true,
                                        caret_here: true,
                                    },
                                    cmds,
                                );
                                field_x = x;
                                x
                            }
                            _ => x,
                        };
                        // The player's selection (`user-select`), under the text.
                        if n.selected.is_some() && field.is_none() {
                            self.select_marks(i, &line.text, first, (x, ly, lh), text_clip, cmds);
                        }
                        // `::selection`'s ink, lines and shadows: the line's
                        // selected glyphs drawn again, clipped to their runs.
                        let selection = self.docs.selection.get(&n.key).filter(|s| {
                            s.color.is_some() || s.decoration.is_some() || s.shadows.is_some()
                        });
                        let range = match &field {
                            Some(e) if e.focused && !e.placeholder => Some((
                                e.selection.0.min(e.selection.1),
                                e.selection.0.max(e.selection.1),
                            )),
                            Some(_) => None,
                            None => n.selected.map(|(a, b)| {
                                (
                                    a.saturating_sub(first) as usize,
                                    b.saturating_sub(first) as usize,
                                )
                            }),
                        };
                        let runs = match (selection, range) {
                            (Some(_), Some((a, b))) if a < b => selected_runs(
                                m,
                                &line.text,
                                self.direction,
                                size,
                                tc.tracking(),
                                (a, b),
                            ),
                            _ => Vec::new(),
                        };
                        first += line.text.chars().count() as u32 + 1;
                        // Bidi runs in visual order, shaped runs shaped, one
                        // piece per font of a mixed text (crate::text).
                        // Justified: every line but the last of its paragraph
                        // takes the free room at its spaces.
                        let spaces = line.text.chars().filter(|c| *c == ' ').count();
                        let justify = tc.align == TextAlign::Justify
                            && k + 1 < n.lines.len()
                            && spaces > 0
                            && free > 0.0
                            && !tc.text.contains(&format!("{}\n", line.text));
                        let pieces = if justify {
                            crate::text::pieces_spaced(
                                font,
                                &line.text,
                                para,
                                tc.tracking(),
                                tc.features(),
                                free / spaces as f32 / size.max(1e-3),
                            )
                        } else {
                            crate::text::pieces_with(
                                font,
                                &line.text,
                                para,
                                tc.tracking(),
                                tc.features(),
                            )
                        };
                        let baseline = m.baseline(&line.text, ly, lh, size);
                        let bounds = Rect::new(x, ly, line.width * sk, lh);
                        let role = tc.font.effect_index();
                        let tracking = tc.tracking();
                        // The style's thickening for the role (`textWeight`, device
                        // pixels: what a stroke between two pixels lacks), shadow included.
                        let weight = t.art.text_weights[role] * sk / dpr + tc.emphasis * size;
                        // A style's text glow: the font's distance field, under the glyphs.
                        let glow = t.art.text_glows[role].map(|g| {
                            let c = g.ink.color(p).fade(g.alpha * n.sampled.ink.alpha * alpha);
                            (c.linear(), TEXT_GLOW_REACH * g.size * sk)
                        });
                        let one = pieces.len() == 1;
                        let text = |dx: f32,
                                    dy: f32,
                                    color: Srgba,
                                    glow,
                                    text_clip: Rect,
                                    cmds: &mut Vec<DrawCmd>| {
                            for pc in &pieces {
                                let Some(font) = pc.font else { continue };
                                let ox = pc.dx * size;
                                cmds.push(DrawCmd::Text {
                                    x: snap(x + dx) + ox,
                                    baseline: snap(baseline + dy),
                                    size,
                                    text: pc.text.clone(),
                                    color: color.fade(alpha).linear(),
                                    font,
                                    bounds: Rect::new(
                                        bounds.x + dx + ox,
                                        bounds.y + dy,
                                        if one { bounds.width } else { pc.width * size },
                                        bounds.height,
                                    ),
                                    clip: text_clip,
                                    tracking,
                                    weight,
                                    glow,
                                    placements: pc.placements.clone(),
                                });
                            }
                        };
                        // The style's drop for the role, unless the element
                        // draws its own text effects (a HUD outline and halo):
                        // those wrap every text command, so a style copy under
                        // them printed a second outlined, haloed glyph (art r4,
                        // note 3; r5 on cartoon's numerals). One hard drop, no
                        // more than 6 % of the size off (at least a pixel), so
                        // a badge's x3 keeps a tight shadow, not a second print.
                        let own = self.draw_of(i).is_some_and(|d| d.text.is_some());
                        // A pixel face on a control's plate (a button, a tab,
                        // a chip) or on a shape its parent fills itself (a
                        // coin, a gem) takes no drop: a whole font pixel of it
                        // filled the counters of CJK strokes (pixel r2, note
                        // 3) and haloed a cost on a light coin. Over the scene
                        // it keeps its hard drop.
                        let parent = n.parent;
                        let plated = m.pixel_grid() > 0
                            && (n.el.interactive
                                || (parent != NONE
                                    && (self.nodes[parent].el.interactive
                                        || self
                                            .draws
                                            .get(&parent)
                                            .is_some_and(|d| !d.fills.is_empty()))));
                        let drop = self
                            .text_drop(role, size, m.pixel_grid())
                            .filter(|_| !own && !plated);
                        let dropped = |clip: Rect, cmds: &mut Vec<DrawCmd>| {
                            if let Some((dx, dy, blur, sc)) = drop {
                                let c = sc.fade(n.sampled.ink.alpha);
                                text(dx, dy, c, None, clip, cmds);
                                if blur > 0.0 {
                                    let d = blur / 2.0;
                                    text(dx + d, dy + d, c.fade(0.5), None, clip, cmds);
                                }
                            }
                        };
                        // The unselected stretches, then each selected run.
                        let band = |x0: f32, x1: f32| {
                            Rect::new(x0, text_clip.y, (x1 - x0).max(0.0), text_clip.height)
                                .intersect(&text_clip)
                        };
                        let mut from = text_clip.x;
                        for &(x0, x1) in &runs {
                            let c = band(from, x + x0);
                            dropped(c, cmds);
                            text(0.0, 0.0, n.sampled.ink, glow, c, cmds);
                            from = x + x1;
                        }
                        let rest = band(from, text_clip.right());
                        dropped(rest, cmds);
                        text(0.0, 0.0, n.sampled.ink, glow, rest, cmds);
                        if let Some(sel) = selection.filter(|_| !runs.is_empty()) {
                            let ink = sel
                                .color
                                .map_or(n.sampled.ink, |c| c.fade(n.sampled.ink.alpha));
                            for &(x0, x1) in &runs {
                                let c = band(x + x0, x + x1);
                                let start = cmds.len();
                                // Without its own `text-shadow`, the text's drop.
                                if sel.shadows.is_none() {
                                    dropped(c, cmds);
                                }
                                text(0.0, 0.0, ink, glow, c, cmds);
                                if let Some(shadows) =
                                    sel.shadows.as_ref().filter(|s| !s.is_empty())
                                {
                                    let shadows: Vec<og_render::UiTextShadow> = shadows
                                        .iter()
                                        .map(|(o, blur, color)| og_render::UiTextShadow {
                                            offset: og_core::glam::Vec2::new(o[0] * sk, o[1] * sk),
                                            blur: blur * sk,
                                            color: color.fade(alpha * n.sampled.ink.alpha).linear(),
                                        })
                                        .collect();
                                    for cmd in &mut cmds[start..] {
                                        let t = std::mem::replace(cmd, DrawCmd::PopClip);
                                        *cmd = DrawCmd::StyledText {
                                            text: Box::new(t),
                                            effects: Box::new(og_render::UiTextEffects {
                                                shadows: shadows.clone(),
                                                transform: [1.0, 0.0, 0.0, 1.0],
                                                ..Default::default()
                                            }),
                                        };
                                    }
                                }
                                if let Some(d) = &sel.decoration {
                                    let th = d
                                        .thickness
                                        .map_or((size * 0.07).max(1.0 / dpr), |t| t * sk);
                                    let lc = d.color.unwrap_or(ink).fade(alpha).linear();
                                    let mut rule = |y: f32| {
                                        crate::rich::decoration_line(
                                            x + x0,
                                            x1 - x0,
                                            y,
                                            th,
                                            d.style,
                                            lc,
                                            text_clip,
                                            dpr,
                                            cmds,
                                        )
                                    };
                                    if d.underline {
                                        rule(baseline + size * 0.12);
                                    }
                                    if d.overline {
                                        rule(baseline - size * 0.95);
                                    }
                                    if d.through {
                                        rule(baseline - size * 0.3);
                                    }
                                }
                            }
                        }
                        // Undressed flourishes frame titles at cap height.
                        // A dressing can opt smaller headings in as well.
                        if n.heading
                            && k == 0
                            && let Some(o) = t.art.flourish.filter(|_| t.art.atlas.is_some())
                        {
                            let cy = ly + lh / 2.0;
                            let gap = o.offset + t.space.sm;
                            let h = CAP_HEIGHT * size;
                            let w = o.w / o.h.max(1e-3) * h;
                            let left = Rect::new(x - gap - w, cy - h / 2.0, w, h);
                            let right = Rect::new(x + line.width * sk + gap, cy - h / 2.0, w, h);
                            // Both sides of a centred heading; the side away from the edge otherwise.
                            for (side, rect, flip, visible) in [
                                ("left", left, false, tc.align != TextAlign::Start),
                                ("right", right, true, tc.align != TextAlign::End),
                            ] {
                                let part = crate::dress::PartRef::of("Heading", "flourish", None)
                                    .attr("side", side);
                                let dressing = self.dress.resolve(&part);
                                if !visible
                                    || (tc.step.size < FLOURISH_MIN_TITLE && dressing.matched == 0)
                                {
                                    continue;
                                }
                                let draw = dressing.draw;
                                let tint = if draw.appearance_value("color").is_some()
                                    || draw.ink.is_some()
                                {
                                    Some(draw.subpart_color(Srgba::WHITE, t).linear())
                                } else {
                                    draw.ornament_tint
                                        .as_ref()
                                        .map(|c| c.resolve(t))
                                        .or(ornament_tint)
                                };
                                self.ornament(
                                    o,
                                    draw.subpart_rect(rect, n.rect),
                                    flip,
                                    false,
                                    tint,
                                    alpha * draw.subpart_opacity(),
                                    clip,
                                    cmds,
                                );
                            }
                        }
                    }
                    if let Some(e) = &field
                        && e.focused
                    {
                        // The composition's underline and the caret, over the text.
                        let text = if !e.placeholder { tc.text.as_str() } else { "" };
                        // An empty field's caret sits at its start.
                        let x = if text.is_empty() {
                            if self.text_flow(i).0 {
                                inner.right()
                            } else {
                                inner.x
                            }
                        } else {
                            field_x
                        };
                        let ly = top;
                        self.edit_marks(
                            i,
                            &EditMarks {
                                m,
                                text,
                                x,
                                top: ly,
                                line_h: lh,
                                size,
                                tracking: tc.tracking(),
                                first: 0,
                                edit: e,
                                clip: text_clip,
                                alpha,
                                under: false,
                                caret_here: true,
                            },
                            cmds,
                        );
                    }
                }
            }
            Content::Rich(rc) => self.rich_commands(i, rc, cmds),
            Content::Image {
                texture: Some(texture),
                uv,
                tint,
                object,
            } if *object != crate::layout::ObjectFit::default() => {
                // Fitted and placed in its box by the renderer, which
                // reads the region's own size (CSS object-fit, object-
                // position, image-rendering).
                let color = tint.fade(alpha).linear();
                let mut shape =
                    og_render::UiShape::rect([rect.x, rect.y, rect.width, rect.height], 0.0, color);
                shape.paint = og_render::UiPaint::Image(og_render::UiImage {
                    texture: *texture,
                    uv: *uv,
                    size: og_core::glam::Vec2::ZERO,
                    tint: color,
                    fit: object.fit,
                    focal: object.focal.into(),
                    offset: object.offset.into(),
                    luminance: false,
                    nearest: object.nearest,
                });
                cmds.push(DrawCmd::Shape {
                    shape: Box::new(shape),
                    clip,
                });
            }
            Content::Image {
                texture, uv, tint, ..
            } => cmds.push(DrawCmd::Image {
                rect,
                texture: *texture,
                uv: *uv,
                color: tint.fade(alpha).linear(),
                clip,
            }),
            Content::None => {}
        }
        self.shape_commands(n, cmds);
    }
}

impl Ui {
    fn shape_commands(&self, n: &Node, cmds: &mut Vec<DrawCmd>) {
        let t = &self.theme;
        let clip = n.clip;
        let alpha = n.alpha;
        let dpr = self.viewport.dpr();
        for shape in &n.shapes {
            match shape {
                Shape::Part { frame, part, fill } => {
                    let r = self
                        .dress
                        .resolve(part)
                        .draw
                        .subpart_rect(frame.resolve(&n.rect), n.rect);
                    if !r.is_empty() {
                        self.part_commands(part, r.snapped(dpr), *fill, alpha, clip, cmds);
                    }
                }
                Shape::DressedRect {
                    frame,
                    part,
                    radius,
                    color,
                } if self.dress.resolve(part).matched > 0 => {
                    let draw = self.dress.resolve(part).draw;
                    let r = draw.subpart_rect(frame.resolve(&n.rect), n.rect);
                    if !r.is_empty() {
                        self.part_commands_at(
                            part,
                            r.snapped(dpr),
                            Some(draw.subpart_color(*color, t)),
                            *radius,
                            alpha,
                            clip,
                            cmds,
                        );
                    }
                }
                Shape::Rect {
                    frame,
                    radius,
                    color,
                }
                | Shape::DressedRect {
                    frame,
                    radius,
                    color,
                    ..
                } => {
                    let r = frame.resolve(&n.rect);
                    if r.is_empty() {
                        continue;
                    }
                    cmds.push(DrawCmd::Quad {
                        rect: r.snapped(dpr),
                        radius: radius.min(r.width / 2.0).min(r.height / 2.0) * 1.0,
                        color: color.fade(alpha).linear(),
                        clip,
                    });
                }
                Shape::Bar {
                    part,
                    frame,
                    angle,
                    color,
                } => {
                    let draw = self.dress.resolve(part).draw;
                    let r = draw.subpart_rect(frame.resolve(&n.rect), n.rect);
                    let color = draw.subpart_color(*color, t);
                    let alpha = alpha * draw.subpart_opacity();
                    cmds.push(DrawCmd::Bar {
                        center: r.center(),
                        w: r.width,
                        h: r.height,
                        angle: *angle,
                        color: color.fade(alpha).linear(),
                        clip,
                    });
                }
                Shape::Half {
                    part,
                    frame,
                    angle,
                    color,
                } => {
                    let draw = self.dress.resolve(part).draw;
                    let r = draw.subpart_rect(frame.resolve(&n.rect), n.rect);
                    let color = draw.subpart_color(*color, t);
                    let alpha = alpha * draw.subpart_opacity();
                    cmds.push(DrawCmd::Half {
                        center: r.center(),
                        radius: r.width.min(r.height) / 2.0,
                        angle: *angle,
                        color: color.fade(alpha).linear(),
                        clip,
                    });
                }
                Shape::Arc {
                    part,
                    frame,
                    start,
                    sweep,
                    thickness,
                    color,
                } => {
                    use crate::paint::{Cap, ElementDraw, Paint, Placed, ShapeKind};
                    let dress = self.dress.resolve(part).draw;
                    let rect = dress.subpart_rect(frame.resolve(&n.rect), n.rect);
                    let color = dress.subpart_color(*color, t);
                    let alpha = alpha * dress.subpart_opacity();
                    let draw = ElementDraw {
                        shape: ShapeKind::Arc {
                            start: *start,
                            sweep: *sweep,
                            thickness: *thickness,
                            cap: Cap::Round,
                        },
                        fills: vec![Paint::Solid(color.into())],
                        ..ElementDraw::default()
                    };
                    cmds.extend(draw.enter(
                        &self.theme,
                        &Placed {
                            rect,
                            radius: 0.0,
                            clip,
                            alpha,
                        },
                    ));
                }
                Shape::Image {
                    part,
                    frame,
                    texture,
                    uv,
                    color,
                } => {
                    let draw = self.dress.resolve(part).draw;
                    cmds.push(DrawCmd::Image {
                        rect: draw.subpart_rect(frame.resolve(&n.rect), n.rect),
                        texture: Some(*texture),
                        uv: *uv,
                        color: draw
                            .subpart_color(*color, t)
                            .fade(alpha * draw.subpart_opacity())
                            .linear(),
                        clip,
                    });
                }
                Shape::Text {
                    part,
                    frame,
                    text,
                    step,
                    color,
                    font,
                    clip: within,
                } => {
                    let draw = self.dress.resolve(part).draw;
                    let color = draw.subpart_color(*color, t);
                    let alpha = alpha * draw.subpart_opacity();
                    let mut ramp = t.type_ramp;
                    let step = match draw.appearance_value("step") {
                        Some(crate::doc::Lit::Str(name)) => {
                            crate::style_file::ramp_step(&mut ramp, name)
                                .map(|s| &*s)
                                .unwrap_or(step)
                        }
                        _ => step,
                    };
                    let clip = clip.intersect(&within.resolve(&n.rect));
                    let f = self.fonts.for_text(*font, text);
                    if f.handle.is_none() {
                        continue;
                    }
                    let m = &f.metrics;
                    let r = draw.subpart_rect(frame.resolve(&n.rect), n.rect);
                    // A pixel font on its grid (crate::layout::snap_pixel_text).
                    let size = m.pixel_size(step.size * n.scale);
                    let tracking = m.pixel_tracking(crate::layout::step_tracking(step, text));
                    let features = crate::layout::text_features(step, *font);
                    let w = m.measure_with(text, size, tracking, features);
                    let lh = step.line_height * n.scale;
                    let x = r.x + (r.width - w) / 2.0;
                    let ly = r.y + (r.height - lh) / 2.0;
                    let baseline = m.baseline(text, ly, lh, size);
                    let snap = |v: f32| crate::layout::snap(v, dpr);
                    let pieces =
                        crate::text::pieces_with(f, text, self.direction, tracking, features);
                    let one = pieces.len() == 1;
                    for pc in pieces {
                        let Some(handle) = pc.font else { continue };
                        let ox = pc.dx * size;
                        cmds.push(DrawCmd::Text {
                            x: snap(x) + ox,
                            baseline: snap(baseline),
                            size,
                            text: pc.text,
                            color: color.fade(alpha).linear(),
                            font: handle,
                            bounds: Rect::new(
                                x + ox,
                                ly,
                                if one { w } else { pc.width * size },
                                lh,
                            ),
                            clip,
                            tracking,
                            weight: 0.0,
                            glow: None,
                            placements: pc.placements,
                        });
                    }
                }
                Shape::Glyph {
                    part,
                    frame,
                    font,
                    ch,
                    color,
                } => {
                    let draw = self.dress.resolve(part).draw;
                    let r = draw.subpart_rect(frame.resolve(&n.rect), n.rect);
                    let color = draw.subpart_color(*color, t);
                    let alpha = alpha * draw.subpart_opacity();
                    let size = r.width.min(r.height);
                    if size <= 0.0 {
                        continue;
                    }
                    // The em square: ascender 1, descender 0 (cook::icons).
                    let (x, top) = (r.x + (r.width - size) / 2.0, r.y + (r.height - size) / 2.0);
                    cmds.push(DrawCmd::Text {
                        x,
                        baseline: top + size,
                        size,
                        text: ch.to_string(),
                        color: color.fade(alpha).linear(),
                        font: *font,
                        bounds: Rect::new(x, top, size, size),
                        clip,
                        tracking: 0.0,
                        weight: 0.0,
                        glow: None,
                        placements: Vec::new(),
                    });
                }
            }
        }
    }
}

/// Where an element's part draws: its rectangle, a pressed control's
/// shrunk by the theme's press scale as far into the press as it is.
pub(super) fn part_rect(n: &crate::layout::Node, press_scale: f32) -> Rect {
    match (n.el.part, &n.widget) {
        (Some(p), Widget::Button { .. }) if p.press > 0.0 => {
            n.rect.scaled(1.0 - (1.0 - press_scale) * p.press.min(1.0))
        }
        (None, Widget::Button { pressed: true, .. }) => n.rect.scaled(press_scale),
        _ => n.rect,
    }
}

/// The smallest heading (CSS px) a style's flourish frames.
const FLOURISH_MIN_TITLE: f32 = 28.0;
/// A face's cap height as a share of its size (Latin display faces run
/// 0.66 to 0.72): what a flourish is drawn as tall as.
const CAP_HEIGHT: f32 = 0.7;

impl Ui {
    /// Text on a path (`-og-text-path`, SVG 2 `<textPath>`): each grapheme
    /// cluster drawn on its own, its advance's middle on the path at its
    /// distance along it, turned to the path there, its baseline on the
    /// path; a cluster whose middle is off an open path is not drawn.
    fn path_text_commands(
        &self,
        i: usize,
        tc: &crate::layout::TextContent,
        tp: &crate::text_path::TextPath,
        rect: Rect,
        clip: Rect,
        cmds: &mut Vec<DrawCmd>,
    ) {
        let n = &self.nodes[i];
        let font = self
            .fonts
            .for_styled(tc.font, tc.step.weight, false, &tc.text)
            .0;
        if font.handle.is_none() {
            return;
        }
        let m = &font.metrics;
        let sk = n.scale;
        let size = tc.step.size * sk;
        let tracking = tc.tracking();
        let pad = n.el.padding;
        let origin = Vec2::new(rect.x + pad.left * sk, rect.y + pad.top * sk);
        let inner = Vec2::new(
            (rect.width - pad.horizontal() * sk).max(0.0),
            (rect.height - pad.vertical() * sk).max(0.0),
        );
        let (subs, closed) = tp.polyline(inner / sk.max(1e-3));
        let subs: Vec<Vec<Vec2>> = subs
            .into_iter()
            .map(|s| s.into_iter().map(|p| origin + p * sk).collect())
            .collect();
        let walk = crate::text_path::Walk::new(&subs, closed);
        // One line: its breaks are spaces.
        let line: String = tc.text.replace('\n', " ");
        let chars: Vec<char> = line.chars().collect();
        let boxes = crate::edit::boxes(m, &line, self.direction, size, tracking);
        let width = boxes.iter().map(|b| b.x1).fold(0.0, f32::max);
        let at = if tp.percent {
            tp.offset * walk.length
        } else {
            tp.offset * sk
        };
        let start = at
            - match tc.align {
                TextAlign::Start | TextAlign::Justify => 0.0,
                TextAlign::Center => width / 2.0,
                TextAlign::End => width,
            };
        let t = &self.theme;
        let dpr = self.viewport.dpr();
        let role = tc.font.effect_index();
        let weight = t.art.text_weights[role] * sk / dpr + tc.emphasis * size;
        let color = n.sampled.ink.fade(n.alpha).linear();
        for (a, b) in og_i18n::graphemes(&line) {
            let span = boxes.iter().filter(|g| g.index >= a && g.index < b).fold(
                None,
                |s: Option<(f32, f32)>, g| {
                    Some(s.map_or((g.x0, g.x1), |(x0, x1)| (x0.min(g.x0), x1.max(g.x1))))
                },
            );
            let Some((x0, x1)) = span else { continue };
            let cluster: String = chars[a..b].iter().collect();
            if cluster.trim().is_empty() {
                continue;
            }
            let advance = x1 - x0;
            let Some((p, angle)) = walk.at(start + x0 + advance / 2.0) else {
                continue;
            };
            let (sin, cos) = angle.sin_cos();
            // The pen: half the advance back along the path's direction.
            let pen = p - Vec2::new(cos, sin) * (advance / 2.0);
            let pieces = crate::text::pieces_with(
                font,
                &cluster,
                self.text_flow(i).1,
                tracking,
                tc.features(),
            );
            for pc in pieces {
                let Some(handle) = pc.font.filter(|_| !pc.text.is_empty()) else {
                    continue;
                };
                let ox = pc.dx * size;
                let pen = pen + Vec2::new(cos, sin) * ox;
                let text = DrawCmd::Text {
                    x: pen.x,
                    baseline: pen.y,
                    size,
                    text: pc.text,
                    color,
                    font: handle,
                    bounds: Rect::new(pen.x - size, pen.y - size, advance + 2.0 * size, 2.0 * size),
                    clip,
                    tracking,
                    weight,
                    glow: None,
                    placements: pc.placements,
                };
                cmds.push(DrawCmd::StyledText {
                    text: Box::new(text),
                    effects: Box::new(og_render::UiTextEffects {
                        transform: [cos, sin, -sin, cos],
                        ..Default::default()
                    }),
                });
            }
        }
    }
}

/// A vertical line where it is drawn ([`Ui::vertical_lines`]).
pub(crate) struct VerticalLine {
    /// Its column's left edge; the column is a line height wide.
    pub col: f32,
    /// Each cluster in logical order: its characters (indices in the line)
    /// and its stretch down the line, physical (top, bottom).
    pub glyphs: Vec<(std::ops::Range<usize>, f32, f32)>,
}

impl Ui {
    /// Text in vertical lines (CSS Writing Modes 4, [`crate::text::vertical`]):
    /// its lines stack from its block-start side, each running down (up in
    /// `sideways-lr`) in its runs: an upright cluster centred across the
    /// line in its advance, in its font's vertical form (`vert`) where it
    /// has one, else turned (brackets, dashes) or moved to the em's upper
    /// right (the ideographic comma and full stop); a sideways run drawn as
    /// a horizontal one turned a quarter clockwise (counter-clockwise in
    /// `sideways-lr`), centred across the line; a combined text one em,
    /// side by side, squeezed to fit. A justified line but its paragraph's
    /// last takes its free room at its spaces, else between its upright
    /// clusters (CSS Text 3 §7.1, inter-character for CJK). With `cmds`,
    /// each line's glyph commands, in the style's glow.
    pub(crate) fn vertical_lines(
        &self,
        i: usize,
        tc: &crate::layout::TextContent,
        v: crate::layout::flow::VerticalText,
        rect: Rect,
        clip: Rect,
        mut cmds: Option<&mut Vec<Vec<DrawCmd>>>,
    ) -> Vec<VerticalLine> {
        use crate::layout::flow::WritingMode;
        use crate::text::vertical::runs;
        let n = &self.nodes[i];
        let font = self
            .fonts
            .for_styled(tc.font, tc.step.weight, false, &tc.text)
            .0;
        let m = &font.metrics;
        let sk = n.scale;
        let size = tc.step.size * sk;
        let lh = tc.line_height() * sk;
        let tracking = tc.tracking();
        let features = tc.features();
        let pad = n.el.padding;
        let inner = Rect::new(
            rect.x + pad.left * sk,
            rect.y + pad.top * sk,
            (rect.width - pad.horizontal() * sk).max(0.0),
            (rect.height - pad.vertical() * sk).max(0.0),
        );
        let dpr = self.viewport.dpr();
        let t = &self.theme;
        let role = tc.font.effect_index();
        let weight = t.art.text_weights[role] * sk / dpr + tc.emphasis * size;
        let color = n.sampled.ink.fade(n.alpha).linear();
        // A style's text glow: the font's distance field, under the glyphs.
        let glow = t.art.text_glows[role].map(|g| {
            let c = g
                .ink
                .color(&t.palette)
                .fade(g.alpha * n.sampled.ink.alpha * n.alpha);
            (c.linear(), TEXT_GLOW_REACH * g.size * sk)
        });
        let (rtl, para) = self.text_flow(i);
        let dir = if rtl {
            og_i18n::Direction::Rtl
        } else {
            og_i18n::Direction::Ltr
        };
        let (asc, desc) = m.extents(&tc.text);
        // Half the em box's baseline offset: what centres a glyph about
        // its line's middle.
        let centre = (asc + desc) / 2.0 * size;
        let up_line = v.mode == WritingMode::SidewaysLr;
        // The inline start is the bottom in `sideways-lr`, and the other
        // end right to left.
        let from_bottom = rtl != up_line;
        let block = lh * n.lines.len() as f32;
        let off = ((inner.width - block) / 2.0).max(0.0);
        let emit = |x: f32,
                    baseline: f32,
                    text: String,
                    handle: Option<og_render::FontHandle>,
                    placements: Vec<og_render::GlyphPlacement>,
                    transform: [f32; 4],
                    bounds: Rect,
                    cmds: &mut Vec<DrawCmd>| {
            let Some(handle) = handle.filter(|_| !text.is_empty()) else {
                return;
            };
            let cmd = DrawCmd::Text {
                x,
                baseline,
                size,
                text,
                color,
                font: handle,
                bounds,
                clip,
                tracking,
                weight,
                glow,
                placements,
            };
            if transform == [1.0, 0.0, 0.0, 1.0] {
                cmds.push(cmd);
            } else {
                cmds.push(DrawCmd::StyledText {
                    text: Box::new(cmd),
                    effects: Box::new(og_render::UiTextEffects {
                        transform,
                        ..Default::default()
                    }),
                });
            }
        };
        let drawn = font.handle.is_some();
        let mut out = Vec::with_capacity(n.lines.len());
        for (k, line) in n.lines.iter().enumerate() {
            let mut line_cmds = Vec::new();
            let mut glyphs = Vec::new();
            let col = if v.mode.block_rl() {
                inner.right() - off - (k + 1) as f32 * lh
            } else {
                inner.x + off + k as f32 * lh
            };
            let mid = col + lh / 2.0;
            let length = line.width * sk;
            let free = inner.height - length;
            let lead = match tc.align {
                TextAlign::Start | TextAlign::Justify => 0.0,
                TextAlign::Center => free / 2.0,
                TextAlign::End => free,
            };
            // The top of a stretch `[at, at + len]` along the line.
            let top = |at: f32, len: f32| {
                if from_bottom {
                    inner.bottom() - lead - at - len
                } else {
                    inner.y + lead + at
                }
            };
            let chars_before = |b: usize| line.text[..b].chars().count();
            let line_runs = runs(&line.text, v);
            // Justified: its free room at its spaces, else after each
            // upright cluster but the line's last.
            let spaces = line.text.chars().filter(|c| *c == ' ').count();
            let justify = tc.align == TextAlign::Justify
                && !v.combine
                && k + 1 < n.lines.len()
                && free > 0.0
                && !tc.text.contains(&format!("{}\n", line.text));
            let uprights = if justify && spaces == 0 {
                let all: usize = line_runs
                    .iter()
                    .filter(|(_, up)| *up)
                    .map(|(r, _)| og_i18n::graphemes(&line.text[r.clone()]).len())
                    .sum();
                let last_up = line_runs.last().is_some_and(|(_, up)| *up);
                all.saturating_sub(usize::from(last_up))
            } else {
                0
            };
            let gaps = if spaces > 0 { spaces } else { uprights };
            let extra = if justify && gaps > 0 {
                free / gaps as f32
            } else {
                0.0
            };
            if v.combine {
                // One upright em: the text side by side, squeezed to fit.
                let y0 = top(0.0, size);
                glyphs.push((0..line.text.chars().count(), y0, y0 + size));
                if drawn && cmds.is_some() {
                    let w = m.measure_with(&line.text, size, tracking, features);
                    let squeeze = if w > size { size / w } else { 1.0 };
                    for pc in crate::text::pieces_with(font, &line.text, para, tracking, features) {
                        let x = mid - w * squeeze / 2.0 + pc.dx * size * squeeze;
                        let baseline = y0 + size / 2.0 + centre;
                        emit(
                            x,
                            baseline,
                            pc.text,
                            pc.font,
                            pc.placements,
                            [squeeze, 0.0, 0.0, 1.0],
                            Rect::new(col, y0, lh, size),
                            &mut line_cmds,
                        );
                    }
                }
                out.push(VerticalLine { col, glyphs });
                if let Some(c) = cmds.as_deref_mut() {
                    c.push(line_cmds);
                }
                continue;
            }
            let mut at = 0.0f32;
            let mut upright_left = uprights;
            let mut gap = || {
                if spaces == 0 && upright_left > 0 {
                    upright_left -= 1;
                    extra
                } else {
                    0.0
                }
            };
            let space_extra = if spaces > 0 { extra } else { 0.0 };
            let draw = drawn && cmds.is_some();
            for (range, upright) in line_runs {
                let base = chars_before(range.start);
                let run = &line.text[range];
                at = vertical_run(
                    &VerticalRun {
                        font,
                        size,
                        tracking,
                        features,
                        para,
                        dir,
                        up_line,
                        mid,
                        centre,
                        col,
                        lh,
                        space_extra,
                        draw,
                    },
                    run,
                    upright,
                    at,
                    &top,
                    &mut gap,
                    &mut |chars, y0, y1| {
                        glyphs.push((base + chars.start..base + chars.end, y0, y1))
                    },
                    &mut |x, baseline, text, handle, placements, transform, bounds| {
                        emit(
                            x,
                            baseline,
                            text,
                            handle,
                            placements,
                            transform,
                            bounds,
                            &mut line_cmds,
                        )
                    },
                );
            }
            out.push(VerticalLine { col, glyphs });
            if let Some(c) = cmds.as_deref_mut() {
                c.push(line_cmds);
            }
        }
        out
    }

    /// Vertical text's commands ([`Ui::vertical_lines`]): the player's
    /// selection under it, the style's drop, the glyphs, and the selected
    /// glyphs again in `::selection`'s ink, shadows and lines, each line's
    /// glyphs cut along the line where a selected run starts and ends.
    fn vertical_text_commands(
        &self,
        i: usize,
        tc: &crate::layout::TextContent,
        v: crate::layout::flow::VerticalText,
        rect: Rect,
        clip: Rect,
        cmds: &mut Vec<DrawCmd>,
    ) {
        let n = &self.nodes[i];
        let font = self
            .fonts
            .for_styled(tc.font, tc.step.weight, false, &tc.text)
            .0;
        if font.handle.is_none() {
            return;
        }
        let m = &font.metrics;
        let (sk, alpha, dpr) = (n.scale, n.alpha, self.viewport.dpr());
        let size = tc.step.size * sk;
        let lh = tc.line_height() * sk;
        let mut drawn = Vec::new();
        let lines = self.vertical_lines(i, tc, v, rect, clip, Some(&mut drawn));
        let role = tc.font.effect_index();
        // The style's drop, as a horizontal text's (`content_commands`).
        let own = self.draw_of(i).is_some_and(|d| d.text.is_some());
        let parent = n.parent;
        let plated = m.pixel_grid() > 0
            && (n.el.interactive
                || (parent != NONE
                    && (self.nodes[parent].el.interactive
                        || self.draws.get(&parent).is_some_and(|d| !d.fills.is_empty()))));
        let drop = self
            .text_drop(role, size, m.pixel_grid())
            .filter(|_| !own && !plated);
        let selection = self
            .docs
            .selection
            .get(&n.key)
            .filter(|s| s.color.is_some() || s.decoration.is_some() || s.shadows.is_some());
        // A copy of a line's glyphs moved, in another ink, cut to `band`.
        let copy = |line: &[DrawCmd],
                    (dx, dy): (f32, f32),
                    ink: Option<Srgba>,
                    band: Rect,
                    glow: bool,
                    cmds: &mut Vec<DrawCmd>| {
            for cmd in line {
                let mut cmd = cmd.clone();
                let text = match &mut cmd {
                    DrawCmd::StyledText { text, .. } => text.as_mut(),
                    c => c,
                };
                if let DrawCmd::Text {
                    x,
                    baseline,
                    color,
                    clip,
                    glow: g,
                    bounds,
                    ..
                } = text
                {
                    *x += dx;
                    *baseline += dy;
                    bounds.x += dx;
                    bounds.y += dy;
                    if let Some(ink) = ink {
                        *color = ink.fade(alpha).linear();
                    }
                    *clip = clip.intersect(&band);
                    if !glow {
                        *g = None;
                    }
                }
                cmds.push(cmd);
            }
        };
        let mut first = 0u32;
        for (line, (glyphs, text)) in drawn.iter().zip(lines.iter().zip(&n.lines)) {
            let count = text.text.chars().count() as u32;
            // Its selected runs down the line, physical (top, bottom).
            let mut runs: Vec<(f32, f32)> = Vec::new();
            if let Some((a, b)) = n.selected {
                let (a, b) = (
                    a.saturating_sub(first) as usize,
                    b.saturating_sub(first) as usize,
                );
                let mut picked: Vec<(f32, f32)> = glyphs
                    .glyphs
                    .iter()
                    .filter(|(c, ..)| c.start >= a && c.start < b)
                    .map(|&(_, y0, y1)| (y0, y1))
                    .collect();
                picked.sort_by(|p, q| p.0.total_cmp(&q.0));
                for (y0, y1) in picked {
                    match runs.last_mut() {
                        Some(r) if y0 <= r.1 + 0.01 => r.1 = r.1.max(y1),
                        _ => runs.push((y0, y1)),
                    }
                }
            }
            first += count + 1;
            // The player's selection (`user-select`), under the text.
            if !runs.is_empty() {
                let fill = self
                    .docs
                    .selection
                    .get(&n.key)
                    .and_then(|s| s.background)
                    .unwrap_or(self.theme.palette.control_strong);
                let part = crate::dress::PartRef::of("TextEdit", "selection", None);
                let dressing = self.dress.resolve(&part);
                let across = size + self.theme.space.xxs * 2.0;
                let x0 = glyphs.col + (lh - across) / 2.0;
                for &(y0, y1) in &runs {
                    let r = Rect::new(x0, y0, across, y1 - y0).snapped(dpr);
                    if dressing.matched == 0 {
                        cmds.push(DrawCmd::Quad {
                            rect: r,
                            radius: 0.0,
                            color: fill.fade(alpha).linear(),
                            clip,
                        });
                    } else {
                        self.part_commands_at(
                            &part,
                            dressing.draw.subpart_rect(r, n.rect).snapped(dpr),
                            Some(dressing.draw.subpart_color(fill, &self.theme)),
                            0.0,
                            alpha,
                            clip,
                            cmds,
                        );
                    }
                }
            }
            // The unselected stretches when `::selection` redraws the
            // selected ones, else the whole line.
            let cut = selection.is_some() && !runs.is_empty();
            let mut bands = Vec::new();
            if cut {
                let mut from = clip.y;
                for &(y0, y1) in &runs {
                    bands.push(Rect::new(clip.x, from, clip.width, (y0 - from).max(0.0)));
                    from = y1;
                }
                bands.push(Rect::new(
                    clip.x,
                    from,
                    clip.width,
                    (clip.bottom() - from).max(0.0),
                ));
            } else {
                bands.push(clip);
            }
            let dropped = |band: Rect, cmds: &mut Vec<DrawCmd>| {
                if let Some((dx, dy, blur, sc)) = drop {
                    let c = sc.fade(n.sampled.ink.alpha);
                    copy(line, (dx, dy), Some(c), band, false, cmds);
                    if blur > 0.0 {
                        let d = blur / 2.0;
                        copy(line, (dx + d, dy + d), Some(c.fade(0.5)), band, false, cmds);
                    }
                }
            };
            for &band in &bands {
                dropped(band, cmds);
                copy(line, (0.0, 0.0), None, band, true, cmds);
            }
            let Some(sel) = selection.filter(|_| cut) else {
                continue;
            };
            let ink = sel
                .color
                .map_or(n.sampled.ink, |c| c.fade(n.sampled.ink.alpha));
            for &(y0, y1) in &runs {
                let band = Rect::new(clip.x, y0, clip.width, y1 - y0);
                if sel.shadows.is_none() {
                    dropped(band, cmds);
                }
                let start = cmds.len();
                copy(line, (0.0, 0.0), Some(ink), band, true, cmds);
                if let Some(shadows) = sel.shadows.as_ref().filter(|s| !s.is_empty()) {
                    let shadows: Vec<og_render::UiTextShadow> = shadows
                        .iter()
                        .map(|(o, blur, color)| og_render::UiTextShadow {
                            offset: og_core::glam::Vec2::new(o[0] * sk, o[1] * sk),
                            blur: blur * sk,
                            color: color.fade(alpha * n.sampled.ink.alpha).linear(),
                        })
                        .collect();
                    for cmd in &mut cmds[start..] {
                        match cmd {
                            DrawCmd::StyledText { effects, .. } => {
                                effects.shadows = shadows.clone();
                            }
                            c => {
                                let t = std::mem::replace(c, DrawCmd::PopClip);
                                *c = DrawCmd::StyledText {
                                    text: Box::new(t),
                                    effects: Box::new(og_render::UiTextEffects {
                                        shadows: shadows.clone(),
                                        transform: [1.0, 0.0, 0.0, 1.0],
                                        ..Default::default()
                                    }),
                                };
                            }
                        }
                    }
                }
                // Lines down the column: over on its right (line-over),
                // under on its left, through its middle (CSS Text
                // Decoration 4 §2.3); a solid rule whatever the style.
                if let Some(d) = &sel.decoration {
                    let th = d.thickness.map_or((size * 0.07).max(1.0 / dpr), |t| t * sk);
                    let lc = d.color.unwrap_or(ink).fade(alpha).linear();
                    let mid = glyphs.col + lh / 2.0;
                    let mut rule = |x: f32| {
                        cmds.push(DrawCmd::Quad {
                            rect: Rect::new(x - th / 2.0, y0, th, y1 - y0).snapped(dpr),
                            radius: 0.0,
                            color: lc,
                            clip,
                        });
                    };
                    if d.underline {
                        rule(mid - size * 0.6);
                    }
                    if d.overline {
                        rule(mid + size * 0.6);
                    }
                    if d.through {
                        rule(mid);
                    }
                }
            }
        }
    }
}

/// What one run of a vertical line is drawn with ([`vertical_run`]).
pub(crate) struct VerticalRun<'a> {
    pub font: &'a crate::ui::UiFont,
    pub size: f32,
    pub tracking: f32,
    pub features: crate::shape::Features,
    pub para: crate::text::Para,
    pub dir: og_i18n::Direction,
    /// The line runs up (`sideways-lr`).
    pub up_line: bool,
    /// The line's middle across, and the offset that centres a glyph's
    /// em box about it.
    pub mid: f32,
    pub centre: f32,
    /// The line's column (its left edge and width).
    pub col: f32,
    pub lh: f32,
    /// Added to each space (a justified line's room).
    pub space_extra: f32,
    /// Emit glyphs (else only measure where each cluster lies).
    pub draw: bool,
}

/// Takes a vertical run's glyph command: pen, baseline, text, font,
/// placements, transform and bounds.
pub(crate) type GlyphEmit<'a> = dyn FnMut(
        f32,
        f32,
        String,
        Option<og_render::FontHandle>,
        Vec<og_render::GlyphPlacement>,
        [f32; 4],
        Rect,
    ) + 'a;

/// One run of a vertical line from `at` along it (`top(at, len)` the
/// physical top of a stretch): an upright run cluster by cluster at its
/// vertical advance, in its vertical form where the font has one, else
/// turned or moved to the em's corner; a sideways run shaped as a
/// horizontal one turned. `gap` is what follows each upright cluster;
/// `glyph` takes each cluster's characters (in the run) and stretch,
/// `emit` each glyph command (pen, baseline, text, font, placements,
/// transform, bounds). Returns where the line goes on.
#[allow(clippy::too_many_arguments)]
pub(crate) fn vertical_run(
    r: &VerticalRun<'_>,
    run: &str,
    upright: bool,
    mut at: f32,
    top: &dyn Fn(f32, f32) -> f32,
    gap: &mut dyn FnMut() -> f32,
    glyph: &mut dyn FnMut(std::ops::Range<usize>, f32, f32),
    emit: &mut GlyphEmit<'_>,
) -> f32 {
    use crate::shape::Features;
    use crate::text::vertical::{Fallback, fallback, upright_advance};
    const TURN: [f32; 4] = [0.0, 1.0, -1.0, 0.0];
    const TURN_BACK: [f32; 4] = [0.0, -1.0, 1.0, 0.0];
    const STRAIGHT: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
    let (font, size, tracking, features) = (r.font, r.size, r.tracking, r.features);
    let m = &font.metrics;
    let (mid, centre, col, lh) = (r.mid, r.centre, r.col, r.lh);
    let chars: Vec<char> = run.chars().collect();
    if !upright {
        let e = r.space_extra;
        let spaced = chars.iter().filter(|c| **c == ' ').count() as f32;
        let len = m.measure_with(run, size, tracking, features) + e * spaced;
        let y0 = top(at, len);
        // Its clusters down the line, each space widened.
        let mut widened = 0.0;
        for b in crate::edit::boxes(m, run, r.dir, size, tracking) {
            let own = if chars.get(b.index) == Some(&' ') {
                e
            } else {
                0.0
            };
            let (lo, hi) = (b.x0 + widened, b.x1 + widened + own);
            widened += own;
            let (a, z) = if r.up_line {
                (y0 + len - hi, y0 + len - lo)
            } else {
                (y0 + lo, y0 + hi)
            };
            glyph(b.index..crate::edit::next_cluster(&chars, b.index), a, z);
        }
        if r.draw {
            let pieces = if e > 0.0 {
                crate::text::pieces_spaced(
                    font,
                    run,
                    r.para,
                    tracking,
                    features,
                    e / size.max(1e-3),
                )
            } else {
                crate::text::pieces_with(font, run, r.para, tracking, features)
            };
            for pc in pieces {
                let d = pc.dx * size;
                let (x, y, t) = if r.up_line {
                    (mid + centre, y0 + len - d, TURN_BACK)
                } else {
                    (mid - centre, y0 + d, TURN)
                };
                emit(
                    x,
                    y,
                    pc.text,
                    pc.font,
                    pc.placements,
                    t,
                    Rect::new(col, y0, lh, len),
                );
            }
        }
        return at + len;
    }
    let forms = m
        .file
        .as_ref()
        .filter(|_| !m.is_mixed() && r.draw)
        .and_then(|f| f.vertical_forms(run, &|key| m.in_atlas_key(key)));
    let bytes: Vec<usize> = run.char_indices().map(|(b, _)| b).collect();
    for (a, b) in og_i18n::graphemes(run) {
        let cluster: String = chars[a..b].iter().collect();
        let after = gap();
        if cluster.trim().is_empty() {
            let e = if cluster.contains(' ') {
                r.space_extra
            } else {
                0.0
            };
            let adv = m.measure_with(&cluster, size, tracking, Features::NONE) + e;
            let y0 = top(at, adv);
            glyph(a..b, y0, y0 + adv);
            at += adv + after;
            continue;
        }
        let adv = (upright_advance(m, &cluster) + tracking) * size;
        let y0 = top(at, adv);
        glyph(a..b, y0, y0 + adv);
        at += adv + after;
        if !r.draw {
            continue;
        }
        let bounds = Rect::new(col, y0, lh, adv);
        let wh = m.measure_with(&cluster, size, 0.0, Features::NONE);
        let form = forms.as_ref().and_then(|f| {
            f.iter()
                .find(|(c, _)| *c as usize == bytes[a])
                .and_then(|(_, g)| *g)
        });
        let (x, baseline) = (mid - wh / 2.0, y0 + adv / 2.0 + centre);
        if let Some(id) = form.filter(|_| b - a == 1) {
            let placement = og_render::GlyphPlacement {
                advance: wh / size.max(1e-3),
                x: 0.0,
                y: 0.0,
                glyph: Some(id),
            };
            emit(
                x,
                baseline,
                cluster,
                font.handle,
                vec![placement],
                STRAIGHT,
                bounds,
            );
            continue;
        }
        for pc in crate::text::pieces_with(font, &cluster, r.para, tracking, features) {
            let ox = pc.dx * size;
            match fallback(chars[a]) {
                Fallback::Turn => {
                    // About the glyph's centre: its pen moves so the
                    // turned centre lands on the em's.
                    let p = (mid - centre, y0 + adv / 2.0 - wh / 2.0 + ox);
                    emit(p.0, p.1, pc.text, pc.font, pc.placements, TURN, bounds);
                }
                Fallback::Corner => emit(
                    x + ox + wh / 2.0,
                    baseline - size / 2.0,
                    pc.text,
                    pc.font,
                    pc.placements,
                    STRAIGHT,
                    bounds,
                ),
                Fallback::Plain => emit(
                    x + ox,
                    baseline,
                    pc.text,
                    pc.font,
                    pc.placements,
                    STRAIGHT,
                    bounds,
                ),
            }
        }
    }
    at
}

/// The x ranges, from the line's start, of the glyphs of `line` whose
/// characters fall in `range` (line-relative), adjacent glyphs merged.
fn selected_runs(
    m: &crate::text::FontMetrics,
    line: &str,
    dir: og_i18n::Direction,
    size: f32,
    tracking: f32,
    (a, b): (usize, usize),
) -> Vec<(f32, f32)> {
    let mut runs: Vec<(f32, f32)> = Vec::new();
    for g in crate::edit::boxes(m, line, dir, size, tracking) {
        if g.index < a || g.index >= b {
            continue;
        }
        match runs.last_mut() {
            Some(r) if (r.1 - g.x0).abs() < 0.01 => r.1 = g.x1,
            _ => runs.push((g.x0, g.x1)),
        }
    }
    runs
}
