//! Content-only scrolling of overflowing text. Layout, the control's face,
//! hit box and accessible source stay still; only its visual line moves.
use super::*;
use crate::layout::TextOverflow;

impl Node {
    pub(crate) fn text_overflow(&self) -> Option<TextOverflow> {
        match &*self.content {
            Content::Text(t) => t.overflow,
            Content::Rich(t) => t.overflow,
            _ => None,
        }
    }

    /// The single overflowing line, in its content viewport. Editable
    /// text keeps its independent caret scrolling; path text follows its path.
    pub(crate) fn scrolling_text(&self) -> Option<(Rect, f32, f32, f32, bool)> {
        let TextOverflow::Scroll { travel_ms, hold_ms } = self.text_overflow()? else {
            return None;
        };
        if self.props.text_path.is_some()
            || matches!(self.widget, Widget::Field(_) | Widget::TextArea { .. })
        {
            return None;
        }
        let (width, vertical) = match &*self.content {
            Content::Text(t) if !t.wrap && !t.ellipsis && self.layout.lines.len() == 1 => {
                (self.layout.lines[0].width, t.vertical.is_some())
            }
            Content::Rich(t) if !t.wrap && !t.ellipsis && self.layout.rich_lines.len() == 1 => (
                self.layout.rich_lines[0].width,
                self.rich_vertical().is_some(),
            ),
            _ => return None,
        };
        let sk = self.placed.scale;
        let p = self.layout.used.padding;
        let r = self.placed.rect;
        let inner = Rect::new(
            r.x + p.left * sk,
            r.y + p.top * sk,
            (r.width - p.horizontal() * sk).max(0.0),
            (r.height - p.vertical() * sk).max(0.0),
        );
        let room = if vertical { inner.height } else { inner.width };
        let overflow = width * sk - room;
        (room > 0.0 && overflow > 0.5).then_some((inner, overflow, travel_ms, hold_ms, vertical))
    }
}

impl Ui {
    /// Run after layout, before memory collection and both retained-paint
    /// shortcuts. Snapshot-owned widget memory resets on text/extent changes.
    pub(super) fn prepare_text_overflow(&mut self, order: &[usize]) {
        for &i in order {
            let n = &self.tree.nodes[i];
            if !matches!(n.text_overflow(), Some(TextOverflow::Scroll { .. })) {
                continue;
            }
            let key = child_key(n.key, "~text-overflow");
            let Some((rect, overflow, travel, hold, vertical)) = n.scrolling_text() else {
                self.interaction.state.anims.remove(&key);
                continue;
            };
            use std::hash::{Hash, Hasher};
            let mut h = std::hash::DefaultHasher::new();
            n.probe_text.hash(&mut h);
            for v in [rect.width, rect.height, overflow, travel, hold] {
                v.to_bits().hash(&mut h);
            }
            vertical.hash(&mut h);
            let hash = h.finish();
            let now = self.interaction.state.tick;
            let fresh = !self.has_anim(key);
            let mut a = self.anim(key);
            if fresh || a.h != hash {
                a = Anim {
                    h: hash,
                    t: now,
                    ..Anim::default()
                };
            }
            let elapsed = now.saturating_sub(a.t) as f64 * 1000.0
                / f64::from(self.env.ticks_per_second().max(1));
            let (travel, hold) = (f64::from(travel), f64::from(hold));
            let phase = elapsed % (2.0 * (travel + hold));
            a.a = if phase < hold {
                0.0
            } else if phase < hold + travel {
                ((phase - hold) / travel) as f32
            } else if phase < 2.0 * hold + travel {
                1.0
            } else {
                (1.0 - (phase - 2.0 * hold - travel) / travel) as f32
            };
            self.set_anim(key, a);
        }
    }

    pub(super) fn scroll_text_commands(&self, i: usize, from: usize, cmds: &mut Vec<DrawCmd>) {
        let n = &self.tree.nodes[i];
        let Some((rect, overflow, _, _, vertical)) = n.scrolling_text() else {
            return;
        };
        let Some(a) = self
            .interaction
            .state
            .anims
            .get(&child_key(n.key, "~text-overflow"))
        else {
            return;
        };
        let Some(bounds) = cmds[from..]
            .iter()
            .filter_map(|c| match c {
                DrawCmd::Text { bounds, .. } => Some(*bounds),
                DrawCmd::StyledText { text, .. } => Some(text.unclipped()),
                _ => None,
            })
            .reduce(|a, b| {
                let (x, y) = (a.x.min(b.x), a.y.min(b.y));
                Rect::new(
                    x,
                    y,
                    a.right().max(b.right()) - x,
                    a.bottom().max(b.bottom()) - y,
                )
            })
        else {
            return;
        };
        let rtl = !vertical && self.text_flow(i).0;
        let progress = a.a.clamp(0.0, 1.0);
        let offset = progress * overflow;
        let (dx, dy) = if vertical {
            (0.0, rect.y - bounds.y - offset)
        } else if rtl {
            (rect.right() - bounds.right() + offset, 0.0)
        } else {
            (rect.x - bounds.x - offset, 0.0)
        };
        let mut content = cmds.split_off(from);
        for c in &mut content {
            match c {
                DrawCmd::BeginLayer { rect, .. } | DrawCmd::EndLayer { rect, .. } => {
                    if !rect.is_empty() {
                        rect.x += dx;
                        rect.y += dy;
                    }
                }
                _ => {
                    commands::shift(c, dx, dy);
                }
            }
            if let Some(clip) = c.clip_mut() {
                *clip = clip.intersect(&rect);
            }
        }
        // CSS Overflow 4's suggested 1em fade, clamped to half the box.
        // Fade only where more content remains; endpoints stay fully legible.
        let em = match &*n.content {
            Content::Text(t) => n.used_text(t).step.size,
            Content::Rich(t) => t.base.0.size,
            _ => unreachable!(),
        } * n.placed.scale;
        let room = if vertical { rect.height } else { rect.width };
        let reach = em.min(room / 2.0);
        let fade = reach / room;
        let (before, after) = if rtl {
            (overflow - offset, offset)
        } else {
            (offset, overflow - offset)
        };
        let white = Srgba::rgb(255, 255, 255);
        let stop = |offset, alpha| crate::paint::Stop {
            offset,
            color: crate::paint::ColorRef::Literal(white.fade(alpha)),
        };
        let draw = crate::paint::ElementDraw {
            clip: crate::paint::Clip::Mask(crate::paint::Mask::Gradient(crate::paint::Gradient {
                from: [0.0, 0.0],
                to: if vertical { [0.0, 1.0] } else { [1.0, 0.0] },
                stops: vec![
                    stop(0.0, 1.0 - (before / reach).min(1.0)),
                    stop(fade, 1.0),
                    stop(1.0 - fade, 1.0),
                    stop(1.0, 1.0 - (after / reach).min(1.0)),
                ],
                ..Default::default()
            })),
            ..Default::default()
        };
        let at = crate::paint::Placed {
            rect,
            radius: 0.0,
            clip: self.paint_clip(n).intersect(&rect),
            alpha: 1.0,
        };
        cmds.extend(draw.begin(self.paint_theme(), &at));
        cmds.extend(content);
        cmds.extend(draw.leave(self.paint_theme(), &at));
    }
}
