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

    /// An element's focus rings (pads 1 to 3, the focus ring or the
    /// style's outline), each cut from the control by what is behind it.
    /// Drawn under the element's fills: before a draw's (a game's or a
    /// dressed part's), which the cut would otherwise cover.
    pub(super) fn focus_ring_commands(&self, i: usize, cmds: &mut Vec<DrawCmd>) {
        let t = &self.theme;
        let p = &t.palette;
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
        if n.pad_rings != 0 {
            // Pads 1 to 3: a ring each in the player's colour, outside where
            // pad 0's ring goes, outermost first (each cut from the next by
            // what is behind).
            let behind = self.behind(i);
            let w = t.space.xxs;
            for k in (0..3).rev().filter(|k| n.pad_rings & (1 << k) != 0) {
                let at = w * (2.0 * k as f32 + 3.0);
                let color = [p.info, p.success, p.warn][k];
                cmds.push(quad(rect.outset(at + w), n.el.radius + at + w, color));
                cmds.push(quad(rect.outset(at), n.el.radius + at, behind));
            }
        }
        // A bare field (no frame of its own) leaves the ring to its frame.
        let field_ring = match &n.widget {
            Widget::Field(e) | Widget::TextArea { edit: e, .. } => {
                e.focused && n.el.background.is_some()
            }
            _ => false,
        };
        let outline = t.outline && (n.nav || n.focusable) && n.el.background.is_some();
        if field_ring || n.ring || outline {
            // A ring, a gap of what is behind, then the control.
            let behind = self.behind(i);
            let w = if field_ring || n.ring {
                t.space.xxs
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
            cmds.push(quad(rect.outset(w + gap), n.el.radius + w + gap, color));
            // Cut always: a field's styled frame is not wholly opaque, and
            // the ring under it tinted its text's ground (the scifi pack's
            // focused placeholder read 3.5:1 on primary-tinted navy).
            cmds.push(quad(rect.outset(gap), n.el.radius + gap, behind));
        }
    }

    /// An element's commands; its focus rings too with `ring` (an element
    /// with a draw has them drawn before its fills, [`Self::focus_ring_commands`]).
    pub(super) fn node_commands(
        &self,
        i: usize,
        cmds: &mut Vec<DrawCmd>,
        snap: &dyn Fn(f32) -> f32,
        ring: bool,
    ) {
        let t = &self.theme;
        let p = &t.palette;
        let n = &self.nodes[i];
        let rect = n.rect;
        let clip = n.clip;
        let dpr = self.viewport.dpr();
        let alpha = n.alpha;
        let quad = |rect: Rect, radius: f32, color: Srgba| DrawCmd::Quad {
            rect,
            radius,
            color: color.fade(alpha).linear(),
            clip,
        };
        if ring {
            self.focus_ring_commands(i, cmds);
        }
        // A part the UI pass draws (`part_draws`) has its background there.
        let part = n.el.part.or_else(|| n.el.frame.map(StylePart::part_ref));
        let atlas_dressed = part.and_then(|p| {
            t.art.atlas?;
            let r = self.dress.resolve(&p);
            let s = r.frame.slice?;
            Some((p, r.frame, s))
        });
        if let Some(bg) =
            n.el.background
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
                    self.dress(&frame, slice, r, n.el.radius, bg, alpha, clip, cmds);
                }
                None => {
                    let r = if pressed {
                        rect.scaled(t.press_scale)
                    } else {
                        rect
                    };
                    cmds.push(quad(r, n.el.radius, bg));
                }
            }
        }
        if self.kinds[i].0 == "divider"
            && let Some(o) = t.art.divider.filter(|_| t.art.atlas.is_some())
        {
            let (cx, cy) = rect.center();
            self.ornament(
                o,
                Rect::new(cx - o.w / 2.0, cy - o.h / 2.0, o.w, o.h),
                false,
                false,
                alpha,
                clip,
                cmds,
            );
        }
        use crate::dress::PartRef;
        use crate::tokens::ElementClass as C;
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
        match n.widget {
            Widget::Progress { value } => {
                let track = PartRef::new("Meter", "track", C::Bar);
                self.part_commands(&track, rect, Some(p.control_strong), alpha, clip, cmds);
                if value > 0.0 {
                    let fill = PartRef::new("Meter", "fill", C::Bar).attr("kind", "primary");
                    let r = bar(rect, value, fill);
                    self.part_commands(&fill, r, Some(p.primary), alpha, clip, cmds);
                }
            }
            Widget::Slider { t: v, active } => {
                let thumb = self.thumb();
                let track_h = t.space.xs;
                let cy = rect.y + rect.height / 2.0;
                let track = Rect::new(rect.x, cy - track_h / 2.0, rect.width, track_h).snapped(dpr);
                let tp = PartRef::new("Slider", "track", C::Bar);
                self.part_commands(&tp, track, Some(p.control_strong), alpha, clip, cmds);
                let tx = rect.x + v * (rect.width - thumb).max(0.0);
                let fp = PartRef::new("Slider", "fill", C::Bar);
                let filled =
                    Rect::new(rect.x, track.y, tx + thumb / 2.0 - rect.x, track_h).snapped(dpr);
                self.part_commands(&fp, filled, Some(p.primary), alpha, clip, cmds);
                let mut knob = Rect::new(tx, cy - thumb / 2.0, thumb, thumb);
                if active {
                    knob = knob.scaled(t.press_scale);
                }
                let kp = PartRef::new("Slider", "knob", C::Chip)
                    .flag("round")
                    .flag_if(active, "pressed");
                let knob = self.piece_rect(&kp, knob);
                self.part_commands(&kp, knob.snapped(dpr), Some(p.knob), alpha, clip, cmds);
            }
            Widget::Toggle { on, slide } => {
                let (sw, sh) = self.switch_size();
                let track =
                    Rect::new(rect.x, rect.y + (rect.height - sh) / 2.0, sw, sh).snapped(dpr);
                let tp = PartRef::new("Toggle", "track", C::Chip)
                    .flag("round")
                    .flag(if on { "on" } else { "off" });
                let color = mix_srgba(p.control_strong, p.primary, slide);
                self.part_commands(&tp, track, Some(color), alpha, clip, cmds);
                let inset = t.space.xxs;
                let k = sh - 2.0 * inset;
                let kx = track.x + inset + (track.width - 2.0 * inset - k) * slide;
                let kp = PartRef::new("Toggle", "knob", C::Chip)
                    .flag("round")
                    .flag(if on { "on" } else { "off" });
                self.part_commands(
                    &kp,
                    Rect::new(kx, track.y + inset, k, k).snapped(dpr),
                    Some(p.knob),
                    alpha,
                    clip,
                    cmds,
                );
            }
            _ => {}
        }
        if matches!(n.widget, Widget::TextArea { .. }) {
            self.text_area_commands(i, rect, clip, cmds);
        }
        match &n.content {
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
                    for (k, line) in n.lines.iter().enumerate() {
                        let ly = top + k as f32 * lh;
                        let free = inner.width - line.width * sk;
                        // Right to left, start is the right edge.
                        let rtl = self.direction == og_i18n::Direction::Rtl;
                        let x = inner.x
                            + match (tc.align, rtl) {
                                (TextAlign::Start, false) | (TextAlign::End, true) => 0.0,
                                (TextAlign::Center, _) => free / 2.0,
                                (TextAlign::End, false) | (TextAlign::Start, true) => free,
                            };
                        // A focused field: scrolled, its selection under the text.
                        let x = match &field {
                            Some(e) if e.focused && tc.color != p.ink_faint => {
                                let x = x - e.scroll;
                                self.edit_marks(
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
                        // Bidi runs in visual order, shaped runs shaped, one
                        // piece per font of a mixed text (crate::text).
                        let pieces = crate::text::pieces_with(
                            font,
                            &line.text,
                            self.direction,
                            tc.tracking(),
                            tc.features(),
                        );
                        let baseline = m.baseline(&line.text, ly, lh, size);
                        let bounds = Rect::new(x, ly, line.width * sk, lh);
                        let role = tc.font.effect_index();
                        let tracking = tc.tracking();
                        // The style's thickening for the role (`textWeight`, device
                        // pixels: what a stroke between two pixels lacks), shadow included.
                        let weight = t.art.text_weights[role] * sk / dpr + tc.emphasis * size;
                        // A style's text glow: the font's distance field, under the glyphs.
                        let glow = t.art.text_glows[role].map(|g| {
                            let c = g.ink.color(p).fade(g.alpha * tc.color.alpha * alpha);
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
                        let own = self
                            .draws
                            .get(&i)
                            .or_else(|| self.part_draws.get(&i))
                            .is_some_and(|d| d.text.is_some());
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
                            let c = sc.fade(tc.color.alpha);
                            text(dx, dy, c, None, cmds);
                            if blur > 0.0 {
                                text(dx + blur / 2.0, dy + blur / 2.0, c.fade(0.5), None, cmds);
                            }
                        }
                        text(0.0, 0.0, tc.color, glow, cmds);
                        // The flourish frames a title, never a smaller
                        // heading (under 28 px it crowds the words), at
                        // the words' cap height (art review, 2026-11-04).
                        if n.heading
                            && k == 0
                            && tc.step.size >= FLOURISH_MIN_TITLE
                            && let Some(o) = t.art.flourish.filter(|_| t.art.atlas.is_some())
                        {
                            let cy = ly + lh / 2.0;
                            let gap = o.offset + t.space.sm;
                            let h = CAP_HEIGHT * size;
                            let w = o.w / o.h.max(1e-3) * h;
                            let left = Rect::new(x - gap - w, cy - h / 2.0, w, h);
                            let right = Rect::new(x + line.width * sk + gap, cy - h / 2.0, w, h);
                            // Both sides of a centred heading; the side away from the edge otherwise.
                            if tc.align != TextAlign::Start {
                                self.ornament(o, left, false, false, alpha, clip, cmds);
                            }
                            if tc.align != TextAlign::End {
                                self.ornament(o, right, true, false, alpha, clip, cmds);
                            }
                        }
                    }
                    if let Some(e) = &field
                        && e.focused
                    {
                        // The composition's underline and the caret, over the text.
                        let text = if tc.color != p.ink_faint {
                            tc.text.as_str()
                        } else {
                            ""
                        };
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
        let clip = n.clip;
        let alpha = n.alpha;
        let dpr = self.viewport.dpr();
        for shape in &n.shapes {
            match shape {
                Shape::Part { frame, part, fill } => {
                    let r = frame.resolve(&n.rect);
                    if !r.is_empty() {
                        self.part_commands(part, r.snapped(dpr), *fill, alpha, clip, cmds);
                    }
                }
                Shape::Rect {
                    frame,
                    radius,
                    color,
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
                    frame,
                    angle,
                    color,
                } => {
                    let r = frame.resolve(&n.rect);
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
                    frame,
                    angle,
                    color,
                } => {
                    let r = frame.resolve(&n.rect);
                    cmds.push(DrawCmd::Half {
                        center: r.center(),
                        radius: r.width.min(r.height) / 2.0,
                        angle: *angle,
                        color: color.fade(alpha).linear(),
                        clip,
                    });
                }
                Shape::Image {
                    frame,
                    texture,
                    uv,
                    color,
                } => cmds.push(DrawCmd::Image {
                    rect: frame.resolve(&n.rect),
                    texture: Some(*texture),
                    uv: *uv,
                    color: color.fade(alpha).linear(),
                    clip,
                }),
                Shape::Text {
                    frame,
                    text,
                    step,
                    color,
                    font,
                    clip: within,
                } => {
                    let clip = clip.intersect(&within.resolve(&n.rect));
                    let f = self.fonts.for_text(*font, text);
                    if f.handle.is_none() {
                        continue;
                    }
                    let m = &f.metrics;
                    let r = frame.resolve(&n.rect);
                    // A pixel font on its grid (crate::layout::snap_pixel_text).
                    let size = m.pixel_size(step.size * n.scale);
                    let tracking = m.pixel_tracking(crate::layout::step_tracking(step, text));
                    let features = crate::layout::text_features(step, *font);
                    let w = m.measure_with(text, size, tracking, features);
                    let lh = step.line_height * n.scale;
                    let x = r.x + (r.width - w) / 2.0;
                    let ly = r.y + (r.height - lh) / 2.0;
                    let baseline = m.baseline(text, ly, lh, size);
                    let snap = |v: f32| (v * dpr).round() / dpr;
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
                    frame,
                    font,
                    ch,
                    color,
                } => {
                    let r = frame.resolve(&n.rect);
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
