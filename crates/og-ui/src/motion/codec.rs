//! Bytes of the motion state for [`crate::Ui::save_state`]: little endian,
//! counted lists, the frozen draw commands of leaving elements included.

use super::PresenceSpec;
use super::spring::{Channel, Transition};
use super::state::*;
use crate::draw::DrawCmd;
use crate::element::{Affine, Prop};
use crate::layout::Rect;
use crate::style::Srgba;
use og_core::glam::Vec2;
use og_render::{FontHandle, GlyphPlacement, TextureHandle};

pub(crate) struct W<'a>(pub &'a mut Vec<u8>);

impl W<'_> {
    pub fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    pub fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn i32(&mut self, v: i32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn f32(&mut self, v: f32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn str(&mut self, s: &str) {
        self.u32(s.len() as u32);
        self.0.extend_from_slice(s.as_bytes());
    }
    pub fn rect(&mut self, r: &Rect) {
        for v in [r.x, r.y, r.width, r.height] {
            self.f32(v);
        }
    }
    pub fn affine(&mut self, m: &Affine) {
        for v in [m.a, m.b, m.c, m.d, m.e, m.f] {
            self.f32(v);
        }
    }
    pub fn vec2(&mut self, v: Vec2) {
        self.f32(v.x);
        self.f32(v.y);
    }
    pub fn color4(&mut self, c: &[f32; 4]) {
        for v in c {
            self.f32(*v);
        }
    }
    pub fn srgba(&mut self, c: Option<Srgba>) {
        match c {
            None => self.u8(0),
            Some(c) => {
                self.u8(1);
                for v in c.rgb {
                    self.u8(v);
                }
                self.f32(c.alpha);
            }
        }
    }
    pub fn channel(&mut self, c: &Channel) {
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
    pub fn transition(&mut self, t: &Transition) {
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
    pub fn presence(&mut self, p: &Option<PresenceSpec>) {
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

    pub fn cmd(&mut self, c: &DrawCmd) {
        match c {
            DrawCmd::Glass {
                rect,
                radius,
                blur,
                tint,
                alpha,
                clip,
            } => {
                self.u8(0);
                self.rect(rect);
                self.f32(*radius);
                self.f32(*blur);
                self.color4(tint);
                self.f32(*alpha);
                self.rect(clip);
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
            DrawCmd::Half {
                center,
                radius,
                angle,
                color,
                clip,
            } => {
                self.u8(4);
                self.f32(center.0);
                self.f32(center.1);
                self.f32(*radius);
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
                }
            }
        }
    }

    pub fn frozen(&mut self, f: &Frozen) {
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

    pub fn motion(&mut self, m: &MotionState) {
        self.u32(m.channels.len() as u32);
        for (k, c) in &m.channels {
            self.u64(*k);
            self.channel(c);
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
        }
    }
}

fn ease_code(e: super::Ease) -> (u8, [f32; 4]) {
    use super::Ease;
    match e {
        Ease::Linear => (0, [0.0; 4]),
        Ease::Out => (1, [0.0; 4]),
        Ease::Exit => (2, [0.0; 4]),
        Ease::InOut => (3, [0.0; 4]),
        Ease::Bezier(c) => (4, c),
    }
}

pub(crate) fn prop_index(p: Prop) -> u8 {
    Prop::ALL.iter().position(|&q| q == p).unwrap_or(0) as u8
}

pub(crate) struct R<'a> {
    pub bytes: &'a [u8],
    pub at: usize,
}

impl R<'_> {
    fn take<const N: usize>(&mut self) -> Option<[u8; N]> {
        let s = self.bytes.get(self.at..self.at + N)?;
        self.at += N;
        s.try_into().ok()
    }
    pub fn u8(&mut self) -> Option<u8> {
        Some(self.take::<1>()?[0])
    }
    pub fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take()?))
    }
    pub fn i32(&mut self) -> Option<i32> {
        Some(i32::from_le_bytes(self.take()?))
    }
    pub fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take()?))
    }
    pub fn f32(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(self.take()?))
    }
    pub fn str(&mut self) -> Option<String> {
        let n = self.u32()? as usize;
        let s = self.bytes.get(self.at..self.at + n)?;
        self.at += n;
        String::from_utf8(s.to_vec()).ok()
    }
    pub fn rect(&mut self) -> Option<Rect> {
        Some(Rect::new(
            self.f32()?,
            self.f32()?,
            self.f32()?,
            self.f32()?,
        ))
    }
    pub fn affine(&mut self) -> Option<Affine> {
        Some(Affine {
            a: self.f32()?,
            b: self.f32()?,
            c: self.f32()?,
            d: self.f32()?,
            e: self.f32()?,
            f: self.f32()?,
        })
    }
    fn vec2(&mut self) -> Option<Vec2> {
        Some(Vec2::new(self.f32()?, self.f32()?))
    }
    fn color4(&mut self) -> Option<[f32; 4]> {
        Some([self.f32()?, self.f32()?, self.f32()?, self.f32()?])
    }
    fn srgba(&mut self) -> Option<Option<Srgba>> {
        Some(match self.u8()? {
            0 => None,
            _ => {
                let rgb = [self.u8()?, self.u8()?, self.u8()?];
                Some(Srgba {
                    rgb,
                    alpha: self.f32()?,
                })
            }
        })
    }
    fn channel(&mut self) -> Option<Channel> {
        Some(Channel {
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
        })
    }
    fn transition(&mut self) -> Option<Transition> {
        use super::{Ease, MotionToken, SpringSpec};
        Some(match self.u8()? {
            0 => Transition::Instant,
            1 => Transition::Spring(SpringSpec::new(self.f32()?, self.f32()?)),
            2 => {
                let duration_ms = self.f32()?;
                let code = self.u8()?;
                let c = self.color4()?;
                let ease = match code {
                    0 => Ease::Linear,
                    2 => Ease::Exit,
                    3 => Ease::InOut,
                    4 => Ease::Bezier(c),
                    _ => Ease::Out,
                };
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

    pub fn cmd(&mut self) -> Option<DrawCmd> {
        Some(match self.u8()? {
            0 => DrawCmd::Glass {
                rect: self.rect()?,
                radius: self.f32()?,
                blur: self.f32()?,
                tint: self.color4()?,
                alpha: self.f32()?,
                clip: self.rect()?,
            },
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
            4 => DrawCmd::Half {
                center: (self.f32()?, self.f32()?),
                radius: self.f32()?,
                angle: self.f32()?,
                color: self.color4()?,
                clip: self.rect()?,
            },
            5 => {
                let x = self.f32()?;
                let baseline = self.f32()?;
                let size = self.f32()?;
                let text = self.str()?;
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
                    placements.push(GlyphPlacement {
                        advance: self.f32()?,
                        x: self.f32()?,
                        y: self.f32()?,
                    });
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

    pub fn motion(&mut self) -> Option<MotionState> {
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
            });
        }
        Some(m)
    }
}
