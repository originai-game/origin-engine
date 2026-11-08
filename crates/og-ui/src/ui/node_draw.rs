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
                    let block = lh * n.lines.len() as f32;
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
                        let ly = top + k as f32 * lh;
                        let free = inner.width - line.width * sk;
                        // Right to left, start is the right edge.
                        let rtl = self.direction == og_i18n::Direction::Rtl;
                        let x = inner.x
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
                                self.direction,
                                tc.tracking(),
                                tc.features(),
                                free / spaces as f32 / size.max(1e-3),
                            )
                        } else {
                            crate::text::pieces_with(
                                font,
                                &line.text,
                                self.direction,
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
                        let text =
                            |dx: f32, dy: f32, color: Srgba, glow, cmds: &mut Vec<DrawCmd>| {
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
                        if let Some((dx, dy, blur, sc)) = self
                            .text_drop(role, size, m.pixel_grid())
                            .filter(|_| !own && !plated)
                        {
                            let c = sc.fade(n.sampled.ink.alpha);
                            text(dx, dy, c, None, cmds);
                            if blur > 0.0 {
                                text(dx + blur / 2.0, dy + blur / 2.0, c.fade(0.5), None, cmds);
                            }
                        }
                        text(0.0, 0.0, n.sampled.ink, glow, cmds);
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
                            if self.direction == og_i18n::Direction::Rtl {
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
            Content::Image { texture, uv, tint } => cmds.push(DrawCmd::Image {
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
            let pieces =
                crate::text::pieces_with(font, &cluster, self.direction, tracking, tc.features());
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
