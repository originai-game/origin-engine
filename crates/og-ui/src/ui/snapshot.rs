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
            u64_(&mut b, h.owner);
            for v in [h.inv.a, h.inv.b, h.inv.c, h.inv.d, h.inv.e, h.inv.f] {
                f32_(&mut b, v);
            }
            rect(&mut b, &h.local);
            rect(&mut b, &h.clip);
            u64_(&mut b, h.group);
            u64_(&mut b, h.gesture);
        }
        u64_(&mut b, s.tick);
        b.push(
            u8::from(s.focus_visible) | u8::from(s.nav_off) << 1 | u8::from(s.scope_locked) << 2,
        );
        u64_(&mut b, s.scope);
        u32_(&mut b, s.focus_returns.len() as u32);
        for &(scope, focus) in &s.focus_returns {
            u64_(&mut b, scope);
            u64_(&mut b, focus);
        }
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
        // Focus groups, explicit moves, recognisers and the press (version 14).
        u32_(&mut b, s.groups.len() as u32);
        for (k, g) in &s.groups {
            u64_(&mut b, *k);
            b.push(u8::from(g.spec.wrap) | u8::from(g.spec.memory) << 1);
            for v in [g.parent, g.last, g.first] {
                u64_(&mut b, v);
            }
        }
        u32_(&mut b, s.nav_links.len() as u32);
        for (k, l) in &s.nav_links {
            u64_(&mut b, *k);
            for v in l {
                u64_(&mut b, *v);
            }
        }
        u32_(&mut b, s.recognizers.len() as u32);
        for (k, (g, up)) in &s.recognizers {
            u64_(&mut b, *k);
            b.push(u8::from(g.tap) | u8::from(g.double_tap) << 1 | u8::from(g.long_press) << 2);
            for v in [g.long_press_ms, g.double_tap_ms, g.slop] {
                f32_(&mut b, v);
            }
            u32_(&mut b, g.priority as u32);
            u64_(&mut b, *up);
        }
        let t = &s.tap;
        u32_(&mut b, t.chain.len() as u32);
        for k in &t.chain {
            u64_(&mut b, *k);
        }
        f32_(&mut b, t.start.x);
        f32_(&mut b, t.start.y);
        u64_(&mut b, t.since);
        f32_(&mut b, t.travel);
        b.push(u8::from(t.taken));
        u64_(&mut b, t.waiting);
        u64_(&mut b, t.waiting_tick);
        f32_(&mut b, t.waiting_at.x);
        f32_(&mut b, t.waiting_at.y);
        // The virtual cursor and the HUD layouts (version 15).
        let v = &s.vcursor;
        b.push(
            u8::from(v.desc.is_some())
                | u8::from(v.shown) << 1
                | u8::from(v.placed) << 2
                | u8::from(v.down) << 3,
        );
        let d = v.desc.unwrap_or_default();
        for x in [d.speed, d.accel, d.magnet, d.scroll, v.pos.x, v.pos.y] {
            f32_(&mut b, x);
        }
        f32_(&mut b, v.last_pointer.x);
        f32_(&mut b, v.last_pointer.y);
        u32_(&mut b, v.held);
        u64_(&mut b, v.over);
        let h = &s.hud;
        for l in &h.layouts {
            u32_(&mut b, l.len() as u32);
            for (k, p) in l {
                text(&mut b, k);
                for x in [p.x, p.y, p.scale, p.opacity] {
                    f32_(&mut b, x);
                }
            }
        }
        b.push(h.player | u8::from(h.edit) << 2 | u8::from(h.drag.is_some()) << 3);
        text(&mut b, &h.selected);
        let (start, from, spread) = h.drag.unwrap_or_default();
        for x in [
            start.x,
            start.y,
            from.x,
            from.y,
            from.scale,
            from.opacity,
            spread,
        ] {
            f32_(&mut b, x);
        }
        u32_(&mut b, h.targets.len() as u32);
        for t in &h.targets {
            text(&mut b, &t.key);
            rect(&mut b, &t.base);
            rect(&mut b, &t.shown);
            let c = &t.spec;
            text(&mut b, &c.key);
            b.push(
                u8::from(c.lock_move) | u8::from(c.lock_scale) << 1 | u8::from(c.lock_opacity) << 2,
            );
            for x in [c.min_scale, c.max_scale, c.min_opacity] {
                f32_(&mut b, x);
            }
        }
        rect(&mut b, &h.editor);
        u64_(&mut b, h.changed);
        // The finger each touch control holds (version 16).
        u32_(&mut b, s.touch_owners.len() as u32);
        for (k, id) in &s.touch_owners {
            u64_(&mut b, *k);
            u32_(&mut b, *id);
        }
        // The list typeahead (version 17).
        text(&mut b, &s.typeahead.0);
        u64_(&mut b, s.typeahead.1);
        // The Rive players (version 18): each element's artboard id, the
        // tick it was last built and its player's bytes, by key.
        let mut rive: Vec<_> = s.rive.iter().collect();
        rive.sort_by_key(|(k, _)| **k);
        u32_(&mut b, rive.len() as u32);
        for (k, slot) in rive {
            u64_(&mut b, *k);
            text(&mut b, &slot.board);
            u64_(&mut b, slot.built);
            let bytes = slot
                .player
                .as_ref()
                .map_or_else(|| slot.saved.clone(), |p| p.save());
            u32_(&mut b, bytes.len() as u32);
            b.extend_from_slice(&bytes);
        }
        // The layer stack, its viewmodels and the documents' states.
        let mut docs = self.docs_state();
        if !s.motion.css.is_empty() {
            docs["cssAnimations"] = serde_json::to_value(&s.motion.css).expect("finite CSS runs");
        }
        if !s.virtual_lists.is_empty() {
            docs["virtualLists"] = serde_json::to_value(&s.virtual_lists).expect("finite extents");
        }
        text(&mut b, &docs.to_string());
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
        let mut s = r
            .state(STATE_VERSION)
            .ok_or_else(|| bad("truncated or malformed".into()))?;
        let docs: serde_json::Value = r
            .string()
            .and_then(|t| serde_json::from_str(&t).ok())
            .ok_or_else(|| bad("truncated or malformed layer stack".into()))?;
        if r.at != bytes.len() {
            return Err(bad("trailing bytes".into()));
        }
        if let Some(runs) = docs.get("cssAnimations") {
            s.motion.css = serde_json::from_value(runs.clone())
                .map_err(|_| bad("malformed CSS animations".into()))?;
        }
        if let Some(lists) = docs.get("virtualLists") {
            s.virtual_lists = serde_json::from_value(lists.clone())
                .map_err(|_| bad("malformed virtual lists".into()))?;
            if !s.virtual_lists.values().all(|v| v.valid_snapshot()) {
                return Err(bad("invalid virtual list extents".into()));
            }
        }
        self.restore_docs_state(&docs).map_err(bad)?;
        self.state = s;
        self.kept = Default::default();
        // The restored mounts emit afresh; the nodes they owned retire
        // unless the next frame writes them again.
        let owned: Vec<usize> = (0..self.persistent.len())
            .filter(|&i| self.persistent[i])
            .collect();
        self.retire_owned(&owned);
        Ok(())
    }
}

impl Ui {
    /// [`Ui::save_state`] bytes of an earlier version as this version's,
    /// for a save from an older engine (og-runtime's engine migrations):
    /// the bytes as they are at this version, the state of version 9
    /// (engine 0.7.7) with what it did not have at its defaults (no hit
    /// transforms, the feedback stream unseeded, no motion channels, an
    /// empty layer stack) and its six sound voices moved to their events,
    /// the state of version 11 with its one focus return as the stack's
    /// entry, the state of version 12 with each hit's focusable owner
    /// unset (the next frame's hit test fills it), and the state of
    /// version 13 with no focus groups, explicit moves or gesture
    /// recognisers (the next frame collects them) and no press recognised,
    /// and the state of version 14 with no virtual cursor and no HUD
    /// layouts (a game gives its saved layouts back with
    /// [`Ui::set_hud_layout`]), and the state of version 15 with no touch
    /// control holding a finger (a held stick takes its finger again on
    /// the next frame), the state of version 16 with no typeahead, the
    /// state of version 17 with no Rive players, and the state of version
    /// 18 with its frozen texts' shaped forms (U+F0000 + glyph id) as
    /// glyph ids.
    /// Refused for another version, or bytes that do not read as theirs.
    pub fn upgrade_state(bytes: &[u8]) -> og_core::Result<Vec<u8>> {
        let bad = |why: String| {
            og_core::Error::new(og_core::ErrorKind::Parse, format!("UI state: {why}"))
        };
        let mut r = Reader { bytes, at: 0 };
        let version = r.u32().ok_or_else(|| bad("truncated".into()))?;
        match version {
            STATE_VERSION => Ok(bytes.to_vec()),
            9 => {
                let s = r
                    .state(9)
                    .ok_or_else(|| bad("truncated or malformed version 9 state".into()))?;
                if r.at != bytes.len() {
                    return Err(bad("trailing bytes after a version 9 state".into()));
                }
                let ui = Ui {
                    state: s,
                    ..Ui::default()
                };
                Ok(ui.save_state())
            }
            11..=18 => {
                let s = r.state(version).ok_or_else(|| {
                    bad(format!("truncated or malformed version {version} state"))
                })?;
                let docs: serde_json::Value = r
                    .string()
                    .and_then(|t| serde_json::from_str(&t).ok())
                    .ok_or_else(|| {
                        bad(format!(
                            "truncated or malformed version {version} layer stack"
                        ))
                    })?;
                if r.at != bytes.len() {
                    return Err(bad(format!(
                        "trailing bytes after a version {version} state"
                    )));
                }
                let mut ui = Ui {
                    state: s,
                    ..Ui::default()
                };
                ui.restore_docs_state(&docs).map_err(bad)?;
                Ok(ui.save_state())
            }
            v => Err(bad(format!(
                "version {v}: this engine reads version {STATE_VERSION} and upgrades versions 9 and 11 to 18"
            ))),
        }
    }
}

/// Version 9's six sound voices, in its order (og-ui's `UiSound` then).
const V9_VOICES: [&str; 6] = ["hover", "press", "open", "close", "success", "error"];

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

    /// The state as `version` (this one's, 13, 12, 11 or 9) wrote it.
    fn state(&mut self, version: u32) -> Option<State> {
        let v9 = version == 9;
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
            let (key, rect, full) = (self.u64()?, self.rect()?, self.rect()?);
            let flags = if v9 {
                u16::from(self.u8()?)
            } else {
                u16::from_le_bytes(self.take()?)
            };
            let (scope, scroller) = (self.u64()?, self.u64()?);
            // Each hit's focusable owner came in at version 13; before it, unset
            // (the next frame's hit test fills it).
            let owner = if version >= 13 { self.u64()? } else { 0 };
            let (inv, local, clip) = if v9 {
                (Affine::IDENTITY, full, rect)
            } else {
                (
                    Affine {
                        a: self.f32()?,
                        b: self.f32()?,
                        c: self.f32()?,
                        d: self.f32()?,
                        e: self.f32()?,
                        f: self.f32()?,
                    },
                    self.rect()?,
                    self.rect()?,
                )
            };
            // Each hit's focus group and recogniser came in at version 14.
            let (group, gesture) = if version >= 14 {
                (self.u64()?, self.u64()?)
            } else {
                (0, 0)
            };
            s.hits.push(Hit {
                key,
                rect,
                full,
                flags,
                scope,
                scroller,
                owner,
                inv,
                local,
                clip,
                group,
                gesture,
            });
        }
        s.tick = self.u64()?;
        let flags = self.u8()?;
        s.focus_visible = flags & 1 != 0;
        s.nav_off = flags & 2 != 0;
        s.scope_locked = flags & 4 != 0;
        s.scope = self.u64()?;
        if version >= 12 {
            for _ in 0..self.u32()? {
                s.focus_returns.push((self.u64()?, self.u64()?));
            }
        } else {
            // Before version 12 one return was kept, for a scope opened over the base.
            let back = self.u64()?;
            if s.scope != 0 {
                s.focus_returns.push((0, back));
            }
        }
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
        if v9 {
            for name in V9_VOICES {
                let v = self.u64()?;
                let event = crate::tokens::SoundEvent::from_name(name)?;
                s.voices[event.index()] = v;
            }
        } else {
            s.feedback_rng = self.u64()?;
            s.feedback_last = self.take()?;
            for v in s.voices.iter_mut() {
                *v = self.u64()?;
            }
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
        if v9 {
            return Some(s);
        }
        let mut r = crate::motion::codec::R {
            bytes: self.bytes,
            at: self.at,
            legacy_glyphs: version < 19,
        };
        s.motion = r.motion()?;
        self.at = r.at;
        if version < 14 {
            return Some(s);
        }
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            let f = self.u8()?;
            let g = crate::recognize::GroupState {
                spec: crate::element::FocusGroup {
                    wrap: f & 1 != 0,
                    memory: f & 2 != 0,
                },
                parent: self.u64()?,
                last: self.u64()?,
                first: self.u64()?,
            };
            s.groups.insert(k, g);
        }
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            let mut l = [0u64; 6];
            for v in l.iter_mut() {
                *v = self.u64()?;
            }
            s.nav_links.insert(k, l);
        }
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            let f = self.u8()?;
            let g = crate::element::GestureSpec {
                tap: f & 1 != 0,
                double_tap: f & 2 != 0,
                long_press: f & 4 != 0,
                long_press_ms: self.f32()?,
                double_tap_ms: self.f32()?,
                slop: self.f32()?,
                priority: self.u32()? as i32,
            };
            s.recognizers.insert(k, (g, self.u64()?));
        }
        let t = &mut s.tap;
        for _ in 0..self.u32()? {
            t.chain.push(self.u64()?);
        }
        t.start = Vec2::new(self.f32()?, self.f32()?);
        t.since = self.u64()?;
        t.travel = self.f32()?;
        t.taken = self.u8()? != 0;
        t.waiting = self.u64()?;
        t.waiting_tick = self.u64()?;
        t.waiting_at = Vec2::new(self.f32()?, self.f32()?);
        if version < 15 {
            return Some(s);
        }
        let f = self.u8()?;
        let mut d = crate::vcursor::VirtualCursorDesc::default();
        let v = &mut s.vcursor;
        v.shown = f & 2 != 0;
        v.placed = f & 4 != 0;
        v.down = f & 8 != 0;
        d.speed = self.f32()?;
        d.accel = self.f32()?;
        d.magnet = self.f32()?;
        d.scroll = self.f32()?;
        v.desc = (f & 1 != 0).then_some(d);
        v.pos = Vec2::new(self.f32()?, self.f32()?);
        v.last_pointer = Vec2::new(self.f32()?, self.f32()?);
        v.held = self.u32()?;
        v.over = self.u64()?;
        let h = &mut s.hud;
        for l in h.layouts.iter_mut() {
            for _ in 0..self.u32()? {
                let k = self.string()?;
                let p = crate::hud_layout::HudPlacement {
                    x: self.f32()?,
                    y: self.f32()?,
                    scale: self.f32()?,
                    opacity: self.f32()?,
                };
                l.insert(k, p);
            }
        }
        let f = self.u8()?;
        h.player = f & 3;
        h.edit = f & 4 != 0;
        h.selected = self.string()?;
        let start = Vec2::new(self.f32()?, self.f32()?);
        let mut from = crate::hud_layout::HudPlacement {
            x: self.f32()?,
            y: self.f32()?,
            ..Default::default()
        };
        from.scale = self.f32()?;
        from.opacity = self.f32()?;
        let spread = self.f32()?;
        h.drag = (f & 8 != 0).then_some((start, from, spread));
        for _ in 0..self.u32()? {
            let key = self.string()?;
            let base = self.rect()?;
            let shown = self.rect()?;
            let ckey = self.string()?;
            let lf = self.u8()?;
            let spec = crate::element::Customize {
                key: ckey,
                lock_move: lf & 1 != 0,
                lock_scale: lf & 2 != 0,
                lock_opacity: lf & 4 != 0,
                min_scale: self.f32()?,
                max_scale: self.f32()?,
                min_opacity: self.f32()?,
            };
            h.targets.push(crate::hud_layout::HudTarget {
                key,
                base,
                shown,
                spec,
            });
        }
        h.editor = self.rect()?;
        h.changed = self.u64()?;
        if version < 16 {
            return Some(s);
        }
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            s.touch_owners.insert(k, self.u32()?);
        }
        if version < 17 {
            return Some(s);
        }
        s.typeahead = (self.string()?, self.u64()?);
        if version < 18 {
            return Some(s);
        }
        for _ in 0..self.u32()? {
            let k = self.u64()?;
            let board = self.string()?;
            let built = self.u64()?;
            let len = self.u32()? as usize;
            let saved = self.bytes.get(self.at..self.at + len)?.to_vec();
            self.at += len;
            s.rive.insert(
                k,
                crate::ui::RiveSlot {
                    board,
                    player: None,
                    saved,
                    built,
                },
            );
        }
        Some(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The version 15 block of a state with no virtual cursor and no HUD
    /// layouts, edit mode or targets: its size.
    const V15: usize = 1 + 6 * 4 + 2 * 4 + 4 + 8 + 4 * 4 + 1 + 4 + 7 * 4 + 4 + 4 * 4 + 8;

    /// The version 16 block of a state with no touch control holding a finger.
    const V16: usize = 4;

    /// The version 17 block of a state with no typeahead.
    const V17: usize = 4 + 8;

    /// The version 18 block of a state with no Rive players.
    const V18: usize = 4;

    /// `b` without the `tail` bytes just before its layer stack.
    fn cut_before_docs(b: &[u8], tail: usize) -> Vec<u8> {
        let n = b.len();
        let mut k = n;
        let docs = loop {
            k -= 1;
            let len = u32::from_le_bytes(b[k - 4..k].try_into().unwrap()) as usize;
            if k + len == n && b[k] == b'{' {
                break len + 4;
            }
        };
        let cut = n - docs;
        let mut out = b[..cut - tail].to_vec();
        out.extend_from_slice(&b[cut..]);
        out
    }

    /// A save of a state with no focus groups, moves, recognisers or press,
    /// virtual cursor or HUD layouts without the version 14 and 15 blocks
    /// they take before the layer stack.
    fn without_v14_tail(b: &[u8]) -> Vec<u8> {
        cut_before_docs(
            b,
            3 * 4 + 4 + 8 + 8 + 4 + 1 + 8 + 8 + 8 + V15 + V16 + V17 + V18,
        )
    }

    /// A version 11 state block (one focus return) restores once upgraded:
    /// the dialog it had open gives the focus back where it was.
    #[test]
    fn a_version_11_state_upgrades_and_restores() {
        let mut ui = Ui::default();
        ui.state.scope = 7;
        ui.state.focus = 9;
        ui.state.focus_returns = vec![(0, 42)];
        let saved = ui.save_state();
        let v12 = without_v14_tail(&saved);
        // Version 11 wrote the one return as a u64 where 12 writes a counted list.
        let mut now = 7u64.to_le_bytes().to_vec();
        now.extend_from_slice(&1u32.to_le_bytes());
        now.extend_from_slice(&0u64.to_le_bytes());
        now.extend_from_slice(&42u64.to_le_bytes());
        let at = v12
            .windows(now.len())
            .position(|w| w == now.as_slice())
            .expect("the scope and its returns");
        let mut v11 = v12[..at].to_vec();
        v11.extend_from_slice(&7u64.to_le_bytes());
        v11.extend_from_slice(&42u64.to_le_bytes());
        v11.extend_from_slice(&v12[at + now.len()..]);
        v11[..4].copy_from_slice(&11u32.to_le_bytes());

        let mut fresh = Ui::default();
        assert!(
            fresh.restore_state(&v11).is_err(),
            "11 is read only through the upgrade"
        );
        let up = Ui::upgrade_state(&v11).expect("version 11 upgrades");
        assert_eq!(up, saved, "the upgrade writes what this version saves");
        fresh
            .restore_state(&up)
            .expect("the upgraded block restores");
        assert_eq!((fresh.state.scope, fresh.state.focus), (7, 9));
        assert_eq!(fresh.state.focus_returns, vec![(0, 42)]);
    }

    /// A version 13 state (no focus groups, moves or recognisers; hits
    /// without their group and recogniser) upgrades to what this version
    /// saves with those empty, and restores; read directly it is refused.
    #[test]
    fn a_version_13_state_upgrades_and_restores() {
        let mut ui = Ui::default();
        ui.state.focus = 9;
        ui.state.tick = 77;
        ui.state.hits = vec![Hit {
            key: 9,
            rect: Rect::new(1.0, 2.0, 3.0, 4.0),
            full: Rect::new(1.0, 2.0, 3.0, 4.0),
            flags: HIT_NAV,
            scope: 0,
            scroller: 0,
            owner: 0,
            inv: Affine::IDENTITY,
            local: Rect::new(1.0, 2.0, 3.0, 4.0),
            clip: Rect::new(0.0, 0.0, 100.0, 100.0),
            group: 0x1111_1111_1111_1111,
            gesture: 0x2222_2222_2222_2222,
        }];
        let v14 = ui.save_state();
        // Version 13 had no group and recogniser per hit…
        let mut ids = 0x1111_1111_1111_1111u64.to_le_bytes().to_vec();
        ids.extend_from_slice(&0x2222_2222_2222_2222u64.to_le_bytes());
        let at = v14
            .windows(ids.len())
            .position(|w| w == ids.as_slice())
            .expect("the hit's group and recogniser");
        let mut v13 = v14[..at].to_vec();
        v13.extend_from_slice(&v14[at + ids.len()..]);
        // …and no groups, moves, recognisers or press before the layer stack.
        let mut v13 = without_v14_tail(&v13);
        v13[..4].copy_from_slice(&13u32.to_le_bytes());

        let mut fresh = Ui::default();
        assert!(
            fresh.restore_state(&v13).is_err(),
            "13 is read only through the upgrade"
        );
        let up = Ui::upgrade_state(&v13).expect("version 13 upgrades");
        ui.state.hits[0].group = 0;
        ui.state.hits[0].gesture = 0;
        assert_eq!(
            up,
            ui.save_state(),
            "the upgrade writes what this version saves"
        );
        fresh
            .restore_state(&up)
            .expect("the upgraded block restores");
        assert_eq!((fresh.state.focus, fresh.state.tick), (9, 77));
        assert_eq!(fresh.state.hits.len(), 1);
        assert!(fresh.state.groups.is_empty() && fresh.state.recognizers.is_empty());
    }

    /// A version 14 state (no virtual cursor, no HUD layouts) upgrades to
    /// what this version saves with neither, and restores; read directly
    /// it is refused.
    #[test]
    fn a_version_14_state_upgrades_and_restores() {
        let mut ui = Ui::default();
        ui.state.focus = 5;
        ui.state.tick = 31;
        let saved = ui.save_state();
        let mut v14 = cut_before_docs(&saved, V15 + V16 + V17 + V18);
        v14[..4].copy_from_slice(&14u32.to_le_bytes());
        let mut fresh = Ui::default();
        assert!(
            fresh.restore_state(&v14).is_err(),
            "14 is read only through the upgrade"
        );
        let up = Ui::upgrade_state(&v14).expect("version 14 upgrades");
        assert_eq!(up, saved, "the upgrade writes what this version saves");
        fresh
            .restore_state(&up)
            .expect("the upgraded block restores");
        assert_eq!((fresh.state.focus, fresh.state.tick), (5, 31));
    }

    /// A version 15 state (no touch control holding a finger) upgrades to
    /// what this version saves, and restores; a held finger round-trips.
    #[test]
    fn a_version_15_state_upgrades_and_touch_owners_round_trip() {
        let mut ui = Ui::default();
        ui.state.tick = 12;
        let saved = ui.save_state();
        let mut v15 = cut_before_docs(&saved, V16 + V17 + V18);
        v15[..4].copy_from_slice(&15u32.to_le_bytes());
        assert!(Ui::default().restore_state(&v15).is_err());
        assert_eq!(Ui::upgrade_state(&v15).expect("version 15 upgrades"), saved);
        ui.state.touch_owners.insert(0xabcd, 3);
        let mut fresh = Ui::default();
        fresh.restore_state(&ui.save_state()).expect("restores");
        assert_eq!(fresh.state.touch_owners.get(&0xabcd), Some(&3));
        // Version 16 (no typeahead) upgrades too; a typeahead in progress round-trips.
        let mut v16 = cut_before_docs(&saved, V17 + V18);
        v16[..4].copy_from_slice(&16u32.to_le_bytes());
        assert_eq!(Ui::upgrade_state(&v16).expect("version 16 upgrades"), saved);
        ui.state.typeahead = ("ré".into(), 41);
        let mut fresh = Ui::default();
        fresh.restore_state(&ui.save_state()).expect("restores");
        assert_eq!(fresh.state.typeahead, ("ré".into(), 41));
        // Version 17 (no Rive players) upgrades too.
        let mut v17 = cut_before_docs(&saved, V18);
        v17[..4].copy_from_slice(&17u32.to_le_bytes());
        assert_eq!(Ui::upgrade_state(&v17).expect("version 17 upgrades"), saved);
        // Version 18 (no frozen text here: the same bytes) upgrades too.
        let mut v18 = saved.clone();
        v18[..4].copy_from_slice(&18u32.to_le_bytes());
        assert!(Ui::default().restore_state(&v18).is_err());
        assert_eq!(Ui::upgrade_state(&v18).expect("version 18 upgrades"), saved);
    }

    /// The virtual cursor and every player's HUD layout, the edit mode, a
    /// move in progress and last frame's targets come back as they were.
    #[test]
    fn the_virtual_cursor_and_hud_layouts_round_trip() {
        use crate::hud_layout::{HudPlacement, HudTarget};
        let mut ui = Ui::default();
        let v = &mut ui.state.vcursor;
        v.desc = Some(crate::VirtualCursorDesc {
            speed: 700.0,
            accel: -1.0,
            magnet: 0.25,
            scroll: 900.0,
        });
        v.pos = Vec2::new(120.5, 64.0);
        v.shown = true;
        v.placed = true;
        v.down = true;
        v.held = 7;
        v.over = 0xabcd;
        v.last_pointer = Vec2::new(3.0, 4.0);
        let h = &mut ui.state.hud;
        let p = HudPlacement {
            x: -40.0,
            y: 12.0,
            scale: 1.3,
            opacity: 0.6,
        };
        h.layouts[0].insert("stick".into(), p);
        h.layouts[2].insert("Hud/Map".into(), HudPlacement { scale: 0.7, ..p });
        h.player = 2;
        h.edit = true;
        h.selected = "Hud/Map".into();
        h.drag = Some((Vec2::new(9.0, 8.0), p, 31.0));
        h.targets.push(HudTarget {
            key: "Hud/Map".into(),
            base: Rect::new(1.0, 2.0, 30.0, 40.0),
            shown: Rect::new(5.0, 6.0, 21.0, 28.0),
            spec: crate::Customize {
                key: String::new(),
                lock_opacity: true,
                max_scale: 1.5,
                ..Default::default()
            },
        });
        h.editor = Rect::new(10.0, 20.0, 300.0, 200.0);
        h.changed = 99;
        let bytes = ui.save_state();
        let mut back = Ui::default();
        back.restore_state(&bytes).expect("restores");
        assert_eq!(back.state.vcursor, ui.state.vcursor);
        assert_eq!(back.state.hud, ui.state.hud);
        assert_eq!(back.save_state(), bytes);
    }
}
