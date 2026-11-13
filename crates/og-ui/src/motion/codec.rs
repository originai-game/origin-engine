//! Bytes of the motion state for [`crate::Ui::save_state`]: little endian,
//! counted lists, the frozen draw commands of leaving elements included.

use super::PresenceSpec;
use super::spring::{Channel, Transition};
use super::state::*;
use crate::draw::DrawCmd;
use crate::draw_codec::{DrawRead, DrawWrite};
use crate::element::Prop;
use og_core::glam::Vec2;
use og_draw::{FontHandle, GlyphPlacement, TextureHandle};
pub(crate) use og_ui_model::codec::{R, W};

/// What the motion state writes beyond [`W`]'s primitives.
pub(crate) trait MotionWrite {
    fn channel(&mut self, c: &Channel);
    fn transition(&mut self, t: &Transition);
    fn presence(&mut self, p: &Option<PresenceSpec>);
    fn cmd(&mut self, c: &DrawCmd);
    fn frozen(&mut self, f: &Frozen);
    fn motion(&mut self, m: &MotionState);
}

impl MotionWrite for W<'_> {
    fn channel(&mut self, c: &Channel) {
        for v in [c.value, c.velocity, c.target, c.from] {
            self.f32(v);
        }
        self.u64(c.since);
        self.u64(c.stepped);
        for v in c.mode {
            self.f32(v);
        }
        self.u8(c.kind);
    }
    fn transition(&mut self, t: &Transition) {
        // Resolved against the default tokens is not wanted here: a token
        // stays a token, so a style change after a restore still applies.
        match *t {
            Transition::Instant => self.u8(0),
            Transition::Spring(s) => {
                self.u8(1);
                self.f32(s.duration_ms);
                self.f32(s.bounce);
            }
            Transition::Tween { duration_ms, ease } => {
                self.u8(2);
                self.f32(duration_ms);
                let (code, c) = ease_code(ease);
                self.u8(code);
                for v in c {
                    self.f32(v);
                }
            }
            Transition::Token(tk) => {
                self.u8(3);
                self.str(tk.name());
            }
        }
    }
    fn presence(&mut self, p: &Option<PresenceSpec>) {
        match p {
            None => self.u8(0),
            Some(p) => {
                self.u8(1);
                self.u32(p.values.len() as u32);
                for (prop, v) in &p.values {
                    self.u8(prop_index(*prop));
                    self.f32(*v);
                }
                self.transition(&p.transition);
                self.f32(p.delay_ms);
            }
        }
    }

    fn cmd(&mut self, c: &DrawCmd) {
        match c {
            DrawCmd::Glass {
                rect,
                radius,
                blur,
                saturation,
                tint,
                material,
                alpha,
                clip,
            } => {
                // Tag 0 is the plain glass as it always was; 7 adds the
                // saturation and the material.
                let plain = *saturation == 1.0 && *material == og_draw::UiGlass::default();
                self.u8(if plain { 0 } else { 7 });
                self.rect(rect);
                self.f32(*radius);
                self.f32(*blur);
                self.color4(tint);
                self.f32(*alpha);
                self.rect(clip);
                if !plain {
                    self.f32(*saturation);
                    self.f32(material.edge);
                    self.color4(&material.edge_color);
                    for sh in &material.shadows {
                        self.f32(sh.offset.x);
                        self.f32(sh.offset.y);
                        self.f32(sh.blur);
                        self.f32(sh.spread);
                        self.color4(&sh.color);
                    }
                    crate::draw_codec::glass_flags(self, material);
                }
            }
            DrawCmd::Quad {
                rect,
                radius,
                color,
                clip,
            } => {
                self.u8(1);
                self.rect(rect);
                self.f32(*radius);
                self.color4(color);
                self.rect(clip);
            }
            DrawCmd::Image {
                rect,
                texture,
                uv,
                color,
                clip,
            } => {
                self.u8(2);
                self.rect(rect);
                self.u32(texture.map_or(0, |t| t.slot()));
                self.color4(uv);
                self.color4(color);
                self.rect(clip);
            }
            DrawCmd::Bar {
                center,
                w,
                h,
                angle,
                color,
                clip,
            } => {
                self.u8(3);
                self.f32(center.0);
                self.f32(center.1);
                self.f32(*w);
                self.f32(*h);
                self.f32(*angle);
                self.color4(color);
                self.rect(clip);
            }
            DrawCmd::Text {
                x,
                baseline,
                size,
                text,
                color,
                font,
                bounds,
                clip,
                tracking,
                weight,
                glow,
                placements,
            } => {
                self.u8(5);
                self.f32(*x);
                self.f32(*baseline);
                self.f32(*size);
                self.str(text);
                self.color4(color);
                self.u32(font.to_bits());
                self.rect(bounds);
                self.rect(clip);
                self.f32(*tracking);
                self.f32(*weight);
                match glow {
                    None => self.u8(0),
                    Some((c, r)) => {
                        self.u8(1);
                        self.color4(c);
                        self.f32(*r);
                    }
                }
                self.u32(placements.len() as u32);
                for p in placements {
                    self.f32(p.advance);
                    self.f32(p.x);
                    self.f32(p.y);
                    // A shaped form's glyph id + 1; 0: the character's own.
                    self.u32(p.glyph.map_or(0, |g| u32::from(g) + 1));
                }
            }
            // The UI pass's commands (crate::draw_codec).
            other => {
                self.u8(6);
                self.ui_cmd(other);
            }
        }
    }

    fn frozen(&mut self, f: &Frozen) {
        self.u32(f.cmds.len() as u32);
        for (c, m) in &f.cmds {
            self.cmd(c);
            self.affine(m);
        }
        self.rect(&f.rect);
        self.affine(&f.world);
        self.f32(f.alpha);
        self.u64(f.parent);
        self.str(&f.path);
        self.presence(&f.exit);
        self.u64(f.match_key);
        self.f32(f.radius);
    }

    fn motion(&mut self, m: &MotionState) {
        self.u32(m.channels.len() as u32);
        let mut keys: Vec<u64> = m.channels.keys().copied().collect();
        keys.sort_unstable();
        for k in keys {
            self.u64(k);
            self.channel(&m.channels[&k]);
        }
        self.u32(m.groups.len() as u32);
        for (k, g) in &m.groups {
            self.u64(*k);
            self.str(&g.state);
            self.u64(g.since);
        }
        for map in [&m.set_states, &m.forced] {
            self.u32(map.len() as u32);
            for (k, s) in map {
                self.u64(*k);
                self.str(s);
            }
        }
        self.u32(m.plays.len() as u32);
        for p in &m.plays {
            self.u32(p.id);
            self.u64(p.root);
            self.u32(p.clips.len() as u32);
            for c in &p.clips {
                self.u64(c.timeline);
                self.u64(c.target);
                self.f32(c.offset_ms);
                self.u32(c.index);
            }
            self.u64(p.start);
            self.f32(p.at_ms);
            self.f32(p.last_ms);
            self.f32(p.speed);
            self.u8(u8::from(p.reverse)
                | u8::from(p.hold) << 1
                | u8::from(p.paused) << 2
                | u8::from(p.ended) << 3);
            self.u8(p.repeat);
            self.f32(p.length_ms);
        }
        self.u32(m.next_play);
        self.u32(m.ghosts.len() as u32);
        for g in &m.ghosts {
            self.u64(g.key);
            self.frozen(&g.frozen);
            self.u32(g.channels.len() as u32);
            for (i, c) in &g.channels {
                self.u8(*i);
                self.channel(c);
            }
            self.u64(g.since);
            self.u64(g.morph_to);
            self.u32(g.ticks);
        }
        self.u32(m.frozen.len() as u32);
        for (k, f) in &m.frozen {
            self.u64(*k);
            self.frozen(f);
        }
        self.u32(m.flip.len() as u32);
        for (k, r) in &m.flip {
            self.u64(*k);
            self.rect(r);
        }
        self.u32(m.matches.len() as u32);
        for (k, v) in &m.matches {
            self.u64(*k);
            self.u64(v.key);
            self.rect(&v.rect);
            self.f32(v.radius);
            self.u64(v.tick);
        }
        self.u32(m.seen.len() as u32);
        for (k, v) in &m.seen {
            self.u64(*k);
            self.u64(*v);
        }
        self.u32(m.gestures.len() as u32);
        for (k, g) in &m.gestures {
            self.u64(*k);
            self.u8(u8::from(g.held)
                | u8::from(g.dragging) << 1
                | u8::from(g.dismissed) << 2
                | u8::from(g.pinch) << 3);
            self.vec2(g.start);
            self.vec2(g.start_offset);
            self.u8(g.lock);
            self.vec2(g.velocity);
            self.vec2(g.last);
            self.vec2(g.offset);
            self.i32(g.snapped);
            self.u64(g.over);
            self.u64(g.dropped_on);
            for v in [
                g.scale,
                g.rotation,
                g.pinch_spread,
                g.pinch_angle,
                g.pinch_scale,
                g.pinch_rotation,
            ] {
                self.f32(v);
            }
            self.u64(g.tick);
        }
        self.u32(m.bursts.len() as u32);
        for b in &m.bursts {
            self.u64(b.key);
            self.u64(b.since);
            self.u32(b.count);
            self.f32(b.radius);
            self.srgba(b.color);
            self.f32(b.duration_ms);
        }
    }
}

pub(crate) fn ease_code(e: super::Ease) -> (u8, [f32; 4]) {
    use super::Ease;
    match e {
        Ease::Linear => (0, [0.0; 4]),
        Ease::CubicBezier(c) => (4, c),
        Ease::Steps { count, position } => {
            let position = match position {
                og_tween::StepPosition::JumpStart => 0.0,
                og_tween::StepPosition::JumpEnd => 1.0,
                og_tween::StepPosition::JumpNone => 2.0,
                og_tween::StepPosition::JumpBoth => 3.0,
            };
            // Two exact 16-bit halves avoid rounding counts above 2^24.
            (
                28,
                [(count & 0xffff) as f32, (count >> 16) as f32, position, 0.0],
            )
        }
        // A `linear()` curve by its points' hash, in exact 16-bit quarters:
        // its document defines it again before a restore reads it.
        Ease::Points(_) => {
            let h = e.points_hash().unwrap_or(0);
            (29, [0, 16, 32, 48].map(|s| ((h >> s) & 0xffff) as f32))
        }
        named => (5 + named.code() as u8, [0.0; 4]),
    }
}

pub(crate) fn ease_from_code(code: u8, c: [f32; 4]) -> Option<super::Ease> {
    use super::Ease;
    // Existing UI snapshot tags retain their exact curve controls.
    let ease = match code {
        0 => Ease::Linear,
        1 => Ease::CubicBezier(crate::theme::EASE_OUT),
        2 => Ease::CubicBezier(crate::theme::EASE_EXIT),
        3 => Ease::CubicBezier([0.65, 0.0, 0.35, 1.0]),
        4 => Ease::CubicBezier(c),
        5..=26 => Ease::from_code(u32::from(code - 5))?,
        29 => {
            if c.iter()
                .any(|n| !(0.0..=65535.0).contains(n) || n.fract() != 0.0)
            {
                return None;
            }
            let h = c
                .iter()
                .enumerate()
                .fold(0u64, |h, (k, &q)| h | (q as u64) << (16 * k));
            return og_tween::Ease::from_points_hash(h);
        }
        28 => {
            if c[..2]
                .iter()
                .any(|n| !(0.0..=65535.0).contains(n) || n.fract() != 0.0)
            {
                return None;
            }
            let position = match c[2] {
                0.0 => og_tween::StepPosition::JumpStart,
                1.0 => og_tween::StepPosition::JumpEnd,
                2.0 => og_tween::StepPosition::JumpNone,
                3.0 => og_tween::StepPosition::JumpBoth,
                _ => return None,
            };
            Ease::Steps {
                count: c[0] as u32 | ((c[1] as u32) << 16),
                position,
            }
        }
        _ => return None,
    };
    ease.is_valid().then_some(ease)
}

pub(crate) fn prop_index(p: Prop) -> u8 {
    Prop::ALL.iter().position(|&q| q == p).unwrap_or(0) as u8
}

/// Where a state before version 19 kept a shaped form: U+F0000 + glyph id.
const LEGACY_GLYPHS: u32 = 0xF_0000;

/// What the motion state reads beyond [`R`]'s primitives.
pub(crate) trait MotionRead {
    fn channel(&mut self) -> Option<Channel>;
    fn transition(&mut self) -> Option<Transition>;
    fn presence(&mut self) -> Option<Option<PresenceSpec>>;
    fn cmd(&mut self) -> Option<DrawCmd>;
    fn frozen(&mut self) -> Option<Frozen>;
    fn motion(&mut self, version: u32) -> Option<MotionState>;
}

impl MotionRead for R<'_> {
    fn channel(&mut self) -> Option<Channel> {
        let channel = Channel {
            value: self.f32()?,
            velocity: self.f32()?,
            target: self.f32()?,
            from: self.f32()?,
            since: self.u64()?,
            stepped: self.u64()?,
            mode: [
                self.f32()?,
                self.f32()?,
                self.f32()?,
                self.f32()?,
                self.f32()?,
                self.f32()?,
            ],
            kind: self.u8()?,
        };
        if channel.kind == 2 {
            let [_, code, a, b, c, d] = channel.mode;
            if code != f32::from(code as u8) {
                return None;
            }
            ease_from_code(code as u8, [a, b, c, d])?;
        }
        Some(channel)
    }
    fn transition(&mut self) -> Option<Transition> {
        use super::{MotionToken, SpringSpec};
        Some(match self.u8()? {
            0 => Transition::Instant,
            1 => Transition::Spring(SpringSpec::new(self.f32()?, self.f32()?)),
            2 => {
                let duration_ms = self.f32()?;
                let code = self.u8()?;
                let c = self.color4()?;
                let ease = ease_from_code(code, c)?;
                Transition::Tween { duration_ms, ease }
            }
            3 => Transition::Token(MotionToken::from_name(&self.str()?)?),
            _ => return None,
        })
    }
    fn presence(&mut self) -> Option<Option<PresenceSpec>> {
        if self.u8()? == 0 {
            return Some(None);
        }
        let mut values = Vec::new();
        for _ in 0..self.u32()? {
            let p = *Prop::ALL.get(self.u8()? as usize)?;
            values.push((p, self.f32()?));
        }
        Some(Some(PresenceSpec {
            values,
            transition: self.transition()?,
            delay_ms: self.f32()?,
        }))
    }

    fn cmd(&mut self) -> Option<DrawCmd> {
        Some(match self.u8()? {
            tag @ (0 | 7) => {
                let (rect, radius, blur, tint, alpha, clip) = (
                    self.rect()?,
                    self.f32()?,
                    self.f32()?,
                    self.color4()?,
                    self.f32()?,
                    self.rect()?,
                );
                let mut saturation = 1.0;
                let mut material = og_draw::UiGlass::default();
                if tag == 7 {
                    saturation = self.f32()?;
                    material.edge = self.f32()?;
                    material.edge_color = self.color4()?;
                    for sh in &mut material.shadows {
                        sh.offset = Vec2::new(self.f32()?, self.f32()?);
                        sh.blur = self.f32()?;
                        sh.spread = self.f32()?;
                        sh.color = self.color4()?;
                    }
                    crate::draw_codec::rglass_flags(self, &mut material)?;
                }
                DrawCmd::Glass {
                    rect,
                    radius,
                    blur,
                    saturation,
                    tint,
                    material,
                    alpha,
                    clip,
                }
            }
            1 => DrawCmd::Quad {
                rect: self.rect()?,
                radius: self.f32()?,
                color: self.color4()?,
                clip: self.rect()?,
            },
            2 => DrawCmd::Image {
                rect: self.rect()?,
                texture: TextureHandle::from_slot(self.u32()?),
                uv: self.color4()?,
                color: self.color4()?,
                clip: self.rect()?,
            },
            3 => DrawCmd::Bar {
                center: (self.f32()?, self.f32()?),
                w: self.f32()?,
                h: self.f32()?,
                angle: self.f32()?,
                color: self.color4()?,
                clip: self.rect()?,
            },
            5 => {
                let x = self.f32()?;
                let baseline = self.f32()?;
                let size = self.f32()?;
                let mut text = self.str()?;
                let color = self.color4()?;
                let font = FontHandle::from_bits(self.u32()?);
                let bounds = self.rect()?;
                let clip = self.rect()?;
                let tracking = self.f32()?;
                let weight = self.f32()?;
                let glow = match self.u8()? {
                    0 => None,
                    _ => Some((self.color4()?, self.f32()?)),
                };
                let mut placements = Vec::new();
                for _ in 0..self.u32()? {
                    let (advance, x, y) = (self.f32()?, self.f32()?, self.f32()?);
                    let glyph = match self.legacy_glyphs {
                        true => None,
                        false => match self.u32()? {
                            0 => None,
                            g => Some(u16::try_from(g - 1).ok()?),
                        },
                    };
                    placements.push(GlyphPlacement {
                        advance,
                        x,
                        y,
                        glyph,
                    });
                }
                if self.legacy_glyphs {
                    // Its shaped forms by their ids: the character drawn
                    // in their place is not known, U+FFFD stands for it.
                    text = text
                        .chars()
                        .zip(0..)
                        .map(|(c, i)| match (c as u32).checked_sub(LEGACY_GLYPHS) {
                            Some(id) if id <= 0xFFFF && i < placements.len() => {
                                placements[i].glyph = Some(id as u16);
                                char::REPLACEMENT_CHARACTER
                            }
                            _ => c,
                        })
                        .collect();
                }
                DrawCmd::Text {
                    x,
                    baseline,
                    size,
                    text,
                    color,
                    font,
                    bounds,
                    clip,
                    tracking,
                    weight,
                    glow,
                    placements,
                }
            }
            6 => self.ui_cmd()?,
            _ => return None,
        })
    }

    fn frozen(&mut self) -> Option<Frozen> {
        let mut cmds = Vec::new();
        for _ in 0..self.u32()? {
            let c = self.cmd()?;
            cmds.push((c, self.affine()?));
        }
        Some(Frozen {
            cmds,
            rect: self.rect()?,
            world: self.affine()?,
            alpha: self.f32()?,
            parent: self.u64()?,
            path: self.str()?,
            exit: self.presence()?,
            match_key: self.u64()?,
            radius: self.f32()?,
        })
    }

    /// A UI state's motion, saved at `version` (a burst's duration came in
    /// at 23).
    fn motion(&mut self, version: u32) -> Option<MotionState> {
        let mut m = MotionState::default();
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            m.channels.insert(k, self.channel()?);
        }
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            let state = self.str()?;
            m.groups.insert(
                k,
                GroupState {
                    state,
                    since: self.u64()?,
                },
            );
        }
        for which in 0..2 {
            for _ in 0..self.u32()? {
                let k = self.u64()?;
                let s = self.str()?;
                if which == 0 {
                    m.set_states.insert(k, s);
                } else {
                    m.forced.insert(k, s);
                }
            }
        }
        for _ in 0..self.u32()? {
            let id = self.u32()?;
            let root = self.u64()?;
            let mut clips = Vec::new();
            for _ in 0..self.u32()? {
                clips.push(Clip {
                    timeline: self.u64()?,
                    target: self.u64()?,
                    offset_ms: self.f32()?,
                    index: self.u32()?,
                });
            }
            let start = self.u64()?;
            let at_ms = self.f32()?;
            let last_ms = self.f32()?;
            let speed = self.f32()?;
            let flags = self.u8()?;
            m.plays.push(Play {
                id,
                root,
                clips,
                start,
                at_ms,
                last_ms,
                speed,
                reverse: flags & 1 != 0,
                hold: flags & 2 != 0,
                paused: flags & 4 != 0,
                ended: flags & 8 != 0,
                repeat: self.u8()?,
                length_ms: self.f32()?,
            });
        }
        m.next_play = self.u32()?;
        for _ in 0..self.u32()? {
            let key = self.u64()?;
            let frozen = self.frozen()?;
            let mut channels = Vec::new();
            for _ in 0..self.u32()? {
                let i = self.u8()?;
                channels.push((i, self.channel()?));
            }
            m.ghosts.push(Ghost {
                key,
                frozen,
                channels,
                since: self.u64()?,
                morph_to: self.u64()?,
                ticks: self.u32()?,
            });
        }
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            m.frozen.insert(k, self.frozen()?);
        }
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            m.flip.insert(k, self.rect()?);
        }
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            m.matches.insert(
                k,
                Matched {
                    key: self.u64()?,
                    rect: self.rect()?,
                    radius: self.f32()?,
                    tick: self.u64()?,
                },
            );
        }
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            m.seen.insert(k, self.u64()?);
        }
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            let flags = self.u8()?;
            let mut g = Gesture {
                held: flags & 1 != 0,
                dragging: flags & 2 != 0,
                dismissed: flags & 4 != 0,
                pinch: flags & 8 != 0,
                start: self.vec2()?,
                start_offset: self.vec2()?,
                lock: self.u8()?,
                velocity: self.vec2()?,
                last: self.vec2()?,
                offset: self.vec2()?,
                snapped: self.i32()?,
                over: self.u64()?,
                dropped_on: self.u64()?,
                ..Gesture::default()
            };
            g.scale = self.f32()?;
            g.rotation = self.f32()?;
            g.pinch_spread = self.f32()?;
            g.pinch_angle = self.f32()?;
            g.pinch_scale = self.f32()?;
            g.pinch_rotation = self.f32()?;
            g.tick = self.u64()?;
            m.gestures.insert(k, g);
        }
        for _ in 0..self.u32()? {
            m.bursts.push(BurstRun {
                key: self.u64()?,
                since: self.u64()?,
                count: self.u32()?,
                radius: self.f32()?,
                color: self.srgba()?,
                duration_ms: if version >= 23 { self.f32()? } else { 0.0 },
            });
        }
        Some(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::Rect;

    /// A frozen text line keeps its shaped forms by glyph id through the
    /// codec; one saved before version 19, whose forms were its text's
    /// characters U+F0000 + id and whose placements had no id, reads them
    /// as the same glyph ids, its other characters as they were.
    #[test]
    fn shaped_forms_read_as_glyph_ids_now_and_from_before_19() {
        let text = |text: &str, glyphs: [Option<u16>; 3]| DrawCmd::Text {
            x: 1.0,
            baseline: 2.0,
            size: 16.0,
            text: text.into(),
            color: [1.0; 4],
            font: FontHandle::from_bits(3),
            bounds: Rect::default(),
            clip: Rect::default(),
            tracking: 0.0,
            weight: 0.0,
            glow: None,
            placements: glyphs
                .iter()
                .map(|&glyph| GlyphPlacement {
                    advance: 0.5,
                    glyph,
                    ..GlyphPlacement::default()
                })
                .collect(),
        };
        let read = |bytes: &[u8], legacy_glyphs| {
            R {
                bytes,
                at: 0,
                legacy_glyphs,
                affine_only: false,
            }
            .cmd()
        };
        let now = text("a\u{F0105}b", [None, Some(0x7), None]);
        let mut bytes = Vec::new();
        W(&mut bytes).cmd(&now);
        assert_eq!(read(&bytes, false), Some(now));
        // Before 19: 12 bytes a placement (no id) at the command's end.
        let old = text("a\u{F0105}b", [None; 3]);
        let mut bytes = Vec::new();
        W(&mut bytes).cmd(&old);
        let end = bytes.len() - 3 * 16;
        let mut legacy = bytes[..end].to_vec();
        for p in bytes[end..].chunks(16) {
            legacy.extend_from_slice(&p[..12]);
        }
        assert_eq!(
            read(&legacy, true),
            Some(text("a\u{FFFD}b", [None, Some(0x105), None]))
        );
    }
}
