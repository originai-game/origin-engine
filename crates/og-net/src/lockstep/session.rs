use super::*;

impl Lockstep {
    pub fn new() -> Lockstep {
        Lockstep::default()
    }

    /// This peer's number in its room.
    pub fn me(&self) -> Option<u32> {
        self.me
    }

    /// Whether the world takes its ticks in lockstep (or rollback) now.
    /// Whether [`Self::gate`] would start a run now: lockstep is enabled,
    /// none runs or is being handed over, and this peer is the authority
    /// of a room that holds `players` peers. A host refuses a start it
    /// cannot share by not calling the gate (the room stays a lobby).
    pub fn would_start(&self, cfg: &Config) -> bool {
        cfg.enabled
            && self.run.is_none()
            && self.handover.is_none()
            && !self.rejoining
            && self.me.is_some()
            && self.authority() == self.me
            && self.peers.len() as u32 >= cfg.players.max(1)
    }

    pub fn running(&self) -> bool {
        self.run.is_some()
    }

    /// Whether tick `tick` runs again (a rollback's re-simulation): its
    /// sounds and effects were already presented once.
    pub fn resimulating(&self, tick: u64) -> bool {
        self.run.is_some() && tick < self.frontier
    }

    /// The last tick whose world is final (every input it ran on real and
    /// confirmed), while running.
    pub fn final_tick(&self) -> Option<u64> {
        self.run.as_ref().map(|_| self.hashed)
    }

    /// The shared hash of the final world at `tick`, while kept (the last
    /// 1024, when hashing): `None` before the tick is final.
    pub fn final_hash(&self, tick: u64) -> Option<u64> {
        self.finals.get(&tick).copied()
    }

    /// The wall clock now, in ms, given before every [`Lockstep::gate`] and
    /// event. It decides only when a silent peer is dropped; the tick the
    /// seat empties at is agreed. A jump past half of [`STALL_NOTICE_MS`]
    /// is this process's own pause (a suspended process or tab), not the
    /// others' silence.
    pub fn set_clock(&mut self, now: u64) {
        if self.now > 0 && now > self.now + STALL_NOTICE_MS / 2 {
            let gap = now - self.now;
            for h in self.heard.values_mut() {
                *h += gap;
            }
        }
        self.now = self.now.max(now);
    }

    /// The seats that change hands as tick `tick` starts: og.SeatDropped
    /// the tick after a dropped peer's last input, og.SeatRejoined when a
    /// seat held before is taken again. Read from the agreed seats alone,
    /// so a re-simulated tick and a late joiner's world see the same.
    pub fn seat_events(&self, tick: u64) -> Vec<SeatEvent> {
        let Some(run) = &self.run else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (s, holds) in run.seats.iter().enumerate() {
            for (i, h) in holds.iter().enumerate() {
                if i > 0 && h.since == tick {
                    out.push(SeatEvent {
                        seat: s as u32,
                        peer: h.peer,
                        dropped: false,
                        left: false,
                    });
                }
                if h.until.is_some_and(|u| u + 1 == tick) {
                    out.push(SeatEvent {
                        seat: s as u32,
                        peer: h.peer,
                        dropped: true,
                        left: h.left,
                    });
                }
            }
        }
        out
    }

    /// og.PeerStalled to send: (peer, its seat, ms silent), once per silence.
    pub fn take_stalls(&mut self) -> Vec<(u32, Option<u32>, u64)> {
        std::mem::take(&mut self.stalls)
    }

    /// The tick the world reached furthest (the present, below which a
    /// tick is re-simulated).
    pub fn frontier(&self) -> u64 {
        self.frontier
    }

    /// This peer's seat at `tick` (`None`: watching, or not running).
    pub fn seat(&self, tick: u64) -> Option<u32> {
        let (run, me) = (self.run.as_ref()?, self.me?);
        run.seat_of(me, tick).or_else(|| run.open_seat(me))
    }

    /// The seats while running, else `None`.
    pub fn players(&self) -> Option<u32> {
        self.run.as_ref().map(|r| r.players)
    }

    /// The input og.net.submit gave last: what the local player holds.
    pub fn set_local(&mut self, input: Vec<u8>) {
        self.local = input;
    }

    pub fn local(&self) -> &[u8] {
        &self.local
    }

    /// Commands (og.net.command, packed) for this peer's next input: sent
    /// once, after its state, on the next tick run for the first time.
    /// Outside a run they are not lockstep's (the runtime plays them on
    /// the local player), and dropped here.
    pub fn push_commands(&mut self, bytes: &[u8]) {
        if self.run.is_some() {
            self.commands.extend_from_slice(bytes);
        }
    }

    /// Seat `seat`'s input for `tick`, as the tick runs it (in rollback, a
    /// prediction until the real one arrives): `None` for an empty input
    /// (a seat nobody holds then, or a tick before its holder's first).
    pub fn input(&self, tick: u64, seat: u32) -> Option<&[u8]> {
        self.used
            .get(&tick)
            .and_then(|m| m.get(seat as usize))
            .map(Vec::as_slice)
            .filter(|b| !b.is_empty())
    }

    /// The messages to send since the last call, for the lockstep channel.
    pub fn take_out(&mut self) -> Vec<Out> {
        std::mem::take(&mut self.out)
    }

    /// og.Desync, once: (first divergent tick, the first peer whose world
    /// differs, how many differ).
    pub fn take_desync(&mut self) -> Option<(u64, u32, u32)> {
        self.desync_event.take()
    }

    pub fn take_notes(&mut self) -> Vec<String> {
        std::mem::take(&mut self.notes)
    }

    pub(super) fn authority(&self) -> Option<u32> {
        match &self.run {
            Some(r) => r
                .running
                .iter()
                .copied()
                .find(|p| self.peers.contains_key(p)),
            None => self.peers.keys().next().copied(),
        }
    }

    pub(super) fn send(&mut self, to: Option<u32>, data: Value) {
        self.out.push((to, data, false));
    }

    /// Leaves lockstep and forgets the room (this peer left it, or lost it).
    pub(super) fn reset(&mut self) {
        let local = std::mem::take(&mut self.local);
        let desync = (self.desync.take(), self.desync_event.take());
        let (now, drops) = (self.now, self.drops);
        *self = Lockstep::default();
        self.local = local;
        (self.desync, self.desync_event) = desync;
        (self.now, self.drops) = (now, drops);
    }

    /// This peer was dropped (it stalled): it leaves the run, keeps the
    /// room, holds its world and tells the others it is back, so the
    /// authority admits it again from a snapshot.
    pub(super) fn dropped_self(&mut self) {
        let me = self.me;
        let peers = std::mem::take(&mut self.peers);
        let names = std::mem::take(&mut self.names);
        let now = self.now;
        self.reset();
        self.me = me;
        self.heard = peers.keys().map(|p| (*p, now)).collect();
        self.peers = peers;
        self.names = names;
        self.rejoining = true;
        self.notes.push(
            "lockstep: this peer was dropped (it stalled); it waits for the authority's world to rejoin".into(),
        );
        self.send(None, json!({ "k": "back" }));
    }

    /// The state of a run starting (or joined) with the world at `tick`.
    pub(super) fn begin(&mut self, run: Run, tick: u64, snapshot: Vec<u8>) {
        self.full_since = None;
        self.inputs.retain(|t, _| *t >= tick);
        self.used.clear();
        self.ring.clear();
        self.ring.insert(tick, (0, snapshot));
        self.confirmed = tick.saturating_sub(1);
        self.frontier = tick;
        self.hashed = tick;
        self.rewind_to = None;
        self.batch_from = tick + 1;
        self.batch.clear();
        self.frozen.clear();
        self.proposals.clear();
        self.rejoining = false;
        let now = self.now;
        for p in &run.running {
            self.heard.insert(*p, now);
        }
        let me = self.me;
        self.acks = run
            .running
            .iter()
            .filter(|p| Some(**p) != me)
            .map(|p| (*p, tick.saturating_sub(1)))
            .collect();
        self.run = Some(run);
    }
}
