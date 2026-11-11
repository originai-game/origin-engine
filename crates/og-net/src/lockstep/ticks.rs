use super::*;

impl Lockstep {
    /// At a tick boundary, before tick `tick`: whether it may run.
    /// `snapshot` gives the world's snapshot now, when one is needed (to
    /// start, or for a newcomer).
    pub fn gate(&mut self, cfg: &Config, tick: u64, snapshot: &mut dyn FnMut() -> Vec<u8>) -> Gate {
        self.state_bytes = cfg.state_bytes as usize;
        if let Some((run, at, bytes)) = self.handover.take() {
            if run.shape != cfg.shape {
                self.deferred.clear();
                self.notes.push(format!(
                    "lockstep: the start of tick {at} was refused: its input shape (og.net.shape) differs from this peer's; every peer sets the same one before the room starts"
                ));
                return Gate::Go;
            }
            self.notes.push(format!(
                "lockstep: joined at tick {at} as {} of {} ({} peers running, {})",
                match self.me.and_then(|me| run.open_seat(me)) {
                    Some(s) => format!("player {s}"),
                    None => "a spectator".into(),
                },
                run.players,
                run.running.len(),
                mode_text(&run),
            ));
            let bytes = Rc::new(bytes);
            self.begin(run, at, bytes.clone());
            for (from, d) in std::mem::take(&mut self.deferred) {
                self.message(from, &d);
            }
            return Gate::Restore {
                snapshot: bytes,
                tick: at,
            };
        }
        let Some(me) = self.me else {
            return Gate::Go;
        };
        if !cfg.enabled {
            if self.run.take().is_some() {
                self.notes
                    .push(format!("lockstep: turned off at tick {tick}; the other peers will wait for this one's input"));
            }
            return Gate::Go;
        }
        if self.run.is_none() {
            if self.rejoining {
                // Dropped: the world holds until the authority's arrives
                // (unless every other peer is gone).
                if self.peers.len() > 1 {
                    self.waits += 1;
                    return Gate::Wait;
                }
                self.rejoining = false;
            }
            let players = cfg.players.max(1);
            if (self.peers.len() as u32) < players {
                self.full_since = None;
                return Gate::Go;
            }
            if self.authority() != Some(me) {
                // The room is full and the authority's start is on its way:
                // this lobby's ticks would be replaced by its snapshot, so
                // they wait for it (a slow authority must not leave a
                // scripted run's steps spent in a lobby), but only so long
                // (an authority that never starts leaves a free lobby).
                let since = *self.full_since.get_or_insert(self.now);
                let bound = match cfg.stall_timeout {
                    0 => START_HOLD_MS,
                    t => u64::from(t),
                };
                if self.now.saturating_sub(since) < bound {
                    self.waits += 1;
                    return Gate::Wait;
                }
                return Gate::Go;
            }
            self.start(cfg, tick, snapshot());
        }
        if let Some(m) = self.rewind_to.take()
            && m < tick
        {
            match self.ring.get(&m) {
                Some((_, bytes)) => {
                    self.rollbacks += 1;
                    // The ticks from m run again: what they ran with goes,
                    // so an input arriving meanwhile marks nothing stale.
                    self.used.retain(|t, _| *t < m);
                    return Gate::Rewind {
                        snapshot: bytes.clone(),
                        tick: m,
                    };
                }
                None => self.notes.push(format!(
                    "lockstep: no snapshot of tick {m} to roll back to (the ring holds {:?}); the worlds will differ",
                    self.ring.keys().next()
                )),
            }
        }
        self.watch();
        if self.run.is_none() {
            // This peer was dropped meanwhile.
            return Gate::Wait;
        }
        if self.authority() == Some(me) {
            let run = self.run.as_ref().expect("running");
            // A newcomer waits for the seats being emptied: one that is a
            // player back after a lost connection may be here before its
            // old seat's drop is agreed, and would find it taken.
            let settled = self.proposals.is_empty()
                && run.seats.iter().all(|h| {
                    h.last()
                        .is_none_or(|l| l.until.is_some() || self.peers.contains_key(&l.peer))
                });
            let newcomers: Vec<u32> = self
                .peers
                .keys()
                .copied()
                .filter(|p| settled && !run.admitted.contains(p) && !self.held_back.contains(p))
                .collect();
            if !newcomers.is_empty() {
                // A newcomer gets a confirmed world: this one when every
                // tick run so far is, else the last final snapshot.
                let (at, bytes) = if self.confirmed + 1 >= tick && self.rewind_to.is_none() {
                    (tick, Rc::new(snapshot()))
                } else {
                    let (t, (_, b)) = self.ring.iter().next().expect("the ring's base");
                    (*t, b.clone())
                };
                for p in newcomers {
                    self.admit(p, tick, at, &bytes);
                }
            }
        }
        self.advance_confirmed();
        let run = self.run.as_ref().expect("running");
        let predict = run.max_prediction.min(u64::from(cfg.budget));
        if tick > self.confirmed + predict {
            let next = self.confirmed + 1;
            let have = self.inputs.get(&next);
            self.waiting_for = (0..run.seats.len())
                .filter(|s| {
                    run.hold(*s, next).is_some()
                        && !have.is_some_and(|m| m.contains_key(&(*s as u32)))
                })
                .map(|s| s as u32)
                .collect();
            self.waits += 1;
            // What the others lack goes again: a lost input message, or a
            // lost acknowledgement, would otherwise hold both sides.
            self.send_inputs(None);
            return Gate::Wait;
        }
        self.waiting_for.clear();
        if tick < self.frontier {
            self.resimulated += 1;
        }
        let used = self.inputs_for(tick);
        self.used.insert(tick, used);
        Gate::Go
    }

    fn start(&mut self, cfg: &Config, tick: u64, bytes: Vec<u8>) {
        let players = cfg.players.max(1);
        let delay = u64::from(cfg.input_delay.max(1));
        let peers: Vec<u32> = self.peers.keys().copied().collect();
        let mut seats = vec![Vec::new(); players as usize];
        for (s, p) in peers.iter().take(players as usize).enumerate() {
            seats[s].push(Hold {
                peer: *p,
                since: tick + delay,
                until: None,
                left: false,
            });
        }
        let run = Run {
            start: tick,
            players,
            delay,
            hash_every: u64::from(cfg.hash_every),
            max_prediction: if cfg.rollback {
                u64::from(cfg.max_prediction)
            } else {
                0
            },
            seats,
            running: peers.iter().copied().collect(),
            admitted: peers.iter().copied().collect(),
            stall_timeout: u64::from(cfg.stall_timeout),
            rejoin: cfg.rejoin,
            shape: cfg.shape,
        };
        let msg = start_msg(&run, tick, &bytes, &self.inputs, &self.seat_last);
        self.notes.push(format!(
            "lockstep: started at tick {tick}: {players} seats for peers {peers:?}, input delay {delay}, hashes every {} ticks, {}",
            run.hash_every,
            mode_text(&run),
        ));
        self.begin(run, tick, Rc::new(bytes));
        self.send(None, msg);
    }

    /// Admits `peer` at this peer's boundary `tick`, handing it the
    /// confirmed world `bytes` of tick `at`.
    fn admit(&mut self, peer: u32, tick: u64, at: u64, bytes: &[u8]) {
        let run = self.run.as_mut().expect("running");
        let name = self.names.get(&peer).cloned().unwrap_or_default();
        let was = self.was.get(&peer).copied();
        let free = |h: &Vec<Hold>| h.last().is_none_or(|h| h.until.is_some());
        // With rejoin, a returning player takes back the seat it held (the
        // same peer, the peer it was before its connection was lost, or a
        // newcomer of the same name), else the first free one; without,
        // only a seat nobody held yet.
        let free = if run.rejoin {
            let held = |by: &dyn Fn(&Hold) -> bool| {
                run.seats
                    .iter()
                    .position(|h| free(h) && h.last().is_some_and(by))
            };
            held(&|l| l.peer == peer || was == Some(l.peer))
                .or_else(|| held(&|l| !name.is_empty() && self.names.get(&l.peer) == Some(&name)))
                .or_else(|| run.seats.iter().position(free))
        } else {
            run.seats.iter().position(Vec::is_empty)
        };
        // After every input this peer made already (in rollback a boundary
        // may be a re-simulated one, its inputs sent up to the frontier's):
        // a peer that confirms a tick from `since` needs one sent after
        // the admit.
        let since = tick.max(self.frontier) + run.delay;
        if let Some(s) = free {
            run.seats[s].push(Hold {
                peer,
                since,
                until: None,
                left: false,
            });
        }
        self.heard.insert(peer, self.now);
        run.admitted.insert(peer);
        run.running.insert(peer);
        self.acks.insert(peer, at.saturating_sub(1));
        self.notes.push(format!(
            "lockstep: admitted peer {peer} at tick {tick} as {} (from the world of tick {at})",
            match free {
                Some(s) => format!("player {s} from tick {since}"),
                None => "a spectator".into(),
            }
        ));
        self.send(
            None,
            json!({ "k": "admit", "peer": peer, "seat": free, "tick": at, "since": since }),
        );
        let run = self.run.as_ref().expect("running");
        let msg = start_msg(run, at, bytes, &self.inputs, &self.seat_last);
        self.send(Some(peer), msg);
    }

    /// Sends this peer's own inputs the others lack: every input from the
    /// lowest acknowledgement on (at most [`MAX_RESEND`]), to `to` or all.
    pub(super) fn send_inputs(&mut self, to: Option<u32>) {
        if to.is_none() {
            // A peer far behind the others (a stalled one) gets its own
            // message, so the rest are not held to its window.
            let lo = self.acks.values().copied().min().unwrap_or(0);
            let hi = self.acks.values().copied().max().unwrap_or(0);
            if hi - lo > MAX_RESEND / 2 {
                let peers: Vec<u32> = self.acks.keys().copied().collect();
                for p in peers {
                    self.send_inputs(Some(p));
                }
                return;
            }
        }
        let (Some(run), Some(me)) = (self.run.as_ref(), self.me) else {
            return;
        };
        let floor = match to {
            Some(p) => self.acks.get(&p).copied(),
            None => self.acks.values().copied().min(),
        };
        // Every message carries the acknowledgement, inputs or not (a
        // spectator's, or a seated peer's before its first input).
        let mut msg = json!({ "k": "inputs", "a": self.confirmed, "e": run.epoch() });
        if let Some(floor) = floor
            && let Some((&first, _)) = self.own.range(floor + 1..).next()
            && let Some(seat) = run.seat_of(me, first)
        {
            let ticks: Vec<String> = self
                .own
                .range(first..)
                .take(MAX_RESEND as usize)
                .enumerate()
                .take_while(|(i, (t, _))| {
                    **t == first + *i as u64 && run.seat_of(me, **t) == Some(seat)
                })
                .map(|(_, (_, b))| base64::encode(b))
                .collect();
            msg["t"] = json!(first);
            msg["s"] = json!(seat);
            msg["h"] = json!(run.hold(seat as usize, first).map(|h| h.since));
            msg["i"] = json!(ticks);
        }
        self.out.push((to, msg, true));
    }

    /// After tick `tick_after - 1` ran (the world's tick is `tick_after`):
    /// on a tick run for the first time, this peer's input for the tick
    /// `inputDelay` on; then the world's hash and snapshot (`hash` gives
    /// both), kept for a rollback, and the hashes of every tick now final.
    pub fn after_tick(&mut self, tick_after: u64, hash: &mut dyn FnMut() -> (u64, Vec<u8>)) {
        let Some(run) = &self.run else { return };
        let ran = tick_after.saturating_sub(1);
        let (delay, every, rollback) = (run.delay, run.hash_every, run.max_prediction > 0);
        if tick_after > self.frontier {
            self.frontier = tick_after;
            let target = ran + delay;
            let commands = std::mem::take(&mut self.commands);
            if let Some(seat) = self.me.and_then(|me| run.seat_of(me, target)) {
                let mut bytes = self.local.clone();
                bytes.extend_from_slice(&commands);
                self.own.insert(target, bytes.clone());
                self.take_input(target, seat, bytes);
            }
            let keep_own = tick_after.saturating_sub(KEEP_OWN + 2 * delay);
            self.own.retain(|t, _| *t >= keep_own);
            self.send_inputs(None);
        }
        if every > 0 || rollback {
            let (hash, bytes) = hash();
            self.ring.insert(tick_after, (hash, Rc::new(bytes)));
        }
        self.advance_confirmed();
        // Every tick whose world is final now: confirmed, run with its
        // real inputs, and not marked for a rollback.
        loop {
            let h = self.hashed + 1;
            let ok = h <= tick_after
                && h - 1 <= self.confirmed
                && self.rewind_to.is_none_or(|m| m > h - 1);
            if !ok {
                break;
            }
            self.hashed = h;
            if every == 0 {
                continue;
            }
            let Some((hv, snapshot)) = self.ring.get(&h).cloned() else {
                continue;
            };
            self.finals.insert(h, hv);
            if self.finals.len() > KEEP_FINALS {
                let first = *self.finals.keys().next().expect("non-empty");
                self.finals.remove(&first);
            }
            self.mine.insert(h, (hv, snapshot));
            self.batch.push(json!(hex(hv)));
            if h.is_multiple_of(every) && !self.batch.is_empty() {
                let hashes = std::mem::take(&mut self.batch);
                let from = self.batch_from;
                self.batch_from = h + 1;
                self.send(None, json!({ "k": "hash", "from": from, "h": hashes }));
            }
        }
        let hashed = self.hashed;
        self.ring.retain(|t, _| *t >= hashed);
        self.used.retain(|t, _| *t >= hashed);
        // Each seat's last real input stays, for the next prediction, and
        // every input a running peer may lack, for a drop to hand on.
        let floor = self
            .acks
            .values()
            .copied()
            .min()
            .map_or(hashed, |a| a.min(hashed))
            .max(hashed.saturating_sub(CONFIRM_AHEAD));
        self.inputs.retain(|t, _| *t + 1 >= floor);
        let peers: Vec<u32> = self.theirs.keys().copied().collect();
        for p in peers {
            self.compare(p);
        }
        // Snapshots every running peer compared (or the oldest, past the cap) go.
        let me = self.me;
        let others: Vec<u32> = self
            .run
            .as_ref()
            .map(|r| {
                r.running
                    .iter()
                    .copied()
                    .filter(|p| Some(*p) != me)
                    .collect()
            })
            .unwrap_or_default();
        let floor = others
            .iter()
            .map(|p| self.compared.get(p).copied().unwrap_or(0))
            .min();
        let desync_tick = self.desync.as_ref().map(|d| d.tick);
        if let Some(floor) = floor {
            self.mine
                .retain(|t, _| *t > floor || Some(*t) == desync_tick);
        }
        while self.mine.len() > MAX_KEPT {
            let first = *self.mine.keys().next().expect("non-empty");
            self.mine.remove(&first);
        }
    }

    /// The probe's `desync` dump: the first divergent tick, the hashes and
    /// the component-level diff of the two snapshots (`null` before any
    /// desync).
    pub fn dump(&self) -> Value {
        let Some(d) = &self.desync else {
            return json!({
                "desync": false,
                "running": self.running(),
                "compared": self.compared.iter().map(|(p, t)| json!({ "peer": p, "tick": t })).collect::<Vec<_>>(),
            });
        };
        let player = |peer: u32| {
            self.run
                .as_ref()
                .and_then(|r| r.seat_of(peer, d.tick).or_else(|| r.open_seat(peer)))
        };
        json!({
            "desync": true,
            "tick": d.tick,
            "local": { "peer": self.me, "player": self.me.and_then(player), "hash": hex(d.local_hash) },
            "remote": { "peer": d.peer, "player": player(d.peer), "hash": hex(d.remote_hash) },
            "diff": d.diff.clone().unwrap_or_else(|| json!({ "pending": format!("waiting for peer {}'s snapshot of tick {}", d.peer, d.tick) })),
        })
    }

    /// og.net.status: the room, the seats, who this peer waits for, and
    /// rollback's counts.
    pub fn status(&self) -> Value {
        let run = self.run.as_ref();
        json!({
            "running": self.running(),
            "peer": self.me,
            "peers": self.peers.keys().collect::<Vec<_>>(),
            "startTick": run.map(|r| r.start),
            "players": run.map(|r| r.players),
            "inputDelay": run.map(|r| r.delay),
            "hashEvery": run.map(|r| r.hash_every),
            "rollback": run.map(|r| r.max_prediction > 0),
            "maxPrediction": run.map(|r| r.max_prediction),
            "seats": run.map(|r| r.holds_json()),
            "spectators": run.map(|r| r.running.iter().filter(|p| r.open_seat(**p).is_none()).collect::<Vec<_>>()),
            "waitingFor": self.waiting_for,
            "waits": self.waits,
            "confirmedTick": run.map(|_| self.confirmed),
            "finalTick": run.map(|_| self.hashed),
            "acknowledged": run.and(self.acks.values().copied().min()),
            "predicted": run.map(|_| self.frontier.saturating_sub(1).saturating_sub(self.confirmed)),
            "rollbacks": self.rollbacks,
            "resimulatedTicks": self.resimulated,
            "snapshotBytes": self.ring.values().map(|(_, b)| b.len()).sum::<usize>(),
            "stalled": self.silent,
            "drops": self.drops,
            "compared": self.compared.iter().map(|(p, t)| json!({ "peer": p, "tick": t })).collect::<Vec<_>>(),
            "desync": self.desync.as_ref().map(|d| json!({ "tick": d.tick, "peer": d.peer })),
        })
    }
}
