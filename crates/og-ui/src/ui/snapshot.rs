use super::*;

impl Ui {
    /// The interaction state (hover, press, focus, caret, scroll offsets,
    /// last frame's hit rectangles, the UI clock, widget memories, toasts and
    /// queued navigation) as bytes, for the host's world snapshot: restoring
    /// it with the world replays the UI exactly.
    pub fn save_state(&self) -> Vec<u8> {
        let s = &self.state;
        let mut b = Vec::new();
        let u32_ = |b: &mut Vec<u8>, v: u32| b.extend_from_slice(&v.to_le_bytes());
        let u64_ = |b: &mut Vec<u8>, v: u64| b.extend_from_slice(&v.to_le_bytes());
        let f32_ = |b: &mut Vec<u8>, v: f32| b.extend_from_slice(&v.to_le_bytes());
        let rect = |b: &mut Vec<u8>, r: &Rect| {
            for v in [r.x, r.y, r.width, r.height] {
                b.extend_from_slice(&v.to_le_bytes());
            }
        };
        u32_(&mut b, STATE_VERSION);
        u64_(&mut b, s.hot);
        u64_(&mut b, s.active);
        u64_(&mut b, s.focus);
        u32_(&mut b, s.caret as u32);
        u32_(&mut b, s.hover_ticks);
        f32_(&mut b, s.pointer.x);
        f32_(&mut b, s.pointer.y);
        u32_(&mut b, s.scroll.len() as u32);
        for (k, v) in &s.scroll {
            u64_(&mut b, *k);
            f32_(&mut b, *v);
        }
        u32_(&mut b, s.hits.len() as u32);
        for h in &s.hits {
            u64_(&mut b, h.key);
            rect(&mut b, &h.rect);
            rect(&mut b, &h.full);
            b.extend_from_slice(&h.flags.to_le_bytes());
            u64_(&mut b, h.scope);
            u64_(&mut b, h.scroller);
            for v in [h.inv.a, h.inv.b, h.inv.c, h.inv.d, h.inv.e, h.inv.f] {
                f32_(&mut b, v);
            }
            rect(&mut b, &h.local);
            rect(&mut b, &h.clip);
        }
        u64_(&mut b, s.tick);
        b.push(
            u8::from(s.focus_visible) | u8::from(s.nav_off) << 1 | u8::from(s.scope_locked) << 2,
        );
        u64_(&mut b, s.scope);
        u64_(&mut b, s.focus_return);
        b.push(s.stick.0);
        u64_(&mut b, s.stick.1);
        u32_(&mut b, s.pending.len() as u32);
        for (n, entering) in &s.pending {
            b.push(n.index() | if *entering { 0x80 } else { 0 });
        }
        u32_(&mut b, s.anims.len() as u32);
        for (k, a) in &s.anims {
            u64_(&mut b, *k);
            f32_(&mut b, a.a);
            f32_(&mut b, a.b);
            u64_(&mut b, a.t);
            u64_(&mut b, a.h);
        }
        u32_(&mut b, s.toasts.len() as u32);
        for t in &s.toasts {
            u32_(&mut b, t.id);
            u32_(&mut b, t.text.len() as u32);
            b.extend_from_slice(t.text.as_bytes());
            b.push(t.tone);
            u64_(&mut b, t.born);
            u32_(&mut b, t.ticks);
            u64_(&mut b, t.leaving);
        }
        u32_(&mut b, s.next_toast);
        u32_(&mut b, s.screens.len() as u32);
        for (k, sc) in &s.screens {
            u64_(&mut b, *k);
            for text in [&sc.shown, &sc.wanted] {
                u32_(&mut b, text.len() as u32);
                b.extend_from_slice(text.as_bytes());
            }
            u64_(&mut b, sc.since);
        }
        u64_(&mut b, s.feedback_rng);
        b.extend_from_slice(&s.feedback_last);
        for v in s.voices {
            u64_(&mut b, v);
        }
        b.push(s.players);
        for k in 0..3 {
            u64_(&mut b, s.pad_focus[k]);
            b.push(s.pad_stick[k].0);
            u64_(&mut b, s.pad_stick[k].1);
        }
        u32_(&mut b, s.scroll_x.len() as u32);
        for (k, v) in &s.scroll_x {
            u64_(&mut b, *k);
            f32_(&mut b, *v);
        }
        u32_(&mut b, s.scroll_info.len() as u32);
        for (k, v) in &s.scroll_info {
            u64_(&mut b, *k);
            for x in [v.range.x, v.range.y, v.bar.x, v.bar.y] {
                f32_(&mut b, x);
            }
        }
        let (fk, fv) = s.fling.unwrap_or((0, Vec2::ZERO));
        u64_(&mut b, fk);
        f32_(&mut b, fv.x);
        f32_(&mut b, fv.y);
        f32_(&mut b, s.drag_v.x);
        f32_(&mut b, s.drag_v.y);
        let t = s.thumb.unwrap_or_default();
        u64_(&mut b, t.key);
        b.push(t.axis);
        f32_(&mut b, t.grab);
        u32_(&mut b, s.anchor as u32);
        u64_(&mut b, s.edit_key);
        u64_(&mut b, s.edit_tick);
        u32_(&mut b, s.composition.len() as u32);
        b.extend_from_slice(s.composition.as_bytes());
        f32_(&mut b, s.edit_scroll);
        b.push(u8::from(s.typed_stream));
        let pinch = s.pinch.unwrap_or_default();
        b.push(u8::from(s.pinch.is_some()));
        u64_(&mut b, pinch.key);
        f32_(&mut b, pinch.center.x);
        f32_(&mut b, pinch.center.y);
        f32_(&mut b, pinch.spread);
        let c = &s.click;
        u64_(&mut b, c.tick);
        f32_(&mut b, c.at.x);
        f32_(&mut b, c.at.y);
        b.push(c.count);
        b.push(u8::from(c.moved));
        u32_(&mut b, c.origin.0 as u32);
        u32_(&mut b, c.origin.1 as u32);
        let text = |b: &mut Vec<u8>, t: &str| {
            u32_(b, t.len() as u32);
            b.extend_from_slice(t.as_bytes());
        };
        u32_(&mut b, s.histories.len() as u32);
        for (k, h) in &s.histories {
            u64_(&mut b, *k);
            u64_(&mut b, h.seen);
            u64_(&mut b, h.used);
            b.push(u8::from(h.open));
            for stack in [&h.undo, &h.redo] {
                u32_(&mut b, stack.len() as u32);
                for c in stack {
                    u32_(&mut b, c.at as u32);
                    text(&mut b, &c.removed);
                    text(&mut b, &c.inserted);
                    b.push(c.kind as u8);
                    for v in [c.before.0, c.before.1, c.after.0, c.after.1] {
                        u32_(&mut b, v as u32);
                    }
                }
            }
        }
        b.push(u8::from(s.caret_upstream) | u8::from(s.killing) << 1);
        text(&mut b, &s.kill);
        crate::motion::codec::W(&mut b).motion(&s.motion);
        // The layer stack, its viewmodels and the documents' states.
        text(&mut b, &self.docs_state().to_string());
        b
    }

    /// The host's voice entity for each UI event's sound (by
    /// [`crate::tokens::SoundEvent`] index), kept in the UI state so
    /// snapshots carry it.
    pub fn voices_mut(&mut self) -> &mut [u64; crate::tokens::SoundEvent::COUNT] {
        &mut self.state.voices
    }

    /// Restores [`Ui::save_state`] bytes. Refused, with nothing changed, for
    /// bytes of another version or a truncated buffer.
    pub fn restore_state(&mut self, bytes: &[u8]) -> og_core::Result<()> {
        let bad = |why: String| {
            og_core::Error::new(
                og_core::ErrorKind::Parse,
                format!("UI state: {why}; pass the bytes Ui::save_state returned"),
            )
        };
        let mut r = Reader { bytes, at: 0 };
        let version = r.u32().ok_or_else(|| bad("truncated".into()))?;
        if version != STATE_VERSION {
            return Err(bad(format!("version {version}, expected {STATE_VERSION}")));
        }
        let s = r
            .state()
            .ok_or_else(|| bad("truncated or malformed".into()))?;
        let docs: serde_json::Value = r
            .string()
            .and_then(|t| serde_json::from_str(&t).ok())
            .ok_or_else(|| bad("truncated or malformed layer stack".into()))?;
        if r.at != bytes.len() {
            return Err(bad("trailing bytes".into()));
        }
        self.state = s;
        self.restore_docs_state(&docs);
        Ok(())
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Option<[u8; N]> {
        let s = self.bytes.get(self.at..self.at + N)?;
        self.at += N;
        s.try_into().ok()
    }
    fn u8(&mut self) -> Option<u8> {
        self.take::<1>().map(|b| b[0])
    }
    fn u32(&mut self) -> Option<u32> {
        self.take().map(u32::from_le_bytes)
    }
    fn u64(&mut self) -> Option<u64> {
        self.take().map(u64::from_le_bytes)
    }
    fn f32(&mut self) -> Option<f32> {
        self.take().map(f32::from_le_bytes)
    }

    fn string(&mut self) -> Option<String> {
        let len = self.u32()? as usize;
        let raw = self.bytes.get(self.at..self.at + len)?;
        self.at += len;
        String::from_utf8(raw.to_vec()).ok()
    }

    fn rect(&mut self) -> Option<Rect> {
        Some(Rect::new(
            self.f32()?,
            self.f32()?,
            self.f32()?,
            self.f32()?,
        ))
    }

    fn state(&mut self) -> Option<State> {
        let mut s = State {
            hot: self.u64()?,
            active: self.u64()?,
            focus: self.u64()?,
            caret: self.u32()? as usize,
            hover_ticks: self.u32()?,
            pointer: Vec2::new(self.f32()?, self.f32()?),
            ..State::default()
        };
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            s.scroll.insert(k, self.f32()?);
        }
        for _ in 0..self.u32()? {
            s.hits.push(Hit {
                key: self.u64()?,
                rect: self.rect()?,
                full: self.rect()?,
                flags: u16::from_le_bytes(self.take()?),
                scope: self.u64()?,
                scroller: self.u64()?,
                inv: Affine {
                    a: self.f32()?,
                    b: self.f32()?,
                    c: self.f32()?,
                    d: self.f32()?,
                    e: self.f32()?,
                    f: self.f32()?,
                },
                local: self.rect()?,
                clip: self.rect()?,
            });
        }
        s.tick = self.u64()?;
        let flags = self.u8()?;
        s.focus_visible = flags & 1 != 0;
        s.nav_off = flags & 2 != 0;
        s.scope_locked = flags & 4 != 0;
        s.scope = self.u64()?;
        s.focus_return = self.u64()?;
        s.stick = (self.u8()?, self.u64()?);
        for _ in 0..self.u32()? {
            let i = self.u8()?;
            s.pending.push((Nav::from_index(i & 0x7f)?, i & 0x80 != 0));
        }
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            let a = Anim {
                a: self.f32()?,
                b: self.f32()?,
                t: self.u64()?,
                h: self.u64()?,
            };
            s.anims.insert(k, a);
        }
        for _ in 0..self.u32()? {
            let id = self.u32()?;
            let len = self.u32()? as usize;
            let raw = self.bytes.get(self.at..self.at + len)?;
            self.at += len;
            let text = String::from_utf8(raw.to_vec()).ok()?;
            s.toasts.push(Toast {
                id,
                text,
                tone: self.u8()?,
                born: self.u64()?,
                ticks: self.u32()?,
                leaving: self.u64()?,
            });
        }
        s.next_toast = self.u32()?;
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            let shown = self.string()?;
            let wanted = self.string()?;
            let since = self.u64()?;
            s.screens.insert(
                k,
                crate::juice::ScreenState {
                    shown,
                    wanted,
                    since,
                },
            );
        }
        s.feedback_rng = self.u64()?;
        s.feedback_last = self.take()?;
        for v in s.voices.iter_mut() {
            *v = self.u64()?;
        }
        s.players = self.u8()?;
        for k in 0..3 {
            s.pad_focus[k] = self.u64()?;
            s.pad_stick[k] = (self.u8()?, self.u64()?);
        }
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            s.scroll_x.insert(k, self.f32()?);
        }
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            let (rx, ry, bx, by) = (self.f32()?, self.f32()?, self.f32()?, self.f32()?);
            s.scroll_info.insert(
                k,
                crate::scroll::ScrollInfo {
                    range: Vec2::new(rx, ry),
                    bar: Vec2::new(bx, by),
                },
            );
        }
        let fk = self.u64()?;
        let fv = Vec2::new(self.f32()?, self.f32()?);
        s.fling = (fk != 0).then_some((fk, fv));
        s.drag_v = Vec2::new(self.f32()?, self.f32()?);
        let t = crate::scroll::Thumb {
            key: self.u64()?,
            axis: self.u8()?,
            grab: self.f32()?,
        };
        s.thumb = (t.key != 0).then_some(t);
        s.anchor = self.u32()? as usize;
        s.edit_key = self.u64()?;
        s.edit_tick = self.u64()?;
        s.composition = self.string()?;
        s.edit_scroll = self.f32()?;
        s.typed_stream = self.u8()? != 0;
        let on = self.u8()? != 0;
        let pinch = crate::scroll::Pinch {
            key: self.u64()?,
            center: Vec2::new(self.f32()?, self.f32()?),
            spread: self.f32()?,
        };
        s.pinch = on.then_some(pinch);
        s.click = crate::edit::Click {
            tick: self.u64()?,
            at: Vec2::new(self.f32()?, self.f32()?),
            count: self.u8()?,
            moved: self.u8()? != 0,
            origin: (self.u32()? as usize, self.u32()? as usize),
        };
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            let mut h = crate::edit::History {
                seen: self.u64()?,
                used: self.u64()?,
                open: self.u8()? != 0,
                ..Default::default()
            };
            for redo in [false, true] {
                let n = self.u32()?;
                let mut stack = Vec::new();
                for _ in 0..n {
                    stack.push(crate::edit::Change {
                        at: self.u32()? as usize,
                        removed: self.string()?,
                        inserted: self.string()?,
                        kind: crate::edit::EditKind::from_u8(self.u8()?),
                        before: (self.u32()? as usize, self.u32()? as usize),
                        after: (self.u32()? as usize, self.u32()? as usize),
                    });
                }
                if redo {
                    h.redo = stack;
                } else {
                    h.undo = stack;
                }
            }
            s.histories.insert(k, h);
        }
        let flags = self.u8()?;
        s.caret_upstream = flags & 1 != 0;
        s.killing = flags & 2 != 0;
        s.kill = self.string()?;
        let mut r = crate::motion::codec::R {
            bytes: self.bytes,
            at: self.at,
        };
        s.motion = r.motion()?;
        self.at = r.at;
        Some(s)
    }
}
