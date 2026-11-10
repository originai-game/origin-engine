use super::*;

impl Lockstep {
    /// A room event: membership, and lockstep's own messages. Game messages
    /// are not lockstep's and are ignored here.
    pub fn on_event(&mut self, event: &Event) {
        match event {
            Event::Joined { peer, peers, .. } => {
                self.reset();
                self.me = Some(*peer);
                self.peers = peers.iter().cloned().collect();
                self.peers.entry(*peer).or_default();
                self.names = self.peers.clone();
                let now = self.now;
                self.heard = self.peers.keys().map(|p| (*p, now)).collect();
            }
            Event::PeerJoined { peer, name } => {
                self.peers.insert(*peer, name.clone());
                self.names.insert(*peer, name.clone());
                self.heard.insert(*peer, self.now);
            }
            Event::PeerLeft { peer } => self.departed(*peer),
            Event::Left { .. } => self.reset(),
            Event::Message {
                from,
                channel: Some(c),
                payload: Payload::Json(data),
            } if c == LOCKSTEP_CHANNEL => {
                if self.heard.contains_key(from) {
                    self.heard.insert(*from, self.now);
                }
                self.held_back.remove(from);
                self.message(*from, data)
            }
            Event::Message { .. } | Event::Error { .. } => {}
        }
    }

    /// A peer left the room. Its seat empties once the authority agreed
    /// how far its inputs reach ([`Lockstep::gate`]): what each peer
    /// received from it before it left may differ, since inputs cross the
    /// room unreliably.
    fn departed(&mut self, peer: u32) {
        self.peers.remove(&peer);
        self.theirs.remove(&peer);
        self.compared.remove(&peer);
        self.acks.remove(&peer);
        self.heard.remove(&peer);
        self.silent.remove(&peer);
        self.held_back.remove(&peer);
        for p in self.proposals.values_mut() {
            p.waiting.remove(&peer);
        }
        if let Some(run) = self.run.as_mut() {
            run.running.remove(&peer);
            run.admitted.remove(&peer);
        }
    }

    /// The last tick through which this peer has every input of `seat`'s
    /// open hold (the hold's `since - 1` when it has none), or what it told
    /// an authority already.
    fn contiguous(&self, seat: u32) -> u64 {
        if let Some(f) = self.frozen.get(&seat) {
            return *f;
        }
        let since = self
            .run
            .as_ref()
            .and_then(|r| r.seats.get(seat as usize)?.last().map(|h| h.since))
            .unwrap_or(0);
        let mut t = (self.confirmed + 1).max(since);
        while self.inputs.get(&t).is_some_and(|m| m.contains_key(&seat)) {
            t += 1;
        }
        t.saturating_sub(1)
    }

    /// The inputs of `seat` this peer holds for `ticks`, base64, while
    /// each is there.
    fn seat_inputs(&self, seat: u32, ticks: std::ops::RangeInclusive<u64>) -> Vec<Value> {
        ticks
            .map_while(|t| {
                self.inputs
                    .get(&t)
                    .and_then(|m| m.get(&seat))
                    .map(|b| json!(base64::encode(b)))
            })
            .collect()
    }

    /// The authority asks how far this peer has a seat's inputs, to drop
    /// it: this peer answers with its own last and the inputs past the
    /// authority's, and takes no later input of the seat from now on.
    pub(super) fn drop_asked(&mut self, from: u32, d: &Value) {
        let (Some(s), Some(p), Some(since), Some(after)) = (
            u32_of(&d["s"]),
            u32_of(&d["p"]),
            u64_of(&d["since"]),
            u64_of(&d["from"]),
        ) else {
            return;
        };
        let Some(run) = &self.run else { return };
        if Some(p) == self.me || !run.running.contains(&from) {
            return;
        }
        let open = run
            .seats
            .get(s as usize)
            .and_then(|h| h.last())
            .is_some_and(|h| h.peer == p && h.since == since && h.until.is_none());
        if !open {
            return;
        }
        let last = self.contiguous(s);
        self.frozen.insert(s, last);
        let list = self.seat_inputs(s, after + 1..=last);
        self.send(
            Some(from),
            json!({ "k": "drop!", "s": s, "since": since, "last": last, "t": after + 1, "i": list }),
        );
    }

    /// A drop every peer applies alike: the seat's hold ends at `until`,
    /// with the inputs to it; a peer that ran a later tick on a predicted
    /// input for the seat runs it again.
    pub(super) fn apply_drop(&mut self, d: &Value) {
        let (Some(p), Some(until)) = (u32_of(&d["p"]), u64_of(&d["until"])) else {
            return;
        };
        let left = d["left"].as_bool().unwrap_or(false);
        let Some(run) = self.run.as_mut() else { return };
        let seat = u32_of(&d["s"]);
        if let Some(s) = seat {
            let Some(since) = u64_of(&d["since"]) else {
                return;
            };
            let Some(h) = run
                .seats
                .get_mut(s as usize)
                .and_then(|h| h.iter_mut().find(|h| h.peer == p && h.since == since))
            else {
                return;
            };
            if h.until.is_some() {
                return;
            }
            h.until = Some(until);
            h.left = left;
        } else if !run.running.contains(&p) {
            return;
        }
        run.running.remove(&p);
        run.admitted.remove(&p);
        self.acks.remove(&p);
        self.theirs.remove(&p);
        self.compared.remove(&p);
        self.silent.remove(&p);
        if !left {
            self.held_back.insert(p);
        }
        self.drops += 1;
        if let Some(s) = seat {
            self.frozen.remove(&s);
            self.proposals.remove(&s);
            let t0 = u64_of(&d["t"]).unwrap_or(until + 1);
            for (i, b) in d["i"].as_array().into_iter().flatten().enumerate() {
                let t = t0 + i as u64;
                if t <= until
                    && let Some(bytes) = b.as_str().and_then(base64::decode)
                {
                    self.take_input(t, s, bytes);
                }
            }
            let later: Vec<u64> = self.inputs.range(until + 1..).map(|(t, _)| *t).collect();
            for t in later {
                if let Some(m) = self.inputs.get_mut(&t) {
                    m.remove(&s);
                }
            }
            // A tick past it ran with the seat held (on a prediction).
            if self.frontier > until + 1 {
                self.mark_wrong(until + 1);
            }
            self.notes.push(format!(
                "lockstep: seat {s} dropped from tick {} (peer {p} {}; its last input is for tick {until})",
                until + 1,
                if left { "left" } else { "stalled" }
            ));
        } else {
            self.notes.push(format!(
                "lockstep: spectator {p} dropped ({})",
                if left { "left" } else { "stalled" }
            ));
        }
        if Some(p) == self.me {
            self.dropped_self();
        }
    }

    /// At every boundary while running: the peers silent past the notice
    /// (og.PeerStalled), and at the acting authority (the lowest running
    /// peer that is not silent past the stall timeout) the drops of the
    /// seats whose holder left or stalled, agreed with the other peers.
    pub(super) fn watch(&mut self) {
        let (Some(run), Some(me)) = (self.run.as_ref(), self.me) else {
            return;
        };
        let timeout = run.stall_timeout;
        let now = self.now;
        let notice = if timeout > 0 {
            STALL_NOTICE_MS.min(timeout / 2)
        } else {
            STALL_NOTICE_MS
        };
        let silence = |p: u32| now.saturating_sub(self.heard.get(&p).copied().unwrap_or(now));
        let stalled = |p: u32| p != me && timeout > 0 && silence(p) >= timeout;
        let mut stalls = Vec::new();
        let mut quiet = BTreeSet::new();
        for p in run.running.iter().copied().filter(|p| *p != me) {
            if silence(p) >= notice {
                quiet.insert(p);
                if !self.silent.contains(&p) {
                    stalls.push((p, run.open_seat(p), silence(p)));
                }
            }
        }
        let acting = run
            .running
            .iter()
            .copied()
            .find(|p| self.peers.contains_key(p) && !stalled(*p));
        let mut targets = Vec::new();
        let mut spectators = Vec::new();
        let mut answer: BTreeSet<u32> = BTreeSet::new();
        if acting == Some(me) {
            for (s, holds) in run.seats.iter().enumerate() {
                if let Some(h) = holds.last().filter(|h| h.until.is_none() && h.peer != me) {
                    let gone = !self.peers.contains_key(&h.peer);
                    if gone || stalled(h.peer) {
                        targets.push((s as u32, h.peer, h.since, gone));
                    }
                }
            }
            for p in run.running.iter().copied() {
                if run.open_seat(p).is_none() && stalled(p) {
                    spectators.push(p);
                }
            }
            answer = run
                .running
                .iter()
                .copied()
                .filter(|p| *p != me && self.peers.contains_key(p) && !stalled(*p))
                .collect();
        }
        self.silent = quiet;
        self.stalls.extend(stalls);
        if acting != Some(me) {
            return;
        }
        for p in spectators {
            let d = json!({ "k": "drop", "p": p, "until": 0, "left": false });
            self.send(None, d.clone());
            self.apply_drop(&d);
        }
        for (s, p, since, left) in targets {
            if self.proposals.get(&s).is_some_and(|x| x.since == since) {
                continue;
            }
            let from = self.contiguous(s);
            self.frozen.insert(s, from);
            let waiting: BTreeSet<u32> = answer.iter().copied().filter(|q| *q != p).collect();
            self.notes.push(format!(
                "lockstep: peer {p} {} seat {s}; agreeing its last input with peers {waiting:?}",
                if left { "left" } else { "stalled in" }
            ));
            self.proposals.insert(
                s,
                Proposal {
                    peer: p,
                    since,
                    left,
                    from,
                    last: from,
                    low: from,
                    inputs: IdMap::new(),
                    waiting,
                },
            );
            self.send(
                None,
                json!({ "k": "drop?", "s": s, "p": p, "since": since, "from": from }),
            );
        }
        // Peers that left or stalled meanwhile answer no more.
        let seats: Vec<u32> = self.proposals.keys().copied().collect();
        for s in seats {
            let p = self.proposals.get_mut(&s).expect("listed");
            p.waiting.retain(|q| answer.contains(q));
            if !p.waiting.is_empty() {
                continue;
            }
            let p = self.proposals.remove(&s).expect("listed");
            let left = p.left || !self.peers.contains_key(&p.peer);
            let until = p.last.max(p.since.saturating_sub(1));
            let t0 = (p.low + 1).max(p.since);
            let mut list = Vec::new();
            for t in t0..=until {
                let b = if t <= p.from {
                    self.inputs.get(&t).and_then(|m| m.get(&s)).cloned()
                } else {
                    p.inputs.get(&t).cloned()
                };
                match b {
                    Some(b) => list.push(json!(base64::encode(&b))),
                    None => break,
                }
            }
            let d = json!({
                "k": "drop", "s": s, "p": p.peer, "since": p.since, "until": until,
                "left": left, "t": t0, "i": list,
            });
            self.send(None, d.clone());
            self.apply_drop(&d);
        }
    }

    pub(super) fn mark_wrong(&mut self, tick: u64) {
        self.rewind_to = Some(self.rewind_to.map_or(tick, |r| r.min(tick)));
    }

    /// A real input for `seat` at `tick`, received or made here.
    pub(super) fn take_input(&mut self, tick: u64, seat: u32, bytes: Vec<u8>) {
        if self.frozen.get(&seat).is_some_and(|f| tick > *f)
            || tick < self.hashed
            || self
                .inputs
                .get(&tick)
                .is_some_and(|m| m.contains_key(&seat))
        {
            return;
        }
        if self
            .used
            .get(&tick)
            .and_then(|u| u.get(seat as usize))
            .is_some_and(|u| *u != bytes)
        {
            self.mark_wrong(tick);
        }
        self.inputs.entry(tick).or_default().insert(seat, bytes);
        let e = self.seat_last.entry(seat).or_insert(tick);
        *e = (*e).max(tick);
    }
}
