//! On-screen touch control zones (`touchStick`, `touchPad`, `touchButton`).

use super::*;

impl Tree {
    /// The on-screen controls' zones for og-input's action map
    /// ([`og_input::ActionMap::set_zones`]), after [`Tree::present`]: each
    /// shown, enabled control's box in surface px. `None` when none came,
    /// went, moved or changed since the last call; such a frame allocates
    /// nothing. A control that is no zone any more (disabled, hidden,
    /// gone) reads as let go: og-input no longer says so.
    pub fn take_zones(&mut self) -> Option<Vec<og_input::ControlZone>> {
        let mut k = 0;
        let mut changed = false;
        let mut was = std::mem::take(&mut self.input.zone_was);
        was.clear();
        was.extend_from_slice(&self.input.zone_from);
        for i in 0..self.input.zone_els.len() {
            let e = self.input.zone_els[i] as usize;
            let p = &self.input.per[e];
            let Some((kind, action)) = &p.zone else {
                continue;
            };
            let Some(&(x, y, true, _)) = self.abs.get(e) else {
                continue;
            };
            let b = self.arena.box_of(self.els[e].node);
            if self.els[e].dead || p.states & DISABLED != 0 || b.w <= 0.0 || b.h <= 0.0 {
                continue;
            }
            let same = |z: &og_input::ControlZone| {
                z.kind == *kind
                    && *z.action == **action
                    && [z.x, z.y, z.width, z.height, z.dead_zone] == [x, y, b.w, b.h, p.dead_zone]
            };
            let next = || og_input::ControlZone {
                action: action.to_string(),
                kind: *kind,
                x,
                y,
                width: b.w,
                height: b.h,
                dead_zone: p.dead_zone,
                floating: false,
            };
            match self.input.zone_from.get_mut(k) {
                Some(f) => *f = e as u32,
                None => self.input.zone_from.push(e as u32),
            }
            match self.input.zones.get_mut(k) {
                Some(z) if same(z) => {}
                Some(z) => {
                    *z = next();
                    changed = true;
                }
                None => {
                    self.input.zones.push(next());
                    changed = true;
                }
            }
            k += 1;
        }
        self.input.zone_from.truncate(k);
        if self.input.zones.len() != k {
            self.input.zones.truncate(k);
            changed = true;
        }
        for &e in &was {
            if !self.input.zone_from.contains(&e) && !self.els[e as usize].dead {
                self.zone_state(e, false, [0.0; 2], [0.0; 2]);
            }
        }
        self.input.zone_was = was;
        self.refresh_data();
        changed.then(|| self.input.zones.clone())
    }

    /// What og-input read from each zone [`Tree::take_zones`] handed out,
    /// in that order (og-input's `ActionMap::zone_states`, surface px over
    /// `scale` tree units): a control held by any pointer is `active:`,
    /// its knob moves the nodes marked `knob` in it, as Unity's
    /// OnScreenStick draws from its control, and a held d-pad carries the
    /// way it reads as `data-dir-up|right|down|left` (two on a diagonal,
    /// none in its dead middle), which its classes read.
    pub fn zone_states<'a>(
        &mut self,
        states: impl IntoIterator<Item = &'a og_input::ZoneState>,
        scale: f32,
    ) {
        let mut states = states.into_iter();
        for k in 0..self.input.zone_from.len() {
            let e = self.input.zone_from[k];
            let (held, knob, value) = match states.next() {
                Some(s) => (
                    s.pointer.is_some(),
                    [s.knob.x / scale, s.knob.y / scale],
                    [s.value.x, s.value.y],
                ),
                None => (false, [0.0; 2], [0.0; 2]),
            };
            if self.els[e as usize].dead {
                continue;
            }
            self.zone_state(e, held, knob, value);
        }
        self.refresh_data();
    }

    /// One control as og-input read it: whether a pointer holds it, its
    /// knob in tree units and the value it gives its action (y up).
    fn zone_state(&mut self, e: u32, held: bool, knob: [f32; 2], value: [f32; 2]) {
        self.set_state(e, PRESSED, held);
        let p = &mut self.input.per[e as usize];
        if p.knob != knob {
            p.knob = knob;
            self.paint = true;
        }
        let pad = held && matches!(p.zone, Some((og_input::ZoneKind::Pad, _)));
        for (name, on) in [
            ("dir-up", value[1] > 0.0),
            ("dir-right", value[0] > 0.0),
            ("dir-down", value[1] < 0.0),
            ("dir-left", value[0] < 0.0),
        ] {
            self.mark(e, name, pad && on);
        }
    }

    /// The knob offset node `e` moves by at paint: its control's, when it
    /// is marked `knob`, held inside the control's ring (its radius less
    /// the knob's, each half its box's smaller side): og-input keeps the
    /// finger's offset to the ring's edge, and the knob is drawn art.
    pub(in crate::tree) fn knob_of(&self, e: u32) -> [f32; 2] {
        if !self.input.per(e).is_some_and(|p| p.rides_knob) {
            return [0.0; 2];
        }
        let mut c = self.els[e as usize].parent;
        while c != NONE {
            if let Some(p) = self.input.per(c)
                && p.zone.is_some()
            {
                let radius = |el: u32| {
                    let b = self.arena.box_of(self.els[el as usize].node);
                    b.w.min(b.h) / 2.0
                };
                let room = (radius(c) - radius(e)).max(0.0);
                let [x, y] = p.knob;
                let len = (x * x + y * y).sqrt();
                return if len > room {
                    [x / len * room, y / len * room]
                } else {
                    p.knob
                };
            }
            c = self.els[c as usize].parent;
        }
        [0.0; 2]
    }
}
